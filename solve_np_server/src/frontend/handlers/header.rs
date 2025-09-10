use actix_session::Session;
use actix_web::error::Error as HttpError;
use askama::Template;

#[derive(Template)]
#[template(path = "header.html")]
pub struct HeaderTemplate {
    pub logged_in: bool,
    pub username: String,
}

impl TryFrom<&Session> for HeaderTemplate {
    type Error = HttpError;

    fn try_from(session: &Session) -> Result<Self, Self::Error> {
        Ok(Self {
            logged_in: session.get::<String>("username")?.is_some(),
            username: session.get("username")?.unwrap_or_default(),
        })
    }
}
