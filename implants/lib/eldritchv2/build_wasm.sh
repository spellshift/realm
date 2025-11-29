#!/bin/bash
set -e

# Ensure wasm-pack is installed, or try to use cargo directly if possible.
# Assuming environment might not have wasm-pack.
# If not, we can try to install it or fail gracefully.

echo "Building WASM..."

# Check if wasm-pack is available
if command -v wasm-pack &> /dev/null; then
    wasm-pack build --target web --out-dir www/pkg --no-typescript
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

echo "Done."
