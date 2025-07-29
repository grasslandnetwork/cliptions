//! Integration tests for the block engine state machine.
//!
//! These tests verify the full lifecycle of a block, mocking external
//! services like the Twitter API to ensure the state machine orchestrates
//! actions correctly.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use cliptions_core::block_engine::state_machine::*;
use mockall::mock;
use std::path::Path;
use twitter_api::{PostTweetResult, Tweet, TwitterApi, TwitterError};

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
        async fn reply_to_tweet_with_image<P: AsRef<Path> + Send + 'static>(
            &self,
            text: &str,
            reply_to_tweet_id: &str,
            image_path: P,
        ) -> Result<PostTweetResult, TwitterError>;
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
async fn test_full_block_lifecycle_with_mocks() {
    let mut mock_twitter_client = MockTwitterApiClient::new();

    // 1. Pending -> CommitmentsOpen
    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| text.contains("New Block Started!"))
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("1001", text);
            Ok(PostTweetResult {
                tweet,
                success: true,
            })
        });

    // 2. CommitmentsOpen -> FeeCollectionOpen
    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| text.contains("Commitments are now closed"))
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("1002", text);
            Ok(PostTweetResult {
                tweet,
                success: true,
            })
        });

    // 3. FeeCollectionOpen -> RevealsOpen
    mock_twitter_client
        .expect_reply_to_tweet_with_image()
        .withf(|text, _, _| text.contains("Target frame revealed!"))
        .times(1)
        .returning(|text, _, _: std::path::PathBuf| {
            let tweet = create_mock_tweet("1003", text);
            Ok(PostTweetResult {
                tweet,
                success: true,
            })
        });

    // 4. RevealsOpen -> Payouts
    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| text.contains("Reveals are now closed"))
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("1004", text);
            Ok(PostTweetResult {
                tweet,
                success: true,
            })
        });

    // 5. Payouts -> Finished
    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| text.contains("Block finished!"))
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("1005", text);
            Ok(PostTweetResult {
                tweet,
                success: true,
            })
        });

    // Create timestamps for the test
    let now = Utc::now();
    let commitment_deadline = now + Duration::hours(24);
    let reveals_deadline = now + Duration::hours(72);
    let target_frame_path: std::path::PathBuf = "test_images/target_frame.jpg".into();

    // Start with a pending block
    let now = Utc::now();
    let pending_block = Block::<Pending>::new(
        "integration-test-block".to_string(),
        "Integration Test Block".to_string(),
        "http://example.com/livestream".to_string(),
        now + Duration::hours(1),
    );
    assert_eq!(pending_block.state_name(), "Pending");

    // 1. Pending -> CommitmentsOpen
    let commitments_open_block = pending_block
        .open_commitments(commitment_deadline, &mock_twitter_client)
        .await
        .expect("Failed to transition to CommitmentsOpen");

    assert_eq!(commitments_open_block.state_name(), "CommitmentsOpen");
    assert_eq!(
        commitments_open_block.commitment_deadline,
        Some(commitment_deadline)
    );
    assert!(commitments_open_block.target_frame_path.is_none());

    // 2. CommitmentsOpen -> FeeCollectionOpen
    let commitments_closed_block = commitments_open_block
        .close_commitments(&mock_twitter_client)
        .await
        .expect("Failed to transition to CommitmentsClosed");

    // 3. CommitmentsClosed -> FrameCaptured -> RevealsOpen
    // Simulate time passing for the check inside capture_frame
    let mut commitments_closed_block = commitments_closed_block;
    commitments_closed_block.target_timestamp = Utc::now() - Duration::seconds(1);
    let frame_captured_block = commitments_closed_block
        .capture_frame(target_frame_path.clone())
        .unwrap();
    assert_eq!(frame_captured_block.state_name(), "FrameCaptured");
    assert_eq!(
        frame_captured_block.target_frame_path.clone().unwrap(),
        target_frame_path.clone()
    );
    let reveals_open_block = frame_captured_block
        .open_reveals(reveals_deadline, &mock_twitter_client, "parent_tweet_id")
        .await
        .unwrap();
    assert_eq!(reveals_open_block.state_name(), "RevealsOpen");
    assert_eq!(reveals_open_block.reveals_deadline, Some(reveals_deadline));
    assert_eq!(
        reveals_open_block.target_frame_path,
        Some(target_frame_path.clone())
    );

    // 4. RevealsOpen -> Payouts
    let payouts_block = reveals_open_block
        .close_reveals(&mock_twitter_client)
        .await
        .expect("Failed to transition to Payouts");

    assert_eq!(payouts_block.state_name(), "Payouts");

    // 5. Payouts -> Finished
    let finished_block = payouts_block
        .process_payouts(&mock_twitter_client)
        .await
        .expect("Failed to transition to Finished");

    assert_eq!(finished_block.state_name(), "Finished");
    assert!(finished_block.is_complete());

    // Verify all mock expectations were met
    // The mockall framework will automatically verify that all expected calls were made
}

#[tokio::test]
async fn test_block_lifecycle_with_machine_readable_tweets() {
    let mut mock_twitter_client = MockTwitterApiClient::new();

    // Set up expectations for machine-readable tweet format
    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| {
            text.contains("#cliptions")
                && text.contains("#block")
                && text.contains("#commitmentsopen")
                && text.contains("#CLIP")
                && !text.contains("#predictionmarket")
        })
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("2001", text);
            Ok(PostTweetResult {
                tweet,
                success: true,
            })
        });

    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| text.contains("#feecollectionopen"))
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("2002", text);
            Ok(PostTweetResult {
                tweet,
                success: true,
            })
        });

    mock_twitter_client
        .expect_reply_to_tweet_with_image()
        .withf(|text, _, _| text.contains("#revealsopen"))
        .times(1)
        .returning(|text, _, _: std::path::PathBuf| {
            let tweet = create_mock_tweet("2003", text);
            Ok(PostTweetResult {
                tweet,
                success: true,
            })
        });

    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| text.contains("#payouts"))
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("2004", text);
            Ok(PostTweetResult {
                tweet,
                success: true,
            })
        });

    mock_twitter_client
        .expect_post_tweet()
        .withf(|text| text.contains("#finished"))
        .times(1)
        .returning(|text| {
            let tweet = create_mock_tweet("2005", text);
            Ok(PostTweetResult {
                tweet,
                success: true,
            })
        });

    // Run through the lifecycle focusing on tweet content verification
    let now = Utc::now();
    let pending_block = Block::<Pending>::new(
        "42".to_string(),
        "Test Block 42".to_string(),
        "http://example.com/livestream".to_string(),
        now + Duration::hours(1),
    ); // Use numeric string for block ID

    let commitments_block = pending_block
        .open_commitments(now + Duration::hours(24), &mock_twitter_client)
        .await
        .expect("Failed to open commitments");

    let commitments_closed_block = commitments_block
        .close_commitments(&mock_twitter_client)
        .await
        .expect("Failed to close commitments");

    // 3. CommitmentsClosed -> FrameCaptured -> RevealsOpen
    let mut commitments_closed_block = commitments_closed_block;
    commitments_closed_block.target_timestamp = now - Duration::seconds(1);
    let test_frame_path: std::path::PathBuf = "test_frame.jpg".into();
    let frame_captured_block = commitments_closed_block
        .capture_frame(test_frame_path.clone())
        .unwrap();
    let reveals_block = frame_captured_block
        .open_reveals(
            now + Duration::hours(72),
            &mock_twitter_client,
            "parent_tweet_id",
        )
        .await
        .unwrap();

    let payouts_block = reveals_block
        .close_reveals(&mock_twitter_client)
        .await
        .expect("Failed to close reveals");

    let _finished_block = payouts_block
        .process_payouts(&mock_twitter_client)
        .await
        .expect("Failed to process payouts");

    // All expectations verified automatically by mockall
}
