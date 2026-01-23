use std::ffi::CString;
use std::path::Path;

use libloading::Library;
use tracing::{error, info};

use crate::models::{ConnectionConfig, ParamId588Result};
use crate::types::CommandResult;

type ConnectMowerFn = unsafe extern "system" fn(u16) -> u8;
type ConnectMowerViaNetworkFn = unsafe extern "system" fn(*mut i8, *mut i8) -> u8;
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

pub struct CommDll {
    _lib: Library,
    connect_mower: ConnectMowerFn,
    connect_mower_via_network: Option<ConnectMowerViaNetworkFn>,
    close_com_port: CloseComPortFn,
    set_read_timeout: Option<SetReadTimeoutFn>,
    param_id588: ParamId588Fn,
    param_id606: ParamId606Fn,
}

pub struct CommSession {
    dll: CommDll,
}

impl CommSession {
    pub fn connect(
        dll: CommDll,
        config: &ConnectionConfig,
        read_timeout_ms: u32,
    ) -> CommandResult<Self> {
        info!("Connecting to device");
        unsafe {
            if let Some(set_read_timeout) = dll.set_read_timeout {
                (set_read_timeout)(read_timeout_ms);
            }
            let code = match config {
                ConnectionConfig::Serial { port_number } => (dll.connect_mower)(*port_number),
                ConnectionConfig::Network { ip_address, port } => {
                    info!("Using network connection {}:{}", ip_address, port);
                    let connect_fn = dll
                        .connect_mower_via_network
                        .ok_or_else(|| "DLL 未提供 ConnectMowerViaNetwork 接口".to_string())?;
                    let ip_c = CString::new(ip_address.as_str())
                        .map_err(|_| "IP 地址包含非法字符".to_string())?;
                    let port_c =
                        CString::new(port.as_str()).map_err(|_| "端口号包含非法字符".to_string())?;
                    (connect_fn)(ip_c.as_ptr() as *mut i8, port_c.as_ptr() as *mut i8)
                }
            };
            if code != 0 {
                error!("Connect failed with code {}", code);
                return Err(format!(
                    "连接串口失败: {} (ReturnCode={})",
                    connect_return_code_message(code),
                    code
                ));
            }
        }

        info!("Connection established");
        Ok(Self { dll })
    }

    pub fn param_id588(&self) -> CommandResult<ParamId588Result> {
        info!("Running ParamId588");
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
            error!("ParamId588 failed with code {}", return_code);
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

    pub fn param_id606(&self, front_light_mode: u8, power: u8) -> CommandResult<()> {
        info!(
            "Running ParamId606 FrontLightMode={} Power={}",
            front_light_mode, power
        );
        let mut return_code: u8 = 9;

        unsafe {
            (self.dll.param_id606)(&mut return_code, front_light_mode, power);
        }

        if return_code != 0 {
            error!("ParamId606 failed with code {}", return_code);
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
    pub unsafe fn load(path: &Path) -> CommandResult<Self> {
        let lib = Library::new(path)
            .map_err(|err| format!("无法加载 CommDllv2.dll ({}): {err}", path.display()))?;

        let connect_mower = load_symbol::<ConnectMowerFn>(
            &lib,
            &[b"ConnectMower\0", b"ConnectMower@2\0", b"_ConnectMower@2\0"],
        )?;
        let connect_mower_via_network = load_symbol_optional::<ConnectMowerViaNetworkFn>(
            &lib,
            &[
                b"ConnectMowerViaNetwork\0",
                b"ConnectMowerViaNetwork@8\0",
                b"_ConnectMowerViaNetwork@8\0",
            ],
        );
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
            connect_mower_via_network,
            close_com_port,
            set_read_timeout,
            param_id588,
            param_id606,
        })
    }
}
