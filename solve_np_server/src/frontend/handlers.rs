use crate::state::State;
use actix_web::{
    HttpResponse, Responder,
    error::{Error as HttpError, ErrorInternalServerError},
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

#[derive(Template)]
#[template(path = "problem_statement.html")]
struct ProblemStatementTemplate {
    logged_in: bool,
    address: String,
    problem_id: String,
}

pub async fn problem_statement(
    path: web::Path<String>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let body = ProblemStatementTemplate {
        logged_in: false,
        address: state.address.to_string(),
        problem_id: path.into_inner(),
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
