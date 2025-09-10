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
#[template(path = "solutions.html")]
struct SolutionsTemplate {
    header: HeaderTemplate,
    problems: Vec<ProblemWithSolutions>,
}

struct ProblemWithSolutions {
    id: String,
    name: String,
    solutions: Vec<Solution>,
}

struct Solution {
    title: String,
    description: String,
    status: i32,
}

pub async fn solutions(
    session: Session,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let problems = state
        .static_data
        .iter()
        .map(|(id, problem)| ProblemWithSolutions {
            id: id.clone(),
            name: problem.name.clone(),
            solutions: vec![],
        })
        .sorted_by_key(|problem| problem.id.clone())
        .collect::<Vec<_>>();
    let body = SolutionsTemplate {
        header: HeaderTemplate::try_from(&session)?,
        problems,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
