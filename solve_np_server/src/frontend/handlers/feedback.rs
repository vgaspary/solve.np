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

#[derive(Template)]
#[template(path = "feedback.html")]
struct FeedbackTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
}

pub async fn feedback(
    session: Session,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let head = HeadTemplate {
        title: "Feedback - solveNP".into(),
    };
    let header = HeaderTemplate::try_from(&session, state.clone())?.with_active(Page::Feedback);
    let body = FeedbackTemplate { head, header }
        .render()
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
