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

    #[error("The ({u}, {v}) edge is not covered.")]
    EdgeNotCovered { u: Vertex, v: Vertex },
}

pub fn pre_validate(solution: &[Vertex], num_vertices: usize) -> Result<HashSet<Vertex>, Error> {
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
}

/// Assumes `solution` has been pre-validated with `pre_validate` and does not contain duplicates.
pub fn validate(input: &Graph, solution: &HashSet<Vertex>) -> Result<Score, Error> {
    if let Some((u, v)) = input
        .edges()
        .find(|&(u, v)| !solution.contains(&u) && !solution.contains(&v))
    {
        return Err(Error::EdgeNotCovered { u, v });
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
    use libgraph::Graph;
    use std::collections::HashSet;

    #[test]
    fn pre_validate_valid_solution() {
        let solution = vec![0, 1, 2];
        let num_vertices = 3;
        let result = super::pre_validate(&solution, num_vertices);
        assert_eq!(result, Ok(HashSet::from([0, 1, 2])));
    }

    #[test]
    fn pre_validate_unknown_vertex() {
        let solution = vec![0, 1, 3];
        let num_vertices = 3;
        let result = super::pre_validate(&solution, num_vertices);
        assert_eq!(result, Err(Error::UnknownVertex { v: 3 }));
    }

    #[test]
    fn pre_validate_repeating_vertex() {
        let solution = vec![0, 1, 1];
        let num_vertices = 3;
        let result = super::pre_validate(&solution, num_vertices);
        assert_eq!(result, Err(Error::RepeatingVertex { v: 1 }));
    }

    #[test]
    fn validate_valid_cover() {
        let graph = Graph::new(3).add_edge((0, 1)).add_edge((1, 2));
        let solution = HashSet::from([1]);
        let result = super::validate(&graph, &solution);
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn validate_another_valid_cover() {
        let graph = Graph::new(3).add_edge((0, 1)).add_edge((1, 2));
        let solution = HashSet::from([0, 2]);
        let result = super::validate(&graph, &solution);
        assert_eq!(result, Ok(2));
    }

    #[test]
    fn validate_not_a_cover() {
        let graph = Graph::new(3).add_edge((0, 1)).add_edge((1, 2));
        let solution = HashSet::from([0]);
        let result = super::validate(&graph, &solution);
        assert_eq!(result, Err(Error::EdgeNotCovered { u: 1, v: 2 }));
    }
}
