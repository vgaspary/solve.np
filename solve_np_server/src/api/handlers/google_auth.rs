use crate::state::GAuthClientId;
use crate::state::GAuthClientSecret;
use crate::state::State;
use actix_session::Session;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::error::ErrorNotImplemented;
use actix_web::error::ErrorUnauthorized;
use actix_web::web;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::decode;
use jsonwebtoken::decode_header;
use log::error;
use percent_encoding::NON_ALPHANUMERIC;
use percent_encoding::utf8_percent_encode;
use serde::Deserialize;
use tokio::sync::OnceCell;

#[derive(Deserialize, Debug)]
pub struct GoogleAuthParams {
    code: String,
}

#[derive(Deserialize, Debug)]
struct GoogleAuthTokenResponse {
    id_token: String,
}

#[derive(Debug, Deserialize, Clone)]
struct GooglePublicKey {
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize, Clone)]
struct GooglePublicKeys {
    keys: Vec<GooglePublicKey>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub sub: String,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
}

const GAUTH_AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GAUTH_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const GAUTH_CERTS_ENDPOINT: &str = "https://www.googleapis.com/oauth2/v3/certs";
const GAUTH_ISSUER: &str = "https://accounts.google.com";
const GAUTH_REDIRECT_URI: &str = "http://127.0.0.1:3333/api/log_in/google/callback";
const GAUTH_SCOPE: &str = "https://www.googleapis.com/auth/userinfo.profile";

static KEYS: OnceCell<GooglePublicKeys> = OnceCell::const_new();

async fn get_google_public_keys() -> Result<&'static GooglePublicKeys, HttpError> {
    KEYS.get_or_try_init(|| async {
        let response = reqwest::get(GAUTH_CERTS_ENDPOINT).await.map_err(|e| {
            log::error!("Failed to fetch Google public keys: {}", e);
            ErrorInternalServerError("Failed to fetch Google public keys")
        })?;
        response.json::<GooglePublicKeys>().await.map_err(|e| {
            log::error!("Failed to parse Google public keys: {}", e);
            ErrorInternalServerError("Failed to parse Google public keys")
        })
    })
    .await
}

async fn validate_id_token(id_token: &str, client_id: &str) -> Result<IdTokenClaims, HttpError> {
    let kid = decode_header(id_token)
        .map_err(|_| ErrorUnauthorized("Invalid token header"))?
        .kid
        .ok_or_else(|| ErrorUnauthorized("Missing 'kid' in token header"))?;
    let keys = get_google_public_keys().await?;
    let key = keys
        .keys
        .iter()
        .find(|k| k.kid == kid)
        .ok_or_else(|| ErrorUnauthorized("No matching public key found for 'kid'"))?;
    let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
        .map_err(|_| ErrorInternalServerError("Failed to create decoding key"))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[client_id]);
    validation.set_issuer(&[GAUTH_ISSUER]);
    let token_data =
        decode::<IdTokenClaims>(id_token, &decoding_key, &validation).map_err(ErrorUnauthorized)?;
    Ok(token_data.claims)
}

pub async fn log_in_via_google(state: web::Data<State>) -> Result<impl Responder, HttpError> {
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
    let Some((GAuthClientId(client_id), GAuthClientSecret(client_secret))) =
        &state.gauth_client_id_secret
    else {
        return Err(ErrorNotImplemented("Internal error #35"));
    };
    let response = reqwest::Client::new()
        .post(GAUTH_TOKEN_ENDPOINT)
        .form(&[
            ("code", query.code.as_str()),
            ("client_id", client_id),
            ("client_secret", client_secret),
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
    let claims = validate_id_token(&token_response.id_token, client_id).await?;
    session.insert("user_id", &claims.sub)?;
    session.insert("username", &claims.name)?;
    // TODO! redirect to home, p_vs_np, or some problem page.
    Ok(web::Redirect::to("/"))
}
