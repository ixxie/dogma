use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// A decision's lifecycle, which it traverses exactly once.
///
/// There is deliberately no `Superseded` variant. Supersession is declared by
/// the *successor* (`supersedes: <id>`) and derived by following that link
/// backwards, so a decision's own file is never edited after it is settled.
/// Decisions are events: an accepted decision that a later one replaced was
/// still accepted at the time, and commits that cited it were correct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Proposed,
    Accepted,
    Rejected,
}

impl Status {
    /// Whether a citation of this decision satisfies the gate.
    pub fn satisfies_gate(self) -> bool {
        matches!(self, Status::Accepted)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Status::Proposed => "proposed",
            Status::Accepted => "accepted",
            Status::Rejected => "rejected",
        };
        f.write_str(name)
    }
}

/// The only structured thing in the whole system. Everything below the
/// frontmatter is prose the team writes however it likes.
#[derive(Debug, Clone, Deserialize)]
pub struct Frontmatter {
    pub status: Status,
    #[serde(default)]
    pub title: Option<String>,
    /// Id of a decision this one replaces.
    #[serde(default)]
    pub supersedes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub id: String,
    pub path: PathBuf,
    pub frontmatter: Frontmatter,
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
    if slug.is_empty() || slug.contains('/') || slug.contains('\\') || slug.starts_with('.') {
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

/// Every decision on disk, ordered by id — which orders them by date, since
/// the date is the id's prefix.
pub fn load_all(decisions_dir: &Path) -> Result<Vec<Decision>> {
    let mut found = Vec::new();
    if !decisions_dir.is_dir() {
        return Ok(found);
    }

    for year in read_dirs(decisions_dir)? {
        for month in read_dirs(&year)? {
            for entry in fs::read_dir(&month)
                .with_context(|| format!("reading {}", month.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let id = id_from_path(decisions_dir, &path)?;
                found.push(load(decisions_dir, &id)?);
            }
        }
    }

    found.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(found)
}

fn read_dirs(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    Ok(dirs)
}

/// The inverse of `path_for`, used when walking the directory.
fn id_from_path(decisions_dir: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(decisions_dir)
        .with_context(|| format!("{} is outside the decisions directory", path.display()))?;

    let parts: Vec<&str> = relative
        .components()
        .map(|c| c.as_os_str().to_str().unwrap_or_default())
        .collect();

    match parts.as_slice() {
        [year, month, file] => {
            let stem = file.strip_suffix(".md").unwrap_or(file);
            Ok(format!("{year}-{month}-{stem}"))
        }
        _ => bail!("{} is not at <decisions>/YY/MM/DD-slug.md", path.display()),
    }
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

    serde_yaml::from_str(&rest[..end])
        .context("expected `status: proposed | accepted | rejected`")
}

/// Turn a human title into the slug half of an id.
pub fn slugify(title: &str) -> Result<String> {
    let mut slug = String::new();
    let mut pending_hyphen = false;

    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_hyphen && !slug.is_empty() {
                slug.push('-');
            }
            pending_hyphen = false;
            slug.push(ch.to_ascii_lowercase());
        } else {
            pending_hyphen = true;
        }
    }

    if slug.is_empty() {
        bail!("title '{title}' has no characters usable in a slug");
    }
    Ok(slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_is_the_path() {
        assert_eq!(
            path_for(Path::new("d"), "26-08-24-session-lifetime").unwrap(),
            Path::new("d/26/08/24-session-lifetime.md")
        );
    }

    #[test]
    fn a_slug_may_contain_hyphens() {
        assert_eq!(path_for(Path::new("d"), "26-08-24-a-b-c").unwrap(), Path::new("d/26/08/24-a-b-c.md"));
    }

    #[test]
    fn malformed_ids_are_refused_rather_than_guessed_at() {
        for id in ["nope", "2026-08-24-x", "26-8-24-x", "26-08-24-", "26-08-24-a/b", "26-08-24-.x"] {
            assert!(path_for(Path::new("d"), id).is_err(), "expected '{id}' to be refused");
        }
    }

    #[test]
    fn path_and_id_round_trip() {
        let dir = Path::new("d");
        let path = path_for(dir, "26-08-24-session-lifetime").unwrap();
        assert_eq!(id_from_path(dir, &path).unwrap(), "26-08-24-session-lifetime");
    }

    #[test]
    fn frontmatter_reads_status_and_ignores_the_prose() {
        let raw = "---\nstatus: accepted\ntitle: Session lifetime\n---\n\nWhatever we like.\n";
        let parsed = parse_frontmatter(raw).unwrap();
        assert_eq!(parsed.status, Status::Accepted);
        assert!(parsed.status.satisfies_gate());
    }

    #[test]
    fn only_accepted_satisfies_the_gate() {
        assert!(!Status::Proposed.satisfies_gate());
        assert!(!Status::Rejected.satisfies_gate());
    }

    #[test]
    fn an_unknown_status_is_an_error_rather_than_a_silent_pass() {
        let raw = "---\nstatus: ratified\n---\n\nbody\n";
        assert!(parse_frontmatter(raw).is_err());
    }

    #[test]
    fn a_file_without_frontmatter_is_an_error() {
        assert!(parse_frontmatter("# Just a heading\n").is_err());
    }

    #[test]
    fn slugs_collapse_punctuation_and_lowercase() {
        assert_eq!(slugify("Session lifetime").unwrap(), "session-lifetime");
        assert_eq!(slugify("  Retry  policy: v2! ").unwrap(), "retry-policy-v2");
        assert!(slugify("???").is_err());
    }
}
