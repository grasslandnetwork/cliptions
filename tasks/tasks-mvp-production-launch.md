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
**Status**: [x] Completed
**Priority**: High
**Description**: Implement the `collect-commitments` subcommand.

**Tasks**:
- [x] Create the `src/actions/collect_commitments.rs` module.
- [x] **Check `binaries_architecture.md` for existing `twitter_search_replies` function signatures before implementing.**
- [x] Implement logic to extract commitment replies from a specific tweet, sourcing from `twitter_search_replies.rs`.
- [x] Wire it up as the `collect-commitments` subcommand.
- [x] **Create tests for the collect-commitments subcommand by moving appropriate tests from the old binary.**
- [ ] Update `README.md` to document the new `cliptions collect-commitments` command.
- [ ] Update `Cargo.toml` to version `0.6.2`.
- [ ] Commit the changes and create a git tag `v0.6.2`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [ ] Update this task list to mark all tasks as completed.

---

### Slice 3: Validator Replies with Fees (v0.6.3)
**Status**: [ ] Not Started
**Priority**: High
**Description**: Implement the `reply-with-fees` subcommand.

**Tasks**:
- [ ] Create the `src/actions/reply_with_fees.rs` module.
- [ ] **Check `binaries_architecture.md` for existing `twitter_post` function signatures before implementing.**
- [ ] Implement logic to reply to each commitment with a unique $TAO address, sourcing from `twitter_post.rs`.
- [ ] Wire it up as the `reply-with-fees` subcommand.
- [ ] **Create tests for the reply-with-fees subcommand by moving appropriate tests from the old binary.**
- [ ] Update `README.md` to document the new `cliptions reply-with-fees` command.
- [ ] Update `Cargo.toml` to version `0.6.3`.
- [ ] Commit the changes and create a git tag `v0.6.3`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [ ] Update this task list to mark all tasks as completed.

---

### Slice 4: Validator Collects Reveals (v0.6.4)
**Status**: [ ] Not Started
**Priority**: High
**Description**: Implement the `collect-reveals` subcommand.

**Tasks**:
- [ ] Create the `src/actions/collect_reveals.rs` module.
- [ ] **Check `binaries_architecture.md` for existing `twitter_search_replies` function signatures before implementing.**
- [ ] Implement logic to extract reveal replies from a specific tweet.
- [ ] Wire it up as the `collect-reveals` subcommand.
- [ ] **Create tests for the collect-reveals subcommand by moving appropriate tests from the old binary.**
- [ ] Update `README.md` to document the new `cliptions collect-reveals` command.
- [ ] Update `Cargo.toml` to version `0.6.4`.
- [ ] Commit the changes and create a git tag `v0.6.4`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [ ] Update this task list to mark all tasks as completed.

---

### Slice 5: Validator Verifies Commitments (v0.6.5)
**Status**: [ ] Not Started
**Priority**: High
**Description**: Implement the `verify-commitments` subcommand.

**Tasks**:
- [ ] Create the `src/actions/verify_commitments.rs` module.
- [ ] **Check `binaries_architecture.md` for existing `verify_commitments` function signatures before implementing.**
- [ ] Move logic from the old `verify_commitments` binary.
- [ ] Wire it up as the `verify-commitments` subcommand.
- [ ] **Create tests for the verify-commitments subcommand by moving appropriate tests from the old binary.**
- [ ] Update `README.md` to document the new `cliptions verify-commitments` command.
- [ ] Update `Cargo.toml` to version `0.6.5`.
- [ ] Commit the changes and create a git tag `v0.6.5`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [ ] Update this task list to mark all tasks as completed.

---

### Slice 6: Calculate Scores & Payouts (v0.6.6)
**Status**: [ ] Not Started
**Priority**: High
**Description**: Implement the `calculate-scores` subcommand.

**Tasks**:
- [ ] Create the `src/actions/calculate_scores.rs` module.
- [ ] **Check `binaries_architecture.md` for existing `calculate_scores` function signatures before implementing.**
- [ ] Implement logic to take a list of **verified** participants as input.
- [ ] For each participant, calculate their similarity score and determine their final payout amount.
- [ ] Ensure the total payout distributed does not exceed the prize pool.
- [ ] Wire it up as the `calculate-scores` subcommand, which should output a clear list of participants and their corresponding payouts.
- [ ] **Create tests for the calculate-scores subcommand by moving appropriate tests from the old binary.**
- [ ] Update `README.md` to document the new `cliptions calculate-scores` command.
- [ ] Update `Cargo.toml` to version `0.6.6`.
- [ ] Commit the changes and create a git tag `v0.6.6`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [ ] Update this task list to mark all tasks as completed.

---

### Finalization
**Status**: [ ] Not Started
**Priority**: Medium
**Description**: Finalize the project for release.

**Tasks**:
- [ ] Test the full, end-to-end user workflow using the single `cliptions` CLI tool.
- [ ] Update `README.md` with detailed, platform-specific (Windows, macOS, Linux) instructions for compiling the application and using each subcommand.
- [ ] **Post-MVP Refactoring**: Review the completed `actions` and identify shared logic. Consolidate business models/rules into a `src/domain/` directory and shared technical services (like the Twitter client) into a `src/infra/` directory to improve long-term maintainability.
- [ ] Update this task list to mark all tasks as completed.