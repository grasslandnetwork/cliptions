//! Cliptions - Unified CLI Tool
//!
//! Single binary that provides all Cliptions functionality through subcommands.
//! Replaces the previous multiple binary approach with a cleaner single-file experience.

use clap::{Parser, Subcommand};

use cliptions_core::error::Result;
use cliptions_core::actions::generate_commitment::{GenerateCommitmentArgs, run as generate_commitment_run};

#[derive(Parser)]
#[command(name = "cliptions")]
#[command(about = "Cliptions - A CLIP-based prediction market")]
#[command(version = "0.6.1")]
#[command(long_about = "
Unified CLI tool for Cliptions prediction market operations.

This tool provides all functionality through subcommands:
- generate-commitment: Generate cryptographic commitments for predictions

Use 'cliptions <SUBCOMMAND> --help' for detailed help on each command.
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate cryptographic commitments for predictions
    #[command(name = "generate-commitment")]
    GenerateCommitment(GenerateCommitmentArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateCommitment(args) => generate_commitment_run(args),
    }
} 