use chrono::{DateTime, Utc};

use crate::types::BlockData;
use crate::block_engine::state_machine::{Block as TypedBlock, CommitmentsOpen, StateMarker};

/// Stable access layer for block data regardless of underlying representation.
pub trait BlockFacade {
    fn block_num(&self) -> &str;
    fn prize_pool(&self) -> f64;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;

    fn is_commitment_phase(&self) -> bool;
}

impl BlockFacade for BlockData {
    fn block_num(&self) -> &str { &self.block_num }
    fn prize_pool(&self) -> f64 { self.prize_pool }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }

    fn is_commitment_phase(&self) -> bool {
        matches!(self.status, crate::types::BlockStatus::Open)
    }
}

impl<S: StateMarker> BlockFacade for TypedBlock<S> {
    fn block_num(&self) -> &str { &self.id }
    fn prize_pool(&self) -> f64 { 0.0 }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.created_at }

    fn is_commitment_phase(&self) -> bool {
        S::state_name() == <CommitmentsOpen as StateMarker>::state_name()
    }
}



