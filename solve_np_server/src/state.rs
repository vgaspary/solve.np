use serde::Deserialize;
use std::{collections::HashMap, net::Ipv4Addr, path::PathBuf, time::SystemTime};

type ProblemId = String;
type InputId = String;
pub type StaticData = HashMap<ProblemId, Problem>;

pub struct State {
    pub start_time: SystemTime,
    pub address: Ipv4Addr,
    pub static_data: StaticData,
}

#[derive(Deserialize)]
pub struct Problem {
    pub name: String,
    pub description: String,
    pub problem_statement: PathBuf,
    pub sample_input: String,
    pub inputs: HashMap<InputId, Input>,
}

#[derive(Deserialize)]
pub struct Input {
    #[serde(alias = "path")]
    pub path: PathBuf,
    #[serde(alias = "cmd")]
    _cmd: String,
}
