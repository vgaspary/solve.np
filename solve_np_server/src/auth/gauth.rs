use crate::auth::IdTokenClaims;
use anyhow::Context;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::decode;
use jsonwebtoken::decode_header;
use serde::Deserialize;
use tokio::sync::OnceCell;

const GOOGLE_AUTH_CERTS_ENDPOINT: &str = "https://www.googleapis.com/oauth2/v3/certs";
pub const GOOGLE_AUTH_AUTHZ_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const GOOGLE_AUTH_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
pub const GOOGLE_AUTH_ISSUER: &str = "https://accounts.google.com";
pub const GOOGLE_AUTH_SCOPE: &str = "https://www.googleapis.com/auth/userinfo.profile";

pub struct GoogleAuthClientId(pub String);
pub struct GoogleAuthClientSecret(pub String);

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

static KEYS: OnceCell<GooglePublicKeys> = OnceCell::const_new();

async fn get_google_public_keys() -> anyhow::Result<&'static GooglePublicKeys> {
    KEYS.get_or_try_init(|| async {
        let response = reqwest::get(GOOGLE_AUTH_CERTS_ENDPOINT)
            .await
            .context("Failed to fetch Google public keys")?;
        response
            .json::<GooglePublicKeys>()
            .await
            .context("Failed to parse Google public keys")
    })
    .await
}

pub async fn validate_id_token(
    id_token: &str,
    client_id: &GoogleAuthClientId,
) -> anyhow::Result<IdTokenClaims> {
    let kid = decode_header(id_token)
        .context("Invalid token header")?
        .kid
        .context("Missing 'kid' in token header")?;
    let keys = get_google_public_keys().await?;
    let key = keys
        .keys
        .iter()
        .find(|k| k.kid == kid)
        .context("No matching public key found for 'kid'")?;
    let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
        .context("Failed to create decoding key")?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&client_id.0]);
    validation.set_issuer(&[GOOGLE_AUTH_ISSUER]);
    let token_data = decode::<IdTokenClaims>(id_token, &decoding_key, &validation)
        .context("Failed decoding ID token.")?;
    Ok(token_data.claims)
}

pub fn redirect_uri(address: &str) -> String {
    format!("{address}/api/google_auth_callback")
}
