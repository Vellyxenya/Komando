#!/bin/bash
# Setup script for installing git hooks

set -e

# Ensure we are in the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "🔧 Setting up git hooks for Komando..."

# Check if we're in a git repository
if [ ! -d .git ]; then
    echo "Error: Not in a git repository (or .git directory not found in project root)"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Copy pre-commit hook
if [ -f hooks/pre-commit ]; then
    cp hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "✅ Installed pre-commit hook"
else
    echo "❌ Error: hooks/pre-commit not found in $PROJECT_ROOT/hooks/"
    exit 1
fi

echo ""
echo "✨ Git hooks installed successfully!"
echo ""
echo "The pre-commit hook will run:"
echo "  - rustfmt (code formatting)"
echo "  - clippy (linting)"
echo "  - cargo test (unit tests)"
echo "  - cargo check (build verification)"
echo ""
echo "To bypass hooks (emergency only): git commit --no-verify"
echo ""
