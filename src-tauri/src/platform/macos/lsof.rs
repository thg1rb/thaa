//! Bounded `lsof` execution and byte-oriented output normalization.

use std::io::{self, Read};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::domain::network::{NetworkListener, NetworkProtocol};
use crate::domain::port_provider::{PortProviderErrorKind, PortScanCompleteness};
use crate::domain::process::ProcessId;

const READ_CHUNK_BYTES: usize = 8192;

pub(super) struct LsofOutput {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub(super) struct RunLimits {
    pub timeout: Duration,
    pub max_capture_bytes: usize,
}

pub(super) enum RunError {
    Io(io::Error),
    TimedOut,
    OutputLimitExceeded,
    ReaderFailed,
}

pub(super) struct ParsedOutput {
    pub(super) listeners: Vec<NetworkListener>,
    pub(super) completeness: PortScanCompleteness,
}

struct FileRecord {
    owner_pid: ProcessId,
    file_type: Option<String>,
    protocol: Option<String>,
    name: Option<String>,
}

/// Runs one command with bounded wall time and combined stdout/stderr capture.
pub(super) fn run_bounded(mut command: Command, limits: RunLimits) -> Result<LsofOutput, RunError> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(RunError::Io)?;

    let stdout = child.stdout.take().ok_or(RunError::ReaderFailed)?;
    let stderr = child.stderr.take().ok_or(RunError::ReaderFailed)?;
    let total_bytes = Arc::new(AtomicUsize::new(0));
    let output_exceeded = Arc::new(AtomicBool::new(false));

    let out_thread = spawn_reader(
        stdout,
        Arc::clone(&total_bytes),
        Arc::clone(&output_exceeded),
        limits.max_capture_bytes,
    );
    let err_thread = spawn_reader(
        stderr,
        Arc::clone(&total_bytes),
        Arc::clone(&output_exceeded),
        limits.max_capture_bytes,
    );

    let deadline = Instant::now() + limits.timeout;
    let (status, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status, false),
            Ok(None) if output_exceeded.load(Ordering::SeqCst) => {
                terminate(&mut child);
                break (wait_after_termination(&mut child)?, false);
            }
            Ok(None) if Instant::now() >= deadline => {
                terminate(&mut child);
                break (wait_after_termination(&mut child)?, true);
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                terminate(&mut child);
                let _ = wait_after_termination(&mut child);
                let _ = join_reader(out_thread);
                let _ = join_reader(err_thread);
                return Err(RunError::Io(error));
            }
        }
    };

    let stdout = join_reader(out_thread)?;
    let stderr = join_reader(err_thread)?;

    if output_exceeded.load(Ordering::SeqCst) {
        return Err(RunError::OutputLimitExceeded);
    }
    if timed_out {
        return Err(RunError::TimedOut);
    }

    Ok(LsofOutput {
        exit_code: status.code(),
        stdout,
        stderr,
    })
}

fn spawn_reader<R: Read + Send + 'static>(
    mut reader: R,
    total_bytes: Arc<AtomicUsize>,
    output_exceeded: Arc<AtomicBool>,
    max_capture_bytes: usize,
) -> thread::JoinHandle<io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut captured = Vec::new();
        let mut chunk = [0; READ_CHUNK_BYTES];
        loop {
            let read = reader.read(&mut chunk)?;
            if read == 0 {
                return Ok(captured);
            }

            let prior = total_bytes.fetch_add(read, Ordering::SeqCst);
            let allowed = max_capture_bytes.saturating_sub(prior);
            if read > allowed {
                captured.extend_from_slice(&chunk[..allowed]);
                output_exceeded.store(true, Ordering::SeqCst);
                return Ok(captured);
            }

            captured.extend_from_slice(&chunk[..read]);
        }
    })
}

fn join_reader(reader: thread::JoinHandle<io::Result<Vec<u8>>>) -> Result<Vec<u8>, RunError> {
    reader
        .join()
        .map_err(|_| RunError::ReaderFailed)?
        .map_err(RunError::Io)
}

fn terminate(child: &mut Child) {
    let _ = child.kill();
}

fn wait_after_termination(child: &mut Child) -> Result<ExitStatus, RunError> {
    child.wait().map_err(RunError::Io)
}

pub(super) fn parse_lsof_output(bytes: &[u8]) -> Result<ParsedOutput, ()> {
    if bytes.is_empty() {
        return Ok(ParsedOutput {
            listeners: Vec::new(),
            completeness: PortScanCompleteness::Complete,
        });
    }

    let last_nul = bytes.iter().rposition(|byte| *byte == 0).ok_or(())?;
    let trailer = &bytes[last_nul + 1..];
    if !trailer.is_empty() && trailer != b"\n" {
        return Err(());
    }

    let mut owner_pid = None;
    let mut record: Option<FileRecord> = None;
    let mut listeners = Vec::new();
    let mut has_scoped_address = false;

    for raw_field in bytes[..last_nul].split(|byte| *byte == 0) {
        let field = raw_field.strip_prefix(b"\n").unwrap_or(raw_field);
        if field.is_empty() {
            continue;
        }

        let (&field_id, value) = field.split_first().ok_or(())?;
        match field_id {
            b'p' => {
                finish_record(&mut record, &mut listeners, &mut has_scoped_address)?;
                owner_pid = Some(ProcessId::new(parse_decimal::<u32>(value)?));
            }
            b'f' => {
                finish_record(&mut record, &mut listeners, &mut has_scoped_address)?;
                let pid = owner_pid.ok_or(())?;
                if value.is_empty() {
                    return Err(());
                }
                record = Some(FileRecord {
                    owner_pid: pid,
                    file_type: None,
                    protocol: None,
                    name: None,
                });
            }
            b't' | b'P' | b'n' => {
                let current = record.as_mut().ok_or(())?;
                let text = std::str::from_utf8(value).map_err(|_| ())?.to_owned();
                let slot = match field_id {
                    b't' => &mut current.file_type,
                    b'P' => &mut current.protocol,
                    b'n' => &mut current.name,
                    _ => unreachable!(),
                };
                if slot.replace(text).is_some() {
                    return Err(());
                }
            }
            _ => {}
        }
    }

    finish_record(&mut record, &mut listeners, &mut has_scoped_address)?;

    Ok(ParsedOutput {
        listeners,
        completeness: if has_scoped_address {
            PortScanCompleteness::Partial(PortProviderErrorKind::Unsupported)
        } else {
            PortScanCompleteness::Complete
        },
    })
}

fn finish_record(
    record: &mut Option<FileRecord>,
    listeners: &mut Vec<NetworkListener>,
    has_scoped_address: &mut bool,
) -> Result<(), ()> {
    let Some(record) = record.take() else {
        return Ok(());
    };

    let file_type = record.file_type.ok_or(())?;
    let protocol = record.protocol.ok_or(())?;
    if protocol != "TCP" {
        return Err(());
    }
    let name = record.name.ok_or(())?;
    let (local_address, local_port) = parse_endpoint(&file_type, &name, has_scoped_address)?;

    listeners.push(NetworkListener {
        protocol: NetworkProtocol::Tcp,
        local_address,
        local_port,
        owner_pid: Some(record.owner_pid),
    });
    Ok(())
}

fn parse_endpoint(
    file_type: &str,
    name: &str,
    has_scoped_address: &mut bool,
) -> Result<(Option<std::net::IpAddr>, std::num::NonZeroU16), ()> {
    let (address_text, port_text) = match file_type {
        "IPv4" => name.split_once(':').ok_or(())?,
        "IPv6" => {
            if name.starts_with("*:") {
                name.split_once(':').ok_or(())?
            } else {
                let bracketed = name.strip_prefix('[').ok_or(())?;
                let (address, port) = bracketed.split_once("]:").ok_or(())?;
                (address, port)
            }
        }
        _ => return Err(()),
    };

    if address_text.is_empty() || port_text.is_empty() || port_text.contains(':') {
        return Err(());
    }
    let port = std::num::NonZeroU16::new(parse_decimal::<u16>(port_text.as_bytes())?).ok_or(())?;

    if address_text == "*" {
        let wildcard = match file_type {
            "IPv4" => std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
            "IPv6" => std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED),
            _ => return Err(()),
        };
        return Ok((Some(wildcard), port));
    }

    if file_type == "IPv6" {
        if let Some((address, scope)) = address_text.split_once('%') {
            if scope.is_empty()
                || !scope
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_.-".contains(&byte))
            {
                return Err(());
            }
            address.parse::<std::net::Ipv6Addr>().map_err(|_| ())?;
            *has_scoped_address = true;
            return Ok((None, port));
        }
    }

    let address = address_text.parse::<std::net::IpAddr>().map_err(|_| ())?;
    match (file_type, address) {
        ("IPv4", std::net::IpAddr::V4(address)) => Ok((Some(std::net::IpAddr::V4(address)), port)),
        ("IPv6", std::net::IpAddr::V6(address)) => Ok((Some(std::net::IpAddr::V6(address)), port)),
        _ => Err(()),
    }
}

fn parse_decimal<T: std::str::FromStr>(bytes: &[u8]) -> Result<T, ()> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return Err(());
    }
    std::str::from_utf8(bytes)
        .map_err(|_| ())?
        .parse()
        .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::parse_lsof_output;
    use crate::domain::network::NetworkListener;
    use crate::domain::port_provider::{PortProviderErrorKind, PortScanCompleteness};
    use crate::domain::process::ProcessId;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    fn parse(fixture: &[u8]) -> Result<Vec<NetworkListener>, ()> {
        parse_lsof_output(fixture).map(|parsed| parsed.listeners)
    }

    #[test]
    fn parses_ipv4_loopback_and_specific_addresses() {
        let fixture =
            b"p101\0\nf3\0tIPv4\0PTCP\0n127.0.0.1:54321\0\nf4\0tIPv4\0PTCP\0n192.0.2.10:54322\0\n";
        let listeners = parse(fixture).expect("verified field format parses");
        assert_eq!(listeners.len(), 2);
        assert_eq!(
            listeners[0].local_address,
            Some(IpAddr::V4(Ipv4Addr::LOCALHOST))
        );
        assert_eq!(listeners[0].owner_pid, Some(ProcessId::new(101)));
        assert_eq!(
            listeners[1].local_address,
            Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10)))
        );
    }

    #[test]
    fn parses_ipv6_loopback_and_specific_addresses() {
        let fixture =
            b"p102\0\nf3\0tIPv6\0PTCP\0n[::1]:54321\0\nf4\0tIPv6\0PTCP\0n[2001:db8::10]:54322\0\n";
        let listeners = parse(fixture).expect("bracketed IPv6 field format parses");
        assert_eq!(listeners.len(), 2);
        assert_eq!(
            listeners[0].local_address,
            Some(IpAddr::V6(Ipv6Addr::LOCALHOST))
        );
        assert_eq!(
            listeners[1].local_address,
            Some("2001:db8::10".parse().unwrap())
        );
    }

    #[test]
    fn address_family_disambiguates_ipv4_and_ipv6_wildcards() {
        let fixture = b"p103\0\nf3\0tIPv4\0PTCP\0n*:5000\0\nf4\0tIPv6\0PTCP\0n*:5000\0\n";
        let listeners = parse(fixture).expect("wildcards use address-family field");
        assert_eq!(
            listeners[0].local_address,
            Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        );
        assert_eq!(
            listeners[1].local_address,
            Some(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
        );
        assert_ne!(listeners[0], listeners[1]);
    }

    #[test]
    fn associates_multiple_files_and_processes_with_their_pid_sets() {
        let fixture = b"p110\0\nf3\0tIPv4\0PTCP\0n127.0.0.1:5001\0\nf4\0tIPv4\0PTCP\0n127.0.0.1:5002\0\np111\0\nf5\0tIPv4\0PTCP\0n127.0.0.1:5001\0\n";
        let listeners = parse(fixture).expect("process and file boundaries are respected");
        assert_eq!(listeners.len(), 3);
        assert_eq!(listeners[0].owner_pid, Some(ProcessId::new(110)));
        assert_eq!(listeners[1].owner_pid, Some(ProcessId::new(110)));
        assert_eq!(listeners[2].owner_pid, Some(ProcessId::new(111)));
    }

    #[test]
    fn ignores_unknown_fields_but_requires_complete_listener_fields() {
        let fixture = b"p112\0\nf3\0tIPv4\0PTCP\0zfuture\0n127.0.0.1:5003\0\n";
        assert_eq!(parse(fixture).expect("unknown fields are ignored").len(), 1);
        assert!(parse(b"p112\0\nf3\0PTCP\0n127.0.0.1:5003\0\n").is_err());
    }

    #[test]
    fn rejects_malformed_pid_port_and_truncated_fields() {
        assert!(parse(b"px\0\nf3\0tIPv4\0PTCP\0n127.0.0.1:5003\0\n").is_err());
        assert!(parse(b"p112\0\nf3\0tIPv4\0PTCP\0n127.0.0.1:0\0\n").is_err());
        assert!(parse(b"p112\0\nf3\0tIPv4\0PTCP\0n127.0.0.1:65536\0\n").is_err());
        assert!(parse(b"p112\0\nf3\0tIPv4\0PTCP\0n127.0.0.1:5").is_err());
    }

    #[test]
    fn empty_output_is_complete_and_scoped_ipv6_is_explicitly_partial() {
        let empty = parse_lsof_output(b"").expect("empty machine output is valid");
        assert!(empty.listeners.is_empty());
        assert_eq!(empty.completeness, PortScanCompleteness::Complete);

        let scoped = parse_lsof_output(b"p113\0\nf3\0tIPv6\0PTCP\0n[fe80::1%en0]:5004\0\n")
            .expect("scoped address preserves the listener");
        assert_eq!(scoped.listeners.len(), 1);
        assert_eq!(scoped.listeners[0].local_address, None);
        assert_eq!(
            scoped.completeness,
            PortScanCompleteness::Partial(PortProviderErrorKind::Unsupported)
        );
    }
}

#[cfg(test)]
mod runner_tests {
    use super::{run_bounded, RunError, RunLimits};
    use std::process::Command;
    use std::time::Duration;

    #[test]
    fn timeout_kills_and_reaps_the_child() {
        let mut command = Command::new("/bin/sleep");
        command.arg("2");
        let result = run_bounded(
            command,
            RunLimits {
                timeout: Duration::from_millis(30),
                max_capture_bytes: 1024,
            },
        );
        assert!(matches!(result, Err(RunError::TimedOut)));
    }

    #[test]
    fn output_limit_kills_a_chatty_child() {
        let command = Command::new("/usr/bin/yes");
        let result = run_bounded(
            command,
            RunLimits {
                timeout: Duration::from_secs(5),
                max_capture_bytes: 1024,
            },
        );
        assert!(matches!(result, Err(RunError::OutputLimitExceeded)));
    }
}
