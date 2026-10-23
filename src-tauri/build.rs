fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/platform/macos/process_identity.c")
            .warnings(true)
            .compile("thaa_macos_process_identity");
        cc::Build::new()
            .file("src/platform/macos/tray_display.c")
            .warnings(true)
            .compile("thaa_macos_tray_display");
        cc::Build::new()
            .file("src/platform/macos/tray_window.m")
            .flag("-fobjc-arc")
            .warnings(true)
            .compile("thaa_macos_tray_window");
        cc::Build::new()
            .file("src/platform/macos/process_icon.m")
            .flag("-fobjc-arc")
            .warnings(true)
            .compile("thaa_macos_process_icon");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
        println!("cargo:rustc-link-lib=proc");
        println!("cargo:rerun-if-changed=src/platform/macos/process_identity.c");
        println!("cargo:rerun-if-changed=src/platform/macos/tray_display.c");
        println!("cargo:rerun-if-changed=src/platform/macos/tray_window.m");
        println!("cargo:rerun-if-changed=src/platform/macos/process_icon.m");
    }

    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_runtime_snapshot",
            "refresh_runtime_snapshot",
            "request_process_action",
            "open_listener_url",
            "reveal_project_folder",
            "open_project_in_terminal",
            "open_project_in_editor",
            "get_tray_runtime_snapshot",
            "get_runtime_process_icons",
            "navigate_to_runtime_entry",
            "show_main_window",
            "dismiss_tray_popover",
        ]),
    ))
    .expect("failed to build Tauri application");
}
