use serde::{Deserialize, Serialize};
use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;

type CommandResult<T> = Result<T, String>;

#[derive(Debug, Deserialize)]
struct TestConfig {
    serial_port: String,
    baud_rate: u32,
    read_timeout_ms: u64,
    tests: Vec<TestItem>,
}

#[derive(Debug, Deserialize, Clone)]
struct TestItem {
    name: String,
    command: String,
    min: f64,
    max: f64,
}

#[derive(Debug, Serialize)]
struct TestResult {
    name: String,
    command: String,
    min: f64,
    max: f64,
    value: f64,
    passed: bool,
    raw_response: String,
}

#[derive(Debug, Serialize)]
struct TestSummary {
    results: Vec<TestResult>,
    overall_passed: bool,
}

fn read_config(app: &tauri::AppHandle) -> CommandResult<TestConfig> {
    let path = app
        .path_resolver()
        .resolve_resource("config/thresholds.json")
        .ok_or_else(|| "无法找到配置文件路径".to_string())?;
    let data = std::fs::read_to_string(&path)
        .map_err(|err| format!("无法读取配置文件: {err}"))?;
    serde_json::from_str(&data).map_err(|err| format!("配置文件解析失败: {err}"))
}

fn open_serial_port(config: &TestConfig) -> CommandResult<Box<dyn SerialPort>> {
    serialport::new(&config.serial_port, config.baud_rate)
        .timeout(Duration::from_millis(config.read_timeout_ms))
        .open()
        .map_err(|err| format!("无法打开串口 {}: {err}", config.serial_port))
}

fn read_line(port: &mut dyn SerialPort) -> CommandResult<String> {
    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        match port.read(&mut byte) {
            Ok(0) => continue,
            Ok(_) => {
                if byte[0] == b'\n' {
                    break;
                }
                buffer.push(byte[0]);
            }
            Err(err) => {
                return Err(format!("读取串口失败: {err}"));
            }
        }
    }
    let response = String::from_utf8_lossy(&buffer).trim().to_string();
    if response.is_empty() {
        return Err("串口返回空内容".to_string());
    }
    Ok(response)
}

fn parse_value(response: &str) -> CommandResult<f64> {
    response
        .split_whitespace()
        .find_map(|token| token.parse::<f64>().ok())
        .ok_or_else(|| format!("无法解析检测值: {response}"))
}

#[tauri::command]
fn start_test(app: tauri::AppHandle) -> CommandResult<TestSummary> {
    let config = read_config(&app)?;
    let mut port = open_serial_port(&config)?;

    let mut results = Vec::with_capacity(config.tests.len());

    for item in config.tests {
        port.write_all(item.command.as_bytes())
            .map_err(|err| format!("发送命令失败: {err}"))?;
        port.flush().map_err(|err| format!("串口刷新失败: {err}"))?;

        let response = read_line(&mut *port)?;
        let value = parse_value(&response)?;
        let passed = value >= item.min && value <= item.max;

        results.push(TestResult {
            name: item.name,
            command: item.command,
            min: item.min,
            max: item.max,
            value,
            passed,
            raw_response: response,
        });
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
