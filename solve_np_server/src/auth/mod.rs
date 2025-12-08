pub mod gauth;
pub mod github_auth;
pub mod session_registry;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionId(u128);

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct IdTokenClaims {
    pub sub: String,
    pub name: String,
}
