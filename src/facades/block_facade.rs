use chrono::{DateTime, Utc};

use crate::types::BlockData;
use crate::block_engine::state_machine::{Block as TypedBlock, CommitmentsOpen, StateMarker};

/// Stable access layer for block data regardless of underlying representation.
pub trait BlockFacade {
    fn block_num(&self) -> &str;
    fn prize_pool(&self) -> f64;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
    fn status(&self) -> crate::types::BlockStatus;
    fn target_image_path(&self) -> &str;

    fn is_commitment_phase(&self) -> bool;
    fn verified_participants_owned(&self) -> Vec<crate::types::Participant>;
}

impl BlockFacade for BlockData {
    fn block_num(&self) -> &str { &self.block_num }
    fn prize_pool(&self) -> f64 { self.prize_pool }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    fn status(&self) -> crate::types::BlockStatus { self.status.clone() }
    fn target_image_path(&self) -> &str { &self.target_image_path }

    fn is_commitment_phase(&self) -> bool {
        matches!(self.status, crate::types::BlockStatus::Open)
    }
    fn verified_participants_owned(&self) -> Vec<crate::types::Participant> {
        self
            .participants
            .iter()
            .filter(|p| p.verified)
            .cloned()
            .collect()
    }
}

impl<S: StateMarker> BlockFacade for TypedBlock<S> {
    fn block_num(&self) -> &str { &self.id }
    fn prize_pool(&self) -> f64 { 0.0 }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.created_at }
    fn status(&self) -> crate::types::BlockStatus { crate::types::BlockStatus::Open }
    fn target_image_path(&self) -> &str { "" }

    fn is_commitment_phase(&self) -> bool {
        S::state_name() == <CommitmentsOpen as StateMarker>::state_name()
    }
    fn verified_participants_owned(&self) -> Vec<crate::types::Participant> { Vec::new() }
}


