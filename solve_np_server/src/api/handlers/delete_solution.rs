use crate::state::State;
use actix_session::Session;
use actix_web::{Responder, error::Error as HttpError, web};
use log::debug;

pub async fn delete_solution(
    _session: Session,
    path: web::Path<String>,
    _state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let solution_id = path.into_inner();
    debug!("Deleting solution: {solution_id}");
    // TODO! remove solution with this ID from this user from the DB.
    Ok(web::Redirect::to("/solutions").see_other())
}
