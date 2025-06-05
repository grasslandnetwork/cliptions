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
from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from browser_use import Agent, Browser, BrowserContextConfig
from pydantic import BaseModel

# Load environment variables
load_dotenv()

def load_llm_config(config_file_path: str = None) -> dict:
    """Load LLM configuration from config/llm.yaml with environment variable substitution"""
    if config_file_path is None:
        # Default to config/llm.yaml in project root
        script_dir = pathlib.Path(__file__).parent.parent  # Go up from browser-use/ to project root
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
                'max_tokens': 4000
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

class TwitterReply(BaseModel):
    """Model for a Twitter reply"""
    url: str
    author: str
    text_preview: str

class TwitterReplies(BaseModel):
    """Model for collection of Twitter replies"""
    original_tweet_url: str
    total_replies_found: int
    replies: List[TwitterReply]

class TwitterReplyExtractor:
    """Extract reply URLs from Twitter threads using Browser Use"""
    
    def __init__(self, config_file_path: str = None):
        """Initialize the extractor with configuration"""
        # Load configuration
        self.config = load_llm_config(config_file_path)
        print(f"Loaded LLM config: {self.config}")
        
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
        
        # Define initial actions to run without LLM (faster and cheaper)
        initial_actions = [
            {'open_tab': {'url': tweet_url}},  # Navigate directly to the tweet
        ]
        
        # Define the extraction task (LLM will start from the tweet page)
        task = f"""
        You are now on the Twitter/X tweet page. Your task is to extract ALL reply tweet URLs from this thread.
        
        CRITICAL INSTRUCTIONS - READ CAREFULLY:
        
        1. DO NOT CLICK ON ANYTHING AT ALL - No buttons, no links, no interactive elements
        2. The page should already show the main tweet - just observe what's there
        3. Use ONLY the scroll_down action to see more content
        4. Look for reply tweets that are already visible or become visible after scrolling
        5. Extract information from what you can see on the page, don't interact with elements
        
        What you're looking for:
        - Reply tweets that appear below the main tweet after scrolling
        - Each reply will have a username (like @username) 
        - Each reply will have its own tweet URL/status link
        - Reply text content
        
        Steps to follow:
        1. Look at the current page content for any visible replies
        2. Use scroll_down action to see more content  
        3. Continue scrolling to load more replies
        4. Extract information from visible replies
        5. DO NOT click on "Reply", "Comment", "Post" or any buttons
        
        For each reply tweet you can see, extract:
        - The direct URL (if visible in page elements)
        - The username/handle (@username)
        - Preview of the reply text
        
        Return results as JSON:
        {{
            "original_tweet_url": "{tweet_url}",
            "total_replies_found": <number>,
            "replies": [
                {{
                    "url": "extracted_or_constructed_url",
                    "author": "@username", 
                    "text_preview": "reply text..."
                }}
            ]
        }}
        
        REMEMBER: ONLY use scroll_down action. NO CLICKING on anything!
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
            max_steps = self.config.get('browser_use', {}).get('max_steps', 50)
            result = await agent.run(max_steps=max_steps)
            
            # Handle different result types
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
    tweet_url = "https://x.com/realmir_testnet/status/1907159517013422578"
    
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
