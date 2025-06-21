//! Python bridge for RealMir core functionality
//! 
//! This module provides Python bindings for the Rust core using PyO3.
//! It handles type conversion between Rust and Python types and exposes
//! the core functionality through a Python API.

use pyo3::prelude::*;
use ndarray::Array1;

use crate::commitment::{CommitmentGenerator, CommitmentVerifier};
use crate::scoring::{ScoringStrategy, BaselineAdjustedStrategy, ScoreValidator, calculate_rankings, calculate_payouts};
use crate::embedder::{MockEmbedder, cosine_similarity};
use crate::round::{RoundProcessor};
use crate::error::{RealMirError};

/// Convert RealMirError to PyErr for Python integration
impl From<RealMirError> for PyErr {
    fn from(err: RealMirError) -> PyErr {
        match err {
            RealMirError::Commitment(_) => {
                pyo3::exceptions::PyValueError::new_err(err.to_string())
            },
            RealMirError::Validation(_) => {
                pyo3::exceptions::PyValueError::new_err(err.to_string())
            },
            RealMirError::Io(_) => {
                pyo3::exceptions::PyIOError::new_err(err.to_string())
            },
            RealMirError::Json(_) => {
                pyo3::exceptions::PyValueError::new_err(err.to_string())
            },
            _ => pyo3::exceptions::PyRuntimeError::new_err(err.to_string()),
        }
    }
}

// =============================================================================
// Commitment Python Bindings
// =============================================================================

/// Python wrapper for CommitmentGenerator
#[pyclass]
pub struct PyCommitmentGenerator {
    inner: CommitmentGenerator,
}

#[pymethods]
impl PyCommitmentGenerator {
    /// Create a new commitment generator
    #[new]
    pub fn new() -> Self {
        Self {
            inner: CommitmentGenerator::new(),
        }
    }
    
    /// Generate a commitment hash
    pub fn generate(&self, message: &str, salt: &str) -> PyResult<String> {
        self.inner.generate(message, salt).map_err(|e| e.into())
    }
    
    /// Generate a random salt
    pub fn generate_salt(&self) -> String {
        self.inner.generate_salt()
    }
    
    /// Verify a commitment
    pub fn verify(&self, message: &str, salt: &str, commitment: &str) -> bool {
        CommitmentVerifier::new().verify(message, salt, commitment)
    }
}

/// Python function for generating commitments
/// 
/// This is a direct replacement for the Python generate_commitment function
#[pyfunction]
pub fn py_generate_commitment(message: &str, salt: &str) -> PyResult<String> {
    CommitmentGenerator::new()
        .generate(message, salt)
        .map_err(|e| e.into())
}

/// Python function for verifying commitments
#[pyfunction]
pub fn py_verify_commitment(message: &str, salt: &str, commitment: &str) -> bool {
    CommitmentVerifier::new().verify(message, salt, commitment)
}

// =============================================================================
// Scoring Python Bindings
// =============================================================================

/// Python wrapper for ScoreValidator
#[pyclass]
pub struct PyScoreValidator {
    inner: ScoreValidator<MockEmbedder, BaselineAdjustedStrategy>,
}

#[pymethods]
impl PyScoreValidator {
    #[new]
    pub fn new() -> Self {
        let embedder = MockEmbedder::clip_like();
        let strategy = BaselineAdjustedStrategy::new();
        Self {
            inner: ScoreValidator::new(embedder, strategy),
        }
    }
    
    pub fn validate_guess(&self, guess: &str) -> bool {
        self.inner.validate_guess(guess)
    }
    
    pub fn calculate_adjusted_score(&self, image_path: &str, guess: &str) -> PyResult<f64> {
        let image_features = self.inner.get_image_embedding(image_path)
            .map_err(|e| PyErr::from(e))?;
        self.inner.calculate_adjusted_score(&image_features, guess)
            .map_err(|e| PyErr::from(e))
    }
}

/// Python function for calculating cosine similarity
#[pyfunction]
pub fn py_calculate_cosine_similarity(a: Vec<f64>, b: Vec<f64>) -> PyResult<f64> {
    let arr_a = Array1::from_vec(a);
    let arr_b = Array1::from_vec(b);
    cosine_similarity(&arr_a, &arr_b).map_err(|e| e.into())
}

/// Python function for calculating baseline adjusted similarity
#[pyfunction]
pub fn py_calculate_baseline_adjusted_similarity(
    image_features: Vec<f64>,
    text_features: Vec<f64>,
    baseline_features: Vec<f64>,
) -> PyResult<f64> {
    let strategy = BaselineAdjustedStrategy::new();
    let img_arr = Array1::from_vec(image_features);
    let txt_arr = Array1::from_vec(text_features);
    let base_arr = Array1::from_vec(baseline_features);
    
    strategy.calculate_score(&img_arr, &txt_arr, Some(&base_arr))
        .map_err(|e| e.into())
}

/// Python function for calculating rankings
#[pyfunction]
pub fn py_calculate_rankings(
    target_image_path: &str,
    guesses: Vec<String>,
) -> PyResult<Vec<(String, f64)>> {
    let embedder = MockEmbedder::clip_like();
    let strategy = BaselineAdjustedStrategy::new();
    let validator = ScoreValidator::new(embedder, strategy);
    
    calculate_rankings(target_image_path, &guesses, &validator)
        .map_err(|e| e.into())
}

/// Python function for calculating payouts
#[pyfunction]
pub fn py_calculate_payouts(
    ranked_results: Vec<(String, f64)>,
    prize_pool: f64,
) -> PyResult<Vec<f64>> {
    calculate_payouts(&ranked_results, prize_pool)
        .map_err(|e| e.into())
}

// =============================================================================
// Round Processing Python Bindings
// =============================================================================

/// Python wrapper for RoundProcessor
#[pyclass]
pub struct PyRoundProcessor {
    inner: RoundProcessor<MockEmbedder, BaselineAdjustedStrategy>,
}

#[pymethods]
impl PyRoundProcessor {
    #[new]
    pub fn new(rounds_file: String) -> Self {
        let embedder = MockEmbedder::clip_like();
        let strategy = BaselineAdjustedStrategy::new();
        Self {
            inner: RoundProcessor::new(rounds_file, embedder, strategy),
        }
    }
    
    pub fn load_rounds(&mut self) -> PyResult<()> {
        self.inner.load_rounds().map_err(|e| e.into())
    }
    
    pub fn verify_commitments(&mut self, round_id: &str) -> PyResult<Vec<bool>> {
        self.inner.verify_commitments(round_id).map_err(|e| e.into())
    }
    
    pub fn process_round_payouts(&mut self, round_id: &str) -> PyResult<Vec<(String, String, f64, usize, f64)>> {
        let results = self.inner.process_round_payouts(round_id).map_err(|e| PyErr::from(e))?;
        
        // Convert to Python-friendly format
        let py_results = results.iter().map(|r| (
            r.participant.user_id.clone(),
            r.participant.guess.text.clone(),
            r.effective_score(),
            r.rank.unwrap_or(0),
            r.payout.unwrap_or(0.0),
        )).collect();
        
        Ok(py_results)
    }
    
    pub fn get_round_ids(&mut self) -> PyResult<Vec<String>> {
        self.inner.get_round_ids().map_err(|e| e.into())
    }
}

/// Python function for processing round payouts
#[pyfunction]
pub fn py_process_round_payouts(
    rounds_file: String,
    round_id: String,
) -> PyResult<Vec<(String, String, f64, usize, f64)>> {
    let embedder = MockEmbedder::clip_like();
    let strategy = BaselineAdjustedStrategy::new();
    let mut processor = RoundProcessor::new(rounds_file, embedder, strategy);
    
    let results = processor.process_round_payouts(&round_id).map_err(|e| PyErr::from(e))?;
    
    // Convert to Python-friendly format
    let py_results = results.iter().map(|r| (
        r.participant.user_id.clone(),
        r.participant.guess.text.clone(),
        r.effective_score(),
        r.rank.unwrap_or(0),
        r.payout.unwrap_or(0.0),
    )).collect();
    
    Ok(py_results)
}

/// Python function for verifying round commitments
#[pyfunction]
pub fn py_verify_round_commitments(
    rounds_file: String,
    round_id: String,
) -> PyResult<Vec<bool>> {
    let embedder = MockEmbedder::clip_like();
    let strategy = BaselineAdjustedStrategy::new();
    let mut processor = RoundProcessor::new(rounds_file, embedder, strategy);
    
    processor.verify_commitments(&round_id).map_err(|e| e.into())
}

// =============================================================================
// Python Module Definition
// =============================================================================

/// Python module definition
/// 
/// This exposes the Rust functionality to Python through PyO3 bindings.
/// All major functions are available as Python functions.
#[pymodule]
fn realmir_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add commitment functions
    m.add_function(wrap_pyfunction!(py_generate_commitment, m)?)?;
    m.add_function(wrap_pyfunction!(py_verify_commitment, m)?)?;
    
    // Add scoring functions
    m.add_function(wrap_pyfunction!(py_calculate_cosine_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(py_calculate_baseline_adjusted_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(py_calculate_rankings, m)?)?;
    m.add_function(wrap_pyfunction!(py_calculate_payouts, m)?)?;
    
    // Add round processing functions
    m.add_function(wrap_pyfunction!(py_process_round_payouts, m)?)?;
    m.add_function(wrap_pyfunction!(py_verify_round_commitments, m)?)?;
    
    // Add classes
    m.add_class::<PyCommitmentGenerator>()?;
    m.add_class::<PyScoreValidator>()?;
    m.add_class::<PyRoundProcessor>()?;
    
    Ok(())
} 