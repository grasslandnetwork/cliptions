//! Simple Twitter API posting test script
//! 
//! Tests posting tweets using Twitter API v2 with OAuth 1.0a authentication
//! Supports text tweets, replies, and image attachments

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use clap::Parser;
use serde_json;

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
    
    // Create HTTP client
    let client = reqwest::Client::new();
    
    // Upload image first if provided
    let media_id = if let Some(image_path) = &args.image {
        if args.verbose {
            println!("🖼️ Uploading image...");
        }
        
        match upload_media(
            &client,
            image_path,
            &api_key,
            &api_secret,
            &access_token,
            &access_token_secret,
            args.verbose,
        ).await {
            Ok(id) => {
                if args.verbose {
                    println!("✅ Image uploaded successfully! Media ID: {}", id);
                }
                Some(id)
            }
            Err(e) => {
                println!("❌ Failed to upload image: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };
    
    // Prepare tweet data
    let mut tweet_data = serde_json::json!({
        "text": args.text
    });
    
    // Add reply if specified
    if let Some(reply_id) = &args.reply_to {
        tweet_data["reply"] = serde_json::json!({
            "in_reply_to_tweet_id": reply_id
        });
    }
    
    // Add media if uploaded
    if let Some(media_id) = media_id {
        tweet_data["media"] = serde_json::json!({
            "media_ids": [media_id]
        });
    }
    
    if args.verbose {
        println!("Tweet data: {}", serde_json::to_string_pretty(&tweet_data).unwrap());
    }
    
    // Twitter API v2 endpoint
    let url = "https://api.twitter.com/2/tweets";
    
    // Create OAuth 1.0a authorization header
    let auth_header = create_oauth_header(
        "POST",
        url,
        &api_key,
        &api_secret,
        &access_token,
        &access_token_secret,
    );
    
    if args.verbose {
        println!("Making request to: {}", url);
    }
    
    // Make the API request
    let response = client
        .post(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&tweet_data)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            let status = resp.status();
            if args.verbose {
                println!("Response status: {}", status);
            }
            
            if status.is_success() {
                let response_text = resp.text().await.unwrap_or_else(|_| "Could not read response".to_string());
                
                if args.verbose {
                    println!("Raw response: {}", response_text);
                }
                
                // Try to parse the response to get tweet ID
                if let Ok(response_data) = serde_json::from_str::<serde_json::Value>(&response_text) {
                    if let Some(tweet_id) = response_data
                        .get("data")
                        .and_then(|data| data.get("id"))
                        .and_then(|id| id.as_str()) 
                    {
                        println!("✅ Tweet posted successfully!");
                        println!("Tweet ID: {}", tweet_id);
                        println!("URL: https://twitter.com/i/status/{}", tweet_id);
                    } else {
                        println!("✅ Tweet posted, but couldn't extract ID from response");
                        println!("Response: {}", response_text);
                    }
                } else {
                    println!("✅ Tweet posted successfully!");
                    println!("Response: {}", response_text);
                }
            } else {
                let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                println!("❌ Twitter API error: {} - {}", status, error_text);
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("❌ HTTP request failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// Upload media file to Twitter and return media_id
async fn upload_media(
    client: &reqwest::Client,
    image_path: &PathBuf,
    consumer_key: &str,
    consumer_secret: &str,
    token: &str,
    token_secret: &str,
    verbose: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    use std::fs;
    
    // Read the image file
    let image_data = fs::read(image_path)?;
    
    if verbose {
        println!("Image file size: {} bytes", image_data.len());
    }
    
    // Validate file size (5MB limit for images)
    if image_data.len() > 5 * 1024 * 1024 {
        return Err("Image file too large (max 5MB)".into());
    }
    
    // Detect media type from file extension
    let media_type = match image_path.extension().and_then(|ext| ext.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png", 
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => return Err("Unsupported image format. Supported: jpg, png, gif, webp".into()),
    };
    
    if verbose {
        println!("Detected media type: {}", media_type);
    }
    
    // Twitter media upload endpoint
    let upload_url = "https://upload.twitter.com/1.1/media/upload.json";
    
    // Create OAuth header for upload request
    let auth_header = create_oauth_header(
        "POST",
        upload_url,
        consumer_key,
        consumer_secret,
        token,
        token_secret,
    );
    
    // Create multipart form
    let form = reqwest::multipart::Form::new()
        .part("media", reqwest::multipart::Part::bytes(image_data)
            .file_name(image_path.file_name().unwrap().to_string_lossy().to_string())
            .mime_str(media_type)?);
    
    if verbose {
        println!("Uploading to: {}", upload_url);
    }
    
    // Make upload request
    let response = client
        .post(upload_url)
        .header("Authorization", auth_header)
        .multipart(form)
        .send()
        .await?;
    
    let status = response.status();
    if verbose {
        println!("Upload response status: {}", status);
    }
    
    if status.is_success() {
        let response_text = response.text().await?;
        
        if verbose {
            println!("Upload response: {}", response_text);
        }
        
        // Parse response to get media_id
        let response_data: serde_json::Value = serde_json::from_str(&response_text)?;
        
        if let Some(media_id) = response_data.get("media_id_string").and_then(|id| id.as_str()) {
            Ok(media_id.to_string())
        } else {
            Err("Could not extract media_id from upload response".into())
        }
    } else {
        let error_text = response.text().await?;
        Err(format!("Media upload failed: {} - {}", status, error_text).into())
    }
}

/// Create OAuth 1.0a authorization header for Twitter API
fn create_oauth_header(
    method: &str,
    url: &str,
    consumer_key: &str,
    consumer_secret: &str,
    token: &str,
    token_secret: &str,
) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    use rand::Rng;
    
    // Generate OAuth parameters
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    
    let nonce: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    // OAuth parameters
    let mut oauth_params = HashMap::new();
    oauth_params.insert("oauth_consumer_key", consumer_key);
    oauth_params.insert("oauth_token", token);
    oauth_params.insert("oauth_signature_method", "HMAC-SHA1");
    oauth_params.insert("oauth_timestamp", &timestamp);
    oauth_params.insert("oauth_nonce", &nonce);
    oauth_params.insert("oauth_version", "1.0");
    
    // Create parameter string for signature
    let mut sorted_params: Vec<_> = oauth_params.iter().collect();
    sorted_params.sort_by_key(|(k, _)| *k);
    
    let param_string = sorted_params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    
    // Create signature base string
    let base_string = format!(
        "{}&{}&{}",
        method.to_uppercase(),
        urlencoding::encode(url),
        urlencoding::encode(&param_string)
    );
    
    // Create signing key
    let signing_key = format!(
        "{}&{}",
        urlencoding::encode(consumer_secret),
        urlencoding::encode(token_secret)
    );
    
    // Generate HMAC-SHA1 signature
    use hmac::{Hmac, Mac};
    use sha1::Sha1;
    
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes()).unwrap();
    mac.update(base_string.as_bytes());
    use base64::Engine;
    let signature = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    
    // Add signature to OAuth parameters
    oauth_params.insert("oauth_signature", &signature);
    
    // Create authorization header
    let auth_params: Vec<String> = oauth_params
        .iter()
        .filter(|(k, _)| k.starts_with("oauth_"))
        .map(|(k, v)| format!("{}=\"{}\"", urlencoding::encode(k), urlencoding::encode(v)))
        .collect();
    
    format!("OAuth {}", auth_params.join(", "))
} 