use actix_session::Session;
use actix_web::error::Error as HttpError;
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

impl TryFrom<&Session> for HeaderTemplate {
    type Error = HttpError;

    fn try_from(session: &Session) -> Result<Self, Self::Error> {
        Ok(Self {
            logged_in: session.get::<String>("username")?.is_some(),
            active: None,
        })
    }
}

impl HeaderTemplate {
    pub fn with_active(self, page: Page) -> Self {
        Self {
            active: Some(page),
            ..self
        }
    }
}
