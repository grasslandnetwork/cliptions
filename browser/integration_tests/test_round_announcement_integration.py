"""
Test script for block announcement functionality
"""

import asyncio
import sys
from datetime import datetime, timedelta
from pathlib import Path

# Add the parent directory to sys.path to allow imports
sys.path.append(str(Path(__file__).parent))

from browser.validator.announce_block import (
    BlockAnnouncementTask,
    create_standard_block_announcement
)

async def main():
    # Create a block announcement task
    task = BlockAnnouncementTask()
    
    try:
    # Create test announcement data
    announcement_data = create_standard_block_announcement(
        block_num="TEST-BLOCK-001",
        livestream_url="https://www.youtube.com/watch?v=SMCRQj9Hbx8",  # Using provided YouTube URL
        entry_fee=0.001,  # 0.001 TAO
        commitment_hours=24,  # 24 hours for commitments
        reveal_hours=48,    # 48 hours for reveals
    )
    
    print("Posting block announcement...")
    result = await task.execute(data=announcement_data)
    
    if result.success:
        print(f"Successfully posted announcement!")
        print(f"Tweet URL: {result.tweet_url}")
        print(f"Tweet ID: {result.tweet_id}")
    else:
        print(f"Failed to post announcement: {result.error_message}")
    
    finally:
        # Ensure proper cleanup of browser resources
        print("Cleaning up browser resources...")
        await task.cleanup()
        print("✅ Cleanup completed")

if __name__ == "__main__":
    asyncio.run(main()) 