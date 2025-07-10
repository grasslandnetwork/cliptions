use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[derive(Debug, Deserialize)]
struct PaymentVerificationRequest {
    twitter_handle: String,
    wallet_address: String,
    message: String,
    signature: String,
}

#[derive(Debug, Serialize)]
struct PaymentVerificationResponse {
    success: bool,
    message: String,
    twitter_handle: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// Simple in-memory storage for verified users
type VerifiedUsers = Arc<Mutex<HashMap<String, String>>>; // twitter_handle -> wallet_address

#[derive(Clone)]
struct AppState {
    verified_users: VerifiedUsers,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse command line arguments
    let matches = Command::new("cliptions_app")
        .about("Cliptions state-driven round engine")
        .arg(
            Arg::new("role")
                .long("role")
                .value_name("ROLE")
                .help("Role to run as: validator or miner")
                .value_parser(["validator", "miner"])
                .default_value("miner"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("PORT")
                .help("Port to run the web server on")
                .default_value("3000"),
        )
        .get_matches();

    let role = matches.get_one::<String>("role").unwrap();
    let port = matches.get_one::<String>("port").unwrap();

    println!("🎯 Starting Cliptions App in {} mode", role);

    // Initialize shared state
    let app_state = AppState {
        verified_users: Arc::new(Mutex::new(HashMap::new())),
    };

    // Start the web server
    let web_server_task = tokio::spawn(start_web_server(app_state.clone(), port.clone()));

    println!("🌐 Web server starting on http://localhost:{}", port);
    println!("💳 Fee payment interface available at http://localhost:{}/", port);

    // Role-specific logic
    match role.as_str() {
        "validator" => {
            println!("🔒 Running as VALIDATOR");
            // TODO: Implement validator logic
            validator_main_loop(app_state).await?;
        }
        "miner" => {
            println!("⛏️  Running as MINER");
            // TODO: Implement miner logic
            miner_main_loop(app_state).await?;
        }
        _ => unreachable!("Invalid role"), // clap should prevent this
    }

    // Wait for web server to complete (it won't unless there's an error)
    web_server_task.await??;

    Ok(())
}

async fn start_web_server(state: AppState, port: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Build the router
    let app = Router::new()
        .route("/verify-payment", post(verify_payment_handler))
        .route("/status", get(status_handler))
        .nest_service("/", ServeDir::new("fee_frontend"))
        .with_state(state);

    // Start the server
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("🚀 Web server listening on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn verify_payment_handler(
    State(state): State<AppState>,
    Json(payload): Json<PaymentVerificationRequest>,
) -> Result<Json<PaymentVerificationResponse>, (StatusCode, Json<ErrorResponse>)> {
    println!("🔍 Verifying payment for user: {}", payload.twitter_handle);

    // Validate the signature
    match verify_signature(&payload.message, &payload.signature, &payload.wallet_address) {
        Ok(true) => {
            // Store the verified user
            let mut verified_users = state.verified_users.lock().unwrap();
            verified_users.insert(payload.twitter_handle.clone(), payload.wallet_address.clone());
            
            println!("✅ Successfully verified payment for {}", payload.twitter_handle);
            
            Ok(Json(PaymentVerificationResponse {
                success: true,
                message: "Payment verified successfully".to_string(),
                twitter_handle: Some(payload.twitter_handle),
            }))
        }
        Ok(false) => {
            println!("❌ Invalid signature for {}", payload.twitter_handle);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid signature".to_string(),
                }),
            ))
        }
        Err(e) => {
            println!("🚨 Error verifying signature for {}: {}", payload.twitter_handle, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Verification error: {}", e),
                }),
            ))
        }
    }
}

async fn status_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let verified_users = state.verified_users.lock().unwrap();
    let user_count = verified_users.len();
    
    Json(serde_json::json!({
        "status": "running",
        "verified_users_count": user_count,
        "message": "Cliptions fee verification service is running"
    }))
}

fn verify_signature(
    _message: &str,
    signature_hex: &str,
    _expected_address: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    // For now, we'll implement a simplified verification
    // In production, you'd want proper signature verification
    
    // Remove "0x" prefix if present
    let signature_hex = signature_hex.strip_prefix("0x").unwrap_or(signature_hex);
    
    // For this MVP, we'll accept any signature that looks valid (hex string of correct length)
    // Real implementation would use ethers to verify the signature
    if signature_hex.len() == 130 && signature_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        println!("📝 Mock signature verification passed for address: {}", _expected_address);
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn validator_main_loop(state: AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔄 Starting validator main loop...");
    
    loop {
        // TODO: Implement state machine polling and round management
        println!("🔒 Validator: Checking round state...");
        
        // For now, just show verified users count
        {
            let verified_users = state.verified_users.lock().unwrap();
            if !verified_users.is_empty() {
                println!("👥 Verified users: {}", verified_users.len());
                for (twitter, wallet) in verified_users.iter() {
                    println!("  - @{}: {}", twitter, wallet);
                }
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}

async fn miner_main_loop(state: AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔄 Starting miner main loop...");
    
    loop {
        // TODO: Implement round state polling and miner guidance
        println!("⛏️  Miner: Checking for active rounds...");
        
        // Show current verification status
        {
            let verified_users = state.verified_users.lock().unwrap();
            println!("💳 Total verified users in this session: {}", verified_users.len());
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
} 