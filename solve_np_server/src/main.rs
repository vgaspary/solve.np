use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    body::BoxBody,
    error::Error,
    get,
    web::{self, Data},
};
use serde::Serialize;
use std::time::{Duration, SystemTime};

struct State {
    start_time: SystemTime,
}

#[get("/echo")]
async fn echo(req: String) -> impl Responder {
    HttpResponse::Ok().body(req)
}

#[get("/health_check")]
async fn health_check(state: web::Data<State>) -> Result<impl Responder, Error> {
    #[derive(Serialize)]
    struct HealthCheckReport {
        uptime: Duration,
    }

    impl Responder for HealthCheckReport {
        type Body = BoxBody;
        fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
            HttpResponse::Ok().json(self)
        }
    }

    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .map_err(|_| {
            Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Duration calculation failed",
            ))
        })?;
    Ok(HealthCheckReport { uptime })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = Data::new(State {
        start_time: SystemTime::now(),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(echo)
            .service(health_check)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
