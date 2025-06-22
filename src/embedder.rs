//! Embedding model interface for RealMir
//! 
//! This module provides the interface for embedding models like CLIP,
//! along with a mock implementation for testing and development.

use ndarray::Array1;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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

/// Placeholder for future CLIP embedder implementation
/// 
/// This would integrate with actual CLIP models when implemented
#[derive(Debug, Clone)]
pub struct ClipEmbedder {
    embedding_dim: usize,
    // model: Option<ClipModel>, // Future: actual CLIP model
}

impl ClipEmbedder {
    /// Create a new CLIP embedder (placeholder)
    pub fn new() -> Result<Self> {
        // For now, return an error indicating this is not yet implemented
        Err(EmbeddingError::ModelLoadFailed.into())
    }
    
    /// Load CLIP model from a specific path (placeholder)
    pub fn from_path(_model_path: &str) -> Result<Self> {
        // Future implementation would load the model from the specified path
        Err(EmbeddingError::ModelLoadFailed.into())
    }
}

impl Default for ClipEmbedder {
    fn default() -> Self {
        Self {
            embedding_dim: 512,
        }
    }
}

impl EmbedderTrait for ClipEmbedder {
    fn get_image_embedding(&self, _image_path: &str) -> Result<Array1<f64>> {
        // Placeholder - would use actual CLIP model
        Err(EmbeddingError::ModelLoadFailed.into())
    }
    
    fn get_text_embedding(&self, _text: &str) -> Result<Array1<f64>> {
        // Placeholder - would use actual CLIP model  
        Err(EmbeddingError::ModelLoadFailed.into())
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
}