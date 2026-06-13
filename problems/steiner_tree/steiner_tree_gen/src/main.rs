use clap::Parser;
use clap::Subcommand;
use libgraph::Graph;
use rand::prelude::*;
use rand_distr::Normal;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of vertices in the graph.
    #[arg(short('v'), default_value_t = 8)]
    num_vertices: usize,

    /// Graph density [0.0, 1.0], where 0.0 means the smallest possible graph that is fully
    /// connected, i.e. a minimum spanning tree, and 1.0 means a full graph.
    #[arg(short('d'), default_value_t = 0.5)]
    graph_density: f64,

    /// Number of terminals in the Steiner Tree problem.
    #[arg(short('t'), default_value_t = 4)]
    num_terminals: usize,

    /// Output files' name.
    #[arg(short('o'), default_value = "sample")]
    output: String,

    #[command(subcommand)]
    weight_dist: WeightDistribution,
}

#[derive(clap::Args)]
struct NormalDistributionArgs {
    /// Mean weight.
    #[arg(long)]
    mean: usize,

    /// Weight standard deviation.
    #[arg(long)]
    sd: usize,
}

#[derive(clap::Args)]
struct BimodalDistributionArgs {
    /// Low peak mean weight.
    #[arg(long)]
    low_mean: usize,

    /// Low peak standard deviation.
    #[arg(long)]
    low_sd: usize,

    /// High peak mean weight.
    #[arg(long)]
    high_mean: usize,

    /// High peak standard deviation.
    #[arg(long)]
    high_sd: usize,

    /// Probability of sampling from the low peak.
    #[arg(long)]
    low_prob: f64,
}

#[derive(Subcommand)]
enum WeightDistribution {
    Normal(NormalDistributionArgs),
    Bimodal(BimodalDistributionArgs),
}

fn main() {
    let Args {
        num_vertices,
        graph_density,
        num_terminals,
        output,
        weight_dist,
    } = Args::parse();
    let mut rng = rand::rng();
    let mut input_file = BufWriter::new(File::create(format!("{output}.txt")).unwrap());
    let terminals = (0..num_vertices)
        .choose_multiple(&mut rng, num_terminals)
        .into_iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>();
    writeln!(&mut input_file, "{}", terminals.join(" ")).unwrap();
    writeln!(&mut input_file).unwrap();
    let mst_edges = Graph::minimum_spanning_tree(num_vertices, &mut rng)
        .edges()
        .collect::<HashSet<_>>();
    let mut extra_edges: Vec<_> = Graph::full(num_vertices)
        .edges()
        .filter(|edge| !mst_edges.contains(edge))
        .collect();
    extra_edges.shuffle(&mut rng);
    let target_extra_edges = (extra_edges.len() as f64 * graph_density) as usize;
    extra_edges.truncate(target_extra_edges);
    let mut edges: Vec<_> = mst_edges
        .into_iter()
        .chain(extra_edges)
        .map(|(u, v)| {
            let w = random_weight(&weight_dist, &mut rng);
            (u, v, w)
        })
        .collect();
    edges.shuffle(&mut rng);
    for (u, v, weight) in &edges {
        writeln!(&mut input_file, "{u} {v} {weight}").unwrap();
    }
    let graph = edges
        .iter()
        .fold(Graph::new(num_vertices), |graph, &(u, v, weight)| {
            graph.add_edge_with_metadata((u, v), weight)
        });
    let input = (terminals, graph);
    let binary_file = BufWriter::new(File::create(format!("{output}.cbor")).unwrap());
    ciborium::into_writer(&input, binary_file).unwrap();
}

fn random_weight(dist: &WeightDistribution, rng: &mut impl Rng) -> usize {
    match dist {
        WeightDistribution::Normal(NormalDistributionArgs { mean, sd }) => {
            let normal = Normal::new(*mean as f64, *sd as f64).unwrap();
            normal.sample(rng).clamp(1.0, usize::MAX as f64) as usize
        }
        WeightDistribution::Bimodal(BimodalDistributionArgs {
            low_mean,
            low_sd,
            high_mean,
            high_sd,
            low_prob,
        }) => {
            if rng.random_bool(*low_prob) {
                let normal = Normal::new(*low_mean as f64, *low_sd as f64).unwrap();
                normal.sample(rng).clamp(1.0, usize::MAX as f64) as usize
            } else {
                let normal = Normal::new(*high_mean as f64, *high_sd as f64).unwrap();
                normal.sample(rng).clamp(1.0, usize::MAX as f64) as usize
            }
        }
    }
}
