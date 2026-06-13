use anyhow::bail;
use libgraph::Graph;
use std::cell::OnceCell;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;

type Score = usize;
type Vertex = usize;
type Edge = (Vertex, Vertex);
type Weight = usize;
type Input = (Vec<Vertex>, Graph<Weight>);

pub const SAMPLE_GRAPH_SIZE: usize = 8;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Solution must not be empty.")]
    EmptySolution,

    #[error("Edge ({u}, {v}) does not exist.")]
    UnknownEdge { u: Vertex, v: Vertex },

    #[error("Edges must be unique: ({u}, {v}) edge is mentioned more than once.")]
    RepeatingEdge { u: Vertex, v: Vertex },

    #[error(
        "Provided edges do not make up a tree: vertex {unreachable} is not reachable from {from}."
    )]
    ForestNotTree { unreachable: Vertex, from: Vertex },

    #[error("Provided edges do not make up a tree: cycle detected with vertex {v}.")]
    CycleDetected { v: Vertex },

    #[error("Terminal {v} is not covered by the tree.")]
    TerminalNotCovered { v: Vertex },
}

pub fn pre_validate(solution: &[Edge], num_vertices: usize) -> Result<(), Error> {
    let graph = solution
        .iter()
        .try_fold(Graph::new(num_vertices), |graph, &edge| {
            let (u, v) = edge;
            if u >= num_vertices || v >= num_vertices || u == v {
                return Err(Error::UnknownEdge { u, v });
            }
            if graph.has_edge(edge) {
                return Err(Error::RepeatingEdge { u, v });
            }
            Ok(graph.add_edge(edge))
        })?;
    let vertices = solution
        .iter()
        .flat_map(|&(u, v)| [u, v].into_iter())
        .collect::<HashSet<_>>();
    let Some(start) = vertices.iter().next() else {
        return Err(Error::EmptySolution);
    };
    let mut visited = HashSet::new();
    let cycle_detected = OnceCell::new();
    libgraph::traverse_tree(&graph, *start, |v| {
        if !visited.insert(v) {
            let _ = cycle_detected.set(v);
            return true;
        }
        false
    });
    if let Some(&v) = cycle_detected.get() {
        return Err(Error::CycleDetected { v });
    }
    if let Some(unvisited) = vertices.difference(&visited).next() {
        return Err(Error::ForestNotTree {
            unreachable: *unvisited,
            from: *start,
        });
    }
    Ok(())
}

/// Assumes `solution` has been pre-validated.
pub fn validate(input: &Input, solution: &[Edge]) -> Result<Score, Error> {
    let (terminals, graph) = input;
    let mut terminals: HashSet<Vertex> = HashSet::from_iter(terminals.iter().cloned());
    let score = solution.iter().try_fold(0, |score, &edge| {
        let (u, v) = edge;
        terminals.remove(&u);
        terminals.remove(&v);
        match graph.get_edge_metadata(edge) {
            Some(&weight) => Ok(score + weight),
            None => Err(Error::UnknownEdge { u, v }),
        }
    })?;
    if let Some(unvisited) = terminals.into_iter().next() {
        return Err(Error::TerminalNotCovered { v: unvisited });
    }
    Ok(score)
}

pub fn parse_binary_input(path: &Path) -> anyhow::Result<Input> {
    Ok(ciborium::from_reader(BufReader::new(File::open(path)?))?)
}

pub fn parse_sample_input(input: &str) -> anyhow::Result<Input> {
    let mut lines = input.lines();
    let terminals = lines
        .next()
        .expect("Sample input is not empty")
        .split_ascii_whitespace()
        .map(|n| n.parse())
        .collect::<Result<_, _>>()?;
    lines.next();
    let graph = lines
        .map(|edge_line| {
            edge_line
                .split_ascii_whitespace()
                .filter_map(|n| n.parse().ok())
                .collect::<Vec<_>>()
        })
        .try_fold(Graph::new(SAMPLE_GRAPH_SIZE), |graph, edge| {
            match edge.as_slice() {
                [u, v, w] => Ok(graph.add_edge_with_metadata((*u, *v), *w)),
                _ => bail!("Unexpected input line format."),
            }
        })?;
    Ok((terminals, graph))
}

#[cfg(test)]
mod tests {
    use super::*;
    use libgraph::Graph;

    #[test]
    fn test_pre_validate_ok() {
        let solution = vec![(0, 1), (1, 2)];
        let num_vertices = 3;
        assert_eq!(pre_validate(&solution, num_vertices), Ok(()));
    }

    #[test]
    fn test_pre_validate_empty_solution() {
        let solution: Vec<Edge> = Vec::new();
        let num_vertices = 3;
        assert_eq!(
            pre_validate(&solution, num_vertices),
            Err(Error::EmptySolution)
        );
    }

    #[test]
    fn test_pre_validate_unknown_edge() {
        let solution = vec![(0, 1), (1, 5)];
        let num_vertices = 3;
        assert_eq!(
            pre_validate(&solution, num_vertices),
            Err(Error::UnknownEdge { u: 1, v: 5 })
        );
    }

    #[test]
    fn test_pre_validate_repeating_edge() {
        let solution = vec![(0, 1), (1, 2), (0, 1)];
        let num_vertices = 3;
        assert_eq!(
            pre_validate(&solution, num_vertices),
            Err(Error::RepeatingEdge { u: 0, v: 1 })
        );
    }

    #[test]
    fn test_pre_validate_cycle_detected() {
        let solution = vec![(0, 1), (1, 2), (2, 0)];
        let num_vertices = 3;
        assert!(matches!(
            pre_validate(&solution, num_vertices),
            Err(Error::CycleDetected { v: _ })
        ));
    }

    #[test]
    fn test_pre_validate_forest_not_tree() {
        let solution = vec![(0, 1), (2, 3)];
        let num_vertices = 4;
        let result = pre_validate(&solution, num_vertices);
        assert!(matches!(
            result,
            Err(Error::ForestNotTree {
                unreachable: _,
                from: _
            })
        ));
    }

    #[test]
    fn test_validate_ok() {
        let terminals = vec![0, 2];
        let mut graph = Graph::new(3);
        graph = graph.add_edge_with_metadata((0, 1), 10);
        graph = graph.add_edge_with_metadata((1, 2), 20);
        let input = (terminals, graph);
        let solution = vec![(0, 1), (1, 2)];
        assert_eq!(validate(&input, &solution), Ok(30));
    }

    #[test]
    fn test_validate_terminal_not_covered() {
        let terminals = vec![0, 2];
        let mut graph = Graph::new(3);
        graph = graph.add_edge_with_metadata((0, 1), 10);
        let input = (terminals, graph);
        let solution = vec![(0, 1)];
        assert_eq!(
            validate(&input, &solution),
            Err(Error::TerminalNotCovered { v: 2 })
        );
    }

    #[test]
    fn test_validate_unknown_edge() {
        let terminals = vec![0, 2];
        let mut graph = Graph::new(3);
        graph = graph.add_edge_with_metadata((0, 1), 10);
        let input = (terminals, graph);
        let solution = vec![(0, 1), (1, 2)];
        assert_eq!(
            validate(&input, &solution),
            Err(Error::UnknownEdge { u: 1, v: 2 })
        );
    }

    #[test]
    fn test_parse_sample_input_ok() {
        let input_str = "0 2\n\n0 1 10\n1 2 20";
        let result = parse_sample_input(input_str);
        assert!(result.is_ok());
        let (terminals, graph) = result.unwrap();
        assert_eq!(terminals, vec![0, 2]);
        assert_eq!(graph.get_edge_metadata((0, 1)), Some(&10));
        assert_eq!(graph.get_edge_metadata((1, 2)), Some(&20));
    }

    #[test]
    fn test_parse_sample_input_invalid_format() {
        let input_str = "0 2\n\n0 1";
        let result = parse_sample_input(input_str);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Unexpected input line format."));
        }
    }

    #[test]
    fn test_parse_sample_input_empty_terminals() {
        let input_str = "\n\n0 1 10";
        let result = parse_sample_input(input_str);
        assert!(result.is_ok());
        let (terminals, _) = result.unwrap();
        assert!(terminals.is_empty());
    }
}
