#![cfg(target_os = "macos")]

mod common;

use common::port_provider_contract::assert_listener_discovered;
use std::net::{Ipv4Addr, Ipv6Addr, TcpListener};
use std::num::NonZeroU16;
use thaa_lib::domain::network::NetworkProtocol;
use thaa_lib::domain::port_provider::{PortProvider, PortProviderErrorKind, PortScanCompleteness};
use thaa_lib::domain::process::ProcessId;
use thaa_lib::platform::macos::port_provider::MacOSPortProvider;

#[test]
fn discovers_controlled_ipv4_listener_and_pid_then_observes_close() {
    let socket = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind ephemeral IPv4");
    let address = socket.local_addr().expect("read listener address");
    let provider = MacOSPortProvider;

    let (found, _completeness) = assert_listener_discovered(
        &provider,
        address.ip(),
        NonZeroU16::new(address.port()).expect("ephemeral port is nonzero"),
    );
    assert_eq!(found.protocol, NetworkProtocol::Tcp);
    assert_eq!(found.owner_pid, Some(ProcessId::new(std::process::id())));
    drop(socket);
    let after_close = provider
        .listeners()
        .expect("provider query should succeed after the controlled listener closes");
    // Scoped IPv6 rows are retained with unknown address. That specific
    // partial status from unrelated host listeners does not hide this exact
    // IPv4 address/port pair; other partial causes still fail this test.
    assert!(matches!(
        after_close.completeness,
        PortScanCompleteness::Complete
            | PortScanCompleteness::Partial(PortProviderErrorKind::Unsupported)
    ));
    assert!(!after_close.listeners.iter().any(|listener| {
        listener.protocol == NetworkProtocol::Tcp
            && listener.local_address == Some(address.ip())
            && listener.local_port == NonZeroU16::new(address.port()).expect("nonzero port")
    }));
}

#[test]
fn discovers_controlled_ipv6_loopback_and_wildcard_listeners() {
    let ipv6_loopback = TcpListener::bind((Ipv6Addr::LOCALHOST, 0))
        .expect("host validation confirmed IPv6 loopback bind support");
    let ipv6_wildcard = TcpListener::bind((Ipv6Addr::UNSPECIFIED, 0))
        .expect("host validation confirmed IPv6 wildcard bind support");
    let provider = MacOSPortProvider;

    for socket in [&ipv6_loopback, &ipv6_wildcard] {
        let address = socket.local_addr().expect("read listener address");
        let (found, _) = assert_listener_discovered(
            &provider,
            address.ip(),
            NonZeroU16::new(address.port()).expect("ephemeral port is nonzero"),
        );
        assert_eq!(found.protocol, NetworkProtocol::Tcp);
    }
}

#[test]
fn discovers_controlled_ipv4_wildcard_listener() {
    let socket = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("bind IPv4 wildcard");
    let address = socket.local_addr().expect("read listener address");
    let (found, _) = assert_listener_discovered(
        &MacOSPortProvider,
        address.ip(),
        NonZeroU16::new(address.port()).expect("ephemeral port is nonzero"),
    );
    assert_eq!(found.local_address, Some(address.ip()));
}
