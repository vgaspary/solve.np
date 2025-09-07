use crate::{
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
use rusqlite::params;
use serde::Deserialize;
use std::fs;

#[derive(Template)]
#[template(path = "header.html")]
struct HeaderTemplate {
    logged_in: bool,
}

impl TryFrom<&Session> for HeaderTemplate {
    type Error = actix_session::SessionGetError;
    fn try_from(session: &Session) -> Result<Self, Self::Error> {
        Ok(Self {
            logged_in: session.get::<String>("username")?.is_some(),
        })
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    header: HeaderTemplate,
    user_name: String,
}

pub async fn home_page(session: Session) -> Result<impl Responder, HttpError> {
    let user_name = session
        .get::<String>("username")?
        .unwrap_or(String::from("Unknown user"));
    let body = IndexTemplate {
        header: HeaderTemplate::try_from(&session)?,
        user_name,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}

#[derive(Template)]
#[template(path = "problems.html")]
struct ProblemsTemplate {
    header: HeaderTemplate,
    problems: Vec<crate::api::types::Problem>,
}

pub async fn problems(
    session: Session,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let problems = state
        .static_data
        .iter()
        .map(|(id, problem)| crate::api::types::Problem {
            id: id.clone(),
            name: problem.name.clone(),
            description: problem.description.clone(),
        })
        .sorted_by_key(|problem| problem.id.clone())
        .collect::<Vec<_>>();
    let body = ProblemsTemplate {
        header: HeaderTemplate::try_from(&session)?,
        problems,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}

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
                    .map(|(_, input)| {
                        get_file_metadata(&input.path)
                            .inspect_err(|err| {
                                debug!(
                                    "Failed acquiring metadata from path {:?}: {err}",
                                    input.path
                                )
                            })
                            .expect("Unexpected error #3")
                    })
                    .map(|(filename, size)| Input {
                        filename,
                        size,
                        size_h: bytes_to_string_h(size),
                    })
                    .sorted_by_key(|input| input.size)
                    .collect(),
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

#[derive(Template)]
#[template(path = "p_vs_np.html")]
struct PvsNpTemplate {
    header: HeaderTemplate,
    logged_in: bool,
    voted: bool,
    num_p_eq_np_votes: usize,
    num_p_neq_np_votes: usize,
}

pub async fn p_vs_np(
    session: Session,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let header = HeaderTemplate::try_from(&session)?;
    let logged_in = header.logged_in;
    let voted = logged_in
        && session
            .get::<String>("user_id")
            .inspect(|user_id| debug!("User ID is {user_id:?}"))
            .unwrap_or_default()
            .and_then(|user_id| {
                state
                    .db
                    .lock()
                    .unwrap()
                    .query_row(
                        "SELECT EXISTS ( SELECT 1 FROM votes WHERE user_id = ?1 )",
                        params![user_id],
                        |row| row.get(0),
                    )
                    .inspect(|result| debug!("voted: {result}"))
                    .inspect_err(|err| debug!("Query failed {err}"))
                    .ok()
            })
            .unwrap_or(false);
    let (num_p_eq_np_votes, num_p_neq_np_votes) = if logged_in {
        state
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FILTER (WHERE vote = 1), COUNT(*) FILTER (WHERE vote = 0) FROM votes",
                params![],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .inspect(|result| debug!("(num_eq, num_neq): {result:?}"))
            .inspect_err(|err| debug!("Query failed {err}"))
            .map_err(|_| ErrorInternalServerError("Internal error #2"))?
    } else {
        (0, 0)
    };
    let body = PvsNpTemplate {
        header,
        logged_in,
        voted,
        num_p_eq_np_votes,
        num_p_neq_np_votes,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}

#[derive(Template)]
#[template(path = "log_in.html")]
struct LogInTemplate {
    header: HeaderTemplate,
    redirect: Option<String>,
    error_msg: String,
}

#[derive(Deserialize)]
pub struct LogInQueryParams {
    redirect: Option<String>,
    err: Option<String>,
}

pub async fn log_in(
    params: web::Query<LogInQueryParams>,
    session: Session,
) -> Result<impl Responder, HttpError> {
    let body = LogInTemplate {
        header: HeaderTemplate::try_from(&session)?,
        redirect: params.redirect.clone(),
        error_msg: params.err.clone().unwrap_or_default(),
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
