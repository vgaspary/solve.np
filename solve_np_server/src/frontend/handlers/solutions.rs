use crate::frontend::handlers::header::HeaderTemplate;
use crate::state::State;
use crate::types::InputId;
use crate::types::ProblemId;
use crate::types::SolutionId;
use crate::types::Status;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::ContentType;
use actix_web::web;
use askama::Template;
use itertools::Itertools;
use log::error;

#[derive(Clone)]
pub struct Solution {
    id: SolutionId,
    problem_id: ProblemId,
    title: String,
    description: String,
    status: Status,
}

pub struct Problem {
    id: ProblemId,
    name: String,
    solutions: Vec<Solution>,
}

#[derive(Template)]
#[template(path = "solutions.html")]
struct SolutionsTemplate {
    header: HeaderTemplate,
    problems: Vec<Problem>,
    input_id: Option<InputId>,
}

#[derive(serde::Deserialize)]
pub struct SolutionsQueryParams {
    problem_id: Option<ProblemId>,
    input_id: Option<InputId>,
}

pub async fn solutions(
    session: Session,
    params: web::Query<SolutionsQueryParams>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let SolutionsQueryParams {
        problem_id,
        input_id,
    } = params.into_inner();
    let user_id = session
        .get::<String>("user_id")?
        .ok_or(ErrorInternalServerError("Internal error #4"))?;
    let solutions = state
        .db
        .lock()
        .unwrap()
        .prepare("SELECT solution_id, problem_id, title, description, status FROM solutions WHERE user_id = ?1")
        .inspect_err(|err| error!("{err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #7"))?
        .query_map(
            [user_id],
            |row| Ok(Solution {
                id: row.get(0)?,
                problem_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: row.get::<usize, i64>(4)?.into(),
            }))
        .map_err(|_| ErrorInternalServerError("Internal error #8"))?
        .filter_map(Result::ok)
        .collect_vec();
    let problems = state
        .static_data
        .iter()
        .filter(|&(id, _)| match &problem_id {
            Some(problem_id) => problem_id == id,
            None => true,
        })
        .map(|(id, problem)| Problem {
            id: id.clone(),
            name: problem.name.clone(),
            solutions: solutions
                .iter()
                .filter(|solution| &solution.problem_id == id)
                .cloned()
                .collect(),
        })
        .sorted_by_key(|problem| problem.id.clone())
        .collect::<Vec<_>>();
    let body = SolutionsTemplate {
        header: HeaderTemplate::try_from(&session)?,
        problems,
        input_id,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
