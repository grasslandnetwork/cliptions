//! Core data types for RealMir
//! 
//! This module defines the fundamental data structures used throughout the RealMir system,
//! including participants, guesses, scoring results, and round data.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use ndarray::Array1;
use std::collections::HashMap;

/// A participant's guess in the prediction market
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Guess {
    /// The text content of the guess
    pub text: String,
    /// Embedding vector for the guess (if computed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f64>>,
    /// Timestamp when the guess was made
    pub timestamp: DateTime<Utc>,
    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Guess {
    /// Create a new guess with the current timestamp
    pub fn new(text: String) -> Self {
        Self {
            text,
            embedding: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a guess with a specific timestamp
    pub fn with_timestamp(text: String, timestamp: DateTime<Utc>) -> Self {
        Self {
            text,
            embedding: None,
            timestamp,
            metadata: HashMap::new(),
        }
    }
    
    /// Set the embedding for this guess
    pub fn with_embedding(mut self, embedding: Vec<f64>) -> Self {
        self.embedding = Some(embedding);
        self
    }
    
    /// Add metadata to the guess
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Get the embedding as an ndarray
    pub fn get_embedding_array(&self) -> Option<Array1<f64>> {
        self.embedding.as_ref().map(|e| Array1::from_vec(e.clone()))
    }
}

/// A participant in the prediction market
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Participant {
    /// Unique identifier for the participant
    pub user_id: String,
    /// Display name or username
    pub username: String,
    /// The participant's guess
    pub guess: Guess,
    /// Cryptographic commitment to the guess
    pub commitment: String,
    /// Salt used for the commitment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    /// Whether the commitment has been verified
    #[serde(default)]
    pub verified: bool,
}

impl Participant {
    /// Create a new participant
    pub fn new(user_id: String, username: String, guess: Guess, commitment: String) -> Self {
        Self {
            user_id,
            username,
            guess,
            commitment,
            salt: None,
            verified: false,
        }
    }
    
    /// Set the salt for commitment verification
    pub fn with_salt(mut self, salt: String) -> Self {
        self.salt = Some(salt);
        self
    }
    
    /// Mark the participant as verified
    pub fn mark_verified(mut self) -> Self {
        self.verified = true;
        self
    }
}

/// Result of scoring a participant's guess
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoringResult {
    /// Reference to the participant
    pub participant: Participant,
    /// Raw similarity score
    pub raw_score: f64,
    /// Adjusted similarity score (if applicable)
    pub adjusted_score: Option<f64>,
    /// Final rank in the competition
    pub rank: Option<usize>,
    /// Calculated payout amount
    pub payout: Option<f64>,
}

impl ScoringResult {
    /// Create a new scoring result
    pub fn new(participant: Participant, raw_score: f64) -> Self {
        Self {
            participant,
            raw_score,
            adjusted_score: None,
            rank: None,
            payout: None,
        }
    }
    
    /// Set the adjusted score
    pub fn with_adjusted_score(mut self, adjusted_score: f64) -> Self {
        self.adjusted_score = Some(adjusted_score);
        self
    }
    
    /// Set the rank
    pub fn with_rank(mut self, rank: usize) -> Self {
        self.rank = Some(rank);
        self
    }
    
    /// Set the payout
    pub fn with_payout(mut self, payout: f64) -> Self {
        self.payout = Some(payout);
        self
    }
    
    /// Get the effective score (adjusted if available, otherwise raw)
    pub fn effective_score(&self) -> f64 {
        self.adjusted_score.unwrap_or(self.raw_score)
    }
}

/// Configuration for a prediction round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundConfig {
    /// Prize pool for the round
    pub prize_pool: f64,
    /// Maximum length for guesses
    pub max_guess_length: usize,
    /// Scoring version to use for this round
    pub scoring_version: String,
}

impl Default for RoundConfig {
    fn default() -> Self {
        Self {
            prize_pool: 100.0,
            max_guess_length: 300,
            scoring_version: "v0.3".to_string(),
        }
    }
}

/// Status of a prediction round
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoundStatus {
    /// Round is accepting submissions
    Open,
    /// Round is closed, processing results
    Processing,
    /// Round is complete with results
    Complete,
    /// Round was cancelled
    Cancelled,
}

/// Raw Twitter reply data from browser automation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TwitterReplyData {
    /// URL of the original tweet
    pub original_tweet_url: String,
    /// Total number of replies found
    pub total_replies_found: u32,
    /// List of individual replies
    pub replies: Vec<TwitterReply>,
}

/// Individual Twitter reply
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TwitterReply {
    /// URL of the reply tweet
    pub url: String,
    /// Author of the reply (e.g., "@username")
    pub author: String,
    /// Preview text of the reply
    pub text_preview: String,
    /// Whether this reply was flagged as spam
    pub was_spam_flagged: bool,
}

/// Result of collecting commitments from Twitter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitmentCollectionResult {
    /// Whether the collection was successful
    pub success: bool,
    /// List of collected commitments
    pub commitments: Vec<CollectedCommitment>,
    /// URL of the announcement tweet
    pub announcement_url: String,
    /// Total number of commitments found
    pub total_commitments_found: u32,
    /// Error message if collection failed
    pub error_message: Option<String>,
}

/// A commitment collected from Twitter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollectedCommitment {
    /// Username of the committer
    pub username: String,
    /// The commitment hash
    pub commitment_hash: String,
    /// Wallet address provided
    pub wallet_address: String,
    /// URL of the commitment tweet
    pub tweet_url: String,
    /// Timestamp when commitment was collected
    pub timestamp: String,
}

/// Complete data for a prediction round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundData {
    /// Unique identifier for the round
    pub round_id: String,
    /// Human-readable title
    pub title: String,
    /// Description of the round
    pub description: String,
    /// Path to the target image
    pub target_image_path: String,
    /// Current status of the round
    pub status: RoundStatus,
    /// Configuration for the round
    pub config: RoundConfig,
    /// List of participants
    pub participants: Vec<Participant>,
    /// Scoring results (if processed)
    #[serde(default)]
    pub results: Vec<ScoringResult>,
    /// Timestamp when the round was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the round was last updated
    pub updated_at: DateTime<Utc>,
    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    
    /// Raw Twitter reply data from browser automation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_commitment_replies: Option<TwitterReplyData>,
    
    /// Processed commitment collection results
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collected_commitments: Option<CommitmentCollectionResult>,
}

impl RoundData {
    /// Create a new round
    pub fn new(
        round_id: String,
        title: String,
        description: String,
        target_image_path: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            round_id,
            title,
            description,
            target_image_path,
            status: RoundStatus::Open,
            config: RoundConfig::default(),
            participants: Vec::new(),
            results: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
            raw_commitment_replies: None,
            collected_commitments: None,
        }
    }
    
    /// Add a participant to the round
    pub fn add_participant(&mut self, participant: Participant) {
        self.participants.push(participant);
        self.updated_at = Utc::now();
    }
    
    /// Update the round status
    pub fn set_status(&mut self, status: RoundStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
    
    /// Set the results for the round
    pub fn set_results(&mut self, results: Vec<ScoringResult>) {
        self.results = results;
        self.status = RoundStatus::Complete;
        self.updated_at = Utc::now();
    }
    
    /// Get participants with verified commitments
    pub fn verified_participants(&self) -> Vec<&Participant> {
        self.participants.iter().filter(|p| p.verified).collect()
    }
    
    /// Check if the round is open for submissions
    pub fn is_open(&self) -> bool {
        matches!(self.status, RoundStatus::Open)
    }
    
    /// Check if the round is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.status, RoundStatus::Complete)
    }
    
    /// Set Twitter reply data for the round
    pub fn set_twitter_replies(&mut self, twitter_data: TwitterReplyData) {
        self.raw_commitment_replies = Some(twitter_data);
        self.updated_at = Utc::now();
    }
    
    /// Set commitment collection results for the round
    pub fn set_commitment_collection(&mut self, collection_result: CommitmentCollectionResult) {
        self.collected_commitments = Some(collection_result);
        self.updated_at = Utc::now();
    }
    
    /// Check if the round has Twitter data
    pub fn has_twitter_data(&self) -> bool {
        self.raw_commitment_replies.is_some()
    }
    
    /// Check if the round has commitment collection results
    pub fn has_commitment_collection(&self) -> bool {
        self.collected_commitments.is_some()
    }
}

/// Payout result for a participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutResult {
    /// Participant information
    pub participant: Participant,
    /// Amount to be paid out
    pub amount: f64,
    /// Rank in the competition
    pub rank: usize,
    /// Score that determined the rank
    pub score: f64,
}

impl PayoutResult {
    /// Create a new payout result
    pub fn new(participant: Participant, amount: f64, rank: usize, score: f64) -> Self {
        Self {
            participant,
            amount,
            rank,
            score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_twitter_reply_data_serialization() {
        let twitter_data = TwitterReplyData {
            original_tweet_url: "https://x.com/realmir_testnet/status/1907159517013422578".to_string(),
            total_replies_found: 2,
            replies: vec![
                TwitterReply {
                    url: "https://x.com/davidynamic/status/1907165981706760445".to_string(),
                    author: "@davidynamic".to_string(),
                    text_preview: "Commit: bc64a7b517b4e0a23c61300bb2e0601641fac6b387c76a1a9abb3d425c230235 Wallet: 5Co2unDtZKZDzYNZHT2fUMkEnpVWnassfbuabvZmGTrYKgtD".to_string(),
                    was_spam_flagged: false,
                }
            ],
        };

        let json = serde_json::to_string(&twitter_data).unwrap();
        let deserialized: TwitterReplyData = serde_json::from_str(&json).unwrap();
        assert_eq!(twitter_data, deserialized);
    }

    #[test]
    fn test_commitment_collection_result_serialization() {
        let collection_result = CommitmentCollectionResult {
            success: true,
            commitments: vec![
                CollectedCommitment {
                    username: "davidynamic".to_string(),
                    commitment_hash: "bc64a7b517b4e0a23c61300bb2e0601641fac6b387c76a1a9abb3d425c230235".to_string(),
                    wallet_address: "5Co2unDtZKZDzYNZHT2fUMkEnpVWnassfbuabvZmGTrYKgtD".to_string(),
                    tweet_url: "https://x.com/davidynamic/status/1907165981706760445".to_string(),
                    timestamp: "2025-06-14 12:42:07.464279".to_string(),
                }
            ],
            announcement_url: "https://x.com/realmir_testnet/status/1907159517013422578".to_string(),
            total_commitments_found: 2,
            error_message: None,
        };

        let json = serde_json::to_string(&collection_result).unwrap();
        let deserialized: CommitmentCollectionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(collection_result, deserialized);
    }

    #[test]
    fn test_round_data_with_enhanced_fields() {
        let mut round_data = RoundData::new(
            "test_round".to_string(),
            "Test Round".to_string(),
            "A test round".to_string(),
            "test.jpg".to_string(),
        );

        // Add Twitter data
        let twitter_data = TwitterReplyData {
            original_tweet_url: "https://x.com/realmir_testnet/status/1907159517013422578".to_string(),
            total_replies_found: 1,
            replies: vec![
                TwitterReply {
                    url: "https://x.com/davidynamic/status/1907165981706760445".to_string(),
                    author: "@davidynamic".to_string(),
                    text_preview: "Test reply".to_string(),
                    was_spam_flagged: false,
                }
            ],
        };
        round_data.set_twitter_replies(twitter_data);

        // Add commitment collection data
        let collection_result = CommitmentCollectionResult {
            success: true,
            commitments: vec![],
            announcement_url: "https://x.com/realmir_testnet/status/1907159517013422578".to_string(),
            total_commitments_found: 0,
            error_message: None,
        };
        round_data.set_commitment_collection(collection_result);

        // Test serialization/deserialization
        let json = serde_json::to_string(&round_data).unwrap();
        let deserialized: RoundData = serde_json::from_str(&json).unwrap();
        
        assert!(deserialized.has_twitter_data());
        assert!(deserialized.has_commitment_collection());
        assert_eq!(deserialized.raw_commitment_replies.unwrap().total_replies_found, 1);
        assert_eq!(deserialized.collected_commitments.unwrap().success, true);
    }

    #[test]
    fn test_actual_rounds_data_deserialization() {
        // Test that we can deserialize the actual data from rounds.json
        if let Ok(content) = fs::read_to_string("data/rounds.json") {
            // Parse the actual file structure (HashMap<String, serde_json::Value>)
            let rounds_data: serde_json::Value = serde_json::from_str(&content).unwrap();
            
            // Test that round2 (which has enhanced data) can be parsed
            if let Some(round2_data) = rounds_data.get("round2") {
                // Test that the enhanced fields exist and have the expected structure
                assert!(round2_data.get("raw_commitment_replies").is_some());
                assert!(round2_data.get("collected_commitments").is_some());

                // Test that we can deserialize the Twitter data specifically
                let twitter_data_json = round2_data.get("raw_commitment_replies").unwrap();
                let twitter_data: TwitterReplyData = serde_json::from_value(twitter_data_json.clone()).unwrap();
                assert_eq!(twitter_data.total_replies_found, 2);
                assert_eq!(twitter_data.replies.len(), 2);

                // Test that we can deserialize the commitment collection data
                let collection_data_json = round2_data.get("collected_commitments").unwrap();
                let collection_data: CommitmentCollectionResult = serde_json::from_value(collection_data_json.clone()).unwrap();
                assert!(collection_data.success);
                assert_eq!(collection_data.total_commitments_found, 2);
                assert_eq!(collection_data.commitments.len(), 2);
            }

            // Test that round0 and round1 (without enhanced data) can still be parsed
            if let Some(round0_data) = rounds_data.get("round0") {
                // These rounds shouldn't have the enhanced fields
                assert!(round0_data.get("raw_commitment_replies").is_none());
                assert!(round0_data.get("collected_commitments").is_none());
            }
        }
    }

    #[test]
    fn test_round_data_optional_fields() {
        // Test that RoundData can be created without the enhanced fields
        let round_data = RoundData::new(
            "test_round".to_string(),
            "Test Round".to_string(),
            "A test round".to_string(),
            "test.jpg".to_string(),
        );

        assert!(!round_data.has_twitter_data());
        assert!(!round_data.has_commitment_collection());
        assert!(round_data.raw_commitment_replies.is_none());
        assert!(round_data.collected_commitments.is_none());

        // Test JSON serialization without enhanced fields
        let json = serde_json::to_string(&round_data).unwrap();
        let deserialized: RoundData = serde_json::from_str(&json).unwrap();
        
        assert!(!deserialized.has_twitter_data());
        assert!(!deserialized.has_commitment_collection());
    }
}