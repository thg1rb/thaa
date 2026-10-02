//! Behavioral assertions reusable by native PortProvider integration tests.

use std::net::IpAddr;
use std::num::NonZeroU16;

use thaa_lib::domain::network::{NetworkListener, NetworkProtocol};
use thaa_lib::domain::port_provider::{PortProvider, PortScanCompleteness};

/// Requires a successful query and returns the matching listener together
/// with scan completeness so callers cannot discard partial-result status.
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
            listener.protocol == NetworkProtocol::Tcp
                && listener.local_address == Some(address)
                && listener.local_port == port
        })
        .expect(
            "controlled listener should be present with normalized protocol, address, and port",
        );

    (listener, completeness)
}

/// Requires a complete successful query and asserts a controlled endpoint is
/// absent, such as after its test-owned socket has been closed.
pub(crate) fn assert_listener_absent(
    provider: &dyn PortProvider,
    address: IpAddr,
    port: NonZeroU16,
) {
    let result = provider
        .listeners()
        .expect("provider query should succeed after the controlled listener closes");
    assert_eq!(result.completeness, PortScanCompleteness::Complete);
    assert!(!result.listeners.iter().any(|listener| {
        listener.protocol == NetworkProtocol::Tcp
            && listener.local_address == Some(address)
            && listener.local_port == port
    }));
}
