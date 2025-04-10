import pytest
from calculate_scores_payout import ScoreValidator
import numpy as np
import os
from PIL import Image
import torch

@pytest.fixture
def validator():
    return ScoreValidator()

@pytest.fixture
def sample_image_features(validator):
    # Create dummy image features matching CLIP's output shape
    return np.random.randn(1, 512).astype(np.float32)

@pytest.fixture
def cat_sanctuary_features(validator):
    """Load features from an actual cat sanctuary image"""
    test_image_path = os.path.join('tests', 'test_images', 'cat_sanctuary.jpg')
    
    if not os.path.exists(test_image_path):
        test_image_path = os.path.join(os.path.dirname(__file__), 'test_images', 'cat_sanctuary.jpg')
    
    # Use the same embedder as production code
    return validator.embedder.get_image_embedding(test_image_path)

def test_length_filtering(validator):
    # Test minimum length requirement
    assert validator.validate_guess("test") is False  # 4 chars
    assert validator.validate_guess("valid") is True   # 5 chars
    assert validator.validate_guess(" longer guess ") is True  # 12 chars

def test_special_char_penalty(validator):
    # Test special character limits
    assert validator.validate_guess("normal guess") is True
    assert validator.validate_guess("guess-with-hyphen") is True  # 1 special
    assert validator.validate_guess("[bracketed]") is True        # 2 specials
    assert validator.validate_guess("{-over-}") is False          # 3 specials

def test_baseline_adjustment(validator, cat_sanctuary_features):
    # Test baseline scoring logic
    nonsense_score = validator.calculate_adjusted_score(
        cat_sanctuary_features, "[-h]"
    )
    legitimate_score = validator.calculate_adjusted_score(
        cat_sanctuary_features, "A cat sanctuary with caretakers"
    )
    
    assert legitimate_score > nonsense_score
    assert nonsense_score < 0.2  # Should be heavily penalized
    # We can add more specific assertions since results should be deterministic
    assert legitimate_score > 0.5  # Good match should score well

def test_full_scoring_flow(validator):
    # Test integration of all components
    valid_guess = "Cat sanctuary with staff wearing colorful uniforms"
    invalid_guess = "[x]"
    
    assert validator.validate_guess(valid_guess) is True
    assert validator.validate_guess(invalid_guess) is False
    
    # Test scoring returns 0 for invalid guesses
    assert validator.calculate_adjusted_score(
        np.zeros((1, 512)), invalid_guess
    ) == 0.0

def test_special_char_penalty_calculation(validator, sample_image_features):
    # Test progressive penalty application
    base_guess = "Guess without special chars"
    base_score = validator.calculate_adjusted_score(sample_image_features, base_guess)
    
    guess1 = "Guess-with-hyphen"  # 1 special
    score1 = validator.calculate_adjusted_score(sample_image_features, guess1)
    
    guess2 = "[Guess-with-brackets]"  # 2 specials
    score2 = validator.calculate_adjusted_score(sample_image_features, guess2)
    
    assert base_score > score1 > score2
    assert np.isclose(score1, base_score * 0.95, rtol=0.01)
    assert np.isclose(score2, base_score * 0.90, rtol=0.01) 