use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use chrono::Local;
use tauri::Emitter;
use tracing::{error, info, warn};

use crate::config::read_config;
use crate::device_gateway::{DeviceGatewayFactory, DllDeviceGatewayFactory};
use crate::events::TEST_GROUP_COMPLETE;
use crate::models::{ConnectionConfig, TestConfig, TestGroup, TestResult, TestSummary};
use crate::test_runner::run_group;
use crate::types::{AppError, CommandResult};

static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn request_stop_test() {
    STOP_REQUESTED.store(true, Ordering::SeqCst);
}

fn clear_stop_request() {
    STOP_REQUESTED.store(false, Ordering::SeqCst);
}

fn is_stop_requested() -> bool {
    STOP_REQUESTED.load(Ordering::SeqCst)
}

pub struct TestService<F = DllDeviceGatewayFactory>
where
    F: DeviceGatewayFactory,
{
    app: tauri::AppHandle,
    gateway_factory: F,
}

impl TestService<DllDeviceGatewayFactory> {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self {
            app,
            gateway_factory: DllDeviceGatewayFactory,
        }
    }
}

impl<F> TestService<F>
where
    F: DeviceGatewayFactory,
{
    #[allow(dead_code)]
    pub fn with_factory(app: tauri::AppHandle, gateway_factory: F) -> Self {
        Self {
            app,
            gateway_factory,
        }
    }

    pub fn start_test(&self, requested_stages: Vec<String>) -> CommandResult<TestSummary> {
        clear_stop_request();
        let started_at = Local::now();
        let run_timer = Instant::now();
        let normalized_stages = normalize_requested_stages(&requested_stages);
        let session_stage = summarize_session_stage(&normalized_stages);
        if normalized_stages.is_empty() {
            info!("Starting full test run");
        } else {
            info!("Starting stage-based test run: {:?}", normalized_stages);
        }
        let config = read_config(&self.app)?;
        let TestConfig {
            connection,
            read_timeout_ms,
            stages: _,
            tests,
        } = config;
        let tests = select_tests_by_stages(tests, &normalized_stages)?;
        info!(
            "Loaded test config: total_groups={}, read_timeout_ms={}",
            tests.len(),
            read_timeout_ms
        );
        let gateway = self.build_gateway(&connection, read_timeout_ms)?;
        let mut results = Vec::with_capacity(tests.len());
        info!("Entering test mode via ParamId374(TestMode=2)");
        if let Err(err) = gateway.param_id374(2) {
            error!("Failed to enter test mode: {}", err);
            return Err(err);
        }
        let mut run_error: Option<AppError> = None;
        let mut stopped = false;

        for group in tests {
            if is_stop_requested() {
                warn!("Stop requested before running next group");
                stopped = true;
                break;
            }
            let name = group.name.clone();
            info!("Starting group {}", name);
            let run_result = run_group(gateway.as_ref(), group, &self.app);
            match run_result {
                Ok(result) => {
                    info!("Completed group {} with passed={}", name, result.passed);
                    if let Err(err) = self.app.emit(TEST_GROUP_COMPLETE, &result) {
                        error!("Failed to emit result for {}: {}", name, err);
                    }
                    results.push(result);
                }
                Err(err) => {
                    error!("Group {} failed: {}", name, err);
                    run_error = Some(err);
                    break;
                }
            }
            if is_stop_requested() {
                warn!("Stop requested after group completed");
                stopped = true;
                break;
            }
        }

        info!("Leaving test mode via ParamId374(TestMode=0)");
        let exit_mode_result = gateway.param_id374(0);
        clear_stop_request();
        if let Some(err) = run_error {
            if let Err(exit_err) = exit_mode_result {
                error!(
                    "Failed to leave test mode after group failure: {}",
                    exit_err
                );
                return Err(AppError::msg(format!(
                    "{err}; 且退出测试模式失败: {exit_err}"
                )));
            }
            if let Err(db_err) = crate::db::persist_test_results(
                &self.app,
                &session_stage,
                "Error",
                started_at,
                run_timer.elapsed().as_millis() as i64,
                &results,
            ) {
                error!("Failed to persist errored test run: {}", db_err);
                return Err(AppError::msg(format!(
                    "{err}; 且保存测试记录失败: {db_err}"
                )));
            }
            warn!("Test run ended with error: {}", err);
            return Err(err);
        }
        exit_mode_result?;
        if stopped {
            let overall_passed = results.iter().all(|item| item.passed);
            let status = if overall_passed { "Pass" } else { "Fail" };
            if let Err(db_err) = crate::db::persist_test_results(
                &self.app,
                &session_stage,
                status,
                started_at,
                run_timer.elapsed().as_millis() as i64,
                &results,
            ) {
                error!("Failed to persist stopped test run: {}", db_err);
                return Err(AppError::msg(format!(
                    "测试已手动停止; 且保存测试记录失败: {db_err}"
                )));
            }
            return Err(AppError::msg("测试已手动停止"));
        }

        let overall_passed = results.iter().all(|item| item.passed);
        info!(
            "Test run finished: overall_passed={}, completed_groups={}",
            overall_passed,
            results.len()
        );
        let status = if overall_passed { "Pass" } else { "Fail" };
        crate::db::persist_test_results(
            &self.app,
            &session_stage,
            status,
            started_at,
            run_timer.elapsed().as_millis() as i64,
            &results,
        )?;
        Ok(TestSummary {
            results,
            overall_passed,
        })
    }

    pub fn retest_group(&self, group_name: String) -> CommandResult<TestResult> {
        info!("Retest group {}", group_name);
        let config = read_config(&self.app)?;
        let TestConfig {
            connection,
            read_timeout_ms,
            stages: _,
            tests,
        } = config;
        let group = tests
            .into_iter()
            .find(|item| item.name == group_name)
            .ok_or_else(|| format!("未找到测试项: {group_name}"))?;
        let gateway = self.build_gateway(&connection, read_timeout_ms)?;
        info!("Retest entering test mode via ParamId374(TestMode=2)");
        gateway.param_id374(2)?;
        let result = run_group(gateway.as_ref(), group, &self.app);
        info!("Retest leaving test mode via ParamId374(TestMode=0)");
        let exit_mode_result = gateway.param_id374(0);
        match (result, exit_mode_result) {
            (Ok(test_result), Ok(())) => {
                info!(
                    "Retest finished: group={}, passed={}",
                    test_result.name, test_result.passed
                );
                Ok(test_result)
            }
            (Err(run_err), Ok(())) => {
                error!("Retest run failed: {}", run_err);
                Err(run_err)
            }
            (Ok(_), Err(exit_err)) => {
                error!("Retest failed to leave test mode: {}", exit_err);
                Err(exit_err)
            }
            (Err(run_err), Err(exit_err)) => Err(AppError::msg(format!(
                "{run_err}; 且退出测试模式失败: {exit_err}"
            ))),
        }
    }

    fn build_gateway(
        &self,
        connection: &ConnectionConfig,
        read_timeout_ms: u32,
    ) -> CommandResult<Box<dyn crate::device_gateway::DeviceGateway>> {
        self.gateway_factory
            .create(&self.app, connection, read_timeout_ms)
    }
}

fn summarize_session_stage(stages: &[String]) -> String {
    if stages.is_empty() {
        return "ALL".to_string();
    }
    stages.join(", ")
}

fn normalize_requested_stages(stages: &[String]) -> Vec<String> {
    let mut seen = HashSet::<String>::new();
    let mut normalized = Vec::<String>::new();

    for stage in stages {
        let trimmed = stage.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_string();
        if seen.insert(key.clone()) {
            normalized.push(key);
        }
    }

    normalized
}

fn select_tests_by_stages(
    tests: Vec<TestGroup>,
    requested_stages: &[String],
) -> CommandResult<Vec<TestGroup>> {
    if requested_stages.is_empty() {
        return Ok(tests);
    }

    let mut selected = Vec::<TestGroup>::new();

    for stage in requested_stages {
        selected.extend(
            tests
                .iter()
                .filter(|group| group.stage.trim() == stage)
                .cloned(),
        );
    }

    if selected.is_empty() {
        return Err(AppError::msg(format!(
            "未找到匹配的测试阶段: {}",
            requested_stages.join(", ")
        )));
    }

    Ok(selected)
}
