# Contributing to RealMir

Thank you for your interest in contributing to RealMir! This document provides detailed setup instructions and development guidelines.

## Table of Contents
- [Development Setup](#development-setup)
- [Browser Automation Setup](#browser-automation-setup)
- [OpenAI Cost Management](#openai-cost-management)
- [Running Tests](#running-tests)
- [Installing Dependencies](#installing-dependencies)
- [Pull Request Process](#pull-request-process)

## Development Setup

### Basic Setup
1. Clone the repository
2. Create a new branch for your feature or bugfix
3. Install dependencies:
```bash
pip install -r requirements.txt
```

### Python Environment Setup
```bash
# Create virtual environment with Python 3.11
uv venv --python 3.11

# Activate virtual environment:
# For Windows (Command Prompt):
.venv\Scripts\activate
# For Windows (PowerShell):
.\.venv\Scripts\Activate.ps1
# For macOS/Linux:
source .venv/bin/activate
```

## Browser Automation Setup

Browser-use enables automated browser interaction for retrieving Twitter data. For detailed instructions and advanced configuration options, please refer to the official documentation at [docs.browser-use.com](https://docs.browser-use.com/introduction).

### Environment Variables
Create a `.env` file in your project root:
```bash
# Twitter credentials for browser automation
TWITTER_NAME=your_twitter_username
TWITTER_PASSWORD=your_twitter_password

# OpenAI configuration
OPENAI_API_KEY=your_openai_api_key
OPENAI_API_KEY_FOR_USAGE_AND_COSTS=your_openai_admin_key
OPENAI_PROJECT_ID=your_openai_project_id
```

Or set them in your shell:
```bash
# For macOS/Linux
export TWITTER_NAME=your_twitter_username
export TWITTER_PASSWORD=your_twitter_password
export OPENAI_API_KEY=your_openai_api_key
export OPENAI_API_KEY_FOR_USAGE_AND_COSTS=your_openai_admin_key
export OPENAI_PROJECT_ID=your_openai_project_id

# For Windows (Command Prompt)
set TWITTER_NAME=your_twitter_username
set TWITTER_PASSWORD=your_twitter_password
set OPENAI_API_KEY=your_openai_api_key
set OPENAI_API_KEY_FOR_USAGE_AND_COSTS=your_openai_admin_key
set OPENAI_PROJECT_ID=your_openai_project_id

# For Windows (PowerShell)
$env:TWITTER_NAME="your_twitter_username"
$env:TWITTER_PASSWORD="your_twitter_password"
$env:OPENAI_API_KEY="your_openai_api_key"
$env:OPENAI_API_KEY_FOR_USAGE_AND_COSTS="your_openai_admin_key"
$env:OPENAI_PROJECT_ID="your_openai_project_id"
```

### Browser Installation
```bash
# Install Python packages
uv pip install -r requirements.txt

# Install browser (Chromium recommended)
playwright install --with-deps chromium
```

### Configuration Setup
```bash
# Copy the template configuration file
cp config/llm.yaml.template config/llm.yaml

# Edit config/llm.yaml to set your API key and project ID:
# Replace "YOUR_API_KEY_HERE" with your actual OpenAI API key for browser-use
# Replace "YOUR_PROJECT_ID_HERE" with your actual OpenAI project ID
# Daily spending limits and model settings are configurable
# Cost tracking can be enabled/disabled as needed
```

## OpenAI Cost Management

The system includes built-in cost tracking and spending limits to prevent unexpected charges:

- **Daily Spending Limits**: Configurable via `config/llm.yaml` (default: $5.00/day)
- **Project-Specific Tracking**: Only tracks costs for your specific OpenAI project
- **Real-Time Monitoring**: Checks spending before each browser automation run
- **Automatic Prevention**: Stops execution if daily limit would be exceeded

### Required OpenAI Setup
1. Create an [OpenAI Admin Key](https://platform.openai.com/settings/organization/admin-keys) for cost tracking
2. Get your Project ID from the OpenAI dashboard
3. Set environment variables as shown above

### Cost Tracking Features
- Tracks actual API usage via OpenAI's Usage and Costs APIs
- Provides spending breakdowns by model and time period
- Syncs data before each execution to ensure accurate limits
- Supports project isolation to avoid tracking other OpenAI usage

### Usage Instructions for LLM
When using browser-use to collect Twitter data, provide these instructions to the LLM:

```
Task: Collect RealMir game guesses from Twitter replies.

Steps:
1. Navigate to Twitter.com
2. Search for @realmir_testnet
3. Find the latest tweet with hashtag #round{NUMBER}
4. Collect all replies containing guesses:
   - Look for patterns like:
     * "I commit to guess: [GUESS]"
     * "My guess: [GUESS]"
     * "Guessing: [GUESS]"
     * "Commit: [GUESS]"
   - If no pattern matches, use the full reply text

Return data in this format:
{
  "round": NUMBER,
  "guesses": [
    {"username": "user1", "guess": "guess text"},
    {"username": "user2", "guess": "guess text"}
  ]
}
```

### Example Usage with Cost Tracking
```bash
# Set required environment variables
export OPENAI_PROJECT_ID="proj_your_project_id_here"
export OPENAI_API_KEY_FOR_USAGE_AND_COSTS="your_admin_key_here"
export TWITTER_NAME="your_twitter_username"
export TWITTER_PASSWORD="your_twitter_password"

# Run Twitter data extraction with automatic cost tracking
python browser-use/twitter_data_fetcher.py --round 1 --target-time "20250523_133057EST"

# Example output:
# ✅ OpenAI usage tracker initialized
# 💰 Daily spending check for project proj_eQM5yuxSlkAmAQIf7mEpL00m:
#    Current: $2.45
#    Limit: $5.00
#    Remaining: $2.55
# 🔄 Syncing latest usage data for project proj_eQM5yuxSlkAmAQIf7mEpL00m...
# 🚀 Starting Twitter data extraction session: twitter_round_1_20250125_143022
# ... browser automation runs ...
# ⏱️ Execution completed in 45.2 seconds
# 📊 Tracking execution costs...
# 💰 Cost tracking completed
```

## Running Tests
```bash
python -m unittest discover tests
```

## Installing Dependencies

The `requirements.txt` file contains different groups of dependencies:

- **Core dependencies**: Always installed by default
  ```bash
  pip install -r requirements.txt
  ```

- **Development dependencies**: For Jupyter notebooks and development tools
  ```bash
  # Edit requirements.txt to uncomment development dependencies
  # Then run:
  pip install -r requirements.txt
  ```

- **Testing dependencies**: Required for running tests
  ```bash
  # Already included when installing requirements.txt
  ```

- **Optional dependencies**: For specific features
  ```bash
  # Edit requirements.txt to uncomment optional dependencies
  # Then run:
  pip install -r requirements.txt
  ```

## Pull Request Process
1. Create a new branch for your feature or bugfix
2. Make your changes
3. Run tests to ensure everything works
4. Commit your changes
5. Push your branch to GitHub
6. Create a pull request
7. Wait for review and merge

## Development Guidelines

- Follow the SOLID principles outlined in the user rules
- Create tests for new features after scoping them out
- Update documentation when changing user interfaces
- Consider using appropriate design patterns (Strategy, Decorator, Observer, Singleton, Facade)
- Follow the "worse is better" philosophy: prioritize simplicity and correctness
- Use git flow methodology for branch management 