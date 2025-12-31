use anyhow::Context;
use libgraph::Graph;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;

type Score = usize;
type Vertex = usize;

pub const SAMPLE_GRAPH_SIZE: usize = 8;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Vertex {v} does not exist.")]
    UnknownVertex { v: Vertex },

    #[error("Vertices must be unique: {v} is mentioned more than once.")]
    RepeatingVertex { v: Vertex },

    #[error("Not a clique: no edge connecting {u} and {v}.")]
    NotClique { u: Vertex, v: Vertex },
}

pub fn pre_validate(solution: &[Vertex], num_vertices: usize) -> Result<(), Error> {
    solution
        .iter()
        .try_fold(HashSet::new(), |mut vertices, &v| {
            if v >= num_vertices {
                return Err(Error::UnknownVertex { v });
            }
            if vertices.insert(v) {
                Ok(vertices)
            } else {
                Err(Error::RepeatingVertex { v })
            }
        })
        .map(|_| ())
}

/// Assumes `solution` has been pre-validated with `pre_validate` and does not contain duplicates.
pub fn validate(input: &Graph, solution: &[Vertex]) -> Result<Score, Error> {
    if let Some((u, v)) = (0..solution.len())
        .flat_map(|i| (i + 1..solution.len()).map(move |j| (i, j)))
        .map(|(i, j)| (solution[i], solution[j]))
        .find(|&(u, v)| !input.adj_list[u].contains_key(&v))
    {
        return Err(Error::NotClique { u, v });
    }
    Ok(solution.len())
}

pub fn parse_binary_input(path: &Path) -> anyhow::Result<Graph> {
    Ok(ciborium::from_reader(BufReader::new(File::open(path)?))?)
}

pub fn parse_sample_input(input: &str) -> anyhow::Result<Graph> {
    input
        .lines()
        .try_fold(Graph::new(SAMPLE_GRAPH_SIZE), |graph, line| {
            let (u, v) = line.split_once(' ').context("Split once failed.")?;
            let (u, v) = (u.parse()?, v.parse()?);
            Ok(graph.add_edge((u, v)))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_validate_valid() {
        let solution = vec![1, 2, 3];
        assert_eq!(pre_validate(&solution, 10), Ok(()));
    }

    #[test]
    fn test_pre_validate_unknown_vertex() {
        let solution = vec![1, 2, 10];
        assert_eq!(
            pre_validate(&solution, 10),
            Err(Error::UnknownVertex { v: 10 })
        );
    }

    #[test]
    fn test_pre_validate_repeating_vertex() {
        let solution = vec![1, 2, 1];
        assert_eq!(
            pre_validate(&solution, 10),
            Err(Error::RepeatingVertex { v: 1 })
        );
    }

    fn create_graph() -> Graph {
        let mut graph = Graph::new(5);
        graph = graph.add_edge((0, 1));
        graph = graph.add_edge((0, 2));
        graph = graph.add_edge((0, 3));
        graph = graph.add_edge((1, 2));
        graph = graph.add_edge((1, 3));
        graph = graph.add_edge((2, 3));
        graph
    }

    #[test]
    fn test_validate_clique() {
        let graph = create_graph();
        let solution = vec![0, 1, 2, 3];
        assert_eq!(validate(&graph, &solution), Ok(4));
    }

    #[test]
    fn test_validate_not_clique() {
        let graph = create_graph();
        let solution = vec![0, 1, 4];
        assert_eq!(validate(&graph, &solution), Err(Error::NotClique { u: 0, v: 4 }));
    }

    #[test]
    fn test_validate_empty_solution() {
        let graph = create_graph();
        let solution = vec![];
        assert_eq!(validate(&graph, &solution), Ok(0));
    }

    #[test]
    fn test_validate_single_vertex_solution() {
        let graph = create_graph();
        let solution = vec![0];
        assert_eq!(validate(&graph, &solution), Ok(1));
    }

    #[test]
    fn test_parse_sample_input_valid() {
        let input = "0 1\n1 2\n2 0";
        let graph = parse_sample_input(input).unwrap();
        assert_eq!(graph.num_vertices(), SAMPLE_GRAPH_SIZE);
        assert!(graph.adj_list[0].contains_key(&1));
        assert!(graph.adj_list[1].contains_key(&2));
        assert!(graph.adj_list[2].contains_key(&0));
    }

    #[test]
    fn test_parse_sample_input_invalid() {
        let input = "0 1\n1 2\na b";
        assert!(parse_sample_input(input).is_err());
    }
}
