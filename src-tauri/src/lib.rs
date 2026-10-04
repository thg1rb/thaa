pub mod application;
mod commands;
pub mod domain;
pub mod platform;

use std::sync::Arc;

use application::runtime_inspection::RuntimeInspector;
use commands::runtime::RuntimeState;
use tauri::menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager};

#[cfg(target_os = "macos")]
fn runtime_inspector() -> Arc<RuntimeInspector> {
    use platform::macos::{
        port_provider::MacOSPortProvider,
        process_controller::{MacOSPlatformCapabilitiesProvider, MacOSProcessController},
        process_icon::MacOSProcessIconProvider,
        process_provider::MacOSProcessProvider,
    };
    Arc::new(RuntimeInspector::new_with_icons(
        Arc::new(MacOSPortProvider),
        Arc::new(MacOSProcessProvider),
        Arc::new(MacOSProcessIconProvider),
        Arc::new(MacOSProcessController),
        Arc::new(MacOSPlatformCapabilitiesProvider),
    ))
}

#[cfg(target_os = "windows")]
fn runtime_inspector() -> Arc<RuntimeInspector> {
    use platform::windows::{
        port_provider::WindowsPortProvider,
        process_controller::{WindowsPlatformCapabilitiesProvider, WindowsProcessController},
        process_icon::WindowsProcessIconProvider,
        process_provider::WindowsProcessProvider,
    };
    Arc::new(RuntimeInspector::new_with_icons(
        Arc::new(WindowsPortProvider),
        Arc::new(WindowsProcessProvider),
        Arc::new(WindowsProcessIconProvider),
        Arc::new(WindowsProcessController),
        Arc::new(WindowsPlatformCapabilitiesProvider),
    ))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn runtime_inspector() -> Arc<RuntimeInspector> {
    compile_error!("Thaa runtime providers currently support macOS and Windows only");
}

fn build_tray_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(app)?;
    if let Some(snapshot) = app
        .try_state::<RuntimeState>()
        .and_then(|state| state.0.current_snapshot())
    {
        for (index, entry) in snapshot.entries.iter().take(8).enumerate() {
            let name = entry
                .process
                .as_ref()
                .and_then(|result| result.as_ref().ok())
                .and_then(|info| match &info.identity.name {
                    crate::domain::metadata::FieldAvailability::Available(name) => {
                        Some(name.to_string_lossy().into_owned())
                    }
                    _ => None,
                })
                .unwrap_or_else(|| "Unknown process".to_owned());
            let name = safe_tray_text(&name, 42);
            let item = MenuItem::with_id(
                app,
                format!("runtime-{index}"),
                format!("TCP {} · {name}", entry.listener.local_port),
                false,
                None::<&str>,
            )?;
            menu.append(&item)?;
        }
        if snapshot.entries.is_empty() {
            menu.append(&MenuItem::new(
                app,
                "No listening TCP ports",
                false,
                None::<&str>,
            )?)?;
        } else if snapshot.entries.len() > 8 {
            menu.append(&MenuItem::new(
                app,
                format!("+ {} more", snapshot.entries.len() - 8),
                false,
                None::<&str>,
            )?)?;
        }
    } else {
        menu.append(&MenuItem::new(
            app,
            "Runtime list loading…",
            false,
            None::<&str>,
        )?)?;
    }
    let show = MenuItem::with_id(app, "show", "Show Thaa", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "Refresh runtimes", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Thaa", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    menu.append_items(&[&separator as &dyn IsMenuItem<R>, &refresh, &show, &quit])?;
    Ok(menu)
}

fn safe_tray_text(value: &str, max_chars: usize) -> String {
    value
        .chars()
        .map(|character| {
            let point = character as u32;
            if character.is_control()
                || (0x202a..=0x202e).contains(&point)
                || (0x2066..=0x2069).contains(&point)
            {
                '�'
            } else if character == '&' {
                '&'
            } else {
                character
            }
        })
        .take(max_chars)
        .collect::<String>()
        .replace('&', "&&")
}

pub(crate) fn update_tray_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id("main-tray"), build_tray_menu(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}

pub fn run() {
    tauri::Builder::default()
        .manage(RuntimeState(runtime_inspector()))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let menu = build_tray_menu(app.handle())?;
            let tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .tooltip("Thaa — Local Runtime Inspector");

            #[cfg(target_os = "macos")]
            let tray = tray
                .icon(tauri::include_image!("./icons/thaa-tray-macos.png"))
                .icon_as_template(true);

            #[cfg(target_os = "windows")]
            let tray = tray.icon(tauri::include_image!("./icons/thaa-tray-windows.png"));

            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            let tray = if let Some(icon) = app.default_window_icon() {
                tray.icon(icon.clone())
            } else {
                tray
            };
            tray.show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    "refresh" => refresh_from_tray(app),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    let activate = matches!(
                        event,
                        tauri::tray::TrayIconEvent::Click {
                            button: tauri::tray::MouseButton::Left,
                            button_state: tauri::tray::MouseButtonState::Up,
                            ..
                        } | tauri::tray::TrayIconEvent::DoubleClick { .. }
                    );
                    let menu_open = matches!(
                        event,
                        tauri::tray::TrayIconEvent::Click {
                            button_state: tauri::tray::MouseButtonState::Up,
                            ..
                        }
                    );
                    if activate || menu_open {
                        refresh_from_tray(tray.app_handle());
                    }
                    if activate {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::runtime::get_runtime_snapshot,
            commands::runtime::refresh_runtime_snapshot,
            commands::runtime::get_runtime_process_icons,
            commands::runtime::request_process_action,
            commands::runtime::open_listener_url,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Thaa");
}

fn refresh_from_tray<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let Some(state) = app.try_state::<RuntimeState>() else {
        return;
    };
    let inspector = Arc::clone(&state.0);
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if let Ok(snapshot) = inspector.refresh() {
            update_tray_menu(&app);
            let _ = app.emit(
                "runtime-snapshot-updated",
                commands::runtime::snapshot_for_event(&snapshot),
            );
        }
    });
}
