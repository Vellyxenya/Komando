// use std::process::Command;
use std::env;
use std::path::Path;
use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};

const SHELL_FUNCTION: &str = r#"
komando() {
    history > /tmp/last_commands.txt
    RUST_PROGRAM="komando_exec"
    if command -v "$RUST_PROGRAM" > /dev/null 2>&1; then
        OUTPUT=$("$RUST_PROGRAM" "$@" 2>&1 1>/dev/tty)
        
        if [ -z "$OUTPUT" ]; then
            return
        fi

        # Check if the output contains a semicolon
        if ! echo "$OUTPUT" | grep -q ";"; then
            echo "$OUTPUT"
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
            echo "Executing '$COMMAND'..."
            cd "$DIR" && eval "$COMMAND"
        fi
    else
        echo "Error: Komando executable not found"
    fi
}
"#;

fn setup_shell_integration() -> std::io::Result<()> {
    // Detect shell type
    let shell = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
    let rc_file = if shell.contains("zsh") {
        PathBuf::from(env::var("HOME").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?).join(".zshrc")
    } else {
        PathBuf::from(env::var("HOME").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?).join(".bashrc")
    };

    // Check if integration is already set up
    if let Ok(content) = fs::read_to_string(&rc_file) {
        println!("cargo:warning=Checking for komando() in {}", rc_file.display());
        if content.contains("komando()") {
            println!("cargo:warning=Content: {}", content);
            println!("cargo:warning=Shell integration already set up");
            return Ok(());
        }
    }

    // Add shell function to rc file
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(rc_file)?;
    writeln!(file, "\n# Komando shell integration")?;
    writeln!(file, "{}", SHELL_FUNCTION)?;
    println!("cargo:warning=Shell integration installed. Please restart your shell or run 'source ~/.bashrc' (or ~/.zshrc)");

    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Run pre-install setup
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "linux" {
        // Create necessary directories
        let home_dir = env::var("HOME").unwrap();
        let config_dir = Path::new(&home_dir).join(".config").join("komando");
        
        std::fs::create_dir_all(&config_dir).unwrap_or_else(|err| {
            println!("cargo:warning=Failed to create config directory: {}", err);
        });
    }

    setup_shell_integration().unwrap_or_else(|err| {
        println!("cargo:warning=Failed to set up shell integration: {}", err);
    });
    
    println!("cargo:warning=Pre-install setup completed");
}