import os
import pytest
from unittest.mock import patch, MagicMock
import sys
import pathlib

# Add the browser-use directory to the path
sys.path.insert(0, str(pathlib.Path(__file__).parent.parent / "browser-use"))

from twitter_data_fetcher import load_llm_config


class TestBrowserUseAPIKey:
    """Test that browser-use operations use the dedicated API key"""
    
    def test_load_llm_config_includes_api_key_from_env(self):
        """Test that LLM config loads the dedicated browser-use API key from environment"""
        test_api_key = "sk-test-browser-use-key-123"
        
        with patch.dict(os.environ, {
            'OPENAI_API_KEY_FOR_REALMIR_BROWSER_USE': test_api_key,
            'OPENAI_REALMIR_PROJECT_ID': 'test-project-id'
        }):
            # Create a temporary config file for testing
            config_content = """
openai:
  api_key: "${OPENAI_API_KEY_FOR_REALMIR_BROWSER_USE}"
  model: "gpt-4o"
  temperature: 0.1
  daily_spending_limit_usd: 5.00
  max_tokens: 4000
  project_id: "${OPENAI_REALMIR_PROJECT_ID}"
  
browser_use:
  max_steps: 25
  use_vision: true
  timeout_seconds: 300
  
cost_tracking:
  enabled: true
  sync_frequency_hours: 1
  alert_threshold_percent: 80
"""
            
            # Write temporary config file
            import tempfile
            with tempfile.NamedTemporaryFile(mode='w', suffix='.yaml', delete=False) as f:
                f.write(config_content)
                temp_config_path = f.name
            
            try:
                config = load_llm_config(temp_config_path)
                
                # Verify the API key is loaded correctly
                assert config['openai']['api_key'] == test_api_key
                assert config['openai']['project_id'] == 'test-project-id'
                assert config['openai']['model'] == 'gpt-4o'
                
            finally:
                # Clean up temporary file
                os.unlink(temp_config_path)
    
    def test_missing_browser_use_api_key_raises_error(self):
        """Test that missing OPENAI_API_KEY_FOR_REALMIR_BROWSER_USE raises appropriate error"""
        
        # Create config that references the missing env var
        config_content = """
openai:
  api_key: "${OPENAI_API_KEY_FOR_REALMIR_BROWSER_USE}"
  model: "gpt-4o"
"""
        
        import tempfile
        with tempfile.NamedTemporaryFile(mode='w', suffix='.yaml', delete=False) as f:
            f.write(config_content)
            temp_config_path = f.name
        
        try:
            # Ensure the env var is not set
            with patch.dict(os.environ, {}, clear=True):
                with pytest.raises(ValueError, match="Environment variable OPENAI_API_KEY_FOR_REALMIR_BROWSER_USE is not set"):
                    load_llm_config(temp_config_path)
        finally:
            os.unlink(temp_config_path)
    
    @patch('browser_use.Agent')
    @patch('langchain_openai.ChatOpenAI')
    def test_chat_openai_uses_dedicated_api_key(self, mock_chat_openai, mock_agent):
        """Test that ChatOpenAI is initialized with the dedicated API key"""
        from twitter_data_fetcher import fetch_round_guesses
        
        test_api_key = "sk-test-browser-use-key-456"
        
        # Mock the config loading to return our test API key
        mock_config = {
            'openai': {
                'api_key': test_api_key,
                'model': 'gpt-4o',
                'temperature': 0.1,
                'max_tokens': 4000,
                'daily_spending_limit_usd': 5.00
            },
            'browser_use': {
                'max_steps': 25
            },
            'cost_tracking': {
                'enabled': False  # Disable for testing
            }
        }
        
        # Mock environment variables
        with patch.dict(os.environ, {
            'OPENAI_API_KEY_FOR_REALMIR_BROWSER_USE': test_api_key,
            'TWITTER_NAME': 'test_user',
            'TWITTER_PASSWORD': 'test_pass',
            'TWITTER_UTILS_TEST_MODE': 'true'
        }):
            with patch('browser_use.twitter_data_fetcher.load_llm_config', return_value=mock_config):
                with patch('browser_use.Browser') as mock_browser:
                    # Mock browser instance and context
                    mock_browser_instance = MagicMock()
                    mock_browser.return_value = mock_browser_instance
                    mock_context = MagicMock()
                    mock_browser_instance.new_context.return_value.__aenter__.return_value = mock_context
                    mock_browser_instance.close.return_value = MagicMock()
                    
                    # Mock agent run to return immediately
                    mock_agent_instance = MagicMock()
                    mock_agent.return_value = mock_agent_instance
                    mock_agent_instance.run.return_value = '{"test": "result"}'
                    
                    # Run the function
                    import asyncio
                    result = asyncio.run(fetch_round_guesses(1, "20250523_133057EST"))
                    
                    # Verify ChatOpenAI was called with the correct API key
                    mock_chat_openai.assert_called_once()
                    call_kwargs = mock_chat_openai.call_args[1]
                    assert call_kwargs['openai_api_key'] == test_api_key
                    assert call_kwargs['model'] == 'gpt-4o'
                    assert call_kwargs['temperature'] == 0.1
                    assert call_kwargs['max_tokens'] == 4000 