#!/bin/bash
set -e

echo "Building WASM for Tavern..."

# Check if wasm-pack is available
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Attempting to install..."
    cargo install wasm-pack
fi

if ! command -v wasm-pack &> /dev/null; then
     echo "Skipping WASM build as wasm-pack is missing."
     exit 1
fi

# Determine absolute path for output to avoid confusion
# We are in implants/lib/eldritch/eldritch-wasm/
# Tavern public dir is ../../../../tavern/internal/www/public/wasm
OUTPUT_DIR="../../../../tavern/internal/www/public/wasm"

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"

# Build with fake_bindings feature. We use --target web to get a standard ES module interface.
wasm-pack build --target web --out-dir "$OUTPUT_DIR" --features fake_bindings

echo "Done. WASM artifacts built and copied to $OUTPUT_DIR"
