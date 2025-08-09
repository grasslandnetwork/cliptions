use clap::Parser;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use crate::config::{ConfigManager, PathManager};
use crate::error::Result;
use twitter_api::{TwitterApi, TwitterClient, TwitterError};

#[derive(Parser)]
pub struct CollectRevealsArgs {
    /// Tweet ID to search replies for (the target frame tweet)
    #[arg(short, long)]
    pub tweet_id: String,

    /// Maximum results per page (default: 100)
    #[arg(short, long, default_value = "100")]
    pub max_results: u32,

    /// Output format: text, json, csv
    #[arg(long, short, default_value = "text", value_parser = ["text", "json", "csv"])]
    pub output: String,

    /// Save collected reveals to file (JSON format, defaults to ~/.cliptions/validator/collected_reveals.json)
    #[arg(long)]
    pub save_to: Option<PathBuf>,

    /// Don't save collected reveals locally
    #[arg(long)]
    pub no_save: bool,

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress colored output (useful for scripts/logging)
    #[arg(long)]
    pub no_color: bool,

    /// Quiet mode - only output the reveal data
    #[arg(short, long)]
    pub quiet: bool,

    /// Show raw Twitter API responses without parsing
    #[arg(long)]
    pub raw: bool,

    /// Config file path (default: config/config.yaml)
    #[arg(long, default_value = "config/config.yaml")]
    pub config: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CollectedRevealData {
    pub username: String,
    pub guess: String,
    pub salt: String,
    pub tweet_url: String,
    pub timestamp: String,
    pub author_id: String,
    pub conversation_id: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CollectedRevealsResults {
    pub reveals: Vec<CollectedRevealData>,
    total_collected: usize,
    original_tweet_id: String,
    collection_timestamp: String,
}

pub async fn run(args: CollectRevealsArgs) -> Result<()> {
    // Initialize colored output
    if args.no_color || args.quiet {
        colored::control::set_override(false);
    }

    if args.verbose {
        println!("Starting Twitter API reply search for reveals...");
        println!("Searching for reveal replies to tweet: {}", args.tweet_id);
    }

    // Load config
    let config_manager = ConfigManager::with_path(&args.config)
        .map_err(|e| format!("Failed to load config file: {}", e))?;
    let config = config_manager.get_config().clone();
    let twitter = &config.twitter;
    
    if !args.quiet {
        println!("✅ Loaded config from: {}", &args.config);
    }

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
            if !args.quiet {
                println!("✅ Search complete!");
                println!("Total replies found: {}", replies.len());
            }

            // If raw mode is enabled, just show all raw results
            if args.raw {
                println!("\n=== RAW TWITTER API RESPONSES ===");
                for (i, reply) in replies.iter().enumerate() {
                    println!("\n--- Raw Reply {} ---", i + 1);
                    println!("ID: {}", reply.id);
                    println!("Author ID: {}", reply.author_id);
                    println!("Text: {}", reply.text);
                    println!("URL: {}", reply.url);
                    if let Some(created_at) = reply.created_at {
                        println!("Created: {}", created_at);
                    }
                    if let Some(conversation_id) = &reply.conversation_id {
                        println!("Conversation ID: {}", conversation_id);
                    }
                    if let Some(metrics) = &reply.public_metrics {
                        println!("Metrics: {:?}", metrics);
                    }
                    println!("--- End Raw Reply {} ---", i + 1);
                }
                return Ok(());
            }

            // Parse and collect reveal data
            let mut collected_reveals = Vec::new();
            
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

                // Try to parse reveal data from reply text
                if let Some(reveal_data) = parse_reveal_from_reply(reply) {
                    collected_reveals.push(reveal_data);
                }
            }

            // Create results structure
            let results = CollectedRevealsResults {
                reveals: collected_reveals.clone(),
                total_collected: collected_reveals.len(),
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
                    let path_manager = PathManager::new()?;
                    path_manager.get_validator_collected_reveals_path()
                };

                save_results(&results, &save_path)?;

                if !args.quiet {
                    println!(
                        "{} Collected reveals saved to {}",
                        "Success:".green().bold(),
                        save_path.display()
                    );
                }
            }

            if replies.is_empty() {
                if !args.quiet {
                    println!("❌ No replies found for tweet {}", args.tweet_id);
                    println!("💡 This could mean:");
                    println!("   - The tweet has no replies");
                    println!("   - The tweet doesn't exist");
                    println!("   - The replies are too old (search only covers recent tweets)");
                }
            }
        }
        Err(TwitterError::ApiError { status, message }) => {
            let error_msg = format!("Twitter API error: {} - {}", status, message);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
                if status == 404 {
                    println!("💡 Make sure the tweet ID exists and is correct");
                }
            }
            return Err(error_msg.into());
        }
        Err(TwitterError::NetworkError(e)) => {
            let error_msg = format!("Network error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            return Err(error_msg.into());
        }
        Err(TwitterError::AuthError(e)) => {
            let error_msg = format!("Authentication error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            return Err(error_msg.into());
        }
        Err(TwitterError::ParseError(e)) => {
            let error_msg = format!("Response parsing error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            return Err(error_msg.into());
        }
        Err(TwitterError::MediaError(e)) => {
            let error_msg = format!("Media upload error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            return Err(error_msg.into());
        }
        Err(TwitterError::InvalidInput(e)) => {
            let error_msg = format!("Invalid input: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            return Err(error_msg.into());
        }
        Err(TwitterError::FileError(e)) => {
            let error_msg = format!("File error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            return Err(error_msg.into());
        }
        Err(TwitterError::HttpError(e)) => {
            let error_msg = format!("HTTP error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            return Err(error_msg.into());
        }
        Err(TwitterError::SerializationError(e)) => {
            let error_msg = format!("Serialization error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            return Err(error_msg.into());
        }
    }

    Ok(())
}

fn parse_reveal_from_reply(reply: &twitter_api::Tweet) -> Option<CollectedRevealData> {
    let text = &reply.text;
    
    // Look for patterns like:
    // "Guess: [text]"
    // "Salt: [salt]"
    let guess_pattern = regex::Regex::new(r"Guess:\s*(.+)").ok()?;
    let salt_pattern = regex::Regex::new(r"Salt:\s*([^\s\n]+)").ok()?;
    
    // Try to find guess and salt in the text
    let guess_capture = guess_pattern.captures(text)?;
    let salt_capture = salt_pattern.captures(text)?;
    
    let guess = guess_capture.get(1)?.as_str().trim().to_string();
    let salt = salt_capture.get(1)?.as_str().trim().to_string();

    if guess == "[your-guess]" && salt == "[your-salt]" { // Ignore placeholder values from the validator tweet
        return None;
    }
    
    // Extract username from author_id (we'll need to get the actual username)
    let username = format!("user_{}", reply.author_id);
    
    Some(CollectedRevealData {
        username,
        guess,
        salt,
        tweet_url: reply.url.clone(),
        timestamp: reply.created_at
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        author_id: reply.author_id.clone(),
        conversation_id: reply.conversation_id.clone(),
    })
}

fn display_results(
    results: &CollectedRevealsResults,
    args: &CollectRevealsArgs,
) -> Result<()> {
    match args.output.as_str() {
        "text" => display_text_format(results, args),
        "json" => display_json_format(results),
        "csv" => display_csv_format(results),
        _ => unreachable!("Invalid output format should be caught by clap"),
    }
}

fn display_text_format(
    results: &CollectedRevealsResults,
    args: &CollectRevealsArgs,
) -> Result<()> {
    if args.quiet {
        // In quiet mode, only output the reveal data
        for reveal_data in &results.reveals {
            println!("{}:{}:{}", reveal_data.username, reveal_data.guess, reveal_data.salt);
        }
        return Ok(());
    }

    if results.reveals.is_empty() {
        println!("{} No valid reveals found in replies", "Warning:".yellow().bold());
        return Ok(());
    }

    if results.reveals.len() == 1 {
        let data = &results.reveals[0];

        if args.verbose {
            println!("{}", "Reveal Collection Results".bold().underline());
            println!("{}: {}", "Username".blue().bold(), data.username);
            println!("{}: {}", "Guess".green().bold(), data.guess);
            println!("{}: {}", "Salt".blue().bold(), data.salt);
            println!("{}: {}", "Tweet URL".blue().bold(), data.tweet_url);
            println!("{}: {}", "Timestamp".blue().bold(), data.timestamp);
        } else {
            // Simple format
            println!("Guess: {}", data.guess);
            println!("Salt: {}", data.salt);
        }
    } else {
        // Multiple reveals
        println!(
            "{}",
            format!("Collected {} Reveals", results.total_collected)
                .bold()
                .underline()
        );
        println!();

        for (index, data) in results.reveals.iter().enumerate() {
            println!(
                "{}{}:",
                "Reveal ".blue().bold(),
                (index + 1).to_string().blue().bold()
            );
            println!("  Username: {}", data.username);
            println!("  Guess: {}", data.guess.green());
            println!("  Salt: {}", data.salt);
            println!("  URL: {}", data.tweet_url);
            println!("  Time: {}", data.timestamp);
            println!();
        }
    }

    Ok(())
}

fn display_json_format(results: &CollectedRevealsResults) -> Result<()> {
    let json_output = serde_json::to_string_pretty(results)?;
    println!("{}", json_output);
    Ok(())
}

fn display_csv_format(results: &CollectedRevealsResults) -> Result<()> {
    println!("username,guess,salt,tweet_url,timestamp,author_id,conversation_id");

    for data in &results.reveals {
        println!(
            "{},{},{},{},{},{},{}",
            csv_escape(&data.username),
            csv_escape(&data.guess),
            csv_escape(&data.salt),
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
    results: &CollectedRevealsResults,
    save_path: &PathBuf,
) -> Result<()> {
    // Load existing reveals if file exists
    let mut existing_reveals = Vec::new();
    if save_path.exists() {
        let file_content = fs::read_to_string(save_path)?;
        if let Ok(existing_results) = serde_json::from_str::<CollectedRevealsResults>(&file_content) {
            existing_reveals = existing_results.reveals;
        }
    }

    // Combine existing and new reveals
    let mut all_reveals = existing_reveals;
    all_reveals.extend(results.reveals.clone());

    // Create new results with combined reveals
    let total_count = all_reveals.len();
    let combined_results = CollectedRevealsResults {
        reveals: all_reveals,
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
    fn test_collect_reveals_args_parsing() {
        let args = CollectRevealsArgs {
            tweet_id: "123456789".to_string(),
            max_results: 50,
            output: "json".to_string(),
            save_to: Some(PathBuf::from("test.json")),
            no_save: false,
            verbose: true,
            no_color: false,
            quiet: false,
            raw: false,
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
    fn test_collect_reveals_args_defaults() {
        let args = CollectRevealsArgs {
            tweet_id: "123456789".to_string(),
            max_results: 100,
            output: "text".to_string(),
            save_to: None,
            no_save: false,
            verbose: false,
            no_color: false,
            quiet: false,
            raw: false,
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
    fn test_parse_reveal_from_reply() {
        use twitter_api::Tweet;
        
        let reply = Tweet {
            id: "123456789".to_string(),
            text: "Guess: A beautiful sunset over the ocean\nSalt: my_secret_salt_123".to_string(),
            author_id: "987654321".to_string(),
            url: "https://twitter.com/user/status/123456789".to_string(),
            created_at: Some(chrono::Utc::now()),
            conversation_id: Some("123456789".to_string()),
            public_metrics: None,
        };

        let result = parse_reveal_from_reply(&reply);
        assert!(result.is_some());
        
        let reveal_data = result.unwrap();
        assert_eq!(reveal_data.guess, "A beautiful sunset over the ocean");
        assert_eq!(reveal_data.salt, "my_secret_salt_123");
        assert_eq!(reveal_data.author_id, "987654321");
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(csv_escape("simple"), "simple");
        assert_eq!(csv_escape("with,comma"), "\"with,comma\"");
        assert_eq!(csv_escape("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(csv_escape("with\nline"), "\"with\nline\"");
    }
} 