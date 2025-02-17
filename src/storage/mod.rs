use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::StorageConfig;

pub struct Storage {
    data: HashMap<String, String>,
    config: StorageConfig,
    current_memory: usize,
}

impl Storage {
    pub fn new(config: StorageConfig) -> Self {
        Storage {
            data: HashMap::new(),
            config,
            current_memory: 0,
        }
    }

    pub fn insert(&mut self, key: String, value: String) -> bool {
        let entry_size = key.len() + value.len();
        
        // Check if we would exceed memory limit
        if self.current_memory + entry_size > self.config.max_memory {
            return false;
        }

        // Update memory usage
        if let Some(old_value) = self.data.get(&key) {
            self.current_memory -= key.len() + old_value.len();
        }
        self.current_memory += entry_size;

        self.data.insert(key, value);
        true
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn memory_usage(&self) -> usize {
        self.current_memory
    }

    pub fn is_persistence_enabled(&self) -> bool {
        self.config.persistence_enabled
    }

    pub fn save_to_disk(&self) -> std::io::Result<()> {
        if !self.config.persistence_enabled {
            return Ok(());
        }
        let data = serde_json::to_string(&self.data)?;
        std::fs::write("dump.rdb", data)
    }

    pub fn load_from_disk(&mut self) -> std::io::Result<()> {
        if !self.config.persistence_enabled {
            return Ok(());
        }
        if let Ok(data) = std::fs::read_to_string("dump.rdb") {
            self.data = serde_json::from_str(&data)?;
            self.current_memory = self.data.iter()
                .map(|(k, v)| k.len() + v.len())
                .sum();
        }
        Ok(())
    }
}

pub type Db = Arc<Mutex<Storage>>;
