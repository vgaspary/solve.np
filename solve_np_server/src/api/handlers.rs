use crate::api::types::{HealthCheckReport, Input, Problem, ProblemDetails};
use crate::state::State;
use actix_web::{
    HttpResponse, Responder, error::Error as HttpError, error::ErrorInternalServerError, web,
};
use pulldown_cmark::Parser;
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
