use crate::state::State;
use crate::types::SolutionId;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::web;
use log::error;
use rusqlite::params;

pub async fn delete_solution(
    session: Session,
    path: web::Path<SolutionId>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let user_id = session
        .get::<String>("user_id")?
        .ok_or(ErrorInternalServerError("Internal error #6"))?;
    let solution_id = path.into_inner();

    state
        .db
        .lock()
        .unwrap()
        .execute(
            "DELETE FROM solutions WHERE solution_id = ?1 AND user_id = ?2",
            params![solution_id, user_id],
        )
        .inspect_err(|err| error!("SQL query failed: {err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #7"))?;
    Ok(HttpResponse::NoContent().finish())
}
