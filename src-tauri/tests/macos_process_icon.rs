#![cfg(target_os = "macos")]

use std::process::{Child, Command, Stdio};

use thaa_lib::application::process_icons::ProcessIconProvider;
use thaa_lib::domain::process_provider::ProcessProvider;
use thaa_lib::platform::macos::process_icon::MacOSProcessIconProvider;
use thaa_lib::platform::macos::process_provider::MacOSProcessProvider;

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn command_line_child_icon_is_optional_and_bounded() {
    let child = Command::new("/bin/sleep")
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn controlled command-line child");
    let child = ChildGuard(child);
    let pid = child.0.id();
    let process = MacOSProcessProvider
        .inspect(thaa_lib::domain::process::ProcessId::new(pid))
        .expect("inspect controlled child");

    let icon = MacOSProcessIconProvider.icon_png(&process);
    assert!(
        icon.is_none(),
        "CLI fixture should use the Thaa fallback icon"
    );
}
