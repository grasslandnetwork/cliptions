//! # RealMir Core
//! 
//! High-performance Rust implementation of the RealMir prediction market core functionality.
//! This library provides cryptographic commitments, scoring strategies, embedding integration,
//! and round processing capabilities.
//! 
//! ## Features
//! 
//! - **Commitment System**: Secure commitment generation and verification using SHA-256
//! - **Scoring Strategies**: Multiple scoring algorithms including baseline-adjusted similarity
//! - **Embedding Integration**: Interface for CLIP and other embedding models
//! - **Round Processing**: Complete round lifecycle management
//! - **Python Bindings**: Full PyO3 integration for seamless Python interoperability
//! 
//! ## Architecture
//! 
//! The library follows SOLID principles and uses the Strategy pattern for scoring algorithms.
//! Core traits define interfaces for embedding models and scoring strategies, allowing
//! for easy extension and testing.

use pyo3::prelude::*;

// Public modules
pub mod commitment;
pub mod scoring;
pub mod embedder;
pub mod round;
pub mod types;
pub mod error;

// Re-export commonly used types
pub use commitment::{CommitmentGenerator, CommitmentVerifier};
pub use scoring::{ScoringStrategy, BaselineAdjustedStrategy, RawSimilarityStrategy, ScoreValidator};
pub use embedder::{EmbedderTrait, MockEmbedder};
pub use round::{RoundProcessor};
pub use types::{Guess, Participant, ScoringResult, RoundData};
pub use error::{RealMirError, Result};

/// Python module definition
/// 
/// This exposes the Rust functionality to Python through PyO3 bindings.
/// All major functions are available as Python functions.
#[pymodule]
fn realmir_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add commitment functions
    m.add_function(wrap_pyfunction!(commitment::py_generate_commitment, m)?)?;
    m.add_function(wrap_pyfunction!(commitment::py_verify_commitment, m)?)?;
    
    // Add scoring functions
    m.add_function(wrap_pyfunction!(scoring::py_calculate_cosine_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(scoring::py_calculate_baseline_adjusted_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(scoring::py_calculate_rankings, m)?)?;
    m.add_function(wrap_pyfunction!(scoring::py_calculate_payouts, m)?)?;
    
    // Add round processing functions
    m.add_function(wrap_pyfunction!(round::py_process_round_payouts, m)?)?;
    m.add_function(wrap_pyfunction!(round::py_verify_round_commitments, m)?)?;
    
    // Add classes
    m.add_class::<commitment::PyCommitmentGenerator>()?;
    m.add_class::<scoring::PyScoreValidator>()?;
    m.add_class::<round::PyRoundProcessor>()?;
    
    Ok(())
}