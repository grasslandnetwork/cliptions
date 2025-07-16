//! Twitter API latest tweet fetcher
//! 
//! Gets the latest tweet from a specific Twitter username using Twitter API v2

use std::env;
use clap::Parser;
use twitter_api::{TwitterApi, TwitterClient, TwitterError};
use cliptions_core::config::ConfigManager;

#[derive(Parser)]
#[command(name = "twitter_latest_tweet")]
#[command(about = "Get the latest tweet from a Twitter username")]
struct Args {
    /// Twitter username (without @)
    #[arg(short, long)]
    username: String,
    
    /// Exclude retweets and replies
    #[arg(long)]
    exclude_retweets_replies: bool,
    
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
        println!("Starting Twitter API latest tweet fetch...");
        println!("Fetching latest tweet from: @{}", args.username);
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
    
    // Get the latest tweet
    let result = client.get_latest_tweet(&args.username, args.exclude_retweets_replies).await;
    
    match result {
        Ok(Some(tweet)) => {
            println!("✅ Latest tweet found!");
            println!("Tweet ID: {}", tweet.id);
            println!("URL: {}", tweet.url);
            println!("Text: {}", tweet.text);
            
            if args.verbose {
                println!("Author ID: {}", tweet.author_id);
                if let Some(created_at) = tweet.created_at {
                    println!("Created: {}", created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                }
                if let Some(conversation_id) = &tweet.conversation_id {
                    println!("Conversation ID: {}", conversation_id);
                }
                if let Some(metrics) = &tweet.public_metrics {
                    println!("Metrics: {} retweets, {} likes, {} replies, {} quotes", 
                             metrics.retweet_count, metrics.like_count, 
                             metrics.reply_count, metrics.quote_count);
                }
            }
        }
        Ok(None) => {
            println!("❌ No tweets found for user @{}", args.username);
            if args.exclude_retweets_replies {
                println!("💡 Try without --exclude-retweets-replies to see all tweets");
            }
        }
        Err(TwitterError::ApiError { status, message }) => {
            println!("❌ Twitter API error: {} - {}", status, message);
            if status == 404 {
                println!("💡 Make sure the username exists and is spelled correctly");
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