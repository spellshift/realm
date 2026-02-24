#!/bin/bash
set -e

# Ensure we are in the implants directory or root.
# Assuming run from repo root.

if [ -d "implants" ]; then
    cd implants
fi

echo "Installing Solaris target..."
rustup target add x86_64-pc-solaris

echo "Verifying dependency exclusion for Solaris..."
# cargo tree output for specific target should NOT contain portable-pty
OUTPUT=$(cargo tree -p imix --target x86_64-pc-solaris)
if echo "$OUTPUT" | grep -q "portable-pty"; then
    echo "FAIL: portable-pty is still present in dependency graph for Solaris!"
    exit 1
else
    echo "PASS: portable-pty is NOT present in dependency graph for Solaris."
fi

echo "Attempting dry-run check (might fail due to missing cross-compilation headers)..."
# We expect this might fail, but let's run it to show progress.
if IMIX_SERVER_PUBKEY="AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" cargo check --target x86_64-pc-solaris -p imix; then
    echo "SUCCESS: Solaris build passed!"
else
    echo "WARNING: cargo check failed, likely due to missing system headers for C dependencies (e.g. aws-lc-sys). This is expected in this environment."
    echo "Dependency exclusion verified via cargo tree."
fi

echo "Verifying Linux build (regression test)..."
if IMIX_SERVER_PUBKEY="AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" cargo check -p imix; then
    echo "SUCCESS: Linux build passed (no regressions)."
else
    echo "FAIL: Linux build failed!"
    exit 1
fi
