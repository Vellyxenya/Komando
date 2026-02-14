use clap::{Command as ClapCommand, Arg};
use crossterm::{
    cursor::{Hide, MoveTo, Show}, event::{self, Event, KeyCode}, execute, queue, style::Print, terminal::{self, Clear, ClearType}
};
use std::fs::{self, File};
use std::io::Write;
use std::env;
use dirs::home_dir;
use std::io::stdout;
use anyhow::Result;

mod ops;

use ops::CommandStore;



fn get_last_commands(count: usize) -> Vec<String> {   

    let file_content = fs::read_to_string("/tmp/last_commands.txt").ok();
    
    let content = if let Some(content) = file_content {
        content
    } else {
        return Vec::new();
    };

    // Process the commands
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.trim().splitn(2, ' ').collect();
            if parts.len() == 2 {
                let cmd = parts[1].trim();
                if !cmd.is_empty() && 
                   !cmd.starts_with("history") && 
                   !cmd.starts_with("komando") {
                    Some(cmd.to_string())
                } else {
                    None
                }
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
        .get_matches();

    let count = matches.get_one::<usize>("count").copied().unwrap_or(5);
    let last_commands = get_last_commands(count);

    // If the storage file does not exist, create it:
    if let Some(home_path) = home_dir() {
        let storage_path = home_path.join(".komando.json");
        if !storage_path.exists() {
            let mut file = File::create(&storage_path)?;
            file.write_all(b"")?;
        }

        let current_dir = env::current_dir()?;

        if matches.get_flag("save") {
            //Get the last command:
            let last_command = last_commands.first().unwrap();           

            let mut store = CommandStore::load(&storage_path)?;

            let working_dir = current_dir.to_str().unwrap();

            // Add a new command
            match store.add_command(
                last_command.to_string(),
                working_dir.to_string(),
                "default_group".to_string(),
                ["default_tag"].iter().map(|&s| s.to_string()).collect(),
                Some("".to_string()),
            ) {
                Ok(_) => {
                    println!(">>> Saved command: {} at {}", last_command, working_dir);
                    store.save(&storage_path)?;
                },
                Err(e) => {
                    eprintln!(">>> Warning: {}", e);
                    // Don't save if it's a duplicate
                }
            }

            return Ok(());
        } else if matches.get_flag("list") {
            let store = CommandStore::load(&storage_path)?;
            let commands = store.list_all();
            
            if commands.is_empty() {
                println!("No saved commands found.");
            } else {
                println!("\n=== Saved Commands ===");
                for cmd in &commands {
                    println!("\nCommand: {}", cmd.command);
                    println!("Directory: {}", cmd.working_directory);
                    println!("ID: {}", cmd.get_id());
                }
                println!("\nTotal: {} command(s)\n", commands.len());
            }
            return Ok(());
        } else if let Some(id) = matches.get_one::<String>("delete") {
            let mut store = CommandStore::load(&storage_path)?;
            
            match store.delete_command(id) {
                Ok(_) => {
                    println!(">>> Command deleted successfully");
                    store.save(&storage_path)?;
                },
                Err(e) => {
                    eprintln!(">>> Error: {}", e);
                }
            }
            return Ok(());
        } else if let Some(query) = matches.get_one::<String>("query") {
            let mut store = CommandStore::load(&storage_path)?;

            // Get command IDs from search results
            let command_ids: Vec<String> = {
                let search_results = store.search(query, None, None);
                search_results.iter().map(|c| c.get_id().to_string()).collect()
            };
            
            // Collect the commands into owned data before passing mutable reference
            let commands_data: Vec<(String, String, String)> = command_ids.iter()
                .filter_map(|id| {
                    store.commands.iter()
                        .find(|c| c.get_id() == id)
                        .map(|c| (c.get_id().to_string(), c.command.clone(), c.working_directory.clone()))
                })
                .collect();
            
            if commands_data.is_empty() {
                println!("No commands found matching '{}'", query);
                return Ok(());
            }
            
            // Interactive selection
            terminal::enable_raw_mode()?;
            let mut stdout = stdout();
            let mut selected = 0;

            loop {
                // Clear screen and reset cursor
                queue!(
                    stdout,
                    Clear(ClearType::All),
                    MoveTo(0, 0),
                    Hide
                )?;

                // Display commands
                for (i, (_, cmd, _)) in commands_data.iter().enumerate() {
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
                    MoveTo(0, commands_data.len() as u16),
                    Print("Press 'Enter' to execute the selected command, 'Esc' to exit"),
                    Print("\n"),
                )?;
                
                stdout.flush()?;

                if let Event::Key(key_event) = event::read()? {
                    match key_event.code {
                        KeyCode::Up => {
                            if selected > 0 {
                                selected -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if selected < commands_data.len() - 1 {
                                selected += 1;
                            }
                        }
                        KeyCode::Enter => {
                            let (cmd_id, cmd_text, cmd_dir) = &commands_data[selected];
                            
                            // Increment usage counter
                            let _ = store.increment_usage(cmd_id);
                            let _ = store.save(&storage_path);
                            
                            eprintln!("{};{}", cmd_dir, cmd_text);
                            break;
                        }
                        KeyCode::Esc => {
                            queue!(
                                stdout,
                                MoveTo(0, (commands_data.len() + 1) as u16),
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

fn interactively_process_commands(commands: Vec<&ops::Command>, store: &mut CommandStore, storage_path: &std::path::PathBuf) -> Result<()> {
    // Interactive command selection
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    let mut selected = 0;

    loop {
        // Clear screen and reset cursor
        queue!(
            stdout,
            Clear(ClearType::All),
            MoveTo(0, 0),
            Hide  // Hide cursor while displaying menu
        )?;

        // Display commands with proper formatting
        for (i, cmd) in commands.iter().enumerate() {
            let prefix = if i == selected { "> " } else { "  " };
            let number = format!("{}. ", i + 1);
            
            // Clear the entire line first
            queue!(
                stdout,
                MoveTo(0, i as u16),
                Clear(ClearType::CurrentLine),
                Print(prefix),
                Print(number),
                Print(cmd.command.as_str()),
            )?;
        }

        queue!(
            stdout,
            MoveTo(0, commands.len() as u16),
            Print("Press 'Enter' to execute the selected command, 'Esc' to exit"),
            Print("\n"),
        )?;
        
        // Make sure to flush the output
        stdout.flush()?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected < commands.len() - 1 {
                        selected += 1;
                    }
                }
                KeyCode::Enter => {
                    let cmd = &commands[selected];
                    let cmd_text = cmd.command.as_str();
                    let cmd_dir = cmd.working_directory.as_str();
                    let cmd_id = cmd.get_id();
                    
                    // Increment usage counter
                    let _ = store.increment_usage(cmd_id);
                    let _ = store.save(storage_path);
                    
                    eprintln!("{};{}", cmd_dir, cmd_text);
                    break;
                }
                KeyCode::Esc => {
                    queue!(
                        stdout,
                        MoveTo(0, (commands.len() + 1) as u16),
                        Clear(ClearType::CurrentLine),
                    )?;
                    break;
                }
                _ => {}
            }
        }
    }

    // Disable raw mode and show the cursor again
    terminal::disable_raw_mode()?;
    execute!(stdout, Show)?;

    Ok(())
}