use anyhow::Result;
use clap::Parser;
use log::debug;
use rand::prelude::*;
use std::collections::HashSet;
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

fn create_graph_base(args: &Args, rng: &mut impl Rng) -> Result<(Vec<Edge>, Graph)> {
    debug!("Creating a random hamiltonian cycle to serve as the base graph.");
    let mut vertices = (0..args.num_vertices)
        .map(|v| Vertex(v))
        .collect::<Vec<_>>();
    vertices.shuffle(rng);
    let edges = (0..vertices.len())
        .map(|i| (vertices[i], vertices[(i + 1) % vertices.len()]))
        .map(|(v, u)| {
            Ok(Edge(
                v,
                u,
                Weight::random(rng, args.weight_mean, args.weight_sd)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let graph = edges
        .iter()
        .fold(Graph::new(args.num_vertices), |mut graph, &edge| {
            graph.add_edge(edge);
            graph
        });
    Ok((edges, graph))
}

fn complete_graph(mut graph: Graph, args: &Args, rng: &mut impl Rng) -> Result<Vec<Edge>> {
    let remaining_edges = (0..args.num_vertices)
        .zip(0..args.num_vertices)
        .filter(|(v, u)| v < u)
        .map(|(v, u)| {
            Ok(Edge(
                Vertex(v),
                Vertex(u),
                Weight::random(rng, args.weight_mean, args.weight_sd)?,
            ))
        })
        .filter(|edge| match edge {
            &Ok(edge) => graph.add_edge(edge),
            _ => true,
        })
        .collect::<Result<Vec<_>>>()?;
    debug!(
        "Graph base had {} edges, the remaining {} were initialized.",
        args.num_vertices,
        remaining_edges.len()
    );
    Ok(remaining_edges)
}

fn reduce_edges(mut extra_edges: Vec<Edge>, args: &Args, rng: &mut impl Rng) -> Vec<Edge> {
    let num_extra_edges = (extra_edges.len() as f64 * args.graph_density) as usize;
    debug!(
        "Removing {} edges to meet the required density. {} edges will remain.",
        extra_edges.len() - num_extra_edges,
        num_extra_edges + args.num_vertices as usize
    );
    extra_edges.shuffle(rng);
    extra_edges.truncate(num_extra_edges);
    extra_edges
}

fn main() -> Result<()> {
    let mut rng = rand::rng();
    let args = Args::parse();
    debug!("Generating a random graph with the following parameters:\n{args:?}.");

    let (base_edges, graph) = create_graph_base(&args, &mut rng)?;
    let remaining_edges = complete_graph(graph, &args, &mut rng)?;
    let extra_edges = reduce_edges(remaining_edges, &args, &mut rng);
    let mut edges = [base_edges, extra_edges].concat();
    edges.shuffle(&mut rng);
    edges
        .into_iter()
        .for_each(|Edge(Vertex(v), Vertex(u), Weight(w))| println!("{v} {u} {w}"));
    Ok(())
}
