"""
Block Announcement Module for Cliptions Validator

This module implements the TwitterPostingInterface to post new block announcements.
The Validator uses this to kick off each prediction block by posting details about
the upcoming block including entry fees, deadlines, and participation instructions.
"""

import logging
import os
from datetime import datetime, timedelta
from typing import Dict, Any, Optional
from pydantic import BaseModel, Field
from browser_use import Agent, Browser

try:
    # Try relative imports first (when used as part of package)
    from ..core.interfaces import TwitterPostingInterface
    from ..core.base_task import BaseTwitterTask
except ImportError:
    # Fall back to direct imports (when used as standalone via sys.path tweaks)
    from core.interfaces import TwitterPostingInterface
    from core.base_task import BaseTwitterTask


class BlockAnnouncementData(BaseModel):
    """Data structure for block announcement content"""
    block_num: str = Field(..., description="Unique identifier for the block")
    entry_fee: float = Field(..., description="Entry fee in TAO")
    commitment_deadline: datetime = Field(..., description="Deadline for commitment submissions")
    reveal_deadline: datetime = Field(..., description="Deadline for reveal submissions")
    livestream_url: str = Field(..., description="URL of the livestream players are predicting")
    instructions: str = Field(default="", description="Additional instructions for participants")
    hashtags: list[str] = Field(default_factory=lambda: ["#cliptions", "$TAO"])


class BlockAnnouncementResult(BaseModel):
    """Result from posting a block announcement"""
    success: bool = Field(..., description="Whether the announcement was posted successfully")
    tweet_url: Optional[str] = Field(None, description="URL of the posted tweet")
    tweet_id: Optional[str] = Field(None, description="ID of the posted tweet")
    block_num: str = Field(..., description="The announced block ID")
    timestamp: datetime = Field(default_factory=datetime.now, description="When the announcement was posted")
    error_message: Optional[str] = Field(None, description="Error message if posting failed")


class BlockAnnouncementTask(BaseTwitterTask):
    """
    Task for posting block announcements to Twitter.
    
    This task implements the TwitterPostingInterface to handle the Validator's
    initial announcement of a new prediction block.
    """
    
    def __init__(self, config_path: Optional[str] = None):
        super().__init__(config_path)
        self.logger = logging.getLogger(__name__)
    
    async def execute(self, **kwargs) -> BlockAnnouncementResult:
        """
        Execute the block announcement posting task.
        
        Args:
            **kwargs: Should contain BlockAnnouncementData fields or a 'data' key
                     with BlockAnnouncementData instance
        
        Returns:
            BlockAnnouncementResult: Result of the announcement posting
        """
        try:
            # Use the base class execute method which includes cleanup
            result = await super().execute(**kwargs)
            return result
        except Exception as e:
            self.logger.error(f"Failed to post block announcement: {str(e)}")
            return BlockAnnouncementResult(
                success=False,
                block_num=kwargs.get('block_num', 'unknown'),
                error_message=str(e)
            )
    
    async def _execute_task(self, **kwargs) -> BlockAnnouncementResult:
        """
        Internal task execution method called by the base class.
        
        Args:
            **kwargs: Should contain BlockAnnouncementData fields or a 'data' key
                     with BlockAnnouncementData instance
        
        Returns:
            BlockAnnouncementResult: Result of the announcement posting
        """
        # Parse input data
        if 'data' in kwargs:
            announcement_data = kwargs['data']
            if not isinstance(announcement_data, BlockAnnouncementData):
                announcement_data = BlockAnnouncementData(**announcement_data)
        else:
            announcement_data = BlockAnnouncementData(**kwargs)
        
        self.logger.info(f"Starting block announcement for block {announcement_data.block_num}")
        
        # Format the announcement content
        content = self.format_content(announcement_data)
        
        # Post the announcement
        result = await self.post_content(content)
        
        return BlockAnnouncementResult(
            success=True,
            tweet_url=result.get('tweet_url'),
            tweet_id=result.get('tweet_id'),
            block_num=announcement_data.block_num,
            timestamp=datetime.now()
        )
    
    def format_content(self, data: BlockAnnouncementData) -> str:
        """
        Format the block announcement content for Twitter.
        
        Args:
            data: Block announcement data
            
        Returns:
            Formatted tweet content
        """
        # Extract block number from block_num (e.g., "TEST-BLOCK-001" -> "TEST BLOCK 001")
        block_display = data.block_num.replace("-", " ").upper()
        
        # Format commitment deadline as readable time (assume UTC if no timezone)
        commitment_time = data.commitment_deadline.strftime('%I:%M:%S %p UTC on %B %d, %Y')
        
        # Combine all hashtags at the top
        block_hashtag = f"#{data.block_num.lower().replace('-', '')}"
        all_hashtags = [block_hashtag, "#blockannouncement"] + data.hashtags
        
        content_parts = [
            " ".join(all_hashtags),
            f"{block_display} - Hash Your Prediction",
            "",
            "How To Play:",
            f"1. Watch: {data.livestream_url}",
            "2. Generate your commitment hash (see instructions)",
            f"3. Reply BEFORE {commitment_time}:",
            "",
            "Reply with:",
            "Commit: [hash]",
            "Wallet: [address]"
        ]
        
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
            # 1. Define initial actions to navigate directly to the URL without LLM
            initial_actions = [
                {'go_to_url': {'url': 'https://x.com/compose/post'}},
            ]

            # 2. Define a very specific task using the confirmed data-testid selectors
            task_description = f"""
            You are on the Twitter/X compose page. Your task is to post a tweet with the following content:

            ---
            {content}
            ---

            Follow these steps precisely:
            1. Locate the tweet input area using the selector `[data-testid="tweetTextarea_0"]`.
            2. Click on the input area to ensure it is focused.
            3. Use the `send_keys` action to type the exact content provided above into the input area.
            4. Locate the 'Post' button using the selector `[data-testid="tweetButtonInline"]`.
            5. Click the 'Post' button to publish the tweet.
            6. Wait for confirmation that the tweet was sent, then use the `done` action.
            """
            
            # 3. Set up the browser agent, passing the initial actions
            agent = await self.setup_agent(
                task=task_description,
                initial_actions=initial_actions,
            )
            
            # Run the agent
            print("🤖 Agent starting task: Posting tweet...")
            result = await agent.run(max_steps=10)
            
            print(f"Agent finished with result: {result}")
            
            # Extract tweet URL and ID from the final result or agent history
            tweet_url = "https://twitter.com/placeholder_tweet_url"  # Placeholder
            tweet_id = "placeholder_tweet_id" # Placeholder
            
            if hasattr(result, 'history'):
                # You can add logic here to parse history for the final URL if needed
                pass

            return {
                "success": True,
                "tweet_url": tweet_url,
                "tweet_id": tweet_id,
                "message": "Successfully posted block announcement"
            }
            
        except Exception as e:
            logging.error(f"Failed to post content to Twitter: {e}")
            import traceback
            traceback.print_exc()
            return {
                "success": False,
                "message": str(e)
            }
    
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
    
    def validate_output(self, result: Any) -> BlockAnnouncementResult:
        """
        Validate that the announcement was posted successfully.
        
        Args:
            result: The result to validate
            
        Returns:
            Validated BlockAnnouncementResult
        """
        if isinstance(result, BlockAnnouncementResult):
            return result
        
        # If it's not already a BlockAnnouncementResult, something went wrong
        return BlockAnnouncementResult(
            success=False,
            block_num="unknown",
            error_message="Invalid result type returned from task execution"
        )


# Utility functions for creating announcement data

def create_standard_block_announcement(
    block_num: str,
    livestream_url: str = "https://www.youtube.com/watch?v=SMCRQj9Hbx8",
    entry_fee: float = 0.001,
    commitment_hours: int = 24,
    reveal_hours: int = 48
) -> BlockAnnouncementData:
    """
    Create a standard block announcement with default timing.
    
    Args:
        block_num: Unique identifier for the block
        livestream_url: URL of the livestream players are predicting (defaults to sample URL)
        entry_fee: Entry fee in TAO (default: 0.001)
        commitment_hours: Hours from now until commitment deadline
        reveal_hours: Hours from now until reveal deadline
        
    Returns:
        BlockAnnouncementData instance
    """
    now = datetime.now()
    commitment_deadline = now + timedelta(hours=commitment_hours)
    reveal_deadline = now + timedelta(hours=reveal_hours)
    
    return BlockAnnouncementData(
        block_num=block_num,
        livestream_url=livestream_url,
        entry_fee=entry_fee,
        commitment_deadline=commitment_deadline,
        reveal_deadline=reveal_deadline
    )


def create_custom_block_announcement(
    block_num: str,
    livestream_url: str,
    entry_fee: float,
    commitment_deadline: datetime,
    reveal_deadline: datetime,
    instructions: str = "",
    hashtags: Optional[list[str]] = None
) -> BlockAnnouncementData:
    """
    Create a custom block announcement with specific parameters.
    
    Args:
        block_num: Unique identifier for the block
        livestream_url: URL of the livestream players are predicting
        entry_fee: Entry fee in TAO
        commitment_deadline: When commitments are due
        reveal_deadline: When reveals are due
        instructions: Additional instructions for participants
        hashtags: Custom hashtags (uses defaults if not provided)
        
    Returns:
        BlockAnnouncementData instance
    """
    return BlockAnnouncementData(
        block_num=block_num,
        livestream_url=livestream_url,
        entry_fee=entry_fee,
        commitment_deadline=commitment_deadline,
        reveal_deadline=reveal_deadline,
        instructions=instructions,
        hashtags=hashtags or ["#cliptions", "$TAO"]
    ) 