//! Explicit Visual Studio Code launch through macOS Launch Services.

use std::path::Path;
use std::process::Command;

use crate::platform::editor::EditorOpenError;

const OPEN_PATH: &str = "/usr/bin/open";
const VS_CODE_BUNDLE_ID: &str = "com.microsoft.VSCode";

fn launch_arguments(path: &str) -> [&str; 4] {
    ["-b", VS_CODE_BUNDLE_ID, "--", path]
}

pub(crate) fn open_project(path: &Path) -> Result<(), EditorOpenError> {
    let path = path.to_str().ok_or(EditorOpenError::InvalidPath)?;
    let arguments = launch_arguments(path);
    let output = Command::new(OPEN_PATH)
        .args(arguments)
        .output()
        .map_err(|_| EditorOpenError::Failed)?;

    if output.status.success() {
        Ok(())
    } else if application_unavailable(&output.stderr) {
        Err(EditorOpenError::Unavailable)
    } else {
        Err(EditorOpenError::Failed)
    }
}

fn application_unavailable(stderr: &[u8]) -> bool {
    let diagnostic = String::from_utf8_lossy(stderr);
    diagnostic.contains("Unable to find application")
        || diagnostic.contains("No application knows how to open")
        || diagnostic
            .contains("failed while trying to determine the application with bundle identifier")
}

#[cfg(test)]
mod tests {
    use super::{application_unavailable, launch_arguments, OPEN_PATH, VS_CODE_BUNDLE_ID};

    #[test]
    fn launch_targets_fixed_vscode_bundle_and_keeps_path_as_a_separate_argument() {
        assert_eq!(OPEN_PATH, "/usr/bin/open");
        assert_eq!(VS_CODE_BUNDLE_ID, "com.microsoft.VSCode");
        for path in [
            "/tmp/project ; $(touch sentinel) 'quoted' 项目",
            "/tmp/-leading folder 'quoted' ; &",
        ] {
            assert_eq!(
                launch_arguments(path),
                ["-b", VS_CODE_BUNDLE_ID, "--", path]
            );
        }
    }

    #[test]
    fn application_availability_errors_are_classified_without_logging_paths() {
        assert!(application_unavailable(
            b"Unable to find application named 'com.microsoft.VSCode'"
        ));
        assert!(application_unavailable(
            b"LSCopyApplicationURLsForBundleIdentifier() failed while trying to determine the application with bundle identifier io.example.missing."
        ));
        assert!(!application_unavailable(b"Launch Services failed"));
    }

    #[test]
    fn no_shell_or_path_based_editor_selection_is_used() {
        assert!(!OPEN_PATH.contains("PATH"));
        assert_eq!(OPEN_PATH, "/usr/bin/open");
    }
}
