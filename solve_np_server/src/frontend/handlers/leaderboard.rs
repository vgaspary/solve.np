use crate::frontend::handlers::header::HeaderTemplate;
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
#[template(path = "leaderboard.html")]
struct LeaderboardTemplate {
    header: HeaderTemplate,
}

pub async fn leaderboard(
    session: Session,
    _state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let body = LeaderboardTemplate {
        header: HeaderTemplate::try_from(&session)?,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
