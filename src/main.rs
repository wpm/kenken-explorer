mod cli;
mod config;
mod experiments;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Command};

fn main() -> Result<()> {
    let args = Cli::parse();

    if let Some(threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()?;
    }

    match args.command {
        Command::Generate(a) => experiments::generate::run(args.config.as_deref(), a),
        Command::Histogram(a) => experiments::histogram::run(args.config.as_deref(), a),
    }
}
