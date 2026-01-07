use clap::Parser;
use clap::Subcommand;
use libgraph::Graph;
use rand::prelude::*;
use rand_distr::Normal;
use rand_distr::Uniform;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

type Weight = usize;

/// Generate input for the Maximum Cut problem.
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
struct UniformDistributionArgs {
    /// Min weight.
    #[arg(long)]
    min: usize,

    /// Max weight.
    #[arg(long)]
    max: usize,
}

#[derive(Subcommand)]
enum WeightDistribution {
    Normal(NormalDistributionArgs),
    Uniform(UniformDistributionArgs),
}

fn main() {
    let Args {
        num_vertices,
        graph_density,
        output,
        weight_dist,
    } = Args::parse();
    let mut rng = rand::rng();
    let graph = Graph::random(num_vertices, graph_density, &mut rng);
    let graph = graph
        .edges()
        .map(|edge| {
            let weight = random_weight(&weight_dist, &mut rng);
            (edge, weight)
        })
        .fold(Graph::new(num_vertices), |graph, (edge, weight)| {
            graph.add_edge_with_metadata(edge, weight)
        });
    let graph_file = BufWriter::new(File::create(format!("{output}.cbor")).unwrap());
    ciborium::into_writer(&graph, graph_file).unwrap();

    let mut edges: Vec<_> = graph.edges_with_metadata().collect();
    edges.shuffle(&mut rng);
    let mut input_file = BufWriter::new(File::create(format!("{output}.txt")).unwrap());
    edges
        .iter()
        .for_each(|((u, v), w)| writeln!(&mut input_file, "{u} {v} {w}").unwrap());
}

fn random_weight(weight_dist: &WeightDistribution, rng: &mut impl Rng) -> Weight {
    match weight_dist {
        WeightDistribution::Normal(NormalDistributionArgs { mean, sd }) => {
            rng.sample(Normal::new(*mean as f64, *sd as f64).expect("SD must be finate"))
                .clamp(1.0, usize::MAX as f64) as usize
        }
        WeightDistribution::Uniform(UniformDistributionArgs { min, max }) => {
            rng.sample(Uniform::new(*min, *max).expect("min < max"))
        }
    }
}
