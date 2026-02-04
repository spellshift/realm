---
title: Imix
tags:
 - User Guide
description: Imix User Guide
permalink: user-guide/imix
---
## Imix

Imix is an offensive security implant designed for stealthy communication and adversary emulation. It functions as a [Beacon](/user-guide/terminology#beacon), receiving [Eldritch](/user-guide/terminology#eldritch) packages called [Tomes](/user-guide/terminology#tome) from a central server ([Tavern](/admin-guide/tavern)) and evaluating them on the host system. It currently supports [gRPC over HTTP(s)](https://grpc.io/) as its primary communication mechanism, but can be extended to support additional transport channels (see the [developer guide](/dev-guide/tavern#agent-development) for more info).

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


Imix has run-time configuration, that may be specified using environment variables during execution.

| Env Var | Description | Default | Required |
| ------- | ----------- | ------- | -------- |
| IMIX_BEACON_ID | The identifier to be used during callback (must be globally unique) | Random UUIDv4 | No |
| IMIX_LOG | Log message level for debug builds. See below for more information. | INFO | No |



## Static cross compilation

**We strongly recommend building agents inside the provided devcontainer `.devcontainer`**
Building in the dev container limits variables that might cause issues and is the most tested way to compile.

**Imix requires a server public key so it can encrypt messages to and from the server check the server status page `http://example.com/status` or logs for `level=INFO msg="public key: <SERVER_PUBKEY_B64>"`. This base64 encoded string should be passed to the agent using the environment variable `IMIX_SERVER_PUBKEY`**

**🚨 Note:** You must cd into the imix directory `implants/imix/` not `implants/` in order to build the agent.

## Setting encryption key

By default imix will automatically collect the IMIX_CALLBACK_URI server's public key during the build process. This can be overridden by manually setting the `IMIX_SERVER_PUBKEY` environment variable but should only be necessary when using redirectors. Redirectors have no visibility into the realm encryption by design, this means that agents must be compiled with the upstream tavern instance's public key.

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
therefore we need to copy it into an accessible location.
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
# Build imix.exe - As a service
cargo build --release --features win_service --target=x86_64-pc-windows-gnu
# Build imix.dll
cargo build --release --lib --target=x86_64-pc-windows-gnu
```


## Advanced Configuration (IMIX_CONFIG)

For more complex setups, such as configuring multiple transports or specifying detailed transport options, you can use the `IMIX_CONFIG` environment variable. This variable accepts a YAML-formatted string.

**Note:** When `IMIX_CONFIG` is set, you cannot use `IMIX_CALLBACK_URI`, `IMIX_CALLBACK_INTERVAL`, or `IMIX_TRANSPORT_EXTRA_*`. All configuration must be provided within the YAML structure.

### YAML Structure

```yaml
transports:
  - URI: <string>
    type: <grpc|http1|dns>
    interval: <integer> # optional, seconds
    extra: <json_string> # required (use "" if none)
server_pubkey: <string> # optional - defaults to checking the first transport URI status page.
```

### Example: Multiple Transports

This example configures Imix to use two transports:
1.  A gRPC transport over HTTP.
2.  A DNS transport as a fallback or alternative.

```bash
export IMIX_CONFIG='
transports:
  - URI: "http://127.0.0.1:8000"
    type: "grpc"
    interval: 5
    extra: ""
  - URI: "dns://*"
    type: "dns"
    interval: 10
    extra: "{\"domain\": \"c2.example.com\", \"type\": \"txt\"}"
server_pubkey: "YOUR_SERVER_PUBKEY_HERE"
'

# Build with the configuration
cargo build --release --bin imix --target=x86_64-unknown-linux-musl
```

## Transport configuration

Imix supports pluggable transports making it easy to adapt to your environment. Out of the box it supports `grpc` (default), `http1` and `dns`. Each transport has a corresponding redirector subcommand in tavern. In order to use a non grpc transport a redirector that can speak to your transport is required.

### global configuration options
- `uri`: specifies the upstream server or redirector the agent should connect to eg. `https://example.com` custom ports can be specified as `https://example.com:8443`
- `interval`: the number of seconds between callbacks.
- `extra`: JSON dictionary for transport specific configuration. These are outlined below:

### grpc

The default GRPC transport uses GRPC over HTTP2 to communicate

**Extra Keys Supported:**
- `doh`: empty string or `cloudflare`
- `http_proxy`: the full URI of the http_proxy that imix should connect through. Eg. `http://127.0.0.1:3128`

This transport supports all eldritch functions.


### http1

The HTTP1 transport uses HTTP post requests to communicate to the redirector.

**Extra Keys Supported:**
- `doh`: empty string or `cloudflare`
- `http_proxy`: the full URI of the http_proxy that imix should connect through. Eg. `http://127.0.0.1:3128`


This transport doesn't support eldritch functions that require bi-directional streaming like reverse shell, or SOCKS5 proxying.


### dns

The DNS transport enables covert C2 communication by tunneling traffic through DNS queries and responses. This transport supports multiple DNS record types (TXT, A, AAAA).

This transport doesn't support eldritch functions that require bi-directional streaming like reverse shell, or SOCKS5 proxying.

*Note*: the uri parameter here is the dns server to communicate with. If `dns://*` is specififed the transport will attempt to use the local systems primary resolver. Custom ports can be specified with `dns://8.8.8.8:53`

**Extra Keys Supported:**
- `domain` - Base domain for DNS queries (e.g., `c2.example.com`)
- `type` (optional) - DNS record type: `txt` (default), `a`, or `aaaa`

*Note*: TXT records provide the best performance.

## Logging

At runtime, you may use the `IMIX_LOG` environment variable to control log levels and verbosity. See [these docs](https://docs.rs/pretty_env_logger/latest/pretty_env_logger/) for more information. **When building a release version of imix, logging is disabled** and is not included in the released binary.

## Installation

The install subcommand executes embedded tomes similar to golem.
It will loop through all embedded files looking for main.eldritch.
Each main.eldritch will execute in a new thread. This is done to allow imix to install redundantly or install additional (non dependent) tools.

Installation scripts are specified in the `realm/implants/imix/install_scripts` directory.

This feature is currently under active development, and may change. We'll do our best to keep these docs updates in the meantime.

## Functionality

Imix derives all its functionality from the eldritch language.
See the [Eldritch User Guide](/user-guide/eldritch) for more information.

## Task management

Imix can execute up to 127 threads concurrently after that the main imix thread will block behind other threads.
Every callback interval imix will query each active thread for new output and relay that back to the c2. This means even long running tasks will report their status as new data comes in.


## Identifying unique hosts

Imix communicates which host it's on to Tavern enabling operators to reliably perform per host actions. The default way that imix does this is through a file on disk. We recognize that this may be not ideal for many situations so we've also provided an environment override and made it easy for admins managing a realm deployment to change how the bot determines uniqueness.

Imix uses the `host_unique` library under `implants/lib/host_unique` to determine which host it's on. The `id` function will fail over all available options returning the first successful ID. If a method is unable to determine the uniqueness of a host it should return `None`.

We recommend that you use the `File` for the most reliability:

- Exists across reboots
- Guaranteed to be unique per host (because the bot creates it)
- Can be used by multiple instances of the beacon on the same host.

If you cannot use the `File` selector we highly recommend manually setting the `Env` selector with the environment variable `IMIX_HOST_ID`. This will override the `File` one avoiding writes to disk but must be managed by the operators.

For Windows hosts, a `Registry` selector is available, but must be enabled before compilation. See the [imix dev guide](/dev-guide/imix#host-selector) on how to enable it.

If all uniqueness selectors fail imix will randomly generate a UUID to avoid crashing.
This isn't ideal as in the UI each new beacon will appear as though it were on a new host.
