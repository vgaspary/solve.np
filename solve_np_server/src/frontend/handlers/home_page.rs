use crate::auth::SessionId;
use crate::frontend_handlers::head::HeadTemplate;
use crate::frontend_handlers::header::HeaderTemplate;
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
#[template(path = "index.html")]
struct HomePageTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
    user_name: String,
}

pub async fn home_page(
    state: web::Data<State>,
    session: Session,
) -> Result<impl Responder, HttpError> {
    let user_name = session
        .get::<SessionId>("session_id")?
        .and_then(|session_id| state.session_registry.get_user_name(session_id))
        .unwrap_or_default();
    let body = HomePageTemplate {
        head: HeadTemplate {
            title: "solve.NP".into(),
        },
        header: HeaderTemplate::try_from(&session, state)?,
        user_name,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
