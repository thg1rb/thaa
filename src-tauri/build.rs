fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/platform/macos/process_identity.c")
            .warnings(true)
            .compile("thaa_macos_process_identity");
        cc::Build::new()
            .file("src/platform/macos/process_icon.m")
            .flag("-fobjc-arc")
            .warnings(true)
            .compile("thaa_macos_process_icon");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=proc");
        println!("cargo:rerun-if-changed=src/platform/macos/process_identity.c");
        println!("cargo:rerun-if-changed=src/platform/macos/process_icon.m");
    }

    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_runtime_snapshot",
            "refresh_runtime_snapshot",
            "request_process_action",
            "open_listener_url",
        ]),
    ))
    .expect("failed to build Tauri application");
}
