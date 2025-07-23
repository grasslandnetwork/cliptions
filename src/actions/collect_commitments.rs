use clap::Parser;
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

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Config file path (default: config/llm.yaml)
    #[arg(long, default_value = "config/llm.yaml")]
    pub config: String,
}

pub async fn run(args: CollectCommitmentsArgs) -> Result<()> {
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

            for (i, reply) in replies.iter().enumerate() {
                println!("\n--- Reply {} ---", i + 1);
                println!("🐦 Tweet ID: {}", reply.id);
                println!("👤 Author ID: {}", reply.author_id);
                if let Some(created_at) = reply.created_at {
                    println!("📅 Created: {}", created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                }
                println!("💬 Text: {}", reply.text);
                println!("🔗 URL: {}", reply.url);

                if args.verbose {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_commitments_args_parsing() {
        let args = CollectCommitmentsArgs {
            tweet_id: "123456789".to_string(),
            max_results: 50,
            verbose: true,
            config: "test_config.yaml".to_string(),
        };

        assert_eq!(args.tweet_id, "123456789");
        assert_eq!(args.max_results, 50);
        assert!(args.verbose);
        assert_eq!(args.config, "test_config.yaml");
    }

    #[test]
    fn test_collect_commitments_args_defaults() {
        let args = CollectCommitmentsArgs {
            tweet_id: "123456789".to_string(),
            max_results: 100,
            verbose: false,
            config: "config/llm.yaml".to_string(),
        };

        assert_eq!(args.max_results, 100);
        assert!(!args.verbose);
        assert_eq!(args.config, "config/llm.yaml");
    }
} 