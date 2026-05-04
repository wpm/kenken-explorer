use anyhow::{Result, anyhow};
use clap::Args;
use kenken::{DEFAULT_SIZE_DISTRIBUTION, Puzzle, generate_with};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use crate::config::{self, OpPolicy, SizeDist};
use crate::experiments::DEFAULT_N;

#[derive(Args, Debug)]
pub struct HistogramArgs {
    #[arg(long)]
    pub n: Option<usize>,
    #[arg(long)]
    pub trials: Option<usize>,
    /// Master RNG seed; per-trial seed is master + trial index.
    #[arg(long)]
    pub seed: Option<u64>,
    #[arg(long, value_enum)]
    pub op_policy: Option<OpPolicy>,
    #[arg(long)]
    pub max_solutions: Option<usize>,
}

#[derive(Serialize)]
struct Resolved {
    n: usize,
    trials: usize,
    seed: u64,
    op_policy: OpPolicy,
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
    /// The bucket whose `solutions` key equals `cap_bucket` aggregates every puzzle with
    /// `>= cap_bucket` solutions; lower-keyed buckets are exact counts.
    cap_bucket: usize,
    no_solution_count: usize,
    unique_count: usize,
    multi_solution_count: usize,
    unique_rate: f64,
}

#[derive(Serialize)]
struct Meta {
    threads: usize,
    elapsed_ms: u64,
}

#[derive(Serialize)]
struct Output {
    experiment: &'static str,
    config: Resolved,
    meta: Meta,
    result: ResultData,
}

pub fn run(config_path: Option<&Path>, args: HistogramArgs) -> Result<()> {
    let cfg = config::load(config_path)?.histogram;

    let resolved = Resolved {
        n: args.n.or(cfg.n).unwrap_or(DEFAULT_N),
        trials: args.trials.or(cfg.trials).unwrap_or(100),
        seed: args.seed.or(cfg.seed).unwrap_or(0),
        op_policy: args.op_policy.or(cfg.op_policy).unwrap_or_default(),
        size_distribution: cfg
            .size_distribution
            .unwrap_or_else(|| DEFAULT_SIZE_DISTRIBUTION.into()),
        max_solutions: args.max_solutions.or(cfg.max_solutions).unwrap_or(100),
    };

    if !(1..=9).contains(&resolved.n) {
        return Err(anyhow!("n must be in 1..=9, got {}", resolved.n));
    }
    if resolved.max_solutions == 0 {
        return Err(anyhow!("max_solutions must be >= 1"));
    }

    let policy = resolved.op_policy.func();
    let dist: kenken::SizeDistribution = resolved.size_distribution.into();
    let n = resolved.n;
    let max_solutions = resolved.max_solutions;
    let master_seed = resolved.seed;
    let trials = resolved.trials;

    let started = Instant::now();
    let counts: Vec<usize> = (0..trials)
        .into_par_iter()
        .map(|i| -> Result<usize> {
            let seed_i = master_seed.wrapping_add(i as u64);
            let mut rng = ChaCha8Rng::seed_from_u64(seed_i);
            let puzzle: Puzzle = generate_with(n, &mut rng, policy, dist)
                .map_err(|e| anyhow!("generate_with failed at trial {i}: {e:?}"))?;
            Ok(puzzle.solutions_at_most(max_solutions))
        })
        .collect::<Result<_>>()?;
    let elapsed = started.elapsed();

    let mut hist: BTreeMap<usize, usize> = BTreeMap::new();
    for c in &counts {
        *hist.entry(*c).or_insert(0) += 1;
    }
    let no_solution_count = hist.get(&0).copied().unwrap_or(0);
    let unique_count = hist.get(&1).copied().unwrap_or(0);
    let multi_solution_count = hist.range(2..).map(|(_, &v)| v).sum();
    let unique_rate = if trials > 0 {
        unique_count as f64 / trials as f64
    } else {
        0.0
    };
    let histogram: Vec<Bucket> = hist
        .into_iter()
        .map(|(solutions, count)| Bucket { solutions, count })
        .collect();

    let result = ResultData {
        histogram,
        cap_bucket: max_solutions,
        no_solution_count,
        unique_count,
        multi_solution_count,
        unique_rate,
    };

    let meta = Meta {
        threads: rayon::current_num_threads(),
        elapsed_ms: u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX),
    };

    let out = Output {
        experiment: "histogram",
        config: resolved,
        meta,
        result,
    };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}
