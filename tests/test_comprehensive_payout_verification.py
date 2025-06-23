#!/usr/bin/env python3
"""
Comprehensive Payout Verification Test Suite

This test suite specifically addresses the two key requirements:
1. Verifies that all players got paid
2. Verifies that they all got paid the correct amount

Tests multiple scenarios including edge cases, different player counts,
ties, and different scoring strategies.
"""

import unittest
import sys
import os
from decimal import Decimal, getcontext
from typing import Dict, List, Tuple, Optional

# Set high precision for decimal calculations
getcontext().prec = 28

# Add parent directory to path to import from root
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from random_number_guess import (
    GameEngine, Player, PlayerRegistry, 
    RandomNumberGenerator, DefaultScoringStrategy,
    LinearScoringStrategy, ExponentialScoringStrategy,
    PrizeDistributor, ConsoleGameOutput
)


class ComprehensivePayoutVerificationTest(unittest.TestCase):
    """
    Comprehensive test suite that verifies payout integrity across multiple scenarios.
    
    Each test method verifies:
    1. All players receive a payout > 0 (unless their score is 0)
    2. Total payouts exactly equal the prize pool
    3. Payout calculations match expected mathematical formulas
    4. Relative payout proportions are correct based on scores
    """
    
    def setUp(self):
        """Set up test environment with high precision for calculations."""
        self.test_scenarios = []
        self.detailed_logs = []
    
    def log_scenario(self, message: str):
        """Log detailed information about test scenarios."""
        self.detailed_logs.append(message)
        print(f"[TEST LOG] {message}")
    
    def create_game_with_strategy(self, scoring_strategy, entry_fee=10.0, platform_fee=0.2):
        """Create a game engine with specified strategy."""
        player_registry = PlayerRegistry(entry_fee, platform_fee)
        number_generator = RandomNumberGenerator()
        prize_distributor = PrizeDistributor()
        output_handler = ConsoleGameOutput()
        
        return GameEngine(
            player_registry,
            number_generator,
            scoring_strategy,
            prize_distributor,
            output_handler
        )
    
    def verify_payout_integrity(self, game: GameEngine, scenario_name: str, 
                              expected_total_payout: Optional[float] = None) -> Dict:
        """
        Core verification method that checks both requirements:
        1. All players got paid (non-zero payouts for non-zero scores)
        2. All players got paid the correct amount
        
        Returns detailed verification results.
        """
        players = game.player_registry.players
        prize_pool = game.player_registry.prize_pool
        
        if expected_total_payout is None:
            expected_total_payout = prize_pool
        
        verification_results = {
            'scenario': scenario_name,
            'total_players': len(players),
            'prize_pool': prize_pool,
            'all_players_paid': True,
            'correct_total_payout': True,
            'correct_individual_payouts': True,
            'player_details': [],
            'issues_found': []
        }
        
        self.log_scenario(f"\n=== VERIFYING SCENARIO: {scenario_name} ===")
        self.log_scenario(f"Players: {len(players)}, Prize Pool: ${prize_pool:.2f}")
        
        # Calculate total actual payouts
        total_actual_payouts = sum(player.payout for player in players.values())
        
        # REQUIREMENT 1: Verify all players got paid
        self.log_scenario("Checking Requirement 1: All players got paid")
        for player_id, player in players.items():
            player_detail = {
                'id': player_id,
                'guess': player.guess,
                'score': player.score,
                'payout': player.payout,
                'got_paid': player.payout > 0 or player.score == 0
            }
            
            if player.score > 0 and player.payout <= 0:
                verification_results['all_players_paid'] = False
                issue = f"Player {player_id} has score {player.score:.4f} but payout ${player.payout:.2f}"
                verification_results['issues_found'].append(issue)
                self.log_scenario(f"❌ ISSUE: {issue}")
            else:
                self.log_scenario(f"✅ Player {player_id}: Score={player.score:.4f}, Payout=${player.payout:.2f}")
            
            verification_results['player_details'].append(player_detail)
        
        # REQUIREMENT 2: Verify correct total payout amount
        self.log_scenario("Checking Requirement 2: Correct total payout amount")
        payout_difference = abs(total_actual_payouts - expected_total_payout)
        
        if payout_difference > 0.01:  # Allow for small floating point errors
            verification_results['correct_total_payout'] = False
            issue = f"Total payouts ${total_actual_payouts:.2f} != Prize pool ${expected_total_payout:.2f}"
            verification_results['issues_found'].append(issue)
            self.log_scenario(f"❌ ISSUE: {issue}")
        else:
            self.log_scenario(f"✅ Total payouts ${total_actual_payouts:.2f} matches prize pool ${expected_total_payout:.2f}")
        
        # REQUIREMENT 2 (continued): Verify individual payout calculations
        self.log_scenario("Checking Requirement 2: Individual payout calculations")
        total_score = sum(player.score for player in players.values())
        
        if total_score > 0:
            for player in players.values():
                expected_payout = (player.score / total_score) * prize_pool
                payout_error = abs(player.payout - expected_payout)
                
                if payout_error > 0.01:
                    verification_results['correct_individual_payouts'] = False
                    issue = f"Player {player.id}: Expected ${expected_payout:.2f}, got ${player.payout:.2f}"
                    verification_results['issues_found'].append(issue)
                    self.log_scenario(f"❌ ISSUE: {issue}")
                else:
                    self.log_scenario(f"✅ Player {player.id}: Payout calculation correct")
        
        # Summary
        if verification_results['all_players_paid'] and verification_results['correct_total_payout'] and verification_results['correct_individual_payouts']:
            self.log_scenario(f"🎉 SCENARIO {scenario_name} PASSED: All requirements verified!")
        else:
            self.log_scenario(f"❌ SCENARIO {scenario_name} FAILED: {len(verification_results['issues_found'])} issues found")
        
        return verification_results
    
    def test_original_example_scenario(self):
        """Test the original example from prompt.txt with detailed verification."""
        game = self.create_game_with_strategy(DefaultScoringStrategy())
        
        # Set target and add players as per original example
        game.target_number = 50
        test_players = [
            ("Player1", 48),
            ("Player2", 52), 
            ("Player3", 45),
            ("Player4", 60),
            ("Player5", 70)
        ]
        
        for player_id, guess in test_players:
            game.player_registry.add_player(player_id, guess)
        
        # Run calculations
        game._calculate_scores()
        game._distribute_prizes()
        
        # Verify both requirements
        results = self.verify_payout_integrity(game, "Original Example (5 players)")
        
        # Assert requirements are met
        self.assertTrue(results['all_players_paid'], 
                       f"Not all players got paid: {results['issues_found']}")
        self.assertTrue(results['correct_total_payout'],
                       f"Total payout incorrect: {results['issues_found']}")
        self.assertTrue(results['correct_individual_payouts'],
                       f"Individual payouts incorrect: {results['issues_found']}")
    
    def test_small_group_scenario(self):
        """Test with minimum viable group (2 players)."""
        game = self.create_game_with_strategy(LinearScoringStrategy())
        
        game.target_number = 30
        game.player_registry.add_player("Alice", 25)
        game.player_registry.add_player("Bob", 35)
        
        game._calculate_scores()
        game._distribute_prizes()
        
        results = self.verify_payout_integrity(game, "Small Group (2 players)")
        
        self.assertTrue(results['all_players_paid'])
        self.assertTrue(results['correct_total_payout'])
        self.assertTrue(results['correct_individual_payouts'])
    
    def test_large_group_scenario(self):
        """Test with larger group to ensure scalability."""
        game = self.create_game_with_strategy(ExponentialScoringStrategy())
        
        game.target_number = 42
        
        # Add 10 players with varying guesses
        test_players = [
            (f"Player{i}", 42 + (i-5)*3) for i in range(1, 11)
        ]
        
        for player_id, guess in test_players:
            # Ensure guesses are within valid range
            guess = max(0, min(100, guess))
            game.player_registry.add_player(player_id, guess)
        
        game._calculate_scores()
        game._distribute_prizes()
        
        results = self.verify_payout_integrity(game, "Large Group (10 players)")
        
        self.assertTrue(results['all_players_paid'])
        self.assertTrue(results['correct_total_payout'])
        self.assertTrue(results['correct_individual_payouts'])
    
    def test_tied_scores_scenario(self):
        """Test scenario where multiple players have identical scores."""
        game = self.create_game_with_strategy(DefaultScoringStrategy())
        
        game.target_number = 50
        
        # Create ties: two players at distance 5, two at distance 10
        game.player_registry.add_player("TiedA1", 45)  # distance 5
        game.player_registry.add_player("TiedA2", 55)  # distance 5  
        game.player_registry.add_player("TiedB1", 40)  # distance 10
        game.player_registry.add_player("TiedB2", 60)  # distance 10
        game.player_registry.add_player("Winner", 50)  # perfect score
        
        game._calculate_scores()
        game._distribute_prizes()
        
        results = self.verify_payout_integrity(game, "Tied Scores (multiple ties)")
        
        # Additional verification for ties
        tied_a_payout = game.player_registry.players["TiedA1"].payout
        tied_a2_payout = game.player_registry.players["TiedA2"].payout
        self.assertAlmostEqual(tied_a_payout, tied_a2_payout, places=2,
                              msg="Players with tied scores should have equal payouts")
        
        tied_b_payout = game.player_registry.players["TiedB1"].payout
        tied_b2_payout = game.player_registry.players["TiedB2"].payout
        self.assertAlmostEqual(tied_b_payout, tied_b2_payout, places=2,
                              msg="Players with tied scores should have equal payouts")
        
        self.assertTrue(results['all_players_paid'])
        self.assertTrue(results['correct_total_payout'])
        self.assertTrue(results['correct_individual_payouts'])
    
    def test_extreme_guesses_scenario(self):
        """Test scenario with extreme guesses (boundaries of 0-100)."""
        game = self.create_game_with_strategy(LinearScoringStrategy())
        
        game.target_number = 50
        
        # Add players with extreme guesses
        game.player_registry.add_player("MinGuess", 0)
        game.player_registry.add_player("MaxGuess", 100)
        game.player_registry.add_player("PerfectGuess", 50)
        game.player_registry.add_player("CloseGuess", 51)
        
        game._calculate_scores()
        game._distribute_prizes()
        
        results = self.verify_payout_integrity(game, "Extreme Guesses (0, 100, 50, 51)")
        
        self.assertTrue(results['all_players_paid'])
        self.assertTrue(results['correct_total_payout'])
        self.assertTrue(results['correct_individual_payouts'])
    
    def test_different_fee_structures(self):
        """Test with different entry fees and platform fee percentages."""
        scenarios = [
            (5.0, 0.1),   # $5 entry, 10% platform fee
            (25.0, 0.3),  # $25 entry, 30% platform fee
            (1.0, 0.05),  # $1 entry, 5% platform fee
        ]
        
        for entry_fee, platform_fee_pct in scenarios:
            game = self.create_game_with_strategy(DefaultScoringStrategy(), entry_fee, platform_fee_pct)
            
            game.target_number = 75
            game.player_registry.add_player("P1", 70)
            game.player_registry.add_player("P2", 80)
            game.player_registry.add_player("P3", 75)
            
            game._calculate_scores()
            game._distribute_prizes()
            
            scenario_name = f"Fee Structure (${entry_fee}, {platform_fee_pct*100}% platform)"
            results = self.verify_payout_integrity(game, scenario_name)
            
            self.assertTrue(results['all_players_paid'])
            self.assertTrue(results['correct_total_payout'])
            self.assertTrue(results['correct_individual_payouts'])
    
    def test_all_scoring_strategies(self):
        """Test all available scoring strategies to ensure payout integrity."""
        strategies = [
            ("Default", DefaultScoringStrategy()),
            ("Linear", LinearScoringStrategy()),
            ("Exponential", ExponentialScoringStrategy())
        ]
        
        for strategy_name, strategy in strategies:
            game = self.create_game_with_strategy(strategy)
            
            game.target_number = 33
            game.player_registry.add_player("A", 30)
            game.player_registry.add_player("B", 33)
            game.player_registry.add_player("C", 40)
            game.player_registry.add_player("D", 50)
            
            game._calculate_scores()
            game._distribute_prizes()
            
            results = self.verify_payout_integrity(game, f"{strategy_name} Scoring Strategy")
            
            self.assertTrue(results['all_players_paid'])
            self.assertTrue(results['correct_total_payout'])
            self.assertTrue(results['correct_individual_payouts'])
    
    def test_mathematical_precision(self):
        """Test mathematical precision with high-precision decimal arithmetic."""
        game = self.create_game_with_strategy(DefaultScoringStrategy())
        
        game.target_number = 50
        
        # Add players that will create repeating decimals in calculations
        game.player_registry.add_player("P1", 49)  # Score = 1/2 = 0.5
        game.player_registry.add_player("P2", 48)  # Score = 1/3 ≈ 0.333...
        game.player_registry.add_player("P3", 47)  # Score = 1/4 = 0.25
        
        game._calculate_scores()
        game._distribute_prizes()
        
        # Verify using high-precision arithmetic
        total_score = Decimal(str(game.player_registry.players["P1"].score)) + \
                     Decimal(str(game.player_registry.players["P2"].score)) + \
                     Decimal(str(game.player_registry.players["P3"].score))
        
        prize_pool = Decimal(str(game.player_registry.prize_pool))
        
        calculated_total_payout = Decimal('0')
        for player in game.player_registry.players.values():
            expected_payout = (Decimal(str(player.score)) / total_score) * prize_pool
            calculated_total_payout += expected_payout
        
        # The calculated total should equal the prize pool
        self.assertAlmostEqual(float(calculated_total_payout), 
                              float(prize_pool), places=10,
                              msg="High precision calculation failed")
        
        results = self.verify_payout_integrity(game, "Mathematical Precision Test")
        
        self.assertTrue(results['all_players_paid'])
        self.assertTrue(results['correct_total_payout']) 
        self.assertTrue(results['correct_individual_payouts'])


class PayoutIntegrityReportGenerator:
    """Generate detailed reports on payout integrity testing."""
    
    @staticmethod
    def generate_summary_report(test_results: List[Dict]) -> str:
        """Generate a comprehensive summary report."""
        total_tests = len(test_results)
        passed_tests = sum(1 for r in test_results if not r['issues_found'])
        
        report = f"""
        
=== COMPREHENSIVE PAYOUT VERIFICATION REPORT ===

Total Scenarios Tested: {total_tests}
Scenarios Passed: {passed_tests}
Scenarios Failed: {total_tests - passed_tests}
Success Rate: {passed_tests/total_tests*100:.1f}%

REQUIREMENT VERIFICATION SUMMARY:
✅ Requirement 1 (All players got paid): {'PASSED' if all(r['all_players_paid'] for r in test_results) else 'FAILED'}
✅ Requirement 2 (Correct payout amounts): {'PASSED' if all(r['correct_total_payout'] for r in test_results) else 'FAILED'}

DETAILED SCENARIO RESULTS:
"""
        
        for result in test_results:
            status = "✅ PASSED" if not result['issues_found'] else "❌ FAILED"
            report += f"  {result['scenario']}: {status}\n"
            if result['issues_found']:
                for issue in result['issues_found']:
                    report += f"    - {issue}\n"
        
        return report


if __name__ == '__main__':
    # Run comprehensive tests
    suite = unittest.TestLoader().loadTestsFromTestCase(ComprehensivePayoutVerificationTest)
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    
    print("\n" + "="*80)
    print("COMPREHENSIVE PAYOUT VERIFICATION COMPLETE")
    print("="*80)
    
    if result.wasSuccessful():
        print("🎉 ALL REQUIREMENTS VERIFIED:")
        print("   ✅ Requirement 1: All players got paid")
        print("   ✅ Requirement 2: All players got paid the correct amount")
    else:
        print("❌ SOME TESTS FAILED - CHECK LOGS ABOVE")
        print(f"   Failures: {len(result.failures)}")
        print(f"   Errors: {len(result.errors)}")