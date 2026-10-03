//! Identity-checked macOS process actions using single-PID POSIX signals.

use crate::domain::capabilities::{
    CapabilitySupport, PlatformCapabilities, PlatformCapabilitiesProvider,
};
use crate::domain::metadata::{FieldAvailability, UnavailableReason};
use crate::domain::process::ProcessIdentity;
use crate::domain::process_action::{
    ProcessAction, ProcessActionError, ProcessActionOutcome, ProcessActionTarget,
};
use crate::domain::process_controller::ProcessController;

use super::process_identity::{native_pid, read_process_snapshot_native, ProcessIdentityError};

const SIGNAL_ACCEPTED: i32 = 0;
const SIGNAL_PROCESS_ABSENT: i32 = 1;
const SIGNAL_PERMISSION_DENIED: i32 = 2;
const SIGNAL_INVALID_ARGUMENT: i32 = 3;
const SIGNAL_OS_FAILURE: i32 = 4;
const SIGNAL_INVALID_REQUEST: i32 = 5;

const ACTION_GRACEFUL_STOP: i32 = 1;
const ACTION_FORCE_STOP: i32 = 2;

unsafe extern "C" {
    fn thaa_macos_signal_process(pid: i32, action: i32) -> i32;
}

/// macOS process action implementation. It does not retain process state.
#[derive(Debug, Clone, Copy, Default)]
pub struct MacOSProcessController;

impl ProcessController for MacOSProcessController {
    fn request(
        &self,
        target: &ProcessActionTarget,
        action: ProcessAction,
    ) -> Result<ProcessActionOutcome, ProcessActionError> {
        let pid = native_pid(target.process_id()).map_err(|error| match error {
            ProcessIdentityError::InvalidPid => ProcessActionError::InvalidTarget,
            _ => ProcessActionError::OperatingSystemFailure,
        })?;
        let native_action = match action {
            ProcessAction::GracefulStop => ACTION_GRACEFUL_STOP,
            ProcessAction::ForceStop => ACTION_FORCE_STOP,
        };

        let snapshot = match read_process_snapshot_native(pid) {
            Ok(snapshot) => snapshot,
            Err(ProcessIdentityError::InvalidPid) => return Err(ProcessActionError::InvalidTarget),
            Err(ProcessIdentityError::ProcessDisappeared) => {
                return Ok(ProcessActionOutcome::AlreadyExited)
            }
            Err(ProcessIdentityError::PermissionDenied) => {
                return Err(ProcessActionError::PermissionDenied)
            }
            Err(ProcessIdentityError::OperatingSystemFailure) => {
                return Err(ProcessActionError::OperatingSystemFailure)
            }
        };

        let current_identity = ProcessIdentity {
            pid: target.process_id(),
            name: FieldAvailability::Unavailable(UnavailableReason::Unsupported),
            executable_path: FieldAvailability::Unavailable(UnavailableReason::Unsupported),
            start_time: FieldAvailability::Available(snapshot.start_time),
        };
        target.validate_current_identity(&current_identity)?;

        // SAFETY: `pid` was checked positive and fits pid_t; `native_action`
        // is selected only from the two fixed actions above. The C shim rejects
        // all other values, calls kill once, and captures errno immediately.
        let result = unsafe { thaa_macos_signal_process(pid, native_action) };
        map_signal_result(result)
    }
}

/// macOS-wide support. Per-process access can still be denied or disappear.
#[derive(Debug, Clone, Copy, Default)]
pub struct MacOSPlatformCapabilitiesProvider;

impl PlatformCapabilitiesProvider for MacOSPlatformCapabilitiesProvider {
    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            command_line: CapabilitySupport::Unsupported,
            working_directory: CapabilitySupport::Supported,
            graceful_stop: CapabilitySupport::Supported,
            force_stop: CapabilitySupport::Supported,
        }
    }
}

fn map_signal_result(result: i32) -> Result<ProcessActionOutcome, ProcessActionError> {
    match result {
        SIGNAL_ACCEPTED => Ok(ProcessActionOutcome::Requested),
        SIGNAL_PROCESS_ABSENT => Ok(ProcessActionOutcome::AlreadyExited),
        SIGNAL_PERMISSION_DENIED => Err(ProcessActionError::PermissionDenied),
        SIGNAL_INVALID_ARGUMENT | SIGNAL_OS_FAILURE => {
            Err(ProcessActionError::OperatingSystemFailure)
        }
        SIGNAL_INVALID_REQUEST => Err(ProcessActionError::InvalidTarget),
        _ => Err(ProcessActionError::OperatingSystemFailure),
    }
}

#[cfg(test)]
mod tests {
    use super::{map_signal_result, MacOSPlatformCapabilitiesProvider};
    use crate::domain::capabilities::{
        CapabilitySupport, PlatformCapabilities, PlatformCapabilitiesProvider,
    };
    use crate::domain::process_action::{ProcessActionError, ProcessActionOutcome};

    #[test]
    fn maps_signal_shim_results_to_stable_action_semantics() {
        assert_eq!(map_signal_result(0), Ok(ProcessActionOutcome::Requested));
        assert_eq!(
            map_signal_result(1),
            Ok(ProcessActionOutcome::AlreadyExited)
        );
        assert_eq!(
            map_signal_result(2),
            Err(ProcessActionError::PermissionDenied)
        );
        assert_eq!(
            map_signal_result(3),
            Err(ProcessActionError::OperatingSystemFailure)
        );
        assert_eq!(
            map_signal_result(4),
            Err(ProcessActionError::OperatingSystemFailure)
        );
        assert_eq!(map_signal_result(5), Err(ProcessActionError::InvalidTarget));
        assert_eq!(
            map_signal_result(99),
            Err(ProcessActionError::OperatingSystemFailure)
        );
    }

    #[test]
    fn reports_macos_metadata_and_action_capabilities_from_implemented_behavior() {
        assert_eq!(
            MacOSPlatformCapabilitiesProvider.capabilities(),
            PlatformCapabilities {
                command_line: CapabilitySupport::Unsupported,
                working_directory: CapabilitySupport::Supported,
                graceful_stop: CapabilitySupport::Supported,
                force_stop: CapabilitySupport::Supported,
            }
        );
    }
}
