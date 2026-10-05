//! Shared macOS process-instance identity query for inspection and actions.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::domain::process::ProcessId;

const SYSCTL_PROCESS_ABSENT: i32 = 1;
const SYSCTL_PERMISSION_DENIED: i32 = 2;

unsafe extern "C" {
    fn thaa_macos_process_start_time(pid: i32, seconds: *mut i64, microseconds: *mut i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ProcessSnapshot {
    pub(super) start_time: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProcessIdentityError {
    InvalidPid,
    ProcessDisappeared,
    PermissionDenied,
    OperatingSystemFailure,
}

pub(super) fn native_pid(process_id: ProcessId) -> Result<i32, ProcessIdentityError> {
    let pid = i32::try_from(process_id.get()).map_err(|_| ProcessIdentityError::InvalidPid)?;
    if pid <= 0 {
        return Err(ProcessIdentityError::InvalidPid);
    }
    Ok(pid)
}

pub(super) fn read_process_snapshot(
    process_id: ProcessId,
) -> Result<ProcessSnapshot, ProcessIdentityError> {
    read_process_snapshot_native(native_pid(process_id)?)
}

pub(super) fn read_process_snapshot_native(
    pid: i32,
) -> Result<ProcessSnapshot, ProcessIdentityError> {
    if pid <= 0 {
        return Err(ProcessIdentityError::InvalidPid);
    }

    let mut seconds = 0_i64;
    let mut microseconds = 0_i32;

    // SAFETY: `pid` is a checked positive i32; both output pointers refer to
    // initialized, correctly aligned locals valid for this synchronous call.
    // The C shim writes only these outputs and validates the SDK-owned table.
    let status = unsafe { thaa_macos_process_start_time(pid, &mut seconds, &mut microseconds) };
    match status {
        0 => {}
        SYSCTL_PROCESS_ABSENT => return Err(ProcessIdentityError::ProcessDisappeared),
        SYSCTL_PERMISSION_DENIED => return Err(ProcessIdentityError::PermissionDenied),
        _ => return Err(ProcessIdentityError::OperatingSystemFailure),
    }

    let start_time = normalize_start_time(seconds, microseconds)
        .ok_or(ProcessIdentityError::OperatingSystemFailure)?;

    Ok(ProcessSnapshot { start_time })
}

fn normalize_start_time(seconds: i64, microseconds: i32) -> Option<SystemTime> {
    if seconds < 0 || !(0..1_000_000).contains(&microseconds) {
        return None;
    }
    let nanos = u32::try_from(microseconds).ok()?.checked_mul(1_000)?;
    UNIX_EPOCH.checked_add(Duration::new(seconds as u64, nanos))
}

#[cfg(test)]
mod tests {
    use super::{native_pid, normalize_start_time, ProcessIdentityError};
    use crate::domain::process::ProcessId;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn native_pid_conversion_is_positive_and_checked() {
        assert_eq!(native_pid(ProcessId::new(1)), Ok(1));
        assert_eq!(native_pid(ProcessId::new(i32::MAX as u32)), Ok(i32::MAX));
        assert_eq!(
            native_pid(ProcessId::new(i32::MAX as u32 + 1)),
            Err(ProcessIdentityError::InvalidPid)
        );
        assert_eq!(
            native_pid(ProcessId::new(0)),
            Err(ProcessIdentityError::InvalidPid)
        );
    }

    #[test]
    fn normalizes_sdk_process_start_time_with_microsecond_precision() {
        assert_eq!(
            normalize_start_time(1_700_000_000, 123_456),
            Some(UNIX_EPOCH + Duration::new(1_700_000_000, 123_456_000))
        );
        assert!(normalize_start_time(-1, 0).is_none());
        assert!(normalize_start_time(1, -1).is_none());
        assert!(normalize_start_time(1, 1_000_000).is_none());
    }
}
