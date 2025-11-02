use crate::types::InputId;
use crate::types::ProblemId;
use rusqlite::Connection;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

pub struct State {
    pub start_time: SystemTime,
    pub static_data: StaticData,
    pub db: Mutex<Connection>,
}

#[derive(Deserialize)]
pub struct StaticData {
    pub problems: Vec<Problem>,
}

#[derive(Deserialize)]
pub struct Problem {
    pub id: ProblemId,
    pub name: String,
    pub description: String,
    pub problem_statement: PathBuf,
    pub sample_input: String,
    pub inputs: Vec<Input>,
}

#[derive(Deserialize)]
pub struct Input {
    pub id: InputId,
    pub path: PathBuf,
    pub filename: String,
    #[serde(alias = "cmd")]
    _cmd: String,
    pub preview: String,
}
