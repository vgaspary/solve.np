use crate::auth::IdTokenClaims;
use anyhow::Context;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::decode;
use jsonwebtoken::decode_header;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::RwLock;

const GOOGLE_AUTH_CERTS_ENDPOINT: &str = "https://www.googleapis.com/oauth2/v3/certs";
pub const GOOGLE_AUTH_AUTHZ_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const GOOGLE_AUTH_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
pub const GOOGLE_AUTH_ISSUER: &str = "https://accounts.google.com";
pub const GOOGLE_AUTH_SCOPE: &str = "https://www.googleapis.com/auth/userinfo.profile";

pub struct GoogleAuthClientId(pub String);
pub struct GoogleAuthClientSecret(pub String);

#[derive(Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct GooglePublicKey {
    pub kid: String,
}

#[derive(Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct RsaComponents {
    pub n: String,
    pub e: String,
}

pub struct GooglePublicKeys(RwLock<HashMap<GooglePublicKey, RsaComponents>>);

impl GooglePublicKeys {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }

    async fn fetch() -> anyhow::Result<HashMap<GooglePublicKey, RsaComponents>> {
        #[derive(Deserialize)]
        struct PubKeyAndRsaComponents {
            kid: String,
            n: String,
            e: String,
        }
        #[derive(Deserialize)]
        struct Response {
            keys: Vec<PubKeyAndRsaComponents>,
        }

        let response = reqwest::get(GOOGLE_AUTH_CERTS_ENDPOINT)
            .await
            .context("Failed to fetch Google public keys")?;
        let keys_response: Response = response
            .json()
            .await
            .context("Failed to parse Google public keys")?;
        Ok(keys_response
            .keys
            .into_iter()
            .map(|PubKeyAndRsaComponents { kid, n, e }| {
                (GooglePublicKey { kid }, RsaComponents { n, e })
            })
            .collect())
    }

    pub async fn validate_id_token(
        &self,
        id_token: &str,
        client_id: &GoogleAuthClientId,
    ) -> anyhow::Result<IdTokenClaims> {
        let kid = decode_header(id_token)
            .context("Invalid token header")?
            .kid
            .context("Missing 'kid' in token header")?;
        let kid = GooglePublicKey { kid };

        let RsaComponents { n, e } = {
            let keys_read_guard = self.0.read().await;
            if let Some(rsa_components) = keys_read_guard.get(&kid) {
                rsa_components.clone()
            } else {
                drop(keys_read_guard);
                let mut keys_write_guard = self.0.write().await;

                if let Some(rsa_components) = keys_write_guard.get(&kid) {
                    rsa_components.clone()
                } else {
                    *keys_write_guard = Self::fetch().await?;
                    keys_write_guard
                        .get(&kid)
                        .context("No matching public key found for 'kid' even after refresh")?
                        .clone()
                }
            }
        };

        let decoding_key =
            DecodingKey::from_rsa_components(&n, &e).context("Failed to create decoding key")?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&client_id.0]);
        validation.set_issuer(&[GOOGLE_AUTH_ISSUER]);
        let token_data = decode::<IdTokenClaims>(id_token, &decoding_key, &validation)
            .context("Failed decoding ID token.")?;
        Ok(token_data.claims)
    }
}

pub fn redirect_uri(address: &str) -> String {
    format!("{address}/api/google_auth_callback")
}
