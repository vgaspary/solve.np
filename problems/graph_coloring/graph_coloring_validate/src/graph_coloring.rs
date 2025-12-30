use anyhow::Context;
use libgraph::Graph;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use thiserror::Error;

type Score = usize;
type Color = usize;

pub const SAMPLE_GRAPH_SIZE: usize = 8;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Input graph has {num_vertices} nodes but {num_colors} colors are provided.")]
    UnexpectedLength {
        num_vertices: usize,
        num_colors: usize,
    },

    #[error("Nodes {u} and {v} are connected and have the same color: {color}")]
    SameColor { u: usize, v: usize, color: Color },
}

pub fn pre_validate(solution: &[Color], num_vertices: usize) -> Result<(), Error> {
    if num_vertices != solution.len() {
        return Err(Error::UnexpectedLength {
            num_vertices: num_vertices,
            num_colors: solution.len(),
        });
    }
    Ok(())
}

/// Assumes `solution` has been pre-validated with `pre_validate` and contains color assignments
/// for all vertices.
pub fn validate(input: &Graph, solution: &[Color]) -> Result<Score, Error> {
    if let Some((u, v)) = input.edges().find(|&(u, v)| solution[u] == solution[v]) {
        return Err(Error::SameColor {
            u,
            v,
            color: solution[u],
        });
    }
    let unique_colors = solution.iter().collect::<HashSet<_>>().len();
    Ok(unique_colors)
}

pub fn parse_input(input: &str) -> Option<Graph> {
    let edges = input
        .lines()
        .map(|line| {
            match line
                .split_ascii_whitespace()
                .flat_map(|number| number.parse().ok())
                .collect::<Vec<usize>>()
                .as_slice()
            {
                [u, v] => Some((*u, *v)),
                _ => None,
            }
        })
        .collect::<Option<Vec<_>>>()?;
    let num_vertices = *edges.iter().map(|(u, v)| u.max(v)).max()? + 1;
    Some(
        edges
            .into_iter()
            .fold(Graph::new(num_vertices), |graph, edge| graph.add_edge(edge)),
    )
}

pub fn parse_binary_input(path: PathBuf) -> anyhow::Result<Graph> {
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
    fn test_valid_colorings() {
        // Linear graph: 0--1--2, 2-coloring
        let graph = Graph::new(3).add_edge((0, 1)).add_edge((1, 2));
        assert_eq!(validate(&graph, &[0, 1, 0]), Ok(2));

        // Disconnected graph: 0--1, 2--3, 2-coloring
        let graph = Graph::new(4).add_edge((0, 1)).add_edge((2, 3));
        assert_eq!(validate(&graph, &[0, 1, 0, 1]), Ok(2));

        // Triangle graph: 0--1, 0--2, 1--2, 3-coloring
        let graph = Graph::new(3)
            .add_edge((0, 1))
            .add_edge((0, 2))
            .add_edge((1, 2));
        assert_eq!(validate(&graph, &[0, 1, 2]), Ok(3));

        // Star graph: 0 connected to 1, 2, 3, 2-coloring
        let graph = Graph::new(4)
            .add_edge((0, 1))
            .add_edge((0, 2))
            .add_edge((0, 3));
        assert_eq!(validate(&graph, &[0, 1, 1, 1]), Ok(2));
    }

    #[test]
    fn test_pre_validate() {
        // Solution too short
        assert_eq!(
            pre_validate(&[0], 2),
            Err(Error::UnexpectedLength {
                num_vertices: 2,
                num_colors: 1
            })
        );

        // Solution too long
        assert_eq!(
            pre_validate(&[0, 1, 2], 2),
            Err(Error::UnexpectedLength {
                num_vertices: 2,
                num_colors: 3
            })
        );

        // Valid solution
        assert_eq!(pre_validate(&[0, 1, 2, 3], 4), Ok(()));

        // Valid solution with 0 vertices
        assert_eq!(pre_validate(&[], 0), Ok(()));
    }

    #[test]
    fn test_adjacent_nodes_with_same_color() {
        {
            // Graph: 0--1--2
            let graph = Graph::new(3).add_edge((0, 1)).add_edge((1, 2));

            // Case 1: 0 and 1 have same color
            assert_eq!(
                validate(&graph, &[0, 0, 1]),
                Err(Error::SameColor {
                    u: 0,
                    v: 1,
                    color: 0
                })
            );

            // Case 2: 1 and 2 have same color
            assert_eq!(
                validate(&graph, &[0, 1, 1]),
                Err(Error::SameColor {
                    u: 1,
                    v: 2,
                    color: 1
                })
            );
        }

        {
            // Graph: 0--1, 0--2, 1--2 (triangle)
            let graph = Graph::full(3);
            assert_eq!(
                validate(&graph, &[0, 0, 1]),
                Err(Error::SameColor {
                    u: 0,
                    v: 1,
                    color: 0
                })
            );
        }
    }

    #[test]
    fn test_edge_cases_graphs() {
        assert_eq!(validate(&Graph::new(0), &[]), Ok(0));
        assert_eq!(validate(&Graph::new(1), &[0]), Ok(1));
        assert_eq!(validate(&Graph::new(4), &[0, 0, 0, 0]), Ok(1));
        assert_eq!(validate(&Graph::new(4), &[0, 1, 2, 3]), Ok(4));
    }

    #[test]
    fn test_parse_input_valid() {
        let input = "0 1\n1 2\n";
        let graph = parse_input(input).unwrap();
        assert_eq!(graph.num_vertices(), 3);
        assert_eq!(graph.edges().count(), 2);
    }

    #[test]
    fn test_parse_input_empty_line() {
        let input = "0 1\n\n1 2\n";
        let graph = parse_input(input);
        assert!(graph.is_none());
    }

    #[test]
    fn test_parse_input_malformed() {
        let input = "0 1\n1 2 3\n";
        assert!(parse_input(input).is_none());
    }

    #[test]
    fn test_parse_input_empty() {
        let input = "";
        assert!(parse_input(input).is_none());
    }
}
