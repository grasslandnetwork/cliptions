"""
Base implementation for Cliptions Twitter automation tasks.

This module provides the concrete base class that implements common functionality
shared across all Twitter automation modules.
"""

import asyncio
import json
import os
import pathlib
import yaml
import re
from datetime import datetime
from typing import Any, Dict, Optional
from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from browser_use import Agent, Browser, BrowserContextConfig
from pydantic import BaseModel

from .interfaces import TwitterTask, TwitterTaskError
from .cost_tracker import create_cost_tracker_from_config

# Load environment variables
load_dotenv()


class BaseTwitterTask(TwitterTask):
    """
    Base implementation of TwitterTask providing shared functionality.
    
    This class implements common features like:
    - Configuration loading and validation
    - Browser context management
    - Cost tracking integration
    - Agent setup and lifecycle management
    - Error handling and cleanup
    """
    
    def __init__(self, config: Optional[Dict[str, Any]] = None, config_file_path: Optional[str] = None):
        """
        Initialize the base Twitter task.
        
        Args:
            config: Configuration dictionary (optional)
            config_file_path: Path to configuration file (optional, defaults to config/llm.yaml)
        """
        # Load configuration
        if config is None:
            config = self.load_llm_config(config_file_path)
        
        super().__init__(config)
        
        # Initialize cost tracking
        self.cost_tracker = create_cost_tracker_from_config(self.config)
        if self.cost_tracker.enabled:
            self.cost_tracker.validate_spending_limit()
            self.cost_tracker.sync_latest_data()
        
        # Initialize LLM
        llm_config = self.config['openai']
        self.llm = ChatOpenAI(
            model=llm_config.get('model', 'gpt-4o'),
            temperature=llm_config.get('temperature', 0.1),
            max_tokens=llm_config.get('max_tokens', 4000),
            openai_api_key=llm_config.get('api_key')
        )
        
        # Setup browser configuration
        self._setup_browser_config()
        
        # Session tracking
        self._session_id: Optional[str] = None
        self._start_time: Optional[datetime] = None
        
    def load_llm_config(self, config_file_path: Optional[str] = None) -> Dict[str, Any]:
        """
        Load LLM configuration from config/llm.yaml with environment variable substitution.
        
        Args:
            config_file_path: Optional path to config file
            
        Returns:
            Configuration dictionary
        """
        if config_file_path is None:
            # Default to config/llm.yaml in project root
            script_dir = pathlib.Path(__file__).parent.parent.parent  # Go up from browser-use/core/ to project root
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
    
    def _setup_browser_config(self) -> None:
        """Setup browser data directory and context configuration."""
        # Create browser data directory
        script_dir = pathlib.Path(__file__).parent.parent  # browser-use/ directory
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
    
    async def setup_agent(self, task: str, initial_actions: Optional[list] = None, **kwargs) -> Agent:
        """
        Configure and return the browser-use agent for this task.
        
        Args:
            task: The task description for the LLM
            initial_actions: Optional list of initial actions to perform
            **kwargs: Additional agent configuration
            
        Returns:
            Configured browser-use agent
        """
        # Setup sensitive data for Twitter credentials
        sensitive_data = {
            'x_name': os.environ.get('TWITTER_NAME'),
            'x_password': os.environ.get('TWITTER_PASSWORD')
        }
        
        if not sensitive_data['x_name'] or not sensitive_data['x_password']:
            raise TwitterTaskError("TWITTER_NAME and TWITTER_PASSWORD environment variables must be set.")
        
        # Create browser context if not already created
        if self._browser_context is None:
            self._browser_context = await self.browser_instance.new_context(config=self.browser_config)
            print("Created persistent browser context with cookies support")
        
        # Create and configure agent
        self._agent = Agent(
            task=task,
            llm=self.llm,
            browser_context=self._browser_context,
            sensitive_data=sensitive_data,
            use_vision=kwargs.get('use_vision', False),
            initial_actions=initial_actions or []
        )
        
        return self._agent
    
    async def _start_session(self, task_description: str = "") -> None:
        """Start a new execution session with cost tracking."""
        self._session_id = self.cost_tracker.generate_session_id(self.__class__.__name__.lower())
        self._start_time = self.cost_tracker.log_execution_start(self._session_id, task_description)
    
    async def _end_session(self) -> None:
        """End the current execution session."""
        if self._start_time and self._session_id:
            self.cost_tracker.log_execution_end(self._start_time, self._session_id)
    
    async def cleanup(self) -> None:
        """Clean up browser resources and close connections."""
        try:
            await self._end_session()
        except Exception as e:
            print(f"Warning: Error ending session: {e}")
        
        if self._browser_context:
            try:
                await self._browser_context.close()
                print("Browser context closed")
            except Exception as e:
                print(f"Warning: Error closing browser context: {e}")
        
        if hasattr(self, 'browser_instance'):
            try:
                await self.browser_instance.close()
                print("Browser instance closed")
            except Exception as e:
                print(f"Warning: Error closing browser instance: {e}")
    
    def validate_output(self, result: Any) -> BaseModel:
        """
        Base implementation of output validation.
        
        Subclasses should override this method for specific validation logic.
        
        Args:
            result: Raw result from agent execution
            
        Returns:
            Validated BaseModel instance
        """
        if isinstance(result, str):
            try:
                # Try to parse as JSON if it looks like JSON
                if result.strip().startswith('{') and result.strip().endswith('}'):
                    data = json.loads(result)
                    # Return as generic dict for now - subclasses should override
                    return BaseModel(**data)
            except json.JSONDecodeError as e:
                raise TwitterTaskError(f"Failed to parse JSON result: {e}")
        
        # If result is already a BaseModel, return it
        if isinstance(result, BaseModel):
            return result
        
        # For other types, try to wrap in a generic response
        return BaseModel()
    
    async def execute(self, **kwargs) -> BaseModel:
        """
        Base execute method - subclasses should override this.
        
        This provides the basic execution framework with error handling and cleanup.
        """
        try:
            await self._start_session(f"{self.__class__.__name__} execution")
            
            # Subclasses should override this method
            result = await self._execute_task(**kwargs)
            
            return self.validate_output(result)
            
        except Exception as e:
            print(f"Error during {self.__class__.__name__} execution: {str(e)}")
            raise TwitterTaskError(f"Task execution failed: {e}")
        finally:
            await self.cleanup()
    
    async def _execute_task(self, **kwargs) -> Any:
        """
        Override this method in subclasses to implement specific task logic.
        
        Args:
            **kwargs: Task-specific parameters
            
        Returns:
            Task-specific results
        """
        raise NotImplementedError("Subclasses must implement _execute_task()")
    
    def get_max_steps(self) -> int:
        """Get the maximum steps configuration for browser-use agent."""
        return self.config.get('browser_use', {}).get('max_steps', 50) 