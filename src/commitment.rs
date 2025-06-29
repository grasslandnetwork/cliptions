//! Cryptographic commitment system for RealMir
//! 
//! This module provides secure commitment generation and verification using SHA-256 hashing.
//! The commitment scheme ensures that participants can commit to their guesses without revealing
//! them until the reveal phase, preventing gaming of the system.

use sha2::{Digest, Sha256};
use crate::error::{CommitmentError, Result};
use crate::proof_of_work::{ProofOfWork, ProofOfWorkSystem};
use serde::{Deserialize, Serialize};



/// Commitment generator for creating cryptographic commitments
#[derive(Debug, Clone)]
pub struct CommitmentGenerator {
    salt_length: usize,
}

impl CommitmentGenerator {
    /// Create a new commitment generator with default salt length
    pub fn new() -> Self {
        Self {
            salt_length: 32,
        }
    }
    
    /// Create a commitment generator with custom salt length
    pub fn with_salt_length(salt_length: usize) -> Self {
        Self { salt_length }
    }
    
    /// Generate a commitment hash from a message and salt
    /// 
    /// This matches the Python implementation: hash(message + salt)
    /// 
    /// # Arguments
    /// * `message` - The plaintext message to commit to
    /// * `salt` - A random salt value to prevent brute force attacks
    /// 
    /// # Returns
    /// The hex-encoded SHA-256 hash of the message concatenated with the salt
    /// 
    /// # Errors
    /// Returns `CommitmentError::EmptySalt` if the salt is empty
    pub fn generate(&self, message: &str, salt: &str) -> Result<String> {
        if message.trim().is_empty() {
            return Err(CommitmentError::EmptyMessage.into());
        }
        if salt.is_empty() {
            return Err(CommitmentError::EmptySalt.into());
        }
        
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        hasher.update(salt.as_bytes());
        let result = hasher.finalize();
        
        Ok(format!("{:x}", result))
    }
    
    /// Generate a random salt of the specified length
    /// 
    /// # Returns
    /// A random hex-encoded salt string
    pub fn generate_salt(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..self.salt_length).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }
}

impl Default for CommitmentGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Commitment verifier for validating commitments
#[derive(Debug, Clone)]
pub struct CommitmentVerifier {
    generator: CommitmentGenerator,
}

impl CommitmentVerifier {
    /// Create a new commitment verifier
    pub fn new() -> Self {
        Self {
            generator: CommitmentGenerator::new(),
        }
    }
    
    /// Verify that a commitment matches the provided message and salt
    /// 
    /// # Arguments
    /// * `message` - The plaintext message
    /// * `salt` - The salt used in the original commitment
    /// * `commitment` - The commitment hash to verify against
    /// 
    /// # Returns
    /// `true` if the commitment is valid, `false` otherwise
    pub fn verify(&self, message: &str, salt: &str, commitment: &str) -> bool {
        match self.generator.generate(message, salt) {
            Ok(calculated_commitment) => calculated_commitment == commitment,
            Err(_) => false,
        }
    }
    
    /// Batch verify multiple commitments
    /// 
    /// # Arguments
    /// * `commitments` - A slice of tuples containing (message, salt, commitment)
    /// 
    /// # Returns
    /// A vector of boolean values indicating whether each commitment is valid
    pub fn verify_batch(&self, commitments: &[(&str, &str, &str)]) -> Vec<bool> {
        commitments
            .iter()
            .map(|(message, salt, commitment)| self.verify(message, salt, commitment))
            .collect()
    }
    
    /// Parallel batch verification for better performance
    /// 
    /// # Arguments
    /// * `commitments` - A slice of tuples containing (message, salt, commitment)
    /// 
    /// # Returns
    /// A vector of boolean values indicating whether each commitment is valid
    pub fn verify_batch_parallel(&self, commitments: &[(&str, &str, &str)]) -> Vec<bool> {
        use rayon::prelude::*;
        
        commitments
            .par_iter()
            .map(|(message, salt, commitment)| self.verify(message, salt, commitment))
            .collect()
    }
}

impl Default for CommitmentVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced commitment that includes CLIP vectors and proof of work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedCommitment {
    /// The basic commitment hash
    pub commitment_hash: String,
    /// The CLIP vector commitment (hash of the vector)
    pub vector_commitment: String,
    /// Proof of work for spam prevention
    pub proof_of_work: ProofOfWork,
    /// Round ID for this commitment
    pub round_id: String,
    /// Timestamp when commitment was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl EnhancedCommitment {
    /// Create a new enhanced commitment
    pub fn new(
        commitment_hash: String,
        vector_commitment: String,
        proof_of_work: ProofOfWork,
        round_id: String,
    ) -> Self {
        Self {
            commitment_hash,
            vector_commitment,
            proof_of_work,
            round_id,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Verify that all components of this commitment are valid
    pub fn is_valid(&self) -> bool {
        // Verify proof of work
        if !self.proof_of_work.is_valid() {
            return false;
        }
        
        // Verify that proof of work challenge is for the commitment phase and includes round ID
        if !self.proof_of_work.challenge.starts_with("commit:") {
            return false;
        }
        
        if !self.proof_of_work.challenge.contains(&self.round_id) {
            return false;
        }
        
        // Basic format validation
        if self.commitment_hash.len() != 64 || self.vector_commitment.len() != 64 {
            return false;
        }
        
        true
    }
}

/// Enhanced commitment generator that creates commitments with CLIP vectors and PoW
#[derive(Debug, Clone)]
pub struct EnhancedCommitmentGenerator {
    basic_generator: CommitmentGenerator,
    pow_system: ProofOfWorkSystem,
}

impl EnhancedCommitmentGenerator {
    /// Create a new enhanced commitment generator
    pub fn new() -> Self {
        Self {
            basic_generator: CommitmentGenerator::new(),
            pow_system: ProofOfWorkSystem::new(),
        }
    }
    
    /// Create generator with custom proof of work difficulty
    pub fn with_pow_difficulty(difficulty: u8) -> Result<Self> {
        Ok(Self {
            basic_generator: CommitmentGenerator::new(),
            pow_system: ProofOfWorkSystem::with_difficulty(difficulty)?,
        })
    }
    
    /// Generate an enhanced commitment including CLIP vector and proof of work
    /// 
    /// This implements the system described in the Slack thread:
    /// hash = sha256(plaintext_prediction || salt || clip_vector)
    /// 
    /// # Arguments
    /// * `prediction` - The plaintext prediction
    /// * `salt` - Random salt for the commitment
    /// * `clip_vector` - The CLIP vector for the prediction
    /// * `round_id` - ID of the round this commitment is for
    /// 
    /// # Returns
    /// EnhancedCommitment with all components
    pub fn generate_enhanced(
        &self,
        prediction: &str,
        salt: &str,
        clip_vector: &[f64],
        round_id: &str,
    ) -> Result<EnhancedCommitment> {
        if prediction.trim().is_empty() {
            return Err(CommitmentError::EmptyMessage.into());
        }
        if salt.is_empty() {
            return Err(CommitmentError::EmptySalt.into());
        }
        
        // Create the commitment hash including the CLIP vector
        let commitment_hash = self.generate_with_vector(prediction, salt, clip_vector)?;
        
        // Create a separate hash just for the CLIP vector (for verification)
        let vector_commitment = self.hash_vector(clip_vector)?;
        
        // Generate proof of work
        let proof_of_work = self.pow_system.generate_commitment_proof(
            prediction,
            salt,
            round_id,
            None, // Use default difficulty
        )?;
        
        Ok(EnhancedCommitment::new(
            commitment_hash,
            vector_commitment,
            proof_of_work,
            round_id.to_string(),
        ))
    }
    
    /// Generate commitment hash including CLIP vector
    /// Implements: hash = sha256(plaintext_prediction || salt || clip_vector)
    pub fn generate_with_vector(
        &self,
        prediction: &str,
        salt: &str,
        clip_vector: &[f64],
    ) -> Result<String> {
        let mut hasher = Sha256::new();
        
        // Add prediction text
        hasher.update(prediction.as_bytes());
        
        // Add salt
        hasher.update(salt.as_bytes());
        
        // Add CLIP vector as bytes
        for &value in clip_vector {
            hasher.update(value.to_le_bytes());
        }
        
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }
    
    /// Create a hash of just the CLIP vector for separate verification
    pub fn hash_vector(&self, clip_vector: &[f64]) -> Result<String> {
        let mut hasher = Sha256::new();
        
        for &value in clip_vector {
            hasher.update(value.to_le_bytes());
        }
        
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }
    
    /// Generate a random salt (delegated to basic generator)
    pub fn generate_salt(&self) -> String {
        self.basic_generator.generate_salt()
    }
}

impl Default for EnhancedCommitmentGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced commitment verifier for validating commitments with CLIP vectors
#[derive(Debug, Clone)]
pub struct EnhancedCommitmentVerifier {
    basic_verifier: CommitmentVerifier,
    generator: EnhancedCommitmentGenerator,
}

impl EnhancedCommitmentVerifier {
    /// Create a new enhanced commitment verifier
    pub fn new() -> Self {
        Self {
            basic_verifier: CommitmentVerifier::new(),
            generator: EnhancedCommitmentGenerator::new(),
        }
    }
    
    /// Verify an enhanced commitment against the reveal data
    /// 
    /// # Arguments
    /// * `commitment` - The original commitment
    /// * `prediction` - The revealed prediction text
    /// * `salt` - The revealed salt
    /// * `clip_vector` - The revealed CLIP vector
    /// 
    /// # Returns
    /// True if the commitment is valid
    pub fn verify_enhanced(
        &self,
        commitment: &EnhancedCommitment,
        prediction: &str,
        salt: &str,
        clip_vector: &[f64],
    ) -> bool {
        // First verify the commitment structure itself
        if !commitment.is_valid() {
            return false;
        }
        
        // Verify the main commitment hash
        match self.generator.generate_with_vector(prediction, salt, clip_vector) {
            Ok(calculated_hash) => {
                if calculated_hash != commitment.commitment_hash {
                    return false;
                }
            }
            Err(_) => return false,
        }
        
        // Verify the vector commitment
        match self.generator.hash_vector(clip_vector) {
            Ok(calculated_vector_hash) => {
                if calculated_vector_hash != commitment.vector_commitment {
                    return false;
                }
            }
            Err(_) => return false,
        }
        
        // All verifications passed
        true
    }
    
    /// Verify just the CLIP vector against its commitment
    pub fn verify_vector(&self, vector_commitment: &str, clip_vector: &[f64]) -> bool {
        match self.generator.hash_vector(clip_vector) {
            Ok(calculated_hash) => calculated_hash == vector_commitment,
            Err(_) => false,
        }
    }
    
    /// Batch verify multiple enhanced commitments
    pub fn verify_enhanced_batch(
        &self,
        commitments: &[(EnhancedCommitment, String, String, Vec<f64>)], // (commitment, prediction, salt, vector)
    ) -> Vec<bool> {
        commitments
            .iter()
            .map(|(commitment, prediction, salt, vector)| {
                self.verify_enhanced(commitment, prediction, salt, vector)
            })
            .collect()
    }
}

impl Default for EnhancedCommitmentVerifier {
    fn default() -> Self {
        Self::new()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_commitment_generation() {
        let generator = CommitmentGenerator::new();
        let message = "Hello, World!";
        let salt = "random_salt_123";
        
        let commitment = generator.generate(message, salt).unwrap();
        
        // Should be 64 characters (32 bytes in hex)
        assert_eq!(commitment.len(), 64);
        
        // Should be deterministic
        let commitment2 = generator.generate(message, salt).unwrap();
        assert_eq!(commitment, commitment2);
    }
    
    #[test]
    fn test_commitment_verification() {
        let generator = CommitmentGenerator::new();
        let verifier = CommitmentVerifier::new();
        let message = "Test message";
        let salt = "test_salt";
        
        let commitment = generator.generate(message, salt).unwrap();
        
        // Valid commitment should verify
        assert!(verifier.verify(message, salt, &commitment));
        
        // Invalid commitment should not verify
        assert!(!verifier.verify("wrong message", salt, &commitment));
        assert!(!verifier.verify(message, "wrong salt", &commitment));
        assert!(!verifier.verify(message, salt, "wrong_commitment"));
    }
    
    #[test]
    fn test_empty_salt() {
        let generator = CommitmentGenerator::new();
        let result = generator.generate("message", "");
        
        assert!(matches!(result, Err(crate::error::RealMirError::Commitment(CommitmentError::EmptySalt))));
    }
    
    #[test]
    fn test_empty_message() {
        let generator = CommitmentGenerator::new();
        
        // Test completely empty message
        let result = generator.generate("", "salt");
        assert!(matches!(result, Err(crate::error::RealMirError::Commitment(CommitmentError::EmptyMessage))));
        
        // Test whitespace-only message
        let result = generator.generate("   ", "salt");
        assert!(matches!(result, Err(crate::error::RealMirError::Commitment(CommitmentError::EmptyMessage))));
        
        // Test tab and newline only
        let result = generator.generate("\t\n  ", "salt");
        assert!(matches!(result, Err(crate::error::RealMirError::Commitment(CommitmentError::EmptyMessage))));
    }
    
    #[test]
    fn test_salt_generation() {
        let generator = CommitmentGenerator::new();
        let salt1 = generator.generate_salt();
        let salt2 = generator.generate_salt();
        
        // Salts should be different
        assert_ne!(salt1, salt2);
        
        // Salts should be hex-encoded (64 characters for 32 bytes)
        assert_eq!(salt1.len(), 64);
        assert_eq!(salt2.len(), 64);
        
        // Should be valid hex
        assert!(salt1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(salt2.chars().all(|c| c.is_ascii_hexdigit()));
    }
    
    #[test]
    fn test_batch_verification() {
        let generator = CommitmentGenerator::new();
        let verifier = CommitmentVerifier::new();
        
        let commitments = vec![
            ("message1", "salt1", generator.generate("message1", "salt1").unwrap()),
            ("message2", "salt2", generator.generate("message2", "salt2").unwrap()),
            ("message3", "salt3", "invalid_commitment".to_string()),
        ];
        
        let commitment_refs: Vec<(&str, &str, &str)> = commitments
            .iter()
            .map(|(m, s, c)| (*m, *s, c.as_str()))
            .collect();
        
        let results = verifier.verify_batch(&commitment_refs);
        
        assert_eq!(results, vec![true, true, false]);
    }
    
    #[test]
    fn test_parallel_batch_verification() {
        let generator = CommitmentGenerator::new();
        let verifier = CommitmentVerifier::new();
        
        // Create a larger batch for parallel testing
        let mut commitments = Vec::new();
        for i in 0..100 {
            let message = format!("message{}", i);
            let salt = format!("salt{}", i);
            let commitment = generator.generate(&message, &salt).unwrap();
            commitments.push((message, salt, commitment));
        }
        
        let commitment_refs: Vec<(&str, &str, &str)> = commitments
            .iter()
            .map(|(m, s, c)| (m.as_str(), s.as_str(), c.as_str()))
            .collect();
        
        let sequential_results = verifier.verify_batch(&commitment_refs);
        let parallel_results = verifier.verify_batch_parallel(&commitment_refs);
        
        // Results should be identical
        assert_eq!(sequential_results, parallel_results);
        
        // All should be valid
        assert!(sequential_results.iter().all(|&r| r));
    }
    
    #[test]
    fn test_rust_core_functionality() {
        // Test that our core implementation works correctly
        let generator = CommitmentGenerator::new();
        let verifier = CommitmentVerifier::new();
        let message = "test message";
        let salt = "test_salt";
        
        let commitment = generator.generate(message, salt).unwrap();
        assert!(verifier.verify(message, salt, &commitment));
        
        // Test with different inputs
        assert!(!verifier.verify("different message", salt, &commitment));
        assert!(!verifier.verify(message, "different_salt", &commitment));
    }
    
    // Enhanced commitment tests
    
    fn create_test_clip_vector() -> Vec<f64> {
        (0..512).map(|i| (i as f64) / 512.0).collect()
    }
    
    #[test]
    fn test_enhanced_commitment_generation() {
        let generator = EnhancedCommitmentGenerator::with_pow_difficulty(1).unwrap(); // Easy PoW for testing
        let prediction = "The cat will be orange";
        let salt = "test_salt_456";
        let clip_vector = create_test_clip_vector();
        let round_id = "test_round_001";
        
        let commitment = generator
            .generate_enhanced(prediction, &salt, &clip_vector, round_id)
            .unwrap();
        
        // Verify commitment structure
        assert!(commitment.is_valid());
        assert_eq!(commitment.round_id, round_id);
        assert_eq!(commitment.commitment_hash.len(), 64);
        assert_eq!(commitment.vector_commitment.len(), 64);
        assert!(commitment.proof_of_work.is_valid());
    }
    
    #[test]
    fn test_enhanced_commitment_verification() {
        let generator = EnhancedCommitmentGenerator::with_pow_difficulty(1).unwrap();
        let verifier = EnhancedCommitmentVerifier::new();
        
        let prediction = "The dog will be brown";
        let salt = generator.generate_salt();
        let clip_vector = create_test_clip_vector();
        let round_id = "verification_test";
        
        // Generate commitment
        let commitment = generator
            .generate_enhanced(prediction, &salt, &clip_vector, round_id)
            .unwrap();
        
        // Should verify correctly
        assert!(verifier.verify_enhanced(&commitment, prediction, &salt, &clip_vector));
        
        // Should fail with wrong prediction
        assert!(!verifier.verify_enhanced(
            &commitment,
            "Wrong prediction",
            &salt,
            &clip_vector
        ));
        
        // Should fail with wrong salt
        assert!(!verifier.verify_enhanced(
            &commitment,
            prediction,
            "wrong_salt",
            &clip_vector
        ));
        
        // Should fail with wrong vector
        let wrong_vector: Vec<f64> = (0..512).map(|i| (i as f64) / 256.0).collect();
        assert!(!verifier.verify_enhanced(&commitment, prediction, &salt, &wrong_vector));
    }
    
    #[test]
    fn test_commitment_with_vector_hash() {
        let generator = EnhancedCommitmentGenerator::new();
        let prediction = "Test prediction";
        let salt = "test_salt";
        let clip_vector = create_test_clip_vector();
        
        // Generate commitment hash with vector
        let hash1 = generator
            .generate_with_vector(prediction, salt, &clip_vector)
            .unwrap();
        
        // Should be deterministic
        let hash2 = generator
            .generate_with_vector(prediction, salt, &clip_vector)
            .unwrap();
        assert_eq!(hash1, hash2);
        
        // Different vector should produce different hash
        let different_vector: Vec<f64> = (0..512).map(|i| (i as f64) / 256.0).collect();
        let hash3 = generator
            .generate_with_vector(prediction, salt, &different_vector)
            .unwrap();
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_vector_commitment() {
        let generator = EnhancedCommitmentGenerator::new();
        let verifier = EnhancedCommitmentVerifier::new();
        let clip_vector = create_test_clip_vector();
        
        let vector_hash = generator.hash_vector(&clip_vector).unwrap();
        
        // Should verify correctly
        assert!(verifier.verify_vector(&vector_hash, &clip_vector));
        
        // Should fail with different vector
        let different_vector: Vec<f64> = (0..512).map(|i| (i as f64) / 256.0).collect();
        assert!(!verifier.verify_vector(&vector_hash, &different_vector));
    }
    
    #[test]
    fn test_enhanced_batch_verification() {
        let generator = EnhancedCommitmentGenerator::with_pow_difficulty(1).unwrap();
        let verifier = EnhancedCommitmentVerifier::new();
        
        // Create test data
        let test_cases = vec![
            ("Prediction 1", "salt1", create_test_clip_vector(), "round1"),
            ("Prediction 2", "salt2", create_test_clip_vector(), "round2"),
        ];
        
        let mut commitments_and_reveals = Vec::new();
        
        for (prediction, salt, vector, round_id) in test_cases {
            let commitment = generator
                .generate_enhanced(prediction, salt, &vector, round_id)
                .unwrap();
            
            commitments_and_reveals.push((
                commitment,
                prediction.to_string(),
                salt.to_string(),
                vector,
            ));
        }
        
        // Verify batch
        let results = verifier.verify_enhanced_batch(&commitments_and_reveals);
        
        // All should be valid
        assert_eq!(results, vec![true, true]);
    }
}