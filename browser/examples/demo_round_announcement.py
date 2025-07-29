#!/usr/bin/env python3
"""
Demo script for testing the Block Announcement module

This script demonstrates how to use the BlockAnnouncementTask to create
and format block announcements for the Cliptions prediction network.
"""

import sys
import asyncio
from datetime import datetime, timedelta
from pathlib import Path

# Ensure project root is in path
ROOT_DIR = Path(__file__).parent.parent.parent
sys.path.insert(0, str(ROOT_DIR))

# Import module via package path
from browser.validator.announce_block import (
    BlockAnnouncementTask,
    create_standard_block_announcement,
    create_custom_block_announcement
)


async def demo_block_announcement():
    """Demonstrate the block announcement functionality"""
    print("🎯 Cliptions Block Announcement Demo")
    print("=" * 50)
    
    # Create a standard block announcement
    print("\n1. Creating a standard block announcement:")
    standard_data = create_standard_block_announcement(
        block_num="demo_block_1",
        entry_fee=0.001,
        prize_pool=0.005
    )
    
    # Initialize the task
    task = BlockAnnouncementTask()
    
    # Format the content (without actually posting)
    content = task.format_content(standard_data)
    print("\nFormatted announcement content:")
    print("-" * 30)
    print(content)
    print("-" * 30)
    
    # Create a custom block announcement
    print("\n2. Creating a custom block announcement:")
    now = datetime.now()
    custom_data = create_custom_block_announcement(
        block_num="demo_block_2",
        entry_fee=0.002,
        commitment_deadline=now + timedelta(hours=12),
        reveal_deadline=now + timedelta(hours=36),
        prize_pool=0.010,
        instructions="This is a special demo block with custom parameters",
        hashtags=["#cliptions", "$TAO", "#customblock"]
    )
    
    custom_content = task.format_content(custom_data)
    print("\nCustom announcement content:")
    print("-" * 30)
    print(custom_content)
    print("-" * 30)
    
    # Test URL extraction
    print("\n3. Testing tweet ID extraction:")
    test_urls = [
        "https://twitter.com/cliptions_test/status/1234567890",
        "https://x.com/cliptions_test/status/9876543210",
        "https://example.com/invalid"
    ]
    
    for url in test_urls:
        tweet_id = task._extract_tweet_id_from_url(url)
        print(f"URL: {url}")
        print(f"Tweet ID: {tweet_id}")
        print()
    
    print("✅ Demo completed successfully!")
    print("\nNote: This demo only shows content formatting.")
    print("Actual Twitter posting would require browser automation setup.")


if __name__ == "__main__":
    # Run the demo
    asyncio.run(demo_block_announcement()) 