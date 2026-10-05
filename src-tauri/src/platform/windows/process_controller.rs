//! Identity-checked Windows force termination through one process HANDLE.

use std::time::SystemTime;

use windows_sys::Win32::Foundation::{GetLastError, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER};
use windows_sys::Win32::System::Threading::{
    TerminateProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};

use crate::domain::capabilities::{
    CapabilitySupport, PlatformCapabilities, PlatformCapabilitiesProvider,
};
use crate::domain::metadata::{FieldAvailability, UnavailableReason};
use crate::domain::process::{ProcessId, ProcessIdentity};
use crate::domain::process_action::{
    ProcessAction, ProcessActionError, ProcessActionOutcome, ProcessActionTarget,
};
use crate::domain::process_controller::ProcessController;

use super::process_native::{
    is_running, open_process, query_executable_path, query_start_time, NativeDataError,
    OwnedProcessHandle,
};

const FORCE_STOP_EXIT_CODE: u32 = 1;

/// Windows process action implementation. Each request owns one short-lived HANDLE.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsProcessController;

impl ProcessController for WindowsProcessController {
    fn request(
        &self,
        target: &ProcessActionTarget,
        action: ProcessAction,
    ) -> Result<ProcessActionOutcome, ProcessActionError> {
        if action == ProcessAction::GracefulStop {
            return Err(ProcessActionError::Unsupported);
        }

        let process_id = target.process_id();
        if process_id.get() == 0 {
            return Err(ProcessActionError::InvalidTarget);
        }

        let handle = match open_process(
            process_id.get(),
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE | PROCESS_SYNCHRONIZE,
        ) {
            Ok(handle) => handle,
            Err(error) => return map_open_action_error(error),
        };
        if !is_running(&handle).map_err(map_wait_error)? {
            return Ok(ProcessActionOutcome::AlreadyExited);
        }

        let current_identity = read_identity(&handle, process_id)?;
        validate_target_identity(target, &current_identity)?;

        // Recheck the same process object immediately before termination. The
        // HANDLE anchors the originally opened object even if its PID is later
        // reused; there is no PID reopen between validation and this call.
        if !is_running(&handle).map_err(map_wait_error)? {
            return Ok(ProcessActionOutcome::AlreadyExited);
        }

        // SAFETY: the owned HANDLE was opened with PROCESS_TERMINATE and remains
        // live. The exit code is a fixed provider constant. On failure, capture
        // GetLastError immediately before any further Win32 call.
        let terminated = unsafe { TerminateProcess(handle.as_raw(), FORCE_STOP_EXIT_CODE) };
        if terminated != 0 {
            return Ok(ProcessActionOutcome::Requested);
        }

        let error = unsafe { GetLastError() };
        map_terminate_error(error, &handle)
    }
}

/// Windows-wide action capability reporting. Per-process rights can still deny an action.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsPlatformCapabilitiesProvider;

impl PlatformCapabilitiesProvider for WindowsPlatformCapabilitiesProvider {
    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            command_line: CapabilitySupport::Unsupported,
            working_directory: CapabilitySupport::Unsupported,
            graceful_stop: CapabilitySupport::Unsupported,
            force_stop: CapabilitySupport::Supported,
        }
    }
}

fn map_open_action_error(error: u32) -> Result<ProcessActionOutcome, ProcessActionError> {
    match error {
        ERROR_ACCESS_DENIED => Err(ProcessActionError::PermissionDenied),
        // W011.2 maps ERROR_INVALID_PARAMETER for a nonzero OpenProcess PID to
        // disappearance. PID zero is rejected before reaching OpenProcess.
        ERROR_INVALID_PARAMETER => Ok(ProcessActionOutcome::AlreadyExited),
        _ => Err(ProcessActionError::OperatingSystemFailure),
    }
}

fn read_identity(
    handle: &OwnedProcessHandle,
    process_id: ProcessId,
) -> Result<ProcessIdentity, ProcessActionError> {
    let start_time = required_start_time(query_start_time(handle).map_err(map_native_data_error)?)?;
    let executable_path = query_executable_path(handle).map_err(map_native_data_error)?;

    Ok(ProcessIdentity {
        pid: process_id,
        name: FieldAvailability::Unavailable(UnavailableReason::Unsupported),
        executable_path,
        start_time: FieldAvailability::Available(start_time),
    })
}

fn required_start_time(
    value: FieldAvailability<SystemTime>,
) -> Result<SystemTime, ProcessActionError> {
    match value {
        FieldAvailability::Available(start_time) => Ok(start_time),
        FieldAvailability::Unavailable(UnavailableReason::PermissionDenied) => {
            Err(ProcessActionError::PermissionDenied)
        }
        FieldAvailability::Unavailable(_) => Err(ProcessActionError::IdentityUnavailable),
    }
}

fn validate_target_identity(
    target: &ProcessActionTarget,
    current: &ProcessIdentity,
) -> Result<(), ProcessActionError> {
    match target.validate_current_identity(current) {
        Err(ProcessActionError::IdentityUnavailable)
            if current.executable_path
                == FieldAvailability::Unavailable(UnavailableReason::PermissionDenied) =>
        {
            Err(ProcessActionError::PermissionDenied)
        }
        result => result,
    }
}

#[cfg(test)]
mod target_validation_tests {
    use super::validate_target_identity;
    use crate::domain::metadata::{FieldAvailability, UnavailableReason};
    use crate::domain::process::{ProcessId, ProcessIdentity};
    use crate::domain::process_action::{ProcessActionError, ProcessActionTarget};
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    fn observed() -> ProcessIdentity {
        ProcessIdentity {
            pid: ProcessId::new(42),
            name: FieldAvailability::Available(OsString::from("fixture.exe")),
            executable_path: FieldAvailability::Available(PathBuf::from(r"C:\fixture.exe")),
            start_time: FieldAvailability::Available(UNIX_EPOCH + Duration::from_secs(10)),
        }
    }

    #[test]
    fn observed_path_is_required_and_permission_denial_stays_explicit() {
        let target = ProcessActionTarget::from_identity(&observed()).expect("valid target");
        let mut current = observed();
        current.executable_path =
            FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation);
        assert_eq!(
            validate_target_identity(&target, &current),
            Err(ProcessActionError::IdentityUnavailable)
        );

        current.executable_path =
            FieldAvailability::Unavailable(UnavailableReason::PermissionDenied);
        assert_eq!(
            validate_target_identity(&target, &current),
            Err(ProcessActionError::PermissionDenied)
        );
    }
}

fn map_native_data_error(error: NativeDataError) -> ProcessActionError {
    match error {
        NativeDataError::AllocationFailure => ProcessActionError::ProviderFailure,
        NativeDataError::MalformedData => ProcessActionError::OperatingSystemFailure,
    }
}

fn map_wait_error(error: u32) -> ProcessActionError {
    if error == ERROR_ACCESS_DENIED {
        ProcessActionError::PermissionDenied
    } else {
        ProcessActionError::OperatingSystemFailure
    }
}

fn map_terminate_error(
    error: u32,
    handle: &OwnedProcessHandle,
) -> Result<ProcessActionOutcome, ProcessActionError> {
    if error == ERROR_ACCESS_DENIED {
        // TerminateProcess can report access denied when the process already
        // terminated. Check the same HANDLE to distinguish that race from a
        // live target for which termination is denied.
        return map_terminate_access_denied(is_running(handle));
    }

    Err(ProcessActionError::OperatingSystemFailure)
}

fn map_terminate_access_denied(
    state: Result<bool, u32>,
) -> Result<ProcessActionOutcome, ProcessActionError> {
    match state {
        Ok(false) => Ok(ProcessActionOutcome::AlreadyExited),
        Ok(true) => Err(ProcessActionError::PermissionDenied),
        Err(error) => Err(map_wait_error(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        map_native_data_error, map_open_action_error, map_terminate_access_denied, map_wait_error,
        required_start_time,
    };
    use crate::domain::metadata::{FieldAvailability, UnavailableReason};
    use crate::domain::process_action::{ProcessActionError, ProcessActionOutcome};
    use crate::platform::windows::process_native::NativeDataError;
    use windows_sys::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER};

    #[test]
    fn unavailable_required_start_time_fails_closed() {
        assert_eq!(
            required_start_time(FieldAvailability::Available(std::time::UNIX_EPOCH)),
            Ok(std::time::UNIX_EPOCH)
        );
        assert_eq!(
            required_start_time(FieldAvailability::Unavailable(
                UnavailableReason::PermissionDenied
            )),
            Err(ProcessActionError::PermissionDenied)
        );
        assert_eq!(
            required_start_time(FieldAvailability::Unavailable(
                UnavailableReason::Inaccessible
            )),
            Err(ProcessActionError::IdentityUnavailable)
        );
    }

    #[test]
    fn open_process_errors_are_mapped_with_action_context() {
        assert_eq!(
            map_open_action_error(ERROR_INVALID_PARAMETER),
            Ok(ProcessActionOutcome::AlreadyExited)
        );
        assert_eq!(
            map_open_action_error(ERROR_ACCESS_DENIED),
            Err(ProcessActionError::PermissionDenied)
        );
        assert_eq!(
            map_open_action_error(1234),
            Err(ProcessActionError::OperatingSystemFailure)
        );
    }

    #[test]
    fn terminate_access_denied_checks_the_same_handle_state() {
        assert_eq!(
            map_terminate_access_denied(Ok(false)),
            Ok(ProcessActionOutcome::AlreadyExited)
        );
        assert_eq!(
            map_terminate_access_denied(Ok(true)),
            Err(ProcessActionError::PermissionDenied)
        );
        assert_eq!(
            map_terminate_access_denied(Err(ERROR_ACCESS_DENIED)),
            Err(ProcessActionError::PermissionDenied)
        );
    }

    #[test]
    fn native_data_errors_remain_stable_and_privacy_safe() {
        assert_eq!(
            map_native_data_error(NativeDataError::AllocationFailure),
            ProcessActionError::ProviderFailure
        );
        assert_eq!(
            map_native_data_error(NativeDataError::MalformedData),
            ProcessActionError::OperatingSystemFailure
        );
    }

    #[test]
    fn wait_error_mapping_does_not_leak_native_text() {
        assert_eq!(
            map_wait_error(ERROR_ACCESS_DENIED),
            ProcessActionError::PermissionDenied
        );
        assert_eq!(
            map_wait_error(ERROR_INVALID_PARAMETER),
            ProcessActionError::OperatingSystemFailure
        );
    }
}
