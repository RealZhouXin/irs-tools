use std::ffi::CString;
use std::path::Path;

use libloading::Library;
use tracing::{error, info};

use crate::models::{
    ConnectionConfig, ParamId068Result, ParamId080Result, ParamId120Result, ParamId122Result,
    ParamId272Result, ParamId470Result, ParamId588Result, ParamId654Result, ParamId776Result,
    ParamId794Result,
};
use crate::types::{AppError, CommandResult};

type ConnectMowerFn = unsafe extern "system" fn(u16) -> u8;
type ConnectMowerViaNetworkFn = unsafe extern "system" fn(*mut i8, *mut i8) -> u8;
type CloseComPortFn = unsafe extern "system" fn() -> u8;
type SetReadTimeoutFn = unsafe extern "system" fn(u32);
type ParamId374Fn = unsafe extern "system" fn(*mut u8, u8);
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
type ParamId120Fn = unsafe extern "system" fn(
    *mut u8,
    *mut i16,
    *mut i16,
    *mut i16,
    *mut f32,
    *mut f32,
    *mut f32,
    *mut f32,
    *mut f32,
    *mut f32,
    *mut u32,
    *mut u32,
);
type ParamId122Fn = unsafe extern "system" fn(
    *mut u8,
    *mut i16,
    *mut i16,
    *mut i16,
    *mut i16,
    *mut i16,
    *mut i16,
    *mut i16,
    *mut i16,
);
type ParamId470Fn = unsafe extern "system" fn(*mut u8, *mut u8);
type ParamId468Fn = unsafe extern "system" fn(*mut u8, u8);
type ParamId606Fn = unsafe extern "system" fn(*mut u8, u8, u8);
type ParamId794Fn =
    unsafe extern "system" fn(*mut u8, *mut u16, *mut u8, *mut u8, *mut u8, *mut u8, *mut u32);
type ParamId776Fn = unsafe extern "system" fn(*mut u8, u8, *mut u8, *mut u8, *mut u8, *mut u8);

pub struct CommDll {
    _lib: Library,
    connect_mower: ConnectMowerFn,
    connect_mower_via_network: Option<ConnectMowerViaNetworkFn>,
    close_com_port: CloseComPortFn,
    set_read_timeout: Option<SetReadTimeoutFn>,
    param_id374: ParamId374Fn,
    param_id068: ParamId068Fn,
    param_id588: ParamId588Fn,
    param_id654: ParamId654Fn,
    param_id272: ParamId272Fn,
    param_id080: ParamId080Fn,
    param_id120: ParamId120Fn,
    param_id122: ParamId122Fn,
    param_id470: ParamId470Fn,
    param_id468: ParamId468Fn,
    param_id606: ParamId606Fn,
    param_id794: ParamId794Fn,
    param_id776: ParamId776Fn,
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

    pub fn param_id374(&self, test_mode: u8) -> CommandResult<()> {
        info!("Running ParamId374 TestMode={}", test_mode);
        let mut return_code: u8 = 9;

        unsafe {
            (self.dll.param_id374)(&mut return_code, test_mode);
        }

        if return_code != 0 {
            error!("ParamId374 failed with code {}", return_code);
            return Err(AppError::msg(format!(
                "ParamId374 执行失败: {} (ReturnCode={})",
                return_code_message(return_code),
                return_code
            )));
        }

        Ok(())
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

    pub fn param_id794(&self) -> CommandResult<ParamId794Result> {
        self.run_common_param_command("ParamId794", self.dll.param_id794)
            .map(
                |(dev_gr_no, sub_dev_gr_no, var_no, maj_par_sw_ver, min_par_sw_ver, build_no)| {
                    ParamId794Result {
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

    pub fn param_id120(&self) -> CommandResult<ParamId120Result> {
        info!("Running ParamId120");
        let mut return_code: u8 = 9;
        let mut pitch_angle: i16 = 0;
        let mut roll_angle: i16 = 0;
        let mut mover_vert_g_force: i16 = 0;
        let mut accel_x: f32 = 0.0;
        let mut accel_y: f32 = 0.0;
        let mut accel_z: f32 = 0.0;
        let mut gyro_x: f32 = 0.0;
        let mut gyro_y: f32 = 0.0;
        let mut gyro_z: f32 = 0.0;
        let mut utc_sec: u32 = 0;
        let mut utc_usec: u32 = 0;

        unsafe {
            (self.dll.param_id120)(
                &mut return_code,
                &mut pitch_angle,
                &mut roll_angle,
                &mut mover_vert_g_force,
                &mut accel_x,
                &mut accel_y,
                &mut accel_z,
                &mut gyro_x,
                &mut gyro_y,
                &mut gyro_z,
                &mut utc_sec,
                &mut utc_usec,
            );
        }

        if return_code != 0 {
            error!("ParamId120 failed with code {}", return_code);
            return Err(AppError::msg(format!(
                "ParamId120 执行失败: {} (ReturnCode={})",
                return_code_message(return_code),
                return_code
            )));
        }

        Ok(ParamId120Result {
            pitch_angle,
            roll_angle,
            mover_vert_g_force,
            accel_x,
            accel_y,
            accel_z,
            gyro_x,
            gyro_y,
            gyro_z,
            utc_sec,
            utc_usec,
        })
    }

    pub fn param_id122(&self) -> CommandResult<ParamId122Result> {
        info!("Running ParamId122");
        let mut return_code: u8 = 9;
        let mut mover_temp: i16 = 0;
        let mut cs_temp: i16 = 0;
        let mut bat_temp: i16 = 0;
        let mut cutting_motor_temp: i16 = 0;
        let mut right_wm_temp: i16 = 0;
        let mut left_wm_temp: i16 = 0;
        let mut app_board_temp: i16 = 0;
        let mut radar_temp: i16 = 0;

        unsafe {
            (self.dll.param_id122)(
                &mut return_code,
                &mut mover_temp,
                &mut cs_temp,
                &mut bat_temp,
                &mut cutting_motor_temp,
                &mut right_wm_temp,
                &mut left_wm_temp,
                &mut app_board_temp,
                &mut radar_temp,
            );
        }

        if return_code != 0 {
            error!("ParamId122 failed with code {}", return_code);
            return Err(AppError::msg(format!(
                "ParamId122 执行失败: {} (ReturnCode={})",
                return_code_message(return_code),
                return_code
            )));
        }

        Ok(ParamId122Result {
            mover_temp,
            cs_temp,
            bat_temp,
            cutting_motor_temp,
            right_wm_temp,
            left_wm_temp,
            app_board_temp,
            radar_temp,
        })
    }

    pub fn param_id470(&self) -> CommandResult<ParamId470Result> {
        info!("Running ParamId470");
        let mut return_code: u8 = 9;
        let mut cutting_height_mm: u8 = 0;

        unsafe {
            (self.dll.param_id470)(&mut return_code, &mut cutting_height_mm);
        }

        if return_code != 0 {
            error!("ParamId470 failed with code {}", return_code);
            return Err(AppError::msg(format!(
                "ParamId470 执行失败: {} (ReturnCode={})",
                return_code_message(return_code),
                return_code
            )));
        }

        Ok(ParamId470Result { cutting_height_mm })
    }

    pub fn param_id776(&self, cmd: u8) -> CommandResult<ParamId776Result> {
        info!("Running ParamId776 Cmd={}", cmd);
        let mut return_code: u8 = 9;
        let mut up_key: u8 = 0;
        let mut down_key: u8 = 0;
        let mut back_key: u8 = 0;
        let mut confirm_key: u8 = 0;

        unsafe {
            (self.dll.param_id776)(
                &mut return_code,
                cmd,
                &mut up_key,
                &mut down_key,
                &mut back_key,
                &mut confirm_key,
            );
        }

        if return_code != 0 {
            error!("ParamId776 failed with code {}", return_code);
            return Err(AppError::msg(format!(
                "ParamId776 执行失败: {} (ReturnCode={})",
                return_code_message(return_code),
                return_code
            )));
        }

        Ok(ParamId776Result {
            up_key,
            down_key,
            back_key,
            confirm_key,
        })
    }

    pub fn param_id468(&self, cutting_height_mm: u8) -> CommandResult<()> {
        info!("Running ParamId468 CuttingHeightMm={}", cutting_height_mm);
        let mut return_code: u8 = 9;

        unsafe {
            (self.dll.param_id468)(&mut return_code, cutting_height_mm);
        }

        if return_code != 0 {
            error!("ParamId468 failed with code {}", return_code);
            return Err(AppError::msg(format!(
                "ParamId468 执行失败: {} (ReturnCode={})",
                return_code_message(return_code),
                return_code
            )));
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
            let param_id374 = load_symbol::<ParamId374Fn>(
                &lib,
                &[b"ParamId374\0", b"ParamId374@8\0", b"_ParamId374@8\0"],
            )?;
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
            let param_id120 = load_symbol::<ParamId120Fn>(
                &lib,
                &[b"ParamId120\0", b"ParamId120@48\0", b"_ParamId120@48\0"],
            )?;
            let param_id122 = load_symbol::<ParamId122Fn>(
                &lib,
                &[b"ParamId122\0", b"ParamId122@36\0", b"_ParamId122@36\0"],
            )?;
            let param_id470 = load_symbol::<ParamId470Fn>(
                &lib,
                &[b"ParamId470\0", b"ParamId470@8\0", b"_ParamId470@8\0"],
            )?;
            let param_id468 = load_symbol::<ParamId468Fn>(
                &lib,
                &[b"ParamId468\0", b"ParamId468@8\0", b"_ParamId468@8\0"],
            )?;
            let param_id794 = load_symbol::<ParamId794Fn>(
                &lib,
                &[b"ParamId794\0", b"ParamId794@28\0", b"_ParamId794@28\0"],
            )?;
            let param_id776 = load_symbol::<ParamId776Fn>(
                &lib,
                &[b"ParamId776\0", b"ParamId776@24\0", b"_ParamId776@24\0"],
            )?;

            Ok(Self {
                _lib: lib,
                connect_mower,
                connect_mower_via_network,
                close_com_port,
                set_read_timeout,
                param_id374,
                param_id068,
                param_id588,
                param_id654,
                param_id272,
                param_id080,
                param_id120,
                param_id122,
                param_id470,
                param_id468,
                param_id606,
                param_id794,
                param_id776,
            })
        }
    }
}
