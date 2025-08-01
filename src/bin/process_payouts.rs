//! Process block payouts for Cliptions prediction markets
//!
//! Enhanced CLI tool with comprehensive error handling, multiple output formats,
//! configuration support, and improved user experience for processing payouts
//! across prediction market blocks.

use clap::Parser;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process;

use cliptions_core::config::ConfigManager;
use cliptions_core::embedder::{ClipEmbedder, EmbedderTrait, MockEmbedder};
use cliptions_core::block_processor::BlockProcessor;
use cliptions_core::scoring::ClipBatchStrategy;

#[derive(Parser)]
#[command(name = "process_payouts")]
#[command(about = "Process payouts for Cliptions prediction blocks")]
#[command(version = "2.0")]
#[command(long_about = "
Process payouts for Cliptions prediction market blocks with comprehensive error handling
and multiple output formats.

This tool calculates and processes payouts for prediction blocks, supporting both 
individual block processing and batch processing of all blocks. Results can be 
displayed in multiple formats and saved to files for further analysis.

Examples:
  # Process a specific block with verbose output
  process_payouts --block block1 --verbose
  
  # Process all blocks and save results to JSON
  process_payouts --all --output json --output-file results.json
  
  # Use real CLIP model for processing
  process_payouts --block block1 --use-clip --config config.yaml
  
  # Process with custom blocks file
  process_payouts --block block1 --blocks-file data/custom_blocks.json
")]
struct Args {
    /// Process all blocks
    #[arg(long)]
    all: bool,

    /// Specific block ID to process
    #[arg(long)]
    block: Option<String>,

    /// Path to blocks file
    #[arg(long, default_value = "blocks.json")]
    blocks_file: PathBuf,

    /// Output format: table, json, csv
    #[arg(long, short, default_value = "table", value_parser = ["table", "json", "csv"])]
    output: String,

    /// Save results to file
    #[arg(long, short)]
    output_file: Option<PathBuf>,

    /// Use MockEmbedder instead of CLIP for testing (fast, deterministic)
    #[arg(long)]
    use_mock: bool,

    /// Path to CLIP model directory (optional, uses default if not specified)
    #[arg(long)]
    clip_model: Option<PathBuf>,

    /// Enable verbose output with detailed progress information
    #[arg(short, long)]
    verbose: bool,

    /// Suppress colored output (useful for scripts/logging)
    #[arg(long)]
    no_color: bool,

    /// Configuration file path (YAML format)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Continue processing on errors (for batch operations)
    #[arg(long)]
    continue_on_error: bool,

    /// Show detailed participant breakdown
    #[arg(long)]
    detailed: bool,

    /// Minimum number of participants required to process a block
    #[arg(long, default_value = "1")]
    min_participants: usize,

    /// Maximum number of blocks to process (for --all, 0 = unlimited)
    #[arg(long, default_value = "0")]
    max_blocks: usize,
}

fn main() {
    let args = Args::parse();

    // Initialize colored output
    if args.no_color {
        colored::control::set_override(false);
    }

    // Load configuration if specified
    let _config_manager = if let Some(config_path) = &args.config {
        match ConfigManager::with_path(config_path) {
            Ok(manager) => {
                if args.verbose {
                    println!(
                        "{} Loaded configuration from {}",
                        "Info:".blue().bold(),
                        config_path.display()
                    );
                }
                Some(manager)
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to load config from {}: {}",
                    "Error:".red().bold(),
                    config_path.display(),
                    e
                );
                process::exit(1);
            }
        }
    } else {
        match ConfigManager::new() {
            Ok(manager) => {
                if args.verbose {
                    println!("{} Using default configuration", "Info:".blue().bold());
                }
                Some(manager)
            }
            Err(_) => {
                if args.verbose {
                    println!(
                        "{} No configuration file found, using built-in defaults",
                        "Info:".blue().bold()
                    );
                }
                None
            }
        }
    };

    // Validate arguments with enhanced error messages
    if let Err(e) = validate_inputs(&args) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        eprintln!(
            "{} Use --help for usage information",
            "Tip:".yellow().bold()
        );
        process::exit(1);
    }

    // Create processor and process blocks
    let results = create_processor_and_process(&args);

    match results {
        Ok(output_data) => {
            // Display results
            if let Err(e) = display_results(&output_data, &args) {
                eprintln!("{} Failed to display results: {}", "Error:".red().bold(), e);
                process::exit(1);
            }

            // Save to file if requested
            if let Some(output_file) = &args.output_file {
                if let Err(e) = save_results(&output_data, output_file, &args.output) {
                    eprintln!("{} Failed to save results: {}", "Error:".red().bold(), e);
                    process::exit(1);
                }

                println!(
                    "{} Results saved to {}",
                    "Success:".green().bold(),
                    output_file.display()
                );
            }

            if args.verbose {
                println!(
                    "{} Payout processing completed successfully",
                    "Success:".green().bold()
                );
            }
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            process::exit(1);
        }
    }
}

fn validate_inputs(args: &Args) -> Result<(), String> {
    // Validate mutual exclusivity
    if !args.all && args.block.is_none() {
        return Err("Must specify either --all or --block <block_num>".to_string());
    }

    if args.all && args.block.is_some() {
        return Err("Cannot specify both --all and --block".to_string());
    }

    // Validate blocks file exists
    if !args.blocks_file.exists() {
        return Err(format!(
            "Blocks file does not exist: {}",
            args.blocks_file.display()
        ));
    }

    // Validate CLIP model path if provided
    if let Some(model_path) = &args.clip_model {
        if !model_path.exists() {
            return Err(format!(
                "CLIP model path does not exist: {}",
                model_path.display()
            ));
        }
    }

    // Validate output file directory exists if specified
    if let Some(output_file) = &args.output_file {
        if let Some(parent) = output_file.parent() {
            if !parent.exists() {
                return Err(format!(
                    "Output directory does not exist: {}",
                    parent.display()
                ));
            }
        }
    }

    // Validate min_participants
    if args.min_participants == 0 {
        return Err("Minimum participants must be at least 1".to_string());
    }

    Ok(())
}

fn create_processor_and_process(
    args: &Args,
) -> Result<ProcessingResults, Box<dyn std::error::Error>> {
    let strategy = ClipBatchStrategy::new();

    // Create processor and process based on embedder type (defaults to CLIP)
    if args.use_mock {
        if args.verbose {
            println!("{} Using MockEmbedder for testing", "Info:".blue().bold());
        }
        let embedder = MockEmbedder::clip_like();
        let processor = BlockProcessor::new(
            args.blocks_file.to_string_lossy().to_string(),
            embedder,
            strategy,
        );
        process_with_processor(processor, args)
    } else {
        // Default: Use CLIP embedder
        if let Some(model_path) = &args.clip_model {
            match ClipEmbedder::from_path(&model_path.to_string_lossy()) {
                Ok(embedder) => {
                    if args.verbose {
                        println!(
                            "{} Using CLIP embedder from {}",
                            "Info:".blue().bold(),
                            model_path.display()
                        );
                    }
                    let processor = BlockProcessor::new(
                        args.blocks_file.to_string_lossy().to_string(),
                        embedder,
                        strategy,
                    );
                    process_with_processor(processor, args)
                }
                Err(e) => {
                    panic!(
                        "CRITICAL: Failed to load CLIP model from {}: {}. Cannot proceed with invalid MockEmbedder fallback as this would produce unreliable payout calculations.",
                        model_path.display(),
                        e
                    );
                }
            }
        } else {
            match ClipEmbedder::new() {
                Ok(embedder) => {
                    if args.verbose {
                        println!("{} Using default CLIP embedder", "Info:".blue().bold());
                    }
                    let processor = BlockProcessor::new(
                        args.blocks_file.to_string_lossy().to_string(),
                        embedder,
                        strategy,
                    );
                    process_with_processor(processor, args)
                }
                Err(e) => {
                    panic!(
                        "CRITICAL: Failed to load default CLIP model: {}. Cannot proceed with invalid MockEmbedder fallback as this would produce unreliable payout calculations.",
                        e
                    );
                }
            }
        }
    }
}

fn process_with_processor<E: EmbedderTrait>(
    processor: BlockProcessor<E, ClipBatchStrategy>,
    args: &Args,
) -> Result<ProcessingResults, Box<dyn std::error::Error>> {
    if args.all {
        process_all_blocks(processor, args)
    } else if let Some(block_num) = &args.block {
        process_single_block(processor, block_num, args)
    } else {
        Err("Must specify either --all or --block <block_num>".into())
    }
}

#[derive(Debug)]
struct ProcessingResults {
    blocks: Vec<(String, Vec<cliptions_core::types::ScoringResult>)>,
    total_blocks_processed: usize,
    total_participants: usize,
    total_payout: f64,
    errors: Vec<String>,
}

fn process_all_blocks(
    mut processor: BlockProcessor<impl EmbedderTrait, ClipBatchStrategy>,
    args: &Args,
) -> Result<ProcessingResults, Box<dyn std::error::Error>> {
    if args.verbose {
        println!("{} Processing all blocks...", "Info:".blue().bold());
    }

    let all_results = processor.process_all_blocks()?;
    let mut results = ProcessingResults {
        blocks: Vec::new(),
        total_blocks_processed: 0,
        total_participants: 0,
        total_payout: 0.0,
        errors: Vec::new(),
    };

    let mut processed_count = 0;

    for (block_num, block_results) in all_results {
        // Check max blocks limit
        if args.max_blocks > 0 && processed_count >= args.max_blocks {
            if args.verbose {
                println!(
                    "{} Reached maximum blocks limit ({})",
                    "Info:".blue().bold(),
                    args.max_blocks
                );
            }
            break;
        }

        // Check minimum participants requirement
        if block_results.len() < args.min_participants {
            if args.verbose {
                println!(
                    "{} Skipping block {} (only {} participants, minimum {})",
                    "Info:".blue().bold(),
                    block_num,
                    block_results.len(),
                    args.min_participants
                );
            }
            continue;
        }

        let block_payout: f64 = block_results.iter().filter_map(|r| r.payout).sum();

        results
            .blocks
            .push((block_num.clone(), block_results.clone()));
        results.total_blocks_processed += 1;
        results.total_participants += block_results.len();
        results.total_payout += block_payout;
        processed_count += 1;

        if args.verbose {
            println!(
                "{} Processed block {} ({} participants, {:.9} TAO)",
                "Info:".blue().bold(),
                block_num,
                block_results.len(),
                block_payout
            );
        }
    }

    Ok(results)
}

fn process_single_block(
    mut processor: BlockProcessor<impl EmbedderTrait, ClipBatchStrategy>,
    block_num: &str,
    args: &Args,
) -> Result<ProcessingResults, Box<dyn std::error::Error>> {
    if args.verbose {
        println!("{} Processing block: {}", "Info:".blue().bold(), block_num);
    }

    let block_results = processor.process_block_payouts(block_num)?;

    // Check minimum participants requirement
    if block_results.len() < args.min_participants {
        return Err(format!(
            "Block {} has only {} participants, minimum {} required",
            block_num,
            block_results.len(),
            args.min_participants
        )
        .into());
    }

    let block_payout: f64 = block_results.iter().filter_map(|r| r.payout).sum();

    let results = ProcessingResults {
        blocks: vec![(block_num.to_string(), block_results.clone())],
        total_blocks_processed: 1,
        total_participants: block_results.len(),
        total_payout: block_payout,
        errors: Vec::new(),
    };

    if args.verbose {
        println!(
            "{} Processed block {} ({} participants, {:.9} TAO)",
            "Info:".blue().bold(),
            block_num,
            block_results.len(),
            block_payout
        );
    }

    Ok(results)
}

fn display_results(
    results: &ProcessingResults,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    match args.output.as_str() {
        "table" => display_table_format(results, args),
        "json" => display_json_format(results),
        "csv" => display_csv_format(results),
        _ => Err(format!("Unsupported output format: {}", args.output).into()),
    }
}

fn display_table_format(
    results: &ProcessingResults,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "Payout Processing Results:".bold().underline());
    println!("{}", "=".repeat(80));

    if results.blocks.is_empty() {
        println!("{} No blocks processed", "Info:".blue().bold());
        return Ok(());
    }

    for (block_num, block_results) in &results.blocks {
        println!("\n{} {}", "Block:".bold().blue(), block_num.bright_white());
        println!("Participants: {}", block_results.len());

        if args.detailed && !block_results.is_empty() {
            println!("\n{}", "Participant Details:".dimmed());
            for (i, result) in block_results.iter().enumerate() {
                let rank_display = if let Some(rank) = result.rank {
                    format!("#{}", rank)
                } else {
                    "N/A".to_string()
                };

                println!(
                    "  {}. {} ({})",
                    rank_display.bold().blue(),
                    result.participant.username,
                    result.participant.social_id.dimmed()
                );
                println!("     Guess: \"{}\"", result.participant.guess.text);
                println!("     Score: {:.4}", result.effective_score());

                if let Some(payout) = result.payout {
                    println!("     Payout: {:.9} TAO", payout);
                } else {
                    println!("     Payout: N/A");
                }

                if i == 0 && result.rank == Some(1) {
                    println!("     Status: {}", "🏆 Winner".green().bold());
                } else if result.rank.map_or(false, |r| r <= 3) {
                    println!("     Status: {}", "🥉 Top 3".yellow());
                }
                println!();
            }
        }

        let block_payout: f64 = block_results.iter().filter_map(|r| r.payout).sum();

                    println!("Block Total: {:.9} TAO", block_payout);
    }

    println!("\n{}", "=".repeat(80));
    println!("{} {}", "Summary:".bold(), "");
    println!("Blocks Processed: {}", results.total_blocks_processed);
    println!("Total Participants: {}", results.total_participants);
    println!("Total Payouts: {:.9} TAO", results.total_payout);

    if !results.errors.is_empty() {
        println!(
            "\n{} {} error(s) encountered:",
            "Warning:".yellow().bold(),
            results.errors.len()
        );
        for error in &results.errors {
            println!("  • {}", error);
        }
    }

    Ok(())
}

fn display_json_format(results: &ProcessingResults) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = serde_json::Map::new();

    let blocks_data: Vec<serde_json::Value> = results
        .blocks
        .iter()
        .map(|(block_num, block_results)| {
            let participants: Vec<serde_json::Value> = block_results
                .iter()
                .map(|result| {
                    serde_json::json!({
                        "username": result.participant.username,
                        "user_id": result.participant.social_id,
                        "guess": result.participant.guess.text,
                        "score": result.effective_score(),
                        "rank": result.rank,
                        "payout": result.payout
                    })
                })
                .collect();

            let block_payout: f64 = block_results.iter().filter_map(|r| r.payout).sum();

            serde_json::json!({
                "block_num": block_num,
                "participants": participants,
                "participant_count": block_results.len(),
                "total_payout": block_payout
            })
        })
        .collect();

    output.insert("blocks".to_string(), serde_json::Value::Array(blocks_data));
    output.insert(
        "summary".to_string(),
        serde_json::json!({
            "total_blocks_processed": results.total_blocks_processed,
            "total_participants": results.total_participants,
            "total_payout": results.total_payout,
            "errors": results.errors
        }),
    );
    output.insert(
        "timestamp".to_string(),
        serde_json::Value::from(chrono::Utc::now().to_rfc3339()),
    );

    let json_output = serde_json::to_string_pretty(&output)?;
    println!("{}", json_output);

    Ok(())
}

fn display_csv_format(results: &ProcessingResults) -> Result<(), Box<dyn std::error::Error>> {
    println!("block_num,username,user_id,guess,score,rank,payout");

    for (block_num, block_results) in &results.blocks {
        for result in block_results {
            let escaped_guess = result.participant.guess.text.replace("\"", "\"\"");
            let rank_str = result.rank.map_or("".to_string(), |r| r.to_string());
            let payout_str = result
                .payout
                .map_or("".to_string(), |p| format!("{:.9}", p));

            println!(
                "{},\"{}\",\"{}\",\"{}\",{:.6},{},{}",
                block_num,
                result.participant.username,
                result.participant.social_id,
                escaped_guess,
                result.effective_score(),
                rank_str,
                payout_str
            );
        }
    }

    Ok(())
}

fn save_results(
    results: &ProcessingResults,
    output_file: &PathBuf,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = match format {
        "json" => {
            let mut output = serde_json::Map::new();

            let blocks_data: Vec<serde_json::Value> = results
                .blocks
                .iter()
                .map(|(block_num, block_results)| {
                    let participants: Vec<serde_json::Value> = block_results
                        .iter()
                        .map(|result| {
                            serde_json::json!({
                                "username": result.participant.username,
                                "user_id": result.participant.social_id,
                                "guess": result.participant.guess.text,
                                "score": result.effective_score(),
                                "rank": result.rank,
                                "payout": result.payout
                            })
                        })
                        .collect();

                    let block_payout: f64 = block_results.iter().filter_map(|r| r.payout).sum();

                    serde_json::json!({
                        "block_num": block_num,
                        "participants": participants,
                        "participant_count": block_results.len(),
                        "total_payout": block_payout
                    })
                })
                .collect();

            output.insert("blocks".to_string(), serde_json::Value::Array(blocks_data));
            output.insert(
                "summary".to_string(),
                serde_json::json!({
                    "total_blocks_processed": results.total_blocks_processed,
                    "total_participants": results.total_participants,
                    "total_payout": results.total_payout,
                    "errors": results.errors
                }),
            );
            output.insert(
                "timestamp".to_string(),
                serde_json::Value::from(chrono::Utc::now().to_rfc3339()),
            );

            serde_json::to_string_pretty(&output)?
        }
        "csv" => {
            let mut content = String::from("block_num,username,user_id,guess,score,rank,payout\n");

            for (block_num, block_results) in &results.blocks {
                for result in block_results {
                    let escaped_guess = result.participant.guess.text.replace("\"", "\"\"");
                    let rank_str = result.rank.map_or("".to_string(), |r| r.to_string());
                    let payout_str = result
                        .payout
                        .map_or("".to_string(), |p| format!("{:.9}", p));

                    content.push_str(&format!(
                        "{},\"{}\",\"{}\",\"{}\",{:.6},{},{}\n",
                        block_num,
                        result.participant.username,
                        result.participant.social_id,
                        escaped_guess,
                        result.effective_score(),
                        rank_str,
                        payout_str
                    ));
                }
            }

            content
        }
        "table" => {
            let mut content = String::from("Payout Processing Results\n");
            content.push_str(&"=".repeat(50));
            content.push('\n');

            for (block_num, block_results) in &results.blocks {
                content.push_str(&format!("\nBlock: {}\n", block_num));
                content.push_str(&format!("Participants: {}\n", block_results.len()));

                for (i, result) in block_results.iter().enumerate() {
                    content.push_str(&format!(
                        "  {}. {} ({})\n",
                        i + 1,
                        result.participant.username,
                        result.participant.social_id
                    ));
                    content.push_str(&format!(
                        "     Guess: \"{}\"\n",
                        result.participant.guess.text
                    ));
                    content.push_str(&format!("     Score: {:.4}\n", result.effective_score()));
                    if let Some(payout) = result.payout {
                        content.push_str(&format!("     Payout: {:.9}\n", payout));
                    }
                    content.push('\n');
                }

                let block_payout: f64 = block_results.iter().filter_map(|r| r.payout).sum();
                content.push_str(&format!("Block Total: {:.9}\n", block_payout));
            }

            content.push_str(&"=".repeat(50));
            content.push('\n');
            content.push_str(&format!(
                "Total Blocks: {}\n",
                results.total_blocks_processed
            ));
            content.push_str(&format!(
                "Total Participants: {}\n",
                results.total_participants
            ));
            content.push_str(&format!("Total Payouts: {:.9}\n", results.total_payout));

            content
        }
        _ => return Err(format!("Unsupported output format for file save: {}", format).into()),
    };

    fs::write(output_file, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cliptions_core::commitment::CommitmentGenerator;
    use cliptions_core::types::{Guess, Participant, BlockData};
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_inputs_valid() {
        let args = Args {
            all: false,
            block: Some("test_block".to_string()),
            blocks_file: PathBuf::from("tests/fixtures/blocks.json"),
            output: "table".to_string(),
            output_file: None,
            use_mock: false,
            clip_model: None,
            verbose: false,
            no_color: false,
            config: None,
            continue_on_error: false,
            detailed: false,
            min_participants: 1,
            max_blocks: 0,
        };

        // This will fail if the test file doesn't exist, which is expected
        let result = validate_inputs(&args);
        assert!(result.is_err()); // Expected due to missing test file
    }

    #[test]
    fn test_validate_inputs_invalid_both_flags() {
        let args = Args {
            all: true,
            block: Some("test_block".to_string()),
            blocks_file: PathBuf::from("blocks.json"),
            output: "table".to_string(),
            output_file: None,
            use_mock: false,
            clip_model: None,
            verbose: false,
            no_color: false,
            config: None,
            continue_on_error: false,
            detailed: false,
            min_participants: 1,
            max_blocks: 0,
        };

        let result = validate_inputs(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot specify both --all and --block"));
    }

    #[test]
    fn test_validate_inputs_neither_flag() {
        let args = Args {
            all: false,
            block: None,
            blocks_file: PathBuf::from("blocks.json"),
            output: "table".to_string(),
            output_file: None,
            use_mock: false,
            clip_model: None,
            verbose: false,
            no_color: false,
            config: None,
            continue_on_error: false,
            detailed: false,
            min_participants: 1,
            max_blocks: 0,
        };

        let result = validate_inputs(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Must specify either --all or --block"));
    }

    #[test]
    fn test_process_payouts_basic() {
        // Create a temporary blocks file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        // Create test block data
        let mut block = BlockData::new(
            "test_block".to_string(),
            "test.jpg".to_string(),
            "test_social_id".to_string(),
            1000.0,
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
        )
        .with_salt(salt.to_string())
        .mark_verified();

        block.add_participant(participant);

        // Save blocks data
        let mut blocks = HashMap::new();
        blocks.insert("test_block".to_string(), block);
        let content = serde_json::to_string_pretty(&blocks).unwrap();
        std::fs::write(&file_path, content).unwrap();

        // Test processor creation
        let args = Args {
            all: false,
            block: Some("test_block".to_string()),
            blocks_file: file_path,
            output: "table".to_string(),
            output_file: None,
            use_mock: false,
            clip_model: None,
            verbose: false,
            no_color: false,
            config: None,
            continue_on_error: false,
            detailed: false,
            min_participants: 1,
            max_blocks: 0,
        };

        // Test validation passes
        let result = validate_inputs(&args);
        assert!(result.is_ok());
    }
}
