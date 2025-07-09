//! Twitter API latest tweet fetcher
//! 
//! Gets the latest tweet from a specific Twitter username using Twitter API v2

use std::env;
use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "twitter_latest_tweet")]
#[command(about = "Get the latest tweet from a Twitter username")]
struct Args {
    /// Twitter username (without @)
    #[arg(short, long)]
    username: String,
    
    /// Exclude retweets and replies
    #[arg(long)]
    exclude_retweets_replies: bool,
    
    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    if args.verbose {
        println!("Starting Twitter API latest tweet fetch...");
        println!("Fetching latest tweet from: @{}", args.username);
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
    
    // Create HTTP client
    let client = reqwest::Client::new();
    
    // Step 1: Get user ID from username
    if args.verbose {
        println!("Step 1: Getting user ID for @{}", args.username);
    }
    
    let user_lookup_url = format!("https://api.twitter.com/2/users/by/username/{}", args.username);
    
    if args.verbose {
        println!("User lookup URL: {}", user_lookup_url);
    }
    
    let user_auth_header = create_oauth_header_with_params(
        "GET",
        &user_lookup_url,
        None,
        &api_key,
        &api_secret,
        &access_token,
        &access_token_secret,
    );
    
    let user_response = client
        .get(&user_lookup_url)
        .header("Authorization", user_auth_header)
        .send()
        .await;
    
    let user_id = match user_response {
        Ok(resp) => {
            let status = resp.status();
            if args.verbose {
                println!("User lookup status: {}", status);
            }
            
            if status.is_success() {
                let response_text = resp.text().await.unwrap_or_else(|_| "Could not read response".to_string());
                
                if args.verbose {
                    println!("User lookup response: {}", response_text);
                }
                
                if let Ok(user_data) = serde_json::from_str::<Value>(&response_text) {
                    if let Some(data) = user_data.get("data") {
                        if let Some(id) = data.get("id") {
                            if let Some(user_id_str) = id.as_str() {
                                if args.verbose {
                                    println!("Found user ID: {}", user_id_str);
                                }
                                user_id_str.to_string()
                            } else {
                                println!("❌ Failed to extract user ID from response");
                                return;
                            }
                        } else {
                            println!("❌ No user ID found in response");
                            return;
                        }
                    } else {
                        println!("❌ User '{}' not found", args.username);
                        println!("💡 Make sure the username exists and is spelled correctly");
                        return;
                    }
                } else {
                    println!("❌ Failed to parse user lookup response");
                    return;
                }
            } else {
                let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                println!("❌ User lookup failed: {} - {}", status, error_text);
                return;
            }
        }
        Err(e) => {
            println!("❌ User lookup request failed: {}", e);
            return;
        }
    };
    
    // Step 2: Get tweets using user ID
    if args.verbose {
        println!("Step 2: Getting tweets for user ID {}", user_id);
    }
    
    let mut tweets_url = format!(
        "https://api.twitter.com/2/users/{}/tweets?max_results=5&tweet.fields=created_at,author_id,public_metrics,conversation_id&user.fields=username,name,verified&expansions=author_id",
        user_id
    );
    
    // Add exclude parameter if requested
    if args.exclude_retweets_replies {
        tweets_url.push_str("&exclude=retweets,replies");
    }
    
    if args.verbose {
        println!("Tweets URL: {}", tweets_url);
    }
    
    // Create OAuth 1.0a authorization header for tweets request
    let (base_url, query_params) = if let Some(pos) = tweets_url.find('?') {
        let base = &tweets_url[..pos];
        let params = &tweets_url[pos+1..];
        (base, Some(params))
    } else {
        (tweets_url.as_str(), None)
    };
    
    let auth_header = create_oauth_header_with_params(
        "GET",
        base_url,
        query_params,
        &api_key,
        &api_secret,
        &access_token,
        &access_token_secret,
    );
    
    // Make the tweets API request
    let response = client
        .get(&tweets_url)
        .header("Authorization", auth_header)
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
                
                // Parse the response
                if let Ok(response_data) = serde_json::from_str::<Value>(&response_text) {
                    // Check if we have data
                    if let Some(data) = response_data.get("data") {
                        if let Some(tweets) = data.as_array() {
                            if args.verbose {
                                println!("Found {} tweets, analyzing for main tweets...", tweets.len());
                            }
                            
                            // Find the first tweet that is NOT a reply (conversation_id == id)
                            let main_tweet = tweets.iter().find(|tweet| {
                                if let Some(tweet_obj) = tweet.as_object() {
                                    let tweet_id = tweet_obj.get("id").and_then(|v| v.as_str());
                                    let conversation_id = tweet_obj.get("conversation_id").and_then(|v| v.as_str());
                                    let text = tweet_obj.get("text").and_then(|v| v.as_str()).unwrap_or("No text");
                                    
                                    if let (Some(id), Some(conv_id)) = (tweet_id, conversation_id) {
                                        let is_main_tweet = id == conv_id;
                                        if args.verbose {
                                            let tweet_type = if is_main_tweet { "MAIN TWEET" } else { "REPLY" };
                                            println!("  {} - ID: {}, Conv: {}, Text: {:.50}...", 
                                                tweet_type, id, conv_id, text);
                                        }
                                        is_main_tweet  // Main tweet: conversation_id equals tweet id
                                    } else {
                                        if args.verbose {
                                            println!("  INVALID - Missing ID or conversation_id");
                                        }
                                        false
                                    }
                                } else {
                                    false
                                }
                            });
                            
                            if let Some(tweet) = main_tweet {
                                if let Some(tweet_obj) = tweet.as_object() {
                                    let tweet_id = tweet_obj.get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown");
                                    let text = tweet_obj.get("text")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("No text");
                                    let author_id = tweet_obj.get("author_id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown");
                                    let created_at = tweet_obj.get("created_at")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown");
                                    
                                    // Get public metrics if available
                                    let mut metrics_str = String::new();
                                    if let Some(metrics) = tweet_obj.get("public_metrics") {
                                        if let Some(metrics_obj) = metrics.as_object() {
                                            let retweets = metrics_obj.get("retweet_count")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(0);
                                            let likes = metrics_obj.get("like_count")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(0);
                                            let replies = metrics_obj.get("reply_count")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(0);
                                            let quotes = metrics_obj.get("quote_count")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(0);
                                            
                                            metrics_str = format!("📊 {} likes, {} retweets, {} replies, {} quotes", 
                                                likes, retweets, replies, quotes);
                                        }
                                    }
                                    
                                    // Get user info if available
                                    let mut user_info = format!("@{}", args.username);
                                    if let Some(includes) = response_data.get("includes") {
                                        if let Some(users) = includes.get("users") {
                                            if let Some(users_array) = users.as_array() {
                                                if let Some(user) = users_array.first() {
                                                    if let Some(user_obj) = user.as_object() {
                                                        let name = user_obj.get("name")
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("");
                                                        let verified = user_obj.get("verified")
                                                            .and_then(|v| v.as_bool())
                                                            .unwrap_or(false);
                                                        
                                                        user_info = format!("{} ({}){}", 
                                                            name, 
                                                            args.username,
                                                            if verified { " ✓" } else { "" }
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    println!("\n🐦 Latest tweet from {}", user_info);
                                    println!("📅 Created: {}", created_at);
                                    println!("💬 Text: {}", text);
                                    if !metrics_str.is_empty() {
                                        println!("{}", metrics_str);
                                    }
                                    println!("🔗 URL: https://twitter.com/i/status/{}", tweet_id);
                                    
                                    println!("\n✅ Latest tweet fetched successfully!");
                                } else {
                                    println!("❌ Failed to parse tweet data");
                                }
                            } else {
                                println!("❌ No main tweets found for user @{}", args.username);
                                println!("💡 This could mean:");
                                println!("   - The user has only posted replies, not original tweets");
                                println!("   - All recent tweets are retweets or replies");
                                println!("   - Try without --exclude-retweets-replies to see all tweets");
                            }
                        } else {
                            println!("❌ Invalid data format in response");
                        }
                    } else {
                        // Check if it's a user not found error
                        if let Some(errors) = response_data.get("errors") {
                            if let Some(errors_array) = errors.as_array() {
                                if let Some(error) = errors_array.first() {
                                    if let Some(error_obj) = error.as_object() {
                                        let title = error_obj.get("title")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("Unknown error");
                                        let detail = error_obj.get("detail")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("No details");
                                        
                                        println!("❌ Error: {} - {}", title, detail);
                                        if title.contains("Not Found") {
                                            println!("💡 Make sure the username '{}' exists and is spelled correctly", args.username);
                                        }
                                    }
                                }
                            }
                        } else {
                            println!("❌ No tweets found for user @{}", args.username);
                            println!("💡 This could mean:");
                            println!("   - The user has no tweets");
                            println!("   - The user's tweets are protected");
                            println!("   - All tweets are retweets/replies (if --exclude-retweets-replies was used)");
                        }
                    }
                } else {
                    println!("❌ Failed to parse response JSON");
                }
            } else {
                let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                println!("❌ Twitter API error: {} - {}", status, error_text);
            }
        }
        Err(e) => {
            println!("❌ HTTP request failed: {}", e);
        }
    }
}

/// Create OAuth 1.0a authorization header for Twitter API with query parameters
fn create_oauth_header_with_params(
    method: &str,
    base_url: &str,
    query_params: Option<&str>,
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
    
    // Combine OAuth parameters with query parameters for signature
    let mut all_params: HashMap<String, String> = HashMap::new();
    
    // Add OAuth parameters
    for (k, v) in oauth_params.iter() {
        all_params.insert(k.to_string(), v.to_string());
    }
    
    // Parse query parameters if they exist
    if let Some(query_str) = query_params {
        for param in query_str.split('&') {
            if let Some(pos) = param.find('=') {
                let key = &param[..pos];
                let value = &param[pos+1..];
                // URL decode the parameters for signature
                let decoded_key = urlencoding::decode(key).unwrap_or_else(|_| key.into());
                let decoded_value = urlencoding::decode(value).unwrap_or_else(|_| value.into());
                all_params.insert(decoded_key.into_owned(), decoded_value.into_owned());
            }
        }
    }
    
    // Create parameter string for signature (all parameters must be included)
    let mut sorted_params: Vec<_> = all_params.iter().collect();
    sorted_params.sort_by_key(|(k, _)| k.as_str());
    
    let param_string = sorted_params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    
    // Create signature base string
    let base_string = format!(
        "{}&{}&{}",
        method.to_uppercase(),
        urlencoding::encode(base_url),
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
    
    // Add signature to OAuth parameters (only OAuth params go in the header)
    oauth_params.insert("oauth_signature", &signature);
    
    // Create authorization header (only OAuth parameters)
    let auth_params: Vec<String> = oauth_params
        .iter()
        .filter(|(k, _)| k.starts_with("oauth_"))
        .map(|(k, v)| format!("{}=\"{}\"", urlencoding::encode(k), urlencoding::encode(v)))
        .collect();
    
    format!("OAuth {}", auth_params.join(", "))
} 