use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobMatcher};

/// Decides whether a path's changes require a cited decision.
///
/// Patterns are evaluated in order and the last match wins, the same rule
/// `.gitignore` uses, so a team can carve exceptions out of a broad sweep:
///
/// ```toml
/// enforce = ["**", "!vendor/**", "!**/*.generated.rs"]
/// ```
pub struct Enforcement {
    rules: Vec<Rule>,
    decisions_dir: PathBuf,
}

struct Rule {
    matcher: GlobMatcher,
    /// True for a `!pattern`, which exempts rather than enforces.
    exempts: bool,
}

impl Enforcement {
    /// `decisions_dir` is relative to the repository root, as are the paths
    /// passed to [`covers`](Self::covers).
    pub fn new(patterns: &[String], decisions_dir: PathBuf) -> Result<Self> {
        let mut rules = Vec::with_capacity(patterns.len());

        for pattern in patterns {
            let (body, exempts) = match pattern.strip_prefix('!') {
                Some(rest) => (rest, true),
                None => (pattern.as_str(), false),
            };
            let matcher = Glob::new(body)
                .with_context(|| format!("`enforce` pattern '{pattern}' is not a valid glob"))?
                .compile_matcher();
            rules.push(Rule { matcher, exempts });
        }

        Ok(Self { rules, decisions_dir })
    }

    pub fn covers(&self, path: &Path) -> bool {
        // Never enforceable, whatever the config says. A repository that
        // enforced its own decisions directory could not add a decision: the
        // commit would need to cite an accepted one, and a new decision is
        // born `proposed`. Leaving this to the user to remember would make
        // permanent lockout a one-line config mistake.
        if path.starts_with(&self.decisions_dir) {
            return false;
        }

        let mut enforced = false;
        for rule in &self.rules {
            if rule.matcher.is_match(path) {
                enforced = !rule.exempts;
            }
        }
        enforced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enforcement(patterns: &[&str]) -> Enforcement {
        let owned: Vec<String> = patterns.iter().map(|p| p.to_string()).collect();
        Enforcement::new(&owned, PathBuf::from(".dogma/decisions")).unwrap()
    }

    #[test]
    fn a_matching_path_is_enforced_and_others_are_not() {
        let rules = enforcement(&["specs/**"]);
        assert!(rules.covers(Path::new("specs/auth.md")));
        assert!(rules.covers(Path::new("specs/billing/invoices.md")));
        assert!(!rules.covers(Path::new("src/main.rs")));
    }

    #[test]
    fn a_later_exemption_carves_out_of_an_earlier_sweep() {
        let rules = enforcement(&["specs/**", "!specs/drafts/**"]);
        assert!(rules.covers(Path::new("specs/auth.md")));
        assert!(!rules.covers(Path::new("specs/drafts/idea.md")));
    }

    #[test]
    fn order_decides_when_patterns_overlap() {
        // Re-including after an exemption is legal and the last word wins.
        let rules = enforcement(&["specs/**", "!specs/drafts/**", "specs/drafts/final.md"]);
        assert!(!rules.covers(Path::new("specs/drafts/idea.md")));
        assert!(rules.covers(Path::new("specs/drafts/final.md")));
    }

    #[test]
    fn enforcing_everything_is_a_legitimate_configuration() {
        let rules = enforcement(&["**", "!vendor/**"]);
        assert!(rules.covers(Path::new("src/main.rs")));
        assert!(rules.covers(Path::new("README.md")));
        assert!(!rules.covers(Path::new("vendor/thing.rs")));
    }

    #[test]
    fn the_decisions_directory_is_never_enforced_however_broad_the_config() {
        let rules = enforcement(&["**"]);
        assert!(!rules.covers(Path::new(".dogma/decisions/26/09/02-a-thing.md")));
    }

    #[test]
    fn even_an_explicit_attempt_to_enforce_decisions_is_refused() {
        let rules = enforcement(&[".dogma/decisions/**"]);
        assert!(!rules.covers(Path::new(".dogma/decisions/26/09/02-a-thing.md")));
    }

    #[test]
    fn a_malformed_pattern_is_an_error_rather_than_a_silent_no_match() {
        let bad = vec!["specs/[".to_string()];
        assert!(Enforcement::new(&bad, PathBuf::from(".dogma/decisions")).is_err());
    }
}
