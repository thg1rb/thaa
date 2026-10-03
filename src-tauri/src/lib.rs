mod commands;
pub mod domain;
pub mod platform;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::app_info::get_app_info])
        .run(tauri::generate_context!())
        .expect("failed to run Thaa");
}
