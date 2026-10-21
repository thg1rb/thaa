pub mod application;
mod commands;
pub mod domain;
pub mod platform;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use application::runtime_inspection::RuntimeInspector;
use commands::runtime::RuntimeState;
use tauri::menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri::{LogicalSize, PhysicalPosition};

#[cfg(target_os = "macos")]
fn runtime_inspector() -> Arc<RuntimeInspector> {
    use platform::macos::{
        port_provider::MacOSPortProvider,
        process_controller::{MacOSPlatformCapabilitiesProvider, MacOSProcessController},
        process_icon::MacOSProcessIconProvider,
        process_provider::MacOSProcessProvider,
    };
    Arc::new(RuntimeInspector::new_with_icons_and_git_context(
        Arc::new(MacOSPortProvider),
        Arc::new(MacOSProcessProvider),
        Arc::new(MacOSProcessIconProvider),
        Arc::new(platform::git_context::GitCliContextProvider::default()),
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
    Arc::new(RuntimeInspector::new_with_icons_and_git_context(
        Arc::new(WindowsPortProvider),
        Arc::new(WindowsProcessProvider),
        Arc::new(WindowsProcessIconProvider),
        Arc::new(platform::git_context::GitCliContextProvider::default()),
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
    let show = MenuItem::with_id(app, "show", "Show Thaa", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "Refresh runtimes", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Thaa", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    menu.append_items(&[&separator as &dyn IsMenuItem<R>, &refresh, &show, &quit])?;
    Ok(menu)
}

pub(crate) fn update_tray_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id("main-tray"), build_tray_menu(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}

pub fn run() {
    tauri::Builder::default()
        .manage(RuntimeState(
            runtime_inspector(),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        ))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let menu = build_tray_menu(app.handle())?;
            let tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .tooltip("Thaa — Local Runtime Inspector");

            #[cfg(target_os = "macos")]
            let tray = tray
                .icon(tauri::include_image!(
                    "./icons/derived/thaa-tray-template-macos.png"
                ))
                .icon_as_template(true);

            #[cfg(target_os = "windows")]
            let tray = tray.icon(tauri::include_image!(
                "./icons/derived/thaa-tray-windows.png"
            ));

            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            let tray = if let Some(icon) = app.default_window_icon() {
                tray.icon(icon.clone())
            } else {
                tray
            };
            tray.show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(popover) = app.get_webview_window("tray-popover") {
                            let _ = popover.hide();
                        }
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
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        position,
                        rect,
                        ..
                    } = event
                    {
                        let app = tray.app_handle().clone();
                        let callback_app = app.clone();
                        let _ = app.run_on_main_thread(move || {
                            toggle_tray_popover(&callback_app, rect, position)
                        });
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "tray-popover"
                && matches!(event, tauri::WindowEvent::Focused(false))
            {
                let _ = window.hide();
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "tray-popover" {
                    api.prevent_close();
                    let _ = window.hide();
                    return;
                }
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
            commands::runtime::reveal_project_folder,
            commands::runtime::open_project_in_terminal,
            commands::runtime::open_project_in_editor,
            commands::runtime::get_tray_runtime_snapshot,
            commands::runtime::navigate_to_runtime_entry,
            commands::runtime::show_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Thaa");
}

fn refresh_from_tray<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let Some(state) = app.try_state::<RuntimeState>() else {
        return;
    };
    let inspector = Arc::clone(&state.0);
    let refresh_failed = Arc::clone(&state.1);
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || match inspector.refresh() {
        Ok(snapshot) => {
            refresh_failed.store(false, Ordering::Release);
            update_tray_menu(&app);
            let _ = app.emit(
                "runtime-snapshot-updated",
                commands::runtime::snapshot_for_event(&snapshot),
            );
        }
        Err(_) => {
            refresh_failed.store(true, Ordering::Release);
            let _ = app.emit_to("tray-popover", "runtime-snapshot-stale", true);
        }
    });
}

fn tray_popover<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<WebviewWindow<R>> {
    if let Some(window) = app.get_webview_window("tray-popover") {
        return Ok(window);
    }

    WebviewWindowBuilder::new(app, "tray-popover", WebviewUrl::App("index.html".into()))
        .title("Thaa Runtime Overview")
        .inner_size(400.0, 620.0)
        .min_inner_size(320.0, 200.0)
        .max_inner_size(480.0, 900.0)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .shadow(true)
        .build()
}

fn toggle_tray_popover<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    anchor: tauri::Rect,
    click_position: PhysicalPosition<f64>,
) {
    let Ok(window) = tray_popover(app) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }

    let monitor = app
        .monitor_from_point(click_position.x, click_position.y)
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let area = monitor.work_area();
        let scale = monitor.scale_factor();
        let anchor_position = anchor.position.to_physical::<f64>(scale);
        let anchor_size = anchor.size.to_physical::<f64>(scale);
        let (x, y, width, height) = calculate_tray_popover_position(
            PopoverAnchor {
                x: anchor_position.x,
                y: anchor_position.y,
                width: anchor_size.width,
                height: anchor_size.height,
            },
            WorkArea {
                x: area.position.x,
                y: area.position.y,
                width: area.size.width,
                height: area.size.height,
            },
            scale,
        );
        let _ = window.set_size(LogicalSize::new(width, height));
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
    let _ = window.show();
    let _ = window.set_focus();
}

#[derive(Clone, Copy)]
struct PopoverAnchor {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Clone, Copy)]
struct WorkArea {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

fn calculate_tray_popover_position(
    anchor: PopoverAnchor,
    area: WorkArea,
    scale: f64,
) -> (i32, i32, f64, f64) {
    const WIDTH: f64 = 400.0;
    const MAX_HEIGHT: f64 = 620.0;
    const MARGIN: f64 = 8.0;
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let work_x = area.x;
    let work_y = area.y;
    let work_width = f64::from(area.width);
    let work_height = f64::from(area.height);
    let logical_width = WIDTH.min((work_width / scale - MARGIN * 2.0).max(320.0));
    let logical_height = MAX_HEIGHT.min((work_height / scale - MARGIN * 2.0).max(200.0));
    let width = logical_width * scale;
    let height = logical_height * scale;
    let work_right = f64::from(work_x) + work_width;
    let work_bottom = f64::from(work_y) + work_height;

    let distances = [
        (anchor.y - f64::from(work_y)).abs(),
        (work_bottom - (anchor.y + anchor.height)).abs(),
        (anchor.x - f64::from(work_x)).abs(),
        (work_right - (anchor.x + anchor.width)).abs(),
    ];
    let nearest = distances
        .iter()
        .enumerate()
        .min_by(|left, right| left.1.total_cmp(right.1))
        .map(|(index, _)| index)
        .unwrap_or(1);

    let (raw_x, raw_y) = match nearest {
        0 => (anchor.x + anchor.width - width, anchor.y + anchor.height),
        1 => (anchor.x + anchor.width - width, anchor.y - height),
        2 => (anchor.x + anchor.width, anchor.y),
        _ => (anchor.x - width, anchor.y),
    };
    let max_x = (work_right - width - MARGIN).max(f64::from(work_x) + MARGIN);
    let max_y = (work_bottom - height - MARGIN).max(f64::from(work_y) + MARGIN);
    let x = raw_x.clamp(f64::from(work_x) + MARGIN, max_x).round() as i32;
    let y = raw_y.clamp(f64::from(work_y) + MARGIN, max_y).round() as i32;
    (x, y, logical_width, logical_height)
}

#[cfg(test)]
mod tests {
    use super::{calculate_tray_popover_position, PopoverAnchor, WorkArea};

    #[test]
    fn tray_popover_position_stays_within_work_area_at_bottom_edge() {
        let (x, y, width, height) = calculate_tray_popover_position(
            PopoverAnchor {
                x: 1800.0,
                y: 1020.0,
                width: 30.0,
                height: 30.0,
            },
            WorkArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            1.0,
        );
        assert!(x >= 8);
        assert!(x + (width as i32) <= 1912);
        assert!(y >= 8);
        assert!(y + height as i32 <= 1072);
        assert_eq!(y, 400);
    }

    #[test]
    fn tray_popover_respects_display_scale_and_nonzero_monitor_origin() {
        let (x, y, width, height) = calculate_tray_popover_position(
            PopoverAnchor {
                x: 3000.0,
                y: 20.0,
                width: 60.0,
                height: 40.0,
            },
            WorkArea {
                x: 2560,
                y: 0,
                width: 1920,
                height: 1080,
            },
            2.0,
        );
        assert!(x >= 2568);
        assert!(x + (width * 2.0) as i32 <= 4472);
        assert!(y >= 8);
        assert!(y + (height * 2.0) as i32 <= 1072);
        assert_eq!(width, 400.0);
        assert_eq!(height, 524.0);
    }
}
