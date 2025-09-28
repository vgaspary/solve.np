use crate::state::State;
use actix_session::Session;
use actix_web::{
    HttpResponse, Responder, error::Error as HttpError, error::ErrorInternalServerError, web,
};
use log::error;
use rusqlite::params;
use serde::Deserialize;

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
    let solution_id = body.title.to_lowercase().replace(' ', "_");
    let title = &body.title;
    let description = &body.description;

    state
        .db
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO attempts (user_id, problem_id, solution_id, title, description, status) VALUES (?1, ?2, ?3, ?4, ?5, 0)", 
            params![user_id, problem_id, solution_id, title, description],
        )
        .inspect_err(|err| error!("SQL query failed: {err}"))
        .map_err(|_| ErrorInternalServerError("Pick a different title. Another attempts exists with the same title."))?;
    Ok(HttpResponse::Ok())
}
