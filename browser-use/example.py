from langchain_openai import ChatOpenAI
from browser_use import Agent, Browser, BrowserConfig
from dotenv import load_dotenv
load_dotenv()

import asyncio
import os

# Configure the browser to connect to your Chrome instance
browser = Browser(
    config=BrowserConfig(
        # Specify the path to your Chrome executable
        chrome_instance_path='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', # for macOS
        # For Windows, typically: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe'
        # For Linux, typically: '/usr/bin/google-chrome' or '/opt/google/chrome/google-chrome
    )
)

# Define sensitive data
# The model will only see the keys (x_name, x_password) but never the actual values
sensitive_data = {'x_name': os.environ['TWITTER_NAME'], 'x_password': os.environ['TWITTER_PASSWORD'] }


llm = ChatOpenAI(model="gpt-4o")

NUMBER = 0
TARGETTIME = "time133057EST"
# Simple login task
task_login = "Go to x.com / formerly twitter.com and if you're not logged in, login with x_name and x_password."

# RealMir game data collection task
task_realmir = """Task: Collect RealMir game guesses from Twitter replies.

Steps:
1."""+task_login+"""
2. Search for @realmir_testnet in the search bar and go to their profile
3. Look for tweets by @realmir_testnet that contains BOTH of these hashtags:
   - Exactly "#round"""+str(NUMBER)+"""" (without quotes)
   - Exactly "#target"""+TARGETTIME+"""" (without quotes)
   - Do NOT accept any other hash tags like #targetframe etc., only match #target"""+TARGETTIME+"""
4. When you find such a tweet, stay on the same page and click on the tweet's DATE to view replies. DON'T CLICK THE REPLY BUTTON, JUST CLICK THE TWEET' DATE ITSELF.
5. Collect all replies containing guesses:
   - Look for patterns like:
     * "I commit to guess: [GUESS]"
     * "My guess: [GUESS]"
     * "Guessing: [GUESS]"
     * "Commit: [GUESS]"
   - If no pattern matches, use the full reply text

Return data in this format:
{
  "round": """+str(NUMBER)+""",
  "targetTime": \""""+TARGETTIME+"""\",
  "guesses": [
    {"username": "user1", "guess": "guess text"},  // NOTE: Username should NOT include the @ symbol
    {"username": "user2", "guess": "guess text"}   // Example: Use "user1" not "@user1"
  ]
}"""


async def main():
    agent = Agent(
        task=task_realmir,  # Using the RealMir task
        llm=llm,
        use_vision=True,              # Enable vision capabilities
        sensitive_data=sensitive_data,
        browser=browser
    )
    result = await agent.run()

    await browser.close()
    print(result)



asyncio.run(main())
