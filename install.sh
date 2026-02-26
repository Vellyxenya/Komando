#!/bin/bash

set -e  # Exit on error

echo "🔨 Building Komando..."

# Check if --embeddings flag is passed
if [[ "$1" == "--embeddings" ]]; then
    echo "Building with semantic embeddings feature..."
    
    # Check if ONNX Runtime is set up
    if [[ -z "$ORT_DYLIB_PATH" ]]; then
        echo ""
        echo "⚠️  WARNING: ONNX Runtime environment not detected!"
        echo "   Run ./setup_embeddings.sh first, then source your shell config"
        echo ""
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    cargo build --release --features embeddings
else
    echo "Building standard version (use --embeddings for semantic search)..."
    cargo build --release
fi

echo ""
echo "✅ Build successful!"
echo ""
echo "📦 Installing to /usr/local/bin/..."

# Copy the binary
sudo cp target/release/komando_exec /usr/local/bin/

echo ""
echo "✨ Installation complete!"
echo ""

# Detect shell and show appropriate source command
SHELL_NAME=$(basename "$SHELL")
if [[ "$SHELL_NAME" == "zsh" ]]; then
    SHELL_RC="~/.zshrc"
elif [[ "$SHELL_NAME" == "bash" ]]; then
    SHELL_RC="~/.bashrc"
else
    SHELL_RC="your shell configuration file"
fi

echo "🔄 Don't forget to reload your shell configuration:"
echo "   source $SHELL_RC"
echo ""
echo "   Or restart your terminal"
echo ""
echo "Then verify with: komando --help"
echo ""

# Show version info
if [[ "$1" == "--embeddings" ]]; then
    echo "ℹ️  Semantic embeddings enabled - model will download on first use (~86MB)"
else
    echo "ℹ️  Standard build - to enable semantic search, run: ./install.sh --embeddings"
fi
