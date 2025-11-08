use crate::state::State;
use crate::types::InputId;
use crate::types::ProblemId;
use crate::types::SolutionId;
use crate::types::Status;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::rt::task;
use actix_web::web;
use itertools::Itertools;
use log::error;
use rusqlite::params;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

pub type Score = usize;

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Solution {
    TspSolution(Vec<usize>),
    GraphColoringSolution(Vec<usize>),
}

impl ToString for Solution {
    fn to_string(&self) -> String {
        match self {
            Self::TspSolution(path) => path.iter().join(" "),
            Self::GraphColoringSolution(colors) => colors.iter().join(" "),
        }
    }
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
    path: web::Path<(ProblemId, SolutionId, InputId)>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let user_id = session
        .get::<String>("user_id")?
        .ok_or(ErrorInternalServerError("Internal error #14"))?;
    let user_name = session
        .get::<String>("username")?
        .ok_or(ErrorInternalServerError("Internal error #33"))?;
    let solution = solution.into_inner();
    let (problem_id, solution_id, input_id) = path.into_inner();

    let hash_digest = hash(&solution) as i64;
    let same_hash_digest: bool = state
        .db
        .lock()
        .unwrap()
        .query_one(
            "SELECT EXISTS (
                SELECT 1 FROM submissions
                WHERE
                    user_id = ?1 AND
                    problem_id = ?2 AND
                    solution_id = ?3 AND
                    input_id = ?4 AND
                    hash_digest = ?5);",
            params![user_id, problem_id, solution_id, input_id, hash_digest],
            |row| row.get(0),
        )
        .inspect_err(|err| error!("{err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #15"))?;
    if same_hash_digest {
        return Ok(HttpResponse::NoContent().finish());
    }

    state
        .db
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO submissions (user_id, user_name, problem_id, solution_id, input_id, solution, hash_digest, status, invalid_message)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(user_id, problem_id, solution_id, input_id)
             DO UPDATE SET
                solution = ?6,
                hash_digest = ?7,
                status = ?8,
                invalid_message = ?9;",
            params![user_id, user_name, problem_id, solution_id, input_id, solution.to_string(), hash_digest, Status::Untested, ""],
        )
        .inspect_err(|err| error!("SQL query failed: {err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #16"))?;

    let state_clone = state.clone();
    let problem_id_clone = problem_id.clone();
    let input_id_clone = input_id.clone();
    let validation_result = task::spawn_blocking(move || {
        let input = state_clone
            .static_data
            .problems
            .iter()
            .find(|problem| problem.id == problem_id_clone)
            .and_then(|problem| {
                problem
                    .inputs
                    .iter()
                    .find(|input| input.id == input_id_clone)
            })
            .and_then(|input| fs::read_to_string(&input.path).ok());
        match solution {
            Solution::TspSolution(path) => {
                let graph = input
                    .and_then(|input| tsp_validate::parse_input(&input))
                    .ok_or("Internal error #12")?;
                Ok(tsp_validate::validate(&graph, path)
                    .map(|score| ValidateSolutionResult::Valid(score))
                    .unwrap_or_else(|err| ValidateSolutionResult::Invalid(err.to_string())))
            }
            Solution::GraphColoringSolution(coloring) => {
                let graph = input
                    .and_then(|input| graph_coloring_validate::parse_input(&input))
                    .ok_or("Internal error #13")?;
                Ok(graph_coloring_validate::validate(&graph, coloring)
                    .map(|score| ValidateSolutionResult::Valid(score))
                    .unwrap_or_else(|err| ValidateSolutionResult::Invalid(err.to_string())))
            }
        }
    })
    .await
    .map_err(|_| ErrorInternalServerError("Internal error #19"))?
    .map_err(|msg: &'static str| ErrorInternalServerError(msg))?;

    match &validation_result {
        ValidateSolutionResult::Valid(score) => {
            state
                .db
                .lock()
                .unwrap()
                .execute(
                    "UPDATE submissions SET
                        status = ?5
                     WHERE
                        user_id = ?1 AND
                        problem_id = ?2 AND
                        solution_id = ?3 AND
                        input_id = ?4;",
                    params![
                        user_id,
                        problem_id,
                        solution_id,
                        input_id,
                        Status::Solved(*score)
                    ],
                )
                .inspect_err(|err| error!("SQL query failed: {err}"))
                .map_err(|_| ErrorInternalServerError("Internal error #17"))?;
        }
        ValidateSolutionResult::Invalid(invalid_message) => {
            state
                .db
                .lock()
                .unwrap()
                .execute(
                    "UPDATE submissions SET
                        status = ?5,
                        invalid_message = ?6
                     WHERE
                        user_id = ?1 AND
                        problem_id = ?2 AND
                        solution_id = ?3 AND
                        input_id = ?4;",
                    params![
                        user_id,
                        problem_id,
                        solution_id,
                        input_id,
                        Status::Attempted,
                        invalid_message,
                    ],
                )
                .inspect_err(|err| error!("SQL query failed: {err}"))
                .map_err(|_| ErrorInternalServerError("Internal error #18"))?;
        }
    }
    Ok(HttpResponse::Ok().json(validation_result))
}
