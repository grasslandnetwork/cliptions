//! Steganography module for embedding CLIP vectors in images
//! 
//! This module provides functionality to embed CLIP vectors (512 or 768 floats)
//! into images using LSB (Least Significant Bit) steganography. This allows
//! for hiding proof-of-work data in images that can be posted to Twitter.

use image::{Rgb, RgbImage};
use crate::error::{Result, SteganographyError};
use serde::{Deserialize, Serialize};

/// Magic header to identify embedded data
const MAGIC_HEADER: &[u8] = b"RMCLIP";

/// Metadata about embedded vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedVectorMeta {
    /// Version of the encoding format
    pub version: u8,
    /// Dimension of the embedded vector
    pub dimension: u32,
    /// Salt used in the commitment
    pub salt: String,
    /// Round ID for verification
    pub round_id: String,
    /// Optional checksum for data integrity
    pub checksum: Option<u32>,
}

/// Steganography encoder/decoder for CLIP vectors
#[derive(Debug, Clone)]
pub struct VectorSteganographer {
    /// Number of bits to use per color channel (1-8)
    bits_per_channel: u8,
}

impl VectorSteganographer {
    /// Create a new steganographer with default settings (2 bits per channel)
    pub fn new() -> Self {
        Self {
            bits_per_channel: 2,
        }
    }
    
    /// Create a steganographer with custom bits per channel
    /// Higher values hide more data but are more detectable
    /// Recommended: 1-3 bits for better stealth
    pub fn with_bits_per_channel(bits_per_channel: u8) -> Result<Self> {
        if bits_per_channel == 0 || bits_per_channel > 8 {
            return Err(SteganographyError::InvalidConfiguration.into());
        }
        
        Ok(Self { bits_per_channel })
    }
    
    /// Embed a CLIP vector into an image
    /// 
    /// # Arguments
    /// * `image_path` - Path to the source image
    /// * `vector` - CLIP vector (f64 values, typically 512 or 768 elements)
    /// * `meta` - Metadata about the vector
    /// * `output_path` - Path where the encoded image will be saved
    pub fn embed_vector(
        &self,
        image_path: &str,
        vector: &[f64],
        meta: &EmbeddedVectorMeta,
        output_path: &str,
    ) -> Result<()> {
        // Load the image
        let mut img = image::open(image_path)
            .map_err(|_| SteganographyError::InvalidImage)?
            .to_rgb8();
        
        // Convert vector to bytes
        let vector_bytes = self.vector_to_bytes(vector)?;
        
        // Serialize metadata
        let meta_bytes = serde_json::to_vec(meta)
            .map_err(|_| SteganographyError::EncodingFailed)?;
        
        // Create payload: MAGIC_HEADER + meta_length + meta + vector_length + vector
        let mut payload = Vec::new();
        payload.extend_from_slice(MAGIC_HEADER);
        payload.extend_from_slice(&(meta_bytes.len() as u32).to_le_bytes());
        payload.extend_from_slice(&meta_bytes);
        payload.extend_from_slice(&(vector_bytes.len() as u32).to_le_bytes());
        payload.extend_from_slice(&vector_bytes);
        
        // Check if image can hold the payload
        let max_capacity = self.calculate_capacity(&img);
        if payload.len() > max_capacity {
            return Err(SteganographyError::InsufficientCapacity.into());
        }
        
        // Embed the payload
        self.embed_bytes(&mut img, &payload)?;
        
        // Save the modified image
        img.save(output_path)
            .map_err(|_| SteganographyError::SaveFailed)?;
        
        Ok(())
    }
    
    /// Extract a CLIP vector from an image
    /// 
    /// # Arguments
    /// * `image_path` - Path to the image containing embedded data
    /// 
    /// # Returns
    /// Tuple of (vector, metadata)
    pub fn extract_vector(&self, image_path: &str) -> Result<(Vec<f64>, EmbeddedVectorMeta)> {
        // Load the image
        let img = image::open(image_path)
            .map_err(|_| SteganographyError::InvalidImage)?
            .to_rgb8();
        
        // Try to extract the payload
        let payload = self.extract_bytes(&img)?;
        
        // Verify magic header
        if payload.len() < MAGIC_HEADER.len() || &payload[..MAGIC_HEADER.len()] != MAGIC_HEADER {
            return Err(SteganographyError::NoEmbeddedData.into());
        }
        
        let mut offset = MAGIC_HEADER.len();
        
        // Read metadata length
        if payload.len() < offset + 4 {
            return Err(SteganographyError::CorruptedData.into());
        }
        let meta_len = u32::from_le_bytes([
            payload[offset], payload[offset + 1], payload[offset + 2], payload[offset + 3]
        ]) as usize;
        offset += 4;
        
        // Read metadata
        if payload.len() < offset + meta_len {
            return Err(SteganographyError::CorruptedData.into());
        }
        let meta_bytes = &payload[offset..offset + meta_len];
        let meta: EmbeddedVectorMeta = serde_json::from_slice(meta_bytes)
            .map_err(|_| SteganographyError::CorruptedData)?;
        offset += meta_len;
        
        // Read vector length
        if payload.len() < offset + 4 {
            return Err(SteganographyError::CorruptedData.into());
        }
        let vector_len = u32::from_le_bytes([
            payload[offset], payload[offset + 1], payload[offset + 2], payload[offset + 3]
        ]) as usize;
        offset += 4;
        
        // Read vector data
        if payload.len() < offset + vector_len {
            return Err(SteganographyError::CorruptedData.into());
        }
        let vector_bytes = &payload[offset..offset + vector_len];
        
        // Convert bytes back to vector
        let vector = self.bytes_to_vector(vector_bytes, meta.dimension as usize)?;
        
        Ok((vector, meta))
    }
    
    /// Check if an image contains embedded data
    pub fn has_embedded_data(&self, image_path: &str) -> bool {
        match self.extract_vector(image_path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    
    /// Calculate the maximum capacity of an image in bytes
    pub fn calculate_capacity(&self, img: &RgbImage) -> usize {
        let (width, height) = img.dimensions();
        let total_pixels = (width * height) as usize;
        let total_channels = total_pixels * 3; // RGB
        let bits_available = total_channels * self.bits_per_channel as usize;
        bits_available / 8 // Convert to bytes
    }
    
    /// Convert f64 vector to bytes for embedding
    fn vector_to_bytes(&self, vector: &[f64]) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        for &value in vector {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Ok(bytes)
    }
    
    /// Convert bytes back to f64 vector
    fn bytes_to_vector(&self, bytes: &[u8], dimension: usize) -> Result<Vec<f64>> {
        if bytes.len() != dimension * 8 {
            return Err(SteganographyError::CorruptedData.into());
        }
        
        let mut vector = Vec::with_capacity(dimension);
        for i in 0..dimension {
            let start = i * 8;
            let float_bytes = [
                bytes[start], bytes[start + 1], bytes[start + 2], bytes[start + 3],
                bytes[start + 4], bytes[start + 5], bytes[start + 6], bytes[start + 7],
            ];
            vector.push(f64::from_le_bytes(float_bytes));
        }
        
        Ok(vector)
    }
    
    /// Embed bytes into image using LSB steganography
    fn embed_bytes(&self, img: &mut RgbImage, data: &[u8]) -> Result<()> {
        let (width, height) = img.dimensions();
        let mut bit_index = 0;
        let total_bits = data.len() * 8;
        
        let mask = (1u8 << self.bits_per_channel) - 1; // Create mask for extracting bits
        let clear_mask = !((1u8 << self.bits_per_channel) - 1); // Mask for clearing LSBs
        
        'outer: for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel_mut(x, y);
                
                // Process each color channel (R, G, B)
                for channel in 0..3 {
                    if bit_index >= total_bits {
                        break 'outer;
                    }
                    
                    // Extract bits from data
                    let byte_index = bit_index / 8;
                    let bit_offset = bit_index % 8;
                    
                    let mut bits_to_embed = 0u8;
                    for i in 0..self.bits_per_channel {
                        if bit_index + i as usize >= total_bits {
                            break;
                        }
                        
                        let data_bit = (data[byte_index] >> (bit_offset + i as usize)) & 1;
                        bits_to_embed |= data_bit << i;
                    }
                    
                    // Clear LSBs and embed new bits
                    pixel[channel] = (pixel[channel] & clear_mask) | (bits_to_embed & mask);
                    
                    bit_index += self.bits_per_channel as usize;
                }
            }
        }
        
        if bit_index < total_bits {
            return Err(SteganographyError::InsufficientCapacity.into());
        }
        
        Ok(())
    }
    
    /// Extract bytes from image using LSB steganography
    fn extract_bytes(&self, img: &RgbImage) -> Result<Vec<u8>> {
        let (width, height) = img.dimensions();
        let mut extracted_bits = Vec::new();
        
        let mask = (1u8 << self.bits_per_channel) - 1;
        
        // First, extract enough bits to read the magic header and sizes
        let min_header_bytes = MAGIC_HEADER.len() + 4 + 4; // magic + meta_len + vector_len
        let min_bits_needed = min_header_bytes * 8;
        
        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                
                for channel in 0..3 {
                    let channel_bits = pixel[channel] & mask;
                    
                    // Extract individual bits
                    for i in 0..self.bits_per_channel {
                        if extracted_bits.len() >= min_bits_needed && extracted_bits.len() >= self.calculate_total_bits_needed(&extracted_bits)? {
                            return self.bits_to_bytes(&extracted_bits);
                        }
                        
                        let bit = (channel_bits >> i) & 1;
                        extracted_bits.push(bit);
                    }
                }
            }
        }
        
        // Convert bits to bytes for final result
        self.bits_to_bytes(&extracted_bits)
    }
    
    /// Calculate total bits needed based on extracted header information
    fn calculate_total_bits_needed(&self, bits: &[u8]) -> Result<usize> {
        if bits.len() < (MAGIC_HEADER.len() + 8) * 8 {
            return Ok(usize::MAX); // Need more bits
        }
        
        // Convert initial bits to bytes to read header
        let initial_bytes = self.bits_to_bytes(&bits[0..(MAGIC_HEADER.len() + 8) * 8])?;
        
        // Verify magic header
        if &initial_bytes[..MAGIC_HEADER.len()] != MAGIC_HEADER {
            return Err(SteganographyError::NoEmbeddedData.into());
        }
        
        let mut offset = MAGIC_HEADER.len();
        
        // Read metadata length
        let meta_len = u32::from_le_bytes([
            initial_bytes[offset], initial_bytes[offset + 1], 
            initial_bytes[offset + 2], initial_bytes[offset + 3]
        ]) as usize;
        offset += 4;
        
        // Read vector length  
        let vector_len = u32::from_le_bytes([
            initial_bytes[offset], initial_bytes[offset + 1],
            initial_bytes[offset + 2], initial_bytes[offset + 3]
        ]) as usize;
        
        let total_payload_bytes = MAGIC_HEADER.len() + 4 + meta_len + 4 + vector_len;
        Ok(total_payload_bytes * 8)
    }
    
    /// Convert bit array to byte array
    fn bits_to_bytes(&self, bits: &[u8]) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        
        for chunk in bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                byte |= bit << i;
            }
            bytes.push(byte);
        }
        
        Ok(bytes)
    }
}

impl Default for VectorSteganographer {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for working with steganography
pub mod utils {
    use super::*;
    
    /// Create a test image suitable for steganography
    /// Returns path to the created image
    pub fn create_test_image(width: u32, height: u32, output_path: &str) -> Result<()> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let mut img = RgbImage::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let r = rng.gen_range(0..=255);
                let g = rng.gen_range(0..=255);
                let b = rng.gen_range(0..=255);
                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        
        img.save(output_path)
            .map_err(|_| SteganographyError::SaveFailed)?;
        
        Ok(())
    }
    
    /// Calculate the minimum image size needed for a given vector
    pub fn min_image_size_for_vector(vector_len: usize, bits_per_channel: u8) -> (u32, u32) {
        let vector_bytes = vector_len * 8; // f64 = 8 bytes each
        let meta_bytes = 256; // Estimated metadata size
        let header_bytes = 6 + 8; // Magic header + lengths
        let total_bytes = vector_bytes + meta_bytes + header_bytes;
        let total_bits = total_bytes * 8;
        
        let bits_per_pixel = 3 * bits_per_channel as usize; // RGB channels
        let pixels_needed = (total_bits + bits_per_pixel - 1) / bits_per_pixel; // Ceiling division
        
        // Find square-ish dimensions
        let side = ((pixels_needed as f64).sqrt().ceil() as u32).max(64); // Minimum 64x64
        (side, side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;
    
    fn create_test_vector() -> Vec<f64> {
        (0..512).map(|i| (i as f64) / 512.0).collect()
    }
    
    fn create_test_meta() -> EmbeddedVectorMeta {
        EmbeddedVectorMeta {
            version: 1,
            dimension: 512,
            salt: "test_salt_123".to_string(),
            round_id: "test_round".to_string(),
            checksum: None,
        }
    }
    
    #[test]
    fn test_embed_and_extract_vector() {
        let temp_dir = TempDir::new().unwrap();
        let test_img_path = temp_dir.path().join("test.png");
        let encoded_img_path = temp_dir.path().join("encoded.png");
        
        // Create test image
        utils::create_test_image(512, 512, test_img_path.to_str().unwrap()).unwrap();
        
        let steganographer = VectorSteganographer::new();
        let test_vector = create_test_vector();
        let test_meta = create_test_meta();
        
        // Embed vector
        steganographer.embed_vector(
            test_img_path.to_str().unwrap(),
            &test_vector,
            &test_meta,
            encoded_img_path.to_str().unwrap(),
        ).unwrap();
        
        // Extract vector
        let (extracted_vector, extracted_meta) = steganographer
            .extract_vector(encoded_img_path.to_str().unwrap())
            .unwrap();
        
        // Verify
        assert_eq!(extracted_vector.len(), test_vector.len());
        assert_eq!(extracted_meta.dimension, test_meta.dimension);
        assert_eq!(extracted_meta.salt, test_meta.salt);
        assert_eq!(extracted_meta.round_id, test_meta.round_id);
        
        // Check vector values (should be very close due to floating point precision)
        for (original, extracted) in test_vector.iter().zip(extracted_vector.iter()) {
            assert!((original - extracted).abs() < 1e-10);
        }
    }
    
    #[test]
    fn test_capacity_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let test_img_path = temp_dir.path().join("test.png");
        
        utils::create_test_image(100, 100, test_img_path.to_str().unwrap()).unwrap();
        
        let img = image::open(test_img_path).unwrap().to_rgb8();
        let steganographer = VectorSteganographer::new();
        
        let capacity = steganographer.calculate_capacity(&img);
        
        // 100x100 pixels * 3 channels * 2 bits per channel / 8 bits per byte
        let expected_capacity = 100 * 100 * 3 * 2 / 8;
        assert_eq!(capacity, expected_capacity);
    }
    
    #[test]
    fn test_has_embedded_data() {
        let temp_dir = TempDir::new().unwrap();
        let test_img_path = temp_dir.path().join("test.png");
        let encoded_img_path = temp_dir.path().join("encoded.png");
        
        utils::create_test_image(256, 256, test_img_path.to_str().unwrap()).unwrap();
        
        let steganographer = VectorSteganographer::new();
        
        // Original image should not have embedded data
        assert!(!steganographer.has_embedded_data(test_img_path.to_str().unwrap()));
        
        // Embed data
        let test_vector = create_test_vector();
        let test_meta = create_test_meta();
        
        steganographer.embed_vector(
            test_img_path.to_str().unwrap(),
            &test_vector,
            &test_meta,
            encoded_img_path.to_str().unwrap(),
        ).unwrap();
        
        // Encoded image should have embedded data
        assert!(steganographer.has_embedded_data(encoded_img_path.to_str().unwrap()));
    }
    
    #[test]
    fn test_insufficient_capacity() {
        let temp_dir = TempDir::new().unwrap();
        let test_img_path = temp_dir.path().join("small.png");
        let encoded_img_path = temp_dir.path().join("encoded.png");
        
        // Create very small image
        utils::create_test_image(10, 10, test_img_path.to_str().unwrap()).unwrap();
        
        let steganographer = VectorSteganographer::new();
        let test_vector = create_test_vector(); // 512 f64 values = 4KB+
        let test_meta = create_test_meta();
        
        // Should fail due to insufficient capacity
        let result = steganographer.embed_vector(
            test_img_path.to_str().unwrap(),
            &test_vector,
            &test_meta,
            encoded_img_path.to_str().unwrap(),
        );
        
        assert!(matches!(result, Err(crate::error::RealMirError::Steganography(SteganographyError::InsufficientCapacity))));
    }
    
    #[test]
    fn test_min_image_size_calculation() {
        let (width, height) = utils::min_image_size_for_vector(512, 2);
        
        // Should be reasonable dimensions that can hold a 512-element vector
        assert!(width >= 64);
        assert!(height >= 64);
        assert!(width * height >= 10000); // Should have enough pixels
    }
}