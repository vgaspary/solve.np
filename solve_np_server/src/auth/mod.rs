pub mod gauth;
pub mod session_registry;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionId(u128);

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
