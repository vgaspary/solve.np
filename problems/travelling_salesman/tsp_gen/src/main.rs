mod generate;

use anyhow::Result;
use clap::Parser;
use generate::{complete_graph, create_graph_base, reduce_edges};
use log::debug;
use rand::prelude::*;
use tsp_common::{Edge, Vertex, Weight};

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

fn main() -> Result<()> {
    env_logger::init();
    let mut rng = rand::rng();
    let args = Args::parse();
    debug!("Generating a random graph with the following parameters:\n{args:?}.");

    let (base_edges, graph) = create_graph_base(
        args.num_vertices,
        args.weight_mean,
        args.weight_sd,
        &mut rng,
    )?;
    let remaining_edges = complete_graph(
        graph,
        args.num_vertices,
        args.weight_mean,
        args.weight_sd,
        &mut rng,
    )?;
    let extra_edges = reduce_edges(
        remaining_edges,
        args.num_vertices,
        args.graph_density,
        &mut rng,
    );
    let mut edges = [base_edges, extra_edges].concat();
    edges.shuffle(&mut rng);
    edges
        .into_iter()
        .for_each(|Edge(Vertex(v), Vertex(u), Weight(w))| println!("{v} {u} {w}"));
    Ok(())
}
