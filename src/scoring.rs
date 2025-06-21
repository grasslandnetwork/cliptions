//! Scoring strategies and payout calculation for RealMir
//! 
//! This module implements various scoring strategies for calculating similarity between
//! image and text embeddings, as well as payout calculation based on rankings.

use ndarray::Array1;
use std::sync::Arc;
use crate::embedder::{EmbedderTrait, cosine_similarity};
use crate::error::{ScoringError, Result};
use crate::types::{Participant, ScoringResult};

/// Trait for scoring strategies
/// 
/// This corresponds to the Python IScoringStrategy interface
pub trait ScoringStrategy: Send + Sync {
    /// Calculate the similarity score between image and text features
    /// 
    /// # Arguments
    /// * `image_features` - The embedding vector for the image
    /// * `text_features` - The embedding vector for the text
    /// * `baseline_features` - Optional baseline features for adjustment
    /// 
    /// # Returns
    /// The calculated similarity score
    fn calculate_score(
        &self,
        image_features: &Array1<f64>,
        text_features: &Array1<f64>,
        baseline_features: Option<&Array1<f64>>,
    ) -> Result<f64>;
    
    /// Get the name of this scoring strategy
    fn name(&self) -> &str;
}

/// Raw cosine similarity scoring strategy
/// 
/// This strategy calculates the raw dot product between normalized vectors,
/// which is equivalent to cosine similarity for normalized vectors.
#[derive(Debug, Clone)]
pub struct RawSimilarityStrategy;

impl RawSimilarityStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RawSimilarityStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoringStrategy for RawSimilarityStrategy {
    fn calculate_score(
        &self,
        image_features: &Array1<f64>,
        text_features: &Array1<f64>,
        _baseline_features: Option<&Array1<f64>>,
    ) -> Result<f64> {
        cosine_similarity(text_features, image_features)
    }
    
    fn name(&self) -> &str {
        "RawSimilarity"
    }
}

/// Baseline-adjusted similarity scoring strategy
/// 
/// This strategy adjusts the raw similarity by comparing it to a baseline,
/// which helps differentiate between meaningful and random matches.
#[derive(Debug, Clone)]
pub struct BaselineAdjustedStrategy;

impl BaselineAdjustedStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BaselineAdjustedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoringStrategy for BaselineAdjustedStrategy {
    fn calculate_score(
        &self,
        image_features: &Array1<f64>,
        text_features: &Array1<f64>,
        baseline_features: Option<&Array1<f64>>,
    ) -> Result<f64> {
        let baseline_features = baseline_features
            .ok_or(ScoringError::MissingBaseline)?;
        
        // Calculate raw similarity
        let raw_score = cosine_similarity(text_features, image_features)?;
        
        // Calculate baseline similarity
        let baseline_score = cosine_similarity(baseline_features, image_features)?;
        
        // Adjust score relative to baseline
        let adjusted_score = if baseline_score >= 1.0 {
            // Avoid division by zero
            raw_score
        } else {
            (raw_score - baseline_score) / (1.0 - baseline_score)
        };
        
        Ok(adjusted_score.max(0.0))
    }
    
    fn name(&self) -> &str {
        "BaselineAdjusted"
    }
}

/// Score validator for validating guesses and calculating scores
/// 
/// This corresponds to the Python ScoreValidator class
pub struct ScoreValidator<E: EmbedderTrait, S: ScoringStrategy> {
    embedder: Arc<E>,
    scoring_strategy: Arc<S>,
    baseline_text: String,
    max_tokens: usize,
    baseline_features: Option<Array1<f64>>,
}

impl<E: EmbedderTrait, S: ScoringStrategy> ScoreValidator<E, S> {
    /// Create a new score validator
    pub fn new(embedder: E, scoring_strategy: S) -> Self {
        let mut validator = Self {
            embedder: Arc::new(embedder),
            scoring_strategy: Arc::new(scoring_strategy),
            baseline_text: "[UNUSED]".to_string(),
            max_tokens: 77, // CLIP's maximum token limit
            baseline_features: None,
        };
        validator.init_baseline().ok(); // Initialize baseline, ignore errors for now
        validator
    }
    
    /// Set custom baseline text
    pub fn with_baseline_text(mut self, baseline_text: String) -> Result<Self> {
        self.baseline_text = baseline_text;
        self.init_baseline()?;
        Ok(self)
    }
    
    /// Initialize baseline features
    fn init_baseline(&mut self) -> Result<()> {
        self.baseline_features = Some(self.embedder.get_text_embedding(&self.baseline_text)?);
        Ok(())
    }
    
    /// Check if guess meets basic validity criteria
    pub fn validate_guess(&self, guess: &str) -> bool {
        // Check if guess is a string with content
        if guess.is_empty() || guess.trim().is_empty() {
            return false;
        }
        
        // CLIP can handle up to 77 tokens, but we'll estimate
        // Average token is ~4 characters in English, so ~300 chars
        // This is a rough estimate; the actual tokenizer would be more accurate
        if guess.len() > 300 {
            return false;
        }
        
        true
    }
    
    /// Calculate adjusted score for a guess
    pub fn calculate_adjusted_score(&self, image_features: &Array1<f64>, guess: &str) -> Result<f64> {
        if !self.validate_guess(guess) {
            return Ok(0.0);
        }
        
        // Encode text
        let text_features = self.embedder.get_text_embedding(guess)?;
        
        // Use the strategy to calculate the score
        self.scoring_strategy.calculate_score(
            image_features,
            &text_features,
            self.baseline_features.as_ref(),
        )
    }
    
    /// Get image embedding for a given image path
    /// 
    /// This is a convenience method for Python bindings
    pub fn get_image_embedding(&self, image_path: &str) -> Result<Array1<f64>> {
        self.embedder.get_image_embedding(image_path)
    }
}

/// Calculate rankings for guesses based on similarity to target image
/// 
/// # Arguments
/// * `target_image_path` - Path to the target image
/// * `guesses` - List of text guesses to rank
/// * `validator` - Score validator to use
/// 
/// # Returns
/// List of tuples (guess, similarity) sorted by similarity (highest to lowest)
pub fn calculate_rankings<E: EmbedderTrait, S: ScoringStrategy>(
    target_image_path: &str,
    guesses: &[String],
    validator: &ScoreValidator<E, S>,
) -> Result<Vec<(String, f64)>> {
    if guesses.is_empty() {
        return Err(ScoringError::EmptyGuesses.into());
    }
    
    // Get target image embedding
    let image_embedding = validator.embedder.get_image_embedding(target_image_path)?;
    
    // Calculate adjusted similarity for each guess
    let mut similarities = Vec::new();
    for guess in guesses {
        let adjusted_score = validator.calculate_adjusted_score(&image_embedding, guess)?;
        similarities.push((guess.clone(), adjusted_score));
    }
    
    // Sort by similarity score (highest to lowest)
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    Ok(similarities)
}

/// Calculate payouts based on rankings
/// 
/// The payout calculation uses a position-based scoring system where:
/// - Scores are based only on position (1st, 2nd, etc), not similarity values
/// - Equal similarity scores get equal payouts (ties split the combined payout)
/// - Scores sum to 1.0 to distribute full prize pool
/// - Higher positions get proportionally higher scores
/// 
/// # Arguments
/// * `ranked_results` - List of (guess, similarity) tuples sorted by similarity
/// * `prize_pool` - Total amount to distribute
/// 
/// # Returns
/// List of payouts corresponding to ranked_results
pub fn calculate_payouts(ranked_results: &[(String, f64)], prize_pool: f64) -> Result<Vec<f64>> {
    if prize_pool <= 0.0 {
        return Err(ScoringError::InvalidPrizePool { amount: prize_pool }.into());
    }
    
    if ranked_results.is_empty() {
        return Ok(Vec::new());
    }
    
    let total_guesses = ranked_results.len();
    let denominator: usize = (1..=total_guesses).sum();
    
    // Group positions by similarity score
    let mut groups = Vec::new();
    let mut current_group = Vec::new();
    let mut current_similarity: Option<f64> = None;
    
    for (guess, similarity) in ranked_results {
        match current_similarity {
            Some(sim) if (sim - similarity).abs() < f64::EPSILON => {
                current_group.push((guess.clone(), *similarity));
            }
            _ => {
                if !current_group.is_empty() {
                    groups.push(current_group);
                }
                current_group = vec![(guess.clone(), *similarity)];
                current_similarity = Some(*similarity);
            }
        }
    }
    
    if !current_group.is_empty() {
        groups.push(current_group);
    }
    
    // Calculate payouts
    let mut payouts = Vec::new();
    let mut position = 0;
    
    for group in groups {
        // Calculate total points for this group's positions
        let group_size = group.len();
        let group_points: usize = (0..group_size)
            .map(|i| total_guesses - (position + i))
            .sum();
        
        // Split points equally among tied positions
        let points_per_position = group_points as f64 / group_size as f64;
        let score = points_per_position / denominator as f64;
        
        // Add same payout for each tied position
        for _ in 0..group_size {
            payouts.push(score * prize_pool);
        }
        
        position += group_size;
    }
    
    Ok(payouts)
}

/// Process participants and calculate their scores and payouts
pub fn process_participants<E: EmbedderTrait, S: ScoringStrategy>(
    participants: &[Participant],
    target_image_path: &str,
    prize_pool: f64,
    validator: &ScoreValidator<E, S>,
) -> Result<Vec<ScoringResult>> {
    if participants.is_empty() {
        return Ok(Vec::new());
    }
    
    // Extract guesses
    let guesses: Vec<String> = participants.iter()
        .map(|p| p.guess.text.clone())
        .collect();
    
    // Calculate rankings
    let ranked_results = calculate_rankings(target_image_path, &guesses, validator)?;
    
    // Calculate payouts
    let payouts = calculate_payouts(&ranked_results, prize_pool)?;
    
    // Create scoring results
    let mut results = Vec::new();
    for (i, ((guess, score), payout)) in ranked_results.iter().zip(payouts.iter()).enumerate() {
        // Find the participant with this guess
        if let Some(participant) = participants.iter().find(|p| &p.guess.text == guess) {
            let result = ScoringResult::new(participant.clone(), *score)
                .with_adjusted_score(*score)
                .with_rank(i + 1)
                .with_payout(*payout);
            results.push(result);
        }
    }
    
    Ok(results)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedder::MockEmbedder;
    
    #[test]
    fn test_raw_similarity_strategy() {
        let strategy = RawSimilarityStrategy::new();
        let embedder = MockEmbedder::new(128);
        
        let img_features = embedder.get_image_embedding("test.jpg").unwrap();
        let txt_features = embedder.get_text_embedding("test text").unwrap();
        
        let score = strategy.calculate_score(&img_features, &txt_features, None).unwrap();
        
        // Score should be between -1 and 1
        assert!(score >= -1.0 && score <= 1.0);
    }
    
    #[test]
    fn test_baseline_adjusted_strategy() {
        let strategy = BaselineAdjustedStrategy::new();
        let embedder = MockEmbedder::new(128);
        
        let img_features = embedder.get_image_embedding("test.jpg").unwrap();
        let txt_features = embedder.get_text_embedding("test text").unwrap();
        let baseline_features = embedder.get_text_embedding("[UNUSED]").unwrap();
        
        let score = strategy.calculate_score(&img_features, &txt_features, Some(&baseline_features)).unwrap();
        
        // Score should be >= 0 for baseline adjusted
        assert!(score >= 0.0);
    }
    
    #[test]
    fn test_score_validator() {
        let embedder = MockEmbedder::new(128);
        let strategy = BaselineAdjustedStrategy::new();
        let validator = ScoreValidator::new(embedder, strategy);
        
        // Valid guess
        assert!(validator.validate_guess("valid guess"));
        
        // Invalid guesses
        assert!(!validator.validate_guess(""));
        assert!(!validator.validate_guess("   "));
        assert!(!validator.validate_guess(&"x".repeat(400))); // Too long
    }
    
    #[test]
    fn test_calculate_rankings() {
        let embedder = MockEmbedder::new(128);
        let strategy = BaselineAdjustedStrategy::new();
        let validator = ScoreValidator::new(embedder, strategy);
        
        let guesses = vec![
            "guess1".to_string(),
            "guess2".to_string(),
            "guess3".to_string(),
        ];
        
        let rankings = calculate_rankings("test.jpg", &guesses, &validator).unwrap();
        
        assert_eq!(rankings.len(), 3);
        
        // Should be sorted by score (highest first)
        for i in 1..rankings.len() {
            assert!(rankings[i-1].1 >= rankings[i].1);
        }
    }
    
    #[test]
    fn test_calculate_payouts_no_ties() {
        let ranked_results = vec![
            ("first".to_string(), 0.9),
            ("second".to_string(), 0.7),
            ("third".to_string(), 0.5),
        ];
        
        let payouts = calculate_payouts(&ranked_results, 100.0).unwrap();
        
        assert_eq!(payouts.len(), 3);
        
        // Total should equal prize pool
        let total: f64 = payouts.iter().sum();
        assert!((total - 100.0).abs() < 1e-10);
        
        // First place should get the most
        assert!(payouts[0] > payouts[1]);
        assert!(payouts[1] > payouts[2]);
    }
    
    #[test]
    fn test_calculate_payouts_with_ties() {
        let ranked_results = vec![
            ("first".to_string(), 0.9),
            ("tied_second_a".to_string(), 0.7),
            ("tied_second_b".to_string(), 0.7),
            ("fourth".to_string(), 0.5),
        ];
        
        let payouts = calculate_payouts(&ranked_results, 100.0).unwrap();
        
        assert_eq!(payouts.len(), 4);
        
        // Total should equal prize pool
        let total: f64 = payouts.iter().sum();
        assert!((total - 100.0).abs() < 1e-10);
        
        // Tied positions should have equal payouts
        assert!((payouts[1] - payouts[2]).abs() < 1e-10);
        
        // First should be highest, tied second should be equal, fourth should be lowest
        assert!(payouts[0] > payouts[1]);
        assert!(payouts[1] > payouts[3]);
    }
    
    #[test]
    fn test_invalid_prize_pool() {
        let ranked_results = vec![("test".to_string(), 0.5)];
        
        let result = calculate_payouts(&ranked_results, -10.0);
        assert!(matches!(result, Err(crate::error::RealMirError::Scoring(ScoringError::InvalidPrizePool { .. }))));
        
        let result = calculate_payouts(&ranked_results, 0.0);
        assert!(matches!(result, Err(crate::error::RealMirError::Scoring(ScoringError::InvalidPrizePool { .. }))));
    }
    
    #[test]
    fn test_empty_guesses() {
        let embedder = MockEmbedder::new(128);
        let strategy = BaselineAdjustedStrategy::new();
        let validator = ScoreValidator::new(embedder, strategy);
        
        let result = calculate_rankings("test.jpg", &[], &validator);
        assert!(matches!(result, Err(crate::error::RealMirError::Scoring(ScoringError::EmptyGuesses))));
    }
}