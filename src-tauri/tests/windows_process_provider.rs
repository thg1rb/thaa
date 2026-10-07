#![cfg(target_os = "windows")]

mod common;

use common::process_provider_contract::{
    assert_inspects_requested_process, assert_process_disappeared,
};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thaa_lib::application::process_inspection::inspect_processes;
use thaa_lib::domain::metadata::{FieldAvailability, UnavailableReason};
use thaa_lib::domain::process::ProcessId;
use thaa_lib::domain::process_provider::ProcessProvider;
use thaa_lib::platform::windows::process_provider::WindowsProcessProvider;

const CHILD_READY_ENV: &str = "THAA_WINDOWS_PROCESS_FIXTURE_READY";
const CHILD_READY_TIMEOUT: Duration = Duration::from_secs(10);
const CHILD_EXIT_TIMEOUT: Duration = Duration::from_secs(10);

struct FixtureDirectory(PathBuf);

impl FixtureDirectory {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "thaa-windows-process-{}-{nonce}",
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
    fn stop_and_reap(&mut self) {
        // Closing stdin is the child fixture's explicit shutdown signal.
        self.0.stdin.take();
        let deadline = Instant::now() + CHILD_EXIT_TIMEOUT;
        loop {
            if let Some(status) = self.0.try_wait().expect("check controlled child exit") {
                assert!(status.success(), "controlled child should exit cleanly");
                return;
            }
            if Instant::now() >= deadline {
                let _ = self.0.kill();
                let _ = self.0.wait();
                panic!("controlled child did not exit before the deadline");
            }
            thread::sleep(Duration::from_millis(10));
        }
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

/// This test doubles as the native integration test's controlled child.
/// It blocks only when the parent passes a unique readiness-file path.
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

fn spawn_controlled_child(ready_path: &std::path::Path) -> ControlledChild {
    let executable = std::env::current_exe().expect("resolve current integration test executable");
    let child = Command::new(executable)
        .args(["--exact", "controlled_child_fixture_mode"])
        .current_dir(
            ready_path
                .parent()
                .expect("readiness file has a parent directory"),
        )
        .env(CHILD_READY_ENV, ready_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn controlled integration-test child");
    ControlledChild(child)
}

fn wait_for_ready(child: &mut ControlledChild, ready_path: &std::path::Path) {
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
            "controlled child did not signal readiness before the deadline"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn assert_windows_path_matches(actual: &std::path::Path, expected: &std::path::Path) {
    let actual = fs::canonicalize(actual).expect("canonicalize provider executable path");
    let expected = fs::canonicalize(expected).expect("canonicalize expected child executable");
    assert_eq!(actual, expected);
}

#[test]
fn inspects_controlled_child_and_reports_disappearance_after_exit() {
    let directory = FixtureDirectory::new();
    let ready_path = directory.0.join("child-ready");
    let mut child = spawn_controlled_child(&ready_path);
    wait_for_ready(&mut child, &ready_path);

    let process_id = ProcessId::new(child.0.id());
    let provider = WindowsProcessProvider;
    let expected_executable = std::env::current_exe().expect("resolve test executable");
    let expected_name = expected_executable
        .file_stem()
        .expect("test executable has a file stem")
        .to_os_string();

    let info = assert_inspects_requested_process(&provider, process_id);
    assert_eq!(
        info.identity.name,
        FieldAvailability::Available(expected_name)
    );
    match &info.identity.executable_path {
        FieldAvailability::Available(path) => {
            assert_windows_path_matches(path, &expected_executable)
        }
        other => panic!("controlled child executable path should be available: {other:?}"),
    }
    let start_time = match info.identity.start_time {
        FieldAvailability::Available(value) => value,
        other => panic!("controlled child creation time should be available: {other:?}"),
    };
    assert!(start_time > UNIX_EPOCH);
    assert!(start_time <= SystemTime::now() + Duration::from_secs(1));

    assert_eq!(
        info.command_arguments,
        FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
    );
    assert_eq!(
        info.working_directory,
        FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
    );
    assert!(matches!(
        &info.resource_sample.cumulative_cpu_time,
        FieldAvailability::Available(_)
    ));
    assert!(matches!(
        &info.resource_sample.resident_memory_bytes,
        FieldAvailability::Available(bytes) if *bytes > 0
    ));

    let repeated = provider
        .inspect(process_id)
        .expect("same live process remains inspectable");
    assert_eq!(repeated.identity.start_time, info.identity.start_time);

    let outcomes = inspect_processes(&provider, &[process_id]);
    assert_eq!(outcomes.len(), 1);
    assert!(outcomes[0].result.is_ok());

    child.stop_and_reap();
    assert_process_disappeared(&provider, process_id);
}
