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
        chrome_instance_path='/opt/google/chrome/google-chrome',  # macOS path
        # For Windows, typically: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe'
        # For Linux, typically: '/usr/bin/google-chrome'
    )
)


# Define sensitive data
# The model will only see the keys (x_name, x_password) but never the actual values
sensitive_data = {'x_name': os.environ['TWITTER_NAME'], 'x_password': os.environ['TWITTER_PASSWORD'] }


llm = ChatOpenAI(model="gpt-4o")

async def main():
    agent = Agent(
        task="Go to x.com and login with x_name and x_password and then open Notifications. Then close the browser",
        llm=llm,
        use_vision=True,              # Enable vision capabilities
        sensitive_data=sensitive_data,
    )
    result = await agent.run()

    await browser.close()
    print(result)



asyncio.run(main())
