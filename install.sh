#!/usr/bin/env bash

set -e

BINARY_NAME="git-nexus"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

echo "🚀 Installing git-nexus..."
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo is not installed."
    echo "   Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build the project
echo "📦 Building git-nexus in release mode..."
cargo build --release

if [ ! -f "target/release/$BINARY_NAME" ]; then
    echo "❌ Error: Build failed. Binary not found at target/release/$BINARY_NAME"
    exit 1
fi

# Create install directory if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
    echo "📁 Creating directory: $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
fi

# Copy binary to install directory
echo "📥 Installing $BINARY_NAME to $INSTALL_DIR..."
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo ""
echo "✅ Installation complete!"
echo ""
echo "📍 Binary installed to: $INSTALL_DIR/$BINARY_NAME"
echo ""

# Check if install directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  Warning: $INSTALL_DIR is not in your PATH"
    echo ""
    echo "   Add it to your PATH by adding this line to your shell config file:"
    echo "   (~/.bashrc, ~/.zshrc, or ~/.config/fish/config.fish)"
    echo ""
    
    if [[ "$SHELL" == *"fish"* ]]; then
        echo "   set -gx PATH $INSTALL_DIR \$PATH"
    else
        echo "   export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
    echo ""
    echo "   Then reload your shell config:"
    
    if [[ "$SHELL" == *"zsh"* ]]; then
        echo "   source ~/.zshrc"
    elif [[ "$SHELL" == *"fish"* ]]; then
        echo "   source ~/.config/fish/config.fish"
    else
        echo "   source ~/.bashrc"
    fi
else
    echo "🎉 You can now run: $BINARY_NAME"
    echo ""
    echo "   Try it out:"
    echo "   $BINARY_NAME --help"
    echo "   $BINARY_NAME"
fi

echo ""
