use crate::frontend::handlers::head::HeadTemplate;
use crate::frontend::handlers::header::HeaderTemplate;
use crate::state::State;
use crate::types::InputId;
use crate::types::ProblemId;
use crate::types::Score;
use crate::types::SolutionId;
use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::ContentType;
use actix_web::web;
use askama::Template;
use itertools::Itertools;
use log::error;
use rusqlite::params;

type Rank = usize;
type Username = String;

struct Solution {
    id: SolutionId,
    title: String,
    description: Option<String>,
}

struct Input {
    id: InputId,
    filename: String,
    rankings: Vec<(Rank, Username, Score, Solution)>,
}

struct Problem {
    id: ProblemId,
    name: String,
    rankings: Vec<(Rank, Username, Solution)>,
    inputs: Vec<Input>,
}

#[derive(Template)]
#[template(path = "leaderboard.html")]
struct LeaderboardTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
    problems: Vec<Problem>,
}

pub async fn leaderboard(
    session: Session,
    state: web::Data<State>,
) -> Result<impl Responder, HttpError> {
    let solutions = state
        .db
        .lock()
        .unwrap()
        .prepare(
            "SELECT
                submissions.user_id,
                submissions.problem_id,
                submissions.solution_id,
                submissions.input_id,
                solutions.title,
                solutions.description
            FROM
                submissions
                LEFT JOIN solutions ON submissions.solution_id = solutions.solution_id
            WHERE
                submissions.status >= 0
                AND solutions.status >= 0",
        )
        .inspect_err(|err| error!("{err}"))
        .map_err(|_| ErrorInternalServerError("Internal error #28"))?
        .query_map(params![], |row| {
            Ok(Solution {
                id: row.get(2)?,
                title: row.get(4)?,
                description: row.get(5)?,
            })
        })
        .map_err(|_| ErrorInternalServerError("Internal error #29"))?
        .filter_map(Result::ok)
        .collect_vec();

    let problems = state
        .static_data
        .problems
        .iter()
        .map(|problem| Problem {
            id: problem.id.clone(),
            name: problem.name.clone(),
            rankings: vec![
                (
                    1,
                    "vanand".into(),
                    Solution {
                        id: "solution_id_1".into(),
                        title: "Greedy".into(),
                        description: Some("Greedy algorithm".into()),
                    },
                ),
                (
                    2,
                    "vanand".into(),
                    Solution {
                        id: "solution_id_2".into(),
                        title: "Bruteforce".into(),
                        description: None,
                    },
                ),
                (
                    1,
                    "vanand".into(),
                    Solution {
                        id: "solution_id_1".into(),
                        title: "Greedy".into(),
                        description: Some("Greedy algorithm".into()),
                    },
                ),
                (
                    2,
                    "vanand".into(),
                    Solution {
                        id: "solution_id_2".into(),
                        title: "Bruteforce".into(),
                        description: None,
                    },
                ),
                (
                    1,
                    "vanand".into(),
                    Solution {
                        id: "solution_id_1".into(),
                        title: "Greedy".into(),
                        description: Some("Greedy algorithm".into()),
                    },
                ),
                (
                    2,
                    "vanand".into(),
                    Solution {
                        id: "solution_id_2".into(),
                        title: "Bruteforce".into(),
                        description: None,
                    },
                ),
                (
                    1,
                    "vanand".into(),
                    Solution {
                        id: "solution_id_1".into(),
                        title: "Greedy".into(),
                        description: Some("Greedy algorithm".into()),
                    },
                ),
                (
                    2,
                    "vanand".into(),
                    Solution {
                        id: "solution_id_2".into(),
                        title: "Bruteforce".into(),
                        description: None,
                    },
                ),
            ],
            inputs: problem
                .inputs
                .iter()
                .map(|input| Input {
                    id: input.id.clone(),
                    filename: input.filename.clone(),
                    rankings: vec![
                        (
                            1,
                            "vanand".into(),
                            42,
                            Solution {
                                id: "solution_id_1".into(),
                                title: "Greedy".into(),
                                description: Some("Greedy algorithm".into()),
                            },
                        ),
                        (
                            2,
                            "vanand".into(),
                            46,
                            Solution {
                                id: "solution_id_2".into(),
                                title: "Bruteforce".into(),
                                description: None,
                            },
                        ),
                        (
                            2,
                            "varand".into(),
                            46,
                            Solution {
                                id: "solution_id_3".into(),
                                title: "Solution1".into(),
                                description: None,
                            },
                        ),
                        (
                            1,
                            "vanand".into(),
                            59,
                            Solution {
                                id: "solution_id_1".into(),
                                title: "Greedy".into(),
                                description: Some("Greedy algorithm".into()),
                            },
                        ),
                        (
                            2,
                            "vanand".into(),
                            59,
                            Solution {
                                id: "solution_id_2".into(),
                                title: "Bruteforce".into(),
                                description: None,
                            },
                        ),
                        (
                            1,
                            "vanand".into(),
                            89,
                            Solution {
                                id: "solution_id_1".into(),
                                title: "Greedy".into(),
                                description: Some("Greedy algorithm".into()),
                            },
                        ),
                        (
                            2,
                            "vanand".into(),
                            89,
                            Solution {
                                id: "solution_id_2".into(),
                                title: "Bruteforce".into(),
                                description: None,
                            },
                        ),
                        (
                            1,
                            "vanand".into(),
                            89,
                            Solution {
                                id: "solution_id_1".into(),
                                title: "Greedy".into(),
                                description: Some("Greedy algorithm".into()),
                            },
                        ),
                        (
                            2,
                            "vanand".into(),
                            90,
                            Solution {
                                id: "solution_id_2".into(),
                                title: "Bruteforce".into(),
                                description: None,
                            },
                        ),
                        (
                            1,
                            "vanand".into(),
                            92,
                            Solution {
                                id: "solution_id_1".into(),
                                title: "Greedy".into(),
                                description: Some("Greedy algorithm".into()),
                            },
                        ),
                        (
                            2,
                            "vanand".into(),
                            94,
                            Solution {
                                id: "solution_id_2".into(),
                                title: "Bruteforce".into(),
                                description: None,
                            },
                        ),
                    ],
                })
                .collect(),
        })
        .collect();
    let body = LeaderboardTemplate {
        head: HeadTemplate {
            title: "Leaderboard - solve.NP".into(),
        },
        header: HeaderTemplate::try_from(&session)?,
        problems,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
