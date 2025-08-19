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
