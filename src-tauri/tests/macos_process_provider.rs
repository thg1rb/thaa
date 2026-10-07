#![cfg(target_os = "macos")]

mod common;

use common::process_provider_contract::{
    assert_inspects_requested_process, assert_process_disappeared,
};
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thaa_lib::application::process_inspection::inspect_processes;
use thaa_lib::domain::metadata::{FieldAvailability, UnavailableReason};
use thaa_lib::domain::process::ProcessId;
use thaa_lib::domain::process_provider::ProcessProvider;
use thaa_lib::platform::macos::process_provider::MacOSProcessProvider;

struct FixtureDirectory(PathBuf);

impl FixtureDirectory {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "thaa-process-provider-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create private controlled fixture directory");
        Self(fs::canonicalize(path).expect("canonicalize controlled fixture directory"))
    }
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

struct ControlledChild(Child);

impl Drop for ControlledChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn inspects_controlled_non_gui_child_and_reports_disappearance_after_exit() {
    let directory = FixtureDirectory::new();
    let child = Command::new("/bin/sleep")
        .arg("60")
        .current_dir(&directory.0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn controlled non-GUI child");
    let process_id = ProcessId::new(child.id());
    let mut child = ControlledChild(child);
    let provider = MacOSProcessProvider;

    let info = assert_inspects_requested_process(&provider, process_id);
    assert_eq!(
        info.identity.name,
        FieldAvailability::Available(OsString::from("sleep"))
    );
    assert!(matches!(
        info.identity.start_time,
        FieldAvailability::Available(start_time)
            if start_time > UNIX_EPOCH && start_time <= SystemTime::now() + Duration::from_secs(1)
    ));
    let repeated = provider
        .inspect(process_id)
        .expect("same live process keeps the same SDK start-time identity");
    assert_eq!(repeated.identity.start_time, info.identity.start_time);
    assert_eq!(
        info.identity.executable_path,
        FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
    );
    assert_eq!(
        info.command_arguments,
        FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
    );
    assert!(matches!(
        info.working_directory,
        FieldAvailability::Available(path) if path == directory.0
    ));
    assert!(matches!(
        &info.resource_sample.cumulative_cpu_time,
        FieldAvailability::Available(_)
    ));
    assert!(matches!(
        &info.resource_sample.resident_memory_bytes,
        FieldAvailability::Available(bytes) if *bytes > 0
    ));

    let inspection = inspect_processes(&provider, &[process_id]);
    assert_eq!(inspection.len(), 1);
    assert!(inspection[0].result.is_ok());

    child.0.kill().expect("stop only the controlled child");
    child.0.wait().expect("reap controlled child");
    assert_process_disappeared(&provider, process_id);
}
