use tauri::Manager;

use crate::models::{ConnectionConfig, TestConfig, TestGroup};
use crate::types::CommandResult;

#[derive(serde::Deserialize)]
struct BaseConfig {
    connection: ConnectionConfig,
    read_timeout_ms: u32,
}

#[derive(serde::Deserialize)]
struct TestsConfig {
    tests: Vec<TestGroup>,
}

pub fn read_config(app: &tauri::AppHandle) -> CommandResult<TestConfig> {
    let base_path = app
        .path()
        .resolve(
            "config/threshold.json",
            tauri::path::BaseDirectory::Resource,
        )
        .map_err(|_| "无法找到配置文件路径".to_string())?;

    let tests_path = app
        .path()
        .resolve("config/tests.json", tauri::path::BaseDirectory::Resource)
        .map_err(|_| "无法找到测试项配置文件路径".to_string())?;

    let base_data = std::fs::read_to_string(&base_path)
        .map_err(|err| format!("无法读取配置文件: {err}"))?;
    let tests_data = std::fs::read_to_string(&tests_path)
        .map_err(|err| format!("无法读取测试项配置文件: {err}"))?;

    let base_config: BaseConfig =
        serde_json::from_str(&base_data).map_err(|err| format!("配置文件解析失败: {err}"))?;
    let tests_config: TestsConfig =
        serde_json::from_str(&tests_data).map_err(|err| format!("测试项配置解析失败: {err}"))?;

    Ok(TestConfig {
        connection: base_config.connection,
        read_timeout_ms: base_config.read_timeout_ms,
        tests: tests_config.tests,
    })
}
