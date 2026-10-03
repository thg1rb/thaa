//! macOS listening TCP discovery using the system `lsof` utility.

use std::io;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::domain::port_provider::{
    PortProvider, PortProviderError, PortProviderErrorKind, PortScanCompleteness, PortScanResult,
};

use super::lsof::{parse_lsof_output, run_bounded, LsofOutput, RunError, RunLimits};

const LSOF_PATH: &str = "/usr/sbin/lsof";
const LSOF_ARGUMENTS: &[&str] = &["-nP", "-F0pftPn", "-a", "-iTCP", "-sTCP:LISTEN"];
const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_CAPTURE_BYTES: usize = 16 * 1024 * 1024;

/// Read-only macOS implementation of the shared listener provider contract.
#[derive(Debug, Clone, Copy, Default)]
pub struct MacOSPortProvider;

impl PortProvider for MacOSPortProvider {
    fn listeners(&self) -> Result<PortScanResult, PortProviderError> {
        let output = invoke_lsof().map_err(map_run_error)?;
        map_output(output)
    }
}

fn invoke_lsof() -> Result<LsofOutput, RunError> {
    let mut command = Command::new(LSOF_PATH);
    command
        .args(LSOF_ARGUMENTS)
        .env_clear()
        .env("LC_ALL", "C")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    run_bounded(
        command,
        RunLimits {
            timeout: COMMAND_TIMEOUT,
            max_capture_bytes: MAX_CAPTURE_BYTES,
        },
    )
}

fn map_run_error(error: RunError) -> PortProviderError {
    let kind = match error {
        RunError::Io(error) if error.kind() == io::ErrorKind::NotFound => {
            PortProviderErrorKind::MechanismUnavailable
        }
        RunError::Io(error) if error.kind() == io::ErrorKind::PermissionDenied => {
            PortProviderErrorKind::PermissionDenied
        }
        RunError::Io(_) | RunError::TimedOut => PortProviderErrorKind::OperatingSystemFailure,
        RunError::OutputLimitExceeded | RunError::ReaderFailed => {
            PortProviderErrorKind::ProviderFailure
        }
    };

    PortProviderError::new(kind)
}

fn map_output(output: LsofOutput) -> Result<PortScanResult, PortProviderError> {
    let parsed = parse_lsof_output(&output.stdout)
        .map_err(|()| PortProviderError::new(PortProviderErrorKind::ParseFailure))?;

    let permission_diagnostic = contains_permission_denial(&output.stderr);
    let unsupported_diagnostic = contains_unsupported_option(&output.stderr);
    let has_diagnostic = !output.stderr.is_empty();
    let is_success = output.exit_code == Some(0);
    let is_observed_empty =
        output.exit_code == Some(1) && output.stdout.is_empty() && output.stderr.is_empty();

    if is_observed_empty {
        return Ok(PortScanResult {
            listeners: Vec::new(),
            completeness: PortScanCompleteness::Complete,
        });
    }

    if is_success && !has_diagnostic {
        return Ok(PortScanResult {
            listeners: parsed.listeners,
            completeness: parsed.completeness,
        });
    }

    let issue_kind = if permission_diagnostic {
        PortProviderErrorKind::PermissionDenied
    } else if unsupported_diagnostic {
        PortProviderErrorKind::Unsupported
    } else {
        PortProviderErrorKind::OperatingSystemFailure
    };

    if !parsed.listeners.is_empty() {
        return Ok(PortScanResult {
            listeners: parsed.listeners,
            completeness: PortScanCompleteness::Partial(issue_kind),
        });
    }

    if has_diagnostic || !is_success {
        return Err(PortProviderError::new(issue_kind));
    }

    Ok(PortScanResult {
        listeners: parsed.listeners,
        completeness: parsed.completeness,
    })
}

fn contains_permission_denial(stderr: &[u8]) -> bool {
    const MARKER: &[u8] = b"permission denied";
    stderr
        .windows(MARKER.len())
        .any(|window| window.eq_ignore_ascii_case(MARKER))
}

fn contains_unsupported_option(stderr: &[u8]) -> bool {
    [
        b"illegal option".as_slice(),
        b"unknown option",
        b"unrecognized option",
    ]
    .iter()
    .any(|marker| {
        stderr
            .windows(marker.len())
            .any(|window| window.eq_ignore_ascii_case(marker))
    })
}

#[cfg(test)]
mod tests {
    use super::{
        contains_permission_denial, contains_unsupported_option, map_output, map_run_error,
    };
    use crate::domain::network::BindingScope;
    use crate::domain::network::NetworkProtocol;
    use crate::domain::port_provider::{PortProviderErrorKind, PortScanCompleteness};
    use crate::domain::process::ProcessId;
    use std::process::ExitStatus;

    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;

    fn status(code: i32) -> ExitStatus {
        #[cfg(unix)]
        {
            ExitStatus::from_raw(code << 8)
        }
        #[cfg(not(unix))]
        {
            let _ = code;
            unreachable!("this macOS-specific module is only compiled on Unix")
        }
    }

    fn output(code: i32, stdout: &[u8], stderr: &[u8]) -> super::LsofOutput {
        super::LsofOutput {
            exit_code: status(code).code(),
            stdout: stdout.to_vec(),
            stderr: stderr.to_vec(),
        }
    }

    #[test]
    fn observed_nonzero_empty_search_is_a_complete_empty_result() {
        let result = map_output(output(1, b"", b"")).expect("empty result is successful");
        assert!(result.listeners.is_empty());
        assert_eq!(result.completeness, PortScanCompleteness::Complete);
    }

    #[test]
    fn nonzero_result_with_valid_rows_is_partial() {
        let bytes = b"p123\0\nf3\0tIPv4\0PTCP\0n127.0.0.1:54321\0\n";
        let result = map_output(output(1, bytes, b"lsof: partial diagnostic\n"))
            .expect("valid listener rows remain usable");
        assert_eq!(result.listeners.len(), 1);
        assert_eq!(
            result.completeness,
            PortScanCompleteness::Partial(PortProviderErrorKind::OperatingSystemFailure)
        );
    }

    #[test]
    fn permission_diagnostic_is_bounded_to_a_stable_category() {
        assert!(contains_permission_denial(
            b"lsof: warning: Permission denied for test-owned metadata\n"
        ));
        assert!(!contains_permission_denial(b"unrelated diagnostic"));

        let bytes = b"p123\0\nf3\0tIPv6\0PTCP\0n[::1]:54321\0\n";
        let result = map_output(output(0, bytes, b"lsof: Permission denied\n"))
            .expect("partial data remains available");
        assert_eq!(
            result.completeness,
            PortScanCompleteness::Partial(PortProviderErrorKind::PermissionDenied)
        );
    }

    #[test]
    fn unsupported_option_diagnostic_is_stable_and_narrow() {
        assert!(contains_unsupported_option(b"lsof: unrecognized option -Q"));
        assert!(!contains_unsupported_option(
            b"unrelated operating system error"
        ));

        let error = map_output(output(1, b"", b"lsof: illegal option -Q\n"))
            .expect_err("unsupported invocation cannot mean empty success");
        assert_eq!(error.kind(), PortProviderErrorKind::Unsupported);
    }

    #[test]
    fn malformed_output_is_not_hidden_by_an_empty_or_partial_result() {
        let error = map_output(output(
            0,
            b"p123\0\nf3\0tIPv4\0PTCP\0n127.0.0.1:bad\0\n",
            b"",
        ))
        .expect_err("malformed required port fails the query");
        assert_eq!(error.kind(), PortProviderErrorKind::ParseFailure);
    }

    #[test]
    fn process_launch_errors_map_to_stable_categories() {
        let missing = map_run_error(super::RunError::Io(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        )));
        assert_eq!(missing.kind(), PortProviderErrorKind::MechanismUnavailable);

        let denied = map_run_error(super::RunError::Io(std::io::Error::from(
            std::io::ErrorKind::PermissionDenied,
        )));
        assert_eq!(denied.kind(), PortProviderErrorKind::PermissionDenied);
    }

    #[test]
    fn scoped_ipv6_remains_a_listener_with_unknown_address_and_partial_status() {
        let bytes = b"p123\0\nf3\0tIPv6\0PTCP\0n[fe80::1%en0]:54321\0\n";
        let result = map_output(output(0, bytes, b"")).expect("scoped listener is retained");
        assert_eq!(result.listeners.len(), 1);
        assert_eq!(result.listeners[0].local_address, None);
        assert_eq!(result.listeners[0].owner_pid, Some(ProcessId::new(123)));
        assert_eq!(
            result.completeness,
            PortScanCompleteness::Partial(PortProviderErrorKind::Unsupported)
        );
        assert_eq!(result.listeners[0].protocol, NetworkProtocol::Tcp);
        assert_eq!(result.listeners[0].binding_scope(), BindingScope::Unknown);
    }
}
