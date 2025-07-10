# PRD: State-Driven Round Engine

## 1. Introduction/Overview

This document outlines the requirements for a new state-driven engine to manage the lifecycle of Cliptions rounds. The core of this feature is to create a robust and autonomous application that understands the current state of a game round and intelligently determines the next logical action.

The system will be driven by the state of the data communicated via the **Twitter/X API**. It will support two primary roles—Validator and Miner—each with distinct responsibilities. This will move the project from a collection of manually-run scripts to a more cohesive, state-aware application, improving reliability and simplifying operations.

## 2. Goals

-   **Goal 1: Semi-Autonomous Operation:** Create an application that can orchestrate the steps of a round, prompting the validator for action and informing the miner of the current status.
-   **Goal 2: Improve Reliability:** Centralize state management in a robust Rust application, reducing the potential for manual error.
-   **Goal 3: Establish a Clear State Machine:** Implement a well-defined round lifecycle using the Rust typestate pattern to ensure correctness at compile time.
-   **Goal 4: Future-Proof Architecture:** Lay the groundwork for a fully autonomous system by establishing a clear client-server architecture with a dedicated persistence layer.
-   **Goal 5: Create an Easy-to-Use Distribution:** Package the application so that non-technical users (miners) can easily download and run it without needing to compile from source.

## 3. User Stories

-   **As a Validator,** I want the application to read the current round state from the database and prompt me to execute the next step, so that I can manage the game lifecycle efficiently and without errors.
-   **As a Miner,** I want the application to connect to Twitter, show me the live status of the active round based on the Validator's tweets, and help me craft my submission, so that I can participate in the game easily.

## 4. Functional Requirements

### FR1: Rust Typestate Pattern for Round Lifecycle
The application **must** implement the round lifecycle using the Rust typestate pattern. This ensures that operations can only be performed when the round is in the correct state, enforced by the compiler.

### FR2: Defined Round States
The round state machine will consist of the following distinct states:
1.  `Pending`
2.  `CommitmentsOpen`
3.  `CommitmentsClosed`
4.  `RevealsOpen`
5.  `RevealsClosed`
6.  `Payouts`
7.  `Finished`

### FR3: Role-Based Operation
The application **must** be launchable with a specific role:
-   `--role validator`: Runs the application in Validator mode.
-   `--role miner`: Runs theapplication in Miner mode.
-   If no role is specified, it **must** default to `miner`.

### FR4: Data Persistence and Synchronization
-   The application **must** use the **Twitter/X API** as its primary data store and communication channel.
-   The application **must** also integrate with the **BASE API** for payment processing.
-   The state of the round will be determined by parsing the content of tweets from the validator's account.
-   Miners will fetch state by reading these public tweets. No local or remote database will be required for this MVP.

### FR5: Payment and Fee Management
The application **must** support a fee-based system for round participation.
- **Implementation Strategy**: A hybrid web-based approach will be used.
  - **Frontend**: A minimal, single-page web application will provide a "Connect Wallet" button. It will use a standard JavaScript library (e.g., Web3Modal) to handle wallet connection and prompt the user to sign a message containing their Twitter handle.
  - **Backend**: The main `cliptions_app` will host a simple web server endpoint (e.g., `/verify-payment`).
- **Verification Flow**: The web frontend will send the signed message to the Rust backend endpoint, which will verify the signature and record the user's Twitter handle as "paid."

### FR6: Validator Workflow
-   When started as a `validator`, the application will use the `cliptions-twitter-api` library to determine the current round state.
-   It will prompt for confirmation before executing actions like closing commitments, opening reveals, and initiating payouts.
-   It **must** post a "Target Frame" image when transitioning to the `RevealsOpen` state.
-   It **must** execute payouts to winning miners using the BASE API.

### FR7: Miner Workflow
-   When started as a `miner`, it will poll for validator tweets to understand the current round status.
-   It **must** guide the miner to a local URL (served by the app) to pay their entry fee.
-   It will provide tools to help the miner craft their commitment and reveal submissions.

### FR7: Data Schema
The round state and data **must** be encoded in the text of the validator's tweets. A clear, machine-parsable format for these tweets (e.g., using hashtags or key-value pairs like `STATE:CommitmentsOpen`) must be established to represent the round state, deadlines, and other relevant data.

## 5. Non-Goals (Out of Scope)

-   **Automatic Round Creation:** The validator must manually initiate the creation of a new round. The system will not automatically start a new round after one finishes.
-   **Direct Twitter API Implementation:** The state machine will not implement Twitter API logic directly. It will orchestrate actions by calling the new, self-contained `cliptions_twitter_*` binaries.
-   **Full Automation:** The goal is a semi-automated system that requires validator confirmation. Fully autonomous operation without any prompts is a future goal.

## 6. Design Considerations

-   **Typestate Pattern:** The API design should heavily lean on the typestate pattern. For example, a `Round<CommitmentsOpen>` struct would have a method like `close_commitments(self) -> Round<CommitmentsClosed>`, but not a method to open reveals. This enforces the lifecycle at the type level.

## 7. Technical Considerations

-   **Primary Dependency:** The Twitter/X API v2.
-   **Orchestration:** The Rust application will call functions from a dedicated `cliptions-twitter-api` shared library crate for all Twitter interactions.
-   **Configuration:** The application will need a configuration file to manage API credentials, the validator's Twitter username, and other settings.
-   **Payment Integration:**
  - A `base-api` Rust crate will use a library like `ethers-rs` for signature verification.
  - A Rust web server framework (e.g., `axum`) will be used to host the verification endpoint.
  - A minimal web frontend (`HTML/JS`) with a library like `web3modal` will handle user wallet interactions.

### Critical Architecture Boundary: Async/Sync Integration

**Issue:** The current Cliptions Rust core is **entirely synchronous**, but the state-driven engine requires **async operations** for polling and API calls.

**Current State:**
- All existing Rust code is sync: `fn process_round_payouts() -> Result<Vec<ScoringResult>>`
- No `tokio` runtime or async traits anywhere
- Pure functional approach with immediate results

**Required Changes:**
- **Async Runtime:** Add `tokio` runtime for the main application loop.
- **Async State Transitions:** State machine methods must be async: `async fn close_commitments(self) -> Result<Round<CommitmentsClosed>>`. These methods will call the Twitter API library.
- **Direct Library Calls:** The state machine will call async functions from the shared `cliptions-twitter-api` library directly, instead of executing external processes.

**Design Pattern:**
```rust
// In our new `cliptions-twitter-api` library
pub struct TwitterClient { /* ... */ }
impl TwitterClient {
    pub async fn post_tweet(&self, text: &str) -> Result<Tweet> {
        // ... logic to call Twitter API ...
    }
}

// In our `cliptions_app` state machine
impl Round<CommitmentsOpen> {
    pub async fn close_commitments(self, client: &TwitterClient) -> Result<Round<CommitmentsClosed>> {
        // 1. Construct the tweet content using the `social.rs` formatter.
        let tweet_text = "Commitments are now closed! Fee payment window is now open.";
        // 2. Call the library function directly.
        client.post_tweet(tweet_text).await?;
        // 3. Return the new state.
        Ok(self.into_fee_collection_state())
    }
}
```

This architecture is designed to enable the new async state-driven engine while keeping the orchestration logic separate from the core business logic.

## 8. Success Metrics

-   The application can be successfully launched with `--role validator` and `--role miner`.
-   The application correctly fetches and parses the latest tweet from the validator's account to determine the current round's state.
-   A validator can successfully advance a round through its entire lifecycle by confirming the prompts, which results in posting a new status tweet.
-   A miner can successfully view the live state of a round by reading the validator's tweets and receives timely updates as the state changes.
-   When a round is complete, the validator instance waits for manual input to start a new round, while miner instances correctly enter a polling state, waiting for the new round.

## 9. Open Questions
- None at this time. 