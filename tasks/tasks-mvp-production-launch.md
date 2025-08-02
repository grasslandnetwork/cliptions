# MVP Production Launch Tasks (Vertical Slice Plan)

## Overview
This task list outlines the refactoring and implementation plan for the Cliptions MVP, following a Vertical Slice architecture. Each task represents a self-contained, testable feature that must pass CI before moving to the next.

**Important Notes:**
- **Rust 2021**: No `mod.rs` files needed - modules are automatically discovered from directory structure
- **Code Reuse**: Always check `binaries_architecture.md` for existing function signatures before implementing new code
- **Vertical Slices**: Each subcommand should be a complete, testable feature slice

---

### Foundational Setup (v0.6.0)
**Status**: [x] Completed
**Priority**: Critical
**Description**: Establish the new single-binary architecture and ensure the baseline compiles in CI.

**Tasks**:
- [x] Create `src/main.rs` as the single entry point with `clap` for subcommand routing.
- [x] Create the `src/actions/` directory to house the vertical slices.
- [x] Modify `Cargo.toml`: comment out all existing `[[bin]]` targets and add a new one for `name = "cliptions", path = "src/main.rs"`.
- [x] Define the shared JSON data models for commitments and reveals.
- [x] Update `Cargo.toml` to version `0.6.0`.
- [x] Commit the changes and create a git tag `v0.6.0`.
- [x] **Verify that an empty `main` function compiles successfully in GitHub Actions.**
- [x] Update this task list to mark all tasks as completed.

---

### Slice 0: Validator Opens Block (v0.6.0)
**Status**: [x] In Progress
**Priority**: Critical
**Description**: Implement the `open-block` subcommand.

**Tasks**:
- [x] Create the `src/actions/new_block.rs` module.
- [x] **Check `binaries_architecture.md` for existing `twitter_post` function signatures before implementing.**
- [x] Implement logic to post the #commitmentsopen tweet with block hashtag (e.g., #block8).
- [x] Generate tweet text with block instructions and appropriate hashtags.
- [x] Wire it up as the `new-block` subcommand.
- [x] Implement Twitter posting functionality for block opening (real API)
- [x] Test posting #commitmentsopen tweet to Twitter
- [ ] Save the posted tweet ID for later use in collect-commitments
- [ ] Ensure config file can be swapped for different Twitter accounts/roles
- [ ] **Create tests for the open-block subcommand by moving appropriate tests from the old binary.**
- [ ] Update `README.md` to document the new `cliptions open-block` command.
- [ ] Update this task list to mark all tasks as completed.

---

### Slice 1: Miner Generates Commitment (v0.6.1)
**Status**: [x] Completed
**Priority**: High
**Description**: Implement the `generate-commitment` subcommand.

**Tasks**:
- [x] Create the `src/actions/generate_commitment.rs` module.
- [x] Move logic from the old `generate_commitment` binary, including saving the guess/salt locally.
- [x] Wire it up as the `generate-commitment` subcommand in `main.rs`.
- [x] Update `README.md` to document the new `cliptions generate-commitment` command.
- [x] Update `Cargo.toml` to version `0.6.1`.
- [x] Commit the changes and create a git tag `v0.6.1`.
- [x] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [x] **Create tests for the generate-commitment subcommand by moving appropriate tests from the old binary.**
- [x] Update this task list to mark all tasks as completed.

---

### Slice 2: Validator Collects Commitments (v0.6.2)
**Status**: [ ] Completed
**Priority**: High
**Description**: Implement the `collect-commitments` subcommand.

**Tasks**:
- [x] Create the `src/actions/collect_commitments.rs` module.
- [x] **Check `binaries_architecture.md` for existing `twitter_search_replies` function signatures before implementing.**
- [x] Implement logic to extract commitment replies from a specific tweet, sourcing from `twitter_search_replies.rs`.
- [x] Wire it up as the `collect-commitments` subcommand.
- [x] **Create tests for the collect-commitments subcommand by moving appropriate tests from the old binary.**
- [x] **Added save functionality with role-based directory structure** (~/.cliptions/validator/collected_commitments.json)
- [x] **Added multiple output formats** (text, json, csv)
- [x] **Added commitment parsing** from Twitter reply text
- [x] **Added append mode** to preserve existing collected commitments
- [x] Update README.md with new subcommand documentation
- [x] Update Cargo.toml version to 0.6.2
- [ ] **Create integration tests for the complete validator workflow**
- [x] **Test with real Twitter data and verify commitment parsing accuracy**

---

### Slice 3: Validator Posts Target Frame (v0.6.3)
**Status**: [x] Completed
**Priority**: High
**Description**: Implement the `post-target-frame` subcommand.

**Tasks**:
- [x] Create the `src/actions/post_target_frame.rs` module.
- [x] **Check `binaries_architecture.md` for existing `twitter_post` function signatures before implementing.**
- [x] Implement logic to post target frame image as a reply to the #commitmentsopen tweet.
- [x] Generate tweet text with #revealsopen hashtag and reveal instructions.
- [x] Wire it up as the `post-target-frame` subcommand.
- [x] Implement Twitter posting functionality for target frame (real API)
- [x] Test posting target frame as reply to #commitmentsopen tweet
- [x] Save the posted tweet ID for later use in collect-reveals
- [x] Ensure config file can be swapped for different Twitter accounts/roles
- [x] **Create tests for the post-target-frame subcommand by moving appropriate tests from the old binary.**
- [x] Update `README.md` to document the new `cliptions post-target-frame` command.
- [x] Update `Cargo.toml` to version `0.6.3`.
- [x] Commit the changes and create a git tag `v0.6.3`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [x] Update this task list to mark all tasks as completed.

---

### Slice 4: Validator Collects Reveals (v0.6.4)
**Status**: [ ] Partially Completed
**Priority**: High
**Description**: Implement the `collect-reveals` subcommand.

**Tasks**:
- [x] Create the `src/actions/collect_reveals.rs` module.
- [x] **Check `binaries_architecture.md` for existing `twitter_search_replies` function signatures before implementing.**
- [x] Implement logic to extract reveal replies from a specific tweet.
- [x] Wire it up as the `collect-reveals` subcommand.
- [x] Implement Twitter reading for reveal collection (real API)
- [ ] Test reveal collection with real Twitter data
- [ ] Ensure config file can be swapped for different Twitter accounts/roles
- [ ] **Create tests for the collect-reveals subcommand by moving appropriate tests from the old binary.**
- [ ] Update `README.md` to document the new `cliptions collect-reveals` command.
- [x] Update `Cargo.toml` to version `0.6.4`.
- [ ] Commit the changes and create a git tag `v0.6.4`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [ ] Update this task list to mark all tasks as completed.

---

### Slice 5: Validator Verifies Commitments (v0.6.5)
**Status**: [ ] Partially Completed
**Priority**: High
**Description**: Implement the `verify-commitments` subcommand.

**Tasks**:
- [x] Create the `src/actions/verify_commitments.rs` module.
- [x] **Check `binaries_architecture.md` for existing `verify_commitments` function signatures before implementing.**
- [ ] Move logic from the old `verify_commitments` binary.
- [x] Wire it up as the `verify-commitments` subcommand.
- [x] Add verification logic using real collected data
- [x] Test verification with real Twitter data
- [ ] Ensure config file can be swapped for different Twitter accounts/roles
- [ ] **Create tests for the verify-commitments subcommand by moving appropriate tests from the old binary.**
- [x] Update `README.md` to document the new `cliptions verify-commitments` command.
- [x] Update `Cargo.toml` to version `0.6.5`.
- [x] Commit the changes and create a git tag `v0.6.5`.
- [x] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [ ] Update this task list to mark all tasks as completed.

---

### Slice 6: Calculate Scores & Payouts (v0.6.6)
**Status**: [x] Completed
**Priority**: High
**Description**: Implement the `calculate-scores` subcommand.

**Tasks**:
- [x] Create the `src/actions/calculate_scores.rs` module.
- [x] **Check `binaries_architecture.md` for existing `calculate_scores` function signatures before implementing.**
- [x] Implement logic to take a list of **verified** participants as input.
- [x] For each participant, calculate their similarity score and determine their final payout amount.
- [x] Ensure the total payout distributed does not exceed the prize pool.
- [x] Wire it up as the `calculate-scores` subcommand, which should output a clear list of participants and their corresponding payouts.
- [x] Calculate scores and payouts using real participant data from Twitter
- [x] Test payout calculation with real-world data
- [ ] Ensure config file can be swapped for different Twitter accounts/roles
- [ ] **Create tests for the calculate-scores subcommand by moving appropriate tests from the old binary.**
- [x] Update `README.md` to document the new `cliptions calculate-scores` command.
- [x] Update `Cargo.toml` to version `0.6.6`.
- [ ] Commit the changes and create a git tag `v0.6.6`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [ ] Update this task list to mark all tasks as completed.

---
### Slice 7: Terminology Update - Round to Block (v0.7.0)
**Status**: [x] Completed
**Priority**: Critical
**Description**: Replace all "round" terminology with "block" throughout the codebase to align with blockchain concepts for future development.

**Note**: All Python (*.py) files are being ignored during this terminology update as they will be deleted as part of the migration to the new Rust-only architecture.

**Phase 1: Directory and File Structure** ✅ **COMPLETED**
- [x] Rename `rounds/` directory to `blocks/`
- [x] Rename subdirectories: `round0/` → `block0/`, `round1/` → `block1/`, `round2/` → `block2/`
- [x] Rename data files: `rounds.json` → `blocks.json`, `test_round.json` → `test_block.json`, `test_rounds.json` → `test_blocks.json`

**Phase 2: Module and File Names** ✅ **COMPLETED**
- [x] Rename `src/round_engine/` → `src/block_engine/`
- [x] Rename `src/round_processor.rs` → `src/block_processor.rs`
- [x] Rename `tests/round_engine_integration.rs` → `tests/block_engine_integration.rs`
- [x] Update all module imports and references

**Phase 3: Rust Types and Structures** ✅ **COMPLETED**
- [x] Replace `Round<T>` struct → `Block<T>`
- [x] Replace `RoundData` → `BlockData`
- [x] Replace `RoundConfig` → `BlockConfig` 
- [x] Replace `RoundStatus` → `BlockStatus`
- [x] Replace `RoundError` → `BlockError`
- [x] Replace `RoundProcessor` → `BlockProcessor`

**Phase 4: Variables and Identifiers** ✅ **COMPLETED**
- [x] Replace `round_id` → `block_num`
- [x] Replace `round_tweet_id` → `block_tweet_id`
- [x] Replace `round_number` → `block_num`
- [x] Replace `round_version` → `block_version`
- [x] Update CLI argument names and help text

**Phase 5: CLI Subcommands** ✅ **COMPLETED**
- [x] Rename `open-round` → `open-block` (when implemented)
- [x] Update all CLI help text and documentation

**Phase 6: Social Media and User-Facing Text** ✅ **COMPLETED**
- [x] Update hashtags: `#round8` → `#block8`
- [x] Update tweet templates: "Round X is now live" → "Block X is now live"
- [x] Update all user-facing messages and prompts

**Phase 7: Documentation and Comments** ✅ **COMPLETED**
- [x] Update README.md with new terminology
- [x] Update all code comments referencing "round"
- [x] Update task documentation and PRD files
- [x] Update CONTRIBUTING.md and other docs

**Phase 8: Testing and Validation** ✅ **COMPLETED**
- [x] Update all test cases with new terminology
- [x] Verify all tests pass after terminology changes
- [x] Update `Cargo.toml` to version `0.7.0`
- [x] Commit changes and create git tag `v0.7.0`
- [x] **Verify the new tag triggers and passes all checks in GitHub Actions**
- [x] Update this task list to mark all tasks as completed

---

### Slice 8: End-to-End Testing & Validation (v0.7.1)
**Status**: [ ] In Progress
**Priority**: Critical
**Description**: Conduct comprehensive end-to-end testing of the full block lifecycle with live Twitter interactions.

**Tasks**:
- [x] Configure Twitter API credentials for live testing (support multiple config files for different roles/slices)
- [x] Start a real block and document the process
- [ ] Test each slice with actual Twitter API calls (post, read, reply, etc.)
- [ ] Integration test: Ensure all slices work together in a real block
- [ ] Monitor block progress (commitments, fees, reveals, verification, payouts)
- [ ] Complete a block and verify all data is correct
- [ ] Monitor, debug, and document any issues found during live testing
- [ ] Document the full block lifecycle with real data
- [ ] Update documentation to reflect real-world usage and troubleshooting

---

### Slice 9: Centralized File Path Management (v0.7.2)
**Status**: [ ] Not Started
**Priority**: Critical
**Description**: Refactor the application to use a centralized path management system for all configuration and data files, storing them in `~/.cliptions`. This will ensure the application can be run from any directory.

**Tasks**:
- [ ] **Add `dirs` Crate**: 
    - [ ] Run `cargo add dirs` to add the dependency to `Cargo.toml`. This crate provides cross-platform access to user directories (like the home directory).
- [ ] **Create `PathManager` in `src/config.rs`**:
    - [ ] Define a new public struct called `PathManager`.
    - [ ] Implement a `new()` function for `PathManager`.
    - [ ] In `new()`, get the user's home directory using `dirs::home_dir()`.
    - [ ] Construct the base path `~/.cliptions`.
    - [ ] Create the `~/.cliptions` directory and its subdirectories (`data`, `miner`, `validator`) if they don't exist. Use `std::fs::create_dir_all()`.
- [ ] **Implement Path Getter Functions**:
    - [ ] Add public methods to `PathManager` to return `PathBuf` for each required file:
        - `get_config_path()` -> `~/.cliptions/config.yaml`
        - `get_blocks_path()` -> `~/.cliptions/data/blocks.json`
        - `get_twitter_posts_path()` -> `~/.cliptions/data/twitter_posts.json`
        - `get_scoring_versions_path()` -> `~/.cliptions/data/scoring_versions.json`
        - `get_validator_tweet_cache_path()` -> `~/.cliptions/validator/validator_tweet_cache.json`
        - `get_miner_commitments_path()` -> `~/.cliptions/miner/commitments.json`
        - `get_validator_collected_commitments_path()` -> `~/.cliptions/validator/collected_commitments.json`
        - `get_validator_collected_reveals_path()` -> `~/.cliptions/validator/collected_reveals.json`
- [ ] **Integrate `PathManager` with `ConfigManager`**:
    - [ ] Modify `ConfigManager::new()` and `ConfigManager::with_path()` to use `PathManager` to determine the config file path. The default `new()` should use `PathManager::new().get_config_path()`.
- [ ] **Refactor Codebase to Use `PathManager`**:
    - [ ] Search the codebase for hardcoded paths (e.g., `"data/blocks.json"`, `"config/config.yaml"`).
    - [ ] Replace all hardcoded paths with calls to the appropriate getter functions in `PathManager`.
- [ ] **Handle Missing `config.yaml`**:
    - [ ] Ensure the application provides a clear error message if `~/.cliptions/config.yaml` is not found, instructing the user to copy the template.
- [ ] **Update Documentation**:
    - [ ] Modify `README.md` and any other relevant documentation to reflect the new file locations under `~/.cliptions`.
- [ ] **Testing**:
    - [ ] Write unit tests for `PathManager` to ensure paths are generated correctly and directories are created.
- [ ] Update `Cargo.toml` to version `0.7.2`.
- [ ] Commit the changes and create a git tag `v0.7.2`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [ ] Update this task list to mark all tasks as completed.

---

### Slice 10: Fix Block Version JSON Type Inconsistency (v0.7.3)
**Status**: [ ] Not Started
**Priority**: Critical
**Description**: Fix the JSON parsing error in calculate-scores caused by inconsistent block_version field types.

**Root Cause Analysis**:
- The `BlockData` struct in `src/types.rs` defines `block_version: i32` (integer)
- The `save_to_blocks_json` function in `src/actions/verify_commitments.rs` writes `"block_version": "1"` (string)
- This creates inconsistency in `data/blocks.json` where some blocks have string values and others have integer values
- The `calculate-scores` command fails with: `Error: Json(Error("invalid type: string \"1\", expected i32", line: 4, column: 24))`

**Tasks**:
- [ ] **Fix the JSON generation code**:
    - [ ] In `src/actions/verify_commitments.rs`, line 356, change `"block_version": "1"` to `"block_version": 1`
    - [ ] Verify this matches the `BlockData` struct definition in `src/types.rs`
- [ ] **Fix existing JSON data**:
    - [ ] Update `data/blocks.json` to ensure all `block_version` fields are integers, not strings
    - [ ] Test that the file can be parsed without errors
- [ ] **Add validation**:
    - [ ] Add a test to ensure `block_version` is always written as an integer
    - [ ] Consider adding a validation function to check JSON consistency
- [ ] **Test the fix**:
    - [ ] Run `./target/release/cliptions calculate-scores -b 3 -p 0.015` to verify it works
    - [ ] Test with other block numbers to ensure consistency
- [ ] **Update documentation**:
    - [ ] Add a note about the block_version field type requirement
    - [ ] Update any relevant documentation about JSON schema
- [ ] Update `Cargo.toml` to version `0.7.3`.
- [ ] Commit the changes and create a git tag `v0.7.3`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [ ] Update this task list to mark all tasks as completed.

---

### Finalization (v0.7.3)
**Status**: [ ] Not Started
**Priority**: Medium
**Description**: Finalize the project for release.

**Tasks**:
- [ ] Test the full, end-to-end user workflow using the single `cliptions` CLI tool.
- [ ] Update `README.md` with detailed, platform-specific (Windows, macOS, Linux) instructions for compiling the application and using each subcommand.
- [ ] **Post-MVP Refactoring**: Review the completed `actions` and identify shared logic. Consolidate business models/rules into a `src/domain/` directory and shared technical services (like the Twitter client) into a `src/infra/` directory to improve long-term maintainability.
- [ ] Update this task list to mark all tasks as completed.
