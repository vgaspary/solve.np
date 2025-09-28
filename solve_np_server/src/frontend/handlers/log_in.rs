use crate::frontend::handlers::header::HeaderTemplate;
use actix_session::Session;
use actix_web::{
    HttpResponse, Responder,
    error::{Error as HttpError, ErrorInternalServerError},
    http::header::ContentType,
    web,
};
use askama::Template;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "log_in.html")]
struct LogInTemplate {
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
    params: web::Query<LogInQueryParams>,
    session: Session,
) -> Result<impl Responder, HttpError> {
    let body = LogInTemplate {
        header: HeaderTemplate::try_from(&session)?,
        redirect: params.redirect.clone(),
        error_msg: params.err.clone().unwrap_or_default(),
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
