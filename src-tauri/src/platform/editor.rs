//! Cross-platform project-editor integration boundary.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditorOpenError {
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    Unsupported,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    InvalidPath,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Unavailable,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Failed,
}

/// Opens the validated Project Root in the fixed, supported editor when available.
pub(crate) fn open_project(path: &Path) -> Result<(), EditorOpenError> {
    #[cfg(target_os = "macos")]
    {
        crate::platform::macos::editor::open_project(path)
    }

    #[cfg(target_os = "windows")]
    {
        let _ = path;
        Err(EditorOpenError::Unsupported)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = path;
        Err(EditorOpenError::Unsupported)
    }
}
