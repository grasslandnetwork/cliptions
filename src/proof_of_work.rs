//! Proof of Work module for spam prevention
//! 
//! This module provides lightweight hashcash-style proof of work to prevent
//! spam in the commitment/reveal system. Both miners and validators can be
//! required to provide proof of work to increase the computational cost
//! of participating in the system.

use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::error::{Result, ProofOfWorkError};

/// Proof of work challenge and solution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfWork {
    /// The challenge data that was hashed
    pub challenge: String,
    /// The nonce that satisfies the difficulty requirement
    pub nonce: u64,
    /// The resulting hash that meets the difficulty
    pub hash: String,
    /// The difficulty level (number of leading zeros required)
    pub difficulty: u8,
    /// Timestamp when the proof was generated
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ProofOfWork {
    /// Create a new proof of work instance
    pub fn new(challenge: String, nonce: u64, hash: String, difficulty: u8) -> Self {
        Self {
            challenge,
            nonce,
            hash,
            difficulty,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Verify that this proof of work is valid
    pub fn is_valid(&self) -> bool {
        // Recalculate the hash
        let calculated_hash = Self::calculate_hash(&self.challenge, self.nonce);
        
        // Check if hash matches
        if calculated_hash != self.hash {
            return false;
        }
        
        // Check if hash meets difficulty requirement
        Self::meets_difficulty(&self.hash, self.difficulty)
    }
    
    /// Calculate SHA-256 hash for challenge and nonce
    fn calculate_hash(challenge: &str, nonce: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(challenge.as_bytes());
        hasher.update(nonce.to_le_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Check if a hash meets the difficulty requirement (leading zeros)
    fn meets_difficulty(hash: &str, difficulty: u8) -> bool {
        if difficulty == 0 {
            return true;
        }
        
        let required_zeros = difficulty as usize;
        hash.chars().take(required_zeros).all(|c| c == '0')
    }
}

/// Proof of work generator and verifier
#[derive(Debug, Clone)]
pub struct ProofOfWorkSystem {
    /// Default difficulty level for new challenges
    default_difficulty: u8,
    /// Maximum time to spend on proof generation
    max_generation_time: Duration,
}

impl ProofOfWorkSystem {
    /// Create a new proof of work system with default settings
    pub fn new() -> Self {
        Self {
            default_difficulty: 4, // Require 4 leading zeros by default
            max_generation_time: Duration::from_secs(30), // 30 second timeout
        }
    }
    
    /// Create a system with custom difficulty
    pub fn with_difficulty(difficulty: u8) -> Result<Self> {
        if difficulty > 20 {
            return Err(ProofOfWorkError::DifficultyTooHigh.into());
        }
        
        Ok(Self {
            default_difficulty: difficulty,
            max_generation_time: Duration::from_secs(30),
        })
    }
    
    /// Create a system with custom settings
    pub fn with_settings(difficulty: u8, max_time: Duration) -> Result<Self> {
        if difficulty > 20 {
            return Err(ProofOfWorkError::DifficultyTooHigh.into());
        }
        
        Ok(Self {
            default_difficulty: difficulty,
            max_generation_time: max_time,
        })
    }
    
    /// Generate proof of work for a given challenge
    /// 
    /// # Arguments
    /// * `challenge` - The challenge string to prove work for
    /// * `difficulty` - Optional custom difficulty (uses default if None)
    /// 
    /// # Returns
    /// ProofOfWork instance with valid nonce, or error if timeout
    pub fn generate_proof(&self, challenge: &str, difficulty: Option<u8>) -> Result<ProofOfWork> {
        let target_difficulty = difficulty.unwrap_or(self.default_difficulty);
        let start_time = Instant::now();
        
        let mut nonce = 0u64;
        
        loop {
            // Check timeout
            if start_time.elapsed() > self.max_generation_time {
                return Err(ProofOfWorkError::GenerationTimeout.into());
            }
            
            // Calculate hash for current nonce
            let hash = ProofOfWork::calculate_hash(challenge, nonce);
            
            // Check if this hash meets the difficulty requirement
            if ProofOfWork::meets_difficulty(&hash, target_difficulty) {
                return Ok(ProofOfWork::new(
                    challenge.to_string(),
                    nonce,
                    hash,
                    target_difficulty,
                ));
            }
            
            nonce += 1;
            
            // Prevent overflow (very unlikely but good practice)
            if nonce == u64::MAX {
                return Err(ProofOfWorkError::NonceOverflow.into());
            }
        }
    }
    
    /// Verify a proof of work
    pub fn verify_proof(&self, proof: &ProofOfWork) -> bool {
        proof.is_valid()
    }
    
    /// Generate proof of work for commitment phase
    /// Combines prediction text, salt, and round ID as challenge
    pub fn generate_commitment_proof(
        &self,
        prediction: &str,
        salt: &str,
        round_id: &str,
        difficulty: Option<u8>,
    ) -> Result<ProofOfWork> {
        let challenge = format!("commit:{}:{}:{}", round_id, prediction, salt);
        self.generate_proof(&challenge, difficulty)
    }
    
    /// Generate proof of work for reveal phase
    /// Includes the commitment hash in the challenge
    pub fn generate_reveal_proof(
        &self,
        prediction: &str,
        salt: &str,
        commitment_hash: &str,
        round_id: &str,
        difficulty: Option<u8>,
    ) -> Result<ProofOfWork> {
        let challenge = format!("reveal:{}:{}:{}:{}", round_id, commitment_hash, prediction, salt);
        self.generate_proof(&challenge, difficulty)
    }
    
    /// Estimate time to generate proof for given difficulty
    /// Returns an estimate based on average hash rate
    pub fn estimate_generation_time(&self, difficulty: u8) -> Duration {
        // Very rough estimate: each difficulty level increases time by ~16x
        // This is highly dependent on hardware, so take with grain of salt
        let base_time_ms = 10u64; // 10ms for difficulty 1
        let multiplier = 16u64.pow(difficulty as u32);
        let estimated_ms = base_time_ms * multiplier;
        
        Duration::from_millis(estimated_ms.min(300_000)) // Cap at 5 minutes
    }
    
    /// Get the default difficulty level
    pub fn default_difficulty(&self) -> u8 {
        self.default_difficulty
    }
    
    /// Set the default difficulty level
    pub fn set_default_difficulty(&mut self, difficulty: u8) -> Result<()> {
        if difficulty > 20 {
            return Err(ProofOfWorkError::DifficultyTooHigh.into());
        }
        self.default_difficulty = difficulty;
        Ok(())
    }
}

impl Default for ProofOfWorkSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Proof of work statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfWorkStats {
    /// Average generation time for recent proofs
    pub avg_generation_time: Duration,
    /// Number of proofs generated
    pub proofs_generated: u64,
    /// Current difficulty level
    pub current_difficulty: u8,
    /// Hash rate (hashes per second)
    pub hash_rate: f64,
}

/// Proof of work manager for tracking and adjusting difficulty
#[derive(Debug)]
pub struct ProofOfWorkManager {
    system: ProofOfWorkSystem,
    stats: ProofOfWorkStats,
    recent_times: Vec<Duration>,
    target_time: Duration,
}

impl ProofOfWorkManager {
    /// Create a new manager with target generation time
    pub fn new(target_time: Duration) -> Self {
        Self {
            system: ProofOfWorkSystem::new(),
            stats: ProofOfWorkStats {
                avg_generation_time: Duration::from_secs(0),
                proofs_generated: 0,
                current_difficulty: 4,
                hash_rate: 0.0,
            },
            recent_times: Vec::new(),
            target_time,
        }
    }
    
    /// Generate proof and update statistics
    pub fn generate_tracked_proof(&mut self, challenge: &str) -> Result<ProofOfWork> {
        let start_time = Instant::now();
        let proof = self.system.generate_proof(challenge, None)?;
        let generation_time = start_time.elapsed();
        
        // Update statistics
        self.recent_times.push(generation_time);
        if self.recent_times.len() > 10 {
            self.recent_times.remove(0); // Keep only recent 10 times
        }
        
        self.stats.proofs_generated += 1;
        self.update_stats(generation_time);
        
        Ok(proof)
    }
    
    /// Update internal statistics
    fn update_stats(&mut self, _last_time: Duration) {
        if !self.recent_times.is_empty() {
            let total_time: Duration = self.recent_times.iter().sum();
            self.stats.avg_generation_time = total_time / self.recent_times.len() as u32;
            
            // Calculate hash rate (very rough estimate)
            let avg_seconds = self.stats.avg_generation_time.as_secs_f64();
            if avg_seconds > 0.0 {
                // Estimate based on difficulty and average time
                let estimated_hashes = 2u64.pow(self.stats.current_difficulty as u32 * 4) as f64;
                self.stats.hash_rate = estimated_hashes / avg_seconds;
            }
        }
    }
    
    /// Adjust difficulty based on recent generation times
    pub fn adjust_difficulty(&mut self) -> Result<()> {
        if self.recent_times.len() < 5 {
            return Ok(());
        }
        
        let avg_time = self.stats.avg_generation_time;
        
        // If average time is too low, increase difficulty
        if avg_time < self.target_time / 2 && self.stats.current_difficulty < 15 {
            self.stats.current_difficulty += 1;
            self.system.set_default_difficulty(self.stats.current_difficulty)?;
        }
        // If average time is too high, decrease difficulty
        else if avg_time > self.target_time * 2 && self.stats.current_difficulty > 1 {
            self.stats.current_difficulty -= 1;
            self.system.set_default_difficulty(self.stats.current_difficulty)?;
        }
        
        Ok(())
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> &ProofOfWorkStats {
        &self.stats
    }
    
    /// Get the underlying proof of work system
    pub fn system(&self) -> &ProofOfWorkSystem {
        &self.system
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proof_generation_and_verification() {
        let pow_system = ProofOfWorkSystem::with_difficulty(2).unwrap(); // Easy difficulty for testing
        let challenge = "test_challenge_123";
        
        let proof = pow_system.generate_proof(challenge, None).unwrap();
        
        // Verify the proof
        assert!(pow_system.verify_proof(&proof));
        assert!(proof.is_valid());
        
        // Check that difficulty is met
        assert!(proof.hash.starts_with("00")); // 2 leading zeros
    }
    
    #[test]
    fn test_commitment_reveal_proofs() {
        let pow_system = ProofOfWorkSystem::with_difficulty(1).unwrap();
        let prediction = "The cat will be orange";
        let salt = "random_salt_456";
        let round_id = "round_001";
        let commitment_hash = "abc123def456";
        
        // Generate commitment proof
        let commit_proof = pow_system
            .generate_commitment_proof(prediction, salt, round_id, None)
            .unwrap();
        
        assert!(commit_proof.is_valid());
        assert!(commit_proof.challenge.contains("commit:"));
        assert!(commit_proof.challenge.contains(round_id));
        
        // Generate reveal proof
        let reveal_proof = pow_system
            .generate_reveal_proof(prediction, salt, commitment_hash, round_id, None)
            .unwrap();
        
        assert!(reveal_proof.is_valid());
        assert!(reveal_proof.challenge.contains("reveal:"));
        assert!(reveal_proof.challenge.contains(commitment_hash));
    }
    
    #[test]
    fn test_invalid_proof() {
        let pow_system = ProofOfWorkSystem::new();
        
        // Create an invalid proof
        let invalid_proof = ProofOfWork::new(
            "challenge".to_string(),
            12345,
            "invalid_hash".to_string(),
            4,
        );
        
        assert!(!pow_system.verify_proof(&invalid_proof));
        assert!(!invalid_proof.is_valid());
    }
    
    #[test]
    fn test_difficulty_levels() {
        let pow_system = ProofOfWorkSystem::with_difficulty(3).unwrap();
        let challenge = "difficulty_test";
        
        let proof = pow_system.generate_proof(challenge, None).unwrap();
        
        // Should have 3 leading zeros
        assert!(proof.hash.starts_with("000"));
        assert_eq!(proof.difficulty, 3);
    }
    
    #[test]
    fn test_difficulty_too_high() {
        let result = ProofOfWorkSystem::with_difficulty(25);
        assert!(matches!(result, Err(crate::error::RealMirError::ProofOfWork(ProofOfWorkError::DifficultyTooHigh))));
    }
    
    #[test]
    fn test_proof_of_work_manager() {
        let mut manager = ProofOfWorkManager::new(Duration::from_millis(100));
        
        let proof = manager.generate_tracked_proof("test_challenge").unwrap();
        assert!(proof.is_valid());
        
        let stats = manager.get_stats();
        assert_eq!(stats.proofs_generated, 1);
        assert!(stats.avg_generation_time.as_millis() > 0);
    }
    
    #[test]
    fn test_hash_meets_difficulty() {
        assert!(ProofOfWork::meets_difficulty("0000abc123", 4));
        assert!(ProofOfWork::meets_difficulty("000abc123", 3));
        assert!(ProofOfWork::meets_difficulty("00abc123", 2));
        assert!(ProofOfWork::meets_difficulty("0abc123", 1));
        assert!(ProofOfWork::meets_difficulty("abc123", 0));
        
        assert!(!ProofOfWork::meets_difficulty("abc123", 1));
        assert!(!ProofOfWork::meets_difficulty("0abc123", 2));
        assert!(!ProofOfWork::meets_difficulty("00abc123", 3));
    }
}