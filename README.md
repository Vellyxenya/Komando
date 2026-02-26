# Komando
A command line utility to help you organize and easily access your commands with SQLite-based storage and optional semantic search.

## Features
- 💾 **SQLite Storage**: Fast, reliable database for your command history
- 🔍 **Smart Search**: Pattern-based search (default) or semantic embeddings (optional)
- 📊 **Usage Tracking**: Tracks command usage for better organization
- 🔄 **Auto Migration**: Seamlessly migrates from legacy JSON storage
- ⚡ **Interactive Selection**: Navigate commands with arrow keys

# Installation

## Install from `Crates.io`
### Standard Version (Pattern-Based Search)
```bash
cargo install komando
```

### With Semantic Embeddings (Optional)
To enable AI-powered semantic search:
```bash
# First, set up ONNX Runtime (one-time setup)
./setup_embeddings.sh
source ~/.bashrc  # or ~/.zshrc

# Then install with embeddings feature
cargo install komando --features embeddings
```

Then source your shell configuration:
```bash
source ~/.bashrc  # or ~/.zshrc
```

Verify the installation works by running:
```bash
komando --help
```

# Build from Source

## Quick Install (Recommended)
Use the install script that builds and installs automatically:

```bash
# Standard build (fast pattern matching)
./install.sh

# Or with semantic embeddings (AI-powered search)
./install.sh --embeddings
```

## Manual Build

### Standard Build
Uses fast SQL pattern matching (LIKE queries) - works on any system:
```bash
cargo build --release
sudo cp target/release/komando_exec /usr/local/bin/
```

### Build with Semantic Embeddings
Enables AI-powered semantic search using vector embeddings:

**Requirements:**
- GLIBC 2.27 or higher (Ubuntu 18.04+, Debian 10+, etc.)
- ~100MB disk space for embedding model (downloaded on first use)

**Setup:**
```bash
# One-time setup: Download and configure ONNX Runtime
./setup_embeddings.sh
source ~/.bashrc  # or ~/.zshrc

# Build with embeddings feature
cargo build --release --features embeddings
sudo cp target/release/komando_exec /usr/local/bin/
```

**What's the difference?**
- **Standard (Pattern-based)**: Searches for exact text matches in commands. Fast and reliable.
- **Embeddings (Semantic)**: Understands meaning - e.g., searching "containers" finds `docker ps` and `kubectl get pods` even though "containers" doesn't appear in either command.

And don't forget to source your shell configuration:
```bash
source ~/.bashrc  # or ~/.zshrc
```

Then verify the installation by running:
```bash
komando --help
```

# Usage

## Initial Setup (Shell Integration)
For the best experience, set up the shell alias:
```bash
komando --init
```
This will output an alias command that you can add to your shell configuration file.

Or add it automatically:
```bash
komando --init >> ~/.bashrc  # or ~/.zshrc
source ~/.bashrc
```

## Saving a command
Run:
```bash
komando --save
```
This will save the previous command entered in the terminal, along with the directory at which it was run.

**Note:** Komando automatically detects duplicate commands in the same directory and will warn you if you try to save the same command twice.

## Listing all saved commands
Run:
```bash
komando --list
```
This will display all your saved commands with their directories and unique IDs.

## Searching and executing a command
Run:
```bash
komando --query <YOUR_QUERY>
```

**Pattern-based search (default):** Searches for commands containing your query text.

**Semantic search (with embeddings):** Understands meaning and context. Examples:
- Query `"containers"` → finds `docker ps`, `kubectl get pods`
- Query `"install packages"` → finds `npm install`, `pip install`
- Query `"version control"` → finds `git commit`, `git push`

You will be presented with an interactive terminal where you can choose the command using the `UP` and `DOWN` arrows.

After choosing the command with `ENTER`, you can execute it immediately.

**Note:** Each time you execute a command through komando, it tracks usage statistics for potential future features.

## Deleting a command
Run:
```bash
komando --delete <COMMAND_ID>
```
This will delete the command with the specified ID. You can find command IDs using `komando --list`.

# Storage and Data

Komando stores your commands in an SQLite database at `~/.komando.db`.

**Automatic Migration:** If you're upgrading from an older version that used JSON storage (`~/.komando.json`), Komando will automatically migrate your commands to the new database format on first run. Your old JSON file will be backed up as `~/.komando.json.bak`.

# Troubleshooting

## Embeddings Not Working

If you get errors when using the embeddings feature:

1. **Check GLIBC version:**
   ```bash
   ldd --version
   ```
   You need GLIBC 2.27 or higher.

2. **Verify ONNX Runtime setup:**
   ```bash
   echo $ORT_DYLIB_PATH
   echo $LD_LIBRARY_PATH
   ```
   These should point to your ONNX Runtime installation.

3. **Re-run setup:**
   ```bash
   ./setup_embeddings.sh
   source ~/.bashrc  # or ~/.zshrc
   ```

4. **Check library compatibility:**
   ```bash
   objdump -T ~/.onnxruntime/onnxruntime-linux-x64-1.23.2/lib/libonnxruntime.so.1.23.2 | grep GLIBC | sed 's/.*GLIBC_/GLIBC_/' | sort -Vr | head -1
   ```
   Should show GLIBC_2.27 or lower.

## Fallback to Pattern-Based Search

If embeddings are causing issues, you can always rebuild without them:
```bash
cargo build --release  # Without --features embeddings
```
Pattern-based search is fast and reliable for most use cases.

# License
This project is licensed under the MIT License.