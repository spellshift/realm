# Shellcode App

A minimal Rust application that compiles to position-independent shellcode for Linux x86_64.

## Features

- Reads the `SHELLCODE_FILE_PATH` environment variable
- Creates a file at the path specified in that environment variable
- Compiles to position-independent code (PIC) suitable for use as shellcode
- No standard library dependencies (`no_std`)
- Direct syscalls (no libc)

## Building

Build the shellcode:

```bash
cargo build --release
```

## Extracting Shellcode

To extract the raw shellcode bytes from the compiled binary:

```bash
objcopy -O binary --only-section=.text target/x86_64-unknown-linux-gnu/release/shellcode_app shellcode.bin
```

Or to get a hex dump:

```bash
objdump -d target/x86_64-unknown-linux-gnu/release/shellcode_app
```

To view as hex bytes:

```bash
hexdump -C shellcode.bin
```

## Usage

Set the environment variable and run:

```bash
export SHELLCODE_FILE_PATH=/tmp/test_file.txt
./target/x86_64-unknown-linux-gnu/release/shellcode_app
```

The file should be created at `/tmp/test_file.txt`.

## Technical Details

- Uses direct Linux syscalls (open, close, exit)
- Position-independent code (PIC) for portability
- Minimal size optimization
- No relocations in the shellcode
- Parses environment variables directly from the process stack

## Security Note

This is a demonstration of position-independent shellcode generation for security research,
penetration testing, and CTF challenges. Use responsibly and only in authorized contexts.
