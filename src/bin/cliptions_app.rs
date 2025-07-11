//! Cliptions - State-Driven Round Engine
//! 
//! Main application entry point that supports both validator and miner roles.
//! Uses async/await for handling Twitter API calls and web server operations.

use std::env;
use clap::Parser;
use tokio::time::{sleep, Duration};
use cliptions_core::config::ConfigManager;
use cliptions_core::round_engine::state_machine::{Round, Pending};
use chrono::{DateTime, Utc};

#[derive(Parser)]
#[command(name = "cliptions_app")]
#[command(about = "Cliptions - State-Driven Round Engine for Twitter-based games")]
#[command(version)]
struct Args {
    /// Role to run the application as (validator or miner)
    #[arg(short, long, default_value = "miner")]
    role: String,
    
    /// Configuration file path
    #[arg(short, long, default_value = "config/llm.yaml")]
    config: String,
    
    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Port for the web server (fee payment interface)
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

#[derive(Debug, Clone)]
pub enum Role {
    Validator,
    Miner,
}

impl std::str::FromStr for Role {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "validator" => Ok(Role::Validator),
            "miner" => Ok(Role::Miner),
            _ => Err(format!("Invalid role: {}. Must be 'validator' or 'miner'", s)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Parse the role
    let role: Role = args.role.parse()
        .map_err(|e| format!("Invalid role specified: {}", e))?;
    
    if args.verbose {
        println!("🚀 Starting Cliptions App...");
        println!("Role: {:?}", role);
        println!("Config: {}", args.config);
        println!("Web Server Port: {}", args.port);
    }
    
    // Load configuration
    let config_manager = ConfigManager::with_path(&args.config)?;
    let config = config_manager.get_config().clone();
    
    if args.verbose {
        println!("✅ Configuration loaded successfully");
    }
    
    // Initialize clients based on configuration
    // TODO: Initialize TwitterClient and BaseClient once those are implemented
    
    // Start the web server for fee payment verification
    let server_handle = tokio::spawn(start_web_server(args.port, args.verbose));
    
    // Run the main application loop based on role
    match role {
        Role::Validator => {
            println!("🛡️  Starting in VALIDATOR mode...");
            run_validator_loop(config, args.verbose).await?;
        }
        Role::Miner => {
            println!("⛏️  Starting in MINER mode...");
            run_miner_loop(config, args.verbose, args.port).await?;
        }
    }
    
    // Shutdown the web server
    server_handle.abort();
    
    Ok(())
}

async fn start_web_server(port: u16, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("🌐 Starting web server on port {}", port);
    }
    
    // TODO: Implement axum web server for fee payment verification
    // For now, just keep the task alive
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}

async fn run_validator_loop(
    config: cliptions_core::config::CliptionsConfig,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validator: Checking current round state...");
    
    // Example: Create a new round (in a real implementation, this would check existing state)
    let round_id = format!("round_{}", Utc::now().timestamp());
    let mut current_round = Some(Round::new(round_id.clone()));
    
    loop {
        match &current_round {
            Some(round) => {
                println!("📋 Current round: {}", round);
                
                // TODO: Real implementation would:
                // 1. Check Twitter for current round state
                // 2. Parse state from validator's latest tweet
                // 3. Determine next action needed
                // 4. Prompt user for confirmation
                // 5. Execute action (post tweet, process payments, etc.)
                
                if verbose {
                    println!("⏰ Round created at: {}", round.created_at);
                    if let Some(ref path) = round.target_image_path {
                        println!("🎯 Target image: {}", path);
                    }
                    if let Some(ref path) = round.target_frame_path {
                        println!("🖼️  Target frame: {}", path);
                    }
                }
                
                // Example logic for advancing state (in real implementation, user would confirm)
                println!("🤖 [Demo Mode] Validator waiting for manual action...");
                println!("💡 Next actions would be prompted to the user:");
                println!("   - Open commitments with target image");
                println!("   - Close commitments and open fee collection");
                println!("   - Close fee collection and reveal target frame");
                println!("   - Close reveals and process payouts");
            }
            None => {
                println!("🆕 No active round. Waiting for round creation...");
                // In real implementation, this would prompt user to create a new round
            }
        }
        
        if verbose {
            println!("💤 Validator: Waiting for next state check...");
        }
        
        sleep(Duration::from_secs(30)).await;
    }
}

async fn run_miner_loop(
    config: cliptions_core::config::CliptionsConfig,
    verbose: bool,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Miner: Monitoring round state...");
    println!("💰 Fee payment interface available at: http://localhost:{}", port);
    
    let validator_username = &config.twitter.validator_username;
    if validator_username.is_empty() {
        println!("⚠️  Warning: No validator username configured. Using demo mode.");
    }
    
    loop {
        // TODO: Real implementation would:
        // 1. Poll validator's tweets for current round state
        // 2. Parse state from tweets using parse_state_from_string()
        // 3. Display current status to user
        // 4. Guide user through participation steps
        // 5. Help craft commitments and reveals
        
        // Demo logic showing how miner would track state
        println!("🔄 Miner: Checking @{} for round updates...", 
                 if validator_username.is_empty() { "validator" } else { validator_username });
        
        // Example of how state would be displayed to miner
        println!("📊 Round Status Check:");
        println!("   🔍 Searching for latest validator tweet...");
        println!("   📝 Parsing round state from tweet content...");
        println!("   🎯 Determining available actions...");
        
        // In real implementation, this would show actual parsed state
        println!("🤖 [Demo Mode] Sample miner guidance:");
        println!("   ✅ Round detected: CommitmentsOpen");
        println!("   💡 Actions available:");
        println!("      - Submit your commitment hash");
        println!("      - Pay entry fee at: http://localhost:{}", port);
        println!("      - Wait for reveals phase");
        
        if verbose {
            println!("🌐 Web interface status: Running on port {}", port);
            println!("📡 Twitter monitoring: Checking every 30 seconds");
            println!("💰 Fee collection: Ready for wallet connections");
        }
        
        println!("💤 Miner: Waiting for next state check...");
        sleep(Duration::from_secs(30)).await;
    }
} 