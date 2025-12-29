use crate::auth::gauth::GoogleAuthClientId;
use crate::auth::gauth::GoogleAuthClientSecret;
use crate::auth::github_auth::GithubAuthClientId;
use crate::auth::github_auth::GithubAuthClientSecret;
use crate::auth::session_registry::SessionRegistry;
use crate::types::InputId;
use crate::types::ProblemId;
use itertools::Itertools;
use lettre::AsyncSmtpTransport;
use lettre::Tokio1Executor;
use rusqlite::Connection;
use serde::Deserialize;
use std::fs::File;
use std::fs::read_to_string;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

pub struct State {
    pub start_time: SystemTime,
    pub static_data: StaticData,
    pub address: String,
    pub db: Mutex<Connection>,
    pub session_registry: SessionRegistry,
    pub google_auth_client_id: Option<GoogleAuthClientId>,
    pub google_auth_client_secret: Option<GoogleAuthClientSecret>,
    pub github_auth_client_id: Option<GithubAuthClientId>,
    pub github_auth_client_secret: Option<GithubAuthClientSecret>,
    pub mail_client: Option<AsyncSmtpTransport<Tokio1Executor>>,
}

impl State {
    pub fn test_auth(&self) -> bool {
        self.google_auth_client_id.is_none()
    }
}

#[derive(Deserialize)]
pub struct LazyInput {
    pub id: InputId,
    pub path: PathBuf,
    pub num_vertices: usize,
}

pub struct Input {
    pub id: InputId,
    pub path: PathBuf,
    pub num_vertices: usize,
    pub filename: String,
    pub preview: String,
}

impl From<LazyInput> for Input {
    fn from(input: LazyInput) -> Self {
        let LazyInput {
            id,
            path,
            num_vertices,
        } = input;
        let filename = (&path)
            .file_name()
            .expect("Path must have a filename.")
            .to_owned()
            .into_string()
            .expect("Failed to convert OsString to String.");
        let preview = BufReader::new(File::open(&path).expect("Input file presence"))
            .lines()
            .filter_map(Result::ok)
            .take(20)
            .join("\n");
        Self {
            id,
            path,
            num_vertices,
            filename,
            preview,
        }
    }
}

#[derive(Deserialize)]
pub struct LazyProblem {
    pub id: ProblemId,
    pub name: String,
    pub description: String,
    pub problem_statement: PathBuf,
    pub inputs: Vec<LazyInput>,
}

pub struct Problem {
    pub id: ProblemId,
    pub name: String,
    pub description: String,
    pub problem_statement: PathBuf,
    pub sample_input: String,
    pub inputs: Vec<Input>,
}

impl From<LazyProblem> for Problem {
    fn from(problem: LazyProblem) -> Self {
        let LazyProblem {
            id,
            name,
            description,
            problem_statement,
            inputs,
        } = problem;
        let mut sample_input_path = problem_statement.clone();
        sample_input_path.pop();
        sample_input_path.push("inputs");
        sample_input_path.push("sample.txt");
        let sample_input = read_to_string(sample_input_path).expect("Failed to read sample input.");
        let inputs = inputs.into_iter().map(Input::from).collect();
        Problem {
            id,
            name,
            description,
            problem_statement,
            sample_input,
            inputs,
        }
    }
}

#[derive(Deserialize)]
pub struct LazyStaticData {
    pub problems: Vec<LazyProblem>,
}

pub struct StaticData {
    pub problems: Vec<Problem>,
}

impl From<LazyStaticData> for StaticData {
    fn from(static_data: LazyStaticData) -> Self {
        let problems = static_data
            .problems
            .into_iter()
            .map(Problem::from)
            .collect();
        Self { problems }
    }
}
