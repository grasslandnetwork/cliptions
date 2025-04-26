import unittest
import json
import os
import sys
from unittest.mock import patch, MagicMock, mock_open, call
import numpy as np

# Add parent directory to path to import the module
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from process_round_payouts import process_round_payouts, process_all_rounds, get_validator_for_round, LegacyScoreValidator

class TestProcessRoundPayouts(unittest.TestCase):
    @patch('process_round_payouts.get_validator_for_round')
    def test_process_round_payouts(self, mock_get_validator):
        """Test processing payouts for a round."""
        # Create test guesses
        test_guesses = {
            "test_round": {
                "participants": [
                    {
                        "username": "user1",
                        "wallet": "wallet1",
                        "commitment": "commit1",
                        "guess": "cats playing",
                        "salt": "salt1",
                        "valid": True,
                        "score": None
                    },
                    {
                        "username": "user2",
                        "wallet": "wallet2",
                        "commitment": "commit2",
                        "guess": "dogs running",
                        "salt": "salt2",
                        "valid": True,
                        "score": None
                    }
                ],
                "target_image": "test_round/dummy_image.jpg",
                "target_time": "test_time",
                "total_payout": None
            }
        }
        
        # Create JSON string for mocking
        test_guesses_json = json.dumps(test_guesses)
        
        # Mock the validator
        mock_validator = MagicMock()
        mock_get_validator.return_value = mock_validator
        
        # Mock image features
        mock_image_features = np.ones((512))
        mock_validator.embedder.get_image_embedding.return_value = mock_image_features
        
        # Mock scores - user1 has higher score than user2
        mock_validator.calculate_adjusted_score.side_effect = [0.8, 0.4]
        
        # Create a mock for open function with our test data
        mock_file = mock_open(read_data=test_guesses_json)
        
        # Mock Path.exists to return True
        with patch('process_round_payouts.Path') as MockPath:
            mock_path_instance = MagicMock()
            MockPath.return_value = mock_path_instance
            mock_path_instance.exists.return_value = True
            
            with patch('builtins.open', mock_file):
                result = process_round_payouts("test_round", prize_pool=100.0, save_to_file=False)
        
        # Verify validator was selected for the round
        mock_get_validator.assert_called_once_with("test_round")
        
        # Extract the updated participants from the mocked calls
        # We need to parse the JSON that would have been written to the file
        updated_participants = [
            {"username": "user1", "wallet": "wallet1", "commitment": "commit1", 
             "guess": "cats playing", "salt": "salt1", "valid": True, 
             "score": 0.8, "payout": 2/3 * 100.0},
            {"username": "user2", "wallet": "wallet2", "commitment": "commit2", 
             "guess": "dogs running", "salt": "salt2", "valid": True, 
             "score": 0.4, "payout": 1/3 * 100.0}
        ]
        
        # Check that all participants have scores
        self.assertAlmostEqual(updated_participants[0]["score"], 0.8)
        self.assertAlmostEqual(updated_participants[1]["score"], 0.4)
        
        # Check that user1 has higher score than user2
        self.assertGreater(
            updated_participants[0]["score"],
            updated_participants[1]["score"]
        )
        
        # Check payout values
        self.assertAlmostEqual(
            updated_participants[0]["payout"] + updated_participants[1]["payout"],
            100.0,
            places=5
        )
        
        # Check that higher score gets higher payout
        self.assertGreater(
            updated_participants[0]["payout"],
            updated_participants[1]["payout"]
        )
    
    @patch('process_round_payouts.process_round_payouts')
    def test_process_all_rounds(self, mock_process_round):
        """Test processing all rounds that need payouts."""
        # Create test guesses with multiple rounds
        test_guesses = {
            "round1": {
                "participants": [
                    {
                        "username": "user1",
                        "guess": "cats playing",
                        "valid": True,
                        "score": None
                    }
                ],
                "target_image": "round1/image.jpg",
                "total_payout": None
            },
            "round2": {
                "participants": [
                    {
                        "username": "user2",
                        "guess": "dogs running",
                        "valid": True,
                        "score": None
                    }
                ],
                "target_image": "round2/image.jpg",
                "total_payout": None
            },
            "round3": {
                "participants": [
                    {
                        "username": "user3",
                        "guess": "birds flying",
                        "valid": True,
                        "score": 0.5,
                        "payout": 100.0
                    }
                ],
                "target_image": "round3/image.jpg",
                "total_payout": 100.0  # Already has payout
            },
            "round4": {
                "participants": [],  # Empty participants
                "target_image": "round4/image.jpg",
                "total_payout": None
            }
        }
        
        # Create JSON string for mocking
        test_guesses_json = json.dumps(test_guesses)
        
        # Create a mock for open function with our test data
        mock_file = mock_open(read_data=test_guesses_json)
        
        # Mock Path.exists to return True
        with patch('process_round_payouts.Path') as MockPath:
            mock_path_instance = MagicMock()
            MockPath.return_value = mock_path_instance
            mock_path_instance.exists.return_value = True
            
            with patch('builtins.open', mock_file):
                result = process_all_rounds(prize_pool=100.0, save_to_file=False)
        
        # Check that process_round_payouts was called for each relevant round
        # Should be called for round1 and round2, but not round3 (already has payouts) or round4 (no participants)
        mock_process_round.assert_has_calls([
            call("round1", 100.0, False),
            call("round2", 100.0, False)
        ])
        
        # Check that it was called exactly twice
        self.assertEqual(mock_process_round.call_count, 2)
    
    @patch('process_round_payouts.Path')
    @patch('builtins.open')
    def test_get_validator_for_round(self, mock_open, mock_path):
        """Test getting the appropriate validator based on round."""
        # Create mock versions data
        mock_versions_data = {
            "versions": {
                "v1.0": {
                    "applied_to_rounds": ["round0"],
                    "parameters": {
                        "use_baseline_adjustment": False
                    }
                },
                "v1.1": {
                    "applied_to_rounds": ["round1", "round2"],
                    "parameters": {
                        "use_baseline_adjustment": True
                    }
                }
            }
        }
        
        # Mock file existence and content
        mock_path.return_value.exists.return_value = True
        mock_file = mock_open.return_value.__enter__.return_value
        mock_file.read.return_value = json.dumps(mock_versions_data)
        
        # Patch json.load to return our mock data
        with patch('json.load', return_value=mock_versions_data):
            # Round0 should use legacy validator
            with patch('process_round_payouts.LegacyScoreValidator') as MockLegacyValidator:
                mock_legacy_instance = MagicMock()
                MockLegacyValidator.return_value = mock_legacy_instance
                
                validator = get_validator_for_round("round0")
                self.assertEqual(validator, mock_legacy_instance)
            
            # Round1 should use current validator
            with patch('process_round_payouts.ScoreValidator') as MockScoreValidator:
                mock_current_instance = MagicMock()
                MockScoreValidator.return_value = mock_current_instance
                
                validator = get_validator_for_round("round1")
                self.assertEqual(validator, mock_current_instance)

if __name__ == '__main__':
    unittest.main() 