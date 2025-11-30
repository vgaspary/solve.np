use crate::auth::SessionId;
use crate::state::State;
use crate::types::InputId;
use crate::types::SolutionId;
use crate::types::Status;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::web;
use log::error;
use rusqlite::params;

pub async fn submission_status(
    session: Session,
    path: web::Path<(SolutionId, InputId)>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let (solution_id, input_id) = path.into_inner();
    let user_id = session
        .get::<SessionId>("session_id")?
        .and_then(|session_id| state.session_registry.get_user_id(session_id))
        .ok_or(ErrorInternalServerError("Internal error #26"))?;
    let status: Status = state
        .db
        .lock()
        .unwrap()
        .query_one(
            "SELECT status FROM submissions
             WHERE
                solution_id = ?1 AND
                input_id = ?2 AND
                user_id = ?3",
            params![solution_id, input_id, user_id],
            |row| row.get(0),
        )
        .inspect_err(|err| error!("SQL query failed: {err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #27"))?;
    Ok(HttpResponse::Ok().json(status))
}
