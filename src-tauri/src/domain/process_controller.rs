//! Platform-neutral process action contract.

use super::process_action::{
    ProcessAction, ProcessActionError, ProcessActionOutcome, ProcessActionTarget,
};

/// Requests one explicit action against an identity-bound process target.
///
/// Implementations own fresh identity revalidation and must perform it
/// immediately before the platform action. PID-only actions are not exposed.
pub trait ProcessController: Send + Sync {
    fn request(
        &self,
        target: &ProcessActionTarget,
        action: ProcessAction,
    ) -> Result<ProcessActionOutcome, ProcessActionError>;
}

#[cfg(test)]
mod tests {
    use super::ProcessController;
    use crate::domain::capabilities::{
        CapabilitySupport, PlatformCapabilities, PlatformCapabilitiesProvider,
    };
    use crate::domain::metadata::FieldAvailability;
    use crate::domain::process::{ProcessId, ProcessIdentity};
    use crate::domain::process_action::{
        ProcessAction, ProcessActionError, ProcessActionOutcome, ProcessActionTarget,
    };
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    struct FixedController(Result<ProcessActionOutcome, ProcessActionError>);

    impl ProcessController for FixedController {
        fn request(
            &self,
            _target: &ProcessActionTarget,
            _action: ProcessAction,
        ) -> Result<ProcessActionOutcome, ProcessActionError> {
            self.0
        }
    }

    struct TestCapabilities(PlatformCapabilities);

    impl PlatformCapabilitiesProvider for TestCapabilities {
        fn capabilities(&self) -> PlatformCapabilities {
            self.0
        }
    }

    fn target() -> ProcessActionTarget {
        let identity = ProcessIdentity {
            pid: ProcessId::new(42),
            name: FieldAvailability::Available(OsString::from("mutable-name")),
            executable_path: FieldAvailability::Available(PathBuf::from("/sample/runtime")),
            start_time: FieldAvailability::Available(UNIX_EPOCH + Duration::from_secs(42)),
        };
        ProcessActionTarget::from_identity(&identity).expect("identity includes required evidence")
    }

    #[test]
    fn action_contract_keeps_graceful_and_force_requests_distinct() {
        let controller = FixedController(Ok(ProcessActionOutcome::Requested));
        let target = target();
        let controller: &dyn ProcessController = &controller;
        assert_eq!(
            controller.request(&target, ProcessAction::GracefulStop),
            Ok(ProcessActionOutcome::Requested)
        );
        assert_eq!(
            controller.request(&target, ProcessAction::ForceStop),
            Ok(ProcessActionOutcome::Requested)
        );
    }

    #[test]
    fn outcome_and_error_semantics_are_explicit_and_platform_neutral() {
        assert_ne!(
            ProcessActionOutcome::Requested,
            ProcessActionOutcome::AlreadyExited
        );
        assert_eq!(
            ProcessActionError::IdentityMismatch.to_string(),
            "process identity changed since observation"
        );
        assert_eq!(
            ProcessActionError::PermissionDenied.to_string(),
            "process action permission denied"
        );

        let target = target();
        let already_exited = FixedController(Ok(ProcessActionOutcome::AlreadyExited));
        assert_eq!(
            already_exited.request(&target, ProcessAction::GracefulStop),
            Ok(ProcessActionOutcome::AlreadyExited)
        );
        for error in [
            ProcessActionError::IdentityUnavailable,
            ProcessActionError::IdentityMismatch,
            ProcessActionError::PermissionDenied,
            ProcessActionError::Unsupported,
            ProcessActionError::OperatingSystemFailure,
            ProcessActionError::ProviderFailure,
        ] {
            assert_eq!(
                FixedController(Err(error)).request(&target, ProcessAction::ForceStop),
                Err(error)
            );
        }
    }

    #[test]
    fn capability_contract_represents_platform_asymmetry() {
        let capabilities = TestCapabilities(PlatformCapabilities {
            command_line: CapabilitySupport::Unsupported,
            working_directory: CapabilitySupport::Supported,
            graceful_stop: CapabilitySupport::Unsupported,
            force_stop: CapabilitySupport::Supported,
        });
        assert_eq!(
            capabilities.capabilities().graceful_stop,
            CapabilitySupport::Unsupported
        );
        assert_eq!(
            capabilities.capabilities().force_stop,
            CapabilitySupport::Supported
        );
    }
}
