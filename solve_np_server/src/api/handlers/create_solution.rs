use crate::state::State;
use crate::types::Status;
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
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateSolutionBody {
    problem_id: String,
    title: String,
    description: String,
}

pub async fn create_solution(
    session: Session,
    body: web::Json<CreateSolutionBody>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let user_id = session
        .get::<String>("user_id")?
        .ok_or(ErrorInternalServerError("Internal error #5"))?;
    let problem_id = &body.problem_id;
    let solution_id = Uuid::new_v4().as_simple().to_string();
    let title = &body.title;
    let description = &body.description;

    state
        .db
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO solutions (solution_id, user_id, problem_id, title, description, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", 
            params![solution_id, user_id, problem_id, title, description, Status::Untested],
        )
        .inspect_err(|err| error!("SQL query failed: {err}"))
        .map_err(|_| ErrorConflict("Pick a different title. Another attempt exists with the same title."))?;
    Ok(HttpResponse::Ok())
}
