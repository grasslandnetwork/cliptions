//! Calculate Scores & Payouts for Cliptions rounds
//!
//! This module implements the `calculate-scores` subcommand, which processes verified participants,
//! calculates CLIP similarity scores, determines payouts, and updates the round state.
//!
//! See MVP Slice 6 (v0.6.6) for requirements.

use std::path::PathBuf;
use clap::Parser;
use colored::Colorize;

use crate::embedder::{ClipEmbedder, MockEmbedder};
use crate::round_processor::RoundProcessor;
use crate::scoring::ClipBatchStrategy;
use crate::types::{Participant, ScoringResult};
use crate::error::Result;
// Note: PayoutCalculator and PayoutConfig are imported for future use
// when we integrate the payout system more deeply
// use crate::payout::{PayoutCalculator, PayoutConfig};

/// Command-line arguments for calculate-scores
#[derive(Parser)]
#[command(name = "calculate-scores")]
#[command(about = "Calculate scores and payouts for verified participants in a round")]
#[command(version = "0.6.6")]
#[command(long_about = "
Calculate similarity scores and payout distribution for verified participants in a Cliptions round.

This subcommand processes verified participants from a round, calculates CLIP similarity scores
against the target image, determines fair payouts based on rankings, and updates the round state.

Examples:
  # Calculate scores for round1 with default settings
  cliptions calculate-scores --round-id round1 --prize-pool 1000.0
  
  # Use MockEmbedder for testing
  cliptions calculate-scores --round-id round1 --prize-pool 1000.0 --use-mock
  
  # Save results to JSON file
  cliptions calculate-scores --round-id round1 --prize-pool 1000.0 --output json --output-file results.json
")]
pub struct CalculateScoresArgs {
    /// Round ID to calculate scores for
    #[arg(short, long)]
    pub round_id: String,
    
    /// Path to rounds data file
    #[arg(short = 'f', long, default_value = "data/rounds.json")]
    pub rounds_file: String,
    
    /// Prize pool amount (must be positive)
    #[arg(short, long)]
    pub prize_pool: f64,
    
    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table", value_parser = ["table", "json", "csv"])]
    pub output: String,
    
    /// Output file path (optional)
    #[arg(short = 'w', long)]
    pub output_file: Option<PathBuf>,
    
    /// Use MockEmbedder for testing
    #[arg(long)]
    pub use_mock: bool,
    
    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Load verified participants from rounds data
fn load_verified_participants(round_id: &str, rounds_file: &str) -> Result<Vec<Participant>> {
    // Create embedder and processor
    let embedder = MockEmbedder::clip_like(); // We'll replace this with real embedder later
    let strategy = ClipBatchStrategy::new();
    let mut processor = RoundProcessor::new(rounds_file.to_string(), embedder, strategy);
    
    // Load rounds data
    processor.load_rounds()?;
    
    // Get the round and extract verified participants
    let round = processor.get_round(round_id)?;
    let verified_participants: Vec<Participant> = round
        .participants
        .iter()
        .filter(|p| p.verified)
        .cloned()
        .collect();
    
    if verified_participants.is_empty() {
        return Err(crate::error::CliptionsError::ValidationError(
            format!("No verified participants found for round {}", round_id)
        ));
    }
    
    Ok(verified_participants)
}

/// Calculate scores and payouts for participants using the PayoutCalculator
fn calculate_scores_and_payouts(
    participants: &[Participant],
    round_id: &str,
    rounds_file: &str,
    prize_pool: f64,
    use_mock: bool,
    verbose: bool,
) -> Result<Vec<ScoringResult>> {
    // Create embedder based on user preference
    let strategy = ClipBatchStrategy::new();
    
    if use_mock {
        if verbose {
            println!("Using MockEmbedder for testing");
        }
        let embedder = MockEmbedder::clip_like();
        let mut processor = RoundProcessor::new(rounds_file.to_string(), embedder, strategy);
        
        // Load rounds data
        processor.load_rounds()?;
        
        // Get target image path from the round
        let round = processor.get_round(round_id)?;
        let target_image_path = round.target_image_path.clone();
        
        if verbose {
            println!("Processing {} participants against target image: {}", participants.len(), target_image_path);
        }
        
        // Process round payouts using the existing RoundProcessor logic
        let results = processor.process_round_payouts(round_id)?;
        
        if verbose {
            println!("Successfully calculated scores and payouts for {} participants", results.len());
        }
        
        Ok(results)
    } else {
        match ClipEmbedder::new() {
            Ok(clip_embedder) => {
                if verbose {
                    println!("Using CLIP embedder for semantic scoring");
                }
                let mut processor = RoundProcessor::new(rounds_file.to_string(), clip_embedder, strategy);
                
                // Load rounds data
                processor.load_rounds()?;
                
                // Get target image path from the round
                let round = processor.get_round(round_id)?;
                let target_image_path = round.target_image_path.clone();
                
                if verbose {
                    println!("Processing {} participants against target image: {}", participants.len(), target_image_path);
                }
                
                // Process round payouts using the existing RoundProcessor logic
                let results = processor.process_round_payouts(round_id)?;
                
                if verbose {
                    println!("Successfully calculated scores and payouts for {} participants", results.len());
                }
                
                Ok(results)
            }
            Err(e) => {
                panic!("CRITICAL: Failed to load CLIP model: {}. Cannot proceed with invalid MockEmbedder fallback as this would produce unreliable scores that could lead to incorrect payouts.", e);
            }
        }
    }
}

/// Update the rounds.json file with calculated scores, payouts, and prize pool
fn update_rounds_file(
    round_id: &str,
    rounds_file: &str,
    results: &[ScoringResult],
    prize_pool: f64,
    verbose: bool,
) -> Result<()> {
    // Load the current rounds data
    let rounds_data = std::fs::read_to_string(rounds_file)
        .map_err(|e| crate::error::CliptionsError::Io(e))?;
    
    let mut rounds: serde_json::Value = serde_json::from_str(&rounds_data)
        .map_err(|e| crate::error::CliptionsError::Json(e))?;
    
    // Get the round object
    let round = rounds.get_mut(round_id)
        .ok_or_else(|| crate::error::CliptionsError::ValidationError(
            format!("Round {} not found in rounds file", round_id)
        ))?;
    
    // Update prize pool
    round["prize_pool"] = serde_json::Value::from(prize_pool);
    
    // Update total payout
    let total_payout: f64 = results.iter().filter_map(|r| r.payout).sum();
    round["total_payout"] = serde_json::Value::from(total_payout);
    
    
    // Update participants with scores and payouts
    if let Some(participants) = round.get_mut("participants") {
        if let Some(participants_array) = participants.as_array_mut() {
            for participant in participants_array {
                if let Some(username) = participant.get("username").and_then(|u| u.as_str()) {
                    // Find matching result
                    if let Some(result) = results.iter().find(|r| r.participant.username == username) {
                        // Update score
                        participant["score"] = serde_json::Value::from(result.raw_score);
                        
                        // Update payout
                        if let Some(payout_amount) = result.payout {
                            participant["payout"]["amount"] = serde_json::Value::from(payout_amount);
                        }
                        
                        // Update rank if available
                        if let Some(rank) = result.rank {
                            participant["rank"] = serde_json::Value::from(rank);
                        }
                    }
                }
            }
        }
    }
    
    // Note: We don't add a redundant "results" section since all the information
    // is already stored in the participant data (scores, payouts, ranks)
    
    // Update timestamp
    round["updated_at"] = serde_json::Value::from(chrono::Utc::now().to_rfc3339());
    
    // Write back to file
    let updated_json = serde_json::to_string_pretty(&rounds)
        .map_err(|e| crate::error::CliptionsError::Json(e))?;
    
    std::fs::write(rounds_file, updated_json)
        .map_err(|e| crate::error::CliptionsError::Io(e))?;
    
    if verbose {
        println!("Updated rounds file with scores, payouts, and prize pool information");
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
    println!("\n{}", "Round Results:".bold().underline());
    println!("{}", "=".repeat(80));
    println!("Round ID: {}", args.round_id.bold().blue());
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
            let mut content = String::from("Round Results\n");
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
    // Load verified participants
    let participants = load_verified_participants(&args.round_id, &args.rounds_file)?;
    
    if args.verbose {
        println!("Loaded {} verified participants for round {}", participants.len(), args.round_id);
    }
    
    // Calculate scores and payouts
    let results = calculate_scores_and_payouts(
        &participants,
        &args.round_id,
        &args.rounds_file,
        args.prize_pool,
        args.use_mock,
        args.verbose,
    )?;
    
    if args.verbose {
        println!("Successfully processed round {} with {} results", args.round_id, results.len());
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
    
    // Update rounds.json file
    update_rounds_file(
        &args.round_id,
        &args.rounds_file,
        &results,
        args.prize_pool,
        args.verbose,
    )?;
    
    Ok(())
} 