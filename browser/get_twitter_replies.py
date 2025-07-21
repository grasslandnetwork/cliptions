#!/usr/bin/env python3
"""
Twitter Reply URL Extractor using Browser Use
Follows Browser Use sensitive data guidelines for secure operation
"""

import asyncio
import json
import os
import pathlib
import yaml
import re
from typing import List, Dict
from datetime import datetime
from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from browser_use import Agent, Browser, BrowserContextConfig
from pydantic import BaseModel
# Handle imports for both direct execution and module import
try:
    from .core.cost_tracker import create_cost_tracker_from_config
    from .core.base_task import BaseTwitterTask
except ImportError:
    # When running directly, use absolute imports
    import sys
    import pathlib
    sys.path.insert(0, str(pathlib.Path(__file__).parent.parent))
    from browser.core.cost_tracker import create_cost_tracker_from_config
    from browser.core.base_task import BaseTwitterTask

# Load environment variables
load_dotenv()

# load_llm_config function moved to browser.core.base_task.BaseTwitterTask

class TwitterReply(BaseModel):
    """Model for a Twitter reply"""
    url: str
    author: str
    text_preview: str
    was_spam_flagged: bool = False

class TwitterReplies(BaseModel):
    """Model for collection of Twitter replies"""
    original_tweet_url: str
    total_replies_found: int
    replies: List[TwitterReply]

class TwitterReplyExtractor:
    """Extract reply URLs from Twitter threads using Browser Use"""
    
    def __init__(self, config_file_path: str = None):
        """Initialize the extractor with configuration and cost tracking"""
        # Load configuration using base task utility
        base_task = BaseTwitterTask()
        self.config = base_task.load_llm_config(config_file_path)
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
        script_dir = pathlib.Path(__file__).parent
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
        # Following best practices by not using credentials in sensitive_data
        self.sensitive_data = {}
    
    async def extract_reply_urls(self, tweet_url: str) -> TwitterReplies:
        """
        Extract all reply tweet URLs from a given tweet
        
        Args:
            tweet_url: URL of the original tweet to get replies for
            
        Returns:
            TwitterReplies: Structured data containing all reply URLs
        """
        
        print(f"Extracting reply URLs from: {tweet_url}")
        
        # Generate session ID and start execution tracking
        session_id = self.cost_tracker.generate_session_id("twitter_replies")
        start_time = self.cost_tracker.log_execution_start(session_id, f"Extracting replies from {tweet_url}")
        
        # Define initial actions to run without LLM (faster and cheaper)
        initial_actions = [
            {'open_tab': {'url': tweet_url}},  # Navigate directly to the tweet
        ]
        
        # Define the extraction task (LLM will start from the tweet page)
        task = f"""
        You are on a Twitter/X tweet page. Extract reply information from what you can see.
        
        SIMPLE STEPS:
        1. Look at the current page - you should see the main tweet
        2. Scroll down a few times to see replies below the main tweet
        3. For each reply you can see, extract the information
        4. Look for "Show probable spam" or "Show more replies" buttons and click them if present
        5. Extract any additional replies that become visible
        
        For each reply, extract:
        - Username (like @username)
        - Reply text preview
        - Try to find the reply URL if visible
        
        Return as JSON:
        {{
            "original_tweet_url": "{tweet_url}",
            "total_replies_found": <number>,
            "replies": [
                {{
                    "author": "@username",
                    "text_preview": "reply text...",
                    "url": "reply_url_if_found",
                    "was_spam_flagged": false
                }}
            ]
        }}
        
        Keep it simple - just scroll, look, and extract what you can see.
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
            print("Starting extraction with initial navigation...")
            max_steps = self.config.get('browser_use', {}).get('max_steps', 15)  # Reduced from 50 to 15
            result = await agent.run(max_steps=max_steps)
            
            # Log execution completion and track costs
            self.cost_tracker.log_execution_end(start_time, session_id)
            
            # Handle different result types (simplified original approach)
            if isinstance(result, str):
                try:
                    # Try to parse as JSON if it looks like JSON
                    if result.strip().startswith('{') and result.strip().endswith('}'):
                        result_data = json.loads(result)
                        return TwitterReplies(**result_data)
                except json.JSONDecodeError:
                    pass
                # If not JSON, return empty result
                return TwitterReplies(
                    original_tweet_url=tweet_url,
                    total_replies_found=0,
                    replies=[]
                )
            elif hasattr(result, 'final_result'):
                final_result = result.final_result()
                if final_result:
                    try:
                        if isinstance(final_result, str) and final_result.strip().startswith('{'):
                            result_data = json.loads(final_result)
                            return TwitterReplies(**result_data)
                    except json.JSONDecodeError:
                        pass
            
            # If we get here, return empty result
            return TwitterReplies(
                original_tweet_url=tweet_url,
                total_replies_found=0,
                replies=[]
            )
            
        except Exception as e:
            print(f"Error during extraction: {str(e)}")
            # Still log execution end even on error
            self.cost_tracker.log_execution_end(start_time, session_id)
            # Return empty result on error
            return TwitterReplies(
                original_tweet_url=tweet_url,
                total_replies_found=0,
                replies=[]
            )
        finally:
            # Close the browser context
            if 'browser_context' in locals():
                await browser_context.close()
                print("Browser context closed")
    

    
    async def save_results(self, results: TwitterReplies, output_file: str = "twitter_replies.json"):
        """Save extraction results to a JSON file"""
        try:
            with open(output_file, 'w', encoding='utf-8') as f:
                json.dump(results.dict(), f, indent=2, ensure_ascii=False)
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

async def main():
    """Main function to run the Twitter reply extraction"""
    
    # Target tweet URL
    tweet_url = "https://x.com/cliptions_test/status/1907159517013422578"
    
    print(f"=== Twitter Reply URL Extractor ===")
    print(f"Target tweet: {tweet_url}")
    print("Following Browser Use sensitive data guidelines...")
    print("- Using initial actions to navigate without LLM")
    print("- Using persistent cookies for better performance")
    print("- Vision disabled for security")
    print("- No sensitive data handling needed for public extraction")
    
    # Check if we should run in test mode
    test_mode = os.environ.get('TEST_MODE', 'false').lower() == 'true'
    
    if test_mode:
        print("\n🧪 Running in TEST MODE - just reporting page content")
    
    # Initialize extractor
    extractor = TwitterReplyExtractor()
    
    try:
        if test_mode:
            # Simple test - just navigate and report what we see
            await test_page_content(extractor, tweet_url)
        else:
            # Full extraction
            results = await extractor.extract_reply_urls(tweet_url)
            
            # Display results
            print(f"\n=== EXTRACTION RESULTS ===")
            print(f"Original tweet: {results.original_tweet_url}")
            print(f"Total replies found: {results.total_replies_found}")
            
            if results.replies:
                print("\nReply URLs:")
                for i, reply in enumerate(results.replies, 1):
                    print(f"{i}. {reply.url}")
                    print(f"   Author: {reply.author}")
                    print(f"   Preview: {reply.text_preview}")
                    print()
            else:
                print("No replies found or extraction failed.")
            
            # Save results
            await extractor.save_results(results)
        
    except Exception as e:
        print(f"Error in main execution: {str(e)}")
    
    finally:
        # Clean up browser resources
        await extractor.cleanup()

async def test_page_content(extractor, tweet_url):
    """Simple test function to see what's on the page"""
    
    print(f"🧪 Testing page content for: {tweet_url}")
    
    # Define initial actions
    initial_actions = [
        {'open_tab': {'url': tweet_url}},
    ]
    
    # Simple task to just report what's on the page
    task = f"""
    You are now on a Twitter/X page. Please tell me:
    
    1. What is the main content you can see?
    2. Is this a tweet page showing a single tweet?
    3. Can you see any replies below the main tweet?
    4. What interactive elements do you see (buttons, links, etc.)?
    5. Does the page look normal or like a compose/posting interface?
    
    Just describe what you see in plain text. Don't try to extract specific data.
    DO NOT click on anything - just observe and report.
    """
    
    # Create browser context
    browser_context = await extractor.browser_instance.new_context(config=extractor.browser_config)
    print("Created browser context for testing")
    
    # Create agent for testing
    agent = Agent(
        task=task,
        llm=extractor.llm,
        browser_context=browser_context,
        use_vision=False,
        initial_actions=initial_actions,
    )
    
    try:
        print("Running test agent...")
        result = await agent.run(max_steps=3)  # Just a few steps for testing
        print(f"\n🧪 TEST RESULT:")
        print(f"{result}")
        
    except Exception as e:
        print(f"Error in test: {str(e)}")
    finally:
        if 'browser_context' in locals():
            await browser_context.close()
            print("Test browser context closed")

if __name__ == "__main__":
    asyncio.run(main())
