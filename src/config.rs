use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Where a project keeps its decisions, and which paths may not change
/// without citing one.
///
/// This is declarative input, not derived state — it records what the team
/// chose, never anything that could fall out of step with the repository.
/// Every field has a default, so a project with no config file works.
///
/// Note what is *not* configurable: the decision statuses, and which of them
/// satisfies the gate. If those varied per repo, `dogma check` would mean
/// something different in each one.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// Directory holding decision records, relative to the repo root.
    pub decisions: PathBuf,
    /// Path globs whose modification requires citing an accepted decision.
    pub guarded: Vec<String>,
    /// Commit trailer key used to cite a decision.
    pub trailer: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            decisions: PathBuf::from(".dogma/decisions"),
            guarded: vec!["specs/**".to_string()],
            trailer: "Decision".to_string(),
        }
    }
}

/// Config locations, in the order tried. Finding none is not an error,
/// because the defaults are usable. `.dogma/` keeps the tool's own files
/// together; `dogma.toml` at the root suits teams who prefer config there.
const CANDIDATES: [&str; 2] = [".dogma/config.toml", "dogma.toml"];

impl Config {
    pub fn load(root: &Path) -> Result<Self> {
        for candidate in CANDIDATES {
            let path = root.join(candidate);
            if !path.is_file() {
                continue;
            }
            let raw =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            return toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()));
        }
        Ok(Config::default())
    }

    pub fn decisions_dir(&self, root: &Path) -> PathBuf {
        root.join(&self.decisions)
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
        assert_eq!(config.decisions, PathBuf::from(".dogma/decisions"));
    }

    #[test]
    fn a_partial_file_keeps_the_remaining_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("dogma.toml"), "trailer = \"Because\"\n").unwrap();

        let config = Config::load(dir.path()).unwrap();
        assert_eq!(config.trailer, "Because");
        assert_eq!(config.guarded, vec!["specs/**".to_string()]);
    }

    #[test]
    fn an_unknown_key_is_an_error_rather_than_silently_ignored() {
        // A typo in `guarded` would otherwise disable the gate while leaving
        // it green, which is the worst failure a gate can have.
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("dogma.toml"), "guardd = [\"specs/**\"]\n").unwrap();

        assert!(Config::load(dir.path()).is_err());
    }
}
