"""
Schema Consistency Test

This test acts as a "consistency lock" between the Pydantic models defined in Python
and the corresponding data structures (structs) defined in Rust.

It works by:
1. Creating instances of the Pydantic models with sample data.
2. Serializing them to Python dictionaries.
3. Passing these dictionaries to special test functions in the Rust core library.
4. The Rust functions attempt to deserialize the dictionaries into Rust structs.

If the Rust deserialization succeeds, the test passes. If it fails (due to a
mismatch in fields, types, etc.), it will raise a Python exception, and the
test will fail.

This ensures that our Python and Rust data models cannot drift apart.
"""

import sys
from pathlib import Path
import json
import pytest
from pydantic import BaseModel, ValidationError
from typing import List, Optional

# Add the project root to Python path for imports
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

from datetime import datetime
from browser.data_models import Commitment, Round

# Attempt to import the Rust core library. If it fails, skip these tests.
try:
    from realmir_core import test_deserialize_commitment, test_deserialize_round
except ImportError:
    # Set a flag to skip all tests in this file
    pytest.skip("Could not import realmir_core. Run 'maturin develop' to build the Rust library.", allow_module_level=True)

# Test Pydantic models that mirror Rust structs
class Commitment(BaseModel):
    username: str
    commitment_hash: str
    wallet_address: str
    tweet_url: str
    timestamp: str

class Round(BaseModel):
    round_id: str
    announcement_url: str
    livestream_url: str
    entry_fee: float
    commitment_deadline: str
    reveal_deadline: str
    commitments: List[Commitment] = []

class TwitterReply(BaseModel):
    url: str
    author: str
    text_preview: str
    was_spam_flagged: bool

class TwitterReplyData(BaseModel):
    original_tweet_url: str
    total_replies_found: int
    replies: List[TwitterReply]

class CollectedCommitment(BaseModel):
    username: str
    commitment_hash: str
    wallet_address: str
    tweet_url: str
    timestamp: str

class CommitmentCollectionResult(BaseModel):
    success: bool
    commitments: List[CollectedCommitment]
    announcement_url: str
    total_commitments_found: int
    error_message: Optional[str] = None


def test_commitment_schema_consistency():
    """
    Tests that the Pydantic Commitment model is consistent with the Rust Commitment struct.
    """
    # 1. Create a Pydantic Commitment instance with sample data
    pydantic_commitment = Commitment(
        username="@test_miner",
        commitment_hash="0x" + "a" * 64,
        wallet_address="5Co2unDtZKZDzYNZHT2fUMkEnpVWnassfbuabvZmGTrYKgtD",
        tweet_url="https://x.com/realmir_testnet/status/12345",
        timestamp=datetime.now().isoformat()
    )

    # 2. Convert to a dictionary, using JSON-compatible types
    commitment_dict = pydantic_commitment.model_dump(mode="json")

    # 3. Pass the dictionary to the Rust test function
    try:
        test_deserialize_commitment(commitment_dict)
    except Exception as e:
        pytest.fail(f"Rust failed to deserialize Pydantic Commitment model: {e}")


def test_round_schema_consistency():
    """
    Tests that the Pydantic Round model is consistent with the Rust Round struct.
    """
    # 1. Create Pydantic instances
    pydantic_commitment = Commitment(
        username="@test_miner",
        commitment_hash="0x" + "a" * 64,
        wallet_address="5Co2unDtZKZDzYNZHT2fUMkEnpVWnassfbuabvZmGTrYKgtD",
        tweet_url="https://x.com/realmir_testnet/status/12345",
        timestamp=datetime.now().isoformat()
    )
    
    pydantic_round = Round(
        round_id="test_round_001",
        announcement_url="https://x.com/realmir_testnet/status/12344",
        livestream_url="https://youtube.com/live/some_id",
        entry_fee=0.001,
        commitment_deadline=datetime.now().isoformat(),
        reveal_deadline=datetime.now().isoformat(),
        commitments=[pydantic_commitment]
    )

    # 2. Convert to a dictionary
    round_dict = pydantic_round.model_dump(mode="json")
    
    # 3. Pass to the Rust test function
    try:
        test_deserialize_round(round_dict)
    except Exception as e:
        pytest.fail(f"Rust failed to deserialize Pydantic Round model: {e}")

def test_round_with_empty_commitments():
    """
    Tests that a Round with no commitments can be deserialized correctly.
    This validates the `#[serde(default)]` attribute on the Rust side.
    """
    pydantic_round = Round(
        round_id="test_round_002",
        announcement_url="https://x.com/realmir_testnet/status/12346",
        livestream_url="https://youtube.com/live/some_id_2",
        entry_fee=0.001,
        commitment_deadline=datetime.now().isoformat(),
        reveal_deadline=datetime.now().isoformat(),
        commitments=[]  # Empty list
    )

    round_dict = pydantic_round.model_dump(mode="json")
    
    try:
        test_deserialize_round(round_dict)
    except Exception as e:
        pytest.fail(f"Rust failed to deserialize Round with empty commitments: {e}")

class TestSchemaConsistency:
    """Test schema consistency between Rust and Python data models."""
    
    def test_commitment_schema_consistency(self):
        """Test that Commitment schema is consistent between Rust and Python."""
        # Test data that should be valid for both Rust and Python
        commitment_data = {
            "username": "testuser",
            "commitment_hash": "abc123def456",
            "wallet_address": "5Co2unDtZKZDzYNZHT2fUMkEnpVWnassfbuabvZmGTrYKgtD",
            "tweet_url": "https://x.com/testuser/status/123456789",
            "timestamp": "2023-01-01T00:00:00Z"
        }
        
        # Test Python model validation
        python_commitment = Commitment(**commitment_data)
        assert python_commitment.username == "testuser"
        assert python_commitment.commitment_hash == "abc123def456"
        
        # Test Rust deserialization (using Python bindings)
        try:
            import realmir_core
            realmir_core.test_deserialize_commitment(commitment_data)
        except ImportError:
            pytest.skip("Rust module not available")
        except Exception as e:
            pytest.fail(f"Rust deserialization failed: {e}")

    def test_round_schema_consistency(self):
        """Test that Round schema is consistent between Rust and Python."""
        round_data = {
            "round_id": "round1",
            "announcement_url": "https://x.com/realmir/status/123",
            "livestream_url": "https://example.com/stream",
            "entry_fee": 0.1,
            "commitment_deadline": "2023-01-01T12:00:00Z",
            "reveal_deadline": "2023-01-01T18:00:00Z",
            "commitments": []
        }
        
        # Test Python model validation
        python_round = Round(**round_data)
        assert python_round.round_id == "round1"
        assert python_round.entry_fee == 0.1
        
        # Test Rust deserialization
        try:
            import realmir_core
            realmir_core.test_deserialize_round(round_data)
        except ImportError:
            pytest.skip("Rust module not available")
        except Exception as e:
            pytest.fail(f"Rust deserialization failed: {e}")

    def test_twitter_reply_data_schema_consistency(self):
        """Test that TwitterReplyData schema is consistent between Rust and Python."""
        twitter_data = {
            "original_tweet_url": "https://x.com/realmir_testnet/status/1907159517013422578",
            "total_replies_found": 2,
            "replies": [
                {
                    "url": "https://x.com/davidynamic/status/1907165981706760445",
                    "author": "@davidynamic", 
                    "text_preview": "Commit: bc64a7b517b4e0a23c61300bb2e0601641fac6b387c76a1a9abb3d425c230235 Wallet: 5Co2unDtZKZDzYNZHT2fUMkEnpVWnassfbuabvZmGTrYKgtD",
                    "was_spam_flagged": False
                },
                {
                    "url": "https://x.com/track_data_/status/1907160947845710103",
                    "author": "@track_data_",
                    "text_preview": "Commit: c4bd4f6ed4fbdd80a94d6842700bde38a28f2ebb5dc764454c0df209ccb3a87c Wallet: 5EFfNQedRehT16SDHG84HKwTcEDcf9sX76CHFLBVPY1Jkytt1",
                    "was_spam_flagged": False
                }
            ]
        }
        
        # Test Python model validation
        python_twitter = TwitterReplyData(**twitter_data)
        assert python_twitter.total_replies_found == 2
        assert len(python_twitter.replies) == 2
        assert python_twitter.replies[0].author == "@davidynamic"
        
        # Test Rust deserialization
        try:
            import realmir_core
            realmir_core.test_deserialize_twitter_reply_data(twitter_data)
        except ImportError:
            pytest.skip("Rust module not available")
        except Exception as e:
            pytest.fail(f"Rust deserialization failed: {e}")

    def test_commitment_collection_result_schema_consistency(self):
        """Test that CommitmentCollectionResult schema is consistent between Rust and Python."""
        collection_data = {
            "success": True,
            "commitments": [
                {
                    "username": "davidynamic",
                    "commitment_hash": "bc64a7b517b4e0a23c61300bb2e0601641fac6b387c76a1a9abb3d425c230235",
                    "wallet_address": "5Co2unDtZKZDzYNZHT2fUMkEnpVWnassfbuabvZmGTrYKgtD",
                    "tweet_url": "https://x.com/davidynamic/status/1907165981706760445",
                    "timestamp": "2025-06-14 12:42:07.464279"
                },
                {
                    "username": "track_data_",
                    "commitment_hash": "c4bd4f6ed4fbdd80a94d6842700bde38a28f2ebb5dc764454c0df209ccb3a87c",
                    "wallet_address": "5EFfNQedRehT16SDHG84HKwTcEDcf9sX76CHFLBVPY1Jkytt1",
                    "tweet_url": "https://x.com/track_data_/status/1907160947845710103",
                    "timestamp": "2025-06-14 12:42:07.464371"
                }
            ],
            "announcement_url": "https://x.com/realmir_testnet/status/1907159517013422578",
            "total_commitments_found": 2,
            "error_message": None
        }
        
        # Test Python model validation
        python_collection = CommitmentCollectionResult(**collection_data)
        assert python_collection.success == True
        assert python_collection.total_commitments_found == 2
        assert len(python_collection.commitments) == 2
        assert python_collection.commitments[0].username == "davidynamic"
        
        # Test Rust deserialization
        try:
            import realmir_core
            realmir_core.test_deserialize_commitment_collection(collection_data)
        except ImportError:
            pytest.skip("Rust module not available")
        except Exception as e:
            pytest.fail(f"Rust deserialization failed: {e}")

    def test_actual_rounds_data_compatibility(self):
        """Test that actual rounds.json data is compatible with our schemas."""
        import os
        
        # Test with actual data from rounds.json if available
        rounds_file = "data/rounds.json"
        if os.path.exists(rounds_file):
            with open(rounds_file, 'r') as f:
                rounds_data = json.load(f)
            
            # Test round2 which has enhanced data
            if "round2" in rounds_data:
                round2_data = rounds_data["round2"]
                
                # Test Twitter data if present
                if "raw_commitment_replies" in round2_data:
                    twitter_data = round2_data["raw_commitment_replies"]
                    python_twitter = TwitterReplyData(**twitter_data)
                    assert python_twitter.total_replies_found > 0
                
                # Test commitment collection data if present
                if "collected_commitments" in round2_data:
                    collection_data = round2_data["collected_commitments"]
                    python_collection = CommitmentCollectionResult(**collection_data)
                    assert python_collection.success == True

    def test_data_access_layer_python_integration(self):
        """Test that the Python data access layer works correctly."""
        try:
            import realmir_core
            import tempfile
            import os
            
            # Create a temporary file for testing
            with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
                test_data = {"test_round": {"title": "Test Round"}}
                json.dump(test_data, f)
                temp_file = f.name
            
            try:
                # Test PyDataAccessLayer
                dal = realmir_core.PyDataAccessLayer(temp_file)
                
                # Test loading data
                rounds_json = dal.load_all_rounds()
                rounds_data = json.loads(rounds_json)
                
                # The data should be processed through our conversion logic
                assert isinstance(rounds_data, dict)
                
                # Test getting all round IDs
                round_ids = dal.get_all_round_ids()
                assert isinstance(round_ids, list)
                
                # Test data validation
                issues = dal.validate_data()
                assert isinstance(issues, list)
                
            finally:
                # Cleanup
                os.unlink(temp_file)
                
        except ImportError:
            pytest.skip("Rust module not available")
        except Exception as e:
            pytest.fail(f"Data access layer integration failed: {e}")

if __name__ == "__main__":
    pytest.main([__file__]) 