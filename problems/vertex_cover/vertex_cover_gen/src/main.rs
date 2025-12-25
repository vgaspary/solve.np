use clap::Parser;
use libgraph::Graph;
use rand::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

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

    /// Output files' name.
    #[arg(short('o'), default_value = "sample")]
    output: String,
}

fn main() {
    let Args {
        num_vertices,
        graph_density,
        output,
    } = Args::parse();
    let mut edges: Vec<_> = Graph::full(num_vertices).edges().collect();
    let mut rng = rand::rng();
    edges.shuffle(&mut rng);
    edges.truncate((edges.len() as f64 * graph_density) as usize);

    let mut input_file = BufWriter::new(File::create(format!("{output}.txt")).unwrap());
    edges
        .iter()
        .for_each(|(u, v)| writeln!(&mut input_file, "{u} {v}").unwrap());

    let graph = edges
        .into_iter()
        .fold(Graph::new(num_vertices), |graph, edge| graph.add_edge(edge));
    let graph_file = BufWriter::new(File::create(format!("{output}.cbor")).unwrap());
    ciborium::into_writer(&graph, graph_file).unwrap();
}
