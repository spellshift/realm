export const TOMES_DOC = `
## Tomes

A [Tome](/user-guide/terminology#tome) is an [Eldritch](/user-guide/terminology#eldritch) package that can be run on one or more [Beacons](/user-guide/terminology#beacon). By default, Tavern includes several core [Tomes](/user-guide/terminology#tome) to get you started.

## Anatomy of a Tome

A [Tome](/user-guide/terminology#tome) has a well-defined structure consisting of three key components:

1. \`metadata.yml\`: This file serves as the [Tome's](/user-guide/terminology#tome) blueprint, containing essential information in YAML format.
2. \`main.eldritch\`: This file is where the magic happens. It contains the [Eldritch](/user-guide/terminology#eldritch) code evaluated by the [Tome](/user-guide/terminology#tome).
3. \`assets/\` (optional): [Tomes](/user-guide/terminology#tome) have the capability to leverage additional resources stored externally.

## Tome Metadata

The \`metadata.yml\` file specifies key information about a [Tome](/user-guide/terminology#tome):

| Name | Description | Required |
|------|-------------|----------|
| \`name\` | Display name of your [Tome](/user-guide/terminology#tome). | Yes |
| \`description\` | Provide a helpful description of functionality. | Yes |
| \`author\` | Your name/handle. | Yes |
| \`support_model\` | \`FIRST_PARTY\` or \`COMMUNITY\` | Yes |
| \`tactic\` | The relevant MITRE ATT&CK tactic (e.g. RECON, EXECUTION). | Yes |
| \`paramdefs\` | A list of parameters. | No |

### Tome Parameters

Parameters have:
- \`name\`: Identifier used to access the parameter via the \`input_params\` global.
- \`label\`: Display name.
- \`type\`: Type (e.g. \`string\`).
- \`placeholder\`: Example value.

#### Tome Parameter Example

\`\`\`yaml
paramdefs:
  - name: path
    type: string
    label: File path
    placeholder: "/etc/open*"
\`\`\`

#### Referencing Tome Parameters

Access via global \`input_params\` dictionary:

\`\`\`python
def print_path_param():
  path = input_params["path"]
  print(path)
\`\`\`

### Tome Metadata Example

\`\`\`yaml
name: List files
description: List the files and directories found at the path.
author: hulto
tactic: RECON
support_model: COMMUNITY
paramdefs:
  - name: path
    type: string
    label: File path
    placeholder: "/etc/open*"
\`\`\`

### Tome Assets

Referenced based on the **name of the directory** your \`main.eldritch\` file is in.
Example: \`mytome/assets/imix.exe\`.

## Tome writing best practices

### Example

\`\`\`python
IMPLANT_ASSET_PATH = "tome_name/assets/implant"
ASSET_SHA1SUM = "44e1bf82832580c11de07f57254bd4af837b658e"

def pre_flight(dest_bin_path):
    if not sys.is_linux():
      return False
    # ... checks ...
    return True

def main(dest_file_path):
  if not pre_flight(dest_file_path):
    return
  # ...
\`\`\`

### Passing input
\`\`\`python
main(input_params['DEST_FILE_PATH'])
\`\`\`

### Idempotence
Check current state before modifying (hash match? permissions match? process running?).
`;

export const ELDRITCH_DOC = `
# Eldritch Overview

Eldritch is a Pythonic red team Domain Specific Language (DSL) based on Starlark.
It supports most python syntax (list comprehension, string operations, etc.).

## Standard Library

- \`agent\`: Interactions with the agent.
- \`assets\`: Interact with files stored natively in the agent.
- \`crypto\`: Encrypt/decrypt or hash data.
- \`file\`: Interact with files on the system.
- \`http\`: Make http(s) requests.
- \`pivot\`: Identify and move between systems.
- \`process\`: Interact with processes.
- \`random\`: Generate random values.
- \`regex\`: Regular expressions.
- \`report\`: Structured data reporting.
- \`sys\`: General system capabilities.
- \`time\`: Time functions.

## Examples

### File Operations
\`\`\`python
file.read("/path/to/file")
file.write("/path/to/file", "content")
file.exists("/path")
file.is_dir("/path")
file.list("/path") # Returns List<Dict>
file.copy(src, dst)
file.remove(path)
\`\`\`

### Process Operations
\`\`\`python
process.list() # Returns List<Dict>
process.kill(pid)
process.info(pid)
\`\`\`

### System Info
\`\`\`python
sys.is_linux()
sys.is_windows()
sys.get_os()
sys.shell("cmd") # Returns { stdout, stderr, status }
sys.exec(path, args, disown, env)
\`\`\`

### HTTP
\`\`\`python
http.get(uri)
http.post(uri, body=...)
http.download(uri, dst)
\`\`\`

### Assets
\`\`\`python
assets.copy("tome_name/assets/file", "/tmp/file")
\`\`\`
`;
