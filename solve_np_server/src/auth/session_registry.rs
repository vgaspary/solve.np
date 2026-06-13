use crate::auth::IdTokenClaims;
use crate::auth::SessionId;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;

/// 30 days.
const EXPIRY: Duration = Duration::from_secs(30 * 24 * 3600);

type Sessions = HashMap<SessionId, (IdTokenClaims, SystemTime)>;

#[derive(Default)]
pub struct SessionRegistry {
    sessions: Mutex<Sessions>,
}

impl SessionRegistry {
    pub fn load_from_file_or_default(path: &PathBuf) -> Self {
        let sessions = File::open(path)
            .and_then(|file| {
                serde_yaml::from_reader(BufReader::new(file)).map_err(std::io::Error::other)
            })
            .unwrap_or_default();
        Self { sessions }
    }

    pub fn get_user_id(&self, session_id: SessionId) -> Option<String> {
        self.sessions
            .lock()
            .unwrap()
            .get(&session_id)
            .map(|(claims, _)| claims.sub.clone())
    }

    pub fn get_user_name(&self, session_id: SessionId) -> Option<String> {
        self.sessions
            .lock()
            .unwrap()
            .get(&session_id)
            .map(|(claims, _)| claims.name.clone())
    }

    pub fn create_session(&self, claims: IdTokenClaims) -> SessionId {
        let session_id = SessionId(rand::random());
        self.sessions
            .lock()
            .unwrap()
            .insert(session_id, (claims, SystemTime::now() + EXPIRY));
        session_id
    }

    pub fn remove_session(&self, session_id: SessionId) {
        self.sessions.lock().unwrap().remove(&session_id);
    }

    pub fn valid_sessions(&self) -> Sessions {
        let now = SystemTime::now();
        self.sessions
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, (_, exp))| *exp > now)
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }
}

pub fn save_to_file(path: &PathBuf, sessions: &Sessions) -> std::io::Result<()> {
    serde_yaml::to_writer(BufWriter::new(File::create(path)?), sessions)
        .map_err(std::io::Error::other)
}
