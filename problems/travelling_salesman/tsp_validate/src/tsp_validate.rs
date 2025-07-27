use libgraph::Graph;
use std::collections::HashSet;
use thiserror::Error;

type Weight = usize;
type Score = usize;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid vertex {u}.")]
    InvalidVertex { u: usize },

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
}

pub fn validate(input: &Graph<Weight>, solution: Vec<usize>) -> Result<Score, Error> {
    if input.num_vertices() + 1 != solution.len() {
        return Err(Error::IncorrectLength {
            expected: input.num_vertices() + 1,
            actual: solution.len(),
        });
    }

    if solution[0] != 0 {
        return Err(Error::IncorrectStart);
    }
    if solution[solution.len() - 1] != 0 {
        return Err(Error::IncorrectFinish);
    }

    let mut visited_vertices = HashSet::new();

    (0..solution.len() - 1)
        .map(|i| (solution[i], solution[i + 1]))
        .try_fold(0, |score, (u, v)| {
            if v != 0 {
                if !visited_vertices.insert(v) {
                    return Err(Error::VisitedTwice { v });
                }
            }
            let Some(neighbors) = input.adj_list.get(u) else {
                return Err(Error::InvalidVertex { u });
            };
            let Some(weight) = neighbors.get(&v) else {
                return Err(Error::NoEdge { u, v });
            };
            Ok(score + weight)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use libgraph::Graph;

    fn complete_graph(n: usize) -> Graph<Weight> {
        let mut graph = Graph::new(n);
        for i in 0..n {
            for j in i + 1..n {
                graph = graph.add_edge_with_metadata((i, j), 1);
            }
        }
        graph
    }

    #[test]
    fn test_correct_solution() {
        let graph = complete_graph(3);
        let solution = vec![0, 1, 2, 0];
        assert_eq!(validate(&graph, solution), Ok(3));
    }

    #[test]
    fn test_incorrect_length() {
        let graph = complete_graph(3);
        let solution = vec![0, 1, 0];
        assert_eq!(
            validate(&graph, solution),
            Err(Error::IncorrectLength {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn test_incorrect_start() {
        let graph = complete_graph(3);
        let solution = vec![1, 0, 2, 0];
        assert_eq!(validate(&graph, solution), Err(Error::IncorrectStart));
    }

    #[test]
    fn test_incorrect_finish() {
        let graph = complete_graph(3);
        let solution = vec![0, 1, 2, 1];
        assert_eq!(validate(&graph, solution), Err(Error::IncorrectFinish));
    }

    #[test]
    fn test_visited_twice() {
        let graph = complete_graph(4);
        let solution = vec![0, 1, 2, 1, 0];
        assert_eq!(validate(&graph, solution), Err(Error::VisitedTwice { v: 1 }));
    }

    #[test]
    fn test_no_edge() {
        let mut graph = Graph::new(3);
        graph = graph.add_edge_with_metadata((0, 1), 1);
        graph = graph.add_edge_with_metadata((0, 2), 1);
        let solution = vec![0, 1, 2, 0];
        assert_eq!(validate(&graph, solution), Err(Error::NoEdge { u: 1, v: 2 }));
    }
}