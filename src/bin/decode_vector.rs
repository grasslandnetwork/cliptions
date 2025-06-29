//! CLI tool for decoding CLIP vectors from images using steganography
//! 
//! This tool allows validators and other users to extract and verify CLIP vectors
//! from images posted during the reveal phase.

use clap::Parser;
use realmir_core::{
    steganography::{VectorSteganographer},
    embedder::{EmbedderTrait, MockEmbedder, ClipEmbedder, cosine_similarity},
    commitment::{EnhancedCommitmentGenerator},
    error::Result,
};
use ndarray::Array1;
use colored::*;
use serde_json;

#[derive(Parser)]
#[command(name = "decode-vector")]
#[command(about = "Decode and verify CLIP vector from an image")]
#[command(version = "1.0")]
struct Args {
    /// Input image path containing embedded vector
    #[arg(short, long)]
    input: String,
    
    /// Optional: verify against this text
    #[arg(short, long)]
    text: Option<String>,
    
    /// Optional: salt for commitment verification
    #[arg(short, long)]
    salt: Option<String>,
    
    /// Optional: commitment hash to verify against
    #[arg(short, long)]
    commitment: Option<String>,
    
    /// Use mock embedder instead of real CLIP (for testing)
    #[arg(long, default_value = "false")]
    mock: bool,
    
    /// Bits per channel for steganography (should match encoding)
    #[arg(long, default_value = "2")]
    bits_per_channel: u8,
    
    /// Output extracted vector to JSON file
    #[arg(long)]
    output_json: Option<String>,
    
    /// Show detailed vector information
    #[arg(long, default_value = "false")]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("{}", "🔍 RealMir Vector Decoder".bright_blue().bold());
    println!("Decoding vector from: {}", args.input.bright_green());
    
    // Create steganographer
    let steganographer = VectorSteganographer::with_bits_per_channel(args.bits_per_channel)?;
    
    // Check if image has embedded data
    if !steganographer.has_embedded_data(&args.input) {
        eprintln!("{} No embedded data found in image!", "❌".red());
        std::process::exit(1);
    }
    
    // Extract the vector
    print!("🔓 Extracting embedded vector... ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    let (extracted_vector, metadata) = steganographer.extract_vector(&args.input)?;
    
    println!("{}", "Done!".green());
    
    // Display metadata
    println!("\n{}", "📋 Embedded Metadata:".bright_blue().bold());
    println!("  Version: {}", metadata.version.to_string().bright_cyan());
    println!("  Dimension: {}", metadata.dimension.to_string().bright_cyan());
    println!("  Salt: {}", metadata.salt.bright_yellow());
    println!("  Round ID: {}", metadata.round_id.bright_cyan());
    println!("  Vector length: {}", extracted_vector.len().to_string().bright_green());
    
    if args.verbose {
        println!("\n{}", "🔢 Vector Details:".bright_blue().bold());
        println!("  First 10 values: {:?}", &extracted_vector[..10.min(extracted_vector.len())]);
        println!("  Last 10 values: {:?}", &extracted_vector[extracted_vector.len().saturating_sub(10)..]);
        
        // Calculate some statistics
        let sum: f64 = extracted_vector.iter().sum();
        let mean = sum / extracted_vector.len() as f64;
        let variance: f64 = extracted_vector.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / extracted_vector.len() as f64;
        let std_dev = variance.sqrt();
        
        println!("  Mean: {:.6}", mean);
        println!("  Std Dev: {:.6}", std_dev);
        println!("  Min: {:.6}", extracted_vector.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
        println!("  Max: {:.6}", extracted_vector.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
    }
    
    // Text verification if provided
    if let Some(text) = &args.text {
        println!("\n{}", "🤖 Text Verification:".bright_blue().bold());
        print!("Generating CLIP vector for text... ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        let text_vector = if args.mock {
            let embedder = MockEmbedder::clip_like();
            embedder.get_text_embedding(text)?
        } else {
            match ClipEmbedder::new() {
                Ok(embedder) => embedder.get_text_embedding(text)?,
                Err(_) => {
                    println!("{}", "Failed, falling back to mock embedder".yellow());
                    let embedder = MockEmbedder::clip_like();
                    embedder.get_text_embedding(text)?
                }
            }
        };
        
        println!("{}", "Done!".green());
        
        // Calculate similarity
        let similarity = cosine_similarity(
            &Array1::from_vec(extracted_vector.clone()),
            &text_vector
        )?;
        
        println!("Text: {}", text.bright_green());
        println!("Cosine similarity: {:.6}", similarity.to_string().bright_cyan());
        
        if similarity > 0.99 {
            println!("{} Vectors match very closely!", "✅".green());
        } else if similarity > 0.95 {
            println!("{} Vectors match well", "✅".green());
        } else if similarity > 0.8 {
            println!("{} Vectors are similar", "⚠️".yellow());
        } else {
            println!("{} Vectors don't match well", "❌".red());
        }
    }
    
    // Commitment verification if provided
    if let (Some(salt), Some(text)) = (&args.salt, &args.text) {
        println!("\n{}", "🔐 Commitment Verification:".bright_blue().bold());
        
        let generator = EnhancedCommitmentGenerator::new();
        let calculated_commitment = generator.generate_with_vector(text, salt, &extracted_vector)?;
        
        println!("Calculated commitment: {}", calculated_commitment.bright_cyan());
        
        if let Some(provided_commitment) = &args.commitment {
            if &calculated_commitment == provided_commitment {
                println!("{} Commitment verified!", "✅".green());
            } else {
                println!("{} Commitment verification failed!", "❌".red());
                println!("Expected: {}", provided_commitment.bright_yellow());
                println!("Got: {}", calculated_commitment.bright_cyan());
            }
        }
    }
    
    // Output to JSON if requested
    if let Some(json_path) = &args.output_json {
        println!("\n{}", "💾 Saving to JSON:".bright_blue().bold());
        
        let output_data = serde_json::json!({
            "metadata": metadata,
            "vector": extracted_vector,
            "extraction_info": {
                "bits_per_channel": args.bits_per_channel,
                "input_file": args.input,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });
        
        std::fs::write(json_path, serde_json::to_string_pretty(&output_data)?)?;
        println!("Saved to: {}", json_path.bright_green());
    }
    
    println!("\n{}", "🎉 Vector successfully decoded!".bright_green().bold());
    
    Ok(())
}