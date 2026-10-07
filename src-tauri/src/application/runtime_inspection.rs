//! Runtime snapshot composition and refresh coordination.

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime};

use crate::application::process_icons::{
    is_bounded_png, ProcessIconAsset, ProcessIconProvider, SNAPSHOT_ICON_MAX_BYTES,
    SNAPSHOT_ICON_MAX_COUNT,
};
use crate::application::process_inspection::inspect_processes;
use crate::application::project_root::detect_project_root;
use crate::domain::capabilities::{
    CapabilitySupport, PlatformCapabilities, PlatformCapabilitiesProvider,
};
use crate::domain::git_context::{GitContext, GitContextProvider};
use crate::domain::metadata::FieldAvailability;
use crate::domain::network::NetworkListener;
use crate::domain::port_provider::{PortProvider, PortProviderErrorKind, PortScanCompleteness};
use crate::domain::process::{ProcessId, ProcessInfo};
use crate::domain::process_action::{
    ProcessAction, ProcessActionError, ProcessActionOutcome, ProcessActionTarget,
};
use crate::domain::process_controller::ProcessController;
use crate::domain::process_provider::{ProcessProvider, ProcessProviderError};

/// One normalized listener and its optional process inspection result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEntry {
    pub entry_ref: String,
    pub listener: NetworkListener,
    pub process: Option<Result<ProcessInfo, ProcessProviderError>>,
    /// Optional local context derived from the observed working directory.
    /// This is presentation metadata, never process identity evidence.
    pub project_root: Option<PathBuf>,
    /// Optional Git repository metadata derived from the observed working
    /// directory. It is presentation metadata, never process identity evidence.
    pub git_context: Option<GitContext>,
    /// A snapshot-scoped lookup reference, not an authorization token.
    pub action_target_ref: Option<String>,
    /// Snapshot-scoped presentation reference; never used as process identity.
    pub process_icon_ref: Option<String>,
}

/// One completed port scan with all listener rows and platform capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSnapshot {
    pub generation: u64,
    pub observed_at: SystemTime,
    pub completeness: PortScanCompleteness,
    pub capabilities: PlatformCapabilities,
    pub entries: Vec<RuntimeEntry>,
}

/// Stable categories for a refresh that could not produce a new snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeScanError {
    PortProvider(PortProviderErrorKind),
    ProviderFailure,
}

impl fmt::Display for RuntimeScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PortProvider(kind) => kind.fmt(formatter),
            Self::ProviderFailure => formatter.write_str("runtime inspection failed"),
        }
    }
}

#[derive(Debug)]
struct ScannedSnapshot {
    snapshot: Arc<RuntimeSnapshot>,
    action_targets: HashMap<String, ProcessActionTarget>,
    icon_sources: Vec<(String, ProcessInfo)>,
}

struct RuntimeScanner {
    ports: Arc<dyn PortProvider>,
    processes: Arc<dyn ProcessProvider>,
    capabilities: Arc<dyn PlatformCapabilitiesProvider>,
    git_contexts: Arc<dyn GitContextProvider>,
}

const GIT_CONTEXT_SCAN_BUDGET: Duration = Duration::from_secs(2);

impl RuntimeScanner {
    fn scan(&self, generation: u64) -> Result<ScannedSnapshot, RuntimeScanError> {
        // A logical refresh performs exactly one port-provider query.
        let port_scan = self
            .ports
            .listeners()
            .map_err(|error| RuntimeScanError::PortProvider(error.kind()))?;
        let process_ids: Vec<ProcessId> = port_scan
            .listeners
            .iter()
            .filter_map(|listener| listener.owner_pid)
            .collect();
        let inspections = inspect_processes(self.processes.as_ref(), &process_ids);

        let mut inspection_by_pid = HashMap::with_capacity(inspections.len());
        let mut action_targets = HashMap::new();
        let mut reference_by_pid = HashMap::new();
        let mut icon_reference_by_pid = HashMap::new();
        let mut project_root_by_pid = HashMap::new();
        let mut git_context_by_pid = HashMap::new();
        let mut git_context_by_directory = HashMap::new();
        let git_context_deadline = Instant::now() + GIT_CONTEXT_SCAN_BUDGET;
        let mut icon_sources = Vec::new();
        for (index, outcome) in inspections.into_iter().enumerate() {
            let pid = outcome.requested_process_id.get();
            if let Ok(info) = &outcome.result {
                if let FieldAvailability::Available(working_directory) = &info.working_directory {
                    if let Some(project_root) = detect_project_root(working_directory) {
                        project_root_by_pid.insert(pid, project_root);
                    }
                    let context = git_context_by_directory
                        .entry(working_directory.clone())
                        .or_insert_with(|| {
                            let budget =
                                git_context_deadline.saturating_duration_since(Instant::now());
                            self.git_contexts.context_for(working_directory, budget)
                        })
                        .clone();
                    if let Some(context) = context {
                        git_context_by_pid.insert(pid, context);
                    }
                }
                if icon_sources.len() < SNAPSHOT_ICON_MAX_COUNT {
                    let reference = format!("icon-{generation:x}-{:x}", icon_sources.len());
                    icon_reference_by_pid.insert(pid, reference.clone());
                    icon_sources.push((reference, info.clone()));
                }
                if let Ok(target) = ProcessActionTarget::from_identity(&info.identity) {
                    let reference = format!("target-{generation:x}-{index:x}");
                    reference_by_pid.insert(pid, reference.clone());
                    action_targets.insert(reference, target);
                }
            }
            inspection_by_pid.insert(pid, outcome.result);
        }

        let entries = port_scan
            .listeners
            .into_iter()
            .enumerate()
            .map(|(index, listener)| {
                let pid = listener.owner_pid.map(ProcessId::get);
                RuntimeEntry {
                    entry_ref: next_entry_ref(generation, index),
                    listener,
                    process: pid.and_then(|pid| inspection_by_pid.get(&pid).cloned()),
                    project_root: pid.and_then(|pid| project_root_by_pid.get(&pid).cloned()),
                    git_context: pid.and_then(|pid| git_context_by_pid.get(&pid).cloned()),
                    action_target_ref: pid.and_then(|pid| reference_by_pid.get(&pid).cloned()),
                    process_icon_ref: pid.and_then(|pid| icon_reference_by_pid.get(&pid).cloned()),
                }
            })
            .collect();

        Ok(ScannedSnapshot {
            snapshot: Arc::new(RuntimeSnapshot {
                generation,
                observed_at: SystemTime::now(),
                completeness: port_scan.completeness,
                capabilities: self.capabilities.capabilities(),
                entries,
            }),
            action_targets,
            icon_sources,
        })
    }
}

#[derive(Default)]
struct RefreshState {
    snapshot: Option<Arc<RuntimeSnapshot>>,
    action_targets: HashMap<String, ProcessActionTarget>,
    icon_sources: Vec<(String, ProcessInfo)>,
    resolved_icons: Option<(u64, Vec<ProcessIconAsset>)>,
    latest_requested: u64,
    latest_completed: u64,
    running: bool,
    latest_result: Option<Result<Arc<RuntimeSnapshot>, RuntimeScanError>>,
    last_error: Option<RuntimeScanError>,
}

/// Owns the canonical snapshot and serializes synchronous provider scans.
///
/// Concurrent refresh requests retain one latest pending generation. A scan
/// superseded before commit is discarded, and the previous snapshot remains
/// available until a newer successful scan is committed.
pub struct RuntimeInspector {
    scanner: RuntimeScanner,
    controller: Arc<dyn ProcessController>,
    icons: Arc<dyn ProcessIconProvider>,
    state: Mutex<RefreshState>,
    icon_resolution: Mutex<()>,
    settled: Condvar,
}

impl RuntimeInspector {
    pub fn new(
        ports: Arc<dyn PortProvider>,
        processes: Arc<dyn ProcessProvider>,
        controller: Arc<dyn ProcessController>,
        capabilities: Arc<dyn PlatformCapabilitiesProvider>,
    ) -> Self {
        Self::new_with_icons(
            ports,
            processes,
            Arc::new(NoProcessIconProvider),
            controller,
            capabilities,
        )
    }

    pub fn new_with_icons(
        ports: Arc<dyn PortProvider>,
        processes: Arc<dyn ProcessProvider>,
        icons: Arc<dyn ProcessIconProvider>,
        controller: Arc<dyn ProcessController>,
        capabilities: Arc<dyn PlatformCapabilitiesProvider>,
    ) -> Self {
        Self::new_with_icons_and_git_context(
            ports,
            processes,
            icons,
            Arc::new(NoGitContextProvider),
            controller,
            capabilities,
        )
    }

    pub fn new_with_icons_and_git_context(
        ports: Arc<dyn PortProvider>,
        processes: Arc<dyn ProcessProvider>,
        icons: Arc<dyn ProcessIconProvider>,
        git_contexts: Arc<dyn GitContextProvider>,
        controller: Arc<dyn ProcessController>,
        capabilities: Arc<dyn PlatformCapabilitiesProvider>,
    ) -> Self {
        Self {
            scanner: RuntimeScanner {
                ports,
                processes,
                capabilities,
                git_contexts,
            },
            controller,
            icons,
            state: Mutex::new(RefreshState::default()),
            icon_resolution: Mutex::new(()),
            settled: Condvar::new(),
        }
    }

    /// Returns the canonical snapshot, sharing one initial scan between callers.
    pub fn snapshot_or_initialize(&self) -> Result<Arc<RuntimeSnapshot>, RuntimeScanError> {
        let (generation, run_worker) = {
            let mut state = self.lock_state();
            if let Some(snapshot) = &state.snapshot {
                return Ok(Arc::clone(snapshot));
            }
            if state.running {
                (state.latest_requested, false)
            } else {
                let generation = next_generation(&mut state)?;
                state.running = true;
                (generation, true)
            }
        };
        if run_worker {
            self.run_worker();
        }
        self.wait_for(generation)
    }

    /// Requests a fresh snapshot; overlapping requests coalesce to the latest pending scan.
    pub fn refresh(&self) -> Result<Arc<RuntimeSnapshot>, RuntimeScanError> {
        let (generation, run_worker) = {
            let mut state = self.lock_state();
            let generation = next_generation(&mut state)?;
            if state.running {
                (generation, false)
            } else {
                state.running = true;
                (generation, true)
            }
        };
        if run_worker {
            self.run_worker();
        }
        self.wait_for(generation)
    }

    pub fn current_snapshot(&self) -> Option<Arc<RuntimeSnapshot>> {
        self.lock_state().snapshot.clone()
    }

    pub fn last_error(&self) -> Option<RuntimeScanError> {
        self.lock_state().last_error
    }

    /// Resolves presentation-only assets for the currently committed snapshot.
    /// Invoke from a background worker after returning runtime rows. A stale
    /// generation cannot resolve icons from a newer snapshot.
    pub fn process_icons_for_snapshot(&self, generation: u64) -> Vec<ProcessIconAsset> {
        let _resolution = self
            .icon_resolution
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let sources = {
            let state = self.lock_state();
            if state.snapshot.as_ref().map(|snapshot| snapshot.generation) != Some(generation) {
                return Vec::new();
            }
            if let Some((resolved_generation, assets)) = &state.resolved_icons {
                if *resolved_generation == generation {
                    return assets.clone();
                }
            }
            state.icon_sources.clone()
        };

        let mut assets = Vec::new();
        let mut total_bytes = 0usize;
        for (reference, process) in sources {
            if assets.len() >= SNAPSHOT_ICON_MAX_COUNT {
                break;
            }
            let Some(png) = self
                .icons
                .icon_png(&process)
                .filter(|png| is_bounded_png(png))
            else {
                continue;
            };
            let Some(next_total) = total_bytes.checked_add(png.len()) else {
                break;
            };
            if next_total > SNAPSHOT_ICON_MAX_BYTES {
                break;
            }
            total_bytes = next_total;
            assets.push(ProcessIconAsset { reference, png });
        }
        let mut state = self.lock_state();
        if state.snapshot.as_ref().map(|snapshot| snapshot.generation) != Some(generation) {
            return Vec::new();
        }
        state.resolved_icons = Some((generation, assets.clone()));
        assets
    }

    /// Resolves a current snapshot reference and delegates final identity validation to the controller.
    pub fn request_action(
        &self,
        action_target_ref: &str,
        action: ProcessAction,
    ) -> Result<ProcessActionOutcome, ProcessActionError> {
        let (target, capabilities) = {
            let state = self.lock_state();
            let target = state
                .action_targets
                .get(action_target_ref)
                .cloned()
                .ok_or(ProcessActionError::IdentityUnavailable)?;
            let capabilities = state
                .snapshot
                .as_ref()
                .ok_or(ProcessActionError::IdentityUnavailable)?
                .capabilities;
            (target, capabilities)
        };
        let support = match action {
            ProcessAction::GracefulStop => capabilities.graceful_stop,
            ProcessAction::ForceStop => capabilities.force_stop,
        };
        if support != CapabilitySupport::Supported {
            return Err(ProcessActionError::Unsupported);
        }
        self.controller.request(&target, action)
    }

    /// Builds a fixed-scheme URL from a listener in the current snapshot only.
    pub fn listener_url(&self, entry_ref: &str) -> Result<String, ListenerUrlError> {
        if entry_ref.len() > 96 {
            return Err(ListenerUrlError::StaleReference);
        }
        let state = self.lock_state();
        let snapshot = state
            .snapshot
            .as_ref()
            .ok_or(ListenerUrlError::StaleReference)?;
        let prefix = format!("entry-{:x}-", snapshot.generation);
        let index = entry_ref
            .strip_prefix(&prefix)
            .and_then(|value| usize::from_str_radix(value, 16).ok())
            .ok_or(ListenerUrlError::StaleReference)?;
        let entry = snapshot
            .entries
            .get(index)
            .ok_or(ListenerUrlError::StaleReference)?;
        build_local_url(&entry.listener).ok_or(ListenerUrlError::AddressUnavailable)
    }

    fn run_worker(&self) {
        loop {
            let generation = self.lock_state().latest_requested;
            let scanned = self.scanner.scan(generation);
            let mut state = self.lock_state();
            if state.latest_requested != generation {
                continue;
            }

            let result = match scanned {
                Ok(scanned) => {
                    state.action_targets = scanned.action_targets;
                    state.icon_sources = scanned.icon_sources;
                    state.resolved_icons = None;
                    state.snapshot = Some(Arc::clone(&scanned.snapshot));
                    state.last_error = None;
                    Ok(scanned.snapshot)
                }
                Err(error) => {
                    state.last_error = Some(error);
                    Err(error)
                }
            };
            state.latest_completed = generation;
            state.latest_result = Some(result);
            state.running = false;
            self.settled.notify_all();
            return;
        }
    }

    fn wait_for(&self, generation: u64) -> Result<Arc<RuntimeSnapshot>, RuntimeScanError> {
        let mut state = self.lock_state();
        while state.latest_completed < generation {
            state = self
                .settled
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        state
            .latest_result
            .clone()
            .unwrap_or(Err(RuntimeScanError::ProviderFailure))
    }

    fn lock_state(&self) -> MutexGuard<'_, RefreshState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

struct NoGitContextProvider;

impl GitContextProvider for NoGitContextProvider {
    fn context_for(
        &self,
        _working_directory: &std::path::Path,
        _time_budget: Duration,
    ) -> Option<GitContext> {
        None
    }
}

struct NoProcessIconProvider;

impl ProcessIconProvider for NoProcessIconProvider {
    fn icon_png(&self, _process: &ProcessInfo) -> Option<Vec<u8>> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerUrlError {
    StaleReference,
    AddressUnavailable,
}

fn next_generation(state: &mut RefreshState) -> Result<u64, RuntimeScanError> {
    let generation = state
        .latest_requested
        .checked_add(1)
        .ok_or(RuntimeScanError::ProviderFailure)?;
    state.latest_requested = generation;
    Ok(generation)
}

/// Uses only an OS-observed address and the fixed `http` scheme.
pub fn build_local_url(listener: &NetworkListener) -> Option<String> {
    let address = listener.local_address?;
    let host = if address.is_unspecified() {
        match address {
            std::net::IpAddr::V4(_) => "127.0.0.1".to_owned(),
            std::net::IpAddr::V6(_) => "::1".to_owned(),
        }
    } else {
        address.to_string()
    };
    let host = if host.contains(':') {
        let host = host.replace('%', "%25");
        format!("[{host}]")
    } else {
        host
    };
    Some(format!("http://{host}:{}", listener.local_port))
}

fn next_entry_ref(generation: u64, index: usize) -> String {
    format!("entry-{generation:x}-{index:x}")
}

#[cfg(test)]
mod tests {
    use super::{
        build_local_url, next_entry_ref, ListenerUrlError, RuntimeInspector, RuntimeScanError,
        GIT_CONTEXT_SCAN_BUDGET,
    };
    use crate::application::process_icons::ProcessIconProvider;
    use crate::domain::capabilities::{
        CapabilitySupport, PlatformCapabilities, PlatformCapabilitiesProvider,
    };
    use crate::domain::git_context::{GitContext, GitContextProvider};
    use crate::domain::metadata::{FieldAvailability, UnavailableReason};
    use crate::domain::network::{NetworkListener, NetworkProtocol};
    use crate::domain::port_provider::{
        PortProvider, PortProviderError, PortProviderErrorKind, PortScanCompleteness,
        PortScanResult,
    };
    use crate::domain::process::{ProcessId, ProcessIdentity, ProcessInfo};
    use crate::domain::process_action::{
        ProcessAction, ProcessActionError, ProcessActionOutcome, ProcessActionTarget,
    };
    use crate::domain::process_controller::ProcessController;
    use crate::domain::process_provider::{
        ProcessProvider, ProcessProviderError, ProcessProviderErrorKind,
    };
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::num::NonZeroU16;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;
    use std::time::{Duration, UNIX_EPOCH};

    struct FakePorts {
        result: Mutex<Result<PortScanResult, PortProviderErrorKind>>,
        calls: AtomicUsize,
    }
    impl PortProvider for FakePorts {
        fn listeners(&self) -> Result<PortScanResult, PortProviderError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.result
                .lock()
                .expect("ports lock")
                .clone()
                .map_err(PortProviderError::new)
        }
    }

    struct FakeProcesses {
        responses: Mutex<HashMap<u32, Result<ProcessInfo, ProcessProviderErrorKind>>>,
        calls: Mutex<Vec<u32>>,
    }
    impl ProcessProvider for FakeProcesses {
        fn inspect(&self, pid: ProcessId) -> Result<ProcessInfo, ProcessProviderError> {
            self.calls.lock().expect("calls lock").push(pid.get());
            self.responses
                .lock()
                .expect("responses lock")
                .get(&pid.get())
                .cloned()
                .unwrap_or(Err(ProcessProviderErrorKind::ProcessDisappeared))
                .map_err(ProcessProviderError::new)
        }
    }

    struct FakeCapabilities;
    impl PlatformCapabilitiesProvider for FakeCapabilities {
        fn capabilities(&self) -> PlatformCapabilities {
            PlatformCapabilities {
                command_line: CapabilitySupport::Unsupported,
                working_directory: CapabilitySupport::Supported,
                graceful_stop: CapabilitySupport::Supported,
                force_stop: CapabilitySupport::Supported,
            }
        }
    }

    struct FakeIcons {
        calls: AtomicUsize,
        icon: Option<Vec<u8>>,
    }

    struct FakeGitContexts {
        calls: Mutex<Vec<PathBuf>>,
        context: Option<GitContext>,
    }

    impl GitContextProvider for FakeGitContexts {
        fn context_for(
            &self,
            working_directory: &std::path::Path,
            time_budget: Duration,
        ) -> Option<GitContext> {
            assert!(time_budget <= GIT_CONTEXT_SCAN_BUDGET);
            self.calls
                .lock()
                .expect("Git context calls lock")
                .push(working_directory.to_path_buf());
            self.context.clone()
        }
    }
    impl ProcessIconProvider for FakeIcons {
        fn icon_png(&self, _process: &ProcessInfo) -> Option<Vec<u8>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.icon.clone()
        }
    }

    #[derive(Default)]
    struct FakeController(Mutex<Vec<(u32, ProcessAction)>>);
    impl ProcessController for FakeController {
        fn request(
            &self,
            target: &ProcessActionTarget,
            action: ProcessAction,
        ) -> Result<ProcessActionOutcome, ProcessActionError> {
            self.0
                .lock()
                .expect("controller lock")
                .push((target.process_id().get(), action));
            Ok(ProcessActionOutcome::Requested)
        }
    }

    fn listener(address: Option<IpAddr>, port: u16, pid: Option<u32>) -> NetworkListener {
        NetworkListener {
            protocol: NetworkProtocol::Tcp,
            local_address: address,
            local_port: NonZeroU16::new(port).expect("fixture port"),
            owner_pid: pid.map(ProcessId::new),
        }
    }

    fn process(pid: u32) -> ProcessInfo {
        process_with_working_directory(
            pid,
            FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation),
        )
    }

    fn process_with_working_directory(
        pid: u32,
        working_directory: FieldAvailability<PathBuf>,
    ) -> ProcessInfo {
        ProcessInfo {
            identity: ProcessIdentity {
                pid: ProcessId::new(pid),
                name: FieldAvailability::Available(OsString::from(format!("process-{pid}"))),
                executable_path: FieldAvailability::Unavailable(
                    UnavailableReason::ProviderLimitation,
                ),
                start_time: FieldAvailability::Available(UNIX_EPOCH + Duration::from_secs(20)),
            },
            command_arguments: FieldAvailability::Unavailable(
                UnavailableReason::ProviderLimitation,
            ),
            working_directory,
        }
    }

    fn inspector(
        port_result: Result<PortScanResult, PortProviderErrorKind>,
        process_responses: impl IntoIterator<
            Item = (u32, Result<ProcessInfo, ProcessProviderErrorKind>),
        >,
    ) -> (Arc<RuntimeInspector>, Arc<FakePorts>, Arc<FakeProcesses>) {
        let ports = Arc::new(FakePorts {
            result: Mutex::new(port_result),
            calls: AtomicUsize::new(0),
        });
        let processes = Arc::new(FakeProcesses {
            responses: Mutex::new(process_responses.into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        });
        let inspector = Arc::new(RuntimeInspector::new(
            ports.clone(),
            processes.clone(),
            Arc::new(FakeController::default()),
            Arc::new(FakeCapabilities),
        ));
        (inspector, ports, processes)
    }

    fn complete(listeners: Vec<NetworkListener>) -> Result<PortScanResult, PortProviderErrorKind> {
        Ok(PortScanResult {
            listeners,
            completeness: PortScanCompleteness::Complete,
        })
    }

    #[test]
    fn snapshot_preserves_listener_rows_and_inspects_shared_pid_once() {
        let (inspector, ports, processes) = inspector(
            complete(vec![
                listener(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), 3000, Some(7)),
                listener(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), 3001, Some(7)),
            ]),
            [(7, Ok(process(7)))],
        );
        let snapshot = inspector.snapshot_or_initialize().expect("scan succeeds");
        assert_eq!(snapshot.entries.len(), 2);
        assert_eq!(ports.calls.load(Ordering::SeqCst), 1);
        assert_eq!(*processes.calls.lock().expect("calls lock"), vec![7]);
        assert_eq!(
            snapshot.entries[0].action_target_ref,
            snapshot.entries[1].action_target_ref
        );
        assert_eq!(
            snapshot.capabilities.force_stop,
            CapabilitySupport::Supported
        );
    }

    #[test]
    fn project_root_enrichment_is_optional_and_does_not_change_action_target() {
        static NEXT_PROJECT_FIXTURE: AtomicUsize = AtomicUsize::new(0);
        let fixture_id = NEXT_PROJECT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let working_directory = std::env::temp_dir().join(format!(
            "thaa-runtime-project-root-{}-{fixture_id}",
            std::process::id()
        ));
        let project_root = working_directory.join("project");
        let nested = project_root.join("src");
        std::fs::create_dir_all(&nested).expect("create working directory fixture");
        std::fs::write(project_root.join("package.json"), b"{}").expect("create marker");

        let (inspector, _, _) = inspector(
            complete(vec![
                listener(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), 3000, Some(7)),
                listener(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), 3001, Some(7)),
            ]),
            [(
                7,
                Ok(process_with_working_directory(
                    7,
                    FieldAvailability::Available(nested),
                )),
            )],
        );
        let snapshot = inspector.snapshot_or_initialize().expect("scan succeeds");
        let expected_root = std::fs::canonicalize(&project_root).expect("canonical fixture root");
        assert_eq!(snapshot.entries.len(), 2);
        assert!(snapshot.entries.iter().all(|entry| {
            entry.project_root.as_ref() == Some(&expected_root) && entry.action_target_ref.is_some()
        }));

        std::fs::remove_dir_all(working_directory).expect("remove working directory fixture");
    }

    #[test]
    fn unavailable_project_root_does_not_hide_listener_or_action() {
        let (inspector, _, _) = inspector(
            complete(vec![listener(
                Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                3000,
                Some(7),
            )]),
            [(7, Ok(process(7)))],
        );
        let snapshot = inspector.snapshot_or_initialize().expect("scan succeeds");
        assert_eq!(snapshot.entries.len(), 1);
        assert_eq!(snapshot.entries[0].project_root, None);
        assert!(snapshot.entries[0].action_target_ref.is_some());
    }

    #[test]
    fn no_project_marker_does_not_hide_listener_or_action() {
        static NEXT_NO_ROOT_FIXTURE: AtomicUsize = AtomicUsize::new(0);
        let fixture_id = NEXT_NO_ROOT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let working_directory = std::env::temp_dir().join(format!(
            "thaa-runtime-no-project-{}-{fixture_id}/nested",
            std::process::id()
        ));
        std::fs::create_dir_all(&working_directory).expect("create unmarked working directory");

        let (inspector, _, _) = inspector(
            complete(vec![listener(
                Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                3000,
                Some(7),
            )]),
            [(
                7,
                Ok(process_with_working_directory(
                    7,
                    FieldAvailability::Available(working_directory.clone()),
                )),
            )],
        );
        let snapshot = inspector.snapshot_or_initialize().expect("scan succeeds");

        assert_eq!(snapshot.entries.len(), 1);
        assert_eq!(snapshot.entries[0].project_root, None);
        assert!(snapshot.entries[0].action_target_ref.is_some());

        std::fs::remove_dir_all(working_directory.parent().expect("fixture parent"))
            .expect("remove unmarked fixture");
    }

    #[test]
    fn git_context_is_deduplicated_by_working_directory_and_is_presentation_only() {
        let working_directory =
            std::env::temp_dir().join(format!("thaa-runtime-git-context-{}", std::process::id()));
        std::fs::create_dir_all(&working_directory).expect("create fixture directory");
        let ports = Arc::new(FakePorts {
            result: Mutex::new(complete(vec![
                listener(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), 3000, Some(7)),
                listener(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), 3001, Some(8)),
            ])),
            calls: AtomicUsize::new(0),
        });
        let processes = Arc::new(FakeProcesses {
            responses: Mutex::new(
                [
                    (
                        7,
                        Ok(process_with_working_directory(
                            7,
                            FieldAvailability::Available(working_directory.clone()),
                        )),
                    ),
                    (
                        8,
                        Ok(process_with_working_directory(
                            8,
                            FieldAvailability::Available(working_directory.clone()),
                        )),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            calls: Mutex::new(Vec::new()),
        });
        let expected = GitContext {
            repository_root: working_directory.clone(),
            branch: crate::domain::git_context::GitBranch::Named("feature/context".into()),
        };
        let git_contexts = Arc::new(FakeGitContexts {
            calls: Mutex::new(Vec::new()),
            context: Some(expected.clone()),
        });
        let inspector = RuntimeInspector::new_with_icons_and_git_context(
            ports,
            processes,
            Arc::new(FakeIcons {
                calls: AtomicUsize::new(0),
                icon: None,
            }),
            git_contexts.clone(),
            Arc::new(FakeController::default()),
            Arc::new(FakeCapabilities),
        );

        let snapshot = inspector.snapshot_or_initialize().expect("scan succeeds");

        assert_eq!(snapshot.entries.len(), 2);
        assert!(snapshot
            .entries
            .iter()
            .all(|entry| entry.git_context.as_ref() == Some(&expected)));
        assert_eq!(
            git_contexts
                .calls
                .lock()
                .expect("Git context calls lock")
                .as_slice(),
            std::slice::from_ref(&working_directory)
        );
        assert!(snapshot
            .entries
            .iter()
            .all(|entry| entry.action_target_ref.is_some()));
        std::fs::remove_dir_all(working_directory).expect("remove fixture directory");
    }

    #[test]
    fn duplicate_listener_processes_share_one_snapshot_icon() {
        let ports = Arc::new(FakePorts {
            result: Mutex::new(complete(vec![
                listener(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), 3000, Some(7)),
                listener(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), 3001, Some(7)),
            ])),
            calls: AtomicUsize::new(0),
        });
        let processes = Arc::new(FakeProcesses {
            responses: Mutex::new([(7, Ok(process(7)))].into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        });
        let icon = png_fixture();
        let icons = Arc::new(FakeIcons {
            calls: AtomicUsize::new(0),
            icon: Some(icon.clone()),
        });
        let inspector = RuntimeInspector::new_with_icons(
            ports,
            processes,
            icons.clone(),
            Arc::new(FakeController::default()),
            Arc::new(FakeCapabilities),
        );

        let snapshot = inspector.snapshot_or_initialize().expect("scan succeeds");
        assert_eq!(icons.calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            snapshot.entries[0].process_icon_ref,
            snapshot.entries[1].process_icon_ref
        );
        let icons_result = inspector.process_icons_for_snapshot(snapshot.generation);
        assert_eq!(icons.calls.load(Ordering::SeqCst), 1);
        assert_eq!(icons_result.len(), 1);
        assert_eq!(icons_result[0].png, icon);
        assert_eq!(
            inspector.process_icons_for_snapshot(snapshot.generation),
            icons_result
        );
        assert_eq!(icons.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn invalid_icon_falls_back_without_dropping_runtime_entry() {
        let ports = Arc::new(FakePorts {
            result: Mutex::new(complete(vec![listener(
                Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                3000,
                Some(7),
            )])),
            calls: AtomicUsize::new(0),
        });
        let processes = Arc::new(FakeProcesses {
            responses: Mutex::new([(7, Ok(process(7)))].into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        });
        let inspector = RuntimeInspector::new_with_icons(
            ports,
            processes,
            Arc::new(FakeIcons {
                calls: AtomicUsize::new(0),
                icon: Some(b"bad png".to_vec()),
            }),
            Arc::new(FakeController::default()),
            Arc::new(FakeCapabilities),
        );

        let snapshot = inspector.snapshot_or_initialize().expect("scan succeeds");
        assert_eq!(snapshot.entries.len(), 1);
        assert!(snapshot.entries[0].process.is_some());
        assert!(snapshot.entries[0].process_icon_ref.is_some());
        assert!(inspector
            .process_icons_for_snapshot(snapshot.generation)
            .is_empty());
    }

    #[test]
    fn stale_snapshot_cannot_resolve_icon_assets_for_newer_processes() {
        let ports = Arc::new(FakePorts {
            result: Mutex::new(complete(vec![listener(
                Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                3000,
                Some(7),
            )])),
            calls: AtomicUsize::new(0),
        });
        let processes = Arc::new(FakeProcesses {
            responses: Mutex::new([(7, Ok(process(7)))].into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        });
        let icons = Arc::new(FakeIcons {
            calls: AtomicUsize::new(0),
            icon: Some(png_fixture()),
        });
        let inspector = RuntimeInspector::new_with_icons(
            ports,
            processes,
            icons.clone(),
            Arc::new(FakeController::default()),
            Arc::new(FakeCapabilities),
        );
        let first = inspector.snapshot_or_initialize().expect("initial scan");
        inspector.refresh().expect("new generation");
        assert!(inspector
            .process_icons_for_snapshot(first.generation)
            .is_empty());
        assert_eq!(icons.calls.load(Ordering::SeqCst), 0);
    }

    fn png_fixture() -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, 2, 2);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("PNG header");
        writer.write_image_data(&[0; 16]).expect("PNG pixels");
        drop(writer);
        bytes
    }

    #[test]
    fn unknown_owners_and_process_errors_keep_listener_visible() {
        let (inspector, _, _) = inspector(
            complete(vec![
                listener(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), 3000, None),
                listener(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)), 3001, Some(9)),
            ]),
            [(9, Err(ProcessProviderErrorKind::PermissionDenied))],
        );
        let snapshot = inspector
            .snapshot_or_initialize()
            .expect("port scan succeeds");
        assert_eq!(snapshot.entries.len(), 2);
        assert!(snapshot.entries[0].process.is_none());
        assert!(matches!(
            snapshot.entries[1].process.as_ref(),
            Some(Err(error)) if error.kind() == ProcessProviderErrorKind::PermissionDenied
        ));
        assert!(snapshot.entries[1].action_target_ref.is_none());
    }

    #[test]
    fn partial_and_empty_scans_are_successful_snapshots() {
        let (partial, _, _) = inspector(
            Ok(PortScanResult {
                listeners: vec![listener(None, 5000, None)],
                completeness: PortScanCompleteness::Partial(
                    PortProviderErrorKind::PermissionDenied,
                ),
            }),
            [],
        );
        let partial_snapshot = partial
            .snapshot_or_initialize()
            .expect("partial data is useful");
        assert_eq!(partial_snapshot.entries.len(), 1);
        assert_eq!(
            partial_snapshot.completeness,
            PortScanCompleteness::Partial(PortProviderErrorKind::PermissionDenied)
        );
        let (empty, _, _) = inspector(complete(vec![]), []);
        assert!(empty
            .snapshot_or_initialize()
            .expect("empty is successful")
            .entries
            .is_empty());
    }

    #[test]
    fn provider_failure_preserves_last_successful_snapshot() {
        let (inspector, ports, _) = inspector(complete(vec![]), []);
        let original = inspector.snapshot_or_initialize().expect("initial scan");
        *ports.result.lock().expect("ports lock") = Err(PortProviderErrorKind::PermissionDenied);
        assert_eq!(
            inspector.refresh(),
            Err(RuntimeScanError::PortProvider(
                PortProviderErrorKind::PermissionDenied
            ))
        );
        assert_eq!(
            inspector
                .current_snapshot()
                .expect("previous snapshot")
                .generation,
            original.generation
        );
        assert_eq!(
            inspector.last_error(),
            Some(RuntimeScanError::PortProvider(
                PortProviderErrorKind::PermissionDenied
            ))
        );
        *ports.result.lock().expect("ports lock") = complete(vec![]);
        let recovered = inspector.refresh().expect("later refresh recovers");
        assert!(recovered.entries.is_empty());
        assert_eq!(inspector.last_error(), None);
    }

    #[test]
    fn action_reference_expires_after_new_snapshot_commit() {
        let (inspector, _, _) = inspector(
            complete(vec![listener(
                Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                3000,
                Some(7),
            )]),
            [(7, Ok(process(7)))],
        );
        let snapshot = inspector.snapshot_or_initialize().expect("scan");
        let target_ref = snapshot.entries[0]
            .action_target_ref
            .as_deref()
            .expect("safe target");
        assert_eq!(
            inspector.request_action(target_ref, ProcessAction::GracefulStop),
            Ok(ProcessActionOutcome::Requested)
        );
        inspector.refresh().expect("new generation");
        assert_eq!(
            inspector.request_action(target_ref, ProcessAction::ForceStop),
            Err(ProcessActionError::IdentityUnavailable)
        );
    }

    #[test]
    fn simultaneous_initial_requests_share_one_provider_scan() {
        struct SlowPorts {
            entered: Barrier,
            release: Barrier,
            calls: AtomicUsize,
        }
        impl PortProvider for SlowPorts {
            fn listeners(&self) -> Result<PortScanResult, PortProviderError> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                self.entered.wait();
                self.release.wait();
                Ok(PortScanResult {
                    listeners: vec![],
                    completeness: PortScanCompleteness::Complete,
                })
            }
        }
        let ports = Arc::new(SlowPorts {
            entered: Barrier::new(2),
            release: Barrier::new(2),
            calls: AtomicUsize::new(0),
        });
        let inspector = Arc::new(RuntimeInspector::new(
            ports.clone(),
            Arc::new(FakeProcesses {
                responses: Mutex::new(HashMap::new()),
                calls: Mutex::new(vec![]),
            }),
            Arc::new(FakeController::default()),
            Arc::new(FakeCapabilities),
        ));
        let first = {
            let inspector = Arc::clone(&inspector);
            thread::spawn(move || inspector.snapshot_or_initialize())
        };
        while ports.calls.load(Ordering::SeqCst) == 0 {
            thread::yield_now();
        }
        let second = {
            let inspector = Arc::clone(&inspector);
            thread::spawn(move || inspector.snapshot_or_initialize())
        };
        ports.entered.wait();
        ports.release.wait();
        assert!(first.join().expect("first thread").is_ok());
        assert!(second.join().expect("second thread").is_ok());
        assert_eq!(ports.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn superseded_scan_is_discarded_and_overlapping_refreshes_coalesce() {
        struct FirstScanGate {
            calls: AtomicUsize,
            entered: Barrier,
            release: Barrier,
        }
        impl PortProvider for FirstScanGate {
            fn listeners(&self) -> Result<PortScanResult, PortProviderError> {
                if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
                    self.entered.wait();
                    self.release.wait();
                }
                Ok(PortScanResult {
                    listeners: vec![],
                    completeness: PortScanCompleteness::Complete,
                })
            }
        }

        let ports = Arc::new(FirstScanGate {
            calls: AtomicUsize::new(0),
            entered: Barrier::new(2),
            release: Barrier::new(2),
        });
        let inspector = Arc::new(RuntimeInspector::new(
            ports.clone(),
            Arc::new(FakeProcesses {
                responses: Mutex::new(HashMap::new()),
                calls: Mutex::new(vec![]),
            }),
            Arc::new(FakeController::default()),
            Arc::new(FakeCapabilities),
        ));

        let initial = {
            let inspector = Arc::clone(&inspector);
            thread::spawn(move || inspector.snapshot_or_initialize())
        };
        while ports.calls.load(Ordering::SeqCst) == 0 {
            thread::yield_now();
        }
        ports.entered.wait();

        let refresh_one = {
            let inspector = Arc::clone(&inspector);
            thread::spawn(move || inspector.refresh())
        };
        while inspector.lock_state().latest_requested < 2 {
            thread::yield_now();
        }
        let refresh_two = {
            let inspector = Arc::clone(&inspector);
            thread::spawn(move || inspector.refresh())
        };
        while inspector.lock_state().latest_requested < 3 {
            thread::yield_now();
        }
        ports.release.wait();

        let initial_result = initial
            .join()
            .expect("initial request")
            .expect("latest succeeds");
        let first_result = refresh_one
            .join()
            .expect("first refresh")
            .expect("latest succeeds");
        let second_result = refresh_two
            .join()
            .expect("second refresh")
            .expect("latest succeeds");
        assert_eq!(ports.calls.load(Ordering::SeqCst), 2);
        assert_eq!(initial_result.generation, 3);
        assert_eq!(first_result.generation, 3);
        assert_eq!(second_result.generation, 3);
        assert_eq!(
            inspector
                .current_snapshot()
                .expect("committed snapshot")
                .generation,
            3
        );
    }

    #[test]
    fn local_urls_use_observed_ip_and_ipv6_brackets() {
        assert_eq!(
            build_local_url(&listener(Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)), 80, None))
                .as_deref(),
            Some("http://127.0.0.1:80")
        );
        assert_eq!(
            build_local_url(&listener(Some(IpAddr::V6(Ipv6Addr::UNSPECIFIED)), 80, None))
                .as_deref(),
            Some("http://[::1]:80")
        );
        assert_eq!(
            build_local_url(&listener(Some(IpAddr::V6(Ipv6Addr::LOCALHOST)), 8080, None))
                .as_deref(),
            Some("http://[::1]:8080")
        );
        assert_eq!(build_local_url(&listener(None, 80, None)), None);
    }

    #[test]
    fn listener_reference_is_current_snapshot_scoped() {
        let (inspector, _, _) = inspector(
            complete(vec![listener(
                Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                8000,
                None,
            )]),
            [],
        );
        let first = inspector.snapshot_or_initialize().expect("snapshot");
        let old_ref = next_entry_ref(first.generation, 0);
        assert_eq!(
            inspector.listener_url(&old_ref),
            Ok("http://127.0.0.1:8000".to_owned())
        );
        inspector.refresh().expect("new snapshot");
        assert_eq!(
            inspector.listener_url(&old_ref),
            Err(ListenerUrlError::StaleReference)
        );
    }
}
