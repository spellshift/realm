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
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/eldritch/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/eldritch-core/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/eldritch-macros/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/eldritch-repl/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/eldritch-wasm/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-libagent/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-libassets/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-libcrypto/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-libfile/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-libhttp/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-libpivot/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-libprocess/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-librandom/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-libregex/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-libreport/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-libsys/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/eldritch-libtime/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/eldritch/stdlib/tests/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/portals/portal-stream/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" implants/lib/c2/Cargo.toml
sed -i -E "s/version = \"[0-9].[0-9].[0-9]\"/version = \"$VERSION\"/" vscode/eldritch-lang/Cargo.toml
sed -i -E "s/version_string = \"v[0-9].[0-9].[0-9]\"/version_string = \"v$VERSION\"/" implants/imix/src/main.rs
sed -i -E "s/Version = \"v[0-9].[0-9].[0-9]\"/Version = \"v$VERSION\"/" tavern/version.go


###
## Rust Setup
###
echo "[v$VERSION] Installing dependencies"
sudo apt update
sudo apt install -y musl-tools gcc-mingw-w64

###
## Release Imix
###
# cd ./implants/imix
# echo "[v$VERSION] Building Imix Release $(pwd)"
# rustup target add x86_64-unknown-linux-musl
# rustup target add x86_64-pc-windows-gnu
# RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target=x86_64-unknown-linux-musl
# RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target=x86_64-pc-windows-gnu
# cd ../..

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
