//! Simple Twitter posting tool for Cliptions [[memory:2899338]]
//!
//! This tool allows posting tweets with text, replies, and image attachments.
//! It supports both direct posting and announcement formatting.
//! Supports text tweets, replies, and image attachments

use chrono::{Duration as ChronoDuration, Utc};
use chrono_tz;
use clap::Parser;
use cliptions_core::config::ConfigManager;
use cliptions_core::social::AnnouncementFormatter;
use cliptions_core::twitter_utils::post_tweet_flexible;
use std::path::PathBuf;
use twitter_api::{TwitterApi, TwitterClient, TwitterConfig, TwitterError};

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

    /// Config file path (default: config/llm.yaml)
    #[arg(long, default_value = "config/llm.yaml")]
    config: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Load config
    let config_manager =
        ConfigManager::with_path(&args.config).expect("Failed to load config file");
    let config = config_manager.get_config().clone();
    let twitter = &config.twitter;

    if args.verbose {
        println!("✅ Loaded config from: {}", &args.config);
        println!("TwitterConfig being sent to API: {{");
        println!("  api_key: {}", &twitter.api_key);
        println!(
            "  api_secret: {}...",
            &twitter.api_secret.chars().take(4).collect::<String>()
        );
        println!(
            "  access_token: {}...",
            &twitter.access_token.chars().take(4).collect::<String>()
        );
        println!(
            "  access_token_secret: {}...",
            &twitter
                .access_token_secret
                .chars()
                .take(4)
                .collect::<String>()
        );
        println!("}}\n");
        println!("[DEBUG] Calling post_tweet_flexible from twitter_post.rs");
        println!("  reply_to: {:?}", args.reply_to);
        println!("  image: {:?}", args.image);
    }

    // Get tweet text either from direct input or generate it from state parameters
    let tweet_text = if let (Some(state), Some(round), Some(hours)) =
        (&args.state, args.round, args.target_time)
    {
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
            block_num: round,
            state_name: state.to_string(),
            target_time: formatted_target_time.clone(),
            hashtags: vec![],       // The formatter will add standard hashtags
            message: String::new(), // Not used for commitment announcements
            prize_pool: None,
            livestream_url: args.livestream.clone(), // Optional - None for reveals, Some for commitments
        };

        // Format the announcement based on state
        let formatter = AnnouncementFormatter::new();
        if state.to_lowercase() == "revealsopen" {
            // For reveals announcements, use the dedicated reveals formatter
            formatter.create_reveals_announcement(&announcement_data)
        } else {
            // For commitment announcements, require livestream URL
            if args.livestream.is_none() {
                eprintln!(
                    "❌ Error: --livestream is required for commitment announcements (state: {})",
                    state
                );
                eprintln!("For reveals announcements, --livestream is optional");
                std::process::exit(1);
            }
            // For commitment announcements, use create_commitment_announcement
            formatter.create_commitment_announcement(&announcement_data)
        }
    } else {
        args.text.ok_or_else(|| {
            eprintln!("Either --text or all of --state, --round, and --target-time must be provided");
            eprintln!("Note: --livestream is optional for reveals announcements but required for commitment announcements");
            std::process::exit(1);
        }).unwrap()
    };

    if args.verbose {
        println!("Tweet text: {}", tweet_text);
        if let Some(ref image_path) = args.image {
            println!("Image to upload: {}", image_path.display());
        }
    }

    // Create TwitterClient from config
    let config = TwitterConfig {
        api_key: twitter.api_key.clone(),
        api_secret: twitter.api_secret.clone(),
        access_token: twitter.access_token.clone(),
        access_token_secret: twitter.access_token_secret.clone(),
    };
    let client = TwitterClient::new(config);

    // Clone image path for use in async context
    let image_path = args.image.clone();

    // Post the tweet using the shared utility function
    let result = post_tweet_flexible(
        &client,
        &tweet_text,
        args.reply_to.as_deref(),
        args.image.clone(), // Pass owned PathBuf instead of &Path
    )
    .await;

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
                    println!(
                        "Initial metrics: {} retweets, {} likes, {} replies",
                        metrics.retweet_count, metrics.like_count, metrics.reply_count
                    );
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
