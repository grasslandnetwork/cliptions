# PRD: MVP Production-Ready Cliptions

## 1. Introduction/Overview

This document outlines the Minimum Viable Product (MVP) requirements for launching Cliptions into production tomorrow. The MVP focuses on core gameplay functionality with a simplified architecture that removes complex features like fee management, web servers, and advanced automation in favor of reliable, manual workflows that can be executed immediately.

**Key Changes from Original Plan:**
- **Architecture**: Consolidating multiple binaries into a single CLI tool with modular subcommands, following a **Vertical Slice** design pattern where each subcommand represents a complete, testable feature.
- **Payment System**: Reverting from BASE API back to $TAO/Bittensor payment addresses
- **Fee Management**: Skipped for MVP to focus on core gameplay
- **Web Server**: Removed - no fee collection interface needed
- **Automation**: Limited - manual validator workflows with CLI tools
- **Payout Limit**: Total payout does not exceed the prize pool to control costs

## 2. Goals

- **Goal 1: Production Launch**: Deploy a working Cliptions system that can run a complete block tomorrow
- **Goal 2: Core Gameplay**: Enable miners to participate and validators to manage blocks end-to-end
- **Goal 3: Cost Control**: Total payout is limited by the prize pool.
- **Goal 4: Manual Reliability**: Use proven CLI tools and manual workflows over complex automation
- **Goal 5: Simple Distribution**: Source code compilation with clear instructions

## 3. User Stories

- **As a Validator**, I want to use the `cliptions collect-commitments` subcommand to get all commitments from Twitter replies and store them locally, so I can track all participants
- **As a Validator**, I want to use the `cliptions reply-with-fees` subcommand to reply to each commitment with a $TAO payment address, so miners know where to send their fees
- **As a Validator**, I want to use the `cliptions collect-reveals` subcommand to get all reveals from Twitter replies and store them locally, so I can calculate scores
- **As a Validator**, I want to use a `cliptions verify-commitments` subcommand to check that each miner's revealed guess and salt match their original commitment hash, so I only calculate scores for valid participants.
- **As a Validator**, I want to use the `cliptions calculate-scores` subcommand to calculate scores and payouts for verified participants, so I can distribute rewards fairly.
- **As a Miner**, I want to use the `cliptions generate-commitment` subcommand to generate commitment hashes and save my plaintext guess securely in a local JSON file, so I have a record for the reveal stage.
- **As a Miner**, I want to run the `cliptions calculate-scores` subcommand to verify that the validator calculated scores correctly, so I can trust the results.
- **As a Developer**, I want to compile and run the single `cliptions` binary from source, so I can participate immediately.

## 4. Functional Requirements

### FR1: Validator Commitment Collection
The validator **must** be able to use a `collect-commitments` subcommand to collect all commitment replies from a specific tweet and store them locally in JSON format.

### FR2: Validator Fee Address Replies
The validator **must** be able to use a `reply-with-fees` subcommand to reply to each commitment tweet with a unique $TAO payment address for fee collection.

### FR3: Validator Reveal Collection
The validator **must** be able to use a `collect-reveals` subcommand to collect all reveal replies from a specific tweet and store them locally in JSON format.

### FR4: Score Calculation and Payouts
The validator **must** be able to use a `calculate-scores` subcommand to calculate similarity scores using CLIP embeddings and determine payouts for verified participants. The total payout must not exceed the prize pool.

### FR5: Miner Commitment Generation
Miners **must** be able to use a `generate-commitment` subcommand to generate commitment hashes and save their plaintext guess and salt to a local JSON file.

### FR6: Commitment Verification
Before calculating scores, the validator **must** use a `verify-commitments` subcommand to ensure that each miner's revealed guess and salt match their original commitment hash. Scoring should only be performed for valid commitments.

### FR7: Miner Score Verification
Miners **must** be able to run the `calculate-scores` subcommand to verify validator calculations independently.

### FR8: CLIP Model Distribution
The CLIP model **must** download automatically when the `calculate_scores` subcommand is first run, not during initial software installation.

### FR9: Local Data Storage
All commitment and reveal data **must** be stored in local JSON files for easy backup and verification.

### FR10: Source Code Distribution
The software **must** be distributed as source code with clear compilation instructions for major platforms.

### FR11: Single Unified Binary
All functionality **must** be accessible through a single `cliptions` binary using subcommands for each action (e.g., `cliptions generate-commitment`).

## 5. Non-Goals (Out of Scope)

- **Multiple Binaries**: The final application will be a single executable.
- **Fee Management**: No automated fee verification or collection
- **Web Server**: No web interface or server components
- **BASE API Integration**: Reverting to $TAO/Bittensor addresses only
- **Advanced Automation**: No automated state machine workflows
- **Multiple Block Management**: Single block focus only
- **User Authentication**: No user accounts or authentication
- **Analytics and Reporting**: No advanced metrics or dashboards
- **Mobile Support**: Desktop/CLI only
- **Advanced Scoring**: Use existing CLIP-based scoring only

## 6. Technical Considerations

### 6.1 Refactoring Plan & Vertical Slice Architecture
The existing binaries will be refactored into a single application following a **Vertical Slice** architecture. Each user story will be implemented as a distinct "slice," corresponding to a subcommand in the final CLI tool. This ensures that each feature is developed and tested end-to-end, from command-line input to final output.

- **`src/main.rs`**: Will serve as the main entry point, using `clap` to route to the appropriate vertical slice (subcommand).
- **`src/actions/`**: A new directory containing a module for each vertical slice. For example:
    - `features/generate_commitment/mod.rs`
    - `features/collect_commitments/mod.rs`
    - `features/verify_commitments/mod.rs`
- **Shared Infrastructure**: Common logic, such as Twitter API clients or data models, will be centralized in the `cliptions_core` library to be used across slices without coupling them.
- **Code Pruning & Selective Compilation**: To manage existing compilation errors, the refactoring will be incremental. The old binary files in `src/bin/` will **never be deleted** as part of this MVP refactor. Instead, their compilation targets will be removed from `Cargo.toml`. Only the new, single `cliptions` binary will be compiled. Logic from the old files will be moved into the new action modules one slice at a time. This ensures that only completed, integrated code is part of the build process, allowing us to get a green CI pipeline quickly.
- **Prototyping Guidelines**: Any new glue code or parsing logic required to create the action slices **must** follow the guidelines outlined in `@prototyping_in_rust.mdc`.

### 6.2 Data Storage Format
```json
{
  "block_num": "3",
  "commitments": [
    {
      "tweet_id": "123456789",
      "twitter_handle": "@user1",
      "commitment_hash": "abc123...",
      "wallet_address": "5F...",
      "timestamp": "2025-07-17T22:00:00Z"
    }
  ],
  "reveals": [
    {
      "tweet_id": "123456790",
      "twitter_handle": "@user1",
      "guess": "ocean waves crashing on rocks",
      "salt": "random_salt_123",
      "timestamp": "2025-07-17T22:00:00Z"
    }
  ]
}
```

### 6.3 CLIP Model Download
- Download triggered on first `calculate_scores.rs` execution
- Store in `models/clip-vit-base-patch32/` directory
- Fallback to MockEmbedder if download fails

## 7. Success Metrics

- **Metric 1**: At least 3 miners can complete a full block (commitment → reveal → verification) using the new `cliptions` CLI tool.
- **Metric 2**: Validator can manage one block end-to-end without critical errors using the new `cliptions` CLI tool.
- **Metric 3**: Score calculations are consistent between validator and miner verification.
- **Metric 4**: The `cliptions` binary can be compiled and run from source on major platforms (macOS, Linux, Windows).

## 8. Implementation Tasks

The implementation will proceed slice by slice. Each slice **must be integrated and pass CI in GitHub Actions** before work begins on the next slice. This ensures the codebase remains stable and shippable at all times.

### 8.1 Foundational Setup (Priority: High)
- [ ] Create `src/main.rs` as the single entry point with `clap` for subcommand routing.
- [ ] Create the `src/actions/` directory to house the vertical slices.
- [ ] Modify `Cargo.toml`: comment out all existing `[[bin]]` targets and add a new one for `name = "cliptions", path = "src/main.rs"`.
- [ ] Define the shared JSON data models for commitments and reveals.
- [ ] Update `Cargo.toml` to version `0.6.0`.
- [ ] Commit the changes and create a git tag `v0.6.0`.
- [ ] **Verify that an empty `main` function compiles successfully in GitHub Actions.**

### 8.2 Slice: Miner Generates Commitment (Priority: High)
- [ ] Create the `src/actions/generate_commitment.rs` module.
- [ ] Implement logic from the old `generate_commitment` binary, including saving the guess/salt locally.
- [ ] Wire it up as the `generate-commitment` subcommand in `main.rs`.
- [ ] Update `Cargo.toml` to version `0.6.1`.
- [ ] Commit the changes and create a git tag `v0.6.1`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**

### 8.3 Slice: Validator Collects Commitments (Priority: High)
- [ ] Create the `src/actions/collect_commitments.rs` module.
- [ ] Implement logic to extract commitment replies from a specific tweet, sourcing from `twitter_search_replies.rs`.
- [ ] Wire it up as the `collect-commitments` subcommand.
- [ ] Update `Cargo.toml` to version `0.6.2`.
- [ ] Commit the changes and create a git tag `v0.6.2`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**

### 8.4 Slice: Validator Replies with Fees (Priority: High)
- [ ] Create the `src/actions/reply_with_fees.rs` module.
- [ ] Implement logic to reply to each commitment with a unique $TAO address, sourcing from `twitter_post.rs`.
- [ ] Wire it up as the `reply-with-fees` subcommand.
- [ ] Update `Cargo.toml` to version `0.6.3`.
- [ ] Commit the changes and create a git tag `v0.6.3`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**

### 8.5 Slice: Validator Collects Reveals (Priority: High)
- [ ] Create the `src/actions/collect_reveals.rs` module.
- [ ] Implement logic to extract reveal replies from a specific tweet.
- [ ] Wire it up as the `collect-reveals` subcommand.
- [ ] Update `Cargo.toml` to version `0.6.4`.
- [ ] Commit the changes and create a git tag `v0.6.4`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**

### 8.6 Slice: Validator Verifies Commitments (Priority: High)
- [ ] Create the `src/actions/verify_commitments.rs` module.
- [ ] Implement logic from the old `verify_commitments` binary.
- [ ] Wire it up as the `verify-commitments` subcommand.
- [ ] Update `Cargo.toml` to version `0.6.5`.
- [ ] Commit the changes and create a git tag `v0.6.5`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**

### 8.7 Slice: Calculate Scores & Payouts (Priority: High)
- [ ] Create the `src/actions/calculate_scores.rs` module.
- [ ] Implement logic to take a list of **verified** participants as input.
- [ ] For each participant, calculate their similarity score and determine their final payout amount.
- [ ] Ensure the total payout distributed does not exceed the prize pool.
- [ ] Wire it up as the `calculate-scores` subcommand, which should output a clear list of participants and their corresponding payouts.
- [ ] Update `Cargo.toml` to version `0.6.6`.
- [ ] Commit the changes and create a git tag `v0.6.6`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**

### 8.8 Finalization (Priority: Medium)
- [ ] Test the full, end-to-end user workflow using the single `cliptions` CLI tool.
- [ ] Update `README.md` with detailed, platform-specific (Windows, macOS, Linux) instructions for compiling the application and using each subcommand.
- [ ] **Post-MVP Refactoring**: Review the completed `actions` and identify shared logic. Consolidate business models/rules into a `src/domain/` directory and shared technical services (like the Twitter client) into a `src/infra/` directory to improve long-term maintainability.

## 9. Open Questions

1. **$TAO Address Generation**: How should unique $TAO addresses be generated for each participant?
2. **Data Backup**: Should we implement automatic backup of commitment/reveal data?
3. **Error Recovery**: What happens if a subcommand fails partway through?
4. **Platform Support**: Which platforms should we prioritize for compilation instructions?

## 10. Risk Mitigation

- **Financial Risk**: Prize pool limit prevents excessive costs.
- **Technical Risk**: Manual workflows and a single, focused binary reduce automation and dependency complexity.
- **Data Risk**: Local JSON storage with clear backup procedures.
- **Distribution Risk**: Source code compilation avoids binary distribution issues.
- **Model Risk**: CLIP download with MockEmbedder fallback

## 11. Timeline

**Day 1 (Today)**: Implement core refactoring and create new MVP action modules.
**Day 2 (Tomorrow)**: Testing, documentation, and production launch.
**Day 3+**: Iterate based on real usage and feedback 