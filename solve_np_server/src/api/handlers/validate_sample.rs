use crate::state::State;
use crate::types::ProblemId;
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
    Tsp(Vec<usize>),
    GraphColoring(Vec<usize>),
    VertexCover(Vec<usize>),
    MaximumClique(Vec<usize>),
    MaximumCut(Vec<usize>),
    SteinerTree(Vec<(usize, usize)>),
    Knapsack(Vec<usize>),
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
        SampleSolution::Tsp(path) => {
            use tsp_validate::SAMPLE_GRAPH_SIZE;
            use tsp_validate::parse_sample_input;
            use tsp_validate::pre_validate;
            use tsp_validate::validate;

            let sample_input = state
                .static_data
                .problems
                .iter()
                .find(|problem| problem.id == ProblemId::TravellingSalesman)
                .map(|problem| problem.sample_input.as_ref())
                .ok_or(ErrorInternalServerError("Error code: 43"))
                .inspect_err(|_| error!("TSP saple input not found"))?;
            let graph = parse_sample_input(sample_input)
                .inspect_err(|err| error!("Failed parsing TSP sample input: {err}"))
                .map_err(|_| ErrorInternalServerError("Error code: 44"))?;
            pre_validate(&path, SAMPLE_GRAPH_SIZE)
                .and_then(|_| validate(&graph, &path))
                .map(|score| HttpResponse::Ok().json(ValidateSampleSolutionResult::Valid(score)))
                .or_else(|err| {
                    Ok(HttpResponse::Ok()
                        .json(ValidateSampleSolutionResult::Invalid(err.to_string())))
                })
        }
        SampleSolution::GraphColoring(coloring) => {
            use graph_coloring_validate::SAMPLE_GRAPH_SIZE;
            use graph_coloring_validate::parse_sample_input;
            use graph_coloring_validate::pre_validate;
            use graph_coloring_validate::validate;

            let sample_input = state
                .static_data
                .problems
                .iter()
                .find(|problem| problem.id == ProblemId::GraphColoring)
                .map(|problem| problem.sample_input.as_ref())
                .ok_or(ErrorInternalServerError("Error code: 47"))
                .inspect_err(|_| error!("Graph Coloring saple input not found"))?;
            let graph = parse_sample_input(sample_input)
                .inspect_err(|err| error!("Failed parsing Graph Coloring sample input: {err}"))
                .map_err(|_| ErrorInternalServerError("Error code: 48"))?;
            pre_validate(&coloring, SAMPLE_GRAPH_SIZE)
                .and_then(|_| validate(&graph, &coloring))
                .map(|score| HttpResponse::Ok().json(ValidateSampleSolutionResult::Valid(score)))
                .or_else(|err| {
                    Ok(HttpResponse::Ok()
                        .json(ValidateSampleSolutionResult::Invalid(err.to_string())))
                })
        }
        SampleSolution::VertexCover(vertex_cover) => {
            use vertex_cover_validate::SAMPLE_GRAPH_SIZE;
            use vertex_cover_validate::parse_sample_input;
            use vertex_cover_validate::pre_validate;
            use vertex_cover_validate::validate;

            let sample_input = state
                .static_data
                .problems
                .iter()
                .find(|problem| problem.id == ProblemId::VertexCover)
                .map(|problem| problem.sample_input.as_ref())
                .ok_or(ErrorInternalServerError("Error code: 45"))
                .inspect_err(|_| error!("Vertex Cover sample input not found"))?;
            let graph = parse_sample_input(sample_input)
                .inspect_err(|err| error!("Failed parsing Vertex Cover sample input: {err}"))
                .map_err(|_| ErrorInternalServerError("Error code: 46"))?;
            pre_validate(&vertex_cover, SAMPLE_GRAPH_SIZE)
                .and_then(|solution| validate(&graph, &solution))
                .map(|score| HttpResponse::Ok().json(ValidateSampleSolutionResult::Valid(score)))
                .or_else(|err| {
                    Ok(HttpResponse::Ok()
                        .json(ValidateSampleSolutionResult::Invalid(err.to_string())))
                })
        }
        SampleSolution::MaximumClique(clique) => {
            use maximum_clique_validate::SAMPLE_GRAPH_SIZE;
            use maximum_clique_validate::parse_sample_input;
            use maximum_clique_validate::pre_validate;
            use maximum_clique_validate::validate;

            let sample_input = state
                .static_data
                .problems
                .iter()
                .find(|problem| problem.id == ProblemId::MaximumClique)
                .map(|problem| problem.sample_input.as_ref())
                .ok_or(ErrorInternalServerError("Error code: 49"))
                .inspect_err(|_| error!("Maximum Clique sample input not found"))?;
            let graph = parse_sample_input(sample_input)
                .inspect_err(|err| error!("Failed parsing Maximum Clique sample input: {err}"))
                .map_err(|_| ErrorInternalServerError("Error code: 50"))?;
            pre_validate(&clique, SAMPLE_GRAPH_SIZE)
                .and_then(|_| validate(&graph, &clique))
                .map(|score| HttpResponse::Ok().json(ValidateSampleSolutionResult::Valid(score)))
                .or_else(|err| {
                    Ok(HttpResponse::Ok()
                        .json(ValidateSampleSolutionResult::Invalid(err.to_string())))
                })
        }
        SampleSolution::MaximumCut(vertices) => {
            use maximum_cut_validate::SAMPLE_GRAPH_SIZE;
            use maximum_cut_validate::parse_sample_input;
            use maximum_cut_validate::pre_validate;
            use maximum_cut_validate::validate;

            let sample_input = state
                .static_data
                .problems
                .iter()
                .find(|problem| problem.id == ProblemId::MaximumCut)
                .map(|problem| problem.sample_input.as_ref())
                .ok_or(ErrorInternalServerError("Error code: 51"))
                .inspect_err(|_| error!("Maximum Cut sample input not found"))?;
            let graph = parse_sample_input(sample_input)
                .inspect_err(|err| error!("Failed parsing Maximum Cut sample input: {err}"))
                .map_err(|_| ErrorInternalServerError("Error code: 52"))?;
            pre_validate(&vertices, SAMPLE_GRAPH_SIZE)
                .and_then(|vertices| validate(&graph, &vertices))
                .map(|score| HttpResponse::Ok().json(ValidateSampleSolutionResult::Valid(score)))
                .or_else(|err| {
                    Ok(HttpResponse::Ok()
                        .json(ValidateSampleSolutionResult::Invalid(err.to_string())))
                })
        }
        SampleSolution::SteinerTree(edges) => {
            use steiner_tree_validate::SAMPLE_GRAPH_SIZE;
            use steiner_tree_validate::parse_sample_input;
            use steiner_tree_validate::pre_validate;
            use steiner_tree_validate::validate;

            let sample_input = state
                .static_data
                .problems
                .iter()
                .find(|problem| problem.id == ProblemId::SteinerTree)
                .map(|problem| problem.sample_input.as_ref())
                .ok_or(ErrorInternalServerError("Error code: 53"))
                .inspect_err(|_| error!("Steiner Tree sample input not found"))?;
            let graph = parse_sample_input(sample_input)
                .inspect_err(|err| error!("Failed parsing Steiner Tree sample input: {err}"))
                .map_err(|_| ErrorInternalServerError("Error code: 54"))?;
            pre_validate(&edges, SAMPLE_GRAPH_SIZE)
                .and_then(|_| validate(&graph, &edges))
                .map(|score| HttpResponse::Ok().json(ValidateSampleSolutionResult::Valid(score)))
                .or_else(|err| {
                    Ok(HttpResponse::Ok()
                        .json(ValidateSampleSolutionResult::Invalid(err.to_string())))
                })
        }
        SampleSolution::Knapsack(items) => {
            use knapsack_validate::SAMPLE_NUM_ITEMS;
            use knapsack_validate::parse_sample_input;
            use knapsack_validate::pre_validate;
            use knapsack_validate::validate;

            let sample_input = state
                .static_data
                .problems
                .iter()
                .find(|problem| problem.id == ProblemId::Knapsack)
                .map(|problem| problem.sample_input.as_ref())
                .ok_or(ErrorInternalServerError("Error code: 55"))
                .inspect_err(|_| error!("Knapsack sample input not found"))?;
            let input = parse_sample_input(sample_input)
                .inspect_err(|err| error!("Failed parsing Knapsack sample input: {err}"))
                .map_err(|_| ErrorInternalServerError("Error code: 56"))?;
            pre_validate(&items, SAMPLE_NUM_ITEMS)
                .and_then(|_| validate(&input, &items))
                .map(|score| {
                    HttpResponse::Ok().json(ValidateSampleSolutionResult::Valid(score as usize))
                })
                .or_else(|err| {
                    Ok(HttpResponse::Ok()
                        .json(ValidateSampleSolutionResult::Invalid(err.to_string())))
                })
        }
    }
}
