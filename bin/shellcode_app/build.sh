#!/bin/bash

set -e

echo "Building shellcode application..."

# Build with special flags for position-independent shellcode
RUSTFLAGS="-C link-arg=-nostartfiles -C link-arg=-static -C link-arg=-Wl,--build-id=none -C relocation-model=pic -C code-model=small" \
    cargo build --release --bin shellcode_app --features shellcode

echo ""
echo "Build complete!"
echo ""
echo "Binary location: target/x86_64-unknown-linux-gnu/release/shellcode_app"
echo ""
echo "To extract raw shellcode:"
echo "  objcopy -O binary --only-section=.text target/x86_64-unknown-linux-gnu/release/shellcode_app shellcode.bin"
echo ""
echo "To view disassembly:"
echo "  objdump -d target/x86_64-unknown-linux-gnu/release/shellcode_app"
echo ""
echo "To test:"
echo "  export SHELLCODE_FILE_PATH=/tmp/test_file.txt"
echo "  ./target/x86_64-unknown-linux-gnu/release/shellcode_app"
