use anyhow::Result;
use clap::Parser;
use log::info;
use rand::prelude::*;
use std::collections::HashSet;
use ts_common::{Edge, Vertex, Weight};

/// Generate input for the Travelling Salesman problem.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of vertices in the graph.
    #[arg(short('v'), default_value_t = 8)]
    num_vertices: u16,

    /// Graph density [0.0, 1.0], where 0.0 means the smallest possible graph that is fully
    /// connected, i.e. a minimum spanning tree, and 1.0 means a full graph.
    #[arg(short('d'), default_value_t = 0.5)]
    graph_density: f64,

    /// Mean weight of the edges. Edge weights have normal distribution.
    #[arg(short('m'), default_value_t = 10)]
    weight_mean: u16,

    /// Standard deviation of edge weights. Edge weights have normal distribution.
    #[arg(short('s'), default_value_t = 3)]
    weight_sd: u16,
}

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
    let args = Args::parse();
    info!("Generating a random graph with the following parameters:\n{args:?}.");
    let Args {
        num_vertices,
        graph_density,
        weight_mean,
        weight_sd,
    } = args;
    info!("Randomize the vertices before building the hamiltonian cycle.");
    let mut vertices: Vec<Vertex> = (0..num_vertices).map(|v| Vertex(v)).collect();
    vertices.shuffle(&mut rng);
    info!("Vertices: {vertices:?}.");

    info!(
        "Build the base hamiltonian cycle to ensure the graph has at least one and the problem has a solution."
    );
    let mut edges: Vec<_> = (0..vertices.len())
        .map(|i| (vertices[i], vertices[(i + 1) % vertices.len()]))
        .zip(std::iter::from_fn(|| {
            Weight::random(&mut rng, weight_mean, weight_sd).ok()
        }))
        .map(|((v, u), w)| Edge(v, u, w))
        .collect();
    let mut graph = edges
        .iter()
        .fold(Graph::new(num_vertices), |mut graph, &edge| {
            graph.add_edge(edge);
            graph
        });
    assert_eq!(edges.len(), vertices.len());
    info!("Established the base graph with {} edges.", edges.len());

    let max_edges = num_vertices as usize * (num_vertices as usize - 1) / 2;
    let remaining_edges = ((max_edges - edges.len()) as f64 * graph_density) as usize;
    info!(
        "Adding {remaining_edges} edges to meet the required density: {}.",
        graph_density
    );
    for _ in 0..remaining_edges {
        loop {
            let edge = Edge::random(&mut rng, num_vertices, weight_mean, weight_sd)?;
            if graph.add_edge(edge) {
                edges.push(edge);
                break;
            }
        }
    }
    info!("Added the remaining edges, {} in total.", edges.len());
    info!("Shuffling the edges before generating the problem input.");
    edges.shuffle(&mut rng);
    info!("Output:");
    edges
        .into_iter()
        .for_each(|Edge(Vertex(v), Vertex(u), Weight(w))| println!("{v} {u} {w}"));

    Ok(())
}
