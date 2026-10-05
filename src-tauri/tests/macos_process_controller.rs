#![cfg(target_os = "macos")]

use std::os::unix::process::ExitStatusExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use thaa_lib::domain::metadata::FieldAvailability;
use thaa_lib::domain::process_action::{
    ProcessAction, ProcessActionError, ProcessActionOutcome, ProcessActionTarget,
};
use thaa_lib::domain::process_controller::ProcessController;
use thaa_lib::domain::process_provider::ProcessProvider;
use thaa_lib::platform::macos::process_controller::{
    MacOSPlatformCapabilitiesProvider, MacOSProcessController,
};
use thaa_lib::platform::macos::process_provider::MacOSProcessProvider;

const CHILD_RUNTIME: Duration = Duration::from_secs(60);
const CHILD_EXIT_TIMEOUT: Duration = Duration::from_secs(5);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

// Darwin signal numbers from the SDK's sys/signal.h.
const SIGKILL_NUMBER: i32 = 9;
const SIGTERM_NUMBER: i32 = 15;

struct ControlledChild(Child);

impl ControlledChild {
    fn spawn() -> Self {
        let child = Command::new("/bin/sleep")
            .arg(CHILD_RUNTIME.as_secs().to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn test-owned /bin/sleep child");
        Self(child)
    }

    fn wait_for_exit(&mut self) -> ExitStatus {
        let deadline = Instant::now() + CHILD_EXIT_TIMEOUT;
        loop {
            if let Some(status) = self.0.try_wait().expect("check controlled child") {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "controlled child did not exit before deadline"
            );
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn assert_still_running(&mut self) {
        assert!(
            self.0.try_wait().expect("check controlled child").is_none(),
            "identity mismatch must not signal the controlled child"
        );
    }
}

impl Drop for ControlledChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

fn action_target(child: &ControlledChild) -> ProcessActionTarget {
    let process_id = thaa_lib::domain::process::ProcessId::new(child.0.id());
    let info = MacOSProcessProvider
        .inspect(process_id)
        .expect("inspect only the test-owned child");
    ProcessActionTarget::from_identity(&info.identity).expect("provider supplied action identity")
}

#[test]
fn graceful_stop_requests_sigterm_for_the_controlled_child() {
    let mut child = ControlledChild::spawn();
    let target = action_target(&child);

    assert_eq!(
        MacOSProcessController.request(&target, ProcessAction::GracefulStop),
        Ok(ProcessActionOutcome::Requested)
    );
    assert_eq!(
        child.wait_for_exit().signal(),
        Some(SIGTERM_NUMBER),
        "graceful action must deliver SIGTERM"
    );
}

#[test]
fn force_stop_requests_sigkill_for_a_separate_controlled_child() {
    let mut child = ControlledChild::spawn();
    let target = action_target(&child);

    assert_eq!(
        MacOSProcessController.request(&target, ProcessAction::ForceStop),
        Ok(ProcessActionOutcome::Requested)
    );
    assert_eq!(
        child.wait_for_exit().signal(),
        Some(SIGKILL_NUMBER),
        "force action must deliver SIGKILL"
    );
}

#[test]
fn changed_start_time_refuses_action_and_leaves_child_alive() {
    let mut child = ControlledChild::spawn();
    let process_id = thaa_lib::domain::process::ProcessId::new(child.0.id());
    let mut identity = MacOSProcessProvider
        .inspect(process_id)
        .expect("inspect only the test-owned child")
        .identity;
    if let FieldAvailability::Available(start_time) = &mut identity.start_time {
        *start_time = start_time
            .checked_add(Duration::from_secs(1))
            .expect("make a deterministic different timestamp");
    } else {
        panic!("controlled child start time must be available");
    }
    let target = ProcessActionTarget::from_identity(&identity).expect("construct stale target");

    assert_eq!(
        MacOSProcessController.request(&target, ProcessAction::ForceStop),
        Err(ProcessActionError::IdentityMismatch)
    );
    child.assert_still_running();
}

#[test]
fn already_exited_child_is_not_signaled_or_retried() {
    let mut child = ControlledChild::spawn();
    let target = action_target(&child);
    child.0.kill().expect("stop only the test-owned child");
    let _ = child.wait_for_exit();

    assert_eq!(
        MacOSProcessController.request(&target, ProcessAction::GracefulStop),
        Ok(ProcessActionOutcome::AlreadyExited)
    );
}

#[test]
fn reports_platform_capabilities_without_claiming_per_process_permission() {
    use thaa_lib::domain::capabilities::{
        CapabilitySupport, PlatformCapabilities, PlatformCapabilitiesProvider,
    };

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
