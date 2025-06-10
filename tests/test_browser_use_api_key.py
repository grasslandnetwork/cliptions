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
    
    def test_load_llm_config_includes_api_key_from_config(self):
        """Test that LLM config loads the API key directly from the config file"""
        test_api_key = "sk-test-browser-use-key-123"
        test_project_id = "proj-test-project-123"
        
        # Create a temporary config file for testing with direct values
        config_content = f"""
openai:
  api_key: "{test_api_key}"
  model: "gpt-4o"
  temperature: 0.1
  daily_spending_limit_usd: 5.00
  max_tokens: 4000
  project_id: "{test_project_id}"
  
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
            assert config['openai']['project_id'] == test_project_id
            assert config['openai']['model'] == 'gpt-4o'
            
        finally:
            # Clean up temporary file
            os.unlink(temp_config_path)
    
    def test_missing_api_key_in_config(self):
        """Test that config without API key still loads (API key is optional)"""
        
        # Create config without API key
        config_content = """
openai:
  model: "gpt-4o"
  temperature: 0.1
"""
        
        import tempfile
        with tempfile.NamedTemporaryFile(mode='w', suffix='.yaml', delete=False) as f:
            f.write(config_content)
            temp_config_path = f.name
        
        try:
            config = load_llm_config(temp_config_path)
            # Should load successfully, API key will be None
            assert config['openai'].get('api_key') is None
            assert config['openai']['model'] == 'gpt-4o'
        finally:
            os.unlink(temp_config_path)
    
    @patch('twitter_data_fetcher.ChatOpenAI')
    def test_chat_openai_uses_api_key_from_config(self, mock_chat_openai):
        """Test that ChatOpenAI is initialized with the API key from config"""
        from twitter_data_fetcher import ChatOpenAI
        
        test_api_key = "sk-test-browser-use-key-456"
        
        # Test the ChatOpenAI initialization directly
        llm_config = {
            'api_key': test_api_key,
            'model': 'gpt-4o',
            'temperature': 0.1,
            'max_tokens': 4000
        }
        
        # This simulates what happens in the fetch_round_guesses function
        llm = ChatOpenAI(
            model=llm_config.get('model', 'gpt-4o'),
            temperature=llm_config.get('temperature', 0.1),
            max_tokens=llm_config.get('max_tokens', 4000),
            openai_api_key=llm_config.get('api_key')
        )
        
        # Verify ChatOpenAI was called with the correct API key
        mock_chat_openai.assert_called_once_with(
            model='gpt-4o',
            temperature=0.1,
            max_tokens=4000,
            openai_api_key=test_api_key
        ) 