//! Twitter API reply search tool
//! 
//! Searches for all replies to a specific tweet using Twitter API v2

use std::env;
use clap::Parser;
use twitter_api::{TwitterApi, TwitterClient, TwitterError};
use cliptions_core::config::ConfigManager;

#[derive(Parser)]
#[command(name = "twitter_search_replies")]
#[command(about = "Search for replies to a specific tweet")]
struct Args {
    /// Tweet ID to search replies for
    #[arg(short, long)]
    tweet_id: String,
    
    /// Maximum results per page (default: 100)
    #[arg(short, long, default_value = "100")]
    max_results: u32,
    
    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
    /// Config file path (default: config/llm.yaml)
    #[arg(long, default_value = "config/llm.yaml")]
    config: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    if args.verbose {
        println!("Starting Twitter API reply search...");
        println!("Searching for replies to tweet: {}", args.tweet_id);
    }
    // Load config
    let config_manager = ConfigManager::with_path(&args.config)
        .expect("Failed to load config file");
    let config = config_manager.get_config().clone();
    let twitter = &config.twitter;
    println!("\u2705 Loaded config from: {}", &args.config);
    // Create TwitterClient
    let config = twitter_api::TwitterConfig {
        api_key: twitter.api_key.clone(),
        api_secret: twitter.api_secret.clone(),
        access_token: twitter.access_token.clone(),
        access_token_secret: twitter.access_token_secret.clone(),
    };
    let client = TwitterClient::new(config);
    
    // Search for replies
    let result = client.search_replies(&args.tweet_id, args.max_results).await;
    
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
                        println!("📊 Metrics: {} retweets, {} likes, {} replies, {} quotes", 
                                 metrics.retweet_count, metrics.like_count, 
                                 metrics.reply_count, metrics.quote_count);
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
            std::process::exit(1);
        }
        Err(TwitterError::NetworkError(e)) => {
            println!("❌ Network error: {}", e);
            std::process::exit(1);
        }
        Err(TwitterError::AuthError(e)) => {
            println!("❌ Authentication error: {}", e);
            std::process::exit(1);
        }
        Err(TwitterError::ParseError(e)) => {
            println!("❌ Response parsing error: {}", e);
            std::process::exit(1);
        }
        Err(TwitterError::MediaError(e)) => {
            println!("❌ Media upload error: {}", e);
            std::process::exit(1);
        }
        Err(TwitterError::InvalidInput(e)) => {
            println!("❌ Invalid input: {}", e);
            std::process::exit(1);
        }
        Err(TwitterError::FileError(e)) => {
            println!("❌ File error: {}", e);
            std::process::exit(1);
        }
        Err(TwitterError::HttpError(e)) => {
            println!("❌ HTTP error: {}", e);
            std::process::exit(1);
        }
        Err(TwitterError::SerializationError(e)) => {
            println!("❌ Serialization error: {}", e);
            std::process::exit(1);
        }
    }
} 