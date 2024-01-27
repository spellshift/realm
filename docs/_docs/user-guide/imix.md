---
title: Imix
tags:
 - User Guide
description: Imix User Guide
permalink: user-guide/imix
---
## What is Imix

Imix is the default agent for realm.
Imix currently only supports http(s) callbacks to Tavern's gRPC API.

## Configuration

Imix has compile-time configuration, that may be specified using environment variables during `cargo build`.

| Env Var | Description | Default | Required |
| ------- | ----------- | ------- | -------- |
| IMIX_CALLBACK_URI | URI for initial callbacks (must specify a scheme, e.g. `http://`) | `http://127.0.0.1:80` | No |
| IMIX_CALLBACK_INTERVAL | Duration between callbacks, in seconds. | `5` | No |
| IMIX_RETRY_INTERVAL | Duration to wait before restarting the agent loop if an error occurs, in seconds. | `5` | No |

## Logging

At runtime, you may use the `IMIX_LOG` environment variable to control log levels and verbosity. See [these docs](https://docs.rs/pretty_env_logger/latest/pretty_env_logger/) for more information. When building a release version of imix, logging is disabled and is not included in the released binary.

## Installation

The install subcommand executes embedded tomes similar to golem.
It will loop through all embedded files looking for main.eldritch.
Each main.eldritch will execute in a new thread. This is done to allow imix to install redundantly or install additional (non dependent) tools.

The install subcommand allows some variables to be passed from the user into the tomes through the -c flag.
When specified input_params['custom_config'] is set to the file path of the config specified Eg.
./imix install -c /tmp/imix-config.json will result in input_params['custom_config'] = "/tmp/imix-config.json

Tomes can parse this with the following:

```python
def main():
    if 'custom_config' in input_params:
        config_data = crypto.from_json(file.read(input_params['custom_config']))
        print(config_data)

main()
```

Installation scripts are specified in the `realm/implants/imix/install_scripts` directory.

## Functionality

Imix derives all it's functionality from the eldritch language.
See the [Eldritch User Guide](/user-guide/eldritch) for more information.

## Task management

Imix can execute up to 127 threads concurrently after that the main imix thread will block behind other threads.
Every callback interval imix will query each active thread for new output and rely that back to the c2. This means even long running tasks will report their status as new data comes in.

## Static cross compilation

### Linux

```bash
rustup target add x86_64-unknown-linux-musl

sudo apt update
sudo apt install musl-tools

RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target=x86_64-unknown-linux-musl
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

# Build the reflective loader
cd realm/bin/reflective_loader
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --lib --target=x86_64-pc-windows-gnu
# You may have to adjust `LOADER_BYTES` include path in `dll_reflect_impl.rs` changing `x86_64-pc-windows-msvc` ---> `x86_64-pc-windows-gnu`

# Build imix
cd realm/implants/imix/
# Build imix.exe
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target=x86_64-pc-windows-gnu
# Build imix.dll
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --lib --target=x86_64-pc-windows-gnu
```
