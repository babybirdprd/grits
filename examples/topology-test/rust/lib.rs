// Example: Rust module with circular dependency
// This file imports utils, and utils imports this file back

mod utils;

use utils::process_data;

pub struct DataStore {
    pub items: Vec<String>,
    pub cache: std::collections::HashMap<String, i32>,
}

impl DataStore {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn add_item(&mut self, item: String) {
        let processed = process_data(&item);
        self.items.push(processed);
    }

    pub fn get_stats(&self) -> StoreStats {
        StoreStats {
            count: self.items.len(),
            cache_size: self.cache.len(),
        }
    }
}

pub struct StoreStats {
    pub count: usize,
    pub cache_size: usize,
}

// Function that utils.rs will call back to, creating a cycle
pub fn validate_store(store: &DataStore) -> bool {
    store.items.len() > 0
}
