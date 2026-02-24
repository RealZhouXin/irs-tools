use std::path::PathBuf;

use tauri::Manager;
use tracing::{info, warn};

use crate::models::{BaseConfig, TestConfig, TestGroup};
use crate::types::{AppError, CommandResult};

const DEFAULT_CONFIG_FILES: &[&str] = &["config/threshold.json", "config/tests.json"];

pub fn init_default_configs(app: &tauri::AppHandle) {
    let app_config_dir = match app.path().app_config_dir() {
        Ok(dir) => dir,
        Err(err) => {
            warn!("Failed to get app config dir: {}", err);
            return;
        }
    };

    for relative in DEFAULT_CONFIG_FILES {
        let dest = app_config_dir.join(relative);
        if dest.exists() {
            continue;
        }
        if let Some(parent) = dest.parent() {
            if let Err(err) = std::fs::create_dir_all(parent) {
                warn!(
                    "Failed to create config directory {}: {}",
                    parent.display(),
                    err
                );
                continue;
            }
        }
        // 从 Resource 拷贝默认配置
        match app
            .path()
            .resolve(relative, tauri::path::BaseDirectory::Resource)
        {
            Ok(src) if src.exists() => match std::fs::copy(&src, &dest) {
                Ok(_) => info!("Initialized default config: {}", dest.display()),
                Err(err) => warn!(
                    "Failed to copy default config to {}: {}",
                    dest.display(),
                    err
                ),
            },
            _ => {
                // 开发模式下从 src-tauri 目录拷贝
                if let Ok(current_dir) = std::env::current_dir() {
                    let dev_src = current_dir.join("src-tauri").join(relative);
                    if !dev_src.exists() {
                        warn!("Default config source not found for: {}", relative);
                        continue;
                    }
                    match std::fs::copy(&dev_src, &dest) {
                        Ok(_) => info!("Initialized default config (dev): {}", dest.display()),
                        Err(err) => warn!(
                            "Failed to copy default config to {}: {}",
                            dest.display(),
                            err
                        ),
                    }
                }
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct TestsConfig {
    tests: Vec<TestGroup>,
}

pub fn read_config(app: &tauri::AppHandle) -> CommandResult<TestConfig> {
    let base_config = read_base_config(app)?;
    let tests = read_tests_config(app)?;

    Ok(TestConfig {
        connection: base_config.connection,
        read_timeout_ms: base_config.read_timeout_ms,
        tests,
    })
}

pub fn read_base_config(app: &tauri::AppHandle) -> CommandResult<BaseConfig> {
    let base_path = resolve_readable_config_path(app, "config/threshold.json", "配置文件")?;
    info!("Using config at {}", base_path.display());
    let base_data =
        std::fs::read_to_string(&base_path).map_err(|err| AppError::io("无法读取配置文件", err))?;
    serde_json::from_str(&base_data).map_err(|err| AppError::json("配置文件解析失败", err))
}

pub fn read_tests_config(app: &tauri::AppHandle) -> CommandResult<Vec<TestGroup>> {
    let tests_path = resolve_readable_config_path(app, "config/tests.json", "测试项配置文件")?;
    info!("Using tests at {}", tests_path.display());
    let tests_data = std::fs::read_to_string(&tests_path)
        .map_err(|err| AppError::io("无法读取测试项配置文件", err))?;
    let tests_config: TestsConfig = serde_json::from_str(&tests_data)
        .map_err(|err| AppError::json("测试项配置解析失败", err))?;
    Ok(tests_config.tests)
}

pub fn write_base_config(app: &tauri::AppHandle, config: &BaseConfig) -> CommandResult<()> {
    let path = resolve_writable_config_path(app, "config/threshold.json", "配置文件")?;
    let data = serde_json::to_string_pretty(config)
        .map_err(|err| AppError::json("配置文件序列化失败", err))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| AppError::io("无法创建配置目录", err))?;
    }
    std::fs::write(&path, data).map_err(|err| AppError::io("无法写入配置文件", err))?;
    info!("Saved config at {}", path.display());
    Ok(())
}

fn resolve_readable_config_path(
    app: &tauri::AppHandle,
    relative: &str,
    label: &str,
) -> CommandResult<PathBuf> {
    if let Ok(app_config_dir) = app.path().app_config_dir() {
        let config_path = app_config_dir.join(relative);
        if config_path.exists() {
            return Ok(config_path);
        }
    }

    Err(AppError::msg(format!("无法找到{label}路径")))
}

fn resolve_writable_config_path(
    app: &tauri::AppHandle,
    relative: &str,
    label: &str,
) -> CommandResult<PathBuf> {
    if let Ok(app_config_dir) = app.path().app_config_dir() {
        return Ok(app_config_dir.join(relative));
    }

    Err(AppError::msg(format!("无法找到{label}写入路径")))
}
