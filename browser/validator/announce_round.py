"""
Round Announcement Module for RealMir Validator

This module implements the TwitterPostingInterface to post new round announcements.
The Validator uses this to kick off each prediction round by posting details about
the upcoming round including entry fees, deadlines, and participation instructions.
"""

import logging
from datetime import datetime, timedelta
from typing import Dict, Any, Optional
from pydantic import BaseModel, Field

try:
    # Try relative imports first (when used as part of package)
    from ..core.interfaces import TwitterPostingInterface
    from ..core.base_task import BaseTwitterTask
except ImportError:
    # Fall back to direct imports (when used as standalone)
    from interfaces import TwitterPostingInterface
    from base_task import BaseTwitterTask


class RoundAnnouncementData(BaseModel):
    """Data structure for round announcement content"""
    round_id: str = Field(..., description="Unique identifier for the round")
    entry_fee: float = Field(..., description="Entry fee in TAO")
    commitment_deadline: datetime = Field(..., description="Deadline for commitment submissions")
    reveal_deadline: datetime = Field(..., description="Deadline for reveal submissions")
    prize_pool: float = Field(..., description="Total prize pool in TAO")
    instructions: str = Field(default="", description="Additional instructions for participants")
    hashtags: list[str] = Field(default_factory=lambda: ["#RealMir", "#TAO", "#BittensorPrediction"])


class RoundAnnouncementResult(BaseModel):
    """Result from posting a round announcement"""
    success: bool = Field(..., description="Whether the announcement was posted successfully")
    tweet_url: Optional[str] = Field(None, description="URL of the posted tweet")
    tweet_id: Optional[str] = Field(None, description="ID of the posted tweet")
    round_id: str = Field(..., description="The announced round ID")
    timestamp: datetime = Field(default_factory=datetime.now, description="When the announcement was posted")
    error_message: Optional[str] = Field(None, description="Error message if posting failed")


class RoundAnnouncementTask(BaseTwitterTask):
    """
    Task for posting round announcements to Twitter.
    
    This task implements the TwitterPostingInterface to handle the Validator's
    initial announcement of a new prediction round.
    """
    
    def __init__(self, config_path: Optional[str] = None):
        super().__init__(config_path)
        self.logger = logging.getLogger(__name__)
    
    async def execute(self, **kwargs) -> RoundAnnouncementResult:
        """
        Execute the round announcement posting task.
        
        Args:
            **kwargs: Should contain RoundAnnouncementData fields or a 'data' key
                     with RoundAnnouncementData instance
        
        Returns:
            RoundAnnouncementResult: Result of the announcement posting
        """
        try:
            # Parse input data
            if 'data' in kwargs:
                announcement_data = kwargs['data']
                if not isinstance(announcement_data, RoundAnnouncementData):
                    announcement_data = RoundAnnouncementData(**announcement_data)
            else:
                announcement_data = RoundAnnouncementData(**kwargs)
            
            self.logger.info(f"Starting round announcement for round {announcement_data.round_id}")
            
            # Format the announcement content
            content = self.format_content(announcement_data)
            
            # Post the announcement
            result = await self.post_content(content)
            
            return RoundAnnouncementResult(
                success=True,
                tweet_url=result.get('tweet_url'),
                tweet_id=result.get('tweet_id'),
                round_id=announcement_data.round_id,
                timestamp=datetime.now()
            )
            
        except Exception as e:
            self.logger.error(f"Failed to post round announcement: {str(e)}")
            return RoundAnnouncementResult(
                success=False,
                round_id=kwargs.get('round_id', 'unknown'),
                error_message=str(e)
            )
    
    def format_content(self, data: RoundAnnouncementData) -> str:
        """
        Format the round announcement content for Twitter.
        
        Args:
            data: Round announcement data
            
        Returns:
            Formatted tweet content
        """
        content_parts = [
            f"🎯 NEW ROUND: {data.round_id}",
            "",
            f"💰 Entry Fee: {data.entry_fee} TAO",
            f"🏆 Prize Pool: {data.prize_pool} TAO",
            "",
            f"⏰ Commitment Deadline: {data.commitment_deadline.strftime('%Y-%m-%d %H:%M UTC')}",
            f"📅 Reveal Deadline: {data.reveal_deadline.strftime('%Y-%m-%d %H:%M UTC')}",
            "",
            "📋 To Participate:",
            "1. Reply with your commitment hash + wallet address",
            "2. Wait for entry fee address assignment",
            "3. Pay entry fee before deadline",
            "4. Submit your reveal when target is posted",
        ]
        
        if data.instructions:
            content_parts.extend(["", f"ℹ️ {data.instructions}"])
        
        # Add hashtags
        if data.hashtags:
            content_parts.extend(["", " ".join(data.hashtags)])
        
        return "\n".join(content_parts)
    
    async def post_content(self, content: str) -> Dict[str, Any]:
        """
        Post content to Twitter using browser automation.
        
        Args:
            content: The formatted tweet content
            
        Returns:
            Dictionary with posting results
        """
        try:
            # Set up the browser agent
            agent = await self.setup_agent()
            
            # Navigate to Twitter compose page
            await agent.get("https://twitter.com/compose/tweet")
            
            # Wait for the compose textarea to be available
            await agent.wait_for_element("div[data-testid='tweetTextarea_0']", timeout=10)
            
            # Type the content
            tweet_textarea = await agent.find_element("div[data-testid='tweetTextarea_0']")
            await tweet_textarea.clear()
            await tweet_textarea.type(content)
            
            # Click the tweet button
            tweet_button = await agent.find_element("div[data-testid='tweetButtonInline']")
            await tweet_button.click()
            
            # Wait for the tweet to be posted and get the URL
            await agent.wait(3)  # Wait for posting to complete
            
            # Get the current URL or tweet ID (implementation depends on browser-use capabilities)
            current_url = await agent.get_current_url()
            
            self.logger.info(f"Successfully posted round announcement")
            
            return {
                "success": True,
                "tweet_url": current_url,
                "tweet_id": self._extract_tweet_id_from_url(current_url)
            }
            
        except Exception as e:
            self.logger.error(f"Failed to post content: {str(e)}")
            raise
    
    def _extract_tweet_id_from_url(self, url: str) -> Optional[str]:
        """Extract tweet ID from Twitter URL"""
        if not url or 'twitter.com' not in url and 'x.com' not in url:
            return None
        
        # Twitter URLs typically have format: https://twitter.com/username/status/tweet_id
        parts = url.split('/')
        if 'status' in parts:
            status_index = parts.index('status')
            if status_index + 1 < len(parts):
                return parts[status_index + 1]
        
        return None
    
    def validate_output(self, result: Any) -> bool:
        """
        Validate that the announcement was posted successfully.
        
        Args:
            result: The result to validate
            
        Returns:
            True if valid, False otherwise
        """
        if not isinstance(result, RoundAnnouncementResult):
            return False
        
        return result.success and (result.tweet_url is not None or result.tweet_id is not None)


# Utility functions for creating announcement data

def create_standard_round_announcement(
    round_id: str,
    entry_fee: float = 0.001,
    prize_pool: Optional[float] = None,
    commitment_hours: int = 24,
    reveal_hours: int = 48
) -> RoundAnnouncementData:
    """
    Create a standard round announcement with default timing.
    
    Args:
        round_id: Unique identifier for the round
        entry_fee: Entry fee in TAO (default: 0.001)
        prize_pool: Prize pool in TAO (defaults to entry_fee if not specified)
        commitment_hours: Hours from now until commitment deadline
        reveal_hours: Hours from now until reveal deadline
        
    Returns:
        RoundAnnouncementData instance
    """
    now = datetime.now()
    commitment_deadline = now + timedelta(hours=commitment_hours)
    reveal_deadline = now + timedelta(hours=reveal_hours)
    
    if prize_pool is None:
        prize_pool = entry_fee
    
    return RoundAnnouncementData(
        round_id=round_id,
        entry_fee=entry_fee,
        commitment_deadline=commitment_deadline,
        reveal_deadline=reveal_deadline,
        prize_pool=prize_pool
    )


def create_custom_round_announcement(
    round_id: str,
    entry_fee: float,
    commitment_deadline: datetime,
    reveal_deadline: datetime,
    prize_pool: float,
    instructions: str = "",
    hashtags: Optional[list[str]] = None
) -> RoundAnnouncementData:
    """
    Create a custom round announcement with specific parameters.
    
    Args:
        round_id: Unique identifier for the round
        entry_fee: Entry fee in TAO
        commitment_deadline: When commitments are due
        reveal_deadline: When reveals are due
        prize_pool: Total prize pool in TAO
        instructions: Additional instructions for participants
        hashtags: Custom hashtags (uses defaults if not provided)
        
    Returns:
        RoundAnnouncementData instance
    """
    return RoundAnnouncementData(
        round_id=round_id,
        entry_fee=entry_fee,
        commitment_deadline=commitment_deadline,
        reveal_deadline=reveal_deadline,
        prize_pool=prize_pool,
        instructions=instructions,
        hashtags=hashtags or ["#RealMir", "#TAO", "#BittensorPrediction"]
    ) 