pub mod application;
mod commands;
pub mod domain;
pub mod platform;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use application::runtime_inspection::RuntimeInspector;
use commands::runtime::RuntimeState;
use tauri::menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
#[cfg(any(target_os = "macos", test))]
use tauri::LogicalPosition;
#[cfg(target_os = "macos")]
use tauri::LogicalSize;
use tauri::PhysicalPosition;
#[cfg(not(target_os = "macos"))]
use tauri::PhysicalSize;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TrayPopoverTransition {
    revision: u64,
    open: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrayPopoverVisibilityAction {
    ShowPopover,
    HidePopover,
}

impl TrayPopoverTransition {
    fn visibility_action(self) -> TrayPopoverVisibilityAction {
        if self.open {
            TrayPopoverVisibilityAction::ShowPopover
        } else {
            TrayPopoverVisibilityAction::HidePopover
        }
    }
}

#[derive(Default)]
struct TrayPopoverStateInner {
    revision: u64,
    open: bool,
    focused_revision: Option<u64>,
}

#[derive(Clone, Default)]
pub(crate) struct TrayPopoverState(Arc<Mutex<TrayPopoverStateInner>>);

impl TrayPopoverState {
    fn toggle(&self) -> TrayPopoverTransition {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        state.revision = state.revision.wrapping_add(1);
        state.open = !state.open;
        state.focused_revision = None;
        TrayPopoverTransition {
            revision: state.revision,
            open: state.open,
        }
    }

    #[cfg(test)]
    fn revision(&self) -> u64 {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .revision
    }

    fn mark_focused(&self) {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if state.open {
            state.focused_revision = Some(state.revision);
        }
    }

    fn focused_revision(&self) -> Option<u64> {
        let state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if state.open && state.focused_revision == Some(state.revision) {
            Some(state.revision)
        } else {
            None
        }
    }

    fn dismiss_if_current(&self, revision: u64) -> bool {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if state.revision != revision || !state.open || state.focused_revision != Some(revision) {
            return false;
        }
        state.revision = state.revision.wrapping_add(1);
        state.open = false;
        state.focused_revision = None;
        true
    }

    fn dismiss(&self) {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        state.revision = state.revision.wrapping_add(1);
        state.open = false;
        state.focused_revision = None;
    }

    #[cfg(test)]
    fn is_open(&self) -> bool {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .open
    }

    fn cancel_open_if_current(&self, revision: u64) {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if state.revision == revision && state.open {
            state.revision = state.revision.wrapping_add(1);
            state.open = false;
            state.focused_revision = None;
        }
    }
}

pub(crate) fn close_tray_popover_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(state) = app.try_state::<TrayPopoverState>() {
        state.dismiss();
    }
    if let Some(popover) = app.get_webview_window("tray-popover") {
        let _ = popover.hide();
    }
}

fn schedule_tray_popover_dismissal<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: TrayPopoverState,
) {
    let Some(revision) = state.focused_revision() else {
        return;
    };
    let Some(popover) = app.get_webview_window("tray-popover") else {
        return;
    };

    let _ = app.run_on_main_thread(move || {
        if state.dismiss_if_current(revision) {
            let _ = popover.hide();
        }
    });
}

#[cfg(target_os = "macos")]
struct MacosApplicationDeactivationContext {
    app: tauri::AppHandle<tauri::Wry>,
    state: TrayPopoverState,
}

#[cfg(target_os = "macos")]
struct MacosApplicationDeactivationObserver {
    observer: *mut std::ffi::c_void,
    context: *mut MacosApplicationDeactivationContext,
}

#[cfg(target_os = "macos")]
impl Drop for MacosApplicationDeactivationObserver {
    fn drop(&mut self) {
        unsafe extern "C" {
            fn thaa_macos_remove_application_deactivation_observer(observer: *mut std::ffi::c_void);
        }

        // SAFETY: The observer was returned by the matching registration
        // function and remains registered until this guard is dropped.
        unsafe { thaa_macos_remove_application_deactivation_observer(self.observer) };
        // SAFETY: `context` was created with Box::into_raw during registration,
        // and observer removal prevents any future callback from borrowing it.
        unsafe { drop(Box::from_raw(self.context)) };
    }
}

#[cfg(target_os = "macos")]
fn observe_application_deactivation(
    app: tauri::AppHandle<tauri::Wry>,
    state: TrayPopoverState,
) -> Option<MacosApplicationDeactivationObserver> {
    unsafe extern "C" {
        fn thaa_macos_observe_application_deactivation(
            callback: unsafe extern "C" fn(*mut std::ffi::c_void),
            context: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
    }

    let context = Box::into_raw(Box::new(MacosApplicationDeactivationContext { app, state }));
    // SAFETY: `context` remains owned by the observer guard until its observer
    // is removed. The callback is synchronous with AppKit notification delivery.
    let observer = unsafe {
        thaa_macos_observe_application_deactivation(handle_application_deactivation, context.cast())
    };
    if observer.is_null() {
        // SAFETY: Registration failed, so AppKit retained no callback/context.
        unsafe { drop(Box::from_raw(context)) };
        None
    } else {
        Some(MacosApplicationDeactivationObserver { observer, context })
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn handle_application_deactivation(context: *mut std::ffi::c_void) {
    // SAFETY: The pointer is the live Box allocated by
    // `observe_application_deactivation`; the observer is removed before it is freed.
    let Some(context) = (unsafe {
        context
            .cast::<MacosApplicationDeactivationContext>()
            .as_ref()
    }) else {
        return;
    };
    schedule_tray_popover_dismissal(&context.app, context.state.clone());
}

fn is_primary_tray_activation(
    button: tauri::tray::MouseButton,
    button_state: Option<tauri::tray::MouseButtonState>,
    double_click: bool,
) -> bool {
    button == tauri::tray::MouseButton::Left
        && (double_click || button_state == Some(tauri::tray::MouseButtonState::Down))
}

pub(crate) fn update_tray_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id("main-tray"), build_tray_menu(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}

pub fn run() {
    let app = tauri::Builder::default()
        .manage(RuntimeState(
            runtime_inspector(),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        ))
        .manage(TrayPopoverState::default())
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
                        close_tray_popover_window(app);
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
                    let (button, button_state, double_click, position, rect) = match event {
                        tauri::tray::TrayIconEvent::Click {
                            button,
                            button_state,
                            position,
                            rect,
                            ..
                        } => (button, Some(button_state), false, position, rect),
                        tauri::tray::TrayIconEvent::DoubleClick {
                            button,
                            position,
                            rect,
                            ..
                        } => (button, None, true, position, rect),
                        _ => return,
                    };
                    if !is_primary_tray_activation(button, button_state, double_click) {
                        return;
                    }

                    // Tauri delivers tray callbacks on its event-loop thread.
                    // Apply the transition there so a queued main-thread task
                    // cannot run after focus-loss has already hidden the window.
                    let app = tray.app_handle();
                    let transition = app.state::<TrayPopoverState>().toggle();
                    #[cfg(target_os = "macos")]
                    let clicked_display = macos_cursor_display_bounds();
                    #[cfg(not(target_os = "macos"))]
                    let clicked_display = None;
                    toggle_tray_popover(app, rect, position, clicked_display, transition);
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "tray-popover" {
                let state = window
                    .app_handle()
                    .state::<TrayPopoverState>()
                    .inner()
                    .clone();
                match event {
                    tauri::WindowEvent::Focused(true) => state.mark_focused(),
                    tauri::WindowEvent::Focused(false) => {
                        schedule_tray_popover_dismissal(window.app_handle(), state)
                    }
                    _ => {}
                }
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "tray-popover" {
                    api.prevent_close();
                    close_tray_popover_window(window.app_handle());
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
            commands::runtime::dismiss_tray_popover,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Thaa");

    #[cfg(target_os = "macos")]
    let mut deactivation_observer = observe_application_deactivation(
        app.handle().clone(),
        app.state::<TrayPopoverState>().inner().clone(),
    );

    #[cfg(target_os = "macos")]
    app.run(move |_, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            deactivation_observer.take();
        }
    });

    #[cfg(not(target_os = "macos"))]
    app.run(|_, _| {});
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
    clicked_display: Option<LogicalMonitorBounds>,
    transition: TrayPopoverTransition,
) {
    if transition.visibility_action() == TrayPopoverVisibilityAction::HidePopover {
        if let Some(window) = app.get_webview_window("tray-popover") {
            let _ = window.hide();
        }
        return;
    }

    let Ok(window) = tray_popover(app) else {
        app.state::<TrayPopoverState>()
            .cancel_open_if_current(transition.revision);
        return;
    };
    // Tray toggles own only the Popover. Main Window visibility is controlled
    // by explicit Show Thaa and validated process-navigation actions.
    let monitors = app.available_monitors().unwrap_or_default();
    let physical_position = anchor.position.to_physical::<f64>(1.0);
    let physical_size = anchor.size.to_physical::<f64>(1.0);
    let tray_anchor = PopoverAnchor {
        x: physical_position.x,
        y: physical_position.y,
        width: physical_size.width,
        height: physical_size.height,
    };
    let monitor =
        monitor_for_tray_click(app, &monitors, tray_anchor, click_position, clicked_display)
            .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let area = monitor.work_area();
        let scale = monitor.scale_factor();
        let anchor = placement_anchor(
            tray_anchor,
            click_position,
            WorkArea {
                x: area.position.x,
                y: area.position.y,
                width: area.size.width,
                height: area.size.height,
            },
        );
        let (x, y, width, height) = calculate_tray_popover_position(
            anchor,
            WorkArea {
                x: area.position.x,
                y: area.position.y,
                width: area.size.width,
                height: area.size.height,
            },
            scale,
        );
        // Set size and position while hidden so a monitor switch never flashes
        // the Popover at its previous display location. On macOS, Tao converts
        // physical setters using the window's current scale factor, which may
        // still belong to the previous display. Logical setters avoid that
        // old-monitor scale conversion; x/y are in the target display's logical
        // coordinate space, including negative virtual-desktop origins.
        #[cfg(target_os = "macos")]
        {
            let logical_position = physical_position_to_logical(x, y, scale);
            let _ = window.set_size(LogicalSize::new(width, height));
            let _ = window.set_position(logical_position);

            // Tao queues its AppKit size and position changes on the main
            // dispatch queue. Queue the reveal after them on that same serial
            // queue so the Popover cannot flash at its previous display.
            unsafe extern "C" {
                fn thaa_macos_order_window_front_after_pending_updates(
                    window: *mut std::ffi::c_void,
                );
            }
            if let Ok(native_window) = window.ns_window() {
                unsafe { thaa_macos_order_window_front_after_pending_updates(native_window) };
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let physical_size = PhysicalSize::new(
                (width * scale).round().max(1.0) as u32,
                (height * scale).round().max(1.0) as u32,
            );
            let _ = window.set_size(physical_size);
            let _ = window.set_position(PhysicalPosition::new(x, y));
            let _ = window.show();
            let _ = window.set_focus();
        }
    } else {
        app.state::<TrayPopoverState>()
            .cancel_open_if_current(transition.revision);
    }
}

fn anchor_is_valid(anchor: PopoverAnchor) -> bool {
    anchor.x.is_finite()
        && anchor.y.is_finite()
        && anchor.width.is_finite()
        && anchor.height.is_finite()
        && anchor.width > 0.0
        && anchor.height > 0.0
}

fn placement_anchor(
    anchor: PopoverAnchor,
    click_position: PhysicalPosition<f64>,
    area: WorkArea,
) -> PopoverAnchor {
    if anchor_is_valid(anchor) {
        return anchor;
    }
    if click_position.x.is_finite() && click_position.y.is_finite() {
        return PopoverAnchor {
            x: click_position.x,
            y: click_position.y,
            width: 1.0,
            height: 1.0,
        };
    }
    PopoverAnchor {
        x: f64::from(area.x) + f64::from(area.width) / 2.0,
        y: f64::from(area.y) + f64::from(area.height) / 2.0,
        width: 1.0,
        height: 1.0,
    }
}

#[cfg(any(target_os = "macos", test))]
fn physical_position_to_logical(x: i32, y: i32, scale: f64) -> LogicalPosition<f64> {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    LogicalPosition::new(f64::from(x) / scale, f64::from(y) / scale)
}

fn same_monitor(left: &tauri::Monitor, right: &tauri::Monitor) -> bool {
    left.position() == right.position()
        && left.size() == right.size()
        && left.scale_factor() == right.scale_factor()
        && left.name() == right.name()
}

fn monitor_for_tray_click<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    monitors: &[tauri::Monitor],
    anchor: PopoverAnchor,
    click_position: PhysicalPosition<f64>,
    clicked_display: Option<LogicalMonitorBounds>,
) -> Option<tauri::Monitor> {
    #[cfg(target_os = "macos")]
    {
        if let Some(bounds) = clicked_display {
            if let Some(monitor) = monitors
                .iter()
                .find(|monitor| monitor_matches_logical_bounds(monitor_geometry(monitor), bounds))
            {
                return Some(monitor.clone());
            }
        }

        // Tauri 2.12's macOS monitor_from_point follows CoreGraphics screen
        // coordinates (logical points), while tray-icon reports physical
        // event coordinates. Test each monitor's scale and accept only the
        // monitor that CoreGraphics resolves for that converted point.
        if click_position.x.is_finite() && click_position.y.is_finite() {
            for candidate in monitors {
                let scale = candidate.scale_factor();
                if !scale.is_finite() || scale <= 0.0 {
                    continue;
                }
                let resolved = app
                    .monitor_from_point(click_position.x / scale, click_position.y / scale)
                    .ok()
                    .flatten();
                if let Some(resolved) = resolved {
                    if same_monitor(candidate, &resolved) {
                        return Some(candidate.clone());
                    }
                }
            }
        }

        // The icon rectangle is also physical. Its center is a safer anchor
        // than a potentially edge-rounded cursor point.
        if anchor_is_valid(anchor) {
            let center_x = anchor.x + anchor.width / 2.0;
            let center_y = anchor.y + anchor.height / 2.0;
            for candidate in monitors {
                let scale = candidate.scale_factor();
                if !scale.is_finite() || scale <= 0.0 {
                    continue;
                }
                let resolved = app
                    .monitor_from_point(center_x / scale, center_y / scale)
                    .ok()
                    .flatten();
                if let Some(resolved) = resolved {
                    if same_monitor(candidate, &resolved) {
                        return Some(candidate.clone());
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = clicked_display;
        if click_position.x.is_finite() && click_position.y.is_finite() {
            if let Some(monitor) = app
                .monitor_from_point(click_position.x, click_position.y)
                .ok()
                .flatten()
            {
                return Some(monitor);
            }
        }
        if anchor_is_valid(anchor) {
            let x = anchor.x + anchor.width / 2.0;
            let y = anchor.y + anchor.height / 2.0;
            if let Some(monitor) = app.monitor_from_point(x, y).ok().flatten() {
                return Some(monitor);
            }
        }
    }

    // Last geometry-based fallback before primary-monitor fallback. Tray
    // events and monitor bounds are physical on Windows and macOS, so a valid
    // click or icon rectangle can still identify a display if API lookup did
    // not return one. Captured CoreGraphics bounds above take precedence.
    let point = if click_position.x.is_finite() && click_position.y.is_finite() {
        Some((click_position.x, click_position.y))
    } else if anchor_is_valid(anchor) {
        Some((
            anchor.x + anchor.width / 2.0,
            anchor.y + anchor.height / 2.0,
        ))
    } else {
        None
    };
    if let Some((x, y)) = point {
        if let Some(monitor) = monitors.iter().find(|monitor| {
            let origin = monitor.position();
            let size = monitor.size();
            x >= f64::from(origin.x)
                && y >= f64::from(origin.y)
                && x < f64::from(origin.x) + f64::from(size.width)
                && y < f64::from(origin.y) + f64::from(size.height)
        }) {
            return Some(monitor.clone());
        }
    }

    None
}

#[derive(Clone, Copy)]
struct LogicalMonitorBounds {
    #[cfg(any(target_os = "macos", test))]
    x: f64,
    #[cfg(any(target_os = "macos", test))]
    y: f64,
    #[cfg(any(target_os = "macos", test))]
    width: f64,
    #[cfg(any(target_os = "macos", test))]
    height: f64,
}

#[cfg(any(target_os = "macos", test))]
#[derive(Clone, Copy)]
struct MonitorGeometry {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale: f64,
}

#[cfg(target_os = "macos")]
fn monitor_geometry(monitor: &tauri::Monitor) -> MonitorGeometry {
    MonitorGeometry {
        x: monitor.position().x,
        y: monitor.position().y,
        width: monitor.size().width,
        height: monitor.size().height,
        scale: monitor.scale_factor(),
    }
}

#[cfg(any(target_os = "macos", test))]
fn monitor_matches_logical_bounds(monitor: MonitorGeometry, bounds: LogicalMonitorBounds) -> bool {
    if !monitor.scale.is_finite() || monitor.scale <= 0.0 {
        return false;
    }
    const TOLERANCE: f64 = 2.0;
    let logical = [
        f64::from(monitor.x) / monitor.scale,
        f64::from(monitor.y) / monitor.scale,
        f64::from(monitor.width) / monitor.scale,
        f64::from(monitor.height) / monitor.scale,
    ];
    (logical[0] - bounds.x).abs() <= TOLERANCE
        && (logical[1] - bounds.y).abs() <= TOLERANCE
        && (logical[2] - bounds.width).abs() <= TOLERANCE
        && (logical[3] - bounds.height).abs() <= TOLERANCE
}

#[cfg(target_os = "macos")]
fn macos_cursor_display_bounds() -> Option<LogicalMonitorBounds> {
    unsafe extern "C" {
        fn thaa_macos_cursor_display_bounds(
            x: *mut f64,
            y: *mut f64,
            width: *mut f64,
            height: *mut f64,
        ) -> i32;
    }

    let mut bounds = LogicalMonitorBounds {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };
    let success = unsafe {
        thaa_macos_cursor_display_bounds(
            &mut bounds.x,
            &mut bounds.y,
            &mut bounds.width,
            &mut bounds.height,
        )
    };
    (success == 1).then_some(bounds)
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
    let logical_width = WIDTH.min((work_width / scale - MARGIN * 2.0).max(1.0));
    let logical_height = MAX_HEIGHT.min((work_height / scale - MARGIN * 2.0).max(1.0));
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
    use super::{
        anchor_is_valid, calculate_tray_popover_position, is_primary_tray_activation,
        monitor_matches_logical_bounds, physical_position_to_logical, placement_anchor,
        LogicalMonitorBounds, MonitorGeometry, PopoverAnchor, TrayPopoverState,
        TrayPopoverVisibilityAction, WorkArea,
    };
    use tauri::{
        tray::{MouseButton, MouseButtonState},
        PhysicalPosition,
    };

    #[test]
    fn tray_click_transitions_open_then_closed() {
        let state = TrayPopoverState::default();

        let open = state.toggle();
        let close = state.toggle();

        assert!(open.open);
        assert!(!close.open);
        assert!(!state.is_open());
        assert!(close.revision > open.revision);
    }

    #[test]
    fn tray_toggle_visibility_effects_are_scoped_to_the_popover() {
        let state = TrayPopoverState::default();

        let open = state.toggle();
        let close = state.toggle();

        assert_eq!(
            open.visibility_action(),
            TrayPopoverVisibilityAction::ShowPopover
        );
        assert_eq!(
            close.visibility_action(),
            TrayPopoverVisibilityAction::HidePopover
        );
    }

    #[test]
    fn rapid_primary_clicks_apply_only_the_latest_visibility_transition() {
        let state = TrayPopoverState::default();
        let first = state.toggle();
        let second = state.toggle();

        assert!(first.open);
        assert!(!second.open);
        assert!(!state.dismiss_if_current(first.revision));
        assert!(!state.is_open());

        for _ in 0..4 {
            state.toggle();
        }
        assert!(!state.is_open());
    }

    #[test]
    fn focus_loss_does_not_override_a_newer_tray_click() {
        let state = TrayPopoverState::default();
        let opened = state.toggle();
        state.mark_focused();
        let focus_loss_revision = state.revision();
        let closed = state.toggle();

        assert!(opened.open);
        assert!(!closed.open);
        assert!(!state.dismiss_if_current(focus_loss_revision));
        assert!(!state.is_open());
    }

    #[test]
    fn outside_dismissal_and_escape_leave_the_next_tray_click_open() {
        let state = TrayPopoverState::default();
        let opened = state.toggle();
        state.mark_focused();
        assert!(state.dismiss_if_current(opened.revision));
        assert!(!state.is_open());
        assert!(state.toggle().open);

        state.dismiss();
        assert!(state.toggle().open);
    }

    #[test]
    fn outside_dismissal_sources_only_close_the_current_open_popover() {
        let state = TrayPopoverState::default();
        let opened = state.toggle();
        state.mark_focused();

        // Window focus loss and application deactivation may report the same
        // outside interaction. Only the first event may consume this revision.
        assert!(state.dismiss_if_current(opened.revision));
        assert!(!state.dismiss_if_current(opened.revision));
        assert!(!state.is_open());

        let reopened = state.toggle();
        assert!(reopened.open);
        assert_eq!(state.focused_revision(), None);
        assert!(!state.dismiss_if_current(opened.revision));
        assert!(state.is_open());
    }

    #[test]
    fn delayed_outside_dismissal_cannot_hide_a_newer_popover_open() {
        let state = TrayPopoverState::default();
        let opened = state.toggle();
        assert!(opened.open);
        state.mark_focused();
        let outside_event_revision = state.revision();

        assert!(state.dismiss_if_current(outside_event_revision));
        let reopened = state.toggle();
        assert!(reopened.open);

        assert!(!state.dismiss_if_current(outside_event_revision));
        assert!(state.is_open());
    }

    #[test]
    fn focus_loss_from_closed_session_is_ignored_after_reopen_until_refocused() {
        let state = TrayPopoverState::default();
        let first_open = state.toggle();
        state.mark_focused();
        let old_focus_revision = state.focused_revision().unwrap();

        assert!(!state.toggle().open);
        assert!(state.toggle().open);
        assert_eq!(state.focused_revision(), None);
        assert!(!state.dismiss_if_current(old_focus_revision));
        assert!(state.is_open());

        state.mark_focused();
        let current_revision = state.focused_revision().unwrap();
        assert!(current_revision > first_open.revision);
        assert!(state.dismiss_if_current(current_revision));
        assert!(!state.is_open());
    }

    #[test]
    fn only_primary_down_and_windows_primary_double_click_toggle() {
        assert!(is_primary_tray_activation(
            MouseButton::Left,
            Some(MouseButtonState::Down),
            false,
        ));
        assert!(!is_primary_tray_activation(
            MouseButton::Left,
            Some(MouseButtonState::Up),
            false,
        ));
        assert!(is_primary_tray_activation(MouseButton::Left, None, true));
        assert!(!is_primary_tray_activation(
            MouseButton::Right,
            Some(MouseButtonState::Down),
            false,
        ));
        assert!(!is_primary_tray_activation(MouseButton::Right, None, true));
    }

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

    #[test]
    fn tray_popover_clamps_to_monitor_with_negative_origin_and_small_work_area() {
        let (x, y, width, height) = calculate_tray_popover_position(
            PopoverAnchor {
                x: -1900.0,
                y: 12.0,
                width: 32.0,
                height: 28.0,
            },
            WorkArea {
                x: -1920,
                y: 0,
                width: 480,
                height: 300,
            },
            1.0,
        );
        assert!(x >= -1912);
        assert!(y >= 8);
        assert!(x as f64 + width <= -1448.0);
        assert!(y as f64 + height <= 292.0);
        assert_eq!(width, 400.0);
        assert_eq!(height, 284.0);
    }

    #[test]
    fn tray_popover_fits_work_area_smaller_than_nominal_minimum() {
        let (x, y, width, height) = calculate_tray_popover_position(
            PopoverAnchor {
                x: 2.0,
                y: 2.0,
                width: 8.0,
                height: 8.0,
            },
            WorkArea {
                x: 0,
                y: 0,
                width: 250,
                height: 160,
            },
            1.0,
        );
        assert_eq!(width, 234.0);
        assert_eq!(height, 144.0);
        assert!(x >= 8 && x + width as i32 <= 242);
        assert!(y >= 8 && y + height as i32 <= 152);
    }

    #[test]
    fn macos_target_position_conversion_uses_target_scale_and_preserves_negative_origin() {
        let one_x = physical_position_to_logical(-1440, 120, 1.0);
        assert_eq!((one_x.x, one_x.y), (-1440.0, 120.0));

        let two_x = physical_position_to_logical(-2880, 240, 2.0);
        assert_eq!((two_x.x, two_x.y), (-1440.0, 120.0));
    }

    #[test]
    fn invalid_tray_geometry_uses_valid_click_then_work_area_center() {
        let invalid = PopoverAnchor {
            x: f64::NAN,
            y: f64::NAN,
            width: 0.0,
            height: 0.0,
        };
        let area = WorkArea {
            x: -1000,
            y: 100,
            width: 800,
            height: 600,
        };
        let from_click = placement_anchor(invalid, PhysicalPosition::new(-900.0, 130.0), area);
        assert_eq!((from_click.x, from_click.y), (-900.0, 130.0));

        let from_area = placement_anchor(invalid, PhysicalPosition::new(f64::NAN, f64::NAN), area);
        assert_eq!((from_area.x, from_area.y), (-600.0, 400.0));
    }

    #[test]
    fn tray_monitor_match_handles_negative_origin_and_per_display_scale() {
        let external_display = MonitorGeometry {
            x: -2560,
            y: 0,
            width: 3840,
            height: 2160,
            scale: 2.0,
        };
        assert!(monitor_matches_logical_bounds(
            external_display,
            LogicalMonitorBounds {
                x: -1280.0,
                y: 0.0,
                width: 1920.0,
                height: 1080.0,
            }
        ));
        assert!(!monitor_matches_logical_bounds(
            external_display,
            LogicalMonitorBounds {
                x: 0.0,
                y: 0.0,
                width: 1920.0,
                height: 1080.0,
            }
        ));
    }

    #[test]
    fn tray_monitor_match_distinguishes_different_scale_displays() {
        let primary = MonitorGeometry {
            x: 0,
            y: 0,
            width: 3024,
            height: 1964,
            scale: 2.0,
        };
        let external = MonitorGeometry {
            x: 1512,
            y: 0,
            width: 1920,
            height: 1080,
            scale: 1.0,
        };
        let external_bounds = LogicalMonitorBounds {
            x: 1512.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        assert!(!monitor_matches_logical_bounds(primary, external_bounds));
        assert!(monitor_matches_logical_bounds(external, external_bounds));
    }

    #[test]
    fn tray_anchor_rejects_invalid_geometry_for_cursor_position_fallback() {
        assert!(!anchor_is_valid(PopoverAnchor {
            x: f64::NAN,
            y: 10.0,
            width: 24.0,
            height: 24.0,
        }));
        assert!(!anchor_is_valid(PopoverAnchor {
            x: 10.0,
            y: 10.0,
            width: 0.0,
            height: 24.0,
        }));
    }
}
