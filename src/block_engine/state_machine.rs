//! Async State Machine for Cliptions Block Engine
//!
//! This module implements the block lifecycle using the Rust typestate pattern.
//! Each state is a marker type that ensures operations can only be performed
//! when the block is in the correct state, enforced by the compiler.

use crate::error::{CliptionsError, Result};
use crate::social::{AnnouncementData, AnnouncementFormatter};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use twitter_api::TwitterApi;
use std::path::PathBuf as StdPathBuf;

use crate::types::{BlockData, BlockStatus};

// --- State Markers ---

/// State marker for a block that hasn't started yet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pending;

/// State marker for a block accepting commitments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitmentsOpen;

/// State marker for a block with closed commitments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitmentsClosed;

/// State marker for when the target time has been reached and the frame captured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameCaptured;

/// State marker for a block accepting reveals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevealsOpen;

/// State marker for a block with closed reveals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevealsClosed;

/// State marker for a block processing payouts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Payouts;

/// State marker for a finished block
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Finished;

/// Trait for state markers to provide display names
pub trait StateMarker {
    fn state_name() -> &'static str;
}

impl StateMarker for Pending {
    fn state_name() -> &'static str {
        "Pending"
    }
}
impl StateMarker for CommitmentsOpen {
    fn state_name() -> &'static str {
        "CommitmentsOpen"
    }
}
impl StateMarker for CommitmentsClosed {
    fn state_name() -> &'static str {
        "CommitmentsClosed"
    }
}
impl StateMarker for FrameCaptured {
    fn state_name() -> &'static str {
        "FrameCaptured"
    }
}
impl StateMarker for RevealsOpen {
    fn state_name() -> &'static str {
        "RevealsOpen"
    }
}
impl StateMarker for RevealsClosed {
    fn state_name() -> &'static str {
        "RevealsClosed"
    }
}
impl StateMarker for Payouts {
    fn state_name() -> &'static str {
        "Payouts"
    }
}
impl StateMarker for Finished {
    fn state_name() -> &'static str {
        "Finished"
    }
}

/// Block data structure with typestate pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block<S> {
    pub block_num: String,
    pub created_at: DateTime<Utc>,

    // --- Block Parameters (Known at Start) ---
    pub description: String,
    pub livestream_url: String,
    pub target_timestamp: DateTime<Utc>,

    // --- Captured Data (Known Later) ---
    pub target_frame_path: Option<PathBuf>,

    // --- Deadlines ---
    pub commitment_deadline: Option<DateTime<Utc>>,
    pub reveals_deadline: Option<DateTime<Utc>>,

    #[serde(skip)]
    pub state: std::marker::PhantomData<S>,
}

impl<S> Block<S> {
    /// Get the current state name
    pub fn state_name(&self) -> &'static str
    where
        S: StateMarker,
    {
        S::state_name()
    }
}

impl<S> fmt::Display for Block<S>
where
    S: StateMarker,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Block {} ({})", self.block_num, S::state_name())
    }
}

/// Implementation for Pending state
impl Block<Pending> {
    /// Create a new pending block
    pub fn new(
        block_num: String,
        description: String,
        livestream_url: String,
        target_timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            block_num,
            created_at: Utc::now(),
            description,
            livestream_url,
            target_timestamp,
            target_frame_path: None,
            commitment_deadline: None,
            reveals_deadline: None,
            state: std::marker::PhantomData,
        }
    }

    /// Start the block by opening commitments
    pub async fn open_commitments<T: TwitterApi>(
        mut self,
        commitment_deadline: DateTime<Utc>,
        client: &T,
    ) -> Result<Block<CommitmentsOpen>> {
        let formatter = AnnouncementFormatter::new();
        let announcement_data = AnnouncementData {
            block_num: self
                .block_num
                .parse()
                .expect(&format!(
                    "CRITICAL: Invalid block ID '{}' - cannot proceed with block announcements",
                    self.block_num
                )),
            state_name: "CommitmentsOpen".to_string(),
            target_time: commitment_deadline.to_rfc3339(),
            hashtags: vec![],       // The formatter will add standard hashtags
            message: String::new(), // Not used for commitment announcements
            prize_pool: None,
            livestream_url: Some(self.livestream_url.clone()),
        };
        let tweet_text = formatter.create_commitment_announcement(&announcement_data);

        client
            .post_tweet(&tweet_text)
            .await
            .map_err(|e| CliptionsError::ApiError(e.to_string()))?;

        self.commitment_deadline = Some(commitment_deadline);

        Ok(Block {
            block_num: self.block_num,
            created_at: self.created_at,
            description: self.description,
            livestream_url: self.livestream_url,
            target_timestamp: self.target_timestamp,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        })
    }
}

/// Implementation for CommitmentsOpen state
impl Block<CommitmentsOpen> {
    /// Start directly in CommitmentsOpen state (skips Pending)
    pub fn start(
        block_num: String,
        description: String,
        livestream_url: String,
        target_timestamp: DateTime<Utc>,
        commitment_deadline: DateTime<Utc>,
    ) -> Self {
        Block::<CommitmentsOpen> {
            block_num,
            created_at: Utc::now(),
            description,
            livestream_url,
            target_timestamp,
            target_frame_path: None,
            commitment_deadline: Some(commitment_deadline),
            reveals_deadline: None,
            state: std::marker::PhantomData,
        }
    }
    /// Close commitments
    pub async fn close_commitments<T: TwitterApi>(
        self,
        client: &T,
    ) -> Result<Block<CommitmentsClosed>> {
        let formatter = AnnouncementFormatter::new();
        let announcement_data = AnnouncementData {
            block_num: self
                .block_num
                .parse()
                .expect(&format!(
                    "CRITICAL: Invalid block ID '{}' - cannot proceed with block announcements",
                    self.block_num
                )),
            state_name: "CommitmentsClosed".to_string(),
            target_time: self.target_timestamp.to_rfc3339(),
            hashtags: vec![],
            message: format!(
                "Block '{}': Commitments are now closed. Waiting for target time at {}.",
                self.block_num,
                self.target_timestamp.to_rfc3339()
            ),
            prize_pool: None,
            livestream_url: Some(self.livestream_url.clone()),
        };
        let tweet_text = formatter.format_announcement(&announcement_data, true);

        client
            .post_tweet(&tweet_text)
            .await
            .map_err(|e| CliptionsError::ApiError(e.to_string()))?;

        Ok(Block {
            block_num: self.block_num,
            created_at: self.created_at,
            description: self.description,
            livestream_url: self.livestream_url,
            target_timestamp: self.target_timestamp,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        })
    }
}

/// Implementation for CommitmentsClosed state
impl Block<CommitmentsClosed> {
    /// Capture the frame after the target time has passed.
    /// This is an internal state transition and does not tweet.
    pub fn capture_frame(mut self, target_frame_path: PathBuf) -> Result<Block<FrameCaptured>> {
        if Utc::now() < self.target_timestamp {
            return Err(CliptionsError::ValidationError(
                "Target timestamp has not yet been reached.".to_string(),
            ));
        }
        self.target_frame_path = Some(target_frame_path);
        Ok(Block {
            block_num: self.block_num,
            created_at: self.created_at,
            description: self.description,
            livestream_url: self.livestream_url,
            target_timestamp: self.target_timestamp,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        })
    }
}

/// Implementation for FrameCaptured state
impl Block<FrameCaptured> {
    /// Open the reveals phase by publishing the target frame.
    pub async fn open_reveals<T: TwitterApi>(
        mut self,
        reveals_deadline: DateTime<Utc>,
        client: &T,
        parent_tweet_id: &str,
    ) -> Result<Block<RevealsOpen>> {
        let formatter = AnnouncementFormatter::new();
        let announcement_data = AnnouncementData {
            block_num: self
                .block_num
                .parse()
                .expect(&format!(
                    "CRITICAL: Invalid block ID '{}' - cannot proceed with block announcements",
                    self.block_num
                )),
            state_name: "RevealsOpen".to_string(),
            target_time: reveals_deadline.to_rfc3339(),
            hashtags: vec![],
            message: format!(
                "Block '{}': Target frame revealed! Reveals are open until {}.",
                self.block_num,
                reveals_deadline.to_rfc3339()
            ),
            prize_pool: None,
            livestream_url: None, // No livestream URL needed for reveals open announcement
        };
        let tweet_text = formatter.format_announcement(&announcement_data, true);

        let frame_path = self.target_frame_path.clone().ok_or_else(|| {
            CliptionsError::ValidationError("Target frame path not set".to_string())
        })?;

        client
            .reply_to_tweet_with_image(&tweet_text, parent_tweet_id, frame_path) // Pass owned PathBuf
            .await
            .map_err(|e| CliptionsError::ApiError(e.to_string()))?;

        self.reveals_deadline = Some(reveals_deadline);

        Ok(Block {
            block_num: self.block_num,
            created_at: self.created_at,
            description: self.description,
            livestream_url: self.livestream_url,
            target_timestamp: self.target_timestamp,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        })
    }
}

/// Implementation for RevealsOpen state
impl Block<RevealsOpen> {
    /// Close reveals and start payout processing
    pub async fn close_reveals<T: TwitterApi>(self, _client: &T) -> Result<Block<Payouts>> {
        // This is a placeholder for the real implementation
        Ok(Block {
            block_num: self.block_num,
            created_at: self.created_at,
            description: self.description,
            livestream_url: self.livestream_url,
            target_timestamp: self.target_timestamp,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        })
    }
}

/// Implementation for Payouts state
impl Block<Payouts> {
    pub async fn process_payouts<T: TwitterApi>(self, _client: &T) -> Result<Block<Finished>> {
        // Placeholder
        Ok(Block {
            block_num: self.block_num,
            created_at: self.created_at,
            description: self.description,
            livestream_url: self.livestream_url,
            target_timestamp: self.target_timestamp,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
            reveals_deadline: self.reveals_deadline,
            state: std::marker::PhantomData,
        })
    }
}

impl Block<Finished> {
    pub fn is_complete(&self) -> bool {
        true
    }
}

// Utility functions for state transitions
impl<S> Block<S> {
    /// Convert to any other state (used for deserialization)
    pub fn into_state<T>(self) -> Block<T> {
        Block {
            block_num: self.block_num,
            created_at: self.created_at,
            description: self.description,
            livestream_url: self.livestream_url,
            target_timestamp: self.target_timestamp,
            target_frame_path: self.target_frame_path,
            commitment_deadline: self.commitment_deadline,
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
        "framecaptured" => Some("FrameCaptured".to_string()),
        "revealsopen" => Some("RevealsOpen".to_string()),
        "revealsclosed" => Some("RevealsClosed".to_string()),
        "payouts" => Some("Payouts".to_string()),
        "finished" => Some("Finished".to_string()),
        _ => None,
    }
}

// =============================================================================
// Legacy DTO Conversions (BlockData ↔ Block<S>)
// =============================================================================

impl From<&BlockData> for Block<CommitmentsOpen> {
    fn from(legacy: &BlockData) -> Self {
        // Map legacy fields into the unified typestate block. We start at CommitmentsOpen.
        let target_frame_path: Option<PathBuf> = if legacy.target_image_path.is_empty() {
            None
        } else {
            Some(PathBuf::from(legacy.target_image_path.clone()))
        };

        Block::<CommitmentsOpen> {
            block_num: legacy.block_num.clone(),
            created_at: legacy.created_at,
            description: String::new(),
            livestream_url: String::new(),
            // Use legacy reveal_deadline as the canonical target moment
            target_timestamp: legacy.reveal_deadline,
            target_frame_path,
            commitment_deadline: Some(legacy.commitment_deadline),
            reveals_deadline: Some(legacy.reveal_deadline),
            state: std::marker::PhantomData,
        }
    }
}

impl Block<CommitmentsOpen> {
    /// Convert this unified block back to legacy `BlockData` by merging with an existing template.
    /// Fields not represented in the unified struct are copied from `template` to preserve data.
    pub fn to_legacy_with_template(&self, template: &BlockData) -> BlockData {
        let mut out = template.clone();

        // Update core identity/time fields from unified block
        out.block_num = self.block_num.clone();
        out.commitment_deadline = self.commitment_deadline.unwrap_or(template.commitment_deadline);
        out.reveal_deadline = self.reveals_deadline.unwrap_or(template.reveal_deadline);

        if let Some(p) = &self.target_frame_path {
            out.target_image_path = p.to_string_lossy().to_string();
        }

        // Keep existing status if it has progressed; otherwise ensure at least Open
        if matches!(out.status, BlockStatus::Open | BlockStatus::Processing | BlockStatus::Complete | BlockStatus::Cancelled) {
            // leave as-is
        } else {
            out.status = BlockStatus::Open;
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Duration;
    use chrono::Utc;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use twitter_api::{PostTweetResult, Tweet, TwitterError};
    use crate::types::{BlockData as LegacyBlockData, BlockStatus as LegacyBlockStatus};

    /// A mock Twitter client that records calls for verification.
    #[derive(Clone)]
    struct MockTwitterClient {
        last_tweet_text: Arc<Mutex<Option<String>>>,
        last_image_path: Arc<Mutex<Option<PathBuf>>>,
    }

    impl MockTwitterClient {
        fn new() -> Self {
            Self {
                last_tweet_text: Arc::new(Mutex::new(None)),
                last_image_path: Arc::new(Mutex::new(None)),
            }
        }
    }

    #[async_trait]
    impl TwitterApi for MockTwitterClient {
        async fn post_tweet(&self, text: &str) -> twitter_api::Result<PostTweetResult> {
            *self.last_tweet_text.lock().unwrap() = Some(text.to_string());
            *self.last_image_path.lock().unwrap() = None;
            Ok(PostTweetResult {
                tweet: Tweet::default(),
                success: true,
            })
        }
        async fn post_tweet_with_image<P: AsRef<Path> + Send + 'static>(
            &self,
            text: &str,
            image_path: P,
        ) -> twitter_api::Result<PostTweetResult> {
            *self.last_tweet_text.lock().unwrap() = Some(text.to_string());
            *self.last_image_path.lock().unwrap() = Some(image_path.as_ref().to_path_buf());
            Ok(PostTweetResult {
                tweet: Tweet::default(),
                success: true,
            })
        }
        async fn reply_to_tweet(
            &self,
            _text: &str,
            _reply_to_tweet_id: &str,
        ) -> twitter_api::Result<PostTweetResult> {
            Ok(PostTweetResult {
                tweet: Tweet::default(),
                success: true,
            })
        }
        async fn reply_to_tweet_with_image<P: AsRef<Path> + Send + 'static>(
            &self,
            text: &str,
            reply_to_tweet_id: &str,
            image_path: P,
        ) -> twitter_api::Result<PostTweetResult> {
            *self.last_tweet_text.lock().unwrap() = Some(text.to_string());
            *self.last_image_path.lock().unwrap() = Some(image_path.as_ref().to_path_buf());
            Ok(PostTweetResult {
                tweet: Tweet::default(),
                success: true,
            })
        }
        async fn get_latest_tweet(
            &self,
            _username: &str,
            _exclude_retweets_replies: bool,
        ) -> twitter_api::Result<Option<Tweet>> {
            unimplemented!()
        }
        async fn search_replies(
            &self,
            _tweet_id: &str,
            _max_results: u32,
        ) -> twitter_api::Result<Vec<Tweet>> {
            unimplemented!()
        }
    }

    fn common_block() -> Block<Pending> {
        Block::new(
            "1".to_string(),
            "Test Theme".to_string(),
            "http://twitch.tv/test".to_string(),
            Utc::now() + Duration::days(1),
        )
    }

    #[tokio::test]
    async fn test_full_lifecycle_correct_flow() {
        let client = MockTwitterClient::new();
        let commitment_deadline = Utc::now() + Duration::hours(24);
        let reveals_deadline = Utc::now() + Duration::hours(48);

        // 1. Pending -> CommitmentsOpen
        let block = common_block();
        let block = block
            .open_commitments(commitment_deadline, &client)
            .await
            .unwrap();
        assert_eq!(block.state_name(), "CommitmentsOpen");
        assert_eq!(block.commitment_deadline, Some(commitment_deadline));
        let tweet1 = client.last_tweet_text.lock().unwrap().clone().unwrap();
        assert!(tweet1.contains("BLOCK 1 - Commitment Phase"));
        assert!(tweet1.contains("livestream: http://twitch.tv/test"));
        assert!(tweet1.contains("How To Play:"));
        assert!(tweet1.contains("Reply format ->"));

        // 2. CommitmentsOpen -> CommitmentsClosed
        let block = block.close_commitments(&client).await.unwrap();
        assert_eq!(block.state_name(), "CommitmentsClosed");
        let tweet2 = client.last_tweet_text.lock().unwrap().clone().unwrap();
        assert!(tweet2.contains("Commitments are now closed"));

        // 3. CommitmentsClosed -> FrameCaptured (Internal state change)
        // We simulate time passing for the check inside capture_frame
        let mut block = block;
        block.target_timestamp = Utc::now() - Duration::seconds(1);
        let frame_path = PathBuf::from("/tmp/test_frame.jpg");
        let block = block.capture_frame(frame_path.clone()).unwrap();
        assert_eq!(block.state_name(), "FrameCaptured");
        assert_eq!(block.target_frame_path.clone().unwrap(), frame_path);

        // 4. FrameCaptured -> RevealsOpen
        let block = block
            .open_reveals(reveals_deadline, &client, &tweet1)
            .await
            .unwrap();
        assert_eq!(block.state_name(), "RevealsOpen");
        assert_eq!(block.reveals_deadline, Some(reveals_deadline));
        let tweet3 = client.last_tweet_text.lock().unwrap().clone().unwrap();
        let image_path = client.last_image_path.lock().unwrap().clone().unwrap();
        assert!(tweet3.contains("Target frame revealed!"));
        assert_eq!(image_path, frame_path);

        // ... subsequent states would follow
    }

    #[tokio::test]
    async fn test_capture_frame_before_timestamp_fails() {
        let client = MockTwitterClient::new();
        let block = common_block()
            .open_commitments(Utc::now() + Duration::hours(1), &client)
            .await
            .unwrap()
            .close_commitments(&client)
            .await
            .unwrap();

        // This should fail because the target_timestamp is in the future
        let result = block.capture_frame(PathBuf::from("/tmp/fail.jpg"));
        assert!(result.is_err());
        if let Err(CliptionsError::ValidationError(msg)) = result {
            assert_eq!(msg, "Target timestamp has not yet been reached.");
        } else {
            panic!("Expected a ValidationError");
        }
    }

    #[test]
    fn test_start_commitments_open_constructor() {
        let now = Utc::now();
        let commit_deadline = now + Duration::hours(24);
        let block = Block::<CommitmentsOpen>::start(
            "42".to_string(),
            "Test".to_string(),
            "http://example.com".to_string(),
            now + Duration::days(1),
            commit_deadline,
        );
        assert_eq!(block.state_name(), "CommitmentsOpen");
        assert_eq!(block.block_num, "42");
        assert_eq!(block.commitment_deadline, Some(commit_deadline));
        assert!(block.reveals_deadline.is_none());
        assert!(block.target_frame_path.is_none());
    }

    #[test]
    fn test_legacy_round_trip_conversion() {
        // Create a legacy block
        let legacy = LegacyBlockData {
            block_version: 1,
            block_num: "7".to_string(),
            target_image_path: "/tmp/target.jpg".to_string(),
            status: LegacyBlockStatus::Open,
            prize_pool: 100.0,
            social_id: "tweet123".to_string(),
            commitment_deadline: Utc::now() + Duration::hours(1),
            reveal_deadline: Utc::now() + Duration::hours(2),
            total_payout: 0.0,
            participants: vec![],
            results: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Convert to unified CommitmentsOpen block
        let unified: Block<CommitmentsOpen> = (&legacy).into();
        assert_eq!(unified.block_num, legacy.block_num);
        assert_eq!(unified.commitment_deadline, Some(legacy.commitment_deadline));
        assert_eq!(unified.reveals_deadline, Some(legacy.reveal_deadline));

        // Convert back to legacy using the original as template
        let legacy_back = unified.to_legacy_with_template(&legacy);
        assert_eq!(legacy_back.block_num, legacy.block_num);
        assert_eq!(legacy_back.commitment_deadline, legacy.commitment_deadline);
        assert_eq!(legacy_back.reveal_deadline, legacy.reveal_deadline);
        assert_eq!(legacy_back.target_image_path, legacy.target_image_path);
    }

    #[test]
    fn test_legacy_pending_maps_to_commitments_open() {
        let legacy = LegacyBlockData {
            block_version: 1,
            block_num: "9".to_string(),
            target_image_path: String::new(),
            status: LegacyBlockStatus::Open, // change to Pending once available in legacy enum
            prize_pool: 0.0,
            social_id: String::new(),
            commitment_deadline: Utc::now() + Duration::hours(1),
            reveal_deadline: Utc::now() + Duration::hours(2),
            total_payout: 0.0,
            participants: vec![],
            results: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let unified: Block<CommitmentsOpen> = (&legacy).into();
        assert_eq!(unified.state_name(), "CommitmentsOpen");
    }
}
