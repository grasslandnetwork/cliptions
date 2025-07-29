//! Cliptions - Unified CLI Tool
//!
//! Single binary that provides all Cliptions functionality through subcommands.
//! Replaces the previous multiple binary approach with a cleaner single-file experience.

use clap::{Parser, Subcommand};

use cliptions_core::error::Result;
use cliptions_core::actions::generate_commitment::{GenerateCommitmentArgs, run as generate_commitment_run};
use cliptions_core::actions::collect_commitments::{CollectCommitmentsArgs, run as collect_commitments_run};
use cliptions_core::actions::post_target_frame::{PostTargetFrameArgs, run as post_target_frame_run};
use cliptions_core::actions::collect_reveals::{CollectRevealsArgs, run as collect_reveals_run};
use cliptions_core::actions::verify_commitments::{VerifyCommitmentsArgs, run as verify_commitments_run};
use cliptions_core::actions::calculate_scores::{CalculateScoresArgs, run as calculate_scores_run};

#[derive(Parser)]
#[command(name = "cliptions")]
#[command(about = "Cliptions - A CLIP-based prediction market")]
#[command(version = "0.7.0")]
#[command(long_about = "
Unified CLI tool for Cliptions prediction market operations.

This tool provides all functionality through subcommands:
- generate-commitment: Generate cryptographic commitments for predictions
- collect-commitments: Collect commitment replies from a specific tweet
- post-target-frame: Post target frame image as reply to commitment tweet
- collect-reveals: Collect reveal replies from target frame tweet
- verify-commitments: Verify commitments against reveals for a block
- calculate-scores: Calculate scores and payouts for verified participants

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
    
    /// Collect commitment replies from a specific tweet
    #[command(name = "collect-commitments")]
    CollectCommitments(CollectCommitmentsArgs),
    
    /// Post target frame image as reply to commitment tweet
    #[command(name = "post-target-frame")]
    PostTargetFrame(PostTargetFrameArgs),
    
    /// Collect reveal replies from target frame tweet
    #[command(name = "collect-reveals")]
    CollectReveals(CollectRevealsArgs),
    
    /// Verify commitments for prediction blocks
    #[command(name = "verify-commitments")]
    VerifyCommitments(VerifyCommitmentsArgs),
    
    /// Calculate scores and payouts for verified participants
    #[command(name = "calculate-scores")]
    CalculateScores(CalculateScoresArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateCommitment(args) => generate_commitment_run(args),
        Commands::CollectCommitments(args) => {
            tokio::runtime::Runtime::new()?.block_on(collect_commitments_run(args))
        }
        Commands::PostTargetFrame(args) => {
            tokio::runtime::Runtime::new()?.block_on(post_target_frame_run(args))
        }
        Commands::CollectReveals(args) => {
            tokio::runtime::Runtime::new()?.block_on(collect_reveals_run(args))
        }
        Commands::VerifyCommitments(args) => {
            tokio::runtime::Runtime::new()?.block_on(verify_commitments_run(args))
        }
        Commands::CalculateScores(args) => calculate_scores_run(args),
    }
} 