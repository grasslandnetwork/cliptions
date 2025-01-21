import unittest
from random_number_guess import (
    GameEngine, Player, PlayerRegistry, 
    RandomNumberGenerator, DefaultScoringStrategy,
    LinearScoringStrategy, ExponentialScoringStrategy,
    PrizeDistributor, ConsoleGameOutput
)
import time

class TestGuessingGame(unittest.TestCase):
    def setUp(self):
        self.player_registry = PlayerRegistry(entry_fee=10.0, platform_fee_percent=0.2)
        self.number_generator = RandomNumberGenerator()
        self.scoring_strategy = DefaultScoringStrategy()
        self.prize_distributor = PrizeDistributor()
        self.output_handler = ConsoleGameOutput()
        
        self.game = GameEngine(
            self.player_registry,
            self.number_generator,
            self.scoring_strategy,
            self.prize_distributor,
            self.output_handler
        )

    def test_example_scenario(self):
        """Test the example scenario from the documentation:
        Target = 50
        Players guess: 48, 52, 45, 60, 70
        Expected scores: 0.33, 0.33, 0.17, 0.09, 0.05
        Expected payouts (from $40 pool): $13.72, $13.72, $6.86, $3.74, $1.96
        """
        # Force target number for testing
        self.game.target_number = 50
        
        # Add players with their guesses
        self.game.player_registry.add_player("Player1", 48)
        self.game.player_registry.add_player("Player2", 52)
        self.game.player_registry.add_player("Player3", 45)
        self.game.player_registry.add_player("Player4", 60)
        self.game.player_registry.add_player("Player5", 70)

        # Run game calculations
        self.game._calculate_scores()
        self.game._distribute_prizes()

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
                self.game.player_registry.players[player_id].score,
                expected_score,
                places=2,
                msg=f"Score mismatch for {player_id}"
            )

        for player_id, expected_payout in expected_payouts.items():
            self.assertAlmostEqual(
                self.game.player_registry.players[player_id].payout,
                expected_payout,
                places=2,
                msg=f"Payout mismatch for {player_id}"
            )

    def test_invalid_guess_range(self):
        """Test that adding a player with an invalid guess raises ValueError"""
        with self.assertRaises(ValueError):
            self.game.player_registry.add_player("InvalidPlayer", 101)
        with self.assertRaises(ValueError):
            self.game.player_registry.add_player("InvalidPlayer", -1)

    def test_minimum_players(self):
        """Test that running a game with fewer than 2 players raises ValueError"""
        self.game.player_registry.add_player("SinglePlayer", 50)
        with self.assertRaises(ValueError):
            self.game.run_game()

    def test_platform_fee_calculation(self):
        """Test that platform fees are correctly calculated and separated from the prize pool."""
        # Add players
        self.game.player_registry.add_player("Player1", 48)
        self.game.player_registry.add_player("Player2", 52)
        self.game.player_registry.add_player("Player3", 45)

        # Calculate expected values
        total_fees_collected = 3 * 10.0  # 3 players * $10 fee
        expected_prize_pool = total_fees_collected * (1 - 0.2)  # 80% goes to prize pool
        expected_platform_fees = total_fees_collected * 0.2  # 20% goes to platform

        # Verify prize pool
        self.assertAlmostEqual(
            self.game.player_registry.prize_pool,
            expected_prize_pool,
            places=2,
            msg="Prize pool calculation is incorrect"
        )

        # Verify platform fees
        self.assertAlmostEqual(
            self.game.player_registry.platform_fees,
            expected_platform_fees,
            places=2,
            msg="Platform fee calculation is incorrect"
        )

        # Verify that total fees equal prize pool plus platform fees
        self.assertAlmostEqual(
            total_fees_collected,
            self.game.player_registry.prize_pool + self.game.player_registry.platform_fees,
            places=2,
            msg="Total fees don't match prize pool plus platform fees"
        )

    def test_payout_distribution(self):
        """Test that all payouts are distributed correctly and completely."""
        # Force target number for testing
        self.game.target_number = 50
        
        # Add players with their guesses
        test_players = [
            ("Player1", 48),
            ("Player2", 52),
            ("Player3", 45),
            ("Player4", 60),
            ("Player5", 70)
        ]
        
        for player_id, guess in test_players:
            self.game.player_registry.add_player(player_id, guess)

        initial_prize_pool = self.game.player_registry.prize_pool
        
        # Run game calculations
        self.game._calculate_scores()
        self.game._distribute_prizes()

        # Test 1: Verify all players received a payout
        for player_id, _ in test_players:
            self.assertGreater(
                self.game.player_registry.players[player_id].payout,
                0,
                f"{player_id} did not receive any payout"
            )

        # Test 2: Verify total payouts equal prize pool
        total_payouts = sum(player.payout for player in self.game.player_registry.players.values())
        self.assertAlmostEqual(
            total_payouts,
            initial_prize_pool,
            places=2,
            msg="Total payouts don't match prize pool"
        )

        # Test 3: Verify relative payout proportions
        # Players with same scores should get same payouts
        player1_payout = self.game.player_registry.players["Player1"].payout
        player2_payout = self.game.player_registry.players["Player2"].payout
        self.assertAlmostEqual(
            player1_payout,
            player2_payout,
            places=2,
            msg="Players with equal scores received different payouts"
        )

        # Test 4: Verify payout order (closer guesses should pay more)
        payouts = [(p.id, p.payout) for p in self.game.player_registry.players.values()]
        sorted_payouts = sorted(payouts, key=lambda x: x[1], reverse=True)
        
        expected_order = ["Player1", "Player2", "Player3", "Player4", "Player5"]
        actual_order = [player_id for player_id, _ in sorted_payouts]
        
        self.assertEqual(
            actual_order,
            expected_order,
            "Payout ordering is incorrect"
        )


# Add LSP-specific tests
class ScoringStrategyTests(unittest.TestCase):
    """Tests to verify LSP compliance for scoring strategies."""
    
    def setUp(self):
        self.strategies = [
            DefaultScoringStrategy(),
            LinearScoringStrategy(),
            ExponentialScoringStrategy()
        ]
    
    def test_score_range(self):
        """Verify all strategies return scores between 0 and 1."""
        for strategy in self.strategies:
            score = strategy.calculate_score(50, 50)
            self.assertEqual(score, 1.0, f"{strategy.__class__.__name__} perfect guess should score 1.0")
            
            score = strategy.calculate_score(0, 100)
            self.assertGreaterEqual(score, 0.0, f"{strategy.__class__.__name__} worst guess should be >= 0.0")
            self.assertLessEqual(score, 1.0, f"{strategy.__class__.__name__} worst guess should be <= 1.0")
    
    def test_equal_distance_symmetry(self):
        """Verify equal distances from target yield equal scores."""
        target = 50
        for strategy in self.strategies:
            score1 = strategy.calculate_score(40, target)
            score2 = strategy.calculate_score(60, target)
            self.assertAlmostEqual(score1, score2, 
                                 msg=f"{strategy.__class__.__name__} failed distance symmetry")

class TestScoringStrategyLSP(ScoringStrategyTests):
    """Run LSP compliance tests for scoring strategies."""
    pass

if __name__ == '__main__':
    unittest.main() 