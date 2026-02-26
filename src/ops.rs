//! Legacy JSON storage format - used only for migration to SQLite.
//! 
//! This module provides minimal functionality to load commands from the old
//! JSON-based storage format (~/.komando.json) and migrate them to the new
//! SQLite database (~/.komando.db).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};

/// Command structure from the legacy JSON format.
/// All fields are preserved for deserialization compatibility with old JSON files.
#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    id: String,
    pub command: String,
    pub working_directory: String,
    #[serde(default)]
    group: String,
    #[serde(default)]
    tags: HashSet<String>,
    description: Option<String>,
    #[serde(default)]
    use_count: u32,
}

impl Command {
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

/// Legacy command store from JSON format.
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandStore {
    pub commands: Vec<Command>,
    #[serde(default)]
    groups: HashSet<String>,
    #[serde(default)]
    tags: HashSet<String>,
}

impl CommandStore {
    fn new() -> Self {
        CommandStore {
            commands: Vec::new(),
            groups: HashSet::new(),
            tags: HashSet::new(),
        }
    }

    /// Load commands from legacy JSON file for migration purposes.
    pub fn load(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(path)
            .context("Failed to read command store")?;
        if content.is_empty() {
            return Ok(Self::new());
        }
        serde_json::from_str(&content)
            .context("Failed to parse command store")
    }

    /// List all commands for migration.
    pub fn list_all(&self) -> Vec<&Command> {
        self.commands.iter().collect()
    }
}