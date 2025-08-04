use clap::Parser;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use crate::config::ConfigManager;
use crate::error::Result;
use twitter_api::{TwitterApi, TwitterClient, TwitterError};

#[derive(Parser)]
pub struct CollectCommitmentsArgs {
    /// Tweet ID to search replies for
    #[arg(short, long)]
    pub tweet_id: String,

    /// Maximum results per page (default: 100)
    #[arg(short, long, default_value = "100")]
    pub max_results: u32,

    /// Output format: text, json, csv
    #[arg(long, short, default_value = "text", value_parser = ["text", "json", "csv"])]
    pub output: String,

    /// Save collected commitments to file (JSON format, defaults to ~/.cliptions/validator/collected_commitments.json)
    #[arg(long)]
    pub save_to: Option<PathBuf>,

    /// Don't save collected commitments locally
    #[arg(long)]
    pub no_save: bool,

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress colored output (useful for scripts/logging)
    #[arg(long)]
    pub no_color: bool,

    /// Quiet mode - only output the commitment data
    #[arg(short, long)]
    pub quiet: bool,

    /// Config file path (default: config/config.yaml)
    #[arg(long, default_value = "config/config.yaml")]
    pub config: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CollectedCommitmentData {
    pub username: String,
    pub commitment_hash: String,
    pub wallet_address: String,
    pub tweet_url: String,
    pub timestamp: String,
    pub author_id: String,
    pub conversation_id: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CollectedCommitmentsResults {
    pub commitments: Vec<CollectedCommitmentData>,
    total_collected: usize,
    original_tweet_id: String,
    collection_timestamp: String,
}

pub async fn run(args: CollectCommitmentsArgs) -> Result<()> {
    // Initialize colored output
    if args.no_color || args.quiet {
        colored::control::set_override(false);
    }

    if args.verbose {
        println!("Starting Twitter API reply search...");
        println!("Searching for replies to tweet: {}", args.tweet_id);
    }

    // Load config
    let config_manager = ConfigManager::with_path(&args.config)
        .map_err(|e| format!("Failed to load config file: {}", e))?;
    let config = config_manager.get_config().clone();
    let twitter = &config.twitter;
    println!("\u{2705} Loaded config from: {}", &args.config);

    // Create TwitterClient
    let twitter_config = twitter_api::TwitterConfig {
        api_key: twitter.api_key.clone(),
        api_secret: twitter.api_secret.clone(),
        access_token: twitter.access_token.clone(),
        access_token_secret: twitter.access_token_secret.clone(),
    };
    let client = TwitterClient::new(twitter_config);

    // Search for replies
    let result = client
        .search_replies(&args.tweet_id, args.max_results)
        .await;

    match result {
        Ok(replies) => {
            println!("✅ Search complete!");
            println!("Total replies found: {}", replies.len());

            // Parse and collect commitment data
            let mut collected_commitments = Vec::new();
            
            for (i, reply) in replies.iter().enumerate() {
                if args.verbose {
                    println!("\n--- Reply {} ---", i + 1);
                    println!("🐦 Tweet ID: {}", reply.id);
                    println!("👤 Author ID: {}", reply.author_id);
                    if let Some(created_at) = reply.created_at {
                        println!("📅 Created: {}", created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                    }
                    println!("💬 Text: {}", reply.text);
                    println!("🔗 URL: {}", reply.url);

                    if let Some(conversation_id) = &reply.conversation_id {
                        println!("🔗 Conversation ID: {}", conversation_id);
                    }
                    if let Some(metrics) = &reply.public_metrics {
                        println!(
                            "📊 Metrics: {} retweets, {} likes, {} replies, {} quotes",
                            metrics.retweet_count,
                            metrics.like_count,
                            metrics.reply_count,
                            metrics.quote_count
                        );
                    }
                }

                // Try to parse commitment data from reply text
                if let Some(commitment_data) = parse_commitment_from_reply(reply) {
                    collected_commitments.push(commitment_data);
                }
            }

            // Create results structure
            let results = CollectedCommitmentsResults {
                commitments: collected_commitments.clone(),
                total_collected: collected_commitments.len(),
                original_tweet_id: args.tweet_id.clone(),
                collection_timestamp: chrono::Utc::now().to_rfc3339(),
            };

            // Display results
            display_results(&results, &args)?;

            // Save to file (default behavior unless --no-save is specified)
            if !args.no_save {
                let save_path = if let Some(custom_path) = &args.save_to {
                    custom_path.clone()
                } else {
                    // Default to ~/.cliptions/validator/collected_commitments.json
                    let home_dir = dirs::home_dir()
                        .ok_or_else(|| "Could not determine home directory".to_string())?;
                    let cliptions_dir = home_dir.join(".cliptions");
                    let validator_dir = cliptions_dir.join("validator");
                    
                    // Create directories if they don't exist
                    if !cliptions_dir.exists() {
                        fs::create_dir_all(&cliptions_dir)?;
                    }
                    if !validator_dir.exists() {
                        fs::create_dir_all(&validator_dir)?;
                    }
                    
                    validator_dir.join("collected_commitments.json")
                };

                save_results(&results, &save_path)?;

                if !args.quiet {
                    println!(
                        "{} Collected commitments saved to {}",
                        "Success:".green().bold(),
                        save_path.display()
                    );
                }
            }

            if replies.is_empty() {
                println!("❌ No replies found for tweet {}", args.tweet_id);
                println!("💡 This could mean:");
                println!("   - The tweet has no replies");
                println!("   - The tweet doesn't exist");
                println!("   - The replies are too old (search only covers recent tweets)");
            }
        }
        Err(TwitterError::ApiError { status, message }) => {
            println!("❌ Twitter API error: {} - {}", status, message);
            if status == 404 {
                println!("💡 Make sure the tweet ID exists and is correct");
            }
            return Err(format!("Twitter API error: {} - {}", status, message).into());
        }
        Err(TwitterError::NetworkError(e)) => {
            println!("❌ Network error: {}", e);
            return Err(format!("Network error: {}", e).into());
        }
        Err(TwitterError::AuthError(e)) => {
            println!("❌ Authentication error: {}", e);
            return Err(format!("Authentication error: {}", e).into());
        }
        Err(TwitterError::ParseError(e)) => {
            println!("❌ Response parsing error: {}", e);
            return Err(format!("Response parsing error: {}", e).into());
        }
        Err(TwitterError::MediaError(e)) => {
            println!("❌ Media upload error: {}", e);
            return Err(format!("Media upload error: {}", e).into());
        }
        Err(TwitterError::InvalidInput(e)) => {
            println!("❌ Invalid input: {}", e);
            return Err(format!("Invalid input: {}", e).into());
        }
        Err(TwitterError::FileError(e)) => {
            println!("❌ File error: {}", e);
            return Err(format!("File error: {}", e).into());
        }
        Err(TwitterError::HttpError(e)) => {
            println!("❌ HTTP error: {}", e);
            return Err(format!("HTTP error: {}", e).into());
        }
        Err(TwitterError::SerializationError(e)) => {
            println!("❌ Serialization error: {}", e);
            return Err(format!("Serialization error: {}", e).into());
        }
    }

    Ok(())
}

fn parse_commitment_from_reply(reply: &twitter_api::Tweet) -> Option<CollectedCommitmentData> {
    let text = &reply.text;
    
    // Look for patterns like:
    // "Commit: [hash]"
    // "Wallet: [address]"
    let commit_pattern = regex::Regex::new(r"Commit:\s*([a-fA-F0-9]{64})").ok()?;
    let wallet_pattern = regex::Regex::new(r"Wallet:\s*([^\s\n]+)").ok()?;
    
    let commitment_hash = commit_pattern.captures(text)?.get(1)?.as_str().to_string();
    let wallet_address = wallet_pattern.captures(text)?.get(1)?.as_str().to_string();
    
    // Extract username from author_id (we'll need to get the actual username)
    let username = format!("user_{}", reply.author_id);
    
    Some(CollectedCommitmentData {
        username,
        commitment_hash,
        wallet_address,
        tweet_url: reply.url.clone(),
        timestamp: reply.created_at
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        author_id: reply.author_id.clone(),
        conversation_id: reply.conversation_id.clone(),
    })
}

fn display_results(
    results: &CollectedCommitmentsResults,
    args: &CollectCommitmentsArgs,
) -> Result<()> {
    match args.output.as_str() {
        "text" => display_text_format(results, args),
        "json" => display_json_format(results),
        "csv" => display_csv_format(results),
        _ => unreachable!("Invalid output format should be caught by clap"),
    }
}

fn display_text_format(
    results: &CollectedCommitmentsResults,
    args: &CollectCommitmentsArgs,
) -> Result<()> {
    if args.quiet {
        // In quiet mode, only output the commitment data
        for commitment_data in &results.commitments {
            println!("{}:{}:{}", commitment_data.username, commitment_data.commitment_hash, commitment_data.wallet_address);
        }
        return Ok(());
    }

    if results.commitments.is_empty() {
        println!("{} No valid commitments found in replies", "Warning:".yellow().bold());
        return Ok(());
    }

    if results.commitments.len() == 1 {
        let data = &results.commitments[0];

        if args.verbose {
            println!("{}", "Commitment Collection Results".bold().underline());
            println!("{}: {}", "Username".blue().bold(), data.username);
            println!("{}: {}", "Commitment Hash".green().bold(), data.commitment_hash);
            println!("{}: {}", "Wallet Address".blue().bold(), data.wallet_address);
            println!("{}: {}", "Tweet URL".blue().bold(), data.tweet_url);
            println!("{}: {}", "Timestamp".blue().bold(), data.timestamp);
        } else {
            // Simple format
            println!("Commitment: {}", data.commitment_hash);
            println!("Wallet: {}", data.wallet_address);
        }
    } else {
        // Multiple commitments
        println!(
            "{}",
            format!("Collected {} Commitments", results.total_collected)
                .bold()
                .underline()
        );
        println!();

        for (index, data) in results.commitments.iter().enumerate() {
            println!(
                "{}{}:",
                "Commitment ".blue().bold(),
                (index + 1).to_string().blue().bold()
            );
            println!("  Username: {}", data.username);
            println!("  Hash: {}", data.commitment_hash.green());
            println!("  Wallet: {}", data.wallet_address);
            println!("  URL: {}", data.tweet_url);
            println!("  Time: {}", data.timestamp);
            println!();
        }
    }

    Ok(())
}

fn display_json_format(results: &CollectedCommitmentsResults) -> Result<()> {
    let json_output = serde_json::to_string_pretty(results)?;
    println!("{}", json_output);
    Ok(())
}

fn display_csv_format(results: &CollectedCommitmentsResults) -> Result<()> {
    println!("username,commitment_hash,wallet_address,tweet_url,timestamp,author_id,conversation_id");

    for data in &results.commitments {
        println!(
            "{},{},{},{},{},{},{}",
            csv_escape(&data.username),
            data.commitment_hash,
            csv_escape(&data.wallet_address),
            csv_escape(&data.tweet_url),
            csv_escape(&data.timestamp),
            csv_escape(&data.author_id),
            data.conversation_id.as_deref().map_or("".to_string(), |s| csv_escape(s))
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

fn save_results(
    results: &CollectedCommitmentsResults,
    save_path: &PathBuf,
) -> Result<()> {
    // Load existing commitments if file exists
    let mut existing_commitments = Vec::new();
    if save_path.exists() {
        let file_content = fs::read_to_string(save_path)?;
        if let Ok(existing_results) = serde_json::from_str::<CollectedCommitmentsResults>(&file_content) {
            existing_commitments = existing_results.commitments;
        }
    }

    // Combine existing and new commitments
    let mut all_commitments = existing_commitments;
    all_commitments.extend(results.commitments.clone());

    // Create new results with combined commitments
    let total_count = all_commitments.len();
    let combined_results = CollectedCommitmentsResults {
        commitments: all_commitments,
        total_collected: total_count,
        original_tweet_id: results.original_tweet_id.clone(),
        collection_timestamp: results.collection_timestamp.clone(),
    };

    let json_output = serde_json::to_string_pretty(&combined_results)?;
    fs::write(save_path, json_output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_commitments_args_parsing() {
        let args = CollectCommitmentsArgs {
            tweet_id: "123456789".to_string(),
            max_results: 50,
            output: "json".to_string(),
            save_to: Some(PathBuf::from("test.json")),
            no_save: false,
            verbose: true,
            no_color: false,
            quiet: false,
            config: "test_config.yaml".to_string(),
        };

        assert_eq!(args.tweet_id, "123456789");
        assert_eq!(args.max_results, 50);
        assert_eq!(args.output, "json");
        assert!(args.save_to.is_some());
        assert!(!args.no_save);
        assert!(args.verbose);
        assert_eq!(args.config, "test_config.yaml");
    }

    #[test]
    fn test_collect_commitments_args_defaults() {
        let args = CollectCommitmentsArgs {
            tweet_id: "123456789".to_string(),
            max_results: 100,
            output: "text".to_string(),
            save_to: None,
            no_save: false,
            verbose: false,
            no_color: false,
            quiet: false,
            config: "config/config.yaml".to_string(),
        };

        assert_eq!(args.max_results, 100);
        assert_eq!(args.output, "text");
        assert!(args.save_to.is_none());
        assert!(!args.no_save);
        assert!(!args.verbose);
        assert_eq!(args.config, "config/config.yaml");
    }

    #[test]
    fn test_parse_commitment_from_reply() {
        use twitter_api::Tweet;
        
        let reply = Tweet {
            id: "123456789".to_string(),
            text: "Commit: abc123def4567890abcdef1234567890abcdef1234567890abcdef1234567890\nWallet: 5Co2unDtZKZDzYNZHT2fUMkEnpVWnassfbuabvZmGTrYKgtD".to_string(),
            author_id: "987654321".to_string(),
            url: "https://twitter.com/user/status/123456789".to_string(),
            created_at: Some(chrono::Utc::now()),
            conversation_id: Some("123456789".to_string()),
            public_metrics: None,
        };

        let result = parse_commitment_from_reply(&reply);
        assert!(result.is_some());
        
        let commitment_data = result.unwrap();
        assert_eq!(commitment_data.commitment_hash, "abc123def4567890abcdef1234567890abcdef1234567890abcdef1234567890");
        assert_eq!(commitment_data.wallet_address, "5Co2unDtZKZDzYNZHT2fUMkEnpVWnassfbuabvZmGTrYKgtD");
        assert_eq!(commitment_data.author_id, "987654321");
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(csv_escape("simple"), "simple");
        assert_eq!(csv_escape("with,comma"), "\"with,comma\"");
        assert_eq!(csv_escape("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(csv_escape("with\nline"), "\"with\nline\"");
    }
} 