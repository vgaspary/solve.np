use crate::{frontend::handlers::header::HeaderTemplate, state::State};
use actix_session::Session;
use actix_web::{
    error::{Error as HttpError, ErrorInternalServerError},
    http::header::ContentType,
    web,
    HttpResponse,
    Responder,
};
use askama::Template;
use itertools::Itertools;

#[derive(Template)]
#[template(path = "problems.html")]
struct ProblemsTemplate {
    header: HeaderTemplate,
    problems: Vec<crate::api::types::Problem>,
}

pub async fn problems(
    session: Session,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let problems = state
        .static_data
        .iter()
        .map(|(id, problem)| crate::api::types::Problem {
            id: id.clone(),
            name: problem.name.clone(),
            description: problem.description.clone(),
        })
        .sorted_by_key(|problem| problem.id.clone())
        .collect::<Vec<_>>();
    let body = ProblemsTemplate {
        header: HeaderTemplate::try_from(&session)?,
        problems,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
