mod config;
mod decision;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// `--help` is the documentation: this tool generates no files into your
/// repository and installs no instruction files, so the only place its
/// behaviour is described is here.
#[derive(Parser)]
#[command(
    name = "dogma",
    version,
    about = "Decision records linked to the changes they justify.",
    long_about = "Records decisions, and enforces that changes to guarded paths cite an \
accepted one. Everything it reports is derived from git and the working tree at the \
moment you ask — it stores no state of its own."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Work with decision records
    #[command(subcommand)]
    Decision(DecisionCommand),

    /// Verify that commits touching guarded paths cite an accepted decision
    Check {
        /// Commit range, e.g. main..HEAD. Defaults to the CI merge base when
        /// one is available, otherwise origin/HEAD..HEAD.
        range: Option<String>,
    },

    /// Show the decision behind a line
    Why {
        /// File and line, e.g. dogma/specs/auth.md:42
        location: String,
    },

    /// Show everything a decision caused
    Impact {
        /// Decision id, e.g. 26-08-24-session-lifetime
        id: String,
    },

    /// List accepted decisions that nothing implements
    Unbuilt,
}

#[derive(Subcommand)]
enum DecisionCommand {
    /// Create a decision, dated today, with status: proposed
    New {
        /// Short title, e.g. "session lifetime"
        title: String,
    },
    /// List decisions and their statuses
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Decision(DecisionCommand::New { title }) => {
            todo!("scaffold a decision titled {title:?}")
        }
        Command::Decision(DecisionCommand::List) => todo!("list decisions"),
        Command::Check { range } => todo!("check range {range:?}"),
        Command::Why { location } => todo!("explain {location}"),
        Command::Impact { id } => todo!("impact of {id}"),
        Command::Unbuilt => todo!("accepted but unimplemented"),
    }
}
