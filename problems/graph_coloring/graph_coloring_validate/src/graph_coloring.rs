use libgraph::Graph;
use std::collections::HashSet;
use thiserror::Error;

type Score = usize;
type Color = usize;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Input grpah has {num_vertices} nodes but {num_colors} colors are provided.")]
    UnexpectedLength {
        num_vertices: usize,
        num_colors: usize,
    },
    #[error("Nodes {u} and {v} are connected and have the same color: {color}")]
    SameColor { u: usize, v: usize, color: Color },
}

pub fn validate(input: &Graph, solution: Vec<Color>) -> Result<Score, Error> {
    if input.num_vertices() != solution.len() {
        return Err(Error::UnexpectedLength {
            num_vertices: input.num_vertices(),
            num_colors: solution.len(),
        });
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_colorings() {
        // Linear graph: 0--1--2, 2-coloring
        let graph = Graph::new(3).add_edge((0, 1)).add_edge((1, 2));
        assert_eq!(validate(&graph, vec![0, 1, 0]), Ok(2));

        // Disconnected graph: 0--1, 2--3, 2-coloring
        let graph = Graph::new(4).add_edge((0, 1)).add_edge((2, 3));
        assert_eq!(validate(&graph, vec![0, 1, 0, 1]), Ok(2));

        // Triangle graph: 0--1, 0--2, 1--2, 3-coloring
        let graph = Graph::new(3)
            .add_edge((0, 1))
            .add_edge((0, 2))
            .add_edge((1, 2));
        assert_eq!(validate(&graph, vec![0, 1, 2]), Ok(3));

        // Star graph: 0 connected to 1, 2, 3, 2-coloring
        let graph = Graph::new(4)
            .add_edge((0, 1))
            .add_edge((0, 2))
            .add_edge((0, 3));
        assert_eq!(validate(&graph, vec![0, 1, 1, 1]), Ok(2));
    }

    #[test]
    fn test_mismatched_solution_size() {
        let graph = Graph::new(2);

        // Solution too short
        assert_eq!(
            validate(&graph, vec![0]),
            Err(Error::UnexpectedLength {
                num_vertices: 2,
                num_colors: 1
            })
        );

        // Solution too long
        assert_eq!(
            validate(&graph, vec![0, 1, 2]),
            Err(Error::UnexpectedLength {
                num_vertices: 2,
                num_colors: 3
            })
        );
    }

    #[test]
    fn test_adjacent_nodes_with_same_color() {
        {
            // Graph: 0--1--2
            let graph = Graph::new(3).add_edge((0, 1)).add_edge((1, 2));

            // Case 1: 0 and 1 have same color
            assert_eq!(
                validate(&graph, vec![0, 0, 1]),
                Err(Error::SameColor {
                    u: 0,
                    v: 1,
                    color: 0
                })
            );

            // Case 2: 1 and 2 have same color
            assert_eq!(
                validate(&graph, vec![0, 1, 1]),
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
                validate(&graph, vec![0, 0, 1]),
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
        assert_eq!(validate(&Graph::new(0), vec![]), Ok(0));
        assert_eq!(validate(&Graph::new(1), vec![0]), Ok(1));
        assert_eq!(validate(&Graph::new(4), vec![0, 0, 0, 0]), Ok(1));
        assert_eq!(validate(&Graph::new(4), vec![0, 1, 2, 3]), Ok(4));
    }
}
