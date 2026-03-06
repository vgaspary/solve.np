use crate::auth::IdTokenClaims;
use crate::auth::github_auth::GITHUB_AUTH_AUTHZ_ENDPOINT;
use crate::auth::github_auth::GITHUB_AUTH_TOKEN_ENDPOINT;
use crate::auth::github_auth::GITHUB_AUTH_USER_ENDPOINT;
use crate::auth::github_auth::GithubAuthClientId;
use crate::auth::github_auth::GithubAuthClientSecret;
use crate::auth::github_auth::redirect_uri;
use crate::state::State;
use actix_session::Session;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::error::ErrorNotImplemented;
use actix_web::web;
use log::error;
use percent_encoding::NON_ALPHANUMERIC;
use percent_encoding::utf8_percent_encode;
use reqwest::header::AUTHORIZATION;
use reqwest::header::USER_AGENT;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GithubAuthQueryParams {
    redirect: Option<String>,
}

pub async fn github_auth(
    state: web::Data<State>,
    query: web::Query<GithubAuthQueryParams>,
) -> Result<impl Responder, HttpError> {
    let client_id = state
        .github_auth_client_id
        .as_ref()
        .ok_or(ErrorNotImplemented("Error code: 39"))?
        .0
        .as_str();
    let redirect_uri = format!(
        "{}?redirect={}",
        redirect_uri(&state.address),
        query.into_inner().redirect.unwrap_or(String::from("/"))
    );
    let redirect_uri = utf8_percent_encode(&redirect_uri, NON_ALPHANUMERIC);
    let scope = "user:email";
    Ok(web::Redirect::to(format!(
        "{GITHUB_AUTH_AUTHZ_ENDPOINT}?client_id={client_id}&redirect_uri={redirect_uri}&scope={scope}",
    )))
}

#[derive(Deserialize)]
pub struct GithubAuthCallbackQueryParams {
    redirect: String,
    code: String,
}

#[derive(Deserialize, Debug)]
struct GithubAuthTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GithubAuthUserResponse {
    login: String,
    id: i64,
    name: Option<String>,
}

pub async fn github_auth_callback(
    state: web::Data<State>,
    session: Session,
    query: web::Query<GithubAuthCallbackQueryParams>,
) -> Result<impl Responder, HttpError> {
    let GithubAuthClientId(client_id) = state
        .github_auth_client_id
        .as_ref()
        .ok_or(ErrorNotImplemented("Error code: 40"))?;
    let GithubAuthClientSecret(client_secret) = state
        .github_auth_client_secret
        .as_ref()
        .ok_or(ErrorNotImplemented("Error code: 41"))?;
    let GithubAuthCallbackQueryParams { redirect, code } = query.into_inner();
    let response = reqwest::Client::new()
        .post(GITHUB_AUTH_TOKEN_ENDPOINT)
        .form(&[
            ("code", &code),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ])
        .header("Accept", "application/json")
        .send()
        .await
        .inspect_err(|e| error!("{e}"))
        .map_err(|_| ErrorInternalServerError("Failed to reach GitHub token endpoint."))?;
    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        error!("GitHub token endpoint failed: {}", error_text);
        return Err(ErrorInternalServerError("GitHub token endpoint failed."));
    }
    let token_response = response
        .json::<GithubAuthTokenResponse>()
        .await
        .inspect_err(|e| error!("{e}"))
        .map_err(|_| ErrorInternalServerError("Failed to parse GitHub token response."))?;
    let bearer_token = format!("Bearer {}", token_response.access_token);
    let response = reqwest::Client::new()
        .get(GITHUB_AUTH_USER_ENDPOINT)
        .header(AUTHORIZATION, bearer_token)
        .header(USER_AGENT, "solvenp.now")
        .send()
        .await
        .map_err(|_| ErrorInternalServerError("Failed to fetch user data with access token."))?;
    let user_response = response
        .json::<GithubAuthUserResponse>()
        .await
        .inspect_err(|e| error!("{e}"))
        .map_err(|_| ErrorInternalServerError("Failed to parse GitHub user response."))?;
    let claims = IdTokenClaims {
        sub: user_response.id.to_string(),
        name: user_response.name.unwrap_or(user_response.login),
    };
    let session_id = state.session_registry.create_session(claims);
    session.insert("session_id", session_id)?;
    Ok(web::Redirect::to(redirect))
}
