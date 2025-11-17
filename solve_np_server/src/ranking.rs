#![allow(dead_code)]

use crate::state::State;
use crate::types::InputId;
use crate::types::ProblemId;
use crate::types::Score;
use crate::types::SolutionId;
use itertools::Itertools;
use librank::Rank;
use librank::RankedExt;
use log::debug;
use std::time::Duration;

struct Submission {
    problem_id: ProblemId,
    input_id: InputId,
    solution_id: SolutionId,
    score: Score,
}

struct Solution {
    solution_id: SolutionId,
    problem_id: ProblemId,
}

// Assumes that all submissions belong to solved solutions.
fn rank_submissions(submissions: Vec<Submission>) -> Vec<(Rank, Submission)> {
    submissions
        .into_iter()
        .into_group_map_by(|submission| {
            (submission.problem_id.clone(), submission.input_id.clone())
        })
        .into_values()
        .map(|submissions| {
            submissions
                .into_iter()
                .rank_by(|submission| submission.score)
        })
        .flatten()
        .collect()
}

fn rank_solutions(submission_ranking: &[(Rank, Submission)]) -> Vec<(Rank, Solution)> {
    submission_ranking
        .iter()
        .into_group_map_by(|(_, submission)| submission.solution_id.clone())
        .into_values()
        .filter_map(|submissions| {
            let solution_id = submissions.first()?.1.solution_id.clone();
            let problem_id = submissions.first()?.1.problem_id.clone();
            let aggregate_rank: usize = submissions.into_iter().map(|(Rank(r), _)| r).sum();
            Some((
                Solution {
                    solution_id,
                    problem_id,
                },
                aggregate_rank,
            ))
        })
        .into_group_map_by(|(solution, _)| solution.problem_id.clone())
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

pub fn update_ranking(state: &State) {
    /*
     * 1. Sanitize.
     *
     * `solutions.status` is the truth source for ranking availability both for solutions and
     * submissions belonging to it. First, we need to make sure that only *solved* solutions and
     * the corresponding submissions have non-`None` ranks. A solution is solved if
     * `solutions.status = 0`.
     *
     * ```sql
     * UPDATE solutions SET
     *  ranking = NULL
     * WHERE
     *  status <> 0;
     *
     * UPDATE submissions SET
     *  ranking = NULL
     * WHERE
     *  solution_id.status <> 0;
     * ```
     *
     * 2. Collect all solved solutions (`WHERE status = 0`) and submissions belonging to them.
     *
     * For each solution there are `N` submissions, where `N` is the number of inputs the problem
     * correpsonding to the solution has.
     *
     * 3. Release the DB object lock. To avoid race conditions, 1. and 2. are executed atomically.
     *
     * 4. Confirm that the solution-submission relationship is upheld as described in 2..
     *
     * 5. Rank the submissions, use an aggregation to rank solutions based on the former.
     *
     * 6. Update the rank of the solution/submission entries as per recalculation.
     *
     * Note: Ranking changes are expected to be rare. It's important to lock the DB object if and
     * only if necessary.
     *
     * 7. Sleep for some time.
     */
}
