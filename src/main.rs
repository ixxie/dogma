mod config;
mod decision;
mod paths;
mod template;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use config::Config;

/// `--help` is the documentation. This tool writes nothing into a consuming
/// repository — no generated instruction files, no per-editor adapters — so
/// there is nowhere else for its behaviour to be described.
#[derive(Parser)]
#[command(
    name = "dogma",
    version,
    about = "Decision records linked to the changes they justify.",
    long_about = "Enforces one rule: a commit that changes an enforced path must cite an \
accepted decision. Everything it reports is derived from git and the working tree at \
the moment you ask — it stores no state of its own."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a decision, dated today, with status: proposed
    New {
        /// Short title, e.g. "session lifetime"
        title: String,
    },

    /// List decisions, oldest first
    List,

    /// The gate: enforced changes cite an accepted decision
    ///
    /// Also verifies that decisions are well-formed and that every `enforce`
    /// pattern matches something. Exits 1 on a violation, 2 on a usage error.
    Check {
        /// Commit range, e.g. main..HEAD. Defaults to the CI merge base when
        /// one is available, otherwise origin/HEAD..HEAD.
        range: Option<String>,
    },

    /// Where the record and reality have come apart
    ///
    /// Accepted decisions nothing implements, and enforced files with no
    /// decision behind them. Always exits 0 — this is a report, never a gate.
    Gaps,

    /// What decided this
    ///
    /// With a line, blames that line. Without one, lists every decision that
    /// shaped the file.
    Why {
        /// File, optionally with a line: dogma/specs/auth.md:42
        location: String,
    },

    /// What this decided — every commit and file it caused
    Impact {
        /// Decision id, e.g. 26-08-24-session-lifetime
        id: String,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(2);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let root = repo_root()?;
    let config = Config::load(&root)?;

    match cli.command {
        Command::New { title } => new_decision(&root, &config, &title),
        Command::List => todo!("list decisions"),
        Command::Check { range } => todo!("check range {range:?}"),
        Command::Gaps => todo!("report gaps"),
        Command::Why { location } => todo!("explain {location}"),
        Command::Impact { id } => todo!("impact of {id}"),
    }
}

/// The repository root, so every path the tool prints or reads is anchored to
/// the same place regardless of which subdirectory it was invoked from.
fn repo_root() -> Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git — is it on PATH?")?;

    if !output.status.success() {
        bail!("not inside a git repository");
    }
    let path = String::from_utf8(output.stdout).context("git printed non-UTF-8 output")?;
    Ok(PathBuf::from(path.trim()))
}

fn new_decision(root: &Path, config: &Config, title: &str) -> Result<()> {
    let slug = decision::slugify(title)?;
    let today = chrono::Local::now().date_naive();
    let id = format!("{}-{slug}", today.format("%y-%m-%d"));

    let decisions_dir = config.decisions_dir(root);
    let path = decision::path_for(&decisions_dir, &id)?;
    if path.exists() {
        bail!("{id} already exists at {}", path.display());
    }

    fs::create_dir_all(path.parent().expect("decision paths always have a parent"))
        .with_context(|| format!("creating {}", path.display()))?;
    fs::write(&path, template::scaffold(title))
        .with_context(|| format!("writing {}", path.display()))?;

    let shown = path.strip_prefix(root).unwrap_or(&path);
    println!("{}", shown.display());
    println!();
    println!("Cite it from the commit that acts on it:");
    println!("    {}: {id}", config.trailer);

    Ok(())
}
