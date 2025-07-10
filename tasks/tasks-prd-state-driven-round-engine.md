## Relevant Files

- `Cargo.toml` - **Modify** to set up a workspace with the main `realmir` package and the new `twitter-api` crate.
- `crates/twitter-api/Cargo.toml` - The `Cargo.toml` for the new shared library.
- `crates/twitter-api/src/lib.rs` - The main library file for the new shared Twitter API crate.
- `crates/twitter-api/src/client.rs` - Contains the `TwitterClient` and its methods for API interaction.
- `crates/twitter-api/src/models.rs` - Will contain the data structures for Tweets, Users, etc., returned by the API.
- `src/lib.rs` - To declare the `round_engine` module.
- `src/config.rs` - To add a `TwitterConfig` section.
- `src/round_engine/state_machine.rs` - To define the async typestate structs and state transitions that call the new library.
- `src/bin/cliptions_app.rs` - The main async application binary.
- `src/bin/twitter_post.rs` - **Refactor** to become a simple client of the `twitter-api` library.
- `src/bin/twitter_search_replies.rs` - **Refactor** to become a simple client of the `twitter-api` library.
- `src/bin/twitter_latest_tweet.rs` - **Refactor** to become a simple client of the `twitter-api` library.
- `tests/round_engine_integration.rs` - Integration tests for the async state machine.

### Notes

- The project will be structured as a Cargo workspace to manage the main application and the `twitter-api` library.
- The core Twitter API logic will be centralized in the `twitter-api` crate, promoting code reuse and maintainability.
- The `round_engine` will interact with the `TwitterClient` from the library, not with external processes.

## Tasks

- [ ] 1.0 Create Shared Twitter API Library
  - [ ] 1.1 **Create**: A new directory `crates/` for our library crates.
  - [ ] 1.2 **Create**: A new library crate at `crates/twitter-api` using `cargo new --lib`.
  - [ ] 1.3 **Refactor**: Convert the root `Cargo.toml` into a workspace manifest that includes both the main `realmir` package and the new `crates/twitter-api` crate.
  - [ ] 1.4 **Move**: The core Twitter API interaction logic (authentication, request building, execution) from the `src/bin/twitter_*.rs` files into a `TwitterClient` in `crates/twitter-api/src/client.rs`.
  - [ ] 1.5 **Refactor**: The `src/bin/twitter_*.rs` binaries to be simple command-line wrappers around the new `twitter-api` library. They should initialize the `TwitterClient` and call its methods.

- [ ] 2.0 Refactor and Setup Application Foundation
  - [ ] 2.1 **Refactor**: Rename `src/round.rs` to `src/round_processor.rs` and update all `use` statements.
  - [ ] 2.2 **Update**: In `Cargo.toml`, ensure the `edition` is `"2024"`. Verify `tokio`, `clap`, `serde`, and `anyhow` dependencies.
  - [ ] 2.3 **Create**: The async module directory `src/round_engine` and the `state_machine.rs` file.
  - [ ] 2.4 **Update**: In `src/lib.rs`, declare the `round_engine` module.
  - [ ] 2.5 **Create**: The async binary entry point at `src/bin/cliptions_app.rs`.

- [ ] 3.0 Implement Twitter Configuration
  - [ ] 3.1 In `src/config.rs`, add a `TwitterConfig` struct with API credentials and validator username. Include it in `CliptionsConfig`.
  - [ ] 3.2 Update `config/llm.yaml.template` with placeholders for the new `twitter` section.

- [ ] 4.0 Implement Async State Machine with API Library
  - [ ] 4.1 In `src/round_engine/state_machine.rs`, define the state marker structs (`Pending`, `CommitmentsOpen`, etc.).
  - [ ] 4.2 Implement the generic async `Round<S>` struct.
  - [ ] 4.3 Implement async state transition methods (e.g., `async fn close_commitments(self, client: &TwitterClient) -> Result<Round<CommitmentsClosed>>`). These methods will call the `TwitterClient` from our `twitter-api` library.

- [ ] 5.0 Implement Role-Based Application Logic
  - [ ] 5.1 In `src/bin/cliptions_app.rs`, use `clap` to parse the `--role` argument.
  - [ ] 5.2 Initialize the `ConfigManager` and the `TwitterClient`.
  - [ ] 5.3 Implement the main async application loop that continuously polls for the round's state using the `TwitterClient`.
  - [ ] 5.4 **Validator Logic**: If the role is `validator`, prompt for confirmation and call the appropriate `TwitterClient` method to post a state-change tweet.
  - [ ] 5.5 **Miner Logic**: If the role is `miner`, continuously poll and display the latest round status from the validator's tweets.
  - [ ] 5.6 Add signal handling for graceful shutdown.

- [ ] 6.0 Integration Testing and Validation
  - [ ] 6.1 In `src/round_engine/state_machine.rs`, add `#[tokio::test]` unit tests for the typestate transitions.
  - [ ] 6.2 Create the async integration test file `tests/round_engine_integration.rs`.
  - [ ] 6.3 In the integration tests, mock the `TwitterClient`. This can be done by defining a `MockTwitterClient` that implements a shared `TwitterApi` trait, allowing you to return canned responses without making real API calls.
  - [ ] 6.4 Write an integration test that drives the state machine through a full round lifecycle using the mocked client.
  - [ ] 6.5 Add async tests for CLI argument parsing in `cliptions_app.rs`. 