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

| Env Var | Description | Default | Required |
| ------- | ----------- | ------- | -------- |
| IMIX_CALLBACK_URI | URI for initial callbacks (must specify a scheme, e.g. `http://`) | `http://127.0.0.1:80` | No |
| IMIX_SERVER_PUBKEY | The public key for the tavern server. | - | Yes |
| IMIX_CALLBACK_INTERVAL | Duration between callbacks, in seconds. | `5` | No |
| IMIX_RETRY_INTERVAL | Duration to wait before restarting the agent loop if an error occurs, in seconds. | `5` | No |
| IMIX_PROXY_URI | Overide system settings for proxy URI over HTTP(S) (must specify a scheme, e.g. `https://`) | No proxy | No |
| IMIX_HOST_ID | Manually specify the host ID for this beacon. Supersedes the file on disk. | - | No |
| IMIX_RUN_ONCE | Imix will only do one callback and execution of queued tasks (may want to pair with runtime environment variable `IMIX_BEACON_ID`) | false | No |

Imix has run-time configuration, that may be specified using environment variables during execution.

| Env Var | Description | Default | Required |
| ------- | ----------- | ------- | -------- |
| IMIX_BEACON_ID | The identifier to be used during callback (must be globally unique) | Random UUIDv4 | No |
| IMIX_LOG | Log message level for debug builds. See below for more information. | INFO | No |

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

### Linux

```bash
rustup target add x86_64-unknown-linux-musl

sudo apt update
sudo apt install musl-tools
cd realm/implants/imix/
# To get a servers pubkey:
# curl $IMIX_CALLBACK_URI/status | jq -r '.Pubkey'
IMIX_SERVER_PUBKEY="<SERVER_PUBKEY>" cargo build --release --bin imix --target=x86_64-unknown-linux-musl
```

### MacOS

**MacOS does not support static compilation**
<https://developer.apple.com/forums/thread/706419>

**Cross compilation is more complicated than we'll support**
Check out this blog a starting point for cross compiling.
<https://wapl.es/rust/2019/02/17/rust-cross-compile-linux-to-macos.html/>

### Windows

```bash
rustup target add x86_64-pc-windows-gnu

sudo apt update
sudo apt install gcc-mingw-w64

# Build imix
cd realm/implants/imix/

# To get a servers pubkey:
# curl $IMIX_CALLBACK_URI/status | jq -r '.Pubkey'

# Build imix.exe
IMIX_SERVER_PUBKEY="<SERVER_PUBKEY>" cargo build --release --target=x86_64-pc-windows-gnu
# Build imix.svc.exe
IMIX_SERVER_PUBKEY="<SERVER_PUBKEY>" cargo build --release --features win_service --target=x86_64-pc-windows-gnu
# Build imix.dll
IMIX_SERVER_PUBKEY="<SERVER_PUBKEY>" cargo build --release --lib --target=x86_64-pc-windows-gnu
```
