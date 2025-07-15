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

- [x] 4.0 Refactor and Setup Application Foundation
  - [x] 4.1 **Refactor**: Rename `src/round.rs` to `src/round_processor.rs` and update all `use` statements.
  - [x] 4.2 **Update**: Postponed Rust 2024 edition upgrade; sticking with 2021 to avoid breaking changes.
  - [x] 4.3 **Create**: The async module directory `src/round_engine` and the `state_machine.rs` file.
  - [x] 4.4 **Update**: In `src/lib.rs`, declare the `round_engine` module.
  - [x] 4.5 **Create**: The async binary entry point at `src/bin/cliptions_app.rs`.

- [x] 5.0 Implement Configuration
  - [x] 5.1 In `src/config.rs`, add `TwitterConfig` and `BaseConfig` structs to `CliptionsConfig`.
  - [x] 5.2 Update `config/llm.yaml.template` with placeholders for the new `twitter` and `base` sections.

- [ ] 6.0 Implement Async State Machine with API Libraries
  - [x] 6.1 In `src/round_engine/state_machine.rs`, define the granular state markers.
  - [x] 6.2 Implement the generic async `Round<S>` struct. This struct MUST store the `round_description`, `livestream_url`, and `target_timestamp`. It will store the `target_frame_path` once it has been captured.
  - [ ] 6.3 Implement async state transition methods that call the `TwitterClient`.
    - [ ] 6.3.1 Implement `Round<Pending>::open_commitments(self, client: &TwitterClient) -> Result<Round<CommitmentsOpen>>`. This method will post a text-only tweet announcing the round, including the description, livestream URL, and target timestamp.
    - [ ] 6.3.2 Implement `Round<CommitmentsOpen>::close_commitments(self, client: &TwitterClient) -> Result<Round<CommitmentsClosed>>`. This method posts a tweet announcing commitments are closed.
    - [ ] 6.3.3 Implement `Round<CommitmentsClosed>::capture_frame(self, target_frame_path: PathBuf) -> Result<Round<FrameCaptured>>`. This is an internal state transition that does not tweet, it simply updates the state to include the path to the now-known frame.
    - [ ] 6.3.4 Implement `Round<FrameCaptured>::open_reveals(self, client: &TwitterClient) -> Result<Round<RevealsOpen>>`. This method MUST post a tweet containing the `target_frame` image.

- [ ] 7.0 Implement Role-Based Application Logic
  - [x] 7.1 In `src/bin/cliptions_app.rs`, parse the `--role` argument.
  - [ ] 7.2 Initialize `ConfigManager` and create `TwitterClient`.
  - [ ] 7.3 Implement main async application loop for both roles.
  - [ ] 7.4 **Validator Logic**: Implement the `run_validator_loop`.
    - [ ] 7.4.1 Check for the latest tweet from the validator to see if a round is in progress.
    - [ ] 7.4.2 If no round is active, prompt the user to start a new round by providing: 1) a text description/theme, 2) the `livestream_url`, and 3) the `target_timestamp`.
    - [ ] 7.4.3 On confirmation, call `Round<Pending>::open_commitments` to announce the new round.
    - [ ] 7.4.4 When the round is in `CommitmentsClosed` state and the `target_timestamp` has been reached, the application MUST prompt the validator: "The target time has arrived. Please capture the frame and provide the local file path."
    - [ ] 7.4.5 Once the validator provides the path, the app will call `capture_frame` to update its internal state. It will then immediately prompt the user to confirm opening the reveal phase.
    - [ ] 7.4.6 On confirmation, the app calls `open_reveals` to publish the `target_frame` on Twitter.
  - [ ] 7.5 **Miner Logic**: Implement the `run_miner_loop`.
    - [ ] 7.5.1 Poll the validator's Twitter account.
    - [ ] 7.5.2 Parse the tweets to determine the round state. The announcement tweet will contain the `livestream_url` and `target_timestamp`.
    - [ ] 7.5.3 When the miner sees the `RevealsOpen` state tweet, it will display the `target_frame` image that was posted.

- [ ] 8.0 Integration Testing and Validation
  - [ ] 8.1 Create `#[tokio::test]` unit tests for the new state transitions (`capture_frame`, etc.).
  - [ ] 8.2 In `tests/round_engine_integration.rs`, write a full end-to-end test.
    - [ ] 8.2.1 The test simulates a validator starting a round by posting a mock tweet with a `livestream_url` and `target_timestamp`.
    - [ ] 8.2.2 The test simulates the time passing and the validator being prompted for a frame path.
    - [ ] 8.2.3 The test must verify that when the validator provides the path, the `open_reveals` method is called on the mock `TwitterClient` with the correct image path.
    - [ ] 8.2.4 The test simulates a miner seeing the `RevealsOpen` tweet and correctly parsing the state.

- [ ] 9.0 User-Facing Distribution and Documentation
  - [ ] 9.1 **Create Release Workflow**: Set up a GitHub Action to compile and package the `cliptions_app` binary.
  - [ ] 9.2 **Update README**: Revise `README.md` with a user-focused "Installation" section.
  - [ ] 9.3 **Update CLI Examples**: Change `README.md` examples to show usage for a downloaded binary. 