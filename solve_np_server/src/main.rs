mod api;
mod frontend;
mod state;

use actix_files::Files;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{
    App, HttpServer,
    web::{Data, get, post, scope},
};
use api::handlers::{get_problem_details, health_check, list_problems, log_in, log_out};
use base64::prelude::*;
use clap::Parser;
use frontend::handlers::{home_page, log_in as log_in_page, p_vs_np, problem_statement, problems};
use state::{State, StaticData};
use std::{fs::File, io::BufReader, net::Ipv4Addr, path::PathBuf, time::SystemTime};

#[derive(Parser)]
struct Args {
    /// Static data path.
    #[arg(short, long)]
    static_data_path: PathBuf,

    /// Server address.
    #[arg(short, long)]
    address: Ipv4Addr,

    /// Session cookie key.
    #[arg(short, long)]
    key: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let static_data: StaticData =
        serde_yaml::from_reader(BufReader::new(File::open(args.static_data_path)?))
            .map_err(std::io::Error::other)?;
    let start_time = SystemTime::now();
    let state = Data::new(State {
        start_time,
        address: args.address,
        static_data,
    });
    let secret_key = actix_web::cookie::Key::from(
        &BASE64_STANDARD
            .decode(args.key)
            .expect("Secret key must be base64 encoded."),
    );
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            /* --- Frontend scope --- */
            .service(Files::new("/static", "./static").show_files_listing())
            .route("/", get().to(home_page))
            .route("/log_in", get().to(log_in_page))
            .route("/p_vs_np", get().to(p_vs_np))
            .service(
                scope("/problems")
                    .service(Files::new("/assets", "./problems_assets").show_files_listing())
                    .route("", get().to(problems))
                    .route("/{problem_id}", get().to(problem_statement)),
            )
            /* ---------------------- */
            /* --- API scope --- */
            .service(
                scope("/api")
                    .route("/problems", get().to(list_problems))
                    .route("/problems/{problem_id}", get().to(get_problem_details))
                    .route("/log_in", post().to(log_in))
                    .route("/log_out", get().to(log_out)),
            )
            /* ----------------- */
            .route("/health_check", get().to(health_check))
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
    .bind(("0.0.0.0", 6520))?
    .run()
    .await
}
