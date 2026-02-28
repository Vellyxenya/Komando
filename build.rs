use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "embeddings")]
use std::process::Command;

const SHELL_FUNCTION: &str = r#"
# Komando Shell Integration v2
komando() {
    # Capture recent history using fc (works in bash and zsh)
    fc -ln -50 -1 | sed 's/^[[:space:]]*//' > /tmp/last_commands.txt
    RUST_PROGRAM="komando_exec"
    if command -v "$RUST_PROGRAM" > /dev/null 2>&1; then
        # Capture stdout (the command to run) while letting stderr (UI) go to the terminal
        OUTPUT=$("$RUST_PROGRAM" "$@")
        
        if [ -z "$OUTPUT" ]; then
            return
        fi

        # Check if the output starts with special prefix
        if [[ "$OUTPUT" != "KOMANDO_EXEC:"* ]]; then
            echo "$OUTPUT"
            return
        fi

        # Remove prefix
        OUTPUT="${OUTPUT#KOMANDO_EXEC:}"
        
        # Split by first semicolon
        DIR="${OUTPUT%%;*}"
        CMD="${OUTPUT#*;}"
        
        echo ""
        echo "=========== Edit the command and then hit 'Enter' ==========="
        echo "Directory: $DIR"
        echo "Command:"
        
        if [ -n "$BASH_VERSION" ]; then
            read -e -i "$CMD" -p "" COMMAND
        elif [ -n "$ZSH_VERSION" ]; then
            # Zsh interactive editing
            COMMAND="$CMD"
            vared -p "" -c COMMAND
        else
            # Fallback for other shells (sh, dash, etc)
            echo "  (Edit above command and press Enter)"
            echo "Current: $CMD"
            read -r COMMAND
            if [ -z "$COMMAND" ]; then
                COMMAND="$CMD"
            fi
        fi
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
        PathBuf::from(env::var("HOME").map_err(io::Error::other)?).join(".zshrc")
    } else {
        PathBuf::from(env::var("HOME").map_err(io::Error::other)?).join(".bashrc")
    };

    // Check if integration is already set up
    if let Ok(content) = fs::read_to_string(&rc_file) {
        println!(
            "cargo:warning=Checking for komando() in {}",
            rc_file.display()
        );
        if content.contains("Komando Shell Integration v2") {
            println!("cargo:warning=Content: {}", content);
            println!("cargo:warning=Shell integration already set up");
            return Ok(());
        }
    }

    // Add shell function to rc file
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(rc_file)?;
    writeln!(file, "\n# Komando shell integration")?;
    writeln!(file, "{}", SHELL_FUNCTION)?;
    println!("cargo:warning=Shell integration installed. Please restart your shell or run 'source ~/.bashrc' (or ~/.zshrc)");

    Ok(())
}

fn setup_onnx_runtime() -> std::io::Result<()> {
    #[cfg(feature = "embeddings")]
    {
        println!("cargo:warning=Embeddings feature enabled, checking ONNX Runtime...");

        let home_dir = env::var("HOME").map_err(io::Error::other)?;
        let ort_dir = PathBuf::from(&home_dir).join(".onnxruntime");
        let ort_version = "1.23.2";
        let ort_extracted_dir = ort_dir.join(format!("onnxruntime-linux-x64-{}", ort_version));
        let ort_lib = ort_extracted_dir.join("lib").join("libonnxruntime.so");

        // Check if already installed
        if ort_lib.exists() {
            println!(
                "cargo:warning=ONNX Runtime already installed at {}",
                ort_lib.display()
            );
            return Ok(());
        }

        println!(
            "cargo:warning=ONNX Runtime not found, downloading version {}...",
            ort_version
        );

        // Create .onnxruntime directory
        fs::create_dir_all(&ort_dir)?;

        // Download ONNX Runtime
        let download_url = format!(
            "https://github.com/microsoft/onnxruntime/releases/download/v{}/onnxruntime-linux-x64-{}.tgz",
            ort_version, ort_version
        );
        let archive_path = ort_dir.join(format!("onnxruntime-{}.tgz", ort_version));

        println!("cargo:warning=Downloading from {}...", download_url);

        // Use curl to download (available in dev container)
        let status = Command::new("curl")
            .args([
                "-L", // Follow redirects
                "-o",
                archive_path.to_str().unwrap(),
                &download_url,
            ])
            .status()?;

        if !status.success() {
            return Err(io::Error::other("Failed to download ONNX Runtime"));
        }

        println!("cargo:warning=Download complete, extracting...");

        // Extract the archive
        let status = Command::new("tar")
            .args([
                "-xzf",
                archive_path.to_str().unwrap(),
                "-C",
                ort_dir.to_str().unwrap(),
            ])
            .status()?;

        if !status.success() {
            return Err(io::Error::other("Failed to extract ONNX Runtime"));
        }

        // Clean up the archive
        let _ = fs::remove_file(&archive_path);

        println!(
            "cargo:warning=ONNX Runtime installed successfully to {}",
            ort_extracted_dir.display()
        );
        println!(
            "cargo:warning=IMPORTANT: Add this to your shell RC file (~/.bashrc or ~/.zshrc):"
        );
        println!("cargo:warning=  export ORT_DYLIB_PATH=~/.onnxruntime/onnxruntime-linux-x64-{}/lib/libonnxruntime.so", ort_version);
        println!("cargo:warning=  export LD_LIBRARY_PATH=~/.onnxruntime/onnxruntime-linux-x64-{}/lib:$LD_LIBRARY_PATH", ort_version);
    }

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

    // Setup ONNX Runtime for embeddings feature
    setup_onnx_runtime().unwrap_or_else(|err| {
        println!("cargo:warning=Failed to set up ONNX Runtime: {}", err);
        println!("cargo:warning=You can manually run: ./setup_embeddings.sh");
    });

    println!("cargo:warning=Pre-install setup completed");
}
