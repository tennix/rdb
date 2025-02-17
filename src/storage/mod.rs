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
}

pub type Db = Arc<Mutex<Storage>>;
