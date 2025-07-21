import unittest
import json
import os
import sys
import tempfile
from pathlib import Path
from unittest.mock import patch, MagicMock, mock_open

# Add parent directory to path to import verify_commitments
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from core.verify_commitments import verify_round_commitments
from core.generate_commitment import generate_commitment

class TestVerifyCommitments(unittest.TestCase):
    def setUp(self):
        """Set up temporary files and test data."""
        # Create a temp directory for test files
        self.temp_dir = tempfile.TemporaryDirectory()
        self.rounds_dir = Path(self.temp_dir.name) / "rounds"
        self.rounds_dir.mkdir(exist_ok=True)
        
        # Prepare test data
        valid_guess = "Test guess for valid commitment"
        valid_salt = "valid_salt"
        # This creates a valid commitment by hashing the guess and salt
        valid_commitment = generate_commitment(valid_guess, valid_salt)
        
        invalid_guess = "Test guess for invalid commitment"
        invalid_salt = "invalid_salt"
        # This is an intentionally invalid commitment that doesn't match the guess and salt
        invalid_commitment = "invalid_commitment_hash_not_matching_guess_and_salt"
        
        # Create test data
        self.test_data = {
            # VALID ROUND: All commitments in this round should validate successfully
            "valid_round": {
                "participants": [
                    {
                        "username": "valid_user",
                        "guess": valid_guess,
                        "salt": valid_salt,
                        "commitment": valid_commitment,  # Valid: generated from valid_guess + valid_salt
                        "valid": False  # Initially false, should become true after verification
                    }
                ],
                "target_image": "valid_round/target.jpg",
                "target_time": "20250401_163057EST",
                "total_payout": 0
            },
            # INVALID ROUND: All commitments in this round should fail validation
            "invalid_round": {
                "participants": [
                    {
                        "username": "invalid_user",
                        "guess": invalid_guess,
                        "salt": invalid_salt,
                        "commitment": invalid_commitment,  # Invalid: hardcoded string, not a proper hash
                        "valid": True  # Initially true, should become false after verification
                    }
                ],
                "target_image": "invalid_round/target.jpg",
                "target_time": "20250401_163057EST",
                "total_payout": 0
            },
            # MIXED ROUND: Some valid, some invalid commitments
            "mixed_round": {
                "participants": [
                    {
                        "username": "valid_user",
                        "guess": valid_guess,
                        "salt": valid_salt,
                        "commitment": valid_commitment,  # Valid commitment
                        "valid": False  # Should become true after verification
                    },
                    {
                        "username": "invalid_user",
                        "guess": invalid_guess,
                        "salt": invalid_salt,
                        "commitment": invalid_commitment,  # Invalid commitment
                        "valid": True  # Should become false after verification
                    }
                ],
                "target_image": "mixed_round/target.jpg",
                "target_time": "20250401_163057EST",
                "total_payout": 0
            },
            # MISSING DATA ROUND: Tests handling of incomplete data
            "missing_data_round": {
                "participants": [
                    {
                        "username": "missing_guess_user",
                        "guess": "",  # Missing guess - should be skipped and remain invalid
                        "salt": valid_salt,
                        "commitment": valid_commitment,
                        "valid": False  # Should remain false (can't verify without guess)
                    },
                    {
                        "username": "missing_salt_user",
                        "guess": valid_guess,
                        "salt": "",  # Missing salt - should be skipped and remain invalid
                        "commitment": valid_commitment,
                        "valid": False  # Should remain false (can't verify without salt)
                    }
                ],
                "target_image": "missing_data_round/target.jpg",
                "target_time": "20250401_163057EST",
                "total_payout": 0
            },
            # EMPTY ROUND: Tests handling of rounds with no participants
            "empty_round": {
                "participants": [],  # No participants to verify
                "target_image": "empty_round/target.jpg",
                "target_time": "20250401_163057EST",
                "total_payout": 0
            }
        }
        
        # Write test data to file
        self.guesses_file = self.rounds_dir / "guesses.json"
        with open(self.guesses_file, 'w') as f:
            json.dump(self.test_data, f, indent=4)
    
    def tearDown(self):
        """Clean up temporary files."""
        self.temp_dir.cleanup()
    
    @patch('core.verify_commitments.Path')
    def test_file_not_found(self, mock_path):
        """Test handling of missing guesses.json file."""
        # Mock Path to return a non-existent file
        mock_path_instance = MagicMock()
        mock_path_instance.exists.return_value = False
        mock_path.return_value = mock_path_instance
        
        # Verify the function returns False when file doesn't exist
        result = verify_round_commitments("any_round")
        self.assertFalse(result)
    
    def test_round_not_found(self):
        """Test handling of non-existent round."""
        with patch('core.verify_commitments.Path', return_value=self.guesses_file):
            result = verify_round_commitments("non_existent_round")
            self.assertFalse(result)
    
    def test_empty_round(self):
        """Test handling of a round with no participants."""
        with patch('core.verify_commitments.Path', return_value=self.guesses_file):
            result = verify_round_commitments("empty_round")
            self.assertFalse(result)
    
    def test_valid_commitments(self):
        """Test verifying a round with valid commitments."""
        with patch('core.verify_commitments.Path', return_value=self.guesses_file):
            with patch('builtins.open', new_callable=mock_open, read_data=json.dumps(self.test_data)):
                # Mock json.load to return our test data directly
                with patch('json.load', return_value=self.test_data):
                    # Mock json.dump to capture what would be written
                    with patch('json.dump') as mock_json_dump:
                        result = verify_round_commitments("valid_round")
                        
                        # Verify function returns True for valid round
                        self.assertTrue(result)
                        
                        # Check that json.dump was called to update the valid field
                        mock_json_dump.assert_called_once()
                        # Get the data that would be written
                        written_data = mock_json_dump.call_args[0][0]
                        # Verify the valid field was updated to True
                        self.assertTrue(written_data["valid_round"]["participants"][0]["valid"])
    
    def test_invalid_commitments(self):
        """Test verifying a round with invalid commitments."""
        with patch('core.verify_commitments.Path', return_value=self.guesses_file):
            with patch('builtins.open', new_callable=mock_open, read_data=json.dumps(self.test_data)):
                # Mock json.load to return our test data directly
                with patch('json.load', return_value=self.test_data):
                    # Mock json.dump to capture what would be written
                    with patch('json.dump') as mock_json_dump:
                        result = verify_round_commitments("invalid_round")
                        
                        # Verify function returns False for invalid round
                        self.assertFalse(result)
                        
                        # Check that json.dump was called to update the valid field
                        mock_json_dump.assert_called_once()
                        # Get the data that would be written
                        written_data = mock_json_dump.call_args[0][0]
                        # Verify the valid field was updated to False
                        self.assertFalse(written_data["invalid_round"]["participants"][0]["valid"])
    
    def test_mixed_commitments(self):
        """Test verifying a round with both valid and invalid commitments."""
        with patch('core.verify_commitments.Path', return_value=self.guesses_file):
            with patch('builtins.open', new_callable=mock_open, read_data=json.dumps(self.test_data)):
                # Mock json.load to return our test data directly
                with patch('json.load', return_value=self.test_data):
                    # Mock json.dump to capture what would be written
                    with patch('json.dump') as mock_json_dump:
                        result = verify_round_commitments("mixed_round")
                        
                        # Verify function returns False when any commitment is invalid
                        self.assertFalse(result)
                        
                        # Check that json.dump was called to update the valid fields
                        mock_json_dump.assert_called_once()
                        # Get the data that would be written
                        written_data = mock_json_dump.call_args[0][0]
                        # Verify the valid field was updated correctly for both participants
                        self.assertTrue(written_data["mixed_round"]["participants"][0]["valid"])
                        self.assertFalse(written_data["mixed_round"]["participants"][1]["valid"])
    
    def test_missing_data(self):
        """Test verifying a round with missing guess or salt."""
        with patch('core.verify_commitments.Path', return_value=self.guesses_file):
            with patch('builtins.open', new_callable=mock_open, read_data=json.dumps(self.test_data)):
                # Mock json.load to return our test data directly
                with patch('json.load', return_value=self.test_data):
                    # Mock json.dump to capture what would be written
                    with patch('json.dump') as mock_json_dump:
                        result = verify_round_commitments("missing_data_round")
                        
                        # Verify function returns False when data is missing
                        self.assertFalse(result)
                        
                        # Check that json.dump was called to update the valid fields
                        mock_json_dump.assert_called_once()
                        # Get the data that would be written
                        written_data = mock_json_dump.call_args[0][0]
                        # Verify the valid fields remain False
                        self.assertFalse(written_data["missing_data_round"]["participants"][0]["valid"])
                        self.assertFalse(written_data["missing_data_round"]["participants"][1]["valid"])

if __name__ == '__main__':
    unittest.main() 