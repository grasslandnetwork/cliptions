"""
Commitment Collection Module for RealMir Validators

This module is responsible for extracting all miner commitment submissions
from the replies to a validator's round announcement tweet.
"""

import asyncio
import json
import os
import pathlib
import yaml
import re
import logging
from datetime import datetime
from typing import List, Optional
from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from browser_use import Agent, Browser, BrowserContextConfig
from pydantic import BaseModel, Field
from ..core.cost_tracker import create_cost_tracker_from_config

# Load environment variables
load_dotenv()

def load_llm_config(config_file_path: str = None) -> dict:
    """Load LLM configuration from config/llm.yaml with environment variable substitution"""
    if config_file_path is None:
        # Default to config/llm.yaml in project root (two levels up from browser/validator/)
        script_dir = pathlib.Path(__file__).parent.parent.parent  
        config_file_path = script_dir / "config" / "llm.yaml"
    else:
        config_file_path = pathlib.Path(config_file_path)
    
    if not config_file_path.exists():
        print(f"Warning: LLM config file not found: {config_file_path}")
        # Return default config
        return {
            'openai': {
                'model': 'gpt-4o',
                'temperature': 0.1,
                'max_tokens': 4000,
                'daily_spending_limit_usd': 5.00
            },
            'cost_tracking': {
                'enabled': True
            },
            'browser_use': {
                'max_steps': 50
            }
        }
    
    with open(config_file_path, 'r') as f:
        config_content = f.read()
    
    # Substitute environment variables
    def replace_env_var(match):
        env_var = match.group(1)
        value = os.environ.get(env_var)
        if value is None:
            print(f"Warning: Environment variable {env_var} is not set")
            return match.group(0)  # Return original if env var not found
        return value
    
    config_content = re.sub(r'\$\{([^}]+)\}', replace_env_var, config_content)
    
    return yaml.safe_load(config_content)


class CommitmentData(BaseModel):
    """
    Represents a single parsed commitment from a tweet reply.
    """
    username: str = Field(..., description="The Twitter username of the miner who submitted the commitment.")
    commitment_hash: str = Field(..., description="The SHA-256 commitment hash.")
    wallet_address: str = Field(..., description="The miner's wallet address for payouts.")
    tweet_url: str = Field(..., description="The URL of the reply tweet containing the commitment.")
    timestamp: datetime = Field(..., description="The timestamp when the reply was posted.")


class CommitmentCollectionResult(BaseModel):
    """
    The result of the commitment collection task, containing all found commitments.
    """
    success: bool = Field(..., description="Indicates whether the extraction was successful.")
    commitments: List[CommitmentData] = Field(default_factory=list, description="A list of all collected commitments.")
    announcement_url: str = Field(..., description="The URL of the announcement tweet that was processed.")
    total_commitments_found: int = Field(default=0, description="Total number of commitments extracted.")
    error_message: Optional[str] = Field(None, description="An error message if the task failed.")


class CollectCommitmentsTask:
    """
    A task to collect commitment submissions from a Twitter thread using Browser Use.
    
    This task extracts replies to a round announcement tweet and parses them to find
    miner commitments in the format:
    - Commit: <hash>
    - Wallet: <address>
    """
    
    def __init__(self, config_file_path: str = None):
        """Initialize the commitment collector with configuration and cost tracking"""
        # Load configuration
        self.config = load_llm_config(config_file_path)
        print(f"Loaded LLM config: {self.config}")
        
        # Initialize cost tracker from config
        self.cost_tracker = create_cost_tracker_from_config(self.config)
        
        # Validate spending limits and sync data if cost tracking is enabled
        if self.cost_tracker.enabled:
            self.cost_tracker.validate_spending_limit()
            self.cost_tracker.sync_latest_data()
        
        # Initialize LLM with config settings
        llm_config = self.config['openai']
        self.llm = ChatOpenAI(
            model=llm_config.get('model', 'gpt-4o'),
            temperature=llm_config.get('temperature', 0.1),
            max_tokens=llm_config.get('max_tokens', 4000),
            openai_api_key=llm_config.get('api_key')
        )
        
        # Setup browser data directory and cookies
        script_dir = pathlib.Path(__file__).parent.parent  # Go up to browser/ directory
        browser_data_dir = script_dir / "browser_data"
        browser_data_dir.mkdir(exist_ok=True)
        self.cookies_file = str(browser_data_dir / 'twitter_cookies.json')
        print(f"Using browser data directory: {browser_data_dir}")
        print(f"Cookies will be saved to: {self.cookies_file}")
        
        # Configure browser context with persistence
        self.browser_config = BrowserContextConfig(
            cookies_file=self.cookies_file
        )
        
        # Initialize browser for persistent sessions
        self.browser_instance = Browser()
        
        # No sensitive data needed for this public extraction task
        self.sensitive_data = {}
        
        self.logger = logging.getLogger(__name__)

    async def execute(self, announcement_url: str) -> CommitmentCollectionResult:
        """
        Execute the commitment collection task by extracting and parsing replies.

        Args:
            announcement_url: The URL of the round announcement tweet.

        Returns:
            A CommitmentCollectionResult containing the collected commitment data.
        """
        self.logger.info(f"Starting commitment collection for announcement: {announcement_url}")
        print(f"Collecting commitments from: {announcement_url}")
        
        # Generate session ID and start execution tracking
        session_id = self.cost_tracker.generate_session_id("commitment_collection")
        start_time = self.cost_tracker.log_execution_start(session_id, f"Collecting commitments from {announcement_url}")
        
        # Define initial actions to run without LLM (faster and cheaper)
        initial_actions = [
            {'open_tab': {'url': announcement_url}},  # Navigate directly to the announcement tweet
        ]
        
        # Define the commitment extraction task (simplified to prevent loops)
        task = f"""
        You are on a Twitter/X page showing an announcement tweet. Your ONLY job is to find and extract commitment replies.

        STRICT RULES - FOLLOW EXACTLY:
        1. NEVER click on usernames, profile pictures, or tweet links
        2. NEVER click "Reply", "Retweet", "Like", or any interaction buttons  
        3. ONLY use scroll_down to see more content
        4. ONLY click "Show more replies" or "Show probable spam" buttons if you see them
        5. When you have scrolled enough and seen all replies, immediately return the JSON result

        WHAT TO LOOK FOR:
        Find replies that contain BOTH lines:
        - "Commit: [some hash]"
        - "Wallet: [some address]"

        PROCESS:
        1. Look at what's currently visible
        2. Scroll down 3-5 times to load more replies
        3. If you see "Show more replies" or "Show probable spam", click it once
        4. Scroll down 2-3 more times
        5. Extract all commitment data you can see
        6. Return the JSON result immediately

        RETURN FORMAT:
        {{
            "announcement_url": "{announcement_url}",
            "success": true,
            "total_commitments_found": <number>,
            "commitments": [
                {{
                    "username": "@username",
                    "commitment_hash": "the_hash_value",
                    "wallet_address": "the_wallet_address",
                    "tweet_url": "",
                    "timestamp": "2024-01-01T00:00:00Z",
                    "was_spam_flagged": false
                }}
            ]
        }}

        IMPORTANT: Do this quickly and efficiently. Don't overthink it. Just scroll, find commitments, return JSON.
        """
        
        # Create browser context with persistent cookies
        browser_context = await self.browser_instance.new_context(config=self.browser_config)
        print("Created persistent browser context with cookies support")
        
        # Create agent with security configurations and initial actions
        agent = Agent(
            task=task,
            llm=self.llm,
            browser_context=browser_context,
            sensitive_data=self.sensitive_data,
            use_vision=False,  # Disable vision as recommended for sensitive data handling
            initial_actions=initial_actions,  # Navigate to URL without LLM
        )
        
        try:
            # Run the extraction
            print("Starting commitment extraction with initial navigation...")
            max_steps = self.config.get('browser_use', {}).get('max_steps', 50)
            result = await agent.run(max_steps=max_steps)
            
            # Log execution completion and track costs
            self.cost_tracker.log_execution_end(start_time, session_id)
            
            # Parse the extraction result
            return await self._parse_extraction_result(result, announcement_url)
            
        except Exception as e:
            error_msg = f"Error during commitment extraction: {str(e)}"
            self.logger.error(error_msg)
            print(error_msg)
            
            # Still log execution end even on error
            self.cost_tracker.log_execution_end(start_time, session_id)
            
            # Return error result
            return CommitmentCollectionResult(
                success=False,
                commitments=[],
                announcement_url=announcement_url,
                total_commitments_found=0,
                error_message=error_msg
            )
        finally:
            # Close the browser context
            if 'browser_context' in locals():
                await browser_context.close()
                print("Browser context closed")

    async def _parse_extraction_result(self, result, announcement_url: str) -> CommitmentCollectionResult:
        """Parse the browser extraction result into structured commitment data"""
        
        # Handle different result types (adapted from get_twitter_replies.py pattern)
        result_data = None
        
        if isinstance(result, str):
            try:
                # Try to parse as JSON if it looks like JSON
                if result.strip().startswith('{') and result.strip().endswith('}'):
                    result_data = json.loads(result)
            except json.JSONDecodeError:
                pass
                
        elif hasattr(result, 'final_result'):
            final_result = result.final_result()
            if final_result:
                try:
                    if isinstance(final_result, str) and final_result.strip().startswith('{'):
                        result_data = json.loads(final_result)
                except json.JSONDecodeError:
                    pass
        
        # If we successfully parsed JSON data, process it
        if result_data and isinstance(result_data, dict):
            try:
                commitments = []
                
                # Parse commitment data
                if 'commitments' in result_data and isinstance(result_data['commitments'], list):
                    for commitment_raw in result_data['commitments']:
                        if isinstance(commitment_raw, dict):
                            # Validate required fields
                            if all(field in commitment_raw for field in ['username', 'commitment_hash', 'wallet_address']):
                                # Parse timestamp - use current time if not provided or invalid
                                timestamp = datetime.now()
                                if 'timestamp' in commitment_raw:
                                    try:
                                        timestamp = datetime.fromisoformat(commitment_raw['timestamp'].replace('Z', '+00:00'))
                                    except (ValueError, AttributeError):
                                        pass  # Use current time
                                
                                commitment = CommitmentData(
                                    username=commitment_raw['username'],
                                    commitment_hash=commitment_raw['commitment_hash'],
                                    wallet_address=commitment_raw['wallet_address'],
                                    tweet_url=commitment_raw.get('tweet_url', ''),
                                    timestamp=timestamp
                                )
                                commitments.append(commitment)
                
                return CommitmentCollectionResult(
                    success=result_data.get('success', True),
                    commitments=commitments,
                    announcement_url=announcement_url,
                    total_commitments_found=len(commitments),
                    error_message=result_data.get('error_message')
                )
                
            except Exception as e:
                error_msg = f"Error parsing commitment data: {str(e)}"
                print(error_msg)
                return CommitmentCollectionResult(
                    success=False,
                    commitments=[],
                    announcement_url=announcement_url,
                    total_commitments_found=0,
                    error_message=error_msg
                )
        
        # If we get here, parsing failed - return empty result (like get_twitter_replies.py does)
        print("Failed to parse extraction result, returning empty result")
        return CommitmentCollectionResult(
            success=True,  # Changed to True to match get_twitter_replies.py pattern
            commitments=[],
            announcement_url=announcement_url,
            total_commitments_found=0,
            error_message=None  # No error message for empty results
        )

    async def save_results(self, results: CommitmentCollectionResult, output_file: str = "commitments.json"):
        """Save commitment collection results to a JSON file"""
        try:
            with open(output_file, 'w', encoding='utf-8') as f:
                json.dump(results.model_dump(), f, indent=2, ensure_ascii=False, default=str)
            print(f"Results saved to {output_file}")
        except Exception as e:
            print(f"Error saving results: {str(e)}")

    async def cleanup(self):
        """Clean up browser resources"""
        try:
            await self.browser_instance.close()
            print("Browser closed")
        except Exception as e:
            print(f"Error during cleanup: {str(e)}") 