use crate::state::State;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::web;
use log::error;
use serde::Deserialize;
use serde::Serialize;

pub type Score = usize;

#[derive(Deserialize)]
pub enum SampleSolution {
    TspSolution(Vec<usize>),
    GraphColoringSolution(Vec<usize>),
    VertexCoverSolution(Vec<usize>),
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
            let sample_input = state
                .static_data
                .problems
                .iter()
                .find(|problem| problem.id.as_ref() == "travelling_salesman")
                .map(|problem| problem.sample_input.as_ref())
                .ok_or(ErrorInternalServerError("Error code: 43"))
                .inspect_err(|_| error!("TSP saple input not found"))?;
            let graph = tsp_validate::parse_sample_input(sample_input)
                .inspect_err(|err| error!("Failed parsing TSP sample input: {err}"))
                .map_err(|_| ErrorInternalServerError("Error code: 44"))?;
            tsp_validate::pre_validate(&path, tsp_validate::SAMPLE_GRAPH_SIZE)
                .and_then(|_| tsp_validate::validate(&graph, &path))
                .map(|score| HttpResponse::Ok().json(ValidateSampleSolutionResult::Valid(score)))
                .or_else(|err| {
                    Ok(HttpResponse::Ok()
                        .json(ValidateSampleSolutionResult::Invalid(err.to_string())))
                })
        }
        SampleSolution::GraphColoringSolution(coloring) => {
            let graph = state
                .static_data
                .problems
                .iter()
                .find(|problem| problem.id == "graph_coloring".into())
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
        SampleSolution::VertexCoverSolution(vertex_cover) => {
            let sample_input = state
                .static_data
                .problems
                .iter()
                .find(|problem| problem.id.as_ref() == "vertex_cover")
                .map(|problem| problem.sample_input.as_ref())
                .ok_or(ErrorInternalServerError("Error code: 45"))
                .inspect_err(|_| error!("Vertex Cover saple input not found"))?;
            let graph = vertex_cover_validate::parse_sample_input(sample_input)
                .inspect_err(|err| error!("Failed parsing Vertex Cover sample input: {err}"))
                .map_err(|_| ErrorInternalServerError("Error code: 46"))?;

            vertex_cover_validate::pre_validate(
                &vertex_cover,
                vertex_cover_validate::SAMPLE_GRAPH_SIZE,
            )
            .and_then(|solution| vertex_cover_validate::validate(&graph, &solution))
            .map(|score| HttpResponse::Ok().json(ValidateSampleSolutionResult::Valid(score)))
            .or_else(|err| {
                Ok(HttpResponse::Ok().json(ValidateSampleSolutionResult::Invalid(err.to_string())))
            })
        }
    }
}
