use crate::auth::SessionId;
use crate::frontend::handlers::head::HeadTemplate;
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
use log::error;
use rusqlite::params;

struct Problem {
    id: ProblemId,
    name: String,
}

struct Input {
    id: InputId,
    filename: String,
    preview: String,
    solution: String,
    status: Status,
    invalid_message: String,
}

struct Solution {
    id: SolutionId,
    problem_id: ProblemId,
    title: String,
    description: String,
    status: Status,
}

#[derive(Template)]
#[template(path = "solution.html")]
struct SolutionTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
    solution: Solution,
    problem: Problem,
    inputs: Vec<Input>,
}

pub async fn solution(
    session: Session,
    path: web::Path<SolutionId>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let user_id = session
        .get::<SessionId>("session_id")?
        .and_then(|session_id| state.session_registry.get_user_id(session_id))
        .ok_or(ErrorInternalServerError("Internal error #9"))?;
    let solution_id = path.into_inner();
    let db = state.db.lock().unwrap();

    let solution = db
        .query_one(
            "SELECT problem_id, title, description, status FROM solutions WHERE user_id = ?1 AND solution_id = ?2",
            params![&user_id, &solution_id],
            |row| Ok(Solution {
                id: solution_id.clone(),
                problem_id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
            })
        )
        .inspect_err(|err| error!("SQL query failed: {err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #10"))?;
    let (problem, inputs) = state
        .static_data
        .problems
        .iter()
        .find(|problem| problem.id == solution.problem_id)
        .map(|problem| {
            (
                Problem {
                    id: problem.id.clone(),
                    name: problem.name.clone(),
                },
                problem
                    .inputs
                    .iter()
                    .map(|input| {
                        let (solution, status, invalid_message) = db
                            .query_one(
                                "SELECT solution, status, invalid_message FROM submissions
                                 WHERE
                                    user_id = ?1 AND
                                    problem_id = ?2 AND
                                    solution_id = ?3 AND
                                    input_id = ?4;",
                                params![user_id, solution.problem_id, solution.id, input.id],
                                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                            )
                            .unwrap_or((None, Status::Untested, None));
                        Input {
                            id: input.id.clone(),
                            filename: input.filename.clone(),
                            preview: input.preview.clone(),
                            solution: solution.unwrap_or_default(),
                            status,
                            invalid_message: invalid_message.unwrap_or_default(),
                        }
                    })
                    .collect(),
            )
        })
        .ok_or(ErrorInternalServerError("Internal error #11"))?;

    let body = SolutionTemplate {
        head: HeadTemplate {
            title: "My Solutions - solve.NP".into(),
        },
        header: HeaderTemplate::try_from(&session, state.clone())?,
        solution,
        problem,
        inputs,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
