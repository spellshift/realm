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
    "service_configs": [],
    "target_forward_connect_ip": "127.0.0.1",
    "target_name": "test1234",
    "callback_config": {
        "interval": 4,
        "jitter": 1,
        "timeout": 4,
        "c2_configs": [
        {
            "priority": 1,
            "uri": "http://127.0.0.1/graphql"
        }
        ]
    }
}
```

- `service_configs`: Currently unused.
- `target_forward_connect_ip`: The IP address that you the red teamer would interact with the host through. This is to help keep track of agents when a hosts internal IP is different from the one you interact with in the case of a host behind a proxy.
- `target_name`: Currently unused.
- `callback_config`: Define where and when the agent should callback.
    - `interval`: Number of seconds between callbacks.
    - `jitter`: Currently unused.
    - `timeout`: The number of seconds to wait before aborting a connection attempt.
    - `c2_config` Define where the c2 should callback to.
        - `priority`: The index that a domain should have.
        - `uri`: The full URI of the callback endpoint.

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

apt update
apt install musl-tools

RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target=x86_64-unknown-linux-musl
```

### MacOS
```bash
rustup target add x86_64-apple-darwin

apt update
apt install llvm-dev libclang-dev clang libxml2-dev libz-dev
export MACOSX_CROSS_COMPILER=$HOME/macosx-cross-compiler
install -d $MACOSX_CROSS_COMPILER/osxcross
install -d $MACOSX_CROSS_COMPILER/cross-compiler
cd $MACOSX_CROSS_COMPILER
git clone https://github.com/tpoechtrager/osxcross && cd osxcross
git checkout 7c090bd8cd4ad28cf332f1d02267630d8f333c19

mv MacOSX10.10.sdk.tar.xz $MACOSX_CROSS_COMPILER/osxcross/tarballs/
UNATTENDED=yes OSX_VERSION_MIN=10.7 TARGET_DIR=$MACOSX_CROSS_COMPILER/cross-compiler ./build.sh

echo "[target.x86_64-apple-darwin]" >> $HOME/.cargo/config
find $MACOSX_CROSS_COMPILER -name x86_64-apple-darwin14-cc -printf 'linker = "%p"\n' >> $HOME/.cargo/config
echo >> $HOME/.cargo/config

C_INCLUDE_PATH=$MACOSX_CROSS_COMPILER/cross-compiler/SDK/MacOSX10.10.sdk/usr/include CC=$MACOSX_CROSS_COMPILER/cross-compiler/bin/x86_64-apple-darwin14-cc RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target=x86_64-apple-darwin
```

https://godot-rust.github.io/gdnative-book/export/macosx.html
https://wapl.es/rust/2019/02/17/rust-cross-compile-linux-to-macos.html/
