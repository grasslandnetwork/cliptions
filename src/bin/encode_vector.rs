//! CLI tool for encoding CLIP vectors into images using steganography
//! 
//! This tool allows users to embed CLIP vectors into images that can then be
//! posted to Twitter as part of the reveal phase.

use clap::Parser;
use realmir_core::{
    steganography::{VectorSteganographer, EmbeddedVectorMeta, utils},
    embedder::{EmbedderTrait, MockEmbedder, ClipEmbedder},
    error::Result,
};
use std::fs;
use colored::*;

#[derive(Parser)]
#[command(name = "encode-vector")]
#[command(about = "Encode CLIP vector into an image using steganography")]
#[command(version = "1.0")]
struct Args {
    /// Input image path
    #[arg(short, long)]
    input: String,
    
    /// Output image path
    #[arg(short, long)]
    output: String,
    
    /// Text to generate CLIP vector for
    #[arg(short, long)]
    text: String,
    
    /// Salt for the commitment
    #[arg(short, long)]
    salt: String,
    
    /// Round ID
    #[arg(short, long)]
    round_id: String,
    
    /// Use mock embedder instead of real CLIP (for testing)
    #[arg(long, default_value = "false")]
    mock: bool,
    
    /// Bits per channel for steganography (1-3 recommended)
    #[arg(long, default_value = "2")]
    bits_per_channel: u8,
    
    /// Create a test image if input doesn't exist
    #[arg(long, default_value = "false")]
    create_test_image: bool,
    
    /// Test image dimensions (if creating)
    #[arg(long, default_value = "512")]
    image_size: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("{}", "🎨 RealMir Vector Encoder".bright_blue().bold());
    println!("Encoding CLIP vector for: {}", args.text.bright_green());
    
    // Create steganographer
    let steganographer = VectorSteganographer::with_bits_per_channel(args.bits_per_channel)?;
    
    // Handle input image - create if needed and requested
    if !std::path::Path::new(&args.input).exists() {
        if args.create_test_image {
            println!("📁 Creating test image: {}", args.input.bright_yellow());
            let (min_width, min_height) = utils::min_image_size_for_vector(512, args.bits_per_channel);
            let width = args.image_size.max(min_width);
            let height = args.image_size.max(min_height);
            
            utils::create_test_image(width, height, &args.input)?;
            println!("✅ Created {}x{} test image", width, height);
        } else {
            eprintln!("{} Input image not found: {}", "❌".red(), args.input);
            eprintln!("Use --create-test-image to create a test image");
            std::process::exit(1);
        }
    }
    
    // Generate CLIP vector
    print!("🤖 Generating CLIP vector... ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    let clip_vector = if args.mock {
        let embedder = MockEmbedder::clip_like();
        embedder.get_text_embedding(&args.text)?
    } else {
        match ClipEmbedder::new() {
            Ok(embedder) => embedder.get_text_embedding(&args.text)?,
            Err(_) => {
                println!("{}", "Failed, falling back to mock embedder".yellow());
                let embedder = MockEmbedder::clip_like();
                embedder.get_text_embedding(&args.text)?
            }
        }
    };
    
    println!("{}", "Done!".green());
    println!("Vector dimension: {}", clip_vector.len().to_string().bright_cyan());
    
    // Create metadata
    let metadata = EmbeddedVectorMeta {
        version: 1,
        dimension: clip_vector.len() as u32,
        salt: args.salt.clone(),
        round_id: args.round_id.clone(),
        checksum: None,
    };
    
    // Check capacity
    let img = image::open(&args.input).map_err(|_e| {
        realmir_core::error::RealMirError::Steganography(
            realmir_core::error::SteganographyError::InvalidImage
        )
    })?;
    let capacity = steganographer.calculate_capacity(&img.to_rgb8());
    let vector_bytes = clip_vector.len() * 8; // f64 = 8 bytes
    let meta_bytes = 256; // Estimated
    let total_needed = vector_bytes + meta_bytes + 20; // Header overhead
    
    println!("📊 Capacity check:");
    println!("  Image capacity: {} bytes", capacity.to_string().bright_cyan());
    println!("  Data needed: {} bytes", total_needed.to_string().bright_yellow());
    
    if total_needed > capacity {
        eprintln!("{} Insufficient capacity in image!", "❌".red());
        eprintln!("Try using a larger image or fewer bits per channel.");
        std::process::exit(1);
    }
    
    // Embed the vector
    print!("🔐 Embedding vector in image... ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    let vector_f64: Vec<f64> = clip_vector.to_vec();
    steganographer.embed_vector(&args.input, &vector_f64, &metadata, &args.output)?;
    
    println!("{}", "Done!".green());
    
    // Verify the embedding
    print!("✅ Verifying embedding... ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    
    match steganographer.extract_vector(&args.output) {
        Ok((extracted_vector, extracted_meta)) => {
            if extracted_vector.len() == vector_f64.len() 
                && extracted_meta.salt == args.salt 
                && extracted_meta.round_id == args.round_id {
                println!("{}", "Verified!".green());
            } else {
                println!("{}", "Verification failed!".red());
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("{} {}", "Failed:".red(), e);
            std::process::exit(1);
        }
    }
    
    // Summary
    println!("\n{}", "📋 Summary:".bright_blue().bold());
    println!("  Input: {}", args.input);
    println!("  Output: {}", args.output);
    println!("  Text: {}", args.text.bright_green());
    println!("  Salt: {}", args.salt.bright_yellow());
    println!("  Round ID: {}", args.round_id.bright_cyan());
    println!("  Vector size: {} elements", clip_vector.len());
    
    let output_size = fs::metadata(&args.output)?.len();
    println!("  Output file size: {} bytes", output_size);
    
    println!("\n{}", "🎉 Vector successfully encoded!".bright_green().bold());
    println!("You can now post {} to Twitter for your reveal.", args.output.bright_blue());
    
    Ok(())
}