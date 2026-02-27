# Komando

[![CI](https://github.com/Vellyxenya/Komando/workflows/CI/badge.svg)](https://github.com/Vellyxenya/Komando/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/komando.svg)](https://crates.io/crates/komando)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

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
# Install with embeddings feature - ONNX Runtime downloads automatically
cargo install komando --features embeddings
```

**Note:** The build process will automatically download and set up ONNX Runtime (v1.23.2, ~70MB) during installation. After installation, add these environment variables to your shell RC file (`~/.bashrc` or `~/.zshrc`):

```bash
export ORT_DYLIB_PATH=~/.onnxruntime/onnxruntime-linux-x64-1.23.2/lib/libonnxruntime.so
export LD_LIBRARY_PATH=~/.onnxruntime/onnxruntime-linux-x64-1.23.2/lib:$LD_LIBRARY_PATH
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
- ~70MB for ONNX Runtime (auto-downloaded during build)

**Setup:**
```bash
# Build with embeddings feature - ONNX Runtime downloads automatically
cargo build --release --features embeddings
sudo cp target/release/komando_exec /usr/local/bin/
```

After building, add these environment variables to your shell RC file (`~/.bashrc` or `~/.zshrc`):
```bash
export ORT_DYLIB_PATH=~/.onnxruntime/onnxruntime-linux-x64-1.23.2/lib/libonnxruntime.so
export LD_LIBRARY_PATH=~/.onnxruntime/onnxruntime-linux-x64-1.23.2/lib:$LD_LIBRARY_PATH
```

**Note:** The setup_embeddings.sh script is still available if you prefer manual setup or encounter issues with the automatic download.

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

3. **Check if ONNX Runtime is installed:**
   ```bash
   ls -la ~/.onnxruntime/onnxruntime-linux-x64-1.23.2/lib/libonnxruntime.so
   ```
   If not found, it should have been auto-downloaded during build. You can manually run:
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

# Development

## Setting Up Development Environment

1. Clone the repository:
   ```bash
   git clone https://github.com/Vellyxenya/Komando.git
   cd Komando
   ```

2. Install pre-commit hooks:
   ```bash
   ./setup_hooks.sh
   ```

   This installs git hooks that automatically run before each commit:
   - `cargo fmt` - Code formatting check
   - `cargo clippy` - Linting with warnings as errors
   - `cargo test` - All unit tests
   - `cargo check` - Build verification

   **Alternative: Using pre-commit framework**
   ```bash
   pip install pre-commit
   pre-commit install
   ```

## Running Tests

```bash
# Run all tests (standard features)
cargo test

# Run tests with all features
cargo test --all-features

# Run specific test
cargo test test_name
```

## Code Quality

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Check without building
cargo check --all-targets --all-features
```

## Building

```bash
# Standard build
./install.sh

# With embeddings
./install.sh --embeddings
```

## Release Process

This project uses [cargo-release](https://github.com/crate-ci/cargo-release) for automated releases with a single source of truth (Cargo.toml).

### Making a Release

1. **Install cargo-release** (one-time setup):
   ```bash
   cargo install cargo-release
   ```

2. **Update CHANGELOG.md**:
   - Add changes under the `[Unreleased]` section
   - cargo-release will automatically create a version section

3. **Run cargo release**:
   ```bash
   # Dry run to preview changes
   cargo release --dry-run
   
   # Patch release (0.1.1 -> 0.1.2)
   cargo release patch
   
   # Minor release (0.1.1 -> 0.2.0)
   cargo release minor
   
   # Major release (0.1.1 -> 1.0.0)
   cargo release major
   
   # Pre-release (alpha, beta, rc)
   cargo release --pre-release alpha
   cargo release --pre-release beta
   cargo release --pre-release rc
   ```

4. **cargo-release will**:
   - Run tests to ensure everything works
   - Update version in Cargo.toml
   - Update CHANGELOG.md with the version and date
   - Create a commit with message: `chore: release komando v{version}`
   - Create a git tag: `v{version}`
   - Push commit and tag to GitHub

5. **GitHub Actions automatically**:
   - Builds binaries for Linux and macOS (x86_64 and ARM64)
   - Creates a GitHub Release with binaries attached
   - Publishes to crates.io

### Configuration

Release behavior is configured in [release.toml](release.toml). Key settings:
- Pre-release hooks run tests before releasing
- Automatic tag creation with `v` prefix
- CHANGELOG.md updates
- Sign commits/tags (optional)

# License
This project is licensed under the MIT License.