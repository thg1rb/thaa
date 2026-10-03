fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/platform/macos/process_identity.c")
            .warnings(true)
            .compile("thaa_macos_process_identity");
        println!("cargo:rerun-if-changed=src/platform/macos/process_identity.c");
    }

    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["get_app_info"])),
    )
    .expect("failed to build Tauri application");
}
