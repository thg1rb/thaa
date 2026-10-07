//! Best-effort process inspection using documented macOS system utilities.

use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use crate::domain::metadata::{FieldAvailability, UnavailableReason};
use crate::domain::process::{ProcessId, ProcessIdentity, ProcessInfo};
use crate::domain::process_provider::{
    ProcessProvider, ProcessProviderError, ProcessProviderErrorKind,
};

use super::lsof::{run_bounded, LsofOutput, RunError, RunLimits};
use super::process_identity::{
    read_process_resources, read_process_snapshot as read_snapshot, ProcessIdentityError,
    ProcessSnapshot,
};

const LSOF_PATH: &str = "/usr/sbin/lsof";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_CAPTURE_BYTES: usize = 256 * 1024;
/// Read-only, best-effort process metadata inspection for macOS.
#[derive(Debug, Clone, Copy, Default)]
pub struct MacOSProcessProvider;

impl ProcessProvider for MacOSProcessProvider {
    fn inspect(&self, process_id: ProcessId) -> Result<ProcessInfo, ProcessProviderError> {
        let before = read_process_snapshot(process_id)?;
        let lsof_output = invoke_lsof(process_id);
        let resource_sample = read_process_resources(process_id);
        let after = read_process_snapshot(process_id)?;

        if before.start_time != after.start_time {
            return Err(provider_error(ProcessProviderErrorKind::ProcessDisappeared));
        }

        let metadata = normalize_lsof_result(process_id, lsof_output)?;

        Ok(ProcessInfo {
            identity: ProcessIdentity {
                pid: process_id,
                name: metadata.name,
                executable_path: FieldAvailability::Unavailable(
                    UnavailableReason::ProviderLimitation,
                ),
                start_time: FieldAvailability::Available(before.start_time),
            },
            parent_process_id: before.parent_pid.map(ProcessId::new),
            command_arguments: FieldAvailability::Unavailable(
                UnavailableReason::ProviderLimitation,
            ),
            working_directory: metadata.working_directory,
            resource_sample,
        })
    }
}

fn read_process_snapshot(process_id: ProcessId) -> Result<ProcessSnapshot, ProcessProviderError> {
    read_snapshot(process_id).map_err(|error| {
        let kind = match error {
            ProcessIdentityError::InvalidPid => ProcessProviderErrorKind::Unsupported,
            ProcessIdentityError::ProcessDisappeared => {
                ProcessProviderErrorKind::ProcessDisappeared
            }
            ProcessIdentityError::PermissionDenied => ProcessProviderErrorKind::PermissionDenied,
            ProcessIdentityError::OperatingSystemFailure => {
                ProcessProviderErrorKind::OperatingSystemFailure
            }
        };
        provider_error(kind)
    })
}

fn invoke_lsof(process_id: ProcessId) -> Result<LsofOutput, RunError> {
    let mut command = Command::new(LSOF_PATH);
    command
        .args([
            "-nP",
            "-F0pcfn",
            "-a",
            "-p",
            &process_id.get().to_string(),
            "-d",
            "cwd",
        ])
        .env_clear()
        .env("LC_ALL", "C")
        .env("TZ", "UTC");

    run_bounded(
        command,
        RunLimits {
            timeout: COMMAND_TIMEOUT,
            max_capture_bytes: MAX_CAPTURE_BYTES,
        },
    )
}

#[derive(Debug, Default, PartialEq, Eq)]
struct LsofMetadata {
    process_id: Option<ProcessId>,
    name: Option<Vec<u8>>,
    working_directory: Option<Vec<u8>>,
}

fn parse_lsof_metadata(bytes: &[u8], expected_pid: ProcessId) -> Result<LsofMetadata, ()> {
    if bytes.is_empty() {
        return Ok(LsofMetadata::default());
    }

    let last_nul = bytes.iter().rposition(|byte| *byte == 0).ok_or(())?;
    let trailer = &bytes[last_nul + 1..];
    if !trailer.is_empty() && trailer != b"\n" {
        return Err(());
    }

    let mut metadata = LsofMetadata::default();
    let mut current_file_is_cwd = false;
    for raw_field in bytes[..last_nul].split(|byte| *byte == 0) {
        let field = raw_field.strip_prefix(b"\n").unwrap_or(raw_field);
        if field.is_empty() {
            continue;
        }
        let (&field_id, value) = field.split_first().ok_or(())?;
        match field_id {
            b'p' => {
                let pid = ProcessId::new(parse_decimal::<u32>(value)?);
                if pid != expected_pid || metadata.process_id.replace(pid).is_some() {
                    return Err(());
                }
                current_file_is_cwd = false;
            }
            b'c' => {
                if metadata.process_id != Some(expected_pid) || metadata.name.is_some() {
                    return Err(());
                }
                metadata.name = Some(value.to_vec());
                current_file_is_cwd = false;
            }
            b'f' => {
                if metadata.process_id != Some(expected_pid) {
                    return Err(());
                }
                current_file_is_cwd = value == b"cwd";
            }
            b'n' if current_file_is_cwd && metadata.working_directory.is_some() => return Err(()),
            b'n' if current_file_is_cwd => metadata.working_directory = Some(value.to_vec()),
            _ => {}
        }
    }
    Ok(metadata)
}

fn parse_decimal<T: std::str::FromStr>(bytes: &[u8]) -> Result<T, ()> {
    std::str::from_utf8(bytes)
        .map_err(|_| ())?
        .parse()
        .map_err(|_| ())
}

fn normalize_lsof_result(
    process_id: ProcessId,
    result: Result<LsofOutput, RunError>,
) -> Result<LsofMetadataAvailability, ProcessProviderError> {
    let output = match result {
        Ok(output) => output,
        Err(RunError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(LsofMetadataAvailability::limited())
        }
        Err(RunError::Io(error)) if error.kind() == io::ErrorKind::PermissionDenied => {
            return Ok(LsofMetadataAvailability::denied())
        }
        Err(RunError::Io(_)) | Err(RunError::TimedOut) => {
            return Ok(LsofMetadataAvailability::limited())
        }
        Err(RunError::OutputLimitExceeded | RunError::ReaderFailed) => {
            return Err(provider_error(ProcessProviderErrorKind::ProviderFailure))
        }
    };

    let parsed = parse_lsof_metadata(&output.stdout, process_id)
        .map_err(|()| provider_error(ProcessProviderErrorKind::ParseFailure))?;
    if parsed.process_id.is_none() {
        if contains_permission_denial(&output.stderr) {
            return Ok(LsofMetadataAvailability::denied());
        }
        if output.exit_code == Some(1) && output.stderr.is_empty() {
            return Ok(LsofMetadataAvailability::inaccessible());
        }
        if output.exit_code == Some(0) && output.stderr.is_empty() {
            return Err(provider_error(ProcessProviderErrorKind::ParseFailure));
        }
        return Ok(LsofMetadataAvailability::limited());
    }

    if output.exit_code != Some(0) && contains_permission_denial(&output.stderr) {
        return Ok(LsofMetadataAvailability::denied());
    }
    if output.exit_code != Some(0) {
        return Ok(LsofMetadataAvailability::limited());
    }

    Ok(LsofMetadataAvailability {
        name: parsed
            .name
            .map(normalize_os_string)
            .unwrap_or_else(|| FieldAvailability::Unavailable(UnavailableReason::Inaccessible)),
        working_directory: parsed
            .working_directory
            .map(normalize_path)
            .unwrap_or_else(|| FieldAvailability::Unavailable(UnavailableReason::Inaccessible)),
    })
}

#[derive(Debug, PartialEq, Eq)]
struct LsofMetadataAvailability {
    name: FieldAvailability<OsString>,
    working_directory: FieldAvailability<PathBuf>,
}

impl LsofMetadataAvailability {
    fn limited() -> Self {
        Self {
            name: FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation),
            working_directory: FieldAvailability::Unavailable(
                UnavailableReason::ProviderLimitation,
            ),
        }
    }

    fn denied() -> Self {
        Self {
            name: FieldAvailability::Unavailable(UnavailableReason::PermissionDenied),
            working_directory: FieldAvailability::Unavailable(UnavailableReason::PermissionDenied),
        }
    }

    fn inaccessible() -> Self {
        Self {
            name: FieldAvailability::Unavailable(UnavailableReason::Inaccessible),
            working_directory: FieldAvailability::Unavailable(UnavailableReason::Inaccessible),
        }
    }
}

fn normalize_os_string(bytes: Vec<u8>) -> FieldAvailability<OsString> {
    if bytes.is_empty() || contains_lsof_escape(&bytes) {
        return FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation);
    }
    FieldAvailability::Available(OsString::from_vec(bytes))
}

fn normalize_path(bytes: Vec<u8>) -> FieldAvailability<PathBuf> {
    if bytes.is_empty() || contains_lsof_escape(&bytes) {
        return FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation);
    }
    FieldAvailability::Available(PathBuf::from(OsString::from_vec(bytes)))
}

fn contains_lsof_escape(bytes: &[u8]) -> bool {
    bytes.windows(2).any(|pair| {
        pair[0] == b'\\' && matches!(pair[1], b'b' | b'f' | b'n' | b'r' | b't' | b'x')
            || pair[0] == b'^' && (pair[1] == b'?' || (b'@'..=b'_').contains(&pair[1]))
    })
}

fn contains_permission_denial(stderr: &[u8]) -> bool {
    [b"permission denied".as_slice(), b"operation not permitted"]
        .iter()
        .any(|marker| {
            stderr
                .windows(marker.len())
                .any(|window| window.eq_ignore_ascii_case(marker))
        })
}

fn provider_error(kind: ProcessProviderErrorKind) -> ProcessProviderError {
    ProcessProviderError::new(kind)
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_lsof_result, normalize_os_string, normalize_path, parse_lsof_metadata,
        LsofMetadataAvailability, LsofOutput,
    };
    use crate::domain::metadata::{FieldAvailability, UnavailableReason};
    use crate::domain::process::ProcessId;
    use std::path::PathBuf;
    #[test]
    fn parses_lsof_process_and_cwd_records_with_nul_fields() {
        let pid = ProcessId::new(42);
        let bytes = b"p42\0csample-process\0\nfcwd\0n/tmp/sample-directory\0\n";
        let parsed = parse_lsof_metadata(bytes, pid).expect("valid selected fields");
        assert_eq!(parsed.process_id, Some(pid));
        assert_eq!(parsed.name.as_deref(), Some(&b"sample-process"[..]));
        assert_eq!(
            parsed.working_directory.as_deref(),
            Some(&b"/tmp/sample-directory"[..])
        );
    }

    #[test]
    fn rejects_lsof_pid_mismatch_and_truncated_fields() {
        assert!(parse_lsof_metadata(b"p43\0", ProcessId::new(42)).is_err());
        assert!(
            parse_lsof_metadata(b"p42\0\nfcwd\0n/tmp/no-terminator", ProcessId::new(42)).is_err()
        );
    }

    #[test]
    fn lsof_escape_like_values_are_unavailable_instead_of_lossily_decoded() {
        assert_eq!(
            normalize_path(b"/tmp/name\\x2fnot-unambiguous".to_vec()),
            FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
        );
        assert_eq!(
            normalize_os_string(b"sample^Aname".to_vec()),
            FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
        );
        assert_eq!(
            normalize_path(b"/tmp/plain path".to_vec()),
            FieldAvailability::Available(PathBuf::from("/tmp/plain path"))
        );
    }

    #[test]
    fn process_with_no_lsof_metadata_keeps_process_and_marks_fields_inaccessible() {
        let result = normalize_lsof_result(
            ProcessId::new(42),
            Ok(LsofOutput {
                exit_code: Some(1),
                stdout: Vec::new(),
                stderr: Vec::new(),
            }),
        )
        .expect("ps independently established the process");
        assert_eq!(
            result,
            LsofMetadataAvailability {
                name: FieldAvailability::Unavailable(UnavailableReason::Inaccessible),
                working_directory: FieldAvailability::Unavailable(UnavailableReason::Inaccessible),
            }
        );
    }

    #[test]
    fn empty_lsof_fields_are_not_fabricated_as_available_metadata() {
        assert_eq!(
            normalize_os_string(Vec::new()),
            FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
        );
        assert_eq!(
            normalize_path(Vec::new()),
            FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
        );
    }

    #[test]
    fn unexpected_lsof_exit_with_partial_output_does_not_claim_complete_fields() {
        let result = normalize_lsof_result(
            ProcessId::new(42),
            Ok(LsofOutput {
                exit_code: Some(2),
                stdout: b"p42\0csample-process\0\nfcwd\0n/tmp/sample-directory\0\n".to_vec(),
                stderr: b"lsof: warning\n".to_vec(),
            }),
        )
        .expect("ps independently established the process");
        assert_eq!(
            result.name,
            FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
        );
        assert_eq!(
            result.working_directory,
            FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
        );
    }

    #[test]
    fn lsof_permission_denial_is_field_level_when_process_is_established() {
        let result = normalize_lsof_result(
            ProcessId::new(42),
            Ok(LsofOutput {
                exit_code: Some(1),
                stdout: Vec::new(),
                stderr: b"lsof: permission denied\n".to_vec(),
            }),
        )
        .expect("per-field denial is successful process inspection");
        assert_eq!(
            result.working_directory,
            FieldAvailability::Unavailable(UnavailableReason::PermissionDenied)
        );
    }
}
