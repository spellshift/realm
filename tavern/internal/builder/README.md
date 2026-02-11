# Builder

The builder package orchestrates agent compilation for target platforms. It connects to the Tavern server via gRPC and compiles agents based on its configuration.

## Overview

- **Registration**: Builders register with Tavern via the `registerBuilder` GraphQL mutation, which returns an mTLS certificate signed by the Tavern Builder CA and a YAML configuration file.
- **mTLS Authentication**: All gRPC requests are authenticated using application-level mTLS. The builder presents its CA-signed certificate and a signed timestamp in gRPC metadata on each request. The server verifies the certificate chain, proof of private key possession, and looks up the builder by the identifier embedded in the certificate CN.
- **gRPC API**: Builders communicate with Tavern over gRPC at the `/builder.Builder/` route. Supports `Ping` (health check), `ClaimBuildTasks` (poll for unclaimed tasks), `StreamBuildTaskOutput` (stream build output incrementally), and `UploadBuildArtifact` (upload compiled binaries).
- **Executor**: Build tasks are executed via the `executor.Executor` interface. The `DockerExecutor` runs builds inside Docker containers; the `MockExecutor` is used in tests.
- **CLI**: Run a builder using the `builder` subcommand with a `--config` flag pointing to a YAML configuration file.

## Configuration

Builders are configured via a YAML file with the following schema:

```yaml
id: <unique builder identifier>
supported_targets:
  - linux
  - macos
  - windows
mtls: <mTLS certificate and key PEM bundle>
upstream: <tavern server address>
```

| Field | Description |
|-------|-------------|
| `id` | Unique identifier for this builder, assigned during registration. Embedded in the mTLS certificate CN as `builder-{id}`. |
| `supported_targets` | List of platforms this builder can compile agents for. Valid values: `linux`, `macos`, `windows`. |
| `mtls` | PEM bundle containing the CA-signed mTLS certificate and private key for authenticating with Tavern. |
| `upstream` | The Tavern server address to connect to. |

## Authentication Flow

1. An admin registers a builder via the `registerBuilder` GraphQL mutation.
2. Tavern generates a unique identifier and an Ed25519 client certificate signed by the Tavern Builder CA, with CN=`builder-{identifier}`.
3. The builder config YAML is returned containing the certificate, private key, identifier, and upstream address.
4. On each gRPC call, the builder client sends three metadata fields:
   - `builder-cert-bin`: DER-encoded certificate (binary metadata)
   - `builder-signature-bin`: Ed25519 signature over the timestamp (binary metadata)
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

## Streaming Build Output

The `StreamBuildTaskOutput` RPC uses client-streaming to send build output incrementally.
The builder sends one message per output/error line as the executor produces them, and each
message is flushed to the database immediately. The final message sets `finished=true` to
signal completion. The `started_at` timestamp is set when the first message is received.
If the stream is interrupted before `finished=true`, partial output is preserved but
`finished_at` is not set.

## Artifact Extraction

Build tasks can specify an `artifact_path` field — the path inside the container where the
compiled binary or output file is written. When the build finishes successfully, the
`DockerExecutor` copies the file from the stopped container using Docker's `CopyFromContainer`
API (which returns a tar archive), extracts the first regular file, and returns the bytes in
`BuildResult`. The builder client then streams the artifact to Tavern via the
`UploadBuildArtifact` RPC in 1 MB chunks. The server creates an `Asset` entity.
The artifact is downloadable via the existing CDN endpoint at `GET /assets/download/{name}`.


### Easy
- Add exitCode to the buildTask ent update executor and build client with build exitCode.
- Builder management
   - Add a remove builder mutation
   - Add Builder as a queryable type
   - Add a last seen at field to builder ent that's updated on each `ClaimBuildTask` call
- Defaults
   - Add default bulidImage: `spellshift/devcontainer:main`
   - Add default target format: `BIN`
   - 'buildImage' should be optional in the mutation and default to `spellshift/devcontainer:main`
   - `artifactPath` should be an optional param to the createBuildTask mutation.
   - `artifactPath` should default to the derived path only if no other path is specified. Use the same pattern as interval:
   ```
      interval := builder.DefaultInterval
      if input.Interval != nil {
         interval = *input.Interval
      }
   ```



### Architectural
- Add a way for the server to interrupt and cancel a build.
- Add support for build caching between jobs (will speed up rust builds a lot)
- Instead of assuming  `/home/vscode` create a correctly permissioned build dir
- Add support for mulitple transports in the builder


### future
- Register redirectors so bulider callback uri can be a drop down.
- Modifying the agent IMIX_CONFIG currently requires changes to both imix and tavern code bases now. Is there a way to codegen a YAML spec from tavern to the agent?
- De-dupe agent builds should the API stop builds that have the same params and point to the existing build? Or is this a UI thing?


### Planning
- Where should realm source code be pulled?
   - which version'd copy of the code to checkout
      - Can we automatically determine which version / main,edge the server is and pass that ot the build script.
   - Ship tavern with imix source code embedded?
      - Hard for teams to bring their own changes.

- Update schema for UX
   - Target OS + Target Format ---> rust target
      - TargetOS's only support certain formats
   - where to get the realm source code from - pull public repo?
   - Currentt pattern with arbitrary bulid script is RCE as a service. Scope and limit this to just build configuration options.
   - upstream should be free form
   - pubkey can be set by the server


## Package Structure

| File | Purpose |
|------|---------|
| `auth.go` | gRPC unary and stream interceptors for mTLS authentication |
| `ca.go` | Builder CA generation, persistence, and certificate signing |
| `client.go` | Builder client: mTLS credentials, polling loop, task execution |
| `config.go` | YAML configuration parsing and validation |
| `server.go` | gRPC server: `Ping`, `ClaimBuildTasks`, `StreamBuildTaskOutput`, `UploadBuildArtifact` |
| `rollback.go` | Transaction rollback helper (matches c2 pattern) |
| `executor/executor.go` | `Executor` interface, `BuildSpec`, and `BuildResult` definitions |
| `executor/docker.go` | `DockerExecutor`: runs builds in Docker containers |
| `executor/mock.go` | `MockExecutor`: test double for unit tests |
| `proto/builder.proto` | Protobuf service definition |
| `builderpb/` | Generated protobuf Go code |
| `integration_test.go` | End-to-end test: registration, mTLS auth, task claiming |
| `executor_integration_test.go` | End-to-end test: claim → execute → submit flow |
