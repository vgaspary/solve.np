use anyhow::Result;
use log::info;
use rand::prelude::*;
use std::collections::HashSet;
use ts_common::{Edge, Vertex, Weight};

#[derive(Clone, Copy, Debug)]
struct Params {
    num_vertices: u16,
    graph_density: f64,
    weight_mean: u16,
    weight_sd: u16,
}

const SAMPLE: Params = Params {
    num_vertices: 8,
    graph_density: 0.5,
    weight_mean: 5,
    weight_sd: 5,
};

struct Graph {
    adj_list: Vec<HashSet<u16>>,
}

impl Graph {
    fn new(num_vertices: u16) -> Self {
        Self {
            adj_list: vec![HashSet::new(); num_vertices as usize],
        }
    }

    fn add_edge(&mut self, edge: Edge) -> bool {
        let Edge(Vertex(v), Vertex(u), _) = edge;
        self.adj_list[v as usize].insert(u) && self.adj_list[u as usize].insert(v)
    }
}

fn main() -> Result<()> {
    let mut rng = rand::rng();
    let params = SAMPLE;
    info!("Generating a random graph with the following parameters:\n{params:?}.");
    info!("Randomize the vertices before building the minimum spanning tree.");
    let mut vertices: Vec<Vertex> = (0..params.num_vertices).map(|v| Vertex(v)).collect();
    vertices.shuffle(&mut rng);
    info!("Vertices: {vertices:?}.");

    info!("Build the minimum spanning tree to make sure the graph is connected.");
    let mut edges: Vec<_> = (1..vertices.len())
        .map(|i| (vertices[i - 1], vertices[i]))
        .zip(std::iter::from_fn(|| {
            Weight::random(&mut rng, params.weight_mean, params.weight_sd).ok()
        }))
        .map(|((v, u), w)| Edge(v, u, w))
        .collect();
    let mut graph = edges
        .iter()
        .fold(Graph::new(params.num_vertices), |mut graph, &edge| {
            graph.add_edge(edge);
            graph
        });
    assert_eq!(edges.len(), vertices.len() - 1);
    info!("Established the base graph with {} edges.", edges.len());

    let max_edges = params.num_vertices * (params.num_vertices - 1) / 2;
    let remaining_edges = ((max_edges - edges.len() as u16) as f64 * params.graph_density) as u16;
    info!(
        "Adding {remaining_edges} edges to meet the required density: {}.",
        params.graph_density
    );
    for _ in 0..remaining_edges {
        loop {
            let edge = Edge::random(
                &mut rng,
                params.num_vertices,
                params.weight_mean,
                params.weight_sd,
            )?;
            if graph.add_edge(edge) {
                edges.push(edge);
                break;
            }
        }
    }
    info!("Added the remaining edges, {} in total.", edges.len());
    info!("Output:");
    edges
        .into_iter()
        .for_each(|Edge(Vertex(v), Vertex(u), Weight(w))| println!("{v} {u} {w}"));

    Ok(())
}
