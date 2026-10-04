#![cfg(any(target_os = "macos", target_os = "windows"))]

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;

use thaa_lib::application::runtime_inspection::RuntimeInspector;

#[cfg(target_os = "macos")]
fn inspector() -> RuntimeInspector {
    use thaa_lib::platform::macos::{
        port_provider::MacOSPortProvider,
        process_controller::{MacOSPlatformCapabilitiesProvider, MacOSProcessController},
        process_provider::MacOSProcessProvider,
    };
    RuntimeInspector::new(
        Arc::new(MacOSPortProvider),
        Arc::new(MacOSProcessProvider),
        Arc::new(MacOSProcessController),
        Arc::new(MacOSPlatformCapabilitiesProvider),
    )
}

#[cfg(target_os = "windows")]
fn inspector() -> RuntimeInspector {
    use thaa_lib::platform::windows::{
        port_provider::WindowsPortProvider,
        process_controller::{WindowsPlatformCapabilitiesProvider, WindowsProcessController},
        process_provider::WindowsProcessProvider,
    };
    RuntimeInspector::new(
        Arc::new(WindowsPortProvider),
        Arc::new(WindowsProcessProvider),
        Arc::new(WindowsProcessController),
        Arc::new(WindowsPlatformCapabilitiesProvider),
    )
}

#[test]
fn runtime_snapshot_combines_native_listener_and_process_information() {
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .expect("bind controlled listener");
    let port = listener.local_addr().expect("listener address").port();
    let inspector = inspector();
    let snapshot = inspector
        .snapshot_or_initialize()
        .expect("native runtime snapshot");
    let entry = snapshot
        .entries
        .iter()
        .find(|entry| entry.listener.local_port.get() == port)
        .expect("controlled listener is present");

    assert_eq!(
        entry.listener.owner_pid.map(|pid| pid.get()),
        Some(std::process::id())
    );
    assert!(
        matches!(&entry.process, Some(Ok(info)) if info.identity.pid.get() == std::process::id())
    );
    assert!(entry.action_target_ref.is_some());
    assert_eq!(
        snapshot.capabilities.force_stop,
        thaa_lib::domain::capabilities::CapabilitySupport::Supported
    );

    drop(listener);
    let next = inspector.refresh().expect("subsequent scan");
    assert!(!next
        .entries
        .iter()
        .any(|entry| entry.listener.local_port.get() == port));
}

#[test]
fn capability_snapshot_keeps_windows_graceful_stop_unsupported() {
    #[cfg(target_os = "windows")]
    {
        let capabilities = inspector()
            .snapshot_or_initialize()
            .expect("snapshot")
            .capabilities;
        assert_eq!(
            capabilities.graceful_stop,
            thaa_lib::domain::capabilities::CapabilitySupport::Unsupported
        );
    }
    #[cfg(target_os = "macos")]
    {
        use thaa_lib::domain::capabilities::CapabilitySupport;
        let capabilities = inspector()
            .snapshot_or_initialize()
            .expect("snapshot")
            .capabilities;
        assert_eq!(capabilities.graceful_stop, CapabilitySupport::Supported);
    }
}
