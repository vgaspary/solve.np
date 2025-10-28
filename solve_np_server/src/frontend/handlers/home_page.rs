use crate::frontend_handlers::head::HeadTemplate;
use crate::frontend_handlers::header::HeaderTemplate;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::ContentType;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct HomePageTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
    user_name: String,
}

pub async fn home_page(session: Session) -> Result<impl Responder, HttpError> {
    let user_name = session
        .get::<String>("username")?
        .unwrap_or(String::from("Unknown user"));
    let body = HomePageTemplate {
        head: HeadTemplate {
            title: "solve.NP".into(),
        },
        header: HeaderTemplate::try_from(&session)?,
        user_name,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
