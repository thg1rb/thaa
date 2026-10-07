//! Normalized network listener values and pure address classification.

use std::net::IpAddr;
use std::num::NonZeroU16;

use super::process::ProcessId;

/// Network protocol included in the current P0 listening-port domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkProtocol {
    Tcp,
}

/// Conservative scope derived from an observed local binding address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingScope {
    LoopbackOnly,
    WildcardIpv4,
    WildcardIpv6,
    SpecificAddress,
    Unknown,
}

/// Classifies a normalized local address without consulting host interfaces.
///
/// A non-loopback address only means that the endpoint is bound beyond
/// loopback. It does not establish LAN or Internet reachability.
pub fn classify_binding(address: Option<IpAddr>) -> BindingScope {
    match address {
        None => BindingScope::Unknown,
        Some(IpAddr::V4(address)) if address.is_loopback() => BindingScope::LoopbackOnly,
        Some(IpAddr::V6(address)) if address.is_loopback() => BindingScope::LoopbackOnly,
        Some(IpAddr::V4(address)) if address.is_unspecified() => BindingScope::WildcardIpv4,
        Some(IpAddr::V6(address)) if address.is_unspecified() => BindingScope::WildcardIpv6,
        Some(IpAddr::V4(address)) if address.is_broadcast() || address.is_multicast() => {
            BindingScope::Unknown
        }
        Some(IpAddr::V6(address)) if address.is_multicast() => BindingScope::Unknown,
        Some(IpAddr::V6(address))
            if address
                .to_ipv4_mapped()
                .is_some_and(|mapped| mapped.is_loopback()) =>
        {
            BindingScope::LoopbackOnly
        }
        Some(IpAddr::V6(address))
            if address
                .to_ipv4_mapped()
                .is_some_and(|mapped| mapped.is_broadcast() || mapped.is_multicast()) =>
        {
            BindingScope::Unknown
        }
        Some(_) => BindingScope::SpecificAddress,
    }
}

/// One normalized, observed listening TCP endpoint.
///
/// Listener state is implicit: values represent listeners already selected by
/// a future provider's listening-TCP query. Missing address or owner data is
/// not a provider failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkListener {
    pub protocol: NetworkProtocol,
    pub local_address: Option<IpAddr>,
    pub local_port: NonZeroU16,
    pub owner_pid: Option<ProcessId>,
}

impl NetworkListener {
    /// Returns the binding classification for this listener's observed address.
    pub fn binding_scope(&self) -> BindingScope {
        classify_binding(self.local_address)
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_binding, BindingScope, NetworkListener, NetworkProtocol};
    use crate::domain::process::ProcessId;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::num::NonZeroU16;

    #[test]
    fn classifies_ipv4_and_ipv6_addresses_conservatively() {
        let cases = [
            (
                Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                BindingScope::LoopbackOnly,
            ),
            (
                Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
                BindingScope::WildcardIpv4,
            ),
            (
                Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10))),
                BindingScope::SpecificAddress,
            ),
            (
                Some(IpAddr::V6(Ipv6Addr::LOCALHOST)),
                BindingScope::LoopbackOnly,
            ),
            (
                Some(IpAddr::V6(Ipv6Addr::UNSPECIFIED)),
                BindingScope::WildcardIpv6,
            ),
            (
                Some(IpAddr::V6(
                    "2001:db8::10".parse().expect("valid IPv6 fixture"),
                )),
                BindingScope::SpecificAddress,
            ),
            (
                Some(IpAddr::V4(Ipv4Addr::new(127, 42, 0, 1))),
                BindingScope::LoopbackOnly,
            ),
            (
                Some(IpAddr::V6(
                    "::ffff:127.0.0.1"
                        .parse()
                        .expect("valid mapped IPv4 fixture"),
                )),
                BindingScope::LoopbackOnly,
            ),
            (
                Some(IpAddr::V6(
                    "::ffff:0.0.0.0"
                        .parse()
                        .expect("valid mapped wildcard fixture"),
                )),
                BindingScope::SpecificAddress,
            ),
            (
                Some(IpAddr::V6(
                    "::ffff:224.0.0.1"
                        .parse()
                        .expect("valid mapped multicast fixture"),
                )),
                BindingScope::Unknown,
            ),
            (
                Some(IpAddr::V6(
                    "::ffff:255.255.255.255"
                        .parse()
                        .expect("valid mapped broadcast fixture"),
                )),
                BindingScope::Unknown,
            ),
            (
                Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20))),
                BindingScope::SpecificAddress,
            ),
            (
                Some(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1))),
                BindingScope::Unknown,
            ),
            (Some(IpAddr::V4(Ipv4Addr::BROADCAST)), BindingScope::Unknown),
            (
                Some(IpAddr::V6(
                    "fe80::1".parse().expect("valid link-local fixture"),
                )),
                BindingScope::SpecificAddress,
            ),
            (
                Some(IpAddr::V6(
                    "ff02::1".parse().expect("valid multicast fixture"),
                )),
                BindingScope::Unknown,
            ),
            (None, BindingScope::Unknown),
        ];

        for (address, expected) in cases {
            assert_eq!(classify_binding(address), expected);
        }
    }

    #[test]
    fn listener_exposure_remains_endpoint_specific() {
        let loopback = NetworkListener {
            protocol: NetworkProtocol::Tcp,
            local_address: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            local_port: NonZeroU16::new(3000).expect("nonzero fixture port"),
            owner_pid: Some(ProcessId::new(42)),
        };
        let wildcard = NetworkListener {
            local_address: Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            local_port: NonZeroU16::new(8080).expect("nonzero fixture port"),
            ..loopback.clone()
        };

        assert_eq!(loopback.binding_scope(), BindingScope::LoopbackOnly);
        assert_eq!(wildcard.binding_scope(), BindingScope::WildcardIpv4);
        assert_eq!(loopback.owner_pid, wildcard.owner_pid);
    }

    #[test]
    fn listener_can_represent_unresolved_ownership_without_a_sentinel_pid() {
        let listener = NetworkListener {
            protocol: NetworkProtocol::Tcp,
            local_address: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            local_port: NonZeroU16::new(3000).expect("nonzero fixture port"),
            owner_pid: None,
        };

        assert_eq!(listener.owner_pid, None);
        assert_eq!(listener.binding_scope(), BindingScope::LoopbackOnly);

        let owned_listener = NetworkListener {
            owner_pid: Some(ProcessId::new(u32::MAX)),
            ..listener
        };

        assert_eq!(owned_listener.owner_pid, Some(ProcessId::new(u32::MAX)));
    }
}
