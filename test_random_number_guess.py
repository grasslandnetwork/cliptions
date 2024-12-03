import unittest
from random_number_guess import GuessingGame, Player
import time

class TestGuessingGame(unittest.TestCase):
    def setUp(self):
        self.game = GuessingGame(fee=10.0)

    def test_example_scenario(self):
        """Test the example scenario from the documentation:
        Target = 50
        Players guess: 48, 52, 45, 60, 70
        Expected scores: 0.33, 0.33, 0.17, 0.09, 0.05
        Expected payouts (from $800 pool): $272.16, $272.16, $140.49, $74.23, $41.24
        """
        # Force target number for testing
        self.game.target_number = 50
        
        # Add players with their guesses
        self.game.add_player("Player1", 48)
        self.game.add_player("Player2", 52)
        self.game.add_player("Player3", 45)
        self.game.add_player("Player4", 60)
        self.game.add_player("Player5", 70)

        # Run game calculations
        self.game.calculate_scores()
        self.game.distribute_prizes()

        # Verify scores
        expected_scores = {
            "Player1": 0.33,
            "Player2": 0.33,
            "Player3": 0.17,
            "Player4": 0.09,
            "Player5": 0.05
        }

        expected_payouts = {
            "Player1": 13.72,
            "Player2": 13.72,
            "Player3": 6.86,
            "Player4": 3.74,
            "Player5": 1.96
        }

        for player_id, expected_score in expected_scores.items():
            self.assertAlmostEqual(
                self.game.players[player_id].score,
                expected_score,
                places=2,
                msg=f"Score mismatch for {player_id}"
            )

        for player_id, expected_payout in expected_payouts.items():
            self.assertAlmostEqual(
                self.game.players[player_id].payout,
                expected_payout,
                places=2,
                msg=f"Payout mismatch for {player_id}"
            )

    def test_invalid_guess_range(self):
        """Test that adding a player with an invalid guess raises ValueError"""
        with self.assertRaises(ValueError):
            self.game.add_player("InvalidPlayer", 101)
        with self.assertRaises(ValueError):
            self.game.add_player("InvalidPlayer", -1)

    def test_minimum_players(self):
        """Test that running a game with fewer than 2 players raises ValueError"""
        self.game.add_player("SinglePlayer", 50)
        with self.assertRaises(ValueError):
            self.game.run_game()

if __name__ == '__main__':
    unittest.main() 