use crate::frontend::handlers::header::HeaderTemplate;
use actix_session::Session;
use actix_web::{
    error::{Error as HttpError, ErrorInternalServerError},
    http::header::ContentType,
    HttpResponse,
    Responder,
};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct HomePageTemplate {
    header: HeaderTemplate,
    user_name: String,
}

pub async fn home_page(session: Session) -> Result<impl Responder, HttpError> {
    let user_name = session
        .get::<String>("username")?
        .unwrap_or(String::from("Unknown user"));
    let body = HomePageTemplate {
        header: HeaderTemplate::try_from(&session)?,
        user_name,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}

