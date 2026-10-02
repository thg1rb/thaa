//! Deterministic tests of the public PortProvider contract.

mod common;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::num::NonZeroU16;

use common::port_provider_contract::{assert_listener_absent, assert_listener_discovered};
use thaa_lib::domain::network::{NetworkListener, NetworkProtocol};
use thaa_lib::domain::port_provider::{
    PortProvider, PortProviderError, PortProviderErrorKind, PortScanCompleteness, PortScanResult,
};
use thaa_lib::domain::process::ProcessId;

struct StubPortProvider {
    result: Result<PortScanResult, PortProviderError>,
}

impl PortProvider for StubPortProvider {
    fn listeners(&self) -> Result<PortScanResult, PortProviderError> {
        self.result.clone()
    }
}

fn listener(address: IpAddr, port: u16, owner_pid: Option<ProcessId>) -> NetworkListener {
    NetworkListener {
        protocol: NetworkProtocol::Tcp,
        local_address: Some(address),
        local_port: NonZeroU16::new(port).expect("nonzero fixture port"),
        owner_pid,
    }
}

#[test]
fn successful_result_preserves_address_family_and_unresolved_ownership() {
    let ipv4 = listener(
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        8080,
        Some(ProcessId::new(41)),
    );
    let ipv6 = listener(IpAddr::V6(Ipv6Addr::LOCALHOST), 8080, None);
    let provider = StubPortProvider {
        result: Ok(PortScanResult {
            listeners: vec![ipv4.clone(), ipv6.clone()],
            completeness: PortScanCompleteness::Complete,
        }),
    };

    let provider_object: &dyn PortProvider = &provider;
    let (found_ipv4, ipv4_completeness) = assert_listener_discovered(
        provider_object,
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        ipv4.local_port,
    );
    let (found_ipv6, ipv6_completeness) = assert_listener_discovered(
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
    assert_listener_absent(
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
                8080,
                None,
            )],
            completeness: PortScanCompleteness::Partial(PortProviderErrorKind::PermissionDenied),
        }),
    };

    let result = provider.listeners().expect("partial data is still usable");

    assert_eq!(result.listeners.len(), 1);
    assert_eq!(
        result.completeness,
        PortScanCompleteness::Partial(PortProviderErrorKind::PermissionDenied)
    );
}
