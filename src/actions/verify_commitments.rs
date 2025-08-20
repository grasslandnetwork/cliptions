use clap::Parser;
use colored::Colorize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use crate::config::{ConfigManager, PathManager};
use crate::error::Result;
use crate::commitment::CommitmentGenerator;
use crate::commitment::CommitmentVerifier;
use crate::block_engine::store::{JsonBlockStore, BlockStore};
use crate::block_engine::state_machine::{Block, CommitmentsOpen};
use crate::types::{Participant, Guess};

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

    /// Config file path (default: config/config.yaml)
    #[arg(long, default_value = "config/config.yaml")]
    pub config: String,
    
    /// Block ID to update (e.g., "block4")
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
        println!("Starting commitment verification for block: {}", args.block_tweet_id);
    }

    // Load config
    let _config_manager = ConfigManager::with_path(&args.config)
        .map_err(|e| format!("Failed to load config file: {}", e))?;
    
    // Determine file paths via PathManager (defaults under ~/.cliptions/validator)
    let path_manager = PathManager::new()?;
    let commitments_path = args
        .commitments_file
        .clone()
        .unwrap_or_else(|| path_manager.get_validator_collected_commitments_path());
    let reveals_path = args
        .reveals_file
        .clone()
        .unwrap_or_else(|| path_manager.get_validator_collected_reveals_path());

    // Load commitments and reveals
    let commitments = load_commitments(&commitments_path, &args.block_tweet_id)?;
    let reveals = load_reveals(&reveals_path, &args.block_tweet_id)?;

    if args.verbose {
        println!("Loaded {} commitments for block {}", commitments.len(), args.block_tweet_id);
        println!("Loaded {} reveals for block {}", reveals.len(), args.block_tweet_id);
    }

    // Require a block_num to operate (typestate + store)
    let block_num = args
        .block_num
        .clone()
        .ok_or_else(|| "--block-num is required for verify-commitments".to_string())?;

    // Load unified block via JsonBlockStore at CommitmentsOpen state
    let store = JsonBlockStore::new()?;
    let mut block: Block<CommitmentsOpen> = store.load_commitments_open(&block_num)?;

    // Upsert participants from collected commitments/reveals by social_id
    // Build display records in parallel to preserve output behavior
    let mut verification_results = Vec::new();
    for (author_id, commitment) in &commitments {
        if let Some(reveal) = reveals.get(author_id) {
            // Construct participant from commitment + reveal
            let participant = Participant::new(
                author_id.clone(),
                commitment.username.clone(),
                Guess::new(reveal.guess.clone()),
                commitment.commitment_hash.clone(),
            )
            .with_wallet(commitment.wallet_address.clone())
            .with_guess_url(reveal.tweet_url.clone())
            .with_commitment_url(commitment.tweet_url.clone())
            .with_salt(reveal.salt.clone());

            upsert_participant(&mut block, participant);

            // Prepare display result (validity filled after verification)
            verification_results.push(VerificationResult {
                author_id: author_id.clone(),
                username: commitment.username.clone(),
                wallet_address: commitment.wallet_address.clone(),
                commitment_hash: commitment.commitment_hash.clone(),
                guess: reveal.guess.clone(),
                salt: reveal.salt.clone(),
                is_valid: false, // temp, update after verification
                verification_error: None,
                commitment_tweet_url: commitment.tweet_url.clone(),
                reveal_tweet_url: reveal.tweet_url.clone(),
            });
        } else {
            // No reveal - still upsert a placeholder participant so it appears in the block
            let participant = Participant::new(
                author_id.clone(),
                commitment.username.clone(),
                Guess::new(String::new()),
                commitment.commitment_hash.clone(),
            )
            .with_wallet(commitment.wallet_address.clone())
            .with_commitment_url(commitment.tweet_url.clone());

            upsert_participant(&mut block, participant);

            verification_results.push(VerificationResult {
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
            });
        }
    }

    // Verify via typestate method and persist
    let verifier = CommitmentVerifier::new();
    let verified_count = block.verify_commitments(&verifier);
    store.save(&block)?;

    // Update display validity flags after verification
    let participant_map: std::collections::BTreeMap<String, bool> = block
        .participants
        .iter()
        .map(|p| (p.social_id.clone(), p.verified))
        .collect();

    let mut valid_count = 0usize;
    let mut invalid_count = 0usize;
    for rec in &mut verification_results {
        if let Some(v) = participant_map.get(&rec.author_id) {
            rec.is_valid = *v;
            if *v { valid_count += 1; } else { invalid_count += 1; }
            if !*v && rec.verification_error.is_none() {
                rec.verification_error = Some("Commitment verification failed".to_string());
            }
        } else {
            // Shouldn't happen; treat as invalid
            rec.is_valid = false;
            invalid_count += 1;
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

    // Persisted via JsonBlockStore above; no direct JSON mutations here
    if args.verbose {
        println!("✅ Verification results saved to store under block '{}'", block_num);
        println!("   Marked {} participants as verified", verified_count);
    }

    Ok(())
}

fn load_commitments(path: &PathBuf, block_tweet_id: &str) -> Result<BTreeMap<String, CollectedCommitmentData>> {
    if !path.exists() {
        return Err("Commitments file not found".to_string().into());
    }

    let content = fs::read_to_string(path)?;
    let results: CollectedCommitmentsResults = serde_json::from_str(&content)?;
    
    // Filter by block
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
    
    // Filter by block
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


// Add new function to save to blocks.json
// Upsert participant by social_id into the typestate block
fn upsert_participant(block: &mut Block<CommitmentsOpen>, participant: Participant) {
    if let Some(idx) = block
        .participants
        .iter()
        .position(|p| p.social_id == participant.social_id)
    {
        block.participants[idx] = participant;
    } else {
        block.participants.push(participant);
    }
}

// Import the data structures from other modules
use crate::actions::collect_commitments::{CollectedCommitmentData, CollectedCommitmentsResults};
use crate::actions::collect_reveals::{CollectedRevealData, CollectedRevealsResults}; 