use tauri::Manager;

use crate::models::TestConfig;
use crate::types::CommandResult;

pub fn read_config(app: &tauri::AppHandle) -> CommandResult<TestConfig> {
    let path = app
        .path()
        .resolve(
            "config/thresholds.json",
            tauri::path::BaseDirectory::Resource,
        )
        .map_err(|_| "无法找到配置文件路径".to_string())?;

    let data = std::fs::read_to_string(&path).map_err(|err| format!("无法读取配置文件: {err}"))?;
    serde_json::from_str(&data).map_err(|err| format!("配置文件解析失败: {err}"))
}
