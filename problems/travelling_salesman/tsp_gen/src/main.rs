use clap::Parser;
use libgraph::Graph;
use rand::prelude::*;
use rand_distr::Normal;
use std::collections::HashSet;
use std::io::Write;
use std::{fs::File, io::BufWriter};

/// Generate input for the Travelling Salesman problem.
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

    /// Mean weight of the edges. Edge weights have normal distribution.
    #[arg(short('m'), default_value_t = 10)]
    weight_mean: usize,

    /// Standard deviation of edge weights. Edge weights have normal distribution.
    #[arg(short('s'), default_value_t = 3)]
    weight_sd: usize,

    /// Output files' name.
    #[arg(short('o'), default_value = "sample")]
    output: String,
}

fn main() {
    env_logger::init();
    let mut rng = rand::rng();
    let Args {
        num_vertices,
        graph_density,
        weight_mean,
        weight_sd,
        output,
    } = Args::parse();

    let base_edges: HashSet<_> = Graph::hamiltonian_cycle(num_vertices, &mut rng)
        .edges()
        .collect();
    let mut extra_edges: Vec<_> = Graph::full(num_vertices)
        .edges()
        .filter(|edge| !base_edges.contains(edge))
        .collect();
    let target_extra_edges = (extra_edges.len() as f64 * graph_density) as usize;
    extra_edges.shuffle(&mut rng);
    extra_edges.truncate(target_extra_edges);
    extra_edges.extend(base_edges.into_iter());
    let mut edges = extra_edges;
    edges.shuffle(&mut rng);

    let mut input_file = BufWriter::new(File::create(format!("{output}.txt")).unwrap());
    let graph = edges
        .into_iter()
        .map(|(u, v)| {
            let w = random_weight(weight_mean, weight_sd, &mut rng);
            writeln!(&mut input_file, "{u} {v} {w}").unwrap();
            (u, v, w)
        })
        .fold(Graph::new(num_vertices), |graph, (u, v, w)| {
            graph.add_edge_with_metadata((u, v), w)
        });
    let graph_file = BufWriter::new(File::create(format!("{output}.cbor")).unwrap());
    ciborium::into_writer(&graph, graph_file).unwrap();
}

fn random_weight(mean: usize, sd: usize, rng: &mut impl Rng) -> usize {
    rng.sample(Normal::new(mean as f64, sd as f64).expect("SD must be finate"))
        .clamp(1.0, usize::MAX as f64) as usize
}
