use crate::device_gateway::DeviceGateway;
use crate::events::{FRONT_LIGHT_CONFIRM_REQUEST, KEY_STATE_UPDATE};
use crate::models::{
    CheckConfig, CheckResult, CheckableResult, CommandGroupSpec, FrontLightConfirmRequestPayload,
    KeyStatePayload, TestGroup, TestResult,
};
use crate::types::{AppError, CommandResult};
use std::fmt::Display;
use std::sync::{Condvar, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};

const PARAM_ID470_MAX_RETRIES: u32 = 5;
#[cfg(test)]
const PARAM_ID470_RETRY_DELAY_MS: u64 = 0;
#[cfg(not(test))]
const PARAM_ID470_RETRY_DELAY_MS: u64 = 1000;

#[derive(Debug)]
struct FrontLightConfirmState {
    waiting: bool,
    response: Option<bool>,
}

static FRONT_LIGHT_CONFIRM_SYNC: OnceLock<(Mutex<FrontLightConfirmState>, Condvar)> =
    OnceLock::new();

fn front_light_confirm_sync() -> &'static (Mutex<FrontLightConfirmState>, Condvar) {
    FRONT_LIGHT_CONFIRM_SYNC.get_or_init(|| {
        (
            Mutex::new(FrontLightConfirmState {
                waiting: false,
                response: None,
            }),
            Condvar::new(),
        )
    })
}

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
    stage: String,
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
        stage,
        command,
        passed,
        raw_response: response.to_string(),
        checks: check_results,
    }
}

fn build_action_result(
    group_name: String,
    stage: String,
    command: String,
    raw_response: String,
) -> TestResult {
    TestResult {
        name: group_name,
        stage,
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

fn build_front_light_result(
    group_name: String,
    stage: String,
    front_light_mode: u8,
    power: u8,
    is_lit: bool,
) -> TestResult {
    TestResult {
        name: group_name,
        stage,
        command: "ParamId606".to_string(),
        passed: is_lit,
        raw_response: format!(
            "FrontLightMode={}, Power={}, ReturnCode=0, LightOn={}",
            front_light_mode,
            power,
            if is_lit { 1 } else { 0 }
        ),
        checks: vec![CheckResult {
            name: "light_confirmed".to_string(),
            min: None,
            max: None,
            value: Some(if is_lit { 1.0 } else { 0.0 }),
            passed: is_lit,
        }],
    }
}

fn build_checked_result_with_retry<TConfig, TResult, F>(
    group_name: String,
    stage: String,
    command: String,
    checks: &[TConfig],
    mut fetch_result: F,
    max_retries: u32,
    retry_delay_ms: u64,
) -> CommandResult<TestResult>
where
    TConfig: CheckConfig,
    TResult: CheckableResult<OutputEnum = TConfig::OutputEnum> + Display,
    F: FnMut() -> CommandResult<TResult>,
{
    let mut response = fetch_result()?;
    let mut check_results = process_checks(checks, &response);
    let mut retry_count = 0_u32;

    while retry_count < max_retries && check_results.iter().any(|item| !item.passed) {
        warn!(
            "Command {} check failed, retry {}/{} in {}ms",
            command,
            retry_count + 1,
            max_retries,
            retry_delay_ms
        );
        thread::sleep(Duration::from_millis(retry_delay_ms));
        response = fetch_result()?;
        check_results = process_checks(checks, &response);
        retry_count += 1;
    }

    let passed = check_results.iter().all(|item| item.passed);
    let mut raw_response = response.to_string();
    if retry_count > 0 {
        raw_response = format!("{raw_response}, Retries={retry_count}");
        if passed {
            info!("Command {} passed after {} retries", command, retry_count);
        } else {
            warn!(
                "Command {} still failed after {} retries",
                command, retry_count
            );
        }
    }

    Ok(TestResult {
        name: group_name,
        stage,
        command,
        passed,
        raw_response,
        checks: check_results,
    })
}

pub fn run_group(
    gateway: &dyn DeviceGateway,
    group: TestGroup,
    app: &AppHandle,
) -> CommandResult<TestResult> {
    run_group_with_emitters(
        gateway,
        group,
        &|state| {
            if let Err(err) = app.emit(KEY_STATE_UPDATE, &state) {
                error!("Failed to emit key state update: {}", err);
            }
        },
        &|request| wait_for_front_light_confirmation(app, request),
    )
}

pub fn submit_front_light_confirmation(is_lit: bool) -> CommandResult<()> {
    let (lock, cvar) = front_light_confirm_sync();
    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("前灯确认状态锁定失败"))?;
    if !state.waiting {
        return Err(AppError::msg("当前没有待确认的前灯测试"));
    }
    state.response = Some(is_lit);
    cvar.notify_one();
    Ok(())
}

fn wait_for_front_light_confirmation(
    app: &AppHandle,
    request: FrontLightConfirmRequestPayload,
) -> CommandResult<bool> {
    let (lock, cvar) = front_light_confirm_sync();
    {
        let mut state = lock
            .lock()
            .map_err(|_| AppError::msg("前灯确认状态锁定失败"))?;
        state.waiting = true;
        state.response = None;
    }

    app.emit(FRONT_LIGHT_CONFIRM_REQUEST, &request)
        .map_err(|err| AppError::msg(format!("发送前灯确认事件失败: {err}")))?;

    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("前灯确认状态锁定失败"))?;
    while state.response.is_none() {
        state = cvar
            .wait(state)
            .map_err(|_| AppError::msg("前灯确认等待失败"))?;
    }
    let result = state.response.take().unwrap_or(false);
    state.waiting = false;
    Ok(result)
}

fn run_group_with_emitters(
    gateway: &dyn DeviceGateway,
    group: TestGroup,
    on_key_state_update: &dyn Fn(KeyStatePayload),
    on_front_light_confirm: &dyn Fn(FrontLightConfirmRequestPayload) -> CommandResult<bool>,
) -> CommandResult<TestResult> {
    let TestGroup {
        name,
        stage,
        command,
    } = group;
    match command {
        CommandGroupSpec::ParamId068 { checks } => {
            let response = gateway.param_id068()?;
            Ok(build_checked_result(
                name,
                stage,
                "ParamId068".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId588 { checks } => {
            let response = gateway.param_id588()?;
            Ok(build_checked_result(
                name,
                stage,
                "ParamId588".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId654 { checks } => {
            let response = gateway.param_id654()?;
            Ok(build_checked_result(
                name,
                stage,
                "ParamId654".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId272 { checks } => {
            let response = gateway.param_id272()?;
            Ok(build_checked_result(
                name,
                stage,
                "ParamId272".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId080 { checks } => {
            let response = gateway.param_id080()?;
            Ok(build_checked_result(
                name,
                stage,
                "ParamId080".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId120 { checks } => {
            let response = gateway.param_id120()?;
            Ok(build_checked_result(
                name,
                stage,
                "ParamId120".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId122 { checks } => {
            let response = gateway.param_id122()?;
            Ok(build_checked_result(
                name,
                stage,
                "ParamId122".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId470 { checks } => build_checked_result_with_retry(
            name,
            stage,
            "ParamId470".to_string(),
            &checks,
            || gateway.param_id470(),
            PARAM_ID470_MAX_RETRIES,
            PARAM_ID470_RETRY_DELAY_MS,
        ),
        CommandGroupSpec::ParamId468 { cutting_height_mm } => {
            gateway.param_id468(cutting_height_mm)?;
            Ok(build_action_result(
                name,
                stage,
                "ParamId468".to_string(),
                format!("CuttingHeightMm={}, ReturnCode=0", cutting_height_mm),
            ))
        }
        CommandGroupSpec::CuttingHeightSetAndVerify {
            cutting_height_mm,
            wait_ms,
            checks,
        } => {
            gateway.param_id468(cutting_height_mm)?;
            thread::sleep(Duration::from_millis(wait_ms));
            build_checked_result_with_retry(
                name,
                stage,
                "CuttingHeightSetAndVerify".to_string(),
                &checks,
                || gateway.param_id470(),
                PARAM_ID470_MAX_RETRIES,
                PARAM_ID470_RETRY_DELAY_MS,
            )
        }
        CommandGroupSpec::ParamId606 {
            front_light_mode,
            power,
        } => {
            gateway.param_id606(front_light_mode, power)?;
            let is_lit = on_front_light_confirm(FrontLightConfirmRequestPayload {
                name: name.clone(),
                stage: stage.clone(),
                front_light_mode,
                power,
            })?;
            Ok(build_front_light_result(
                name,
                stage,
                front_light_mode,
                power,
                is_lit,
            ))
        }
        CommandGroupSpec::ParamId794 { checks } => {
            let response = gateway.param_id794()?;
            Ok(build_checked_result(
                name,
                stage,
                "ParamId794".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId776 { timeout_ms } => {
            run_key_test_group(gateway, name, stage, timeout_ms, on_key_state_update)
        }
    }
}

const KEY_PRESSED_THRESHOLD: u8 = 2;
#[cfg(test)]
const KEY_TEST_POLL_INTERVAL_MS: u64 = 0;
#[cfg(not(test))]
const KEY_TEST_POLL_INTERVAL_MS: u64 = 1000;

fn run_key_test_group(
    gateway: &dyn DeviceGateway,
    name: String,
    stage: String,
    timeout_ms: u64,
    on_state_update: &dyn Fn(KeyStatePayload),
) -> CommandResult<TestResult> {
    info!("Starting key test: {}", name);
    gateway.param_id776(0)?;

    let mut up = false;
    let mut down = false;
    let mut back = false;
    let mut confirm = false;

    let elapsed_limit = if timeout_ms == 0 {
        u64::MAX
    } else {
        timeout_ms
    };
    let mut elapsed_ms: u64 = 0;

    loop {
        thread::sleep(Duration::from_millis(KEY_TEST_POLL_INTERVAL_MS));
        elapsed_ms = elapsed_ms.saturating_add(KEY_TEST_POLL_INTERVAL_MS.max(1));

        let result = gateway.param_id776(1)?;
        if result.up_key >= KEY_PRESSED_THRESHOLD {
            up = true;
        }
        if result.down_key >= KEY_PRESSED_THRESHOLD {
            down = true;
        }
        if result.back_key >= KEY_PRESSED_THRESHOLD {
            back = true;
        }
        if result.confirm_key >= KEY_PRESSED_THRESHOLD {
            confirm = true;
        }

        on_state_update(KeyStatePayload {
            up_pressed: up,
            down_pressed: down,
            back_pressed: back,
            confirm_pressed: confirm,
        });

        if up && down && back && confirm {
            info!("Key test passed: all keys pressed");
            return Ok(TestResult {
                name,
                stage,
                command: "ParamId776".to_string(),
                passed: true,
                raw_response: format!(
                    "UpKey={}, DownKey={}, BackKey={}, ConfirmKey={}",
                    result.up_key, result.down_key, result.back_key, result.confirm_key
                ),
                checks: vec![CheckResult {
                    name: "all_keys_pressed".to_string(),
                    min: None,
                    max: None,
                    value: None,
                    passed: true,
                }],
            });
        }

        if elapsed_ms >= elapsed_limit {
            warn!("Key test timed out after {}ms", elapsed_ms);
            return Ok(TestResult {
                name,
                stage,
                command: "ParamId776".to_string(),
                passed: false,
                raw_response: format!(
                    "Timeout={}ms, UpKey={}, DownKey={}, BackKey={}, ConfirmKey={}",
                    elapsed_ms, result.up_key, result.down_key, result.back_key, result.confirm_key
                ),
                checks: vec![CheckResult {
                    name: "all_keys_pressed".to_string(),
                    min: None,
                    max: None,
                    value: None,
                    passed: false,
                }],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::run_group_with_emitters;
    use crate::device_gateway::DeviceGateway;
    use crate::models::{
        CommandGroupSpec, KeyStatePayload, ParamId068Check, ParamId068Output, ParamId068Result,
        ParamId080Result, ParamId120Result, ParamId122Result, ParamId272Result, ParamId470Result,
        ParamId588Result, ParamId654Result, ParamId776Result, ParamId794Result, TestGroup,
    };
    use crate::types::CommandResult;

    struct FakeGateway {
        result_068: ParamId068Result,
        result_470_sequence: Vec<u8>,
        called_470: Cell<usize>,
        called_468: Cell<bool>,
        called_606: Cell<bool>,
    }

    impl DeviceGateway for FakeGateway {
        fn param_id374(&self, _test_mode: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

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

        fn param_id120(&self) -> CommandResult<ParamId120Result> {
            panic!("not used in this test")
        }

        fn param_id122(&self) -> CommandResult<ParamId122Result> {
            panic!("not used in this test")
        }

        fn param_id470(&self) -> CommandResult<ParamId470Result> {
            let idx = self.called_470.get();
            let value = self
                .result_470_sequence
                .get(idx)
                .copied()
                .or_else(|| self.result_470_sequence.last().copied())
                .unwrap_or(0);
            self.called_470.set(idx + 1);
            Ok(ParamId470Result {
                cutting_height_mm: value,
            })
        }

        fn param_id468(&self, _cutting_height_mm: u8) -> CommandResult<()> {
            self.called_468.set(true);
            Ok(())
        }

        fn param_id606(&self, _front_light_mode: u8, _power: u8) -> CommandResult<()> {
            self.called_606.set(true);
            Ok(())
        }

        fn param_id794(&self) -> CommandResult<ParamId794Result> {
            panic!("not used in this test")
        }

        fn param_id776(&self, _cmd: u8) -> CommandResult<ParamId776Result> {
            panic!("not used in this test")
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
            result_470_sequence: vec![30],
            called_470: Cell::new(0),
            called_468: Cell::new(false),
            called_606: Cell::new(false),
        };

        let group = TestGroup {
            name: "068 test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId068 {
                checks: vec![ParamId068Check {
                    name: "maj".to_string(),
                    output: ParamId068Output::MajParSwVer,
                    min: 5.0,
                    max: 20.0,
                }],
            },
        };

        let result = run_group_with_emitters(&gateway, group, &|_| {}, &|_| Ok(true))
            .expect("group should run");
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
            result_470_sequence: vec![30],
            called_470: Cell::new(0),
            called_468: Cell::new(false),
            called_606: Cell::new(false),
        };

        let group = TestGroup {
            name: "068 test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId068 {
                checks: vec![ParamId068Check {
                    name: "maj".to_string(),
                    output: ParamId068Output::MajParSwVer,
                    min: 5.0,
                    max: 20.0,
                }],
            },
        };

        let result = run_group_with_emitters(&gateway, group, &|_| {}, &|_| Ok(true))
            .expect("group should run");
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
            result_470_sequence: vec![30],
            called_470: Cell::new(0),
            called_468: Cell::new(false),
            called_606: Cell::new(false),
        };

        let group = TestGroup {
            name: "606 test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId606 {
                front_light_mode: 1,
                power: 80,
            },
        };

        let result = run_group_with_emitters(&gateway, group, &|_| {}, &|_| Ok(true))
            .expect("group should run");
        assert!(result.passed);
        assert!(gateway.called_606.get());
        assert_eq!(result.command, "ParamId606");
        assert!(result.raw_response.contains("LightOn=1"));
    }

    #[test]
    fn run_group_param_id606_fails_when_not_confirmed() {
        let gateway = FakeGateway {
            result_068: ParamId068Result {
                dev_gr_no: 0,
                sub_dev_gr_no: 0,
                var_no: 0,
                maj_par_sw_ver: 0,
                min_par_sw_ver: 0,
                build_no: 0,
            },
            result_470_sequence: vec![30],
            called_470: Cell::new(0),
            called_468: Cell::new(false),
            called_606: Cell::new(false),
        };

        let group = TestGroup {
            name: "606 test fail".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId606 {
                front_light_mode: 1,
                power: 80,
            },
        };

        let result = run_group_with_emitters(&gateway, group, &|_| {}, &|_| Ok(false))
            .expect("group should run");
        assert!(!result.passed);
        assert!(gateway.called_606.get());
        assert_eq!(result.command, "ParamId606");
        assert!(result.raw_response.contains("LightOn=0"));
    }

    #[test]
    fn run_group_param_id468_then_470_sets_and_checks_height() {
        let gateway = FakeGateway {
            result_068: ParamId068Result {
                dev_gr_no: 0,
                sub_dev_gr_no: 0,
                var_no: 0,
                maj_par_sw_ver: 0,
                min_par_sw_ver: 0,
                build_no: 0,
            },
            result_470_sequence: vec![30],
            called_470: Cell::new(0),
            called_468: Cell::new(false),
            called_606: Cell::new(false),
        };

        let group = TestGroup {
            name: "468-470 test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::CuttingHeightSetAndVerify {
                cutting_height_mm: 30,
                wait_ms: 0,
                checks: vec![crate::models::ParamId470Check {
                    name: "height".to_string(),
                    output: crate::models::ParamId470Output::CuttingHeightMm,
                    min: 30.0,
                    max: 30.0,
                }],
            },
        };

        let result = run_group_with_emitters(&gateway, group, &|_| {}, &|_| Ok(true))
            .expect("group should run");
        assert!(gateway.called_468.get());
        assert!(result.passed);
        assert_eq!(result.command, "CuttingHeightSetAndVerify");
    }

    #[test]
    fn run_group_param_id470_retries_until_threshold_passes() {
        let gateway = FakeGateway {
            result_068: ParamId068Result {
                dev_gr_no: 0,
                sub_dev_gr_no: 0,
                var_no: 0,
                maj_par_sw_ver: 0,
                min_par_sw_ver: 0,
                build_no: 0,
            },
            result_470_sequence: vec![26, 27, 30],
            called_470: Cell::new(0),
            called_468: Cell::new(false),
            called_606: Cell::new(false),
        };

        let group = TestGroup {
            name: "470 retry test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId470 {
                checks: vec![crate::models::ParamId470Check {
                    name: "height".to_string(),
                    output: crate::models::ParamId470Output::CuttingHeightMm,
                    min: 30.0,
                    max: 30.0,
                }],
            },
        };

        let result = run_group_with_emitters(&gateway, group, &|_| {}, &|_| Ok(true))
            .expect("group should run");
        assert!(result.passed);
        assert_eq!(gateway.called_470.get(), 3);
    }

    struct FakeKeyGateway {
        responses: Vec<ParamId776Result>,
        called_start: Cell<bool>,
        called_poll: Cell<usize>,
    }

    impl DeviceGateway for FakeKeyGateway {
        fn param_id374(&self, _test_mode: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id068(&self) -> CommandResult<ParamId068Result> {
            panic!("not used in this test")
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

        fn param_id120(&self) -> CommandResult<ParamId120Result> {
            panic!("not used in this test")
        }

        fn param_id122(&self) -> CommandResult<ParamId122Result> {
            panic!("not used in this test")
        }

        fn param_id470(&self) -> CommandResult<ParamId470Result> {
            panic!("not used in this test")
        }

        fn param_id468(&self, _cutting_height_mm: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id606(&self, _front_light_mode: u8, _power: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id794(&self) -> CommandResult<ParamId794Result> {
            panic!("not used in this test")
        }

        fn param_id776(&self, cmd: u8) -> CommandResult<ParamId776Result> {
            match cmd {
                0 => {
                    self.called_start.set(true);
                    Ok(ParamId776Result {
                        up_key: 0,
                        down_key: 0,
                        back_key: 0,
                        confirm_key: 0,
                    })
                }
                1 => {
                    let idx = self.called_poll.get();
                    let value = self
                        .responses
                        .get(idx)
                        .copied()
                        .or_else(|| self.responses.last().copied())
                        .unwrap_or(ParamId776Result {
                            up_key: 0,
                            down_key: 0,
                            back_key: 0,
                            confirm_key: 0,
                        });
                    self.called_poll.set(idx + 1);
                    Ok(value)
                }
                _ => panic!("unexpected cmd"),
            }
        }
    }

    #[test]
    fn run_group_param_id776_passes_and_emits_updates() {
        let gateway = FakeKeyGateway {
            responses: vec![
                ParamId776Result {
                    up_key: 2,
                    down_key: 0,
                    back_key: 0,
                    confirm_key: 0,
                },
                ParamId776Result {
                    up_key: 2,
                    down_key: 2,
                    back_key: 2,
                    confirm_key: 2,
                },
            ],
            called_start: Cell::new(false),
            called_poll: Cell::new(0),
        };

        let group = TestGroup {
            name: "776 pass".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId776 { timeout_ms: 10 },
        };

        let updates = RefCell::new(Vec::<KeyStatePayload>::new());
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|state| {
                updates.borrow_mut().push(state);
            },
            &|_| Ok(true),
        )
        .expect("group should run");

        assert!(gateway.called_start.get());
        assert!(result.passed);
        assert_eq!(result.command, "ParamId776");
        assert_eq!(updates.borrow().len(), 2);
        assert!(updates.borrow().last().expect("has update").confirm_pressed);
    }

    #[test]
    fn run_group_param_id776_times_out() {
        let gateway = FakeKeyGateway {
            responses: vec![ParamId776Result {
                up_key: 0,
                down_key: 0,
                back_key: 0,
                confirm_key: 0,
            }],
            called_start: Cell::new(false),
            called_poll: Cell::new(0),
        };

        let group = TestGroup {
            name: "776 timeout".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId776 { timeout_ms: 1 },
        };

        let update_count = Cell::new(0usize);
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {
                update_count.set(update_count.get() + 1);
            },
            &|_| Ok(true),
        )
        .expect("group should run");

        assert!(gateway.called_start.get());
        assert!(!result.passed);
        assert_eq!(result.command, "ParamId776");
        assert_eq!(update_count.get(), 1);
    }
}
