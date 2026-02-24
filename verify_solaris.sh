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

# Verify aws-lc-sys is gone
if echo "$OUTPUT" | grep -q "aws-lc-sys"; then
    echo "FAIL: aws-lc-sys is still present in dependency graph for Solaris!"
    exit 1
else
    echo "PASS: aws-lc-sys is NOT present in dependency graph for Solaris."
fi

# Verify netdev is gone
if echo "$OUTPUT" | grep -q "netdev"; then
    echo "FAIL: netdev is still present in dependency graph for Solaris!"
    exit 1
else
    echo "PASS: netdev is NOT present in dependency graph for Solaris."
fi

# Verify nix is gone (except maybe via dev-dependencies? No, we excluded it)
if echo "$OUTPUT" | grep -q "nix "; then
    echo "FAIL: nix is still present in dependency graph for Solaris!"
    exit 1
else
    echo "PASS: nix is NOT present in dependency graph for Solaris."
fi

echo "Running cargo check for Solaris target..."
# This should now PASS completely.
# We set IMIX_SERVER_PUBKEY to a dummy value to satisfy build script.
if IMIX_SERVER_PUBKEY="AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" cargo check --target x86_64-pc-solaris -p imix; then
    echo "SUCCESS: Solaris build passed!"
else
    echo "FAIL: Solaris build failed!"
    exit 1
fi

echo "Verifying Linux build (regression test)..."
if IMIX_SERVER_PUBKEY="AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" cargo check -p imix; then
    echo "SUCCESS: Linux build passed (no regressions)."
else
    echo "FAIL: Linux build failed!"
    exit 1
fi
