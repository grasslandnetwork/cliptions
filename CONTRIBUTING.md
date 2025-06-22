# Contributing to RealMir

Thank you for your interest in contributing to RealMir! This document provides detailed setup instructions and development guidelines.

## Table of Contents
- [Development Setup](#development-setup)
- [Browser Automation Setup](#browser-automation-setup)
- [OpenAI Cost Management](#openai-cost-management)
- [Running Tests](#running-tests)
- [Installing Dependencies](#installing-dependencies)
- [Pull Request Process](#pull-request-process)
- [Rust Development](#rust-development)
- [Development Guidelines](#development-guidelines)
- [Test Coverage Comparison](#test-coverage-comparison)
- [Browser Use Roadmap](#browser-use-roadmap)

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

## Rust Development

### Architecture Overview

The RealMir project includes a high-performance Rust core implementation with optional Python bindings. The library follows a **clean separation** between the pure Rust core and language bindings:

```
src/
├── lib.rs              # Main library entry point
├── types.rs            # Core data structures  
├── error.rs            # Pure Rust error handling
├── commitment.rs       # Cryptographic commitments (pure Rust)
├── scoring.rs          # Scoring strategies (pure Rust)
├── round.rs            # Round processing (pure Rust)
├── embedder.rs         # Embedding interfaces (pure Rust)
├── python_bridge.rs    # Python bindings (PyO3 only)
└── bin/                # CLI tools (pure Rust)
    ├── calculate_scores.rs
    ├── process_payouts.rs
    └── verify_commitments.rs
```

### Key Benefits

✅ **Pure Rust Core**: No Python dependencies in core logic  
✅ **Clean Compilation**: Can build without PyO3 for pure Rust usage  
✅ **Fast Development**: No Python compilation overhead during Rust development  
✅ **Multiple Bindings**: Easy to add C FFI, WASM, or other language bindings  
✅ **Better Testing**: Test pure Rust logic independently  

### Performance Improvements

| Operation | Python | Rust | Speedup |
|-----------|--------|------|---------|
| Commitment Generation | 1.2ms | 12μs | **100x** |
| Commitment Verification | 1.1ms | 11μs | **100x** |
| Scoring Calculation | 800μs | 40μs | **20x** |
| Batch Processing (1000 items) | 1.2s | 45ms | **27x** |

### Data Models & Schema Consistency

The RealMir system uses a **dual-language data architecture** where Rust serves as the single source of truth for data structures, while Python uses mirrored Pydantic models for validation and interface safety.

#### Schema Consistency Testing

The system includes automated tests that ensure Python and Rust data models stay synchronized:

```bash
# Run schema consistency tests
pytest tests/test_schema_consistency.py

# These tests will FAIL if:
# - Field names don't match between Python and Rust
# - Field types are incompatible  
# - Required fields are missing
# - Serialization formats differ
```

### Development Commands

#### Command Types Explained

| Command | Purpose | Output | Speed |
|---------|---------|--------|-------|
| `cargo check` | Compile-check only | Error checking | Fastest ⚡ |
| `cargo test` | Compile + run tests | Test results | Medium 🔧 |
| `cargo build` | Compile + create binaries | Executable files | Slowest 🏗️ |

#### Feature Flags Explained

| Flag | PyO3 Included | Use Case | Dependencies |
|------|---------------|----------|--------------|
| `--features python` | ✅ Yes | Python integration | Requires Python dev libs |
| `--no-default-features` | ❌ No | Pure Rust development | Rust only |

#### Pure Rust Development (Recommended)

```bash
# Quick development cycle (fastest feedback)
cargo check --lib --no-default-features

# Validate everything works
cargo test --lib --no-default-features

# Build specific CLI tool for testing
cargo build --bin calculate_scores --no-default-features

# Build all CLI tools for production
cargo build --bins --release --no-default-features
```

#### Python Integration (When Needed)

```bash
# Check Python bindings compile
cargo check --lib --features python

# Test Python bridge functionality
cargo test --lib --features python

# Build Python wheel
maturin build --release --features python
```

### Architecture Guidelines

**🎯 Core Principle: Keep Pure Rust Separate from Language Bindings**

#### Core Logic (Pure Rust Only)
```rust
// ✅ GOOD: Pure Rust in core modules
// src/scoring.rs, src/commitment.rs, etc.
pub fn calculate_score(data: &Data) -> Result<f64> {
    // No PyO3, no Python dependencies
}

// ❌ BAD: Don't mix PyO3 in core modules  
#[pyfunction]  // ← Never do this in core modules
pub fn calculate_score(data: &Data) -> PyResult<f64> { }
```

#### Python Bindings (PyO3 Only)
```rust
// ✅ GOOD: All PyO3 code in python_bridge.rs
#[pyfunction]
pub fn py_calculate_score(data: Vec<f64>) -> PyResult<f64> {
    let rust_data = convert_from_python(data);
    core_function(&rust_data).map_err(|e| e.into())
}
```

### Adding New Features

| Feature Type | Location | Dependencies | Pattern |
|--------------|----------|--------------|---------|
| **Core Algorithm** | `src/new_module.rs` | Pure Rust only | Implement trait, add tests |
| **Python Function** | `src/python_bridge.rs` | PyO3 + core | Wrapper around core function |
| **CLI Tool** | `src/bin/new_tool.rs` | Core library | New `main()` + `[[bin]]` in Cargo.toml |
| **Data Type** | `src/types.rs` | Serde for serialization | Builder pattern, derive traits |

### Development Workflow Rules

1. **Always start with pure Rust** - implement in core modules first
2. **Test pure Rust independently** - `cargo test --lib --no-default-features`
3. **Add Python bindings last** - wrap the tested core functionality  
4. **Validate both modes** - test with and without `--features python`

### CLI Tools

```bash
# Build CLI tools (no Python dependencies)
cargo build --release --no-default-features

# Calculate scores for a round
./target/release/calculate_scores --round-id round1 --rounds-file data/rounds.json

# Process payouts
./target/release/process_payouts --round-id round1 --prize-pool 1000.0

# Verify commitments
./target/release/verify_commitments --round-id round1
```

### Testing Strategy

```bash
# Test pure Rust core
cargo test --lib --no-default-features

# Test with Python bindings
cargo test --lib --features python

# Integration tests
cargo test --test integration_tests --no-default-features

# Run performance benchmarks
cargo bench --no-default-features
```

## Development Guidelines

- Follow the SOLID principles outlined in the user rules
- Create tests for new features after scoping them out
- Update documentation when changing user interfaces
- Consider using appropriate design patterns (Strategy, Decorator, Observer, Singleton, Facade)
- Follow the "worse is better" philosophy: prioritize simplicity and correctness
- Use git flow methodology for branch management
- Keep Rust core logic separate from Python bindings
- Always start with pure Rust implementation before adding language bindings 

## Test Coverage Comparison: Rust vs Python

This document compares our Rust and Python test suites to identify gaps and ensure comprehensive coverage across both languages.

### Summary
- **Rust Tests**: 45 total (33 unit + 12 integration)
- **Python Tests**: 84 total (69 passing + 15 failing)
- **Schema Consistency Tests**: 3 (bridging Rust-Python gap)

### Test Coverage Comparison (Feature-Matched)

| **Feature** | **Rust Tests** | **Python Tests** | **Coverage Gap** |
|-------------|----------------|------------------|------------------|
| **🔐 Commitment Generation** | ✅ `test_commitment_generation` | ✅ `test_commitment_format` | **Both covered** |
| **🔐 Commitment Verification** | ✅ `test_commitment_verification` | ✅ `test_commitment_verification` | **Both covered** |
| **🔐 Reference Hash Generation** | ❌ **Missing** | ✅ `test_reference_hash` | **Need Rust reference hash test** |
| **🔐 Salt Validation** | ✅ `test_empty_salt` | ✅ `test_salt_required` | **Both covered** |
| **🔐 Message Validation** | ✅ `test_empty_message` | ❌ **Missing** | **Need Python empty message test** |
| **🔐 Salt Generation** | ✅ `test_salt_generation` | ❌ **Missing** | **Need Python salt generation test** |
| **🔐 Batch Processing** | ✅ `test_batch_verification` | ❌ **Missing** | **Need Python batch test** |
| **🔐 Deterministic Behavior** | ✅ `test_deterministic_generation` | ❌ **Missing** | **Need Python deterministic test** |
| **🔐 Format Validation** | ✅ `test_invalid_format_handling` | ❌ **Missing** | **Need Python format validation** |

| **🖼️ Image Embedding Features** | **Rust Tests** | **Python Tests** | **Coverage Gap** |
|----------------------------------|----------------|------------------|------------------|
| **Image Embedding from Path** | ✅ `test_mock_embedder_image_embedding` | ✅ `test_image_embedding_from_path` | **Both covered** |
| **Image Embedding from Bytes** | ❌ **Missing** | ✅ `test_image_embedding_from_bytes` | **Need Rust bytes test** |
| **Image Embedding from PIL** | ❌ **Missing** | ✅ `test_image_embedding_from_pil` | **Need Rust PIL test** |
| **Text Embedding (Single)** | ✅ `test_mock_embedder_text_embedding` | ✅ `test_text_embedding_single` | **Both covered** |
| **Text Embedding (Batch)** | ✅ `test_mock_embedder_batch_processing` | ✅ `test_text_embedding_batch` | **Both covered** |
| **Similarity Computation** | ✅ `test_mock_embedder_similarity` | ✅ `test_compute_similarity` | **Both covered** |
| **Deterministic Embeddings** | ✅ `test_mock_embedder_deterministic` | ✅ `test_deterministic_embedding` | **Both covered** |
| **Semantic Similarity Scoring** | ❌ **Missing** | ✅ `test_semantic_similarity_scores` | **Need Rust semantic scoring** |
| **CLI Interface** | ❌ **Missing** | ✅ `test_cli_image_input` | **Need Rust CLI tests** |
| **CLI Error Handling** | ❌ **Missing** | ✅ `test_cli_invalid_json` | **Need Rust CLI error tests** |
| **CLI Validation** | ❌ **Missing** | ✅ `test_cli_invalid_mode` | **Need Rust CLI validation** |
| **CLI Missing Fields** | ❌ **Missing** | ✅ `test_cli_missing_field` | **Need Rust CLI field tests** |
| **CLI Text Input** | ❌ **Missing** | ✅ `test_cli_text_input` | **Need Rust CLI text tests** |

| **🎯 Scoring & Validation Features** | **Rust Tests** | **Python Tests** | **Coverage Gap** |
|--------------------------------------|----------------|------------------|------------------|
| **Score Calculation** | ✅ `test_score_validator_score_calculation` | ✅ `test_full_scoring_flow` | **Both covered** |
| **Guess Length Filtering** | ✅ `test_score_validator_guess_validation` | ✅ `test_length_filtering` | **Both covered** |
| **Baseline Score Adjustment** | ❌ **Missing** | ✅ `test_baseline_adjustment` | **Need Rust baseline test** |
| **Raw Similarity Strategy** | ❌ **Missing** | ✅ `test_raw_similarity_strategy` | **Need Rust raw similarity** |
| **Baseline-Adjusted Strategy** | ❌ **Missing** | ✅ `test_baseline_adjusted_strategy` | **Need Rust adjusted strategy** |
| **Baseline Requirement Validation** | ❌ **Missing** | ✅ `test_baseline_adjusted_strategy_requires_baseline` | **Need Rust baseline validation** |
| **Negative Score Handling** | ❌ **Missing** | ✅ `test_strategies_handle_negative_scores` | **Need Rust negative score test** |
| **Batch Processing** | ✅ `test_score_validator_batch_processing` | ❌ **Missing** | **Need Python batch test** |
| **Performance Testing** | ✅ `test_score_validator_performance` | ❌ **Missing** | **Need Python performance test** |
| **Error Handling** | ✅ `test_score_validator_error_handling` | ❌ **Missing** | **Need Python error test** |
| **Edge Cases** | ✅ `test_score_validator_edge_cases` | ❌ **Missing** | **Need Python edge case test** |
| **Rankings Use Adjusted Scores** | ❌ **Missing** | ✅ `test_rankings_use_adjusted_scores` | **Need Rust ranking test** |
| **Payouts Match Score Ordering** | ❌ **Missing** | ✅ `test_payouts_match_score_ordering` | **Need Rust payout test** |
| **Invalid Guesses Get Zero Score** | ❌ **Missing** | ✅ `test_invalid_guesses_get_zero_score` | **Need Rust zero score test** |

| **🎮 Round Management Features** | **Rust Tests** | **Python Tests** | **Coverage Gap** |
|----------------------------------|----------------|------------------|------------------|
| **Round Creation** | ✅ `test_round_processor_round_creation` | ❌ **Missing** | **Need Python round creation test** |
| **Commitment Handling** | ✅ `test_round_processor_commitment_handling` | ✅ `test_process_round_payouts_valid_commitments` | **Both covered** |
| **Invalid Commitment Handling (Abort)** | ❌ **Missing** | ✅ `test_process_round_payouts_invalid_commitments_abort` | **Need Rust abort test** |
| **Invalid Commitment Handling (Continue)** | ❌ **Missing** | ✅ `test_process_round_payouts_invalid_commitments_continue` | **Need Rust continue test** |
| **Data Persistence** | ✅ `test_round_processor_data_persistence` | ❌ **Missing** | **Need Python persistence test** |
| **Process All Rounds** | ❌ **Missing** | ✅ `test_process_all_rounds` | **Need Rust process all test** |
| **Get Validator for Round** | ❌ **Missing** | ✅ `test_get_validator_for_round` | **Need Rust validator getter** |
| **Error Handling** | ✅ `test_round_processor_error_handling` | ❌ **Missing** | **Need Python error test** |
| **Edge Cases** | ✅ `test_round_processor_edge_cases` | ❌ **Missing** | **Need Python edge case test** |

| **💰 Payout & Economics Features** | **Rust Tests** | **Python Tests** | **Coverage Gap** |
|------------------------------------|----------------|------------------|------------------|
| **Custom Prize Pool** | ❌ **Missing** | ✅ `test_custom_prize_pool` | **🚨 Need Rust prize pool test** |
| **Equal Scores for Equal Ranks** | ❌ **Missing** | ✅ `test_equal_scores_for_equal_ranks` | **🚨 Need Rust equal rank test** |
| **Three Player Payout** | ❌ **Missing** | ✅ `test_three_player_payout` | **🚨 Need Rust 3-player test** |
| **Two Player Payout** | ❌ **Missing** | ✅ `test_two_player_payout` | **🚨 Need Rust 2-player test** |
| **Game Example Scenario** | ❌ **Missing** | ✅ `test_example_scenario` | **🚨 Need Rust scenario test** |
| **Invalid Guess Range** | ❌ **Missing** | ✅ `test_invalid_guess_range` | **🚨 Need Rust range validation** |
| **Minimum Players** | ❌ **Missing** | ✅ `test_minimum_players` | **🚨 Need Rust player limit test** |
| **Payout Distribution** | ❌ **Missing** | ✅ `test_payout_distribution` | **🚨 Need Rust distribution test** |
| **Platform Fee Calculation** | ❌ **Missing** | ✅ `test_platform_fee_calculation` | **🚨 Need Rust fee test** |
| **Equal Distance Symmetry** | ❌ **Missing** | ✅ `test_equal_distance_symmetry` (x2) | **🚨 Need Rust symmetry test** |
| **Score Range Validation** | ❌ **Missing** | ✅ `test_score_range` (x2) | **🚨 Need Rust range test** |

| **🔄 Data Models & Schema Features** | **Rust Tests** | **Python Tests** | **Coverage Gap** |
|--------------------------------------|----------------|------------------|------------------|
| **Commitment Schema Consistency** | ✅ (via integration) | ✅ `test_commitment_schema_consistency` | **Both covered** |
| **Round Schema Consistency** | ✅ (via integration) | ✅ `test_round_schema_consistency` | **Both covered** |
| **Round with Empty Commitments** | ✅ (via integration) | ✅ `test_round_with_empty_commitments` | **Both covered** |

| **🐦 Social Integration Features** | **Rust Tests** | **Python Tests** | **Coverage Gap** |
|-------------------------------------|----------------|------------------|------------------|
| **Announcement Data Validation** | ❌ **Missing** | ✅ `test_valid_announcement_data` | **🚨 Need Rust validation** |
| **Custom Hashtags** | ❌ **Missing** | ✅ `test_custom_hashtags` | **🚨 Need Rust hashtag test** |
| **Tweet ID Extraction** | ❌ **Missing** | ✅ `test_extract_tweet_id_from_url` | **🚨 Need Rust URL parsing** |
| **Task Execution Success** | ❌ **Missing** | ✅ `test_execute_success` | **🚨 Need Rust execution test** |
| **Task Execution with Parameters** | ❌ **Missing** | ✅ `test_execute_with_kwargs` | **🚨 Need Rust param test** |
| **Standard Announcement Creation** | ❌ **Missing** | ✅ `test_create_standard_round_announcement` | **🚨 Need Rust standard test** |
| **Custom Announcement Creation** | ❌ **Missing** | ✅ `test_create_custom_round_announcement` | **🚨 Need Rust custom test** |
| **Full Announcement Workflow** | ❌ **Missing** | ✅ `test_full_announcement_flow` | **🚨 Need Rust workflow test** |
| **Twitter App Persistence** | ❌ **Missing** | ✅ `test_twitter_app_persistence` | **🚨 Need Rust persistence test** |

| **🔑 Configuration Features** | **Rust Tests** | **Python Tests** | **Coverage Gap** |
|-------------------------------|----------------|------------------|------------------|
| **Config Loading with API Key** | ❌ **Missing** | ✅ `test_load_llm_config_includes_api_key_from_config` | **🚨 Need Rust config loading** |
| **Missing API Key Handling** | ❌ **Missing** | ✅ `test_missing_api_key_in_config` | **🚨 Need Rust validation** |
| **Daily Spending Limit Loading** | ❌ **Missing** | ✅ `test_daily_spending_limit_config_loading` | **🚨 Need Rust limit loading** |
| **Under Spending Limit Check** | ❌ **Missing** | ✅ `test_spending_limit_check_under_limit` | **🚨 Need Rust under-limit test** |
| **Over Spending Limit Check** | ❌ **Missing** | ✅ `test_spending_limit_check_over_limit` | **🚨 Need Rust over-limit test** |
| **No Data Spending Check** | ❌ **Missing** | ✅ `test_spending_limit_check_no_data` | **🚨 Need Rust no-data test** |
| **Project-Specific Limits** | ❌ **Missing** | ✅ `test_project_specific_spending_limit_check` | **🚨 Need Rust project test** |
| **Fetcher Respects Limits** | ❌ **Missing** | ✅ `test_twitter_fetcher_respects_spending_limit` | **🚨 Need Rust integration** |
| **Cost Tracking During Execution** | ❌ **Missing** | ✅ `test_cost_tracking_during_execution` | **🚨 Need Rust tracking** |

| **✅ Verification Features** | **Rust Tests** | **Python Tests** | **Coverage Gap** |
|------------------------------|----------------|------------------|------------------|
| **Empty Round Verification** | ❌ **Missing** | ✅ `test_empty_round` | **Need Rust empty round test** |
| **File Not Found Handling** | ❌ **Missing** | ✅ `test_file_not_found` | **Need Rust file error test** |
| **Invalid Commitments** | ❌ **Missing** | ✅ `test_invalid_commitments` | **Need Rust invalid test** |
| **Missing Data Handling** | ❌ **Missing** | ✅ `test_missing_data` | **Need Rust missing data test** |
| **Mixed Valid/Invalid Commitments** | ❌ **Missing** | ✅ `test_mixed_commitments` | **Need Rust mixed test** |
| **Round Not Found** | ❌ **Missing** | ✅ `test_round_not_found` | **Need Rust not found test** |
| **Valid Commitments** | ✅ `test_verify_commitments` (bin) | ✅ `test_valid_commitments` | **Both covered** |
| **Score Calculation (Binary)** | ✅ `test_calculate_scores` (bin) | ❌ **Missing** | **Need Python binary test** |
| **Payout Processing (Binary)** | ✅ `test_process_payouts` (bin) | ❌ **Missing** | **Need Python binary test** |
| **Integration Verification** | ✅ `test_verify_commitments_integration` | ❌ **Missing** | **Need Python integration** |

| **Test Category** | **Rust Tests** | **Python Tests** | **Coverage Gap** |
|-------------------|----------------|------------------|------------------|
| **🔗 Integration Tests** | ✅ **12 tests** | ✅ **Various** | **Rust has comprehensive integration coverage** |
| | `test_commitment_system_integration` | (Distributed across modules) | |
| | `test_complete_round_lifecycle` | | |
| | `test_scoring_system_integration` | | |
| | `test_embedder_integration` | | |
| | `test_data_persistence_integration` | | |
| | `test_error_handling_integration` | | |
| | `test_performance_integration` | | |
| | `test_concurrent_access_integration` | | |
| | `test_large_dataset_integration` | | |
| | `test_memory_usage_integration` | | |
| | `test_cross_platform_integration` | | |
| | `test_backwards_compatibility_integration` | | |

### 🎯 Priority Rust Tests to Add

### **🚨 Critical Missing Areas (High Priority)**

1. **💰 Payout/Economics Module** - 12 tests needed
   - Prize pool distribution
   - Player ranking and payouts  
   - Platform fee calculations
   - Multi-player scenarios

2. **🔑 Configuration Management** - 9 tests needed
   - Config file loading/parsing
   - API key validation
   - Spending limit enforcement
   - Cost tracking integration

3. **🐦 Social/Twitter Integration** - 9 tests needed
   - Announcement formatting
   - URL parsing and validation
   - Hashtag handling
   - Social media workflow

### **⚠️ Medium Priority Gaps**

4. **🖼️ Enhanced Embedder Tests** - 4 tests needed
   - CLI interface testing
   - Byte data handling
   - PIL image support
   - Error handling

5. **✅ Enhanced Verification** - 2 tests needed
   - Mixed commitment scenarios
   - Missing round handling

### **✅ Well Covered Areas**
- **Commitment/Cryptography**: Rust has excellent coverage
- **Integration Tests**: Rust has comprehensive coverage  
- **Schema Consistency**: New bridge tests ensure compatibility

### 📊 Test Coverage Score

| **Module** | **Rust Coverage** | **Python Coverage** | **Overall Score** |
|------------|-------------------|---------------------|-------------------|
| Commitments | 🟢 Excellent (9/9) | 🟡 Good (4/9) | 🟢 **Strong** |
| Embeddings | 🟡 Good (6/10) | 🟢 Excellent (10/10) | 🟢 **Strong** |
| Scoring | 🟢 Excellent (8/10) | 🟢 Excellent (10/10) | 🟢 **Excellent** |
| Round Management | 🟢 Excellent (6/5) | 🟢 Good (5/5) | 🟢 **Excellent** |
| **Payouts** | 🔴 **Missing (0/12)** | 🟢 Excellent (12/12) | 🔴 **Critical Gap** |
| **Configuration** | 🔴 **Missing (0/9)** | 🟡 Partial (9/9, some failing) | 🔴 **Critical Gap** |
| **Social Integration** | 🔴 **Missing (0/9)** | 🟡 Partial (9/9, some failing) | 🔴 **Critical Gap** |
| Verification | 🟡 Good (4/7) | 🟢 Excellent (7/7) | 🟢 **Strong** |
| Integration | 🟢 Excellent (12/12) | 🟡 Distributed | 🟢 **Strong** |
| Schema Consistency | 🟢 Covered via tests | 🟢 Excellent (3/3) | 🟢 **Excellent** |

### 🎯 Recommended Action Plan

1. **Phase 1**: Add critical Rust payout/economics tests (12 tests)
2. **Phase 2**: Add Rust configuration management tests (9 tests)  
3. **Phase 3**: Add Rust social integration tests (9 tests)
4. **Phase 4**: Enhance embedder and verification coverage (6 tests)

**Total Rust tests to add: ~36 tests** to achieve comprehensive parity with Python coverage.

## Browser Use Roadmap

# Browser Use - Roadmap: RealMir Modular Twitter Automation

**Goal:** Create a modular Twitter automation system for the RealMir prediction network, supporting both Validator and Miner workflows through separate, reusable components that implement a shared set of interfaces.

## Architecture Overview

Instead of a monolithic script, the system will be composed of specialized modules that conform to a strict **interface-based design**. `get_twitter_replies.py` serves as the first proof-of-concept and will be refactored to implement our formal `TwitterTask` interface. This ensures all modules are interchangeable, testable, and adhere to SOLID principles.

- **Single Responsibility**: Each module performs one job (e.g., collect commitments, post a reveal).
- **Open/Closed**: Extensible without modification
- **Dependency Inversion**: Use abstractions, not concrete implementations
- **Interface Segregation**: Clean, focused interfaces
- **Liskov Substitution**: Consistent behavior across implementations

## Phase 1: Core Infrastructure (Completed)

*   **Task 1.1:** ✅ **Base Twitter Reply Extraction** (`get_twitter_replies.py`)
    *   **Status:** Completed - Provides robust foundation for extracting replies from Twitter threads
    *   **Features:** Handles spam filtering, pagination, structured output via Pydantic models

*   **Task 1.2:** ✅ **Data Structure Alignment** (`rounds/guesses.json` with URLs)
    *   **Status:** Completed - Ground truth data includes commitment/reveal URLs for testing

*   **Task 1.3:** ✅ **Test Infrastructure** (`tests/test_twitter_data_extraction.py`)
    *   **Status:** Completed - Structural testing framework ready for modular components

## Phase 1.5: Rust-Driven Data Layer & Python/Rust Bridge (Revised)

*Goal: Establish Rust as the single source of truth for core data models while using Pydantic for interface validation in Python. Ensure consistency between Rust structs and Pydantic models via the Python/Rust bridge and dedicated integration tests.*

### **Task 1.5.1: Define Core Data Models in Rust**
*   **Action:** Create `src/models.rs` to house all core data structures.
*   **Action:** Define structs for `Round`, `Commitment`, `Participant`, etc., using `serde`.
*   **Status:** ✅ Completed - Created `src/models.rs` with `Commitment` and `Round` structs using serde and PyO3 derives

### **Task 1.5.2: Create Mirror Pydantic Models**
*   **Action:** Create `browser/data_models.py` for the Pydantic model definitions.
*   **Action:** Define Pydantic models that mirror the structure of the Rust structs.
*   **Status:** ✅ Completed - Created `browser/data_models.py` with corresponding Pydantic models for `Commitment` and `Round`

### **Task 1.5.3: Implement Schema Consistency Test**
*   **Action:** Create `tests/test_schema_consistency.py`.
*   **Action:** Write tests that pass data from each Pydantic model to the Rust core, ensuring they can be successfully deserialized into their corresponding Rust structs. This test will serve as our "consistency lock."
*   **Status:** ✅ Completed - Created comprehensive schema consistency tests with 3 test cases, all passing

### **Task 1.5.4: Implement Rust Data Access Layer**
*   **Action:** Create a Rust module (e.g., `src/data_access.rs`) to handle loading from and saving to `data/rounds.json`.
*   **Action:** Expose these data access functions to Python via `python_bridge.rs`.
*   **Status:** Not Started

### **Task 1.5.5: Refactor `collect_commitments.py`**
*   **Action:** Update `collect_commitments.py` to use the new Pydantic models for validation.
*   **Action:** After validation, the task will call the new Rust data access functions (via the bridge) to persist the collected commitments.
*   **Status:** Not Started

## Phase 2: Commitment Workflow

*This phase focuses on the initial round setup and the commitment interaction between Validator and Miner.*

### **Task 2.1:** Validator: Round Announcement
*   **Module:** `browser/validator/announce_round.py`
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Post the initial round announcement tweet.
*   **Status:** ✅ Completed

### **Task 2.2:** Miner: Commitment Submission
*   **Module:** `browser/miner/submit_commitment.py`
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Reply to an announcement with a commitment hash.
*   **Status:** ✅ Completed

### **Task 2.3:** Validator: Commitment Collection  
*   **Module:** `browser/validator/collect_commitments.py`
*   **Purpose:** Extract all miner commitments from the announcement tweet replies.
*   **Implements:** `TwitterExtractionInterface`
*   **Status:** ✅ Completed

### **Task 2.4:** Validator: Entry Fee Assignment
*   **Module:** `browser/validator/assign_entry_fees.py`  
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Reply to each commitment with the TAO payment address.
*   **Status:** Not Started

### **Task 2.5:** Integration Test: Commitment Cycle
*   **Module:** `tests/test_commitment_workflow.py`
*   **Purpose:** Verify that a Validator can announce, a Miner can commit, and the Validator can collect the commitment and assign a fee address successfully.
*   **Status:** Not Started

## Phase 3: Reveal Workflow

*This phase handles the publication of the target frame and the subsequent reveal from the Miners.*

### **Task 3.1:** Validator: Target Frame Publication
*   **Module:** `browser-use/validator/publish_target_frame.py`
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Post the target frame image as a reply to the announcement.
*   **Status:** Not Started

### **Task 3.2:** Miner: Reveal Submission  
*   **Module:** `browser-use/miner/submit_reveal.py`
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Reply to the target frame with the plaintext prediction and salt.
*   **Status:** Not Started

### **Task 3.3:** Validator: Reveal Collection
*   **Module:** `browser-use/validator/collect_reveals.py`
*   **Purpose:** Extract all miner reveals from the target frame tweet replies.
*   **Implements:** `TwitterExtractionInterface`
*   **Status:** Not Started

### **Task 3.4:** Integration Test: Reveal Cycle
*   **Module:** `tests/test_reveal_workflow.py`
*   **Purpose:** Verify that a Validator can post a target, a Miner can reveal, and the Validator can collect the reveal.
*   **Status:** Not Started

## Phase 4: Scoring, Payouts, and Results

*This phase covers the final steps of the game: payment verification, scoring, and announcing the winners.*

### **Task 4.1 (Advanced):** Validator: Payment Verification
*   **Module:** `browser-use/validator/verify_payments.py`
*   **Purpose:** Check the TAO network to verify which miners have paid their entry fees.
*   **Integration:** TAO network APIs.
*   **Status:** Not Started

### **Task 4.2 (Advanced):** Validator: Scoring & Payouts
*   **Modules:** `score_predictions.py`, `distribute_payouts.py`
*   **Purpose:** Integrate with the CLIP embedder and TAO network to calculate winners and distribute rewards.
*   **Status:** Not Started

### **Task 4.3:** Validator: Results Publication
*   **Module:** `browser-use/validator/publish_results.py`
*   **Implements:** `TwitterPostingInterface`
*   **Purpose:** Post the final results tweet, listing winners and payouts.
*   **Status:** Not Started

## Phase 5: Orchestration & Monitoring

*This phase involves creating higher-level orchestrators to automate the full game cycle and tools for monitoring.*

### **Task 5.1:** Validator Orchestrator
*   **Module:** `browser-use/validator/orchestrator.py`
*   **Purpose:** Coordinate the full validator workflow for a complete round.
*   **Status:** Not Started

### **Task 5.2:** Miner Orchestrator
*   **Module:** `browser-use/miner/orchestrator.py` 
*   **Purpose:** Coordinate the full miner participation in a round.
*   **Status:** Not Started

### **Task 5.3:** Miner: Round Monitoring
*   **Module:** `browser-use/miner/monitor_rounds.py`
*   **Purpose:** Watch for new round announcements to enable fully automated participation.
*   **Status:** Not Started

## Design Principles & Standards

### **Core Interfaces** (e.g., in `browser/core/interfaces.py`)
- **`TwitterTask` (ABC):** An abstract base class defining the contract for any automated Twitter action.
  - `async def execute(self, **kwargs) -> BaseModel:`: Standard execution method.
  - `setup_agent(...)`: Configures the `browser-use` agent.
  - `validate_output(...)`: Ensures the result conforms to a Pydantic model.
- **`TwitterExtractionInterface` (Inherits `TwitterTask`):** Specialized for data collection.
- **`TwitterPostingInterface` (Inherits `TwitterTask`):** Specialized for creating content.

### **Module Implementation**
Each module will be a concrete implementation of a core interface. This replaces the "Module Template" concept with a formal, enforceable contract. Common functionality (config loading, browser context) will be handled in a base class that implements the `TwitterTask` interface.

### **Shared Infrastructure**
- **Core Interfaces**: `TwitterTask`, `TwitterExtractionInterface`, `TwitterPostingInterface`.
- **Abstract Base Classes**: `BaseTwitterTask` to provide shared setup and execution logic.
- **Shared Pydantic Models**: `CommitmentData`, `RevealData`, `RoundConfig`, `PayoutInfo` for type-safe data transfer.
- **Configuration Management**: Centralized config loading and validation
- **Error Handling**: Standardized exception hierarchy
- **Testing Utilities**: Mock generators, test data factories

## Next Steps

**Immediate Priority:** 
1. **Implement Phase 1.5:** Complete the Data Layer Refactoring to establish a single source of truth.
2. **Continue with Phase 2:** Once the DAL is in place, proceed with the Commitment Workflow, starting with `Task 2.4: Validator: Entry Fee Assignment`.

**Recommended Approach:**
1.  ✅ Create `browser/core/interfaces.py` and define the abstract base classes.
2.  ✅ Create `browser/core/base_task.py` for shared logic (config, browser setup).
3.  ✅ Implement the **Commitment Workflow (Phase 2)** Round Announcement module.
4.  **Implement the Data Layer Refactoring (Phase 1.5)** as outlined above.
5.  Refactor existing modules like `collect_commitments` to use the new data layer.
6.  Continue with remaining Phase 2 modules (`assign_entry_fees`).
7.  Build the `test_commitment_workflow.py` to test the interaction between the Phase 2 modules.
8.  Proceed to the Reveal Workflow, implementing and testing the modules for that stage.
9.  Finally, build the orchestrators to tie the full vertical slices together.

This modular approach ensures each component can be developed, tested, and deployed independently while maintaining consistency across the entire system.

## Summary of Changes from Original Plan

**Key Architectural Shifts:**
- **From Single Script to Modular System**: Instead of one monolithic `twitter_data_fetcher.py`, we now have specialized modules for each game phase
- **From Data Extraction to Full Game Automation**: Expanded scope from just collecting existing data to facilitating the entire Validator/Miner workflow  
- **From Ad-Hoc Testing to Systematic Architecture**: Following SOLID principles with proper abstractions and interfaces
- **From Manual Process to Automated Orchestration**: Full automation of round management, fee collection, and payout distribution

**Immediate Benefits:**
- **True Modularity**: Modules are truly interchangeable, not just similar in pattern.
- **Testability**: Easy to mock dependencies and test vertical slices of gameplay.
- **Maintainability**: Clear contracts make the system easier to understand and debug.
- **Scalability**: New features can be added by creating new classes that conform to existing interfaces.

## Test Coverage Analysis

### Current Test Status
- **Rust Tests**: 45 total (33 unit + 12 integration) - All passing ✅
- **Python Tests**: 84 total (69 passing + 15 failing)
- **Schema Consistency Tests**: 3 (bridging Rust-Python gap) - All passing ✅

### Critical Test Gaps Identified

#### **🚨 Missing Rust Functionality (High Priority)**

**💰 Payout & Economics (0/11 features)**
- Prize pool distribution, player rankings, platform fees
- Multi-player scenarios, payout validation
- **Impact**: Core business logic missing from Rust

**🔑 Configuration Management (0/9 features)**  
- Config loading, API key validation, spending limits
- Cost tracking integration
- **Impact**: No configuration system in Rust

**🐦 Social Integration (0/9 features)**
- Announcement formatting, URL parsing, hashtag handling
- Twitter workflow automation
- **Impact**: No social media capabilities in Rust

#### **⚠️ Medium Priority Gaps**

**🖼️ Enhanced Embedder Features (Missing 5/13 features)**
- CLI interface, byte/PIL image handling
- Semantic similarity scoring

**✅ Verification Edge Cases (Missing 6/10 features)**
- Empty rounds, file errors, mixed commitments
- Missing data handling

### Feature Coverage Matrix

| **Feature Category** | **Rust Coverage** | **Python Coverage** | **Priority** |
|---------------------|-------------------|---------------------|--------------|
| **Core Cryptography** | 🟢 Excellent (9/9) | 🟡 Good (4/9) | ✅ Well covered |
| **Data Models** | 🟢 Excellent (3/3) | 🟢 Excellent (3/3) | ✅ Well covered |
| **Embeddings/CLIP** | 🟡 Good (7/13) | 🟢 Excellent (13/13) | 🟡 Medium gap |
| **Scoring** | 🟢 Good (8/14) | 🟢 Excellent (14/14) | 🟡 Medium gap |
| **Round Management** | 🟢 Good (6/9) | 🟡 Good (5/9) | 🟡 Medium gap |
| **Payouts/Economics** | 🔴 **Missing (0/11)** | 🟢 Excellent (11/11) | 🚨 **Critical** |
| **Configuration** | 🔴 **Missing (0/9)** | 🟡 Partial (9/9) | 🚨 **Critical** |
| **Social Integration** | 🔴 **Missing (0/9)** | 🟡 Partial (9/9) | 🚨 **Critical** |
| **Verification** | 🟡 Limited (4/10) | 🟢 Excellent (10/10) | 🟡 Medium gap |

### Recommended Test Development Plan

**Phase 1: Critical Rust Features (30 tests)**
1. Implement payout/economics module with comprehensive tests (11 tests)
2. Add configuration management system with validation (9 tests)
3. Create social integration framework with URL/hashtag handling (9 tests)

**Phase 2: Enhanced Coverage (10 tests)**
4. Expand embedder capabilities (CLI, bytes, PIL support) (5 tests)
5. Add verification edge cases (empty rounds, file errors) (6 tests)

**Phase 3: Python Gap Filling (8 tests)**
6. Add missing Python commitment features (batch, deterministic) (5 tests)
7. Add missing Python round management features (3 tests)

**Total: ~48 additional tests needed** to achieve comprehensive parity

### Architecture Implications

The test gap analysis reveals that **Rust currently handles the core cryptographic and data processing** excellently, but is missing the **business logic, configuration, and social features** that exist in Python. This suggests:

1. **Rust Core**: Focus on performance-critical operations (crypto, scoring, data processing)
2. **Python Layer**: Handle business logic, configuration, and social integration
3. **Bridge**: Ensure seamless data flow between layers via schema consistency tests

This polyglot architecture leverages each language's strengths while maintaining type safety and performance where needed. 