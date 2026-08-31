use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// The only structured thing in the whole system: a decision's frontmatter.
/// Everything below it is prose the team writes however it likes.
#[derive(Debug, Clone, Deserialize)]
pub struct Frontmatter {
    pub status: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub supersedes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub id: String,
    pub path: PathBuf,
    pub frontmatter: Frontmatter,
}

impl Decision {
    pub fn is_accepted(&self, accepted_states: &[String]) -> bool {
        accepted_states.iter().any(|state| state == &self.frontmatter.status)
    }
}

/// Map an id to its file. The id *is* the path: `26-08-24-foo` lives at
/// `<decisions>/26/08/24-foo.md`, so resolution is a split rather than a
/// lookup table that could disagree with the filesystem.
pub fn path_for(decisions_dir: &Path, id: &str) -> Result<PathBuf> {
    let parts: Vec<&str> = id.splitn(4, '-').collect();
    if parts.len() != 4 {
        bail!("decision id '{id}' should look like YY-MM-DD-slug");
    }
    let (year, month, day, slug) = (parts[0], parts[1], parts[2], parts[3]);

    let two_digits = |s: &str| s.len() == 2 && s.bytes().all(|b| b.is_ascii_digit());
    if !two_digits(year) || !two_digits(month) || !two_digits(day) {
        bail!("decision id '{id}' should start with a YY-MM-DD date");
    }
    if slug.is_empty() || slug.contains('/') || slug.contains('\\') {
        bail!("decision id '{id}' has an unusable slug");
    }

    Ok(decisions_dir.join(year).join(month).join(format!("{day}-{slug}.md")))
}

pub fn load(decisions_dir: &Path, id: &str) -> Result<Decision> {
    let path = path_for(decisions_dir, id)?;
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("no decision '{id}' at {}", path.display()))?;
    let frontmatter = parse_frontmatter(&raw)
        .with_context(|| format!("reading frontmatter of {}", path.display()))?;

    Ok(Decision { id: id.to_string(), path, frontmatter })
}

/// Frontmatter is a leading `---` fenced YAML block, as in every static site
/// generator. Anything after it is the team's own business.
fn parse_frontmatter(raw: &str) -> Result<Frontmatter> {
    let rest = raw
        .strip_prefix("---\n")
        .or_else(|| raw.strip_prefix("---\r\n"))
        .ok_or_else(|| anyhow::anyhow!("missing leading `---` frontmatter block"))?;

    let end = rest
        .find("\n---")
        .ok_or_else(|| anyhow::anyhow!("frontmatter block is never closed with `---`"))?;

    let yaml = &rest[..end];
    serde_yaml::from_str(yaml).context("frontmatter is not valid YAML with a `status` field")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_is_the_path() {
        let dir = Path::new("dogma/decisions");
        assert_eq!(
            path_for(dir, "26-08-24-session-lifetime").unwrap(),
            Path::new("dogma/decisions/26/08/24-session-lifetime.md")
        );
    }

    #[test]
    fn a_slug_may_contain_hyphens() {
        let path = path_for(Path::new("d"), "26-08-24-a-b-c").unwrap();
        assert_eq!(path, Path::new("d/26/08/24-a-b-c.md"));
    }

    #[test]
    fn malformed_ids_are_refused_rather_than_guessed_at() {
        for id in ["nope", "2026-08-24-x", "26-8-24-x", "26-08-24-", "26-08-24-a/b"] {
            assert!(path_for(Path::new("d"), id).is_err(), "expected '{id}' to be refused");
        }
    }

    #[test]
    fn frontmatter_reads_status_and_ignores_the_prose() {
        let raw = "---\nstatus: accepted\ntitle: Session lifetime\n---\n\nWhatever we like.\n";
        let parsed = parse_frontmatter(raw).unwrap();
        assert_eq!(parsed.status, "accepted");
        assert_eq!(parsed.title.as_deref(), Some("Session lifetime"));
    }

    #[test]
    fn a_file_without_frontmatter_is_an_error() {
        assert!(parse_frontmatter("# Just a heading\n").is_err());
    }
}
