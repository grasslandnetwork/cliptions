#!/usr/bin/env python3
"""
Browser Use Cost Tracker

Modular cost tracking system for Browser Use applications.
Handles daily spending limits, session tracking, and project-specific cost monitoring.
"""

from datetime import datetime
from typing import Optional, List, Dict
from openai_usage_tracker import OpenAIUsageTracker


class BrowserUseCostTracker:
    """
    Manages cost tracking for Browser Use applications.
    
    Responsibilities:
    - Daily spending limit enforcement
    - Session cost tracking
    - Project-specific cost filtering
    - Pre and post-execution cost monitoring
    """
    
    def __init__(self, daily_limit: float, project_id: Optional[str] = None, enabled: bool = True):
        """
        Initialize the cost tracker.
        
        Args:
            daily_limit: Daily spending limit in USD
            project_id: Optional OpenAI project ID for filtering
            enabled: Whether cost tracking is enabled
        """
        self.daily_limit = daily_limit
        self.project_id = project_id
        self.project_ids = [project_id] if project_id else None
        self.enabled = enabled
        self.usage_tracker = None
        
        if self.enabled:
            self._initialize_tracker()
    
    def _initialize_tracker(self) -> None:
        """Initialize the OpenAI usage tracker"""
        try:
            self.usage_tracker = OpenAIUsageTracker()
            print("✅ OpenAI usage tracker initialized")
        except Exception as e:
            print(f"Warning: Could not initialize usage tracker: {e}")
            self.enabled = False
    
    def check_daily_spending_limit(self) -> Dict:
        """
        Check if current daily spending is under the limit.
        Daily reset occurs at 00:00 UTC.
        
        Returns:
            Dict with can_proceed, current_spending, remaining_budget, etc.
        """
        if not self.enabled or not self.usage_tracker:
            return {
                'can_proceed': True,
                'current_spending': 0.00,
                'remaining_budget': self.daily_limit,
                'daily_limit': self.daily_limit,
                'date': datetime.utcnow().date().isoformat(),
                'project_ids': self.project_ids,
                'tracking_enabled': False
            }
        
        today = datetime.utcnow().date()
        
        # If project filtering is requested, fetch project-specific data
        if self.project_ids:
            self.usage_tracker.fetch_costs_data(days_back=1, project_ids=self.project_ids)
        
        costs_data = self.usage_tracker.get_daily_costs(today)
        
        if costs_data is None:
            current_spending = 0.00
        else:
            current_spending = costs_data['total_cost_usd']
        
        remaining_budget = self.daily_limit - current_spending
        can_proceed = remaining_budget > 0
        
        return {
            'can_proceed': can_proceed,
            'current_spending': current_spending,
            'remaining_budget': remaining_budget,
            'daily_limit': self.daily_limit,
            'date': today.isoformat(),
            'project_ids': self.project_ids,
            'tracking_enabled': True
        }
    
    def validate_spending_limit(self) -> None:
        """
        Validate that current spending is under the daily limit.
        
        Raises:
            Exception: If daily spending limit is exceeded
        """
        spending_check = self.check_daily_spending_limit()
        
        project_info = f" for project {self.project_id}" if self.project_id else " (all projects)"
        print(f"💰 Daily spending check{project_info}:")
        print(f"   Current: ${spending_check['current_spending']:.4f}")
        print(f"   Limit: ${spending_check['daily_limit']:.2f}")
        print(f"   Remaining: ${spending_check['remaining_budget']:.4f}")
        
        if not spending_check['can_proceed']:
            raise Exception(
                f"Daily spending limit exceeded{project_info}! "
                f"Current: ${spending_check['current_spending']:.4f}, "
                f"Limit: ${self.daily_limit:.2f}"
            )
    
    def sync_latest_data(self) -> None:
        """Sync latest usage data before execution"""
        if not self.enabled or not self.usage_tracker:
            return
        
        project_info = f" for project {self.project_id}" if self.project_id else " (all projects)"
        print(f"🔄 Syncing latest usage data{project_info}...")
        
        if self.project_ids:
            self.usage_tracker.fetch_usage_data(days_back=1, project_ids=self.project_ids)
            self.usage_tracker.fetch_costs_data(days_back=1, project_ids=self.project_ids)
        else:
            self.usage_tracker.sync_daily_data()
    
    def track_execution_costs(self, session_id: str) -> Dict:
        """
        Track costs for a browser-use execution session.
        
        Args:
            session_id: Unique identifier for this session
            
        Returns:
            Dict with session tracking information
        """
        if not self.enabled or not self.usage_tracker:
            return {
                'session_id': session_id,
                'start_time': datetime.now().isoformat(),
                'end_time': datetime.now().isoformat(),
                'tracking_enabled': False
            }
        
        start_time = datetime.now()
        
        # Sync latest usage data
        sync_result = self.usage_tracker.sync_daily_data()
        
        end_time = datetime.now()
        
        return {
            'session_id': session_id,
            'start_time': start_time.isoformat(),
            'end_time': end_time.isoformat(),
            'sync_result': sync_result,
            'tracking_enabled': True
        }
    
    def generate_session_id(self, prefix: str = "browser_use") -> str:
        """
        Generate a unique session ID.
        
        Args:
            prefix: Prefix for the session ID
            
        Returns:
            Unique session ID string
        """
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        return f"{prefix}_{timestamp}"
    
    def log_execution_start(self, session_id: str, task_description: str = "") -> datetime:
        """
        Log the start of an execution session.
        
        Args:
            session_id: Unique session identifier
            task_description: Optional description of the task
            
        Returns:
            Start time for duration tracking
        """
        start_time = datetime.now()
        task_info = f": {task_description}" if task_description else ""
        print(f"🚀 Starting execution session: {session_id}{task_info}")
        return start_time
    
    def log_execution_end(self, start_time: datetime, session_id: str) -> None:
        """
        Log the end of an execution session and track costs.
        
        Args:
            start_time: Start time from log_execution_start
            session_id: Session identifier
        """
        end_time = datetime.now()
        duration = (end_time - start_time).total_seconds()
        print(f"⏱️ Execution completed in {duration:.1f} seconds")
        
        # Track costs after execution if enabled
        if self.enabled:
            try:
                print("📊 Tracking execution costs...")
                cost_tracking_result = self.track_execution_costs(session_id)
                print(f"💰 Cost tracking completed: {cost_tracking_result}")
            except Exception as e:
                print(f"Warning: Could not track execution costs: {e}")


def create_cost_tracker_from_config(config: Dict) -> BrowserUseCostTracker:
    """
    Factory function to create a cost tracker from configuration.
    
    Args:
        config: Configuration dictionary containing OpenAI and cost tracking settings
        
    Returns:
        Configured BrowserUseCostTracker instance
    """
    # Extract settings from config
    openai_config = config.get('openai', {})
    cost_config = config.get('cost_tracking', {})
    
    daily_limit = openai_config.get('daily_spending_limit_usd', 5.00)
    project_id = openai_config.get('project_id')
    enabled = cost_config.get('enabled', True)
    
    return BrowserUseCostTracker(
        daily_limit=daily_limit,
        project_id=project_id,
        enabled=enabled
    ) 