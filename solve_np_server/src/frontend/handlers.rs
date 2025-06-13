use crate::state::State;
use actix_web::{
    HttpResponse, Responder,
    error::{Error as HttpError, ErrorInternalServerError},
    get,
    http::header::ContentType,
    web,
};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    user_name: String,
    logged_in: bool,
}

#[get("/")]
pub async fn home_page() -> Result<impl Responder, HttpError> {
    let body = IndexTemplate {
        user_name: String::from("Vanand Gasparyan"),
        logged_in: false,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}

#[derive(Template)]
#[template(path = "problems.html")]
struct ProblemsTemplate {
    logged_in: bool,
    address: String,
}

#[get("/problems")]
pub async fn problems(state: web::Data<State>) -> Result<impl Responder, HttpError> {
    let body = ProblemsTemplate {
        logged_in: false,
        address: state.address.to_string(),
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
