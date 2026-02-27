//! Legacy JSON storage format - used only for migration to SQLite.
//!
//! This module provides minimal functionality to load commands from the old
//! JSON-based storage format (~/.komando.json) and migrate them to the new
//! SQLite database (~/.komando.db).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

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
        let content = fs::read_to_string(path).context("Failed to read command store")?;
        if content.is_empty() {
            return Ok(Self::new());
        }
        serde_json::from_str(&content).context("Failed to parse command store")
    }

    /// List all commands for migration.
    pub fn list_all(&self) -> Vec<&Command> {
        self.commands.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let store = CommandStore::load(&temp_file.path().to_path_buf()).unwrap();
        assert_eq!(store.commands.len(), 0);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let path = PathBuf::from("/tmp/nonexistent_komando_test.json");
        let store = CommandStore::load(&path).unwrap();
        assert_eq!(store.commands.len(), 0);
    }

    #[test]
    fn test_load_valid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let json_content = r#"{
            "commands": [
                {
                    "id": "test-id-1",
                    "command": "ls -la",
                    "working_directory": "/home/test",
                    "group": "file",
                    "tags": ["list"],
                    "description": "List files",
                    "use_count": 5
                }
            ],
            "groups": ["file"],
            "tags": ["list"]
        }"#;

        temp_file.write_all(json_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let store = CommandStore::load(&temp_file.path().to_path_buf()).unwrap();
        assert_eq!(store.commands.len(), 1);
        assert_eq!(store.commands[0].command, "ls -la");
        assert_eq!(store.commands[0].get_id(), "test-id-1");
    }

    #[test]
    fn test_list_all() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let json_content = r#"{
            "commands": [
                {"id": "1", "command": "cmd1", "working_directory": "/tmp"},
                {"id": "2", "command": "cmd2", "working_directory": "/tmp"}
            ],
            "groups": [],
            "tags": []
        }"#;

        temp_file.write_all(json_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let store = CommandStore::load(&temp_file.path().to_path_buf()).unwrap();
        let all_commands = store.list_all();
        assert_eq!(all_commands.len(), 2);
    }
}
