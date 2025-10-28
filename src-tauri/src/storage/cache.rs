// SQLite cache database for media metadata and auto-saves
// Provides fast lookups and persistence for app state

use crate::models::clip::MediaClip;
use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Thread-safe wrapper for cache database
#[derive(Debug, Clone)]
pub struct CacheDb {
    conn: Arc<Mutex<Connection>>,
}

impl CacheDb {
    pub fn new(cache_path: &PathBuf) -> SqliteResult<Self> {
        let conn = initialize_cache(cache_path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn insert_media_clip(&self, clip: &MediaClip) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO media_clips 
             (id, name, source_path, proxy_path, thumbnail_path, duration, resolution, 
              width, height, fps, codec, audio_codec, file_size, bitrate, has_audio, imported_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            rusqlite::params![
                clip.id,
                clip.name,
                clip.source_path,
                clip.proxy_path,
                clip.thumbnail_path,
                clip.duration,
                clip.resolution,
                clip.width,
                clip.height,
                clip.fps,
                clip.codec,
                clip.audio_codec,
                clip.file_size,
                clip.bitrate,
                clip.has_audio,
                clip.imported_at.to_rfc3339(),
            ],
        )
        .map_err(|e| format!("Failed to insert media clip: {}", e))?;
        
        Ok(())
    }
}

/// Initialize the SQLite cache database
/// Creates the database file and sets up schema if it doesn't exist
pub fn initialize_cache(cache_path: &PathBuf) -> SqliteResult<Connection> {
    let conn = Connection::open(cache_path)?;
    
    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    
    create_schema(&conn)?;
    
    Ok(conn)
}

/// Create database schema (idempotent - safe to call multiple times)
fn create_schema(conn: &Connection) -> SqliteResult<()> {
    // Media clips metadata cache
    // Stores clip metadata for fast access without re-reading video files
    conn.execute(
        "CREATE TABLE IF NOT EXISTS media_clips (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            source_path TEXT NOT NULL,
            proxy_path TEXT,
            thumbnail_path TEXT,
            duration REAL NOT NULL,
            resolution TEXT NOT NULL,
            width INTEGER NOT NULL,
            height INTEGER NOT NULL,
            fps REAL NOT NULL,
            codec TEXT NOT NULL,
            audio_codec TEXT,
            file_size INTEGER NOT NULL,
            bitrate INTEGER,
            has_audio INTEGER NOT NULL,
            imported_at TEXT NOT NULL,
            UNIQUE(source_path)
        )",
        [],
    )?;

    // Auto-saves table
    // Stores periodic snapshots of project state for crash recovery
    conn.execute(
        "CREATE TABLE IF NOT EXISTS auto_saves (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id TEXT NOT NULL,
            project_name TEXT NOT NULL,
            saved_at TEXT NOT NULL,
            project_json TEXT NOT NULL,
            file_size INTEGER NOT NULL
        )",
        [],
    )?;

    // Index for fast auto-save queries (most recent first)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_auto_saves_project_id 
         ON auto_saves(project_id, saved_at DESC)",
        [],
    )?;

    Ok(())
}

/// Clean up old auto-saves (keep only last N saves per project)
pub fn cleanup_old_autosaves(conn: &Connection, project_id: &str, keep_count: usize) -> SqliteResult<usize> {
    conn.execute(
        "DELETE FROM auto_saves 
         WHERE project_id = ?1 
         AND id NOT IN (
             SELECT id FROM auto_saves 
             WHERE project_id = ?1 
             ORDER BY saved_at DESC 
             LIMIT ?2
         )",
        rusqlite::params![project_id, keep_count],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_initialize_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.db");
        
        let conn = initialize_cache(&cache_path).unwrap();
        
        // Verify tables were created
        let table_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('media_clips', 'auto_saves')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        
        assert_eq!(table_count, 2, "Should create 2 tables");
    }

    #[test]
    fn test_schema_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.db");
        
        // Initialize twice - should not error
        let _conn1 = initialize_cache(&cache_path).unwrap();
        let conn2 = initialize_cache(&cache_path).unwrap();
        
        // Verify schema still valid
        let table_count: i32 = conn2
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        
        assert!(table_count >= 2, "Tables should still exist");
    }

    #[test]
    fn test_cleanup_old_autosaves() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.db");
        let conn = initialize_cache(&cache_path).unwrap();
        
        let project_id = "test-project-123";
        
        // Insert 5 auto-saves
        for i in 0..5 {
            conn.execute(
                "INSERT INTO auto_saves (project_id, project_name, saved_at, project_json, file_size) 
                 VALUES (?1, ?2, datetime('now', ?3), ?4, ?5)",
                rusqlite::params![
                    project_id,
                    "Test Project",
                    format!("-{} seconds", 5 - i), // Make them chronologically ordered
                    "{}",
                    100
                ],
            ).unwrap();
        }
        
        // Keep only 3 most recent
        let deleted = cleanup_old_autosaves(&conn, project_id, 3).unwrap();
        assert_eq!(deleted, 2, "Should delete 2 old auto-saves");
        
        // Verify count
        let remaining: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM auto_saves WHERE project_id = ?1",
                rusqlite::params![project_id],
                |row| row.get(0),
            )
            .unwrap();
        
        assert_eq!(remaining, 3, "Should have 3 auto-saves remaining");
    }
}

