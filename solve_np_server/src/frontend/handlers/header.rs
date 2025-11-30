use crate::auth::SessionId;
use crate::state::State;
use actix_session::Session;
use actix_web::error::Error as HttpError;
use actix_web::web;
use askama::Template;

#[derive(Eq, PartialEq)]
pub enum Page {
    Problems,
    Leaderboard,
    PvsNP,
    Solutions,
    LogIn,
}

#[derive(Template)]
#[template(path = "header.html")]
pub struct HeaderTemplate {
    pub logged_in: bool,
    pub active: Option<Page>,
}

impl HeaderTemplate {
    pub fn try_from(session: &Session, state: web::Data<State>) -> Result<Self, HttpError> {
        Ok(Self {
            logged_in: session
                .get::<SessionId>("session_id")?
                .and_then(|session_id| state.session_registry.get_user_id(session_id))
                .is_some(),
            active: None,
        })
    }

    pub fn with_active(self, page: Page) -> Self {
        Self {
            active: Some(page),
            ..self
        }
    }
}
