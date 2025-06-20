//! Process round payouts for RealMir prediction markets
//! 
//! This tool processes payouts for prediction rounds, supporting both individual
//! round processing and batch processing of all rounds.

use std::process;
use clap::Parser;

use realmir_core::embedder::MockEmbedder;
use realmir_core::scoring::BaselineAdjustedStrategy;
use realmir_core::round::RoundProcessor;

#[derive(Parser)]
#[command(name = "process_payouts")]
#[command(about = "Process payouts for RealMir prediction rounds")]
#[command(version = "1.0")]
struct Args {
    /// Process all rounds
    #[arg(long)]
    all: bool,
    
    /// Specific round ID to process
    #[arg(long)]
    round: Option<String>,
    
    /// Path to rounds file
    #[arg(long, default_value = "rounds.json")]
    rounds_file: String,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    
    // Validate arguments
    if !args.all && args.round.is_none() {
        eprintln!("Error: Must specify either --all or --round <round_id>");
        process::exit(1);
    }
    
    if args.all && args.round.is_some() {
        eprintln!("Error: Cannot specify both --all and --round");
        process::exit(1);
    }
    
    // Create processor
    let embedder = MockEmbedder::clip_like();
    let strategy = BaselineAdjustedStrategy::new();
    let mut processor = RoundProcessor::new(args.rounds_file, embedder, strategy);
    
    if args.all {
        // Process all rounds
        match processor.process_all_rounds() {
            Ok(results) => {
                println!("Processed {} rounds:", results.len());
                for (round_id, round_results) in results {
                    println!("\nRound: {}", round_id);
                    display_round_results(&round_results, args.verbose);
                }
            }
            Err(e) => {
                eprintln!("Error processing rounds: {}", e);
                process::exit(1);
            }
        }
    } else if let Some(round_id) = args.round {
        // Process specific round
        match processor.process_round_payouts(&round_id) {
            Ok(results) => {
                println!("Round: {}", round_id);
                display_round_results(&results, args.verbose);
            }
            Err(e) => {
                eprintln!("Error processing round {}: {}", round_id, e);
                process::exit(1);
            }
        }
    }
}

fn display_round_results(results: &[realmir_core::types::ScoringResult], verbose: bool) {
    if results.is_empty() {
        println!("  No results to display");
        return;
    }
    
    println!("  {} participants processed", results.len());
    
    if verbose {
        println!("  Detailed results:");
        for (i, result) in results.iter().enumerate() {
            println!("    {}. {} ({})", 
                i + 1, 
                result.participant.username,
                result.participant.user_id
            );
            println!("       Guess: \"{}\"", result.participant.guess.text);
            println!("       Score: {:.4}", result.effective_score());
            if let Some(rank) = result.rank {
                println!("       Rank: {}", rank);
            }
            if let Some(payout) = result.payout {
                println!("       Payout: {:.9}", payout);
            }
            println!();
        }
    }
    
    let total_payout: f64 = results.iter()
        .filter_map(|r| r.payout)
        .sum();
    
    println!("  Total payout: {:.9}", total_payout);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use realmir_core::types::{RoundData, Participant, Guess};
    use realmir_core::commitment::CommitmentGenerator;
    use std::collections::HashMap;
    
    #[test]
    fn test_process_payouts_basic() {
        // Create a temporary rounds file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();
        
        // Create test round data
        let mut round = RoundData::new(
            "test_round".to_string(),
            "Test Round".to_string(),
            "A test round".to_string(),
            "test.jpg".to_string(),
        );
        
        // Add a test participant with valid commitment
        let commitment_gen = CommitmentGenerator::new();
        let salt = "test_salt";
        let message = "test guess";
        let commitment = commitment_gen.generate(message, salt).unwrap();
        
        let participant = Participant::new(
            "user1".to_string(),
            "user_user1".to_string(),
            Guess::new(message.to_string()),
            commitment,
        ).with_salt(salt.to_string()).mark_verified();
        
        round.add_participant(participant);
        
        // Save rounds data
        let mut rounds = HashMap::new();
        rounds.insert("test_round".to_string(), round);
        let content = serde_json::to_string_pretty(&rounds).unwrap();
        std::fs::write(&file_path, content).unwrap();
        
        // Test processor
        let embedder = MockEmbedder::clip_like();
        let strategy = BaselineAdjustedStrategy::new();
        let mut processor = RoundProcessor::new(file_path, embedder, strategy);
        
        // This would fail in the test environment due to missing image file,
        // but we can test the basic setup
        let round_ids = processor.get_round_ids().unwrap();
        assert_eq!(round_ids.len(), 1);
        assert_eq!(round_ids[0], "test_round");
    }
}