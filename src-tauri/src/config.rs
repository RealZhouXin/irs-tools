use std::path::PathBuf;

use tauri::Manager;
use tracing::info;

use crate::models::{BaseConfig, TestConfig, TestGroup};
use crate::types::{AppError, CommandResult};

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
    let base_data = std::fs::read_to_string(&base_path).map_err(|err| AppError::io("无法读取配置文件", err))?;
    serde_json::from_str(&base_data).map_err(|err| AppError::json("配置文件解析失败", err))
}

pub fn read_tests_config(app: &tauri::AppHandle) -> CommandResult<Vec<TestGroup>> {
    let tests_path =
        resolve_readable_config_path(app, "config/tests.json", "测试项配置文件")?;
    info!("Using tests at {}", tests_path.display());
    let tests_data =
        std::fs::read_to_string(&tests_path).map_err(|err| AppError::io("无法读取测试项配置文件", err))?;
    let tests_config: TestsConfig =
        serde_json::from_str(&tests_data).map_err(|err| AppError::json("测试项配置解析失败", err))?;
    Ok(tests_config.tests)
}

pub fn write_base_config(app: &tauri::AppHandle, config: &BaseConfig) -> CommandResult<()> {
    let path = resolve_writable_config_path(app, "config/threshold.json", "配置文件")?;
    let data =
        serde_json::to_string_pretty(config).map_err(|err| AppError::json("配置文件序列化失败", err))?;
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
    if let Ok(current_dir) = std::env::current_dir() {
        let direct = current_dir.join(relative);
        if direct.exists() {
            return Ok(direct);
        }

        let nested = current_dir.join("src-tauri").join(relative);
        if nested.exists() {
            return Ok(nested);
        }
    }

    if let Ok(resource_path) = app
        .path()
        .resolve(relative, tauri::path::BaseDirectory::Resource)
    {
        if resource_path.exists() {
            return Ok(resource_path);
        }
    }

    Err(AppError::msg(format!("无法找到{label}路径")))
}

fn resolve_writable_config_path(
    _app: &tauri::AppHandle,
    relative: &str,
    label: &str,
) -> CommandResult<PathBuf> {
    if let Ok(current_dir) = std::env::current_dir() {
        let direct = current_dir.join(relative);
        if direct.exists() {
            return Ok(direct);
        }

        let nested_root = current_dir.join("src-tauri");
        if nested_root.exists() {
            let nested = nested_root.join(relative);
            return Ok(nested);
        }

        return Ok(direct);
    }

    Err(AppError::msg(format!("无法找到{label}写入路径")))
}
