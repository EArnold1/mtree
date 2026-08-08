mod build;
mod diff;
mod verify;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::error::MtreeError;

#[derive(Debug, Parser)]
#[command(name = "mtree")]
#[command(version = "0.1.0")]
#[command(about = "Directory integrity verification")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Build a directory snapshot.
    #[command(alias = "b")]
    Build {
        /// Directory to snapshot.
        dir: PathBuf,
        /// Optional path to write snapshot JSON. If omitted, writes to stdout.
        output: Option<PathBuf>,
    },
    /// Compares a directory with provided snapshot
    #[command(alias = "v")]
    Verify {
        /// Live directory.
        live_dir: PathBuf,
        /// Path to the snapshot JSON file.
        baseline_snapshot: PathBuf,
    },
    /// Compares two snapshots and reports file-level changes.
    #[command(alias = "d")]
    Diff {
        /// Path to the baseline snapshot JSON file.
        baseline_snapshot: PathBuf,
        /// Path to the snapshot JSON file being compared with the baseline.
        comparison_snapshot: PathBuf,
    },
}

pub fn run() -> Result<(), MtreeError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { dir, output } => build::execute(&dir, output.as_deref())?,
        Commands::Verify {
            live_dir,
            baseline_snapshot,
        } => verify::execute(&live_dir, &baseline_snapshot)?,
        Commands::Diff {
            baseline_snapshot,
            comparison_snapshot,
        } => diff::execute(&baseline_snapshot, &comparison_snapshot)?,
    }
    Ok(())
}
