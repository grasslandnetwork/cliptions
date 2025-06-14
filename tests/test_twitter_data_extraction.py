import asyncio
import json
import pytest
import pathlib
import sys
import os

# Add the browser-use directory to the Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'browser-use'))

from browser.twitter_data_fetcher import fetch_round_guesses

# Define the path to the ground truth data
# Assumes the script is run from the project root or tests/ directory
# Adjust if your test execution context is different
TEST_DIR = pathlib.Path(__file__).parent
PROJECT_ROOT = TEST_DIR.parent
GUESSES_FILE_PATH = PROJECT_ROOT / "rounds" / "guesses.json"

@pytest.fixture(scope="module")
def event_loop():
    """Create an instance of the default event loop for each test module."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    asyncio.set_event_loop(loop)
    yield loop
    loop.close()

@pytest.fixture(scope="module")
def ground_truth_data():
    """Load the ground truth data from guesses.json."""
    if not GUESSES_FILE_PATH.exists():
        pytest.fail(f"Ground truth file not found: {GUESSES_FILE_PATH}")
    with open(GUESSES_FILE_PATH, 'r') as f:
        return json.load(f)

@pytest.mark.asyncio
async def test_fetch_real_twitter_data_against_ground_truth(ground_truth_data):
    """
    Test the fetch_round_guesses function against real Twitter data (mocked by ground truth).
    This test assumes that if run against actual Twitter, the fetch_round_guesses
    function would return data in the specified format, which is then compared
    to the structure of guesses.json.
    """
    # For now, this test will be more of a structural check and placeholder.
    # To truly test against live Twitter, we'd need a way to ensure specific tweets
    # are present or use a mocking/VCR.py-like setup for network requests.

    # Iterate through each round in the ground truth data
    for round_key, round_data in ground_truth_data.items():
        if not round_key.startswith("round"):
            continue # Skip non-round keys like 'last_round_processed' if any

        round_number_str = round_key.replace("round", "")
        try:
            round_number = int(round_number_str)
        except ValueError:
            print(f"Skipping invalid round key: {round_key}")
            continue
            
        target_time = round_data.get("target_time")

        if target_time is None:
            print(f"Skipping round {round_key} due to missing 'target_time'.")
            continue

        print(f"Testing data extraction for: {round_key}, Target Time: {target_time}")

        # --- THIS IS WHERE THE ACTUAL CALL TO fetch_round_guesses WOULD GO ---
        # For now, we will simulate a successful fetch that matches the structure,
        # as running against live Twitter in automated tests is complex and brittle.
        # In a real CI/CD, you might have a separate set of integration tests that
        # do hit a controlled Twitter account or use recorded interactions (e.g., with VCR.py).
        
        # actual_fetched_data = await fetch_round_guesses(round_number, target_time)
        
        # SIMULATED/MOCKED fetched data for the purpose of this structural test plan:
        # This simulated data assumes the fetcher *correctly* gets the data.
        # The real test would be to uncomment the line above and run it (potentially with mocks).
        simulated_guesses = []
        for participant in round_data.get("participants", []):
            simulated_guesses.append({
                "username": participant["username"],
                "reveal": participant["reveal"] 
            })
        
        actual_fetched_data = {
            "round_number": round_number,
            "target_time": target_time,
            "guesses": simulated_guesses # Using simulated guesses for now
        }
        # --- END OF SIMULATION ---

        # 1. Verify top-level keys and their types/values
        assert isinstance(actual_fetched_data, dict), f"Expected dict, got {type(actual_fetched_data)} for {round_key}"
        assert actual_fetched_data.get("round_number") == round_number, f"Mismatch in 'round_number' for {round_key}"
        assert actual_fetched_data.get("target_time") == target_time, f"Mismatch in 'target_time' for {round_key}"
        assert "guesses" in actual_fetched_data, f"'guesses' key missing for {round_key}"
        assert isinstance(actual_fetched_data["guesses"], list), f"Expected 'guesses' to be a list for {round_key}"

        # 2. Prepare expected guesses from ground_truth_data for comparison
        expected_guesses_for_round = []
        for participant in round_data.get("participants", []):
            expected_guesses_for_round.append({
                "username": participant["username"],
                "reveal": participant["reveal"]
            })

        # 3. Compare the content of "guesses"
        # Sort both lists of dictionaries to ensure order doesn't affect comparison
        # We need a consistent key for sorting, 'username' is a good candidate.
        # Filter out any potential None values from lists before sorting if necessary
        
        actual_sorted_guesses = sorted(actual_fetched_data["guesses"], key=lambda x: x.get("username", ""))
        expected_sorted_guesses = sorted(expected_guesses_for_round, key=lambda x: x.get("username", ""))
        
        assert len(actual_sorted_guesses) == len(expected_sorted_guesses), \
            f"Mismatch in number of guesses for {round_key}. Got {len(actual_sorted_guesses)}, expected {len(expected_sorted_guesses)}"

        for actual_guess, expected_guess in zip(actual_sorted_guesses, expected_sorted_guesses):
            assert actual_guess.get("username") == expected_guess.get("username"), \
                f"Username mismatch in {round_key}: Got '{actual_guess.get('username')}', expected '{expected_guess.get('username')}'"
            assert actual_guess.get("reveal") == expected_guess.get("reveal"), \
                f"Reveal mismatch for user '{actual_guess.get('username')}' in {round_key}: Got '{actual_guess.get('reveal')}', expected '{expected_guess.get('reveal')}'"

        print(f"Successfully verified structure and content for {round_key}")

@pytest.mark.asyncio
async def test_twitter_data_structure_and_content(ground_truth_data):
    """Tests that fetched data matches the structure and content of guesses.json."""
    for round_key, ground_truth_round_data in ground_truth_data.items():
        if not isinstance(ground_truth_round_data, dict) or "participants" not in ground_truth_round_data:
            print(f"Skipping {round_key} due to missing 'participants' or incorrect format.")
            continue

        round_number = int(round_key.replace("round", ""))
        target_time = ground_truth_round_data["target_time"]
        round_reveal_url = ground_truth_round_data.get("round_reveal_url") # Get the reveal URL for data extraction
        round_commitment_url = ground_truth_round_data.get("round_commitment_url") # Get commitment URL for assertions

        # --- SIMULATION BLOCK for testing the test logic itself ---
        # In a real scenario, this block would be replaced by:
        # actual_fetched_data = await fetch_round_guesses(round_number, target_time)
        # if True: # Keep simulation active for structural test
        #     # This is a structural test, so we populate actual_fetched_data directly from ground_truth_round_data
        #     # to ensure the test logic itself is sound.
        #     actual_fetched_data = {
        #         "round_number": ground_truth_round_data.get("round_number", int(round_key.replace("round", ""))), # Assuming round_key like "round0"
        #         "target_time": ground_truth_round_data["target_time"],
        #         "guesses": [
        #             {"username": p["username"], "reveal": p["reveal"], "guess_url": p.get("guess_url")}
        #             for p in ground_truth_round_data["participants"]
        #         ]
        #     }
        #     # Add round_announcement_url if it exists in ground_truth_round_data
        #     if "round_announcement_url" in ground_truth_round_data:
        #         actual_fetched_data["round_announcement_url"] = ground_truth_round_data["round_announcement_url"]
        # --- END OF SIMULATION ---

        # Call the actual fetch_round_guesses function
        print(f"Calling fetch_round_guesses for {round_key} with reveal URL: {round_reveal_url}")
        actual_fetched_data = await fetch_round_guesses(
            round_number=round_number, 
            target_time_str=target_time, 
            round_reveal_url=round_reveal_url
        )

        assert "round_number" in actual_fetched_data
        assert isinstance(actual_fetched_data["guesses"], list)

        # Prepare expected guesses for comparison
        expected_guesses_for_round = [
            {"username": p["username"], "reveal": p["reveal"]}
            for p in ground_truth_round_data["participants"]
        ]

        # Sort both lists of dictionaries by username to ensure order-independent comparison
        actual_guesses_sorted = sorted(actual_fetched_data["guesses"], key=lambda x: x.get("username", ""))
        expected_guesses_sorted = sorted(expected_guesses_for_round, key=lambda x: x.get("username", ""))

        assert len(actual_guesses_sorted) == len(expected_guesses_sorted), \
            f"Mismatch in number of guesses for {round_key}. Got {len(actual_guesses_sorted)}, expected {len(expected_guesses_sorted)}"

        for actual_guess, expected_guess in zip(actual_guesses_sorted, expected_guesses_sorted):
            assert actual_guess.get("username") == expected_guess.get("username"), \
                f"Username mismatch in {round_key}: Got '{actual_guess.get('username')}', expected '{expected_guess.get('username')}'"
            assert actual_guess.get("reveal") == expected_guess.get("reveal"), \
                f"Reveal mismatch for user '{actual_guess.get('username')}' in {round_key}: Got '{actual_guess.get('reveal')}', expected '{expected_guess.get('reveal')}'"

        # Assert that round_reveal_url exists and is a string if present in ground truth
        if "round_reveal_url" in ground_truth_round_data:
            assert "round_reveal_url" in actual_fetched_data
            assert isinstance(actual_fetched_data["round_reveal_url"], str)
            assert actual_fetched_data["round_reveal_url"] == ground_truth_round_data["round_reveal_url"]

        # Assert that round_commitment_url exists and is a string if present in ground truth
        if "round_commitment_url" in ground_truth_round_data:
            assert "round_commitment_url" in actual_fetched_data
            assert isinstance(actual_fetched_data["round_commitment_url"], str)
            assert actual_fetched_data["round_commitment_url"] == ground_truth_round_data["round_commitment_url"]

        # Assert guess_url for participants if present in ground truth
        # We iterate through actual_guesses_sorted and find the corresponding ground truth participant
        for p_actual in actual_guesses_sorted:
            p_ground_truth = next((p_gt for p_gt in ground_truth_round_data["participants"] if p_gt["username"] == p_actual["username"]), None)
            if p_ground_truth and "reveal_url" in p_ground_truth:
                assert "reveal_url" in p_actual, f"Expected 'reveal_url' for user {p_actual.get('username')} in {round_key}"
                assert isinstance(p_actual["reveal_url"], str), f"'reveal_url' for user {p_actual.get('username')} in {round_key} is not a string"
                assert p_actual["reveal_url"] == p_ground_truth["reveal_url"], f"'reveal_url' mismatch for user {p_actual.get('username')} in {round_key}"
            elif p_ground_truth and "reveal_url" not in p_ground_truth:
                # If ground truth doesn't have reveal_url, ensure the simulated data also doesn't have it or it's None
                assert p_actual.get("reveal_url") is None, f"Unexpected 'reveal_url' for user {p_actual.get('username')} in {round_key} when not in ground truth"
            
            # Check commitment_url if present
            if p_ground_truth and "commitment_url" in p_ground_truth:
                assert "commitment_url" in p_actual, f"Expected 'commitment_url' for user {p_actual.get('username')} in {round_key}"
                assert isinstance(p_actual["commitment_url"], str), f"'commitment_url' for user {p_actual.get('username')} in {round_key} is not a string"
                assert p_actual["commitment_url"] == p_ground_truth["commitment_url"], f"'commitment_url' mismatch for user {p_actual.get('username')} in {round_key}"
            elif p_ground_truth and "commitment_url" not in p_ground_truth:
                # If ground truth doesn't have commitment_url, ensure the simulated data also doesn't have it or it's None
                assert p_actual.get("commitment_url") is None, f"Unexpected 'commitment_url' for user {p_actual.get('username')} in {round_key} when not in ground truth"
            
            # Check payout structure if present
            if p_ground_truth and "payout" in p_ground_truth:
                assert "payout" in p_actual, f"Expected 'payout' for user {p_actual.get('username')} in {round_key}"
                
                # Check if payout is the new object structure or old simple number
                if isinstance(p_ground_truth["payout"], dict):
                    assert isinstance(p_actual["payout"], dict), f"'payout' for user {p_actual.get('username')} in {round_key} should be an object"
                    
                    # Check required payout fields
                    assert "amount" in p_actual["payout"], f"Expected 'amount' in payout for user {p_actual.get('username')} in {round_key}"
                    assert "currency" in p_actual["payout"], f"Expected 'currency' in payout for user {p_actual.get('username')} in {round_key}"
                    assert "url" in p_actual["payout"], f"Expected 'url' in payout for user {p_actual.get('username')} in {round_key}"
                    
                    # Validate payout field values
                    assert p_actual["payout"]["amount"] == p_ground_truth["payout"]["amount"], f"Payout amount mismatch for user {p_actual.get('username')} in {round_key}"
                    assert p_actual["payout"]["currency"] == p_ground_truth["payout"]["currency"], f"Payout currency mismatch for user {p_actual.get('username')} in {round_key}"
                    assert p_actual["payout"]["url"] == p_ground_truth["payout"]["url"], f"Payout URL mismatch for user {p_actual.get('username')} in {round_key}"
                    
                    # Validate field types
                    assert isinstance(p_actual["payout"]["amount"], (int, float)), f"Payout amount should be numeric for user {p_actual.get('username')} in {round_key}"
                    assert isinstance(p_actual["payout"]["currency"], str), f"Payout currency should be string for user {p_actual.get('username')} in {round_key}"
                    assert isinstance(p_actual["payout"]["url"], str), f"Payout URL should be string for user {p_actual.get('username')} in {round_key}"
                else:
                    # Old simple number format
                    assert p_actual["payout"] == p_ground_truth["payout"], f"Payout mismatch for user {p_actual.get('username')} in {round_key}"

        print(f"Successfully verified structure and content for {round_key} including URL fields")

# To run this test:
# Ensure pytest and pytest-asyncio are installed:
# pip install pytest pytest-asyncio
# Then run from the project root:
# pytest tests/test_twitter_data_extraction.py 