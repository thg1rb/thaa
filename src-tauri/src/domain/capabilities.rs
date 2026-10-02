//! Platform-wide support for P0 operations.

/// Whether the current platform generally supports an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilitySupport {
    Supported,
    Unsupported,
}

/// Platform-wide capabilities relevant to P0 process inspection and actions.
///
/// These values do not grant permission for a particular process. Per-process
/// denial or unavailable fields are represented by query/action outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformCapabilities {
    pub command_line: CapabilitySupport,
    pub working_directory: CapabilitySupport,
    pub graceful_stop: CapabilitySupport,
    pub force_stop: CapabilitySupport,
}
