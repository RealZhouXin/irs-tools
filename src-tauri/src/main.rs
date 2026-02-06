mod commands;
mod comm_dll;
mod config;
mod models;
mod types;

use tracing::warn;

fn main() {
    init_logging();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::start_test,
            commands::retest_group,
            commands::show_main_window,
            commands::get_base_config,
            commands::save_base_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_logging() {
    if let Err(err) = tracing_subscriber::fmt().with_target(false).try_init() {
        warn!("Tracing already initialized: {}", err);
    }
}
