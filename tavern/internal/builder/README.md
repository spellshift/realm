# Builder

The builder package orchestrates agent compilation for target platforms. It connects to the Tavern server via gRPC and compiles agents based on its configuration.

## Overview

- **Registration**: Builders register with Tavern via the `registerBuilder` GraphQL mutation, which returns an mTLS certificate and a YAML configuration file.
- **gRPC API**: Builders communicate with Tavern over gRPC at the `/builder.Builder/` route. Currently supports a `Ping` health check endpoint.
- **CLI**: Run a builder using the `builder` subcommand with a `--config` flag pointing to a YAML configuration file.

## Configuration

Builders are configured via a YAML file with the following schema:

```yaml
supported_targets:
  - linux
  - macos
  - windows
mtls: <base64-encoded mTLS certificate and key PEM bundle>
```

| Field | Description |
|-------|-------------|
| `supported_targets` | List of platforms this builder can compile agents for. Valid values: `linux`, `macos`, `windows`. |
| `mtls` | Base64-encoded PEM bundle containing the mTLS certificate and private key for authenticating with Tavern. |

## Usage

```bash
# Register a builder via GraphQL (returns config YAML)
# Then run it:
go run ./tavern builder --config /path/to/builder-config.yaml
```

## Package Structure

| File | Purpose |
|------|---------|
| `config.go` | YAML configuration parsing and validation |
| `server.go` | gRPC server implementation (Ping) |
| `run.go` | Builder run loop (connects to Tavern and awaits work) |
| `proto/builder.proto` | Protobuf service definition |
| `builderpb/` | Generated protobuf Go code |
| `integration_test.go` | End-to-end test covering registration and gRPC communication |
