mod api;
mod frontend;
mod state;

use actix_files::Files;
use actix_web::{App, HttpServer, web::Data};
use api::handlers::{get_problem_details, health_check, list_problems};
use clap::Parser;
use frontend::handlers::{home_page, problems};
use state::{State, StaticData};
use std::{fs::File, io::BufReader, path::PathBuf, time::SystemTime};

#[derive(Parser)]
struct Args {
    /// Static data path.
    #[arg(short, long)]
    static_data_path: PathBuf,
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
        static_data,
    });
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            /* --- Frontend scope --- */
            .service(Files::new("/static", "./static").show_files_listing())
            .service(home_page)
            .service(problems)
            /* ---------------------- */
            /* --- API scope --- */
            .service(health_check)
            .service(list_problems)
            .service(get_problem_details)
            /* ----------------- */
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("localhost", 6520))?
    .run()
    .await
}
