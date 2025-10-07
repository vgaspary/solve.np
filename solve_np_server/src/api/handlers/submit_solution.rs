use crate::api::utils;
use crate::state::State;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::web;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

pub type Score = usize;

#[derive(Deserialize, Hash, Eq, PartialEq)]
pub enum Solution {
    TspSolution(Vec<usize>),
    GraphColoringSolution(Vec<usize>),
}

#[derive(Serialize)]
pub enum ValidateSolutionResult {
    Invalid(String),
    Valid(Score),
}

fn hash(solution: &Solution) -> u64 {
    let mut hasher = DefaultHasher::new();
    solution.hash(&mut hasher);
    hasher.finish()
}

pub async fn submit_solution(
    session: Session,
    solution: web::Json<Solution>,
    path: web::Path<(String, String, String)>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let _user_id = session
        .get::<String>("user_id")?
        .ok_or(ErrorInternalServerError("Internal error #14"))?;
    let solution = solution.into_inner();
    let (problem_id, _solution_id, input_id) = path.into_inner();

    let solution_hash = hash(&solution);
    // if there's a record in DB with this hash, there's nothing to be done
    if solution_hash == 0 {
        return Ok(HttpResponse::NoContent().finish());
    }

    match solution {
        Solution::TspSolution(path) => {
            let graph = state
                .static_data
                .get(&problem_id)
                .and_then(|problem| problem.inputs.get(&input_id))
                .and_then(|input| fs::read_to_string(&input.path).ok())
                .and_then(|input| utils::parse_tsp_input(&input))
                .ok_or(ErrorInternalServerError("Internal error #12"))?;
            match tsp_validate::validate(&graph, path) {
                Ok(score) => Ok(HttpResponse::Ok().json(ValidateSolutionResult::Valid(score))),
                Err(err) => {
                    Ok(HttpResponse::Ok().json(ValidateSolutionResult::Invalid(err.to_string())))
                }
            }
        }
        Solution::GraphColoringSolution(coloring) => {
            let graph = state
                .static_data
                .get(&problem_id)
                .and_then(|problem| problem.inputs.get(&input_id))
                .and_then(|input| fs::read_to_string(&input.path).ok())
                .and_then(|input| utils::parse_graph_coloring_input(&input))
                .ok_or(ErrorInternalServerError("Internal error #13"))?;
            match graph_coloring_validate::validate(&graph, coloring) {
                Ok(score) => Ok(HttpResponse::Ok().json(ValidateSolutionResult::Valid(score))),
                Err(err) => {
                    Ok(HttpResponse::Ok().json(ValidateSolutionResult::Invalid(err.to_string())))
                }
            }
        }
    }
}
