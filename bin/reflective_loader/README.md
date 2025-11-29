## Bulid

cargo install xwin --target x86_64-unknown-linux-gnu
export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=/FIXED"

cargo xwin build --release \
    -Z build-std=core,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    --target x86_64-pc-windows-msvc

cargo test --no-run --target=x86_64-pc-windows-gnu
