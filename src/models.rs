//! src/models.rs
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "python", derive(FromPyObject))]
pub struct Commitment {
    pub username: String,
    pub commitment_hash: String,
    pub wallet_address: String,
    pub tweet_url: String,
    pub timestamp: String, // Using String for now for simplicity with Python's datetime strings
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "python", derive(FromPyObject))]
pub struct Round {
    pub round_id: String,
    pub announcement_url: String,
    pub livestream_url: String,
    pub entry_fee: f64,
    pub commitment_deadline: String,
    pub reveal_deadline: String,
    #[serde(default)]
    pub commitments: Vec<Commitment>,
}
