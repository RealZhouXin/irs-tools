use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use chrono::Local;
use tauri::Emitter;
use tracing::{error, info, warn};

use crate::config::read_config;
use crate::device_gateway::{DeviceGateway, DeviceGatewayFactory, DllDeviceGatewayFactory};
use crate::events::{DEVICE_SN_UPDATE, TEST_GROUP_COMPLETE};
use crate::models::{
    ConnectionConfig, DeviceSnPayload, TestConfig, TestGroup, TestResult, TestSummary,
};
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

struct ConnectedGateway {
    gateway: Box<dyn DeviceGateway>,
    session_sn: Option<u32>,
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
        let ConnectedGateway {
            gateway,
            session_sn: connected_session_sn,
        } = self.build_gateway(&connection, read_timeout_ms)?;
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
        let session_sn = connected_session_sn.or_else(|| extract_session_sn(&results));
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
                session_sn,
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
                session_sn,
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
            session_sn,
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
        let ConnectedGateway {
            gateway,
            session_sn: _,
        } = self.build_gateway(&connection, read_timeout_ms)?;
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
    ) -> CommandResult<ConnectedGateway> {
        let gateway = self
            .gateway_factory
            .create(&self.app, connection, read_timeout_ms)?;
        let session_sn = match read_connected_session_sn(gateway.as_ref()) {
            Ok(sn) => Some(sn),
            Err(err) => {
                warn!(
                    "Failed to read mower SN via ParamId526 after connect: {}",
                    err
                );
                None
            }
        };
        if let Some(sn) = session_sn {
            if let Err(err) = self.app.emit(DEVICE_SN_UPDATE, DeviceSnPayload { sn }) {
                warn!("Failed to emit device SN update: {}", err);
            }
        }
        Ok(ConnectedGateway {
            gateway,
            session_sn,
        })
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

fn read_connected_session_sn(gateway: &dyn DeviceGateway) -> CommandResult<u32> {
    info!("Reading mower SN via ParamId526 immediately after connection");
    let response = gateway.param_id526()?;
    info!("Mower SN read successfully: {}", response.pcb_ser_no);
    Ok(response.pcb_ser_no)
}

fn extract_session_sn(results: &[TestResult]) -> Option<u32> {
    results
        .iter()
        .find(|result| result.command == "ParamId526")
        .and_then(|result| {
            result
                .raw_response
                .split(',')
                .map(str::trim)
                .find_map(|part| part.strip_prefix("PcbSerNo="))
        })
        .and_then(|value| value.parse::<u32>().ok())
}

#[cfg(test)]
mod tests {
    use super::{extract_session_sn, read_connected_session_sn};
    use crate::device_gateway::DeviceGateway;
    use crate::models::{
        CheckResult, ParamId068Result, ParamId080Result, ParamId096Result, ParamId114Result,
        ParamId118Result, ParamId120Result, ParamId122Result, ParamId272Result, ParamId470Result,
        ParamId526Result, ParamId588Result, ParamId654Result, ParamId776Result, ParamId794Result,
        ParamId796Result, ParamId798Result, TestResult,
    };
    use crate::types::{AppError, CommandResult};

    struct SnGateway {
        sn: u32,
        fail: bool,
    }

    impl DeviceGateway for SnGateway {
        fn param_id374(&self, _test_mode: u8) -> CommandResult<()> {
            unreachable!()
        }

        fn param_id068(&self) -> CommandResult<ParamId068Result> {
            unreachable!()
        }

        fn param_id588(&self) -> CommandResult<ParamId588Result> {
            unreachable!()
        }

        fn param_id654(&self) -> CommandResult<ParamId654Result> {
            unreachable!()
        }

        fn param_id272(&self) -> CommandResult<ParamId272Result> {
            unreachable!()
        }

        fn param_id526(&self) -> CommandResult<ParamId526Result> {
            if self.fail {
                return Err(AppError::msg("526 failed"));
            }

            Ok(ParamId526Result {
                pcb_de_gr_no: 1,
                pcb_sub_de_no: 2,
                pcb_var_no: 3,
                pcb_pn: 4,
                pcb_rev: 5,
                pcb_ser_no: self.sn,
                pcb_prod_time: 6,
                pcb_ext_flash: 7,
                pcb_ext_eeprom: 8,
                pcb_accelerometer: 9,
            })
        }

        fn param_id096(&self) -> CommandResult<ParamId096Result> {
            unreachable!()
        }

        fn param_id080(&self) -> CommandResult<ParamId080Result> {
            unreachable!()
        }

        fn param_id118(&self) -> CommandResult<ParamId118Result> {
            unreachable!()
        }

        fn param_id120(&self) -> CommandResult<ParamId120Result> {
            unreachable!()
        }

        fn param_id122(&self) -> CommandResult<ParamId122Result> {
            unreachable!()
        }

        fn param_id470(&self) -> CommandResult<ParamId470Result> {
            unreachable!()
        }

        fn param_id468(&self, _cutting_height_mm: u8) -> CommandResult<()> {
            unreachable!()
        }

        fn param_id606(&self, _front_light_mode: u8, _power: u8) -> CommandResult<()> {
            unreachable!()
        }

        fn param_id254(&self, _right_motor_speed: i16) -> CommandResult<()> {
            unreachable!()
        }

        fn param_id256(&self, _left_motor_speed: i16) -> CommandResult<()> {
            unreachable!()
        }

        fn param_id114(&self) -> CommandResult<ParamId114Result> {
            unreachable!()
        }

        fn param_id568(&self, _on: u8) -> CommandResult<()> {
            unreachable!()
        }

        fn param_id610(&self, _rear_light_mode: u8) -> CommandResult<()> {
            unreachable!()
        }

        fn param_id794(&self) -> CommandResult<ParamId794Result> {
            unreachable!()
        }

        fn param_id796(&self) -> CommandResult<ParamId796Result> {
            unreachable!()
        }

        fn param_id798(&self) -> CommandResult<ParamId798Result> {
            unreachable!()
        }

        fn param_id776(&self, _cmd: u8) -> CommandResult<ParamId776Result> {
            unreachable!()
        }
    }

    fn make_test_result(command: &str, raw_response: &str) -> TestResult {
        TestResult {
            name: "group".to_string(),
            names: Default::default(),
            stage: "stage".to_string(),
            command: command.to_string(),
            passed: true,
            raw_response: raw_response.to_string(),
            checks: vec![CheckResult {
                name: "check".to_string(),
                min: None,
                max: None,
                value: None,
                display_min: None,
                display_max: None,
                display_value: None,
                passed: true,
            }],
        }
    }

    #[test]
    fn extract_session_sn_returns_value_from_param_id526() {
        let results = vec![make_test_result(
            "ParamId526",
            "PcbDeGrNo=1, PcbSubDeNo=1, PcbSerNo=12345678, PcbRev=4",
        )];
        assert_eq!(extract_session_sn(&results), Some(12345678));
    }

    #[test]
    fn extract_session_sn_returns_none_when_param_id526_missing() {
        let results = vec![make_test_result("ParamId080", "MowerMainP=1")];
        assert_eq!(extract_session_sn(&results), None);
    }

    #[test]
    fn extract_session_sn_returns_none_when_parse_fails() {
        let results = vec![make_test_result("ParamId526", "PcbSerNo=ABC")];
        assert_eq!(extract_session_sn(&results), None);
    }

    #[test]
    fn read_connected_session_sn_returns_value_from_param_id526() {
        let gateway = SnGateway {
            sn: 87654321,
            fail: false,
        };

        assert_eq!(read_connected_session_sn(&gateway).unwrap(), 87654321);
    }

    #[test]
    fn read_connected_session_sn_returns_error_when_param_id526_fails() {
        let gateway = SnGateway { sn: 0, fail: true };

        assert!(read_connected_session_sn(&gateway).is_err());
    }
}
