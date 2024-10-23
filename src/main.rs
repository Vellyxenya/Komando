use clap::{Command as ClapCommand, Arg};
use std::process::{Command, Stdio};
use std::io::Write;

fn get_last_command() -> Option<String> {
    // Create a new Command for bash in interactive mode
    let mut child = Command::new("bash")
        .arg("-i")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())  // Suppress stderr as bash might print some startup messages
        .spawn()
        .ok()?;

    // Write the history command to stdin
    // Get the last 3 commands to account for both 'history' and 'exit' commands
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"history 3\n").ok()?;
        stdin.write_all(b"exit\n").ok()?;
    }

    // Get the output
    let output = child.wait_with_output().ok()?;
    
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        // Get all non-empty lines
        let lines: Vec<&str> = output_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        // Get the first line (which should be the command before our 'history' command)
        if let Some(line) = lines.first() {
            // Parse out the command (skip the history number)
            let command = line
                .split_whitespace()
                .skip(1)  // Skip the history number
                .collect::<Vec<&str>>()
                .join(" ");
            
            if !command.is_empty() && !command.starts_with("history") {
                Some(command)
            } else if lines.len() > 1 {
                // If the first command was 'history', try the second line
                let command = lines[1]
                    .split_whitespace()
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join(" ");
                
                if !command.is_empty() && !command.starts_with("history") {
                    Some(command)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn main() {
    let matches = ClapCommand::new("Komando")
        .version("0.1.0")
        .author("Noureddine Gueddach")
        .about("A command line utility to better organize and keep track of your commands.")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Sets an input file")
                .num_args(1),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Print additional information")
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .help("Print debug information about history commands")
        )
        .get_matches();

    let debug = matches.contains_id("debug");

    // If debug mode is enabled, show raw output
    if debug {
        println!("Attempting to get history through interactive shell:");
        
        if let Ok(mut child) = Command::new("bash")
            .arg("-i")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn() {
            
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(b"history 3\nexit\n");
            }
            
            if let Ok(output) = child.wait_with_output() {
                println!("Raw output:\n{}", String::from_utf8_lossy(&output.stdout));
            }
        }
    }

    match get_last_command() {
        Some(cmd) => println!("Last command: {}", cmd),
        None => println!("No last command found. Make sure you have bash history enabled."),
    }

    if let Some(input_file) = matches.get_one::<String>("input") {
        println!("Input file: {}", input_file);
    }
}