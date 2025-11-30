mod api;
mod auth;
mod frontend;
mod ranking;
mod state;
mod types;

use actix_files::Files;
use actix_session::SessionMiddleware;
use actix_session::config::PersistentSession;
use actix_session::storage::CookieSessionStore;
use actix_web::App;
use actix_web::HttpServer;
use actix_web::http::header::DispositionType;
use actix_web::web::Data;
use actix_web::web::delete;
use actix_web::web::get;
use actix_web::web::post;
use actix_web::web::scope;
use api::handlers as api_handlers;
use auth::gauth::GAuthClientId;
use auth::gauth::GAuthClientSecret;
use base64::prelude::*;
use clap::Parser;
use frontend::handlers as frontend_handlers;
use log::error;
use ranking::update_ranking;
use rusqlite::Connection;
use state::State;
use state::StaticData;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;
use tokio::task;
use tokio::time::sleep;

use crate::auth::session_registry::SessionRegistry;

#[derive(Parser)]
struct Args {
    /// Static data path.
    #[arg(long, default_value = "static_data.yaml")]
    static_data_path: PathBuf,

    /// Session cookie key.
    #[arg(long)]
    key: String,

    /// DB path.
    #[arg(long, default_value = "solve_np.db3")]
    db_path: PathBuf,

    /// Port.
    #[arg(long, default_value_t = 3333)]
    port: u16,

    /// Ranking update period.
    #[arg(long, value_parser = humantime::parse_duration, default_value = "5m")]
    rerank_period: Duration,

    /// Google Auth Client ID.
    #[arg(long)]
    gauth_client_id: Option<String>,

    /// Google Auth Client secret.
    #[arg(long)]
    gauth_client_secret: Option<String>,
}

impl TryFrom<&Args> for State {
    type Error = std::io::Error;

    fn try_from(args: &Args) -> Result<Self, Self::Error> {
        let start_time = SystemTime::now();
        let static_data: StaticData =
            serde_yaml::from_reader(BufReader::new(File::open(&args.static_data_path)?))
                .map_err(std::io::Error::other)?;
        let db = Mutex::new(Connection::open(&args.db_path).map_err(std::io::Error::other)?);
        let session_registry = SessionRegistry::default();
        let gauth_client_id_secret = match (&args.gauth_client_id, &args.gauth_client_secret) {
            (Some(client_id), Some(client_secret)) => Some((
                GAuthClientId(client_id.clone()),
                GAuthClientSecret(client_secret.clone()),
            )),
            _ => None,
        };
        Ok(Self {
            start_time,
            static_data,
            db,
            session_registry,
            gauth_client_id_secret,
        })
    }
}

fn start_background_tasks(rerank_period: Duration, state: Arc<State>) {
    task::spawn(async move {
        loop {
            sleep(rerank_period).await;
            let _ = update_ranking(state.as_ref()).inspect_err(|err| error!("{err}"));
        }
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));
    let args = Args::parse();
    let state = Data::new(State::try_from(&args)?);
    let key = BASE64_STANDARD
        .decode(args.key)
        .expect("Secret key must be base64 encoded.");
    start_background_tasks(args.rerank_period, Arc::clone(&state));
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            /* --- Frontend scope --- */
            .service(Files::new("/static", "./static"))
            .service(Files::new("/js/dist", "./js/dist"))
            .route("/", get().to(frontend_handlers::home_page))
            .route("/log_in", get().to(frontend_handlers::log_in))
            .route("/leaderboard", get().to(frontend_handlers::leaderboard))
            .route("/p_vs_np", get().to(frontend_handlers::p_vs_np))
            .route("/problems", get().to(frontend_handlers::problems))
            .route(
                "/problems/{problem_id}",
                get().to(frontend_handlers::problem_statement),
            )
            .route("/solutions", get().to(frontend_handlers::solutions))
            .route(
                "/solutions/{solution_id}",
                get().to(frontend_handlers::solution),
            )
            .route(
                "/solution_table/{problem_id}",
                get().to(frontend_handlers::solution_table),
            )
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
                    .route("/google_auth", get().to(api_handlers::google_auth))
                    .route(
                        "/google_auth_callback",
                        get().to(api_handlers::google_auth_callback),
                    )
                    .route("/log_out", get().to(api_handlers::log_out))
                    .route(
                        "/solutions/{solution_id}",
                        delete().to(api_handlers::delete_solution),
                    )
                    .route("/solutions", post().to(api_handlers::create_solution))
                    .route(
                        "/submit_solution/{problem_id}/{solution_id}/{input_id}",
                        post().to(api_handlers::submit_solution),
                    )
                    .route(
                        "/solutions/{solution_id}/status",
                        get().to(api_handlers::solution_status),
                    )
                    .route(
                        "/solutions/{solution_id}/input/{input_id}/status",
                        get().to(api_handlers::submission_status),
                    ),
            )
            /* ----------------- */
            .route("/health_check", get().to(api_handlers::health_check))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    actix_web::cookie::Key::from(&key),
                )
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
