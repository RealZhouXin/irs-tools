use tauri::Emitter;
use tracing::{error, info};

use crate::config::read_config;
use crate::device_gateway::{DeviceGatewayFactory, DllDeviceGatewayFactory};
use crate::events::TEST_GROUP_COMPLETE;
use crate::models::{ConnectionConfig, TestConfig, TestResult, TestSummary};
use crate::test_runner::run_group;
use crate::types::CommandResult;

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
                    return Err(err);
                }
            }
        }

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
        run_group(gateway.as_ref(), group)
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
