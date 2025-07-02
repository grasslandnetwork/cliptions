## Relevant Files

- `Cargo.toml` - To add new async dependencies: `tokio`, `sqlx` (with postgres and sqlite features), `config`, `clap`.
- `src/lib.rs` - To declare the new `round_engine` module and update the path for the renamed `round_processor` module.
- `src/config.rs` - **Modify** to add a `database` section to `CliptionsConfig` for Supabase connection details.
- `config/llm.yaml.template` - **Modify** to include the new `database` configuration section.
- `src/round_processor.rs` - Formerly `src/round.rs`. This file will be renamed to better reflect its purpose as a stateless toolkit.
- `src/round_engine/` - The new module directory for the async state-driven application logic.
- `src/round_engine/state_machine.rs` - To define the async typestate structs and `Round<S>` wrapper with async state transitions.
- `src/round_engine/db.rs` - To handle all async database interactions using the new config.
- `src/round_engine/sync_bridge.rs` - To provide sync compatibility wrappers for existing Python bindings.
- `src/bin/cliptions_app.rs` - The new async main application binary for the validator/miner state machine.
- `tests/round_engine_integration.rs` - Integration tests for the async state machine.

### Notes

- We will **reuse** the data models from `src/types.rs` (`RoundData`, `Participant`, etc.) as the core data carriers within our new typestate structs.
- We will **extend** the existing `ConfigManager` from `src/config.rs` instead of creating a new one.
- The async state machine will maintain sync compatibility through wrapper functions for existing Python bindings.
- **Database Strategy**: Using `sqlx` allows the same codebase to support both Postgres (validators with Supabase) and SQLite (miners with local storage).
- Unit tests for the async state machine logic can be placed in `src/round_engine/state_machine.rs`.

## Tasks

- [ ] 1.0 Setup Async Architecture Foundation
  - [ ] 1.1 **Refactor**: Rename `src/round.rs` to `src/round_processor.rs` to better distinguish it from the new async `round_engine`.
  - [ ] 1.2 **Refactor**: Update module declarations in `src/lib.rs` and any `use` statements in other files to reflect the rename from `round` to `round_processor`.
  - [ ] 1.3 Add async dependencies to `Cargo.toml`: `tokio` (with features: `["full"]`), `sqlx` (with features: `["runtime-tokio-rustls", "postgres", "sqlite", "migrate"]`), `clap`, `serde`, `anyhow` for async error handling. Also update `edition = "2021"` to `edition = "2024"`.
  - [ ] 1.4 Create the new async module directory `src/round_engine` and the files: `state_machine.rs`, `db.rs`, and `sync_bridge.rs`.
  - [ ] 1.5 In `src/lib.rs`, declare the new `round_engine` module and set up proper async exports and re-exports for the state machine, database layer, and sync bridge.
  - [ ] 1.6 Create the new async binary entry point at `src/bin/cliptions_app.rs` with `#[tokio::main]` async main function.
  - [ ] 1.7 Ensure existing sync functions remain available for Python bindings in `src/lib.rs`.

- [ ] 2.0 Setup Database Integration and Configuration
  - [ ] 2.1 In `src/config.rs`, add a `DatabaseConfig` struct with fields for Supabase URL, API key, connection pool settings, and include it in `CliptionsConfig`.
  - [ ] 2.2 Update `config/llm.yaml.template` with placeholder values for the new `database` section including Supabase connection details.
  - [ ] 2.3 Create a new Supabase project and define the database schema in SQL, ensuring tables match the fields in `RoundData`, `Participant`, and other structs from `src/types.rs`.
  - [ ] 2.4 In `src/round_engine/db.rs`, implement an async `DatabaseManager` struct that reads the database config from `ConfigManager` and manages async `sqlx` database connections (Postgres for validators, SQLite for miners).
  - [ ] 2.5 Implement async functions in `db.rs` to fetch the latest round from the database (Postgres/SQLite), determine its current state, and perform CRUD operations on rounds and participants using `sqlx`.
  - [ ] 2.6 Add async database migration/population script at `src/bin/populate_database.rs` to load existing `data/rounds.json` into the database (supports both Postgres and SQLite via `sqlx` migrations).

- [ ] 3.0 Implement Async Typestate State Machine
  - [ ] 3.1 In `src/round_engine/state_machine.rs`, define the empty state marker structs: `Pending`, `CommitmentsOpen`, `CommitmentsClosed`, `RevealsOpen`, `RevealsClosed`, `Payouts`, `Finished`.
  - [ ] 3.2 Implement the generic async `Round<S>` struct that wraps an instance of `crate::types::RoundData` and includes async database context.
  - [ ] 3.3 Implement async state transition methods that consume the round in the old state and return it in the new state, with database persistence: `async fn close_commitments(self) -> Result<Round<CommitmentsClosed>>`.
  - [ ] 3.4 Add async methods to query round state from database: `async fn fetch_current_state() -> Result<Round<impl State>>` and `async fn save_state(&self) -> Result<()>`.
  - [ ] 3.5 In `src/round_engine/sync_bridge.rs`, implement sync wrapper functions for each async state transition to maintain compatibility with existing Python bindings using `tokio::runtime::Runtime::block_on()`.
  - [ ] 3.6 Add error handling and logging throughout the async state machine, ensuring proper async error propagation.

- [ ] 4.0 Implement Role-Based Application Logic
  - [ ] 4.1 In `src/bin/cliptions_app.rs`, use `clap` to parse the `--role` command-line argument (validator/miner), with "miner" as the default.
  - [ ] 4.2 Initialize the async `ConfigManager` and `DatabaseManager`, setting up the tokio runtime and async connection pools.
  - [ ] 4.3 Implement the main async application loop that continuously polls the database for the current round's state using async intervals.
  - [ ] 4.4 **Validator Logic**: If role is `validator`, implement async workflow that matches on current state, prompts user for confirmation, and calls Python scripts using `tokio::process::Command` with proper async process handling.
  - [ ] 4.5 **Miner Logic**: If role is `miner`, implement async workflow that displays current round status and continuously polls for state changes using async timers and database queries.
  - [ ] 4.6 Add proper async signal handling (Ctrl+C) and graceful shutdown logic that cleanly closes database connections and running tasks.
  - [ ] 4.7 Implement async user interface helpers for validator confirmations and miner status display, ensuring non-blocking I/O operations.

- [ ] 5.0 Integration Testing and Validation
  - [ ] 5.1 In `src/round_engine/state_machine.rs`, add a `#[cfg(test)]` module with async unit tests (`#[tokio::test]`) confirming that typestate transitions work correctly and invalid operations fail to compile.
  - [ ] 5.2 Create the async integration test file `tests/round_engine_integration.rs` using `#[tokio::test]` for all async test functions.
  - [ ] 5.3 In the integration test, mock async database interactions and async calls to Python scripts, ensuring proper async test isolation.
  - [ ] 5.4 Write an async integration test that drives the state machine through a full round lifecycle, asserting the correct state at each async step.
  - [ ] 5.5 Add async tests for CLI argument parsing in `cliptions_app.rs`, testing both validator and miner modes with proper async runtime setup.
  - [ ] 5.6 Create specific tests for the sync bridge functions in `sync_bridge.rs`, ensuring they properly bridge async operations to sync interfaces for Python compatibility.
  - [ ] 5.7 Add async database integration tests that verify proper connection handling, state persistence, and error recovery in async database operations. 