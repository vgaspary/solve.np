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
use auth::gauth::GoogleAuthClientId;
use auth::gauth::GoogleAuthClientSecret;
use auth::github_auth::GithubAuthClientId;
use auth::github_auth::GithubAuthClientSecret;
use auth::session_registry::SessionRegistry;
use auth::session_registry::save_to_file;
use base64::prelude::*;
use clap::Parser;
use frontend::handlers as frontend_handlers;
use log::debug;
use log::error;
use log::info;
use ranking::update_ranking;
use rusqlite::Connection;
use rustls::ServerConfig;
use rustls_pemfile::certs;
use rustls_pemfile::private_key;
use state::LazyStaticData;
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

    /// Address.
    #[arg(long, default_value = "http://127.0.0.1:3333")]
    address: String,

    /// Port.
    #[arg(long, default_value_t = 3333)]
    port: u16,

    /// Ranking update period.
    #[arg(long, value_parser = humantime::parse_duration, default_value = "5m")]
    rerank_period: Duration,

    /// Session store.
    #[arg(long, default_value = "sessions.yaml")]
    session_store: PathBuf,

    /// Session cleanup and saving period.
    #[arg(long, value_parser = humantime::parse_duration, default_value = "10s")]
    session_cleanup_period: Duration,

    #[command(flatten)]
    auth_args: AuthArgs,

    #[command(flatten)]
    tls_args: TlsArgs,
}

#[derive(clap::Args)]
#[group(id = "auth", required = false, multiple = true)]
struct AuthArgs {
    /// Google Auth Client ID.
    #[arg(long, requires = "auth")]
    google_auth_client_id: Option<String>,

    /// Google Auth Client secret.
    #[arg(long, requires = "auth")]
    google_auth_client_secret: Option<String>,

    /// GitHub Auth Client ID.
    #[arg(long, requires = "auth")]
    github_auth_client_id: Option<String>,

    /// GitHub Auth Client secret.
    #[arg(long, requires = "auth")]
    github_auth_client_secret: Option<String>,
}

#[derive(clap::Args)]
#[group(id = "tls", required = false, multiple = true)]
struct TlsArgs {
    /// Cert file path.
    #[arg(long, requires = "tls")]
    tls_cert: Option<PathBuf>,

    /// Key file path.
    #[arg(long, requires = "tls")]
    tls_key: Option<PathBuf>,
}

impl TryFrom<&Args> for State {
    type Error = std::io::Error;

    fn try_from(args: &Args) -> Result<Self, Self::Error> {
        let start_time = SystemTime::now();
        let static_data: StaticData = serde_yaml::from_reader::<_, LazyStaticData>(BufReader::new(
            File::open(&args.static_data_path)?,
        ))
        .map_err(std::io::Error::other)?
        .into();
        let db = Mutex::new(Connection::open(&args.db_path).map_err(std::io::Error::other)?);
        let address = args.address.clone();
        let session_registry = SessionRegistry::load_from_file_or_default(&args.session_store);
        let google_auth_client_id = args
            .auth_args
            .google_auth_client_id
            .clone()
            .map(GoogleAuthClientId);
        let google_auth_client_secret = args
            .auth_args
            .google_auth_client_secret
            .clone()
            .map(GoogleAuthClientSecret);
        let github_auth_client_id = args
            .auth_args
            .github_auth_client_id
            .clone()
            .map(GithubAuthClientId);
        let github_auth_client_secret = args
            .auth_args
            .github_auth_client_secret
            .clone()
            .map(GithubAuthClientSecret);
        Ok(Self {
            start_time,
            static_data,
            db,
            address,
            session_registry,
            google_auth_client_id,
            google_auth_client_secret,
            github_auth_client_id,
            github_auth_client_secret,
        })
    }
}

impl TryFrom<&TlsArgs> for ServerConfig {
    type Error = std::io::Error;
    fn try_from(tls_args: &TlsArgs) -> std::io::Result<ServerConfig> {
        let mut key_file = BufReader::new(File::open(
            tls_args.tls_key.as_ref().expect("Invalid args"),
        )?);
        let pkey = private_key(&mut key_file)?.expect("Private key");
        let mut cert_file = BufReader::new(File::open(
            tls_args.tls_cert.as_ref().expect("Invalid args"),
        )?);
        let cert_chain = certs(&mut cert_file).collect::<Result<_, _>>()?;
        let tls_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, pkey)
            .inspect_err(|e| error!("{e}"))
            .map_err(|_| std::io::ErrorKind::Other)?;
        Ok(tls_config)
    }
}

fn start_background_tasks(args: &Args, state: Arc<State>) {
    let rerank_period = args.rerank_period;
    let session_cleanup_period = args.session_cleanup_period;
    let session_store = args.session_store.clone();
    {
        let state = Arc::clone(&state);
        task::spawn(async move {
            loop {
                sleep(rerank_period).await;
                let _ = update_ranking(state.as_ref()).inspect_err(|err| error!("{err}"));
            }
        });
    }
    {
        let state = Arc::clone(&state);
        task::spawn(async move {
            let mut sessions = state.session_registry.valid_sessions();
            loop {
                sleep(session_cleanup_period).await;
                let new_sessions = state.session_registry.valid_sessions();
                if sessions == new_sessions {
                    debug!("Sessions have not changed.");
                    continue;
                }
                sessions = new_sessions;
                info!("Saving new session set");
                let _ = save_to_file(&session_store, &sessions).inspect_err(|err| error!("{err}"));
            }
        });
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));
    let args = Args::parse();
    let state = Data::new(State::try_from(&args)?);
    let key = BASE64_STANDARD
        .decode(&args.key)
        .expect("Secret key must be base64 encoded.");
    start_background_tasks(&args, Arc::clone(&state));
    let server = HttpServer::new(move || {
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
                    .route("/github_auth", get().to(api_handlers::github_auth))
                    .route(
                        "/github_auth_callback",
                        get().to(api_handlers::github_auth_callback),
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
                .session_lifecycle(
                    PersistentSession::default()
                        .session_ttl(actix_web::cookie::time::Duration::days(90)),
                )
                .build(),
            )
    });
    let server = if args.tls_args.tls_cert.is_none() {
        server.bind(("0.0.0.0", args.port))?
    } else {
        server.bind_rustls_0_23(
            ("0.0.0.0", args.port),
            ServerConfig::try_from(&args.tls_args)?,
        )?
    };
    server.run().await
}
