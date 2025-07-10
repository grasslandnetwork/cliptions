## Relevant Files

- `Cargo.toml` - To verify async dependencies (`tokio`, `clap`, `anyhow`) and update the Rust edition.
- `src/lib.rs` - To declare the new `round_engine` module and update paths.
- `src/config.rs` - **Modify** to add a `TwitterConfig` section to `CliptionsConfig` for API keys and validator username.
- `config/llm.yaml.template` - **Modify** to include the new `twitter` configuration section.
- `src/round_processor.rs` - Formerly `src/round.rs`. This file will be renamed.
- `src/round_engine/` - The new module directory for the async state-driven application logic.
- `src/round_engine/state_machine.rs` - To define the async typestate structs and `Round<S>` wrapper with async state transitions.
- `src/round_engine/twitter.rs` - To create a new client for orchestrating calls to the `cliptions_twitter_*` binaries.
- `src/bin/cliptions_app.rs` - The new async main application binary for the validator/miner state machine.
- `tests/round_engine_integration.rs` - Integration tests for the async state machine.

### Notes

- We will **reuse** the data models from `src/types.rs` (`RoundData`, `Participant`, etc.) as the core data carriers within our new typestate structs.
- The state machine will orchestrate calls to the pre-built Twitter binaries (`cliptions_twitter_post`, `cliptions_twitter_latest_tweet`, etc.) using `tokio::process::Command`.
- The `round_engine/twitter.rs` module will be responsible for building the commands, executing them, and parsing their output.

## Tasks

- [ ] 1.0 Refactor and Setup Foundation
  - [ ] 1.1 **Refactor**: Rename `src/round.rs` to `src/round_processor.rs`.
  - [ ] 1.2 **Refactor**: Update module declarations in `src/lib.rs` and any `use` statements to reflect the rename.
  - [ ] 1.3 **Update**: In `Cargo.toml`, ensure the `edition` is `"2024"`. Verify that `tokio`, `clap`, `serde`, and `anyhow` are listed as dependencies.
  - [ ] 1.4 **Create**: The new async module directory `src/round_engine` and the files: `state_machine.rs` and `twitter.rs`.
  - [ ] 1.5 **Update**: In `src/lib.rs`, declare the new `round_engine` module and its submodules.
  - [ ] 1.6 **Create**: The new async binary entry point at `src/bin/cliptions_app.rs` with a `#[tokio::main]` async main function.

- [ ] 2.0 Implement Twitter Configuration
  - [ ] 2.1 In `src/config.rs`, add a `TwitterConfig` struct with fields for API credentials and the validator's Twitter username. Include it in `CliptionsConfig`.
  - [ ] 2.2 Update `config/llm.yaml.template` with placeholder values for the new `twitter` section.

- [ ] 3.0 Implement Async State Machine with Twitter Integration
  - [ ] 3.1 In `src/round_engine/state_machine.rs`, define the empty state marker structs: `Pending`, `CommitmentsOpen`, `CommitmentsClosed`, etc.
  - [ ] 3.2 Implement the generic async `Round<S>` struct that holds round data.
  - [ ] 3.3 In `src/round_engine/twitter.rs`, implement async wrapper functions that execute the `cliptions_twitter_*` binaries using `tokio::process::Command`. These functions will parse `stdout` to return structured results (e.g., tweet content or ID).
  - [ ] 3.4 In `src/round_engine/state_machine.rs`, implement the async state transition methods (e.g., `async fn close_commitments(self) -> Result<Round<CommitmentsClosed>>`). These methods will call the command wrappers from `twitter.rs` to post new state tweets.

- [ ] 4.0 Implement Role-Based Application Logic
  - [ ] 4.1 In `src/bin/cliptions_app.rs`, use `clap` to parse the `--role` command-line argument.
  - [ ] 4.2 Initialize the `ConfigManager` and the `Twitter` client from `round_engine::twitter.rs`.
  - [ ] 4.3 Implement the main async application loop that continuously polls for the current round's state by calling the wrapper for `cliptions_twitter_latest_tweet`.
  - [ ] 4.4 **Validator Logic**: If role is `validator`, match on the current state, prompt for confirmation, and call the appropriate wrapper function from `twitter.rs` to post a new state-change tweet.
  - [ ] 4.5 **Miner Logic**: If role is `miner`, continuously poll and display the latest round status based on the validator's tweets.
  - [ ] 4.6 Add proper async signal handling (Ctrl+C) and graceful shutdown logic.

- [ ] 5.0 Integration Testing and Validation
  - [ ] 5.1 In `src/round_engine/state_machine.rs`, add `#[tokio::test]` unit tests for the typestate transitions.
  - [ ] 5.2 Create the async integration test file `tests/round_engine_integration.rs`.
  - [ ] 5.3 In the integration tests, mock the execution of the Twitter binaries. This can be done by passing a path to a mock script to the wrapper functions instead of the real binary path. The mock script will return expected `stdout`.
  - [ ] 5.4 Write an integration test that drives the state machine through a full round lifecycle, asserting correct state transitions and command outputs.
  - [ ] 5.5 Add async tests for CLI argument parsing in `cliptions_app.rs`. 