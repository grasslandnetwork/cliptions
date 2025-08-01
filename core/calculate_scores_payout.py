import sys
from .clip_embedder import ClipEmbedder
import numpy as np
import re
import torch
from .interfaces import IScoreValidator, IEmbedder
from .scoring_strategies import IScoringStrategy, BaselineAdjustedStrategy

def calculate_rankings(target_image_path, guesses, validator=None):
    """Calculate rankings for guesses based on similarity to target image.
    
    Args:
        target_image_path: Path to the target image
        guesses: List of text guesses to rank
        validator: Instance of IScoreValidator (defaults to ScoreValidator if None)
    
    Returns:
        List of tuples (guess, similarity) sorted by similarity (highest to lowest)
    """
    # Use dependency injection with default implementation
    validator = validator or ScoreValidator()
    
    # Get target image embedding
    image_embedding = validator.embedder.get_image_embedding(target_image_path)
    
    # Calculate adjusted similarity for each guess
    similarities = []
    for guess in guesses:
        adjusted_score = validator.calculate_adjusted_score(image_embedding, guess)
        similarities.append((guess, adjusted_score))
    
    # Sort by similarity score (highest to lowest)
    return sorted(similarities, key=lambda x: x[1], reverse=True)

def calculate_payouts(ranked_results, prize_pool):
    """Calculate payouts based on rankings.
    
    The payout calculation uses a position-based scoring system where:
    - Scores are based only on position (1st, 2nd, etc), not similarity values
    - Equal similarity scores get equal payouts (ties split the combined payout)
    - Scores sum to 1.0 to distribute full prize pool
    - Higher positions get proportionally higher scores
    
    For n players without ties:
    - Denominator = sum(1..n)
    - Each position's score = (n-position)/denominator
    
    For players with ties:
    - Players with equal similarity scores split their combined payout
    
    Args:
        ranked_results: List of (guess, similarity) tuples sorted by similarity
        prize_pool: Total amount to distribute
    
    Returns:
        List of payouts corresponding to ranked_results
    """
    total_guesses = len(ranked_results)
    denominator = sum(range(1, total_guesses + 1))
    
    # Group positions by similarity score
    groups = []
    current_group = []
    current_similarity = None
    
    for guess, similarity in ranked_results:
        if similarity != current_similarity:
            if current_group:
                groups.append(current_group)
            current_group = [(guess, similarity)]
            current_similarity = similarity
        else:
            current_group.append((guess, similarity))
    
    if current_group:
        groups.append(current_group)
    
    # Calculate payouts
    payouts = []
    position = 0
    
    for group in groups:
        # Calculate total points for this group's positions
        group_size = len(group)
        group_points = sum(total_guesses - (position + i) for i in range(group_size))
        
        # Split points equally among tied positions
        points_per_position = group_points / group_size
        score = points_per_position / denominator
        
        # Add same payout for each tied position
        payouts.extend([score * prize_pool] * group_size)
        position += group_size
    
    return payouts

def display_results(ranked_results, payouts, prize_pool):
    """Display rankings and payouts in a formatted way."""
    print("\nRankings and Payouts:")
    print("-" * 50)
    for i, ((guess, similarity), payout) in enumerate(zip(ranked_results, payouts), 1):
        print(f"{i}. \"{guess}\"")
        print(f"   Similarity score: {similarity:.4f}")
        print(f"   Payout: {payout:.9f}")
        print()
    
    print(f"Total prize pool: {prize_pool:.9f}")
    print(f"Total payout: {sum(payouts):.9f}")

class ScoreValidator(IScoreValidator):
    def __init__(self, embedder=None, scoring_strategy=None):
        """Initialize the score validator.
        
        Args:
            embedder: Optional implementation of IEmbedder (defaults to ClipEmbedder)
            scoring_strategy: Optional implementation of IScoringStrategy
                             (defaults to BaselineAdjustedStrategy)
        """
        self.embedder = embedder or ClipEmbedder()
        self.scoring_strategy = scoring_strategy or BaselineAdjustedStrategy()
        self.baseline_text = "[UNUSED]"
        self.max_tokens = 77  # CLIP's maximum token limit
        self._init_baseline()
    
    def _init_baseline(self):
        """Initialize baseline score for relative scoring"""
        self.baseline_features = self.embedder.get_text_embedding(self.baseline_text)
    
    def validate_guess(self, guess: str) -> bool:
        """Check if guess meets basic validity criteria"""
        # Check if guess is a string with content
        if not guess or not isinstance(guess, str) or len(guess.strip()) == 0:
            return False
        
        # CLIP can handle up to 77 tokens, but we'll estimate
        # Average token is ~4 characters in English, so ~300 chars
        # This is a rough estimate; the actual tokenizer would be more accurate
        if len(guess) > 300:  # Conservative estimate
            return False
            
        return True
    
    def calculate_adjusted_score(self, image_features, guess: str) -> float:
        """Calculate score with baseline adjustment"""
        if not self.validate_guess(guess):
            return 0.0
            
        # Encode text
        text_features = self.embedder.get_text_embedding(guess)
        
        # Use the strategy to calculate the score
        return self.scoring_strategy.calculate_score(
            image_features=image_features,
            text_features=text_features,
            baseline_features=self.baseline_features
        )

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python3 calculate_guess_ranking.py <target_image_path> <prize_pool> <guess1> <guess2> [guess3 ...]")
        print("Note: prize_pool can be a small decimal value (up to 9 decimal places)")
        sys.exit(1)
        
    target_path = sys.argv[1]
    prize_pool = float(sys.argv[2])
    
    # Validate inputs
    if prize_pool <= 0:
        print("Error: Prize pool must be greater than zero")
        sys.exit(1)
        
    if len(sys.argv) < 4:
        print("Usage: python3 calculate_guess_ranking.py <target_image_path> <prize_pool> <guess1> <guess2> [guess3 ...]")
        print("Note: prize_pool can be a small decimal value (up to 9 decimal places)")
        sys.exit(1)
    
    guesses = sys.argv[3:]
    
    if len(guesses) == 0:
        print("Error: At least one guess must be provided")
        sys.exit(1)
    
    try:
        # Calculate rankings
        ranked_results = calculate_rankings(target_path, guesses)
        
        # Calculate payouts
        payouts = calculate_payouts(ranked_results, prize_pool)
        
        # Display results
        display_results(ranked_results, payouts, prize_pool)
    except Exception as e:
        print(f"Error processing results: {e}")
        sys.exit(1) 