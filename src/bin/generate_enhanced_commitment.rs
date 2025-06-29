//! CLI tool for generating enhanced commitments with CLIP vectors and proof of work
//! 
//! This tool generates the enhanced commitment described in the Slack thread:
//! hash = sha256(plaintext_prediction || salt || clip_vector)
//! Plus proof of work for spam prevention.

use clap::Parser;
use realmir_core::{
    commitment::{EnhancedCommitmentGenerator},
    embedder::{EmbedderTrait, MockEmbedder, ClipEmbedder},
    error::Result,
};
use colored::*;
use serde_json;

#[derive(Parser)]
#[command(name = "generate-enhanced-commitment")]
#[command(about = "Generate enhanced commitment with CLIP vector and proof of work")]
#[command(version = "1.0")]
struct Args {
    /// Prediction text to commit to
    #[arg(short, long)]
    prediction: String,
    
    /// Round ID for this commitment
    #[arg(short, long)]
    round_id: String,
    
    /// Optional salt (will be generated if not provided)
    #[arg(short, long)]
    salt: Option<String>,
    
    /// Proof of work difficulty (1-10, default: 4)
    #[arg(long, default_value = "4")]
    difficulty: u8,
    
    /// Use mock embedder instead of real CLIP (for testing)
    #[arg(long, default_value = "false")]
    mock: bool,
    
    /// Output commitment to JSON file
    #[arg(long)]
    output_json: Option<String>,
    
    /// Show detailed output including vector information
    #[arg(long, default_value = "false")]
    verbose: bool,
    
    /// Skip proof of work generation (for testing)
    #[arg(long, default_value = "false")]
    skip_pow: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("{}", "🔐 RealMir Enhanced Commitment Generator".bright_blue().bold());
    println!("Prediction: {}", args.prediction.bright_green());
    println!("Round ID: {}", args.round_id.bright_cyan());
    
    // Create generator with appropriate difficulty
    let generator = if args.skip_pow {
        EnhancedCommitmentGenerator::with_pow_difficulty(1)? // Minimal difficulty for testing
    } else {
        EnhancedCommitmentGenerator::with_pow_difficulty(args.difficulty)?
    };
    
    // Generate or use provided salt
    let salt = args.salt.unwrap_or_else(|| {
        let generated_salt = generator.generate_salt();
        println!("Generated salt: {}", generated_salt.bright_yellow());
        generated_salt
    });
    
    // Generate CLIP vector
    print!("🤖 Computing CLIP vector... ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    let clip_vector = if args.mock {
        let embedder = MockEmbedder::clip_like();
        embedder.get_text_embedding(&args.prediction)?
    } else {
        match ClipEmbedder::new() {
            Ok(embedder) => embedder.get_text_embedding(&args.prediction)?,
            Err(_) => {
                println!("{}", "Failed, falling back to mock embedder".yellow());
                let embedder = MockEmbedder::clip_like();
                embedder.get_text_embedding(&args.prediction)?
            }
        }
    };
    
    println!("{}", "Done!".green());
    println!("Vector dimension: {}", clip_vector.len().to_string().bright_cyan());
    
    if args.verbose {
        println!("Vector preview: {:?}...", &clip_vector.to_vec()[..5.min(clip_vector.len())]);
    }
    
    // Generate proof of work
    if !args.skip_pow {
        println!("⛏️ Generating proof of work (difficulty: {})...", args.difficulty);
        print!("This may take a moment... ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }
    
    let start_time = std::time::Instant::now();
    
    // Generate the enhanced commitment
    let commitment = generator.generate_enhanced(
        &args.prediction,
        &salt,
        &clip_vector.to_vec(),
        &args.round_id,
    )?;
    
    let generation_time = start_time.elapsed();
    
    if !args.skip_pow {
        println!("{} (took {:.2}s)", "Done!".green(), generation_time.as_secs_f64());
    }
    
    // Display results
    println!("\n{}", "✅ Enhanced Commitment Generated:".bright_green().bold());
    println!("  Commitment Hash: {}", commitment.commitment_hash.bright_cyan());
    println!("  Vector Hash: {}", commitment.vector_commitment.bright_yellow());
    println!("  Salt: {}", salt.bright_yellow());
    println!("  Round ID: {}", commitment.round_id.bright_cyan());
    println!("  Timestamp: {}", commitment.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    
    // Proof of work details
    println!("\n{}", "⛏️ Proof of Work:".bright_blue().bold());
    println!("  Difficulty: {}", commitment.proof_of_work.difficulty.to_string().bright_cyan());
    println!("  Nonce: {}", commitment.proof_of_work.nonce.to_string().bright_yellow());
    println!("  Hash: {}", commitment.proof_of_work.hash.bright_green());
    
    if args.verbose {
        println!("  Challenge: {}", commitment.proof_of_work.challenge.bright_white());
        println!("  PoW Timestamp: {}", commitment.proof_of_work.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    // Verify the commitment
    print!("\n🔍 Verifying commitment... ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    if commitment.is_valid() {
        println!("{}", "Valid!".green());
    } else {
        println!("{}", "Invalid!".red());
        eprintln!("⚠️ Generated commitment failed validation!");
        std::process::exit(1);
    }
    
    // Show Twitter posting instructions
    println!("\n{}", "📱 Twitter Posting Instructions:".bright_blue().bold());
    println!("1. For the COMMITMENT phase, post:");
    println!("   Commit: {}", commitment.commitment_hash.bright_cyan());
    println!("   Wallet: your_wallet_address");
    println!("   (Optional: Include PoW nonce {} for extra verification)", commitment.proof_of_work.nonce);
    
    println!("\n2. For the REVEAL phase, post:");
    println!("   Prediction: {}", args.prediction.bright_green());
    println!("   Salt: {}", salt.bright_yellow());
    println!("   [Attach image with embedded CLIP vector]");
    
    println!("\n{}", "💡 Next Steps:".bright_blue().bold());
    println!("1. Use encode-vector tool to embed CLIP vector in an image");
    println!("2. Post the commitment hash to Twitter");
    println!("3. During reveal, post prediction + salt + image");
    
    // Output to JSON if requested
    if let Some(json_path) = &args.output_json {
        println!("\n{}", "💾 Saving to JSON:".bright_blue().bold());
        
        let output_data = serde_json::json!({
            "commitment": commitment,
            "reveal_data": {
                "prediction": args.prediction,
                "salt": salt,
                "clip_vector": clip_vector.to_vec()
            },
            "generation_info": {
                "difficulty": args.difficulty,
                "generation_time_seconds": generation_time.as_secs_f64(),
                "mock_embedder": args.mock,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });
        
        std::fs::write(json_path, serde_json::to_string_pretty(&output_data)?)?;
        println!("Saved to: {}", json_path.bright_green());
    }
    
    println!("\n{}", "🎉 Enhanced commitment ready for Twitter!".bright_green().bold());
    
    Ok(())
}