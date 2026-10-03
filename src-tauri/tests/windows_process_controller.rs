#![cfg(target_os = "windows")]

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use thaa_lib::domain::capabilities::{
    CapabilitySupport, PlatformCapabilities, PlatformCapabilitiesProvider,
};
use thaa_lib::domain::metadata::FieldAvailability;
use thaa_lib::domain::process::{ProcessId, ProcessIdentity};
use thaa_lib::domain::process_action::{
    ProcessAction, ProcessActionError, ProcessActionOutcome, ProcessActionTarget,
};
use thaa_lib::domain::process_controller::ProcessController;
use thaa_lib::domain::process_provider::ProcessProvider;
use thaa_lib::platform::windows::process_controller::{
    WindowsPlatformCapabilitiesProvider, WindowsProcessController,
};
use thaa_lib::platform::windows::process_provider::WindowsProcessProvider;

const CHILD_READY_ENV: &str = "THAA_WINDOWS_ACTION_FIXTURE_READY";
const CHILD_READY_TIMEOUT: Duration = Duration::from_secs(10);
const CHILD_EXIT_TIMEOUT: Duration = Duration::from_secs(10);
const FORCE_STOP_EXIT_CODE: i32 = 1;

struct FixtureDirectory(PathBuf);

impl FixtureDirectory {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "thaa-windows-action-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create private controlled fixture directory");
        Self(fs::canonicalize(path).expect("canonicalize controlled fixture directory"))
    }
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct ControlledChild(Child);

impl ControlledChild {
    fn wait_until_exit(&mut self) -> std::process::ExitStatus {
        let deadline = Instant::now() + CHILD_EXIT_TIMEOUT;
        loop {
            if let Some(status) = self.0.try_wait().expect("check controlled child") {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "controlled child did not exit before deadline"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn assert_running(&mut self) {
        assert!(
            self.0.try_wait().expect("check controlled child").is_none(),
            "refused action must leave the controlled child running"
        );
    }

    fn stop_and_reap(&mut self) {
        self.0.stdin.take();
        let status = self.wait_until_exit();
        assert!(
            status.success(),
            "fixture's ordinary shutdown is successful"
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

/// This test doubles as the controlled child. It blocks only when launched
/// with the unique readiness-file environment variable by its parent test.
#[test]
fn controlled_child_fixture_mode() {
    let Ok(ready_path) = std::env::var(CHILD_READY_ENV) else {
        return;
    };

    fs::write(ready_path, b"ready").expect("signal fixture readiness");
    let mut stdin = std::io::stdin().lock();
    let mut shutdown = Vec::new();
    stdin
        .read_to_end(&mut shutdown)
        .expect("wait for parent shutdown signal");
}

fn spawn_controlled_child(ready_path: &Path) -> ControlledChild {
    let executable = std::env::current_exe().expect("resolve controlled child executable");
    let child = Command::new(executable)
        .args(["--exact", "controlled_child_fixture_mode"])
        .current_dir(ready_path.parent().expect("readiness file has parent"))
        .env(CHILD_READY_ENV, ready_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn test-owned controlled child");
    ControlledChild(child)
}

fn wait_for_ready(child: &mut ControlledChild, ready_path: &Path) {
    let deadline = Instant::now() + CHILD_READY_TIMEOUT;
    loop {
        assert!(
            child
                .0
                .try_wait()
                .expect("check controlled child")
                .is_none(),
            "controlled child exited before signaling readiness"
        );
        if ready_path.exists() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "controlled child did not signal readiness before deadline"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn observe_identity(child: &ControlledChild) -> ProcessIdentity {
    WindowsProcessProvider
        .inspect(ProcessId::new(child.0.id()))
        .expect("inspect only the test-owned controlled child")
        .identity
}

fn action_target(identity: &ProcessIdentity) -> ProcessActionTarget {
    ProcessActionTarget::from_identity(identity)
        .expect("controlled process has start-time evidence")
}

fn fixture() -> (FixtureDirectory, PathBuf, ControlledChild) {
    let directory = FixtureDirectory::new();
    let ready_path = directory.0.join("child-ready");
    let mut child = spawn_controlled_child(&ready_path);
    wait_for_ready(&mut child, &ready_path);
    (directory, ready_path, child)
}

#[test]
fn force_stop_terminates_only_the_controlled_child_with_fixed_exit_code() {
    let (_directory, _ready_path, mut child) = fixture();
    let target = action_target(&observe_identity(&child));

    assert_eq!(
        WindowsProcessController.request(&target, ProcessAction::ForceStop),
        Ok(ProcessActionOutcome::Requested)
    );
    let status = child.wait_until_exit();
    assert_eq!(status.code(), Some(FORCE_STOP_EXIT_CODE));
}

#[test]
fn creation_time_mismatch_refuses_force_stop_and_child_remains_alive() {
    let (_directory, _ready_path, mut child) = fixture();
    let mut identity = observe_identity(&child);
    let FieldAvailability::Available(start_time) = &mut identity.start_time else {
        panic!("controlled process start time should be available");
    };
    *start_time = start_time
        .checked_add(Duration::from_secs(1))
        .expect("construct a distinct expected creation time");
    let target = action_target(&identity);

    assert_eq!(
        WindowsProcessController.request(&target, ProcessAction::ForceStop),
        Err(ProcessActionError::IdentityMismatch)
    );
    child.assert_running();
}

#[test]
fn executable_path_mismatch_refuses_force_stop_and_child_remains_alive() {
    let (_directory, _ready_path, mut child) = fixture();
    let mut identity = observe_identity(&child);
    let FieldAvailability::Available(path) = &mut identity.executable_path else {
        panic!("controlled process executable path should be available");
    };
    *path = path.with_file_name("thaa-intentionally-different-image.exe");
    let target = action_target(&identity);

    assert_eq!(
        WindowsProcessController.request(&target, ProcessAction::ForceStop),
        Err(ProcessActionError::IdentityMismatch)
    );
    child.assert_running();
}

#[test]
fn graceful_stop_is_unsupported_and_leaves_child_running() {
    let (_directory, _ready_path, mut child) = fixture();
    let target = action_target(&observe_identity(&child));

    assert_eq!(
        WindowsProcessController.request(&target, ProcessAction::GracefulStop),
        Err(ProcessActionError::Unsupported)
    );
    child.assert_running();
}

#[test]
fn already_exited_child_is_not_reopened_or_retargeted() {
    let (_directory, _ready_path, mut child) = fixture();
    let target = action_target(&observe_identity(&child));
    child.stop_and_reap();

    assert_eq!(
        WindowsProcessController.request(&target, ProcessAction::ForceStop),
        Ok(ProcessActionOutcome::AlreadyExited)
    );
}

#[test]
fn reports_windows_action_capabilities_honestly() {
    assert_eq!(
        WindowsPlatformCapabilitiesProvider.capabilities(),
        PlatformCapabilities {
            command_line: CapabilitySupport::Unsupported,
            working_directory: CapabilitySupport::Unsupported,
            graceful_stop: CapabilitySupport::Unsupported,
            force_stop: CapabilitySupport::Supported,
        }
    );
}
