use crate::state::State;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::web;
use serde::{Deserialize, Serialize};

pub type Score = usize;

#[derive(Deserialize)]
pub enum SampleSolution {
    TspSolution(Vec<usize>),
    GraphColoringSolution(Vec<usize>),
}

#[derive(Serialize)]
pub enum ValidateSampleSolutionResult {
    Invalid(String),
    Valid(Score),
}

pub async fn validate_sample(
    solution: web::Json<SampleSolution>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    match solution.0 {
        SampleSolution::TspSolution(path) => {
            let graph = state
                .static_data
                .get(&"travelling_salesman".into())
                .map(|problem| problem.sample_input.as_ref())
                .and_then(tsp_validate::parse_input)
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
                .get(&"graph_coloring".into())
                .map(|problem| problem.sample_input.as_ref())
                .and_then(graph_coloring_validate::parse_input)
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
