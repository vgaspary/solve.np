use crate::state::State;
use actix_session::Session;
use actix_web::{
    HttpResponse, Responder,
    error::{Error as HttpError, ErrorInternalServerError},
    http::header::ContentType,
    web,
};
use askama::Template;
use serde::Deserialize;
use log::{debug};

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
    let mut problems = state
        .static_data
        .iter()
        .map(|(id, problem)| crate::api::types::Problem {
            id: id.clone(),
            name: problem.name.clone(),
            description: problem.description.clone(),
        })
        .collect::<Vec<_>>();
    problems.sort_by_key(|problem| problem.id.clone());
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
    problem_id: String,
    sample_input: String,
}

pub async fn problem_statement(
    session: Session,
    path: web::Path<String>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let problem_id = path.into_inner();
    let sample_input = state
        .static_data
        .get(&problem_id)
        .map(|problem| problem.sample_input.clone())
        .unwrap_or_default();
    let body = ProblemStatementTemplate {
        header: HeaderTemplate::try_from(&session)?,
        problem_id,
        sample_input,
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
                        [user_id],
                        |row| row.get(0),
                    )
                    .inspect(|result| debug!("Result is {result}"))
                    .inspect_err(|err| debug!("Query failed {err}"))
                    .ok()
            })
            .unwrap_or(false);
    let body = PvsNpTemplate {
        header,
        logged_in,
        voted,
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
    error_msg: String,
}

#[derive(Deserialize)]
pub struct LogInQueryParams {
    err: Option<String>,
}

pub async fn log_in(
    params: web::Query<LogInQueryParams>,
    session: Session,
) -> Result<impl Responder, HttpError> {
    let body = LogInTemplate {
        header: HeaderTemplate::try_from(&session)?,
        error_msg: params.err.clone().unwrap_or_default(),
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
