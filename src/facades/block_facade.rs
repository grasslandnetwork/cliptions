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
    fn participants_len(&self) -> usize;
    fn verified_participants_len(&self) -> usize;
    fn is_complete(&self) -> bool;
    fn total_payout(&self) -> f64;
    fn participants_owned(&self) -> Vec<crate::types::Participant>;
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
    fn participants_len(&self) -> usize { self.participants.len() }
    fn verified_participants_len(&self) -> usize { self.verified_participants().len() }
    fn is_complete(&self) -> bool { self.is_complete() }
    fn total_payout(&self) -> f64 {
        if self.is_complete() {
            self.results.iter().filter_map(|r| r.payout).sum()
        } else {
            0.0
        }
    }
    fn participants_owned(&self) -> Vec<crate::types::Participant> {
        self.participants.clone()
    }
}

impl<S: StateMarker> BlockFacade for TypedBlock<S> {
    fn block_num(&self) -> &str { &self.block_num }
    fn prize_pool(&self) -> f64 { self.prize_pool }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.created_at }
    fn status(&self) -> crate::types::BlockStatus {
        match S::state_name() {
            "Finished" => crate::types::BlockStatus::Complete,
            "CommitmentsOpen" | "RevealsOpen" => crate::types::BlockStatus::Open,
            _ => crate::types::BlockStatus::Processing,
        }
    }
    fn target_image_path(&self) -> &str { "" }

    fn is_commitment_phase(&self) -> bool {
        S::state_name() == <CommitmentsOpen as StateMarker>::state_name()
    }
    fn verified_participants_owned(&self) -> Vec<crate::types::Participant> {
        self.participants.iter().cloned().filter(|p| p.verified).collect()
    }
    fn participants_len(&self) -> usize { self.participants.len() }
    fn verified_participants_len(&self) -> usize { self.participants.iter().filter(|p| p.verified).count() }
    fn is_complete(&self) -> bool { matches!(self.status(), crate::types::BlockStatus::Complete) }
    fn total_payout(&self) -> f64 { self.total_payout }
    fn participants_owned(&self) -> Vec<crate::types::Participant> { self.participants.clone() }
}


