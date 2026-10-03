#![cfg(target_os = "macos")]

mod common;

use common::port_provider_contract::{assert_listener_absent, assert_listener_discovered};
use std::net::{Ipv4Addr, Ipv6Addr, TcpListener};
use std::num::NonZeroU16;
use thaa_lib::domain::network::NetworkProtocol;
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
    assert_listener_absent(
        &provider,
        address.ip(),
        NonZeroU16::new(address.port()).expect("ephemeral port is nonzero"),
    );
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
