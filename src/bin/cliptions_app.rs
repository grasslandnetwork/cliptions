//! Cliptions - State-Driven Round Engine
//! 
//! Main application entry point that supports both validator and miner roles.
//! Uses async/await for handling Twitter API calls and web server operations.

use clap::Parser;
use tokio::time::{sleep, Duration};
use cliptions_core::config::ConfigManager;
use cliptions_core::round_engine::state_machine::{Round, Pending};
use cliptions_core::error::Result;
use chrono::{Utc, DateTime};
use std::io::{self, Write};
use twitter_api::{TwitterApi, TwitterClient, TwitterConfig};

#[derive(Parser)]
#[command(name = "cliptions_app")]
#[command(about = "Cliptions [[memory:2899338]] - A CLIP-based prediction market")]
#[command(version = "1.0")]
struct Args {
    /// Role to run as (validator or miner)
    #[arg(short, long, default_value = "miner")]
    role: String,
    
    /// Configuration file path
    #[arg(short, long, default_value = "config/llm.yaml")]
    config: String,
    
    /// Web server port for fee collection
    #[arg(short, long, default_value = "8080")]
    port: u16,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug)]
enum Role {
    Validator,
    Miner,
}

impl std::str::FromStr for Role {
    type Err = String;
    
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "validator" => Ok(Role::Validator),
            "miner" => Ok(Role::Miner),
            _ => Err(format!("Invalid role: {}. Must be 'validator' or 'miner'", s)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let role: Role = args.role.parse()
        .unwrap_or_else(|e| panic!("Invalid role specified: {}", e));
    
    if args.verbose {
        println!("🚀 Starting Cliptions [[memory:2899338]] App");
        println!("Role: {:?}", role);
        println!("Config: {}", args.config);
        println!("Web Server Port: {}", args.port);
    }
    
    let config_manager = ConfigManager::with_path(&args.config)?;
    let config = config_manager.get_config().clone();
    
    if args.verbose {
        println!("✅ Configuration loaded successfully");
    }
    
    // Initialize TwitterClient
    let twitter_config = TwitterConfig {
        api_key: config.twitter.api_key.clone(),
        api_secret: config.twitter.api_secret.clone(),
        access_token: config.twitter.access_token.clone(),
        access_token_secret: config.twitter.access_token_secret.clone(),
    };
    let twitter_client = TwitterClient::new(twitter_config);
    
    // Start the web server for fee payment verification
    let server_handle = tokio::spawn(start_web_server(args.port, args.verbose));
    
    // Run the appropriate role
    match role {
        Role::Validator => {
            println!("🛡️  Starting in VALIDATOR mode...");
            run_validator_loop(config, args.verbose, twitter_client).await?;
        }
        Role::Miner => {
            println!("⛏️  Starting in MINER mode...");
            run_miner_loop(config, args.verbose, twitter_client, args.port).await?;
        }
    }
    
    // Clean up
    server_handle.abort();
    
    Ok(())
}

async fn start_web_server(port: u16, verbose: bool) -> Result<()> {
    if verbose {
        println!("🌐 Starting web server on port {}", port);
    }
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}

/// Helper function to prompt user for input
fn prompt_user(prompt_text: &str) -> String {
    print!("{}", prompt_text);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

async fn run_validator_loop(
    _config: cliptions_core::config::CliptionsConfig,
    _verbose: bool,
    client: TwitterClient,
) -> Result<()> {
    println!("🔍 Validator: Checking for active round...");
    
    // For now, we assume no round is active and prompt to create one.
    // A real implementation would first check Twitter for the latest round state.

    let create_new = prompt_user("No active round found. Create a new one? (y/n): ");
    if create_new.to_lowercase() != "y" {
        println!("Exiting.");
        return Ok(());
    }

    // --- Gather New Round Info ---
    let round_id = prompt_user("Enter Round ID (e.g., '10'): ");
    let description = prompt_user("Enter Round Description/Theme: ");
    let livestream_url = prompt_user("Enter Livestream URL: ");
    let target_timestamp_str = prompt_user("Enter Target Timestamp (YYYY-MM-DD HH:MM:SS): ");
    let target_timestamp = DateTime::parse_from_str(&format!("{} +0000", target_timestamp_str), "%Y-%m-%d %H:%M:%S %z")
        .unwrap_or_else(|e| panic!("Invalid timestamp format: {}", e))
        .with_timezone(&Utc);

    let commitment_hours_str = prompt_user("Enter commitment duration in hours (e.g., 24): ");
    let commitment_hours: i64 = commitment_hours_str.parse()
        .unwrap_or_else(|e| panic!("Invalid hours format: {}", e));
    let commitment_deadline = Utc::now() + chrono::Duration::hours(commitment_hours);
    
    // --- Create and Announce Round ---
    let round = Round::<Pending>::new(round_id, description, livestream_url, target_timestamp);
    println!("📢 Announcing new round on Twitter...");

    match round.open_commitments(commitment_deadline, &client).await {
        Ok(_) => {
            println!("✅ Round announced successfully!");
        },
        Err(e) => {
            eprintln!("🔥 Failed to announce round: {}", e);
            return Err(e);
        }
    }
    
    println!("💤 Validator loop finished for this demo. Exiting.");

    Ok(())
}

async fn run_miner_loop(
    config: cliptions_core::config::CliptionsConfig,
    verbose: bool,
    client: TwitterClient,
    port: u16,
) -> Result<()> {
    println!("🔍 Miner: Monitoring round state...");
    println!("💰 Fee payment interface would be available at: http://localhost:{}", port);
    
    let validator_username = &config.twitter.validator_username;
    if validator_username.is_empty() {
        println!("⚠️  Warning: No validator username configured in llm.yaml. Cannot monitor state.");
        return Ok(());
    }
    
    loop {
        println!("🔄 Miner: Checking @{} for round updates...", validator_username);

        match client.get_latest_tweet(validator_username, true).await {
            Ok(Some(latest_tweet)) => {
                println!("- Found latest tweet: \"{}\"", latest_tweet.text.lines().next().unwrap_or(""));
                // Here, we would parse the state from the tweet text
                // For now, we just print it.
                // let state = state_machine::parse_state_from_string(&latest_tweet.text);
                println!("   (Full state parsing not yet implemented)");

            },
            Ok(None) => {
                println!("- No tweets found for validator @{}.", validator_username);
            },
            Err(e) => {
                eprintln!("🔥 Error fetching validator tweet: {}", e);
            }
        }
        
        if verbose {
            println!("📡 Twitter monitoring: Checking every 60 seconds");
        }
        
        println!("💤 Miner: Waiting for next state check...");
        sleep(Duration::from_secs(60)).await;
    }
} 