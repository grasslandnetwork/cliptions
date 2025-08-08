//! Facade modules for stable, future-proof access to domain types.
//!
//! This module provides facades around internal structs (e.g., `BlockData`,
//! `Participant`, `Guess`, and typestate `Block<S>`) so that callers depend on
//! stable accessor methods rather than concrete fields.

pub mod block_facade;
pub mod participant_facade;
pub mod guess_facade;


