use serde::{Serialize, Deserialize};

pub type ProblemId = String;
pub type Score = usize;

#[derive(Serialize)]
pub struct Problem {
    pub id: ProblemId,
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct HealthCheckReport {
    pub uptime: String,
}

#[derive(Deserialize)]
pub struct LogInForm {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub enum SampleSolution {
    TspSolution(Vec<usize>),
    GraphColoringSolution(Vec<usize>),
}

#[derive(Serialize)]
pub enum ValidateSampleSolutionResult {
    Invalid(String),
    Valid(Score),
}
