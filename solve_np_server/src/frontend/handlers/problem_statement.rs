use crate::{
    frontend::handlers::header::HeaderTemplate,
    frontend::utils::{bytes_to_string_h, get_file_metadata},
    state::State,
};
use actix_session::Session;
use actix_web::{
    HttpResponse, Responder,
    error::{Error as HttpError, ErrorInternalServerError},
    http::header::ContentType,
    web,
};
use askama::Template;
use itertools::Itertools;
use log::debug;
use pulldown_cmark::Parser;
use std::fs;

#[derive(Template)]
#[template(path = "problem_statement.html")]
struct ProblemStatementTemplate {
    header: HeaderTemplate,
    problem: Problem,
}

struct Problem {
    id: String,
    name: String,
    problem_statement: String,
    sample_input: String,
    inputs: Vec<Input>,
}

struct Input {
    id: String,
    filename: String,
    size: u64,
    size_h: String,
}

pub async fn problem_statement(
    session: Session,
    path: web::Path<String>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let problem_id = path.into_inner();
    let problem = state
        .static_data
        .get(&problem_id)
        .ok_or_else(|| ErrorInternalServerError("No problem found with ID"))
        .and_then(move |problem| {
            let md_content = fs::read_to_string(&problem.problem_statement)?;
            let parser = Parser::new(&md_content);
            let mut problem_statement = String::new();
            pulldown_cmark::html::push_html(&mut problem_statement, parser);
            Ok(Problem {
                id: problem_id,
                name: problem.name.clone(),
                problem_statement,
                sample_input: problem.sample_input.clone(),
                inputs: problem
                    .inputs
                    .iter()
                    .map(|(id, input)| {
                        let (filename, size) = get_file_metadata(&input.path)
                            .inspect_err(|err| {
                                debug!(
                                    "Failed acquiring metadata from path {:?}: {err}",
                                    input.path
                                )
                            })
                            .map_err(|_| ErrorInternalServerError("Internal error #3"))?;
                        Ok::<Input, HttpError>(Input {
                            id: id.clone(),
                            filename,
                            size,
                            size_h: bytes_to_string_h(size),
                        })
                    })
                    .sorted_by_key(|input| input.as_ref().map_or(0, |input| input.size))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })?;
    let body = ProblemStatementTemplate {
        header: HeaderTemplate::try_from(&session)?,
        problem,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
