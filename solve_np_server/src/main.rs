use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    error::Error as HttpError,
    error::ErrorInternalServerError,
    get,
    web::{self, Data},
};
use serde::Serialize;
use std::{collections::HashMap, time::SystemTime};

struct State {
    start_time: SystemTime,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ProblemId(pub String);

#[derive(Serialize, Clone, Debug)]
struct InputId(pub String);

#[get("/health_check")]
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

#[get("/problems")]
async fn list_problems() -> impl Responder {
    #[derive(Serialize)]
    struct Problem {
        id: ProblemId,
        name: String,
        description: String,
    }

    #[derive(Serialize)]
    struct ListProblemsResponse {
        problems: Vec<Problem>,
    }

    let result = ListProblemsResponse {
        problems: vec![
            Problem {
                id: ProblemId("travelling_salesman".into()),
                name: String::from("Travelling Salesman"),
                description: String::from(
                    "Find the shortest path in the graph that starts and ends in a specific vertex and visits **all** other vertices **exactly once**. This kind of a path is also known as a Hamiltonian cycle.",
                ),
            },
            Problem {
                id: ProblemId("knapsack".into()),
                name: String::from("Knapsack"),
                description: String::from(
                    "Given a set of items, each with a weight and a value, determine the number of each item to include in a collection so that the total weight is less than or equal to a given limit and the total value is as large as possible.",
                ),
            },
        ],
    };

    HttpResponse::Ok().json(result)
}

#[get("/problem/{problem_id}")]
async fn get_problem_details(path: web::Path<String>) -> Result<impl Responder, HttpError> {
    #[derive(Serialize)]
    struct ProblemDetails {
        id: ProblemId,
        name: String,
        problem_statement: String,
        inputs: Vec<InputId>,
    }

    let problem_repository = HashMap::from([
        (
            ProblemId("travelling_salesman".into()),
            ProblemDetails {
                id: ProblemId("travelling_salesman".into()),
                name: String::from("Travelling Salesman"),
                problem_statement: String::from(
                    "problems/travelling_salesman/problem_statement.md",
                ),
                inputs: vec![
                    InputId("problems/travelling_salesman/inputs/sample.txt".into()),
                    InputId("problems/travelling_salesman/inputs/small_sparse.txt".into()),
                    InputId("problems/travelling_salesman/inputs/small_dense.txt".into()),
                    InputId("problems/travelling_salesman/inputs/small_full.txt".into()),
                    InputId("problems/travelling_salesman/inputs/medium_sparse.txt".into()),
                    InputId("problems/travelling_salesman/inputs/medium_dense.txt".into()),
                    InputId("problems/travelling_salesman/inputs/medium_full.txt".into()),
                    InputId("problems/travelling_salesman/inputs/large_sparse.txt".into()),
                    InputId("problems/travelling_salesman/inputs/large_dense.txt".into()),
                    InputId("problems/travelling_salesman/inputs/large_full.txt".into()),
                ],
            },
        ),
        (
            ProblemId("knapsack".into()),
            ProblemDetails {
                id: ProblemId("knapsack".into()),
                name: String::from("Knapsack"),
                problem_statement: String::from("problems/knapsack/problem_statement.md"),
                inputs: vec![
                    InputId("problems/knapsack/inputs/sample.txt".into()),
                    InputId("problems/knapsack/inputs/small.txt".into()),
                    InputId("problems/knapsack/inputs/medium.txt".into()),
                    InputId("problems/knapsack/inputs/large.txt".into()),
                ],
            },
        ),
    ]);

    let problem_id = ProblemId(path.into_inner());
    problem_repository
        .get(&problem_id)
        .map(|problem_details| HttpResponse::Ok().json(problem_details))
        .ok_or_else(|| ErrorInternalServerError("No problem found with ID"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = Data::new(State {
        start_time: SystemTime::now(),
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
