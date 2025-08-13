//! Block processing for Cliptions prediction markets
//!
//! This module handles the complete lifecycle of prediction blocks,
//! including participant management, commitment verification, scoring, and payout calculation.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json;

use crate::commitment::CommitmentVerifier;
use crate::facades::block_facade::BlockFacade;
use crate::embedder::EmbedderTrait;
use crate::error::{Result, BlockError};
use crate::scoring::{process_participants, ScoreValidator, ScoringStrategy};
use crate::types::{Participant, BlockData, BlockStatus, ScoringResult};

/// Block processor for managing prediction blocks
pub struct BlockProcessor<E: EmbedderTrait, S: ScoringStrategy> {
    blocks_file: String,
    commitment_verifier: CommitmentVerifier,
    score_validator: ScoreValidator<E, S>,
    blocks_cache: HashMap<String, BlockData>,
}

impl<E: EmbedderTrait, S: ScoringStrategy> BlockProcessor<E, S> {
    /// Create a new block processor
    pub fn new(blocks_file: String, embedder: E, scoring_strategy: S) -> Self {
        Self {
            blocks_file,
            commitment_verifier: CommitmentVerifier::new(),
            score_validator: ScoreValidator::new(embedder, scoring_strategy),
            blocks_cache: HashMap::new(),
        }
    }

    /// Load blocks data from file
    pub fn load_blocks(&mut self) -> Result<()> {
        if !Path::new(&self.blocks_file).exists() {
                    // Create empty blocks file if it doesn't exist
        let empty_blocks: HashMap<String, BlockData> = HashMap::new();
            self.save_blocks(&empty_blocks)?;
            return Ok(());
        }

        let content =
            fs::read_to_string(&self.blocks_file).map_err(|_e| BlockError::DataFileNotFound {
                path: self.blocks_file.clone(),
            })?;

        // Handle empty file case
        if content.trim().is_empty() {
            let empty_blocks: HashMap<String, BlockData> = HashMap::new();
            self.save_blocks(&empty_blocks)?;
            return Ok(());
        }

        let blocks: HashMap<String, BlockData> = serde_json::from_str(&content)?;
        self.blocks_cache = blocks;

        Ok(())
    }

    /// Save blocks data to file
    pub fn save_blocks(&self, blocks: &HashMap<String, BlockData>) -> Result<()> {
        let content = serde_json::to_string_pretty(blocks)?;
        fs::write(&self.blocks_file, content)?;
        Ok(())
    }

    /// Get a block by ID
    pub fn get_block(&mut self, block_num: &str) -> Result<&BlockData> {
        if self.blocks_cache.is_empty() {
            self.load_blocks()?;
        }

        self.blocks_cache.get(block_num).ok_or_else(|| {
            BlockError::BlockNotFound {
                block_num: block_num.to_string(),
            }
            .into()
        })
    }

    /// Get a mutable reference to a block
    pub fn get_block_mut(&mut self, block_num: &str) -> Result<&mut BlockData> {
        if self.blocks_cache.is_empty() {
            self.load_blocks()?;
        }

        self.blocks_cache.get_mut(block_num).ok_or_else(|| {
            BlockError::BlockNotFound {
                block_num: block_num.to_string(),
            }
            .into()
        })
    }

    /// Create a new block
    pub fn create_block(
        &mut self,
        block_num: String,
        target_image_path: String,
        social_id: String,
        prize_pool: f64,
        commitment_deadline: Option<DateTime<Utc>>,
        reveal_deadline: Option<DateTime<Utc>>,
    ) -> Result<()> {
        if self.blocks_cache.is_empty() {
            self.load_blocks()?;
        }

        if self.blocks_cache.contains_key(&block_num) {
            return Err(BlockError::AlreadyProcessed.into());
        }

        let block = if let (Some(commit_deadline), Some(reveal_deadline)) = (commitment_deadline, reveal_deadline) {
            BlockData::with_deadlines(
                block_num.clone(),
                target_image_path,
                social_id,
                prize_pool,
                commit_deadline,
                reveal_deadline,
            )
        } else {
            BlockData::new(
                block_num.clone(),
                target_image_path,
                social_id,
                prize_pool,
            )
        };

        self.blocks_cache.insert(block_num, block);
        self.save_blocks(&self.blocks_cache)?;

        Ok(())
    }

    /// Add a participant to a block
    pub fn add_participant(&mut self, block_num: &str, participant: Participant) -> Result<()> {
        let block = self.get_block_mut(block_num)?;

        // if !block.is_open() {
        //     return Err(BlockError::AlreadyProcessed.into());
        // }

        block.add_participant(participant);
        self.save_blocks(&self.blocks_cache)?;

        Ok(())
    }

    /// Verify commitments for a block
    pub fn verify_commitments(&mut self, block_num: &str) -> Result<Vec<bool>> {
        // Load blocks if needed
        if self.blocks_cache.is_empty() {
            self.load_blocks()?;
        }

        let block =
            self.blocks_cache
                .get_mut(block_num)
                .ok_or_else(|| BlockError::BlockNotFound {
                    block_num: block_num.to_string(),
                })?;

        let mut results = Vec::new();

        for participant in &mut block.participants {
            if let Some(salt) = &participant.salt {
                let is_valid = self.commitment_verifier.verify(
                    &participant.guess.text,
                    salt,
                    &participant.commitment,
                );

                if is_valid {
                    participant.verified = true;
                }

                results.push(is_valid);
            } else {
                results.push(false);
            }
        }

        self.save_blocks(&self.blocks_cache)?;
        Ok(results)
    }

    /// Process block payouts
    pub fn process_block_payouts(&mut self, block_num: &str) -> Result<Vec<ScoringResult>> {
        // Load blocks if needed
        if self.blocks_cache.is_empty() {
            self.load_blocks()?;
        }

        // Get block data first
        let (target_image_path, prize_pool, verified_participants) = {
            let block =
                self.blocks_cache
                    .get(block_num)
                    .ok_or_else(|| BlockError::BlockNotFound {
                        block_num: block_num.to_string(),
                    })?;

            // Use facade for reads
            let facade = block as &dyn BlockFacade;

            // Verify target image exists
            if !Path::new(facade.target_image_path()).exists() {
                return Err(BlockError::TargetImageNotFound {
                    path: facade.target_image_path().to_string(),
                }
                .into());
            }

            // Get verified participants via facade
            let verified_participants: Vec<Participant> = facade.verified_participants_owned();

            if verified_participants.is_empty() {
                return Err(BlockError::NoParticipants {
                    block_num: block_num.to_string(),
                }
                .into());
            }

            (
                facade.target_image_path().to_string(),
                facade.prize_pool(),
                verified_participants,
            )
        };

        // Process participants and calculate scores
        let results = process_participants(
            &verified_participants,
            &target_image_path,
            prize_pool,
            &self.score_validator,
        )?;

        // Update block status to Complete (but don't add redundant results section)
        let block = self.blocks_cache.get_mut(block_num).unwrap(); // Safe because we checked above
        block.set_status(BlockStatus::Complete);
        self.save_blocks(&self.blocks_cache)?;

        Ok(results)
    }

    /// Get all block IDs
    pub fn get_block_nums(&mut self) -> Result<Vec<String>> {
        if self.blocks_cache.is_empty() {
            self.load_blocks()?;
        }

        Ok(self.blocks_cache.keys().cloned().collect())
    }

    /// Process all blocks
    pub fn process_all_blocks(&mut self) -> Result<HashMap<String, Vec<ScoringResult>>> {
        let block_nums = self.get_block_nums()?;
        let mut all_results = HashMap::new();

        for block_num in block_nums {
            // Only process blocks that are open or processing
            let block = self.get_block(&block_num)?;
            let facade = block as &dyn BlockFacade;
            if matches!(facade.status(), BlockStatus::Open | BlockStatus::Processing) {
                match self.process_block_payouts(&block_num) {
                    Ok(results) => {
                        all_results.insert(block_num, results);
                    }
                    Err(e) => {
                        panic!("CRITICAL: Failed to process block {}: {}. Cannot continue batch processing with incomplete results as this could lead to missing payouts.", block_num, e);
                    }
                }
            }
        }

        Ok(all_results)
    }

    /// Get block statistics
    pub fn get_block_stats(&mut self, block_num: &str) -> Result<BlockStats> {
        let block = self.get_block(block_num)?;
        let facade = block as &dyn BlockFacade;

        let total_participants = facade.participants_len();
        let verified_participants = facade.verified_participants_len();
        let total_prize_pool = facade.prize_pool();
        let is_complete = facade.is_complete();
        let total_payout = facade.total_payout();

        Ok(BlockStats {
            block_num: block_num.to_string(),
            total_participants,
            verified_participants,
            total_prize_pool,
            total_payout,
            is_complete,
            status: facade.status(),
        })
    }
}

/// Statistics for a block
#[derive(Debug, Clone)]
pub struct BlockStats {
    pub block_num: String,
    pub total_participants: usize,
    pub verified_participants: usize,
    pub total_prize_pool: f64,
    pub total_payout: f64,
    pub is_complete: bool,
    pub status: BlockStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedder::ClipEmbedder;
    use crate::scoring::ClipBatchStrategy;
    use crate::types::Guess;
    use tempfile::NamedTempFile;

    fn create_test_processor() -> (BlockProcessor<ClipEmbedder, ClipBatchStrategy>, String) {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let embedder = ClipEmbedder::new().expect("CLIP model should load in tests");
        let strategy = ClipBatchStrategy::new();
        let processor = BlockProcessor::new(file_path.clone(), embedder, strategy);

        (processor, file_path)
    }

    fn create_test_participant(user_id: &str, guess_text: &str, commitment: &str) -> Participant {
        let guess = Guess::new(guess_text.to_string());
        Participant::new(
            user_id.to_string(), // This is now social_id
            format!("user_{}", user_id),
            guess,
            commitment.to_string(),
        )
        .with_salt("test_salt".to_string())
        .mark_verified()
    }

    #[test]
    fn test_block_processor_creation() {
        let (mut processor, _) = create_test_processor();

        // Should be able to load empty blocks
        processor.load_blocks().unwrap();
        assert!(processor.get_block_nums().unwrap().is_empty());
    }

    #[test]
    fn test_create_block() {
        let (mut processor, _) = create_test_processor();

        processor
            .create_block(
                "test_block".to_string(),
                "test.jpg".to_string(),
                "test_social_id".to_string(),
                100.0,
                Some(Utc::now() + chrono::Duration::days(1)),
                Some(Utc::now() + chrono::Duration::days(2)),
            )
            .unwrap();

        let block = processor.get_block("test_block").unwrap();
        assert_eq!(block.block_num, "test_block");
        assert_eq!(block.target_image_path, "test.jpg");
        assert_eq!(block.social_id, "test_social_id");
        assert_eq!(block.prize_pool, 100.0);
    }

    #[test]
    fn test_add_participant() {
        let (mut processor, _) = create_test_processor();

        processor
            .create_block(
                "test_block".to_string(),
                "test.jpg".to_string(),
                "test_social_id".to_string(),
                100.0,
                Some(Utc::now() + chrono::Duration::days(1)),
                Some(Utc::now() + chrono::Duration::days(2)),
            )
            .unwrap();

        let participant = create_test_participant("user1", "test guess", "commitment123");

        processor
            .add_participant("test_block", participant)
            .unwrap();

        let block = processor.get_block("test_block").unwrap();
        assert_eq!(block.participants.len(), 1);
        assert_eq!(block.participants[0].social_id, "user1");
    }

    #[test]
    fn test_verify_commitments() {
        let (mut processor, _) = create_test_processor();

        processor
            .create_block(
                "test_block".to_string(),
                "test.jpg".to_string(),
                "test_social_id".to_string(),
                100.0,
                Some(Utc::now() + chrono::Duration::days(1)),
                Some(Utc::now() + chrono::Duration::days(2)),
            )
            .unwrap();

        // Create a valid commitment
        let commitment_gen = crate::commitment::CommitmentGenerator::new();
        let salt = "test_salt";
        let message = "test guess";
        let commitment = commitment_gen.generate(message, salt).unwrap();

        let participant = Participant::new(
            "user1".to_string(),
            "user_user1".to_string(),
            Guess::new(message.to_string()),
            commitment,
        )
        .with_salt(salt.to_string());

        processor
            .add_participant("test_block", participant)
            .unwrap();

        let results = processor.verify_commitments("test_block").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0]); // Should be valid

        let block = processor.get_block("test_block").unwrap();
        assert!(block.participants[0].verified);
    }

    #[test]
    fn test_block_stats() {
        let (mut processor, _) = create_test_processor();

        processor
            .create_block(
                "test_block".to_string(),
                "test.jpg".to_string(),
                "test_social_id".to_string(),
                100.0,
                Some(Utc::now() + chrono::Duration::days(1)),
                Some(Utc::now() + chrono::Duration::days(2)),
            )
            .unwrap();

        let participant = create_test_participant("user1", "test guess", "commitment123");
        processor
            .add_participant("test_block", participant)
            .unwrap();

        let stats = processor.get_block_stats("test_block").unwrap();
        assert_eq!(stats.block_num, "test_block");
        assert_eq!(stats.total_participants, 1);
        assert_eq!(stats.verified_participants, 1);
        assert!(!stats.is_complete);
    }

    #[test]
    fn test_nonexistent_block() {
        let (mut processor, _) = create_test_processor();

        let result = processor.get_block("nonexistent");
        assert!(matches!(
            result,
            Err(crate::error::CliptionsError::Block(
                BlockError::BlockNotFound { .. }
            ))
        ));
    }
}
