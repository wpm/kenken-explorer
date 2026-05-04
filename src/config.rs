use anyhow::{Context, Result};
use kenken::{Index, N, Operation, default_op_policy};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct File {
    #[serde(default)]
    pub generate: GenerateFile,
    #[serde(default)]
    pub histogram: HistogramFile,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct GenerateFile {
    pub n: Option<usize>,
    pub seed: Option<u64>,
    pub op_policy: Option<OpPolicy>,
    pub size_distribution: Option<SizeDist>,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct HistogramFile {
    pub n: Option<usize>,
    pub trials: Option<usize>,
    pub seed: Option<u64>,
    pub op_policy: Option<OpPolicy>,
    pub size_distribution: Option<SizeDist>,
    pub max_solutions: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default, PartialEq, clap::ValueEnum)]
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

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp_config(content: &str) -> NamedTempFile {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "{}", content).unwrap();
        tmp
    }

    #[test]
    fn load_none_returns_default() {
        assert_eq!(load(None).unwrap(), File::default());
    }

    #[test]
    fn load_parses_histogram_section() {
        let tmp = write_temp_config(
            r#"
[histogram]
n = 4
trials = 1000
seed = 42
max_solutions = 50

[histogram.size_distribution]
type = "uniform"
min = 1
max = 4
"#,
        );

        let file = load(Some(tmp.path())).unwrap();
        let h = &file.histogram;
        assert_eq!(h.n, Some(4));
        assert_eq!(h.trials, Some(1000));
        assert_eq!(h.seed, Some(42));
        assert_eq!(h.max_solutions, Some(50));
        assert_eq!(h.op_policy, None);
        assert!(matches!(h.size_distribution, Some(SizeDist::Uniform { min: 1, max: 4 })));
    }

    #[test]
    fn load_parses_generate_section() {
        let tmp = write_temp_config(
            r#"
[generate]
n = 6
seed = 7

[generate.size_distribution]
type = "fixed"
size = 3
"#,
        );

        let file = load(Some(tmp.path())).unwrap();
        let g = &file.generate;
        assert_eq!(g.n, Some(6));
        assert_eq!(g.seed, Some(7));
        assert_eq!(g.op_policy, None);
        assert!(matches!(g.size_distribution, Some(SizeDist::Fixed { size: 3 })));
    }

    #[test]
    fn load_missing_file_returns_error() {
        let msg = format!(
            "{:#}",
            load(Some(Path::new("/nonexistent/path/config.toml"))).unwrap_err()
        );
        assert!(msg.contains("reading config"));
    }

    #[test]
    fn load_invalid_toml_returns_error() {
        let tmp = write_temp_config("this is not valid toml = = =");
        let msg = format!("{:#}", load(Some(tmp.path())).unwrap_err());
        assert!(msg.contains("parsing config"));
    }
}
