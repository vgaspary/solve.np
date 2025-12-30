use anyhow::bail;
use libgraph::Graph;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;

type Weight = usize;
type Score = usize;

pub const SAMPLE_GRAPH_SIZE: usize = 8;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid vertex {v}.")]
    InvalidVertex { v: usize },

    #[error("The path must start at vertex 0.")]
    IncorrectStart,

    #[error("The path must finish at vertex 0.")]
    IncorrectFinish,

    #[error(
        "Expected a path of length {expected} but found {actual}. A valid path contains all vertices exactly once and the start vertex (0) twice."
    )]
    IncorrectLength { expected: usize, actual: usize },

    #[error("The path visits vertex {v} twice.")]
    VisitedTwice { v: usize },

    #[error("The path incorrectly assumes an edge between {u} and {v}.")]
    NoEdge { u: usize, v: usize },

    #[error("Internal error.")]
    Internal,
}

pub fn pre_validate(solution: &[usize], num_vertices: usize) -> Result<(), Error> {
    if solution.len() != num_vertices + 1 {
        return Err(Error::IncorrectLength {
            expected: num_vertices + 1,
            actual: solution.len(),
        });
    }
    if solution[0] != 0 {
        return Err(Error::IncorrectStart);
    }
    if solution[solution.len() - 1] != 0 {
        return Err(Error::IncorrectFinish);
    }
    solution
        .iter()
        .skip(1)
        .try_fold(HashSet::new(), |mut visited, &v| {
            if v >= num_vertices {
                return Err(Error::InvalidVertex { v });
            }
            if !visited.insert(v) {
                return Err(Error::VisitedTwice { v });
            }
            Ok(visited)
        })
        .map(|_| ())
}

/// Assumes `solution` has been pre-validated with `pre_validate` and is a valid Hamiltonian cycle
/// starting from (and finishing at) `0`.
pub fn validate(input: &Graph<Weight>, solution: &[usize]) -> Result<Score, Error> {
    (0..solution.len() - 1)
        .map(|i| (solution[i], solution[i + 1]))
        .try_fold(0, |score, (u, v)| {
            let Some(neighbors) = input.adj_list.get(u) else {
                return Err(Error::InvalidVertex { v: u });
            };
            let Some(weight) = neighbors.get(&v) else {
                return Err(Error::NoEdge { u, v });
            };
            Ok(score + weight)
        })
}

pub fn parse_binary_input(path: &Path) -> anyhow::Result<Graph<Weight>> {
    Ok(ciborium::from_reader(BufReader::new(File::open(path)?))?)
}

pub fn parse_sample_input(input: &str) -> anyhow::Result<Graph<Weight>> {
    let edges = input
        .lines()
        .map(|line| {
            match line
                .split_ascii_whitespace()
                .flat_map(|number| number.parse().ok())
                .collect::<Vec<usize>>()
                .as_slice()
            {
                [u, v, w] => Ok((*u, *v, *w)),
                unexpected_line => bail!("Failed parsing: {unexpected_line:?}"),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let graph = edges.into_iter().fold(
        Graph::<usize>::new(SAMPLE_GRAPH_SIZE),
        |graph, (u, v, w)| graph.add_edge_with_metadata((u, v), w),
    );
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use libgraph::Graph;

    fn complete_graph(n: usize) -> Graph<Weight> {
        (0..n)
            .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
            .fold(Graph::new(n), |g, (i, j)| {
                g.add_edge_with_metadata((i, j), 1)
            })
    }

    //
    // pre_validate tests
    //

    #[test]
    fn test_pre_validate_correct_solution() {
        let solution = vec![0, 1, 2, 0];
        assert_eq!(pre_validate(&solution, 3), Ok(()));
    }

    #[test]
    fn test_pre_validate_incorrect_length() {
        let solution = vec![0, 1, 0];
        assert_eq!(
            pre_validate(&solution, 3),
            Err(Error::IncorrectLength {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn test_pre_validate_incorrect_start() {
        let solution = vec![1, 0, 2, 0];
        assert_eq!(pre_validate(&solution, 3), Err(Error::IncorrectStart));
    }

    #[test]
    fn test_pre_validate_incorrect_finish() {
        let solution = vec![0, 1, 2, 1];
        assert_eq!(pre_validate(&solution, 3), Err(Error::IncorrectFinish));
    }

    #[test]
    fn test_pre_validate_visited_twice() {
        let solution = vec![0, 1, 2, 1, 0];
        assert_eq!(
            pre_validate(&solution, 4),
            Err(Error::VisitedTwice { v: 1 })
        );
    }

    #[test]
    fn test_pre_validate_invalid_vertex() {
        let solution = vec![0, 1, 4, 0];
        assert_eq!(
            pre_validate(&solution, 3),
            Err(Error::InvalidVertex { v: 4 })
        );
    }

    //
    // validate tests
    //

    #[test]
    fn test_correct_solution() {
        let graph = complete_graph(3);
        let solution = vec![0, 1, 2, 0];
        assert_eq!(validate(&graph, &solution), Ok(3));
    }

    #[test]
    fn test_no_edge() {
        let graph = Graph::new(3)
            .add_edge_with_metadata((0, 1), 1)
            .add_edge_with_metadata((0, 2), 1);
        let solution = vec![0, 1, 2, 0];
        assert_eq!(
            validate(&graph, &solution),
            Err(Error::NoEdge { u: 1, v: 2 })
        );
    }

    //
    // parse_sample_input tests
    //

    #[test]
    fn test_parse_sample_input_valid() {
        let input = "0 1 10\n1 2 20\n";
        let graph = parse_sample_input(input).unwrap();
        assert_eq!(graph.num_vertices(), SAMPLE_GRAPH_SIZE);
        assert_eq!(graph.edges().count(), 2);
        assert_eq!(
            graph
                .edges_with_metadata()
                .find(|((u, v), _)| *u == 0 && *v == 1)
                .map(|(_, metadata)| metadata),
            Some(10)
        );
        assert_eq!(
            graph
                .edges_with_metadata()
                .find(|((u, v), _)| *u == 1 && *v == 2)
                .map(|(_, metadata)| metadata),
            Some(20)
        );
    }

    #[test]
    fn test_parse_sample_input_empty_line() {
        let input = "0 1 10\n\n1 2 20\n";
        let graph = parse_sample_input(input);
        assert!(graph.is_err());
    }

    #[test]
    fn test_parse_sample_input_malformed() {
        let input = "0 1 10\n1 2\n";
        assert!(parse_sample_input(input).is_err());
    }

    #[test]
    fn test_parse_sample_input_empty() {
        let input = "";
        let graph = parse_sample_input(input).unwrap();
        assert_eq!(graph.num_vertices(), SAMPLE_GRAPH_SIZE);
        assert_eq!(graph.edges().count(), 0);
    }
}
