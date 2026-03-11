use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestConfig {
    pub connection: ConnectionConfig,
    pub read_timeout_ms: u32,
    #[serde(default)]
    pub stages: Vec<String>,
    pub tests: Vec<TestGroup>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ConnectionConfig {
    Serial { port_number: u16 },
    Network { ip_address: String, port: String },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BaseConfig {
    pub connection: ConnectionConfig,
    pub read_timeout_ms: u32,
    #[serde(default)]
    pub log_level: LogLevel,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestGroup {
    pub name: String,
    #[serde(default)]
    pub names: HashMap<String, String>,
    #[serde(default)]
    pub stage: String,
    #[serde(flatten)]
    pub command: CommandGroupSpec,
}

fn default_lift_sensor_threshold() -> u8 {
    0
}

fn default_wheel_motor_speed() -> i16 {
    45
}

fn default_wheel_sample_interval_ms() -> u64 {
    1000
}

fn default_wheel_sample_count() -> u8 {
    3
}

fn default_wheel_inactive_max_speed() -> f64 {
    10.0
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum CommandGroupSpec {
    ParamId068 {
        checks: Vec<VersionCheck>,
    },
    ParamId588 {
        checks: Vec<VersionCheck>,
    },
    ParamId654 {
        checks: Vec<VersionCheck>,
    },
    ParamId272 {
        checks: Vec<ParamId272Check>,
    },
    ParamId526 {
        checks: Vec<ParamId526Check>,
    },
    ParamId096 {
        checks: Vec<ParamId096Check>,
    },
    ParamId080 {
        checks: Vec<ParamId080Check>,
    },
    ParamId080EmergencyStop {
        timeout_ms: u64,
    },
    ParamId118CollisionBar {
        timeout_ms: u64,
    },
    ParamId118LiftSensor {
        timeout_ms: u64,
        #[serde(default = "default_lift_sensor_threshold")]
        lift_threshold: u8,
    },
    ParamId120 {
        checks: Vec<ParamId120Check>,
    },
    ParamId122 {
        checks: Vec<ParamId122Check>,
    },
    ParamId470 {
        checks: Vec<ParamId470Check>,
    },
    ParamId468 {
        cutting_height_mm: u8,
    },
    CuttingHeightSetAndVerify {
        cutting_height_mm: u8,
        wait_ms: u64,
        checks: Vec<ParamId470Check>,
    },
    ParamId606 {
        front_light_mode: u8,
        power: u8,
    },
    WheelMotorTest {
        #[serde(default = "default_wheel_motor_speed")]
        right_motor_speed: i16,
        #[serde(default = "default_wheel_motor_speed")]
        left_motor_speed: i16,
        #[serde(default = "default_wheel_sample_interval_ms")]
        sample_interval_ms: u64,
        #[serde(default = "default_wheel_sample_count")]
        sample_count: u8,
        #[serde(default = "default_wheel_inactive_max_speed")]
        right_test_inactive_max_speed_mm_s: f64,
        #[serde(default = "default_wheel_inactive_max_speed")]
        left_test_inactive_max_speed_mm_s: f64,
        checks: Vec<WheelMotorCheck>,
    },
    ParamId568,
    ParamId610,
    ParamId794 {
        checks: Vec<VersionCheck>,
    },
    ParamId796 {
        checks: Vec<ParamId796Check>,
    },
    ParamId798,
    ParamId776 {
        timeout_ms: u64,
    },
}

pub trait CheckConfig {
    type OutputEnum: Copy;
    fn name(&self) -> &str;
    fn output(&self) -> Self::OutputEnum;
    fn min(&self) -> f64;
    fn max(&self) -> f64;
}

impl CheckConfig for ParamId272Check {
    type OutputEnum = ParamId272Output;
    fn name(&self) -> &str {
        &self.name
    }
    fn output(&self) -> Self::OutputEnum {
        self.output
    }
    fn min(&self) -> f64 {
        self.min
    }
    fn max(&self) -> f64 {
        self.max
    }
}

impl CheckConfig for ParamId526Check {
    type OutputEnum = ParamId526Output;
    fn name(&self) -> &str {
        &self.name
    }
    fn output(&self) -> Self::OutputEnum {
        self.output
    }
    fn min(&self) -> f64 {
        self.min
    }
    fn max(&self) -> f64 {
        self.max
    }
}

impl CheckConfig for ParamId096Check {
    type OutputEnum = ParamId096Output;
    fn name(&self) -> &str {
        &self.name
    }
    fn output(&self) -> Self::OutputEnum {
        self.output
    }
    fn min(&self) -> f64 {
        self.min
    }
    fn max(&self) -> f64 {
        self.max
    }
}

impl CheckConfig for ParamId080Check {
    type OutputEnum = ParamId080Output;
    fn name(&self) -> &str {
        &self.name
    }
    fn output(&self) -> Self::OutputEnum {
        self.output
    }
    fn min(&self) -> f64 {
        self.min
    }
    fn max(&self) -> f64 {
        self.max
    }
}

impl CheckConfig for ParamId120Check {
    type OutputEnum = ParamId120Output;
    fn name(&self) -> &str {
        &self.name
    }
    fn output(&self) -> Self::OutputEnum {
        self.output
    }
    fn min(&self) -> f64 {
        self.min
    }
    fn max(&self) -> f64 {
        self.max
    }
}

impl CheckConfig for ParamId122Check {
    type OutputEnum = ParamId122Output;
    fn name(&self) -> &str {
        &self.name
    }
    fn output(&self) -> Self::OutputEnum {
        self.output
    }
    fn min(&self) -> f64 {
        self.min
    }
    fn max(&self) -> f64 {
        self.max
    }
}

impl CheckConfig for ParamId470Check {
    type OutputEnum = ParamId470Output;
    fn name(&self) -> &str {
        &self.name
    }
    fn output(&self) -> Self::OutputEnum {
        self.output
    }
    fn min(&self) -> f64 {
        self.min
    }
    fn max(&self) -> f64 {
        self.max
    }
}

impl CheckConfig for ParamId796Check {
    type OutputEnum = ParamId796Output;
    fn name(&self) -> &str {
        &self.name
    }
    fn output(&self) -> Self::OutputEnum {
        self.output
    }
    fn min(&self) -> f64 {
        self.min
    }
    fn max(&self) -> f64 {
        self.max
    }
}

impl CheckConfig for WheelMotorCheck {
    type OutputEnum = WheelMotorOutput;
    fn name(&self) -> &str {
        &self.name
    }
    fn output(&self) -> Self::OutputEnum {
        self.output
    }
    fn min(&self) -> f64 {
        self.min
    }
    fn max(&self) -> f64 {
        self.max
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionCheck {
    pub name: String,
    pub min: String,
    pub max: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamId272Check {
    pub name: String,
    pub output: ParamId272Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId272Output {
    BattPackPn,
    BattPackRev,
    BattPackProdDate,
    BattSwVer,
    BattSerNo,
    BattDevGrNo,
    BattSubDevNo,
    BattVarNo,
    BmsDevGrNo,
    BmsSubDevNo,
    BmsVarNo,
    BmsPcbaPn,
    BmsPcbaRev,
    BmsTempSensorType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamId526Check {
    pub name: String,
    pub output: ParamId526Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId526Output {
    PcbDeGrNo,
    PcbSubDeNo,
    PcbVarNo,
    PcbPn,
    PcbRev,
    PcbSerNo,
    PcbProdTime,
    PcbExtFlash,
    PcbExtEeprom,
    PcbAccelerometer,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamId096Check {
    pub name: String,
    pub output: ParamId096Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId096Output {
    GprsLteStat,
    GprsLteSignQual,
    GnssHwStat,
    SimStatus,
    BleHwStat,
    GprsLteConnStat,
    BleConnStat,
    WifiConnStat,
    WifiHwStat,
    LoraConnStat,
    LoraHwStat,
    RtkHwStat,
    RtkConnStat,
    ConnectedRaSerial,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamId080Check {
    pub name: String,
    pub output: ParamId080Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId080Output {
    MowerMainP,
    MowerSubState,
    TimeStpNxtStart,
    BattStat,
    StatFlags,
    WrlessConStat,
    SignQuality,
    SourceForNextStartStop,
    Notify,
    ConfigurationHash,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamId120Check {
    pub name: String,
    pub output: ParamId120Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId120Output {
    PitchAngle,
    RollAngle,
    MoverVertGForce,
    AccelX,
    AccelY,
    AccelZ,
    GyroX,
    GyroY,
    GyroZ,
    UtcSec,
    UtcUsec,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamId122Check {
    pub name: String,
    pub output: ParamId122Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId122Output {
    MoverTemp,
    CsTemp,
    BatTemp,
    CuttingMotorTemp,
    RightWmTemp,
    LeftWmTemp,
    AppBoardTemp,
    RadarTemp,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamId470Check {
    pub name: String,
    pub output: ParamId470Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId470Output {
    CuttingHeightMm,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamId796Check {
    pub name: String,
    pub output: ParamId796Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId796Output {
    MqttStatus,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WheelMotorCheck {
    pub name: String,
    pub output: WheelMotorOutput,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WheelMotorOutput {
    RightWheelMotor,
    LeftWheelMotor,
}

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub value: Option<f64>,
    pub display_min: Option<String>,
    pub display_max: Option<String>,
    pub display_value: Option<String>,
    pub passed: bool,
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub name: String,
    pub names: HashMap<String, String>,
    pub stage: String,
    pub command: String,
    pub passed: bool,
    pub raw_response: String,
    pub checks: Vec<CheckResult>,
}

#[derive(Debug, Serialize)]
pub struct TestSummary {
    pub results: Vec<TestResult>,
    pub overall_passed: bool,
}

pub trait CheckableResult {
    type OutputEnum;
    fn get_value(&self, output: Self::OutputEnum) -> f64;
}

impl CheckableResult for ParamId272Result {
    type OutputEnum = ParamId272Output;
    fn get_value(&self, output: Self::OutputEnum) -> f64 {
        match output {
            ParamId272Output::BattPackPn => self.batt_pack_pn as f64,
            ParamId272Output::BattPackRev => self.batt_pack_rev as f64,
            ParamId272Output::BattPackProdDate => self.batt_pack_prod_date as f64,
            ParamId272Output::BattSwVer => self.batt_sw_ver as f64,
            ParamId272Output::BattSerNo => self.batt_ser_no as f64,
            ParamId272Output::BattDevGrNo => self.batt_dev_gr_no as f64,
            ParamId272Output::BattSubDevNo => self.batt_sub_dev_no as f64,
            ParamId272Output::BattVarNo => self.batt_var_no as f64,
            ParamId272Output::BmsDevGrNo => self.bms_dev_gr_no as f64,
            ParamId272Output::BmsSubDevNo => self.bms_sub_dev_no as f64,
            ParamId272Output::BmsVarNo => self.bms_var_no as f64,
            ParamId272Output::BmsPcbaPn => self.bms_pcba_pn as f64,
            ParamId272Output::BmsPcbaRev => self.bms_pcba_rev as f64,
            ParamId272Output::BmsTempSensorType => self.bms_temp_sensor_type as f64,
        }
    }
}

impl CheckableResult for ParamId526Result {
    type OutputEnum = ParamId526Output;
    fn get_value(&self, output: Self::OutputEnum) -> f64 {
        match output {
            ParamId526Output::PcbDeGrNo => self.pcb_de_gr_no as f64,
            ParamId526Output::PcbSubDeNo => self.pcb_sub_de_no as f64,
            ParamId526Output::PcbVarNo => self.pcb_var_no as f64,
            ParamId526Output::PcbPn => self.pcb_pn as f64,
            ParamId526Output::PcbRev => self.pcb_rev as f64,
            ParamId526Output::PcbSerNo => self.pcb_ser_no as f64,
            ParamId526Output::PcbProdTime => self.pcb_prod_time as f64,
            ParamId526Output::PcbExtFlash => self.pcb_ext_flash as f64,
            ParamId526Output::PcbExtEeprom => self.pcb_ext_eeprom as f64,
            ParamId526Output::PcbAccelerometer => self.pcb_accelerometer as f64,
        }
    }
}

impl CheckableResult for ParamId096Result {
    type OutputEnum = ParamId096Output;
    fn get_value(&self, output: Self::OutputEnum) -> f64 {
        match output {
            ParamId096Output::GprsLteStat => self.gprs_lte_stat as f64,
            ParamId096Output::GprsLteSignQual => self.gprs_lte_sign_qual as f64,
            ParamId096Output::GnssHwStat => self.gnss_hw_stat as f64,
            ParamId096Output::SimStatus => self.sim_status as f64,
            ParamId096Output::BleHwStat => self.ble_hw_stat as f64,
            ParamId096Output::GprsLteConnStat => self.gprs_lte_conn_stat as f64,
            ParamId096Output::BleConnStat => self.ble_conn_stat as f64,
            ParamId096Output::WifiConnStat => self.wifi_conn_stat as f64,
            ParamId096Output::WifiHwStat => self.wifi_hw_stat as f64,
            ParamId096Output::LoraConnStat => self.lora_conn_stat as f64,
            ParamId096Output::LoraHwStat => self.lora_hw_stat as f64,
            ParamId096Output::RtkHwStat => self.rtk_hw_stat as f64,
            ParamId096Output::RtkConnStat => self.rtk_conn_stat as f64,
            ParamId096Output::ConnectedRaSerial => self.connected_ra_serial as f64,
        }
    }
}

impl CheckableResult for ParamId080Result {
    type OutputEnum = ParamId080Output;
    fn get_value(&self, output: Self::OutputEnum) -> f64 {
        match output {
            ParamId080Output::MowerMainP => self.mower_main_p as f64,
            ParamId080Output::MowerSubState => self.mower_sub_state as f64,
            ParamId080Output::TimeStpNxtStart => self.time_stp_nxt_start as f64,
            ParamId080Output::BattStat => self.batt_stat as f64,
            ParamId080Output::StatFlags => self.stat_flags as f64,
            ParamId080Output::WrlessConStat => self.wrless_con_stat as f64,
            ParamId080Output::SignQuality => self.sign_quality as f64,
            ParamId080Output::SourceForNextStartStop => self.source_for_next_start_stop as f64,
            ParamId080Output::Notify => self.notify as f64,
            ParamId080Output::ConfigurationHash => self.configuration_hash as f64,
        }
    }
}

impl CheckableResult for ParamId120Result {
    type OutputEnum = ParamId120Output;
    fn get_value(&self, output: Self::OutputEnum) -> f64 {
        match output {
            ParamId120Output::PitchAngle => self.pitch_angle as f64,
            ParamId120Output::RollAngle => self.roll_angle as f64,
            ParamId120Output::MoverVertGForce => self.mover_vert_g_force as f64,
            ParamId120Output::AccelX => self.accel_x as f64,
            ParamId120Output::AccelY => self.accel_y as f64,
            ParamId120Output::AccelZ => self.accel_z as f64,
            ParamId120Output::GyroX => self.gyro_x as f64,
            ParamId120Output::GyroY => self.gyro_y as f64,
            ParamId120Output::GyroZ => self.gyro_z as f64,
            ParamId120Output::UtcSec => self.utc_sec as f64,
            ParamId120Output::UtcUsec => self.utc_usec as f64,
        }
    }
}

impl CheckableResult for ParamId122Result {
    type OutputEnum = ParamId122Output;
    fn get_value(&self, output: Self::OutputEnum) -> f64 {
        match output {
            ParamId122Output::MoverTemp => self.mover_temp as f64,
            ParamId122Output::CsTemp => self.cs_temp as f64,
            ParamId122Output::BatTemp => self.bat_temp as f64,
            ParamId122Output::CuttingMotorTemp => self.cutting_motor_temp as f64,
            ParamId122Output::RightWmTemp => self.right_wm_temp as f64,
            ParamId122Output::LeftWmTemp => self.left_wm_temp as f64,
            ParamId122Output::AppBoardTemp => self.app_board_temp as f64,
            ParamId122Output::RadarTemp => self.radar_temp as f64,
        }
    }
}

impl CheckableResult for ParamId470Result {
    type OutputEnum = ParamId470Output;
    fn get_value(&self, output: Self::OutputEnum) -> f64 {
        match output {
            ParamId470Output::CuttingHeightMm => self.cutting_height_mm as f64,
        }
    }
}

impl CheckableResult for ParamId796Result {
    type OutputEnum = ParamId796Output;
    fn get_value(&self, output: Self::OutputEnum) -> f64 {
        match output {
            ParamId796Output::MqttStatus => self.mqtt_status as f64,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId588Result {
    pub dev_gr_no: u16,
    pub sub_dev_gr_no: u8,
    pub var_no: u8,
    pub maj_par_sw_ver: u8,
    pub min_par_sw_ver: u8,
    pub build_no: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId068Result {
    pub dev_gr_no: u16,
    pub sub_dev_gr_no: u8,
    pub var_no: u8,
    pub maj_par_sw_ver: u8,
    pub min_par_sw_ver: u8,
    pub build_no: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId654Result {
    pub dev_gr_no: u16,
    pub sub_dev_gr_no: u8,
    pub var_no: u8,
    pub maj_par_sw_ver: u8,
    pub min_par_sw_ver: u8,
    pub build_no: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId272Result {
    pub batt_pack_pn: u32,
    pub batt_pack_rev: u16,
    pub batt_pack_prod_date: u32,
    pub batt_sw_ver: u32,
    pub batt_ser_no: u32,
    pub batt_dev_gr_no: u32,
    pub batt_sub_dev_no: u32,
    pub batt_var_no: u16,
    pub bms_dev_gr_no: u16,
    pub bms_sub_dev_no: u16,
    pub bms_var_no: u16,
    pub bms_pcba_pn: u32,
    pub bms_pcba_rev: u16,
    pub bms_temp_sensor_type: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId526Result {
    pub pcb_de_gr_no: u16,
    pub pcb_sub_de_no: u8,
    pub pcb_var_no: u8,
    pub pcb_pn: u32,
    pub pcb_rev: u16,
    pub pcb_ser_no: u32,
    pub pcb_prod_time: u32,
    pub pcb_ext_flash: u8,
    pub pcb_ext_eeprom: u8,
    pub pcb_accelerometer: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId096Result {
    pub gprs_lte_stat: u8,
    pub gprs_lte_sign_qual: u8,
    pub gnss_hw_stat: u8,
    pub sim_status: u8,
    pub ble_hw_stat: u8,
    pub gprs_lte_conn_stat: u8,
    pub ble_conn_stat: u8,
    pub wifi_conn_stat: u8,
    pub wifi_hw_stat: u8,
    pub lora_conn_stat: u8,
    pub lora_hw_stat: u8,
    pub rtk_hw_stat: u8,
    pub rtk_conn_stat: u8,
    pub connected_ra_serial: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId080Result {
    pub mower_main_p: u8,
    pub mower_sub_state: u8,
    pub time_stp_nxt_start: u32,
    pub batt_stat: u8,
    pub stat_flags: u16,
    pub wrless_con_stat: u8,
    pub sign_quality: u8,
    pub source_for_next_start_stop: u8,
    pub notify: u16,
    pub configuration_hash: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId118Result {
    pub collision_sen: u8,
    pub lift_sen: u8,
    pub status_flags: u16,
    pub stop_sen: u8,
    pub disabling_sen: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId120Result {
    pub pitch_angle: i16,
    pub roll_angle: i16,
    pub mover_vert_g_force: i16,
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
    pub utc_sec: u32,
    pub utc_usec: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId122Result {
    pub mover_temp: i16,
    pub cs_temp: i16,
    pub bat_temp: i16,
    pub cutting_motor_temp: i16,
    pub right_wm_temp: i16,
    pub left_wm_temp: i16,
    pub app_board_temp: i16,
    pub radar_temp: i16,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId470Result {
    pub cutting_height_mm: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId108Result {
    pub batt_vol_mw: u16,
    pub batt_curr: i16,
    pub batt_en_lvl: i16,
    pub batt_temp: i16,
    pub main_voltage: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId114Result {
    pub right_whl_motor_p: i8,
    pub right_whl_motor_curr: i16,
    pub right_whl_motor_sp: i16,
    pub left_whl_motor_p: i8,
    pub lef_whl_motor_curr: i16,
    pub lef_whl_motor_sp: i16,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId794Result {
    pub dev_gr_no: u16,
    pub sub_dev_gr_no: u8,
    pub var_no: u8,
    pub maj_par_sw_ver: u8,
    pub min_par_sw_ver: u8,
    pub build_no: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId796Result {
    pub mqtt_status: u8,
}

#[derive(Debug, Clone)]
pub struct ParamId798Result {
    pub version: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamId776Result {
    pub up_key: u8,
    pub down_key: u8,
    pub back_key: u8,
    pub confirm_key: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyStatePayload {
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub back_pressed: bool,
    pub confirm_pressed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceSnPayload {
    pub sn: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontLightConfirmRequestPayload {
    pub name: String,
    pub stage: String,
    pub front_light_mode: u8,
    pub power: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpeakerConfirmRequestPayload {
    pub name: String,
    pub stage: String,
    pub on: u8,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RearLightColor {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, Serialize)]
pub struct RearLightConfirmRequestPayload {
    pub name: String,
    pub stage: String,
    pub rear_light_mode: u8,
    pub expected_color: RearLightColor,
    pub step_index: u8,
    pub total_steps: u8,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmergencyStopPhase {
    PressEmergencyStop,
    UnlockByBackAndConfirm,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmergencyStopTestPayload {
    pub name: String,
    pub stage: String,
    pub phase: EmergencyStopPhase,
    pub mower_main_p: u8,
    pub elapsed_ms: u64,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollisionBarPromptPayload {
    pub name: String,
    pub stage: String,
    pub prompt_kind: SensorPromptKind,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WheelMotorTestPhase {
    LiftConfirm,
    TestingRight,
    TestingLeft,
}

#[derive(Debug, Clone, Serialize)]
pub struct WheelMotorTestUpdatePayload {
    pub name: String,
    pub stage: String,
    pub phase: WheelMotorTestPhase,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorPromptKind {
    CollisionBar,
    LiftSensor,
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

impl fmt::Display for ParamId068Result {
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

impl fmt::Display for ParamId654Result {
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

impl fmt::Display for ParamId272Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BattPackPN={}, BattPackRev={}, BattPackProdDate={}, BattSwVer={}, BattSerNo={}, BattDevGrNo={}, BattSubDevNo={}, BattVarNo={}, BmsDevGrNo={}, BmsSubDevNo={}, BmsVarNo={}, BmsPcbaPN={}, BmsPcbaRev={}, BmsTempSensorType={}",
            self.batt_pack_pn,
            self.batt_pack_rev,
            self.batt_pack_prod_date,
            self.batt_sw_ver,
            self.batt_ser_no,
            self.batt_dev_gr_no,
            self.batt_sub_dev_no,
            self.batt_var_no,
            self.bms_dev_gr_no,
            self.bms_sub_dev_no,
            self.bms_var_no,
            self.bms_pcba_pn,
            self.bms_pcba_rev,
            self.bms_temp_sensor_type
        )
    }
}

impl fmt::Display for ParamId526Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PcbDeGrNo={}, PcbSubDeNo={}, PcbVarNo={}, PcbPN={}, PcbRev={}, PcbSerNo={}, PcbProdTime={}, PcbExtFlash={}, PcbExtEeprom={}, PcbAccelerometer={}",
            self.pcb_de_gr_no,
            self.pcb_sub_de_no,
            self.pcb_var_no,
            self.pcb_pn,
            self.pcb_rev,
            self.pcb_ser_no,
            self.pcb_prod_time,
            self.pcb_ext_flash,
            self.pcb_ext_eeprom,
            self.pcb_accelerometer
        )
    }
}

impl fmt::Display for ParamId096Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GprsLteStat={}, GprsLteSignQual={}, GnssHwStat={}, SimStatus={}, BleHwStat={}, GprsLteConnStat={}, BleConnStat={}, WifiConnStat={}, WifiHwStat={}, LoraConnStat={}, LoraHwStat={}, RtkHwStat={}, RtkConnStat={}, ConnectedRaSerial={}",
            self.gprs_lte_stat,
            self.gprs_lte_sign_qual,
            self.gnss_hw_stat,
            self.sim_status,
            self.ble_hw_stat,
            self.gprs_lte_conn_stat,
            self.ble_conn_stat,
            self.wifi_conn_stat,
            self.wifi_hw_stat,
            self.lora_conn_stat,
            self.lora_hw_stat,
            self.rtk_hw_stat,
            self.rtk_conn_stat,
            self.connected_ra_serial
        )
    }
}

impl fmt::Display for ParamId080Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MowerMainP={}, MowerSubState={}, TimeStpNxtStart={}, BattStat={}, StatFlags={}, WrlessConStat={}, SignQuality={}, SourceForNextStartStop={}, Notify={}, ConfigurationHash={}",
            self.mower_main_p,
            self.mower_sub_state,
            self.time_stp_nxt_start,
            self.batt_stat,
            self.stat_flags,
            self.wrless_con_stat,
            self.sign_quality,
            self.source_for_next_start_stop,
            self.notify,
            self.configuration_hash
        )
    }
}

impl fmt::Display for ParamId118Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CollisionSen={}, LiftSen={}, StatusFlags={}, StopSen={}, DisablingSen={}",
            self.collision_sen, self.lift_sen, self.status_flags, self.stop_sen, self.disabling_sen
        )
    }
}

impl fmt::Display for ParamId120Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PitchAngle={}, RollAngle={}, MoverVertGForce={}, AccelX={}, AccelY={}, AccelZ={}, GyroX={}, GyroY={}, GyroZ={}, UtcSec={}, UtcUsec={}",
            self.pitch_angle,
            self.roll_angle,
            self.mover_vert_g_force,
            self.accel_x,
            self.accel_y,
            self.accel_z,
            self.gyro_x,
            self.gyro_y,
            self.gyro_z,
            self.utc_sec,
            self.utc_usec
        )
    }
}

impl fmt::Display for ParamId122Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MoverTemp={}, CSTemp={}, BatTemp={}, CuttingMotorTemp={}, RightWmTemp={}, LeftWmTemp={}, AppBoardTemp={}, RadarTemp={}",
            self.mover_temp,
            self.cs_temp,
            self.bat_temp,
            self.cutting_motor_temp,
            self.right_wm_temp,
            self.left_wm_temp,
            self.app_board_temp,
            self.radar_temp
        )
    }
}

impl fmt::Display for ParamId470Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CuttingHeightMm={}", self.cutting_height_mm)
    }
}

impl fmt::Display for ParamId108Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BattVolMW={}, BattCurr={}, BattEnLvl={}, BattTemp={}, MainVoltage={}",
            self.batt_vol_mw, self.batt_curr, self.batt_en_lvl, self.batt_temp, self.main_voltage
        )
    }
}

impl fmt::Display for ParamId114Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RightWhlMotorP={}, RightWhlMotorCurr={}, RightWhlMotorSp={}, LeftWhlMotorP={}, LefWhlMotorCurr={}, LefWhlMotorSp={}",
            self.right_whl_motor_p,
            self.right_whl_motor_curr,
            self.right_whl_motor_sp,
            self.left_whl_motor_p,
            self.lef_whl_motor_curr,
            self.lef_whl_motor_sp
        )
    }
}

impl fmt::Display for ParamId794Result {
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

impl fmt::Display for ParamId796Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MqttStatus={}", self.mqtt_status)
    }
}

impl fmt::Display for ParamId798Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Version={}", self.version)
    }
}
