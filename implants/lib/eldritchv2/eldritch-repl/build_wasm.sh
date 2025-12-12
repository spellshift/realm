#!/bin/bash
set -e

echo "Building WASM..."

# Check if wasm-pack is available
if command -v wasm-pack &> /dev/null; then
    # Enable fake_bindings feature for the WASM build
    wasm-pack build --target web --out-dir www/pkg --no-typescript -- --features fake_bindings
else
    echo "wasm-pack not found. Attempting to install..."
    cargo install wasm-pack

    if command -v wasm-pack &> /dev/null; then
        echo "wasm-pack installed successfully."
    else
        echo "Skipping WASM build as wasm-pack is missing. Please cargo install wasm-pack to run the demo."
        exit 0
    fi
fi

# Deploy to docs
# Assuming the script runs from implants/lib/eldritchv2/eldritch-repl/
TARGET_DIR="../../../../docs/assets/eldritch-repl"
echo "Deploying to $TARGET_DIR..."

mkdir -p "$TARGET_DIR"

# Copy index.html and pkg
cp www/index.html "$TARGET_DIR/"
cp -r www/pkg "$TARGET_DIR/"

echo "Done. REPL is available at docs/assets/eldritch-repl/index.html"
