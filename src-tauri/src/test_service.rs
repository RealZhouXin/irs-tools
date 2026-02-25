use tauri::Emitter;
use tracing::{error, info};

use crate::config::read_config;
use crate::device_gateway::{DeviceGatewayFactory, DllDeviceGatewayFactory};
use crate::events::TEST_GROUP_COMPLETE;
use crate::models::{ConnectionConfig, TestConfig, TestResult, TestSummary};
use crate::test_runner::run_group;
use crate::types::{AppError, CommandResult};

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

    pub fn start_test(&self) -> CommandResult<TestSummary> {
        info!("Starting full test run");
        let config = read_config(&self.app)?;
        let TestConfig {
            connection,
            read_timeout_ms,
            tests,
        } = config;
        let gateway = self.build_gateway(&connection, read_timeout_ms)?;
        let mut results = Vec::with_capacity(tests.len());
        gateway.param_id374(2)?;
        let mut run_error: Option<AppError> = None;

        for group in tests {
            let name = group.name.clone();
            match run_group(gateway.as_ref(), group) {
                Ok(result) => {
                    info!("Completed group {}", name);
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
        }

        let exit_mode_result = gateway.param_id374(0);
        if let Some(err) = run_error {
            if let Err(exit_err) = exit_mode_result {
                return Err(AppError::msg(format!(
                    "{err}; 且退出测试模式失败: {exit_err}"
                )));
            }
            return Err(err);
        }
        exit_mode_result?;

        let overall_passed = results.iter().all(|item| item.passed);
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
            tests,
        } = config;
        let group = tests
            .into_iter()
            .find(|item| item.name == group_name)
            .ok_or_else(|| format!("未找到测试项: {group_name}"))?;
        let gateway = self.build_gateway(&connection, read_timeout_ms)?;
        gateway.param_id374(2)?;
        let result = run_group(gateway.as_ref(), group);
        let exit_mode_result = gateway.param_id374(0);
        match (result, exit_mode_result) {
            (Ok(test_result), Ok(())) => Ok(test_result),
            (Err(run_err), Ok(())) => Err(run_err),
            (Ok(_), Err(exit_err)) => Err(exit_err),
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
