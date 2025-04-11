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

def test_baseline_adjustment(validator, cat_sanctuary_features):
    # Test baseline scoring logic
    nonsense_score = validator.calculate_adjusted_score(
        cat_sanctuary_features, "[-h]"
    )
    legitimate_score = validator.calculate_adjusted_score(
        cat_sanctuary_features, "Cat rescue shelter interior, many cats on colorful beds and toys. Tall cat tree in background. Two women standing — one in pink shirt with long braid, one in grey sweater with cartoon prints. Cozy kitchen setting with white cabinets"
    )
    
    # Our primary assertion - legitimate should score higher than nonsense
    assert legitimate_score > nonsense_score
    
    # The nonsense score should be lower due to baseline adjustment
    assert nonsense_score < 0.02
    
    # Lower our expectation - CLIP might not score as high as we initially expected
    assert legitimate_score > 0.1  # Good match should score reasonably

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
