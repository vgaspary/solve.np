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
#[template(path = "feedback.html")]
struct FeedbackTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
    status: Option<Result<(), String>>,
    title: String,
    content: String,
}

#[derive(Deserialize)]
pub struct FeedbackQuery {
    status: Option<String>,
    title: Option<String>,
    content: Option<String>,
}

pub async fn feedback(
    session: Session,
    state: web::Data<State>,
    query: web::Query<FeedbackQuery>,
) -> Result<impl Responder, HttpError> {
    let head = HeadTemplate::with_title("Feedback - solveNP");
    let header = HeaderTemplate::try_from(&session, state.clone())?.with_active(Page::Feedback);
    let status = query.status.as_ref().map(|status| match status.as_str() {
        "success" => Ok(()),
        err_msg => Err(err_msg.to_owned()),
    });
    let body = FeedbackTemplate {
        head,
        header,
        status,
        title: query.title.clone().unwrap_or("".to_string()),
        content: query.content.clone().unwrap_or("".to_string()),
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
