use anyhow::{Result, anyhow};
use clap::Args;
use kenken::{DEFAULT_SIZE_DISTRIBUTION, generate_with};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use std::path::Path;

use crate::config::{self, OpPolicy, SizeDist};
use crate::experiments::{DEFAULT_N, uniqueness_str};

#[derive(Args, Debug)]
pub struct GenerateArgs {
    #[arg(long)]
    pub n: Option<usize>,
    #[arg(long)]
    pub seed: Option<u64>,
    #[arg(long, value_enum)]
    pub op_policy: Option<OpPolicy>,
}

#[derive(Serialize)]
struct Resolved {
    n: usize,
    seed: u64,
    op_policy: OpPolicy,
    size_distribution: SizeDist,
}

#[derive(Serialize)]
struct ResultData {
    uniqueness: &'static str,
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
        n: args.n.or(cfg.n).unwrap_or(DEFAULT_N),
        seed: args.seed.or(cfg.seed).unwrap_or(0),
        op_policy: args.op_policy.or(cfg.op_policy).unwrap_or_default(),
        size_distribution: cfg
            .size_distribution
            .unwrap_or_else(|| DEFAULT_SIZE_DISTRIBUTION.into()),
    };

    let mut rng = ChaCha8Rng::seed_from_u64(resolved.seed);
    let puzzle = generate_with(
        resolved.n,
        &mut rng,
        resolved.op_policy.func(),
        resolved.size_distribution.into(),
    )
    .map_err(|e| anyhow!("generate_with failed: {e:?}"))?;

    let result = ResultData {
        uniqueness: uniqueness_str(puzzle.uniqueness()),
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
