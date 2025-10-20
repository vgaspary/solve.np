use crate::frontend_handlers::header::HeaderTemplate;
use crate::frontend_handlers::solution_table::Solution;
use crate::frontend_handlers::solution_table::SolutionTable;
use crate::state::State;
use crate::types::InputId;
use crate::types::ProblemId;
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

pub struct Problem {
    id: ProblemId,
    name: String,
    solution_table: SolutionTable,
}

#[derive(Template)]
#[template(path = "solutions.html")]
struct SolutionsTemplate {
    header: HeaderTemplate,
    problems: Vec<Problem>,
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
            |row| Ok(
                (
                    row.get::<usize, ProblemId>(1)?,
                    Solution {
                        id: row.get(0)?,
                        title: row.get(2)?,
                        description: row.get(3)?,
                        status: row.get(4)?,
                    },
                )
                )
            )
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
            solution_table: SolutionTable {
                solutions: solutions
                    .iter()
                    .filter_map(|(problem_id, solution)| {
                        if problem_id == id {
                            Some(solution)
                        } else {
                            None
                        }
                    })
                    .cloned()
                    .collect(),
                problem_id: id.clone(),
                input_id: input_id.clone(),
            },
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
