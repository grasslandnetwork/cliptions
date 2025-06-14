"""
Test script for miner commitment submission functionality
"""

import asyncio
import sys
from pathlib import Path

# Add the project root to sys.path to allow imports
sys.path.append(str(Path(__file__).parent.parent.parent))

from browser.miner.submit_commitment import (
    CommitmentSubmissionTask,
    create_commitment_submission
)
from core.generate_commitment import generate_commitment

async def main():
    # Create a commitment submission task
    task = CommitmentSubmissionTask()
    
    try:
        # Create test commitment data using the announcement we just posted
        announcement_url = "https://x.com/realmir_testnet/status/1933572314794226070"  # Latest announcement from our test
        
        commitment_data = create_commitment_submission(
            prediction="Cat sanctuary with caretakers feeding cats",
            salt="test-salt-123",
            wallet_address="5Co2unDtZKZDzYNZHT2fUMkEnpVWnassfbuabvZmGTrYKgtD",  # Example wallet from rounds data
            reply_to_url=announcement_url
        )
        
        print("Submitting commitment...")
        print(f"Commitment Hash: {generate_commitment(commitment_data.prediction, commitment_data.salt)}")
        print(f"Wallet: {commitment_data.wallet_address}")
        print(f"Replying to: {commitment_data.reply_to_url}")
        
        result = await task.execute(data=commitment_data)
        
        if result.success:
            print(f"✅ Successfully submitted commitment!")
            print(f"Tweet URL: {result.tweet_url}")
            print(f"Tweet ID: {result.tweet_id}")
            print(f"Commitment Hash: {result.commitment_hash}")
        else:
            print(f"❌ Failed to submit commitment: {result.error_message}")
    
    finally:
        # Ensure proper cleanup of browser resources
        print("Cleaning up browser resources...")
        await task.cleanup()
        print("✅ Cleanup completed")

if __name__ == "__main__":
    print("🧪 Testing Miner Commitment Submission")
    print("=" * 50)
    
    # First, let's test the commitment hash generation without browser automation
    from browser.miner.submit_commitment import CommitmentSubmissionTask
    task = CommitmentSubmissionTask()
    
    test_prediction = "Cat sanctuary with caretakers feeding cats"
    test_salt = "test-salt-123"
    generated_hash = generate_commitment(test_prediction, test_salt)
    
    print(f"🔐 Generated Commitment Hash:")
    print(f"Prediction: {test_prediction}")
    print(f"Salt: {test_salt}")
    print(f"Hash: {generated_hash}")
    print()
    
    print(f"📋 Test Parameters:")
    print(f"Replying to: https://x.com/realmir_testnet/status/1933572314794226070")
    print(f"Wallet: 5Co2unDtZKZDzYNZHT2fUMkEnpVWnassfbuabvZmGTrYKgtD")
    print()
    
    # Ask user if they want to continue with browser automation
    response = input("Continue with browser automation test? (y/n): ").lower().strip()
    
    if response == 'y':
        print("\n🤖 Starting browser automation test...")
        asyncio.run(main())
    else:
        print("Skipping browser automation test.") 