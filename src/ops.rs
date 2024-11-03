use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};


#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    id: String,  // UUID for unique identification
    command: String,
    working_directory: String,
    group: String,  // Primary group/category
    tags: HashSet<String>,  // Additional tags for flexible categorization
    description: Option<String>,
    use_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandStore {
    commands: Vec<Command>,
    groups: HashSet<String>,  // Track all unique groups
    tags: HashSet<String>,    // Track all unique tags
}

impl CommandStore {
    fn new() -> Self {
        CommandStore {
            commands: Vec::new(),
            groups: HashSet::new(),
            tags: HashSet::new(),
        }
    }

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

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize command store")?;
        fs::write(path, content)
            .context("Failed to write command store")
    }

    pub fn add_command(&mut self, 
        command: String, 
        working_dir: String,
        group: String, 
        tags: HashSet<String>,
        description: Option<String>
    ) -> Result<()> {
        let cmd = Command {
            id: uuid::Uuid::new_v4().to_string(),
            command,
            working_directory: working_dir,
            group: group.clone(),
            tags: tags.clone(),
            description,
            use_count: 1,
        };

        self.groups.insert(group);
        self.tags.extend(tags);
        self.commands.push(cmd);
        Ok(())
    }

    pub fn search(&self, query: &str, group: Option<&str>, tags: Option<&HashSet<String>>) -> Vec<&Command> {
        self.commands.iter()
            .filter(|cmd| {
                // Match group if specified
                if let Some(g) = group {
                    if cmd.group != g {
                        return false;
                    }
                }
                
                // Match any specified tags if any
                if let Some(t) = tags {
                    if !t.iter().any(|tag| cmd.tags.contains(tag)) {
                        return false;
                    }
                }

                // Match command text
                cmd.command.contains(query) || 
                cmd.description.as_ref().map_or(false, |d| d.contains(query))
            })
            .collect()
    }

    pub fn update_command(&mut self, id: &str, updates: CommandUpdates) -> Result<()> {
        // First find the command and apply updates
        if let Some(cmd) = self.commands.iter_mut().find(|c| c.id == id) {
            if let Some(new_command) = updates.command {
                cmd.command = new_command;
            }
            if let Some(new_desc) = updates.description {
                cmd.description = Some(new_desc);
            }
            
            let should_update_groups = updates.group.is_some();
            let should_update_tags = updates.tags.is_some();
            
            // Apply group and tags updates
            if let Some(new_group) = updates.group {
                cmd.group = new_group;
            }
            if let Some(new_tags) = updates.tags {
                cmd.tags = new_tags;
            }
            
            // Update collections after modifying the command
            if should_update_groups {
                self.groups = self.commands
                    .iter()
                    .map(|c| c.group.clone())
                    .collect();
            }
            
            if should_update_tags {
                self.tags = self.commands
                    .iter()
                    .flat_map(|c| c.tags.clone())
                    .collect();
            }
            
            Ok(())
        } else {
            anyhow::bail!("Command not found")
        }
    }

    pub fn increment_usage(&mut self, id: &str) -> Result<()> {
        if let Some(cmd) = self.commands.iter_mut().find(|c| c.id == id) {
            cmd.use_count += 1;
            Ok(())
        } else {
            anyhow::bail!("Command not found")
        }
    }
}

#[derive(Debug)]
pub struct CommandUpdates {
    command: Option<String>,
    group: Option<String>,
    tags: Option<HashSet<String>>,
    description: Option<String>,
}