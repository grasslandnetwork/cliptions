import pytest
import numpy as np
import sys
import os
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from scoring_strategies import RawSimilarityStrategy, BaselineAdjustedStrategy

@pytest.fixture
def image_features():
    return np.array([0.1, 0.2, 0.3])

@pytest.fixture
def text_features():
    return np.array([0.3, 0.2, 0.1])

@pytest.fixture
def baseline_features():
    return np.array([0.05, 0.05, 0.05])

def test_raw_similarity_strategy(image_features, text_features):
    # Create strategy
    strategy = RawSimilarityStrategy()
    
    # Calculate score
    score = strategy.calculate_score(image_features, text_features)
    
    # Verify it matches dot product
    expected = np.dot(image_features, text_features)
    assert score == pytest.approx(expected)
    
    # Ensure 1D vectors work properly
    image_features_2d = image_features.reshape(1, -1)
    score_2d = strategy.calculate_score(image_features_2d, text_features)
    assert score_2d == pytest.approx(expected)

def test_baseline_adjusted_strategy(image_features, text_features, baseline_features):
    # Create strategy
    strategy = BaselineAdjustedStrategy()
    
    # Calculate raw score (dot product)
    raw_score = np.dot(image_features, text_features)
    
    # Calculate baseline score
    baseline_score = np.dot(image_features, baseline_features)
    
    # Expected adjusted score
    expected = (raw_score - baseline_score) / (1 - baseline_score)
    
    # Calculate using strategy
    score = strategy.calculate_score(
        image_features, 
        text_features, 
        baseline_features=baseline_features
    )
    
    # Verify it matches expected formula
    assert score == pytest.approx(expected)

def test_baseline_adjusted_strategy_requires_baseline():
    # Create strategy
    strategy = BaselineAdjustedStrategy()
    
    # Should raise error when baseline_features is not provided
    with pytest.raises(ValueError):
        strategy.calculate_score(
            np.array([0.1, 0.2, 0.3]), 
            np.array([0.3, 0.2, 0.1])
        )

def test_strategies_handle_negative_scores():
    # Create features that will produce negative dot product
    image_features = np.array([0.1, 0.2, 0.3])
    text_features = np.array([-0.5, -0.5, -0.5])  # Will give negative dot product
    baseline_features = np.array([0.05, 0.05, 0.05])
    
    # Raw strategy should return negative value
    raw_strategy = RawSimilarityStrategy()
    raw_score = raw_strategy.calculate_score(image_features, text_features)
    assert raw_score < 0
    
    # Baseline strategy should return 0 for very negative scores
    # First set up a case that would give very negative adjusted score
    baseline_adj_strategy = BaselineAdjustedStrategy()
    score = baseline_adj_strategy.calculate_score(
        image_features, 
        text_features, 
        baseline_features=baseline_features
    )
    
    # Should be clamped to 0
    assert score == 0.0 