use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct TestConfig {
    pub connection: ConnectionConfig,
    pub read_timeout_ms: u32,
    pub tests: Vec<TestGroup>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ConnectionConfig {
    Serial { port_number: u16 },
    Network { ip_address: String, port: String },
}

#[derive(Debug, Deserialize, Clone)]
pub struct TestGroup {
    pub name: String,
    #[serde(flatten)]
    pub command: CommandGroupSpec,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum CommandGroupSpec {
    ParamId068 { checks: Vec<ParamId068Check> },
    ParamId588 { checks: Vec<ParamId588Check> },
    ParamId606 {
        front_light_mode: u8,
        power: u8,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct ParamId068Check {
    pub name: String,
    pub output: ParamId068Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId068Output {
    DevGrNo,
    SubDevGrNo,
    VarNo,
    MajParSwVer,
    MinParSwVer,
    BuildNo,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ParamId588Check {
    pub name: String,
    pub output: ParamId588Output,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ParamId588Output {
    DevGrNo,
    SubDevGrNo,
    VarNo,
    MajParSwVer,
    MinParSwVer,
    BuildNo,
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
