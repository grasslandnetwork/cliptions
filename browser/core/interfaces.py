"""
Core interfaces for RealMir Twitter automation system.

This module defines the abstract base classes that all Twitter automation modules
must implement, ensuring consistency and testability across the system.
"""

from abc import ABC, abstractmethod
from typing import Any, Dict, Optional, List
from pydantic import BaseModel
from browser_use import Agent
# Browser_use may not expose BrowserContext in newer versions, so use Any for context
after_import_browser_context = False
try:
    from browser_use import BrowserContext  # type: ignore
    after_import_browser_context = True
except ImportError:
    from typing import Any as BrowserContext  # type: ignore


class TwitterTask(ABC):
    """
    Abstract base class defining the contract for any automated Twitter action.
    
    All Twitter automation modules (extraction, posting, monitoring) must inherit
    from this class and implement its abstract methods.
    """
    
    def __init__(self, config: Dict[str, Any]):
        """
        Initialize the Twitter task with configuration.
        
        Args:
            config: Configuration dictionary containing LLM settings, browser options, etc.
        """
        self.config = config
        self._agent: Optional[Agent] = None
        self._browser_context: Optional[BrowserContext] = None
    
    @abstractmethod
    async def execute(self, **kwargs) -> BaseModel:
        """
        Execute the Twitter task and return structured results.
        
        Args:
            **kwargs: Task-specific parameters
            
        Returns:
            BaseModel: Pydantic model containing the results of the operation
            
        Raises:
            TwitterTaskError: If the task fails to execute
        """
        pass
    
    @abstractmethod
    async def setup_agent(self, **kwargs) -> Agent:
        """
        Configure and return the browser-use agent for this task.
        
        Args:
            **kwargs: Agent-specific configuration parameters
            
        Returns:
            Agent: Configured browser-use agent ready for execution
        """
        pass
    
    @abstractmethod
    def validate_output(self, result: Any) -> BaseModel:
        """
        Validate and structure the output from the agent execution.
        
        Args:
            result: Raw result from agent.run()
            
        Returns:
            BaseModel: Validated and structured result
            
        Raises:
            ValidationError: If the result cannot be validated
        """
        pass
    
    async def cleanup(self) -> None:
        """
        Clean up browser resources and close connections.
        
        This method should be called after task completion to prevent resource leaks.
        """
        if self._browser_context:
            await self._browser_context.close()
        if self._agent:
            # Additional cleanup if needed
            pass


class TwitterExtractionInterface(TwitterTask):
    """
    Interface for Twitter data extraction tasks.
    
    Specialized for modules that collect information from Twitter (replies, tweets, etc.).
    Examples: CommitmentCollector, RevealCollector, ReplyExtractor
    """
    
    @abstractmethod
    async def extract_from_url(self, tweet_url: str, **kwargs) -> BaseModel:
        """
        Extract data from a specific Twitter URL.
        
        Args:
            tweet_url: URL of the tweet to extract data from
            **kwargs: Additional extraction parameters
            
        Returns:
            BaseModel: Structured extraction results
        """
        pass
    
    @abstractmethod
    def parse_extracted_content(self, raw_content: str) -> BaseModel:
        """
        Parse raw extracted content into structured data.
        
        Args:
            raw_content: Raw text or HTML content extracted from Twitter
            
        Returns:
            BaseModel: Parsed and structured content
        """
        pass


class TwitterPostingInterface(TwitterTask):
    """
    Interface for Twitter content creation tasks.
    
    Specialized for modules that create tweets, replies, or other content on Twitter.
    Examples: RoundAnnouncer, CommitmentSubmitter, RevealSubmitter, ResultsPublisher
    """
    
    @abstractmethod
    async def post_content(self, content: str, **kwargs) -> BaseModel:
        """
        Post content to Twitter (tweet, reply, etc.).
        
        Args:
            content: The text content to post
            **kwargs: Additional posting parameters (reply_to_url, image_path, etc.)
            
        Returns:
            BaseModel: Posting result including URL, timestamp, etc.
        """
        pass
    
    @abstractmethod
    def format_content(self, data: Dict[str, Any]) -> str:
        """
        Format structured data into Twitter-ready content.
        
        Args:
            data: Structured data to format
            
        Returns:
            str: Formatted content ready for posting
        """
        pass


class TwitterTaskError(Exception):
    """Base exception for Twitter task errors."""
    pass


class ExtractionError(TwitterTaskError):
    """Exception raised when data extraction fails."""
    pass


class PostingError(TwitterTaskError):
    """Exception raised when content posting fails."""
    pass


class ValidationError(TwitterTaskError):
    """Exception raised when output validation fails."""
    pass


class DataAccessInterface(ABC):
    """
    Abstract interface for data access operations.
    
    This interface defines the contract for all data storage implementations,
    allowing the system to be decoupled from specific storage mechanisms.
    """
    
    @abstractmethod
    async def get_round(self, round_id: str) -> Optional[Dict[str, Any]]:
        """
        Retrieve a specific round by ID.
        
        Args:
            round_id: The unique identifier of the round
            
        Returns:
            Round data dictionary or None if not found
        """
        pass
    
    @abstractmethod
    async def save_round(self, round_id: str, round_data: Dict[str, Any]) -> bool:
        """
        Save or update a round's data.
        
        Args:
            round_id: The unique identifier of the round
            round_data: Complete round data dictionary
            
        Returns:
            True if save was successful, False otherwise
        """
        pass
    
    @abstractmethod
    async def list_rounds(self) -> List[str]:
        """
        List all available round IDs.
        
        Returns:
            List of round identifiers
        """
        pass
    
    @abstractmethod
    async def save_commitments(self, round_id: str, commitments: List[Dict[str, Any]]) -> bool:
        """
        Save collected commitments for a round.
        
        Args:
            round_id: The round identifier
            commitments: List of commitment data dictionaries
            
        Returns:
            True if save was successful, False otherwise
        """
        pass
    
    @abstractmethod
    async def get_commitments(self, round_id: str) -> List[Dict[str, Any]]:
        """
        Get all commitments for a specific round.
        
        Args:
            round_id: The round identifier
            
        Returns:
            List of commitment data dictionaries
        """
        pass
    
    @abstractmethod
    async def update_participant(self, round_id: str, username: str, updates: Dict[str, Any]) -> bool:
        """
        Update participant data within a round.
        
        Args:
            round_id: The round identifier
            username: The participant's username
            updates: Dictionary of fields to update
            
        Returns:
            True if update was successful, False otherwise
        """
        pass
    
    @abstractmethod
    async def get_all_rounds(self) -> Dict[str, Any]:
        """
        Get all rounds data.
        
        Returns:
            Dictionary containing all rounds data
        """
        pass 