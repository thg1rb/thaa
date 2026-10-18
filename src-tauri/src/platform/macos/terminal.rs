//! Apple Terminal.app integration for explicit project-directory actions.

use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::platform::terminal::TerminalOpenError;

const OSASCRIPT_PATH: &str = "/usr/bin/osascript";
const AUTOMATION_TIMEOUT: Duration = Duration::from_secs(30);
const TERMINAL_SCRIPT: &str = r#"
on run argv
    tell application id "com.apple.Terminal"
        activate
        set projectWindow to make new window
        do script ("cd -- " & quoted form of (item 1 of argv) & " || exit") in projectWindow
    end tell
end run
"#;

pub(crate) fn open_project(path: &Path) -> Result<(), TerminalOpenError> {
    let path = path.to_str().ok_or(TerminalOpenError::InvalidPath)?;
    let mut child = Command::new(OSASCRIPT_PATH)
        .args(["-e", TERMINAL_SCRIPT, "--", path])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| TerminalOpenError::Failed)?;
    let status = wait_for_automation(&mut child, AUTOMATION_TIMEOUT)?;

    if status.success() {
        Ok(())
    } else {
        Err(TerminalOpenError::Failed)
    }
}

fn wait_for_automation(
    child: &mut Child,
    timeout: Duration,
) -> Result<ExitStatus, TerminalOpenError> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            Ok(None) | Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(TerminalOpenError::Failed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OSASCRIPT_PATH, TERMINAL_SCRIPT};
    use std::process::{Command, Stdio};

    #[test]
    fn terminal_automation_uses_a_new_window_and_only_changes_directory() {
        assert!(TERMINAL_SCRIPT.contains("make new window"));
        assert!(TERMINAL_SCRIPT.contains("do script"));
        assert!(TERMINAL_SCRIPT.contains("in projectWindow"));
        assert!(TERMINAL_SCRIPT.contains("quoted form of (item 1 of argv)"));
        assert!(TERMINAL_SCRIPT.contains("|| exit"));
        assert!(!TERMINAL_SCRIPT.contains("package.json"));
        assert!(!TERMINAL_SCRIPT.contains("npm "));
    }

    #[test]
    fn applescript_quotes_special_paths_as_one_shell_argument() {
        let quote_script = [
            "-e",
            "on run argv",
            "-e",
            "return quoted form of (item 1 of argv)",
            "-e",
            "end run",
            "--",
        ];
        let cases = [
            ("/tmp/space and 项目", "'/tmp/space and 项目'"),
            ("/tmp/single'quote", "'/tmp/single'\\''quote'"),
            (
                "/tmp/\"quotes\";$HOME`text`\\tail",
                "'/tmp/\"quotes\";$HOME`text`\\tail'",
            ),
            ("/tmp/line\nbreak", "'/tmp/line\nbreak'"),
            ("/tmp/-leading", "'/tmp/-leading'"),
        ];

        for (path, expected) in cases {
            let output = Command::new(OSASCRIPT_PATH)
                .args(quote_script)
                .arg(path)
                .stdin(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .expect("run AppleScript path-quoting check");
            assert!(output.status.success());
            assert_eq!(
                String::from_utf8(output.stdout).unwrap().trim_end(),
                expected
            );
        }
    }

    #[test]
    fn automation_wait_is_bounded_and_reaps_a_timed_out_child() {
        use super::wait_for_automation;
        use crate::platform::terminal::TerminalOpenError;
        use std::process::Command;
        use std::time::Duration;

        let mut child = Command::new("/bin/sleep")
            .arg("5")
            .spawn()
            .expect("start a harmless sleep fixture");
        assert_eq!(
            wait_for_automation(&mut child, Duration::from_millis(10)),
            Err(TerminalOpenError::Failed)
        );
        assert!(child.try_wait().expect("reaped child state").is_some());
    }
}
