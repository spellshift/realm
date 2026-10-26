# Agents

Welcome to our repository! Most commands need to be run from the root of the project directory (e.g. where this `AGENTS.md` file is located).

## Project Overview

Realm is an Adversary Emulation Framework focused on scalability, reliability, and automation. It is designed for red team engagements of any size (up to thousands of beacons).

## Project Structure

```text
realm/
├── tavern/          # Go server - GraphQL API, Web UI (TypeScript), gRPC API
│   └── internal/www # React/TypeScript frontend (npm managed)
├── implants/        # Rust code deployed to targets
│   ├── imix/        # Primary agent/beacon
│   └── lib/eldritch # Eldritch DSL implementation
├── docs/            # Jekyll documentation site
├── terraform/       # GCP deployment infrastructure
├── bin/             # Utility scripts and tools
├── docker/          # Container configurations
├── tests/           # Integration tests
└── build/           # Build artifacts
```

## Core Components

- **Tavern**: Go server with GraphQL/gRPC APIs and web interface
- **Imix**: Rust agent that runs on target systems (supports Linux, macOS, Windows)
- **Eldritch**: Pythonic DSL based on Starlark for defining red team operations
- **Tomes**: Bundled Eldritch scripts with metadata and assets

## Key Terminology

- **Beacon**: Instance of an agent running as a process on a target
- **Quest**: Multi-beacon task execution (one tome across multiple beacons)
- **Task**: Single tome execution against a single beacon
- **Tome**: Eldritch bundle with code, metadata, and embedded files

## Common Commands

### Build & Run

```bash
# Start Tavern server (from project root)
go run ./tavern

# Start Imix agent (from implants/imix)
cd implants/imix && cargo run
```

### Testing

```bash
# Run all Go tests
go test ./...

# Run Rust tests (from implants/)
cargo test
```

### Code Generation

```bash
# Regenerate code after ent schemas, GraphQL, or frontend changes
go generate ./...
```

### Formatting

```bash
# Format Rust code (REQUIRED before commits)
cargo fmt
```

## Development Guidelines

1. **Rust Dependencies**: Add to workspace root `Cargo.toml`, reference as workspace dependencies in crates.
2. **Code Generation**: Run `go generate ./...` after modifying ent schemas, GraphQL, or frontend.
3. **Rust Formatting**: Always run `cargo fmt` - CI will fail without it.
4. **Linear History**: Use squash merge for PRs.
5. **React Components**: Avoid large ternary operations, instead break out the two pieces into components and use a simple ternary operation e.g. `condition ? <ComponentA /> : <ComponentB />`
6. **React Utilities**: Separate styling from logic and abstract reuseable functions into utils files.
7. **React Styling**: Attempt to reference existing pages and components for style guidelines, be sure to re-use components and match styling to maintain consistency.
8. **Prefer Line-of-Sight Coding**: Avoid large indentation by returning early and keeping an unnested control flow, for example:

```golang
func doAThing(myparam *Thing):
    if myparam == nil {
        // Return early
        return
    }
    if myparam.Other <= 25 {
        // Perform an op and return early
        myparam.Other += 1
        return
    }
    // Expected code after edge cases have been handled
    fmt.Printf("myparam.Other has reached %d\n", myparam.Other)
}
```

## Eldritch Standard Library

The Eldritch DSL provides 27 modules (plus deprecated compatibility modules `crypto` and `pivot`):

| Module    | Purpose                                                       |
|-----------|---------------------------------------------------------------|
| `aes`     | AES encryption and decryption for bytes and files             |
| `agent`   | Agent metadata and control                                    |
| `assets`  | Embedded file access                                          |
| `base64`  | Base64 encoding and decoding                                  |
| `chain`   | Multi-agent C2 chaining                                       |
| `dns`     | DNS lookups                                                   |
| `file`    | File system operations                                        |
| `gzip`    | Gzip compression and decompression for bytes and files        |
| `hash`    | Cryptographic hashing (`md5`, `sha1`, `sha256`, `file`)       |
| `http`    | HTTP/HTTPS requests                                           |
| `json`    | JSON encoding, decoding, and validation                       |
| `linux`   | Linux-specific operations (`uname`)                           |
| `net`     | Network interfaces, listening sockets, and TCP/UDP clients    |
| `portal`  | Bi-directional portal streams                                 |
| `process` | Process management, PID, and environment variables            |
| `random`  | Cryptographically secure random                               |
| `regex`   | Regular expression operations                                 |
| `report`  | Structured reporting to Tavern                                |
| `scan`    | Network discovery and port scanning (`arp`, `port`)           |
| `ssh`     | SSH command execution, file copy, and deployment              |
| `sys`     | System OS info, hostname, and command execution               |
| `template` | Jinja2 template rendering from strings and files            |
| `time`    | Time formatting and delays                                    |
| `user`    | Current user info and user account enumeration                |
| `utf8`    | UTF-8 byte/string encoding and decoding                       |
| `utf16le` | UTF-16LE byte/string encoding and decoding                    |
| `windows` | Windows-specific DLL injection/reflection, registry, & tokens |

# Eldritch Changelog

## v0.4.1

In Eldritch `0.4.1`, the standard library API was reorganized into more granular, purpose-specific libraries. Old function names remain available temporarily, emit a deprecation warning to `stderr` via the `#[eldritch_deprecated("...")]` macro, and delegate to the new implementations.

To automatically migrate existing `.eldritch` tomes to the `0.4.1` API, run:

```bash
python3 bin/migrate/eldritch_migrate_0.4.1.py <path-to-tomes-or-file>
```

### Renamed Functions

- `file.temp_file` → `file.tmp`
- `file.pwd` → `process.cwd`
- `file.template` → `template.render_file`
- `file.template_str` → `template.render`
- `time.format_to_epoch` → `time.to_epoch`
- `time.format_to_readable` → `time.format`
- `agent.get_config` → `agent.config`
- `agent.get_transport` → `agent.transport`
- `agent.get_callback_interval` → `agent.callback_interval`
- `crypto.aes_decrypt` → `aes.decrypt`
- `crypto.aes_decrypt_file` → `aes.decrypt_file`
- `crypto.aes_encrypt` → `aes.encrypt`
- `crypto.aes_encrypt_file` → `aes.encrypt_file`
- `crypto.decode_b64` → `base64.decode`
- `crypto.decode_utf16le` → `utf16le.decode`
- `crypto.encode_b64` → `base64.encode`
- `crypto.encode_utf16le` → `utf16le.encode`
- `crypto.from_json` → `json.decode`
- `crypto.hash_file` → `hash.file`
- `crypto.is_json` → `json.is_valid`
- `crypto.md5` → `hash.md5`
- `crypto.sha1` → `hash.sha1`
- `crypto.sha256` → `hash.sha256`
- `crypto.to_json` → `json.encode`
- `file.compress` → `gzip.compress_file`
- `file.decompress` → `gzip.decompress_file`
- `pivot.arp_scan` → `scan.arp`
- `pivot.create_portal` → `portal.create`
- `pivot.ncat` → `net.tcp` & `net.udp` (split TCP and UDP into dedicated functions without the `protocol` parameter)
- `pivot.port_scan` → `scan.port`
- `pivot.smb_exec` → `smb.exec` (removed unimplemented proposal from docs; supported in migration script)
- `pivot.ssh_copy` → `ssh.copy`
- `pivot.ssh_deploy` → `ssh.deploy`
- `pivot.ssh_exec` → `ssh.exec`
- `process.netstat` → `net.sockets`
- `sys.dll_inject` → `windows.dll_inject`
- `sys.dll_reflect` → `windows.dll_reflect`
- `sys.get_env` → `process.env`
- `sys.get_ip` → `net.interfaces`
- `sys.get_os` → `sys.info`
- `sys.get_pid` → `process.pid`
- `sys.get_reg` → `windows.read_reg`
- `sys.get_user` → `user.info`
- `sys.impersonate` → `windows.impersonate`
- `sys.list_users` → `user.list`
- `sys.tokens` → `windows.tokens`
- `sys.use_token` → `windows.use_token`
- `sys.whereis` → `file.which`
- `sys.write_reg` → `windows.write_reg`

### New Functions

- `utf8.encode(content: str) -> Bytes`
- `utf8.decode(content: Bytes, lossy: Optional<bool>) -> str`
- `gzip.compress(data: Bytes) -> Bytes`
- `gzip.decompress(data: Bytes) -> Bytes`
- `linux.uname() -> Dict`

### Removed Documentation Proposals

- `pivot.bind_proxy` and `pivot.port_forward` (unimplemented proposals removed from documentation).

## Testing Requirements

- **Go**: Unit tests for new functionality.
- **Rust**: Unit tests required; use `cargo test`.
- **GraphQL**: YAML test specifications in tavern tests.
- **Frontend**: Tests in `tavern/internal/www`.

## Configuration

### Imix (Agent)

- Compile-time: `IMIX_CALLBACK_URI`, `IMIX_SERVER_PUBKEY`
- Runtime: `IMIX_BEACON_ID`, `IMIX_LOG`

### Tavern (Server)

- `HTTP_LISTEN_ADDR`: Server binding
- `ENABLE_METRICS`: Prometheus metrics
- `TAVERN_API_TOKEN`: API authentication

## Documentation Structure

The `docs/_docs/` folder contains three guides:

- **user-guide/**: Operational usage (getting-started, eldritch reference, imix, tomes, golem)
- **admin-guide/**: Deployment and configuration (GCP/Terraform, redirectors, MySQL)
- **dev-guide/**: Contributing and architecture (tavern, eldritch, imix development)
