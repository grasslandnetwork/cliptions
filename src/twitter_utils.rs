use std::path::PathBuf;
use twitter_api::{PostTweetResult, TwitterApi, TwitterClient, TwitterError};

/// Post a tweet, reply, or tweet with image using the provided TwitterClient.
/// If both reply_to and image_path are Some, prints a warning and posts image tweet only.
pub async fn post_tweet_flexible(
    client: &TwitterClient,
    tweet_text: &str,
    reply_to: Option<&str>,
    image_path: Option<PathBuf>,
) -> Result<PostTweetResult, TwitterError> {
    if std::env::var("CLIPTIONS_DEBUG").is_ok() {
        println!("[twitter_utils] post_tweet_flexible called:");
        println!("  tweet_text: {}", tweet_text);
        println!("  reply_to: {:?}", reply_to);
        println!("  image_path: {:?}", image_path);
    }
    match (reply_to, image_path) {
        (Some(reply_id), Some(path)) => {
            client
                .reply_to_tweet_with_image(tweet_text, reply_id, path)
                .await
        }
        (None, Some(path)) => client.post_tweet_with_image(tweet_text, path).await,
        (Some(reply_id), None) => client.reply_to_tweet(tweet_text, reply_id).await,
        (None, None) => client.post_tweet(tweet_text).await,
    }
}
