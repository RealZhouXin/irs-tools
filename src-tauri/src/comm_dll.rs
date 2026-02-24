use std::ffi::CString;
use std::path::Path;

use libloading::Library;
use tracing::{error, info};

use crate::models::{
    ConnectionConfig, ParamId068Result, ParamId080Result, ParamId272Result, ParamId588Result,
    ParamId654Result,
};
use crate::types::{AppError, CommandResult};

type ConnectMowerFn = unsafe extern "system" fn(u16) -> u8;
type ConnectMowerViaNetworkFn = unsafe extern "system" fn(*mut i8, *mut i8) -> u8;
type CloseComPortFn = unsafe extern "system" fn() -> u8;
type SetReadTimeoutFn = unsafe extern "system" fn(u32);
type ParamId588Fn =
    unsafe extern "system" fn(*mut u8, *mut u16, *mut u8, *mut u8, *mut u8, *mut u8, *mut u32);
type ParamId068Fn =
    unsafe extern "system" fn(*mut u8, *mut u16, *mut u8, *mut u8, *mut u8, *mut u8, *mut u32);
type ParamId654Fn =
    unsafe extern "system" fn(*mut u8, *mut u16, *mut u8, *mut u8, *mut u8, *mut u8, *mut u32);
type ParamId272Fn = unsafe extern "system" fn(
    *mut u8,
    *mut u32,
    *mut u16,
    *mut u32,
    *mut u32,
    *mut u32,
    *mut u32,
    *mut u32,
    *mut u16,
    *mut u16,
    *mut u16,
    *mut u16,
    *mut u32,
    *mut u16,
    *mut u32,
);
type ParamId080Fn = unsafe extern "system" fn(
    *mut u8,
    *mut u8,
    *mut u8,
    *mut u32,
    *mut u8,
    *mut u16,
    *mut u8,
    *mut u8,
    *mut u8,
    *mut u16,
    *mut u8,
);
type ParamId606Fn = unsafe extern "system" fn(*mut u8, u8, u8);

pub struct CommDll {
    _lib: Library,
    connect_mower: ConnectMowerFn,
    connect_mower_via_network: Option<ConnectMowerViaNetworkFn>,
    close_com_port: CloseComPortFn,
    set_read_timeout: Option<SetReadTimeoutFn>,
    param_id068: ParamId068Fn,
    param_id588: ParamId588Fn,
    param_id654: ParamId654Fn,
    param_id272: ParamId272Fn,
    param_id080: ParamId080Fn,
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
                    let port_c = CString::new(port.as_str())
                        .map_err(|_| "端口号包含非法字符".to_string())?;
                    (connect_fn)(ip_c.as_ptr() as *mut i8, port_c.as_ptr() as *mut i8)
                }
            };
            if code != 0 {
                error!("Connect failed with code {}", code);
                return Err(AppError::msg(format!(
                    "连接串口失败: {} (ReturnCode={})",
                    connect_return_code_message(code),
                    code
                )));
            }
        }

        info!("Connection established");
        Ok(Self { dll })
    }

    pub fn param_id588(&self) -> CommandResult<ParamId588Result> {
        self.run_common_param_command("ParamId588", self.dll.param_id588)
            .map(
                |(dev_gr_no, sub_dev_gr_no, var_no, maj_par_sw_ver, min_par_sw_ver, build_no)| {
                    ParamId588Result {
                        dev_gr_no,
                        sub_dev_gr_no,
                        var_no,
                        maj_par_sw_ver,
                        min_par_sw_ver,
                        build_no,
                    }
                },
            )
    }

    pub fn param_id068(&self) -> CommandResult<ParamId068Result> {
        self.run_common_param_command("ParamId068", self.dll.param_id068)
            .map(
                |(dev_gr_no, sub_dev_gr_no, var_no, maj_par_sw_ver, min_par_sw_ver, build_no)| {
                    ParamId068Result {
                        dev_gr_no,
                        sub_dev_gr_no,
                        var_no,
                        maj_par_sw_ver,
                        min_par_sw_ver,
                        build_no,
                    }
                },
            )
    }

    pub fn param_id654(&self) -> CommandResult<ParamId654Result> {
        self.run_common_param_command("ParamId654", self.dll.param_id654)
            .map(
                |(dev_gr_no, sub_dev_gr_no, var_no, maj_par_sw_ver, min_par_sw_ver, build_no)| {
                    ParamId654Result {
                        dev_gr_no,
                        sub_dev_gr_no,
                        var_no,
                        maj_par_sw_ver,
                        min_par_sw_ver,
                        build_no,
                    }
                },
            )
    }

    fn run_common_param_command(
        &self,
        name: &str,
        func: unsafe extern "system" fn(
            *mut u8,
            *mut u16,
            *mut u8,
            *mut u8,
            *mut u8,
            *mut u8,
            *mut u32,
        ),
    ) -> CommandResult<(u16, u8, u8, u8, u8, u32)> {
        info!("Running {}", name);
        let mut return_code: u8 = 9;
        let mut dev_gr_no: u16 = 0;
        let mut sub_dev_gr_no: u8 = 0;
        let mut var_no: u8 = 0;
        let mut maj_par_sw_ver: u8 = 0;
        let mut min_par_sw_ver: u8 = 0;
        let mut build_no: u32 = 0;

        unsafe {
            func(
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
            error!("{} failed with code {}", name, return_code);
            return Err(AppError::msg(format!(
                "{} 执行失败: {} (ReturnCode={})",
                name,
                return_code_message(return_code),
                return_code
            )));
        }

        Ok((
            dev_gr_no,
            sub_dev_gr_no,
            var_no,
            maj_par_sw_ver,
            min_par_sw_ver,
            build_no,
        ))
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
            return Err(AppError::msg(format!(
                "ParamId606 执行失败: {} (ReturnCode={})",
                return_code_message(return_code),
                return_code
            )));
        }

        Ok(())
    }

    pub fn param_id272(&self) -> CommandResult<ParamId272Result> {
        info!("Running ParamId272");
        let mut return_code: u8 = 9;
        let mut batt_pack_pn: u32 = 0;
        let mut batt_pack_rev: u16 = 0;
        let mut batt_pack_prod_date: u32 = 0;
        let mut batt_sw_ver: u32 = 0;
        let mut batt_ser_no: u32 = 0;
        let mut batt_dev_gr_no: u32 = 0;
        let mut batt_sub_dev_no: u32 = 0;
        let mut batt_var_no: u16 = 0;
        let mut bms_dev_gr_no: u16 = 0;
        let mut bms_sub_dev_no: u16 = 0;
        let mut bms_var_no: u16 = 0;
        let mut bms_pcba_pn: u32 = 0;
        let mut bms_pcba_rev: u16 = 0;
        let mut bms_temp_sensor_type: u32 = 0;

        unsafe {
            (self.dll.param_id272)(
                &mut return_code,
                &mut batt_pack_pn,
                &mut batt_pack_rev,
                &mut batt_pack_prod_date,
                &mut batt_sw_ver,
                &mut batt_ser_no,
                &mut batt_dev_gr_no,
                &mut batt_sub_dev_no,
                &mut batt_var_no,
                &mut bms_dev_gr_no,
                &mut bms_sub_dev_no,
                &mut bms_var_no,
                &mut bms_pcba_pn,
                &mut bms_pcba_rev,
                &mut bms_temp_sensor_type,
            );
        }

        if return_code != 0 {
            error!("ParamId272 failed with code {}", return_code);
            return Err(AppError::msg(format!(
                "ParamId272 执行失败: {} (ReturnCode={})",
                return_code_message(return_code),
                return_code
            )));
        }

        Ok(ParamId272Result {
            batt_pack_pn,
            batt_pack_rev,
            batt_pack_prod_date,
            batt_sw_ver,
            batt_ser_no,
            batt_dev_gr_no,
            batt_sub_dev_no,
            batt_var_no,
            bms_dev_gr_no,
            bms_sub_dev_no,
            bms_var_no,
            bms_pcba_pn,
            bms_pcba_rev,
            bms_temp_sensor_type,
        })
    }

    pub fn param_id080(&self) -> CommandResult<ParamId080Result> {
        info!("Running ParamId080");
        let mut return_code: u8 = 9;
        let mut mower_main_p: u8 = 0;
        let mut mower_sub_state: u8 = 0;
        let mut time_stp_nxt_start: u32 = 0;
        let mut batt_stat: u8 = 0;
        let mut stat_flags: u16 = 0;
        let mut wrless_con_stat: u8 = 0;
        let mut sign_quality: u8 = 0;
        let mut source_for_next_start_stop: u8 = 0;
        let mut notify: u16 = 0;
        let mut configuration_hash: u8 = 0;

        unsafe {
            (self.dll.param_id080)(
                &mut return_code,
                &mut mower_main_p,
                &mut mower_sub_state,
                &mut time_stp_nxt_start,
                &mut batt_stat,
                &mut stat_flags,
                &mut wrless_con_stat,
                &mut sign_quality,
                &mut source_for_next_start_stop,
                &mut notify,
                &mut configuration_hash,
            );
        }

        if return_code != 0 {
            error!("ParamId080 failed with code {}", return_code);
            return Err(AppError::msg(format!(
                "ParamId080 执行失败: {} (ReturnCode={})",
                return_code_message(return_code),
                return_code
            )));
        }

        Ok(ParamId080Result {
            mower_main_p,
            mower_sub_state,
            time_stp_nxt_start,
            batt_stat,
            stat_flags,
            wrless_con_stat,
            sign_quality,
            source_for_next_start_stop,
            notify,
            configuration_hash,
        })
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
        if let Ok(symbol) = unsafe { lib.get::<T>(*name) } {
            return Ok(*symbol);
        }
    }

    let name_list = names
        .iter()
        .map(|name| {
            String::from_utf8_lossy(name)
                .trim_end_matches('\0')
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("/");
    Err(AppError::msg(format!("无法加载 DLL 符号: {name_list}")))
}

unsafe fn load_symbol_optional<T>(lib: &Library, names: &[&[u8]]) -> Option<T>
where
    T: Copy,
{
    for name in names {
        if let Ok(symbol) = unsafe { lib.get::<T>(*name) } {
            return Some(*symbol);
        }
    }
    None
}

impl CommDll {
    pub unsafe fn load(path: &Path) -> CommandResult<Self> {
        unsafe {
            let lib =
                Library::new(path).map_err(|err| AppError::dll("无法加载 CommDllv2.dll", err))?;

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
                &[
                    b"SetReadTimeout\0",
                    b"SetReadTimeout@4\0",
                    b"_SetReadTimeout@4\0",
                ],
            );
            let param_id588 = load_symbol::<ParamId588Fn>(
                &lib,
                &[b"ParamId588\0", b"ParamId588@28\0", b"_ParamId588@28\0"],
            )?;
            let param_id068 = load_symbol::<ParamId068Fn>(
                &lib,
                &[b"ParamId068\0", b"ParamId068@28\0", b"_ParamId068@28\0"],
            )?;
            let param_id606 = load_symbol::<ParamId606Fn>(
                &lib,
                &[b"ParamId606\0", b"ParamId606@12\0", b"_ParamId606@12\0"],
            )?;
            let param_id654 = load_symbol::<ParamId654Fn>(
                &lib,
                &[b"ParamId654\0", b"ParamId654@28\0", b"_ParamId654@28\0"],
            )?;
            let param_id272 = load_symbol::<ParamId272Fn>(
                &lib,
                &[b"ParamId272\0", b"ParamId272@60\0", b"_ParamId272@60\0"],
            )?;
            let param_id080 = load_symbol::<ParamId080Fn>(
                &lib,
                &[b"ParamId080\0", b"ParamId080@16\0", b"_ParamId080@16\0"],
            )?;

            Ok(Self {
                _lib: lib,
                connect_mower,
                connect_mower_via_network,
                close_com_port,
                set_read_timeout,
                param_id068,
                param_id588,
                param_id654,
                param_id272,
                param_id080,
                param_id606,
            })
        }
    }
}
