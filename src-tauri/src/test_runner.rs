use crate::device_gateway::DeviceGateway;
use crate::events::{
    COLLISION_BAR_PROMPT_REQUEST, EMERGENCY_STOP_TEST_UPDATE, FRONT_LIGHT_CONFIRM_REQUEST,
    KEY_STATE_UPDATE, REAR_LIGHT_CONFIRM_REQUEST, SPEAKER_CONFIRM_REQUEST, WHEEL_MOTOR_TEST_UPDATE,
};
use crate::models::{
    CheckConfig, CheckResult, CheckableResult, CollisionBarPromptPayload, CommandGroupSpec,
    EmergencyStopPhase, EmergencyStopTestPayload, FrontLightConfirmRequestPayload, KeyStatePayload,
    RearLightColor, RearLightConfirmRequestPayload, SensorPromptKind, SpeakerConfirmRequestPayload,
    TestGroup, TestResult, VersionCheck, WheelMotorCheck, WheelMotorOutput, WheelMotorTestPhase,
    WheelMotorTestUpdatePayload,
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
#[cfg(test)]
const PARAM_ID118_POLL_INTERVAL_MS: u64 = 0;
#[cfg(not(test))]
const PARAM_ID118_POLL_INTERVAL_MS: u64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SemanticVersion {
    major: u8,
    minor: u8,
    patch: u32,
}

impl SemanticVersion {
    fn parse(input: &str) -> CommandResult<Self> {
        let trimmed = input.trim();
        let mut parts = trimmed.split('.');
        let major = parts
            .next()
            .ok_or_else(|| AppError::msg(format!("无效版本号: {trimmed}")))?
            .parse::<u8>()
            .map_err(|err| AppError::msg(format!("无效版本号 {trimmed}: {err}")))?;
        let minor = parts
            .next()
            .ok_or_else(|| AppError::msg(format!("无效版本号: {trimmed}")))?
            .parse::<u8>()
            .map_err(|err| AppError::msg(format!("无效版本号 {trimmed}: {err}")))?;
        let patch = parts
            .next()
            .ok_or_else(|| AppError::msg(format!("无效版本号: {trimmed}")))?
            .parse::<u32>()
            .map_err(|err| AppError::msg(format!("无效版本号 {trimmed}: {err}")))?;

        if parts.next().is_some() {
            return Err(AppError::msg(format!("无效版本号: {trimmed}")));
        }

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug)]
struct FrontLightConfirmState {
    waiting: bool,
    response: Option<bool>,
}

static FRONT_LIGHT_CONFIRM_SYNC: OnceLock<(Mutex<FrontLightConfirmState>, Condvar)> =
    OnceLock::new();

#[derive(Debug)]
struct RearLightConfirmState {
    waiting: bool,
    response: Option<bool>,
}

static REAR_LIGHT_CONFIRM_SYNC: OnceLock<(Mutex<RearLightConfirmState>, Condvar)> = OnceLock::new();

#[derive(Debug)]
struct SpeakerConfirmState {
    waiting: bool,
    response: Option<bool>,
}

static SPEAKER_CONFIRM_SYNC: OnceLock<(Mutex<SpeakerConfirmState>, Condvar)> = OnceLock::new();

#[derive(Debug)]
struct KeyTestState {
    waiting: bool,
    canceled: bool,
}

static KEY_TEST_SYNC: OnceLock<(Mutex<KeyTestState>, Condvar)> = OnceLock::new();

#[derive(Debug)]
struct EmergencyStopTestState {
    waiting: bool,
    canceled: bool,
}

static EMERGENCY_STOP_TEST_SYNC: OnceLock<Mutex<EmergencyStopTestState>> = OnceLock::new();

#[derive(Debug)]
struct SensorPromptState {
    waiting: bool,
    canceled: bool,
}

static SENSOR_PROMPT_SYNC: OnceLock<Mutex<SensorPromptState>> = OnceLock::new();

#[derive(Debug)]
struct WheelMotorLiftConfirmState {
    waiting: bool,
    response: Option<bool>,
}

static WHEEL_MOTOR_LIFT_CONFIRM_SYNC: OnceLock<(Mutex<WheelMotorLiftConfirmState>, Condvar)> =
    OnceLock::new();

const REAR_LIGHT_NORMAL_MODE: u8 = 3;
const REAR_LIGHT_CONFIRM_STEPS: [(u8, RearLightColor); 3] = [
    (4, RearLightColor::Red),
    (5, RearLightColor::Green),
    (6, RearLightColor::Blue),
];
#[cfg(test)]
const EMERGENCY_STOP_POLL_INTERVAL_MS: u64 = 0;
#[cfg(not(test))]
const EMERGENCY_STOP_POLL_INTERVAL_MS: u64 = 1000;

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

fn rear_light_confirm_sync() -> &'static (Mutex<RearLightConfirmState>, Condvar) {
    REAR_LIGHT_CONFIRM_SYNC.get_or_init(|| {
        (
            Mutex::new(RearLightConfirmState {
                waiting: false,
                response: None,
            }),
            Condvar::new(),
        )
    })
}

fn speaker_confirm_sync() -> &'static (Mutex<SpeakerConfirmState>, Condvar) {
    SPEAKER_CONFIRM_SYNC.get_or_init(|| {
        (
            Mutex::new(SpeakerConfirmState {
                waiting: false,
                response: None,
            }),
            Condvar::new(),
        )
    })
}

fn key_test_sync() -> &'static (Mutex<KeyTestState>, Condvar) {
    KEY_TEST_SYNC.get_or_init(|| {
        (
            Mutex::new(KeyTestState {
                waiting: false,
                canceled: false,
            }),
            Condvar::new(),
        )
    })
}

fn start_key_test_session() -> CommandResult<()> {
    let (lock, _) = key_test_sync();
    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("按键测试状态锁定失败"))?;
    state.waiting = true;
    state.canceled = false;
    Ok(())
}

fn finish_key_test_session() {
    if let Ok(mut state) = key_test_sync().0.lock() {
        state.waiting = false;
        state.canceled = false;
    }
}

fn is_key_test_canceled() -> CommandResult<bool> {
    let state = key_test_sync()
        .0
        .lock()
        .map_err(|_| AppError::msg("按键测试状态锁定失败"))?;
    Ok(state.canceled)
}

fn wait_key_test_or_cancel(wait_ms: u64) -> CommandResult<bool> {
    let (lock, cvar) = key_test_sync();
    let state = lock
        .lock()
        .map_err(|_| AppError::msg("按键测试状态锁定失败"))?;
    if wait_ms == 0 {
        return Ok(state.canceled);
    }
    let (state, _) = cvar
        .wait_timeout(state, Duration::from_millis(wait_ms))
        .map_err(|_| AppError::msg("按键测试等待失败"))?;
    Ok(state.canceled)
}

fn emergency_stop_test_sync() -> &'static Mutex<EmergencyStopTestState> {
    EMERGENCY_STOP_TEST_SYNC.get_or_init(|| {
        Mutex::new(EmergencyStopTestState {
            waiting: false,
            canceled: false,
        })
    })
}

fn sensor_prompt_sync() -> &'static Mutex<SensorPromptState> {
    SENSOR_PROMPT_SYNC.get_or_init(|| {
        Mutex::new(SensorPromptState {
            waiting: false,
            canceled: false,
        })
    })
}

fn wheel_motor_lift_confirm_sync() -> &'static (Mutex<WheelMotorLiftConfirmState>, Condvar) {
    WHEEL_MOTOR_LIFT_CONFIRM_SYNC.get_or_init(|| {
        (
            Mutex::new(WheelMotorLiftConfirmState {
                waiting: false,
                response: None,
            }),
            Condvar::new(),
        )
    })
}

fn start_sensor_prompt_session() -> CommandResult<()> {
    let lock = sensor_prompt_sync();
    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("传感器提示状态锁定失败"))?;
    state.waiting = true;
    state.canceled = false;
    Ok(())
}

fn finish_sensor_prompt_session() {
    if let Ok(mut state) = sensor_prompt_sync().lock() {
        state.waiting = false;
        state.canceled = false;
    }
}

fn is_sensor_prompt_canceled() -> CommandResult<bool> {
    let state = sensor_prompt_sync()
        .lock()
        .map_err(|_| AppError::msg("传感器提示状态锁定失败"))?;
    Ok(state.canceled)
}

fn start_emergency_stop_test_session() -> CommandResult<()> {
    let lock = emergency_stop_test_sync();
    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("急停测试状态锁定失败"))?;
    state.waiting = true;
    state.canceled = false;
    Ok(())
}

fn finish_emergency_stop_test_session() {
    if let Ok(mut state) = emergency_stop_test_sync().lock() {
        state.waiting = false;
        state.canceled = false;
    }
}

fn is_emergency_stop_test_canceled() -> CommandResult<bool> {
    let state = emergency_stop_test_sync()
        .lock()
        .map_err(|_| AppError::msg("急停测试状态锁定失败"))?;
    Ok(state.canceled)
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
                display_min: None,
                display_max: None,
                display_value: None,
                passed,
            }
        })
        .collect()
}

trait VersionedResult {
    fn semantic_version(&self) -> SemanticVersion;
}

fn build_version_checked_result<TResult>(
    group_name: String,
    stage: String,
    command: String,
    checks: &[VersionCheck],
    response: &TResult,
) -> CommandResult<TestResult>
where
    TResult: VersionedResult + Display,
{
    if checks.len() != 1 {
        return Err(AppError::msg(format!(
            "{command} 版本检查需要且仅允许 1 条 check"
        )));
    }

    let check = &checks[0];
    let min = SemanticVersion::parse(&check.min)?;
    let max = SemanticVersion::parse(&check.max)?;

    if min > max {
        return Err(AppError::msg(format!(
            "{command} 版本检查配置非法: min {} 大于 max {}",
            check.min, check.max
        )));
    }

    let actual = response.semantic_version();
    let actual_display = actual.to_string();
    let passed = actual >= min && actual <= max;

    Ok(TestResult {
        name: group_name,
        names: Default::default(),
        stage,
        command,
        passed,
        raw_response: response.to_string(),
        checks: vec![CheckResult {
            name: check.name.clone(),
            min: None,
            max: None,
            value: None,
            display_min: Some(check.min.clone()),
            display_max: Some(check.max.clone()),
            display_value: Some(actual_display),
            passed,
        }],
    })
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
        names: Default::default(),
        stage,
        command,
        passed,
        raw_response: response.to_string(),
        checks: check_results,
    }
}

impl VersionedResult for crate::models::ParamId068Result {
    fn semantic_version(&self) -> SemanticVersion {
        SemanticVersion {
            major: self.maj_par_sw_ver,
            minor: self.min_par_sw_ver,
            patch: self.build_no,
        }
    }
}

impl VersionedResult for crate::models::ParamId588Result {
    fn semantic_version(&self) -> SemanticVersion {
        SemanticVersion {
            major: self.maj_par_sw_ver,
            minor: self.min_par_sw_ver,
            patch: self.build_no,
        }
    }
}

impl VersionedResult for crate::models::ParamId654Result {
    fn semantic_version(&self) -> SemanticVersion {
        SemanticVersion {
            major: self.maj_par_sw_ver,
            minor: self.min_par_sw_ver,
            patch: self.build_no,
        }
    }
}

impl VersionedResult for crate::models::ParamId794Result {
    fn semantic_version(&self) -> SemanticVersion {
        SemanticVersion {
            major: self.maj_par_sw_ver,
            minor: self.min_par_sw_ver,
            patch: self.build_no,
        }
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
        names: Default::default(),
        stage,
        command,
        passed: true,
        raw_response,
        checks: vec![CheckResult {
            name: "执行结果".to_string(),
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

fn build_param_id798_result(group_name: String, stage: String, version: String) -> TestResult {
    let has_version = !version.is_empty();
    TestResult {
        name: group_name,
        names: Default::default(),
        stage,
        command: "ParamId798".to_string(),
        passed: has_version,
        raw_response: format!("Version={version}"),
        checks: vec![CheckResult {
            name: "version_not_empty".to_string(),
            min: None,
            max: None,
            value: Some(if has_version { 1.0 } else { 0.0 }),
            display_min: None,
            display_max: None,
            display_value: Some(version),
            passed: has_version,
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
        names: Default::default(),
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
            display_min: None,
            display_max: None,
            display_value: None,
            passed: is_lit,
        }],
    }
}

fn build_speaker_result(
    group_name: String,
    stage: String,
    on: u8,
    heard_sound: bool,
) -> TestResult {
    TestResult {
        name: group_name,
        names: Default::default(),
        stage,
        command: "ParamId568".to_string(),
        passed: heard_sound,
        raw_response: format!(
            "On={}, ReturnCode=0, HeardSound={}",
            on,
            if heard_sound { 1 } else { 0 }
        ),
        checks: vec![CheckResult {
            name: "speaker_confirmed".to_string(),
            min: None,
            max: None,
            value: Some(if heard_sound { 1.0 } else { 0.0 }),
            display_min: None,
            display_max: None,
            display_value: None,
            passed: heard_sound,
        }],
    }
}

fn rear_light_color_name(color: RearLightColor) -> &'static str {
    match color {
        RearLightColor::Red => "red",
        RearLightColor::Green => "green",
        RearLightColor::Blue => "blue",
    }
}

fn build_rear_light_result(
    group_name: String,
    stage: String,
    confirmations: &[(u8, RearLightColor, bool)],
) -> TestResult {
    let passed = confirmations.iter().all(|(_, _, confirmed)| *confirmed);
    let sequence = confirmations
        .iter()
        .map(|(mode, color, confirmed)| {
            format!(
                "RearLightMode={mode}/Color={}/Confirmed={}",
                rear_light_color_name(*color),
                if *confirmed { 1 } else { 0 }
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let checks = confirmations
        .iter()
        .map(|(_, color, confirmed)| CheckResult {
            name: format!("{}_light_confirmed", rear_light_color_name(*color)),
            min: None,
            max: None,
            value: Some(if *confirmed { 1.0 } else { 0.0 }),
            display_min: None,
            display_max: None,
            display_value: None,
            passed: *confirmed,
        })
        .collect();

    TestResult {
        name: group_name,
        names: Default::default(),
        stage,
        command: "ParamId610".to_string(),
        passed,
        raw_response: format!(
            "{sequence}; RestoredToNormalMode={}",
            REAR_LIGHT_NORMAL_MODE
        ),
        checks,
    }
}

fn build_emergency_stop_result(
    group_name: String,
    stage: String,
    passed: bool,
    reason: &str,
    elapsed_ms: u64,
    timeout_ms: u64,
    mower_main_p: u8,
) -> TestResult {
    TestResult {
        name: group_name,
        names: Default::default(),
        stage,
        command: "ParamId080EmergencyStop".to_string(),
        passed,
        raw_response: format!(
            "MowerMainP={mower_main_p}, ElapsedMs={elapsed_ms}, TimeoutMs={timeout_ms}, Reason={reason}"
        ),
        checks: vec![CheckResult {
            name: "mower_main_p_reached_2".to_string(),
            min: Some(2.0),
            max: Some(2.0),
            value: Some(mower_main_p as f64),
            display_min: None,
            display_max: None,
            display_value: None,
            passed,
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
        names: Default::default(),
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
    run_group_with_emitters_internal(
        gateway,
        group,
        &|state| {
            if let Err(err) = app.emit(KEY_STATE_UPDATE, &state) {
                error!("Failed to emit key state update: {}", err);
            }
        },
        &|request| wait_for_front_light_confirmation(app, request),
        &|request| wait_for_rear_light_confirmation(app, request),
        &|request| wait_for_speaker_confirmation(app, request),
        &|request| wait_for_collision_bar_prompt(app, request),
        &|payload| {
            if let Err(err) = app.emit(EMERGENCY_STOP_TEST_UPDATE, &payload) {
                error!("Failed to emit emergency stop update: {}", err);
            }
        },
        &|request| wait_for_wheel_motor_lift_confirmation(app, request),
        &|payload| {
            if let Err(err) = app.emit(WHEEL_MOTOR_TEST_UPDATE, &payload) {
                error!("Failed to emit wheel motor test update: {}", err);
            }
            Ok(())
        },
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

pub fn submit_rear_light_confirmation(is_lit: bool) -> CommandResult<()> {
    let (lock, cvar) = rear_light_confirm_sync();
    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("尾灯确认状态锁定失败"))?;
    if !state.waiting {
        return Err(AppError::msg("当前没有待确认的尾灯测试"));
    }
    state.response = Some(is_lit);
    cvar.notify_one();
    Ok(())
}

pub fn submit_speaker_confirmation(heard_sound: bool) -> CommandResult<()> {
    let (lock, cvar) = speaker_confirm_sync();
    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("扬声器确认状态锁定失败"))?;
    if !state.waiting {
        return Err(AppError::msg("当前没有待确认的扬声器测试"));
    }
    state.response = Some(heard_sound);
    cvar.notify_one();
    Ok(())
}

pub fn submit_emergency_stop_cancel() -> CommandResult<()> {
    let lock = emergency_stop_test_sync();
    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("急停测试状态锁定失败"))?;
    if !state.waiting {
        return Err(AppError::msg("当前没有进行中的急停测试"));
    }
    state.canceled = true;
    Ok(())
}

pub fn submit_key_test_cancel() -> CommandResult<()> {
    let (lock, cvar) = key_test_sync();
    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("按键测试状态锁定失败"))?;
    if !state.waiting {
        return Err(AppError::msg("当前没有进行中的按键测试"));
    }
    state.canceled = true;
    cvar.notify_one();
    Ok(())
}

pub fn submit_sensor_prompt_cancel() -> CommandResult<()> {
    let lock = sensor_prompt_sync();
    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("传感器提示状态锁定失败"))?;
    if !state.waiting {
        return Err(AppError::msg("当前没有进行中的传感器提示测试"));
    }
    state.canceled = true;
    Ok(())
}

pub fn submit_wheel_motor_lift_confirmation(is_lifted: bool) -> CommandResult<()> {
    let (lock, cvar) = wheel_motor_lift_confirm_sync();
    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("轮电机抬起确认状态锁定失败"))?;
    if !state.waiting {
        return Err(AppError::msg("当前没有待确认的轮电机测试"));
    }
    state.response = Some(is_lifted);
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

fn wait_for_rear_light_confirmation(
    app: &AppHandle,
    request: RearLightConfirmRequestPayload,
) -> CommandResult<bool> {
    let (lock, cvar) = rear_light_confirm_sync();
    {
        let mut state = lock
            .lock()
            .map_err(|_| AppError::msg("尾灯确认状态锁定失败"))?;
        state.waiting = true;
        state.response = None;
    }

    app.emit(REAR_LIGHT_CONFIRM_REQUEST, &request)
        .map_err(|err| AppError::msg(format!("发送尾灯确认事件失败: {err}")))?;

    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("尾灯确认状态锁定失败"))?;
    while state.response.is_none() {
        state = cvar
            .wait(state)
            .map_err(|_| AppError::msg("尾灯确认等待失败"))?;
    }
    let result = state.response.take().unwrap_or(false);
    state.waiting = false;
    Ok(result)
}

fn wait_for_speaker_confirmation(
    app: &AppHandle,
    request: SpeakerConfirmRequestPayload,
) -> CommandResult<bool> {
    let (lock, cvar) = speaker_confirm_sync();
    {
        let mut state = lock
            .lock()
            .map_err(|_| AppError::msg("扬声器确认状态锁定失败"))?;
        state.waiting = true;
        state.response = None;
    }

    app.emit(SPEAKER_CONFIRM_REQUEST, &request)
        .map_err(|err| AppError::msg(format!("发送扬声器确认事件失败: {err}")))?;

    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("扬声器确认状态锁定失败"))?;
    while state.response.is_none() {
        state = cvar
            .wait(state)
            .map_err(|_| AppError::msg("扬声器确认等待失败"))?;
    }
    let result = state.response.take().unwrap_or(false);
    state.waiting = false;
    Ok(result)
}

fn wait_for_collision_bar_prompt(
    app: &AppHandle,
    request: CollisionBarPromptPayload,
) -> CommandResult<()> {
    app.emit(COLLISION_BAR_PROMPT_REQUEST, &request)
        .map_err(|err| AppError::msg(format!("发送碰撞条提示事件失败: {err}")))
}

fn wait_for_wheel_motor_lift_confirmation(
    app: &AppHandle,
    request: WheelMotorTestUpdatePayload,
) -> CommandResult<bool> {
    let (lock, cvar) = wheel_motor_lift_confirm_sync();
    {
        let mut state = lock
            .lock()
            .map_err(|_| AppError::msg("轮电机抬起确认状态锁定失败"))?;
        state.waiting = true;
        state.response = None;
    }

    app.emit(WHEEL_MOTOR_TEST_UPDATE, &request)
        .map_err(|err| AppError::msg(format!("发送轮电机抬起确认事件失败: {err}")))?;

    let mut state = lock
        .lock()
        .map_err(|_| AppError::msg("轮电机抬起确认状态锁定失败"))?;
    while state.response.is_none() {
        state = cvar
            .wait(state)
            .map_err(|_| AppError::msg("轮电机抬起确认等待失败"))?;
    }
    let result = state.response.take().unwrap_or(false);
    state.waiting = false;
    Ok(result)
}

#[cfg(test)]
fn run_group_with_emitters(
    gateway: &dyn DeviceGateway,
    group: TestGroup,
    on_key_state_update: &dyn Fn(KeyStatePayload),
    on_front_light_confirm: &dyn Fn(FrontLightConfirmRequestPayload) -> CommandResult<bool>,
    on_rear_light_confirm: &dyn Fn(RearLightConfirmRequestPayload) -> CommandResult<bool>,
    on_speaker_confirm: &dyn Fn(SpeakerConfirmRequestPayload) -> CommandResult<bool>,
    on_collision_bar_prompt: &dyn Fn(CollisionBarPromptPayload) -> CommandResult<()>,
    on_emergency_stop_update: &dyn Fn(EmergencyStopTestPayload),
) -> CommandResult<TestResult> {
    run_group_with_emitters_internal(
        gateway,
        group,
        on_key_state_update,
        on_front_light_confirm,
        on_rear_light_confirm,
        on_speaker_confirm,
        on_collision_bar_prompt,
        on_emergency_stop_update,
        &|_| Ok(true),
        &|_| Ok(()),
    )
}

fn run_group_with_emitters_internal(
    gateway: &dyn DeviceGateway,
    group: TestGroup,
    on_key_state_update: &dyn Fn(KeyStatePayload),
    on_front_light_confirm: &dyn Fn(FrontLightConfirmRequestPayload) -> CommandResult<bool>,
    on_rear_light_confirm: &dyn Fn(RearLightConfirmRequestPayload) -> CommandResult<bool>,
    on_speaker_confirm: &dyn Fn(SpeakerConfirmRequestPayload) -> CommandResult<bool>,
    on_collision_bar_prompt: &dyn Fn(CollisionBarPromptPayload) -> CommandResult<()>,
    on_emergency_stop_update: &dyn Fn(EmergencyStopTestPayload),
    on_wheel_motor_lift_confirm: &dyn Fn(WheelMotorTestUpdatePayload) -> CommandResult<bool>,
    on_wheel_motor_test_update: &dyn Fn(WheelMotorTestUpdatePayload) -> CommandResult<()>,
) -> CommandResult<TestResult> {
    let TestGroup {
        name,
        stage,
        command,
        names,
    } = group;
    let result = match command {
        CommandGroupSpec::ParamId068 { checks } => {
            let response = gateway.param_id068()?;
            build_version_checked_result(name, stage, "ParamId068".to_string(), &checks, &response)
        }
        CommandGroupSpec::ParamId588 { checks } => {
            let response = gateway.param_id588()?;
            build_version_checked_result(name, stage, "ParamId588".to_string(), &checks, &response)
        }
        CommandGroupSpec::ParamId654 { checks } => {
            let response = gateway.param_id654()?;
            build_version_checked_result(name, stage, "ParamId654".to_string(), &checks, &response)
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
        CommandGroupSpec::ParamId526 { checks } => {
            let response = gateway.param_id526()?;
            Ok(build_checked_result(
                name,
                stage,
                "ParamId526".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId096 { checks } => {
            let response = gateway.param_id096()?;
            Ok(build_checked_result(
                name,
                stage,
                "ParamId096".to_string(),
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
        CommandGroupSpec::ParamId080EmergencyStop { timeout_ms } => run_emergency_stop_test_group(
            gateway,
            name,
            stage,
            timeout_ms,
            on_emergency_stop_update,
        ),
        CommandGroupSpec::ParamId118CollisionBar { timeout_ms } => {
            run_collision_bar_test_group(gateway, name, stage, timeout_ms, on_collision_bar_prompt)
        }
        CommandGroupSpec::ParamId118LiftSensor {
            timeout_ms,
            lift_threshold,
        } => run_lift_sensor_test_group(
            gateway,
            name,
            stage,
            timeout_ms,
            lift_threshold,
            on_collision_bar_prompt,
        ),
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
            info!("close front light");
            gateway.param_id606(front_light_mode, 0)?;
            Ok(build_front_light_result(
                name,
                stage,
                front_light_mode,
                power,
                is_lit,
            ))
        }
        CommandGroupSpec::WheelMotorTest {
            right_motor_speed,
            left_motor_speed,
            sample_interval_ms,
            sample_count,
            right_test_inactive_max_speed_mm_s,
            left_test_inactive_max_speed_mm_s,
            checks,
        } => run_wheel_motor_test_group(
            gateway,
            name,
            stage,
            right_motor_speed,
            left_motor_speed,
            sample_interval_ms,
            sample_count,
            right_test_inactive_max_speed_mm_s,
            left_test_inactive_max_speed_mm_s,
            checks,
            on_wheel_motor_lift_confirm,
            on_wheel_motor_test_update,
        ),
        CommandGroupSpec::ParamId568 => {
            let on = 1;
            gateway.param_id568(on)?;
            let heard_sound = on_speaker_confirm(SpeakerConfirmRequestPayload {
                name: name.clone(),
                stage: stage.clone(),
                on,
            })?;
            Ok(build_speaker_result(name, stage, on, heard_sound))
        }
        CommandGroupSpec::ParamId610 => {
            let mut confirmations: Vec<(u8, RearLightColor, bool)> =
                Vec::with_capacity(REAR_LIGHT_CONFIRM_STEPS.len());

            let run_result = (|| -> CommandResult<()> {
                for (index, (rear_light_mode, expected_color)) in
                    REAR_LIGHT_CONFIRM_STEPS.iter().enumerate()
                {
                    gateway.param_id610(*rear_light_mode)?;
                    let confirmed = on_rear_light_confirm(RearLightConfirmRequestPayload {
                        name: name.clone(),
                        stage: stage.clone(),
                        rear_light_mode: *rear_light_mode,
                        expected_color: *expected_color,
                        step_index: (index + 1) as u8,
                        total_steps: REAR_LIGHT_CONFIRM_STEPS.len() as u8,
                    })?;
                    confirmations.push((*rear_light_mode, *expected_color, confirmed));
                    if !confirmed {
                        break;
                    }
                }
                Ok(())
            })();

            let restore_result = gateway.param_id610(REAR_LIGHT_NORMAL_MODE);
            match (run_result, restore_result) {
                (Ok(()), Ok(())) => Ok(build_rear_light_result(name, stage, &confirmations)),
                (Err(run_err), Ok(())) => Err(run_err),
                (Ok(()), Err(restore_err)) => Err(restore_err),
                (Err(run_err), Err(restore_err)) => Err(AppError::msg(format!(
                    "{run_err}; 且恢复尾灯模式失败: {restore_err}"
                ))),
            }
        }
        CommandGroupSpec::ParamId794 { checks } => {
            let response = gateway.param_id794()?;
            build_version_checked_result(name, stage, "ParamId794".to_string(), &checks, &response)
        }
        CommandGroupSpec::ParamId796 { checks } => {
            let response = gateway.param_id796()?;
            Ok(build_checked_result(
                name,
                stage,
                "ParamId796".to_string(),
                &checks,
                &response,
            ))
        }
        CommandGroupSpec::ParamId798 => {
            let response = gateway.param_id798()?;
            Ok(build_param_id798_result(name, stage, response.version))
        }
        CommandGroupSpec::ParamId776 { timeout_ms } => {
            run_key_test_group(gateway, name, stage, timeout_ms, on_key_state_update)
        }
    }?;
    Ok(TestResult { names, ..result })
}

fn abs_speed_as_f64(value: i16) -> f64 {
    i32::from(value).abs() as f64
}

fn collect_wheel_speed_avg(
    gateway: &dyn DeviceGateway,
    sample_count: u8,
    sample_interval_ms: u64,
) -> CommandResult<(f64, f64)> {
    let count = usize::from(sample_count.max(1));
    let mut right_sum = 0.0;
    let mut left_sum = 0.0;
    let mut right_non_zero_count = 0usize;
    let mut left_non_zero_count = 0usize;

    for _ in 0..count {
        thread::sleep(Duration::from_millis(sample_interval_ms));
        let sample = gateway.param_id114()?;
        let right_speed = abs_speed_as_f64(sample.right_whl_motor_sp);
        let left_speed = abs_speed_as_f64(sample.lef_whl_motor_sp);

        if right_speed != 0.0 {
            right_sum += right_speed;
            right_non_zero_count += 1;
        }

        if left_speed != 0.0 {
            left_sum += left_speed;
            left_non_zero_count += 1;
        }
    }

    let right_avg = if right_non_zero_count == 0 {
        0.0
    } else {
        right_sum / right_non_zero_count as f64
    };
    let left_avg = if left_non_zero_count == 0 {
        0.0
    } else {
        left_sum / left_non_zero_count as f64
    };

    Ok((right_avg, left_avg))
}

fn run_wheel_motor_test_group(
    gateway: &dyn DeviceGateway,
    name: String,
    stage: String,
    right_motor_speed: i16,
    left_motor_speed: i16,
    sample_interval_ms: u64,
    sample_count: u8,
    right_test_inactive_max_speed_mm_s: f64,
    left_test_inactive_max_speed_mm_s: f64,
    checks: Vec<WheelMotorCheck>,
    on_lift_confirm: &dyn Fn(WheelMotorTestUpdatePayload) -> CommandResult<bool>,
    on_test_update: &dyn Fn(WheelMotorTestUpdatePayload) -> CommandResult<()>,
) -> CommandResult<TestResult> {
    if checks.len() != 2 {
        return Err(AppError::msg(
            "wheel_motor_test 需要且仅允许 2 条 checks（right_wheel_motor 与 left_wheel_motor）",
        ));
    }

    let mut right_check: Option<WheelMotorCheck> = None;
    let mut left_check: Option<WheelMotorCheck> = None;
    for check in checks {
        match check.output {
            WheelMotorOutput::RightWheelMotor => {
                if right_check.is_some() {
                    return Err(AppError::msg(
                        "wheel_motor_test checks 不允许重复配置 right_wheel_motor",
                    ));
                }
                right_check = Some(check);
            }
            WheelMotorOutput::LeftWheelMotor => {
                if left_check.is_some() {
                    return Err(AppError::msg(
                        "wheel_motor_test checks 不允许重复配置 left_wheel_motor",
                    ));
                }
                left_check = Some(check);
            }
        }
    }
    let right_check = right_check
        .ok_or_else(|| AppError::msg("wheel_motor_test 缺少 right_wheel_motor check"))?;
    let left_check =
        left_check.ok_or_else(|| AppError::msg("wheel_motor_test 缺少 left_wheel_motor check"))?;

    let confirmed = on_lift_confirm(WheelMotorTestUpdatePayload {
        name: name.clone(),
        stage: stage.clone(),
        phase: WheelMotorTestPhase::LiftConfirm,
    })?;
    if !confirmed {
        return Ok(TestResult {
            name,
            names: Default::default(),
            stage,
            command: "WheelMotorTest".to_string(),
            passed: false,
            raw_response: "LiftNotConfirmed".to_string(),
            checks: vec![
                CheckResult {
                    name: right_check.name,
                    min: Some(right_check.min),
                    max: Some(right_check.max),
                    value: None,
                    display_min: None,
                    display_max: None,
                    display_value: None,
                    passed: false,
                },
                CheckResult {
                    name: left_check.name,
                    min: Some(left_check.min),
                    max: Some(left_check.max),
                    value: None,
                    display_min: None,
                    display_max: None,
                    display_value: None,
                    passed: false,
                },
            ],
        });
    }

    let test_result = (|| -> CommandResult<TestResult> {
        on_test_update(WheelMotorTestUpdatePayload {
            name: name.clone(),
            stage: stage.clone(),
            phase: WheelMotorTestPhase::TestingRight,
        })?;
        gateway.param_id254(right_motor_speed)?;
        let (right_active_avg, right_inactive_avg) =
            collect_wheel_speed_avg(gateway, sample_count, sample_interval_ms)?;
        gateway.param_id254(0)?;

        on_test_update(WheelMotorTestUpdatePayload {
            name: name.clone(),
            stage: stage.clone(),
            phase: WheelMotorTestPhase::TestingLeft,
        })?;
        gateway.param_id256(left_motor_speed)?;
        let (left_inactive_avg, left_active_avg) =
            collect_wheel_speed_avg(gateway, sample_count, sample_interval_ms)?;
        gateway.param_id256(0)?;

        let right_pass = right_active_avg >= right_check.min
            && right_active_avg <= right_check.max
            && right_inactive_avg <= right_test_inactive_max_speed_mm_s;
        let left_pass = left_active_avg >= left_check.min
            && left_active_avg <= left_check.max
            && left_inactive_avg <= left_test_inactive_max_speed_mm_s;

        let check_results = vec![
            CheckResult {
                name: right_check.name.clone(),
                min: Some(right_check.min),
                max: Some(right_check.max),
                value: Some(right_active_avg),
                display_min: None,
                display_max: None,
                display_value: None,
                passed: right_pass,
            },
            CheckResult {
                name: left_check.name.clone(),
                min: Some(left_check.min),
                max: Some(left_check.max),
                value: Some(left_active_avg),
                display_min: None,
                display_max: None,
                display_value: None,
                passed: left_pass,
            },
        ];

        Ok(TestResult {
            name: name.clone(),
            names: Default::default(),
            stage: stage.clone(),
            command: "WheelMotorTest".to_string(),
            passed: check_results.iter().all(|item| item.passed),
            raw_response: format!(
                "RightActiveAvg={:.2}, RightInactiveAvg={:.2}, LeftActiveAvg={:.2}, LeftInactiveAvg={:.2}, RightInactiveMax={:.2}, LeftInactiveMax={:.2}, SampleCount={}, SampleIntervalMs={}, RightMotorSpeed={}, LeftMotorSpeed={}",
                right_active_avg,
                right_inactive_avg,
                left_active_avg,
                left_inactive_avg,
                right_test_inactive_max_speed_mm_s,
                left_test_inactive_max_speed_mm_s,
                sample_count.max(1),
                sample_interval_ms,
                right_motor_speed,
                left_motor_speed
            ),
            checks: check_results,
        })
    })();

    let stop_right_result = gateway.param_id254(0);
    let stop_left_result = gateway.param_id256(0);

    match (test_result, stop_right_result, stop_left_result) {
        (Ok(result), Ok(()), Ok(())) => Ok(result),
        (Ok(_), Err(stop_err), Ok(())) | (Ok(_), Ok(()), Err(stop_err)) => Err(stop_err),
        (Ok(_), Err(right_err), Err(left_err)) => Err(AppError::msg(format!(
            "轮电机测试完成但停止电机失败: right={right_err}, left={left_err}"
        ))),
        (Err(run_err), Ok(()), Ok(())) => Err(run_err),
        (Err(run_err), Err(stop_err), Ok(())) | (Err(run_err), Ok(()), Err(stop_err)) => Err(
            AppError::msg(format!("{run_err}; 且停止电机失败: {stop_err}")),
        ),
        (Err(run_err), Err(right_err), Err(left_err)) => Err(AppError::msg(format!(
            "{run_err}; 且停止电机失败: right={right_err}, left={left_err}"
        ))),
    }
}

fn run_collision_bar_test_group(
    gateway: &dyn DeviceGateway,
    name: String,
    stage: String,
    timeout_ms: u64,
    on_prompt: &dyn Fn(CollisionBarPromptPayload) -> CommandResult<()>,
) -> CommandResult<TestResult> {
    info!("Starting collision bar test: {}", name);
    let initial = gateway.param_id118()?;
    let initial_ok = initial.collision_sen == 0;
    if !initial_ok {
        return Ok(TestResult {
            name,
            names: Default::default(),
            stage,
            command: "ParamId118".to_string(),
            passed: false,
            raw_response: format!(
                "{initial}, Reason=InitialCollisionNotZero, ExpectedCollisionSen=0"
            ),
            checks: vec![
                CheckResult {
                    name: "initial_collision_sen_is_0".to_string(),
                    min: Some(0.0),
                    max: Some(0.0),
                    value: Some(initial.collision_sen as f64),
                    display_min: None,
                    display_max: None,
                    display_value: None,
                    passed: false,
                },
                CheckResult {
                    name: "collision_sen_triggered".to_string(),
                    min: Some(1.0),
                    max: None,
                    value: Some(initial.collision_sen as f64),
                    display_min: None,
                    display_max: None,
                    display_value: None,
                    passed: false,
                },
            ],
        });
    }

    start_sensor_prompt_session()?;

    let result = (|| -> CommandResult<TestResult> {
        on_prompt(CollisionBarPromptPayload {
            name: name.clone(),
            stage: stage.clone(),
            prompt_kind: SensorPromptKind::CollisionBar,
        })?;

        let elapsed_limit = if timeout_ms == 0 {
            u64::MAX
        } else {
            timeout_ms
        };
        let mut elapsed_ms: u64 = 0;
        let mut last = initial;

        while elapsed_ms < elapsed_limit {
            if is_sensor_prompt_canceled()? {
                return Ok(TestResult {
                    name,
                    names: Default::default(),
                    stage,
                    command: "ParamId118".to_string(),
                    passed: false,
                    raw_response: format!(
                        "CanceledByUser, ElapsedMs={}, InitialCollisionSen=0, LastCollisionSen={}, LastResult={}",
                        elapsed_ms, last.collision_sen, last
                    ),
                    checks: vec![
                        CheckResult {
                            name: "initial_collision_sen_is_0".to_string(),
                            min: Some(0.0),
                            max: Some(0.0),
                            value: Some(0.0),
                            display_min: None,
                            display_max: None,
                            display_value: None,
                            passed: true,
                        },
                        CheckResult {
                            name: "collision_sen_triggered".to_string(),
                            min: Some(1.0),
                            max: None,
                            value: Some(last.collision_sen as f64),
                            display_min: None,
                            display_max: None,
                            display_value: None,
                            passed: false,
                        },
                    ],
                });
            }

            thread::sleep(Duration::from_millis(PARAM_ID118_POLL_INTERVAL_MS));
            elapsed_ms = elapsed_ms.saturating_add(PARAM_ID118_POLL_INTERVAL_MS.max(1));
            last = gateway.param_id118()?;
            if last.collision_sen > 0 {
                return Ok(TestResult {
                    name,
                    names: Default::default(),
                    stage,
                    command: "ParamId118".to_string(),
                    passed: true,
                    raw_response: format!(
                        "InitialCollisionSen=0, TriggeredAfter={}ms, {}",
                        elapsed_ms, last
                    ),
                    checks: vec![
                        CheckResult {
                            name: "initial_collision_sen_is_0".to_string(),
                            min: Some(0.0),
                            max: Some(0.0),
                            value: Some(0.0),
                            display_min: None,
                            display_max: None,
                            display_value: None,
                            passed: true,
                        },
                        CheckResult {
                            name: "collision_sen_triggered".to_string(),
                            min: Some(1.0),
                            max: None,
                            value: Some(last.collision_sen as f64),
                            display_min: None,
                            display_max: None,
                            display_value: None,
                            passed: true,
                        },
                    ],
                });
            }
        }

        Ok(TestResult {
            name,
            names: Default::default(),
            stage,
            command: "ParamId118".to_string(),
            passed: false,
            raw_response: format!(
                "Timeout={}ms, InitialCollisionSen=0, LastCollisionSen={}, LastResult={}",
                elapsed_ms, last.collision_sen, last
            ),
            checks: vec![
                CheckResult {
                    name: "initial_collision_sen_is_0".to_string(),
                    min: Some(0.0),
                    max: Some(0.0),
                    value: Some(0.0),
                    display_min: None,
                    display_max: None,
                    display_value: None,
                    passed: true,
                },
                CheckResult {
                    name: "collision_sen_triggered".to_string(),
                    min: Some(1.0),
                    max: None,
                    value: Some(last.collision_sen as f64),
                    display_min: None,
                    display_max: None,
                    display_value: None,
                    passed: false,
                },
            ],
        })
    })();

    finish_sensor_prompt_session();
    result
}

fn run_lift_sensor_test_group(
    gateway: &dyn DeviceGateway,
    name: String,
    stage: String,
    timeout_ms: u64,
    lift_threshold: u8,
    on_prompt: &dyn Fn(CollisionBarPromptPayload) -> CommandResult<()>,
) -> CommandResult<TestResult> {
    info!(
        "Starting lift sensor test: {}, threshold={}",
        name, lift_threshold
    );

    start_sensor_prompt_session()?;

    let result = (|| -> CommandResult<TestResult> {
        on_prompt(CollisionBarPromptPayload {
            name: name.clone(),
            stage: stage.clone(),
            prompt_kind: SensorPromptKind::LiftSensor,
        })?;

        let elapsed_limit = if timeout_ms == 0 {
            u64::MAX
        } else {
            timeout_ms
        };
        let mut elapsed_ms: u64 = 0;
        let mut last_lift_sen: u8 = 0;
        let mut last_raw = "N/A".to_string();

        while elapsed_ms < elapsed_limit {
            if is_sensor_prompt_canceled()? {
                return Ok(TestResult {
                    name,
                    names: Default::default(),
                    stage,
                    command: "ParamId118".to_string(),
                    passed: false,
                    raw_response: format!(
                        "CanceledByUser, ElapsedMs={}, LiftThreshold={}, LastLiftSen={}, LastResult={}",
                        elapsed_ms, lift_threshold, last_lift_sen, last_raw
                    ),
                    checks: vec![CheckResult {
                        name: "lift_sen_triggered".to_string(),
                        min: Some((lift_threshold as f64) + 1.0),
                        max: None,
                        value: Some(last_lift_sen as f64),
                        display_min: None,
                        display_max: None,
                        display_value: None,
                        passed: false,
                    }],
                });
            }

            thread::sleep(Duration::from_millis(PARAM_ID118_POLL_INTERVAL_MS));
            elapsed_ms = elapsed_ms.saturating_add(PARAM_ID118_POLL_INTERVAL_MS.max(1));
            let response = gateway.param_id118()?;
            last_lift_sen = response.lift_sen;
            last_raw = response.to_string();

            if response.lift_sen > lift_threshold {
                return Ok(TestResult {
                    name,
                    names: Default::default(),
                    stage,
                    command: "ParamId118".to_string(),
                    passed: true,
                    raw_response: format!(
                        "LiftThreshold={}, TriggeredAfter={}ms, {}",
                        lift_threshold, elapsed_ms, response
                    ),
                    checks: vec![CheckResult {
                        name: "lift_sen_triggered".to_string(),
                        min: Some((lift_threshold as f64) + 1.0),
                        max: None,
                        value: Some(response.lift_sen as f64),
                        display_min: None,
                        display_max: None,
                        display_value: None,
                        passed: true,
                    }],
                });
            }
        }

        Ok(TestResult {
            name,
            names: Default::default(),
            stage,
            command: "ParamId118".to_string(),
            passed: false,
            raw_response: format!(
                "Timeout={}ms, LiftThreshold={}, LastLiftSen={}, LastResult={}",
                elapsed_ms, lift_threshold, last_lift_sen, last_raw
            ),
            checks: vec![CheckResult {
                name: "lift_sen_triggered".to_string(),
                min: Some((lift_threshold as f64) + 1.0),
                max: None,
                value: Some(last_lift_sen as f64),
                display_min: None,
                display_max: None,
                display_value: None,
                passed: false,
            }],
        })
    })();

    finish_sensor_prompt_session();
    result
}

fn run_emergency_stop_test_group(
    gateway: &dyn DeviceGateway,
    name: String,
    stage: String,
    timeout_ms: u64,
    on_state_update: &dyn Fn(EmergencyStopTestPayload),
) -> CommandResult<TestResult> {
    info!("Starting emergency stop test: {}", name);
    start_emergency_stop_test_session()?;

    let result = (|| -> CommandResult<TestResult> {
        let elapsed_limit = if timeout_ms == 0 {
            u64::MAX
        } else {
            timeout_ms
        };
        let mut elapsed_ms: u64 = 0;
        let mut phase = EmergencyStopPhase::PressEmergencyStop;
        let mut mower_main_p: u8 = 0;

        on_state_update(EmergencyStopTestPayload {
            name: name.clone(),
            stage: stage.clone(),
            phase,
            mower_main_p,
            elapsed_ms,
            timeout_ms,
        });

        loop {
            if is_emergency_stop_test_canceled()? {
                return Ok(build_emergency_stop_result(
                    name,
                    stage,
                    false,
                    "dialog_closed",
                    elapsed_ms,
                    timeout_ms,
                    mower_main_p,
                ));
            }

            if elapsed_ms >= elapsed_limit {
                return Ok(build_emergency_stop_result(
                    name,
                    stage,
                    false,
                    "timeout",
                    elapsed_ms,
                    timeout_ms,
                    mower_main_p,
                ));
            }

            thread::sleep(Duration::from_millis(EMERGENCY_STOP_POLL_INTERVAL_MS));
            elapsed_ms = elapsed_ms.saturating_add(EMERGENCY_STOP_POLL_INTERVAL_MS.max(1));

            if is_emergency_stop_test_canceled()? {
                return Ok(build_emergency_stop_result(
                    name,
                    stage,
                    false,
                    "dialog_closed",
                    elapsed_ms,
                    timeout_ms,
                    mower_main_p,
                ));
            }

            let response = gateway.param_id080()?;
            mower_main_p = response.mower_main_p;

            if mower_main_p == 1 && matches!(phase, EmergencyStopPhase::PressEmergencyStop) {
                phase = EmergencyStopPhase::UnlockByBackAndConfirm;
                on_state_update(EmergencyStopTestPayload {
                    name: name.clone(),
                    stage: stage.clone(),
                    phase,
                    mower_main_p,
                    elapsed_ms,
                    timeout_ms,
                });
            }

            if mower_main_p == 2 && matches!(phase, EmergencyStopPhase::UnlockByBackAndConfirm) {
                return Ok(build_emergency_stop_result(
                    name,
                    stage,
                    true,
                    "completed",
                    elapsed_ms,
                    timeout_ms,
                    mower_main_p,
                ));
            }
        }
    })();

    finish_emergency_stop_test_session();
    result
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
    start_key_test_session()?;
    let run_result = (|| -> CommandResult<TestResult> {
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
            if is_key_test_canceled()? {
                return Ok(TestResult {
                    name,
                    names: Default::default(),
                    stage,
                    command: "ParamId776".to_string(),
                    passed: false,
                    raw_response: format!("CanceledByUser, Elapsed={}ms", elapsed_ms),
                    checks: vec![CheckResult {
                        name: "all_keys_pressed".to_string(),
                        min: None,
                        max: None,
                        value: None,
                        display_min: None,
                        display_max: None,
                        display_value: None,
                        passed: false,
                    }],
                });
            }

            if wait_key_test_or_cancel(KEY_TEST_POLL_INTERVAL_MS)? {
                return Ok(TestResult {
                    name,
                    names: Default::default(),
                    stage,
                    command: "ParamId776".to_string(),
                    passed: false,
                    raw_response: format!("CanceledByUser, Elapsed={}ms", elapsed_ms),
                    checks: vec![CheckResult {
                        name: "all_keys_pressed".to_string(),
                        min: None,
                        max: None,
                        value: None,
                        display_min: None,
                        display_max: None,
                        display_value: None,
                        passed: false,
                    }],
                });
            }
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
                    names: Default::default(),
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
                        display_min: None,
                        display_max: None,
                        display_value: None,
                        passed: true,
                    }],
                });
            }

            if elapsed_ms >= elapsed_limit {
                warn!("Key test timed out after {}ms", elapsed_ms);
                return Ok(TestResult {
                    name,
                    names: Default::default(),
                    stage,
                    command: "ParamId776".to_string(),
                    passed: false,
                    raw_response: format!(
                        "Timeout={}ms, UpKey={}, DownKey={}, BackKey={}, ConfirmKey={}",
                        elapsed_ms,
                        result.up_key,
                        result.down_key,
                        result.back_key,
                        result.confirm_key
                    ),
                    checks: vec![CheckResult {
                        name: "all_keys_pressed".to_string(),
                        min: None,
                        max: None,
                        value: None,
                        display_min: None,
                        display_max: None,
                        display_value: None,
                        passed: false,
                    }],
                });
            }
        }
    })();
    finish_key_test_session();
    run_result
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use serial_test::serial;

    use super::{run_group_with_emitters, submit_emergency_stop_cancel, submit_key_test_cancel};
    use crate::device_gateway::DeviceGateway;
    use crate::models::{
        CommandGroupSpec, EmergencyStopPhase, EmergencyStopTestPayload, KeyStatePayload,
        ParamId068Result, ParamId080Result, ParamId096Check, ParamId096Output, ParamId114Result,
        ParamId120Result, ParamId122Result, ParamId272Result, ParamId470Result, ParamId526Check,
        ParamId526Output, ParamId588Result, ParamId654Result, ParamId776Result, ParamId794Result,
        ParamId796Check, ParamId796Output, ParamId796Result, ParamId798Result, TestGroup,
        VersionCheck, WheelMotorCheck, WheelMotorOutput,
    };
    use crate::types::CommandResult;

    struct FakeGateway {
        result_068: ParamId068Result,
        result_470_sequence: Vec<u8>,
        called_470: Cell<usize>,
        called_468: Cell<bool>,
        called_606: Cell<bool>,
        called_568: Cell<bool>,
        called_610_modes: RefCell<Vec<u8>>,
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

        fn param_id526(&self) -> CommandResult<crate::models::ParamId526Result> {
            Ok(crate::models::ParamId526Result {
                pcb_de_gr_no: 0,
                pcb_sub_de_no: 0,
                pcb_var_no: 0,
                pcb_pn: 40085400,
                pcb_rev: 4,
                pcb_ser_no: 12345678,
                pcb_prod_time: 0,
                pcb_ext_flash: 0,
                pcb_ext_eeprom: 0,
                pcb_accelerometer: 0,
            })
        }

        fn param_id096(&self) -> CommandResult<crate::models::ParamId096Result> {
            Ok(crate::models::ParamId096Result {
                gprs_lte_stat: 0,
                gprs_lte_sign_qual: 60,
                gnss_hw_stat: 3,
                sim_status: 1,
                ble_hw_stat: 0,
                gprs_lte_conn_stat: 0,
                ble_conn_stat: 0,
                wifi_conn_stat: 0,
                wifi_hw_stat: 0,
                lora_conn_stat: 0,
                lora_hw_stat: 0,
                rtk_hw_stat: 0,
                rtk_conn_stat: 0,
                connected_ra_serial: 0,
            })
        }

        fn param_id080(&self) -> CommandResult<ParamId080Result> {
            panic!("not used in this test")
        }

        fn param_id118(&self) -> CommandResult<crate::models::ParamId118Result> {
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

        fn param_id568(&self, _on: u8) -> CommandResult<()> {
            self.called_568.set(true);
            Ok(())
        }

        fn param_id610(&self, rear_light_mode: u8) -> CommandResult<()> {
            self.called_610_modes.borrow_mut().push(rear_light_mode);
            Ok(())
        }

        fn param_id794(&self) -> CommandResult<ParamId794Result> {
            panic!("not used in this test")
        }

        fn param_id796(&self) -> CommandResult<ParamId796Result> {
            Ok(ParamId796Result { mqtt_status: 1 })
        }

        fn param_id798(&self) -> CommandResult<ParamId798Result> {
            Ok(ParamId798Result {
                version: "COMM_SW_1.0.0".to_string(),
            })
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
        };

        let group = TestGroup {
            name: "068 test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId068 {
                checks: vec![VersionCheck {
                    name: "version".to_string(),
                    min: "5.0.0".to_string(),
                    max: "20.0.0".to_string(),
                }],
            },
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
        };

        let group = TestGroup {
            name: "068 test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId068 {
                checks: vec![VersionCheck {
                    name: "version".to_string(),
                    min: "5.0.0".to_string(),
                    max: "20.0.0".to_string(),
                }],
            },
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");
        assert!(!result.passed);
        assert_eq!(result.checks.len(), 1);
        assert!(!result.checks[0].passed);
    }

    #[test]
    fn run_group_param_id068_uses_semantic_version_range_and_display() {
        let gateway = FakeGateway {
            result_068: ParamId068Result {
                dev_gr_no: 0,
                sub_dev_gr_no: 0,
                var_no: 0,
                maj_par_sw_ver: 1,
                min_par_sw_ver: 2,
                build_no: 4,
            },
            result_470_sequence: vec![30],
            called_470: Cell::new(0),
            called_468: Cell::new(false),
            called_606: Cell::new(false),
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
        };

        let group = TestGroup {
            name: "068 test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId068 {
                checks: vec![VersionCheck {
                    name: "software_version".to_string(),
                    min: "1.2.3".to_string(),
                    max: "1.3.5".to_string(),
                }],
            },
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(result.passed);
        assert_eq!(result.checks[0].display_min.as_deref(), Some("1.2.3"));
        assert_eq!(result.checks[0].display_max.as_deref(), Some("1.3.5"));
        assert_eq!(result.checks[0].display_value.as_deref(), Some("1.2.4"));
        assert_eq!(result.checks[0].value, None);
    }

    #[test]
    fn run_group_param_id096_passes_when_values_match_checks() {
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
        };

        let group = TestGroup {
            name: "096 wireless".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId096 {
                checks: vec![
                    ParamId096Check {
                        name: "sim_status".to_string(),
                        output: ParamId096Output::SimStatus,
                        min: 1.0,
                        max: 1.0,
                    },
                    ParamId096Check {
                        name: "gprs_lte_sign_qual".to_string(),
                        output: ParamId096Output::GprsLteSignQual,
                        min: 51.0,
                        max: 255.0,
                    },
                    ParamId096Check {
                        name: "gnss_hw_stat".to_string(),
                        output: ParamId096Output::GnssHwStat,
                        min: 3.0,
                        max: 3.0,
                    },
                ],
            },
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");
        assert!(result.passed);
        assert_eq!(result.command, "ParamId096");
        assert_eq!(result.checks.len(), 3);
        assert!(result.checks.iter().all(|c| c.passed));
    }

    #[test]
    fn run_group_param_id526_passes_when_values_match_checks() {
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
        };

        let group = TestGroup {
            name: "526 app board".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId526 {
                checks: vec![
                    ParamId526Check {
                        name: "pcb_pn".to_string(),
                        output: ParamId526Output::PcbPn,
                        min: 40085400.0,
                        max: 40085400.0,
                    },
                    ParamId526Check {
                        name: "pcb_rev".to_string(),
                        output: ParamId526Output::PcbRev,
                        min: 4.0,
                        max: 4.0,
                    },
                ],
            },
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(result.passed);
        assert_eq!(result.command, "ParamId526");
        assert!(result.raw_response.contains("PcbSerNo=12345678"));
        assert_eq!(result.checks.len(), 2);
        assert!(result.checks.iter().all(|c| c.passed));
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
        };

        let group = TestGroup {
            name: "606 test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId606 {
                front_light_mode: 1,
                power: 80,
            },
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");
        assert!(result.passed);
        assert!(gateway.called_606.get());
        assert_eq!(result.command, "ParamId606");
        assert!(result.raw_response.contains("LightOn=1"));
    }

    #[test]
    fn run_group_param_id798_passes_when_version_present() {
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
        };

        let group = TestGroup {
            name: "798 test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId798,
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(result.passed);
        assert_eq!(result.command, "ParamId798");
        assert_eq!(result.raw_response, "Version=COMM_SW_1.0.0");
        assert_eq!(result.checks.len(), 1);
        assert_eq!(result.checks[0].name, "version_not_empty");
        assert!(result.checks[0].passed);
    }

    #[test]
    fn run_group_param_id796_passes_when_mqtt_status_is_online() {
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
        };

        let group = TestGroup {
            name: "796 mqtt".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId796 {
                checks: vec![ParamId796Check {
                    name: "mqtt_status".to_string(),
                    output: ParamId796Output::MqttStatus,
                    min: 1.0,
                    max: 1.0,
                }],
            },
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(result.passed);
        assert_eq!(result.command, "ParamId796");
        assert_eq!(result.raw_response, "MqttStatus=1");
        assert_eq!(result.checks.len(), 1);
        assert_eq!(result.checks[0].name, "mqtt_status");
        assert!(result.checks[0].passed);
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
        };

        let group = TestGroup {
            name: "606 test fail".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId606 {
                front_light_mode: 1,
                power: 80,
            },
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(false),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");
        assert!(!result.passed);
        assert!(gateway.called_606.get());
        assert_eq!(result.command, "ParamId606");
        assert!(result.raw_response.contains("LightOn=0"));
    }

    #[test]
    fn run_group_param_id610_passes_and_restores_mode() {
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
        };

        let group = TestGroup {
            name: "610 test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId610,
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");
        assert!(result.passed);
        assert_eq!(result.command, "ParamId610");
        assert_eq!(result.checks.len(), 3);
        assert_eq!(gateway.called_610_modes.borrow().as_slice(), &[4, 5, 6, 3]);
    }

    #[test]
    fn run_group_param_id610_fails_on_second_step_and_restores_mode() {
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
        };

        let group = TestGroup {
            name: "610 test fail".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId610,
        };

        let confirm_index = Cell::new(0usize);
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| {
                let index = confirm_index.get();
                confirm_index.set(index + 1);
                Ok(index == 0)
            },
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(!result.passed);
        assert_eq!(result.command, "ParamId610");
        assert_eq!(result.checks.len(), 2);
        assert_eq!(gateway.called_610_modes.borrow().as_slice(), &[4, 5, 3]);
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
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

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
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
            called_568: Cell::new(false),
            called_610_modes: RefCell::new(Vec::new()),
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

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");
        assert!(result.passed);
        assert_eq!(gateway.called_470.get(), 3);
    }

    struct FakeCollisionGateway {
        collision_sequence: Vec<u8>,
        called_118: Cell<usize>,
    }

    impl DeviceGateway for FakeCollisionGateway {
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

        fn param_id526(&self) -> CommandResult<crate::models::ParamId526Result> {
            panic!("not used in this test")
        }

        fn param_id096(&self) -> CommandResult<crate::models::ParamId096Result> {
            panic!("not used in this test")
        }

        fn param_id080(&self) -> CommandResult<ParamId080Result> {
            panic!("not used in this test")
        }

        fn param_id118(&self) -> CommandResult<crate::models::ParamId118Result> {
            let idx = self.called_118.get();
            let value = self
                .collision_sequence
                .get(idx)
                .copied()
                .or_else(|| self.collision_sequence.last().copied())
                .unwrap_or(0);
            self.called_118.set(idx + 1);
            Ok(crate::models::ParamId118Result {
                collision_sen: value,
                lift_sen: 0,
                status_flags: 0,
                stop_sen: 0,
                disabling_sen: 0,
            })
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

        fn param_id568(&self, _on: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id610(&self, _rear_light_mode: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id794(&self) -> CommandResult<ParamId794Result> {
            panic!("not used in this test")
        }

        fn param_id798(&self) -> CommandResult<ParamId798Result> {
            panic!("not used in this test")
        }

        fn param_id776(&self, _cmd: u8) -> CommandResult<ParamId776Result> {
            panic!("not used in this test")
        }
    }

    #[test]
    #[serial]
    fn run_group_param_id118_collision_bar_passes_after_trigger() {
        let gateway = FakeCollisionGateway {
            collision_sequence: vec![0, 0, 1],
            called_118: Cell::new(0),
        };

        let group = TestGroup {
            name: "118 collision test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId118CollisionBar { timeout_ms: 10 },
        };

        let prompted = Cell::new(0usize);
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| {
                prompted.set(prompted.get() + 1);
                Ok(())
            },
            &|_| {},
        )
        .expect("group should run");

        assert!(result.passed);
        assert_eq!(result.command, "ParamId118");
        assert_eq!(gateway.called_118.get(), 3);
        assert_eq!(prompted.get(), 1);
    }

    #[test]
    #[serial]
    fn run_group_param_id118_collision_bar_fails_when_initial_not_zero() {
        let gateway = FakeCollisionGateway {
            collision_sequence: vec![1],
            called_118: Cell::new(0),
        };

        let group = TestGroup {
            name: "118 collision test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId118CollisionBar { timeout_ms: 10 },
        };

        let prompted = Cell::new(0usize);
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| {
                prompted.set(prompted.get() + 1);
                Ok(())
            },
            &|_| {},
        )
        .expect("group should run");

        assert!(!result.passed);
        assert_eq!(result.command, "ParamId118");
        assert_eq!(gateway.called_118.get(), 1);
        assert_eq!(prompted.get(), 0);
    }

    #[test]
    #[serial]
    fn run_group_param_id118_collision_bar_fails_when_dialog_closed() {
        let gateway = FakeCollisionGateway {
            collision_sequence: vec![0, 0, 1],
            called_118: Cell::new(0),
        };

        let group = TestGroup {
            name: "118 collision cancel".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId118CollisionBar { timeout_ms: 10 },
        };

        let prompted = Cell::new(0usize);
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| {
                prompted.set(prompted.get() + 1);
                let _ = super::submit_sensor_prompt_cancel();
                Ok(())
            },
            &|_| {},
        )
        .expect("group should run");

        assert!(!result.passed);
        assert_eq!(result.command, "ParamId118");
        assert!(result.raw_response.contains("CanceledByUser"));
        assert_eq!(prompted.get(), 1);
    }

    struct FakeLiftGateway {
        lift_sequence: Vec<u8>,
        called_118: Cell<usize>,
    }

    impl DeviceGateway for FakeLiftGateway {
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

        fn param_id526(&self) -> CommandResult<crate::models::ParamId526Result> {
            panic!("not used in this test")
        }

        fn param_id096(&self) -> CommandResult<crate::models::ParamId096Result> {
            panic!("not used in this test")
        }

        fn param_id080(&self) -> CommandResult<ParamId080Result> {
            panic!("not used in this test")
        }

        fn param_id118(&self) -> CommandResult<crate::models::ParamId118Result> {
            let idx = self.called_118.get();
            let value = self
                .lift_sequence
                .get(idx)
                .copied()
                .or_else(|| self.lift_sequence.last().copied())
                .unwrap_or(0);
            self.called_118.set(idx + 1);
            Ok(crate::models::ParamId118Result {
                collision_sen: 0,
                lift_sen: value,
                status_flags: 0,
                stop_sen: 0,
                disabling_sen: 0,
            })
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

        fn param_id568(&self, _on: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id610(&self, _rear_light_mode: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id794(&self) -> CommandResult<ParamId794Result> {
            panic!("not used in this test")
        }

        fn param_id798(&self) -> CommandResult<ParamId798Result> {
            panic!("not used in this test")
        }

        fn param_id776(&self, _cmd: u8) -> CommandResult<ParamId776Result> {
            panic!("not used in this test")
        }
    }

    #[test]
    #[serial]
    fn run_group_param_id118_lift_sensor_passes_after_trigger() {
        let gateway = FakeLiftGateway {
            lift_sequence: vec![0, 1, 2],
            called_118: Cell::new(0),
        };

        let group = TestGroup {
            name: "118 lift test".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId118LiftSensor {
                timeout_ms: 10,
                lift_threshold: 1,
            },
        };

        let prompted = Cell::new(0usize);
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| {
                prompted.set(prompted.get() + 1);
                Ok(())
            },
            &|_| {},
        )
        .expect("group should run");

        assert!(result.passed);
        assert_eq!(result.command, "ParamId118");
        assert_eq!(gateway.called_118.get(), 3);
        assert_eq!(prompted.get(), 1);
    }

    #[test]
    #[serial]
    fn run_group_param_id118_lift_sensor_times_out() {
        let gateway = FakeLiftGateway {
            lift_sequence: vec![0, 1, 1],
            called_118: Cell::new(0),
        };

        let group = TestGroup {
            name: "118 lift test timeout".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId118LiftSensor {
                timeout_ms: 2,
                lift_threshold: 1,
            },
        };

        let prompted = Cell::new(0usize);
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| {
                prompted.set(prompted.get() + 1);
                Ok(())
            },
            &|_| {},
        )
        .expect("group should run");

        assert!(!result.passed);
        assert_eq!(result.command, "ParamId118");
        assert_eq!(gateway.called_118.get(), 2);
        assert_eq!(prompted.get(), 1);
    }

    #[test]
    #[serial]
    fn run_group_param_id118_lift_sensor_fails_when_dialog_closed() {
        let gateway = FakeLiftGateway {
            lift_sequence: vec![0, 1, 2],
            called_118: Cell::new(0),
        };

        let group = TestGroup {
            name: "118 lift cancel".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId118LiftSensor {
                timeout_ms: 10,
                lift_threshold: 1,
            },
        };

        let prompted = Cell::new(0usize);
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| {
                prompted.set(prompted.get() + 1);
                let _ = super::submit_sensor_prompt_cancel();
                Ok(())
            },
            &|_| {},
        )
        .expect("group should run");

        assert!(!result.passed);
        assert_eq!(result.command, "ParamId118");
        assert!(result.raw_response.contains("CanceledByUser"));
        assert_eq!(prompted.get(), 1);
    }

    struct FakeEmergencyStopGateway {
        mower_main_p_sequence: Vec<u8>,
        called_080: Cell<usize>,
    }

    impl DeviceGateway for FakeEmergencyStopGateway {
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

        fn param_id526(&self) -> CommandResult<crate::models::ParamId526Result> {
            panic!("not used in this test")
        }

        fn param_id096(&self) -> CommandResult<crate::models::ParamId096Result> {
            panic!("not used in this test")
        }

        fn param_id080(&self) -> CommandResult<ParamId080Result> {
            let idx = self.called_080.get();
            let mower_main_p = self
                .mower_main_p_sequence
                .get(idx)
                .copied()
                .or_else(|| self.mower_main_p_sequence.last().copied())
                .unwrap_or(0);
            self.called_080.set(idx + 1);
            Ok(ParamId080Result {
                mower_main_p,
                mower_sub_state: 0,
                time_stp_nxt_start: 0,
                batt_stat: 0,
                stat_flags: 0,
                wrless_con_stat: 0,
                sign_quality: 0,
                source_for_next_start_stop: 0,
                notify: 0,
                configuration_hash: 0,
            })
        }

        fn param_id118(&self) -> CommandResult<crate::models::ParamId118Result> {
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

        fn param_id568(&self, _on: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id610(&self, _rear_light_mode: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id794(&self) -> CommandResult<ParamId794Result> {
            panic!("not used in this test")
        }

        fn param_id798(&self) -> CommandResult<ParamId798Result> {
            panic!("not used in this test")
        }

        fn param_id776(&self, _cmd: u8) -> CommandResult<ParamId776Result> {
            panic!("not used in this test")
        }
    }

    #[test]
    #[serial]
    fn run_group_param_id080_emergency_stop_passes() {
        let gateway = FakeEmergencyStopGateway {
            mower_main_p_sequence: vec![0, 1, 2],
            called_080: Cell::new(0),
        };

        let group = TestGroup {
            name: "080 emergency stop".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId080EmergencyStop { timeout_ms: 10 },
        };

        let updates = RefCell::new(Vec::<EmergencyStopTestPayload>::new());
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|payload| {
                updates.borrow_mut().push(payload);
            },
        )
        .expect("group should run");

        assert!(result.passed);
        assert_eq!(result.command, "ParamId080EmergencyStop");
        assert_eq!(result.checks[0].value, Some(2.0));
        assert_eq!(updates.borrow().len(), 2);
        assert!(matches!(
            updates.borrow()[0].phase,
            EmergencyStopPhase::PressEmergencyStop
        ));
        assert!(matches!(
            updates.borrow()[1].phase,
            EmergencyStopPhase::UnlockByBackAndConfirm
        ));
    }

    #[test]
    #[serial]
    fn run_group_param_id080_emergency_stop_does_not_pass_before_seen_one() {
        let gateway = FakeEmergencyStopGateway {
            mower_main_p_sequence: vec![2, 1, 2],
            called_080: Cell::new(0),
        };

        let group = TestGroup {
            name: "080 emergency stop starts at 2".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId080EmergencyStop { timeout_ms: 10 },
        };

        let updates = RefCell::new(Vec::<EmergencyStopTestPayload>::new());
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|payload| {
                updates.borrow_mut().push(payload);
            },
        )
        .expect("group should run");

        assert!(result.passed);
        assert_eq!(result.command, "ParamId080EmergencyStop");
        assert!(updates.borrow().len() >= 2);
        assert!(matches!(
            updates.borrow()[1].phase,
            EmergencyStopPhase::UnlockByBackAndConfirm
        ));
    }

    #[test]
    #[serial]
    fn run_group_param_id080_emergency_stop_times_out() {
        let gateway = FakeEmergencyStopGateway {
            mower_main_p_sequence: vec![0, 1, 1],
            called_080: Cell::new(0),
        };

        let group = TestGroup {
            name: "080 emergency stop timeout".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId080EmergencyStop { timeout_ms: 2 },
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(!result.passed);
        assert_eq!(result.command, "ParamId080EmergencyStop");
        assert!(result.raw_response.contains("Reason=timeout"));
    }

    #[test]
    #[serial]
    fn run_group_param_id080_emergency_stop_fails_when_dialog_closed() {
        let gateway = FakeEmergencyStopGateway {
            mower_main_p_sequence: vec![0, 1, 2],
            called_080: Cell::new(0),
        };

        let group = TestGroup {
            name: "080 emergency stop cancel".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId080EmergencyStop { timeout_ms: 10 },
        };

        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|payload| {
                if matches!(payload.phase, EmergencyStopPhase::PressEmergencyStop) {
                    let _ = submit_emergency_stop_cancel();
                }
            },
        )
        .expect("group should run");

        assert!(!result.passed);
        assert_eq!(result.command, "ParamId080EmergencyStop");
        assert!(result.raw_response.contains("Reason=dialog_closed"));
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

        fn param_id526(&self) -> CommandResult<crate::models::ParamId526Result> {
            panic!("not used in this test")
        }

        fn param_id096(&self) -> CommandResult<crate::models::ParamId096Result> {
            panic!("not used in this test")
        }

        fn param_id080(&self) -> CommandResult<ParamId080Result> {
            panic!("not used in this test")
        }

        fn param_id118(&self) -> CommandResult<crate::models::ParamId118Result> {
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

        fn param_id568(&self, _on: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id610(&self, _rear_light_mode: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id794(&self) -> CommandResult<ParamId794Result> {
            panic!("not used in this test")
        }

        fn param_id798(&self) -> CommandResult<ParamId798Result> {
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
    #[serial]
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
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(gateway.called_start.get());
        assert!(result.passed);
        assert_eq!(result.command, "ParamId776");
        assert_eq!(updates.borrow().len(), 2);
        assert!(updates.borrow().last().expect("has update").confirm_pressed);
    }

    #[test]
    #[serial]
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
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(gateway.called_start.get());
        assert!(!result.passed);
        assert_eq!(result.command, "ParamId776");
        assert_eq!(update_count.get(), 1);
    }

    #[test]
    #[serial]
    fn run_group_param_id776_fails_when_dialog_closed() {
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
                    back_key: 0,
                    confirm_key: 0,
                },
            ],
            called_start: Cell::new(false),
            called_poll: Cell::new(0),
        };

        let group = TestGroup {
            name: "776 cancel".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::ParamId776 { timeout_ms: 10 },
        };

        let update_count = Cell::new(0usize);
        let result = run_group_with_emitters(
            &gateway,
            group,
            &|_| {
                if update_count.get() == 0 {
                    let _ = submit_key_test_cancel();
                }
                update_count.set(update_count.get() + 1);
            },
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(gateway.called_start.get());
        assert!(!result.passed);
        assert_eq!(result.command, "ParamId776");
        assert!(result.raw_response.contains("CanceledByUser"));
        assert_eq!(update_count.get(), 1);
    }

    struct FakeWheelMotorGateway {
        samples: RefCell<Vec<ParamId114Result>>,
        calls_254: RefCell<Vec<i16>>,
        calls_256: RefCell<Vec<i16>>,
    }

    impl DeviceGateway for FakeWheelMotorGateway {
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

        fn param_id526(&self) -> CommandResult<crate::models::ParamId526Result> {
            panic!("not used in this test")
        }

        fn param_id096(&self) -> CommandResult<crate::models::ParamId096Result> {
            panic!("not used in this test")
        }

        fn param_id080(&self) -> CommandResult<ParamId080Result> {
            panic!("not used in this test")
        }

        fn param_id118(&self) -> CommandResult<crate::models::ParamId118Result> {
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

        fn param_id254(&self, right_motor_speed: i16) -> CommandResult<()> {
            self.calls_254.borrow_mut().push(right_motor_speed);
            Ok(())
        }

        fn param_id256(&self, left_motor_speed: i16) -> CommandResult<()> {
            self.calls_256.borrow_mut().push(left_motor_speed);
            Ok(())
        }

        fn param_id114(&self) -> CommandResult<ParamId114Result> {
            let mut samples = self.samples.borrow_mut();
            if samples.is_empty() {
                return Err(crate::types::AppError::msg("no sample"));
            }
            Ok(samples.remove(0))
        }

        fn param_id568(&self, _on: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id610(&self, _rear_light_mode: u8) -> CommandResult<()> {
            panic!("not used in this test")
        }

        fn param_id794(&self) -> CommandResult<ParamId794Result> {
            panic!("not used in this test")
        }

        fn param_id798(&self) -> CommandResult<ParamId798Result> {
            panic!("not used in this test")
        }

        fn param_id776(&self, _cmd: u8) -> CommandResult<ParamId776Result> {
            panic!("not used in this test")
        }
    }

    fn wheel_group() -> TestGroup {
        TestGroup {
            name: "wheel".to_string(),
            stage: "unit".to_string(),
            command: CommandGroupSpec::WheelMotorTest {
                right_motor_speed: 45,
                left_motor_speed: 45,
                sample_interval_ms: 0,
                sample_count: 3,
                right_test_inactive_max_speed_mm_s: 10.0,
                left_test_inactive_max_speed_mm_s: 10.0,
                checks: vec![
                    WheelMotorCheck {
                        name: "right_wheel_motor".to_string(),
                        output: WheelMotorOutput::RightWheelMotor,
                        min: 700.0,
                        max: 999999.0,
                    },
                    WheelMotorCheck {
                        name: "left_wheel_motor".to_string(),
                        output: WheelMotorOutput::LeftWheelMotor,
                        min: 700.0,
                        max: 999999.0,
                    },
                ],
            },
        }
    }

    #[test]
    fn wheel_motor_right_inactive_over_limit_fails_right_check() {
        let gateway = FakeWheelMotorGateway {
            samples: RefCell::new(vec![
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 800,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 20,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 820,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 20,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 810,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 20,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 810,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 820,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 830,
                },
            ]),
            calls_254: RefCell::new(Vec::new()),
            calls_256: RefCell::new(Vec::new()),
        };

        let result = run_group_with_emitters(
            &gateway,
            wheel_group(),
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert_eq!(result.checks.len(), 2);
        assert!(!result.passed);
        assert!(!result.checks[0].passed);
        assert!(result.checks[1].passed);
        assert_eq!(gateway.calls_254.borrow().as_slice(), &[45, 0, 0]);
        assert_eq!(gateway.calls_256.borrow().as_slice(), &[45, 0, 0]);
    }

    #[test]
    fn wheel_motor_right_fail_still_runs_left() {
        let gateway = FakeWheelMotorGateway {
            samples: RefCell::new(vec![
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 200,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 0,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 220,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 0,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 210,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 0,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 760,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 770,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 780,
                },
            ]),
            calls_254: RefCell::new(Vec::new()),
            calls_256: RefCell::new(Vec::new()),
        };

        let result = run_group_with_emitters(
            &gateway,
            wheel_group(),
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(!result.passed);
        assert!(!result.checks[0].passed);
        assert!(result.checks[1].passed);
        assert!(!gateway.calls_256.borrow().is_empty());
    }

    #[test]
    fn wheel_motor_both_sides_pass() {
        let gateway = FakeWheelMotorGateway {
            samples: RefCell::new(vec![
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 810,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 0,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 820,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 0,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 830,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 0,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 810,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 820,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 830,
                },
            ]),
            calls_254: RefCell::new(Vec::new()),
            calls_256: RefCell::new(Vec::new()),
        };

        let result = run_group_with_emitters(
            &gateway,
            wheel_group(),
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(result.passed);
        assert_eq!(result.checks.len(), 2);
        assert!(result.checks[0].passed);
        assert!(result.checks[1].passed);
    }

    #[test]
    fn wheel_motor_ignores_zero_samples_when_averaging() {
        let gateway = FakeWheelMotorGateway {
            samples: RefCell::new(vec![
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 0,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 810,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 0,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 820,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 0,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 0,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 810,
                },
                ParamId114Result {
                    right_whl_motor_p: 0,
                    right_whl_motor_curr: 0,
                    right_whl_motor_sp: 0,
                    left_whl_motor_p: 0,
                    lef_whl_motor_curr: 0,
                    lef_whl_motor_sp: 820,
                },
            ]),
            calls_254: RefCell::new(Vec::new()),
            calls_256: RefCell::new(Vec::new()),
        };

        let result = run_group_with_emitters(
            &gateway,
            wheel_group(),
            &|_| {},
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(true),
            &|_| Ok(()),
            &|_| {},
        )
        .expect("group should run");

        assert!(result.passed);
        assert_eq!(result.checks[0].value, Some(815.0));
        assert_eq!(result.checks[1].value, Some(815.0));
    }
}
