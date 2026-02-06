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
    ParamId606 { front_light_mode: u8, power: u8 },
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
