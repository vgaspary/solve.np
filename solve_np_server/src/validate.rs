use crate::state::State;
use crate::types::InputId;
use crate::types::ProblemId;
use crate::types::SolutionId;
use crate::types::Status;
use anyhow::Context;
use anyhow::bail;
use itertools::Itertools;
use rusqlite::params;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

type UserId = String;
type SubmissionId = (UserId, ProblemId, SolutionId, InputId);

pub type Score = usize;

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Solution {
    Tsp(Vec<usize>),
    GraphColoring(Vec<usize>),
    VertexCover(Vec<usize>),
    MaximumClique(Vec<usize>),
    MaximumCut(Vec<usize>),
    SteinerTree(Vec<(usize, usize)>),
    Knapsack(Vec<usize>),
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tsp(path) => write!(f, "{}", path.iter().join(" ")),
            Self::GraphColoring(colors) => write!(f, "{}", colors.iter().join(" ")),
            Self::VertexCover(vertices) => write!(f, "{}", vertices.iter().join(" ")),
            Self::MaximumClique(clique) => write!(f, "{}", clique.iter().join(" ")),
            Self::MaximumCut(vertices) => write!(f, "{}", vertices.iter().join(" ")),
            Self::SteinerTree(edges) => write!(
                f,
                "{}",
                edges.iter().map(|(u, v)| format!("{u} {v}")).join("\n")
            ),
            Self::Knapsack(items) => write!(f, "{}", items.iter().join(" ")),
        }
    }
}

pub fn hash(solution: &Solution) -> u64 {
    let mut hasher = DefaultHasher::new();
    solution.hash(&mut hasher);
    hasher.finish()
}

#[derive(Serialize)]
pub enum ValidateSolutionResult {
    Invalid(String),
    Valid(Score),
}

pub fn untested_submissions(state: Arc<State>) -> anyhow::Result<Vec<SubmissionId>> {
    Ok(state
        .db
        .lock()
        .unwrap()
        .prepare(
            "SELECT
                user_id,
                problem_id,
                solution_id,
                input_id
             FROM
                submissions
             WHERE
                status = ?",
        )?
        .query_map(params![Status::Untested], |row| {
            let user_id = row.get(0)?;
            let problem_id = row.get(1)?;
            let solution_id = row.get(2)?;
            let input_id = row.get(3)?;
            Ok((user_id, problem_id, solution_id, input_id))
        })?
        .collect::<Result<_, _>>()?)
}

pub fn validate(
    submission_id: SubmissionId,
    state: Arc<State>,
) -> anyhow::Result<ValidateSolutionResult> {
    let (user_id, problem_id, solution_id, input_id) = &submission_id;
    let input = state
        .static_data
        .problems
        .iter()
        .find(|problem| problem.id == *problem_id)
        .context("Bad problem ID")?
        .inputs
        .iter()
        .find(|input| input.id == *input_id)
        .context("Bad input ID")?;
    let input_path = input.path.with_extension("cbor");
    let solution = read_solution_from_db(&submission_id, state.as_ref())?;
    let validation_result = match problem_id {
        ProblemId::TravellingSalesman => {
            use tsp_validate::parse_binary_input;
            use tsp_validate::pre_validate;
            use tsp_validate::validate;

            let num_vertices = input.pre_validate_metadata;
            let path = parse_tsp_solution(&solution)?;
            pre_validate(&path, num_vertices)
                .and_then(|_| {
                    let graph = parse_binary_input(&input_path).expect("TSP input");
                    validate(&graph, &path)
                })
                .map(ValidateSolutionResult::Valid)
                .unwrap_or_else(|err| ValidateSolutionResult::Invalid(err.to_string()))
        }
        ProblemId::GraphColoring => {
            use graph_coloring_validate::parse_binary_input;
            use graph_coloring_validate::pre_validate;
            use graph_coloring_validate::validate;

            let num_vertices = input.pre_validate_metadata;
            let coloring = parse_graph_coloring_solution(&solution)?;
            pre_validate(&coloring, num_vertices)
                .and_then(|_| {
                    let graph = parse_binary_input(input_path).expect("Graph Coloring input");
                    validate(&graph, &coloring)
                })
                .map(ValidateSolutionResult::Valid)
                .unwrap_or_else(|err| ValidateSolutionResult::Invalid(err.to_string()))
        }
        ProblemId::VertexCover => {
            use vertex_cover_validate::parse_binary_input;
            use vertex_cover_validate::pre_validate;
            use vertex_cover_validate::validate;

            let num_vertices = input.pre_validate_metadata;
            let vertex_cover = parse_vertex_cover_solution(&solution)?;
            pre_validate(&vertex_cover, num_vertices)
                .and_then(|vertex_cover| {
                    let graph = parse_binary_input(&input_path).expect("Vertex Cover input");
                    validate(&graph, &vertex_cover)
                })
                .map(ValidateSolutionResult::Valid)
                .unwrap_or_else(|err| ValidateSolutionResult::Invalid(err.to_string()))
        }
        ProblemId::MaximumClique => {
            use maximum_clique_validate::parse_binary_input;
            use maximum_clique_validate::pre_validate;
            use maximum_clique_validate::validate;

            let num_vertices = input.pre_validate_metadata;
            let clique = parse_maximum_clique_solution(&solution)?;
            pre_validate(&clique, num_vertices)
                .and_then(|_| {
                    let graph = parse_binary_input(&input_path).expect("Maximum Clique input");
                    validate(&graph, &clique)
                })
                .map(ValidateSolutionResult::Valid)
                .unwrap_or_else(|err| ValidateSolutionResult::Invalid(err.to_string()))
        }
        ProblemId::MaximumCut => {
            use maximum_cut_validate::parse_binary_input;
            use maximum_cut_validate::pre_validate;
            use maximum_cut_validate::validate;

            let num_vertices = input.pre_validate_metadata;
            let vertices = parse_maximum_cut_solution(&solution)?;
            pre_validate(&vertices, num_vertices)
                .and_then(|vertices| {
                    let graph = parse_binary_input(&input_path).expect("Maximum Cut input");
                    validate(&graph, &vertices)
                })
                .map(ValidateSolutionResult::Valid)
                .unwrap_or_else(|err| ValidateSolutionResult::Invalid(err.to_string()))
        }
        ProblemId::SteinerTree => {
            use steiner_tree_validate::parse_binary_input;
            use steiner_tree_validate::pre_validate;
            use steiner_tree_validate::validate;

            let num_vertices = input.pre_validate_metadata;
            let edges = parse_steiner_tree_solution(&solution)?;
            pre_validate(&edges, num_vertices)
                .and_then(|_| {
                    let graph = parse_binary_input(&input_path).expect("Steiner Tree input");
                    validate(&graph, &edges)
                })
                .map(ValidateSolutionResult::Valid)
                .unwrap_or_else(|err| ValidateSolutionResult::Invalid(err.to_string()))
        }
        ProblemId::Knapsack => {
            use knapsack_validate::parse_binary_input;
            use knapsack_validate::pre_validate;
            use knapsack_validate::validate;

            let num_items = input.pre_validate_metadata;
            let items = parse_knapsack_solution(&solution)?;
            pre_validate(&items, num_items)
                .and_then(|_| {
                    let input = parse_binary_input(&input_path).expect("Knapsack input");
                    validate(&input, &items)
                })
                .map(|score| ValidateSolutionResult::Valid(score as usize))
                .unwrap_or_else(|err| ValidateSolutionResult::Invalid(err.to_string()))
        }
    };
    match &validation_result {
        ValidateSolutionResult::Valid(score) => state.db.lock().unwrap().execute(
            "UPDATE submissions SET
                        status = ?5
                     WHERE
                        user_id = ?1 AND
                        problem_id = ?2 AND
                        solution_id = ?3 AND
                        input_id = ?4;",
            params![
                user_id,
                problem_id,
                solution_id,
                input_id,
                Status::Solved(*score)
            ],
        )?,
        ValidateSolutionResult::Invalid(invalid_message) => state.db.lock().unwrap().execute(
            "UPDATE submissions SET
                        status = ?5,
                        invalid_message = ?6
                     WHERE
                        user_id = ?1 AND
                        problem_id = ?2 AND
                        solution_id = ?3 AND
                        input_id = ?4;",
            params![
                user_id,
                problem_id,
                solution_id,
                input_id,
                Status::Attempted,
                invalid_message,
            ],
        )?,
    };
    Ok(validation_result)
}

fn read_solution_from_db(submission_id: &SubmissionId, state: &State) -> anyhow::Result<String> {
    let (user_id, problem_id, solution_id, input_id) = &submission_id;
    Ok(state.db.lock().unwrap().query_row(
        "SELECT solution FROM submissions
             WHERE
                user_id = ? AND
                problem_id = ? AND
                solution_id = ? AND
                input_id = ?",
        params![user_id, problem_id, solution_id, input_id],
        |row| row.get(0),
    )?)
}

fn parse_tsp_solution(solution: &str) -> anyhow::Result<Vec<usize>> {
    Ok(solution
        .split_whitespace()
        .map(|node| node.parse())
        .collect::<Result<_, _>>()?)
}

fn parse_graph_coloring_solution(solution: &str) -> anyhow::Result<Vec<usize>> {
    Ok(solution
        .split_whitespace()
        .map(|color| color.parse())
        .collect::<Result<_, _>>()?)
}

fn parse_vertex_cover_solution(solution: &str) -> anyhow::Result<Vec<usize>> {
    Ok(solution
        .split_whitespace()
        .map(|vertex| vertex.parse())
        .collect::<Result<_, _>>()?)
}

fn parse_maximum_clique_solution(solution: &str) -> anyhow::Result<Vec<usize>> {
    Ok(solution
        .split_whitespace()
        .map(|vertex| vertex.parse())
        .collect::<Result<_, _>>()?)
}

fn parse_maximum_cut_solution(solution: &str) -> anyhow::Result<Vec<usize>> {
    Ok(solution
        .split_whitespace()
        .map(|vertex| vertex.parse())
        .collect::<Result<_, _>>()?)
}

fn parse_steiner_tree_solution(solution: &str) -> anyhow::Result<Vec<(usize, usize)>> {
    solution
        .lines()
        .map(
            |line| match line.split_whitespace().collect_vec().as_slice() {
                [u, v] => Ok((u.parse()?, v.parse()?)),
                _ => bail!("Malformed line: {line}"),
            },
        )
        .collect()
}

fn parse_knapsack_solution(solution: &str) -> anyhow::Result<Vec<usize>> {
    Ok(solution
        .split_whitespace()
        .map(|item| item.parse())
        .collect::<Result<_, _>>()?)
}
