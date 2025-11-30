use std::time::UNIX_EPOCH;

use crate::auth::IdTokenClaims;
use crate::state::State;
use actix_session::Session;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorNotImplemented;
use actix_web::web;

#[derive(serde::Deserialize)]
pub struct LogInQueryParams {
    redirect: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct LogInForm {
    username: String,
    password: String,
}

pub async fn log_in(
    state: web::Data<State>,
    session: Session,
    query: web::Query<LogInQueryParams>,
    form: web::Form<LogInForm>,
) -> Result<impl Responder, HttpError> {
    if state.gauth_client_id_secret.is_some() {
        return Err(ErrorNotImplemented("Internal error #36"));
    }
    let LogInForm { username, password } = form.into_inner();
    Ok(if username == password {
        web::Redirect::to("/log_in?err=Invalid%20credentials").see_other()
    } else {
        let iat = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Now is later than UNIX epoch.")
            .as_secs() as usize;
        let user_id = sha256::digest(format!("{username}{password}"));
        let session_id = state.session_registry.create_session(IdTokenClaims {
            iss: String::new(),
            aud: String::new(),
            exp: usize::MAX,
            iat,
            sub: user_id,
            name: username,
            given_name: String::new(),
            family_name: String::new(),
        });
        session.insert("session_id", session_id)?;
        web::Redirect::to(query.redirect.clone().unwrap_or("/".to_string())).see_other()
    })
}
