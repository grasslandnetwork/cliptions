//! Simple Twitter API posting test script
//! 
//! Tests posting tweets using Twitter API v2 with OAuth 1.0a authentication
//! Supports text tweets, replies, and image attachments

use std::env;
use std::path::PathBuf;
use clap::Parser;
use twitter_api::{TwitterClient, TwitterError};

#[derive(Parser)]
#[command(name = "twitter_post")]
#[command(about = "Post a tweet using Twitter API with optional image attachment")]
struct Args {
    /// Tweet text to post
    #[arg(short, long)]
    text: String,
    
    /// Reply to tweet ID (optional)
    #[arg(long)]
    reply_to: Option<String>,
    
    /// Image file to attach (optional)
    #[arg(short, long)]
    image: Option<PathBuf>,
    
    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    if args.verbose {
        println!("Starting Twitter API posting test...");
    }
    
    // Get Twitter API credentials from environment
    let api_key = env::var("TWITTER_API_KEY")
        .expect("TWITTER_API_KEY environment variable not set");
    let api_secret = env::var("TWITTER_API_SECRET")
        .expect("TWITTER_API_SECRET environment variable not set");
    let access_token = env::var("TWITTER_ACCESS_TOKEN")
        .expect("TWITTER_ACCESS_TOKEN environment variable not set");
    let access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET")
        .expect("TWITTER_ACCESS_TOKEN_SECRET environment variable not set");
    
    if args.verbose {
        println!("Credentials loaded from environment");
        println!("Tweet text: {}", args.text);
        if let Some(ref image_path) = args.image {
            println!("Image to upload: {}", image_path.display());
        }
    }
    
    // Create TwitterClient
    let config = twitter_api::TwitterConfig {
        api_key,
        api_secret,
        access_token,
        access_token_secret,
    };
    let client = TwitterClient::new(config);
    
    // Post the tweet
    let result = if let Some(image_path) = &args.image {
        if args.verbose {
            println!("🖼️ Posting tweet with image...");
        }
        
                if let Some(_reply_id) = &args.reply_to {
            // Reply with image - TwitterClient doesn't support reply with image yet
            // For now, we'll post with image (without reply functionality)
            let reply_text = format!("{}", args.text);
            client.post_tweet_with_image(&reply_text, image_path).await
        } else {
            client.post_tweet_with_image(&args.text, image_path).await
        }
    } else if let Some(reply_id) = &args.reply_to {
        if args.verbose {
            println!("💬 Posting reply tweet...");
        }
        client.reply_to_tweet(&args.text, reply_id).await
    } else {
        if args.verbose {
            println!("📝 Posting text tweet...");
        }
        client.post_tweet(&args.text).await
    };
    
    match result {
        Ok(post_result) => {
            let tweet = &post_result.tweet;
            println!("✅ Tweet posted successfully!");
            println!("Tweet ID: {}", tweet.id);
            println!("URL: https://twitter.com/i/status/{}", tweet.id);
            if args.verbose {
                println!("Created: {:?}", tweet.created_at);
                println!("Text: {}", tweet.text);
                if let Some(metrics) = &tweet.public_metrics {
                    println!("Initial metrics: {} retweets, {} likes, {} replies", 
                             metrics.retweet_count, metrics.like_count, metrics.reply_count);
                }
            }
        }
        Err(TwitterError::ApiError { status, message }) => {
            println!("❌ Twitter API error: {} - {}", status, message);
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