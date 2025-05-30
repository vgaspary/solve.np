use clap::Parser;
use libgraph::Graph;
use log::debug;
use rand::prelude::*;
use std::collections::HashSet;

/// Generate input for the Graph Coloring problem.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of vertices in the graph.
    #[arg(short('v'), default_value_t = 8)]
    num_vertices: usize,
    /// Graph density [0.0, 1.0], where 0.0 means the smallest possible graph that is fully
    /// connected, i.e. a minimum spanning tree, and 1.0 means a full graph.
    #[arg(short('d'), default_value_t = 0.5)]
    graph_density: f64,
}

fn main() {
    env_logger::init();
    let mut rng = rand::rng();
    let args = Args::parse();
    debug!("Generating a random graph with the following parameters:\n{args:?}.");

    let base_edges: HashSet<_> = Graph::minimum_spanning_tree(args.num_vertices, &mut rng)
        .edges()
        .collect();
    let mut extra_edges: Vec<_> = Graph::full(args.num_vertices)
        .edges()
        .filter(|edge| !base_edges.contains(edge))
        .collect();
    let target_extra_edges = (extra_edges.len() as f64 * args.graph_density) as usize;
    println!("Target extra edges: {target_extra_edges}");
    extra_edges.shuffle(&mut rng);
    extra_edges.truncate(target_extra_edges);
    extra_edges.extend(base_edges.into_iter());
    let mut edges = extra_edges;
    edges.shuffle(&mut rng);
    edges.into_iter().for_each(|(u, v)| println!("{u} {v}"));
}
