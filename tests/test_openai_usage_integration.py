import pytest
import os
import json
import tempfile
import yaml
from datetime import datetime, timedelta
from pathlib import Path
from unittest.mock import patch, MagicMock

# Import the actual modules we'll be testing
import sys
sys.path.append('browser-use')
from browser.openai_usage_tracker import OpenAIUsageTracker

# Import the actual twitter_data_fetcher functions
sys.path.append('browser-use')
from browser.core.base_task import BaseTwitterTask
from browser.core.cost_tracker import BrowserUseCostTracker


class TestOpenAIUsageIntegration:
    """Test OpenAI usage tracking and spending limits integration"""
    
    @pytest.fixture
    def temp_config_dir(self):
        """Create a temporary directory for config files"""
        with tempfile.TemporaryDirectory() as temp_dir:
            yield Path(temp_dir)
    
    @pytest.fixture
    def mock_usage_tracker(self, temp_config_dir):
        """Create a mock usage tracker with temporary data directory"""
        with patch.dict(os.environ, {'OPENAI_API_KEY_FOR_USAGE_AND_COSTS': 'test-key'}):
            tracker = OpenAIUsageTracker()
            # Override data directory to use temp directory
            tracker.data_dir = temp_config_dir / 'browser_data'
            tracker.data_dir.mkdir(exist_ok=True)
            tracker.usage_file = tracker.data_dir / 'openai_actual_usage.json'
            tracker.costs_file = tracker.data_dir / 'openai_actual_costs.json'
            return tracker
    
    def test_daily_spending_limit_config_loading(self, temp_config_dir):
        """Test loading daily spending limit from config/llm.yaml with environment variables"""
        # Create config directory and file
        config_dir = temp_config_dir / 'config'
        config_dir.mkdir()
        
        # Create config with environment variable substitution
        config_content = """
openai:
  daily_spending_limit_usd: 5.00
  model: "gpt-4o"
  temperature: 0.1
  project_id: "${TEST_PROJECT_ID}"
"""
        
        config_file = config_dir / 'llm.yaml'
        with open(config_file, 'w') as f:
            f.write(config_content)
        
        # Test that we can load the config using the actual function with env var
        with patch.dict(os.environ, {'TEST_PROJECT_ID': 'proj_test123'}):
            config = load_llm_config(str(config_file))
        
        assert config['openai']['daily_spending_limit_usd'] == 5.00
        assert config['openai']['model'] == 'gpt-4o'
        assert config['openai']['project_id'] == 'proj_test123'
    
    def test_spending_limit_check_under_limit(self, mock_usage_tracker):
        """Test that spending check passes when under daily limit"""
        # Mock cost data showing $2.50 spent today
        today = datetime.now().date()
        mock_costs = {
            'date': today.isoformat(),
            'total_cost_usd': 2.50,
            'breakdown': {'gpt-4o (test-project)': 2.50}
        }
        
        with patch.object(mock_usage_tracker, 'get_daily_costs', return_value=mock_costs):
            with patch.object(mock_usage_tracker, 'fetch_costs_data', return_value=None):
                # Should pass with $5.00 limit using actual function
                result = check_daily_spending_limit(mock_usage_tracker, daily_limit=5.00)
                assert result['can_proceed'] is True
                assert result['current_spending'] == 2.50
                assert result['remaining_budget'] == 2.50
    
    def test_spending_limit_check_over_limit(self, mock_usage_tracker):
        """Test that spending check fails when over daily limit"""
        # Mock cost data showing $6.00 spent today
        today = datetime.now().date()
        mock_costs = {
            'date': today.isoformat(),
            'total_cost_usd': 6.00,
            'breakdown': {'gpt-4o (test-project)': 6.00}
        }
        
        with patch.object(mock_usage_tracker, 'get_daily_costs', return_value=mock_costs):
            with patch.object(mock_usage_tracker, 'fetch_costs_data', return_value=None):
                # Should fail with $5.00 limit using actual function
                result = check_daily_spending_limit(mock_usage_tracker, daily_limit=5.00)
                assert result['can_proceed'] is False
                assert result['current_spending'] == 6.00
                assert result['remaining_budget'] == -1.00
    
    def test_spending_limit_check_no_data(self, mock_usage_tracker):
        """Test spending check when no cost data is available"""
        with patch.object(mock_usage_tracker, 'get_daily_costs', return_value=None):
            with patch.object(mock_usage_tracker, 'fetch_costs_data', return_value=None):
                # Should proceed when no data available (assume $0 spent) using actual function
                result = check_daily_spending_limit(mock_usage_tracker, daily_limit=5.00)
                assert result['can_proceed'] is True
                assert result['current_spending'] == 0.00
                assert result['remaining_budget'] == 5.00
    
    def test_project_specific_spending_limit_check(self, mock_usage_tracker):
        """Test spending check with project-specific filtering"""
        # Mock cost data for specific project
        today = datetime.now().date()
        mock_costs = {
            'date': today.isoformat(),
            'total_cost_usd': 1.50,
            'breakdown': {'gpt-4o (proj_eQM5yuxSlkAmAQIf7mEpL00m)': 1.50}
        }
        
        project_ids = ['proj_eQM5yuxSlkAmAQIf7mEpL00m']
        
        with patch.object(mock_usage_tracker, 'get_daily_costs', return_value=mock_costs):
            with patch.object(mock_usage_tracker, 'fetch_costs_data', return_value=None) as mock_fetch:
                # Should pass with project-specific limit
                result = check_daily_spending_limit(mock_usage_tracker, daily_limit=5.00, project_ids=project_ids)
                
                # Verify project-specific data was fetched
                mock_fetch.assert_called_once_with(days_back=1, project_ids=project_ids)
                
                assert result['can_proceed'] is True
                assert result['current_spending'] == 1.50
                assert result['remaining_budget'] == 3.50
                assert result['project_ids'] == project_ids
    
    @pytest.mark.asyncio
    async def test_twitter_fetcher_respects_spending_limit(self, temp_config_dir):
        """Test that twitter_data_fetcher respects daily spending limits"""
        # Create config with low spending limit
        config_dir = temp_config_dir / 'config'
        config_dir.mkdir()
        
        llm_config = {
            'openai': {
                'daily_spending_limit_usd': 0.01,  # Very low limit
                'model': 'gpt-4o'
            },
            'cost_tracking': {
                'enabled': True
            }
        }
        
        config_file = config_dir / 'llm.yaml'
        with open(config_file, 'w') as f:
            yaml.dump(llm_config, f)
        
        # Mock high spending for today
        mock_costs = {
            'date': datetime.now().date().isoformat(),
            'total_cost_usd': 5.00,
            'breakdown': {'gpt-4o (test-project)': 5.00}
        }
        
        with patch.dict(os.environ, {
            'OPENAI_API_KEY_FOR_USAGE_AND_COSTS': 'test-key',
            'TWITTER_UTILS_TEST_MODE': 'true'
        }):
            with patch('browser.twitter_data_fetcher.OpenAIUsageTracker') as mock_tracker_class:
                mock_tracker = MagicMock()
                mock_tracker.get_daily_costs.return_value = mock_costs
                mock_tracker.sync_daily_data.return_value = None
                mock_tracker_class.return_value = mock_tracker
                
                # fetch_round_guesses function is being deprecated - tests will be updated separately
                
                # Should raise exception due to spending limit
                with pytest.raises(Exception) as exc_info:
                    await fetch_round_guesses(0, config_file_path=str(config_file))
                
                assert "daily spending limit" in str(exc_info.value).lower()
    
    def test_cost_tracking_during_execution(self, mock_usage_tracker):
        """Test that costs are tracked during browser-use execution"""
        # This test would verify that we're properly tracking costs
        # during actual browser automation runs
        
        # Mock the browser-use agent to simulate cost tracking
        with patch('browser_use.Agent') as mock_agent_class:
            mock_agent = MagicMock()
            mock_agent_class.return_value = mock_agent
            
            # Mock the usage tracker to record a call
            with patch.object(mock_usage_tracker, 'sync_daily_data') as mock_sync:
                mock_sync.return_value = {
                    'usage': {'total_input_tokens': 1000, 'total_output_tokens': 500},
                    'costs': {'total_cost_usd': 0.15}
                }
                
                # Simulate tracking costs for a browser-use session using actual function
                result = track_execution_costs(mock_usage_tracker, "test_session")
                
                assert result['session_id'] == "test_session"
                assert 'start_time' in result
                assert 'end_time' in result
                mock_sync.assert_called_once() 