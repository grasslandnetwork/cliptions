//! # Cliptions Core
//!
//! High-performance Rust implementation of the Cliptions prediction market core functionality.
//! This library provides cryptographic commitments, scoring strategies, embedding integration,
//! and round processing capabilities.
//!
//! ## Features
//!
//! - **Commitment System**: Secure commitment generation and verification using SHA-256
//! - **Scoring Strategies**: Multiple scoring algorithms including CLIP batch processing
//! - **Embedding Integration**: Interface for CLIP and other embedding models
//! - **Round Processing**: Complete round lifecycle management
//! - **Pure Rust Core**: Clean separation between core logic and language bindings
//!
//! ## Architecture
//!
//! The library follows SOLID principles and uses the Strategy pattern for scoring algorithms.
//! Core traits define interfaces for embedding models and scoring strategies, allowing
//! for easy extension and testing.

// Core library modules
// pub mod browser_integration;  // TODO: File missing, needs to be created or removed
pub mod commitment;
pub mod config;
pub mod data_models;
pub mod embedder;
pub mod error;
pub mod models;
pub mod payout;
pub mod round_processor;
pub mod scoring;
pub mod social;
pub mod twitter_utils;
pub mod types;

// New async round engine
pub mod round_engine;

// Python bindings module (conditional compilation)
#[cfg(feature = "python")]
pub mod python_bridge;

// Re-export commonly used types
// pub use browser_integration::{BrowserIntegration, Commitment, CommitmentCollectionResult};  // TODO: File missing
pub use commitment::{CommitmentGenerator, CommitmentVerifier};
pub use config::{CliptionsConfig, ConfigManager, CostTracker, OpenAIConfig, SpendingStatus};
pub use embedder::{EmbedderTrait, MockEmbedder};
pub use error::{CliptionsError, Result};
pub use payout::{PayoutCalculator, PayoutConfig, PayoutInfo};
pub use round_processor::RoundProcessor;
pub use scoring::{ClipBatchStrategy, ScoreValidator, ScoringStrategy};
pub use social::{
    AnnouncementData, AnnouncementFormatter, HashtagManager, SocialWorkflow, TweetId, UrlParser,
};
pub use types::{Guess, Participant, RoundData, ScoringResult};

// Re-export Python module when feature is enabled
#[cfg(feature = "python")]
pub use python_bridge::*;
