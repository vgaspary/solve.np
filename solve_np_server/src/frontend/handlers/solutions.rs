use crate::{frontend::handlers::header::HeaderTemplate, state::State};
use actix_session::Session;
use actix_web::{
    HttpResponse, Responder,
    error::{Error as HttpError, ErrorInternalServerError},
    http::header::ContentType,
    web,
};
use askama::Template;
use itertools::Itertools;

type Score = usize;

pub enum Status {
    Untested,
    Attempted,
    Solved(Score),
}

pub struct Solution {
    id: String,
    title: String,
    description: String,
    status: Status,
}

pub struct Problem {
    id: String,
    name: String,
    solutions: Vec<Solution>,
}

#[derive(Template)]
#[template(path = "solutions.html")]
struct SolutionsTemplate {
    header: HeaderTemplate,
    problems: Vec<Problem>,
}

#[derive(serde::Deserialize)]
pub struct SolutionsQueryParams {
    problem_id: Option<String>,
}

pub async fn solutions(
    session: Session,
    params: web::Query<SolutionsQueryParams>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let problems = state
        .static_data
        .iter()
        .filter(|&(id, _)| match &params.problem_id {
            Some(problem_id) => problem_id == id,
            None => true,
        })
        .map(|(id, problem)| Problem {
            id: id.clone(),
            name: problem.name.clone(),
            solutions: vec![
                // TODO! Read solutions from DB
                Solution {
                    id: format!("{id}:bruteforce"),
                    title: "Bruteforce".to_owned(),
                    description: "A bruteforce solution simply checks all possible solutions and picks the one with the best score.".to_owned(),
                    status: Status::Attempted,
                },
                Solution {
                    id: format!("{id}:greedy"),
                    title: "Greedy".to_owned(),
                    description: "This greedy algorithm picks the shortest possible path forward that doesn't violate the path requirements. It's really fast but probably doesn't yield a good score.".to_owned(),
                    status: Status::Solved(43),
                },
                Solution {
                    id: format!("{id}:clever"),
                    title: "Clever solution".to_owned(),
                    description: "Actual real solution.".to_owned(),
                    status: Status::Untested,
                },
            ],
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
