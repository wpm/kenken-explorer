use anyhow::{Result, anyhow};
use clap::Args;
use kenken::generate_with;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use std::path::Path;

use crate::config::{self, SizeDist};
use crate::experiments::resolve_op_policy;

#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// Grid side length (1..=9).
    #[arg(long)]
    pub n: Option<usize>,
    /// RNG seed.
    #[arg(long)]
    pub seed: Option<u64>,
    /// Op policy name (currently only "default").
    #[arg(long)]
    pub op_policy: Option<String>,
}

#[derive(Serialize)]
struct Resolved {
    n: usize,
    seed: u64,
    op_policy: String,
    size_distribution: SizeDist,
}

#[derive(Serialize)]
struct ResultData {
    n: usize,
    uniqueness: String,
    solutions: usize,
    debug: String,
}

#[derive(Serialize)]
struct Output<'a> {
    experiment: &'a str,
    config: &'a Resolved,
    result: ResultData,
}

pub fn run(config_path: Option<&Path>, args: GenerateArgs) -> Result<()> {
    let cfg = config::load(config_path)?.generate;

    let resolved = Resolved {
        n: args.n.or(cfg.n).unwrap_or(4),
        seed: args.seed.or(cfg.seed).unwrap_or(0),
        op_policy: args
            .op_policy
            .or(cfg.op_policy)
            .unwrap_or_else(|| "default".to_string()),
        size_distribution: cfg
            .size_distribution
            .unwrap_or(SizeDist::Uniform { min: 1, max: 4 }),
    };

    let policy = resolve_op_policy(&resolved.op_policy)?;
    let mut rng = ChaCha8Rng::seed_from_u64(resolved.seed);
    let puzzle = generate_with(resolved.n, &mut rng, policy, resolved.size_distribution.into())
        .map_err(|e| anyhow!("generate_with failed: {e:?}"))?;

    let result = ResultData {
        n: puzzle.n(),
        uniqueness: format!("{:?}", puzzle.uniqueness()),
        solutions: puzzle.solutions(),
        debug: format!("{puzzle:?}"),
    };

    let out = Output {
        experiment: "generate",
        config: &resolved,
        result,
    };
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}
