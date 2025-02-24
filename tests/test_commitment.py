import unittest
import hashlib
import sys
import os

# Add parent directory to path to import generate_commitment
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from generate_commitment import generate_commitment

class TestCommitment(unittest.TestCase):
    def test_commitment_verification(self):
        """Test that a commitment hash matches its reference."""
        # Example message and salt
        message = "Sunset over city skyline with birds flying"
        salt = "test_salt"
        
        # Generate commitment
        commitment_hash = generate_commitment(message, salt)
        
        # Verify by generating again with same inputs
        verification_hash = generate_commitment(message, salt)
        self.assertEqual(
            commitment_hash,
            verification_hash,
            "Commitment hash should match when using same message and salt"
        )
        
        # Verify it differs with wrong message
        wrong_hash = generate_commitment("wrong message", salt)
        self.assertNotEqual(
            commitment_hash,
            wrong_hash,
            "Commitment hash should differ for wrong message"
        )
        
        # Verify it differs with wrong salt
        wrong_salt_hash = generate_commitment(message, "wrong_salt")
        self.assertNotEqual(
            commitment_hash,
            wrong_salt_hash,
            "Commitment hash should differ for wrong salt"
        )

    def test_reference_hash(self):
        """Test against a known reference hash."""
        message = "Sunset over city skyline with birds flying"
        salt = "test_salt"
        
        # Known reference hash for this message and salt
        reference_hash = "05ba60fa7bb9efb3e7b3bfe1946d91d6bae3d0cc88918072ece01efbd1207cad"
        
        self.assertEqual(
            generate_commitment(message, salt),
            reference_hash,
            "Hash should match reference value"
        )

    def test_commitment_format(self):
        """Test that commitment hash has expected format."""
        message = "test message"
        salt = "test_salt"
        
        commitment_hash = generate_commitment(message, salt)
        
        # Should be a 64-character hex string (SHA-256)
        self.assertEqual(
            len(commitment_hash), 
            64,
            "Commitment hash should be 64 characters long"
        )
        self.assertTrue(
            all(c in '0123456789abcdef' for c in commitment_hash),
            "Commitment hash should only contain hex characters"
        )

    def test_salt_required(self):
        """Test that empty salt raises ValueError."""
        with self.assertRaises(ValueError):
            generate_commitment("test message", "")

if __name__ == '__main__':
    unittest.main() 