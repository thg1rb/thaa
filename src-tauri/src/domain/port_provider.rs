//! Shared contract for discovering normalized listening TCP endpoints.

use std::error::Error;
use std::fmt;

use super::network::NetworkListener;

/// Stable failure categories for a port-discovery operation or partial scan.
///
/// Platform-specific error codes and raw command output stay in the provider
/// implementation and are not part of this shared contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortProviderErrorKind {
    /// The current query was denied by the operating system.
    PermissionDenied,
    /// The platform cannot provide the requested listener query.
    Unsupported,
    /// A required executable, library, or query mechanism is unavailable.
    MechanismUnavailable,
    /// Provider input could not be normalized into the shared domain model.
    ParseFailure,
    /// An available operating-system mechanism returned an operation failure.
    OperatingSystemFailure,
    /// An otherwise unclassified failure occurred inside the provider.
    ProviderFailure,
}

impl fmt::Display for PortProviderErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::PermissionDenied => "port discovery permission denied",
            Self::Unsupported => "port discovery is unsupported",
            Self::MechanismUnavailable => "port discovery mechanism is unavailable",
            Self::ParseFailure => "port discovery data could not be interpreted",
            Self::OperatingSystemFailure => "operating system port query failed",
            Self::ProviderFailure => "port discovery provider failed",
        };

        formatter.write_str(message)
    }
}

/// A query-level failure with a stable, privacy-safe category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortProviderError {
    kind: PortProviderErrorKind,
}

impl PortProviderError {
    pub const fn new(kind: PortProviderErrorKind) -> Self {
        Self { kind }
    }

    pub const fn kind(self) -> PortProviderErrorKind {
        self.kind
    }
}

impl fmt::Display for PortProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(formatter)
    }
}

impl Error for PortProviderError {}

/// Whether the provider completed its system query without known omissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortScanCompleteness {
    /// The query completed. An empty listener vector means no listeners found.
    Complete,
    /// Some listener data may be missing; the category explains why.
    Partial(PortProviderErrorKind),
}

/// Normalized listener data and the completeness of the provider query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortScanResult {
    pub listeners: Vec<NetworkListener>,
    pub completeness: PortScanCompleteness,
}

/// Consumer-facing contract for listening TCP endpoint discovery.
///
/// Implementations normalize platform data into [`NetworkListener`] values.
/// They do not return raw command/API output, perform process enrichment, or
/// impose UI ordering. Results must preserve semantically distinct endpoints,
/// including listeners that share a port but differ by address or ownership.
/// Exact duplicate rows may be coalesced only when all normalized endpoint and
/// ownership information is identical.
///
/// The synchronous method keeps the contract independent of a particular
/// async runtime. The application refresh coordinator is responsible for
/// invoking blocking implementations away from the UI thread and must wait
/// for an in-flight scan to settle before starting another one. Cancellation
/// is therefore coordinated above this contract; implementations should bound
/// their own operations where their mechanism permits it.
pub trait PortProvider: Send + Sync {
    /// Queries the system for listening TCP endpoints.
    ///
    /// `Ok` with [`PortScanCompleteness::Complete`] and no listeners means the
    /// query succeeded and found none. Known omissions must be reported as a
    /// partial result. `Err` means the query could not produce a usable scan.
    fn listeners(&self) -> Result<PortScanResult, PortProviderError>;
}
