use crate::state::State;
use crate::types::InputId;
use crate::types::ProblemId;
use crate::types::Score;
use crate::types::SolutionId;
use anyhow::Context;
use itertools::Itertools;
use librank::Rank;
use librank::RankedExt;
use log::debug;
use log::info;
use rusqlite::params;
use rusqlite::types::FromSql;
use std::collections::HashMap;

struct Submission {
    problem_id: ProblemId,
    solution_id: SolutionId,
    input_id: InputId,
    score: Score,
    ranking: Ranking,
}

struct Solution {
    solution_id: SolutionId,
    problem_id: ProblemId,
    ranking: Ranking,
}

#[derive(Clone, Copy)]
struct Ranking(Option<Rank>);

impl FromSql for Ranking {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value.as_i64_or_null()? {
            Some(n) => Ok(Self(Some(Rank(n as usize)))),
            None => Ok(Self(None)),
        }
    }
}

// Assumes that all submissions belong to solved solutions.
fn rank_submissions(submissions: Vec<Submission>) -> Vec<(Rank, Submission)> {
    submissions
        .into_iter()
        .into_group_map_by(|submission| (submission.problem_id, submission.input_id.clone()))
        .into_values()
        .map(|submissions| {
            submissions
                .into_iter()
                .rank_by(|submission| match submission.problem_id {
                    ProblemId::TravellingSalesman => submission.score,
                    ProblemId::GraphColoring => submission.score,
                    ProblemId::VertexCover => submission.score,
                    ProblemId::MaximumClique => submission.score.wrapping_neg(),
                    ProblemId::MaximumCut => submission.score.wrapping_neg(),
                })
        })
        .flatten()
        .collect()
}

fn rank_solutions(
    solutions: Vec<Solution>,
    submission_ranking: &[(Rank, Submission)],
) -> Vec<(Rank, Solution)> {
    let solution_ranking = solutions
        .into_iter()
        .map(
            |Solution {
                 solution_id,
                 problem_id,
                 ranking,
             }| ((solution_id, problem_id), ranking),
        )
        .collect::<HashMap<_, _>>();
    submission_ranking
        .iter()
        .into_group_map_by(|(_, submission)| submission.solution_id.clone())
        .into_values()
        .filter_map(|submissions| {
            let solution_id = submissions.first()?.1.solution_id.clone();
            let problem_id = submissions.first()?.1.problem_id;
            let ranking = *solution_ranking.get(&(solution_id.clone(), problem_id))?;
            let aggregate_rank: usize = submissions.into_iter().map(|(Rank(r), _)| r).sum();
            Some((
                Solution {
                    solution_id,
                    problem_id,
                    ranking,
                },
                aggregate_rank,
            ))
        })
        .into_group_map_by(|(solution, _)| solution.problem_id)
        .into_values()
        .map(|solutions| {
            solutions
                .into_iter()
                .rank_by(|(_, aggregate_rank)| *aggregate_rank)
        })
        .flatten()
        .map(|(rank, (solution, _))| (rank, solution))
        .collect()
}

fn sanitize_and_select(state: &State) -> anyhow::Result<(Vec<Solution>, Vec<Submission>)> {
    let db = state.db.lock().unwrap();
    debug!("[Background task] Sanitizing...");
    db.execute(
        "UPDATE solutions SET
                ranking = NULL
            WHERE
                status <> 0;",
        [],
    )
    .context("Sanitizing solutions")?;
    db.execute(
        "UPDATE submissions SET 
            ranking = NULL
        FROM solutions AS s
        WHERE
            submissions.solution_id = s.solution_id AND
            s.status <> 0;",
        [],
    )
    .context("Sanitizing submissions")?;
    debug!("[Background task] Selecting...");
    let solutions = db
        .prepare(
            "SELECT solution_id, problem_id, ranking
            FROM solutions
            WHERE
                status = 0;",
        )
        .context("Collecting solutions")?
        .query_map([], |row| {
            Ok(Solution {
                solution_id: row.get(0)?,
                problem_id: row.get(1)?,
                ranking: row.get(2)?,
            })
        })
        .context("Collecting solutions")?
        .filter_map(Result::ok)
        .collect_vec();
    let submissions = db
        .prepare(
            "SELECT
                s.problem_id, s.solution_id, s.input_id, s.status, s.ranking
            FROM submissions AS s
            INNER JOIN solutions
            ON s.solution_id = solutions.solution_id
            WHERE
                solutions.status = 0;",
        )
        .context("Collecting solutions")?
        .query_map([], |row| {
            Ok(Submission {
                problem_id: row.get(0)?,
                solution_id: row.get(1)?,
                input_id: row.get(2)?,
                score: row.get(3)?,
                ranking: row.get(4)?,
            })
        })
        .context("Collecting solutions")?
        .filter_map(Result::ok)
        .collect_vec();
    Ok((solutions, submissions))
}

fn maybe_update(
    state: &State,
    ranked_submissions: Vec<(Rank, Submission)>,
    ranked_solutions: Vec<(Rank, Solution)>,
) -> anyhow::Result<()> {
    let submissions_to_update = ranked_submissions
        .into_iter()
        .filter(|(rank, submission)| submission.ranking.0 != Some(*rank))
        .collect_vec();
    let solutions_to_update = ranked_solutions
        .into_iter()
        .filter(|(rank, solution)| solution.ranking.0 != Some(*rank))
        .collect_vec();

    if submissions_to_update.is_empty() && solutions_to_update.is_empty() {
        debug!("[Background task] Rankings are unchanged. Nothing to update.");
        return Ok(());
    }

    info!(
        "[Background task] Updating rankings for {} solutions and {} submissions.",
        solutions_to_update.len(),
        submissions_to_update.len()
    );

    let mut db = state.db.lock().unwrap();
    let tx = db.transaction().context("Starting transaction")?;

    for (rank, solution) in solutions_to_update {
        tx.execute(
            "UPDATE solutions SET ranking = ?1 WHERE solution_id = ?2",
            params![rank.0 as i64, &solution.solution_id],
        )
        .with_context(|| format!("Updating solution {}", solution.solution_id))?;
    }

    for (rank, submission) in submissions_to_update {
        tx.execute(
            "UPDATE submissions SET ranking = ?1 WHERE solution_id = ?2 AND input_id = ?3",
            params![rank.0 as i64, &submission.solution_id, &submission.input_id],
        )
        .with_context(|| {
            format!(
                "Updating submission for solution {} on input {}",
                submission.solution_id, submission.input_id
            )
        })?;
    }

    tx.commit().context("Committing transaction")?;
    debug!("[Background task] Rankings updated.");
    Ok(())
}

pub fn update_ranking(state: &State) -> anyhow::Result<()> {
    debug!("[Background task] Updating solution and submission rankings.");
    let (solutions, submissions) = sanitize_and_select(state)?;

    debug!("[Background task] Ranking...");
    let ranked_submissions = rank_submissions(submissions);
    let ranked_solutions = rank_solutions(solutions, &ranked_submissions);

    maybe_update(state, ranked_submissions, ranked_solutions)?;
    Ok(())
}
