use tauri::Manager;
use tracing::error;

use crate::config::{read_base_config, read_test_stages, write_base_config};
use crate::models::{BaseConfig, TestResult, TestSummary};
use crate::test_runner::submit_front_light_confirmation;
use crate::test_service::{TestService, request_stop_test};
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

#[tauri::command]
pub async fn export_test_results_csv(
    app: tauri::AppHandle,
    start_date: String,
    end_date: String,
    output_path: String,
) -> CommandResult<usize> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::db::export_test_results_csv(&app_handle, &start_date, &end_date, &output_path)
    })
    .await
    .map_err(|err| {
        error!("export_test_results_csv worker failed: {}", err);
        format!("导出任务线程执行失败: {err}")
    })?
}

#[tauri::command]
pub async fn get_available_export_dates(app: tauri::AppHandle) -> CommandResult<Vec<String>> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || crate::db::get_available_export_dates(&app_handle))
        .await
        .map_err(|err| {
            error!("get_available_export_dates worker failed: {}", err);
            format!("导出日期查询线程执行失败: {err}")
        })?
}

#[tauri::command]
pub fn stop_test() -> CommandResult<()> {
    request_stop_test();
    Ok(())
}

#[tauri::command]
pub fn confirm_front_light(is_lit: bool) -> CommandResult<()> {
    submit_front_light_confirmation(is_lit)
}
