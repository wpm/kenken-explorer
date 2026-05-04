use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::experiments::generate::GenerateArgs;
use crate::experiments::histogram::HistogramArgs;

#[derive(Parser, Debug)]
#[command(name = "kenken-explorer", about = "Empirical KenKen experiments")]
pub struct Cli {
    /// Path to a TOML config file.
    #[arg(long, short = 'c', global = true)]
    pub config: Option<PathBuf>,

    /// Worker thread count. Defaults to rayon's choice (typically the number of cores).
    #[arg(long, short = 't', global = true)]
    pub threads: Option<usize>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Generate a single puzzle and print it (debugging aid).
    Generate(GenerateArgs),
    /// Run trials and report a histogram of solution counts.
    Histogram(HistogramArgs),
}
