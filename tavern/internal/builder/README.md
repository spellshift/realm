# Builder

The builder package orchestrates agent compilation for target platforms. It connects to the Tavern server via gRPC and compiles agents based on its configuration.

## Overview

- **Registration**: Builders register with Tavern via the `registerBuilder` GraphQL mutation, which returns an mTLS certificate signed by the Tavern Builder CA and a YAML configuration file.
- **mTLS Authentication**: All gRPC requests are authenticated using application-level mTLS. The builder presents its CA-signed certificate and a signed timestamp in gRPC metadata on each request. The server verifies the certificate chain, proof of private key possession, and looks up the builder by the identifier embedded in the certificate CN.
- **gRPC API**: Builders communicate with Tavern over gRPC at the `/builder.Builder/` route. Currently supports a `Ping` health check endpoint.
- **CLI**: Run a builder using the `builder` subcommand with a `--config` flag pointing to a YAML configuration file.

## Configuration

Builders are configured via a YAML file with the following schema:

```yaml
id: <unique builder identifier>
supported_targets:
  - linux
  - macos
  - windows
mtls: <base64-encoded mTLS certificate and key PEM bundle>
upstream: <tavern server address>
```

| Field | Description |
|-------|-------------|
| `id` | Unique identifier for this builder, assigned during registration. Embedded in the mTLS certificate CN as `builder-{id}`. |
| `supported_targets` | List of platforms this builder can compile agents for. Valid values: `linux`, `macos`, `windows`. |
| `mtls` | Base64-encoded PEM bundle containing the CA-signed mTLS certificate and private key for authenticating with Tavern. |
| `upstream` | The Tavern server address to connect to. |

## Authentication Flow

1. An admin registers a builder via the `registerBuilder` GraphQL mutation.
2. Tavern generates a unique identifier and an ECDSA P-256 client certificate signed by the Tavern Builder CA, with CN=`builder-{identifier}`.
3. The builder config YAML is returned containing the certificate, private key, identifier, and upstream address.
4. On each gRPC call, the builder client sends three metadata fields:
   - `builder-cert`: Base64-encoded DER certificate
   - `builder-signature`: Base64-encoded ECDSA signature over the timestamp
   - `builder-timestamp`: RFC3339Nano timestamp
5. The server interceptor verifies:
   - Certificate was signed by the Tavern Builder CA
   - Signature proves private key possession
   - Timestamp is within 5 minutes (replay prevention)
   - Certificate has not expired
   - Builder identifier from CN exists in the database

## Usage

```bash
# Register a builder via GraphQL (returns config YAML)
# Then run it:
go run ./tavern builder --config /path/to/builder-config.yaml
```

## Package Structure

| File | Purpose |
|------|---------|
| `auth.go` | gRPC unary interceptor for mTLS authentication |
| `ca.go` | Builder CA generation, persistence, and certificate signing |
| `client.go` | Builder client with `PerRPCCredentials` for mTLS auth |
| `config.go` | YAML configuration parsing and validation |
| `server.go` | gRPC server implementation (Ping) |
| `proto/builder.proto` | Protobuf service definition |
| `builderpb/` | Generated protobuf Go code |
| `integration_test.go` | End-to-end test covering registration, mTLS auth, and unauthenticated rejection |
