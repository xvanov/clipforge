// Storage layer for ClipForge
// Handles persistence: SQLite cache, project files, and media storage

pub mod cache;

pub use cache::{CacheDb, initialize_cache};
