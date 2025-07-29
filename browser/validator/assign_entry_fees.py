"""
Entry Fee Assignment Module for Cliptions Validators (TEST VERSION)

🧪 THIS IS A TWITTER AUTOMATION TEST - NOT A GAME SIMULATION
- Focus: Test if we can reply to existing tweets with text content
- Goal: Verify browser-use can post replies reliably
- Scope: Testing posting mechanics, not real payment processing

This module tests the ability to reply to commitment tweets with entry fee instructions.
"""

import asyncio
import json
import logging
import time
from datetime import datetime
from typing import List, Optional, Any, Dict
from pathlib import Path
from pydantic import BaseModel, Field

# Fixed imports based on diagnostics
from browser_use import Agent
from langchain_openai import ChatOpenAI
from ..core.base_task import BaseTwitterTask
from ..core.interfaces import ExtractionError


class EntryFeeReply(BaseModel):
    """Reply posted for entry fee assignment."""
    commitment_url: str
    reply_url: str
    reply_text: str
    posted_at: datetime
    success: bool = True


class EntryFeeAssignmentResult(BaseModel):
    """Result of entry fee assignment process."""
    total_commitments: int = 0
    successful_replies: int = 0
    failed_replies: int = 0
    replies: List[EntryFeeReply] = Field(default_factory=list)
    execution_time_seconds: float = 0.0
    errors: List[str] = Field(default_factory=list)


class AssignEntryFeesTask(BaseTwitterTask):
    """
    Task to assign entry fees by replying to commitment tweets.
    
    🧪 TEST FOCUS: Can we reliably post text replies to existing tweets?
    """
    
    def __init__(self):
        super().__init__()
        self.logger = logging.getLogger(__name__)  # Add logger
        self.fake_tao_address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"  # Fake TAO address for testing
        # Initialize LLM for browser-use
        self.llm = ChatOpenAI(
            model="gpt-4o-mini",  # Use cheaper model for testing
            temperature=0.0,
        )
        
    def load_commitment_urls(self) -> List[str]:
        """Load commitment URLs from blocks.json."""
        try:
            blocks_file = Path("data/blocks.json")
            if not blocks_file.exists():
                self.logger.warning(f"Blocks file not found: {blocks_file}")
                return []
                
            with open(blocks_file, 'r') as f:
                blocks_data = json.load(f)
            
            # Extract commitment URLs from block2 (known to have data)
            commitment_urls = []
            if "block2" in blocks_data:
                block2_data = blocks_data["block2"]
                # Try collected_commitments first (newer format)
                if "collected_commitments" in block2_data and block2_data["collected_commitments"].get("commitments"):
                    commitments = block2_data["collected_commitments"]["commitments"]
                    for commitment in commitments:
                        if "tweet_url" in commitment:
                            commitment_urls.append(commitment["tweet_url"])
                # Fallback to participants (older format)
                elif "participants" in block2_data:
                    participants = block2_data["participants"]
                    for participant in participants:
                        if "commitment_url" in participant:
                            commitment_urls.append(participant["commitment_url"])
            
            self.logger.info(f"Loaded {len(commitment_urls)} commitment URLs")
            return commitment_urls[:2]  # Limit to 2 for testing
            
        except Exception as e:
            self.logger.error(f"Error loading commitment URLs: {e}")
            return []
    
    async def execute(self, **kwargs) -> EntryFeeAssignmentResult:
        """Execute entry fee assignment task."""
        
        start_time = time.time()
        result = EntryFeeAssignmentResult()
        
        try:
            # Load commitment URLs
            commitment_urls = self.load_commitment_urls()
            if not commitment_urls:
                raise ExtractionError("No commitment URLs found to process")
            
            result.total_commitments = len(commitment_urls)
            self.logger.info(f"🎯 Processing {len(commitment_urls)} commitment URLs")
            
            # Process each commitment URL
            for i, commitment_url in enumerate(commitment_urls, 1):
                self.logger.info(f"📝 Processing commitment {i}/{len(commitment_urls)}: {commitment_url}")
                
                try:
                    # Create reply task for this commitment
                    reply_result = await self._reply_to_commitment(commitment_url)
                    
                    if reply_result:
                        result.replies.append(reply_result)
                        result.successful_replies += 1
                        self.logger.info(f"✅ Successfully replied to commitment {i}")
                    else:
                        result.failed_replies += 1
                        result.errors.append(f"Failed to reply to commitment {i}: {commitment_url}")
                        self.logger.error(f"❌ Failed to reply to commitment {i}")
                        
                except Exception as e:
                    result.failed_replies += 1
                    error_msg = f"Error processing commitment {i}: {str(e)}"
                    result.errors.append(error_msg)
                    self.logger.error(error_msg)
            
            # Calculate execution time
            result.execution_time_seconds = time.time() - start_time
            
            # Log summary
            self.logger.info(f"📊 Entry fee assignment completed:")
            self.logger.info(f"   Total commitments: {result.total_commitments}")
            self.logger.info(f"   Successful replies: {result.successful_replies}")
            self.logger.info(f"   Failed replies: {result.failed_replies}")
            self.logger.info(f"   Execution time: {result.execution_time_seconds:.1f}s")
            
            return result
            
        except Exception as e:
            result.execution_time_seconds = time.time() - start_time
            error_msg = f"Entry fee assignment failed: {str(e)}"
            result.errors.append(error_msg)
            self.logger.error(error_msg)
            raise ExtractionError(error_msg)
    
    async def _reply_to_commitment(self, commitment_url: str) -> Optional[EntryFeeReply]:
        """Reply to a specific commitment tweet with entry fee instructions."""
        
        try:
            # Create reply text
            reply_text = f"💰 Entry fee required: Send 0.1 TAO to {self.fake_tao_address} to participate in this block. #cliptions #entry_fee"
            
            # Create browser-use task (focused on checking for existing replies first, then replying if needed)
            task = f"""
            You are already on the correct tweet page. Your task is to check if we've already replied, and if not, post a reply.
            
            Reply text: "{reply_text}"
            
            Steps:
            1. First, look at the current page and check if there are any existing replies from @cliptions_test or @cliptions_test
            2. If you see a reply from our account (cliptions_test or cliptions_test), DO NOT post another reply - just report that we already replied
            3. If you don't see any existing reply from our account, then proceed to post the reply:
               a. Look for the reply button or reply text area on the current page
               b. Click the reply button to open the compose interface
               c. Type the reply text exactly as provided: "{reply_text}"
               d. Click the Reply/Post button to submit the reply
               e. Wait for the reply to be posted and verify it appears in the conversation
            
            IMPORTANT: 
            - You are already on the correct tweet page, do NOT navigate anywhere
            - Check for existing replies from @cliptions_test or @cliptions_test BEFORE posting
            - If we already replied, just say "Already replied" and use the done action
            - If posting a new reply, use the exact reply text provided above
            - Make sure to actually post the reply, don't just draft it
            
            Success criteria: Either confirm we already replied, or successfully post a new reply
            """
            
            # Create initial actions to navigate to the tweet URL
            initial_actions = [
                {'go_to_url': {'url': commitment_url}}
            ]
            
            # Use BaseTwitterTask's setup_agent method which handles cookies and browser context
            agent = await self.setup_agent(
                task=task,
                initial_actions=initial_actions,
                use_vision=True  # Enable vision for better Twitter interaction
            )
            
            self.logger.info(f"🚀 Starting browser automation for: {commitment_url}")
            self.logger.info(f"📍 Using initial actions to navigate to tweet URL")
            self.logger.info(f"🍪 Using saved cookies and browser context from BaseTwitterTask")
            self.logger.info(f"🔍 Will check for existing replies before posting")
            
            # Execute with timeout
            try:
                result = await agent.run(max_steps=15)  # More steps to allow for checking existing replies
                
                # Check if the result indicates we already replied
                result_str = str(result).lower()
                if "already replied" in result_str or "existing reply" in result_str:
                    self.logger.info(f"✅ Already replied to this tweet, skipping")
                    return EntryFeeReply(
                        commitment_url=commitment_url,
                        reply_url="already_replied",
                        reply_text="Already replied to this tweet",
                        posted_at=datetime.now(),
                        success=True
                    )
                
                # Parse result to extract reply URL
                reply_url = self._extract_reply_url_from_result(result)
                
                if reply_url:
                    return EntryFeeReply(
                        commitment_url=commitment_url,
                        reply_url=reply_url,
                        reply_text=reply_text,
                        posted_at=datetime.now(),
                        success=True
                    )
                else:
                    self.logger.warning(f"Could not extract reply URL from result")
                    return None
                    
            except Exception as e:
                self.logger.error(f"Browser automation failed: {e}")
                return None
                
        except Exception as e:
            self.logger.error(f"Error in _reply_to_commitment: {e}")
            return None
    
    def _extract_reply_url_from_result(self, result) -> Optional[str]:
        """Extract reply URL from browser automation result."""
        try:
            # Convert result to string and look for Twitter URLs
            result_str = str(result)
            
            # Look for Twitter/X URLs in the result
            import re
            twitter_urls = re.findall(r'https://(?:twitter\.com|x\.com)/\w+/status/\d+', result_str)
            
            if twitter_urls:
                # Return the last URL found (likely the reply)
                return twitter_urls[-1]
            
            return None
            
        except Exception as e:
            self.logger.error(f"Error extracting reply URL: {e}")
            return None


async def main():
    """Main function for testing entry fee assignment."""
    
    print("🧪 TESTING: Entry Fee Assignment")
    print("=" * 50)
    print("🎯 OBJECTIVE: Test if we can reply to commitment tweets with entry fee instructions")
    print("📋 SCOPE: Twitter automation mechanics testing")
    print()
    
    try:
        # Create and execute task
        task = AssignEntryFeesTask()
        result = await task.execute()
        
        # Output results
        print("📊 RESULTS:")
        print(f"✅ Total commitments processed: {result.total_commitments}")
        print(f"✅ Successful replies: {result.successful_replies}")
        print(f"❌ Failed replies: {result.failed_replies}")
        print(f"⏱️  Execution time: {result.execution_time_seconds:.1f} seconds")
        
        if result.replies:
            print("\n🔗 Posted replies:")
            for reply in result.replies:
                print(f"  • {reply.commitment_url} → {reply.reply_url}")
        
        if result.errors:
            print("\n❌ Errors:")
            for error in result.errors:
                print(f"  • {error}")
        
        # Output JSON for verification
        output_file = f"entry_fee_assignment_result_{int(time.time())}.json"
        with open(output_file, 'w') as f:
            json.dump(result.dict(), f, indent=2, default=str)
        print(f"\n📄 Detailed results saved to: {output_file}")
        
    except Exception as e:
        print(f"💥 TASK FAILED: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    asyncio.run(main()) 