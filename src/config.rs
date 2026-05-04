use anyhow::{Context, Result};
use kenken::{Index, N, Operation, default_op_policy};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, Debug, Default)]
pub struct File {
    #[serde(default)]
    pub generate: GenerateFile,
    #[serde(default)]
    pub histogram: HistogramFile,
}

#[derive(Deserialize, Debug, Default)]
pub struct GenerateFile {
    pub n: Option<usize>,
    pub seed: Option<u64>,
    pub op_policy: Option<OpPolicy>,
    pub size_distribution: Option<SizeDist>,
}

#[derive(Deserialize, Debug, Default)]
pub struct HistogramFile {
    pub n: Option<usize>,
    pub trials: Option<usize>,
    pub seed: Option<u64>,
    pub op_policy: Option<OpPolicy>,
    pub size_distribution: Option<SizeDist>,
    pub max_solutions: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum OpPolicy {
    #[default]
    Default,
}

impl OpPolicy {
    pub fn func(self) -> fn(&[N], Index) -> Operation {
        match self {
            Self::Default => default_op_policy,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SizeDist {
    Fixed { size: usize },
    Uniform { min: usize, max: usize },
}

impl From<SizeDist> for kenken::SizeDistribution {
    fn from(s: SizeDist) -> Self {
        match s {
            SizeDist::Fixed { size } => Self::Fixed(size),
            SizeDist::Uniform { min, max } => Self::Uniform { min, max },
        }
    }
}

impl From<kenken::SizeDistribution> for SizeDist {
    fn from(s: kenken::SizeDistribution) -> Self {
        match s {
            kenken::SizeDistribution::Fixed(size) => Self::Fixed { size },
            kenken::SizeDistribution::Uniform { min, max } => Self::Uniform { min, max },
        }
    }
}

pub fn load(path: Option<&Path>) -> Result<File> {
    let Some(path) = path else {
        return Ok(File::default());
    };
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading config {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing config {}", path.display()))
}
