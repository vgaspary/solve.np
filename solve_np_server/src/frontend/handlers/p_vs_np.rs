use crate::{frontend::handlers::header::HeaderTemplate, state::State};
use actix_session::Session;
use actix_web::{
    HttpResponse, Responder,
    error::{Error as HttpError, ErrorInternalServerError},
    http::header::ContentType,
    web,
};
use askama::Template;
use log::debug;
use rusqlite::params;

#[derive(Template)]
#[template(path = "p_vs_np.html")]
struct PvsNpTemplate {
    header: HeaderTemplate,
    voted: bool,
    num_p_eq_np_votes: usize,
    num_p_neq_np_votes: usize,
}

pub async fn p_vs_np(
    session: Session,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let header = HeaderTemplate::try_from(&session)?;
    let voted = header.logged_in
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
    let (num_p_eq_np_votes, num_p_neq_np_votes) = if header.logged_in {
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
