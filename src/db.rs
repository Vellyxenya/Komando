use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;
use chrono::Utc;

#[cfg(feature = "embeddings")]
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
#[cfg(feature = "embeddings")]
use rusqlite::ffi::sqlite3_auto_extension;
#[cfg(feature = "embeddings")]
use sqlite_vec::sqlite3_vec_init;

pub struct Db {
    conn: Connection,
}

impl Db {
    #[cfg(feature = "embeddings")]
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Initialize sqlite-vec extension
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }

        let conn = Connection::open(path)?;

        // Create commands table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS commands (
                id TEXT PRIMARY KEY,
                cmd TEXT NOT NULL,
                description TEXT,
                working_directory TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create cmd_embeddings virtual table
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS cmd_embeddings USING vec0(
                cmd_id TEXT PRIMARY KEY,
                embedding float[384]
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    #[cfg(not(feature = "embeddings"))]
    pub fn new<P: AsRef<Path>>(path: P) ->Result<Self> {
        let conn = Connection::open(path)?;

        // Create commands table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS commands (
                id TEXT PRIMARY KEY,
                cmd TEXT NOT NULL,
                description TEXT,
                working_directory TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    #[cfg(feature = "embeddings")]
    pub fn insert_command(&self, id: &str, cmd: &str, description: Option<&str>, working_dir: Option<&str>, embedding: &[f32]) -> Result<()> {
        let created_at = Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO commands (id, cmd, description, working_directory, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, cmd, description, working_dir, created_at],
        )?;

        // Convert &[f32] to bytes for sqlite-vec
        let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_ne_bytes()).collect();

        self.conn.execute(
            "INSERT INTO cmd_embeddings (cmd_id, embedding) VALUES (?1, ?2)",
            params![id, embedding_bytes],
        )?;

        Ok(())
    }

    #[cfg(not(feature = "embeddings"))]
    pub fn insert_command(&self, id: &str, cmd: &str, description: Option<&str>, working_dir: Option<&str>) -> Result<()> {
        let created_at = Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO commands (id, cmd, description, working_directory, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, cmd, description, working_dir, created_at],
        )?;

        Ok(())
    }

    #[cfg(feature = "embeddings")]
    pub fn search_commands(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<(String, String, f32)>> {
        let embedding_bytes: Vec<u8> = query_embedding.iter().flat_map(|f| f.to_ne_bytes()).collect();

        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.cmd, e.distance 
             FROM cmd_embeddings e
             JOIN commands c ON c.id = e.cmd_id
             WHERE e.embedding MATCH ?1 
               AND k = ?2
             ORDER BY e.distance"
        )?;

        let rows = stmt.query_map(params![embedding_bytes, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f32>(2)?,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    #[cfg(not(feature = "embeddings"))]
    pub fn search_commands(&self, query: &str, limit: usize) -> Result<Vec<(String, String)>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, cmd FROM commands WHERE cmd LIKE ?1 ORDER BY created_at DESC LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![search_pattern, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
    
    pub fn get_all_commands(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare("SELECT id, cmd FROM commands ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
            ))
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        
        Ok(results)
    }
    
    pub fn delete_command(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM commands WHERE id = ?1", params![id])?;
        #[cfg(feature = "embeddings")]
        self.conn.execute("DELETE FROM cmd_embeddings WHERE cmd_id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(feature = "embeddings")]
pub struct Embedder {
    model: TextEmbedding,
}

#[cfg(feature = "embeddings")]
impl Embedder {
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(true)
        )?;
        
        Ok(Self { model })
    }

    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.embed(vec![text], None)?;
        if let Some(embedding) = embeddings.into_iter().next() {
            Ok(embedding)
        } else {
            anyhow::bail!("Failed to generate embedding")
        }
    }
}
