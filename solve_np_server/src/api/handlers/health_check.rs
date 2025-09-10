use crate::state::State;
use actix_web::{
    error::Error as HttpError, error::ErrorInternalServerError, web, HttpResponse, Responder,
};
use std::time::SystemTime;

#[derive(serde::Serialize)]
pub struct HealthCheckReport {
    uptime: String,
}

pub async fn health_check(state: web::Data<State>) -> Result<impl Responder, HttpError> {
    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .map_err(ErrorInternalServerError)?;
    let response = HealthCheckReport {
        uptime: format!("{}s", uptime.as_secs()),
    };
    Ok(HttpResponse::Ok().json(response))
}