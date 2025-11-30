use crate::frontend::handlers::head::HeadTemplate;
use crate::frontend::handlers::header::HeaderTemplate;
use crate::frontend::handlers::header::Page;
use crate::state::State;
use crate::types::ProblemId;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::ContentType;
use actix_web::web;
use askama::Template;

struct Problem {
    id: ProblemId,
    name: String,
    description: String,
}

#[derive(Template)]
#[template(path = "problems.html")]
struct ProblemsTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
    problems: Vec<Problem>,
}

pub async fn problems(
    session: Session,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let problems = state
        .static_data
        .problems
        .iter()
        .map(|problem| Problem {
            id: problem.id.clone(),
            name: problem.name.clone(),
            description: problem.description.clone(),
        })
        .collect();
    let body = ProblemsTemplate {
        head: HeadTemplate {
            title: "Problems - solve.NP".into(),
        },
        header: HeaderTemplate::try_from(&session, state)?.with_active(Page::Problems),
        problems,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
