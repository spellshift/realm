---
title: Imix
tags:
 - User Guide
description: Imix User Guide
permalink: user-guide/imix
---
## What is Imix

Imix is the default agent for realm.
Imix currently only supports http callbacks which interact directly with the graphql API.

## Configuration

By default Imix is configured using a JSON file at run time.

The config is specified at run time with the `-c` flag.
For example:

```bash
./imix -c /tmp/imix-config.json
```

The imix config is as follows:

```json
{
    "service_configs": [
        {
            "name": "imix",
            "description": "Imix c2 agent",
            "executable_name": "imix",
            "executable_args": ""
        }
    ],
    "target_forward_connect_ip": "127.0.0.1",
    "target_name": "test1234",
    "callback_config": {
        "interval": 4,
        "jitter": 1,
        "timeout": 4,
        "c2_configs": [
        {
            "priority": 1,
            "uri": "http://127.0.0.1/grpc"
        }
        ]
    }
}
```

- `service_configs`: Defining persistence variables.
  - `name`: The name of the service to install as.
  - `description`: If possible set a description for the service.
  - `executable_name`: What imix should be named Eg. `not-supicious-serviced`.
  - `executable_args`: Args to append after the executable.
- `target_forward_connect_ip`: The IP address that you the red teamer would interact with the host through. This is to help keep track of agents when a hosts internal IP is different from the one you interact with in the case of a host behind a proxy.
- `target_name`: Currently unused.
- `callback_config`: Define where and when the agent should callback.
  - `interval`: Number of seconds between callbacks.
  - `jitter`: Currently unused.
  - `timeout`: The number of seconds to wait before aborting a connection attempt.
  - `c2_config` Define where the c2 should callback to.
    - `priority`: The index that a domain should have.
    - `uri`: The full URI of the callback endpoint.

## Installation

The install subcommand executes embedded tomes similar to golem.
It will loop through all embedded files looking for main.eld
Each main.eld will execute in a new thread. This is done to allow imix to install redundantly or install additional (non dependent) tools.

The install subcommand makes allows some variables to be passed form the user into the tomes through the -c flag.
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

Installation scripts are specified in the `realm/implants/imix/install_scripts` directeroy.

## Functionality

Imix derives all it's functionality from the eldritch language.
See the [Eldritch User Guide](/user-guide/eldritch) for more information.

## Task management

Imix can execute up to 127 threads concurrently after that the main imix thread will block behind other threads.
Every callback interval imix will query each active thread for new output and rely that back to the c2. This means even long running tasks will report their status as new data comes in.

## Static cross compiliation

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
