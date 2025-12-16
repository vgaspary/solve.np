mod api;
mod args;
mod auth;
mod frontend;
mod mailer;
mod ranking;
mod state;
mod types;
mod validate;

use actix_files::Files;
use actix_session::SessionExt;
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
use actix_web::web::put;
use actix_web::web::scope;
use api::handlers as api_handlers;
use args::Args;
use auth::SessionId;
use auth::session_registry::save_to_file;
use base64::prelude::*;
use clap::Parser;
use frontend::handlers as frontend_handlers;
use frontend::public_files::robots_txt;
use frontend::public_files::sitemap_xml;
use log::debug;
use log::error;
use log::info;
use ranking::update_ranking;
use rustls::ServerConfig;
use state::State;
use std::sync::Arc;
use tokio::task;
use tokio::time::sleep;
use validate::untested_submissions;
use validate::validate;

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
    {
        let state = Arc::clone(&state);
        task::spawn(async move {
            let submission_ids = untested_submissions(Arc::clone(&state))
                .inspect_err(|err| error!("{err}"))
                .unwrap_or_default();
            if submission_ids.is_empty() {
                return;
            }
            info!("Re-validating {} submissions.", submission_ids.len());
            submission_ids.into_iter().for_each(|submission_id| {
                let _ =
                    validate(submission_id, Arc::clone(&state)).inspect_err(|err| error!("{err}"));
            });
        });
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
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
            .route("/robots.txt", get().to(robots_txt))
            .route("/sitemap.xml", get().to(sitemap_xml))
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
            .route("/feedback", get().to(frontend_handlers::feedback))
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
                    .route(
                        "/solutions/{solution_id}",
                        get().to(api_handlers::get_solution),
                    )
                    .route(
                        "/solutions/{solution_id}",
                        put().to(api_handlers::update_solution),
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
                    )
                    .route("/feedback", post().to(api_handlers::send_feedback)),
            )
            /* ----------------- */
            .route("/health_check", get().to(api_handlers::health_check))
            .wrap(
                actix_web::middleware::Logger::new(
                    r#"%{session_id}xi %a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
                )
                .custom_request_replace("session_id", |req| {
                    req.get_session()
                        .get::<SessionId>("session_id")
                        .ok()
                        .flatten()
                        .map(SessionId::short)
                        .map(|id| id.to_string())
                        .unwrap_or_default()
                }),
            )
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
