use actix_web::{
    HttpResponse, Responder,
    error::{Error as HttpError, ErrorInternalServerError},
    get,
    http::header::ContentType,
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
