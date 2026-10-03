//! Best-effort process inspection using documented macOS system utilities.

use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::domain::metadata::{FieldAvailability, UnavailableReason};
use crate::domain::process::{ProcessId, ProcessIdentity, ProcessInfo};
use crate::domain::process_provider::{
    ProcessProvider, ProcessProviderError, ProcessProviderErrorKind,
};

use super::lsof::{run_bounded, LsofOutput, RunError, RunLimits};

const PS_PATH: &str = "/bin/ps";
const LSOF_PATH: &str = "/usr/sbin/lsof";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_CAPTURE_BYTES: usize = 256 * 1024;

/// Read-only, best-effort process metadata inspection for macOS.
#[derive(Debug, Clone, Copy, Default)]
pub struct MacOSProcessProvider;

impl ProcessProvider for MacOSProcessProvider {
    fn inspect(&self, process_id: ProcessId) -> Result<ProcessInfo, ProcessProviderError> {
        let before = read_process_snapshot(process_id)?;
        if before.is_zombie {
            return Err(provider_error(ProcessProviderErrorKind::ProcessDisappeared));
        }
        let lsof_output = invoke_lsof(process_id);
        let after = read_process_snapshot(process_id)?;

        if after.is_zombie || before != after {
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
            command_arguments: FieldAvailability::Unavailable(
                UnavailableReason::ProviderLimitation,
            ),
            working_directory: metadata.working_directory,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProcessSnapshot {
    process_id: ProcessId,
    start_time: SystemTime,
    is_zombie: bool,
}

fn read_process_snapshot(process_id: ProcessId) -> Result<ProcessSnapshot, ProcessProviderError> {
    let output = invoke_ps(process_id).map_err(map_run_error)?;
    if output.exit_code == Some(1) && output.stdout.is_empty() && output.stderr.is_empty() {
        return Err(provider_error(ProcessProviderErrorKind::ProcessDisappeared));
    }
    if output.exit_code != Some(0) {
        return Err(map_ps_failure(&output));
    }

    parse_ps_snapshot(&output.stdout, process_id)
        .map_err(|()| provider_error(ProcessProviderErrorKind::ParseFailure))
}

fn invoke_ps(process_id: ProcessId) -> Result<LsofOutput, RunError> {
    let mut command = Command::new(PS_PATH);
    command
        .args([
            "-p",
            &process_id.get().to_string(),
            "-o",
            "pid=",
            "-o",
            "state=",
            "-o",
            "lstart=",
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

fn parse_ps_snapshot(bytes: &[u8], expected_pid: ProcessId) -> Result<ProcessSnapshot, ()> {
    let text = std::str::from_utf8(bytes).map_err(|_| ())?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let line = lines.next().ok_or(())?.trim();
    if lines.next().is_some() {
        return Err(());
    }

    let columns: Vec<_> = line.split_ascii_whitespace().collect();
    if columns.len() != 7 {
        return Err(());
    }
    let observed_pid = columns[0].parse::<u32>().map_err(|_| ())?;
    if ProcessId::new(observed_pid) != expected_pid {
        return Err(());
    }
    let state = columns[1];
    if state.is_empty() {
        return Err(());
    }

    let start_time = parse_lstart(&columns[2..]).ok_or(())?;
    Ok(ProcessSnapshot {
        process_id: expected_pid,
        start_time,
        is_zombie: state.starts_with('Z'),
    })
}

fn parse_lstart(columns: &[&str]) -> Option<SystemTime> {
    if columns.len() != 5 {
        return None;
    }

    let month = match columns[1] {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    };
    let year = columns[4].parse::<i64>().ok()?;
    let day = columns[2].parse::<u32>().ok()?;
    let (hour, minute, second) = parse_clock(columns[3])?;
    if !(1..=31).contains(&day) || hour > 23 || minute > 59 || second > 60 {
        return None;
    }

    let days = days_from_civil(year, month, day)?;
    let seconds = days
        .checked_mul(86_400)?
        .checked_add(i64::from(hour * 3_600 + minute * 60 + second))?;
    if seconds >= 0 {
        UNIX_EPOCH.checked_add(Duration::from_secs(seconds as u64))
    } else {
        UNIX_EPOCH.checked_sub(Duration::from_secs(seconds.unsigned_abs()))
    }
}

fn parse_clock(value: &str) -> Option<(u32, u32, u32)> {
    let mut parts = value.split(':');
    let hour = parts.next()?.parse().ok()?;
    let minute = parts.next()?.parse().ok()?;
    let second = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((hour, minute, second))
}

/// Converts a Gregorian civil date to days from 1970-01-01.
fn days_from_civil(year: i64, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=9_999).contains(&year) {
        return None;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if day == 0 || day > month_days {
        return None;
    }

    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
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

fn map_ps_failure(output: &LsofOutput) -> ProcessProviderError {
    if contains_permission_denial(&output.stderr) {
        provider_error(ProcessProviderErrorKind::PermissionDenied)
    } else {
        provider_error(ProcessProviderErrorKind::OperatingSystemFailure)
    }
}

fn map_run_error(error: RunError) -> ProcessProviderError {
    let kind = match error {
        RunError::Io(error) if error.kind() == io::ErrorKind::NotFound => {
            ProcessProviderErrorKind::MechanismUnavailable
        }
        RunError::Io(error) if error.kind() == io::ErrorKind::PermissionDenied => {
            ProcessProviderErrorKind::PermissionDenied
        }
        RunError::Io(_) | RunError::TimedOut => ProcessProviderErrorKind::OperatingSystemFailure,
        RunError::OutputLimitExceeded | RunError::ReaderFailed => {
            ProcessProviderErrorKind::ProviderFailure
        }
    };
    provider_error(kind)
}

fn provider_error(kind: ProcessProviderErrorKind) -> ProcessProviderError {
    ProcessProviderError::new(kind)
}

#[cfg(test)]
mod tests {
    use super::{
        days_from_civil, normalize_lsof_result, normalize_os_string, normalize_path,
        parse_lsof_metadata, parse_lstart, parse_ps_snapshot, LsofMetadataAvailability, LsofOutput,
    };
    use crate::domain::metadata::{FieldAvailability, UnavailableReason};
    use crate::domain::process::ProcessId;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_documented_utc_ps_start_time_and_gregorian_leap_day() {
        assert_eq!(
            parse_lstart(&["Thu", "Feb", "29", "12:34:56", "2024"]),
            Some(UNIX_EPOCH + std::time::Duration::from_secs(1_709_210_096))
        );
        assert!(parse_lstart(&["Thu", "Feb", "29", "12:34:56", "2023"]).is_none());
        assert!(parse_lstart(&["Thu", "Foo", "29", "12:34:56", "2024"]).is_none());
    }

    #[test]
    fn parses_ps_identity_and_rejects_mismatched_pid_or_bad_layout() {
        let pid = ProcessId::new(42);
        let bytes = b" 42 S Thu Feb 29 12:34:56 2024\n";
        let parsed = parse_ps_snapshot(bytes, pid).expect("valid ps row");
        assert_eq!(parsed.process_id, pid);
        assert_eq!(
            parsed.start_time,
            parse_lstart(&["Thu", "Feb", "29", "12:34:56", "2024"]).unwrap()
        );
        assert!(!parsed.is_zombie);
        assert!(parse_ps_snapshot(bytes, ProcessId::new(43)).is_err());
        assert!(parse_ps_snapshot(b"42 S malformed\n", pid).is_err());
    }

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

    #[test]
    fn civil_date_conversion_handles_epoch_and_rejects_invalid_dates() {
        assert_eq!(days_from_civil(1970, 1, 1), Some(0));
        assert_eq!(days_from_civil(1969, 12, 31), Some(-1));
        assert_eq!(days_from_civil(2024, 2, 30), None);
        let before_epoch = parse_lstart(&["Wed", "Dec", "31", "23:59:59", "1969"]);
        assert_eq!(
            before_epoch,
            UNIX_EPOCH.checked_sub(std::time::Duration::from_secs(1))
        );
        assert_eq!(
            UNIX_EPOCH.checked_add(std::time::Duration::from_secs(0)),
            Some(SystemTime::UNIX_EPOCH)
        );
    }
}
