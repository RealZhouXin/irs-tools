use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestConfig {
    pub connection: ConnectionConfig,
    pub read_timeout_ms: u32,
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
    #[serde(flatten)]
    pub command: CommandGroupSpec,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum CommandGroupSpec {
    ParamId068 { checks: Vec<ParamId068Check> },
    ParamId588 { checks: Vec<ParamId588Check> },
    ParamId654 { checks: Vec<ParamId654Check> },
    ParamId272 { checks: Vec<ParamId272Check> },
    ParamId080 { checks: Vec<ParamId080Check> },
    ParamId120 { checks: Vec<ParamId120Check> },
    ParamId122 { checks: Vec<ParamId122Check> },
    ParamId470 { checks: Vec<ParamId470Check> },
    ParamId468 { cutting_height_mm: u8 },
    ParamId606 { front_light_mode: u8, power: u8 },
}

pub trait CheckConfig {
    type OutputEnum: Copy;
    fn name(&self) -> &str;
    fn output(&self) -> Self::OutputEnum;
    fn min(&self) -> f64;
    fn max(&self) -> f64;
}

impl CheckConfig for ParamId068Check {
    type OutputEnum = ParamId068Output;
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

impl CheckConfig for ParamId588Check {
    type OutputEnum = ParamId588Output;
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

impl CheckConfig for ParamId654Check {
    type OutputEnum = ParamId654Output;
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamId068Check {
    pub name: String,
    pub output: ParamId068Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId068Output {
    DevGrNo,
    SubDevGrNo,
    VarNo,
    MajParSwVer,
    MinParSwVer,
    BuildNo,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamId588Check {
    pub name: String,
    pub output: ParamId588Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId588Output {
    DevGrNo,
    SubDevGrNo,
    VarNo,
    MajParSwVer,
    MinParSwVer,
    BuildNo,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamId654Check {
    pub name: String,
    pub output: ParamId654Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId654Output {
    DevGrNo,
    SubDevGrNo,
    VarNo,
    MajParSwVer,
    MinParSwVer,
    BuildNo,
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

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub value: Option<f64>,
    pub passed: bool,
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub name: String,
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

impl CheckableResult for ParamId588Result {
    type OutputEnum = ParamId588Output;
    fn get_value(&self, output: Self::OutputEnum) -> f64 {
        match output {
            ParamId588Output::DevGrNo => self.dev_gr_no as f64,
            ParamId588Output::SubDevGrNo => self.sub_dev_gr_no as f64,
            ParamId588Output::VarNo => self.var_no as f64,
            ParamId588Output::MajParSwVer => self.maj_par_sw_ver as f64,
            ParamId588Output::MinParSwVer => self.min_par_sw_ver as f64,
            ParamId588Output::BuildNo => self.build_no as f64,
        }
    }
}

impl CheckableResult for ParamId068Result {
    type OutputEnum = ParamId068Output;
    fn get_value(&self, output: Self::OutputEnum) -> f64 {
        match output {
            ParamId068Output::DevGrNo => self.dev_gr_no as f64,
            ParamId068Output::SubDevGrNo => self.sub_dev_gr_no as f64,
            ParamId068Output::VarNo => self.var_no as f64,
            ParamId068Output::MajParSwVer => self.maj_par_sw_ver as f64,
            ParamId068Output::MinParSwVer => self.min_par_sw_ver as f64,
            ParamId068Output::BuildNo => self.build_no as f64,
        }
    }
}

impl CheckableResult for ParamId654Result {
    type OutputEnum = ParamId654Output;
    fn get_value(&self, output: Self::OutputEnum) -> f64 {
        match output {
            ParamId654Output::DevGrNo => self.dev_gr_no as f64,
            ParamId654Output::SubDevGrNo => self.sub_dev_gr_no as f64,
            ParamId654Output::VarNo => self.var_no as f64,
            ParamId654Output::MajParSwVer => self.maj_par_sw_ver as f64,
            ParamId654Output::MinParSwVer => self.min_par_sw_ver as f64,
            ParamId654Output::BuildNo => self.build_no as f64,
        }
    }
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
