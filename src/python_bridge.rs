//! Python bridge for RealMir core functionality
//! 
//! This module provides Python bindings for the Rust core using PyO3.
//! It handles type conversion between Rust and Python types and exposes
//! the core functionality through a Python API.

use pyo3::prelude::*;
use pyo3::types::PyModule;
use ndarray::Array1;
use serde_json;

use crate::commitment::{CommitmentGenerator, CommitmentVerifier};
use crate::scoring::{ClipBatchStrategy, ScoreValidator, calculate_rankings, calculate_payouts};
use crate::embedder::{MockEmbedder, cosine_similarity};
use crate::round::{RoundProcessor};
use crate::data_access::{DataAccessLayer};
use crate::types::{TwitterReplyData, CommitmentCollectionResult};
use crate::error::{RealMirError};

/// Convert RealMirError to PyErr for Python integration
impl From<RealMirError> for PyErr {
    fn from(err: RealMirError) -> PyErr {
        match err {
            RealMirError::Commitment(_) => {
                pyo3::exceptions::PyValueError::new_err(err.to_string())
            },
            RealMirError::ValidationError(_) => {
                pyo3::exceptions::PyValueError::new_err(err.to_string())
            },
            RealMirError::Io(_) => {
                pyo3::exceptions::PyIOError::new_err(err.to_string())
            },
            RealMirError::Json(_) => {
                pyo3::exceptions::PyValueError::new_err(err.to_string())
            },
            RealMirError::DataAccess(_) => {
                pyo3::exceptions::PyRuntimeError::new_err(err.to_string())
            },
            _ => pyo3::exceptions::PyRuntimeError::new_err(err.to_string()),
        }
    }
}

// =============================================================================
// Data Access Python Bindings
// =============================================================================

/// Python wrapper for DataAccessLayer
#[pyclass]
pub struct PyDataAccessLayer {
    inner: DataAccessLayer,
}

#[pymethods]
impl PyDataAccessLayer {
    /// Create a new data access layer instance
    #[new]
    pub fn new(rounds_file_path: String) -> Self {
        Self {
            inner: DataAccessLayer::new(rounds_file_path),
        }
    }

    /// Load all rounds and return as JSON string
    pub fn load_all_rounds(&self) -> PyResult<String> {
        let rounds = self.inner.load_all_rounds().map_err(|e| PyErr::from(e))?;
        let json = serde_json::to_string(&rounds).map_err(|e| PyErr::from(RealMirError::Json(e)))?;
        Ok(json)
    }

    /// Get a specific round by ID and return as JSON string
    pub fn get_round(&self, round_id: String) -> PyResult<String> {
        let round_data = self.inner.get_round(&round_id).map_err(|e| PyErr::from(e))?;
        let json = serde_json::to_string(&round_data).map_err(|e| PyErr::from(RealMirError::Json(e)))?;
        Ok(json)
    }

    /// Update a round with JSON data
    pub fn update_round(&self, round_id: String, round_json: String) -> PyResult<()> {
        let round_data = serde_json::from_str(&round_json).map_err(|e| PyErr::from(RealMirError::Json(e)))?;
        self.inner.update_round(&round_id, round_data).map_err(|e| PyErr::from(e))
    }

    /// Delete a round by ID
    pub fn delete_round(&self, round_id: String) -> PyResult<()> {
        self.inner.delete_round(&round_id).map_err(|e| PyErr::from(e))
    }

    /// Check if a round exists
    pub fn round_exists(&self, round_id: String) -> PyResult<bool> {
        self.inner.round_exists(&round_id).map_err(|e| PyErr::from(e))
    }

    /// Get all round IDs
    pub fn get_all_round_ids(&self) -> PyResult<Vec<String>> {
        self.inner.get_all_round_ids().map_err(|e| PyErr::from(e))
    }

    /// Get rounds that have Twitter data
    pub fn get_rounds_with_twitter_data(&self) -> PyResult<Vec<String>> {
        self.inner.get_rounds_with_twitter_data().map_err(|e| PyErr::from(e))
    }

    /// Get rounds that have commitment collection results
    pub fn get_rounds_with_commitments(&self) -> PyResult<Vec<String>> {
        self.inner.get_rounds_with_commitments().map_err(|e| PyErr::from(e))
    }

    /// Validate data consistency and return list of issues
    pub fn validate_data(&self) -> PyResult<Vec<String>> {
        self.inner.validate_data_consistency().map_err(|e| PyErr::from(e))
    }

    /// Create a backup and return the backup path
    pub fn create_backup(&self) -> PyResult<String> {
        self.inner.create_backup().map_err(|e| PyErr::from(e))
    }

    /// Restore from a backup file
    pub fn restore_from_backup(&self, backup_path: String) -> PyResult<()> {
        self.inner.restore_from_backup(&backup_path).map_err(|e| PyErr::from(e))
    }

    /// Update Twitter data for a specific round
    pub fn update_round_twitter_data(&self, round_id: String, twitter_data_json: String) -> PyResult<()> {
        let twitter_data: TwitterReplyData = serde_json::from_str(&twitter_data_json)
            .map_err(|e| PyErr::from(RealMirError::Json(e)))?;
        self.inner.update_round_twitter_data(&round_id, twitter_data).map_err(|e| PyErr::from(e))
    }

    /// Update commitment collection results for a specific round
    pub fn update_round_commitments(&self, round_id: String, commitments_json: String) -> PyResult<()> {
        let commitments: CommitmentCollectionResult = serde_json::from_str(&commitments_json)
            .map_err(|e| PyErr::from(RealMirError::Json(e)))?;
        self.inner.update_round_commitments(&round_id, commitments).map_err(|e| PyErr::from(e))
    }
}

// =============================================================================
// Standalone functions for backward compatibility
// =============================================================================

/// Python function for loading rounds data
#[pyfunction]
pub fn py_load_rounds_data() -> PyResult<String> {
    let dal = DataAccessLayer::new("data/rounds.json".to_string());
    let rounds = dal.load_all_rounds().map_err(|e| PyErr::from(e))?;
    let json = serde_json::to_string(&rounds).map_err(|e| PyErr::from(RealMirError::Json(e)))?;
    Ok(json)
}

/// Python function for saving rounds data
#[pyfunction]
pub fn py_save_rounds_data(json_data: String) -> PyResult<()> {
    let dal = DataAccessLayer::new("data/rounds.json".to_string());
    let rounds = serde_json::from_str(&json_data).map_err(|e| PyErr::from(RealMirError::Json(e)))?;
    dal.save_all_rounds(&rounds).map_err(|e| PyErr::from(e))
}

/// Python function for getting round data
#[pyfunction]
pub fn py_get_round_data(round_id: String) -> PyResult<String> {
    let dal = DataAccessLayer::new("data/rounds.json".to_string());
    let round_data = dal.get_round(&round_id).map_err(|e| PyErr::from(e))?;
    let json = serde_json::to_string(&round_data).map_err(|e| PyErr::from(RealMirError::Json(e)))?;
    Ok(json)
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
        let guesses = vec![guess.to_string()];
        let similarities = self.inner.calculate_batch_similarities(image_path, &guesses)
            .map_err(|e| PyErr::from(e))?;
        Ok(similarities.get(0).copied().unwrap_or(0.0))
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
    #[allow(unused_variables)] use_mock: bool,
) -> PyResult<Vec<(String, f64)>> {
    let strategy = ClipBatchStrategy::new();
    
    // For now, always use MockEmbedder to avoid type compatibility issues
    let embedder = MockEmbedder::clip_like();
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
    inner: RoundProcessor<MockEmbedder, ClipBatchStrategy>,
}

#[pymethods]
impl PyRoundProcessor {
    #[new]
    pub fn new(rounds_file: String) -> Self {
        let embedder = MockEmbedder::clip_like();
        let strategy = ClipBatchStrategy::new();
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
#[pyo3(signature = (rounds_file, round_id, _use_mock = false))]
pub fn py_process_round_payouts(
    rounds_file: String,
    round_id: String,
    _use_mock: bool,
) -> PyResult<Vec<(String, String, f64, usize, f64)>> {
    let strategy = ClipBatchStrategy::new();
    
    // For now, always use MockEmbedder to avoid type compatibility issues
    let embedder = MockEmbedder::clip_like();
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
#[pyo3(signature = (rounds_file, round_id, _use_mock = false))]
pub fn py_verify_round_commitments(
    rounds_file: String,
    round_id: String,
    _use_mock: bool,
) -> PyResult<Vec<bool>> {
    let strategy = ClipBatchStrategy::new();
    
    // For now, always use MockEmbedder to avoid type compatibility issues
    let embedder = MockEmbedder::clip_like();
    let mut processor = RoundProcessor::new(rounds_file, embedder, strategy);
    
    processor.verify_commitments(&round_id).map_err(|e| e.into())
}

// =============================================================================
// Schema Consistency Test Bindings
// =============================================================================

/// Test function to deserialize a Commitment from a Python dict.
/// Used for ensuring Pydantic models and Rust structs are in sync.
#[pyfunction]
fn test_deserialize_commitment(commitment_dict: &Bound<'_, pyo3::types::PyDict>) -> PyResult<()> {
    // Convert Python dict to JSON string, then deserialize to Rust struct
    let _json_str = commitment_dict.call_method0("__str__")?
        .extract::<String>()?
        .replace("'", "\""); // Convert single quotes to double quotes for valid JSON
    
    // Alternative approach: use Python's json module
    let json_module = PyModule::import_bound(commitment_dict.py(), "json")?;
    let json_str = json_module
        .getattr("dumps")?
        .call1((commitment_dict,))?
        .extract::<String>()?;
    
    let _: crate::models::Commitment = serde_json::from_str(&json_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("JSON deserialization failed: {}", e)))?;
    
    Ok(())
}

/// Test function to deserialize a Round from a Python dict.
/// Used for ensuring Pydantic models and Rust structs are in sync.
#[pyfunction]
fn test_deserialize_round(round_dict: &Bound<'_, pyo3::types::PyDict>) -> PyResult<()> {
    // Convert Python dict to JSON string, then deserialize to Rust struct
    let json_module = PyModule::import_bound(round_dict.py(), "json")?;
    let json_str = json_module
        .getattr("dumps")?
        .call1((round_dict,))?
        .extract::<String>()?;
    
    let _: crate::models::Round = serde_json::from_str(&json_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("JSON deserialization failed: {}", e)))?;
    
    Ok(())
}

/// Test function for Twitter reply data schema consistency
#[pyfunction]
fn test_deserialize_twitter_reply_data(twitter_dict: &Bound<'_, pyo3::types::PyDict>) -> PyResult<()> {
    let json_module = PyModule::import_bound(twitter_dict.py(), "json")?;
    let json_str = json_module
        .getattr("dumps")?
        .call1((twitter_dict,))?
        .extract::<String>()?;
    
    let _: TwitterReplyData = serde_json::from_str(&json_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("JSON deserialization failed: {}", e)))?;
    
    Ok(())
}

/// Test function for commitment collection schema consistency
#[pyfunction]
fn test_deserialize_commitment_collection(collection_dict: &Bound<'_, pyo3::types::PyDict>) -> PyResult<()> {
    let json_module = PyModule::import_bound(collection_dict.py(), "json")?;
    let json_str = json_module
        .getattr("dumps")?
        .call1((collection_dict,))?
        .extract::<String>()?;
    
    let _: CommitmentCollectionResult = serde_json::from_str(&json_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("JSON deserialization failed: {}", e)))?;
    
    Ok(())
}

/// Main Python module definition
#[pymodule]
fn realmir_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Classes
    m.add_class::<PyCommitmentGenerator>()?;
    m.add_class::<PyScoreValidator>()?;
    m.add_class::<PyRoundProcessor>()?;
    m.add_class::<PyDataAccessLayer>()?;

    // Functions
    m.add_function(wrap_pyfunction!(py_generate_commitment, m)?)?;
    m.add_function(wrap_pyfunction!(py_verify_commitment, m)?)?;
    m.add_function(wrap_pyfunction!(py_calculate_cosine_similarity, m)?)?;

    m.add_function(wrap_pyfunction!(py_calculate_rankings, m)?)?;
    m.add_function(wrap_pyfunction!(py_calculate_payouts, m)?)?;
    m.add_function(wrap_pyfunction!(py_process_round_payouts, m)?)?;
    m.add_function(wrap_pyfunction!(py_verify_round_commitments, m)?)?;

    // Data access functions
    m.add_function(wrap_pyfunction!(py_load_rounds_data, m)?)?;
    m.add_function(wrap_pyfunction!(py_save_rounds_data, m)?)?;
    m.add_function(wrap_pyfunction!(py_get_round_data, m)?)?;

    // Schema test functions
    m.add_function(wrap_pyfunction!(test_deserialize_commitment, m)?)?;
    m.add_function(wrap_pyfunction!(test_deserialize_round, m)?)?;
    m.add_function(wrap_pyfunction!(test_deserialize_twitter_reply_data, m)?)?;
    m.add_function(wrap_pyfunction!(test_deserialize_commitment_collection, m)?)?;

    Ok(())
} 