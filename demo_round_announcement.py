#!/usr/bin/env python3
"""
Demo script for testing the Round Announcement module

This script demonstrates how to use the RoundAnnouncementTask to create
and format round announcements for the RealMir prediction network.
"""

import sys
import asyncio
from datetime import datetime, timedelta
from pathlib import Path

# Add browser directory to path
browser_path = Path(__file__).parent / "browser"
sys.path.insert(0, str(browser_path))
sys.path.append(str(browser_path / "validator"))
sys.path.append(str(browser_path / "core"))

# Import modules directly
from announce_round import (
    RoundAnnouncementTask,
    create_standard_round_announcement,
    create_custom_round_announcement
)


async def demo_round_announcement():
    """Demonstrate the round announcement functionality"""
    print("🎯 RealMir Round Announcement Demo")
    print("=" * 50)
    
    # Create a standard round announcement
    print("\n1. Creating a standard round announcement:")
    standard_data = create_standard_round_announcement(
        round_id="demo_round_1",
        entry_fee=0.001,
        prize_pool=0.005
    )
    
    # Initialize the task
    task = RoundAnnouncementTask()
    
    # Format the content (without actually posting)
    content = task.format_content(standard_data)
    print("\nFormatted announcement content:")
    print("-" * 30)
    print(content)
    print("-" * 30)
    
    # Create a custom round announcement
    print("\n2. Creating a custom round announcement:")
    now = datetime.now()
    custom_data = create_custom_round_announcement(
        round_id="demo_round_2",
        entry_fee=0.002,
        commitment_deadline=now + timedelta(hours=12),
        reveal_deadline=now + timedelta(hours=36),
        prize_pool=0.010,
        instructions="This is a special demo round with custom parameters",
        hashtags=["#RealMir", "#Demo", "#CustomRound"]
    )
    
    custom_content = task.format_content(custom_data)
    print("\nCustom announcement content:")
    print("-" * 30)
    print(custom_content)
    print("-" * 30)
    
    # Test URL extraction
    print("\n3. Testing tweet ID extraction:")
    test_urls = [
        "https://twitter.com/realmir_testnet/status/1234567890",
        "https://x.com/realmir_testnet/status/9876543210",
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
    asyncio.run(demo_round_announcement()) 