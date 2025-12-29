pub mod fs;
pub mod git;
pub mod merge;
pub mod models;
pub mod search;
pub mod store;
pub mod strategic;
pub mod sync;
pub mod topology;
pub mod util;

#[cfg(not(target_arch = "wasm32"))]
pub mod context;

pub use models::*;
pub use store::Store;

#[cfg(not(target_arch = "wasm32"))]
pub use store::SqliteStore;

pub use git::GitOps;
#[cfg(not(target_arch = "wasm32"))]
pub use git::StdGit;

pub use fs::FileSystem;
#[cfg(not(target_arch = "wasm32"))]
pub use fs::StdFileSystem;

pub mod memory_store;
pub use memory_store::MemoryStore;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::{WasmFileSystem, WasmGit};
