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
6. **React Utilities**: Separate styling from logic and abstract reusable functions into utils files.
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

The Eldritch DSL provides 12 modules:

| Module    | Purpose                              |
|-----------|--------------------------------------|
| `agent`   | Agent metadata and control           |
| `assets`  | Embedded file access                 |
| `crypto`  | Encryption, decryption, hashing      |
| `file`    | File system operations               |
| `http`    | HTTP/HTTPS requests                  |
| `pivot`   | Network enumeration, lateral movement|
| `process` | Process management                   |
| `random`  | Cryptographically secure random      |
| `regex`   | Regular expression operations        |
| `report`  | Structured reporting to Tavern       |
| `sys`     | System info and execution            |
| `time`    | Time formatting and delays           |

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
