use libgraph::Graph;

pub(crate) fn parse_tsp_input(input: &str) -> Option<Graph<usize>> {
    let edges = input
        .split("\n")
        .filter(|line| !line.is_empty())
        .map(|line| {
            match line
                .split_ascii_whitespace()
                .flat_map(|number| number.parse().ok())
                .collect::<Vec<usize>>()
                .as_slice()
            {
                [u, v, w] => Some((*u, *v, *w)),
                _ => None,
            }
        })
        .collect::<Option<Vec<_>>>()?;
    let num_vertices = *edges.iter().map(|(u, v, _)| u.max(v)).max()? + 1;
    Some(
        edges
            .into_iter()
            .fold(Graph::<usize>::new(num_vertices), |graph, (u, v, w)| {
                graph.add_edge_with_metadata((u, v), w)
            }),
    )
}

pub(crate) fn parse_graph_coloring_input(input: &str) -> Option<Graph> {
    let edges = input
        .split("\n")
        .filter(|line| !line.is_empty())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tsp_input_valid() {
        let input = "0 1 10\n1 2 20\n";
        let graph = parse_tsp_input(input).unwrap();
        assert_eq!(graph.num_vertices(), 3);
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
    fn test_parse_tsp_input_empty_line() {
        let input = "0 1 10\n\n1 2 20\n";
        let graph = parse_tsp_input(input).unwrap();
        assert_eq!(graph.num_vertices(), 3);
        assert_eq!(graph.edges().count(), 2);
    }

    #[test]
    fn test_parse_tsp_input_malformed() {
        let input = "0 1 10\n1 2\n";
        assert!(parse_tsp_input(input).is_none());
    }

    #[test]
    fn test_parse_tsp_input_empty() {
        let input = "";
        assert!(parse_tsp_input(input).is_none());
    }

    #[test]
    fn test_parse_graph_coloring_input_valid() {
        let input = "0 1\n1 2\n";
        let graph = parse_graph_coloring_input(input).unwrap();
        assert_eq!(graph.num_vertices(), 3);
        assert_eq!(graph.edges().count(), 2);
    }

    #[test]
    fn test_parse_graph_coloring_input_empty_line() {
        let input = "0 1\n\n1 2\n";
        let graph = parse_graph_coloring_input(input).unwrap();
        assert_eq!(graph.num_vertices(), 3);
        assert_eq!(graph.edges().count(), 2);
    }

    #[test]
    fn test_parse_graph_coloring_input_malformed() {
        let input = "0 1\n1 2 3\n";
        assert!(parse_graph_coloring_input(input).is_none());
    }

    #[test]
    fn test_parse_graph_coloring_input_empty() {
        let input = "";
        assert!(parse_graph_coloring_input(input).is_none());
    }
}
