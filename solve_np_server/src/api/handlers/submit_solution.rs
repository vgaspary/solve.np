use crate::auth::SessionId;
use crate::state::State;
use crate::types::InputId;
use crate::types::ProblemId;
use crate::types::SolutionId;
use crate::types::Status;
use crate::validate::Solution;
use crate::validate::validate;
use crate::validate::hash;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::rt::task;
use actix_web::web;
use log::error;
use rusqlite::params;

pub async fn submit_solution(
    session: Session,
    solution: web::Json<Solution>,
    path: web::Path<(ProblemId, SolutionId, InputId)>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let session_id = session
        .get::<SessionId>("session_id")?
        .ok_or(ErrorInternalServerError("Internal error #38"))?;
    let user_id = state
        .session_registry
        .get_user_id(session_id)
        .ok_or(ErrorInternalServerError("Internal error #14"))?;
    let user_name = state
        .session_registry
        .get_user_name(session_id)
        .ok_or(ErrorInternalServerError("Internal error #33"))?;
    let solution = solution.into_inner();
    let (problem_id, solution_id, input_id) = path.into_inner();

    let hash_digest = hash(&solution) as i64;
    let same_hash_digest: bool = state
        .db
        .lock()
        .unwrap()
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM submissions
                WHERE
                    user_id = ?1 AND
                    problem_id = ?2 AND
                    solution_id = ?3 AND
                    input_id = ?4 AND
                    hash_digest = ?5);",
            params![user_id, problem_id, solution_id, input_id, hash_digest],
            |row| row.get(0),
        )
        .inspect_err(|err| error!("{err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #15"))?;
    if same_hash_digest {
        return Ok(HttpResponse::NoContent().finish());
    }

    state
        .db
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO submissions (user_id, user_name, problem_id, solution_id, input_id, solution, hash_digest, status, invalid_message)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(user_id, problem_id, solution_id, input_id)
             DO UPDATE SET
                solution = ?6,
                hash_digest = ?7,
                status = ?8,
                invalid_message = ?9;",
            params![user_id, user_name, problem_id, solution_id, input_id, solution.to_string(), hash_digest, Status::Untested, ""],
        )
        .inspect_err(|err| error!("SQL query failed: {err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #16"))?;

    let validation_result = task::spawn_blocking(move || {
        validate(
            (user_id, problem_id, solution_id, input_id),
            state.into_inner(),
        )
    })
    .await
    .map_err(|_| ErrorInternalServerError("Internal error #17"))?
    .inspect_err(|err| error!("{err}"))
    .map_err(|_| ErrorInternalServerError("Internal error #18"))?;
    Ok(HttpResponse::Ok().json(validation_result))
}
