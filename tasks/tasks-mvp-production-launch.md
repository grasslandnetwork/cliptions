# MVP Production Launch Tasks (Vertical Slice Plan)

## Overview
This task list outlines the refactoring and implementation plan for the Cliptions MVP, following a Vertical Slice architecture. Each task represents a self-contained, testable feature that must pass CI before moving to the next.

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
**Status**: [ ] Not Started
**Priority**: High
**Description**: Implement the `generate-commitment` subcommand.

**Tasks**:
- [ ] Create the `src/actions/generate_commitment.rs` module.
- [ ] Move logic from the old `generate_commitment` binary, including saving the guess/salt locally.
- [ ] Wire it up as the `generate-commitment` subcommand in `main.rs`.
- [ ] Update `README.md` to document the new `cliptions generate-commitment` command.
- [ ] Update `Cargo.toml` to version `0.6.1`.
- [ ] Commit the changes and create a git tag `v0.6.1`.
- [ ] **Verify the new tag triggers and passes all checks in GitHub Actions.**
- [ ] Update this task list to mark all tasks as completed.

---

### Slice 2: Validator Collects Commitments (v0.6.2)
**Status**: [ ] Not Started
**Priority**: High
**Description**: Implement the `collect-commitments` subcommand.

**Tasks**:
- [ ] Create the `src/actions/collect_commitments.rs` module.
- [ ] Implement logic to extract commitment replies from a specific tweet, sourcing from `twitter_search_replies.rs`.
- [ ] Wire it up as the `collect-commitments` subcommand.
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
- [ ] Implement logic to reply to each commitment with a unique $TAO address, sourcing from `twitter_post.rs`.
- [ ] Wire it up as the `reply-with-fees` subcommand.
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
- [ ] Implement logic to extract reveal replies from a specific tweet.
- [ ] Wire it up as the `collect-reveals` subcommand.
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
- [ ] Move logic from the old `verify_commitments` binary.
- [ ] Wire it up as the `verify-commitments` subcommand.
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
- [ ] Implement logic to take a list of **verified** participants as input.
- [ ] For each participant, calculate their similarity score and determine their final payout amount.
- [ ] Ensure the total payout distributed does not exceed the prize pool.
- [ ] Wire it up as the `calculate-scores` subcommand, which should output a clear list of participants and their corresponding payouts.
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