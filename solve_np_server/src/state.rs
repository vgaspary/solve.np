use rusqlite::Connection;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

type ProblemId = String;
type InputId = String;
pub type StaticData = HashMap<ProblemId, Problem>;

pub struct State {
    pub start_time: SystemTime,
    pub static_data: StaticData,
    pub db: Mutex<Connection>,
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
    #[serde(alias = "filename")]
    pub filename: String,
    #[serde(alias = "cmd")]
    _cmd: String,
    #[serde(alias = "preview")]
    pub preview: String,
}
