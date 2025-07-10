## Relevant Files

- `Cargo.toml` - To set up a workspace including `realmir`, `twitter-api`, and the new `base-api` crate.
- `crates/twitter-api/Cargo.toml` - For the shared Twitter library.
- `crates/twitter-api/src/lib.rs` - The main library file for Twitter API interactions.
- `crates/base-api/Cargo.toml` - For the shared BASE API library.
- `crates/base-api/src/lib.rs` - The main library file for BASE API interactions.
- `src/lib.rs` - To declare the `round_engine` module.
- `src/config.rs` - To add `TwitterConfig` and `BaseConfig` sections.
- `src/round_engine/state_machine.rs` - To define the more granular async typestate machine.
- `src/bin/cliptions_app.rs` - The main async application binary.
- `tests/round_engine_integration.rs` - Integration tests for the async state machine.

### Notes

- The project will be a Cargo workspace to manage the main application and its component libraries (`twitter-api`, `base-api`).
- The `round_engine` will orchestrate calls to the `TwitterClient` and `BaseClient` from the libraries.
- Mocking will be crucial for testing both Twitter and BASE API interactions without making real calls.

## Tasks

- [ ] 1.0 Research and Plan Fee Management
  - [ ] 1.1 **Research**: Investigate the simplest method to link a BASE wallet payment to a Twitter user identity.
  - [ ] 1.2 **Decision**: Document the chosen approach (e.g., web endpoint with wallet connection, signed message, etc.).
  - [ ] 1.3 **Plan**: Outline the implementation steps for the chosen fee verification mechanism.

- [ ] 2.0 Create Core API Libraries
  - [ ] 2.1 **Create**: A new directory `crates/` for our library crates.
  - [ ] 2.2 **Create**: A new library crate at `crates/twitter-api` using `cargo new --lib`.
  - [ ] 2.3 **Create**: A new library crate at `crates/base-api` using `cargo new --lib`.
  - [ ] 2.4 **Refactor**: Convert the root `Cargo.toml` into a workspace manifest that includes `realmir`, `crates/twitter-api`, and `crates/base-api`.
  - [ ] 2.5 **Implement `twitter-api`**: Move Twitter logic into a `TwitterClient`. Implement functions for `post_tweet`, `post_tweet_with_image`, `reply_to_tweet`, and `get_latest_tweet`.
  - [ ] 2.6 **Refactor Binaries**: Refactor the `src/bin/twitter_*.rs` binaries to be simple wrappers around the new `twitter-api` library.

- [ ] 3.0 Refactor and Setup Application Foundation
  - [ ] 3.1 **Refactor**: Rename `src/round.rs` to `src/round_processor.rs` and update all `use` statements.
  - [ ] 3.2 **Update**: In `Cargo.toml`, ensure the `edition` is `"2024"`. Verify core dependencies.
  - [ ] 3.3 **Create**: The async module directory `src/round_engine` and the `state_machine.rs` file.
  - [ ] 3.4 **Update**: In `src/lib.rs`, declare the `round_engine` module.
  - [ ] 3.5 **Create**: The async binary entry point at `src/bin/cliptions_app.rs`.

- [ ] 4.0 Implement Configuration
  - [ ] 4.1 In `src/config.rs`, add `TwitterConfig` and `BaseConfig` structs to `CliptionsConfig`.
  - [ ] 4.2 Update `config/llm.yaml.template` with placeholders for the new `twitter` and `base` sections.

- [ ] 5.0 Implement Fee Management
  - [ ] 5.1 **Implement `base-api`**: Based on the research from Task 1, implement a `BaseClient` in the `base-api` crate. It should handle wallet interactions and payment verification.
  - [ ] 5.2 **Integrate**: Connect the `BaseClient` into the main `cliptions_app`.

- [ ] 6.0 Implement Async State Machine with API Libraries
  - [ ] 6.1 In `src/round_engine/state_machine.rs`, define the more granular state markers: `Pending`, `CommitmentsOpen`, `FeeCollectionOpen`, `FeeCollectionClosed`, `RevealsOpen`, etc.
  - [ ] 6.2 Implement the generic async `Round<S>` struct.
  - [ ] 6.3 Implement async state transition methods that call the `TwitterClient` and `BaseClient` as needed for each step of the round.

- [ ] 7.0 Implement Role-Based Application Logic
  - [ ] 7.1 In `src/bin/cliptions_app.rs`, parse the `--role` argument.
  - [ ] 7.2 Initialize the `ConfigManager`, `TwitterClient`, and `BaseClient`.
  - [ ] 7.3 Implement the main async application loop that polls for state and drives the round forward based on validator prompts.
  - [ ] 7.4 **Validator Logic**: Ensure the validator flow correctly posts tweets, checks for fees, and triggers payouts.
  - [ ] 7.5 **Miner Logic**: Ensure the miner flow correctly displays the round status and fee payment instructions.

- [ ] 8.0 Integration Testing and Validation
  - [ ] 8.1 Create `#[tokio::test]` unit tests for the state machine transitions.
  - [ ] 8.2 In `tests/round_engine_integration.rs`, mock both the `TwitterClient` and `BaseClient` (e.g., using traits and mock objects).
  - [ ] 8.3 Write an integration test that drives the state machine through a full round lifecycle using the mocked clients, asserting correct API calls at each step.

- [ ] 9.0 User-Facing Distribution and Documentation
  - [ ] 9.1 **Create Release Workflow**: Set up a GitHub Action to compile and package the `cliptions_app` binary for macOS, Windows, and Linux on new git tags.
  - [ ] 9.2 **Update README**: Revise the `README.md` to be user-focused, with a clear "Installation" section pointing to GitHub Releases.
  - [ ] 9.3 **Update CLI Examples**: Change the examples in the `README.md` to show usage for a downloaded binary (e.g., `./cliptions_app`) instead of `cargo run`. 