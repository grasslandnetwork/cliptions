//! Core data types for Cliptions
//!
//! This module defines the fundamental data structures used throughout the Cliptions system,
//! including participants, guesses, scoring results, and block data.

use chrono::{DateTime, Utc};
use ndarray::Array1;
use serde::{Deserialize, Serialize};
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
    /// Twitter User ID for the participant
    pub social_id: String,
    /// Twitter User ID for the participant
    pub username: String,
    /// The participant's plaintext guess
    pub guess: Guess,
    /// Guess URL for the participant
    pub guess_url: String,
    /// Cryptographic commitment to the guess
    pub commitment: String,
    /// Twitter URL ID for the commitment
    pub commitment_url: String,
    /// Wallet address for the participant
    pub wallet: String,
    /// Score for the participant
    pub score: f64,
    /// Payout for the participant
    pub payout: Payout,
    /// Salt used for the commitment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    /// Whether the commitment has been verified
    #[serde(default)]
    pub verified: bool,
}

impl Participant {
    /// Create a new participant
    pub fn new(social_id: String, username: String, guess: Guess, commitment: String) -> Self {
        Self {
            social_id,
            username,
            guess,
            guess_url: String::new(), // Will be set later
            commitment,
            commitment_url: String::new(), // Will be set later
            wallet: String::new(), // Will be set later
            score: 0.0, // Will be calculated later
            payout: Payout {
                amount: 0.0,
                currency: "TAO".to_string(),
                url: String::new(),
            },
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

    /// Set the guess URL
    pub fn with_guess_url(mut self, guess_url: String) -> Self {
        self.guess_url = guess_url;
        self
    }

    /// Set the commitment URL
    pub fn with_commitment_url(mut self, commitment_url: String) -> Self {
        self.commitment_url = commitment_url;
        self
    }

    /// Set the wallet address
    pub fn with_wallet(mut self, wallet: String) -> Self {
        self.wallet = wallet;
        self
    }

    /// Set the score
    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score;
        self
    }

    /// Set the payout
    pub fn with_payout(mut self, payout: Payout) -> Self {
        self.payout = payout;
        self
    }
}

/// Payout for a participant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Payout {
    pub amount: f64,
    pub currency: String,
    pub url: String,
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

/// Configuration for a prediction block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockConfig {
    /// Prize pool for the block
    pub prize_pool: f64,
    /// Maximum length for guesses
    pub max_guess_length: usize,
    /// Scoring version to use for this block
    pub scoring_version: String,
}

impl Default for BlockConfig {
    fn default() -> Self {
        Self {
            prize_pool: 100.0,
            max_guess_length: 300,
            scoring_version: "v0.3".to_string(),
        }
    }
}

/// Status of a prediction block
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockStatus {
    /// Block is accepting submissions
    Open,
    /// Block is closed, processing results
    Processing,
    /// Block is complete with results
    Complete,
    /// Block was cancelled
    Cancelled,
}

/// Complete data for a prediction block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    /// The block version number indicates which set of block validation rules to follow
    pub block_version: i32,
    /// Unique identifier for the block
    pub block_num: String,
    /// Path to the target image
    pub target_image_path: String,
    /// Current status of the block
    pub status: BlockStatus,
    /// Prize pool for the block
    pub prize_pool: f64,
    /// Twitter Conversation URL ID for the block
    pub social_id: String,
    /// Commitment deadline for the block
    pub commitment_deadline: DateTime<Utc>,
    /// Reveal deadline for the block
    pub reveal_deadline: DateTime<Utc>,
    /// Total payout for the block
    pub total_payout: f64,
    /// List of participants
    pub participants: Vec<Participant>,
    /// Scoring results (if processed)
    #[serde(default)]
    pub results: Vec<ScoringResult>,
    /// Timestamp when the block was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the block was last updated
    pub updated_at: DateTime<Utc>,
}

impl BlockData {
    /// Create a new block
    pub fn new(
        block_num: String,
        target_image_path: String,
        social_id: String,
        prize_pool: f64,
    ) -> Self {
        let now = Utc::now();
        Self {
            block_version: 1,
            block_num,
            target_image_path,
            status: BlockStatus::Open,
            prize_pool,
            social_id,
            commitment_deadline: now + chrono::Duration::hours(24), // Default 24 hours
            reveal_deadline: now + chrono::Duration::hours(48), // Default 48 hours
            total_payout: 0.0, // Will be calculated later
            participants: Vec::new(),
            results: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new block with custom deadlines
    pub fn with_deadlines(
        block_num: String,
        target_image_path: String,
        social_id: String,
        prize_pool: f64,
        commitment_deadline: DateTime<Utc>,
        reveal_deadline: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            block_version: 1,
            block_num,
            target_image_path,
            status: BlockStatus::Open,
            prize_pool,
            social_id,
            commitment_deadline,
            reveal_deadline,
            total_payout: 0.0, // Will be calculated later
            participants: Vec::new(),
            results: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a participant to the block
    pub fn add_participant(&mut self, participant: Participant) {
        self.participants.push(participant);
        self.updated_at = Utc::now();
    }

    /// Update the block status
    pub fn set_status(&mut self, status: BlockStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Set the results for the block
    pub fn set_results(&mut self, results: Vec<ScoringResult>) {
        self.results = results;
        self.status = BlockStatus::Complete;
        self.updated_at = Utc::now();
    }

    /// Get participants with verified commitments
    pub fn verified_participants(&self) -> Vec<&Participant> {
        self.participants.iter().filter(|p| p.verified).collect()
    }

    /// Check if the block is open for submissions
    pub fn is_open(&self) -> bool {
        matches!(self.status, BlockStatus::Open)
    }

    /// Check if the block is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.status, BlockStatus::Complete)
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
