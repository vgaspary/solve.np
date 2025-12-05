use crate::auth::SessionId;
use crate::frontend::handlers::head::HeadTemplate;
use crate::frontend::handlers::header::HeaderTemplate;
use crate::frontend::handlers::header::Page;
use crate::state::State;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::ContentType;
use actix_web::web;
use askama::Template;
use log::error;
use rusqlite::params;

#[derive(Clone, Copy)]
struct PEqualsNP(bool);

#[derive(Template)]
#[template(path = "p_vs_np.html")]
struct PvsNpTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
    eq_neq_text: Option<(String, String)>,
}

fn get_vote_data(
    session: &Session,
    state: &web::Data<State>,
) -> Result<Option<(PEqualsNP, i64, i64)>, HttpError> {
    let Some(user_id) = session
        .get::<SessionId>("session_id")?
        .and_then(|session_id| state.session_registry.get_user_id(session_id))
    else {
        return Ok(None);
    };
    let db = state.db.lock().unwrap();
    let Some(voted_eq) = db
        .query_one(
            "SELECT vote FROM votes WHERE user_id = ?1",
            params![user_id],
            |row| row.get(0),
        )
        .inspect_err(|err| error!("{err}"))
        .ok()
    else {
        return Ok(None);
    };
    let vote = PEqualsNP(voted_eq);
    let (num_eq, num_neq) = db
        .query_one(
            "SELECT COUNT(*) FILTER (WHERE vote = 1), COUNT(*) FILTER (WHERE vote = 0) FROM votes",
            params![],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((0, 0));
    Ok(Some((vote, num_eq, num_neq)))
}

fn format_eq_neq_row(num_votes: i64, vote: PEqualsNP, eq: PEqualsNP) -> String {
    let single = num_votes == 1;
    format!(
        "<em>{num_votes}</em> {} <em>P {} NP</em>{}",
        if single { "user thinks" } else { "users think" },
        if eq.0 { "==" } else { "!=" },
        if vote.0 == eq.0 {
            if single {
                " <i>(only you)</i>"
            } else {
                " <i>(including you)</i>"
            }
        } else {
            ""
        },
    )
}

pub async fn p_vs_np(
    session: Session,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let head = HeadTemplate {
        title: "P vs NP - solveNP".into(),
    };
    let header = HeaderTemplate::try_from(&session, state.clone())?.with_active(Page::PvsNP);
    let eq_neq_text = if header.logged_in {
        get_vote_data(&session, &state)?.and_then(|(vote, num_eq, num_neq)| {
            Some((
                format_eq_neq_row(num_eq, vote, PEqualsNP(true)),
                format_eq_neq_row(num_neq, vote, PEqualsNP(false)),
            ))
        })
    } else {
        None
    };

    let body = PvsNpTemplate {
        head,
        header,
        eq_neq_text,
    }
    .render()
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
