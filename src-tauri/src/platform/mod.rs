//! Operating-system adapters, selected only at the platform boundary.

#[cfg(target_os = "macos")]
pub mod macos;
