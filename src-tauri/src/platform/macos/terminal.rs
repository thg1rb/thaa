//! Apple Terminal.app integration for explicit project-directory actions.

use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::platform::terminal::TerminalOpenError;

const OSASCRIPT_PATH: &str = "/usr/bin/osascript";
const AUTOMATION_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_DIAGNOSTIC_BYTES: usize = 4096;
const TERMINAL_SCRIPT: &str = r#"
on run argv
    tell application id "com.apple.Terminal"
        activate
        do script ("cd -- " & quoted form of (item 1 of argv) & " || exit")
    end tell
end run
"#;

pub(crate) fn open_project(path: &Path) -> Result<(), TerminalOpenError> {
    let path = path.to_str().ok_or(TerminalOpenError::InvalidPath)?;
    let mut child = Command::new(OSASCRIPT_PATH)
        .args(["-e", TERMINAL_SCRIPT, "--", path])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| TerminalOpenError::Failed)?;
    let stderr = child.stderr.take().ok_or(TerminalOpenError::Failed)?;
    let diagnostics = thread::spawn(move || read_bounded(stderr));
    let status = wait_for_automation(&mut child, AUTOMATION_TIMEOUT)?;
    let diagnostics = diagnostics.join().unwrap_or_default();

    if status.success() {
        Ok(())
    } else if automation_was_denied(&diagnostics) {
        Err(TerminalOpenError::AutomationDenied)
    } else {
        Err(TerminalOpenError::Failed)
    }
}

fn read_bounded(mut stderr: impl Read) -> Vec<u8> {
    let mut captured = Vec::with_capacity(MAX_DIAGNOSTIC_BYTES);
    let mut buffer = [0_u8; 1024];
    loop {
        match stderr.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                let remaining = MAX_DIAGNOSTIC_BYTES.saturating_sub(captured.len());
                captured.extend_from_slice(&buffer[..read.min(remaining)]);
            }
        }
    }
    captured
}

fn automation_was_denied(stderr: &[u8]) -> bool {
    let stderr = String::from_utf8_lossy(stderr);
    stderr.contains("-1743") || stderr.contains("Not authorized to send Apple events")
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
    fn terminal_automation_uses_terminal_shell_without_running_project_commands() {
        assert!(TERMINAL_SCRIPT.contains("do script"));
        assert!(!TERMINAL_SCRIPT.contains("make new window"));
        assert!(!TERMINAL_SCRIPT.contains("in projectWindow"));
        assert!(TERMINAL_SCRIPT.contains("quoted form of (item 1 of argv)"));
        assert!(TERMINAL_SCRIPT.contains("|| exit"));
        assert!(!TERMINAL_SCRIPT.contains("package.json"));
        assert!(!TERMINAL_SCRIPT.contains("npm "));
    }

    #[test]
    fn only_an_observed_apple_events_denial_is_classified_as_permission_denied() {
        use super::automation_was_denied;

        assert!(automation_was_denied(
            b"execution error: Not authorized to send Apple events. (-1743)"
        ));
        assert!(!automation_was_denied(
            b"execution error: Terminal AppleEvent handler failed. (-10000)"
        ));
        assert!(!automation_was_denied(b"unrecognized Terminal error"));
    }

    #[test]
    fn native_diagnostic_capture_is_bounded() {
        use super::{read_bounded, MAX_DIAGNOSTIC_BYTES};

        let input = vec![b'x'; MAX_DIAGNOSTIC_BYTES + 128];
        let captured = read_bounded(input.as_slice());
        assert_eq!(captured.len(), MAX_DIAGNOSTIC_BYTES);
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
                "/tmp/\"quotes\";$HOME`text`\\tail & (group)",
                "'/tmp/\"quotes\";$HOME`text`\\tail & (group)'",
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
    fn shell_quoted_project_path_is_literal_and_cannot_create_a_sentinel() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("thaa-w022-shell-safety-{nonce}"));
        let sentinel = base.join("injected");
        let project = base.join(format!("project$(touch {})' & (café)", sentinel.display()));
        fs::create_dir_all(&project).expect("create disposable project directory");

        let quoted_path = Command::new(OSASCRIPT_PATH)
            .args([
                "-e",
                "on run argv",
                "-e",
                "return quoted form of (item 1 of argv)",
                "-e",
                "end run",
                "--",
            ])
            .arg(&project)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .expect("quote the path using the same AppleScript primitive as production");
        assert!(quoted_path.status.success());
        let quoted_path = String::from_utf8(quoted_path.stdout)
            .expect("quoted path is UTF-8")
            .trim_end()
            .to_owned();
        let command = format!("cd -- {quoted_path} && pwd -P");
        let result = Command::new("/bin/sh")
            .args(["-c", &command])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .expect("run only the harmless directory and pwd checks");

        assert!(result.status.success());
        assert_eq!(
            String::from_utf8(result.stdout)
                .expect("pwd output is UTF-8")
                .trim_end(),
            fs::canonicalize(&project)
                .expect("canonical disposable project")
                .to_str()
                .expect("test path is UTF-8")
        );
        assert!(!sentinel.exists(), "path content executed as shell code");
        fs::remove_dir_all(base).expect("remove disposable path-safety fixture");
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
