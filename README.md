# Cliptions

Predict how an AI Agent will caption upcoming frames from live video streams. Players compete for cryptocurrency rewards based on prediction accuracy.

### Index
- [Gameplay](#gameplay)
- [Key Rules](#key-rules)
- [Key Features](#key-features)
- [Example Round](#example-round)
- [Getting Started](#getting-started)
  - [Download the CLI Tool](#download-the-cli-tool)
  - [Installation](#installation)
  - [Verify Installation](#verify-installation)
- [CLI Tools](#cli-tools)
  - [Commitment Generation](#commitment-generation)
  - [Score Calculation](#score-calculation)
  - [Payout Processing](#payout-processing)
  - [Commitment Verification](#commitment-verification)
  - [Advanced Usage](#advanced-usage)
- [Score and Payout Calculation](#score-and-payout-calculation)
  - [Ranking Process](#ranking-process)
  - [Payout Distribution](#payout-distribution)
  - [Basic Scoring](#basic-scoring-no-ties)
  - [Handling Ties](#handling-ties)
- [CLIP Embedder](#clip-embedder)
- [Contributing](#contributing)

### Gameplay
1. The Agent announces a new round on Twitter with a target frame
2. Players submit predictions in two steps:
   - First, reply to announcement with a commitment hash of their prediction
   - Later, reveal their actual prediction and salt to verify the commitment
3. When that moment arrives and the frame is revealed, each prediction is compared using CLIP ([OpenAI's vision-language AI model](https://github.com/openai/CLIP))
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

## Getting Started

### Download the CLI Tool

Cliptions provides a unified CLI tool that works on Windows, macOS, and Linux. Download the appropriate version for your operating system:

#### Option 1: Download from GitHub Releases (Recommended)

1. Go to the [GitHub Releases page](https://github.com/grasslandnetwork/cliptions/releases)
2. Find the latest release (the version number will be different, e.g., `v1.2.3`)
3. Download the appropriate file for your OS:
   - **Windows**: `cliptions-windows-v1.2.3.zip`
   - **macOS**: `cliptions-macos-v1.2.3.tar.gz`
   - **Linux**: `cliptions-linux-v1.2.3.tar.gz`

#### Option 2: Build from Source

If you prefer to build from source:

```bash
# Clone the repository
git clone https://github.com/grasslandnetwork/cliptions.git
cd cliptions

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the CLI tool
cargo build --release

# The binary will be available at target/release/cliptions
```

### Installation

#### Windows
1. Extract the downloaded `cliptions-windows-v1.2.3.zip`
2. Move `cliptions.exe` to a directory in your PATH (e.g., `C:\Windows\System32\` or create a new directory and add it to PATH)
3. Open Command Prompt or PowerShell and test: `cliptions --help`

#### macOS
```bash
# Extract the archive
tar -xzf cliptions-macos-v1.2.3.tar.gz

# Move to a directory in your PATH
sudo mv cliptions /usr/local/bin/

# Test the installation
cliptions --help
```

#### Linux
```bash
# Extract the archive
tar -xzf cliptions-linux-v1.2.3.tar.gz

# Move to a directory in your PATH
sudo mv cliptions /usr/local/bin/

# Test the installation
cliptions --help
```

### Verify Installation

After installation, verify that the CLI tool is working:

```bash
cliptions --help
```

You should see output showing the available subcommands and options.

## CLI Tools

Cliptions provides a unified CLI tool with subcommands for all game operations. All tools use real CLIP models by default for accurate similarity calculations.

### Commitment Generation

Generate secure commitment hashes for your predictions:

```bash
# Basic commitment generation
cliptions generate-commitment "Cat sanctuary with woman wearing snoopy sweater" --salt "random_secret_123"

# Verbose output with details
cliptions generate-commitment "My prediction" --salt "mysalt" --verbose

# Save to custom location
cliptions generate-commitment "My prediction" --salt "mysalt" --save-to /path/to/commitments.json

# Don't save locally (for scripts)
cliptions generate-commitment "My prediction" --salt "mysalt" --no-save
```

**Example Output:**
```
Commitment: b30bc27636a63a2c9ce07b9b24e39161e64e975399df2c773c4240b924735ed4
Success: Commitment data saved to /Users/username/.cliptions/commitments.json
```

**Features:**
- **Automatic saving**: By default, commitments are saved to `~/.cliptions/commitments.json`
- **Append mode**: New commitments are added to existing ones, not overwritten
- **Multiple formats**: Output in text, JSON, or CSV format
- **Batch processing**: Process multiple commitments from a JSON file

### Commitment Collection

Collect commitment replies from a specific tweet:

```bash
# Basic commitment collection
cliptions collect-commitments --tweet-id "1234567890123456789"

# Verbose output with detailed information
cliptions collect-commitments --tweet-id "1234567890123456789" --verbose

# Limit results per page
cliptions collect-commitments --tweet-id "1234567890123456789" --max-results 50

# Use custom config file
cliptions collect-commitments --tweet-id "1234567890123456789" --config config/custom.yaml
```

**Example Output:**
```
✅ Loaded config from: config/llm.yaml
✅ Search complete!
Total replies found: 3

--- Reply 1 ---
🐦 Tweet ID: 1234567890123456789
👤 Author ID: 9876543210987654321
📅 Created: 2024-01-15 14:30:00 UTC
💬 Text: My commitment hash: abc123def456...
🔗 URL: https://twitter.com/user/status/1234567890123456789
```

**Features:**
- **Twitter API integration**: Uses Twitter API v2 for reliable data collection
- **Configurable limits**: Control maximum results per page
- **Verbose mode**: Detailed output with metrics and conversation IDs
- **Error handling**: Comprehensive error messages for API issues

### Target Frame Posting

Post target frame images as replies to commitment tweets:

```bash
# Basic target frame posting
cliptions post-target-frame --reply-to "1234567890123456789" --image "rounds/round2/target.jpg" --round 3 --target-time 2

# Verbose output with detailed information
cliptions post-target-frame --reply-to "1234567890123456789" --image "rounds/round2/target.jpg" --round 3 --target-time 2 --verbose

# Use custom config file
cliptions post-target-frame --reply-to "1234567890123456789" --image "rounds/round2/target.jpg" --round 3 --target-time 2 --config config/custom.yaml
```

**Example Output:**
```
✅ Loaded config from: config/llm.yaml
✅ Target frame posted successfully!
Tweet ID: 9876543210987654321
URL: https://twitter.com/i/status/9876543210987654321
Reply to: 1234567890123456789
Round: 3
Target time: 2025-04-01 | 16:30:57 | EST
```

**Features:**
- **Image attachment**: Posts target frame images as replies to commitment tweets
- **Automatic formatting**: Generates proper tweet text with #revealsopen hashtag
- **Time calculation**: Automatically calculates target time from hours parameter
- **Error handling**: Comprehensive error messages for API and file issues
- **Configurable**: Support for different Twitter accounts via config files

### Score Calculation

Calculate similarity scores and rankings for a round:

```bash
# Basic score calculation with CLIP
./target/release/calculate_scores target.jpg 100.0 "ocean waves" "mountain sunset" "city skyline"

# Save results to JSON file
./target/release/calculate_scores --output json --output-file results.json target.jpg 100.0 "guess1" "guess2"

# Detailed similarity breakdown
./target/release/calculate_scores --detailed --verbose target.jpg 100.0 "prediction1" "prediction2"
```

### Payout Processing

Process payouts for completed rounds:

```bash
# Process single round
./target/release/process_payouts round1 --prize-pool 100.0

# Process all rounds with batch mode
./target/release/process_payouts --all

# Save payout results with error handling
./target/release/process_payouts --all --continue-on-error --output csv --output-file payouts.csv
```

### Commitment Verification

Verify the integrity of player commitments:

```bash
# Verify single round
./target/release/verify_commitments round1

# Batch verify all rounds
./target/release/verify_commitments --all --verbose

# Strict mode - fail on any invalid commitment
./target/release/verify_commitments --all --strict --output json --output-file verification.json
```

### Advanced Usage

All CLI tools support advanced features for production use:

```bash
# Use custom CLIP model
./target/release/calculate_scores --clip-model models/custom-clip target.jpg 100.0 "guess1"

# Load configuration from YAML
./target/release/process_payouts --config config.yaml --all

# Testing mode with MockEmbedder
./target/release/calculate_scores --use-mock target.jpg 100.0 "test1" "test2"

# Multiple output formats
./target/release/verify_commitments round1 --output table  # Default
./target/release/verify_commitments round1 --output json
./target/release/verify_commitments round1 --output csv

# Quiet mode for scripts
./target/release/generate_commitment "My prediction" --salt "mysalt" --quiet
```

**Common Options:**
- `--verbose` - Detailed progress information
- `--no-color` - Disable colored output for scripts
- `--output-file <path>` - Save results to file
- `--config <path>` - Load YAML configuration
- `--continue-on-error` - Continue batch processing on errors

### Getting Help

Each CLI tool provides comprehensive built-in documentation with examples and detailed option descriptions:

```bash
# Get help for any command
./target/release/generate_commitment --help
./target/release/calculate_scores --help
./target/release/process_payouts --help
./target/release/verify_commitments --help
```

The built-in help includes:
- **Usage syntax** with required and optional parameters
- **Real-world examples** for common use cases
- **Complete option reference** with descriptions and defaults
- **Configuration guidance** for YAML files and advanced features

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

## Contributing

We welcome contributions! For detailed setup instructions, development guidelines, and advanced configuration options, please see [CONTRIBUTING.md](CONTRIBUTING.md).

### Quick Start
1. Clone the repository
2. Install dependencies: `pip install -r requirements.txt`
3. Build CLI tools: `cargo build --release --no-default-features`
4. Run tests: `python -m unittest discover tests`
5. Create a pull request
