use actix_session::Session;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
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
    session: Session,
    query: web::Query<LogInQueryParams>,
    form: web::Form<LogInForm>,
) -> Result<impl Responder, HttpError> {
    let LogInForm { username, password } = form.into_inner();
    Ok(if username == password {
        web::Redirect::to("/log_in?err=Invalid%20credentials").see_other()
    } else {
        session.insert("user_id", sha256::digest(format!("{username}{password}")))?;
        session.insert("username", username)?;
        web::Redirect::to(query.redirect.clone().unwrap_or("/".to_string())).see_other()
    })
}
