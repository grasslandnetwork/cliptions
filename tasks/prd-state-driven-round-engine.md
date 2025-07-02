# PRD: State-Driven Round Engine

## 1. Introduction/Overview

This document outlines the requirements for a new state-driven engine to manage the lifecycle of Cliptions rounds. The core of this feature is to create a robust and autonomous application that understands the current state of a game round and intelligently determines the next logical action.

The system will be driven by the state of the data in a central database (Supabase). It will support two primary roles—Validator and Miner—each with distinct responsibilities. This will move the project from a collection of manually-run scripts to a more cohesive, state-aware application, improving reliability and simplifying operations.

## 2. Goals

- **Goal 1: Semi-Autonomous Operation:** Create an application that can orchestrate the steps of a round, prompting the validator for action and informing the miner of the current status.
- **Goal 2: Improve Reliability:** Centralize state management in a robust Rust application, reducing the potential for manual error.
- **Goal 3: Establish a Clear State Machine:** Implement a well-defined round lifecycle using the Rust typestate pattern to ensure correctness at compile time.
- **Goal 4: Future-Proof Architecture:** Lay the groundwork for a fully autonomous system by establishing a clear client-server architecture with a dedicated persistence layer.

## 3. User Stories

- **As a Validator,** I want the application to read the current round state from the database and prompt me to execute the next step, so that I can manage the game lifecycle efficiently and without errors.
- **As a Miner,** I want the application to connect to the game server, show me the live status of the active round, and help me craft my submission, so that I can participate in the game easily.

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
-   `--role miner`: Runs the application in Miner mode.
-   If no role is specified, it **must** default to `miner`.

### FR4: Data Persistence and Synchronization
-   The application **must** use Supabase as its primary database for storing all round data.
-   It **must** integrate with ElectricSQL to sync state between the server and the local client.
-   The client-side application **must** use `rusqlite` for local storage, powered by the ElectricSQL sync layer.

### FR5: Validator Workflow
-   When started as a `validator`, the application will connect to the database and determine the state of the current round.
-   It will identify the next logical action in the lifecycle (e.g., "Close Commitments", "Open Reveals").
-   It **must** prompt the validator for confirmation before executing the action.
-   Upon confirmation, it will orchestrate the task by calling the relevant, pre-existing Python script.

### FR6: Miner Workflow
-   When started as a `miner`, the application will connect to the database and fetch the current round's status.
-   It will display the status to the user (e.g., "Commitments are open until...").
-   It will continuously poll for state changes (e.g., waiting for a new round to be announced by the validator).
-   It **must** provide a user interface or tool to help the miner craft and prepare their commitment/reveal submissions.

### FR7: Data Schema
The Rust data structures for a round **must** be based on the fields present in the `data/rounds.json` file, adapted for a relational structure in Supabase. This includes participants, commitments, reveals, scores, URLs, and timestamps.

## 5. Non-Goals (Out of Scope)

-   **Automatic Round Creation:** The validator must manually initiate the creation of a new round. The system will not automatically start a new round after one finishes.
-   **Replacement of Python Scripts:** For this initial version, the Rust application will only *orchestrate* actions. The actual implementation of tasks like posting to Twitter will remain in the existing Python scripts.
-   **Full Automation:** The goal is a semi-automated system that requires validator confirmation. Fully autonomous operation without any prompts is a future goal.

## 6. Design Considerations

-   **Typestate Pattern:** The API design should heavily lean on the typestate pattern. For example, a `Round<CommitmentsOpen>` struct would have a method like `close_commitments(self) -> Round<CommitmentsClosed>`, but not a method to open reveals. This enforces the lifecycle at the type level.

## 7. Technical Considerations

-   **Backend Stack:** Supabase (Postgres) and ElectricSQL (Sync Service).
-   **Client Stack:** Rust application using `rusqlite` for the local database cache.
-   **Orchestration:** The Rust application will execute Python scripts as child processes to perform external actions.
-   **Configuration:** The application will need a configuration file to manage database connection details, role, etc.

### Critical Architecture Boundary: Async/Sync Integration

**Issue:** The current Cliptions Rust core is **entirely synchronous** with pure functional interfaces, but the state-driven engine requires **async database operations** and real-time polling.

**Current State:**
- All existing Rust code is sync: `fn process_round_payouts() -> Result<Vec<ScoringResult>>`
- No `tokio` runtime or async traits anywhere
- Pure functional approach with immediate results

**Required Changes:**
- **Async Runtime:** Add `tokio` runtime for async database operations
- **Async State Transitions:** State machine methods must be async: `async fn close_commitments(self) -> Result<Round<CommitmentsClosed>>`
- **Async Database Layer:** All database operations must be async
- **Sync Bridge:** Maintain sync compatibility for existing Python bindings using `tokio::runtime::Runtime::block_on()`

**Design Pattern:**
```rust
// New async state machine
impl Round<CommitmentsOpen> {
    pub async fn close_commitments(self) -> Result<Round<CommitmentsClosed>> {
        // Async database operations
    }
}

// Sync compatibility wrapper for Python bindings
pub fn close_commitments_sync(round: Round<CommitmentsOpen>) -> Result<Round<CommitmentsClosed>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(round.close_commitments())
}
```

This architectural boundary is **critical** for maintaining the existing mature Rust core while enabling the new async state-driven engine.

## 8. Success Metrics

-   The application can be successfully launched with `--role validator` and `--role miner`.
-   The application correctly connects to the database, reads the current round's state, and accurately identifies the next logical step.
-   A validator can successfully advance a round through its entire lifecycle by confirming the prompts given by the application.
-   A miner can successfully view the live state of a round and receives timely updates as the state changes.
-   When a round is complete, the validator instance waits for manual input to start a new round, while miner instances correctly enter a polling state, waiting for the new round.

## 9. Open Questions
- None at this time. 