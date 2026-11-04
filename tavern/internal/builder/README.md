# Builder

The builder package orchestrates agent compilation for target platforms. It connects to the Tavern server via gRPC and compiles agents based on its configuration.

# TODO
- Add guardrails
- Add e2e testsq
- BuildProfile structs are spread across models and builderpb can we consolidate these
- [x] Streamline the buildScript with a setupScript and buildScript
- [x] Add support for build caching between jobs (will speed up rust builds a lot; see "Build Caching" below)

### Architectural
- Add a way for the server to interrupt and cancel a build.
- Instead of assuming  `/home/vscode` create a correctly permissioned build dir

### future
- Add terraform for build server
- Register redirectors so bulider callback uri can be a drop down.
- Modifying the agent IMIX_CONFIG currently requires changes to both imix and tavern code bases now. Is there a way to codegen a YAML spec from tavern to the agent?
- De-dupe agent builds should the API stop builds that have the same params and point to the existing build? Or is this a UI thing?


### Planning
- Change exit Code to a bool and rename to errored to make API querying easier
- Change exit Code to a bool and rename to errored to
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
## Overview

- **Registration**: Builders register with Tavern via the `registerBuilder` GraphQL mutation, which returns an mTLS certificate signed by the Tavern Builder CA and a YAML configuration file.
- **mTLS Authentication**: All gRPC requests are authenticated using application-level mTLS. The builder presents its CA-signed certificate and a signed timestamp in gRPC metadata on each request. The server verifies the certificate chain, proof of private key possession, and looks up the builder by the identifier embedded in the certificate CN.
- **gRPC API**: Builders communicate with Tavern over gRPC at the `/builder.Builder/` route. Supports `ClaimBuildTasks` (poll for unclaimed tasks), `StreamBuildTaskOutput` (stream build output and exit code incrementally), and `UploadBuildArtifact` (upload compiled binaries).
- **Builder Management**: Builders are queryable via the `builders` GraphQL query (paginated, filterable, orderable by `LAST_SEEN_AT`) and removable via `deleteBuilder`. Deleting a builder cascade-deletes its build tasks. The `last_seen_at` field is updated on each `ClaimBuildTasks` call.
- **Build Profiles**: Tasks use a `BuildProfile` to define agent configuration, including prioritized transports, pre/post build scripts, and bundled tomes.
- **Executor**: Build tasks are executed via the `executor.Executor` interface. The `DockerExecutor` runs builds inside Docker containers; `LocalExecutor` runs them directly on the host for fast, containerless testing (Linux only — see "Local Executor" below); the `MockExecutor` is used in unit tests.
- **CLI**: Run a builder using the `builder` subcommand with a `--config` flag and an optional `--executor [docker|local]` flag (defaults to the config file's `executor` setting).

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
| `poll_interval` | Task polling interval in seconds (1–86400); defaults to 5. Set during registration using `pollInterval` so the server and generated config agree. Builder freshness allows three polling intervals, independently of agent transports. |
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
message is flushed to the database immediately. The final message sets `finished=true` and
includes the container's `exit_code` to signal completion. The server persists both
`finished_at` and `exit_code` on the build task. The `started_at` timestamp is set when
the first message is received. If the stream is interrupted before `finished=true`, partial
output is preserved but `finished_at` and `exit_code` are not set.

## Build Profiles

Build tasks reference a `BuildProfile` which centralizes the configuration for the compiled agent. A Build Profile defines:
- **Transports**: The list of C2 transports (e.g., HTTP, gRPC) the agent should use, in order of priority.
- **Build Image**: The Docker image used for the build environment (default: `spellshift/devcontainer:main`).
- **Scripts**: Optional `prebuildscript` and `postbuildscript` that execute as Bash scripts before and after the `cargo build` command.
- **Tomes**: Bundled Eldritch scripts. During execution, tomes are mounted to `/mnt/tomes/` and copied into the agent's `install_scripts/` directory before building.

### Saved build inputs

A task retains its required profile relationship for provenance and captures an
immutable, typed `profile_at_creation` JSON object. GraphQL exposes it as
`profileAtCreation`, including nested transport settings and tome names/parameters.
Updating a profile affects new tasks only. Claiming tasks, authorizing tome
downloads, and naming output artifacts use the saved profile.

Profiles own `build_script` and `artifact_path` templates. Their defaults are
`{{.BuildCommand}}` and `{{.ArtifactPath}}`; the renderer also exposes
`TargetOS`, `TargetFormat` (enum names), and `TargetTriple`. The resolved command
and output path are stored on the task at creation. Unknown template variables
and empty rendered recipes are rejected.

Task creation packages each selected tome's script and assets into a tar.gz
archive. A task's `bundle` Asset contains these archives in a tar file named by
tome ID (`<id>.tar.gz`). The snapshot, bundle, and task are saved in one
transaction. Source tome or asset edits/deletions do not alter saved build inputs.
The bundle and final `artifact` are separate relationships.

## Build Caching

By default the Docker executor builds into a throwaway container, so every build
re-downloads dependencies (the crates.io index, all dependency crates, git
dependencies, Go modules) and recompiles from scratch — including the entire
dependency tree. This makes repeated builds slow. To speed them up, the Docker
executor can mount a persistent Docker **volume** that survives container
teardown and reuse it across builds.

Enable it with the `cache_volume` builder config option (a Docker volume name). It
defaults to `realm-build-cache`, so caching is on for the Docker executor unless
explicitly disabled:

```yaml
# builder.yaml
executor: docker
cache_volume: realm-build-cache   # omit to keep the default; "" disables caching
```

When `cache_volume` is set, the executor:

1. Ensures the named Docker volume exists (created on first use; idempotent).
2. Mounts it into the build container at `/realm-cache` (override with
   `BuildSpec.CachePath` if needed).
3. Points the toolchain cache directories at the volume by exporting:
   - `CARGO_HOME=/realm-cache/cargo` (registry index, downloaded crates, git deps)
   - `CARGO_TARGET_DIR=/realm-cache/target/<key>` (compiled dependency artifacts)
   - `GOPATH=/realm-cache/gopath`, `GOMODCACHE=.../pkg/mod`, `GOCACHE=.../go-cache`
   - `REALM_CACHE_DIR=/realm-cache` (convenience hook for your scripts)

The `<key>` suffix isolates the cargo target directory per build recipe (image +
OS + each script), so concurrent builds of different profiles use independent
target dirs and can run in parallel, while repeated builds of the *same* profile
reuse the cached registry, crates, and compiled artifacts. Builds that share a
key are serialized within a single builder process to keep cargo from racing
inside the shared target directory.

Note: the cache volume name is a single shared string. If one Docker host runs
builders for more than one independent Tavern deployment (e.g. separate
engagements sharing a build host), they will silently share this volume (and its
cached dependencies and checkouts) unless you give each deployment a distinct
`cache_volume`.

### Getting the most out of the cache

The Docker executor resolves `{{.BuildDir}}` to a fixed `/tmp/realm-build` path
(the container already isolates builds), so the rendered recipe — and therefore
`CARGO_TARGET_DIR` — is stable across builds. That means cargo reuses **both** the
dependency tree **and** Realm's own workspace crates from the shared volume, so
the bulk of compile time is cached automatically; you don't need to do anything
special to get it. (The Local executor keeps a random, unique `BuildDir` and does
not use the cache volume.)

One optional optimization is to keep the git checkout itself on the volume so the
clone isn't re-downloaded every build (the container's `/tmp/realm-build` is
discarded on teardown). Have your setup script clone into `$REALM_CACHE_DIR/repo`,
e.g.:

```sh
# setup script
git -C "$REALM_CACHE_DIR/repo" pull 2>/dev/null || \
  git clone https://github.com/spellshift/realm.git "$REALM_CACHE_DIR/repo"
cd "$REALM_CACHE_DIR/repo"
# then ${BuildCommand} from the cached checkout
```

Caveat: if multiple build profiles share the same `cache_volume` and clone
different repositories (or different refs) into `$REALM_CACHE_DIR/repo`, they will
overwrite each other's checkout. Only do this when all profiles build the same
repo/ref, or give each profile its own `cache_volume`.

If the volume cannot be created (e.g. driver limits or insufficient permission),
the executor logs a warning and falls back to building without the cache rather
than failing the build.

Existing tasks without a snapshot remain queryable with a null
`profileAtCreation`. Builders skip them; recreate them explicitly to build with
the current profile. Historical configuration is not inferred or backfilled.
This preserves the recipe and embedded contents, but mutable external references
such as image tags and Git branches should be pinned when reproducible binaries
are required.

Artifact uploads create the output Asset and attach it to the task atomically.

## Build Task Defaults

The `createBuildTask` mutation requires `targetOS` and `profileID`. All other fields have sensible
defaults resolved server-side:

| Field | Default | Notes |
|-------|---------|-------|
| `targetFormat` | `BIN` | Can also be `CDYLIB` or `WINDOWS_SERVICE` (Windows only) |
| `buildImage` | `spellshift/devcontainer:main` | Docker image for the build container |
| `artifactPath` | Derived from `targetOS` | Path inside the container to extract the compiled binary |

## Artifact Extraction

Build tasks can specify an `artifact_path` field — the path inside the container where the
compiled binary or output file is written. If not specified, it is derived from the target
OS (e.g. `/home/vscode/realm/implants/target/x86_64-unknown-linux-musl/release/imix` for
Linux). When the build finishes successfully, the `DockerExecutor` copies the file from
the stopped container using Docker's `CopyFromContainer` API (which returns a tar archive),
extracts the first regular file, and returns the bytes in `BuildResult`. The builder client
then streams the artifact to Tavern via the `UploadBuildArtifact` RPC in 1 MB chunks. The
server creates an `Asset` entity. The artifact is downloadable via the existing CDN endpoint
at `GET /assets/download/{name}`.



## Local Executor

`LocalExecutor` (`--executor local`) runs build scripts directly on the builder host
instead of inside a Docker container. It exists to make the builder fast to test
(e.g. the e2e suite) without requiring a Docker daemon; it is **not** a sandboxed
or production-equivalent alternative to `DockerExecutor` and is only supported on
Linux (`Build` returns an error on any other `GOOS`).

Key differences from `DockerExecutor`:

- **Unsandboxed**: the build script runs as the builder process itself, with the
  full host filesystem and the builder's inherited environment (`os.Environ()`)
  available to it — not a container's isolated root. Scripts should not be trusted
  any more than the builder process itself is.
- **Workspace paths**: both executors write scripts to `<workspace>/scripts/` and
  tomes to `<workspace>/tomes/` before running them in order, and both expose
  `REALM_WORKSPACE_DIR` / `REALM_TOMES_DIR` env vars pointing at those paths.
  `LocalExecutor`'s workspace is a host temp dir (`/tmp/realm-build-*`);
  `DockerExecutor`'s workspace is mounted into the container at `/mnt`. A recipe
  that hardcodes `/mnt/tomes/...` instead of using `$REALM_TOMES_DIR` will work
  under Docker but not under the local executor.
- **Artifact extraction**: `DockerExecutor` resolves `artifactPath` inside the
  container's filesystem. `LocalExecutor` resolves it against the host: a
  relative path is confined to the workspace dir, but an absolute path is read
  directly from the host if it exists. Since `createBuildTask`/`createBuildProfile`
  are ADMIN-only, this is a trust boundary rather than an open vulnerability, but
  it means an absolute `artifactPath` can read any file visible to the builder
  process. Default artifact paths derived by `DeriveArtifactPath` assume a
  container filesystem layout (e.g.
  `/home/vscode/realm/implants/target/<triple>/release/imix`) and typically won't
  exist on a host, so local-mode recipes should override `artifactPath` with a
  workspace-relative path.

## Package Structure

| File | Purpose |
|------|---------|
| `auth.go` | gRPC unary and stream interceptors for mTLS authentication |
| `build_config.go` | Build defaults, target formats, IMIX config, build script generation |
| `ca.go` | Builder CA generation, persistence, and certificate signing |
| `client.go` | Builder client: mTLS credentials, polling loop, task execution |
| `config.go` | YAML configuration parsing and validation |
| `server.go` | gRPC server: `ClaimBuildTasks`, `StreamBuildTaskOutput`, `UploadBuildArtifact` |
| `rollback.go` | Transaction rollback helper (matches c2 pattern) |
| `executor/executor.go` | `Executor` interface, `BuildSpec`, and `BuildResult` definitions |
| `executor/docker.go` | `DockerExecutor`: runs builds in Docker containers |
| `executor/local.go` | `LocalExecutor`: runs builds directly on the host (Linux-only, testing) |
| `executor/mock.go` | `MockExecutor`: test double for unit tests |
| `proto/builder.proto` | Protobuf service definition |
| `builderpb/` | Generated protobuf Go code |
| `integration_test.go` | End-to-end test: registration, mTLS auth, task claiming |
| `executor_integration_test.go` | End-to-end test: claim → execute → submit flow |
