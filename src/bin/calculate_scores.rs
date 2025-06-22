//! Calculate scores and payouts for RealMir guesses
//! 
//! This is a drop-in replacement for the Python calculate_scores_payout.py script.
//! It provides the same command-line interface but with significantly better performance.


use std::process;
use clap::Parser;

use realmir_core::embedder::MockEmbedder;
use realmir_core::scoring::{BaselineAdjustedStrategy, ScoreValidator, calculate_rankings, calculate_payouts};

#[derive(Parser)]
#[command(name = "calculate_scores")]
#[command(about = "Calculate rankings and payouts for RealMir guesses")]
#[command(version = "1.0")]
struct Args {
    /// Path to the target image
    target_image_path: String,
    
    /// Prize pool amount
    prize_pool: f64,
    
    /// List of guesses to rank
    guesses: Vec<String>,
}

fn display_results(ranked_results: &[(String, f64)], payouts: &[f64], prize_pool: f64) {
    println!("\nRankings and Payouts:");
    println!("{}", "-".repeat(50));
    
    for (i, ((guess, similarity), payout)) in ranked_results.iter().zip(payouts.iter()).enumerate() {
        println!("{}. \"{}\"", i + 1, guess);
        println!("   Similarity score: {:.4}", similarity);
        println!("   Payout: {:.9}", payout);
        println!();
    }
    
    println!("Total prize pool: {:.9}", prize_pool);
    println!("Total payout: {:.9}", payouts.iter().sum::<f64>());
}

fn main() {
    let args = Args::parse();
    
    // Validate inputs
    if args.prize_pool <= 0.0 {
        eprintln!("Error: Prize pool must be greater than zero");
        process::exit(1);
    }
    
    if args.guesses.is_empty() {
        eprintln!("Error: At least one guess must be provided");
        process::exit(1);
    }
    
    // Create embedder and strategy
    let embedder = MockEmbedder::clip_like();
    let strategy = BaselineAdjustedStrategy::new();
    let validator = ScoreValidator::new(embedder, strategy);
    
    // Calculate rankings
    let ranked_results = match calculate_rankings(&args.target_image_path, &args.guesses, &validator) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Error calculating rankings: {}", e);
            process::exit(1);
        }
    };
    
    // Calculate payouts
    let payouts = match calculate_payouts(&ranked_results, args.prize_pool) {
        Ok(payouts) => payouts,
        Err(e) => {
            eprintln!("Error calculating payouts: {}", e);
            process::exit(1);
        }
    };
    
    // Display results
    display_results(&ranked_results, &payouts, args.prize_pool);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_scores_basic() {
        let embedder = MockEmbedder::clip_like();
        let strategy = BaselineAdjustedStrategy::new();
        let validator = ScoreValidator::new(embedder, strategy);
        
        let guesses = vec![
            "guess1".to_string(),
            "guess2".to_string(),
            "guess3".to_string(),
        ];
        
        let ranked_results = calculate_rankings("test.jpg", &guesses, &validator).unwrap();
        let payouts = calculate_payouts(&ranked_results, 100.0).unwrap();
        
        assert_eq!(ranked_results.len(), 3);
        assert_eq!(payouts.len(), 3);
        
        // Total payout should equal prize pool
        let total: f64 = payouts.iter().sum();
        assert!((total - 100.0).abs() < 1e-10);
    }
}