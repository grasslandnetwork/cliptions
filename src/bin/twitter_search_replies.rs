//! Twitter API reply search tool
//! 
//! Searches for all replies to a specific tweet using Twitter API v2

use std::env;
use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "twitter_search_replies")]
#[command(about = "Search for replies to a specific tweet")]
struct Args {
    /// Tweet ID to search replies for
    #[arg(short, long)]
    tweet_id: String,
    
    /// Maximum results per page (default: 100)
    #[arg(short, long, default_value = "100")]
    max_results: u32,
    
    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    if args.verbose {
        println!("Starting Twitter API reply search...");
        println!("Searching for replies to tweet: {}", args.tweet_id);
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
    
    // Search for replies using conversation_id
    let mut all_replies = Vec::new();
    let mut next_token: Option<String> = None;
    let mut page_count = 0;
    
    loop {
        page_count += 1;
        if args.verbose {
            println!("\n--- Page {} ---", page_count);
        }
        
        // Build query string
        let query = format!("conversation_id:{} is:reply", args.tweet_id);
        
        // Build URL with query parameters
        let mut url = format!(
            "https://api.twitter.com/2/tweets/search/recent?query={}&max_results={}&tweet.fields=created_at,author_id,conversation_id,in_reply_to_user_id,referenced_tweets&user.fields=username,name&expansions=author_id",
            urlencoding::encode(&query),
            args.max_results
        );
        
        // Add pagination token if we have one
        if let Some(token) = &next_token {
            url.push_str(&format!("&pagination_token={}", token));
        }
        
        if args.verbose {
            println!("Request URL: {}", url);
        }
        
        // Create OAuth 1.0a authorization header
        // For GET requests with query parameters, we need to separate base URL from query params
        let (base_url, query_params) = if let Some(pos) = url.find('?') {
            let base = &url[..pos];
            let params = &url[pos+1..];
            (base, Some(params))
        } else {
            (url.as_str(), None)
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
        
        // Make the API request
        let response = client
            .get(&url)
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
                        // Get tweets from this page
                        if let Some(data) = response_data.get("data") {
                            if let Some(tweets) = data.as_array() {
                                println!("Found {} replies on page {}", tweets.len(), page_count);
                                
                                // Process each tweet
                                for tweet in tweets {
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
                                        
                                        println!("\n🐦 Tweet ID: {}", tweet_id);
                                        println!("👤 Author ID: {}", author_id);
                                        println!("📅 Created: {}", created_at);
                                        println!("💬 Text: {}", text);
                                        println!("🔗 URL: https://twitter.com/i/status/{}", tweet_id);
                                        
                                        all_replies.push(tweet.clone());
                                    }
                                }
                            }
                        } else {
                            println!("No replies found on page {}", page_count);
                        }
                        
                        // Check for next page
                        if let Some(meta) = response_data.get("meta") {
                            if let Some(token) = meta.get("next_token") {
                                if let Some(token_str) = token.as_str() {
                                    next_token = Some(token_str.to_string());
                                    if args.verbose {
                                        println!("Next token: {}", token_str);
                                    }
                                } else {
                                    break; // No more pages
                                }
                            } else {
                                break; // No more pages
                            }
                        } else {
                            break; // No meta object
                        }
                    } else {
                        println!("❌ Failed to parse response JSON");
                        break;
                    }
                } else {
                    let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    println!("❌ Twitter API error: {} - {}", status, error_text);
                    break;
                }
            }
            Err(e) => {
                println!("❌ HTTP request failed: {}", e);
                break;
            }
        }
    }
    
    println!("\n✅ Search complete!");
    println!("Total replies found: {}", all_replies.len());
    println!("Pages searched: {}", page_count);
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