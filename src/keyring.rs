use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Serialize, Deserialize, Clone)]
pub struct KeyringEntry {
    pub agent_id: String,
    pub public_key: [u8; 32],
    pub added_at: u64,
}

pub struct Keyring {
    entries: HashMap<String, KeyringEntry>,
    path: PathBuf,
}

impl Keyring {
    pub fn load(path: PathBuf) -> Result<Self> {
        let entries = if path.exists() {
            let data = std::fs::read(&path)?;
            rmp_serde::from_slice(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };
        Ok(Self { entries, path })
    }

    pub fn add(&mut self, agent_id: String, public_key: [u8; 32]) -> Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        self.entries.insert(agent_id.clone(), KeyringEntry {
            agent_id,
            public_key,
            added_at: timestamp,
        });
        self.save()
    }

    pub fn get(&self, agent_id: &str) -> Option<&[u8; 32]> {
        self.entries.get(agent_id).map(|e| &e.public_key)
    }

    pub fn list(&self) -> Vec<&KeyringEntry> {
        self.entries.values().collect()
    }

    fn save(&self) -> Result<()> {
        let data = rmp_serde::to_vec(&self.entries)?;
        std::fs::write(&self.path, data)?;
        Ok(())
    }
}
