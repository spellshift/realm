#!/bin/bash
cd implants
cargo install cargo-llvm-cov || true
cargo llvm-cov --manifest-path lib/eldritch/eldritch-core/Cargo.toml
