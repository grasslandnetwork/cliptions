use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use crate::config::ConfigManager;
use crate::error::Result;
use twitter_api::{TwitterApi, TwitterClient, TwitterError};
use chrono::{Duration as ChronoDuration, Utc};
use chrono_tz;
use crate::social::AnnouncementFormatter;
use crate::twitter_utils::post_tweet_flexible;

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
    pub round: u64,

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

    /// Config file path (default: config/llm.yaml)
    #[arg(long, default_value = "config/llm.yaml")]
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
        println!("Block: {}", args.round);
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

    // Create announcement data for reveals
    let announcement_data = crate::social::AnnouncementData {
        block_num: args.round,
        state_name: "revealsopen".to_string(),
        target_time: formatted_target_time.clone(),
        hashtags: vec![], // The formatter will add standard hashtags
        message: String::new(), // Not used for reveal announcements
        prize_pool: None,
        livestream_url: None, // Optional for reveals
    };

    // Format the reveals announcement
    let formatter = AnnouncementFormatter::new();
    let tweet_text = formatter.create_reveals_announcement(&announcement_data);

    if args.verbose {
        println!("Tweet text: {}", tweet_text);
        println!("Image to upload: {}", args.image.display());
    }

    // Post the tweet with image as reply
    let result = post_tweet_flexible(
        &client,
        &tweet_text,
        Some(&args.reply_to),
        Some(args.image.clone()),
    )
    .await;

    match result {
        Ok(post_result) => {
            let tweet = &post_result.tweet;
            
            let results = PostTargetFrameResults {
                tweet_id: tweet.id.clone(),
                tweet_url: format!("https://twitter.com/i/status/{}", tweet.id),
                reply_to_tweet_id: args.reply_to.clone(),
                image_path: args.image.display().to_string(),
                block_num: args.round,
                target_time: formatted_target_time,
                posted_at: Utc::now().to_rfc3339(),
            };

            if !args.quiet {
                println!("✅ Target frame posted successfully!");
                println!("Tweet ID: {}", results.tweet_id);
                println!("URL: {}", results.tweet_url);
                println!("Reply to: {}", results.reply_to_tweet_id);
                println!("Block: {}", results.block_num);
                println!("Target time: {}", results.target_time);
            }

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

            // Output JSON if quiet mode
            if args.quiet {
                println!("{}", serde_json::to_string_pretty(&results)?);
            }

            Ok(())
        }
        Err(TwitterError::ApiError { status, message }) => {
            let error_msg = format!("Twitter API error: {} - {}", status, message);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            Err(error_msg.into())
        }
        Err(TwitterError::NetworkError(e)) => {
            let error_msg = format!("Network error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            Err(error_msg.into())
        }
        Err(TwitterError::AuthError(e)) => {
            let error_msg = format!("Authentication error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            Err(error_msg.into())
        }
        Err(TwitterError::ParseError(e)) => {
            let error_msg = format!("Response parsing error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            Err(error_msg.into())
        }
        Err(TwitterError::MediaError(e)) => {
            let error_msg = format!("Media upload error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            Err(error_msg.into())
        }
        Err(TwitterError::InvalidInput(e)) => {
            let error_msg = format!("Invalid input: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            Err(error_msg.into())
        }
        Err(TwitterError::FileError(e)) => {
            let error_msg = format!("File error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            Err(error_msg.into())
        }
        Err(TwitterError::HttpError(e)) => {
            let error_msg = format!("HTTP error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            Err(error_msg.into())
        }
        Err(TwitterError::SerializationError(e)) => {
            let error_msg = format!("Serialization error: {}", e);
            if args.quiet {
                eprintln!("{}", error_msg);
            } else {
                println!("❌ {}", error_msg);
            }
            Err(error_msg.into())
        }
    }
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
            "--round", "5",
            "--target-time", "2",
        ]).unwrap();

        assert_eq!(args.reply_to, "123456789");
        assert_eq!(args.image, PathBuf::from("/path/to/image.jpg"));
        assert_eq!(args.round, 5);
        assert_eq!(args.target_time, 2);
        assert!(!args.verbose);
        assert!(!args.no_color);
        assert!(!args.quiet);
        assert_eq!(args.config, "config/llm.yaml");
    }

    #[test]
    fn test_post_target_frame_args_with_options() {
        let args = PostTargetFrameArgs::try_parse_from(&[
            "post-target-frame",
            "--reply-to", "123456789",
            "--image", "/path/to/image.jpg",
            "--round", "5",
            "--target-time", "2",
            "--verbose",
            "--no-color",
            "--quiet",
            "--config", "custom_config.yaml",
        ]).unwrap();

        assert_eq!(args.reply_to, "123456789");
        assert_eq!(args.image, PathBuf::from("/path/to/image.jpg"));
        assert_eq!(args.round, 5);
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
            "--round", "5",
            "--target-time", "2",
        ]);
        assert!(result.is_err());
    }
} 