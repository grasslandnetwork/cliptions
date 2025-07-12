//! Async State Machine for Cliptions Round Engine
//! 
//! This module implements the round lifecycle using the Rust typestate pattern.
//! Each state is a marker type that ensures operations can only be performed
//! when the round is in the correct state, enforced by the compiler.

use std::fmt;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::{CliptionsError, Result};

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
    pub async fn open_commitments(
        mut self,
        commitment_deadline: DateTime<Utc>,
    ) -> Result<Round<CommitmentsOpen>> {
        // TODO: Post announcement tweet
        // This will be implemented when TwitterClient is available
        
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
    pub async fn close_commitments(
        mut self,
        fee_deadline: DateTime<Utc>,
    ) -> Result<Round<FeeCollectionOpen>> {
        // TODO: Post tweet announcing commitments are closed and fee collection is open
        // This will be implemented when TwitterClient is available
        
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
    pub async fn close_fee_collection(
        mut self,
        target_frame_path: String,
        reveals_deadline: DateTime<Utc>,
    ) -> Result<Round<RevealsOpen>> {
        // TODO: Post tweet with target frame and open reveals
        // This will be implemented when TwitterClient is available
        
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
    pub async fn close_reveals(self) -> Result<Round<Payouts>> {
        // TODO: Post tweet announcing reveals are closed
        // This will be implemented when TwitterClient is available
        
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
    pub async fn process_payouts(self) -> Result<Round<Finished>> {
        // TODO: Calculate scores, process payments, and post results
        // This will be implemented when scoring and payment systems are available
        
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

    #[tokio::test]
    async fn test_round_lifecycle() {
        let round_id = "test_round_001".to_string();
        let round = Round::new(round_id.clone());
        
        assert_eq!(round.id, round_id);
        assert_eq!(round.state_name(), "Pending");
        
        // Test state transitions
        let commitment_deadline = Utc::now() + Duration::hours(24);
        let round = round.open_commitments(commitment_deadline).await.unwrap();
        
        assert_eq!(round.state_name(), "CommitmentsOpen");
        
        let fee_deadline = Utc::now() + Duration::hours(48);
        let round = round.close_commitments(fee_deadline).await.unwrap();
        
        assert_eq!(round.state_name(), "FeeCollectionOpen");
        assert!(round.fee_deadline.is_some());
        
        let reveals_deadline = Utc::now() + Duration::hours(72);
        let round = round.close_fee_collection(
            "target_frame.jpg".to_string(),
            reveals_deadline,
        ).await.unwrap();
        
        assert_eq!(round.state_name(), "RevealsOpen");
        assert!(round.target_frame_path.is_some());
        
        let round = round.close_reveals().await.unwrap();
        assert_eq!(round.state_name(), "Payouts");
        
        let round = round.process_payouts().await.unwrap();
        assert_eq!(round.state_name(), "Finished");
        assert!(round.is_complete());
    }
    
    #[test]
    fn test_parse_state_from_string() {
        assert_eq!(parse_state_from_string("pending"), Some("Pending".to_string()));
        assert_eq!(parse_state_from_string("CommitmentsOpen"), Some("CommitmentsOpen".to_string()));
        assert_eq!(parse_state_from_string("REVEALSOPEN"), Some("RevealsOpen".to_string()));
        assert_eq!(parse_state_from_string("invalid"), None);
    }
}
