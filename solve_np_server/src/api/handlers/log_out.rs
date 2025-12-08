use crate::auth::SessionId;
use crate::state::State;
use actix_session::Session;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::web;

pub async fn log_out(
    state: web::Data<State>,
    session: Session,
) -> Result<impl Responder, HttpError> {
    if let Some(session_id) = session.get::<SessionId>("session_id")? {
        state.session_registry.remove_session(session_id);
    }
    session.clear();
    Ok(web::Redirect::to("/").see_other())
}
