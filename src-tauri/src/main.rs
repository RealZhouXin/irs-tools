use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Manager;

use libloading::Library;

use std::fmt;

type CommandResult<T> = Result<T, String>;

#[derive(Debug, Deserialize)]
struct TestConfig {
    port_number: u16,
    read_timeout_ms: u32,
    tests: Vec<TestItem>,
}

#[derive(Debug, Deserialize, Clone)]
struct TestItem {
    name: String,
    #[serde(flatten)]
    command: CommandSpec,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "command", rename_all = "snake_case")]
enum CommandSpec {
    ParamId588 {
        output: ParamId588Output,
        min: f64,
        max: f64,
    },
    ParamId606 {
        front_light_mode: u8,
        power: u8,
    },
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum ParamId588Output {
    DevGrNo,
    SubDevGrNo,
    VarNo,
    MajParSwVer,
    MinParSwVer,
    BuildNo,
}

#[derive(Debug, Serialize)]
struct TestResult {
    name: String,
    command: String,
    min: Option<f64>,
    max: Option<f64>,
    value: Option<f64>,
    passed: bool,
    raw_response: String,
}

#[derive(Debug, Serialize)]
struct TestSummary {
    results: Vec<TestResult>,
    overall_passed: bool,
}

#[derive(Debug, Clone, Copy)]
struct ParamId588Result {
    dev_gr_no: u16,
    sub_dev_gr_no: u8,
    var_no: u8,
    maj_par_sw_ver: u8,
    min_par_sw_ver: u8,
    build_no: u32,
}

impl fmt::Display for ParamId588Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DevGrNo={}, SubDevGrNo={}, VarNo={}, MajParSwVer={}, MinParSwVer={}, BuildNo={}",
            self.dev_gr_no,
            self.sub_dev_gr_no,
            self.var_no,
            self.maj_par_sw_ver,
            self.min_par_sw_ver,
            self.build_no
        )
    }
}

type ConnectMowerFn = unsafe extern "system" fn(u16) -> u8;
type CloseComPortFn = unsafe extern "system" fn() -> u8;
type SetReadTimeoutFn = unsafe extern "system" fn(u32);
type ParamId588Fn = unsafe extern "system" fn(
    *mut u8,
    *mut u16,
    *mut u8,
    *mut u8,
    *mut u8,
    *mut u8,
    *mut u32,
);
type ParamId606Fn = unsafe extern "system" fn(*mut u8, u8, u8);

struct CommDll {
    _lib: Library,
    connect_mower: ConnectMowerFn,
    close_com_port: CloseComPortFn,
    set_read_timeout: Option<SetReadTimeoutFn>,
    param_id588: ParamId588Fn,
    param_id606: ParamId606Fn,
}

struct CommSession {
    dll: CommDll,
}

impl CommSession {
    fn connect(dll: CommDll, port_number: u16, read_timeout_ms: u32) -> CommandResult<Self> {
        unsafe {
            if let Some(set_read_timeout) = dll.set_read_timeout {
                (set_read_timeout)(read_timeout_ms);
            }
            let code = (dll.connect_mower)(port_number);
            if code != 0 {
                return Err(format!(
                    "连接串口失败: {} (ReturnCode={})",
                    connect_return_code_message(code),
                    code
                ));
            }
        }

        Ok(Self { dll })
    }

    fn param_id588(&self) -> CommandResult<ParamId588Result> {
        let mut return_code: u8 = 9;
        let mut dev_gr_no: u16 = 0;
        let mut sub_dev_gr_no: u8 = 0;
        let mut var_no: u8 = 0;
        let mut maj_par_sw_ver: u8 = 0;
        let mut min_par_sw_ver: u8 = 0;
        let mut build_no: u32 = 0;

        unsafe {
            (self.dll.param_id588)(
                &mut return_code,
                &mut dev_gr_no,
                &mut sub_dev_gr_no,
                &mut var_no,
                &mut maj_par_sw_ver,
                &mut min_par_sw_ver,
                &mut build_no,
            );
        }

        if return_code != 0 {
            return Err(format!(
                "ParamId588 执行失败: {} (ReturnCode={})",
                return_code_message(return_code),
                return_code
            ));
        }

        Ok(ParamId588Result {
            dev_gr_no,
            sub_dev_gr_no,
            var_no,
            maj_par_sw_ver,
            min_par_sw_ver,
            build_no,
        })
    }

    fn param_id606(&self, front_light_mode: u8, power: u8) -> CommandResult<()> {
        let mut return_code: u8 = 9;

        unsafe {
            (self.dll.param_id606)(&mut return_code, front_light_mode, power);
        }

        if return_code != 0 {
            return Err(format!(
                "ParamId606 执行失败: {} (ReturnCode={})",
                return_code_message(return_code),
                return_code
            ));
        }

        Ok(())
    }
}

impl Drop for CommSession {
    fn drop(&mut self) {
        unsafe {
            let _ = (self.dll.close_com_port)();
        }
    }
}

fn return_code_message(code: u8) -> &'static str {
    match code {
        0 => "OK",
        1 => "Error, invalid data",
        2 => "Error, unknown",
        3 => "Error, not available",
        254 => "Exception error from CommDLL",
        255 => "NAK response",
        _ => "Unknown error",
    }
}

fn connect_return_code_message(code: u8) -> &'static str {
    match code {
        0 => "OK",
        1 => "Connection failed",
        2 => "Could not open COM port",
        _ => "Unknown error",
    }
}

unsafe fn load_symbol<T>(lib: &Library, names: &[&[u8]]) -> CommandResult<T>
where
    T: Copy,
{
    for name in names {
        if let Ok(symbol) = lib.get::<T>(*name) {
            return Ok(*symbol);
        }
    }

    let name_list = names
        .iter()
        .map(|name| String::from_utf8_lossy(name).trim_end_matches('\0').to_string())
        .collect::<Vec<_>>()
        .join("/");
    Err(format!("无法加载 DLL 符号: {name_list}"))
}

unsafe fn load_symbol_optional<T>(lib: &Library, names: &[&[u8]]) -> Option<T>
where
    T: Copy,
{
    for name in names {
        if let Ok(symbol) = lib.get::<T>(*name) {
            return Some(*symbol);
        }
    }
    None
}

impl CommDll {
    unsafe fn load(path: &Path) -> CommandResult<Self> {
        let lib = Library::new(path)
            .map_err(|err| format!("无法加载 CommDllv2.dll ({}): {err}", path.display()))?;

        let connect_mower = load_symbol::<ConnectMowerFn>(
            &lib,
            &[b"ConnectMower\0", b"ConnectMower@2\0", b"_ConnectMower@2\0"],
        )?;
        let close_com_port = load_symbol::<CloseComPortFn>(
            &lib,
            &[b"CloseCOMPort\0", b"CloseCOMPort@0\0", b"_CloseCOMPort@0\0"],
        )?;
        let set_read_timeout = load_symbol_optional::<SetReadTimeoutFn>(
            &lib,
            &[b"SetReadTimeout\0", b"SetReadTimeout@4\0", b"_SetReadTimeout@4\0"],
        );
        let param_id588 = load_symbol::<ParamId588Fn>(
            &lib,
            &[b"ParamId588\0", b"ParamId588@28\0", b"_ParamId588@28\0"],
        )?;
        let param_id606 = load_symbol::<ParamId606Fn>(
            &lib,
            &[b"ParamId606\0", b"ParamId606@12\0", b"_ParamId606@12\0"],
        )?;

        Ok(Self {
            _lib: lib,
            connect_mower,
            close_com_port,
            set_read_timeout,
            param_id588,
            param_id606,
        })
    }
}

fn read_config(app: &tauri::AppHandle) -> CommandResult<TestConfig> {
    let path = app
        .path()
        .resolve(
            "config/thresholds.json",
            tauri::path::BaseDirectory::Resource,
        )
        .map_err(|_| "无法找到配置文件路径".to_string())?;

    let data = std::fs::read_to_string(&path).map_err(|err| format!("无法读取配置文件: {err}"))?;
    serde_json::from_str(&data).map_err(|err| format!("配置文件解析失败: {err}"))
}

fn locate_dll(app: &tauri::AppHandle) -> CommandResult<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(resource_path) = app.path().resolve("CommDllv2.dll", tauri::path::BaseDirectory::Resource) {
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

fn command_name(command: &CommandSpec) -> &'static str {
    match command {
        CommandSpec::ParamId588 { .. } => "ParamId588",
        CommandSpec::ParamId606 { .. } => "ParamId606",
    }
}

#[tauri::command]
fn start_test(app: tauri::AppHandle) -> CommandResult<TestSummary> {
    let config = read_config(&app)?;
    let dll_path = locate_dll(&app)?;
    let dll = unsafe { CommDll::load(&dll_path)? };
    let session = CommSession::connect(dll, config.port_number, config.read_timeout_ms)?;

    let mut results = Vec::with_capacity(config.tests.len());

    for item in config.tests {
        let command_label = command_name(&item.command).to_string();
        match item.command {
            CommandSpec::ParamId588 { output, min, max } => {
                let response = session.param_id588()?;
                let value = pick_param_id588_value(output, response);
                let passed = value >= min && value <= max;

                results.push(TestResult {
                    name: item.name,
                    command: command_label,
                    min: Some(min),
                    max: Some(max),
                    value: Some(value),
                    passed,
                    raw_response: response.to_string(),
                });
            }
            CommandSpec::ParamId606 {
                front_light_mode,
                power,
            } => {
                session.param_id606(front_light_mode, power)?;

                results.push(TestResult {
                    name: item.name,
                    command: command_label,
                    min: None,
                    max: None,
                    value: None,
                    passed: true,
                    raw_response: format!(
                        "FrontLightMode={}, Power={}, ReturnCode=0",
                        front_light_mode, power
                    ),
                });
            }
        }
    }

    let overall_passed = results.iter().all(|item| item.passed);

    Ok(TestSummary {
        results,
        overall_passed,
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_test])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
