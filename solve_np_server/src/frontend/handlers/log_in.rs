use crate::frontend::handlers::head::HeadTemplate;
use crate::frontend::handlers::header::HeaderTemplate;
use crate::frontend::handlers::header::Page;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::ContentType;
use actix_web::web;
use askama::Template;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "log_in.html")]
struct LogInTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
    redirect: Option<String>,
    error_msg: String,
}

#[derive(Deserialize)]
pub struct LogInQueryParams {
    redirect: Option<String>,
    err: Option<String>,
}

pub async fn log_in(
    query: web::Query<LogInQueryParams>,
    session: Session,
) -> Result<impl Responder, HttpError> {
    let body = LogInTemplate {
        head: HeadTemplate { title: "Log In - solve.NP".into() },
        header: HeaderTemplate::try_from(&session)?.with_active(Page::LogIn),
        redirect: query.redirect.clone(),
        error_msg: query.err.clone().unwrap_or_default(),
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
