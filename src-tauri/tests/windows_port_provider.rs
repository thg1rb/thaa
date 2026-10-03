#![cfg(target_os = "windows")]

mod common;

use common::port_provider_contract::assert_listener_discovered;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, TcpListener};
use std::num::NonZeroU16;
use std::thread;
use std::time::{Duration, Instant};
use thaa_lib::domain::network::{NetworkListener, NetworkProtocol};
use thaa_lib::domain::port_provider::{PortProvider, PortProviderErrorKind, PortScanCompleteness};
use thaa_lib::domain::process::ProcessId;
use thaa_lib::platform::windows::port_provider::WindowsPortProvider;

fn assert_supported_completeness(completeness: PortScanCompleteness) {
    match completeness {
        PortScanCompleteness::Complete => {}
        PortScanCompleteness::Partial(PortProviderErrorKind::Unsupported) => {}
        other => panic!("unexpected scan completeness: {other:?}"),
    }
}

fn assert_controlled_listener(
    provider: &WindowsPortProvider,
    socket: &TcpListener,
) -> NetworkListener {
    let address = socket
        .local_addr()
        .expect("read controlled listener address");
    let port = NonZeroU16::new(address.port()).expect("ephemeral listener port is nonzero");
    let (listener, completeness) = assert_listener_discovered(provider, address.ip(), port);

    assert_eq!(listener.protocol, NetworkProtocol::Tcp);
    assert_eq!(listener.local_address, Some(address.ip()));
    assert_eq!(listener.local_port, port);
    assert_eq!(listener.owner_pid, Some(ProcessId::new(std::process::id())));
    assert_supported_completeness(completeness);
    listener
}

fn wait_for_listener_absence(provider: &WindowsPortProvider, address: IpAddr, port: NonZeroU16) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let result = provider
            .listeners()
            .expect("provider query should succeed after closing the controlled listener");
        assert_supported_completeness(result.completeness);
        let still_present = result.listeners.iter().any(|listener| {
            listener.protocol == NetworkProtocol::Tcp
                && listener.local_address == Some(address)
                && listener.local_port == port
        });
        if !still_present {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "controlled listener remained in the table after close"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

#[test]
fn discovers_controlled_ipv4_loopback_listener_and_pid_then_observes_close() {
    let socket = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind ephemeral IPv4 loopback");
    let listener = assert_controlled_listener(&WindowsPortProvider, &socket);
    let address = listener.local_address.expect("controlled IPv4 address");
    let port = listener.local_port;

    drop(socket);
    wait_for_listener_absence(&WindowsPortProvider, address, port);
}

#[test]
fn discovers_controlled_ipv4_wildcard_listener() {
    let socket = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("bind IPv4 wildcard");
    assert_controlled_listener(&WindowsPortProvider, &socket);
}

#[test]
fn discovers_controlled_ipv6_loopback_listener_and_pid() {
    let socket = TcpListener::bind((Ipv6Addr::LOCALHOST, 0))
        .expect("Windows CI runner must provide IPv6 loopback");
    let listener = assert_controlled_listener(&WindowsPortProvider, &socket);
    assert_eq!(
        listener.local_address,
        Some(IpAddr::V6(Ipv6Addr::LOCALHOST))
    );
}

#[test]
fn discovers_controlled_ipv6_wildcard_listener() {
    let socket = TcpListener::bind((Ipv6Addr::UNSPECIFIED, 0))
        .expect("Windows CI runner must provide IPv6 wildcard binding");
    let listener = assert_controlled_listener(&WindowsPortProvider, &socket);
    assert_eq!(
        listener.local_address,
        Some(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
    );
}
