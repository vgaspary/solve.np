use crate::auth::SessionId;
use crate::state::State;
use crate::types::SolutionId;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorConflict;
use actix_web::error::ErrorInternalServerError;
use actix_web::web;
use log::error;
use rusqlite::params;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateSolutionBody {
    title: String,
    description: String,
}

pub async fn update_solution(
    session: Session,
    path: web::Path<SolutionId>,
    body: web::Json<UpdateSolutionBody>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let user_id = session
        .get::<SessionId>("session_id")?
        .and_then(|session_id| state.session_registry.get_user_id(session_id))
        .ok_or(ErrorInternalServerError("Internal error #6"))?;
    let solution_id = path.into_inner();
    let title = &body.title;
    let description = &body.description;

    state
        .db
        .lock()
        .unwrap()
        .execute(
            "UPDATE solutions SET title = ?1, description = ?2 WHERE solution_id = ?3 AND user_id = ?4",
            params![title, description, solution_id, user_id],
        )
        .inspect_err(|err| error!("SQL query failed: {err}"))
        .map_err(|_| {
            ErrorConflict("Pick a different title. Another attempt exists with the same title.")
        })?;
    Ok(HttpResponse::Ok())
}
