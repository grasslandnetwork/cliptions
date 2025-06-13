"""
Test script for round announcement functionality
"""

import asyncio
import sys
from datetime import datetime, timedelta
from pathlib import Path

# Add the parent directory to sys.path to allow imports
sys.path.append(str(Path(__file__).parent))

from browser.validator.announce_round import (
    RoundAnnouncementTask,
    create_standard_round_announcement
)

async def main():
    # Create a round announcement task
    task = RoundAnnouncementTask()
    
    # Create test announcement data
    announcement_data = create_standard_round_announcement(
        round_id="TEST-ROUND-001",
        livestream_url="https://www.youtube.com/watch?v=SMCRQj9Hbx8",  # Using provided YouTube URL
        entry_fee=0.001,  # 0.001 TAO
        commitment_hours=24,  # 24 hours for commitments
        reveal_hours=48,    # 48 hours for reveals
    )
    
    print("Posting round announcement...")
    result = await task.execute(data=announcement_data)
    
    if result.success:
        print(f"Successfully posted announcement!")
        print(f"Tweet URL: {result.tweet_url}")
        print(f"Tweet ID: {result.tweet_id}")
    else:
        print(f"Failed to post announcement: {result.error_message}")

if __name__ == "__main__":
    asyncio.run(main()) 