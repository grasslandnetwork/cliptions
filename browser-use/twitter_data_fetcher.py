import asyncio
import os
import json
import pathlib
import yaml
from datetime import datetime
from dotenv import load_dotenv
from langchain_openai import ChatOpenAI
from browser_use import Agent, Browser, BrowserContextConfig
from openai_usage_tracker import OpenAIUsageTracker

# Load environment variables from .env file
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
        raise FileNotFoundError(f"LLM config file not found: {config_file_path}")
    
    with open(config_file_path, 'r') as f:
        config_content = f.read()
    
    # Substitute environment variables
    import re
    def replace_env_var(match):
        env_var = match.group(1)
        value = os.environ.get(env_var)
        if value is None:
            raise ValueError(f"Environment variable {env_var} is not set")
        return value
    
    config_content = re.sub(r'\$\{([^}]+)\}', replace_env_var, config_content)
    
    return yaml.safe_load(config_content)

def check_daily_spending_limit(usage_tracker: OpenAIUsageTracker, daily_limit: float, project_ids: list = None) -> dict:
    """
    Check if current daily spending is under the limit.
    Daily reset occurs at 00:00 UTC.
    
    Args:
        usage_tracker: OpenAI usage tracker instance
        daily_limit: Daily spending limit in USD
        project_ids: Optional list of project IDs to filter by
        
    Returns:
        Dict with can_proceed, current_spending, remaining_budget
    """
    today = datetime.utcnow().date()
    
    # If project filtering is requested, fetch project-specific data
    if project_ids:
        # Fetch fresh project-specific cost data
        usage_tracker.fetch_costs_data(days_back=1, project_ids=project_ids)
    
    costs_data = usage_tracker.get_daily_costs(today)
    
    if costs_data is None:
        # No cost data available, assume $0 spent
        current_spending = 0.00
    else:
        current_spending = costs_data['total_cost_usd']
    
    remaining_budget = daily_limit - current_spending
    can_proceed = remaining_budget > 0
    
    return {
        'can_proceed': can_proceed,
        'current_spending': current_spending,
        'remaining_budget': remaining_budget,
        'daily_limit': daily_limit,
        'date': today.isoformat(),
        'project_ids': project_ids
    }

def track_execution_costs(usage_tracker: OpenAIUsageTracker, session_id: str) -> dict:
    """
    Track costs for a browser-use execution session.
    
    Args:
        usage_tracker: OpenAI usage tracker instance
        session_id: Unique identifier for this session
        
    Returns:
        Dict with session tracking information
    """
    start_time = datetime.now()
    
    # Sync latest usage data
    sync_result = usage_tracker.sync_daily_data()
    
    end_time = datetime.now()
    
    return {
        'session_id': session_id,
        'start_time': start_time.isoformat(),
        'end_time': end_time.isoformat(),
        'sync_result': sync_result
    }

async def fetch_round_guesses(round_number: int, target_time_str: str = None, config_file_path: str = None, round_reveal_url: str = None) -> dict:
    """
    Uses Browser Use to fetch RealMir game guesses from Twitter for a specific round.

    Args:
        round_number: The round number to fetch guesses for.
        target_time_str: The target time string (e.g., "20250223_133057EST") associated with the round.
        config_file_path: Path to LLM config file (defaults to config/llm.yaml)
        round_reveal_url: Optional URL to the specific tweet where round reveals/guesses are posted.

    Returns:
        A dictionary containing the parsed JSON data from Twitter,
        matching the format specified in the task prompt.
        Returns an empty dictionary if an error occurs or no data is found.
    """
    # Load configuration
    try:
        config = load_llm_config(config_file_path)
        print(f"Loaded LLM config: {config}")
    except Exception as e:
        print(f"Warning: Could not load LLM config: {e}")
        # Use default config
        config = {
            'openai': {
                'model': 'gpt-4o',
                'daily_spending_limit_usd': 5.00,
                'temperature': 0.1
            },
            'browser_use': {
                'max_steps': 25
            }
        }
    
    # Initialize usage tracker if cost tracking is enabled
    usage_tracker = None
    if config.get('cost_tracking', {}).get('enabled', True):
        try:
            usage_tracker = OpenAIUsageTracker()
            print("✅ OpenAI usage tracker initialized")
            
            # Get project-specific settings
            daily_limit = config['openai']['daily_spending_limit_usd']
            project_id = config['openai'].get('project_id')
            project_ids = [project_id] if project_id else None
            
            # Check daily spending limit (project-specific if configured)
            spending_check = check_daily_spending_limit(usage_tracker, daily_limit, project_ids)
            
            project_info = f" for project {project_id}" if project_id else " (all projects)"
            print(f"💰 Daily spending check{project_info}:")
            print(f"   Current: ${spending_check['current_spending']:.4f}")
            print(f"   Limit: ${spending_check['daily_limit']:.2f}")
            print(f"   Remaining: ${spending_check['remaining_budget']:.4f}")
            
            if not spending_check['can_proceed']:
                raise Exception(f"Daily spending limit exceeded{project_info}! Current: ${spending_check['current_spending']:.4f}, Limit: ${daily_limit:.2f}")
            
            # Sync latest data before execution (project-specific if configured)
            print(f"🔄 Syncing latest usage data{project_info}...")
            if project_ids:
                usage_tracker.fetch_usage_data(days_back=1, project_ids=project_ids)
                usage_tracker.fetch_costs_data(days_back=1, project_ids=project_ids)
            else:
                usage_tracker.sync_daily_data()
            
        except Exception as e:
            if "daily spending limit" in str(e).lower():
                raise  # Re-raise spending limit errors
            print(f"Warning: Could not initialize usage tracker: {e}")
            usage_tracker = None
    
    # Set default target_time_str if not provided
    if target_time_str is None:
        target_time_str = "20250523_133057EST"
    # Create a directory for storing browser state within the browser-use folder
    # This keeps all Browser Use data separate from the user's main Chrome profile
    script_dir = pathlib.Path(__file__).parent
    browser_data_dir = script_dir / "browser_data"
    browser_data_dir.mkdir(exist_ok=True)
    cookies_file = str(browser_data_dir / 'twitter_cookies.json')
    print(f"Using browser data directory: {browser_data_dir}")
    print(f"Cookies will be saved to: {cookies_file}")
    
    # Configure browser context with persistence
    browser_config = BrowserContextConfig(
        cookies_file=cookies_file
    )
    
    # Initialize browser for persistent sessions
    browser_instance = Browser()

    sensitive_data = {
        'x_name': os.environ.get('TWITTER_NAME'),
        'x_password': os.environ.get('TWITTER_PASSWORD')
    }

    if not sensitive_data['x_name'] or not sensitive_data['x_password']:
        print("Error: TWITTER_NAME and TWITTER_PASSWORD environment variables must be set.")
        return {"error": "Twitter credentials not set."}

    # Initialize LLM with config settings
    llm_config = config['openai']
    llm = ChatOpenAI(
        model=llm_config.get('model', 'gpt-4o'),
        temperature=llm_config.get('temperature', 0.1),
        max_tokens=llm_config.get('max_tokens', 4000)
    )

    # Define initial actions to run without LLM (faster and cheaper)
    if round_reveal_url:
        print(f"Navigating directly to round reveal: {round_reveal_url}")
        initial_actions = [
            {'open_tab': {'url': round_reveal_url}},  # Go directly to the reveal tweet
        ]
        task = f"""
            You are now on the specific Twitter post where RealMir Round {round_number} reveals/guesses are posted (Target: {target_time_str}). Your task is to:
            
            1. If not already logged in, log in using sensitive_data['x_name'] and sensitive_data['x_password']
            2. DO NOT click the "Reply" button. Instead, scroll down to see the existing replies to this post.
            3. Look through all the replies below this post to find user reveals for this round.
            4. Extract all user reveals from the replies, collecting:
               - Twitter username (the @handle of the person who replied)
               - Their reveal text (the revealed prediction they posted in their reply)
            5. Continue scrolling if needed to see more replies.
            6. Format the results as JSON with this exact structure:
            {{
                "round_number": {round_number},
                "target_time": "{target_time_str}",
                "guesses": [
                    {{
                        "username": "username_here",
                        "reveal": "full_reveal_text_here"
                    }}
                ]
            }}
        """
    else:
        print(f"Searching for round reveal for Round {round_number} on realmir_testnet profile.")
        initial_actions = [
            {'open_tab': {'url': 'https://x.com'}},  # Go to Twitter/X
            {'open_tab': {'url': 'https://x.com/realmir_testnet'}},  # Navigate to realmir_testnet account
        ]
        task = f"""
            You are now on the realmir_testnet Twitter account page. Your task is to:
            
            1. If not already logged in, log in using sensitive_data['x_name'] and sensitive_data['x_password']
            2. Find the tweet containing "RealMir Round {round_number} Target: {target_time_str}"
            3. Look through all the replies to that specific tweet to find user reveals for this round
            4. Extract all user reveals from the replies, collecting:
               - Twitter username (the @handle of the person who replied)
               - Their reveal text (the revealed prediction they posted in their reply)
            5. Format the results as JSON with this exact structure:
            {{
                "round_number": {round_number},
                "target_time": "{target_time_str}",
                "guesses": [
                    {{
                        "username": "username_here",
                        "reveal": "full_reveal_text_here"
                    }}
                ]
            }}
        """

    # For testing purposes, provide a test mode option
    if os.environ.get('TWITTER_UTILS_TEST_MODE', 'false').lower() == 'true':
        print("Running in test mode with simplified task...")
        initial_actions = [{'open_tab': {'url': 'https://example.com'}}]
        task = "Tell me the main heading on this example.com page."
    else:
        print(f"Running Twitter task for Round {round_number}, target time: {target_time_str}...")

    # Create browser context with persistent cookies
    browser_context = await browser_instance.new_context(config=browser_config)
    print("Created persistent browser context with cookies support")

    agent = Agent(
        task=task,
        llm=llm,
        use_vision=True,
        sensitive_data=sensitive_data,
        browser_context=browser_context,
        initial_actions=initial_actions
    )

    raw_result = None
    session_id = f"twitter_round_{round_number}_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
    
    try:
        # Use max_steps from config or environment
        max_steps = config.get('browser_use', {}).get('max_steps', int(os.environ.get('TWITTER_UTILS_MAX_STEPS', '25')))
        print(f"Running agent with max_steps={max_steps}...")
        
        # Track execution start time
        execution_start = datetime.now()
        print(f"🚀 Starting Twitter data extraction session: {session_id}")
        
        raw_result = await agent.run(max_steps=max_steps)
        print(f"Raw result from Browser Use agent: {raw_result}")
        
        # Track execution completion
        execution_end = datetime.now()
        execution_duration = (execution_end - execution_start).total_seconds()
        print(f"⏱️ Execution completed in {execution_duration:.1f} seconds")
        
        # Track costs after execution if usage tracker is available
        if usage_tracker:
            try:
                print("📊 Tracking execution costs...")
                cost_tracking_result = track_execution_costs(usage_tracker, session_id)
                print(f"💰 Cost tracking completed: {cost_tracking_result}")
            except Exception as e:
                print(f"Warning: Could not track execution costs: {e}")

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
                        extracted = result.extracted_content
                        print(f"Found extracted content: {extracted}")
                        try:
                            # Try to parse as JSON if it looks like JSON
                            if isinstance(extracted, str) and extracted.strip().startswith('{'):
                                return json.loads(extracted)
                        except json.JSONDecodeError:
                            pass
                        # Return the extracted content directly
                        return {"output": extracted}
            
            # Check if we can get the final result using the final_result() method
            if hasattr(raw_result, 'final_result'):
                final_result = raw_result.final_result()
                if final_result:
                    print(f"Found final result: {final_result}")
                    try:
                        if isinstance(final_result, str) and final_result.strip().startswith('{'):
                            return json.loads(final_result)
                    except json.JSONDecodeError:
                        pass
                    return {"output": final_result}
            
            # Fallback to returning the string representation
            return {"output": str(raw_result)}

    except Exception as e:
        print(f"An error occurred while running the Browser Use agent: {e}")
        return {"error": str(e)}
    finally:
        # Close the browser context and browser properly
        if 'browser_context' in locals():
            await browser_context.close()
            print("Browser context closed")
        await browser_instance.close()
        print("Browser closed")
        
        # Keep the browser data persistent for future sessions
        print(f"Browser data (including cookies) preserved at: {browser_data_dir}")

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