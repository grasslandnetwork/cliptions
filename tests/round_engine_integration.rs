//! Integration tests for the round engine state machine.
//!
//! These tests verify the full lifecycle of a round, mocking external
//! services like the Twitter API to ensure the state machine orchestrates
//! actions correctly.

use cliptions_core::round_engine::state_machine::*;
use cliptions_core::error::Result;
use twitter_api::{PostTweetResult, Tweet, TwitterApi, TwitterError};
use chrono::{Duration, Utc};
use std::path::Path;
use async_trait::async_trait;
use mockall::mock;

mock! {
    pub TwitterApiClient {
        // This mirrors the TwitterApi trait
    }

    #[async_trait]
    impl TwitterApi for TwitterApiClient {
        async fn post_tweet(&self, text: &str) -> Result<PostTweetResult, TwitterError>;
        async fn post_tweet_with_image<P: AsRef<Path> + Send + 'static>(
            &self,
            text: &str,
            image_path: P,
        ) -> Result<PostTweetResult, TwitterError>;
        async fn reply_to_tweet(&self, text: &str, reply_to_tweet_id: &str) -> Result<PostTweetResult, TwitterError>;
        async fn get_latest_tweet(
            &self,
            username: &str,
            exclude_retweets_replies: bool,
        ) -> Result<Option<Tweet>, TwitterError>;
        async fn search_replies(&self, tweet_id: &str, max_results: u32) -> Result<Vec<Tweet>, TwitterError>;
    }
}

fn create_mock_tweet(id: &str, text: &str) -> Tweet {
    Tweet {
        id: id.to_string(),
        text: text.to_string(),
        author_id: "test_user".to_string(),
        created_at: Some(Utc::now()),
        conversation_id: None,
        public_metrics: None,
        url: format!("https://twitter.com/i/status/{}", id),
    }
}

#[tokio::test]
async fn test_full_round_lifecycle_with_mocks() {
    let mut mock_twitter_client = MockTwitterApiClient::new();

    // 1. Pending -> CommitmentsOpen
    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| text.contains("New Round Started!"))
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("1001", text);
            Ok(PostTweetResult { tweet, success: true })
        });

    // 2. CommitmentsOpen -> FeeCollectionOpen
    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| text.contains("Commitments are now closed"))
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("1002", text);
            Ok(PostTweetResult { tweet, success: true })
        });

    // 3. FeeCollectionOpen -> RevealsOpen
    mock_twitter_client
        .expect_post_tweet_with_image()
        .withf(|text, _| text.contains("Time to reveal your commitments!"))
        .times(1)
        .returning(|text, _| {
            let tweet = create_mock_tweet("1003", text);
            Ok(PostTweetResult { tweet, success: true })
        });

    // 4. RevealsOpen -> Payouts
    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| text.contains("Reveals are now closed"))
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("1004", text);
            Ok(PostTweetResult { tweet, success: true })
        });
    
    // 5. Payouts -> Finished
    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| text.contains("Round finished!"))
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("1005", text);
            Ok(PostTweetResult { tweet, success: true })
        });


    // Now, we need to modify the state machine to accept the trait
    // For now, this test file will not compile until we do that.
    // I will modify the state machine in the next step.
} 