#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod comm_dll;
mod commands;
mod config;
mod db;
mod device_gateway;
mod events;
mod models;
mod test_runner;
mod test_service;
mod types;

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use chrono::{Local, NaiveDate};
use tauri::Manager;
use tracing::warn;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{Registry, reload};

use crate::models::LogLevel;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            init_logging(app.handle());
            crate::config::init_default_configs(app.handle());
            if let Err(err) = crate::db::init_database(app.handle()) {
                warn!("Failed to initialize database: {}", err);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_test,
            commands::stop_test,
            commands::retest_group,
            commands::show_main_window,
            commands::get_base_config,
            commands::save_base_config,
            commands::get_test_stages,
            commands::export_test_results_csv,
            commands::get_available_export_dates,
            commands::confirm_front_light,
            commands::confirm_rear_light,
            commands::confirm_speaker,
            commands::cancel_emergency_stop_test,
            commands::cancel_key_test
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

static FILE_LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
static LOG_LEVEL_RELOAD_HANDLE: OnceLock<reload::Handle<LevelFilter, Registry>> = OnceLock::new();
const LOG_PREFIX: &str = "irs-tools-";
const LOG_SUFFIX: &str = ".log";
const LOG_RETENTION_DAYS: i64 = 3;
const CONFIG_RELATIVE_PATH: &str = "config/threshold.toml";

fn init_logging(app: &tauri::AppHandle) {
    let initial_level = read_configured_log_level(app);
    let log_dir = resolve_log_dir(app);
    if let Err(err) = std::fs::create_dir_all(&log_dir) {
        warn!(
            "Failed to create log directory {}: {}",
            log_dir.display(),
            err
        );
    } else if let Err(err) = cleanup_old_logs(&log_dir) {
        warn!(
            "Failed to cleanup old log files in {}: {}",
            log_dir.display(),
            err
        );
    }
    let log_path = resolve_log_path(&log_dir);

    let stdout_layer = if cfg!(debug_assertions) {
        Some(
            tracing_subscriber::fmt::layer()
                .with_timer(LocalTime::rfc_3339())
                .with_target(false),
        )
    } else {
        None
    };
    let init_result = match OpenOptions::new().create(true).append(true).open(&log_path) {
        Ok(file) => {
            let (file_writer, guard) = tracing_appender::non_blocking(file);
            let _ = FILE_LOG_GUARD.set(guard);
            let (level_filter, handle) = reload::Layer::new(to_level_filter(initial_level));
            let _ = LOG_LEVEL_RELOAD_HANDLE.set(handle);
            let file_layer = tracing_subscriber::fmt::layer()
                .with_timer(LocalTime::rfc_3339())
                .with_ansi(false)
                .with_target(false)
                .with_writer(file_writer);
            tracing_subscriber::registry()
                .with(level_filter)
                .with(stdout_layer)
                .with(file_layer)
                .try_init()
        }
        Err(err) => {
            warn!("Failed to open log file {}: {}", log_path.display(), err);
            let (level_filter, handle) = reload::Layer::new(to_level_filter(initial_level));
            let _ = LOG_LEVEL_RELOAD_HANDLE.set(handle);
            tracing_subscriber::registry()
                .with(level_filter)
                .with(stdout_layer)
                .try_init()
        }
    };

    if let Err(err) = init_result {
        warn!("Tracing already initialized: {}", err);
    }
}

pub fn apply_log_level(log_level: LogLevel) {
    if let Some(handle) = LOG_LEVEL_RELOAD_HANDLE.get() {
        let level = to_level_filter(log_level);
        if let Err(err) = handle.modify(|current| {
            *current = level;
        }) {
            warn!("Failed to apply log level: {}", err);
            return;
        }
        warn!("Log level updated to {:?}", log_level);
    }
}

fn resolve_log_dir(app: &tauri::AppHandle) -> PathBuf {
    if let Ok(app_log_dir) = app.path().app_log_dir() {
        return app_log_dir;
    }

    if let Ok(current_dir) = std::env::current_dir() {
        return current_dir.join("logs");
    }

    PathBuf::from("logs")
}

fn resolve_log_path(log_dir: &Path) -> PathBuf {
    let today = Local::now().date_naive();
    let filename = format!("{LOG_PREFIX}{}{LOG_SUFFIX}", today.format("%Y-%m-%d"));
    log_dir.join(filename)
}

fn cleanup_old_logs(log_dir: &Path) -> std::io::Result<()> {
    let today = Local::now().date_naive();
    for entry_result in std::fs::read_dir(log_dir)? {
        let entry = entry_result?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if let Some(log_date) = parse_log_date(&file_name) {
            let age_days = (today - log_date).num_days();
            if age_days >= LOG_RETENTION_DAYS {
                std::fs::remove_file(entry.path())?;
            }
        }
    }

    Ok(())
}

fn parse_log_date(file_name: &str) -> Option<NaiveDate> {
    let date_part = file_name
        .strip_prefix(LOG_PREFIX)?
        .strip_suffix(LOG_SUFFIX)?;
    NaiveDate::parse_from_str(date_part, "%Y-%m-%d").ok()
}

fn read_configured_log_level(app: &tauri::AppHandle) -> LogLevel {
    let Some(path) = resolve_config_path(app) else {
        return LogLevel::default();
    };

    let Ok(config_text) = std::fs::read_to_string(path) else {
        return LogLevel::default();
    };

    #[derive(serde::Deserialize)]
    struct LogLevelConfig {
        #[serde(default)]
        log_level: LogLevel,
    }

    toml::from_str::<LogLevelConfig>(&config_text)
        .map(|cfg| cfg.log_level)
        .unwrap_or_default()
}

fn resolve_config_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    if let Ok(app_config_dir) = app.path().app_config_dir() {
        let config_path = app_config_dir.join(CONFIG_RELATIVE_PATH);
        if config_path.exists() {
            return Some(config_path);
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        let direct = current_dir.join(CONFIG_RELATIVE_PATH);
        if direct.exists() {
            return Some(direct);
        }

        let nested = current_dir.join("src-tauri").join(CONFIG_RELATIVE_PATH);
        if nested.exists() {
            return Some(nested);
        }
    }
    None
}

fn to_level_filter(level: LogLevel) -> LevelFilter {
    match level {
        LogLevel::Error => LevelFilter::ERROR,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Trace => LevelFilter::TRACE,
    }
}
