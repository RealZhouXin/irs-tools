use std::collections::HashSet;
use std::path::PathBuf;

use tauri::Manager;
use tracing::{info, warn};

use crate::models::{BaseConfig, TestConfig, TestGroup};
use crate::types::{AppError, CommandResult};

const DEFAULT_CONFIG_FILES: &[&str] = &["config/threshold.toml", "config/tests.toml"];

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
    #[serde(default)]
    stages: Vec<String>,
    tests: Vec<TestGroup>,
}

pub fn read_config(app: &tauri::AppHandle) -> CommandResult<TestConfig> {
    let base_config = read_base_config(app)?;
    let tests_config = read_tests_config_file(app)?;
    let stages = resolve_stage_order(&tests_config);
    let tests = tests_config.tests;

    Ok(TestConfig {
        connection: base_config.connection,
        read_timeout_ms: base_config.read_timeout_ms,
        stages,
        tests,
    })
}

pub fn read_base_config(app: &tauri::AppHandle) -> CommandResult<BaseConfig> {
    let base_path = resolve_readable_config_path(app, "config/threshold.toml", "配置文件")?;
    info!("Using config at {}", base_path.display());
    let base_data =
        std::fs::read_to_string(&base_path).map_err(|err| AppError::io("无法读取配置文件", err))?;
    toml::from_str(&base_data).map_err(|err| AppError::toml_de("配置文件解析失败", err))
}

pub fn read_test_stages(app: &tauri::AppHandle) -> CommandResult<Vec<String>> {
    let tests_config = read_tests_config_file(app)?;
    Ok(resolve_stage_order(&tests_config))
}

fn read_tests_config_file(app: &tauri::AppHandle) -> CommandResult<TestsConfig> {
    let tests_path = resolve_readable_config_path(app, "config/tests.toml", "测试项配置文件")?;
    info!("Using tests at {}", tests_path.display());
    let tests_data = std::fs::read_to_string(&tests_path)
        .map_err(|err| AppError::io("无法读取测试项配置文件", err))?;
    toml::from_str(&tests_data).map_err(|err| AppError::toml_de("测试项配置解析失败", err))
}

fn resolve_stage_order(tests_config: &TestsConfig) -> Vec<String> {
    let mut seen = HashSet::<String>::new();
    let mut resolved = Vec::<String>::new();

    for stage in &tests_config.stages {
        if let Some(normalized) = normalize_stage(stage) {
            if seen.insert(normalized.clone()) {
                resolved.push(normalized);
            }
        }
    }

    for test in &tests_config.tests {
        if let Some(normalized) = normalize_stage(&test.stage) {
            if seen.insert(normalized.clone()) {
                resolved.push(normalized);
            }
        }
    }

    resolved
}

fn normalize_stage(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

pub fn write_base_config(app: &tauri::AppHandle, config: &BaseConfig) -> CommandResult<()> {
    let path = resolve_writable_config_path(app, "config/threshold.toml", "配置文件")?;
    let data = toml::to_string_pretty(config)
        .map_err(|err| AppError::toml_ser("配置文件序列化失败", err))?;
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
        warn!(
            "{} not found at expected path: {}",
            label,
            config_path.display()
        );
    } else {
        warn!("Failed to get app config dir when reading {}", label);
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
    warn!("Failed to get app config dir when writing {}", label);

    Err(AppError::msg(format!("无法找到{label}写入路径")))
}
