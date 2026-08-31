use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Where a project keeps its decisions and the paths whose changes need one.
///
/// This is declarative input, not derived state — it says nothing that could
/// fall out of date with the repository, only what the team chose. Every
/// field has a default, so a project with no config file still works.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// Directory holding decision records, relative to the repo root.
    pub decisions: PathBuf,
    /// Path globs whose modification requires citing an accepted decision.
    pub guarded: Vec<String>,
    /// Commit trailer key used to cite a decision.
    pub trailer: String,
    /// Decision statuses that satisfy the gate.
    pub accepted_states: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            decisions: PathBuf::from("dogma/decisions"),
            guarded: vec!["dogma/specs/**".to_string()],
            trailer: "Decision".to_string(),
            accepted_states: vec!["accepted".to_string()],
        }
    }
}

/// Config file locations, in the order they are tried. The first that exists
/// wins; finding none is not an error, because the defaults are usable.
const CANDIDATES: [&str; 3] = ["dogma.toml", "dogma/config.toml", ".dogma/config.toml"];

impl Config {
    pub fn load(root: &Path) -> Result<Self> {
        for candidate in CANDIDATES {
            let path = root.join(candidate);
            if !path.is_file() {
                continue;
            }
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            let config: Config = toml::from_str(&raw)
                .with_context(|| format!("parsing {}", path.display()))?;
            return Ok(config);
        }
        Ok(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_apply_when_no_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load(dir.path()).unwrap();
        assert_eq!(config.trailer, "Decision");
        assert_eq!(config.decisions, PathBuf::from("dogma/decisions"));
    }

    #[test]
    fn a_partial_file_keeps_the_remaining_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("dogma.toml"), "trailer = \"Because\"\n").unwrap();

        let config = Config::load(dir.path()).unwrap();
        assert_eq!(config.trailer, "Because");
        assert_eq!(config.accepted_states, vec!["accepted".to_string()]);
    }

    #[test]
    fn an_unknown_key_is_an_error_rather_than_silently_ignored() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("dogma.toml"), "decisionz = \"typo\"\n").unwrap();

        assert!(Config::load(dir.path()).is_err());
    }
}
