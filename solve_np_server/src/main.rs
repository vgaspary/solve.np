mod api;
mod frontend;
mod state;

use actix_files::Files;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{
    App, HttpServer,
    http::header::DispositionType,
    web::{Data, delete, get, post, scope},
};
use api::handlers as api_handlers;
use base64::prelude::*;
use clap::Parser;
use frontend::handlers as frontend_handlers;
use rusqlite::Connection;
use state::{State, StaticData};
use std::{fs::File, io::BufReader, path::PathBuf, sync::Mutex, time::SystemTime};

#[derive(Parser)]
struct Args {
    /// Static data path.
    #[arg(short, long)]
    static_data_path: PathBuf,

    /// Session cookie key.
    #[arg(short, long)]
    key: String,

    /// DB path.
    #[arg(short, long)]
    db_path: PathBuf,

    /// Port
    #[arg(short, long)]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let start_time = SystemTime::now();
    let static_data: StaticData =
        serde_yaml::from_reader(BufReader::new(File::open(args.static_data_path)?))
            .map_err(std::io::Error::other)?;
    let db = Mutex::new(Connection::open(args.db_path).map_err(std::io::Error::other)?);
    let state = Data::new(State {
        start_time,
        static_data,
        db,
    });
    let secret_key = actix_web::cookie::Key::from(
        &BASE64_STANDARD
            .decode(args.key)
            .expect("Secret key must be base64 encoded."),
    );
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            /* --- Frontend scope --- */
            .service(Files::new("/static", "./static"))
            .service(Files::new("/js_utils", "./js_utils"))
            .route("/", get().to(frontend_handlers::home_page))
            .route("/log_in", get().to(frontend_handlers::log_in))
            .route("/p_vs_np", get().to(frontend_handlers::p_vs_np))
            .route("/problems", get().to(frontend_handlers::problems))
            .route(
                "/problems/{problem_id}",
                get().to(frontend_handlers::problem_statement),
            )
            .route("/solutions", get().to(frontend_handlers::solutions))
            /* ---------------------- */
            /* --- API scope --- */
            .service(
                scope("/api")
                    .service(
                        Files::new("/problems", "../problems")
                            .mime_override(|_| DispositionType::Attachment),
                    )
                    .route("/validate_sample", post().to(api_handlers::validate_sample))
                    .route("/vote/{vote}", post().to(api_handlers::vote))
                    .route("/log_in", post().to(api_handlers::log_in))
                    .route("/log_out", get().to(api_handlers::log_out))
                    .route(
                        "/solutions/{problem_id}/{solution_id}",
                        delete().to(api_handlers::delete_solution),
                    )
                    .route("/solutions", post().to(api_handlers::create_solution)),
            )
            /* ----------------- */
            .route("/health_check", get().to(api_handlers::health_check))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    .session_lifecycle(
                        PersistentSession::default()
                            .session_ttl(actix_web::cookie::time::Duration::days(5)),
                    )
                    .build(),
            )
    })
    .bind(("0.0.0.0", args.port))?
    .run()
    .await
}
