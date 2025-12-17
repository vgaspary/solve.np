use crate::frontend::handlers::head::HeadTemplate;
use crate::frontend::handlers::header::HeaderTemplate;
use crate::frontend::handlers::header::Page;
use crate::state::State;
use crate::types::InputId;
use crate::types::ProblemId;
use crate::types::Score;
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
use std::collections::BinaryHeap;

struct Ranked<T>(pub usize, pub T);

impl<T> PartialOrd for Ranked<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl<T> Ord for Ranked<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<T> PartialEq for Ranked<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T> Eq for Ranked<T> {}

struct Solution {
    user_name: String,
    title: String,
    description: Option<String>,
}

struct Submission {
    user_name: String,
    score: Score,
    solution_title: String,
    solution_description: Option<String>,
}

struct Input {
    filename: String,
    ranking: Vec<Ranked<Submission>>,
}

struct Problem {
    id: ProblemId,
    name: String,
    overall_ranking: Vec<Ranked<Solution>>,
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
    let mut overall_rankings = state
        .db
        .lock()
        .unwrap()
        .prepare(
            "SELECT
                user_name,
                problem_id,
                title,
                description,
                ranking
            FROM
                solutions
            WHERE
                ranking IS NOT NULL",
        )
        .inspect_err(|err| error!("{err}"))
        .map_err(|_| ErrorInternalServerError("Error code: 28"))?
        .query_map(params![], |row| {
            let problem_id = row.get(1)?;
            let rank = row.get(4)?;
            let solution = Solution {
                user_name: row.get(0)?,
                title: row.get(2)?,
                description: row.get(3)?,
            };
            Ok((problem_id, Ranked(rank, solution)))
        })
        .map_err(|_| ErrorInternalServerError("Error code: 29"))?
        .filter_map(Result::ok)
        .into_group_map::<ProblemId, Ranked<Solution>>();
    let mut rankings_per_problem_input = state
        .db
        .lock()
        .unwrap()
        .prepare(
            "SELECT
                submissions.user_name,
                submissions.problem_id,
                submissions.input_id,
                submissions.status,
                submissions.ranking,
                solutions.title,
                solutions.description
            FROM
                submissions
            LEFT JOIN
                solutions ON submissions.solution_id = solutions.solution_id
            WHERE
                submissions.ranking IS NOT NULL",
        )
        .inspect_err(|err| error!("{err}"))
        .map_err(|_| ErrorInternalServerError("Error code: 30"))?
        .query_map(params![], |row| {
            let problem_id = row.get(1)?;
            let input_id = row.get(2)?;
            let rank = row.get(4)?;
            let submission = Submission {
                user_name: row.get(0)?,
                score: row.get(3)?,
                solution_title: row.get(5)?,
                solution_description: row.get(6)?,
            };
            Ok(((problem_id, input_id), Ranked(rank, submission)))
        })
        .map_err(|_| ErrorInternalServerError("Error code: 31"))?
        .filter_map(Result::ok)
        .into_group_map::<(ProblemId, InputId), Ranked<Submission>>();
    let problem_input_iter = state
        .static_data
        .problems
        .iter()
        .map(|problem| {
            problem
                .inputs
                .iter()
                .map(|input| (problem.id.clone(), input.id.clone(), input.filename.clone()))
        })
        .flatten();
    let problems = state
        .static_data
        .problems
        .iter()
        .map(|problem| {
            let overall_ranking = match overall_rankings.remove(&problem.id) {
                Some(solutions) => solutions
                    .into_iter()
                    .fold(BinaryHeap::with_capacity(10), |mut heap, solution| {
                        heap.push(solution);
                        if heap.len() > 10 {
                            heap.pop();
                        }
                        heap
                    })
                    .into_sorted_vec(),
                None => vec![],
            };
            let inputs = problem_input_iter
                .clone()
                .filter(|(problem_id, _, _)| problem_id == &problem.id)
                .map(|(problem_id, input_id, filename)| {
                    let ranking = match rankings_per_problem_input.remove(&(problem_id, input_id)) {
                        Some(submissions) => submissions
                            .into_iter()
                            .fold(BinaryHeap::with_capacity(10), |mut heap, submission| {
                                heap.push(submission);
                                if heap.len() > 10 {
                                    heap.pop();
                                }
                                heap
                            })
                            .into_sorted_vec(),
                        None => vec![],
                    };
                    Input { filename, ranking }
                })
                .collect();
            Problem {
                id: problem.id.clone(),
                name: problem.name.clone(),
                overall_ranking,
                inputs,
            }
        })
        .collect();
    let body = LeaderboardTemplate {
        head: HeadTemplate::with_title("Leaderboard - solveNP"),
        header: HeaderTemplate::try_from(&session, state)?.with_active(Page::Leaderboard),
        problems,
    }
    .render()
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}
