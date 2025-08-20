//! Calculate Scores & Payouts for Cliptions blocks
//!
//! This module implements the `calculate-scores` subcommand, which processes verified participants,
//! calculates CLIP similarity scores, determines payouts, and updates the block state.
//!
//! See MVP Slice 6 (v0.6.6) for requirements.

use std::path::PathBuf;
use clap::Parser;
use colored::Colorize;

use crate::embedder::ClipEmbedder;
use crate::scoring::ClipBatchStrategy;
use crate::types::{Participant, ScoringResult};
use crate::error::Result;
use crate::block_engine::store::{JsonBlockStore, BlockStore};
use crate::block_engine::state_machine::{Block, CommitmentsOpen, Payouts, Finished};
use crate::scoring::{ScoreValidator, process_participants};
// Note: PayoutCalculator and PayoutConfig are imported for future use
// when we integrate the payout system more deeply
// use crate::payout::{PayoutCalculator, PayoutConfig};

/// Command-line arguments for calculate-scores
#[derive(Parser)]
#[command(name = "calculate-scores")]
#[command(about = "Calculate scores and payouts for verified participants in a block")]
#[command(version = "0.6.6")]
#[command(long_about = "
Calculate similarity scores and payout distribution for verified participants in a Cliptions block.

This subcommand processes verified participants from a block, calculates CLIP similarity scores
against the target image, determines fair payouts based on rankings, and updates the block state.

Examples:
  # Calculate scores for block1 with default settings
  cliptions calculate-scores --block-num block1 --prize-pool 1000.0
  
  # Save results to JSON file
  cliptions calculate-scores --block-num block1 --prize-pool 1000.0 --output json --output-file results.json
")]
pub struct CalculateScoresArgs {
    /// Block ID to calculate scores for
    #[arg(short, long)]
    pub block_num: String,
    
    /// DEPRECATED: path to blocks data file (store uses PathManager)
    #[arg(short = 'f', long, default_value = "data/blocks.json")]
    pub blocks_file: String,
    
    /// Prize pool amount (must be positive)
    #[arg(short, long)]
    pub prize_pool: f64,
    
    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table", value_parser = ["table", "json", "csv"])]
    pub output: String,
    
    /// Output file path (optional)
    #[arg(short = 'w', long)]
    pub output_file: Option<PathBuf>,
    
    // Mock embedder support removed. Always use CLIP.
    
    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Load verified participants from blocks data
fn load_verified_participants_from_store(block_num: &str) -> Result<(Block<CommitmentsOpen>, Vec<Participant>, String, f64)> {
    let store = JsonBlockStore::new()?;
    let block: Block<CommitmentsOpen> = store.load_commitments_open(block_num)?;
    
    let verified: Vec<Participant> = block
        .participants
        .iter()
        .cloned()
        .filter(|p| p.verified)
        .collect();
    if verified.is_empty() {
        return Err(crate::error::CliptionsError::ValidationError(
            format!("No verified participants found for block {}", block_num),
        ));
    }

    let image_path = block
        .target_frame_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| crate::error::CliptionsError::ValidationError("Target frame path not set on block".to_string()))?;

    let prize = block.prize_pool;
    Ok((block, verified, image_path, prize))
}

/// Calculate scores and payouts for participants using the PayoutCalculator
fn calculate_scores_and_payouts(
    participants: &[Participant],
    image_path: &str,
    prize_pool: f64,
    verbose: bool,
) -> Result<Vec<ScoringResult>> {
    let strategy = ClipBatchStrategy::new();
    let clip_embedder = ClipEmbedder::new().map_err(|e| format!("Failed to load CLIP model: {}", e))?;
    let validator = ScoreValidator::new(clip_embedder, strategy);
    if verbose {
        println!("Processing {} participants against target image: {}", participants.len(), image_path);
    }
    let results = process_participants(participants, image_path, prize_pool, &validator)?;
    if verbose {
        println!("Successfully calculated scores and payouts for {} participants", results.len());
    }
    Ok(results)
}

/// Update the blocks.json file with calculated scores, payouts, and prize pool
fn update_block_in_store(block: &mut Block<CommitmentsOpen>, results: &[ScoringResult], verbose: bool) -> Result<()> {
    // Update participant scores/payouts
    for res in results {
        if let Some(idx) = block
            .participants
            .iter()
            .position(|p| p.social_id == res.participant.social_id)
        {
            block.participants[idx].score = res.raw_score;
            if let Some(p) = res.payout { block.participants[idx].payout.amount = p; }
            if let Some(rank) = res.rank {
                // Store rank in metadata for now (legacy BlockData used results vec; typestate not yet)
                block.participants[idx].guess.metadata.insert("rank".to_string(), rank.to_string());
            }
        }
    }
    // Update totals
    block.total_payout = results.iter().filter_map(|r| r.payout).sum();

    let store = JsonBlockStore::new()?;
    store.save(block)?;
    if verbose {
        println!("Updated store with scores and payouts. Total distributed: {:.9} TAO", block.total_payout);
    }
    Ok(())
}

/// Display results in the specified format
fn display_results(results: &[ScoringResult], args: &CalculateScoresArgs) -> Result<()> {
    match args.output.as_str() {
        "table" => display_table_format(results, args),
        "json" => display_json_format(results),
        "csv" => display_csv_format(results),
        _ => Err(crate::error::CliptionsError::ValidationError(
            format!("Unsupported output format: {}", args.output)
        )),
    }
}

/// Display results in table format
fn display_table_format(results: &[ScoringResult], args: &CalculateScoresArgs) -> Result<()> {
    println!("\n{}", "Block Results:".bold().underline());
    println!("{}", "=".repeat(80));
    println!("Block ID: {}", args.block_num.bold().blue());
    println!("Prize Pool: {:.9} TAO", args.prize_pool);
    println!("{}", "=".repeat(80));

    for (i, result) in results.iter().enumerate() {
        let rank = format!("#{}", i + 1);
        println!("{} {}", rank.bold().blue(), result.participant.username.bright_white());
        println!("   Guess: \"{}\"", result.participant.guess.text);
        println!("   Similarity Score: {:.6}", result.raw_score);
        if let Some(rank_num) = result.rank {
            println!("   Rank: {}", rank_num);
        }
        println!("   Payout: {:.9} TAO", result.payout.unwrap_or(0.0));

        if i == 0 {
            println!("   Status: {}", "🏆 Winner".green().bold());
        } else if i < 3 {
            println!("   Status: {}", "🥉 Top 3".yellow());
        }
        println!();
    }

    println!("{}", "=".repeat(80));
    let total_payout: f64 = results.iter().filter_map(|r| r.payout).sum();
    println!("Total Distributed: {:.9} TAO", total_payout);
    
    let efficiency = (total_payout / args.prize_pool) * 100.0;
    println!("Distribution Efficiency: {:.2}%", efficiency);

    Ok(())
}

/// Display results in JSON format
fn display_json_format(results: &[ScoringResult]) -> Result<()> {
    let mut output = serde_json::Map::new();
    
    let rankings: Vec<serde_json::Value> = results.iter().map(|result| {
        serde_json::json!({
            "rank": result.rank,
            "username": result.participant.username,
            "guess": result.participant.guess.text,
            "similarity_score": result.raw_score,
            "payout": result.payout
        })
    }).collect();

    output.insert("rankings".to_string(), serde_json::Value::Array(rankings));
    output.insert("num_participants".to_string(), serde_json::Value::from(results.len()));
    output.insert("timestamp".to_string(), serde_json::Value::from(chrono::Utc::now().to_rfc3339()));

    let json_output = serde_json::to_string_pretty(&output).map_err(|e| crate::error::CliptionsError::Json(e))?;
    println!("{}", json_output);

    Ok(())
}

/// Display results in CSV format
fn display_csv_format(results: &[ScoringResult]) -> Result<()> {
    println!("rank,username,guess,similarity_score,payout");

    for result in results {
        let escaped_guess = result.participant.guess.text.replace("\"", "\"\"");
        println!(
            "{},\"{}\",\"{}\",{:.6},{:.9}",
            result.rank.unwrap_or(0),
            result.participant.username,
            escaped_guess,
            result.raw_score,
            result.payout.unwrap_or(0.0)
        );
    }

    Ok(())
}

/// Save results to file
fn save_results(results: &[ScoringResult], output_file: &PathBuf, format: &str) -> Result<()> {
    let content = match format {
        "json" => {
            let mut output = serde_json::Map::new();
            
            let rankings: Vec<serde_json::Value> = results.iter().map(|result| {
                serde_json::json!({
                    "rank": result.rank,
                    "username": result.participant.username,
                    "guess": result.participant.guess.text,
                    "similarity_score": result.raw_score,
                    "payout": result.payout
                })
            }).collect();

            output.insert("rankings".to_string(), serde_json::Value::Array(rankings));
            output.insert("num_participants".to_string(), serde_json::Value::from(results.len()));
            output.insert("timestamp".to_string(), serde_json::Value::from(chrono::Utc::now().to_rfc3339()));

            serde_json::to_string_pretty(&output).map_err(|e| crate::error::CliptionsError::Json(e))?
        }
        "csv" => {
            let mut content = String::from("rank,username,guess,similarity_score,payout\n");

            for result in results {
                let escaped_guess = result.participant.guess.text.replace("\"", "\"\"");
                content.push_str(&format!(
                    "{},\"{}\",\"{}\",{:.6},{:.9}\n",
                    result.rank.unwrap_or(0),
                    result.participant.username,
                    escaped_guess,
                    result.raw_score,
                    result.payout.unwrap_or(0.0)
                ));
            }

            content
        }
        "table" => {
            let mut content = String::from("Block Results\n");
            content.push_str(&"=".repeat(50));
            content.push('\n');

            for result in results {
                content.push_str(&format!("{}. {}\n", result.rank.unwrap_or(0), result.participant.username));
                content.push_str(&format!("   Guess: \"{}\"\n", result.participant.guess.text));
                content.push_str(&format!("   Similarity score: {:.4}\n", result.raw_score));
                content.push_str(&format!("   Payout: {:.9}\n\n", result.payout.unwrap_or(0.0)));
            }

            content
        }
        _ => return Err(crate::error::CliptionsError::ValidationError(
            format!("Unsupported output format for file save: {}", format)
        )),
    };

    std::fs::write(output_file, content).map_err(|e| crate::error::CliptionsError::Io(e))?;
    Ok(())
}

/// Entry point for the calculate-scores subcommand
pub fn run(args: CalculateScoresArgs) -> Result<()> {
    // Load block and verified participants via store
    let (mut block, participants, image_path, prize_pool_from_block) = load_verified_participants_from_store(&args.block_num)?;

    // Use provided prize_pool arg if > 0 else fallback to block value
    let prize_pool = if args.prize_pool > 0.0 { args.prize_pool } else { prize_pool_from_block };

    if args.verbose {
        println!("Loaded {} verified participants for block {}", participants.len(), args.block_num);
    }

    // Calculate scores and payouts
    let results = calculate_scores_and_payouts(
        &participants,
        &image_path,
        prize_pool,
        args.verbose,
    )?;

    if args.verbose {
        println!("Successfully processed block {} with {} results", args.block_num, results.len());
    }

    // Display results
    display_results(&results, &args)?;

    // Save results to file if requested
    if let Some(output_file) = &args.output_file {
        save_results(&results, output_file, &args.output)?;
        if args.verbose {
            println!("Results saved to {}", output_file.display());
        }
    }

    // Persist into store: update participant fields and total_payout
    update_block_in_store(&mut block, &results, args.verbose)?;

    Ok(())
}