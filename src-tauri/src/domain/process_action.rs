//! Shared, identity-bound process action values and validation policy.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::time::SystemTime;

use super::metadata::FieldAvailability;
use super::process::{ProcessId, ProcessIdentity};

/// An explicit process action. Graceful and force requests remain distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessAction {
    GracefulStop,
    ForceStop,
}

/// Evidence captured when the user observes a process and requests an action.
///
/// A target cannot be created from a PID alone. Its fields are private so
/// callers must pass through [`ProcessActionTarget::from_identity`].
#[derive(Clone, PartialEq, Eq)]
pub struct ProcessActionTarget {
    pid: ProcessId,
    start_time: SystemTime,
    executable_path: Option<PathBuf>,
}

impl fmt::Debug for ProcessActionTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProcessActionTarget")
            .field("pid", &self.pid)
            .field("start_time", &self.start_time)
            .field("executable_path_observed", &self.executable_path.is_some())
            .finish()
    }
}

impl ProcessActionTarget {
    /// Creates an action target only when the observation contains usable
    /// start-time evidence and a positive PID.
    pub fn from_identity(identity: &ProcessIdentity) -> Result<Self, ProcessActionError> {
        if identity.pid.get() == 0 {
            return Err(ProcessActionError::InvalidTarget);
        }

        let FieldAvailability::Available(start_time) = identity.start_time else {
            return Err(ProcessActionError::IdentityUnavailable);
        };

        let executable_path = match &identity.executable_path {
            FieldAvailability::Available(path) => Some(path.clone()),
            FieldAvailability::Unavailable(_) => None,
        };

        Ok(Self {
            pid: identity.pid,
            start_time,
            executable_path,
        })
    }

    pub const fn process_id(&self) -> ProcessId {
        self.pid
    }

    /// Compares fresh platform evidence against the action target.
    ///
    /// A path observed originally is mandatory on revalidation and must match.
    /// Name, arguments, and working directory are intentionally excluded as
    /// mutable or presentation-oriented metadata.
    pub fn validate_current_identity(
        &self,
        current: &ProcessIdentity,
    ) -> Result<(), ProcessActionError> {
        if current.pid != self.pid {
            return Err(ProcessActionError::IdentityMismatch);
        }

        match current.start_time {
            FieldAvailability::Available(start_time) if start_time == self.start_time => {}
            FieldAvailability::Available(_) => return Err(ProcessActionError::IdentityMismatch),
            FieldAvailability::Unavailable(_) => {
                return Err(ProcessActionError::IdentityUnavailable)
            }
        }

        if let Some(expected_path) = &self.executable_path {
            match &current.executable_path {
                FieldAvailability::Available(path) if path == expected_path => {}
                FieldAvailability::Available(_) => {
                    return Err(ProcessActionError::IdentityMismatch)
                }
                FieldAvailability::Unavailable(_) => {
                    return Err(ProcessActionError::IdentityUnavailable)
                }
            }
        }

        Ok(())
    }
}

/// What a controller can report after a safe action request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessActionOutcome {
    /// The operating system accepted the request; this does not confirm exit.
    Requested,
    /// The observed process instance had already exited before action.
    AlreadyExited,
}

/// Stable failure categories for process action requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessActionError {
    InvalidTarget,
    IdentityUnavailable,
    IdentityMismatch,
    PermissionDenied,
    Unsupported,
    OperatingSystemFailure,
    ProviderFailure,
}

impl fmt::Display for ProcessActionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidTarget => "process action target is invalid",
            Self::IdentityUnavailable => "process identity could not be revalidated",
            Self::IdentityMismatch => "process identity changed since observation",
            Self::PermissionDenied => "process action permission denied",
            Self::Unsupported => "process action is unsupported",
            Self::OperatingSystemFailure => "operating system process action failed",
            Self::ProviderFailure => "process controller failed",
        };

        formatter.write_str(message)
    }
}

impl Error for ProcessActionError {}

#[cfg(test)]
mod tests {
    use super::{ProcessActionError, ProcessActionTarget};
    use crate::domain::metadata::{FieldAvailability, UnavailableReason};
    use crate::domain::process::{ProcessId, ProcessIdentity};
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn identity(pid: u32) -> ProcessIdentity {
        ProcessIdentity {
            pid: ProcessId::new(pid),
            name: FieldAvailability::Available(OsString::from("mutable-name")),
            executable_path: FieldAvailability::Available(PathBuf::from("/sample/runtime")),
            start_time: FieldAvailability::Available(UNIX_EPOCH + Duration::from_secs(42)),
        }
    }

    #[test]
    fn action_target_requires_nonzero_pid_and_start_time() {
        assert_eq!(
            ProcessActionTarget::from_identity(&identity(0)),
            Err(ProcessActionError::InvalidTarget)
        );

        let mut missing_start = identity(42);
        missing_start.start_time =
            FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation);
        assert_eq!(
            ProcessActionTarget::from_identity(&missing_start),
            Err(ProcessActionError::IdentityUnavailable)
        );
    }

    #[test]
    fn matching_start_time_and_observed_executable_validate() {
        let observed = identity(42);
        let target = ProcessActionTarget::from_identity(&observed).expect("valid target");
        assert_eq!(target.validate_current_identity(&observed), Ok(()));
    }

    #[test]
    fn changed_pid_time_or_executable_fails_closed() {
        let observed = identity(42);
        let target = ProcessActionTarget::from_identity(&observed).expect("valid target");

        let mut changed_pid = observed.clone();
        changed_pid.pid = ProcessId::new(43);
        assert_eq!(
            target.validate_current_identity(&changed_pid),
            Err(ProcessActionError::IdentityMismatch)
        );

        let mut changed_start = observed.clone();
        changed_start.start_time =
            FieldAvailability::Available(SystemTime::UNIX_EPOCH + Duration::from_secs(43));
        assert_eq!(
            target.validate_current_identity(&changed_start),
            Err(ProcessActionError::IdentityMismatch)
        );

        let mut changed_path = observed.clone();
        changed_path.executable_path =
            FieldAvailability::Available(PathBuf::from("/sample/other-runtime"));
        assert_eq!(
            target.validate_current_identity(&changed_path),
            Err(ProcessActionError::IdentityMismatch)
        );
    }

    #[test]
    fn unavailable_required_revalidation_evidence_fails_closed() {
        let observed = identity(42);
        let target = ProcessActionTarget::from_identity(&observed).expect("valid target");

        let mut missing_start = observed.clone();
        missing_start.start_time = FieldAvailability::Unavailable(UnavailableReason::Inaccessible);
        assert_eq!(
            target.validate_current_identity(&missing_start),
            Err(ProcessActionError::IdentityUnavailable)
        );

        let mut missing_path = observed;
        missing_path.executable_path =
            FieldAvailability::Unavailable(UnavailableReason::PermissionDenied);
        assert_eq!(
            target.validate_current_identity(&missing_path),
            Err(ProcessActionError::IdentityUnavailable)
        );
    }

    #[test]
    fn mutable_name_does_not_authorize_or_reject_identity() {
        let observed = identity(42);
        let target = ProcessActionTarget::from_identity(&observed).expect("valid target");
        let mut current = observed;
        current.name = FieldAvailability::Available(OsString::from("changed-display-name"));

        assert_eq!(target.validate_current_identity(&current), Ok(()));
    }

    #[test]
    fn absent_observed_executable_does_not_become_required_later() {
        let mut observed = identity(42);
        observed.executable_path =
            FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation);
        let target = ProcessActionTarget::from_identity(&observed).expect("start time is present");
        let mut current = observed;
        current.executable_path = FieldAvailability::Available(PathBuf::from("/new/path"));

        assert_eq!(target.validate_current_identity(&current), Ok(()));
    }
}
