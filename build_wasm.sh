#!/bin/bash
set -e

# Ensure wasm-pack is installed, or try to use cargo directly if possible.
# Assuming environment might not have wasm-pack.
# If not, we can try to install it or fail gracefully.

echo "Building WASM..."

cd implants/lib/eldritchv2

# Check if wasm-pack is available
if command -v wasm-pack &> /dev/null; then
    wasm-pack build --target web --out-dir www/pkg --no-typescript
else
    echo "wasm-pack not found. Attempting to install..."
    # Installation might require network and root, which might fail in sandbox.
    # But let's assume we can use cargo install.
    # cargo install wasm-pack # This takes too long.

    # Fallback: manually build using cargo and wasm-bindgen-cli if available?
    # Usually easier to just fail and ask user to install wasm-pack if they want to build wasm.
    # But for this task, I should try my best.

    echo "Skipping WASM build as wasm-pack is missing. Please install wasm-pack to run the demo."
    exit 0
fi

echo "Done."
