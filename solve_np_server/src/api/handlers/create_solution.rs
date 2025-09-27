use crate::state::State;
use actix_session::Session;
use actix_web::{
    HttpResponse, Responder, error::Error as HttpError, error::ErrorInternalServerError, web,
};
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
    let solution_id = format!(
        "{}.{}",
        body.problem_id,
        body.title.to_lowercase().replace(' ', "_")
    );

    // TODO: Check if solution with the same title already exists

    // TODO: Save the new solution to the database

    if solution_id == "whatever" {
        Ok(HttpResponse::Ok())
    } else {
        Err(ErrorInternalServerError(format!(
            "There is already an attempt with this name: {solution_id}"
        )))
    }
}
