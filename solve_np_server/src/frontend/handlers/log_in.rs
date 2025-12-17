use crate::frontend::handlers::head::HeadTemplate;
use crate::frontend::handlers::header::HeaderTemplate;
use crate::frontend::handlers::header::Page;
use crate::state::State;
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
    test_auth: bool,
}

#[derive(Deserialize)]
pub struct LogInQueryParams {
    redirect: Option<String>,
    err: Option<String>,
}

pub async fn log_in(
    state: web::Data<State>,
    query: web::Query<LogInQueryParams>,
    session: Session,
) -> Result<impl Responder, HttpError> {
    let body = LogInTemplate {
        head: HeadTemplate::with_title("Log In - solveNP"),
        header: HeaderTemplate::try_from(&session, state.clone())?.with_active(Page::LogIn),
        redirect: query.redirect.clone(),
        error_msg: query.err.clone().unwrap_or_default(),
        test_auth: state.test_auth(),
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
