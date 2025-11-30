use crate::auth::IdTokenClaims;
use crate::auth::SessionId;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
pub struct SessionRegistry {
    sessions: Mutex<HashMap<SessionId, IdTokenClaims>>,
}

impl SessionRegistry {
    pub fn get_user_id(&self, session_id: SessionId) -> Option<String> {
        self.sessions
            .lock()
            .unwrap()
            .get(&session_id)
            .map(|claims| claims.sub.clone())
    }

    pub fn get_user_name(&self, session_id: SessionId) -> Option<String> {
        self.sessions
            .lock()
            .unwrap()
            .get(&session_id)
            .map(|claims| claims.name.clone())
    }

    pub fn create_session(&self, claims: IdTokenClaims) -> SessionId {
        let session_id = SessionId(rand::random());
        self.sessions.lock().unwrap().insert(session_id, claims);
        session_id
    }
}
