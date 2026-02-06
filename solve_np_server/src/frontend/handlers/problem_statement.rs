use crate::frontend::handlers::head::HeadTemplate;
use crate::frontend::utils::bytes_to_string_h;
use crate::frontend::utils::get_file_metadata;
use crate::frontend::utils::md_to_html;
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
use log::debug;
use log::error;
use rusqlite::params;
use std::fs;

#[derive(Template)]
#[template(path = "problem_statement.html")]
struct ProblemStatementTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
    problem: Problem,
    solution_table: SolutionTable,
}

struct Problem {
    id: ProblemId,
    name: String,
    problem_statement: String,
    sample_input: String,
    inputs: Vec<Input>,
}

struct Input {
    id: InputId,
    filename: String,
    size_h: String,
}

pub async fn problem_statement(
    session: Session,
    path: web::Path<ProblemId>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let problem_id: ProblemId = path.into_inner();
    let problem = state
        .static_data
        .problems
        .iter()
        .find(|problem| problem.id == problem_id)
        .ok_or_else(|| ErrorInternalServerError("Error code: 42"))
        .and_then(|problem| {
            let problem_statement = fs::read_to_string(&problem.problem_statement)?;
            Ok(Problem {
                id: problem_id,
                name: problem.name.clone(),
                problem_statement: md_to_html(problem_statement),
                sample_input: problem.sample_input.clone(),
                inputs: problem
                    .inputs
                    .iter()
                    .map(|input| {
                        let (filename, size) = get_file_metadata(&input.path)
                            .inspect_err(|err| {
                                debug!(
                                    "Failed acquiring metadata from path {:?}: {err}",
                                    input.path
                                )
                            })
                            .map_err(|_| ErrorInternalServerError("Error code: 3"))?;
                        Ok::<Input, HttpError>(Input {
                            id: input.id.clone(),
                            filename,
                            size_h: bytes_to_string_h(size),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })?;

    let solutions = if let Some(user_id) = session.get::<String>("user_id")? {
        state.db
            .lock()
            .unwrap()
            .prepare("SELECT solution_id, title, description, status FROM solutions WHERE user_id = ?1 AND problem_id = ?2")
            .inspect_err(|err| error!("{err}"))
            .map_err(|_| ErrorInternalServerError("Error code: 20"))?
            .query_map(params![user_id, problem_id],
                |row| Ok(
                    Solution {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: row.get(2)?,
                        status: row.get(3)?,
                    }))
            .map_err(|_| ErrorInternalServerError("Error code: 21"))?
            .filter_map(Result::ok)
            .collect_vec()
    } else {
        vec![]
    };
    let body = ProblemStatementTemplate {
        head: HeadTemplate::with_title(&format!("{} - solveNP", problem.name)),
        header: HeaderTemplate::try_from(&session, state)?,
        problem,
        solution_table: SolutionTable {
            solutions,
            problem_id,
            input_id: None,
        },
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
