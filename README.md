# realmir

Predict how an AI Agent will caption upcoming frames from live video streams. Players compete for cryptocurrency rewards based on prediction accuracy.

### Index
- [Gameplay](#gameplay)
- [Key Rules](#key-rules)
- [Key Features](#key-features)
- [Example Round](#example-round)
- [Commitment Hash Generation](#commitment-hash-generation)
  - [Usage](#usage)
  - [Common Errors](#common-errors)
- [Score and Payout Calculation](#score-and-payout-calculation)
  - [Ranking Process](#ranking-process)
  - [Payout Distribution](#payout-distribution)
  - [Basic Scoring](#basic-scoring-no-ties)
  - [Handling Ties](#handling-ties)
- [CLIP Embedder](#clip-embedder)
- [Contributing](#contributing)
  - [Support the Network](#support-the-network)
  - [Development Setup](#development-setup)
  - [Running Tests](#running-tests)
  - [Pull Request Process](#pull-request-process)

### Gameplay
1. The Agent announces a new round on Twitter with a target frame
2. Players submit predictions in two steps:
   - First, reply to announcement with a commitment hash of their prediction
   - Later, reveal their actual prediction and salt to verify the commitment
3. When that moment arrives and the frame is revealed, each prediction is compared using CLIP
4. Players are ranked by how well their predictions matched CLIP's understanding
5. The prize pool is distributed based on rankings, with better predictions earning larger shares

### Key Rules
- **Submission Deadline:** All predictions must be submitted **before** the target timestamp. Late submissions are disqualified.
- **Commitment Format:** Predictions must be submitted as **hash commitments** along with the player's wallet address.
- **Reveal Phase:** After the target frame is posted, players must publicly reveal their plaintext guess and salt.
- **No Edited Tweets:** Edited commitment tweets are **disqualified** and **no refunds** are issued for fees paid.

### Key Features
- **Timestamp Predictions:** Guess how an AI Agent will interpret a specific future video frame.
- **AI-Powered:** Uses OpenAI's CLIP model for objective scoring.
- **Web3 Integration:** Decentralized gameplay and prize distribution.
- **Crypto Rewards:** Prize pools paid out based on prediction accuracy.
- **Transparent:** All calculations and rankings are verifiable.

### Example Round
1. Agent tweets "@round2 #targetframe20250223_133057EST from live stream of a cat sanctuary"
2. Players participate via Twitter:
   - Submit commitment hash as reply to announcement
   - After frame reveal, reply with prediction and salt
   - Example prediction: "Cat shelter with caretakers"
3. After target frame is revealed and players share their predictions, CLIP calculates similarity scores
4. Players are ranked by score
5. Prize pool is distributed according to rankings

## Commitment Hash Generation
Players must generate a hash commitment for their prediction.

### Usage
Run the script using the following format:
```bash
python3 generate_commitment.py "Your predicted caption here" --salt "your-salt-value-here"
```

**Example:**  
```bash
python3 generate_commitment.py "Cat sanctuary with woman wearing snoopy sweater" --salt "random_secret_123"
```

This will output:
```
Commitment: f7b7889b520e01e8a2e915e8e9124bc299c6f584c0e0dd255a7e38fe8ec35747
```

### Common Errors
- **Missing Quotes:** Always use **quotes** around both the **caption** and **salt**.
- **Argument Order:** The **caption** must come **first**, followed by the `--salt` argument.
- **PowerShell Users:** Ensure you use **double quotes** to avoid argument parsing issues.

## Score and Payout Calculation
The system calculates payouts based on similarity rankings between guesses and the target image.

### Ranking Process
1. Calculate CLIP embeddings for the target image and each guess.
2. Calculate cosine similarity between the target and each guess.
3. Rank guesses by similarity (highest to lowest).

### Payout Distribution
The payout system uses a position-based scoring method that:
- Distributes the entire prize pool.
- Rewards higher ranks with larger shares.
- Handles ties fairly.

#### Basic Scoring (No Ties)
For n players, each position's score is calculated as:
```
position_score = (n - position) / sum(1..n)
```

Example for 3 players:
- Denominator = 1 + 2 + 3 = 6
- 1st place: 3/6 = 0.50 (50% of pool)
- 2nd place: 2/6 ≈ 0.33 (33% of pool)
- 3rd place: 1/6 ≈ 0.17 (17% of pool)

#### Handling Ties
When multiple guesses have equal similarity scores:
1. Group tied positions together.
2. Calculate combined points for tied positions.
3. Split points equally among tied guesses.

Example with 5 players and ties:
```
Similarities:
Player1: 0.9 (tied for 1st/2nd)
Player2: 0.9 (tied for 1st/2nd)
Player3: 0.7
Player4: 0.5 (tied for 4th/5th)
Player5: 0.5 (tied for 4th/5th)

Groups:
[Player1, Player2]   - Split points for 1st/2nd
[Player3]           - Gets points for 3rd
[Player4, Player5]   - Split points for 4th/5th
```

## CLIP Embedder
The CLIP embedder generates embeddings for images and text using OpenAI's CLIP model. It can be used from the command line and accepts input via stdin.

### Installation
```bash
pip install -r requirements.txt
```

### Usage

#### Generate Image Embeddings
```bash
echo '{"image": "'$(cat image.jpg | base64)'"}' | python clip_embedder.py --mode image
```

#### Generate Text Embeddings
```bash
echo '{"text": "a photo of a dog"}' | python clip_embedder.py --mode text
```

### Output Format
```json
{
    "embedding": [0.1, 0.2, ...],
    "shape": [512]
}
```

## Contributing

### Support the Network 
Browser-use enables automated browser interaction for retrieving Twitter data. For detailed instructions and advanced configuration options, please refer to the official documentation at [docs.browser-use.com](https://docs.browser-use.com/introduction).

##### Setup
1. **Environment Variables**
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

2. **Python Environment Setup**
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

3. **Install Dependencies**
   ```bash
   # Install Python packages
   uv pip install -r requirements.txt
   
   # Install browser (Chromium recommended)
   playwright install --with-deps chromium
   ```

4. **Configuration Setup**
   ```bash
   # Copy the template configuration file
   cp config/llm.yaml.template config/llm.yaml
   
   # Edit config/llm.yaml to set your API key and project ID:
   # Replace "YOUR_API_KEY_HERE" with your actual OpenAI API key for browser-use
   # Replace "YOUR_PROJECT_ID_HERE" with your actual OpenAI project ID
   # Daily spending limits and model settings are configurable
   # Cost tracking can be enabled/disabled as needed
   ```

##### OpenAI Cost Management
The system includes built-in cost tracking and spending limits to prevent unexpected charges:

- **Daily Spending Limits**: Configurable via `config/llm.yaml` (default: $5.00/day)
- **Project-Specific Tracking**: Only tracks costs for your specific OpenAI project
- **Real-Time Monitoring**: Checks spending before each browser automation run
- **Automatic Prevention**: Stops execution if daily limit would be exceeded

**Required OpenAI Setup:**
1. Create an [OpenAI Admin Key](https://platform.openai.com/settings/organization/admin-keys) for cost tracking
2. Get your Project ID from the OpenAI dashboard
3. Set environment variables as shown above

**Cost Tracking Features:**
- Tracks actual API usage via OpenAI's Usage and Costs APIs
- Provides spending breakdowns by model and time period
- Syncs data before each execution to ensure accurate limits
- Supports project isolation to avoid tracking other OpenAI usage

##### Usage Instructions for LLM
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

##### Example Usage with Cost Tracking
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

### Development Setup
1. Clone the repository.
2. Create a new branch for your feature or bugfix.
3. Install dependencies using:
```bash
pip install -r requirements.txt
```
4. Set up pre-commit hooks.

### Installing Dependencies

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

### Running Tests
```bash
python -m unittest discover tests
```

### Pull Request Process
1. Create a new branch for your feature or bugfix.
2. Make your changes.
3. Run tests to ensure everything works.
4. Commit your changes.
5. Push your branch to GitHub.
6. Create a pull request.
7. Wait for review and merge.
