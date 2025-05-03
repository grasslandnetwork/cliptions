#!/usr/bin/env python3
import json
import sys
from pathlib import Path
from generate_commitment import generate_commitment

def verify_round_commitments(round_id):
    """Verify that all guesses and salts in a round match their commitments.
    
    Args:
        round_id: The ID of the round to verify (e.g., 'round2')
        
    Returns:
        bool: True if all commitments are valid, False otherwise
    """
    guesses_file = Path("rounds/guesses.json")
    if not guesses_file.exists():
        print(f"Error: {guesses_file} does not exist")
        return False
        
    with open(guesses_file, 'r') as f:
        round_data = json.load(f)
    
    if round_id not in round_data:
        print(f"Error: Round {round_id} not found in guesses.json")
        return False
    
    participants = round_data[round_id]['participants']
    
    if not participants:
        print(f"No participants found for {round_id}")
        return False
    
    all_valid = True
    for participant in participants:
        username = participant['username']
        guess = participant['guess']
        salt = participant['salt']
        stored_commitment = participant['commitment']
        
        # Skip if no guess or salt (not yet provided)
        if not guess or not salt:
            print(f"Skipping {username}: missing guess or salt")
            all_valid = False
            continue
        
        # Generate commitment from guess and salt
        calculated_commitment = generate_commitment(guess, salt)
        
        # Check if it matches the stored commitment
        if calculated_commitment == stored_commitment:
            print(f"✅ {username}'s commitment is valid")
            participant['valid'] = True
        else:
            print(f"❌ {username}'s commitment is INVALID")
            print(f"  Stored:     {stored_commitment}")
            print(f"  Calculated: {calculated_commitment}")
            participant['valid'] = False
            all_valid = False
    
    # Save updates to valid field
    with open(guesses_file, 'w') as f:
        json.dump(round_data, f, indent=4)
    
    return all_valid

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python3 verify_commitments.py <round_id>")
        sys.exit(1)
        
    round_id = sys.argv[1]
    if verify_round_commitments(round_id):
        print(f"\nAll commitments for {round_id} are valid.")
        sys.exit(0)
    else:
        print(f"\nSome commitments for {round_id} are not valid.")
        sys.exit(1) 