import asyncio
import os
import json
import tempfile
import shutil
from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from browser_use import Agent, Browser, BrowserConfig

# Load environment variables from .env file
load_dotenv()

async def fetch_round_guesses(round_number: int, target_time_str: str) -> dict:
    """
    Uses Browser Use to fetch RealMir game guesses from Twitter for a specific round.

    Args:
        round_number: The round number to fetch guesses for.
        target_time_str: The target time string (e.g., "20250223_133057EST") associated with the round.

    Returns:
        A dictionary containing the parsed JSON data from Twitter,
        matching the format specified in the task prompt.
        Returns an empty dictionary if an error occurs or no data is found.
    """
    # Configure the browser
    chrome_path = os.environ.get('CHROME_EXECUTABLE_PATH', '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome')
    
    # Create a temporary user data directory for Chrome
    # This is required on macOS for remote debugging to work
    user_data_dir = os.path.join(tempfile.gettempdir(), "browser-use-chrome-data")
    os.makedirs(user_data_dir, exist_ok=True)
    print(f"Using Chrome user data directory: {user_data_dir}")
    
    # Configure browser to use specific Chrome path with user data directory
    browser_instance = Browser(
        config=BrowserConfig(
            chrome_instance_path=chrome_path,
            headless=False,  # Ensure browser is visible for debugging
            extra_chromium_args=[
                f"--user-data-dir={user_data_dir}",
                "--remote-debugging-port=9222",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-extensions"
            ]
        )
    )

    sensitive_data = {
        'x_name': os.environ.get('TWITTER_NAME'),
        'x_password': os.environ.get('TWITTER_PASSWORD')
    }

    if not sensitive_data['x_name'] or not sensitive_data['x_password']:
        print("Error: TWITTER_NAME and TWITTER_PASSWORD environment variables must be set.")
        return {"error": "Twitter credentials not set."}

    llm = ChatOpenAI(model="gpt-4o")

    # Actual Twitter task for production
    task = f"""
        1. Go to https://twitter.com
        2. Log in using sensitive_data['x_name'] and sensitive_data['x_password']
        3. Search for "RealMir Round {round_number} Target: {target_time_str}"
        4. Extract all user guesses for this round from any tweets in the search results
        5. For each guess, collect:
           - Twitter username
           - Their guess time
           - The original tweet text
        6. Format as JSON with the structure {{
            "round_number": {round_number},
            "target_time": "{target_time_str}",
            "guesses": [
                {{
                    "username": "username_here",
                    "guess_time": "guess_time_here",
                    "tweet_text": "full_tweet_text_here"
                }},
                // ...more guesses
            ]
        }}
    """

    # For testing purposes, provide a test mode option
    if os.environ.get('TWITTER_UTILS_TEST_MODE', 'false').lower() == 'true':
        print("Running in test mode with simplified task...")
        task = "Go to example.com and tell me the main heading on the page."
    else:
        print(f"Running Twitter task for Round {round_number}, target time: {target_time_str}...")

    agent = Agent(
        task=task,
        llm=llm,
        use_vision=True,
        sensitive_data=sensitive_data,
        browser=browser_instance
    )

    raw_result = None
    try:
        max_steps = int(os.environ.get('TWITTER_UTILS_MAX_STEPS', '25'))
        print(f"Running agent with max_steps={max_steps}...")
        raw_result = await agent.run(max_steps=max_steps)
        print(f"Raw result from Browser Use agent: {raw_result}")

        if isinstance(raw_result, str):
            try:
                # Try to parse as JSON if it looks like JSON
                if raw_result.strip().startswith('{') and raw_result.strip().endswith('}'):
                    return json.loads(raw_result)
                # Otherwise return as is
                return {"output": raw_result}
            except json.JSONDecodeError as e:
                print(f"Error decoding JSON from Browser Use agent: {e}")
                return {"error": "Failed to decode JSON output", "raw_output": raw_result}
        elif raw_result is None:
             print("Agent run returned None. Max steps might have been reached or task failed silently.")
             return {"error": "Agent run returned None", "raw_output": None}
        else:
            print(f"Raw result from agent.run() is: {raw_result} (type: {type(raw_result)})")
            # Try to get the final extracted content from the agent history
            if hasattr(raw_result, 'all_results') and raw_result.all_results:
                # Get the last successful result
                for result in reversed(raw_result.all_results):
                    if result.is_done and result.success and result.extracted_content:
                        try:
                            if isinstance(result.extracted_content, str) and result.extracted_content.strip().startswith('{'):
                                return json.loads(result.extracted_content)
                        except json.JSONDecodeError:
                            pass
                        return {"output": result.extracted_content}
            
            # Fallback to returning the string representation
            return {"output": str(raw_result)}

    except Exception as e:
        print(f"An error occurred while running the Browser Use agent: {e}")
        return {"error": str(e)}
    finally:
        # Close the browser instance properly
        await browser_instance.close()
        
        # Optionally clean up the temp directory when done
        if os.environ.get('TWITTER_UTILS_CLEANUP_TEMP', 'true').lower() == 'true':
            try:
                print(f"Cleaning up temporary Chrome data directory: {user_data_dir}")
                shutil.rmtree(user_data_dir, ignore_errors=True)
            except Exception as e:
                print(f"Error cleaning up temporary directory: {e}")

if __name__ == '__main__':
    import argparse
    
    parser = argparse.ArgumentParser(description='Fetch RealMir game guesses from Twitter')
    parser.add_argument('--round', type=int, default=1, help='Round number to fetch guesses for')
    parser.add_argument('--target-time', type=str, default='20250523_133057EST', 
                        help='Target time string (e.g., "20250523_133057EST")')
    parser.add_argument('--test-mode', action='store_true', 
                        help='Run in test mode with example.com instead of Twitter')
    parser.add_argument('--debug', action='store_true', 
                        help='Enable debug logging')
    
    args = parser.parse_args()
    
    async def main():
        # Setup environment based on command line args
        if args.test_mode:
            os.environ['TWITTER_UTILS_TEST_MODE'] = 'true'
            print("Running in TEST MODE with example.com...")
        else:
            os.environ['TWITTER_UTILS_TEST_MODE'] = 'false'
            print(f"Running Twitter task for Round {args.round}, target: {args.target_time}")
            
            # Check Twitter credentials
            if not os.environ.get('TWITTER_NAME') or not os.environ.get('TWITTER_PASSWORD'):
                print("ERROR: TWITTER_NAME and TWITTER_PASSWORD environment variables must be set.")
                print("Set them using:")
                print("  export TWITTER_NAME='your_twitter_username'")
                print("  export TWITTER_PASSWORD='your_twitter_password'")
                return
        
        # Set log level
        if args.debug:
            os.environ['BROWSER_USE_LOG_LEVEL'] = 'debug'
        else:
            os.environ['BROWSER_USE_LOG_LEVEL'] = 'info'
        
        # Run the main function
        result = await fetch_round_guesses(args.round, args.target_time)
        
        print("\n--- Result ---")
        # Print as JSON if it's a dict, otherwise print directly
        if isinstance(result, dict):
            print(json.dumps(result, indent=2))
        else:
            print(result)
        print("--- End Result ---")

    asyncio.run(main()) 