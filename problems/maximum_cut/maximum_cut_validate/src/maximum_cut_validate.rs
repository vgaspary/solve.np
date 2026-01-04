use anyhow::bail;
use libgraph::Graph;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;

type Score = usize;
type Vertex = usize;
type Weight = usize;

pub const SAMPLE_GRAPH_SIZE: usize = 8;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Vertex {v} does not exist.")]
    UnknownVertex { v: Vertex },

    #[error("Vertices must be unique: {v} is mentioned more than once.")]
    RepeatingVertex { v: Vertex },

    #[error(
        "All vertices are provided partitioning the graph into the full set and the empty set. Partitions must be proper subsets."
    )]
    ProperSubset,
}

pub fn pre_validate(solution: &[Vertex], num_vertices: usize) -> Result<HashSet<Vertex>, Error> {
    let vertices = solution
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
        })?;
    if vertices.len() == num_vertices || solution.is_empty() {
        return Err(Error::ProperSubset);
    }
    Ok(vertices)
}

pub fn validate(input: &Graph<Weight>, solution: &HashSet<Vertex>) -> Result<Score, Error> {
    let score = input
        .edges_with_metadata()
        .filter(|((u, v), _)| solution.contains(&u) ^ solution.contains(&v))
        .map(|(_, w)| w)
        .sum::<Score>();
    Ok(score)
}

pub fn parse_binary_input(path: &Path) -> anyhow::Result<Graph<Weight>> {
    Ok(ciborium::from_reader(BufReader::new(File::open(path)?))?)
}

pub fn parse_sample_input(input: &str) -> anyhow::Result<Graph<Weight>> {
    input
        .lines()
        .try_fold(Graph::new(SAMPLE_GRAPH_SIZE), |graph, line| {
            let line = line.split_ascii_whitespace().collect::<Vec<_>>();
            let [u, v, w] = line.as_slice() else {
                bail!("Unexpected line");
            };
            let edge = (u.parse()?, v.parse()?);
            let weight = w.parse()?;
            Ok(graph.add_edge_with_metadata(edge, weight))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use libgraph::Graph;

    #[test]
    fn test_pre_validate_ok() {
        let result = pre_validate(&[0, 2], 4);
        assert_eq!(result, Ok(HashSet::from([0, 2])));
    }

    #[test]
    fn test_pre_validate_unknown_vertex() {
        let result = pre_validate(&[0, 5], 4);
        assert_eq!(result, Err(Error::UnknownVertex { v: 5 }));
    }

    #[test]
    fn test_pre_validate_repeating_vertex() {
        let result = pre_validate(&[0, 2, 0], 4);
        assert_eq!(result, Err(Error::RepeatingVertex { v: 0 }));
    }

    #[test]
    fn test_pre_validate_proper_subset_error() {
        let result = pre_validate(&[0, 1, 2, 3], 4);
        assert_eq!(result, Err(Error::ProperSubset));
        let result = pre_validate(&[], 4);
        assert_eq!(result, Err(Error::ProperSubset));
    }

    #[test]
    fn test_validate_basic_cut() {
        let graph = Graph::new(4)
            .add_edge_with_metadata((0, 1), 10)
            .add_edge_with_metadata((1, 2), 20)
            .add_edge_with_metadata((2, 3), 30)
            .add_edge_with_metadata((0, 3), 5);
        let solution = HashSet::from([0, 2]);
        let score = validate(&graph, &solution).unwrap();
        assert_eq!(score, 65);
    }

    #[test]
    fn test_validate_no_cut_edges() {
        let graph = Graph::new(4)
            .add_edge_with_metadata((0, 1), 10)
            .add_edge_with_metadata((1, 2), 20);
        let solution = HashSet::from([0, 1]);
        let score = validate(&graph, &solution).unwrap();
        assert_eq!(score, 20);
    }

    #[test]
    fn test_validate_all_edges_cut() {
        let graph = Graph::new(2).add_edge_with_metadata((0, 1), 100);
        let solution = HashSet::from([0]);
        let score = validate(&graph, &solution).unwrap();
        assert_eq!(score, 100);
    }

    #[test]
    fn test_validate_empty_graph() {
        let graph = Graph::new(4);
        let solution = HashSet::from([0, 2]);
        let score = validate(&graph, &solution).unwrap();
        assert_eq!(score, 0);
    }

    #[test]
    fn test_parse_sample_input_ok() {
        let input = "0 1 10\n1 2 20\n0 2 5";
        let graph = parse_sample_input(input).unwrap();

        assert_eq!(graph.num_vertices(), SAMPLE_GRAPH_SIZE); // Default size
        assert_eq!(graph.edges().count(), 3);

        let edges_with_metadata: Vec<((Vertex, Vertex), Weight)> =
            graph.edges_with_metadata().collect();
        assert!(edges_with_metadata.contains(&((0, 1), 10)));
        assert!(edges_with_metadata.contains(&((1, 2), 20)));
        assert!(edges_with_metadata.contains(&((0, 2), 5)));
    }

    #[test]
    fn test_parse_sample_input_malformed_line() {
        let input = "0 1 10\n1 2\n0 2 5"; // Missing weight on second line
        let result = parse_sample_input(input);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Unexpected line");
    }

    #[test]
    fn test_parse_sample_input_non_numeric_vertex() {
        let input = "0 1 10\nA 2 20"; // Non-numeric vertex 'A'
        let result = parse_sample_input(input);
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("invalid digit found in string")
        );
    }

    #[test]
    fn test_parse_sample_input_non_numeric_weight() {
        let input = "0 1 10\n1 2 B"; // Non-numeric weight 'B'
        let result = parse_sample_input(input);
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("invalid digit found in string")
        );
    }

    #[test]
    fn test_parse_sample_input_empty_string() {
        let input = "";
        let graph = parse_sample_input(input).unwrap();
        assert_eq!(graph.num_vertices(), SAMPLE_GRAPH_SIZE);
        assert_eq!(graph.edges().count(), 0);
    }
}
