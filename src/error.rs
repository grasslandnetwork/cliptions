//! Error handling for RealMir core functionality
//! 
//! This module provides comprehensive error handling using the `thiserror` crate
//! for ergonomic error definitions and proper error propagation.

use thiserror::Error;

/// Result type alias for RealMir operations
pub type Result<T> = std::result::Result<T, RealMirError>;

/// Main error type for RealMir operations
#[derive(Error, Debug)]
pub enum RealMirError {
    #[error("Commitment error: {0}")]
    Commitment(#[from] CommitmentError),
    
    #[error("Scoring error: {0}")]
    Scoring(#[from] ScoringError),
    
    #[error("Embedding error: {0}")]
    Embedding(#[from] EmbeddingError),
    
    #[error("Round processing error: {0}")]
    Round(#[from] RoundError),
    
    #[error("Steganography error: {0}")]
    Steganography(#[from] SteganographyError),
    
    #[error("Proof of work error: {0}")]
    ProofOfWork(#[from] ProofOfWorkError),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Commitment-related errors
#[derive(Error, Debug)]
pub enum CommitmentError {
    #[error("Message cannot be empty")]
    EmptyMessage,
    
    #[error("Salt is required for generating commitments")]
    EmptySalt,
    
    #[error("Invalid commitment format")]
    InvalidFormat,
    
    #[error("Commitment verification failed")]
    VerificationFailed,
    
    #[error("Missing commitment data")]
    MissingData,
}

/// Scoring-related errors
#[derive(Error, Debug)]
pub enum ScoringError {
    #[error("Feature vectors must have the same length")]
    DimensionMismatch,
    
    #[error("Invalid similarity score: {score}")]
    InvalidScore { score: f64 },
    
    #[error("Empty guess list")]
    EmptyGuesses,
    
    #[error("Invalid prize pool: {amount}")]
    InvalidPrizePool { amount: f64 },
    
    #[error("Operation not supported for this strategy")]
    UnsupportedOperation,
}

/// Embedding-related errors
#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("Failed to load embedding model")]
    ModelLoadFailed,
    
    #[error("Invalid embedding dimensions")]
    InvalidDimensions,
    
    #[error("Text tokenization failed")]
    TokenizationFailed,
    
    #[error("Image processing failed")]
    ImageProcessingFailed,
    
    #[error("Invalid tensor shape")]
    InvalidTensorShape,
    
    #[error("Unsupported format")]
    UnsupportedFormat,
}

/// Round processing errors
#[derive(Error, Debug)]
pub enum RoundError {
    #[error("Round {round_id} not found")]
    RoundNotFound { round_id: String },
    
    #[error("No participants in round {round_id}")]
    NoParticipants { round_id: String },
    
    #[error("Target image not found: {path}")]
    TargetImageNotFound { path: String },
    
    #[error("Round data file not found: {path}")]
    DataFileNotFound { path: String },
    
    #[error("Round already processed")]
    AlreadyProcessed,
}

/// Validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Guess is empty or invalid")]
    InvalidGuess,
    
    #[error("Guess too long: {length} characters")]
    GuessTooLong { length: usize },
    
    #[error("Username is required")]
    MissingUsername,
    
    #[error("Invalid participant data")]
    InvalidParticipant,
}

/// Steganography-related errors
#[derive(Error, Debug)]
pub enum SteganographyError {
    #[error("Invalid image format or corrupted image")]
    InvalidImage,
    
    #[error("No embedded data found in image")]
    NoEmbeddedData,
    
    #[error("Embedded data is corrupted or invalid")]
    CorruptedData,
    
    #[error("Image has insufficient capacity for the data")]
    InsufficientCapacity,
    
    #[error("Failed to encode data")]
    EncodingFailed,
    
    #[error("Failed to save image")]
    SaveFailed,
    
    #[error("Invalid steganography configuration")]
    InvalidConfiguration,
}

/// Proof of work related errors
#[derive(Error, Debug)]
pub enum ProofOfWorkError {
    #[error("Difficulty level too high (maximum is 20)")]
    DifficultyTooHigh,
    
    #[error("Proof generation timed out")]
    GenerationTimeout,
    
    #[error("Nonce overflow occurred")]
    NonceOverflow,
    
    #[error("Invalid proof of work")]
    InvalidProof,
}