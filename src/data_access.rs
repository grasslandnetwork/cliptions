//! Data access layer for RealMir round data
//! 
//! This module provides a centralized data access layer with CRUD operations,
//! data integrity validation, backup/restore functionality, and atomic file operations.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json;
use chrono::Utc;

use crate::types::{RoundData, TwitterReplyData, CommitmentCollectionResult};
use crate::error::{DataAccessError, Result};

/// Centralized data access layer for round data
pub struct DataAccessLayer {
    rounds_file_path: String,
    backup_directory: String,
}

impl DataAccessLayer {
    /// Create new data access layer instance
    pub fn new(rounds_file_path: String) -> Self {
        let backup_directory = format!("{}.backups", rounds_file_path);
        Self {
            rounds_file_path,
            backup_directory,
        }
    }

    /// Create a new data access layer with custom backup directory
    pub fn with_backup_dir(rounds_file_path: String, backup_directory: String) -> Self {
        Self {
            rounds_file_path,
            backup_directory,
        }
    }

    // Core CRUD operations

    /// Load all rounds from the data file
    pub fn load_all_rounds(&self) -> Result<HashMap<String, RoundData>> {
        if !Path::new(&self.rounds_file_path).exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.rounds_file_path)
            .map_err(|_| DataAccessError::DataFileNotFound {
                path: self.rounds_file_path.clone(),
            })?;

        if content.trim().is_empty() {
            return Ok(HashMap::new());
        }

        // Parse as raw JSON first to handle the actual data structure
        let raw_data: serde_json::Value = serde_json::from_str(&content)?;
        
        // Convert to our RoundData structure
        let mut rounds = HashMap::new();
        if let Some(obj) = raw_data.as_object() {
            for (round_id, round_value) in obj {
                // Try to convert the raw round data to our RoundData structure
                match self.convert_raw_round_to_round_data(round_id, round_value) {
                    Ok(round_data) => {
                        rounds.insert(round_id.clone(), round_data);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse round {}: {}", round_id, e);
                        // Continue processing other rounds even if one fails
                    }
                }
            }
        }

        Ok(rounds)
    }

    /// Save all rounds to the data file atomically
    pub fn save_all_rounds(&self, rounds: &HashMap<String, RoundData>) -> Result<()> {
        let content = serde_json::to_string_pretty(rounds)?;
        self.atomic_write(&self.rounds_file_path, &content)?;
        Ok(())
    }

    /// Get a specific round by ID
    pub fn get_round(&self, round_id: &str) -> Result<RoundData> {
        let rounds = self.load_all_rounds()?;
        rounds.get(round_id)
            .cloned()
            .ok_or_else(|| DataAccessError::RoundNotFound {
                round_id: round_id.to_string(),
            }.into())
    }

    /// Update a specific round
    pub fn update_round(&self, round_id: &str, round_data: RoundData) -> Result<()> {
        let mut rounds = self.load_all_rounds()?;
        rounds.insert(round_id.to_string(), round_data);
        self.save_all_rounds(&rounds)?;
        Ok(())
    }

    /// Delete a specific round
    pub fn delete_round(&self, round_id: &str) -> Result<()> {
        let mut rounds = self.load_all_rounds()?;
        if rounds.remove(round_id).is_none() {
            return Err(DataAccessError::RoundNotFound {
                round_id: round_id.to_string(),
            }.into());
        }
        self.save_all_rounds(&rounds)?;
        Ok(())
    }

    /// Check if a round exists
    pub fn round_exists(&self, round_id: &str) -> Result<bool> {
        let rounds = self.load_all_rounds()?;
        Ok(rounds.contains_key(round_id))
    }

    // Enhanced operations

    /// Get all round IDs
    pub fn get_all_round_ids(&self) -> Result<Vec<String>> {
        let rounds = self.load_all_rounds()?;
        Ok(rounds.keys().cloned().collect())
    }

    /// Get rounds that have Twitter data
    pub fn get_rounds_with_twitter_data(&self) -> Result<Vec<String>> {
        let rounds = self.load_all_rounds()?;
        Ok(rounds.iter()
            .filter(|(_, round)| round.has_twitter_data())
            .map(|(id, _)| id.clone())
            .collect())
    }

    /// Get rounds that have commitment collection results
    pub fn get_rounds_with_commitments(&self) -> Result<Vec<String>> {
        let rounds = self.load_all_rounds()?;
        Ok(rounds.iter()
            .filter(|(_, round)| round.has_commitment_collection())
            .map(|(id, _)| id.clone())
            .collect())
    }

    // Data integrity operations

    /// Validate data consistency across all rounds
    pub fn validate_data_consistency(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        let rounds = self.load_all_rounds()?;

        for (round_id, round_data) in &rounds {
            // Check if round_id matches the data
            if round_data.round_id != *round_id {
                issues.push(format!("Round ID mismatch: key '{}' vs data '{}'", round_id, round_data.round_id));
            }

            // Check if target image exists
            if !Path::new(&round_data.target_image_path).exists() {
                issues.push(format!("Round {}: target image not found at '{}'", round_id, round_data.target_image_path));
            }

            // Validate Twitter data consistency
            if let Some(twitter_data) = &round_data.raw_commitment_replies {
                if twitter_data.replies.len() != twitter_data.total_replies_found as usize {
                    issues.push(format!("Round {}: Twitter data inconsistency - found {} replies but count is {}", 
                        round_id, twitter_data.replies.len(), twitter_data.total_replies_found));
                }
            }

            // Validate commitment collection consistency
            if let Some(collection_data) = &round_data.collected_commitments {
                if collection_data.commitments.len() != collection_data.total_commitments_found as usize {
                    issues.push(format!("Round {}: Commitment collection inconsistency - found {} commitments but count is {}", 
                        round_id, collection_data.commitments.len(), collection_data.total_commitments_found));
                }
            }

            // Check for duplicate participants
            let mut usernames = std::collections::HashSet::new();
            for participant in &round_data.participants {
                if !usernames.insert(&participant.username) {
                    issues.push(format!("Round {}: duplicate participant '{}'", round_id, participant.username));
                }
            }
        }

        Ok(issues)
    }

    /// Create a backup of the current data
    pub fn create_backup(&self) -> Result<String> {
        // Ensure backup directory exists
        if !Path::new(&self.backup_directory).exists() {
            fs::create_dir_all(&self.backup_directory)?;
        }

        // Generate backup filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("rounds_backup_{}.json", timestamp);
        let backup_path = format!("{}/{}", self.backup_directory, backup_filename);

        // Copy current data file to backup
        if Path::new(&self.rounds_file_path).exists() {
            fs::copy(&self.rounds_file_path, &backup_path)?;
        } else {
            // Create empty backup if original doesn't exist
            fs::write(&backup_path, "{}")?;
        }

        Ok(backup_path)
    }

    /// Restore data from a backup
    pub fn restore_from_backup(&self, backup_path: &str) -> Result<()> {
        if !Path::new(backup_path).exists() {
            return Err(DataAccessError::BackupFailed {
                reason: format!("Backup file not found: {}", backup_path),
            }.into());
        }

        // Validate backup file can be parsed
        let content = fs::read_to_string(backup_path)?;
        let _: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| DataAccessError::DataCorruption {
                details: format!("Backup file is corrupted: {}", e),
            })?;

        // Copy backup to main data file
        fs::copy(backup_path, &self.rounds_file_path)?;
        Ok(())
    }

    // Atomic update operations

    /// Update Twitter data for a specific round
    pub fn update_round_twitter_data(&self, round_id: &str, twitter_data: TwitterReplyData) -> Result<()> {
        let mut rounds = self.load_all_rounds()?;
        let round = rounds.get_mut(round_id)
            .ok_or_else(|| DataAccessError::RoundNotFound {
                round_id: round_id.to_string(),
            })?;

        round.set_twitter_replies(twitter_data);
        self.save_all_rounds(&rounds)?;
        Ok(())
    }

    /// Update commitment collection results for a specific round
    pub fn update_round_commitments(&self, round_id: &str, commitments: CommitmentCollectionResult) -> Result<()> {
        let mut rounds = self.load_all_rounds()?;
        let round = rounds.get_mut(round_id)
            .ok_or_else(|| DataAccessError::RoundNotFound {
                round_id: round_id.to_string(),
            })?;

        round.set_commitment_collection(commitments);
        self.save_all_rounds(&rounds)?;
        Ok(())
    }

    // Private helper methods

    /// Perform atomic write operation to prevent corruption
    fn atomic_write(&self, file_path: &str, content: &str) -> Result<()> {
        let temp_path = format!("{}.tmp", file_path);
        
        // Write to temporary file first
        fs::write(&temp_path, content)?;
        
        // Atomically move to final location
        fs::rename(&temp_path, file_path)?;
        
        Ok(())
    }

    /// Convert raw JSON round data to RoundData structure
    fn convert_raw_round_to_round_data(&self, round_id: &str, round_value: &serde_json::Value) -> Result<RoundData> {
        // First try to deserialize directly as RoundData (for data saved by our system)
        if let Ok(mut round_data) = serde_json::from_value::<RoundData>(round_value.clone()) {
            // Ensure the round_id matches
            round_data.round_id = round_id.to_string();
            return Ok(round_data);
        }

        // If direct deserialization fails, convert from the legacy format
        let mut round_data = RoundData::new(
            round_id.to_string(),
            round_value.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or(&format!("Round {}", round_id))
                .to_string(),
            round_value.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or(&format!("Prediction round {}", round_id))
                .to_string(),
            round_value.get("target_image")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown.jpg")
                .to_string(),
        );

        // Handle enhanced fields
        if let Some(twitter_data) = round_value.get("raw_commitment_replies") {
            let twitter_reply_data: TwitterReplyData = serde_json::from_value(twitter_data.clone())?;
            round_data.set_twitter_replies(twitter_reply_data);
        }

        if let Some(collection_data) = round_value.get("collected_commitments") {
            let commitment_collection: CommitmentCollectionResult = serde_json::from_value(collection_data.clone())?;
            round_data.set_commitment_collection(commitment_collection);
        }

        // TODO: Handle other fields like participants, results, etc.
        // For now, we focus on the enhanced fields as specified in the task

        Ok(round_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::types::{TwitterReply, CollectedCommitment};

    fn create_test_dal() -> (DataAccessLayer, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let rounds_file = temp_dir.path().join("rounds.json");
        let backup_dir = temp_dir.path().join("backups");
        
        let dal = DataAccessLayer::with_backup_dir(
            rounds_file.to_string_lossy().to_string(),
            backup_dir.to_string_lossy().to_string()
        );
        
        (dal, temp_dir)
    }

    #[test]
    fn test_crud_operations() {
        let (dal, _temp_dir) = create_test_dal();

        // Test empty state
        let rounds = dal.load_all_rounds().unwrap();
        assert!(rounds.is_empty());

        // Test create
        let round_data = RoundData::new(
            "test_round".to_string(),
            "Test Round".to_string(),
            "Test description".to_string(),
            "test.jpg".to_string(),
        );
        dal.update_round("test_round", round_data.clone()).unwrap();

        // Test read
        let loaded_round = dal.get_round("test_round").unwrap();
        assert_eq!(loaded_round.round_id, "test_round");
        assert_eq!(loaded_round.title, "Test Round");

        // Test round exists
        assert!(dal.round_exists("test_round").unwrap());
        assert!(!dal.round_exists("nonexistent").unwrap());

        // Test get all round IDs
        let round_ids = dal.get_all_round_ids().unwrap();
        assert_eq!(round_ids.len(), 1);
        assert!(round_ids.contains(&"test_round".to_string()));

        // Test delete
        dal.delete_round("test_round").unwrap();
        assert!(!dal.round_exists("test_round").unwrap());
    }

    #[test]
    fn test_atomic_updates() {
        let (dal, _temp_dir) = create_test_dal();

        let round_data = RoundData::new(
            "test_round".to_string(),
            "Test Round".to_string(),
            "Test description".to_string(),
            "test.jpg".to_string(),
        );

        dal.update_round("test_round", round_data.clone()).unwrap();

        // Test atomic Twitter data update
        let twitter_data = TwitterReplyData {
            original_tweet_url: "https://example.com/tweet".to_string(),
            total_replies_found: 1,
            replies: vec![
                TwitterReply {
                    url: "https://example.com/reply".to_string(),
                    author: "@testuser".to_string(),
                    text_preview: "Test reply".to_string(),
                    was_spam_flagged: false,
                }
            ],
        };

        dal.update_round_twitter_data("test_round", twitter_data).unwrap();

        // Verify update
        let updated_round = dal.get_round("test_round").unwrap();
        assert!(updated_round.has_twitter_data());
        assert_eq!(updated_round.raw_commitment_replies.unwrap().total_replies_found, 1);

        // Test atomic commitment collection update
        let collection_result = CommitmentCollectionResult {
            success: true,
            commitments: vec![
                CollectedCommitment {
                    username: "testuser".to_string(),
                    commitment_hash: "test_hash".to_string(),
                    wallet_address: "test_wallet".to_string(),
                    tweet_url: "https://example.com/commit".to_string(),
                    timestamp: "2023-01-01 00:00:00".to_string(),
                }
            ],
            announcement_url: "https://example.com/announcement".to_string(),
            total_commitments_found: 1,
            error_message: None,
        };

        dal.update_round_commitments("test_round", collection_result).unwrap();

        // Verify update
        let updated_round = dal.get_round("test_round").unwrap();
        assert!(updated_round.has_commitment_collection());
        assert_eq!(updated_round.collected_commitments.unwrap().total_commitments_found, 1);
    }

    #[test]
    fn test_backup_restore() {
        let (dal, _temp_dir) = create_test_dal();

        // Create some test data
        let round_data = RoundData::new(
            "test_round".to_string(),
            "Test Round".to_string(),
            "Test description".to_string(),
            "test.jpg".to_string(),
        );
        dal.update_round("test_round", round_data).unwrap();

        // Create backup
        let backup_path = dal.create_backup().unwrap();
        assert!(Path::new(&backup_path).exists());

        // Modify original data
        let modified_round = RoundData::new(
            "modified_round".to_string(),
            "Modified Round".to_string(),
            "Modified description".to_string(),
            "modified.jpg".to_string(),
        );
        dal.update_round("modified_round", modified_round).unwrap();
        dal.delete_round("test_round").unwrap();

        // Restore from backup
        dal.restore_from_backup(&backup_path).unwrap();

        // Verify restoration
        assert!(dal.round_exists("test_round").unwrap());
        let restored_round = dal.get_round("test_round").unwrap();
        assert_eq!(restored_round.title, "Test Round");
    }

    #[test]
    fn test_data_validation() {
        let (dal, _temp_dir) = create_test_dal();

        // Create test data with inconsistencies
        let mut round_data = RoundData::new(
            "test_round".to_string(),
            "Test Round".to_string(),
            "Test description".to_string(),
            "nonexistent.jpg".to_string(), // This file doesn't exist
        );

        // Add Twitter data with inconsistent counts
        let twitter_data = TwitterReplyData {
            original_tweet_url: "https://example.com/tweet".to_string(),
            total_replies_found: 5, // Inconsistent with actual replies count
            replies: vec![
                TwitterReply {
                    url: "https://example.com/reply".to_string(),
                    author: "@testuser".to_string(),
                    text_preview: "Test reply".to_string(),
                    was_spam_flagged: false,
                }
            ], // Only 1 reply but count says 5
        };
        round_data.set_twitter_replies(twitter_data);

        dal.update_round("test_round", round_data).unwrap();

        // Run validation
        let issues = dal.validate_data_consistency().unwrap();
        assert!(!issues.is_empty());
        
        // Should find the image file issue and Twitter data inconsistency
        let issues_text = issues.join(" ");
        assert!(issues_text.contains("target image not found"));
        assert!(issues_text.contains("Twitter data inconsistency"));
    }

    #[test]
    fn test_error_handling() {
        let (dal, _temp_dir) = create_test_dal();

        // Test round not found
        let result = dal.get_round("nonexistent");
        assert!(matches!(result, Err(crate::error::RealMirError::DataAccess(_))));

        // Test delete nonexistent round
        let result = dal.delete_round("nonexistent");
        assert!(matches!(result, Err(crate::error::RealMirError::DataAccess(_))));

        // Test restore from nonexistent backup
        let result = dal.restore_from_backup("nonexistent_backup.json");
        assert!(matches!(result, Err(crate::error::RealMirError::DataAccess(_))));
    }

    #[test]
    fn test_enhanced_queries() {
        let (dal, _temp_dir) = create_test_dal();

        // Create rounds with different characteristics
        let mut round1 = RoundData::new("round1".to_string(), "Round 1".to_string(), "Description".to_string(), "test.jpg".to_string());
        let mut round2 = RoundData::new("round2".to_string(), "Round 2".to_string(), "Description".to_string(), "test.jpg".to_string());
        let round3 = RoundData::new("round3".to_string(), "Round 3".to_string(), "Description".to_string(), "test.jpg".to_string());

        // Add Twitter data to round1
        let twitter_data = TwitterReplyData {
            original_tweet_url: "https://example.com/tweet".to_string(),
            total_replies_found: 0,
            replies: vec![],
        };
        round1.set_twitter_replies(twitter_data);

        // Add commitment collection to round2
        let collection_result = CommitmentCollectionResult {
            success: true,
            commitments: vec![],
            announcement_url: "https://example.com/announcement".to_string(),
            total_commitments_found: 0,
            error_message: None,
        };
        round2.set_commitment_collection(collection_result);

        // Save all rounds
        dal.update_round("round1", round1).unwrap();
        dal.update_round("round2", round2).unwrap();
        dal.update_round("round3", round3).unwrap();

        // Test enhanced queries
        let rounds_with_twitter = dal.get_rounds_with_twitter_data().unwrap();
        assert_eq!(rounds_with_twitter.len(), 1);
        assert!(rounds_with_twitter.contains(&"round1".to_string()));

        let rounds_with_commitments = dal.get_rounds_with_commitments().unwrap();
        assert_eq!(rounds_with_commitments.len(), 1);
        assert!(rounds_with_commitments.contains(&"round2".to_string()));

        let all_rounds = dal.get_all_round_ids().unwrap();
        assert_eq!(all_rounds.len(), 3);
    }
}