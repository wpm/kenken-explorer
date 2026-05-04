use anyhow::{Result, anyhow};
use clap::Args;
use kenken::{Puzzle, generate_with};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use crate::config::{self, SizeDist};
use crate::experiments::resolve_op_policy;

#[derive(Args, Debug)]
pub struct HistogramArgs {
    /// Grid side length (1..=9).
    #[arg(long)]
    pub n: Option<usize>,
    /// Number of independent puzzles to generate.
    #[arg(long)]
    pub trials: Option<usize>,
    /// Master RNG seed; per-trial seed is master + trial index.
    #[arg(long)]
    pub seed: Option<u64>,
    /// Op policy name (currently only "default").
    #[arg(long)]
    pub op_policy: Option<String>,
    /// Stop counting solutions for any one puzzle after this many.
    #[arg(long)]
    pub max_solutions: Option<usize>,
}

#[derive(Serialize)]
struct Resolved {
    n: usize,
    trials: usize,
    seed: u64,
    op_policy: String,
    size_distribution: SizeDist,
    max_solutions: usize,
}

#[derive(Serialize)]
struct Bucket {
    solutions: usize,
    count: usize,
}

#[derive(Serialize)]
struct ResultData {
    histogram: Vec<Bucket>,
    cap_bucket: usize,
    no_solution_count: usize,
    unique_count: usize,
    unique_rate: f64,
    multi_solution_count: usize,
}

#[derive(Serialize)]
struct Meta {
    threads: usize,
    elapsed_ms: u128,
}

#[derive(Serialize)]
struct Output<'a> {
    experiment: &'a str,
    config: &'a Resolved,
    meta: Meta,
    result: ResultData,
}

pub fn run(config_path: Option<&Path>, args: HistogramArgs) -> Result<()> {
    let cfg = config::load(config_path)?.histogram;

    let resolved = Resolved {
        n: args.n.or(cfg.n).unwrap_or(4),
        trials: args.trials.or(cfg.trials).unwrap_or(100),
        seed: args.seed.or(cfg.seed).unwrap_or(0),
        op_policy: args
            .op_policy
            .or(cfg.op_policy)
            .unwrap_or_else(|| "default".to_string()),
        size_distribution: cfg
            .size_distribution
            .unwrap_or(SizeDist::Uniform { min: 1, max: 4 }),
        max_solutions: args.max_solutions.or(cfg.max_solutions).unwrap_or(100),
    };

    if !(1..=9).contains(&resolved.n) {
        return Err(anyhow!("n must be in 1..=9, got {}", resolved.n));
    }

    let policy = resolve_op_policy(&resolved.op_policy)?;
    let dist: kenken::SizeDistribution = resolved.size_distribution.into();
    let n = resolved.n;
    let max_solutions = resolved.max_solutions;
    let master_seed = resolved.seed;
    let trials = resolved.trials;

    let started = Instant::now();
    let counts: Vec<usize> = (0..trials)
        .into_par_iter()
        .map(|i| {
            let seed_i = master_seed.wrapping_add(i as u64);
            let mut rng = ChaCha8Rng::seed_from_u64(seed_i);
            let puzzle: Puzzle = generate_with(n, &mut rng, policy, dist)
                .expect("generate_with cannot fail for validated n");
            puzzle.solutions_at_most(max_solutions)
        })
        .collect();
    let elapsed = started.elapsed();

    let mut hist: BTreeMap<usize, usize> = BTreeMap::new();
    for c in &counts {
        *hist.entry(*c).or_insert(0) += 1;
    }
    let histogram: Vec<Bucket> = hist
        .into_iter()
        .map(|(solutions, count)| Bucket { solutions, count })
        .collect();

    let no_solution_count = counts.iter().filter(|&&c| c == 0).count();
    let unique_count = counts.iter().filter(|&&c| c == 1).count();
    let multi_solution_count = counts.iter().filter(|&&c| c >= 2).count();
    let unique_rate = if trials > 0 {
        unique_count as f64 / trials as f64
    } else {
        0.0
    };

    let result = ResultData {
        histogram,
        cap_bucket: max_solutions,
        no_solution_count,
        unique_count,
        unique_rate,
        multi_solution_count,
    };

    let meta = Meta {
        threads: rayon::current_num_threads(),
        elapsed_ms: elapsed.as_millis(),
    };

    let out = Output {
        experiment: "histogram",
        config: &resolved,
        meta,
        result,
    };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}
