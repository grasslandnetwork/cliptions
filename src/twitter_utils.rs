use twitter_api::{TwitterClient, TwitterError, PostTweetResult, TwitterApi};

/// Post a tweet or reply using the provided TwitterClient.
/// If reply_to is Some, posts a reply. Otherwise, posts a normal tweet.
pub async fn post_tweet_or_reply(
    client: &TwitterClient,
    tweet_text: &str,
    reply_to: Option<&str>,
) -> Result<PostTweetResult, TwitterError> {
    if std::env::var("CLIPTIONS_DEBUG").is_ok() {
        match reply_to {
            Some(reply_id) => {
                println!("[twitter_utils] Posting reply to {}: {}", reply_id, tweet_text);
            }
            None => {
                println!("[twitter_utils] Posting tweet: {}", tweet_text);
            }
        }
    }
    match reply_to {
        Some(reply_id) => client.reply_to_tweet(tweet_text, reply_id).await,
        None => client.post_tweet(tweet_text).await,
    }
} 