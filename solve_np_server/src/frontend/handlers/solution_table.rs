use crate::auth::SessionId;
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
use rusqlite::params;

#[derive(Clone)]
pub struct Solution {
    pub id: SolutionId,
    pub title: String,
    pub description: String,
    pub status: Status,
}

#[derive(serde::Deserialize)]
pub struct SolutionTableParams {
    input_id: Option<InputId>,
}

#[derive(Template)]
#[template(path = "solution_table.html")]
pub struct SolutionTable {
    pub solutions: Vec<Solution>,
    pub problem_id: ProblemId,
    pub input_id: Option<InputId>,
}

pub async fn solution_table(
    session: Session,
    path: web::Path<ProblemId>,
    query: web::Query<SolutionTableParams>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let user_id = session
        .get::<SessionId>("session_id")?
        .and_then(|session_id| state.session_registry.get_user_id(session_id))
        .ok_or(ErrorInternalServerError(
            "Something went wrong. Please log in.",
        ))?;
    let problem_id = path.into_inner();

    let solutions = state
        .db
        .lock()
        .unwrap()
        .prepare(
            "SELECT solution_id, title, description, status
            FROM solutions
            WHERE
                user_id = ?1 AND
                problem_id = ?2;",
        )
        .inspect_err(|err| error!("{err}"))
        .map_err(|_| ErrorInternalServerError("Error code: 22"))?
        .query_map(params![user_id, problem_id], |row| {
            Ok(Solution {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
            })
        })
        .map_err(|_| ErrorInternalServerError("Error code: 23"))?
        .filter_map(Result::ok)
        .collect_vec();
    let body = SolutionTable {
        solutions,
        problem_id,
        input_id: query.into_inner().input_id,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
