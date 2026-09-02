/// The scaffold written by `dogma new`.
///
/// The slots are chosen to be conspicuous when empty. A record that skips
/// "what else we considered" or "what would make us revisit" is visibly
/// hollow, which matters more now that decisions are often drafted by an
/// agent: a skipped human record is two terse lines you can spot, whereas a
/// skipped generated one is three fluent paragraphs that restate the change
/// and record no decision at all.
pub fn scaffold(title: &str) -> String {
    format!(
        "---\n\
         status: proposed\n\
         title: {title}\n\
         ---\n\
         \n\
         ## Context\n\
         \n\
         What forced a choice here? The situation, constraint, or problem —\n\
         not the solution.\n\
         \n\
         ## Decision\n\
         \n\
         What we are doing, stated so someone can disagree with it.\n\
         \n\
         ## Alternatives\n\
         \n\
         What else was on the table, and the reason each was not chosen. If\n\
         this section is empty, no decision was made — a preference was\n\
         recorded.\n\
         \n\
         ## Consequences\n\
         \n\
         What this makes easier, what it makes harder, and what would make us\n\
         revisit it.\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision::Status;

    #[test]
    fn a_scaffold_is_born_proposed_and_parses() {
        let raw = scaffold("Session lifetime");
        assert!(raw.starts_with("---\nstatus: proposed\n"));

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("26").join("08").join("24-session-lifetime.md");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &raw).unwrap();

        let decision = crate::decision::load(dir.path(), "26-08-24-session-lifetime").unwrap();
        assert_eq!(decision.frontmatter.status, Status::Proposed);
        assert_eq!(decision.frontmatter.title.as_deref(), Some("Session lifetime"));
    }
}
