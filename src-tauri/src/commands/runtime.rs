//! Deliberate transport DTOs and thin runtime command boundary.

use std::sync::Arc;
use std::time::UNIX_EPOCH;

use base64::Engine;
use serde::Serialize;
use tauri::State;
use tauri_plugin_opener::OpenerExt;

use crate::application::runtime_inspection::{
    ListenerUrlError, RuntimeInspector, RuntimeScanError, RuntimeSnapshot,
};
use crate::domain::capabilities::CapabilitySupport;
use crate::domain::metadata::{FieldAvailability, UnavailableReason};
use crate::domain::network::BindingScope;
use crate::domain::process::ProcessInfo;
use crate::domain::process_action::{ProcessAction, ProcessActionError, ProcessActionOutcome};
use crate::domain::process_provider::ProcessProviderErrorKind;

#[derive(Clone)]
pub struct RuntimeState(pub Arc<RuntimeInspector>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshotDto {
    pub generation: u64,
    pub observed_at_unix_ms: u128,
    pub completeness: ScanCompletenessDto,
    pub capabilities: CapabilitiesDto,
    pub entries: Vec<RuntimeEntryDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessIconAssetDto {
    pub reference: String,
    pub png_base64: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "state", content = "reason")]
pub enum ScanCompletenessDto {
    Complete,
    Partial(String),
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesDto {
    pub graceful_stop: bool,
    pub force_stop: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEntryDto {
    pub entry_ref: String,
    pub protocol: &'static str,
    pub local_address: Option<String>,
    pub port: u16,
    pub binding: BindingDto,
    pub process_id: Option<u32>,
    pub process: ProcessDetailsDto,
    pub project_root: Option<String>,
    pub local_url: Option<String>,
    pub action_target_ref: Option<String>,
    pub process_icon_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BindingDto {
    LoopbackOnly,
    PotentiallyReachable,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "state", content = "details")]
pub enum ProcessDetailsDto {
    NoOwner,
    Available(ProcessInfoDto),
    Unavailable { process_id: u32, reason: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfoDto {
    pub process_id: u32,
    pub name: FieldDto<String>,
    pub executable_path: FieldDto<String>,
    pub start_time_unix_ms: FieldDto<u128>,
    pub command_arguments: FieldDto<Vec<String>>,
    pub working_directory: FieldDto<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "state", content = "value")]
pub enum FieldDto<T> {
    Available(T),
    Unavailable(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "state", content = "message")]
pub enum RuntimeCommandErrorDto {
    PermissionDenied(String),
    Unsupported(String),
    ProviderFailure(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum ActionResultDto {
    Requested,
    AlreadyExited,
    Refused { reason: String },
    Failed { reason: String },
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionDto {
    GracefulStop,
    ForceStop,
}

impl From<ActionDto> for ProcessAction {
    fn from(value: ActionDto) -> Self {
        match value {
            ActionDto::GracefulStop => Self::GracefulStop,
            ActionDto::ForceStop => Self::ForceStop,
        }
    }
}

#[tauri::command]
pub async fn get_runtime_snapshot(
    app: tauri::AppHandle,
    state: State<'_, RuntimeState>,
) -> Result<RuntimeSnapshotDto, RuntimeCommandErrorDto> {
    let inspector = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || {
        inspector.snapshot_or_initialize().map(|snapshot| {
            crate::update_tray_menu(&app);
            snapshot_dto(&snapshot)
        })
    })
    .await
    .map_err(|_| RuntimeCommandErrorDto::ProviderFailure("Runtime inspection failed.".into()))?
    .map_err(scan_error_dto)
}

#[tauri::command]
pub async fn refresh_runtime_snapshot(
    app: tauri::AppHandle,
    state: State<'_, RuntimeState>,
) -> Result<RuntimeSnapshotDto, RuntimeCommandErrorDto> {
    let inspector = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || {
        inspector.refresh().map(|snapshot| {
            crate::update_tray_menu(&app);
            snapshot_dto(&snapshot)
        })
    })
    .await
    .map_err(|_| RuntimeCommandErrorDto::ProviderFailure("Runtime inspection failed.".into()))?
    .map_err(scan_error_dto)
}

/// Resolves optional presentation assets after the runtime rows have returned.
/// The generation is validated against the current backend snapshot; callers
/// cannot submit process IDs or filesystem paths for icon extraction.
#[tauri::command]
pub async fn get_runtime_process_icons(
    state: State<'_, RuntimeState>,
    generation: u64,
) -> Result<Vec<ProcessIconAssetDto>, String> {
    let inspector = Arc::clone(&state.0);
    let assets = tauri::async_runtime::spawn_blocking(move || {
        process_icon_assets_dto(inspector.process_icons_for_snapshot(generation))
    })
    .await
    .unwrap_or_default();
    Ok(assets)
}

#[tauri::command]
pub async fn request_process_action(
    state: State<'_, RuntimeState>,
    action_target_ref: String,
    action: ActionDto,
) -> Result<ActionResultDto, String> {
    if action_target_ref.len() > 96 || !valid_reference(&action_target_ref) {
        return Ok(ActionResultDto::Refused {
            reason: "This runtime entry is no longer available. Refresh and try again.".into(),
        });
    }
    let inspector = Arc::clone(&state.0);
    match tauri::async_runtime::spawn_blocking(move || {
        inspector.request_action(&action_target_ref, action.into())
    })
    .await
    {
        Ok(Ok(ProcessActionOutcome::Requested)) => Ok(ActionResultDto::Requested),
        Ok(Ok(ProcessActionOutcome::AlreadyExited)) => Ok(ActionResultDto::AlreadyExited),
        Ok(Err(error)) => Ok(action_error_dto(error)),
        Err(_) => Ok(ActionResultDto::Failed {
            reason: "The process action could not be completed.".into(),
        }),
    }
}

#[tauri::command]
pub async fn open_listener_url(
    app: tauri::AppHandle,
    state: State<'_, RuntimeState>,
    entry_ref: String,
) -> Result<(), String> {
    if entry_ref.len() > 96 || !entry_ref.starts_with("entry-") {
        return Err("The listener is no longer in the current scan.".into());
    }
    let inspector = Arc::clone(&state.0);
    let url = tauri::async_runtime::spawn_blocking(move || inspector.listener_url(&entry_ref))
        .await
        .map_err(|_| "The listener URL is no longer available.".to_owned())?
        .map_err(listener_url_error)?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|_| "Thaa could not open this local address.".to_owned())
}

fn valid_reference(value: &str) -> bool {
    let Some((kind, rest)) = value.split_once('-') else {
        return false;
    };
    let Some((generation, index)) = rest.split_once('-') else {
        return false;
    };
    matches!(kind, "entry" | "target")
        && !generation.is_empty()
        && generation.len() <= 16
        && !index.is_empty()
        && index.len() <= 16
        && generation.bytes().all(|byte| byte.is_ascii_hexdigit())
        && index.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn snapshot_dto(snapshot: &RuntimeSnapshot) -> RuntimeSnapshotDto {
    RuntimeSnapshotDto {
        generation: snapshot.generation,
        observed_at_unix_ms: snapshot
            .observed_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        completeness: match snapshot.completeness {
            crate::domain::port_provider::PortScanCompleteness::Complete => {
                ScanCompletenessDto::Complete
            }
            crate::domain::port_provider::PortScanCompleteness::Partial(error) => {
                ScanCompletenessDto::Partial(error.to_string())
            }
        },
        capabilities: CapabilitiesDto {
            graceful_stop: snapshot.capabilities.graceful_stop == CapabilitySupport::Supported,
            force_stop: snapshot.capabilities.force_stop == CapabilitySupport::Supported,
        },
        entries: snapshot
            .entries
            .iter()
            .map(|entry| {
                let binding = match entry.listener.binding_scope() {
                    BindingScope::LoopbackOnly => BindingDto::LoopbackOnly,
                    BindingScope::PotentiallyReachable => BindingDto::PotentiallyReachable,
                    BindingScope::Unknown => BindingDto::Unknown,
                };
                let process_id = entry.listener.owner_pid.map(|pid| pid.get());
                RuntimeEntryDto {
                    entry_ref: entry.entry_ref.clone(),
                    protocol: "tcp",
                    local_address: entry
                        .listener
                        .local_address
                        .map(|address| address.to_string()),
                    port: entry.listener.local_port.get(),
                    binding,
                    process_id,
                    process: match (&entry.process, process_id) {
                        (None, None) => ProcessDetailsDto::NoOwner,
                        (Some(Ok(info)), Some(_)) => {
                            ProcessDetailsDto::Available(process_info_dto(info))
                        }
                        (Some(Err(error)), Some(pid)) => ProcessDetailsDto::Unavailable {
                            process_id: pid,
                            reason: process_error_text(error.kind()),
                        },
                        _ => ProcessDetailsDto::Unavailable {
                            process_id: process_id.unwrap_or_default(),
                            reason: "Process details are unavailable.".into(),
                        },
                    },
                    project_root: entry
                        .project_root
                        .as_ref()
                        .map(|path| path.to_string_lossy().into_owned()),
                    local_url: crate::application::runtime_inspection::build_local_url(
                        &entry.listener,
                    ),
                    action_target_ref: entry.action_target_ref.clone(),
                    process_icon_ref: entry.process_icon_ref.clone(),
                }
            })
            .collect(),
    }
}

fn process_icon_assets_dto(
    icons: Vec<crate::application::process_icons::ProcessIconAsset>,
) -> Vec<ProcessIconAssetDto> {
    icons
        .into_iter()
        .map(|icon| ProcessIconAssetDto {
            reference: icon.reference,
            png_base64: base64::engine::general_purpose::STANDARD.encode(icon.png),
        })
        .collect()
}

pub fn snapshot_for_event(snapshot: &RuntimeSnapshot) -> RuntimeSnapshotDto {
    snapshot_dto(snapshot)
}

fn process_info_dto(info: &ProcessInfo) -> ProcessInfoDto {
    ProcessInfoDto {
        process_id: info.identity.pid.get(),
        name: map_field(&info.identity.name, |value| {
            value.to_string_lossy().into_owned()
        }),
        executable_path: map_field(&info.identity.executable_path, |value| {
            value.to_string_lossy().into_owned()
        }),
        start_time_unix_ms: map_field(&info.identity.start_time, |value| {
            value
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        }),
        command_arguments: map_field(&info.command_arguments, |values| {
            values
                .iter()
                .map(|value| value.to_string_lossy().into_owned())
                .collect()
        }),
        working_directory: map_field(&info.working_directory, |value| {
            value.to_string_lossy().into_owned()
        }),
    }
}

fn map_field<T, U>(value: &FieldAvailability<T>, map: impl FnOnce(&T) -> U) -> FieldDto<U> {
    match value {
        FieldAvailability::Available(value) => FieldDto::Available(map(value)),
        FieldAvailability::Unavailable(reason) => {
            FieldDto::Unavailable(unavailable_text(*reason).into())
        }
    }
}

fn unavailable_text(reason: UnavailableReason) -> &'static str {
    match reason {
        UnavailableReason::PermissionDenied => "Permission denied",
        UnavailableReason::Unsupported => "Unsupported on this platform",
        UnavailableReason::Inaccessible => "Unavailable for this process",
        UnavailableReason::ProviderLimitation => "Not provided by the operating system",
    }
}

fn process_error_text(error: ProcessProviderErrorKind) -> String {
    match error {
        ProcessProviderErrorKind::ProcessDisappeared => "Process exited or disappeared".into(),
        ProcessProviderErrorKind::PermissionDenied => "Permission denied".into(),
        ProcessProviderErrorKind::Unsupported | ProcessProviderErrorKind::MechanismUnavailable => {
            "Process details are unsupported".into()
        }
        _ => "Process details are unavailable".into(),
    }
}

fn scan_error_dto(error: RuntimeScanError) -> RuntimeCommandErrorDto {
    match error {
        RuntimeScanError::PortProvider(
            crate::domain::port_provider::PortProviderErrorKind::PermissionDenied,
        ) => RuntimeCommandErrorDto::PermissionDenied(
            "Thaa could not inspect listening ports due to a permission restriction.".into(),
        ),
        RuntimeScanError::PortProvider(
            crate::domain::port_provider::PortProviderErrorKind::Unsupported
            | crate::domain::port_provider::PortProviderErrorKind::MechanismUnavailable,
        ) => RuntimeCommandErrorDto::Unsupported(
            "Listening-port inspection is unavailable on this system.".into(),
        ),
        _ => RuntimeCommandErrorDto::ProviderFailure(
            "Thaa could not inspect listening ports. Try refreshing.".into(),
        ),
    }
}

fn action_error_dto(error: ProcessActionError) -> ActionResultDto {
    match error {
        ProcessActionError::IdentityMismatch => ActionResultDto::Refused {
            reason: "The process changed since this entry was loaded. Refresh and try again."
                .into(),
        },
        ProcessActionError::IdentityUnavailable | ProcessActionError::InvalidTarget => {
            ActionResultDto::Refused {
                reason: "Process identity could not be verified. Refresh and try again.".into(),
            }
        }
        ProcessActionError::Unsupported => ActionResultDto::Refused {
            reason: "This action is unavailable on this platform.".into(),
        },
        ProcessActionError::PermissionDenied => ActionResultDto::Failed {
            reason: "Thaa does not have permission to perform this action.".into(),
        },
        _ => ActionResultDto::Failed {
            reason: "The process action could not be completed.".into(),
        },
    }
}

fn listener_url_error(error: ListenerUrlError) -> String {
    match error {
        ListenerUrlError::StaleReference => "The listener is no longer in the current scan.".into(),
        ListenerUrlError::AddressUnavailable => {
            "This listener has no safe local address to open.".into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{process_icon_assets_dto, snapshot_dto, valid_reference, ActionDto};
    use crate::application::process_icons::ProcessIconAsset;
    use crate::application::runtime_inspection::{RuntimeEntry, RuntimeSnapshot};
    use crate::domain::capabilities::{CapabilitySupport, PlatformCapabilities};
    use crate::domain::network::{NetworkListener, NetworkProtocol};
    use crate::domain::port_provider::PortScanCompleteness;
    use base64::Engine;
    use std::net::{IpAddr, Ipv4Addr};
    use std::num::NonZeroU16;
    use std::path::PathBuf;
    use std::time::UNIX_EPOCH;

    #[test]
    fn action_references_are_bounded_and_structured() {
        assert!(valid_reference("target-12-a"));
        assert!(!valid_reference("target-12-a/../../pid"));
        assert!(!valid_reference("1234"));
        assert!(!valid_reference(&format!("target-{}-1", "f".repeat(17))));
    }

    #[test]
    fn action_enum_uses_closed_camel_case_values() {
        assert!(matches!(
            serde_json::from_str::<ActionDto>("\"forceStop\""),
            Ok(ActionDto::ForceStop)
        ));
        assert!(serde_json::from_str::<ActionDto>("\"terminateAnything\"").is_err());
    }

    #[test]
    fn snapshot_dto_uses_explicit_camel_case_shape_and_capabilities() {
        let snapshot = RuntimeSnapshot {
            generation: 3,
            observed_at: UNIX_EPOCH,
            completeness: PortScanCompleteness::Complete,
            capabilities: PlatformCapabilities {
                command_line: CapabilitySupport::Unsupported,
                working_directory: CapabilitySupport::Supported,
                graceful_stop: CapabilitySupport::Unsupported,
                force_stop: CapabilitySupport::Supported,
            },
            entries: vec![RuntimeEntry {
                entry_ref: "entry-3-0".into(),
                listener: NetworkListener {
                    protocol: NetworkProtocol::Tcp,
                    local_address: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                    local_port: NonZeroU16::new(80).expect("fixture port"),
                    owner_pid: None,
                },
                process: None,
                project_root: Some(PathBuf::from("/sample/project")),
                action_target_ref: None,
                process_icon_ref: None,
            }],
        };
        let json = serde_json::to_value(snapshot_dto(&snapshot)).expect("serializes");
        assert_eq!(json["generation"], 3);
        assert_eq!(json["entries"][0]["entryRef"], "entry-3-0");
        assert_eq!(json["entries"][0]["binding"], "loopbackOnly");
        assert_eq!(json["entries"][0]["process"]["state"], "noOwner");
        assert_eq!(json["entries"][0]["projectRoot"], "/sample/project");
        assert_eq!(json["capabilities"]["gracefulStop"], false);
        assert_eq!(json["capabilities"]["forceStop"], true);
        assert_eq!(
            json["entries"][0]["processIconRef"],
            serde_json::Value::Null
        );
        assert_eq!(json["entries"][0]["localUrl"], "http://127.0.0.1:80");
    }

    #[test]
    fn icon_asset_dto_encodes_presentation_png_separately() {
        let assets = process_icon_assets_dto(vec![ProcessIconAsset {
            reference: "icon-3-0".into(),
            png: b"png bytes".to_vec(),
        }]);
        assert_eq!(assets[0].reference, "icon-3-0");
        assert_eq!(
            assets[0].png_base64,
            base64::engine::general_purpose::STANDARD.encode(b"png bytes")
        );
    }
}
