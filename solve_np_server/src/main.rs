mod api;
mod frontend;
mod state;

use actix_files::Files;
use actix_web::{
    App, HttpServer,
    web::{Data, get, scope},
};
use api::handlers::{get_problem_details, health_check, list_problems};
use clap::Parser;
use frontend::handlers::{home_page, problem_statement, problems};
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
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            /* --- Frontend scope --- */
            .service(Files::new("/static", "./static").show_files_listing())
            .route("/", get().to(home_page))
            .service(
                scope("/problems")
                    .service(
                        Files::new("/assets", "./problems_assets").show_files_listing(),
                    )
                    .route("", get().to(problems))
                    .route("/{problem_id}", get().to(problem_statement)),
            )
            /* ---------------------- */
            /* --- API scope --- */
            .service(
                scope("/api")
                    .route("/problems", get().to(list_problems))
                    .route("/problems/{problem_id}", get().to(get_problem_details)),
            )
            /* ----------------- */
            .route("/health_check", get().to(health_check))
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("0.0.0.0", 6520))?
    .run()
    .await
}
