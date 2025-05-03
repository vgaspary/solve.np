use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    error::Error as HttpError,
    error::ErrorInternalServerError,
    get,
    web::{self, Data},
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf, time::SystemTime};

struct State {
    start_time: SystemTime,
    static_data: StaticData,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ProblemId(pub String);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct InputId(pub String);

#[get("/api/health_check")]
async fn health_check(state: web::Data<State>) -> Result<impl Responder, HttpError> {
    #[derive(Serialize)]
    struct HealthCheckReport {
        uptime: String,
    }

    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .map_err(ErrorInternalServerError)?;
    let response = HealthCheckReport {
        uptime: format!("{}s", uptime.as_secs()),
    };
    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/problems")]
async fn list_problems(state: web::Data<State>) -> impl Responder {
    #[derive(Serialize)]
    struct Problem {
        id: ProblemId,
        name: String,
        description: String,
    }

    let result = state
        .static_data
        .0
        .iter()
        .map(|(id, problem)| Problem {
            id: id.clone(),
            name: problem.name.clone(),
            description: problem.description.clone(),
        })
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(result)
}

#[get("/api/problem/{problem_id}")]
async fn get_problem_details(
    path: web::Path<String>,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    #[derive(Serialize)]
    struct ProblemDetails {
        id: ProblemId,
        name: String,
        problem_statement: PathBuf,
        inputs: Vec<InputId>,
    }

    let problem_id = ProblemId(path.into_inner());
    state
        .static_data
        .0
        .get(&problem_id)
        .map(|problem| ProblemDetails {
            id: problem_id,
            name: problem.name.clone(),
            problem_statement: problem.problem_statement.clone(),
            inputs: problem.inputs.keys().cloned().collect(),
        })
        .map(|problem_details| HttpResponse::Ok().json(problem_details))
        .ok_or_else(|| ErrorInternalServerError("No problem found with ID"))
}

#[derive(Parser)]
struct Args {
    /// Static data path.
    #[arg(short, long)]
    static_data_path: PathBuf,
}

#[derive(Deserialize)]
struct Input {
    #[serde(alias = "path")]
    _path: PathBuf,
    #[serde(alias = "cmd")]
    _cmd: String,
}

#[derive(Deserialize)]
struct Problem {
    name: String,
    description: String,
    problem_statement: PathBuf,
    inputs: HashMap<InputId, Input>,
}

#[derive(Deserialize)]
struct StaticData(HashMap<ProblemId, Problem>);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let static_data: StaticData =
        serde_json::from_reader(BufReader::new(File::open(args.static_data_path)?))?;
    let state = Data::new(State {
        start_time: SystemTime::now(),
        static_data,
    });
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(health_check)
            .service(list_problems)
            .service(get_problem_details)
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
