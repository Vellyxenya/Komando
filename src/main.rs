use clap::{Command as ClapCommand, Arg};
use crossterm::{
    cursor::{Hide, MoveTo, Show}, event::{self, Event, KeyCode}, execute, queue, style::Print, terminal::{self, Clear, ClearType}
};
use std::os::unix::fs::PermissionsExt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::env;
use dirs::home_dir;
use std::io::stdout;
use anyhow::Result;

mod ops;

use ops::CommandStore;

const SHELL_SCRIPT: &str = r#"#!/bin/bash
history > /tmp/last_commands.txt
"#;

const SHELL_FUNCTION: &str = r#"
komando() {
    history > /tmp/last_commands.txt
    RUST_PROGRAM="./target/debug/komando_executable"
    if [ -x "$RUST_PROGRAM" ]; then
        OUTPUT=$("$RUST_PROGRAM" "$@" 2>&1 1>/dev/tty)
        
        if [ -z "$OUTPUT" ]; then
            return
        fi

        IFS=';' read -r DIR CMD <<< "$OUTPUT"
        echo ""
        echo "=========== Edit the command and then hit 'Enter' ==========="
        echo "Directory: $DIR"
        echo "Command:"
        
        read -e -i "$CMD" COMMAND
        echo ""
        
        if [ -n "$COMMAND" ]; then
            cd "$DIR" && eval "$COMMAND"
        fi
    else
        echo "Error: Komando executable not found"
    fi
}
"#;

fn setup_shell_integration() -> std::io::Result<()> {
    // Create shell script
    let script_path = "/tmp/komando_history.sh";
    let mut file = File::create(script_path)?;
    file.write_all(SHELL_SCRIPT.as_bytes())?;
    fs::set_permissions(script_path, fs::Permissions::from_mode(0o755))?;

    // Detect shell type
    let shell = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
    let rc_file = if shell.contains("zsh") {
        PathBuf::from(env::var("HOME").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?).join(".zshrc")
    } else {
        PathBuf::from(env::var("HOME").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?).join(".bashrc")
    };

    // Check if integration is already set up
    if let Ok(content) = fs::read_to_string(&rc_file) {
        if content.contains("komando()") {
            return Ok(());
        }
    }

    // Add shell function to rc file if user approves
    println!("Would you like to set up shell integration for easier command history access? (y/N)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() == "y" {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(rc_file)?;
        writeln!(file, "\n# Komando shell integration")?;
        writeln!(file, "{}", SHELL_FUNCTION)?;
        println!("Shell integration installed. Please restart your shell or run 'source ~/.bashrc' (or ~/.zshrc)");
    }

    Ok(())
}

fn get_last_commands(count: usize) -> Vec<String> {   

    let file_content = fs::read_to_string("/tmp/last_commands.txt").ok();
    
    let content = if let Some(content) = file_content {
        content
    } else {
        println!("No file content");
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
            Arg::new("setup")
                .long("setup")
                .help("Set up shell integration")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("save")
                .short('s')
                .long("save")
                .help("Save the last command to a file")
                .action(clap::ArgAction::SetTrue),
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

    if matches.get_flag("setup") {
        setup_shell_integration()?;
        println!("Setup complete. Run 'komando' to access your command history.");
        return Ok(());
    }

    let count = matches.get_one::<usize>("count").copied().unwrap_or(5);
    let commands = get_last_commands(count);

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
            let last_command = commands.first().unwrap();           

            let mut store = CommandStore::load(&storage_path)?;

            let working_dir = current_dir.to_str().unwrap();

            // Add a new command
            store.add_command(
                last_command.to_string(),
                working_dir.to_string(),
                "default_group".to_string(),
                ["default_tag"].iter().map(|&s| s.to_string()).collect(),
                Some("".to_string()),
            )?;

            println!(">>> Saved command: {} at {}", last_command, working_dir);

            store.save(&storage_path)?;
            return Ok(());
        }
    } else {
        println!("Could not determine home directory.");
        // Exit early if we can't determine the home directory
        return Ok(());
    }

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
                Print(cmd),
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
                    let dir = "."; // default to current directory
                    let cmd = &commands[selected];
                    eprintln!("{};{}", dir, cmd);
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