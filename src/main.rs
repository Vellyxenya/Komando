use anyhow::Result;
use clap::{Arg, Command as ClapCommand};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType},
};
use dirs::home_dir;
use std::env;
use std::fs;
use std::io::{stdout, Write};

mod db;
mod ops;

use db::Db;
use ops::CommandStore;

#[cfg(feature = "embeddings")]
use db::Embedder;

fn get_last_commands(count: usize) -> Vec<String> {
    let file_content = fs::read_to_string("/tmp/last_commands.txt").ok();

    let content = if let Some(content) = file_content {
        content
    } else {
        return Vec::new();
    };

    // Process the commands - fc -ln output has no line numbers, just commands
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let cmd = line.trim();
            if !cmd.is_empty()
                && !cmd.starts_with("history")
                && !cmd.starts_with("komando")
                && !cmd.contains("komando_exec")
            {
                Some(cmd.to_string())
            } else {
                None
            }
        })
        .rev()
        .take(count)
        .collect()
}

fn main() -> Result<()> {
    // println!("Debug: Received arguments: {:?}", std::env::args().collect::<Vec<_>>());

    let matches = ClapCommand::new("Komando")
        .version("0.1.0")
        .author("Noureddine Gueddach")
        .about("A command line utility to better organize and keep track of your commands.")
        .arg(
            Arg::new("save")
                .short('s')
                .long("save")
                .help("Save the last command to a file")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("query")
                .short('q')
                .long("query")
                .value_name("QUERY")
                .help("Search for a command")
                .num_args(1),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .help("List all saved commands")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("delete")
                .short('d')
                .long("delete")
                .value_name("ID")
                .help("Delete a command by ID")
                .num_args(1),
        )
        .arg(
            Arg::new("count")
                .short('n')
                .long("number")
                .value_name("COUNT")
                .help("Number of commands to show (default: 5)")
                .value_parser(clap::value_parser!(usize))
                .default_value("5"),
        )
        .arg(
            Arg::new("init")
                .long("init")
                .help("Initialize shell integration")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let count = matches.get_one::<usize>("count").copied().unwrap_or(5);
    let last_commands = get_last_commands(count);

    if let Some(home_path) = home_dir() {
        if matches.get_flag("init") {
            println!("alias komando='eval \"$(komando_exec --query \\\"$@\\\")\"'");
            return Ok(());
        }

        let db_path = home_path.join(".komando.db");
        let json_path = home_path.join(".komando.json");

        let db = Db::new(&db_path)?;

        // Migration logic
        #[cfg(feature = "embeddings")]
        let mut embedder = Embedder::new()?;

        if json_path.exists() {
            if let Ok(store) = CommandStore::load(&json_path) {
                println!("Migrating commands from JSON to SQLite...");
                for cmd in store.list_all() {
                    #[cfg(feature = "embeddings")]
                    {
                        if let Ok(embedding) = embedder.embed(&cmd.command) {
                            let _ = db.insert_command(
                                cmd.get_id(),
                                &cmd.command,
                                None,
                                Some(&cmd.working_directory),
                                &embedding,
                            );
                        }
                    }
                    #[cfg(not(feature = "embeddings"))]
                    {
                        let _ = db.insert_command(
                            cmd.get_id(),
                            &cmd.command,
                            None,
                            Some(&cmd.working_directory),
                        );
                    }
                }
                // Rename the old file so we don't migrate again
                let _ = fs::rename(&json_path, home_path.join(".komando.json.bak"));
                println!("Migration complete.");
            }
        }

        let current_dir = env::current_dir()?;

        if matches.get_flag("save") {
            // Get the last command:
            let last_command = last_commands.first();
            if let Some(last_command) = last_command {
                let working_dir = current_dir.to_str().unwrap();
                let id = uuid::Uuid::new_v4().to_string();

                #[cfg(feature = "embeddings")]
                {
                    match embedder.embed(last_command) {
                        Ok(embedding) => {
                            match db.insert_command(
                                &id,
                                last_command,
                                None,
                                Some(working_dir),
                                &embedding,
                            ) {
                                Ok(_) => println!(
                                    ">>> Saved command: {} at {}",
                                    last_command, working_dir
                                ),
                                Err(e) => eprintln!(">>> Error saving command: {}", e),
                            }
                        }
                        Err(e) => eprintln!(">>> Error generating embedding: {}", e),
                    }
                }
                #[cfg(not(feature = "embeddings"))]
                {
                    match db.insert_command(&id, last_command, None, Some(working_dir)) {
                        Ok(_) => println!(">>> Saved command: {} at {}", last_command, working_dir),
                        Err(e) => eprintln!(">>> Error saving command: {}", e),
                    }
                }
            } else {
                eprintln!(">>> Error: No last command found to save. Please ensure /tmp/last_commands.txt contains valid command history.");
            }
            return Ok(());
        } else if matches.get_flag("list") {
            let commands = db.get_all_commands()?;

            if commands.is_empty() {
                println!("No saved commands found.");
            } else {
                println!("\n=== Saved Commands ===");
                for (id, cmd) in &commands {
                    println!("\nCommand: {}", cmd);
                    println!("ID: {}", id);
                }
                println!("\nTotal: {} command(s)\n", commands.len());
            }
            return Ok(());
        } else if let Some(id) = matches.get_one::<String>("delete") {
            match db.delete_command(id) {
                Ok(_) => println!(">>> Command deleted successfully"),
                Err(e) => eprintln!(">>> Error: {}", e),
            }
            return Ok(());
        } else if let Some(query) = matches.get_one::<String>("query") {
            #[cfg(feature = "embeddings")]
            let search_results = {
                let query_embedding = embedder.embed(query)?;
                db.search_commands(&query_embedding, 10)?
                    .into_iter()
                    .map(|(id, cmd, _dist)| (id, cmd))
                    .collect::<Vec<_>>()
            };

            #[cfg(not(feature = "embeddings"))]
            let search_results = db.search_commands(query, 10)?;

            if search_results.is_empty() {
                println!("No commands found matching '{}'", query);
                return Ok(());
            }

            // Interactive selection
            terminal::enable_raw_mode()?;
            let mut stdout = stdout();
            let mut selected = 0;

            loop {
                // Clear screen and reset cursor
                queue!(stdout, Clear(ClearType::All), MoveTo(0, 0), Hide)?;

                // Display commands
                for (i, (_, cmd)) in search_results.iter().enumerate() {
                    let prefix = if i == selected { "> " } else { "  " };
                    let number = format!("{}. ", i + 1);

                    queue!(
                        stdout,
                        MoveTo(0, i as u16),
                        Clear(ClearType::CurrentLine),
                        Print(prefix),
                        Print(number),
                        Print(cmd.as_str()),
                    )?;
                }

                queue!(
                    stdout,
                    MoveTo(0, search_results.len() as u16),
                    Print("Press 'Enter' to execute the selected command, 'Esc' to exit"),
                    Print("\n"),
                )?;

                stdout.flush()?;

                if let Event::Key(key_event) = event::read()? {
                    match key_event.code {
                        KeyCode::Up => {
                            selected = selected.saturating_sub(1);
                        }
                        KeyCode::Down => {
                            if selected < search_results.len() - 1 {
                                selected += 1;
                            }
                        }
                        KeyCode::Enter => {
                            let (_, cmd_text) = &search_results[selected];

                            eprintln!("{}", cmd_text);
                            break;
                        }
                        KeyCode::Esc => {
                            queue!(
                                stdout,
                                MoveTo(0, (search_results.len() + 1) as u16),
                                Clear(ClearType::CurrentLine),
                            )?;
                            break;
                        }
                        _ => {}
                    }
                }
            }

            // Disable raw mode and show cursor
            terminal::disable_raw_mode()?;
            execute!(stdout, Show)?;
        }
    } else {
        println!("Could not determine home directory.");
        // Exit early if we can't determine the home directory
        return Ok(());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_last_commands_filters_correctly() {
        // Create a temp file with test data
        let content = r#"ls -la
git commit -m 'test'
history
komando -s
docker ps
komando_exec --list
kubectl get pods
"#;

        std::fs::write("/tmp/last_commands.txt", content).unwrap();

        let commands = get_last_commands(10);

        // Should filter out history, komando commands
        assert!(
            !commands.iter().any(|c| c.contains("history")),
            "Should not include history command"
        );
        assert!(
            !commands.iter().any(|c| c.contains("komando")),
            "Should not include komando commands"
        );

        // Should include valid commands (note: commands are reversed, so latest first)
        let all_cmds = commands.join(" ");
        assert!(
            all_cmds.contains("ls -la") || commands.iter().any(|c| c == "ls -la"),
            "Should include ls -la"
        );
        assert!(
            all_cmds.contains("git commit") || commands.iter().any(|c| c.contains("git commit")),
            "Should include git commit"
        );
        assert!(
            all_cmds.contains("docker ps") || commands.iter().any(|c| c == "docker ps"),
            "Should include docker ps"
        );

        // Verify we got some commands
        assert!(!commands.is_empty(), "Should have at least some commands");

        // Clean up
        let _ = std::fs::remove_file("/tmp/last_commands.txt");
    }
}
