pub struct GithubAuthClientId(pub String);
pub struct GithubAuthClientSecret(pub String);

pub const GITHUB_AUTH_AUTHZ_ENDPOINT: &str = "https://github.com/login/oauth/authorize";
pub const GITHUB_AUTH_TOKEN_ENDPOINT: &str = "https://github.com/login/oauth/access_token";
pub const GITHUB_AUTH_USER_ENDPOINT: &str = "https://api.github.com/user";

pub fn redirect_uri(address: &str) -> String {
    format!("{address}/api/github_auth_callback")
}
