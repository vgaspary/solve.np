use std::fs::metadata;
use std::os::unix::fs::MetadataExt;

use crate::frontend::handlers::header::HeaderTemplate;
use crate::state::State;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::ContentType;
use actix_web::web;
use askama::Template;
use itertools::Itertools;
use log::error;

type Score = usize;

#[derive(Clone, Copy)]
pub enum Status {
    Untested,
    Attempted,
    Solved(Score),
}

impl TryFrom<i64> for Status {
    type Error = ();
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Status::Untested),
            1 => Ok(Status::Attempted),
            2 => Ok(Status::Solved(0)),
            _ => Err(()),
        }
    }
}

struct Problem {
    id: String,
    name: String,
}

struct Input {
    id: String,
    filename: String,
    preview: String,
    status: Status,
    size: u64,
}

struct Solution {
    id: String,
    problem_id: String,
    title: String,
    description: String,
    status: Status,
}

#[derive(Template)]
#[template(path = "solution.html")]
struct SolutionTemplate {
    header: HeaderTemplate,
    solution: Solution,
    problem: Problem,
    inputs: Vec<Input>,
}

pub async fn solution(
    session: Session,
    path: web::Path<String>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let solution_id = path.into_inner();
    let user_id = session
        .get::<String>("user_id")?
        .ok_or(ErrorInternalServerError("Internal error #9"))?;
    let solution = state
        .db
        .lock()
        .unwrap()
        .query_row(
            "SELECT problem_id, title, description, status FROM attempts WHERE user_id = ?1 AND solution_id = ?2",
            [&user_id, &solution_id],
            |row| Ok(Solution {
                id: solution_id.clone(),
                problem_id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                status: Status::try_from(row.get::<usize, i64>(3)?).expect("Status value must be 0 || 1 || 2."),
            })
        )
        .inspect_err(|err| error!("SQL query failed: {err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #10"))?;
    let (problem, inputs) = state
        .static_data
        .get(&solution.problem_id)
        .map(|problem| {
            (
                Problem {
                    id: solution.problem_id.clone(),
                    name: problem.name.clone(),
                },
                problem
                    .inputs
                    .iter()
                    .map(|(id, input)| {
                        (
                            id,
                            input,
                            metadata(&input.path)
                                .map(|metadata| metadata.size())
                                .unwrap_or(0),
                        )
                    })
                    .sorted_by_key(|(_, _, size)| *size)
                    .map(|(id, input, _)| Input {
                        id: id.clone(),
                        filename: input.filename.clone(),
                        preview: input.preview.clone(),
                        status: if id.starts_with("s") {
                            Status::Solved(0)
                        } else if id.starts_with("m") {
                            Status::Attempted
                        } else {
                            Status::Untested
                        }, // TODO!
                        size: metadata(&input.path)
                            .map(|metadata| metadata.size())
                            .unwrap_or_default(),
                    })
                    .sorted_by_key(|input| input.size)
                    .collect(),
            )
        })
        .ok_or(ErrorInternalServerError("Internal error #11"))?;

    let body = SolutionTemplate {
        header: HeaderTemplate::try_from(&session)?,
        solution,
        problem,
        inputs,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
