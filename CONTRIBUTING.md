# Contributing to Komando

Thank you for your interest in contributing to Komando! This document provides guidelines and instructions for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/Komando.git
   cd Komando
   ```
3. Set up pre-commit hooks:
   ```bash
   ./setup_hooks.sh
   ```

## Development Workflow

### 1. Create a Branch
```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 2. Make Your Changes

Ensure your code follows our standards:
- Write tests for new functionality
- Update documentation as needed
- Follow Rust naming conventions
- Add comments for complex logic

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run with all features
cargo test --all-features

# Run specific test
cargo test test_name
```

### 4. Check Code Quality

The pre-commit hook will automatically run these checks, but you can run them manually:

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Build check
cargo check --all-targets --all-features
```

### 5. Commit Your Changes

The pre-commit hook will automatically run before each commit:
- Code formatting check (`cargo fmt`)
- Linting (`cargo clippy`)
- Tests (`cargo test`)
- Build verification (`cargo check`)

```bash
git add .
git commit -m "feat: add new feature"
# or
git commit -m "fix: resolve issue #123"
```

**Commit Message Format:**
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `test:` Adding or updating tests
- `refactor:` Code refactoring
- `perf:` Performance improvements
- `chore:` Maintenance tasks

### 6. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Code Standards

### Rust Style
- Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Use `cargo fmt` for formatting
- Fix all `cargo clippy` warnings

### Testing
- Write unit tests for new functions
- Add integration tests for new features
- Aim for good test coverage
- Test both success and error cases

### Documentation
- Add doc comments for public APIs
- Update README.md for user-facing changes
- Include examples in doc comments when helpful

## Pre-commit Hooks

We use git hooks to maintain code quality. The hooks run:

1. **Format Check** - Ensures code is formatted with `rustfmt`
2. **Linting** - Checks for common mistakes with `clippy`
3. **Tests** - Runs all unit tests
4. **Build Check** - Verifies the code compiles

To bypass hooks (emergencies only):
```bash
git commit --no-verify
```

**Note:** CI will still run all checks, so bypassing hooks may cause CI failures.

## Alternative: Pre-commit Framework

If you prefer the pre-commit framework:

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run manually
pre-commit run --all-files
```

## Project Structure

```
Komando/
├── src/
│   ├── main.rs      # CLI entry point
│   ├── db.rs        # SQLite database operations
│   └── ops.rs       # Legacy JSON migration
├── hooks/           # Git hook scripts
├── .github/         # CI/CD workflows
└── tests/           # Integration tests (future)
```

## Features

- **Standard Build**: Pattern-based search (default)
- **Embeddings Feature**: Semantic search with vector embeddings

Test both features:
```bash
cargo test                    # Standard
cargo test --all-features     # With embeddings
```

## Getting Help

- Open an issue for bugs or feature requests
- Join discussions in existing issues
- Ask questions in pull requests

## Code Review Process

1. All PRs must pass CI checks
2. At least one maintainer approval required
3. Address review feedback
4. Squash commits before merge (optional)

## Release Process (Maintainers Only)

Komando uses [cargo-release](https://github.com/crate-ci/cargo-release) for automated releases:

**Single Source of Truth:** Version is only defined in `Cargo.toml`. cargo-release handles creating matching git tags automatically.

### Steps to Release

1. **Update CHANGELOG.md**: Add changes under `[Unreleased]` section

2. **Run cargo-release**:
   ```bash
   # Dry run first
   cargo release --dry-run
   
   # Then release (examples)
   cargo release patch              # 0.1.1 -> 0.1.2
   cargo release minor              # 0.1.1 -> 0.2.0
   cargo release major              # 0.1.1 -> 1.0.0
   cargo release --pre-release alpha # 0.1.1 -> 0.2.0-alpha.1
   ```

3. **Automated CI/CD**: GitHub Actions automatically:
   - Builds binaries for all platforms
   - Creates GitHub Release
   - Publishes to crates.io

**No Manual Steps Required!** cargo-release handles:
- ✅ Version bump in Cargo.toml
- ✅ CHANGELOG.md updates
- ✅ Git commit and tag creation
- ✅ Pushing to GitHub

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Feel free to open an issue or reach out to the maintainers!
