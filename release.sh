#!/bin/sh

VERSION=$1
if [ -z "$VERSION" ]
then
    echo "Please specify a version, for example ./release.sh 0.3.0"
    exit 2
fi

echo "Now releasing Realm v$VERSION"
echo "Please go get some coffee, this may take a while ☕"

###
## Update Versions
###
echo "[v$VERSION] Updating build versions"
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/imix/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/golem/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/tavern/Cargo.toml


###
## Rust Setup
###
echo "[v$VERSION] Installing dependencies"
apt update
apt install -y musl-tools gcc-mingw-w64

###
## Release Imix
###
cd ./implants/imix
echo "[v$VERSION] Building Imix Release $(pwd)"
rustup target add x86_64-unknown-linux-musl
rustup target add x86_64-pc-windows-gnu
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target=x86_64-unknown-linux-musl
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target=x86_64-pc-windows-gnu
cd ../..

###
## Release Golem
###
cd ./implants/golem
echo "[v$VERSION] Building Golem Release $(pwd)"
rustup target add x86_64-unknown-linux-musl
rustup target add x86_64-pc-windows-gnu
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target=x86_64-unknown-linux-musl
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target=x86_64-pc-windows-gnu
cd ../..


###
## Complete
###
echo "[v$VERSION][WARN] MacOS cannot be cross-compiled yet, please manually build artifacts"
echo "[v$VERSION] Release completed"
