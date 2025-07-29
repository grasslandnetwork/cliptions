//! Python bridge for Cliptions core functionality
//!
//! This module provides Python bindings for the Rust core using PyO3.
//! It handles type conversion between Rust and Python types and exposes
//! the core functionality through a Python API.

use ndarray::Array1;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use serde_json;

use crate::commitment::{CommitmentGenerator, CommitmentVerifier};
use crate::embedder::{cosine_similarity, ClipEmbedder, MockEmbedder};
use crate::error::CliptionsError;
use crate::block_processor::BlockProcessor;
use crate::scoring::{
    calculate_payouts, calculate_rankings, ClipBatchStrategy, ScoreValidator, ScoringStrategy,
};

/// Convert CliptionsError to PyErr for Python integration
impl From<CliptionsError> for PyErr {
    fn from(err: CliptionsError) -> PyErr {
        match err {
            CliptionsError::Commitment(_) => {
                pyo3::exceptions::PyValueError::new_err(err.to_string())
            }
            CliptionsError::Validation(_) => {
                pyo3::exceptions::PyValueError::new_err(err.to_string())
            }
            CliptionsError::Io(_) => pyo3::exceptions::PyIOError::new_err(err.to_string()),
            CliptionsError::Json(_) => pyo3::exceptions::PyValueError::new_err(err.to_string()),
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
    inner: ScoreValidator<MockEmbedder, ClipBatchStrategy>,
}

#[pymethods]
impl PyScoreValidator {
    #[new]
    pub fn new() -> Self {
        let embedder = MockEmbedder::clip_like();
        let strategy = ClipBatchStrategy::new();
        Self {
            inner: ScoreValidator::new(embedder, strategy),
        }
    }

    pub fn validate_guess(&self, guess: &str) -> bool {
        self.inner.validate_guess(guess)
    }

    pub fn calculate_adjusted_score(&self, image_path: &str, guess: &str) -> PyResult<f64> {
        let image_features = self
            .inner
            .get_image_embedding(image_path)
            .map_err(|e| PyErr::from(e))?;
        self.inner
            .calculate_adjusted_score(&image_features, guess)
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

/// Python function for calculating rankings
#[pyfunction]
#[pyo3(signature = (target_image_path, guesses, use_mock = false))]
pub fn py_calculate_rankings(
    target_image_path: &str,
    guesses: Vec<String>,
    use_mock: bool,
) -> PyResult<Vec<(String, f64)>> {
    let strategy = ClipBatchStrategy::new();

    if use_mock {
        let embedder = MockEmbedder::clip_like();
        let validator = ScoreValidator::new(embedder, strategy);
        calculate_rankings(target_image_path, &guesses, &validator).map_err(|e| e.into())
    } else {
        // Try CLIP - panic if it fails
        match ClipEmbedder::new() {
            Ok(embedder) => {
                let validator = ScoreValidator::new(embedder, strategy);
                calculate_rankings(target_image_path, &guesses, &validator).map_err(|e| e.into())
            }
            Err(e) => {
                panic!("CRITICAL: Failed to load CLIP model: {}. Cannot proceed with invalid MockEmbedder fallback as this would produce unreliable scores.", e);
            }
        }
    }
}

/// Python function for calculating payouts
#[pyfunction]
pub fn py_calculate_payouts(
    ranked_results: Vec<(String, f64)>,
    prize_pool: f64,
) -> PyResult<Vec<f64>> {
    calculate_payouts(&ranked_results, prize_pool).map_err(|e| e.into())
}

// =============================================================================
// Block Processing Python Bindings
// =============================================================================

/// Python wrapper for BlockProcessor
#[pyclass]
pub struct PyBlockProcessor {
    inner: BlockProcessor<MockEmbedder, ClipBatchStrategy>,
}

#[pymethods]
impl PyBlockProcessor {
    #[new]
    pub fn new(blocks_file: String) -> Self {
        let embedder = MockEmbedder::clip_like();
        let strategy = ClipBatchStrategy::new();
        Self {
            inner: BlockProcessor::new(blocks_file, embedder, strategy),
        }
    }

    pub fn load_blocks(&mut self) -> PyResult<()> {
        self.inner.load_blocks().map_err(|e| e.into())
    }

    pub fn verify_commitments(&mut self, block_num: &str) -> PyResult<Vec<bool>> {
        self.inner
            .verify_commitments(block_num)
            .map_err(|e| e.into())
    }

    pub fn process_block_payouts(
        &mut self,
        block_num: &str,
    ) -> PyResult<Vec<(String, String, f64, usize, f64)>> {
        let results = self
            .inner
            .process_block_payouts(block_num)
            .map_err(|e| PyErr::from(e))?;

        // Convert to Python-friendly format
        let py_results = results
            .iter()
            .map(|r| {
                (
                    r.participant.user_id.clone(),
                    r.participant.guess.text.clone(),
                    r.effective_score(),
                    r.rank.unwrap_or(0),
                    r.payout.unwrap_or(0.0),
                )
            })
            .collect();

        Ok(py_results)
    }

    pub fn get_block_nums(&mut self) -> PyResult<Vec<String>> {
        self.inner.get_block_nums().map_err(|e| e.into())
    }
}

/// Python function for processing block payouts
#[pyfunction]
#[pyo3(signature = (blocks_file, block_num, use_mock = false))]
pub fn py_process_block_payouts(
    blocks_file: String,
    block_num: String,
    use_mock: bool,
) -> PyResult<Vec<(String, String, f64, usize, f64)>> {
    let strategy = ClipBatchStrategy::new();

    let mut processor = if use_mock {
        let embedder = MockEmbedder::clip_like();
        BlockProcessor::new(blocks_file, embedder, strategy)
    } else {
        // Try CLIP - panic if it fails
        match ClipEmbedder::new() {
            Ok(embedder) => BlockProcessor::new(blocks_file, embedder, strategy),
            Err(e) => {
                panic!("CRITICAL: Failed to load CLIP model: {}. Cannot proceed with invalid MockEmbedder fallback as this would produce unreliable payout calculations.", e);
            }
        }
    };

    let results = processor
        .process_block_payouts(&block_num)
        .map_err(|e| PyErr::from(e))?;

    // Convert to Python-friendly format
    let py_results = results
        .iter()
        .map(|r| {
            (
                r.participant.user_id.clone(),
                r.participant.guess.text.clone(),
                r.effective_score(),
                r.rank.unwrap_or(0),
                r.payout.unwrap_or(0.0),
            )
        })
        .collect();

    Ok(py_results)
}

/// Python function for verifying block commitments
#[pyfunction]
#[pyo3(signature = (blocks_file, block_num, use_mock = false))]
pub fn py_verify_block_commitments(
    blocks_file: String,
    block_num: String,
    use_mock: bool,
) -> PyResult<Vec<bool>> {
    let strategy = ClipBatchStrategy::new();

    let mut processor = if use_mock {
        let embedder = MockEmbedder::clip_like();
        BlockProcessor::new(blocks_file, embedder, strategy)
    } else {
        // Try CLIP - panic if it fails
        match ClipEmbedder::new() {
            Ok(embedder) => BlockProcessor::new(blocks_file, embedder, strategy),
            Err(e) => {
                panic!("CRITICAL: Failed to load CLIP model: {}. Cannot proceed with invalid MockEmbedder fallback as this would produce unreliable verification results.", e);
            }
        }
    };

    processor
        .verify_commitments(&block_num)
        .map_err(|e| e.into())
}

// =============================================================================
// Schema Consistency Test Bindings
// =============================================================================

/// Test function to deserialize a Commitment from a Python dict.
/// Used for ensuring Pydantic models and Rust structs are in sync.
#[pyfunction]
fn test_deserialize_commitment(commitment_dict: &Bound<'_, pyo3::types::PyDict>) -> PyResult<()> {
    // Convert Python dict to JSON string, then deserialize to Rust struct
    let json_str = commitment_dict
        .call_method0("__str__")?
        .extract::<String>()?
        .replace("'", "\""); // Convert single quotes to double quotes for valid JSON

    // Alternative approach: use Python's json module
    let json_module = PyModule::import_bound(commitment_dict.py(), "json")?;
    let json_str = json_module
        .getattr("dumps")?
        .call1((commitment_dict,))?
        .extract::<String>()?;

    let _: crate::models::Commitment = serde_json::from_str(&json_str).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("JSON deserialization failed: {}", e))
    })?;

    Ok(())
}

/// Test function to deserialize a Block from a Python dict.
/// Used for ensuring Pydantic models and Rust structs are in sync.
#[pyfunction]
fn test_deserialize_block(block_dict: &Bound<'_, pyo3::types::PyDict>) -> PyResult<()> {
    // Convert Python dict to JSON string, then deserialize to Rust struct
    let json_module = PyModule::import_bound(block_dict.py(), "json")?;
    let json_str = json_module
        .getattr("dumps")?
        .call1((block_dict,))?
        .extract::<String>()?;

    let _: crate::models::Block = serde_json::from_str(&json_str).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("JSON deserialization failed: {}", e))
    })?;

    Ok(())
}

/// Main Python module definition
#[pymodule]
fn cliptions_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Classes
    m.add_class::<PyCommitmentGenerator>()?;
    m.add_class::<PyScoreValidator>()?;
    m.add_class::<PyBlockProcessor>()?;

    // Functions
    m.add_function(wrap_pyfunction!(py_generate_commitment, m)?)?;
    m.add_function(wrap_pyfunction!(py_verify_commitment, m)?)?;
    m.add_function(wrap_pyfunction!(py_calculate_cosine_similarity, m)?)?;

    m.add_function(wrap_pyfunction!(py_calculate_rankings, m)?)?;
    m.add_function(wrap_pyfunction!(py_calculate_payouts, m)?)?;
    m.add_function(wrap_pyfunction!(py_process_block_payouts, m)?)?;
    m.add_function(wrap_pyfunction!(py_verify_block_commitments, m)?)?;

    // Schema test functions
    m.add_function(wrap_pyfunction!(test_deserialize_commitment, m)?)?;
    m.add_function(wrap_pyfunction!(test_deserialize_block, m)?)?;

    Ok(())
}
