use serde::Serialize;
use std::path::PathBuf;

pub type ProblemId = String;
pub type InputId = String;

#[derive(Serialize)]
pub struct Problem {
    pub id: ProblemId,
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct Input {
    pub id: InputId,
    pub path: PathBuf,
}

#[derive(Serialize)]
pub struct ProblemDetails {
    pub id: ProblemId,
    pub name: String,
    pub problem_statement_html: String,
    pub inputs: Vec<Input>,
}

#[derive(Serialize)]
pub struct HealthCheckReport {
    pub uptime: String,
}
