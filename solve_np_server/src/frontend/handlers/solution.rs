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

struct Problem<'a> {
    id: &'a str,
    name: &'a str,
    description: &'a str,
}

struct Input<'a> {
    id: &'a str,
    filename: &'a str,
    preview: &'a str,
}

#[derive(Template)]
#[template(path = "solution.html")]
struct SolutionTemplate<'a> {
    header: HeaderTemplate,
    problem: Problem<'a>,
    sample_input: &'a str,
    inputs: Vec<Input<'a>>,
}

pub async fn solution(
    session: Session,
    _state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let body = SolutionTemplate {
        header: HeaderTemplate::try_from(&session)?,
        problem: Problem {
            id: "problem.id",
            name: "problem.name",
            description: "problem.description",
        },
        sample_input: "sample_input",
        inputs: vec![
            Input {
                id: "input.id.1",
                filename: "input.filename.1",
                preview: "input.preview.1",
            },
            Input {
                id: "input.id.2",
                filename: "input.filename.2",
                preview: "input.preview.2",
            },
            Input {
                id: "input.id.3",
                filename: "input.filename.3",
                preview: "input.preview.3",
            },
        ],
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
