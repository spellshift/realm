#!/bin/bash
cd implants
rustup run stable cargo fmt
rustup run stable cargo check -p screenshot --target x86_64-unknown-linux-gnu
rustup run stable cargo check -p screenshot --target x86_64-apple-darwin
rustup run stable cargo check -p screenshot --target x86_64-pc-windows-msvc
