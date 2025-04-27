import unittest
from calculate_scores_payout import calculate_payouts

class TestCalculatePayouts(unittest.TestCase):
    def test_two_player_payout(self):
        """Test payout distribution for 2 players.
        
        With 2 players:
        - 1st place should get 2/3 ≈ 0.67 of prize pool
        - 2nd place should get 1/3 ≈ 0.33 of prize pool
        """
        # Mock ranked results (actual similarity values don't matter for payout)
        ranked_results = [
            ("first", 0.9),
            ("second", 0.5)
        ]
        
        payouts = calculate_payouts(ranked_results, prize_pool=1.0)
        
        # Verify correct proportions
        self.assertAlmostEqual(payouts[0], 2/3, places=2)
        self.assertAlmostEqual(payouts[1], 1/3, places=2)
        
        # Verify sum equals prize pool
        self.assertAlmostEqual(sum(payouts), 1.0)

    def test_three_player_payout(self):
        """Test payout distribution for 3 players.
        
        With 3 players:
        - 1st place should get 3/6 = 0.50 of prize pool
        - 2nd place should get 2/6 ≈ 0.33 of prize pool
        - 3rd place should get 1/6 ≈ 0.17 of prize pool
        """
        ranked_results = [
            ("first", 0.9),
            ("second", 0.5),
            ("third", 0.1)
        ]
        
        payouts = calculate_payouts(ranked_results, prize_pool=1.0)
        
        # Verify correct proportions
        self.assertAlmostEqual(payouts[0], 3/6, places=2)
        self.assertAlmostEqual(payouts[1], 2/6, places=2)
        self.assertAlmostEqual(payouts[2], 1/6, places=2)
        
        # Verify sum equals prize pool
        self.assertAlmostEqual(sum(payouts), 1.0)

    def test_custom_prize_pool(self):
        """Test that payouts scale correctly with different prize pools."""
        ranked_results = [
            ("first", 0.9),
            ("second", 0.5)
        ]
        
        # Test with prize pool of 1
        payouts = calculate_payouts(ranked_results, prize_pool=1.0)
        
        # Verify correct proportions
        self.assertAlmostEqual(payouts[0], 1.0 * 2/3, places=2)
        self.assertAlmostEqual(payouts[1], 1.0 * 1/3, places=2)
        
        # Verify sum equals prize pool
        self.assertAlmostEqual(sum(payouts), 1.0)

    def test_equal_scores_for_equal_ranks(self):
        """Test that positions with equal similarity scores get equal payouts."""
        # Mock results where first two places have equal similarity
        ranked_results = [
            ("first", 0.9),
            ("first_tie", 0.9),  # Same similarity as first
            ("third", 0.5)
        ]
        
        payouts = calculate_payouts(ranked_results, prize_pool=1.0)
        
        # First two places should get equal payouts
        self.assertAlmostEqual(payouts[0], payouts[1])
        
        # Verify sum still equals prize pool
        self.assertAlmostEqual(sum(payouts), 1.0)

if __name__ == '__main__':
    unittest.main() 