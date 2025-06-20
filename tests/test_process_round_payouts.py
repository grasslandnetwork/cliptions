import unittest
import json
import os
import sys
from unittest.mock import patch, MagicMock, mock_open, call
import numpy as np

# Add parent directory to path to import the module
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from core.process_round_payouts import process_round_payouts, process_all_rounds, get_validator_for_round, LegacyScoreValidator

class TestProcessRoundPayouts(unittest.TestCase):
    @patch('core.process_round_payouts.get_validator_for_round')
    @patch('core.process_round_payouts.verify_round_commitments')
    @patch('builtins.input')  # Add patch for input function
    def test_process_round_payouts_valid_commitments(self, mock_input, mock_verify_commitments, mock_get_validator):
        """Test processing payouts for a round with valid commitments."""
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
                "total_payout": None,
                "prize_pool": 1.0  # Add prize_pool to test data
            }
        }
        
        # Create JSON string for mocking
        test_guesses_json = json.dumps(test_guesses)
        
        # Mock verification to return True (all commitments valid)
        mock_verify_commitments.return_value = True
        
        # Set up input mock (should not be called when all commitments are valid)
        mock_input.return_value = "y"
        
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
        with patch('core.process_round_payouts.Path') as MockPath:
            mock_path_instance = MagicMock()
            MockPath.return_value = mock_path_instance
            mock_path_instance.exists.return_value = True
            
            with patch('builtins.open', mock_file):
                # Use the prize_pool from the guesses.json data
                result = process_round_payouts("test_round", save_to_file=False, verify_commitments=True)
        
        # Verify input was not called since all commitments are valid
        mock_input.assert_not_called()
        
        # Check that the validator was used correctly
        mock_get_validator.assert_called_once_with("test_round")
        mock_validator.embedder.get_image_embedding.assert_called_once()
        self.assertEqual(mock_validator.calculate_adjusted_score.call_count, 2)
        
        # Check that participants were updated with scores and payouts
        self.assertEqual(len(result["test_round"]["participants"]), 2)
        
        # Check scores were assigned
        self.assertEqual(result["test_round"]["participants"][0]["score"], 0.8)
        self.assertEqual(result["test_round"]["participants"][1]["score"], 0.4)
        
        # Check payouts were calculated (2/3 for first place, 1/3 for second place with prize pool of 1.0)
        self.assertAlmostEqual(result["test_round"]["participants"][0]["payout"], 2/3, places=3)
        self.assertAlmostEqual(result["test_round"]["participants"][1]["payout"], 1/3, places=3)
        
        # Check total payout was updated
        self.assertEqual(result["test_round"]["total_payout"], 1.0)

    @patch('core.process_round_payouts.get_validator_for_round')
    @patch('core.process_round_payouts.verify_round_commitments')
    @patch('builtins.input')  # Add patch for input function
    def test_process_round_payouts_invalid_commitments_continue(self, mock_input, mock_verify_commitments, mock_get_validator):
        """Test processing payouts for a round with invalid commitments, user chooses to continue."""
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
                "total_payout": None,
                "prize_pool": 1.0
            }
        }
        
        # Create JSON string for mocking
        test_guesses_json = json.dumps(test_guesses)
        
        # Mock verification to return False (some commitments invalid)
        mock_verify_commitments.return_value = False
        
        # Mock input to return 'y' (continue despite invalid commitments)
        mock_input.return_value = "y"
        
        # Mock the validator
        mock_validator = MagicMock()
        mock_get_validator.return_value = mock_validator
        
        # Mock image features
        mock_image_features = np.ones((512))
        mock_validator.embedder.get_image_embedding.return_value = mock_image_features
        
        # Mock scores
        mock_validator.calculate_adjusted_score.side_effect = [0.8, 0.4]
        
        # Create a mock for open function with our test data
        mock_file = mock_open(read_data=test_guesses_json)
        
        # Mock Path.exists to return True
        with patch('core.process_round_payouts.Path') as MockPath:
            mock_path_instance = MagicMock()
            MockPath.return_value = mock_path_instance
            mock_path_instance.exists.return_value = True
            
            with patch('builtins.open', mock_file):
                # Process with invalid commitments but continue
                result = process_round_payouts("test_round", save_to_file=False, verify_commitments=True)
        
        # Verify input was called since commitments are invalid
        mock_input.assert_called_once()
        
        # Verify the processing was completed anyway
        self.assertEqual(result["test_round"]["total_payout"], 1.0)

    @patch('core.process_round_payouts.get_validator_for_round')
    @patch('core.process_round_payouts.verify_round_commitments')
    @patch('builtins.input')  # Add patch for input function
    def test_process_round_payouts_invalid_commitments_abort(self, mock_input, mock_verify_commitments, mock_get_validator):
        """Test processing payouts for a round with invalid commitments, user chooses to abort."""
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
                    }
                ],
                "target_image": "test_round/dummy_image.jpg",
                "target_time": "test_time",
                "total_payout": None,
                "prize_pool": 1.0
            }
        }
        
        # Create JSON string for mocking
        test_guesses_json = json.dumps(test_guesses)
        
        # Mock verification to return False (some commitments invalid)
        mock_verify_commitments.return_value = False
        
        # Mock input to return 'n' (abort due to invalid commitments)
        mock_input.return_value = "n"
        
        # Create a mock for open function with our test data
        mock_file = mock_open(read_data=test_guesses_json)
        
        # Mock Path.exists to return True
        with patch('core.process_round_payouts.Path') as MockPath:
            mock_path_instance = MagicMock()
            MockPath.return_value = mock_path_instance
            mock_path_instance.exists.return_value = True
            
            with patch('builtins.open', mock_file):
                # Process with invalid commitments but abort
                with self.assertRaises(SystemExit):
                    process_round_payouts("test_round", save_to_file=False, verify_commitments=True)
        
        # Verify input was called since commitments are invalid
        mock_input.assert_called_once()
        
        # Validator should not be used since we aborted
        mock_get_validator.assert_not_called()
    
    @patch('core.process_round_payouts.process_round_payouts')
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
                "total_payout": None,
                "prize_pool": 1.0  # Add prize pool to each round
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
                "total_payout": None,
                "prize_pool": 0.5  # Different prize pool for this round
            },
            "round3": {
                "participants": [
                    {
                        "username": "user3",
                        "guess": "birds flying",
                        "valid": True,
                        "score": 0.5,
                        "payout": 1.0
                    }
                ],
                "target_image": "round3/image.jpg",
                "total_payout": 1.0,  # Already has payout
                "prize_pool": 1.0
            },
            "round4": {
                "participants": [],  # Empty participants
                "target_image": "round4/image.jpg",
                "total_payout": None
            },
            "round5": {
                "participants": [
                    {
                        "username": "user5",
                        "guess": "fish swimming",
                        "valid": True,
                        "score": None
                    }
                ],
                "target_image": "round5/image.jpg",
                "total_payout": None
                # No prize pool defined, should be skipped
            }
        }
        
        # Create JSON string for mocking
        test_guesses_json = json.dumps(test_guesses)
        
        # Create a mock for open function with our test data
        mock_file = mock_open(read_data=test_guesses_json)
        
        # Mock Path.exists to return True
        with patch('core.process_round_payouts.Path') as MockPath:
            mock_path_instance = MagicMock()
            MockPath.return_value = mock_path_instance
            mock_path_instance.exists.return_value = True
            
            with patch('builtins.open', mock_file):
                # No need to pass prize_pool parameter anymore
                result = process_all_rounds(save_to_file=False)
        
        # Should process round1 and round2, but not round3 (already has payout), 
        # not round4 (no participants), and not round5 (no prize pool)
        self.assertEqual(mock_process_round.call_count, 2)
        
        # Check that it called process_round_payouts with the correct prize pools for each round
        mock_process_round.assert_any_call("round1", 1.0, False, True)
        mock_process_round.assert_any_call("round2", 0.5, False, True)
    
    @patch('core.process_round_payouts.ScoreValidator')
    @patch('core.process_round_payouts.LegacyScoreValidator')
    def test_get_validator_for_round(self, MockLegacyValidator, MockScoreValidator):
        """Test getting the appropriate validator for a round."""
        # Create mock validator instances
        mock_legacy_instance = MagicMock()
        mock_current_instance = MagicMock()
        MockLegacyValidator.return_value = mock_legacy_instance
        MockScoreValidator.return_value = mock_current_instance
        
        # Test with legacy version
        with patch('builtins.open', mock_open(read_data='''
        {
            "versions": {
                "v0.1": {
                    "applied_to_rounds": ["round0"],
                    "parameters": {
                        "use_baseline_adjustment": false
                    }
                },
                "v0.2": {
                    "applied_to_rounds": ["round1", "round2"],
                    "parameters": {
                        "use_baseline_adjustment": true
                    }
                }
            }
        }
        ''')):
            with patch('core.process_round_payouts.Path.exists', return_value=True):
                # Should return LegacyScoreValidator for round0
                validator = get_validator_for_round("round0")
                self.assertEqual(validator, mock_legacy_instance)
                MockLegacyValidator.assert_called_once()
                
                # Should return ScoreValidator for round1
                validator = get_validator_for_round("round1")
                self.assertEqual(validator, mock_current_instance)
                
                # Should return ScoreValidator for unknown round
                validator = get_validator_for_round("unknown_round")
                self.assertEqual(validator, mock_current_instance)

if __name__ == '__main__':
    unittest.main() 