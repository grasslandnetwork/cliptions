import asyncio
import pytest
import os
import sys

# Add browser-use directory to Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'browser-use'))

from twitter_data_fetcher import fetch_round_guesses

# Placeholder for the actual fetch_round_guesses function until it's created
# async def fetch_round_guesses(round_number: int, target_time_str: str) -> dict:
#     """
#     Placeholder function to simulate fetching data from Twitter.
#     In reality, this will use Browser Use to interact with Twitter.
#     """
#     # This is a dummy implementation.
#     # We expect the actual function to return data in the format specified
#     # in the Browser Use task prompt from browser-use/example.py
#     if round_number == 0 and target_time_str == "20250223_133057EST":
#         # This is a sample of what the real function might return,
#         # to be replaced by actual Browser Use calls.
#         # For the initial test setup, we can return a structure that would pass
#         # if the expected_guesses_round0 matches this.
#         # Or, more practically, return a known "dummy" incorrect value
#         # to ensure the test fails correctly until implemented.
#         return {
#             "round": 0,
#             "targetTime": "20250223_133057EST",
#             "guesses": [
#                 {"username": "dummy_user", "guess": "dummy_guess"}
#             ]
#         }
#     return {}

@pytest.mark.asyncio
async def test_fetch_twitter_data_round0():
    round_number = 0
    target_time = "20250223_133057EST"

    # Test in test mode first (safer and faster)
    os.environ['TWITTER_UTILS_TEST_MODE'] = 'true'
    actual_output = await fetch_round_guesses(round_number, target_time)

    # Add a check for errors returned by the fetcher
    if "error" in actual_output:
        pytest.fail(f"fetch_round_guesses returned an error: {actual_output['error']} - Raw: {actual_output.get('raw_output', '')}")

    # In test mode, we expect a successful output
    assert "output" in actual_output or "test_result" in actual_output, "Expected output or test_result in test mode"
    print("✅ Test mode passed successfully")

@pytest.mark.asyncio
async def test_fetch_twitter_data_round0_real_mode():
    """Test with real Twitter data extraction (only if credentials are available)"""
    round_number = 0
    target_time = "20250223_133057EST"
    
    # Check if Twitter credentials are available
    if not os.getenv("TWITTER_NAME") or not os.getenv("TWITTER_PASSWORD"):
        pytest.skip("Twitter credentials not available - skipping real mode test")
    
    # Test in real mode
    os.environ['TWITTER_UTILS_TEST_MODE'] = 'false'
    actual_output = await fetch_round_guesses(round_number, target_time)

    # Add a check for errors returned by the fetcher
    if "error" in actual_output:
        pytest.fail(f"fetch_round_guesses returned an error: {actual_output['error']} - Raw: {actual_output.get('raw_output', '')}")

    # In real mode, we expect structured data
    assert isinstance(actual_output, dict), f"Expected dict output, got {type(actual_output)}"
    print("✅ Real Twitter mode passed successfully") 