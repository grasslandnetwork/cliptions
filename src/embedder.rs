//! Embedding model interface for RealMir
//! 
//! This module provides the interface for embedding models like CLIP,
//! along with a mock implementation for testing and development.

use ndarray::Array1;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::Command;
use base64::prelude::*;
use crate::error::{EmbeddingError, Result};

/// Trait for embedding models that can convert images and text to feature vectors
/// 
/// This corresponds to the Python IEmbedder interface
pub trait EmbedderTrait: Send + Sync {
    /// Generate embedding for an image
    /// 
    /// # Arguments
    /// * `image_path` - Path to the image file
    /// 
    /// # Returns
    /// Normalized image embedding vector
    fn get_image_embedding(&self, image_path: &str) -> Result<Array1<f64>>;
    
    /// Generate embedding for text
    /// 
    /// # Arguments
    /// * `text` - Text string to embed
    /// 
    /// # Returns
    /// Normalized text embedding vector
    fn get_text_embedding(&self, text: &str) -> Result<Array1<f64>>;
    
    /// Get the dimensionality of embeddings produced by this model
    fn embedding_dim(&self) -> usize;
}

/// Mock embedder for testing and development
/// 
/// This embedder generates deterministic embeddings based on hash functions,
/// allowing for consistent testing without requiring actual CLIP models.
#[derive(Debug, Clone)]
pub struct MockEmbedder {
    embedding_dim: usize,
}

impl MockEmbedder {
    /// Create a new mock embedder with specified dimensions
    pub fn new(embedding_dim: usize) -> Self {
        Self { embedding_dim }
    }
    
    /// Create a mock embedder with CLIP-like dimensions (512)
    pub fn clip_like() -> Self {
        Self::new(512)
    }
    
    /// Generate a deterministic embedding from a hash
    fn hash_to_embedding(&self, input: &str) -> Array1<f64> {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Generate deterministic values from the hash
        let mut values = Vec::with_capacity(self.embedding_dim);
        let mut seed = hash;
        
        for _ in 0..self.embedding_dim {
            // Simple linear congruential generator for deterministic values
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let normalized = (seed as f64) / (u64::MAX as f64) * 2.0 - 1.0;
            values.push(normalized);
        }
        
        let mut embedding = Array1::from_vec(values);
        
        // Normalize the embedding vector
        let norm = embedding.dot(&embedding).sqrt();
        if norm > 0.0 {
            embedding /= norm;
        }
        
        embedding
    }
}

impl Default for MockEmbedder {
    fn default() -> Self {
        Self::clip_like()
    }
}

impl EmbedderTrait for MockEmbedder {
    fn get_image_embedding(&self, image_path: &str) -> Result<Array1<f64>> {
        // For mock purposes, just hash the image path
        // In a real implementation, this would load and process the image
        let embedding_input = format!("image:{}", image_path);
        Ok(self.hash_to_embedding(&embedding_input))
    }
    
    fn get_text_embedding(&self, text: &str) -> Result<Array1<f64>> {
        // Hash the text to create a deterministic embedding
        let embedding_input = format!("text:{}", text);
        Ok(self.hash_to_embedding(&embedding_input))
    }
    
    fn embedding_dim(&self) -> usize {
        self.embedding_dim
    }
}

/// CLIP embedder implementation that calls the Python CLIP implementation
/// 
/// This integrates with the existing Python CLIP implementation via subprocess calls
#[derive(Debug, Clone)]
pub struct ClipEmbedder {
    embedding_dim: usize,
    python_script_path: String,
}

impl ClipEmbedder {
    /// Create a new CLIP embedder
    pub fn new() -> Result<Self> {
        Self::with_script_path("core/clip_embedder.py")
    }
    
    /// Create CLIP embedder with custom Python script path
    pub fn with_script_path(script_path: &str) -> Result<Self> {
        // Check if Python script exists
        if !Path::new(script_path).exists() {
            return Err(EmbeddingError::ModelLoadFailed.into());
        }
        
        Ok(Self {
            embedding_dim: 512, // CLIP standard embedding size
            python_script_path: script_path.to_string(),
        })
    }
    
    /// Load CLIP model from a specific path (uses default Python implementation)
    pub fn from_path(_model_path: &str) -> Result<Self> {
        // For now, use the default Python implementation
        // Future versions could support loading different model checkpoints
        Self::new()
    }
    
    /// Call Python CLIP implementation for image embedding
    fn call_python_for_image(&self, image_path: &str) -> Result<Array1<f64>> {
        // Check if image file exists
        if !Path::new(image_path).exists() {
            return Err(EmbeddingError::ImageProcessingFailed.into());
        }
        
        // Read image and encode as base64
        let image_data = std::fs::read(image_path)
            .map_err(|_| EmbeddingError::ImageProcessingFailed)?;
        let base64_image = base64::prelude::BASE64_STANDARD.encode(&image_data);
        
        // Create JSON input
        let input_json = serde_json::json!({
            "image": base64_image
        });
        
        // Call Python script
        let output = Command::new("python")
            .arg(&self.python_script_path)
            .arg("--mode")
            .arg("image")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|_| EmbeddingError::ModelLoadFailed)?;
        
        // Write input to stdin
        let mut child = output;
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin.write_all(input_json.to_string().as_bytes())
                .map_err(|_| EmbeddingError::ImageProcessingFailed)?;
        }
        
        // Wait for completion and read output
        let output = child.wait_with_output()
            .map_err(|_| EmbeddingError::ImageProcessingFailed)?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            eprintln!("Python CLIP error: {}", error_msg);
            return Err(EmbeddingError::ImageProcessingFailed.into());
        }
        
        // Parse JSON output
        let output_str = String::from_utf8_lossy(&output.stdout);
        let result: serde_json::Value = serde_json::from_str(&output_str)
            .map_err(|_| EmbeddingError::ImageProcessingFailed)?;
        
        // Extract embedding
        let embedding_vec: Vec<f64> = result["embedding"]
            .as_array()
            .ok_or(EmbeddingError::ImageProcessingFailed)?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0))
            .collect();
        
        Ok(Array1::from_vec(embedding_vec))
    }
    
    /// Call Python CLIP implementation for text embedding
    fn call_python_for_text(&self, text: &str) -> Result<Array1<f64>> {
        // Create JSON input
        let input_json = serde_json::json!({
            "text": text
        });
        
        // Call Python script
        let output = Command::new("python")
            .arg(&self.python_script_path)
            .arg("--mode")
            .arg("text")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|_| EmbeddingError::ModelLoadFailed)?;
        
        // Write input to stdin
        let mut child = output;
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin.write_all(input_json.to_string().as_bytes())
                .map_err(|_| EmbeddingError::TokenizationFailed)?;
        }
        
        // Wait for completion and read output
        let output = child.wait_with_output()
            .map_err(|_| EmbeddingError::TokenizationFailed)?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            eprintln!("Python CLIP error: {}", error_msg);
            return Err(EmbeddingError::TokenizationFailed.into());
        }
        
        // Parse JSON output
        let output_str = String::from_utf8_lossy(&output.stdout);
        let result: serde_json::Value = serde_json::from_str(&output_str)
            .map_err(|_| EmbeddingError::TokenizationFailed)?;
        
        // Extract embedding
        let embedding_vec: Vec<f64> = result["embedding"]
            .as_array()
            .ok_or(EmbeddingError::TokenizationFailed)?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0))
            .collect();
        
        Ok(Array1::from_vec(embedding_vec))
    }
}

impl Default for ClipEmbedder {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            embedding_dim: 512,
            python_script_path: "core/clip_embedder.py".to_string(),
        })
    }
}

impl EmbedderTrait for ClipEmbedder {
    fn get_image_embedding(&self, image_path: &str) -> Result<Array1<f64>> {
        self.call_python_for_image(image_path)
    }
    
    fn get_text_embedding(&self, text: &str) -> Result<Array1<f64>> {
        self.call_python_for_text(text)
    }
    
    fn embedding_dim(&self) -> usize {
        self.embedding_dim
    }
}

/// Calculate cosine similarity between two embedding vectors
/// 
/// # Arguments
/// * `a` - First embedding vector
/// * `b` - Second embedding vector
/// 
/// # Returns
/// Cosine similarity score between -1 and 1
pub fn cosine_similarity(a: &Array1<f64>, b: &Array1<f64>) -> Result<f64> {
    if a.len() != b.len() {
        return Err(EmbeddingError::InvalidDimensions.into());
    }
    
    // For normalized vectors, cosine similarity is just the dot product
    let similarity = a.dot(b);
    
    // Clamp to valid range to handle floating point precision issues
    Ok(similarity.max(-1.0).min(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_embedder_deterministic() {
        let embedder = MockEmbedder::new(128);
        
        let text = "test text";
        let embedding1 = embedder.get_text_embedding(text).unwrap();
        let embedding2 = embedder.get_text_embedding(text).unwrap();
        
        // Should be deterministic
        assert_eq!(embedding1, embedding2);
    }
    
    #[test]
    fn test_mock_embedder_different_inputs() {
        let embedder = MockEmbedder::new(128);
        
        let embedding1 = embedder.get_text_embedding("text1").unwrap();
        let embedding2 = embedder.get_text_embedding("text2").unwrap();
        
        // Different inputs should produce different embeddings
        assert_ne!(embedding1, embedding2);
    }
    
    #[test]
    fn test_mock_embedder_normalized() {
        let embedder = MockEmbedder::new(128);
        
        let embedding = embedder.get_text_embedding("test").unwrap();
        let norm = embedding.dot(&embedding).sqrt();
        
        // Should be approximately normalized (within floating point precision)
        assert!((norm - 1.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_cosine_similarity() {
        let embedder = MockEmbedder::new(128);
        
        let embedding1 = embedder.get_text_embedding("similar text").unwrap();
        let embedding2 = embedder.get_text_embedding("similar text").unwrap();
        let embedding3 = embedder.get_text_embedding("different text").unwrap();
        
        // Identical embeddings should have similarity of 1.0
        let sim_identical = cosine_similarity(&embedding1, &embedding2).unwrap();
        assert!((sim_identical - 1.0).abs() < 1e-10);
        
        // Different embeddings should have different similarity
        let sim_different = cosine_similarity(&embedding1, &embedding3).unwrap();
        assert!(sim_different < 1.0);
        assert!(sim_different >= -1.0);
    }
    
    #[test]
    fn test_cosine_similarity_dimension_mismatch() {
        let embedder1 = MockEmbedder::new(128);
        let embedder2 = MockEmbedder::new(256);
        
        let embedding1 = embedder1.get_text_embedding("test").unwrap();
        let embedding2 = embedder2.get_text_embedding("test").unwrap();
        
        let result = cosine_similarity(&embedding1, &embedding2);
        assert!(matches!(result, Err(crate::error::RealMirError::Embedding(EmbeddingError::InvalidDimensions))));
    }
    
    #[test]
    fn test_image_vs_text_embeddings() {
        let embedder = MockEmbedder::new(128);
        
        let image_embedding = embedder.get_image_embedding("test.jpg").unwrap();
        let text_embedding = embedder.get_text_embedding("test.jpg").unwrap();
        
        // Same input should produce different embeddings for image vs text
        assert_ne!(image_embedding, text_embedding);
    }
    
    #[test]
    fn test_embedding_dimensions() {
        let embedder = MockEmbedder::new(256);
        
        assert_eq!(embedder.embedding_dim(), 256);
        
        let embedding = embedder.get_text_embedding("test").unwrap();
        assert_eq!(embedding.len(), 256);
    }
    
    #[test]
    fn test_clip_embedder_creation() {
        // Test creation with non-existent script path
        let result = ClipEmbedder::with_script_path("non_existent_script.py");
        assert!(result.is_err());
        
        // Test default creation (this should succeed since core/clip_embedder.py exists in this workspace)
        let result = ClipEmbedder::new();
        if std::path::Path::new("core/clip_embedder.py").exists() {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }
    
    #[test]
    fn test_clip_embedder_default_fallback() {
        // Test that default creation falls back gracefully
        let embedder = ClipEmbedder::default();
        assert_eq!(embedder.embedding_dim(), 512);
        
        // Verify that methods return appropriate errors when Python script is not available
        let result = embedder.get_text_embedding("test text");
        assert!(result.is_err());
        
        let result = embedder.get_image_embedding("test_image.jpg");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_clip_embedder_interface_consistency() {
        // Verify that ClipEmbedder implements the EmbedderTrait properly
        let embedder = ClipEmbedder::default();
        
        // Check that it has the correct embedding dimension
        assert_eq!(embedder.embedding_dim(), 512);
        
        // Verify the trait methods exist and return Results
        let text_result = embedder.get_text_embedding("test");
        assert!(matches!(text_result, Err(_)));
        
        let image_result = embedder.get_image_embedding("test.jpg");
        assert!(matches!(image_result, Err(_)));
    }
}