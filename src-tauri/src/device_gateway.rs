use std::path::PathBuf;

use tauri::Manager;
use tracing::info;

use crate::comm_dll::{CommDll, CommSession};
use crate::models::{
    ConnectionConfig, ParamId068Result, ParamId080Result, ParamId120Result, ParamId122Result,
    ParamId272Result, ParamId470Result, ParamId588Result, ParamId654Result, ParamId776Result,
    ParamId794Result,
};
use crate::types::{AppError, CommandResult};

pub trait DeviceGateway {
    fn param_id374(&self, test_mode: u8) -> CommandResult<()>;
    fn param_id068(&self) -> CommandResult<ParamId068Result>;
    fn param_id588(&self) -> CommandResult<ParamId588Result>;
    fn param_id654(&self) -> CommandResult<ParamId654Result>;
    fn param_id272(&self) -> CommandResult<ParamId272Result>;
    fn param_id080(&self) -> CommandResult<ParamId080Result>;
    fn param_id120(&self) -> CommandResult<ParamId120Result>;
    fn param_id122(&self) -> CommandResult<ParamId122Result>;
    fn param_id470(&self) -> CommandResult<ParamId470Result>;
    fn param_id468(&self, cutting_height_mm: u8) -> CommandResult<()>;
    fn param_id606(&self, front_light_mode: u8, power: u8) -> CommandResult<()>;
    fn param_id568(&self, on: u8) -> CommandResult<()>;
    fn param_id610(&self, rear_light_mode: u8) -> CommandResult<()>;
    fn param_id794(&self) -> CommandResult<ParamId794Result>;
    fn param_id776(&self, cmd: u8) -> CommandResult<ParamId776Result>;
}

pub trait DeviceGatewayFactory {
    fn create(
        &self,
        app: &tauri::AppHandle,
        connection: &ConnectionConfig,
        read_timeout_ms: u32,
    ) -> CommandResult<Box<dyn DeviceGateway>>;
}

pub struct DllDeviceGatewayFactory;

impl DeviceGatewayFactory for DllDeviceGatewayFactory {
    fn create(
        &self,
        app: &tauri::AppHandle,
        connection: &ConnectionConfig,
        read_timeout_ms: u32,
    ) -> CommandResult<Box<dyn DeviceGateway>> {
        let dll_path = locate_dll(app)?;
        let dll = unsafe { CommDll::load(&dll_path)? };
        let session = CommSession::connect(dll, connection, read_timeout_ms)?;
        Ok(Box::new(DllDeviceGateway { session }))
    }
}

struct DllDeviceGateway {
    session: CommSession,
}

impl DeviceGateway for DllDeviceGateway {
    fn param_id374(&self, test_mode: u8) -> CommandResult<()> {
        self.session.param_id374(test_mode)
    }

    fn param_id068(&self) -> CommandResult<ParamId068Result> {
        self.session.param_id068()
    }

    fn param_id588(&self) -> CommandResult<ParamId588Result> {
        self.session.param_id588()
    }

    fn param_id654(&self) -> CommandResult<ParamId654Result> {
        self.session.param_id654()
    }

    fn param_id272(&self) -> CommandResult<ParamId272Result> {
        self.session.param_id272()
    }

    fn param_id080(&self) -> CommandResult<ParamId080Result> {
        self.session.param_id080()
    }

    fn param_id120(&self) -> CommandResult<ParamId120Result> {
        self.session.param_id120()
    }

    fn param_id122(&self) -> CommandResult<ParamId122Result> {
        self.session.param_id122()
    }

    fn param_id470(&self) -> CommandResult<ParamId470Result> {
        self.session.param_id470()
    }

    fn param_id468(&self, cutting_height_mm: u8) -> CommandResult<()> {
        self.session.param_id468(cutting_height_mm)
    }

    fn param_id606(&self, front_light_mode: u8, power: u8) -> CommandResult<()> {
        self.session.param_id606(front_light_mode, power)
    }

    fn param_id568(&self, on: u8) -> CommandResult<()> {
        self.session.param_id568(on)
    }

    fn param_id610(&self, rear_light_mode: u8) -> CommandResult<()> {
        self.session.param_id610(rear_light_mode)
    }

    fn param_id794(&self) -> CommandResult<ParamId794Result> {
        self.session.param_id794()
    }

    fn param_id776(&self, cmd: u8) -> CommandResult<ParamId776Result> {
        self.session.param_id776(cmd)
    }
}

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

    Err(AppError::msg(
        "未找到 CommDllv2.dll，请确认 DLL 已放在资源目录或程序当前目录",
    ))
}
