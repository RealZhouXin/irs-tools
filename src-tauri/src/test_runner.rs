use crate::device_gateway::DeviceGateway;
use crate::models::{
    CheckConfig, CheckResult, CheckableResult, CommandGroupSpec, TestGroup, TestResult,
};
use crate::types::CommandResult;
use std::fmt::Display;

fn process_checks<TConfig, TResult>(checks: &[TConfig], result: &TResult) -> Vec<CheckResult>
where
    TConfig: CheckConfig,
    TResult: CheckableResult<OutputEnum = TConfig::OutputEnum>,
{
    checks
        .iter()
        .map(|check| {
            let value = result.get_value(check.output());
            let passed = value >= check.min() && value <= check.max();
            CheckResult {
                name: check.name().to_string(),
                min: Some(check.min()),
                max: Some(check.max()),
                value: Some(value),
                passed,
            }
        })
        .collect()
}

fn build_checked_result<TConfig, TResult>(
    group_name: String,
    command: String,
    checks: &[TConfig],
    response: &TResult,
) -> TestResult
where
    TConfig: CheckConfig,
    TResult: CheckableResult<OutputEnum = TConfig::OutputEnum> + Display,
{
    let check_results = process_checks(checks, response);
    let passed = check_results.iter().all(|item| item.passed);
    TestResult {
        name: group_name,
        command,
        passed,
        raw_response: response.to_string(),
        checks: check_results,
    }
}

fn build_action_result(group_name: String, command: String, raw_response: String) -> TestResult {
    TestResult {
        name: group_name,
        command,
        passed: true,
        raw_response,
        checks: vec![CheckResult {
            name: "执行结果".to_string(),
            min: None,
            max: None,
            value: None,
            passed: true,
        }],
    }
}

pub fn run_group(gateway: &dyn DeviceGateway, group: TestGroup) -> CommandResult<TestResult> {
    let TestGroup { name, command } = group;
    match command {
        CommandGroupSpec::ParamId068 { checks } => {
            let response = gateway.param_id068()?;
            Ok(build_checked_result(
                name,
                "ParamId068".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId588 { checks } => {
            let response = gateway.param_id588()?;
            Ok(build_checked_result(
                name,
                "ParamId588".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId654 { checks } => {
            let response = gateway.param_id654()?;
            Ok(build_checked_result(
                name,
                "ParamId654".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId272 { checks } => {
            let response = gateway.param_id272()?;
            Ok(build_checked_result(
                name,
                "ParamId272".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId080 { checks } => {
            let response = gateway.param_id080()?;
            Ok(build_checked_result(
                name,
                "ParamId080".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId606 {
            front_light_mode,
            power,
        } => {
            gateway.param_id606(front_light_mode, power)?;
            Ok(build_action_result(
                name,
                "ParamId606".to_string(),
                format!(
                    "FrontLightMode={}, Power={}, ReturnCode=0",
                    front_light_mode, power
                ),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::run_group;
    use crate::device_gateway::DeviceGateway;
    use crate::models::{
        CommandGroupSpec, ParamId068Check, ParamId068Output, ParamId068Result, ParamId080Result,
        ParamId272Result, ParamId588Result, ParamId654Result, TestGroup,
    };
    use crate::types::CommandResult;

    struct FakeGateway {
        result_068: ParamId068Result,
        called_606: Cell<bool>,
    }

    impl DeviceGateway for FakeGateway {
        fn param_id068(&self) -> CommandResult<ParamId068Result> {
            Ok(self.result_068)
        }

        fn param_id588(&self) -> CommandResult<ParamId588Result> {
            panic!("not used in this test")
        }

        fn param_id654(&self) -> CommandResult<ParamId654Result> {
            panic!("not used in this test")
        }

        fn param_id272(&self) -> CommandResult<ParamId272Result> {
            panic!("not used in this test")
        }

        fn param_id080(&self) -> CommandResult<ParamId080Result> {
            panic!("not used in this test")
        }

        fn param_id606(&self, _front_light_mode: u8, _power: u8) -> CommandResult<()> {
            self.called_606.set(true);
            Ok(())
        }
    }

    #[test]
    fn run_group_param_id068_passes_when_value_in_range() {
        let gateway = FakeGateway {
            result_068: ParamId068Result {
                dev_gr_no: 0,
                sub_dev_gr_no: 0,
                var_no: 0,
                maj_par_sw_ver: 10,
                min_par_sw_ver: 0,
                build_no: 0,
            },
            called_606: Cell::new(false),
        };

        let group = TestGroup {
            name: "068 test".to_string(),
            command: CommandGroupSpec::ParamId068 {
                checks: vec![ParamId068Check {
                    name: "maj".to_string(),
                    output: ParamId068Output::MajParSwVer,
                    min: 5.0,
                    max: 20.0,
                }],
            },
        };

        let result = run_group(&gateway, group).expect("group should run");
        assert!(result.passed);
        assert_eq!(result.checks.len(), 1);
        assert!(result.checks[0].passed);
        assert_eq!(result.command, "ParamId068");
    }

    #[test]
    fn run_group_param_id068_fails_when_value_out_of_range() {
        let gateway = FakeGateway {
            result_068: ParamId068Result {
                dev_gr_no: 0,
                sub_dev_gr_no: 0,
                var_no: 0,
                maj_par_sw_ver: 3,
                min_par_sw_ver: 0,
                build_no: 0,
            },
            called_606: Cell::new(false),
        };

        let group = TestGroup {
            name: "068 test".to_string(),
            command: CommandGroupSpec::ParamId068 {
                checks: vec![ParamId068Check {
                    name: "maj".to_string(),
                    output: ParamId068Output::MajParSwVer,
                    min: 5.0,
                    max: 20.0,
                }],
            },
        };

        let result = run_group(&gateway, group).expect("group should run");
        assert!(!result.passed);
        assert_eq!(result.checks.len(), 1);
        assert!(!result.checks[0].passed);
    }

    #[test]
    fn run_group_param_id606_calls_gateway() {
        let gateway = FakeGateway {
            result_068: ParamId068Result {
                dev_gr_no: 0,
                sub_dev_gr_no: 0,
                var_no: 0,
                maj_par_sw_ver: 0,
                min_par_sw_ver: 0,
                build_no: 0,
            },
            called_606: Cell::new(false),
        };

        let group = TestGroup {
            name: "606 test".to_string(),
            command: CommandGroupSpec::ParamId606 {
                front_light_mode: 1,
                power: 80,
            },
        };

        let result = run_group(&gateway, group).expect("group should run");
        assert!(result.passed);
        assert!(gateway.called_606.get());
        assert_eq!(result.command, "ParamId606");
    }
}
