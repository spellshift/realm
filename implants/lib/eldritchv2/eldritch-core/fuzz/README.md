# Eldritch Core Fuzzing

This directory contains fuzz tests for `eldritch-core` using [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).

## Prerequisites

You need to have `cargo-fuzz` installed.

```bash
cargo install cargo-fuzz
```

## Running Fuzz Tests

To run a specific fuzz target, use `cargo fuzz run <target>`.

Available targets:
- `interpret`: Fuzzes the parser and interpreter with random string input.
- `complete`: Fuzzes the tab-completion logic with random code and cursor positions.
- `operations`: Fuzzes binary and unary operations with structurally generated `Value` types.

Example:

```bash
# Run the interpret fuzzer
cargo fuzz run interpret

# Run the operations fuzzer
cargo fuzz run operations
```

By default, the fuzzer runs indefinitely until a crash is found. You can stop it with `Ctrl+C`.

To run for a limited time or number of executions, you can pass arguments to libfuzzer after `--`:

```bash
# Run for max 10 seconds
cargo fuzz run interpret -- -max_total_time=10
```

## adding New Targets

1.  Create a new file in `fuzz_targets/`.
2.  Add a `[[bin]]` entry in `Cargo.toml` with the `name` and `path`.
3.  Implement the `fuzz_target!` macro.

```toml
[[bin]]
name = "my_target"
path = "fuzz_targets/my_target.rs"
```

## Structure

- `fuzz_targets/`: Contains the source code for each fuzz target.
- `Cargo.toml`: configuration for the fuzz crate.
- `eldritch.dict`: A dictionary file containing keywords and symbols to help the fuzzer generate valid Eldritch code.

## Notes

- The fuzz targets are compiled with the sanitizer options enabled, so they are efficient at catching memory safety issues and panics.
- `operations` target uses the `arbitrary` crate to generate structured data (`Value` types) rather than just raw bytes, allowing for deeper testing of operation semantics.
