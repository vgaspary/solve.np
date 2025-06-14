use crate::api::types::{HealthCheckReport, Input, Problem, ProblemDetails};
use crate::state::State;
use actix_web::{
    HttpResponse, Responder, error::Error as HttpError, error::ErrorInternalServerError, web,
};
use std::time::SystemTime;

pub async fn health_check(state: web::Data<State>) -> Result<impl Responder, HttpError> {
    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .map_err(ErrorInternalServerError)?;
    let response = HealthCheckReport {
        uptime: format!("{}s", uptime.as_secs()),
    };
    Ok(HttpResponse::Ok().json(response))
}

pub async fn list_problems(state: web::Data<State>) -> impl Responder {
    let result = state
        .static_data
        .iter()
        .map(|(id, problem)| Problem {
            id: id.clone(),
            name: problem.name.clone(),
            description: problem.description.clone(),
        })
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(result)
}

pub async fn get_problem_details(
    path: web::Path<String>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let problem_id = path.into_inner();
    state
        .static_data
        .get(&problem_id)
        .map(|problem| ProblemDetails {
            id: problem_id,
            name: problem.name.clone(),
            problem_statement: problem.problem_statement.clone(),
            inputs: problem
                .inputs
                .iter()
                .map(|(id, input)| Input {
                    id: id.clone(),
                    path: input.path.clone(),
                })
                .collect(),
        })
        .map(|problem_details| HttpResponse::Ok().json(problem_details))
        .ok_or_else(|| ErrorInternalServerError("No problem found with ID"))
}
