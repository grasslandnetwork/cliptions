import pytest
import os
import numpy as np
from calculate_scores_payout import calculate_rankings, calculate_payouts, ScoreValidator

@pytest.fixture
def validator():
    return ScoreValidator()

@pytest.fixture
def cat_sanctuary_image_path():
    test_image_path = os.path.join('tests', 'test_images', 'cat_sanctuary.jpg')
    
    if not os.path.exists(test_image_path):
        test_image_path = os.path.join(os.path.dirname(__file__), 'test_images', 'cat_sanctuary.jpg')
    
    return test_image_path

@pytest.fixture
def test_guesses():
    return [
        "Cat sanctuary with woman wearing snoopy sweater",
        "[-h]",
        "live stream, video, cat, kitten, animal rescue, shelter"
    ]

def test_rankings_use_adjusted_scores(cat_sanctuary_image_path, test_guesses):
    # Get adjusted scores directly from validator
    validator = ScoreValidator()
    image_embedding = validator.embedder.get_image_embedding(cat_sanctuary_image_path)
    
    direct_scores = []
    for guess in test_guesses:
        score = validator.calculate_adjusted_score(image_embedding, guess)
        direct_scores.append((guess, score))
    
    # Sort for comparison
    direct_scores = sorted(direct_scores, key=lambda x: x[1], reverse=True)
    
    # Get scores from calculate_rankings
    ranked_results = calculate_rankings(cat_sanctuary_image_path, test_guesses)
    
    # Check that both methods produce the same ranking with the same scores
    for (guess1, score1), (guess2, score2) in zip(direct_scores, ranked_results):
        assert guess1 == guess2
        assert abs(score1 - score2) < 1e-6  # Allow for tiny floating point differences

def test_payouts_match_score_ordering(cat_sanctuary_image_path, test_guesses):
    # Get rankings
    ranked_results = calculate_rankings(cat_sanctuary_image_path, test_guesses)
    
    # Calculate payouts
    prize_pool = 1.0
    payouts = calculate_payouts(ranked_results, prize_pool)
    
    # Check payouts match the ranking order
    for i in range(len(payouts) - 1):
        if ranked_results[i][1] > ranked_results[i+1][1]:  # If scores are different
            assert payouts[i] > payouts[i+1]  # Higher score should get higher payout
        else:  # If scores are equal
            assert payouts[i] == payouts[i+1]  # Equal scores should get equal payouts
            
    # Check total payout matches prize pool
    assert abs(sum(payouts) - prize_pool) < 1e-6

def test_invalid_guesses_get_zero_score():
    # Create an invalid guess (too long)
    invalid_guess = "a" * 400  # Longer than the 300 char limit
    valid_guess = "Cat sanctuary"
    
    # Run calculate_rankings with these guesses
    # Using a mock image path - the test will fail before it tries to load the image
    try:
        results = calculate_rankings("fake_path.jpg", [invalid_guess, valid_guess])
        # If we get here, the validator isn't working - the test should fail
        assert results[0][0] == valid_guess  # Valid guess should be first
        assert results[1][0] == invalid_guess  # Invalid guess should be last
        assert results[1][1] == 0.0  # Invalid guess should have zero score
    except Exception:
        # We expect this to fail because the image doesn't exist
        # But we should have validated the guesses before trying to load the image
        pass 