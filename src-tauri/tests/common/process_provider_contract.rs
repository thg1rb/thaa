//! Behavioral assertions reusable by native ProcessProvider integration tests.

use thaa_lib::domain::process::{ProcessId, ProcessInfo};
use thaa_lib::domain::process_provider::{ProcessProvider, ProcessProviderErrorKind};

/// Requires successful inspection of a controlled process and verifies the
/// normalized identity retains the requested PID.
#[allow(dead_code)] // Shared with future native provider integration targets.
pub(crate) fn assert_inspects_requested_process(
    provider: &dyn ProcessProvider,
    process_id: ProcessId,
) -> ProcessInfo {
    let info = provider
        .inspect(process_id)
        .expect("inspection of the controlled process should succeed");
    assert_eq!(info.identity.pid, process_id);
    info
}

/// Requires a process that has exited or is no longer present to be reported
/// as an operation-level disappearance, not fabricated metadata.
#[allow(dead_code)] // Used by future native provider integration targets.
pub(crate) fn assert_process_disappeared(provider: &dyn ProcessProvider, process_id: ProcessId) {
    let error = provider
        .inspect(process_id)
        .expect_err("missing process must not produce a default ProcessInfo");
    assert_eq!(error.kind(), ProcessProviderErrorKind::ProcessDisappeared);
}
