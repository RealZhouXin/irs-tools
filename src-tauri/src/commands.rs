use std::path::PathBuf;

use tauri::{Emitter, Manager};
use tracing::{error, info};

use crate::comm_dll::{CommDll, CommSession};
use crate::config::{read_base_config, read_config, write_base_config};
use crate::models::{
    BaseConfig, CheckResult, CommandGroupSpec, ParamId068Output, ParamId068Result, ParamId588Output,
    ParamId588Result, ParamId654Output, ParamId654Result, ParamId272Output, ParamId272Result,
    ParamId080Output, ParamId080Result, TestConfig, TestGroup, TestResult, TestSummary,
};
use crate::types::CommandResult;

fn locate_dll(app: &tauri::AppHandle) -> CommandResult<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(resource_path) = app
        .path()
        .resolve("CommDllv2.dll", tauri::path::BaseDirectory::Resource)
    {
        candidates.push(resource_path);
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            candidates.push(parent.join("CommDllv2.dll"));
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("CommDllv2.dll"));
        if let Some(parent) = current_dir.parent() {
            candidates.push(parent.join("CommDllv2.dll"));
        }
    }

    for candidate in candidates {
        if candidate.exists() {
            info!("Using CommDllv2.dll at {}", candidate.display());
            return Ok(candidate);
        }
    }

    Err("未找到 CommDllv2.dll，请确认 DLL 已放在资源目录或程序当前目录".to_string())
}

fn pick_param_id588_value(output: ParamId588Output, result: ParamId588Result) -> f64 {
    match output {
        ParamId588Output::DevGrNo => result.dev_gr_no as f64,
        ParamId588Output::SubDevGrNo => result.sub_dev_gr_no as f64,
        ParamId588Output::VarNo => result.var_no as f64,
        ParamId588Output::MajParSwVer => result.maj_par_sw_ver as f64,
        ParamId588Output::MinParSwVer => result.min_par_sw_ver as f64,
        ParamId588Output::BuildNo => result.build_no as f64,
    }
}

fn pick_param_id068_value(output: ParamId068Output, result: ParamId068Result) -> f64 {
    match output {
        ParamId068Output::DevGrNo => result.dev_gr_no as f64,
        ParamId068Output::SubDevGrNo => result.sub_dev_gr_no as f64,
        ParamId068Output::VarNo => result.var_no as f64,
        ParamId068Output::MajParSwVer => result.maj_par_sw_ver as f64,
        ParamId068Output::MinParSwVer => result.min_par_sw_ver as f64,
        ParamId068Output::BuildNo => result.build_no as f64,
    }
}

fn pick_param_id654_value(output: ParamId654Output, result: ParamId654Result) -> f64 {
    match output {
        ParamId654Output::DevGrNo => result.dev_gr_no as f64,
        ParamId654Output::SubDevGrNo => result.sub_dev_gr_no as f64,
        ParamId654Output::VarNo => result.var_no as f64,
        ParamId654Output::MajParSwVer => result.maj_par_sw_ver as f64,
        ParamId654Output::MinParSwVer => result.min_par_sw_ver as f64,
        ParamId654Output::BuildNo => result.build_no as f64,
    }
}

fn pick_param_id272_value(output: ParamId272Output, result: ParamId272Result) -> f64 {
    match output {
        ParamId272Output::BattPackPn => result.batt_pack_pn as f64,
        ParamId272Output::BattPackRev => result.batt_pack_rev as f64,
        ParamId272Output::BattPackProdDate => result.batt_pack_prod_date as f64,
        ParamId272Output::BattSwVer => result.batt_sw_ver as f64,
        ParamId272Output::BattSerNo => result.batt_ser_no as f64,
        ParamId272Output::BattDevGrNo => result.batt_dev_gr_no as f64,
        ParamId272Output::BattSubDevNo => result.batt_sub_dev_no as f64,
        ParamId272Output::BattVarNo => result.batt_var_no as f64,
        ParamId272Output::BmsDevGrNo => result.bms_dev_gr_no as f64,
        ParamId272Output::BmsSubDevNo => result.bms_sub_dev_no as f64,
        ParamId272Output::BmsVarNo => result.bms_var_no as f64,
        ParamId272Output::BmsPcbaPn => result.bms_pcba_pn as f64,
        ParamId272Output::BmsPcbaRev => result.bms_pcba_rev as f64,
        ParamId272Output::BmsTempSensorType => result.bms_temp_sensor_type as f64,
    }
}

fn pick_param_id080_value(output: ParamId080Output, result: ParamId080Result) -> f64 {
    match output {
        ParamId080Output::MowerMainP => result.mower_main_p as f64,
        ParamId080Output::MowerSubState => result.mower_sub_state as f64,
        ParamId080Output::TimeStpNxtStart => result.time_stp_nxt_start as f64,
        ParamId080Output::BattStat => result.batt_stat as f64,
        ParamId080Output::StatFlags => result.stat_flags as f64,
        ParamId080Output::WrlessConStat => result.wrless_con_stat as f64,
        ParamId080Output::SignQuality => result.sign_quality as f64,
        ParamId080Output::SourceForNextStartStop => result.source_for_next_start_stop as f64,
        ParamId080Output::Notify => result.notify as f64,
        ParamId080Output::ConfigurationHash => result.configuration_hash as f64,
    }
}

fn command_name(command: &CommandGroupSpec) -> &'static str {
    match command {
        CommandGroupSpec::ParamId068 { .. } => "ParamId068",
        CommandGroupSpec::ParamId588 { .. } => "ParamId588",
        CommandGroupSpec::ParamId654 { .. } => "ParamId654",
        CommandGroupSpec::ParamId272 { .. } => "ParamId272",
        CommandGroupSpec::ParamId080 { .. } => "ParamId080",
        CommandGroupSpec::ParamId606 { .. } => "ParamId606",
    }
}

fn run_group(session: &CommSession, group: TestGroup) -> CommandResult<TestResult> {
    let command_label = command_name(&group.command).to_string();
    match group.command {
        CommandGroupSpec::ParamId068 { checks } => {
            let response = session.param_id068()?;
            let mut check_results = Vec::with_capacity(checks.len());

            for check in checks {
                let value = pick_param_id068_value(check.output, response);
                let passed = value >= check.min && value <= check.max;
                check_results.push(CheckResult {
                    name: check.name,
                    min: Some(check.min),
                    max: Some(check.max),
                    value: Some(value),
                    passed,
                });
            }

            let passed = check_results.iter().all(|item| item.passed);

            Ok(TestResult {
                name: group.name,
                command: command_label,
                passed,
                raw_response: response.to_string(),
                checks: check_results,
            })
        }
        CommandGroupSpec::ParamId588 { checks } => {
            let response = session.param_id588()?;
            let mut check_results = Vec::with_capacity(checks.len());

            for check in checks {
                let value = pick_param_id588_value(check.output, response);
                let passed = value >= check.min && value <= check.max;
                check_results.push(CheckResult {
                    name: check.name,
                    min: Some(check.min),
                    max: Some(check.max),
                    value: Some(value),
                    passed,
                });
            }

            let passed = check_results.iter().all(|item| item.passed);

            Ok(TestResult {
                name: group.name,
                command: command_label,
                passed,
                raw_response: response.to_string(),
                checks: check_results,
            })
        }
        CommandGroupSpec::ParamId654 { checks } => {
            let response = session.param_id654()?;
            let mut check_results = Vec::with_capacity(checks.len());

            for check in checks {
                let value = pick_param_id654_value(check.output, response);
                let passed = value >= check.min && value <= check.max;
                check_results.push(CheckResult {
                    name: check.name,
                    min: Some(check.min),
                    max: Some(check.max),
                    value: Some(value),
                    passed,
                });
            }

            let passed = check_results.iter().all(|item| item.passed);

            Ok(TestResult {
                name: group.name,
                command: command_label,
                passed,
                raw_response: response.to_string(),
                checks: check_results,
            })
        }
        CommandGroupSpec::ParamId272 { checks } => {
            let response = session.param_id272()?;
            let mut check_results = Vec::with_capacity(checks.len());

            for check in checks {
                let value = pick_param_id272_value(check.output, response);
                let passed = value >= check.min && value <= check.max;
                check_results.push(CheckResult {
                    name: check.name,
                    min: Some(check.min),
                    max: Some(check.max),
                    value: Some(value),
                    passed,
                });
            }

            let passed = check_results.iter().all(|item| item.passed);

            Ok(TestResult {
                name: group.name,
                command: command_label,
                passed,
                raw_response: response.to_string(),
                checks: check_results,
            })
        }
        CommandGroupSpec::ParamId080 { checks } => {
            let response = session.param_id080()?;
            let mut check_results = Vec::with_capacity(checks.len());

            for check in checks {
                let value = pick_param_id080_value(check.output, response);
                let passed = value >= check.min && value <= check.max;
                check_results.push(CheckResult {
                    name: check.name,
                    min: Some(check.min),
                    max: Some(check.max),
                    value: Some(value),
                    passed,
                });
            }

            let passed = check_results.iter().all(|item| item.passed);

            Ok(TestResult {
                name: group.name,
                command: command_label,
                passed,
                raw_response: response.to_string(),
                checks: check_results,
            })
        }
        CommandGroupSpec::ParamId606 {
            front_light_mode,
            power,
        } => {
            session.param_id606(front_light_mode, power)?;

            let check_results = vec![CheckResult {
                name: "执行结果".to_string(),
                min: None,
                max: None,
                value: None,
                passed: true,
            }];

            Ok(TestResult {
                name: group.name,
                command: command_label,
                passed: true,
                raw_response: format!(
                    "FrontLightMode={}, Power={}, ReturnCode=0",
                    front_light_mode, power
                ),
                checks: check_results,
            })
        }
    }
}

#[tauri::command]
pub fn start_test(app: tauri::AppHandle) -> CommandResult<TestSummary> {
    info!("Starting full test run");
    let config = read_config(&app)?;
    let TestConfig {
        connection,
        read_timeout_ms,
        tests,
    } = config;
    let dll_path = locate_dll(&app)?;
    let dll = unsafe { CommDll::load(&dll_path)? };
    let session = CommSession::connect(dll, &connection, read_timeout_ms)?;

    let mut results = Vec::with_capacity(tests.len());

    for group in tests {
        let name = group.name.clone();
        match run_group(&session, group) {
            Ok(result) => {
                info!("Completed group {}", name);
                if let Err(err) = app.emit("test-group-complete", &result) {
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

#[tauri::command]
pub fn retest_group(app: tauri::AppHandle, group_name: String) -> CommandResult<TestResult> {
    info!("Retest group {}", group_name);
    let config = read_config(&app)?;
    let TestConfig {
        connection,
        read_timeout_ms,
        tests,
    } = config;
    let group = tests
        .into_iter()
        .find(|item| item.name == group_name)
        .ok_or_else(|| format!("未找到测试项: {group_name}"))?;

    let dll_path = locate_dll(&app)?;
    let dll = unsafe { CommDll::load(&dll_path)? };
    let session = CommSession::connect(dll, &connection, read_timeout_ms)?;

    run_group(&session, group)
}

#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle) -> CommandResult<()> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "未找到主窗口".to_string())?;
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
    read_base_config(&app)
}
