"""
Tests for the Round Announcement module

This module tests the RoundAnnouncementTask to ensure it correctly formats
and posts round announcements to Twitter.
"""

import pytest
import sys
from datetime import datetime, timedelta
from unittest.mock import AsyncMock, MagicMock, patch
from pathlib import Path

# Add project root to path for package import
sys.path.insert(0, str(Path(__file__).parent.parent))

# Import via package path
from browser.validator.announce_round import (
    RoundAnnouncementTask,
    RoundAnnouncementData,
    RoundAnnouncementResult,
    create_standard_round_announcement,
    create_custom_round_announcement
)


class TestRoundAnnouncementData:
    """Test the RoundAnnouncementData model"""
    
    def test_valid_announcement_data(self):
        """Test creating valid announcement data"""
        now = datetime.now()
        data = RoundAnnouncementData(
            round_id="test_round_1",
            entry_fee=0.001,
            commitment_deadline=now + timedelta(hours=24),
            reveal_deadline=now + timedelta(hours=48),
            prize_pool=0.001
        )
        
        assert data.round_id == "test_round_1"
        assert data.entry_fee == 0.001
        assert data.prize_pool == 0.001
        assert len(data.hashtags) == 3
        assert "#RealMir" in data.hashtags
    
    def test_custom_hashtags(self):
        """Test creating announcement data with custom hashtags"""
        now = datetime.now()
        custom_hashtags = ["#CustomTag", "#Test"]
        
        data = RoundAnnouncementData(
            round_id="test_round_2",
            entry_fee=0.002,
            commitment_deadline=now + timedelta(hours=12),
            reveal_deadline=now + timedelta(hours=36),
            prize_pool=0.005,
            hashtags=custom_hashtags
        )
        
        assert data.hashtags == custom_hashtags


class TestRoundAnnouncementTask:
    """Test the RoundAnnouncementTask implementation"""
    
    @pytest.fixture
    def task(self):
        """Create a RoundAnnouncementTask instance for testing"""
        return RoundAnnouncementTask()
    
    @pytest.fixture
    def sample_data(self):
        """Create sample announcement data for testing"""
        now = datetime.now()
        return RoundAnnouncementData(
            round_id="test_round_1",
            entry_fee=0.001,
            commitment_deadline=now + timedelta(hours=24),
            reveal_deadline=now + timedelta(hours=48),
            prize_pool=0.001,
            instructions="This is a test round"
        )
    
    def test_format_content(self, task, sample_data):
        """Test the content formatting functionality"""
        content = task.format_content(sample_data)
        
        assert "🎯 NEW ROUND: test_round_1" in content
        assert "💰 Entry Fee: 0.001 TAO" in content
        assert "🏆 Prize Pool: 0.001 TAO" in content
        assert "ℹ️ This is a test round" in content
        assert "#RealMir" in content
        assert "1. Reply with your commitment hash + wallet address" in content
    
    def test_format_content_no_instructions(self, task):
        """Test content formatting without instructions"""
        now = datetime.now()
        data = RoundAnnouncementData(
            round_id="test_round_2",
            entry_fee=0.002,
            commitment_deadline=now + timedelta(hours=24),
            reveal_deadline=now + timedelta(hours=48),
            prize_pool=0.002
        )
        
        content = task.format_content(data)
        
        assert "🎯 NEW ROUND: test_round_2" in content
        assert "ℹ️" not in content  # No instructions section
        assert "#RealMir" in content
    
    def test_extract_tweet_id_from_url(self, task):
        """Test tweet ID extraction from various URL formats"""
        # Test Twitter URL
        twitter_url = "https://twitter.com/realmir_testnet/status/1234567890"
        tweet_id = task._extract_tweet_id_from_url(twitter_url)
        assert tweet_id == "1234567890"
        
        # Test X.com URL
        x_url = "https://x.com/realmir_testnet/status/9876543210"
        tweet_id = task._extract_tweet_id_from_url(x_url)
        assert tweet_id == "9876543210"
        
        # Test invalid URL
        invalid_url = "https://example.com/something"
        tweet_id = task._extract_tweet_id_from_url(invalid_url)
        assert tweet_id is None
        
        # Test None URL
        tweet_id = task._extract_tweet_id_from_url(None)
        assert tweet_id is None
    
    def test_validate_output_success(self, task):
        """Test output validation for successful results"""
        result = RoundAnnouncementResult(
            success=True,
            tweet_url="https://twitter.com/test/status/123",
            tweet_id="123",
            round_id="test_round"
        )
        
        assert task.validate_output(result) is True
    
    def test_validate_output_failure(self, task):
        """Test output validation for failed results"""
        result = RoundAnnouncementResult(
            success=False,
            round_id="test_round",
            error_message="Test error"
        )
        
        assert task.validate_output(result) is False
    
    def test_validate_output_invalid_type(self, task):
        """Test output validation with invalid result type"""
        assert task.validate_output("invalid") is False
        assert task.validate_output(None) is False
    
    @pytest.mark.asyncio
    async def test_execute_success(self, task, sample_data):
        """Test successful execution of the announcement task"""
        # Mock the post_content method
        mock_result = {
            "success": True,
            "tweet_url": "https://twitter.com/test/status/123",
            "tweet_id": "123"
        }
        
        with patch.object(task, 'post_content', return_value=mock_result):
            result = await task.execute(data=sample_data)
            
            assert isinstance(result, RoundAnnouncementResult)
            assert result.success is True
            assert result.tweet_url == "https://twitter.com/test/status/123"
            assert result.tweet_id == "123"
            assert result.round_id == "test_round_1"
    
    @pytest.mark.asyncio
    async def test_execute_with_kwargs(self, task):
        """Test execution with keyword arguments instead of data object"""
        now = datetime.now()
        
        mock_result = {
            "success": True,
            "tweet_url": "https://twitter.com/test/status/456",
            "tweet_id": "456"
        }
        
        with patch.object(task, 'post_content', return_value=mock_result):
            result = await task.execute(
                round_id="kwargs_round",
                entry_fee=0.003,
                commitment_deadline=now + timedelta(hours=24),
                reveal_deadline=now + timedelta(hours=48),
                prize_pool=0.003
            )
            
            assert isinstance(result, RoundAnnouncementResult)
            assert result.success is True
            assert result.round_id == "kwargs_round"
    
    @pytest.mark.asyncio
    async def test_execute_failure(self, task, sample_data):
        """Test execution failure handling"""
        # Mock post_content to raise an exception
        with patch.object(task, 'post_content', side_effect=Exception("Test error")):
            result = await task.execute(data=sample_data)
            
            assert isinstance(result, RoundAnnouncementResult)
            assert result.success is False
            assert result.error_message == "Test error"
            assert result.round_id == "test_round_1"


class TestUtilityFunctions:
    """Test the utility functions for creating announcement data"""
    
    def test_create_standard_round_announcement(self):
        """Test creating a standard round announcement"""
        data = create_standard_round_announcement("standard_round")
        
        assert data.round_id == "standard_round"
        assert data.entry_fee == 0.001
        assert data.prize_pool == 0.001
        assert data.commitment_deadline > datetime.now()
        assert data.reveal_deadline > data.commitment_deadline
    
    def test_create_standard_round_announcement_custom_params(self):
        """Test creating a standard round announcement with custom parameters"""
        data = create_standard_round_announcement(
            "custom_standard",
            entry_fee=0.005,
            prize_pool=0.010,
            commitment_hours=12,
            reveal_hours=36
        )
        
        assert data.round_id == "custom_standard"
        assert data.entry_fee == 0.005
        assert data.prize_pool == 0.010
        
        # Check timing is approximately correct (within 1 minute)
        now = datetime.now()
        expected_commit = now + timedelta(hours=12)
        expected_reveal = now + timedelta(hours=36)
        
        assert abs((data.commitment_deadline - expected_commit).total_seconds()) < 60
        assert abs((data.reveal_deadline - expected_reveal).total_seconds()) < 60
    
    def test_create_custom_round_announcement(self):
        """Test creating a custom round announcement"""
        now = datetime.now()
        commit_deadline = now + timedelta(hours=8)
        reveal_deadline = now + timedelta(hours=24)
        
        data = create_custom_round_announcement(
            round_id="custom_round",
            entry_fee=0.002,
            commitment_deadline=commit_deadline,
            reveal_deadline=reveal_deadline,
            prize_pool=0.008,
            instructions="Custom instructions",
            hashtags=["#Custom", "#Test"]
        )
        
        assert data.round_id == "custom_round"
        assert data.entry_fee == 0.002
        assert data.prize_pool == 0.008
        assert data.commitment_deadline == commit_deadline
        assert data.reveal_deadline == reveal_deadline
        assert data.instructions == "Custom instructions"
        assert data.hashtags == ["#Custom", "#Test"]
    
    def test_create_custom_round_announcement_default_hashtags(self):
        """Test creating a custom round announcement with default hashtags"""
        now = datetime.now()
        
        data = create_custom_round_announcement(
            round_id="default_hashtags",
            entry_fee=0.001,
            commitment_deadline=now + timedelta(hours=24),
            reveal_deadline=now + timedelta(hours=48),
            prize_pool=0.001
        )
        
        assert data.hashtags == ["#RealMir", "#TAO", "#BittensorPrediction"]


# Integration test that would require actual browser automation
class TestRoundAnnouncementIntegration:
    """Integration tests that would require actual browser automation (marked as slow)"""
    
    @pytest.mark.slow
    @pytest.mark.asyncio
    async def test_full_announcement_flow(self):
        """Test the full announcement flow with mocked browser interactions"""
        # This test would be marked as slow and would typically be run separately
        # It would test the actual browser automation part
        pass


if __name__ == "__main__":
    pytest.main([__file__, "-v"]) 