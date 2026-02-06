use crate::auth::SessionId;
use crate::frontend::utils::md_to_html;
use crate::frontend_handlers::head::HeadTemplate;
use crate::frontend_handlers::header::HeaderTemplate;
use crate::frontend_handlers::header::Page;
use crate::frontend_handlers::solution_table::Solution;
use crate::frontend_handlers::solution_table::SolutionTable;
use crate::state::State;
use crate::types::InputId;
use crate::types::ProblemId;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::ContentType;
use actix_web::http::header::LOCATION;
use actix_web::web;
use askama::Template;
use itertools::Itertools;
use log::error;

pub struct Problem {
    id: ProblemId,
    name: String,
    solution_table: SolutionTable,
}

#[derive(Template)]
#[template(path = "solutions.html")]
struct SolutionsTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
    problems: Vec<Problem>,
}

#[derive(serde::Deserialize)]
pub struct SolutionsQueryParams {
    problem_id: Option<ProblemId>,
    input_id: Option<InputId>,
}

pub async fn solutions(
    session: Session,
    query: web::Query<SolutionsQueryParams>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let SolutionsQueryParams {
        problem_id,
        input_id,
    } = query.into_inner();
    let Some(user_id) = session
        .get::<SessionId>("session_id")?
        .and_then(|session_id| state.session_registry.get_user_id(session_id))
    else {
        return Ok(HttpResponse::SeeOther()
            .append_header((LOCATION, "/log_in?redirect=/solutions"))
            .finish());
    };
    let solutions = state
        .db
        .lock()
        .unwrap()
        .prepare("SELECT solution_id, problem_id, title, description, status FROM solutions WHERE user_id = ?1")
        .inspect_err(|err| error!("{err}"))
        .map_err(|_| ErrorInternalServerError("Error code: 7"))?
        .query_map(
            [user_id],
            |row| Ok(
                (
                    row.get::<usize, ProblemId>(1)?,
                    Solution {
                        id: row.get(0)?,
                        title: row.get(2)?,
                        description: md_to_html(row.get::<_, String>(3)?),
                        status: row.get(4)?,
                    },
                )
                )
            )
        .map_err(|_| ErrorInternalServerError("Error code: 8"))?
        .filter_map(Result::ok)
        .collect_vec();
    let problems = state
        .static_data
        .problems
        .iter()
        .filter(|&problem| match &problem_id {
            Some(problem_id) => problem_id == &problem.id,
            None => true,
        })
        .map(|problem| Problem {
            id: problem.id.clone(),
            name: problem.name.clone(),
            solution_table: SolutionTable {
                solutions: solutions
                    .iter()
                    .filter_map(|(problem_id, solution)| {
                        if problem_id == &problem.id {
                            Some(solution)
                        } else {
                            None
                        }
                    })
                    .cloned()
                    .collect(),
                problem_id: problem.id.clone(),
                input_id: input_id.clone(),
            },
        })
        .collect();
    let body = SolutionsTemplate {
        head: HeadTemplate::with_title("My Solutions - solveNP"),
        header: HeaderTemplate::try_from(&session, state)?.with_active(Page::Solutions),
        problems,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
