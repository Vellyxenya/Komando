#!/bin/bash
# Setup script for installing git hooks

set -e

echo "🔧 Setting up git hooks for Komando..."

# Check if we're in a git repository
if [ ! -d .git ]; then
    echo "Error: Not in a git repository"
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
    echo "❌ Error: hooks/pre-commit not found"
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
