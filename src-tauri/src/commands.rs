use tauri::Manager;
use tracing::error;

use crate::config::{read_base_config, read_test_stages, write_base_config};
use crate::models::{BaseConfig, TestResult, TestSummary};
use crate::test_service::TestService;
use crate::types::CommandResult;

#[tauri::command]
pub async fn start_test(
    app: tauri::AppHandle,
    stages: Option<Vec<String>>,
) -> CommandResult<TestSummary> {
    let requested_stages = stages.unwrap_or_default();
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        TestService::new(app_handle).start_test(requested_stages)
    })
    .await
    .map_err(|err| {
        error!("start_test worker failed: {}", err);
        format!("测试任务线程执行失败: {err}")
    })?
}

#[tauri::command]
pub async fn retest_group(app: tauri::AppHandle, group_name: String) -> CommandResult<TestResult> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        TestService::new(app_handle).retest_group(group_name)
    })
    .await
    .map_err(|err| {
        error!("retest_group worker failed: {}", err);
        format!("重测任务线程执行失败: {err}")
    })?
}

#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle) -> CommandResult<()> {
    let window = app.get_webview_window("main").ok_or("未找到主窗口")?;
    window.show().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_base_config(app: tauri::AppHandle) -> CommandResult<BaseConfig> {
    read_base_config(&app)
}

#[tauri::command]
pub fn save_base_config(app: tauri::AppHandle, config: BaseConfig) -> CommandResult<BaseConfig> {
    write_base_config(&app, &config)?;
    let saved = read_base_config(&app)?;
    crate::apply_log_level(saved.log_level);
    Ok(saved)
}

#[tauri::command]
pub fn get_test_stages(app: tauri::AppHandle) -> CommandResult<Vec<String>> {
    read_test_stages(&app)
}
