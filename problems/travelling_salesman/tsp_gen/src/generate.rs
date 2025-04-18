use anyhow::Result;
use itertools::Itertools;
use log::debug;
use rand::prelude::*;
use std::collections::HashSet;
use tsp_common::{Edge, Vertex, Weight};

pub(crate) struct Graph {
    adj_list: Vec<HashSet<u16>>,
}

impl Graph {
    pub(crate) fn new(num_vertices: u16) -> Self {
        Self {
            adj_list: vec![HashSet::new(); num_vertices as usize],
        }
    }

    fn add_edge(&mut self, edge: Edge) -> bool {
        let Edge(Vertex(v), Vertex(u), _) = edge;
        self.adj_list[v as usize].insert(u) && self.adj_list[u as usize].insert(v)
    }
}

pub(crate) fn create_graph_base(
    num_vertices: u16,
    weight_mean: u16,
    weight_sd: u16,
    rng: &mut impl Rng,
) -> Result<(Vec<Edge>, Graph)> {
    debug!("Creating a random hamiltonian cycle to serve as the base graph.");
    let mut vertices = (0..num_vertices).map(|v| Vertex(v)).collect::<Vec<_>>();
    vertices.shuffle(rng);
    let edges = (0..vertices.len())
        .map(|i| (vertices[i], vertices[(i + 1) % vertices.len()]))
        .inspect(|(Vertex(v), Vertex(u))| debug!("{v}\t{u}"))
        .map(|(v, u)| Ok(Edge(v, u, Weight::random(rng, weight_mean, weight_sd)?)))
        .collect::<Result<Vec<_>>>()?;
    let graph = edges
        .iter()
        .fold(Graph::new(num_vertices), |mut graph, &edge| {
            graph.add_edge(edge);
            graph
        });
    Ok((edges, graph))
}

pub(crate) fn complete_graph(
    mut graph: Graph,
    num_vertices: u16,
    weight_mean: u16,
    weight_sd: u16,
    rng: &mut impl Rng,
) -> Result<Vec<Edge>> {
    let remaining_edges = (0..num_vertices)
        .cartesian_product(0..num_vertices)
        .filter(|(v, u)| v < u)
        .map(|(v, u)| {
            Ok(Edge(
                Vertex(v),
                Vertex(u),
                Weight::random(rng, weight_mean, weight_sd)?,
            ))
        })
        .filter(|edge| match edge {
            &Ok(edge) => graph.add_edge(edge),
            _ => true,
        })
        .collect::<Result<Vec<_>>>()?;
    debug!(
        "Graph base had {} edges, the remaining {} were initialized.",
        num_vertices,
        remaining_edges.len()
    );
    Ok(remaining_edges)
}

pub(crate) fn reduce_edges(
    mut extra_edges: Vec<Edge>,
    num_vertices: u16,
    graph_density: f64,
    rng: &mut impl Rng,
) -> Vec<Edge> {
    let num_extra_edges = (extra_edges.len() as f64 * graph_density) as usize;
    debug!(
        "Removing {} edges to meet the required density. {} edges will remain.",
        extra_edges.len() - num_extra_edges,
        num_extra_edges + num_vertices as usize
    );
    extra_edges.shuffle(rng);
    extra_edges.truncate(num_extra_edges);
    extra_edges
}
