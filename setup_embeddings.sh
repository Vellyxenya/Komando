#!/bin/bash
set -e

echo "Setting up ONNX Runtime for Komando embeddings..."

# Create directory
ONNX_DIR="$HOME/.onnxruntime"
mkdir -p "$ONNX_DIR"

# Download ONNX Runtime 1.23.2 (compatible with GLIBC 2.27+)
ONNX_VERSION="1.23.2"
ONNX_TARBALL="onnxruntime-linux-x64-${ONNX_VERSION}.tgz"
ONNX_URL="https://github.com/microsoft/onnxruntime/releases/download/v${ONNX_VERSION}/${ONNX_TARBALL}"

if [ ! -d "$ONNX_DIR/onnxruntime-linux-x64-${ONNX_VERSION}" ]; then
    echo "Downloading ONNX Runtime ${ONNX_VERSION}..."
    cd "$ONNX_DIR"
    wget -q --show-progress "$ONNX_URL"
    echo "Extracting..."
    tar xzf "$ONNX_TARBALL"
    rm "$ONNX_TARBALL"
    echo "✓ ONNX Runtime installed to $ONNX_DIR"
else
    echo "✓ ONNX Runtime already installed"
fi

# Add to shell RC file
ONNX_LIB_PATH="$ONNX_DIR/onnxruntime-linux-x64-${ONNX_VERSION}/lib/libonnxruntime.so.${ONNX_VERSION}"
SHELL_RC="${HOME}/.$(basename $SHELL)rc"

if ! grep -q "ORT_DYLIB_PATH" "$SHELL_RC" 2>/dev/null; then
    echo "" >> "$SHELL_RC"
    echo "# ONNX Runtime for Komando embeddings" >> "$SHELL_RC"
    echo "export ORT_DYLIB_PATH=\"$ONNX_LIB_PATH\"" >> "$SHELL_RC"
    echo "export LD_LIBRARY_PATH=\"$ONNX_DIR/onnxruntime-linux-x64-${ONNX_VERSION}/lib:\$LD_LIBRARY_PATH\"" >> "$SHELL_RC"
    echo "✓ Added environment variables to $SHELL_RC"
else
    echo "✓ Environment variables already configured"
fi

echo ""
echo "Setup complete! To use embeddings:"
echo "  1. Build with: cargo build --release --features embeddings"
echo "  2. Restart your shell or run: source $SHELL_RC"
echo ""
echo "To verify, check your GLIBC requirement:"
echo "  objdump -T $ONNX_LIB_PATH | grep GLIBC | sed 's/.*GLIBC_/GLIBC_/' | sort -Vr | head -1"
echo "  (Should show GLIBC_2.27 or lower)"
