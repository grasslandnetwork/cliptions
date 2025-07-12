//! Async State Machine for Cliptions Round Engine
//! 
//! This module implements the round lifecycle using the Rust typestate pattern.
//! Each state is a marker type that ensures operations can only be performed
//! when the round is in the correct state, enforced by the compiler.

use std::fmt;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::{CliptionsError, Result};
use crate::social::{AnnouncementFormatter, AnnouncementData};
use twitter_api::TwitterApi;

/// State marker for a round that hasn't started yet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pending;

/// State marker for a round accepting commitments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitmentsOpen;

/// State marker for a round with closed commitments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitmentsClosed;

/// State marker for a round with open fee collection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeCollectionOpen;

/// State marker for a round with closed fee collection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeCollectionClosed;

/// State marker for a round accepting reveals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevealsOpen;

/// State marker for a round with closed reveals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevealsClosed;

/// State marker for a round processing payouts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Payouts;

/// State marker for a finished round
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Finished;

/// Trait for state markers to provide display names
pub trait StateMarker {
    fn state_name() -> &'static str;
}

impl StateMarker for Pending {
    fn state_name() -> &'static str { "Pending" }
}

impl StateMarker for CommitmentsOpen {
    fn state_name() -> &'static str { "CommitmentsOpen" }
}

impl StateMarker for CommitmentsClosed {
    fn state_name() -> &'static str { "CommitmentsClosed" }
}

impl StateMarker for FeeCollectionOpen {
    fn state_name() -> &'static str { "FeeCollectionOpen" }
}

impl StateMarker for FeeCollectionClosed {
    fn state_name() -> &'static str { "FeeCollectionClosed" }
}

impl StateMarker for RevealsOpen {
    fn state_name() -> &'static str { "RevealsOpen" }
}

impl StateMarker for RevealsClosed {
    fn state_name() -> &'static str { "RevealsClosed" }
}

impl StateMarker for Payouts {
    fn state_name() -> &'static str { "Payouts" }
}

impl StateMarker for Finished {
    fn state_name() -> &'static str { "Finished" }
}

/// Round data structure with typestate pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round<S> {
    /// Round identifier
    pub id: String,
    /// Round creation timestamp
    pub created_at: DateTime<Utc>,
    /// Target frame path (only available during reveals phase and after)
    pub target_frame_path: Option<String>,
    /// Commitment deadline
    pub commitment_deadline: Option<DateTime<Utc>>,
    /// Fee collection deadline
    pub fee_deadline: Option<DateTime<Utc>>,
    /// Reveals deadline
    pub reveals_deadline: Option<DateTime<Utc>>,
    /// State marker (not serialized)
    #[serde(skip)]
    pub state: std::marker::PhantomData<S>,
}

impl<S> Round<S> {
    /// Get the current state name
    pub fn state_name(&self) -> &'static str
    where
        S: StateMarker,
    {
        S::state_name()
    }
}

impl<S> fmt::Display for Round<S>
where
    S: StateMarker,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Round {} ({})",
            self.id,
            S::state_name()
        )
    }
}

/// Implementation for Pending state
impl Round<Pending> {
    /// Create a new pending round
    pub fn new(id: String) -> Self {
        Self {
            id,
            created_at: Utc::now(),
            target_frame_path: None,
            commitment_deadline: None,
            fee_deadline: None,
            reveals_deadline: None,
            state: std::marker::PhantomData,
        }
    }

    /// Start the round by opening commitments
    pub async fn open_commitments<T: TwitterApi>(
        mut self,
        commitment_deadline: DateTime<Utc>,
        client: &T,
    ) -> Result<Round<CommitmentsOpen>> {
        let formatter = AnnouncementFormatter::new();
        let announcement_data = AnnouncementData {
            round_id: self.id.clone(),
            target_time: commitment_deadline.to_rfc3339(),
            hashtags: vec![],
            message: format!(
                "Commitments are open until {}.",
                commitment_deadline.format("%Y-%m-%d %H:%M:%S UTC")
            ),
            prize_pool: None,
        };
        let tweet_text = formatter.format_announcement(&announcement_data, true);

        client
            .post_tweet(&tweet_text)
            .await
            .map_err(|e| CliptionsError::ApiError(e.to_string()))?;
        
        self.commitment_deadline = Some(commitment_deadline);
        
        Ok(Round {
            id: self.id,
            created_at: self.created_at,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            fee_deadline: self.fee_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        })
    }
}

/// Implementation for CommitmentsOpen state
impl Round<CommitmentsOpen> {
    /// Close commitments and open fee collection
    pub async fn close_commitments<T: TwitterApi>(
        mut self,
        fee_deadline: DateTime<Utc>,
        client: &T,
    ) -> Result<Round<FeeCollectionOpen>> {
        let formatter = AnnouncementFormatter::new();
        let announcement_data = AnnouncementData {
            round_id: self.id.clone(),
            target_time: fee_deadline.to_rfc3339(),
            hashtags: vec![],
            message: format!(
                "Commitments are now closed. Fee collection is open until {}.",
                fee_deadline.format("%Y-%m-%d %H:%M:%S UTC")
            ),
            prize_pool: None,
        };
        let tweet_text = formatter.format_announcement(&announcement_data, true);
        
        client
            .post_tweet(&tweet_text)
            .await
            .map_err(|e| CliptionsError::ApiError(e.to_string()))?;
        
        self.fee_deadline = Some(fee_deadline);
        
        Ok(Round {
            id: self.id,
            created_at: self.created_at,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            fee_deadline: self.fee_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        })
    }
}

/// Implementation for FeeCollectionOpen state
impl Round<FeeCollectionOpen> {
    /// Close fee collection and open reveals
    pub async fn close_fee_collection<T: TwitterApi>(
        mut self,
        target_frame_path: String,
        reveals_deadline: DateTime<Utc>,
        client: &T,
    ) -> Result<Round<RevealsOpen>> {
        let formatter = AnnouncementFormatter::new();
        let announcement_data = AnnouncementData {
            round_id: self.id.clone(),
            target_time: reveals_deadline.to_rfc3339(),
            hashtags: vec![],
            message: format!(
                "Time to reveal your commitments! Reveals are open until {}.",
                reveals_deadline.format("%Y-%m-%d %H:%M:%S UTC")
            ),
            prize_pool: None,
        };
        let tweet_text = formatter.format_announcement(&announcement_data, true);

        client
            .post_tweet_with_image(&tweet_text, target_frame_path.clone())
            .await
            .map_err(|e| CliptionsError::ApiError(e.to_string()))?;
        
        self.target_frame_path = Some(target_frame_path);
        self.reveals_deadline = Some(reveals_deadline);
        
        Ok(Round {
            id: self.id,
            created_at: self.created_at,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            fee_deadline: self.fee_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        })
    }
}

/// Implementation for RevealsOpen state
impl Round<RevealsOpen> {
    /// Close reveals and start payout processing
    pub async fn close_reveals<T: TwitterApi>(self, client: &T) -> Result<Round<Payouts>> {
        let formatter = AnnouncementFormatter::new();
        let announcement_data = AnnouncementData {
            round_id: self.id.clone(),
            target_time: "".to_string(),
            hashtags: vec![],
            message: "Reveals are now closed. Calculating scores...".to_string(),
            prize_pool: None,
        };
        let tweet_text = formatter.format_announcement(&announcement_data, true);

        client
            .post_tweet(&tweet_text)
            .await
            .map_err(|e| CliptionsError::ApiError(e.to_string()))?;
        
        Ok(Round {
            id: self.id,
            created_at: self.created_at,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            fee_deadline: self.fee_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        })
    }
}

/// Implementation for Payouts state
impl Round<Payouts> {
    /// Process payouts and finish the round
    pub async fn process_payouts<T: TwitterApi>(self, client: &T) -> Result<Round<Finished>> {
        let formatter = AnnouncementFormatter::new();
        let announcement_data = AnnouncementData {
            round_id: self.id.clone(),
            target_time: "".to_string(),
            hashtags: vec![],
            message: "Round finished! Payouts have been processed.".to_string(),
            prize_pool: None,
        };
        let tweet_text = formatter.format_announcement(&announcement_data, true);

        client
            .post_tweet(&tweet_text)
            .await
            .map_err(|e| CliptionsError::ApiError(e.to_string()))?;
        
        Ok(Round {
            id: self.id,
            created_at: self.created_at,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            fee_deadline: self.fee_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        })
    }
}

/// Implementation for Finished state
impl Round<Finished> {
    /// Check if the round is complete
    pub fn is_complete(&self) -> bool {
        true
    }
}

/// Utility functions for state transitions
impl<S> Round<S> {
    /// Convert to any other state (used for deserialization)
    pub fn into_state<T>(self) -> Round<T> {
        Round {
            id: self.id,
            created_at: self.created_at,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            fee_deadline: self.fee_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        }
    }
}

/// Helper function to parse state from string (for tweet parsing)
pub fn parse_state_from_string(state_str: &str) -> Option<String> {
    match state_str.to_lowercase().as_str() {
        "pending" => Some("Pending".to_string()),
        "commitmentsopen" => Some("CommitmentsOpen".to_string()),
        "commitmentsclosed" => Some("CommitmentsClosed".to_string()),
        "feecollectionopen" => Some("FeeCollectionOpen".to_string()),
        "feecollectionclosed" => Some("FeeCollectionClosed".to_string()),
        "revealsopen" => Some("RevealsOpen".to_string()),
        "revealsclosed" => Some("RevealsClosed".to_string()),
        "payouts" => Some("Payouts".to_string()),
        "finished" => Some("Finished".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use twitter_api::{PostTweetResult, Tweet, TwitterApi, TwitterError};
    use std::path::Path;
    use async_trait::async_trait;

    /// A dummy Twitter client that does nothing, for unit tests.
    struct DummyTwitterClient;

    #[async_trait]
    impl TwitterApi for DummyTwitterClient {
        async fn post_tweet(&self, text: &str) -> twitter_api::Result<PostTweetResult> {
            let tweet = Tweet {
                id: "dummy_id".to_string(),
                text: text.to_string(),
                author_id: "dummy_author".to_string(),
                created_at: None,
                conversation_id: None,
                public_metrics: None,
                url: "".to_string(),
            };
            Ok(PostTweetResult { tweet, success: true })
        }
        async fn post_tweet_with_image<P: AsRef<Path> + Send + 'static>(
            &self,
            text: &str,
            _image_path: P,
        ) -> twitter_api::Result<PostTweetResult> {
            let tweet = Tweet {
                id: "dummy_id".to_string(),
                text: text.to_string(),
                author_id: "dummy_author".to_string(),
                created_at: None,
                conversation_id: None,
                public_metrics: None,
                url: "".to_string(),
            };
            Ok(PostTweetResult { tweet, success: true })
        }
        async fn reply_to_tweet(&self, text: &str, _reply_to_tweet_id: &str) -> twitter_api::Result<PostTweetResult> {
            let tweet = Tweet {
                id: "dummy_id".to_string(),
                text: text.to_string(),
                author_id: "dummy_author".to_string(),
                created_at: None,
                conversation_id: None,
                public_metrics: None,
                url: "".to_string(),
            };
            Ok(PostTweetResult { tweet, success: true })
        }
        async fn get_latest_tweet(
            &self,
            _username: &str,
            _exclude_retweets_replies: bool,
        ) -> twitter_api::Result<Option<Tweet>> {
            Ok(None)
        }
        async fn search_replies(&self, _tweet_id: &str, _max_results: u32) -> twitter_api::Result<Vec<Tweet>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_pending_to_commitments_open() {
        let round = Round::<Pending>::new("test-round".to_string());
        let deadline = Utc::now() + Duration::days(1);
        
        let mut client = DummyTwitterClient;
        // We need to return a successful result for the test to pass
        // but since we are unit testing the state machine, we can just ignore the client
        // and its result. The integration test will verify the client interaction.
        let next_round = round.open_commitments(deadline, &client).await.unwrap();

        assert_eq!(next_round.state_name(), "CommitmentsOpen");
        assert_eq!(next_round.commitment_deadline, Some(deadline));
        assert!(next_round.target_frame_path.is_none());
    }

    #[tokio::test]
    async fn test_commitments_open_to_fee_collection_open() {
        let client = DummyTwitterClient;
        let round = Round::<Pending>::new("test-round".to_string())
            .open_commitments(Utc::now() + Duration::days(1), &client)
            .await
            .unwrap();
        
        let fee_deadline = Utc::now() + Duration::days(2);
        let next_round = round.close_commitments(fee_deadline, &client).await.unwrap();

        assert_eq!(next_round.state_name(), "FeeCollectionOpen");
        assert_eq!(next_round.fee_deadline, Some(fee_deadline));
    }

    #[tokio::test]
    async fn test_fee_collection_open_to_reveals_open() {
        let client = DummyTwitterClient;
        let round = Round::<Pending>::new("test-round".to_string())
            .open_commitments(Utc::now() + Duration::days(1), &client)
            .await
            .unwrap()
            .close_commitments(Utc::now() + Duration::days(2), &client)
            .await
            .unwrap();

        let reveals_deadline = Utc::now() + Duration::days(3);
        let target_frame_path = "path/to/frame.jpg".to_string();
        let next_round = round
            .close_fee_collection(target_frame_path.clone(), reveals_deadline, &client)
            .await
            .unwrap();

        assert_eq!(next_round.state_name(), "RevealsOpen");
        assert_eq!(next_round.reveals_deadline, Some(reveals_deadline));
        assert_eq!(next_round.target_frame_path, Some(target_frame_path));
    }

    #[tokio::test]
    async fn test_reveals_open_to_payouts() {
        let client = DummyTwitterClient;
        let round = Round::<Pending>::new("test-round".to_string())
            .open_commitments(Utc::now() + Duration::days(1), &client)
            .await
            .unwrap()
            .close_commitments(Utc::now() + Duration::days(2), &client)
            .await
            .unwrap()
            .close_fee_collection("path/to/frame.jpg".to_string(), Utc::now() + Duration::days(3), &client)
            .await
            .unwrap();

        let next_round = round.close_reveals(&client).await.unwrap();

        assert_eq!(next_round.state_name(), "Payouts");
    }

    #[tokio::test]
    async fn test_payouts_to_finished() {
        let client = DummyTwitterClient;
        let round = Round::<Pending>::new("test-round".to_string())
            .open_commitments(Utc::now() + Duration::days(1), &client)
            .await
            .unwrap()
            .close_commitments(Utc::now() + Duration::days(2), &client)
            .await
            .unwrap()
            .close_fee_collection("path/to/frame.jpg".to_string(), Utc::now() + Duration::days(3), &client)
            .await
            .unwrap()
            .close_reveals(&client)
            .await
            .unwrap();
        
        let next_round = round.process_payouts(&client).await.unwrap();

        assert_eq!(next_round.state_name(), "Finished");
        assert!(next_round.is_complete());
    }

    #[test]
    fn test_parse_state_from_string() {
        assert_eq!(parse_state_from_string("pending"), Some("Pending".to_string()));
        assert_eq!(parse_state_from_string("CommitmentsOpen"), Some("CommitmentsOpen".to_string()));
        assert_eq!(parse_state_from_string("REVEALSOPEN"), Some("RevealsOpen".to_string()));
        assert_eq!(parse_state_from_string("invalid"), None);
    }
}
