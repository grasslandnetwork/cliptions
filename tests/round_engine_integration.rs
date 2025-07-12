//! Integration tests for the round engine state machine.
//!
//! These tests verify the full lifecycle of a round, mocking external
//! services like the Twitter API to ensure the state machine orchestrates
//! actions correctly.

use cliptions_core::round_engine::state_machine::*;
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
        .returning(|text, _: std::path::PathBuf| {
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


    // Create timestamps for the test
    let now = Utc::now();
    let commitment_deadline = now + Duration::hours(24);
    let fee_deadline = now + Duration::hours(48);
    let reveals_deadline = now + Duration::hours(72);
    let target_frame_path = "test_images/target_frame.jpg".to_string();

    // Start with a pending round
    let pending_round = Round::<Pending>::new("integration-test-round".to_string());
    assert_eq!(pending_round.state_name(), "Pending");

    // 1. Pending -> CommitmentsOpen
    let commitments_open_round = pending_round
        .open_commitments(commitment_deadline, &mock_twitter_client)
        .await
        .expect("Failed to transition to CommitmentsOpen");
    
    assert_eq!(commitments_open_round.state_name(), "CommitmentsOpen");
    assert_eq!(commitments_open_round.commitment_deadline, Some(commitment_deadline));
    assert!(commitments_open_round.target_frame_path.is_none());

    // 2. CommitmentsOpen -> FeeCollectionOpen
    let fee_collection_open_round = commitments_open_round
        .close_commitments(fee_deadline, &mock_twitter_client)
        .await
        .expect("Failed to transition to FeeCollectionOpen");
    
    assert_eq!(fee_collection_open_round.state_name(), "FeeCollectionOpen");
    assert_eq!(fee_collection_open_round.fee_deadline, Some(fee_deadline));

    // 3. FeeCollectionOpen -> RevealsOpen
    let reveals_open_round = fee_collection_open_round
        .close_fee_collection(target_frame_path.clone(), reveals_deadline, &mock_twitter_client)
        .await
        .expect("Failed to transition to RevealsOpen");
    
    assert_eq!(reveals_open_round.state_name(), "RevealsOpen");
    assert_eq!(reveals_open_round.reveals_deadline, Some(reveals_deadline));
    assert_eq!(reveals_open_round.target_frame_path, Some(target_frame_path));

    // 4. RevealsOpen -> Payouts
    let payouts_round = reveals_open_round
        .close_reveals(&mock_twitter_client)
        .await
        .expect("Failed to transition to Payouts");
    
    assert_eq!(payouts_round.state_name(), "Payouts");

    // 5. Payouts -> Finished
    let finished_round = payouts_round
        .process_payouts(&mock_twitter_client)
        .await
        .expect("Failed to transition to Finished");
    
    assert_eq!(finished_round.state_name(), "Finished");
    assert!(finished_round.is_complete());

    // Verify all mock expectations were met
    // The mockall framework will automatically verify that all expected calls were made
}

#[tokio::test]
async fn test_round_lifecycle_with_machine_readable_tweets() {
    let mut mock_twitter_client = MockTwitterApiClient::new();

    // Set up expectations for machine-readable tweet format
    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| {
            text.contains("#cliptions") &&
            text.contains("#round") &&
            text.contains("#state_commitmentsopen") &&
            text.contains("#CLIP") &&
            !text.contains("#predictionmarket")
        })
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("2001", text);
            Ok(PostTweetResult { tweet, success: true })
        });

    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| {
            text.contains("#state_feecollectionopen")
        })
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("2002", text);
            Ok(PostTweetResult { tweet, success: true })
        });

    mock_twitter_client
        .expect_post_tweet_with_image()
        .withf(|text, _| {
            text.contains("#state_revealsopen")
        })
        .times(1)
        .returning(|text, _: std::path::PathBuf| {
            let tweet = create_mock_tweet("2003", text);
            Ok(PostTweetResult { tweet, success: true })
        });

    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| {
            text.contains("#state_payouts")
        })
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("2004", text);
            Ok(PostTweetResult { tweet, success: true })
        });

    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| {
            text.contains("#state_finished")
        })
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("2005", text);
            Ok(PostTweetResult { tweet, success: true })
        });

    // Run through the lifecycle focusing on tweet content verification
    let now = Utc::now();
    let pending_round = Round::<Pending>::new("42".to_string()); // Use numeric string for round ID

    let commitments_round = pending_round
        .open_commitments(now + Duration::hours(24), &mock_twitter_client)
        .await
        .expect("Failed to open commitments");

    let fee_round = commitments_round
        .close_commitments(now + Duration::hours(48), &mock_twitter_client)
        .await
        .expect("Failed to close commitments");

    let reveals_round = fee_round
        .close_fee_collection("test_frame.jpg".to_string(), now + Duration::hours(72), &mock_twitter_client)
        .await
        .expect("Failed to close fee collection");

    let payouts_round = reveals_round
        .close_reveals(&mock_twitter_client)
        .await
        .expect("Failed to close reveals");

    let _finished_round = payouts_round
        .process_payouts(&mock_twitter_client)
        .await
        .expect("Failed to process payouts");

    // All expectations verified automatically by mockall
} 