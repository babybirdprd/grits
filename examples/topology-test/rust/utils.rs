// Example: Rust utils module that creates a circular dependency
// This file imports lib (validate_store), and lib imports this file (process_data)

use crate::{validate_store, DataStore};

pub fn process_data(input: &str) -> String {
    // Simulate some data processing
    let trimmed = input.trim().to_lowercase();
    format!("processed:{}", trimmed)
}

pub fn batch_process(store: &mut DataStore, items: Vec<String>) {
    // This creates a cycle: utils -> lib (validate_store) -> utils (process_data)
    if validate_store(store) {
        for item in items {
            let result = process_data(&item);
            store.items.push(result);
        }
    }
}

pub struct ProcessingConfig {
    pub max_items: usize,
    pub timeout_ms: u64,
}

impl ProcessingConfig {
    pub fn default() -> Self {
        Self {
            max_items: 1000,
            timeout_ms: 5000,
        }
    }
}
