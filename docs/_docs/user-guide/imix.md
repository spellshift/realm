---
title: Imix
tags:
 - User Guide
description: Imix User Guide
permalink: user-guide/imix
---
## Imix

Imix is an offensive security implant designed for stealthy communication and adversary emulation. It functions as a [Beacon](/user-guide/terminology#beacon), receiving [Eldritch](/user-guide/terminology#eldritch) packages called [Tomes](/user-guide/terminology#tome) from a central server ([Tavern](/admin-guide/tavern)) and evaluating them on the host system. It currently supports [gRPC over HTTP(s)](https://grpc.io/) as it's primary communication mechanism, but can be extended to support additional transport channels (see the [developer guide](/dev-guide/tavern#agent-development) for more info).

## Configuration

Imix has compile-time configuration, that may be specified using environment variables during `cargo build`.

**We strongly recommend building agents inside the provided devcontainer `.devcontainer`**
Building in the dev container limits variables that might cause issues and is the most tested way to compile.

| Env Var | Description | Default | Required |
| ------- | ----------- | ------- | -------- |
| IMIX_CALLBACK_URI | URI for initial callbacks (must specify a scheme, e.g. `http://` or `dns://`) | `http://127.0.0.1:8000` | No |
| IMIX_SERVER_PUBKEY | The public key for the tavern server (obtain from server using `curl $IMIX_CALLBACK_URI/status`). | automatic | Yes |
| IMIX_CALLBACK_INTERVAL | Duration between callbacks, in seconds. | `5` | No |
| IMIX_RETRY_INTERVAL | Duration to wait before restarting the agent loop if an error occurs, in seconds. | `5` | No |
| IMIX_HOST_ID | Manually specify the host ID for this beacon. Supersedes the file on disk. | - | No |
| IMIX_RUN_ONCE | Imix will only do one callback and execution of queued tasks (may want to pair with runtime environment variable `IMIX_BEACON_ID`) | false | No |
| IMIX_TRANSPORT_EXTRA_HTTP_PROXY | Overide system settings for proxy URI over HTTP(S) (must specify a scheme, e.g. `https://`) | No proxy | No |
| IMIX_TRANSPORT_EXTRA_DOH | Enable DoH, eventually specify which DoH service to use. Requires the grpc-doh flag. | No DoH. | No |


Imix has run-time configuration, that may be specified using environment variables during execution.

| Env Var | Description | Default | Required |
| ------- | ----------- | ------- | -------- |
| IMIX_BEACON_ID | The identifier to be used during callback (must be globally unique) | Random UUIDv4 | No |
| IMIX_LOG | Log message level for debug builds. See below for more information. | INFO | No |

## DNS Transport Configuration

The DNS transport enables covert C2 communication by tunneling traffic through DNS queries and responses. This transport supports multiple DNS record types (TXT, A, AAAA) and can use either specific DNS servers or the system's default resolver with automatic fallback.

### DNS URI Format

When using the DNS transport, configure `IMIX_CALLBACK_URI` with the following format:

```
dns://<server>?domain=<DOMAIN>[&type=<TYPE>]
```

**Parameters:**
- `<server>` - DNS server address(es), `*` to use system resolver, or comma-separated list (e.g., `8.8.8.8:53,1.1.1.1:53`)
- `domain` - Base domain for DNS queries (e.g., `c2.example.com`)
- `type` (optional) - DNS record type: `txt` (default), `a`, or `aaaa`

**Examples:**

```bash
# Use specific DNS server with TXT records (default)
export IMIX_CALLBACK_URI="dns://8.8.8.8:53?domain=c2.example.com"

# Use system resolver with fallbacks
export IMIX_CALLBACK_URI="dns://*?domain=c2.example.com"

# Use multiple DNS servers with A records
export IMIX_CALLBACK_URI="dns://8.8.8.8:53,1.1.1.1:53?domain=c2.example.com&type=a"

# Use AAAA records
export IMIX_CALLBACK_URI="dns://8.8.8.8:53?domain=c2.example.com&type=aaaa"
```

### DNS Resolver Fallback

When using `*` as the server, the agent uses system DNS servers followed by public resolvers (1.1.1.1, 8.8.8.8) as fallbacks. If system configuration cannot be read, only the public resolvers are used. When multiple servers are configured, the agent tries each server in order on every failed request until one succeeds, then uses the working server for subsequent requests.

### Record Types

| Type | Description | Use Case |
|------|-------------|----------|
| TXT | Text records (default) | Best throughput, data encoded in TXT RDATA |
| A | IPv4 address records | Lower profile, data encoded across multiple A records |
| AAAA | IPv6 address records | Medium profile, more data per record than A |

### Protocol Details

The DNS transport uses an async windowed protocol to handle UDP unreliability:

- **Chunked transmission**: Large requests are split into chunks that fit within DNS query limits (253 bytes total domain length)
- **Windowed sending**: Up to 10 packets are sent concurrently
- **ACK/NACK protocol**: The server responds with acknowledgments for received chunks and requests retransmission of missing chunks
- **Automatic retries**: Failed chunks are retried up to 3 times before the request fails
- **CRC32 verification**: Data integrity is verified using CRC32 checksums

**Limits:**
- Maximum data size: 50MB per request
- Maximum concurrent conversations on server: 10,000

## Logging

At runtime, you may use the `IMIX_LOG` environment variable to control log levels and verbosity. See [these docs](https://docs.rs/pretty_env_logger/latest/pretty_env_logger/) for more information. **When building a release version of imix, logging is disabled** and is not included in the released binary.

## Installation

The install subcommand executes embedded tomes similar to golem.
It will loop through all embedded files looking for main.eldritch.
Each main.eldritch will execute in a new thread. This is done to allow imix to install redundantly or install additional (non dependent) tools.

Installation scripts are specified in the `realm/implants/imix/install_scripts` directory.

This feature is currently under active development, and may change. We'll do our best to keep these docs updates in the meantime.

## Functionality

Imix derives all it's functionality from the eldritch language.
See the [Eldritch User Guide](/user-guide/eldritch) for more information.

## Task management

Imix can execute up to 127 threads concurrently after that the main imix thread will block behind other threads.
Every callback interval imix will query each active thread for new output and rely that back to the c2. This means even long running tasks will report their status as new data comes in.

## Proxy support

Imix's default `grpc` transport supports http and https proxies for outbound communication.
By default imix will try to determine the systems proxy settings:

- On Linux reading the environment variables `http_proxy` and then `https_proxy`
- On Windows - we cannot automatically determine the default proxy
- On MacOS - we cannot automatically determine the default proxy
- On FreeBSD - we cannot automatically determine the default proxy

## Identifying unique hosts

Imix communicates which host it's on to Tavern enabling operators to reliably perform per host actions. The default way that imix does this is through a file on disk. We recognize that this may be un-ideal for many situations so we've also provided an environment override and made it easy for admins managing a realm deployment to change how the bot determines uniqueness.

Imix uses the `host_unique` library under `implants/lib/host_unique` to determine which host it's on. The `id` function will fail over all available options returning the first successful ID. If a method is unable to determine the uniqueness of a host it should return `None`.

We recommend that you use the `File` for the most reliability:

- Exists across reboots
- Guaranteed to be unique per host (because the bot creates it)
- Can be used by multiple instances of the beacon on the same host.

If you cannot use the `File` selector we highly recommend manually setting the `Env` selector with the environment variable `IMIX_HOST_ID`. This will override the `File` one avoiding writes to disk but must be managed by the operators.

For Windows hosts, a `Registry` selector is available, but must be enabled before compilation. See the [imix dev guide](/dev-guide/imix#host-selector) on how to enable it.

If all uniqueness selectors fail imix will randomly generate a UUID to avoid crashing.
This isn't ideal as in the UI each new beacon will appear as thought it were on a new host.

## Static cross compilation

**We strongly recommend building agents inside the provided devcontainer `.devcontainer`**
Building in the dev container limits variables that might cause issues and is the most tested way to compile.

**Imix requires a server public key so it can encrypt messsages to and from the server check the server log for `level=INFO msg="public key: <SERVER_PUBKEY_B64>"`. This base64 encoded string should be passed to the agent using the environment variable `IMIX_SERVER_PUBKEY`**

## Optional build flags

These flags are passed to cargo build Eg.:
`cargo build --release --bin imix  --bin imix --target=x86_64-unknown-linux-musl --features foo-bar`

- `--features grpc-doh` - Enable DNS over HTTP using cloudflare DNS for the grpc transport
- `--features http1 --no-default-features` - Changes the default grpc transport to use HTTP/1.1. Requires running the http redirector.
- `--features dns --no-default-features` - Changes the default grpc transport to use DNS. Requires running the dns redirector. See the [DNS Transport Configuration](#dns-transport-configuration) section for more information on how to configure the DNS transport URI.

## Setting encryption key

By default imix will automatically collect the IMIX_CALLBACK_URI server's public key during the build process. This can be overridden by manually setinng the `IMIX_SERVER_PUBKEY` environment variable but should only be necesarry when using redirectors. Redirectors have no visibliity into the realm encryption by design, this means that agents must be compiled with the upstream tavern instance's public key.

A server's public key can be found using:
```bash
export IMIX_SERVER_PUBKEY="$(curl $IMIX_CALLBACK_URI/status | jq -r '.Pubkey')"
```

### Linux

```bash
rustup target add x86_64-unknown-linux-musl

sudo apt update
sudo apt install musl-tools
cd realm/implants/imix/
export IMIX_CALLBACK_URI="http://localhost"

cargo build --release --bin imix --target=x86_64-unknown-linux-musl
```

### MacOS

**MacOS does not support static compilation**
<https://developer.apple.com/forums/thread/706419>

[Apple's SDK and XCode TOS](https://www.apple.com/legal/sla/docs/xcode.pdf) require compilation be performed on apple hardware. Rust doesn't support cross compiling Linux -> MacOS out of the box due to dependencies on the above SDKs. In order to cross compile you first need to make the SDK available to the runtime. Below we've documented how you can compile MacOS binaries from the Linux devcontainer.

#### Setup
Setup the MacOS SDK in a place that docker can access.
Rancher desktop doesn't allow you to mount folders besides ~/ and /tmp/
therefore we need to copy it into an accesible location.
Run the following on your MacOS host:

```bash
sudo cp -r $(readlink -f $(xcrun --sdk macosx --show-sdk-path)) ~/MacOSX.sdk
```

Modify .devcontainer/devcontainer.json by uncommenting the MacOSX.sdk mount. This will expose the newly copied SDK into the container allowing cargo to link against the MacOS SDK.
```json
    "mounts": [
	 	"source=${localEnv:HOME}${localEnv:USERPROFILE}/MacOSX.sdk,target=/MacOSX.sdk,readonly,type=bind"
    ],
```

#### Build
*Reopen realm in devcontainer*
```bash
cd realm/implants/imix/
# Tell the linker to use the MacOSX.sdk
export SDKROOT="/MacOSX.sdk/"; export RUSTFLAGS="-Clink-arg=-isysroot -Clink-arg=/MacOSX.sdk -Clink-arg=-F/MacOSX.sdk/System/Library/Frameworks -Clink-arg=-L/MacOSX.sdk/usr/lib -Clink-arg=-lresolv"

export IMIX_SERVER_PUBKEY="<SERVER_PUBKEY>"

cargo zigbuild  --release --target aarch64-apple-darwin
```


### Windows

```bash
# Build imix
cd realm/implants/imix/

export IMIX_CALLBACK_URI="http://localhost"

# Build imix.exe
 cargo build --release --target=x86_64-pc-windows-gnu
# Build imix.svc.exe
cargo build --release --features win_service --target=x86_64-pc-windows-gnu
# Build imix.dll
cargo build --release --lib --target=x86_64-pc-windows-gnu
```
