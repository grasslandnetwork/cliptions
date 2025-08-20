use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use crate::config::ConfigManager;
use crate::error::Result;
use twitter_api::{TwitterClient, TwitterApi};
use chrono::{Duration as ChronoDuration, Utc};
use chrono_tz;
use crate::block_engine::store::{JsonBlockStore, BlockStore};
use crate::block_engine::state_machine::{Block, CommitmentsOpen, CommitmentsClosed, FrameCaptured, RevealsOpen};

#[derive(Parser)]
pub struct PostTargetFrameArgs {
    /// Tweet ID to reply to (the #commitmentsopen tweet)
    #[arg(short, long)]
    pub reply_to: String,

    /// Target frame image file to post
    #[arg(short, long)]
    pub image: PathBuf,

    /// Block number
    #[arg(long)]
    pub block: u64,

    /// Target time in hours from now
    #[arg(long)]
    pub target_time: u64,

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress colored output (useful for scripts/logging)
    #[arg(long)]
    pub no_color: bool,

    /// Quiet mode - only output the tweet data
    #[arg(short, long)]
    pub quiet: bool,

    /// Config file path (default: config/config.yaml)
    #[arg(long, default_value = "config/config.yaml")]
    pub config: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct PostTargetFrameResults {
    tweet_id: String,
    tweet_url: String,
    reply_to_tweet_id: String,
    image_path: String,
    block_num: u64,
    target_time: String,
    posted_at: String,
}

pub async fn run(args: PostTargetFrameArgs) -> Result<()> {
    // Initialize colored output
    if args.no_color || args.quiet {
        colored::control::set_override(false);
    }

    if args.verbose {
        println!("Starting target frame posting...");
        println!("Replying to tweet: {}", args.reply_to);
        println!("Image file: {}", args.image.display());
        println!("Block: {}", args.block);
        println!("Target time: {} hours from now", args.target_time);
    }

    // Validate image file exists
    if !args.image.exists() {
        return Err(format!("Image file does not exist: {}", args.image.display()).into());
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

    // Calculate target time (hours from now)
    let target_time = Utc::now() + ChronoDuration::hours(args.target_time as i64);

    // Format target time as "2025-04-01 | 16:30:57 | EST"
    let eastern = chrono_tz::US::Eastern;
    let target_time_eastern = target_time.with_timezone(&eastern);
    let formatted_target_time = format!(
        "{} | {} | EST",
        target_time_eastern.format("%Y-%m-%d"),
        target_time_eastern.format("%H:%M:%S")
    );

    // Use typestate: load block, set frame, open reveals (tweets image reply), and persist
    let store = JsonBlockStore::new()?;
    let block_id = args.block.to_string();
    let block_open: Block<CommitmentsOpen> = store.load_commitments_open(&block_id)?;
    // Coerce to commitments closed without tweeting again
    let block_closed: Block<CommitmentsClosed> = block_open.into_state();
    // Capture frame path
    let frame_captured: Block<FrameCaptured> = block_closed
        .capture_frame(args.image.clone())?;
    // Open reveals by replying to the parent tweet with the image
    let reveals_open: Block<RevealsOpen> = frame_captured
        .open_reveals(target_time, &client, &args.reply_to)
        .await?;
    // Persist reveals_open state
    store.save(&reveals_open)?;

    // Build output using the data we have; tweet id is not exposed from typestate here
    if !args.quiet {
        println!("✅ Target frame posted and reveals opened!");
        println!("Reply to: {}", args.reply_to);
        println!("Block: {}", args.block);
        println!("Target time: {}", formatted_target_time);
    }

    // Quiet JSON output
    if args.quiet {
        let results = PostTargetFrameResults {
            tweet_id: String::new(),
            tweet_url: String::new(),
            reply_to_tweet_id: args.reply_to.clone(),
            image_path: args.image.display().to_string(),
            block_num: args.block,
            target_time: formatted_target_time,
            posted_at: Utc::now().to_rfc3339(),
        };
        println!("{}", serde_json::to_string_pretty(&results)?);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_post_target_frame_args_parsing() {
        let args = PostTargetFrameArgs::try_parse_from(&[
            "post-target-frame",
            "--reply-to", "123456789",
            "--image", "/path/to/image.jpg",
            "--block", "5",
            "--target-time", "2",
        ]).unwrap();

        assert_eq!(args.reply_to, "123456789");
        assert_eq!(args.image, PathBuf::from("/path/to/image.jpg"));
        assert_eq!(args.block, 5);
        assert_eq!(args.target_time, 2);
        assert!(!args.verbose);
        assert!(!args.no_color);
        assert!(!args.quiet);
        assert_eq!(args.config, "config/config.yaml");
    }

    #[test]
    fn test_post_target_frame_args_with_options() {
        let args = PostTargetFrameArgs::try_parse_from(&[
            "post-target-frame",
            "--reply-to", "123456789",
            "--image", "/path/to/image.jpg",
            "--block", "5",
            "--target-time", "2",
            "--verbose",
            "--no-color",
            "--quiet",
            "--config", "custom_config.yaml",
        ]).unwrap();

        assert_eq!(args.reply_to, "123456789");
        assert_eq!(args.image, PathBuf::from("/path/to/image.jpg"));
        assert_eq!(args.block, 5);
        assert_eq!(args.target_time, 2);
        assert!(args.verbose);
        assert!(args.no_color);
        assert!(args.quiet);
        assert_eq!(args.config, "custom_config.yaml");
    }

    #[test]
    fn test_post_target_frame_args_missing_required() {
        let result = PostTargetFrameArgs::try_parse_from(&[
            "post-target-frame",
            "--reply-to", "123456789",
            // Missing --image
            "--block", "5",
            "--target-time", "2",
        ]);
        assert!(result.is_err());
    }
} 