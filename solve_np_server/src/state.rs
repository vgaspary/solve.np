use crate::auth::gauth::GoogleAuthClientId;
use crate::auth::gauth::GoogleAuthClientSecret;
use crate::auth::github_auth::GithubAuthClientId;
use crate::auth::github_auth::GithubAuthClientSecret;
use crate::auth::session_registry::SessionRegistry;
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
    pub address: String,
    pub db: Mutex<Connection>,
    pub session_registry: SessionRegistry,
    pub google_auth_client_id: Option<GoogleAuthClientId>,
    pub google_auth_client_secret: Option<GoogleAuthClientSecret>,
    pub github_auth_client_id: Option<GithubAuthClientId>,
    pub github_auth_client_secret: Option<GithubAuthClientSecret>,
}

impl State {
    pub fn test_auth(&self) -> bool {
        self.google_auth_client_id.is_none()
    }
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
