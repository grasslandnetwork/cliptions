//! Simple Twitter posting tool for Cliptions [[memory:2899338]]
//! 
//! This tool allows posting tweets with text, replies, and image attachments.
//! It supports both direct posting and announcement formatting.
//! Supports text tweets, replies, and image attachments

use clap::Parser;
use std::env;
use std::path::PathBuf;
use twitter_api::{TwitterApi, TwitterClient, TwitterConfig, TwitterError};
use chrono::{Utc, Duration as ChronoDuration};
use chrono_tz;
use cliptions_core::social::AnnouncementFormatter;

#[derive(Parser)]
#[command(name = "twitter_post")]
#[command(about = "Post a tweet using Twitter API with optional image attachment")]
struct Args {
    /// Tweet text to post (optional - will be generated if state parameters provided)
    #[arg(short, long)]
    text: Option<String>,
    
    /// State name (e.g., commitmentsopen)
    #[arg(long)]
    state: Option<String>,
    
    /// Round number
    #[arg(long)]
    round: Option<u64>,
    
    /// Livestream URL
    #[arg(long)]
    livestream: Option<String>,
    
    /// Target time in hours from now
    #[arg(long)]
    target_time: Option<u64>,
    
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
    
         // Get tweet text either from direct input or generate it from state parameters
     let tweet_text = if let (Some(state), Some(round), Some(livestream), Some(hours)) = 
         (&args.state, args.round, &args.livestream, args.target_time) {
         // Calculate target time (hours from now)
         let target_time = Utc::now() + ChronoDuration::hours(hours as i64);
         
         // Format target time as "2025-04-01 | 16:30:57 | EST"
         let eastern = chrono_tz::US::Eastern;
         let target_time_eastern = target_time.with_timezone(&eastern);
         let formatted_target_time = format!(
             "{} | {} | EST",
             target_time_eastern.format("%Y-%m-%d"),
             target_time_eastern.format("%H:%M:%S")
         );
         
         // Create announcement data
         let announcement_data = cliptions_core::social::AnnouncementData {
             round_id: round,
             state_name: state.to_string(),
             target_time: formatted_target_time,
             hashtags: vec![], // The formatter will add standard hashtags
             message: String::new(), // Not used for commitment announcements
             prize_pool: None,
             livestream_url: Some(livestream.to_string()),
         };
         
         // Format the announcement
         let formatter = AnnouncementFormatter::new();
         formatter.create_commitment_announcement(&announcement_data)
     } else {
         args.text.ok_or_else(|| {
             eprintln!("Either --text or all of --state, --round, --livestream, and --target-time must be provided");
             std::process::exit(1);
         }).unwrap()
     };

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
        println!("Tweet text: {}", tweet_text);
        if let Some(ref image_path) = args.image {
            println!("Image to upload: {}", image_path.display());
        }
    }
    
    // Create TwitterClient
    let config = TwitterConfig {
        api_key,
        api_secret,
        access_token,
        access_token_secret,
    };
    let client = TwitterClient::new(config);
    
    // Clone image path for use in async context
    let image_path = args.image.clone();
    
    // Post the tweet using the existing logic
    let result = if let Some(image_path) = image_path {
        if args.verbose {
            println!("🖼️ Posting tweet with image...");
        }
        
                if let Some(_reply_id) = &args.reply_to {
            // Reply with image - TwitterClient doesn't support reply with image yet
            // For now, we'll post with image (without reply functionality)
            let reply_text = format!("{}", tweet_text);
            client.post_tweet_with_image(&reply_text, image_path.clone()).await
        } else {
            client.post_tweet_with_image(&tweet_text, image_path).await
        }
    } else if let Some(reply_id) = &args.reply_to {
        if args.verbose {
            println!("💬 Posting reply tweet...");
        }
        client.reply_to_tweet(&tweet_text, reply_id).await
    } else {
        if args.verbose {
            println!("📝 Posting text tweet...");
        }
        client.post_tweet(&tweet_text).await
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