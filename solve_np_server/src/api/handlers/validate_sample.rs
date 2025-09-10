use crate::api::types::{SampleSolution, ValidateSampleSolutionResult};
use crate::api::utils;
use crate::state::State;
use actix_web::{
    error::Error as HttpError, error::ErrorInternalServerError, web, HttpResponse, Responder,
};

pub async fn validate_sample(
    solution: web::Json<SampleSolution>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    match solution.0 {
        SampleSolution::TspSolution(path) => {
            let graph = state
                .static_data
                .get(&String::from("travelling_salesman"))
                .map(|problem| problem.sample_input.as_ref())
                .and_then(utils::parse_tsp_input)
                .ok_or(ErrorInternalServerError(
                    "Server error: failed parsing TSP sample input.",
                ))?;
            match tsp_validate::validate(&graph, path) {
                Ok(score) => {
                    Ok(HttpResponse::Ok().json(ValidateSampleSolutionResult::Valid(score)))
                }
                Err(err) => {
                    Ok(HttpResponse::Ok()
                        .json(ValidateSampleSolutionResult::Invalid(err.to_string())))
                }
            }
        }
        SampleSolution::GraphColoringSolution(coloring) => {
            let graph = state
                .static_data
                .get(&String::from("graph_coloring"))
                .map(|problem| problem.sample_input.as_ref())
                .and_then(utils::parse_graph_coloring_input)
                .ok_or(ErrorInternalServerError(
                    "Server error: failed parsing Graph Coloring sample input.",
                ))?;
            match graph_coloring_validate::validate(&graph, coloring) {
                Ok(score) => {
                    Ok(HttpResponse::Ok().json(ValidateSampleSolutionResult::Valid(score)))
                }
                Err(err) => {
                    Ok(HttpResponse::Ok()
                        .json(ValidateSampleSolutionResult::Invalid(err.to_string())))
                }
            }
        }
    }
}
