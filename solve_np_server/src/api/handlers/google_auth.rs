use crate::auth::gauth::GOOGLE_AUTH_AUTHZ_ENDPOINT;
use crate::auth::gauth::GOOGLE_AUTH_SCOPE;
use crate::auth::gauth::GOOGLE_AUTH_TOKEN_ENDPOINT;
use crate::auth::gauth::redirect_uri;
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

#[derive(serde::Deserialize)]
pub struct GoogleAuthQueryParams {
    redirect: Option<String>,
}

pub async fn google_auth(
    state: web::Data<State>,
    query: web::Query<GoogleAuthQueryParams>,
) -> Result<impl Responder, HttpError> {
    let client_id = state
        .google_auth_client_id
        .as_ref()
        .ok_or(ErrorNotImplemented("Error code: 26"))?
        .0
        .as_str();
    let redirect_uri = redirect_uri(&state.address);
    let redirect_uri = utf8_percent_encode(&redirect_uri, NON_ALPHANUMERIC);
    let scope = utf8_percent_encode(GOOGLE_AUTH_SCOPE, NON_ALPHANUMERIC);
    let source = query.into_inner().redirect.unwrap_or(String::from("/"));
    Ok(web::Redirect::to(format!(
        "{GOOGLE_AUTH_AUTHZ_ENDPOINT}?client_id={client_id}&response_type=code&redirect_uri={redirect_uri}&scope={scope}&state={source}",
    )))
}

#[derive(Deserialize, Debug)]
pub struct GoogleAuthCallbackQueryParams {
    code: String,
    state: String,
}

#[derive(Deserialize, Debug)]
struct GoogleAuthTokenResponse {
    id_token: String,
}

pub async fn google_auth_callback(
    state: web::Data<State>,
    session: Session,
    query: web::Query<GoogleAuthCallbackQueryParams>,
) -> Result<impl Responder, HttpError> {
    let client_id = state
        .google_auth_client_id
        .as_ref()
        .ok_or(ErrorNotImplemented("Error code: 34"))?;
    let client_secret = state
        .google_auth_client_secret
        .as_ref()
        .ok_or(ErrorNotImplemented("Error code: 35"))?;
    let query = query.into_inner();
    let response = reqwest::Client::new()
        .post(GOOGLE_AUTH_TOKEN_ENDPOINT)
        .form(&[
            ("code", query.code.as_str()),
            ("client_id", &client_id.0),
            ("client_secret", &client_secret.0),
            ("redirect_uri", &redirect_uri(&state.address)),
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
    Ok(web::Redirect::to(query.state))
}
