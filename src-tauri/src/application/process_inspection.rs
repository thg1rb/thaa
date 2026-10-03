//! Orchestration for inspecting a caller-supplied set of process IDs.

use std::collections::HashSet;

use crate::domain::process::{ProcessId, ProcessInfo};
use crate::domain::process_provider::{
    ProcessProvider, ProcessProviderError, ProcessProviderErrorKind,
};

/// The result of inspecting one requested process ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInspectionOutcome {
    /// The ID requested by the caller, retained even when inspection fails.
    pub requested_process_id: ProcessId,
    /// Normalized metadata or the query-level error returned for this process.
    pub result: Result<ProcessInfo, ProcessProviderError>,
}

/// Inspects each distinct process ID once, preserving its first-seen order.
///
/// Each provider error remains attached to its requested ID and does not
/// prevent inspection of later IDs. The provider contract does not classify
/// errors as per-process or provider-wide, so this use case does not infer a
/// fail-fast policy from error categories. Empty input returns an empty list.
///
/// A successful provider response with a different identity PID violates the
/// W010 contract and is reported as `ProviderFailure` for the requested ID.
pub fn inspect_processes(
    provider: &dyn ProcessProvider,
    process_ids: &[ProcessId],
) -> Vec<ProcessInspectionOutcome> {
    let mut seen = HashSet::with_capacity(process_ids.len());
    let mut outcomes = Vec::with_capacity(process_ids.len());

    for &requested_process_id in process_ids {
        if !seen.insert(requested_process_id.get()) {
            continue;
        }

        let result = provider.inspect(requested_process_id).and_then(|info| {
            if info.identity.pid == requested_process_id {
                Ok(info)
            } else {
                Err(ProcessProviderError::new(
                    ProcessProviderErrorKind::ProviderFailure,
                ))
            }
        });

        outcomes.push(ProcessInspectionOutcome {
            requested_process_id,
            result,
        });
    }

    outcomes
}
