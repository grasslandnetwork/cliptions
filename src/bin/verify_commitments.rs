//! Verify commitments for RealMir prediction rounds
//! 
//! This tool verifies cryptographic commitments for participants in prediction rounds.

use std::process;
use clap::Parser;

use realmir_core::embedder::MockEmbedder;
use realmir_core::scoring::BaselineAdjustedStrategy;
use realmir_core::round::RoundProcessor;

#[derive(Parser)]
#[command(name = "verify_commitments")]
#[command(about = "Verify commitments for RealMir prediction rounds")]
#[command(version = "1.0")]
struct Args {
    /// Round ID to verify
    round_id: String,
    
    /// Path to rounds file
    #[arg(long, default_value = "rounds.json")]
    rounds_file: String,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    
    // Create processor
    let embedder = MockEmbedder::clip_like();
    let strategy = BaselineAdjustedStrategy::new();
    let mut processor = RoundProcessor::new(args.rounds_file, embedder, strategy);
    
    // Load rounds to get participant info
    if let Err(e) = processor.load_rounds() {
        eprintln!("Error loading rounds: {}", e);
        process::exit(1);
    }
    
    // Get round info before verification
    let round = match processor.get_round(&args.round_id) {
        Ok(round) => round,
        Err(e) => {
            eprintln!("Error getting round {}: {}", args.round_id, e);
            process::exit(1);
        }
    };
    
    println!("Verifying commitments for round: {}", args.round_id);
    println!("Round title: {}", round.title);
    println!("Participants: {}", round.participants.len());
    
    if round.participants.is_empty() {
        println!("No participants to verify.");
        return;
    }
    
    // Verify commitments
    match processor.verify_commitments(&args.round_id) {
        Ok(results) => {
            let valid_count = results.iter().filter(|&&r| r).count();
            let total_count = results.len();
            
            println!("\nVerification Results:");
            println!("Valid commitments: {}/{}", valid_count, total_count);
            
            if args.verbose {
                println!("\nDetailed Results:");
                let round = processor.get_round(&args.round_id).unwrap();
                
                for (i, (participant, &is_valid)) in round.participants.iter().zip(results.iter()).enumerate() {
                    let status = if is_valid { "✓ VALID" } else { "✗ INVALID" };
                    println!("  {}. {} ({}): {}", 
                        i + 1, 
                        participant.username,
                        participant.user_id,
                        status
                    );
                    
                    if args.verbose {
                        println!("     Guess: \"{}\"", participant.guess.text);
                        println!("     Commitment: {}", participant.commitment);
                        if let Some(salt) = &participant.salt {
                            println!("     Salt: {}", salt);
                        } else {
                            println!("     Salt: [NOT PROVIDED]");
                        }
                    }
                    println!();
                }
            }
            
            if valid_count == total_count {
                println!("All commitments verified successfully!");
            } else {
                println!("Warning: {} commitment(s) failed verification.", total_count - valid_count);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error verifying commitments: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use realmir_core::types::{RoundData, Participant, Guess};
    use realmir_core::commitment::CommitmentGenerator;
    use std::collections::HashMap;
    
    #[test]
    fn test_verify_commitments_basic() {
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
        ).with_salt(salt.to_string());
        
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
        
        // Verify commitments
        let results = processor.verify_commitments("test_round").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0]); // Should be valid
    }
    
    #[test]
    fn test_verify_invalid_commitment() {
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
        
        // Add a test participant with invalid commitment
        let participant = Participant::new(
            "user1".to_string(),
            "user_user1".to_string(),
            Guess::new("test guess".to_string()),
            "invalid_commitment".to_string(),
        ).with_salt("test_salt".to_string());
        
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
        
        // Verify commitments
        let results = processor.verify_commitments("test_round").unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0]); // Should be invalid
    }
}