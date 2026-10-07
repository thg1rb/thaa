//! Shared contract for read-only inspection of one process.

use std::error::Error;
use std::fmt;

use super::process::{ProcessId, ProcessInfo};

/// Stable failure categories for a process-inspection operation.
///
/// Field-specific restrictions belong in `FieldAvailability`; these
/// categories describe failures that prevent a useful `ProcessInfo` result.
/// Platform error codes and native diagnostic text stay inside adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessProviderErrorKind {
    /// The requested PID was absent or the process disappeared during query.
    ProcessDisappeared,
    /// The operating system denied the inspection operation as a whole.
    PermissionDenied,
    /// The platform cannot perform the requested inspection operation.
    Unsupported,
    /// A required executable, library, or query mechanism is unavailable.
    MechanismUnavailable,
    /// Provider data could not be normalized into the shared process model.
    ParseFailure,
    /// An available operating-system mechanism returned an operation failure.
    OperatingSystemFailure,
    /// An otherwise unclassified failure occurred inside the provider.
    ProviderFailure,
}

impl fmt::Display for ProcessProviderErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ProcessDisappeared => "process disappeared",
            Self::PermissionDenied => "process inspection permission denied",
            Self::Unsupported => "process inspection is unsupported",
            Self::MechanismUnavailable => "process inspection mechanism is unavailable",
            Self::ParseFailure => "process inspection data could not be interpreted",
            Self::OperatingSystemFailure => "operating system process query failed",
            Self::ProviderFailure => "process provider failed",
        };

        formatter.write_str(message)
    }
}

/// A query-level failure with a stable, privacy-safe category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessProviderError {
    kind: ProcessProviderErrorKind,
}

impl ProcessProviderError {
    pub const fn new(kind: ProcessProviderErrorKind) -> Self {
        Self { kind }
    }

    pub const fn kind(self) -> ProcessProviderErrorKind {
        self.kind
    }
}

impl fmt::Display for ProcessProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(formatter)
    }
}

impl Error for ProcessProviderError {}

/// Consumer-facing contract for read-only inspection of one process.
///
/// A successful result must carry the requested PID in its identity. A PID
/// identifies a lookup target, not a durable process instance; returned
/// identity evidence is useful data but equality is not authorization for a
/// later destructive action. If the provider establishes that the process is
/// absent or disappeared during inspection, it returns
/// [`ProcessProviderErrorKind::ProcessDisappeared`]. Field-specific
/// restrictions remain successful metadata with explicit field availability
/// when process existence is still established. Platform-wide capabilities
/// are separate from this per-process metadata contract.
pub trait ProcessProvider: Send + Sync {
    /// Inspects the process currently associated with `process_id`.
    fn inspect(&self, process_id: ProcessId) -> Result<ProcessInfo, ProcessProviderError>;

    /// Inspects a batch of distinct process IDs. Providers may override this
    /// to share one native snapshot across the batch; the default preserves
    /// existing per-process behavior.
    fn inspect_many(
        &self,
        process_ids: &[ProcessId],
    ) -> Vec<Result<ProcessInfo, ProcessProviderError>> {
        process_ids.iter().map(|id| self.inspect(*id)).collect()
    }
}
