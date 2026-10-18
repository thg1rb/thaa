//! Cross-platform project-terminal integration boundary.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminalOpenError {
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    Unsupported,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    InvalidPath,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    AutomationDenied,
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Failed,
}

/// Opens a new terminal at an already validated project root.
pub(crate) fn open_project(path: &Path) -> Result<(), TerminalOpenError> {
    #[cfg(target_os = "macos")]
    {
        crate::platform::macos::terminal::open_project(path)
    }

    #[cfg(target_os = "windows")]
    {
        let _ = path;
        Err(TerminalOpenError::Unsupported)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = path;
        Err(TerminalOpenError::Unsupported)
    }
}
