mod build;

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
}

pub fn run() -> Result<(), MtreeError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { dir, output } => build::execute(&dir, output.as_deref())?,
    }
    Ok(())
}
