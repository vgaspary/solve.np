use clap::Parser;
use rand::Rng;
use rand_distr::Distribution;
use rand_distr::Normal;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

/// Generate input for the 0-1 Knapsack problem.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of items.
    #[arg(short('n'), long, default_value_t = 10)]
    num_items: usize,

    /// Weight distribution params.
    #[arg(long, default_value_t = 5)]
    w_low_mean: u64,
    #[arg(long, default_value_t = 2)]
    w_low_sd: u64,
    #[arg(long, default_value_t = 10)]
    w_high_mean: u64,
    #[arg(long, default_value_t = 5)]
    w_high_sd: u64,
    #[arg(long, default_value_t = 0.7)]
    w_low_prob: f64,

    /// Normal distribution params of K. Each item's value is derived by sampling this random
    /// variable and adding it to the weight.
    #[arg(long, default_value_t = 4)]
    k_mean: u64,
    #[arg(long, default_value_t = 2)]
    k_sd: u64,

    /// Output files' name.
    #[arg(short('o'), default_value = "sample")]
    output: String,
}

fn main() {
    let Args {
        num_items,
        w_low_mean,
        w_low_sd,
        w_high_mean,
        w_high_sd,
        w_low_prob,
        k_mean,
        k_sd,
        output,
    } = Args::parse();
    let w_low_dist = Normal::new(w_low_mean as f64, w_low_sd as f64).unwrap();
    let w_high_dist = Normal::new(w_high_mean as f64, w_high_sd as f64).unwrap();
    let k_dist = Normal::new(k_mean as f64, k_sd as f64).unwrap();
    let mut rng = rand::rng();
    let items = std::iter::from_fn(|| {
        let weight_dist = if rng.random_bool(w_low_prob) {
            w_low_dist
        } else {
            w_high_dist
        };
        let weight = weight_dist.sample(&mut rng).max(1.0) as u64;
        let k = k_dist.sample(&mut rng).max(1.0);
        let value = (weight as f64 + k).max(1.0) as u64;
        Some((weight, value))
    })
    .take(num_items)
    .collect::<Vec<_>>();
    let total_weight = items.iter().map(|(weight, _)| *weight).sum::<u64>();
    let capacity = total_weight / 2;

    let mut txt_file = BufWriter::new(File::create(format!("{output}.txt")).unwrap());
    writeln!(&mut txt_file, "{}", capacity).unwrap();
    writeln!(&mut txt_file).unwrap();
    for (weight, value) in &items {
        writeln!(&mut txt_file, "{} {}", weight, value).unwrap();
    }

    let cbor_file = BufWriter::new(File::create(format!("{output}.cbor")).unwrap());
    let binary_input = (capacity, items);
    ciborium::into_writer(&binary_input, cbor_file).unwrap();
}
