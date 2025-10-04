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
use itertools::Itertools;
use log::error;

type Score = usize;

#[derive(Clone, Copy)]
pub enum Status {
    Untested,
    Attempted,
    Solved(Score),
}

impl TryFrom<i64> for Status {
    type Error = ();
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Status::Untested),
            1 => Ok(Status::Attempted),
            2 => Ok(Status::Solved(0)),
            _ => Err(()),
        }
    }
}

#[derive(Clone)]
pub struct Solution {
    id: String,
    problem_id: String,
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
    let user_id = session
        .get::<String>("user_id")?
        .ok_or(ErrorInternalServerError("Internal error #4"))?;
    let solutions = state
        .db
        .lock()
        .unwrap()
        .prepare("SELECT solution_id, problem_id, title, description, status FROM attempts WHERE user_id = ?1")
        .inspect_err(|err| error!("{err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #7"))?
        .query_map(
            [user_id],
            |row| Ok(Solution {
                id: row.get(0)?,
                problem_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: Status::try_from(row.get::<usize, i64>(4)?).expect("Status value must be 0 || 1 || 2."),
            }))
        .map_err(|_| ErrorInternalServerError("Internal error #8"))?
        .filter_map(Result::ok)
        .collect_vec();
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
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
