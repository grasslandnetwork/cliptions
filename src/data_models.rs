use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Commitment {
    pub tweet_id: String,
    pub twitter_handle: String,
    pub commitment_hash: String,
    pub wallet_address: String,
    pub timestamp: String,
    #[serde(default)]
    pub fee_address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reveal {
    pub tweet_id: String,
    pub twitter_handle: String,
    pub guess: String,
    pub salt: String,
    pub timestamp: String,
    #[serde(default)]
    pub commitment_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RoundData {
    pub round_id: String,
    #[serde(default)]
    pub commitments: Vec<Commitment>,
    #[serde(default)]
    pub reveals: Vec<Reveal>,
} 