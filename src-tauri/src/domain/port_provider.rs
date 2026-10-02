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

/// Shared assertions for native provider tests. These helpers are compiled
/// only for tests and never become part of the production API.
#[cfg(test)]
pub(crate) mod contract_test_support {
    use super::{PortProvider, PortScanCompleteness};
    use crate::domain::network::NetworkListener;
    use std::net::IpAddr;
    use std::num::NonZeroU16;

    /// Requires a successful query and returns the matching listener plus scan
    /// completeness, so callers cannot silently discard partial-result status.
    ///
    /// Native provider tests can use this after creating a controlled listener;
    /// ownership remains intentionally optional and is asserted separately only
    /// when the platform contract can require it.
    pub(crate) fn assert_listener_discovered(
        provider: &dyn PortProvider,
        address: IpAddr,
        port: NonZeroU16,
    ) -> (NetworkListener, PortScanCompleteness) {
        let result = provider
            .listeners()
            .expect("provider query should succeed for the controlled listener");

        let completeness = result.completeness;
        let listener = result
            .listeners
            .into_iter()
            .find(|listener| {
                listener.protocol == crate::domain::network::NetworkProtocol::Tcp
                    && listener.local_address == Some(address)
                    && listener.local_port == port
            })
            .expect("controlled listener should be present with its normalized protocol, address, and port");

        (listener, completeness)
    }

    /// Requires a successful complete query and asserts a controlled endpoint
    /// is absent, for example after its test-owned socket has been closed.
    pub(crate) fn assert_listener_absent(
        provider: &dyn PortProvider,
        address: IpAddr,
        port: NonZeroU16,
    ) {
        let result = provider
            .listeners()
            .expect("provider query should succeed after the controlled listener closes");
        assert_eq!(result.completeness, PortScanCompleteness::Complete);
        assert!(!result.listeners.iter().any(|listener| listener.protocol
            == crate::domain::network::NetworkProtocol::Tcp
            && listener.local_address == Some(address)
            && listener.local_port == port));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        contract_test_support, PortProvider, PortProviderError, PortProviderErrorKind,
        PortScanCompleteness, PortScanResult,
    };
    use crate::domain::network::{NetworkListener, NetworkProtocol};
    use crate::domain::process::ProcessId;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::num::NonZeroU16;

    struct StubPortProvider {
        result: Result<PortScanResult, PortProviderError>,
    }

    impl PortProvider for StubPortProvider {
        fn listeners(&self) -> Result<PortScanResult, PortProviderError> {
            self.result.clone()
        }
    }

    fn listener(address: IpAddr, owner_pid: Option<ProcessId>) -> NetworkListener {
        NetworkListener {
            protocol: NetworkProtocol::Tcp,
            local_address: Some(address),
            local_port: NonZeroU16::new(8080).expect("nonzero fixture port"),
            owner_pid,
        }
    }

    #[test]
    fn successful_result_preserves_address_family_and_unresolved_ownership() {
        let ipv4 = listener(IpAddr::V4(Ipv4Addr::LOCALHOST), Some(ProcessId::new(41)));
        let ipv6 = listener(IpAddr::V6(Ipv6Addr::LOCALHOST), None);
        let provider = StubPortProvider {
            result: Ok(PortScanResult {
                listeners: vec![ipv4.clone(), ipv6.clone()],
                completeness: PortScanCompleteness::Complete,
            }),
        };

        let provider_object: &dyn PortProvider = &provider;
        let (found_ipv4, ipv4_completeness) = contract_test_support::assert_listener_discovered(
            provider_object,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            ipv4.local_port,
        );
        let (found_ipv6, ipv6_completeness) = contract_test_support::assert_listener_discovered(
            provider_object,
            IpAddr::V6(Ipv6Addr::LOCALHOST),
            ipv6.local_port,
        );

        assert_eq!(found_ipv4, ipv4);
        assert_eq!(found_ipv6, ipv6);
        assert_eq!(found_ipv4.local_port, found_ipv6.local_port);
        assert_ne!(found_ipv4.local_address, found_ipv6.local_address);
        assert_eq!(ipv4_completeness, PortScanCompleteness::Complete);
        assert_eq!(ipv6_completeness, PortScanCompleteness::Complete);
    }

    #[test]
    fn empty_success_is_distinct_from_query_failure() {
        let empty_provider = StubPortProvider {
            result: Ok(PortScanResult {
                listeners: Vec::new(),
                completeness: PortScanCompleteness::Complete,
            }),
        };
        let failing_provider = StubPortProvider {
            result: Err(PortProviderError::new(
                PortProviderErrorKind::MechanismUnavailable,
            )),
        };

        let empty_result = empty_provider
            .listeners()
            .expect("an empty successful scan is not an error");
        let failure = failing_provider
            .listeners()
            .expect_err("a failed query must not look like an empty scan");

        assert!(empty_result.listeners.is_empty());
        assert_eq!(empty_result.completeness, PortScanCompleteness::Complete);
        assert_eq!(failure.kind(), PortProviderErrorKind::MechanismUnavailable);
        contract_test_support::assert_listener_absent(
            &empty_provider,
            IpAddr::V4(Ipv4Addr::new(192, 0, 2, 200)),
            NonZeroU16::new(49152).expect("nonzero fixture port"),
        );
    }

    #[test]
    fn provider_errors_expose_only_stable_categories() {
        let categories = [
            PortProviderErrorKind::PermissionDenied,
            PortProviderErrorKind::Unsupported,
            PortProviderErrorKind::MechanismUnavailable,
            PortProviderErrorKind::ParseFailure,
            PortProviderErrorKind::OperatingSystemFailure,
            PortProviderErrorKind::ProviderFailure,
        ];

        for category in categories {
            let error = PortProviderError::new(category);
            assert_eq!(error.kind(), category);
            assert_eq!(error.to_string(), category.to_string());
        }
    }

    #[test]
    fn partial_scan_carries_a_bounded_failure_category() {
        let provider = StubPortProvider {
            result: Ok(PortScanResult {
                listeners: vec![listener(
                    IpAddr::V6("2001:db8::10".parse().expect("valid IPv6 fixture")),
                    None,
                )],
                completeness: PortScanCompleteness::Partial(
                    PortProviderErrorKind::PermissionDenied,
                ),
            }),
        };

        let result = provider.listeners().expect("partial data is still usable");

        assert_eq!(result.listeners.len(), 1);
        assert_eq!(
            result.completeness,
            PortScanCompleteness::Partial(PortProviderErrorKind::PermissionDenied)
        );
    }
}
