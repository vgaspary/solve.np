use crate::api::types::{
    HealthCheckReport, Input, LogInForm, Problem, ProblemDetails, SampleSolution,
    ValidateSampleSolutionResult,
};
use crate::api::utils;
use crate::state::State;
use actix_session::Session;
use actix_web::error::ErrorBadRequest;
use actix_web::{
    HttpResponse, Responder, error::Error as HttpError, error::ErrorInternalServerError, web,
};
use log::debug;
use pulldown_cmark::Parser;
use rusqlite::params;
use serde::Deserialize;
use std::fs;
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
        .ok_or_else(|| ErrorInternalServerError("No problem found with ID"))
        .and_then(|problem| {
            let md_content = fs::read_to_string(&problem.problem_statement)?;
            let parser = Parser::new(&md_content);
            let mut problem_statement_html = String::new();
            pulldown_cmark::html::push_html(&mut problem_statement_html, parser);
            Ok(ProblemDetails {
                id: problem_id,
                name: problem.name.clone(),
                problem_statement_html,
                inputs: problem
                    .inputs
                    .iter()
                    .map(|(id, input)| Input {
                        id: id.clone(),
                        path: input.path.clone(),
                    })
                    .collect(),
            })
        })
        .map(|problem_details| HttpResponse::Ok().json(problem_details))
}

#[derive(Deserialize)]
pub struct LogInQueryParams {
    redirect: Option<String>,
}

pub async fn log_in(
    session: Session,
    params: web::Query<LogInQueryParams>,
    form: web::Form<LogInForm>,
) -> Result<impl Responder, HttpError> {
    let LogInForm { username, password } = form.into_inner();
    Ok(if username == password {
        web::Redirect::to("/log_in?err=Invalid%20credentials").see_other()
    } else {
        session.insert("user_id", sha256::digest(format!("{username}{password}")))?;
        session.insert("username", username)?;
        web::Redirect::to(params.redirect.clone().unwrap_or("/".to_string())).see_other()
    })
}

pub async fn log_out(session: Session) -> impl Responder {
    session.clear();
    web::Redirect::to("/").see_other()
}

pub async fn validate_sample_solution(
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

pub async fn vote(
    session: Session,
    path: web::Path<String>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let user_id = session
        .get::<String>("user_id")?
        .ok_or(ErrorInternalServerError("user_id not found"))?;
    let vote = match path.into_inner().as_str() {
        "eq" => true,
        "neq" => false,
        _ => return Err(ErrorBadRequest("Vote may only be 'eq' or 'neq'.")),
    };
    debug!("user_id {user_id} voteed on P vs NP: {vote}");
    state
        .db
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO votes (user_id, vote) VALUES (?1, ?2)",
            params![user_id, vote],
        )
        .inspect_err(|err| debug!("SQL query failed: {err}"))
        .map_err(|_| ErrorInternalServerError("Unexpected error #1"))?;
    Ok(web::Redirect::to("/p_vs_np").see_other())
}
