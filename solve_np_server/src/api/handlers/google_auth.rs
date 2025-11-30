use crate::auth::gauth::GAUTH_AUTH_ENDPOINT;
use crate::auth::gauth::GAUTH_REDIRECT_URI;
use crate::auth::gauth::GAUTH_SCOPE;
use crate::auth::gauth::GAUTH_TOKEN_ENDPOINT;
use crate::auth::gauth::GAuthClientId;
use crate::auth::gauth::validate_id_token;
use crate::state::State;
use actix_session::Session;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::error::ErrorNotImplemented;
use actix_web::error::ErrorUnauthorized;
use actix_web::web;
use log::error;
use percent_encoding::NON_ALPHANUMERIC;
use percent_encoding::utf8_percent_encode;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GoogleAuthParams {
    code: String,
}

#[derive(Deserialize, Debug)]
struct GoogleAuthTokenResponse {
    id_token: String,
}

pub async fn google_auth(state: web::Data<State>) -> Result<impl Responder, HttpError> {
    let Some((GAuthClientId(client_id), _)) = &state.gauth_client_id_secret else {
        return Err(ErrorNotImplemented("Internal error #34"));
    };
    let redirect_uri = utf8_percent_encode(GAUTH_REDIRECT_URI, NON_ALPHANUMERIC);
    let scope = utf8_percent_encode(GAUTH_SCOPE, NON_ALPHANUMERIC);
    Ok(web::Redirect::to(format!(
        "{GAUTH_AUTH_ENDPOINT}?client_id={client_id}&response_type=code&redirect_uri={redirect_uri}&scope={scope}"
    )))
}

pub async fn google_auth_callback(
    state: web::Data<State>,
    session: Session,
    query: web::Query<GoogleAuthParams>,
) -> Result<impl Responder, HttpError> {
    let Some((client_id, client_secret)) = &state.gauth_client_id_secret else {
        return Err(ErrorNotImplemented("Internal error #35"));
    };
    let response = reqwest::Client::new()
        .post(GAUTH_TOKEN_ENDPOINT)
        .form(&[
            ("code", query.code.as_str()),
            ("client_id", &client_id.0),
            ("client_secret", &client_secret.0),
            ("redirect_uri", GAUTH_REDIRECT_URI),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .inspect_err(|e| error!("{e}"))
        .map_err(|_| ErrorInternalServerError("Failed to reach Google token endpoint."))?;
    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        error!("Google token endpoint failed: {}", error_text);
        return Err(ErrorInternalServerError("Google token endpoint failed."));
    }
    let token_response = response
        .json::<GoogleAuthTokenResponse>()
        .await
        .inspect_err(|e| error!("{e}"))
        .map_err(|_| ErrorInternalServerError("Failed to parse Google token response."))?;
    let claims = validate_id_token(&token_response.id_token, client_id)
        .await
        .map_err(ErrorUnauthorized)?;
    let session_id = state.session_registry.create_session(claims);
    session.insert("session_id", session_id)?;
    // TODO! redirect to home, p_vs_np, or some problem page.
    Ok(web::Redirect::to("/"))
}
