## Relevant Files

- `Cargo.toml` - To set up a workspace and add new web server dependencies (`axum`, `ethers-rs`).
- `crates/twitter-api/` - The shared Twitter library.
- `crates/base-api/` - The shared BASE API library (for signature verification).
- `fee_frontend/index.html` - The simple HTML file for the fee payment web page.
- `fee_frontend/main.js` - The JavaScript file to handle WalletConnect and backend communication.
- `src/round_engine/state_machine.rs` - To define the async typestate machine.
- `src/bin/cliptions_app.rs` - The main binary, which will also launch the web server.

### Notes
- The project will be a Cargo workspace managing the main application and its component libraries.
- The fee verification process will be handled by a simple web service hosted directly by the `cliptions_app`.

## Tasks

- [x] 1.0 Research and Plan Fee Management
  - [x] 1.1 **Research**: Investigate the simplest method to link a BASE wallet payment to a Twitter user identity.
  - [x] 1.2 **Decision**: A hybrid approach using a simple JS frontend for wallet signing and a Rust backend for verification is the simplest path.
  - [x] 1.3 **Plan**: The fee management system will be a web service hosted by the main application.

- [x] 2.0 Implement Fee Management Web Service
  - [x] 2.1 **Create Frontend Directory**: Create a new top-level directory named `fee_frontend`.
  - [x] 2.2 **Implement Frontend**: Inside `fee_frontend`, create an `index.html` and a `main.js`. Implement the wallet connection and message signing flow using `web3modal` or a similar library. The page should prompt the user for their Twitter handle.
  - [x] 2.3 **Add Backend Dependencies**: Add `axum`, `tokio`, and `ethers-rs` to the dependencies of the `cliptions-core` crate in `Cargo.toml`.
  - [x] 2.4 **Implement Backend Endpoint**: In `cliptions_app.rs`, create a simple `axum` web server with a `/verify-payment` endpoint. This endpoint will receive the signed message and use `ethers-rs` to verify it.
  - [x] 2.5 **Integrate Web Server**: Modify the `main` function in `cliptions_app.rs` to launch the `axum` server in a separate async task, so it runs alongside the main application logic. The server should also be configured to serve the static files from the `fee_frontend` directory.

- [x] 3.0 Create Core API Libraries
  - [x] 3.1 **Create**: A new directory `crates/` for our library crates.
  - [x] 3.2 **Create**: A new library crate at `crates/twitter-api` using `cargo new --lib`.
  - [x] 3.3 **Create**: A new library crate at `crates/base-api` using `cargo new --lib` (this will be used for payout logic later).
  - [x] 3.4 **Refactor**: Convert the root `Cargo.toml` into a workspace manifest that includes `cliptions-core`, `crates/twitter-api`, and `crates/base-api`.
  - [x] 3.5 **Implement `twitter-api`**: Move Twitter logic into a `TwitterClient`. Implement functions for `post_tweet`, `post_tweet_with_image`, `reply_to_tweet`, and `get_latest_tweet`.
  - [x] 3.6 **Refactor Binaries**: Refactor the `src/bin/twitter_*.rs` binaries to be simple wrappers around the new `twitter-api` library.

- [ ] 4.0 Refactor and Setup Application Foundation
  - [ ] 4.1 **Refactor**: Rename `src/round.rs` to `src/round_processor.rs` and update all `use` statements.
  - [ ] 4.2 **Update**: In `Cargo.toml`, ensure the `edition` is `"2024"`.
  - [ ] 4.3 **Create**: The async module directory `src/round_engine` and the `state_machine.rs` file.
  - [ ] 4.4 **Update**: In `src/lib.rs`, declare the `round_engine` module.
  - [ ] 4.5 **Create**: The async binary entry point at `src/bin/cliptions_app.rs`.

- [ ] 5.0 Implement Configuration
  - [ ] 5.1 In `src/config.rs`, add `TwitterConfig` and `BaseConfig` structs to `CliptionsConfig`.
  - [ ] 5.2 Update `config/llm.yaml.template` with placeholders for the new `twitter` and `base` sections.

- [ ] 6.0 Implement Async State Machine with API Libraries
  - [ ] 6.1 In `src/round_engine/state_machine.rs`, define the granular state markers: `Pending`, `CommitmentsOpen`, `FeeCollectionOpen`, `FeeCollectionClosed`, `RevealsOpen`, etc.
  - [ ] 6.2 Implement the generic async `Round<S>` struct.
  - [ ] 6.3 Implement async state transition methods that call the `TwitterClient` and `BaseClient` as needed.

- [ ] 7.0 Implement Role-Based Application Logic
  - [ ] 7.1 In `src/bin/cliptions_app.rs`, parse the `--role` argument.
  - [ ] 7.2 Initialize the `ConfigManager`, `TwitterClient`, and `BaseClient`.
  - [ ] 7.3 Implement the main async application loop that polls for state and drives the round forward.
  - [ ] 7.4 **Validator Logic**: Ensure the flow correctly posts tweets, checks for fees, and triggers payouts.
  - [ ] 7.5 **Miner Logic**: Ensure the flow correctly displays the round status and the URL for the local fee payment page.

- [ ] 8.0 Integration Testing and Validation
  - [ ] 8.1 Create `#[tokio::test]` unit tests for the state machine transitions.
  - [ ] 8.2 In `tests/round_engine_integration.rs`, mock the `TwitterClient`, `BaseClient`, and the web endpoint.
  - [ ] 8.3 Write an integration test that drives the state machine through a full round lifecycle.

- [ ] 9.0 User-Facing Distribution and Documentation
  - [ ] 9.1 **Create Release Workflow**: Set up a GitHub Action to compile and package the `cliptions_app` binary.
  - [ ] 9.2 **Update README**: Revise `README.md` with a user-focused "Installation" section.
  - [ ] 9.3 **Update CLI Examples**: Change `README.md` examples to show usage for a downloaded binary. 