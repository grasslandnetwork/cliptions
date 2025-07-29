use clap::Parser;
use colored::Colorize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use crate::config::ConfigManager;
use crate::error::Result;
use crate::commitment::CommitmentGenerator;
use serde_json::json;

#[derive(Parser)]
pub struct VerifyCommitmentsArgs {
    /// Block tweet ID from Twitter URL (the original #commitmentsopen tweet)
    #[arg(short, long)]
    pub block_tweet_id: String,

    /// Path to collected commitments file (default: ~/.cliptions/validator/collected_commitments.json)
    #[arg(long)]
    pub commitments_file: Option<PathBuf>,

    /// Path to collected reveals file (default: ~/.cliptions/validator/collected_reveals.json)
    #[arg(long)]
    pub reveals_file: Option<PathBuf>,

    /// Output format: text, json, csv
    #[arg(long, short, default_value = "text", value_parser = ["text", "json", "csv"])]
    pub output: String,

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress colored output
    #[arg(long)]
    pub no_color: bool,

    /// Config file path (default: config/llm.yaml)
    #[arg(long, default_value = "config/llm.yaml")]
    pub config: String,
    
    /// Path to blocks.json file (default: data/blocks.json)
    #[arg(long, default_value = "data/rounds.json")]
    pub rounds_file: PathBuf,
    
    /// Block ID to save results under (e.g., "block4")
    #[arg(long)]
    pub block_num: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct VerificationResult {
    author_id: String,
    username: String,
    wallet_address: String,
    commitment_hash: String,
    guess: String,
    salt: String,
    is_valid: bool,
    verification_error: Option<String>,
    commitment_tweet_url: String,
    reveal_tweet_url: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct VerificationResults {
    block_tweet_id: String,
    total_participants: usize,
    valid_commitments: usize,
    invalid_commitments: usize,
    results: Vec<VerificationResult>,
    verification_timestamp: String,
}

pub async fn run(args: VerifyCommitmentsArgs) -> Result<()> {
    // Initialize colored output
    if args.no_color {
        colored::control::set_override(false);
    }

    if args.verbose {
        println!("Starting commitment verification for round: {}", args.block_tweet_id);
    }

    // Load config
    let _config_manager = ConfigManager::with_path(&args.config)
        .map_err(|e| format!("Failed to load config file: {}", e))?;
    
    // Determine file paths
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?;
    let cliptions_dir = home_dir.join(".cliptions").join("validator");
    
    let commitments_path = args.commitments_file.clone().unwrap_or_else(|| 
        cliptions_dir.join("collected_commitments.json"));
    let reveals_path = args.reveals_file.clone().unwrap_or_else(|| 
        cliptions_dir.join("collected_reveals.json"));

    // Load commitments and reveals
    let commitments = load_commitments(&commitments_path, &args.block_tweet_id)?;
    let reveals = load_reveals(&reveals_path, &args.block_tweet_id)?;

    if args.verbose {
        println!("Loaded {} commitments for round {}", commitments.len(), args.block_tweet_id);
        println!("Loaded {} reveals for round {}", reveals.len(), args.block_tweet_id);
    }

    // Loop through commitments and look for matching reveals
    let mut verification_results = Vec::new();
    let mut valid_count = 0;
    let mut invalid_count = 0;

    for (author_id, commitment) in &commitments {
        if let Some(reveal) = reveals.get(author_id) {
            // Verify the commitment
            let verification_result = verify_commitment(commitment.clone(), reveal, args.verbose);
            if verification_result.is_valid {
                valid_count += 1;
            } else {
                invalid_count += 1;
            }
            verification_results.push(verification_result);
        } else {
            // Commitment without reveal
            let verification_result = VerificationResult {
                author_id: author_id.clone(),
                username: commitment.username.clone(),
                wallet_address: commitment.wallet_address.clone(),
                commitment_hash: commitment.commitment_hash.clone(),
                guess: "".to_string(),
                salt: "".to_string(),
                is_valid: false,
                verification_error: Some("No reveal found".to_string()),
                commitment_tweet_url: commitment.tweet_url.clone(),
                reveal_tweet_url: "".to_string(),
            };
            invalid_count += 1;
            verification_results.push(verification_result);
        }
    }

    let results = VerificationResults {
        block_tweet_id: args.block_tweet_id.clone(),
        total_participants: verification_results.len(),
        valid_commitments: valid_count,
        invalid_commitments: invalid_count,
        results: verification_results,
        verification_timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Display results
    display_verification_results(&results, &args)?;

    // Save to rounds.json if block_num is provided
    if let Some(block_num) = &args.block_num {
        save_to_rounds_json(&results, &args.rounds_file, block_num)?;
        
        if args.verbose {
            println!("✅ Verification results saved to {} under round '{}'", 
                args.rounds_file.display(), block_num);
        }
    }

    Ok(())
}

fn load_commitments(path: &PathBuf, block_tweet_id: &str) -> Result<BTreeMap<String, CollectedCommitmentData>> {
    if !path.exists() {
        return Err("Commitments file not found".to_string().into());
    }

    let content = fs::read_to_string(path)?;
    let results: CollectedCommitmentsResults = serde_json::from_str(&content)?;
    
    // Filter by round
    let mut commitments = BTreeMap::new();
    for commitment in results.commitments {
        if commitment.conversation_id.as_deref() == Some(block_tweet_id) {
            commitments.insert(commitment.author_id.clone(), commitment);
        }
    }
    
    Ok(commitments)
}

fn load_reveals(path: &PathBuf, block_tweet_id: &str) -> Result<BTreeMap<String, CollectedRevealData>> {
    if !path.exists() {
        return Err("Reveals file not found".to_string().into());
    }

    let content = fs::read_to_string(path)?;
    let results: CollectedRevealsResults = serde_json::from_str(&content)?;
    
    // Filter by round
    let mut reveals = BTreeMap::new();
    for reveal in results.reveals {
        if reveal.conversation_id.as_deref() == Some(block_tweet_id) {
            reveals.insert(reveal.author_id.clone(), reveal);
        }
    }
    
    Ok(reveals)
}

fn verify_commitment(
    commitment: CollectedCommitmentData, 
    reveal: &CollectedRevealData, 
    verbose: bool
) -> VerificationResult {
    // Generate hash from guess + salt
    let generator = CommitmentGenerator::new();
    let computed_hash = match generator.generate(&reveal.guess, &reveal.salt) {
        Ok(hash) => hash,
        Err(_) => "ERROR".to_string(),
    };
    
    let is_valid = computed_hash == commitment.commitment_hash;
    let verification_error = if is_valid {
        None
    } else {
        Some(format!("Hash mismatch. Expected: {}, Got: {}", commitment.commitment_hash, computed_hash))
    };

    if verbose {
        println!("Verifying {}: guess='{}', salt='{}'", commitment.author_id, reveal.guess, reveal.salt);
        println!("  Expected hash: {}", commitment.commitment_hash);
        println!("  Computed hash: {}", computed_hash);
        println!("  Valid: {}", is_valid);
    }

    VerificationResult {
        author_id: commitment.author_id,
        username: commitment.username,
        wallet_address: commitment.wallet_address,
        commitment_hash: commitment.commitment_hash,
        guess: reveal.guess.clone(),
        salt: reveal.salt.clone(),
        is_valid,
        verification_error,
        commitment_tweet_url: commitment.tweet_url,
        reveal_tweet_url: reveal.tweet_url.clone(),
    }
}

fn display_verification_results(
    results: &VerificationResults,
    _args: &VerifyCommitmentsArgs,
) -> Result<()> {
    match results.block_tweet_id.as_str() {
        _ => display_text_format(results),
    }
}

fn display_text_format(results: &VerificationResults) -> Result<()> {
    println!("{}", "Commitment Verification Results".bold().underline());
    println!("Block Tweet ID: {}", results.block_tweet_id);
    println!("Total Participants: {}", results.total_participants);
    println!("Valid Commitments: {}", results.valid_commitments.to_string().green());
    println!("Invalid Commitments: {}", results.invalid_commitments.to_string().red());
    println!();

    for (i, result) in results.results.iter().enumerate() {
        println!("Participant {}: {}", i + 1, result.author_id);
        println!("  Username: {}", result.username);
        println!("  Wallet: {}", result.wallet_address);
        println!("  Commitment Hash: {}", result.commitment_hash);
        println!("  Guess: {}", result.guess);
        println!("  Salt: {}", result.salt);
        println!("  Valid: {}", if result.is_valid { "✅".green() } else { "❌".red() });
        if let Some(error) = &result.verification_error {
            println!("  Error: {}", error.red());
        }
        println!();
    }

    Ok(())
}

fn display_json_format(results: &VerificationResults) -> Result<()> {
    let json_output = serde_json::to_string_pretty(results)?;
    println!("{}", json_output);
    Ok(())
}

fn display_csv_format(results: &VerificationResults) -> Result<()> {
    println!("author_id,username,wallet_address,commitment_hash,guess,salt,is_valid,verification_error,commitment_tweet_url,reveal_tweet_url");

    for result in &results.results {
        println!(
            "{},{},{},{},{},{},{},{},{},{}",
            csv_escape(&result.author_id),
            csv_escape(&result.username),
            csv_escape(&result.wallet_address),
            result.commitment_hash,
            csv_escape(&result.guess),
            csv_escape(&result.salt),
            result.is_valid,
            result.verification_error.as_deref().map_or("".to_string(), |s| csv_escape(s)),
            csv_escape(&result.commitment_tweet_url),
            csv_escape(&result.reveal_tweet_url)
        );
    }

    Ok(())
}

fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}


// Add new function to save to rounds.json
fn save_to_rounds_json(
    results: &VerificationResults,
    rounds_file: &PathBuf,
    block_num: &str,
) -> Result<()> {
    // Load existing rounds data
    let mut rounds_data: BTreeMap<String, serde_json::Value> = if rounds_file.exists() {
        let content = fs::read_to_string(rounds_file)?;
        serde_json::from_str(&content)?
    } else {
        BTreeMap::new()
    };

    // Create participants array from verification results
    let participants: Vec<serde_json::Value> = results.results.iter().map(|result| {
        json!({
            "social_id": result.author_id,
            "username": result.username,
            "guess": {
                "text": result.guess,
                "embedding": null,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "metadata": {}
            },
            "guess_url": result.reveal_tweet_url,
            "commitment": result.commitment_hash,
            "commitment_url": result.commitment_tweet_url,
            "wallet": result.wallet_address,
            "score": 0.0, // Will be calculated in Slice 6
            "payout": {
                "amount": 0.0,
                "currency": "TAO",
                "url": ""
            },
            "salt": result.salt,
            "verified": result.is_valid
        })
    }).collect();

            // Create round data that matches BlockData struct
    let round_data = json!({
        "block_version": "1",
        "block_num": block_num,
        "target_image_path": "", // Will be set in other slices
        "status": "Open",
        "prize_pool": 0.0, // Will be set in other slices
        "social_id": "", // Will be set in other slices
        "commitment_deadline": chrono::Utc::now().to_rfc3339(), // Will be set in other slices
        "reveal_deadline": chrono::Utc::now().to_rfc3339(), // Will be set in other slices
        "total_payout": 0.0, // Will be calculated in Slice 6
        "participants": participants,
        "results": [], // Will be populated in Slice 6
        "created_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    // Insert or update the round
    rounds_data.insert(block_num.to_string(), round_data);

    // Save back to file
    let content = serde_json::to_string_pretty(&rounds_data)?;
    fs::write(rounds_file, content)?;

    Ok(())
}

// Import the data structures from other modules
use crate::actions::collect_commitments::{CollectedCommitmentData, CollectedCommitmentsResults};
use crate::actions::collect_reveals::{CollectedRevealData, CollectedRevealsResults}; 