# PRD: State-Driven Round Engine

## 1. Introduction/Overview

This document outlines the requirements for a new state-driven engine to manage the lifecycle of Cliptions rounds. The core of this feature is to create a robust and autonomous application that understands the current state of a game round and intelligently determines the next logical action.

The system will be driven by the state of the data communicated via the **Twitter/X API**. It will support two primary roles—Validator and Miner—each with distinct responsibilities. This will move the project from a collection of manually-run scripts to a more cohesive, state-aware application, improving reliability and simplifying operations.

## 2. Goals

- **Goal 1: Semi-Autonomous Operation:** Create an application that can orchestrate the steps of a round, prompting the validator for action and informing the miner of the current status.
- **Goal 2: Improve Reliability:** Centralize state management in a robust Rust application, reducing the potential for manual error.
- **Goal 3: Establish a Clear State Machine:** Implement a well-defined round lifecycle using the Rust typestate pattern to ensure correctness at compile time.
- **Goal 4: Future-Proof Architecture:** Lay the groundwork for a fully autonomous system by establishing a clear client-server architecture with a dedicated persistence layer.

## 3. User Stories

- **As a Validator,** I want the application to read the current round state from the database and prompt me to execute the next step, so that I can manage the game lifecycle efficiently and without errors.
- **As a Miner,** I want the application to connect to Twitter, show me the live status of the active round based on the Validator's tweets, and help me craft my submission, so that I can participate in the game easily.

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
-   The state of the round will be determined by parsing the content of tweets from the validator's account.
-   Miners will fetch state by reading these public tweets. No local or remote database will be required for this MVP.

### FR5: Validator Workflow
-   When started as a `validator`, the application will use the `cliptions_twitter_latest_tweet` binary to fetch the latest tweet from its own account to determine the current round state.
-   It will identify the next logical action in the lifecycle (e.g., "Close Commitments", "Open Reveals").
-   It **must** prompt the validator for confirmation before executing the action.
-   Upon confirmation, it will orchestrate the task by calling the relevant `cliptions_twitter_*` binary (e.g., `cliptions_twitter_post` to announce the next state).

### FR6: Miner Workflow
-   When started as a `miner`, the application will use the `cliptions_twitter_latest_tweet` binary to fetch the validator's latest tweet and display the current round status.
-   It will continuously poll for new tweets from the validator to detect state changes.
-   It **must** provide a user interface or tool to help the miner craft and prepare their commitment/reveal submissions.

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
        // 1. Construct the tweet content.
        let tweet_text = "Commitments are now closed!";
        // 2. Call the library function directly.
        client.post_tweet(tweet_text).await?;
        // 3. Return the new state.
        Ok(self.into_closed_state())
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