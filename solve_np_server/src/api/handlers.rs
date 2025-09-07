use crate::api::types::{
    HealthCheckReport, LogInForm, SampleSolution, ValidateSampleSolutionResult,
};
use crate::api::utils;
use crate::state::State;
use actix_session::Session;
use actix_web::error::ErrorBadRequest;
use actix_web::{
    HttpResponse, Responder, error::Error as HttpError, error::ErrorInternalServerError, web,
};
use log::debug;
use rusqlite::params;
use serde::Deserialize;
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
        .map_err(|_| ErrorInternalServerError("Internal error #1"))?;
    Ok(web::Redirect::to("/p_vs_np").see_other())
}
