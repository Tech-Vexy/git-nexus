#!/usr/bin/env bash

set -e

BINARY_NAME="git-nexus"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"

echo "🗑️  Uninstalling git-nexus..."
echo ""

if [ -f "$BINARY_PATH" ]; then
    echo "📍 Found: $BINARY_PATH"
    rm "$BINARY_PATH"
    echo "✅ Successfully removed $BINARY_NAME"
else
    echo "⚠️  Binary not found at: $BINARY_PATH"
    
    # Check other common locations
    OTHER_LOCATIONS=(
        "/usr/local/bin/$BINARY_NAME"
        "/usr/bin/$BINARY_NAME"
        "$HOME/bin/$BINARY_NAME"
    )
    
    FOUND=false
    for location in "${OTHER_LOCATIONS[@]}"; do
        if [ -f "$location" ]; then
            echo "📍 Found at: $location"
            
            if [ -w "$location" ]; then
                rm "$location"
                echo "✅ Successfully removed $BINARY_NAME"
                FOUND=true
                break
            else
                echo "❌ No write permission. Try running with sudo:"
                echo "   sudo rm $location"
                FOUND=true
                break
            fi
        fi
    done
    
    if [ "$FOUND" = false ]; then
        echo "❌ Could not find $BINARY_NAME in common locations"
        exit 1
    fi
fi

echo ""
echo "👋 git-nexus has been uninstalled"
echo ""
