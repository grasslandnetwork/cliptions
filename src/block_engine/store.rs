use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::config::PathManager;
use crate::error::{CliptionsError, Result};
use crate::error::BlockError;
use crate::block_engine::state_machine::{Block, CommitmentsOpen, StateMarker};

/// Minimal storage abstraction for typestate blocks
pub trait BlockStore {
	fn load_commitments_open(&self, num: &str) -> Result<Block<CommitmentsOpen>>;
	fn save<S: StateMarker>(&self, block: &Block<S>) -> Result<()>;
	fn list(&self) -> Result<Vec<String>>;
}

/// JSON-backed store using ~/.cliptions/data/blocks.json
pub struct JsonBlockStore {
	blocks_path: PathBuf,
}

impl JsonBlockStore {
	pub fn new() -> Result<Self> {
		let pm = PathManager::new()?;
		Ok(Self { blocks_path: pm.get_blocks_path() })
	}

	fn read_db(&self) -> Result<BTreeMap<String, Value>> {
		if !self.blocks_path.exists() {
			return Ok(BTreeMap::new());
		}
		let content = fs::read_to_string(&self.blocks_path)
			.map_err(|e| CliptionsError::Io(e))?;
		if content.trim().is_empty() {
			return Ok(BTreeMap::new());
		}
		let map: BTreeMap<String, Value> = serde_json::from_str(&content)
			.map_err(|e| CliptionsError::Json(e))?;
		Ok(map)
	}

	fn write_db(&self, map: &BTreeMap<String, Value>) -> Result<()> {
		let content = serde_json::to_string_pretty(map)
			.map_err(|e| CliptionsError::Json(e))?;
		fs::write(&self.blocks_path, content)
			.map_err(|e| CliptionsError::Io(e))?;
		Ok(())
	}
}

impl BlockStore for JsonBlockStore {
	fn load_commitments_open(&self, num: &str) -> Result<Block<CommitmentsOpen>> {
		let db = self.read_db()?;
		let Some(value) = db.get(num) else {
			return Err(CliptionsError::Block(BlockError::BlockNotFound { block_num: num.to_string() }));
		};
		let legacy: crate::types::BlockData = serde_json::from_value(value.clone())
			.map_err(|e| CliptionsError::Json(e))?;
		Ok((&legacy).into())
	}

	fn save<S: StateMarker>(&self, block: &Block<S>) -> Result<()> {
		// Round-trip through legacy DTO to preserve unknown fields via template strategy
		let mut db = self.read_db()?;
		let template = if let Some(existing) = db.get(&block.block_num) {
			serde_json::from_value::<crate::types::BlockData>(existing.clone())
				.unwrap_or_else(|_| crate::types::BlockData::new(block.block_num.clone(), String::new(), String::new(), 0.0))
		} else {
			crate::types::BlockData::new(block.block_num.clone(), String::new(), String::new(), 0.0)
		};
		// Only CommitmentsOpen supported for now; extend later as needed
		let legacy = Block::<CommitmentsOpen> {
			block_num: block.block_num.clone(),
			created_at: block.created_at,
			description: block.description.clone(),
			livestream_url: block.livestream_url.clone(),
			target_timestamp: block.target_timestamp,
			target_frame_path: block.target_frame_path.clone(),
			commitment_deadline: block.commitment_deadline,
			reveals_deadline: block.reveals_deadline,
			participants: block.participants.clone(),
			prize_pool: block.prize_pool,
			total_payout: block.total_payout,
			state: std::marker::PhantomData,
		}.to_legacy_with_template(&template);
		let value = serde_json::to_value(&legacy)
			.map_err(|e| CliptionsError::Json(e))?;
		db.insert(block.block_num.clone(), value);
		self.write_db(&db)
	}

	fn list(&self) -> Result<Vec<String>> {
		let db = self.read_db()?;
		Ok(db.keys().cloned().collect())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::TempDir;
	use std::fs;
	use chrono::Utc;

	#[test]
	fn json_store_load_save_round_trip() {
		// Prepare temp file
		let tmp = TempDir::new().unwrap();
		let db_path = tmp.path().join("blocks.json");

		// Build a store pointing to temp path by overriding PathManager via env? Quick hack: instantiate and then overwrite path.
		let mut store = JsonBlockStore::new().unwrap();
		store.blocks_path = db_path.clone();

		// Start with empty DB
		let mut block = Block::<CommitmentsOpen>::start(
			"101".to_string(),
			"desc".to_string(),
			"http://live".to_string(),
			Utc::now(),
			Utc::now(),
		);
		block.prize_pool = 33.0;
		block.total_payout = 0.0;
		store.save(&block).unwrap();

		let loaded = store.load_commitments_open("101").unwrap();
		assert_eq!(loaded.block_num, "101");
		assert_eq!(loaded.prize_pool, 33.0);
		assert_eq!(loaded.total_payout, 0.0);

		let keys = store.list().unwrap();
		assert!(keys.contains(&"101".to_string()));

		// Inspect raw JSON
		let raw = fs::read_to_string(&db_path).unwrap();
		assert!(raw.contains("\"101\""));
	}
}
