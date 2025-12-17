use crate::auth::SessionId;
use crate::state::State;
use actix_session::Session;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorBadRequest;
use actix_web::error::ErrorInternalServerError;
use actix_web::web;
use log::debug;
use rusqlite::params;

pub async fn vote(
    session: Session,
    path: web::Path<String>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let Some(user_id) = session
        .get::<SessionId>("session_id")?
        .and_then(|session_id| state.session_registry.get_user_id(session_id))
    else {
        return Ok(web::Redirect::to("/log_in?redirect=/p_vs_np"));
    };
    let vote = match path.into_inner().as_str() {
        "eq" => true,
        "neq" => false,
        _ => return Err(ErrorBadRequest("Vote may only be 'eq' or 'neq'.")),
    };
    debug!("user_id {user_id} voted on P vs NP: {vote}");
    state
        .db
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO votes (user_id, vote) VALUES (?1, ?2)",
            params![user_id, vote],
        )
        .inspect_err(|err| debug!("SQL query failed: {err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #1"))?;
    Ok(web::Redirect::to("/p_vs_np").see_other())
}
