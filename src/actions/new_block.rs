use crate::error::Result;
use clap::Parser;
use crate::config::ConfigManager;
use crate::social::{AnnouncementData, AnnouncementFormatter, TweetCache, TweetCacheManager};
use crate::twitter_utils::post_tweet_flexible;

use twitter_api::{TwitterClient, TwitterConfig};

#[derive(Parser)]
pub struct NewBlockArgs {
    /// Block number
    #[arg(short, long)]
    block_num: u64,

    /// Livestream URL (required for commitment announcements)
    #[arg(short, long)]
    livestream_url: String,

    /// Target time in hours from now
    #[arg(short, long)]
    target_time_hours: u64,

    /// Custom hashtags (optional)
    #[arg(long)]
    hashtags: Option<Vec<String>>,

    /// Custom message (optional)
    #[arg(long)]
    message: Option<String>,

    /// Prize pool amount (optional)
    #[arg(long)]
    prize_pool: Option<f64>,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Suppress non-error output
    #[arg(short, long)]
    quiet: bool,

    /// Config file path (default: config/config.yaml)
    #[arg(long, default_value = "config/config.yaml")]
    config: String,
}

pub async fn run(args: NewBlockArgs) -> Result<()> {
    // Initialize colored output
    if args.no_color || args.quiet {
        colored::control::set_override(false);
    }

    if args.verbose {
        println!("🚀 Starting block opening process...");
        println!("Block number: {}", args.block_num);
        println!("Livestream URL: {}", args.livestream_url);
        println!("Target time: {} hours from now", args.target_time_hours);
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
    let twitter_config = TwitterConfig {
        api_key: twitter.api_key.clone(),
        api_secret: twitter.api_secret.clone(),
        access_token: twitter.access_token.clone(),
        access_token_secret: twitter.access_token_secret.clone(),
    };
    let client = TwitterClient::new(twitter_config);

    // Calculate target time
    let target_time = chrono::Utc::now() + chrono::Duration::hours(args.target_time_hours as i64);
    let eastern = chrono_tz::US::Eastern;
    let target_time_eastern = target_time.with_timezone(&eastern);
    let formatted_target_time = format!(
        "{} | {} | EST",
        target_time_eastern.format("%Y-%m-%d"),
        target_time_eastern.format("%H:%M:%S")
    );

    // Create announcement data
    let announcement_data = AnnouncementData {
        block_num: args.block_num,
        state_name: "commitmentsopen".to_string(),
        target_time: formatted_target_time,
        hashtags: args.hashtags.unwrap_or_default(),
        message: args.message.unwrap_or_default(),
        prize_pool: args.prize_pool,
        livestream_url: Some(args.livestream_url),
    };

    // Format the announcement
    let formatter = AnnouncementFormatter::new();
    let tweet_text = formatter.create_commitment_announcement(&announcement_data);

    if args.verbose {
        println!("📝 Generated tweet text:");
        println!("{}", tweet_text);
    }

    // Post the tweet
    let result = post_tweet_flexible(&client, &tweet_text, None, None).await;

    match result {
        Ok(post_result) => {
            let tweet = &post_result.tweet;
            if !args.quiet {
                println!("✅ Block {} opened successfully!", args.block_num);
                println!("Tweet ID: {}", tweet.id);
                println!("URL: https://twitter.com/i/status/{}", tweet.id);
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

            // Save tweet ID for later use in collect-commitments
            let tweet_cache = TweetCache::new(
                tweet.id.clone(),
                tweet.text.clone(),
                twitter.validator_username.clone(),
            );
            
            let cache_manager = TweetCacheManager::default();
            if let Err(e) = cache_manager.save_cache(&tweet_cache) {
                eprintln!("⚠️  Warning: Failed to save tweet cache: {}", e);
            } else if args.verbose {
                println!("💾 Saved tweet cache for later use");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to open block: {}", e);
            return Err(format!("Twitter API error: {}", e).into());
        }
    }

    Ok(())
} 