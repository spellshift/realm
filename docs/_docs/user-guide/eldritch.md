---
title: Eldritch
tags:
 - User Guide
description: Eldritch User Guide
permalink: user-guide/eldritch
---
# Overview

Eldritch is a Pythonic red team Domain Specific Language (DSL) based on [starlark](https://github.com/facebookexperimental/starlark-rust). It uses and supports most python syntax and basic functionality such as list comprehension, string operations (`lower()`, `join()`, `replace()`, etc.), and built-in methods (`any()`, `dir()`, `hex()`, `sorted()`, etc.). For more details on the supported functionality not listed here, please consult the [Starlark Spec Reference](https://github.com/bazelbuild/starlark/blob/master/spec.md), but for the most part you can treat this like basic Python with extra red team functionality.

Eldritch is a small interpreter that can be embedded into a c2 agent as it is with Golem and Imix.
By embedding the interpreter into the agent conditional logic can be quickly evaluated without requiring multiple callbacks.

**Trying to create a tome? Check out the guide in [Golem](/user-guide/golem).**

## Examples

_Kill a specific process name_

```python
for p in process.list():
    if p['name'] == "golem":
        process.kill(p['pid'])
```

_Copy your current executable somewhere else_

```python
cur_bin_path = process.info()['exe']
dest_path = '/tmp/win'
file.copy(cur_bin_path, dest_path)
file.remove(cur_bin_path)
```

_Parse a JSON file_

```python
json_str = file.read("/tmp/config.json")
config_data = crypto.from_json(json_str)
print(config_data['key1'])
```

## Data types

Eldritch currently only supports the [default starlark data types.](https://github.com/facebookexperimental/starlark-rust/blob/main/docs/types.md)

## Error handling

Eldritch doesn't implement any form of error handling. If a function fails it will stop the tome from completing execution. There is no way to recover after a function has errored.

If you're using a function that has a chance to error (functions that do file / network IO) test preemptively with function like `is_file`, `is_dir`, `is_windows`, etc.

For example:

```python
def read_passwd():
    if sys.is_linux():
        if file.is_file("/etc/passwd"):
            print(file.read("/etc/passwd"))
read_passwd()
```

```python
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
```

# Built-Ins

### abs

`abs(x) -> Int | Float`

The **abs** method returns the absolute value of a number.

### all

`all(iterable) -> Bool`

The **all** method returns True if all elements of the iterable are true (or if the iterable is empty).

### any

`any(iterable) -> Bool`

The **any** method returns True if any element of the iterable is true. If the iterable is empty, return False.

### assert

`assert(condition) -> None`

The **assert** method aborts execution if the condition is false.

### assert_eq

`assert_eq(a, b) -> None`

The **assert_eq** method aborts execution if `a` is not equal to `b`.

### bool

`bool(x) -> Bool`

The **bool** method converts a value to a Boolean. Returns True when the argument x is true, False otherwise.

### bytes

`bytes(source) -> Bytes`

The **bytes** method creates a bytes object.
- If source is an integer, the array will have that size and will be initialized with null bytes.
- If source is a string, it will be converted using UTF-8 encoding.
- If source is an iterable, it must be an iterable of integers in the range 0 <= x < 256.

### chr

`chr(i) -> Str`

The **chr** method returns the string representing a character whose Unicode code point is the integer `i`.

### dict

`dict(**kwargs) -> Dict`
`dict(iterable, **kwargs) -> Dict`

The **dict** method creates a new dictionary.

### dir

`dir([object]) -> List<str>`

The **dir** method returns a list of valid attributes for the object. Without arguments, return the list of names in the current local scope.

### enumerate

`enumerate(iterable, start=0) -> List<Tuple>`

The **enumerate** method returns a list of tuples containing (index, value) pairs.

### eprint

`eprint(*args) -> None`

The **eprint** method prints objects to the standard error output, separated by spaces.

### fail

`fail(message) -> None`

The **fail** method aborts execution with an error message.

### filter

`filter(function, iterable) -> List`

The **filter** method constructs a list from those elements of iterable for which function returns true. If function is None, the identity function is assumed.

### float

`float(x) -> Float`

The **float** method converts a number or string to a floating point number.

### hex

`hex(x) -> Str`

The **hex** method converts an integer number to a lowercase hexadecimal string prefixed with "0x".

### int

`int(x, base=10) -> Int`

The **int** method converts a number or string to an integer.

### len

`len(s) -> Int`

The **len** method returns the length of an object. The argument may be a sequence (such as a string, bytes, tuple, list, or range) or a collection (such as a dictionary or set).

### libs

`libs() -> List<str>`

The **libs** method lists all registered libraries.

### list

`list([iterable]) -> List`

The **list** method creates a list.

### map

`map(function, iterable) -> List`

The **map** method applies function to every item of iterable and returns a list of the results.

### max

`max(iterable, *args) -> Any`

The **max** method returns the largest item in an iterable or the largest of two or more arguments.

### min

`min(iterable, *args) -> Any`

The **min** method returns the smallest item in an iterable or the smallest of two or more arguments.

### ord

`ord(c) -> Int`

The **ord** method returns an integer representing the Unicode code point of that character.

### pprint

`pprint(object, indent=2) -> None`

The **pprint** method pretty-prints an object.

### print

`print(*args) -> None`

The **print** method prints objects to the standard output, separated by spaces.

### range

`range(stop) -> List<Int>`
`range(start, stop[, step]) -> List<Int>`

The **range** method returns a list of arithmetic progressions.

### reduce

`reduce(function, iterable[, initializer]) -> Any`

The **reduce** method applies a function of two arguments cumulatively to the items of a sequence, from left to right, so as to reduce the sequence to a single value.

### repr

`repr(object) -> Str`

The **repr** method returns a string containing a printable representation of an object.

### reversed

`reversed(seq) -> List`

The **reversed** method returns a list with the elements of the sequence in reverse order.

### set

`set([iterable]) -> Set`

The **set** method creates a new set object.

### sorted

`sorted(iterable, *, key=None, reverse=False) -> List`

The **sorted** method returns a new sorted list from the items in iterable.

### str

`str(object) -> Str`

The **str** method returns a string containing a nicely printable representation of an object.

### tprint

`tprint(list_of_dicts) -> None`

The **tprint** method prints a list of dictionaries as a markdown table.

### tuple

`tuple([iterable]) -> Tuple`

The **tuple** method creates a tuple.

### type

`type(object) -> Str`

The **type** method returns the type of the object.

### zip

`zip(*iterables) -> List<Tuple>`

The **zip** method returns a list of tuples, where the i-th tuple contains the i-th element from each of the argument sequences or iterables.

# Standard Library

The standard library is the default functionality that eldritch provides. It contains the following libraries:

- `agent` - Used for meta-style interactions with the agent itself.
- `assets` - Used to interact with files stored natively in the agent.
- `crypto` - Used to encrypt/decrypt or hash data.
- `file` - Used to interact with files on the system.
- `http` - Used to make http(s) requests from the agent.
- `pivot` - Used to identify and move between systems.
- `process` - Used to interact with processes on the system.
- `random` - Used to generate cryptographically secure random values.
- `regex` - Regular expression capabilities for operating on strings.
- `report` - Structured data reporting capabilities.
- `sys` - General system capabilities can include loading libraries, or information about the current context.
- `time` - General functions for obtaining and formatting time, also add delays into code.

**🚨 DANGER 🚨: Name shadowing**

Do not use the standard library names as local variables as it will prevent you from accessing library functions.
For example, if you do:

```rust
for file in file.list("/home/"):
    print(file["file_name"])
```

The file library will become inaccessible.

It may even raise an error: `error: Local variable 'file' referenced before assignment`

Instead we recommend using more descriptive names like:

```rust
for user_home_dir in file.list("/home/"):
    print(user_home_dir["file_name"])
```

---

## Agent

The `agent` library provides functions for meta-style interactions with the agent itself. It allows you to inspect its configuration, check transport details, or list and manage background tasks.

### agent._terminate_this_process_clowntown

`agent._terminate_this_process_clowntown() -> None`

**🚨 DANGER 🚨:** The **agent._terminate_this_process_clowntown** method terminates the agent process immediately by calling `std::process::exit(0)`. This effectively kills the agent and should be used with extreme caution. This function does not return as the process exits.

### agent.get_config

`agent.get_config() -> Dict<str, Value>`

The **agent.get_config** method returns the current configuration of the agent as a dictionary containing configuration keys and values. This method will error if the configuration cannot be retrieved.

### agent.get_transport

`agent.get_transport() -> str`

The **agent.get_transport** method returns the name of the currently active transport (e.g., "http", "grpc", "dns").

### agent.list_transports

`agent.list_transports() -> List<str>`

The **agent.list_transports** method returns a list of available transport names supported by the agent.

### agent.get_callback_interval

`agent.get_callback_interval() -> int`

The **agent.get_callback_interval** method returns the current callback interval in seconds.

### agent.list_tasks

`agent.list_tasks() -> List<Dict>`

The **agent.list_tasks** method returns a list of dictionaries representing the currently running or queued background tasks on the agent. Each dictionary contains task metadata and status.

```python
>>> agent.list_tasks()
[{"id": 42949672964, "quest_name": "The Nightmare of the Netherworld Nexus"}]
```

### agent.stop_task

`agent.stop_task(task_id: int) -> None`

The **agent.stop_task** method stops a specific background task by its ID. If the task cannot be stopped or does not exist, the method will error.


### agent.set_callback_interval

`agent.set_callback_interval(new_interval: int) -> None`

The **agent.set_callback_interval** method takes an unsigned int and changes the
running agent's callback interval to the passed value as seconds. This configuration change will
not persist across agent reboots.

### agent.set_callback_uri

`agent.set_callback_uri(new_uri: str) -> None`

The **agent.set_callback_uri** method takes an string and changes the
running agent's callback URI to the passed value. This configuration change will
not persist across agent reboots. NOTE: please ensure the passed URI path is correct
for the underlying `Transport` being used, as a URI can take many forms and we make no
assumptions on `Transport` requirements no checks are applied to the passed string.

---

## Assets

The `assets` library is used to interact with embedded files stored natively within the agent. It provides capabilities to list, read, or extract these files to disk for further execution or deployment.

### assets.copy

`assets.copy(src: str, dst: str) -> None`

The **assets.copy** method copies an embedded file from the agent to disk.
The `src` variable will be the path from the `embed_files_golem_prod` as the root dir.
For example `embed_files_golem_prod/sliver/agent-x64` can be referenced as `sliver/agent-x64`.
If `dst` exists it will be overwritten. If it doesn't exist the function will fail.

```python
def deploy_agent():
    if file.is_dir("/usr/bin"):
        assets.copy("sliver/agent-x64","/usr/bin/notsu")
        sys.exec("/usr/bin/notsu",[],true)
deploy_agent()
```

### assets.list

`assets.list() -> List<str>`

The **assets.list** method returns a list of asset names that the agent is aware of.

### assets.read_binary

`assets.read_binary(src: str) -> List<int>`

The **assets.read_binary** method returns a list of u32 numbers representing the asset files bytes.

### assets.read

`assets.read(src: str) -> str`

The **assets.read** method returns a UTF-8 string representation of the asset file.

---

## Crypto

The `crypto` library offers functionalities to encrypt, decrypt, and hash data. It includes support for algorithms like AES, MD5, SHA1, and SHA256, as well as helpers for base64 encoding and JSON parsing.

### crypto.aes_decrypt

`crypto.aes_decrypt(key: Bytes, iv: Bytes, data: Bytes) -> Bytes`

The **crypto.aes_decrypt** method decrypts the given data using the given key and IV (AES CBC).

### crypto.aes_encrypt

`crypto.aes_encrypt(key: Bytes, iv: Bytes, data: Bytes) -> Bytes`

The **crypto.aes_encrypt** method encrypts the given data using the given key and IV (AES CBC).

### crypto.aes_decrypt_file

`crypto.aes_decrypt_file(src: str, dst: str, key: str) -> None`

The **crypto.aes_decrypt_file** method decrypts the given src file using the given key and writes it to disk at the dst location.

Key must be 16 Bytes (Characters)

### crypto.aes_encrypt_file

`crypto.aes_encrypt_file(src: str, dst: str, key: str) -> None`

The **crypto.aes_encrypt_file** method encrypts the given src file, encrypts it using the given key and writes it to disk at the dst location.

Key must be 16 Bytes (Characters)

### crypto.encode_b64

`crypto.encode_b64(content: str, encode_type: Optional<str>) -> str`

The **crypto.encode_b64** method encodes the given text using the given base64 encoding method. Valid methods include:

- STANDARD (default)
- STANDARD_NO_PAD
- URL_SAFE
- URL_SAFE_NO_PAD

### crypto.decode_b64

`crypto.decode_b64(content: str, decode_type: Optional<str>) -> str`

The **crypto.decode_b64** method encodes the given text using the given base64 decoding method. Valid methods include:

- STANDARD (default)
- STANDARD_NO_PAD
- URL_SAFE
- URL_SAFE_NO_PAD

### crypto.from_json

`crypto.from_json(content: str) -> Value`

The **crypto.from_json** method converts JSON text to an object of correct type.

```python
crypto.from_json("{\"foo\":\"bar\"}")
{
    "foo": "bar"
}
```

### crypto.is_json

`crypto.is_json(content: str) -> bool`

The **crypto.is_json** tests if JSON is valid.

```python
crypto.is_json("{\"foo\":\"bar\"}")
True
```

```python
crypto.is_json("foobar")
False
```

### crypto.to_json

`crypto.to_json(content: Value) -> str`

The **crypto.to_json** method converts given type to JSON text.

```python
crypto.to_json({"foo": "bar"})
"{\"foo\":\"bar\"}"
```

### crypto.hash_file

`crypto.hash_file(file: str, algo: str) -> str`

The **crypto.hash_file** method will produce the hash of the given file's contents. Valid algorithms include:

- MD5
- SHA1
- SHA256
- SHA512

### crypto.md5

`crypto.md5(data: Bytes) -> str`

The **crypto.md5** method calculates the MD5 hash of the provided data.

### crypto.sha1

`crypto.sha1(data: Bytes) -> str`

The **crypto.sha1** method calculates the SHA1 hash of the provided data.

### crypto.sha256

`crypto.sha256(data: Bytes) -> str`

The **crypto.sha256** method calculates the SHA256 hash of the provided data.

---

## File

The `file` library gives you comprehensive control to interact with files and directories on the host system. It includes methods for reading, writing, moving, copying, and compressing files, as well as searching and timestomping.

### file.append

`file.append(path: str, content: str) -> None`

The **file.append** method appends the `content` to file at `path`. If no file exists at path create the file with the content.

### file.compress

`file.compress(src: str, dst: str) -> None`

The **file.compress** method compresses a file using the gzip algorithm. If the destination file doesn't exist it will be created. If the source file doesn't exist an error will be thrown. If the source path is a directory the contents will be placed in a tar archive and then compressed.

### file.copy

`file.copy(src: str, dst: str) -> None`

The **file.copy** method copies a file from `src` path to `dst` path. If `dst` file doesn't exist it will be created.

### file.decompress

`file.decompress(src: str, dst: str) -> None`

The **file.decompress** method decompresses a file using the gzip algorithm. If the destination file doesn't exist it will be created. If the source file doesn't exist an error will be thrown. If the output path is a tar archive, the contents will be extracted to a directory at the `dst` path. Note the original directory will also be added to the new directory.

```python
file.compress('/home/bob/.ssh', '/tmp/bob_ssh.tar.gz')
file.decompress('/tmp/bob_ssh.tar.gz', '/tmp/bob_ssh_output')
# Files will exist in /tmp/bob_ssh_output/.ssh/*
```

### file.exists

`file.exists(path: str) -> bool`

The **file.exists** method checks if a file or directory exists at the path specified.

### file.follow

`file.follow(path: str, fn: function(str)) -> None`

The **file.follow** method will call `fn(line)` for any new `line` that is added to the file (such as from `bash_history` and other logs).

```python
# Print every line added to bob's bash history
file.follow('/home/bob/.bash_history', print)
```

### file.is_dir

`file.is_dir(path: str) -> bool`

The **file.is_dir** method checks if a path exists and is a directory. If it doesn't exist or is not a directory it will return `False`.

### file.is_file

`file.is_file(path: str) -> bool`

The **file.is_file** method checks if a path exists and is a file. If it doesn't exist or is not a file it will return `False`.

### file.list

`file.list(path: str) -> List<Dict>`

The **file.list** method returns a list of files at the specified path. The path is relative to your current working directory and can be traversed with `../`.
This function also supports globbing with `*` for example:

```python
file.list("/home/*/.bash_history") # List all files called .bash_history in sub dirs of `/home/`
file.list("/etc/*ssh*") # List the contents of all dirs that have `ssh` in the name and all files in etc with `ssh` in the name
file.list("\\\\127.0.0.1\\c$\\Windows\\*.yml") # List files over UNC paths
```

Each file is represented by a Dict type.
Here is an example of the Dict layout:

```json
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
```

### file.list_recent

`file.list_recent(path: str, limit: int) -> List<str>`

The **file.list_recent** method returns a list of file paths in the specified directory, sorted by most recent modification time. The scan is recursive.

- `path`: The directory to scan. Defaults to `/`.
- `limit`: The maximum number of files to return. Defaults to `10`.

```python
# Get the 5 most recently modified files in /var/log
recent_logs = file.list_recent("/var/log", 5)
for log in recent_logs:
    print(log)
```

### file.mkdir

`file.mkdir(path: str, parent: Option<bool>) -> None`

The **file.mkdir** method will make a new directory at `path`. If the parent directory does not exist or the directory cannot be created, it will error; unless the `parent` parameter is passed as `True`.

### file.move

`file.move(src: str, dst: str) -> None`

The **file.move** method moves a file or directory from `src` to `dst`. If the `dst` directory or file exists it will be deleted before being replaced to ensure consistency across systems.

### file.parent_dir

`file.parent_dir(path: str) -> str`

The **file.parent_dir** method returns the parent directory of a give path. Eg `/etc/ssh/sshd_config` -> `/etc/ssh`

### file.pwd

`file.pwd() -> Option<str>`

The **file.pwd** method returns the current working directory of the process. If it could not be determined, `None` is returned.

### file.read

`file.read(path: str) -> str`

The **file.read** method will read the contents of a file. If the file or directory doesn't exist the method will error to avoid this ensure the file exists, and you have permission to read it.
This function supports globbing with `*` for example:

```python
file.read("/home/*/.bash_history") # Read all files called .bash_history in sub dirs of `/home/`
file.read("/etc/*ssh*") # Read the contents of all files that have `ssh` in the name. Will error if a dir is found.
file.read("\\\\127.0.0.1\\c$\\Windows\\Temp\\metadata.yml") # Read file over Windows UNC
```

### file.read_binary

`file.read(path: str) -> List<int>`

The **file.read_binary** method will read the contents of a file, **returning as a list of bytes**. If the file or directory doesn't exist the method will error to avoid this ensure the file exists, and you have permission to read it.
This function supports globbing with `*` for example:

```python
file.read_binary("/home/*/.bash_history") # Read all files called .bash_history in sub dirs of `/home/`
file.read_binary("/etc/*ssh*") # Read the contents of all files that have `ssh` in the name. Will error if a dir is found.
file.read_binary("\\\\127.0.0.1\\c$\\Windows\\Temp\\metadata.yml") # Read file over Windows UNC
```

### file.remove

`file.remove(path: str) -> None`

The **file.remove** method deletes a file or directory (and it's contents) specified by path.

### file.replace

`file.replace(path: str, pattern: str, value: str) -> None`

The **file.replace** method finds the first string matching a regex pattern in the specified file and replaces them with the value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

### file.replace_all

`file.replace_all(path: str, pattern: str, value: str) -> None`

The **file.replace_all** method finds all strings matching a regex pattern in the specified file and replaces them with the value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

### file.temp_file

`file.temp_file(name: Option<str>) -> str`

The ** file.temp** method returns the path of a new temporary file with a random filename or the optional filename provided as an argument.

### file.template

`file.template(template_path: str, dst: str, args: Dict<String, Value>, autoescape: bool) -> None`

The **file.template** method reads a Jinja2 template file from disk, fill in the variables using `args` and then write it to the destination specified.
If the destination file doesn't exist it will be created (if the parent directory exists). If the destination file does exist it will be overwritten.
The `args` dictionary currently supports values of: `int`, `str`, and `List`.
`autoescape` when `True` will perform HTML character escapes according to the [OWASP XSS guidelines](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)

### file.timestomp

`file.timestomp(path: str, mtime: Option<int|str>, atime: Option<int|str>, ctime: Option<int|str>, ref_file: Option<str>) -> None`

The **file.timestomp** method modifies the timestamps of a file.
It can use specific timestamps (epoch seconds or string format) or copy timestamps from a reference file.

### file.write

`file.write(path: str, content: str) -> None`

The **file.write** method writes to a given file path with the given content.
If a file already exists at this path, the method will overwite it. If a directory
already exists at the path the method will error.

### file.find

`file.find(path: str, name: Option<str>, file_type: Option<str>, permissions: Option<int>, modified_time: Option<int>, create_time: Option<int>) -> List<str>`

The **file.find** method finds all files matching the used parameters. Returns file path for all matching items.

- name: Checks if file name contains provided input
- file_type: Checks for 'file' or 'dir' for files or directories, respectively.
- permissions: On UNIX systems, takes numerical input of standard unix permissions (rwxrwxrwx == 777). On Windows, takes 1 or 0, 1 if file is read only.
- modified_time: Checks if last modified time matches input specified in time since EPOCH
- create_time: Checks if last modified time matches input specified in time since EPOCH

---

## HTTP

The `http` library allows the agent to send HTTP and HTTPS requests over the network. You can download files, submit form data, or interact with external REST APIs directly from the agent.

The HTTP library also allows the user to allow the http client to ignore TLS validation via the `allow_insecure` optional parameter (defaults to `false`).

### http.download

`http.download(uri: str, dst: str, allow_insecure: Option<bool>) -> None`

The **http.download** method downloads a file at the URI specified in `uri` to the path specified in `dst`. If a file already exists at that location, it will be overwritten.

### http.get

`http.get(uri: str, query_params: Option<Dict<str, str>>, headers: Option<Dict<str, str>>, allow_insecure: Option<bool>) -> str`

The **http.get** method sends an HTTP GET request to the URI specified in `uri` with the optional query parameters specified in `query_params` and headers specified in `headers`, then return the response body as a string. Note: in order to conform with HTTP2+ all header names are transmuted to lowercase.

### http.post

`http.post(uri: str, body: Option<str>, form: Option<Dict<str, str>>, headers: Option<Dict<str, str>>, allow_insecure: Option<bool>) -> str`

The **http.post** method sends an HTTP POST request to the URI specified in `uri` with the optional request body specified by `body`, form parameters specified in `form`, and headers specified in `headers`, then return the response body as a string. Note: in order to conform with HTTP2+ all header names are transmuted to lowercase. Other Note: if a `body` and a `form` are supplied the value of `body` will be used.

---

## Pivot

The `pivot` library provides tools to identify and move laterally between systems on a network. It includes functionalities like port scanning, reverse shells, port forwarding, and executing commands remotely over SMB or SSH.

### pivot.arp_scan

`pivot.arp_scan(target_cidrs: List<str>) -> List<str>`

The **pivot.arp_scan** method is for enumerating hosts on the agent network without using TCP connect or ping.

- `target_cidrs` must be in a CIDR format eg. `127.0.0.1/32`. Domains and single IPs `example.com` / `127.0.0.1` cannot be passed.
- Must be running as `root` to use.
- Not supported on Windows

Results will be in the format:

```python
$> pivot.arp_scan(["192.168.1.1/32"])
```

**Success**

```json
[
    { "ip": "192.168.1.1", "mac": "ab:cd:ef:01:23:45", "interface": "eno0" }
]
```

**Failure**

```json
[]
```

### pivot.bind_proxy

`pivot.bind_proxy(listen_address: str, listen_port: int, username: str, password: str ) -> None`

The **pivot.bind_proxy** method is being proposed to provide users another option when trying to connect and pivot within an environment. This function will start a SOCKS5 proxy on the specified port and interface, with the specified username and password (if provided).

### pivot.ncat

`pivot.ncat(address: str, port: int, data: str, protocol: str ) -> str`

The **pivot.ncat** method allows a user to send arbitrary data over TCP/UDP to a host. If the server responds that response will be returned.

`protocol` must be `tcp`, or `udp` anything else will return an error `Protocol not supported please use: udp or tcp.`.

### pivot.port_forward

`pivot.port_forward(listen_address: str, listen_port: int, forward_address: str, forward_port:  int, str: protocol  ) -> None`

The **pivot.port_forward** method is being proposed to provide socat like functionality by forwarding traffic from a port on a local machine to a port on a different machine allowing traffic to be relayed.

### pivot.port_scan

`pivot.port_scan(target_cidrs: List<str>, ports: List<int>, protocol: str, timeout: int, fd_limit: Option<int>) -> List<str>`

The **pivot.port_scan** method allows users to scan TCP/UDP ports within the eldritch language.
Inputs:

- `target_cidrs` must be in a CIDR format eg. `127.0.0.1/32`. Domains and single IPs `example.com` / `127.0.0.1` cannot be passed.
- `ports` can be a list of any number of integers between 1 and 65535.
- `protocol` must be: `tcp` or `udp`. These are the only supported options.
- `timeout` is the number of seconds a scan will wait without a response before it's marked as `timeout`
- `fd_limit` is the maximum concurrent file descriptors/sockets (defaults to 64).

Results will be in the format:

```json
[
    { "ip": "127.0.0.1", "port": 22, "protocol": "tcp", "status": "open"},
    { "ip": "127.0.0.1", "port": 21, "protocol": "tcp", "status": "closed"},
    { "ip": "127.0.0.1", "port": 80, "protocol": "tcp", "status": "timeout"},
]
```

A port's status can be open, closed, or timeout:

|**State**|**Protocol**| **Meaning**                                          |
|---------|------------|------------------------------------------------------|
| open    | tcp        | Connection successful.                               |
| close   | tcp        | Connection refused.                                  |
| timeout | tcp        | Connection dropped or didn't respond.                |
| open    | udp        | Connection returned some data.                       |
| timeout | udp        | Connection was refused, dropped, or didn't respond.  |

Each IP in the specified CIDR will be returned regardless of if it returns any open ports.
Be mindful of this when scanning large CIDRs as it may create large return objects.

NOTE: Windows scans against `localhost`/`127.0.0.1` can behave unexpectedly or even treat the action as malicious. Eg. scanning ports 1-65535 against windows localhost may cause the stack to overflow or process to hang indefinitely.

### pivot.create_portal

`pivot.create_portal() -> None`

The **pivot.create_portal** method opens a portal bi-directional stream.

### pivot.reverse_shell_pty

`pivot.reverse_shell_pty(cmd: Optional<str>) -> None`

The **pivot.reverse_shell_pty** method spawns the provided command in a cross-platform PTY and opens a reverse shell over the agent's current transport (e.g. gRPC). If no command is provided, Windows will use `cmd.exe`. On other platforms, `/bin/bash` is used as a default, but if it does not exist then `/bin/sh` is used.

### pivot.reverse_shell_repl

`pivot.reverse_shell_repl() -> None`

The **pivot.reverse_shell_repl** method spawns a basic REPL-style reverse shell with an Eldritch interpreter.

### pivot.smb_exec

`pivot.smb_exec(target: str, port: int, username: str, password: str, hash: str, command: str) -> str`

The **pivot.smb_exec** method is being proposed to allow users a way to move between hosts running smb.

### pivot.ssh_copy

`pivot.ssh_copy(target: str, port: int, src: str, dst: str, username: str, password: Optional<str>, key: Optional<str>, key_password: Optional<str>, timeout: Optional<int>) -> str`

The **pivot.ssh_copy** method copies a local file to a remote system.
ssh_copy will return `"Success"` if successful and `"Failed to run handle_ssh_copy: ..."` on failure.
If the connection is successful but the copy writes a file error will be returned.
ssh_copy will overwrite the remote file if it exists.
The file directory the `dst` file exists in must exist in order for ssh_copy to work.

### pivot.ssh_exec

`pivot.ssh_exec(target: str, port: int, command: str, username: str, password: Optional<str>, key: Optional<str>, key_password: Optional<str>, timeout: Optional<int>) -> List<Dict>`

The **pivot.ssh_exec** method executes a command string on the remote host using the default shell.
Stdout returns the string result from the command output.
Stderr will return any errors from the SSH connection but not the command being executed.
Status will be equal to the code returned by the command being run and -1 in the event that the ssh connection raises an error.

```json
{
    "stdout": "uid=1000(kali) gid=1000(kali) groups=1000(kali),24(cdrom),25(floppy),27(sudo),29(audio),30(dip),44(video),46(plugdev),109(netdev),118(bluetooth),128(lpadmin),132(scanner),143(docker)\n",
    "stderr":"",
    "status": 0
}
```

---

## Process

The `process` library is used to interact with running processes on the local system. It provides functionalities to list processes, gather detailed information, enumerate network sockets, or terminate specific processes.

### process.info

`process.info(pid: Optional<int>) -> Dict`

The **process.info** method returns all information on a given process ID. Default is the current process.

```json
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
```

### process.kill

`process.kill(pid: int) -> None`

The **process.kill** method will kill a process using the KILL signal given its process id.

### process.list

`process.list() -> List<Dict>`

The **process.list** method returns a list of dictionaries that describe each process. The dictionaries follow the schema:

```json
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
```

### process.name

`process.name(pid: int) -> str`

The **process.name** method returns the name of the process from its given process ID.

### process.netstat

`process.netstat() -> List<Dict>`

The **process.netstat** method returns all information on TCP, UDP, and Unix sockets on the system. Will also return PID and Process Name of attached process, if one exists.

_Currently only shows LISTENING TCP connections_

```json
[
    {
        "socket_type": "TCP",
        "local_address": "127.0.0.1",
        "local_port": 46341,
        "pid": 2359037
    },
    ...
]
```

---

## Random

The `random` library is designed to enable generation of cryptographically secure random values without blocking execution. It allows you to create random booleans, integers, bytes, strings, and UUIDs.

### random.bool

`random.bool() -> bool`

The **random.bool** method returns a randomly sourced boolean value.

### random.bytes

`random.bytes(len: int) -> List<int>`

The **random.bytes** method returns a list of randomly generated bytes.

### random.int

`random.int(min: i32, max: i32) -> i32`

The **random.int** method returns randomly generated integer value between a specified range. The range is inclusive on the minimum and exclusive on the maximum.

### random.string

`random.string(length: uint, charset: Optional<str>) -> str`
The **random.string** method returns a randomly generated string of the specified length. If `charset` is not provided defaults to [Alphanumeric](https://docs.rs/rand_distr/latest/rand_distr/struct.Alphanumeric.html). Warning, the string is stored entirely in memory so exceptionally large files (multiple megabytes) can lead to performance issues.

### random.uuid

`random.uuid() -> str`

The **random.uuid** method returns a randomly generated UUID (v4).

---

## Regex

The `regex` library provides regular expression capabilities for operating on strings. Using Rust's regex syntax, you can match, extract, or replace substrings within larger text blocks. Be aware as the underlying implementation is written in Rust we rely on the Rust Regex Syntax as talked about [here](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html). Further, we only support a single capture group currently, defining more/less than one will cause the tome to error.

### regex.match_all

`regex.match_all(haystack: str, pattern: str) -> List<str>`

The **regex.match_all** method returns a list of capture group strings that matched the given pattern within the given haystack. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

### regex.match

`regex.match(haystack: str, pattern: str) -> str`

The **regex.match** method returns the first capture group string that matched the given pattern within the given
haystack. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

### regex.replace_all

`regex.replace_all(haystack: str, pattern: str, value: string) -> str`

The **regex.replace_all** method returns the given haystack with all the capture group strings that matched the given pattern replaced with the given value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

### regex.replace

`regex.replace(haystack: str, pattern: str, value: string) -> str`

The **regex.replace** method returns the given haystack with the first capture group string that matched the given pattern replaced with the given value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.

---

## Report

The `report` library is designed to enable reporting structured data to Tavern. You can use it to securely exfiltrate files, process lists, captured credentials, or screenshots from the host. Its API is still in the active development phase, so **future versions of Eldritch may break tomes that rely on this API**.

### report.file

`report.file(path: str) -> None`

Reports a file from the host that an Eldritch Tome is being evaluated on (e.g. a compromised system) to Tavern. It has a 1GB size limit, and will report the file in 1MB chunks. This process happens asynchronously, so after `report.file()` returns **there are no guarantees about when this file will be reported**. This means that if you delete the file immediately after reporting it, it may not be reported at all (race condition).

### report.process_list

`report.process_list(list: List<Dict>) -> None`

Reports a snapshot of the currently running processes on the host system. This should only be called with the entire process list (e.g. from calling `process.list()`), as it will replace Tavern's current list of processes for the host with this new snapshot.

### report.ssh_key

`report.ssh_key(username: str, key: str) -> None`

Reports a captured SSH Key credential to Tavern. It will automatically be associated with the host that the Eldritch Tome was being evaluated on.

### report.user_password

`report.user_password(username: str, password: str) -> None`

Reports a captured username & password combination to Tavern. It will automatically be associated with the host that the Eldritch Tome was being evaluated on.

### report.ntlm_hash

`report.ntlm_hash(username: str, password_hash: str) -> None`

Reports a captured username & ntlm hash combination to Tavern. It will automatically be associated with the host that the Eldritch Tome was being evaluated on.

### report.screenshot

`report.screenshot() -> None`

Reports a screenshot of all screens to Tavern.

---

## Sys

The `sys` library offers general system capabilities to retrieve context about the host environment. It provides functionalities to check the operating system, retrieve environment variables, query the registry, and run native shell commands.

### sys.dll_inject

`sys.dll_inject(dll_path: str, pid: int) -> None`

The **sys.dll_inject** method will attempt to inject a dll on disk into a remote process by using the `CreateRemoteThread` function call.

### sys.dll_reflect

`sys.dll_reflect(dll_bytes: List<int>, pid: int, function_name: str) -> None`

The **sys.dll_reflect** method will attempt to inject a dll from memory into a remote process by using the loader defined in `realm/bin/reflective_loader`.

The ints in dll_bytes will be cast down from int u32 ---> u8 in rust.
If your dll_bytes array contains a value greater than u8::MAX it will cause the function to fail. If you're doing any decryption in starlark make sure to be careful of the u8::MAX bound for each byte.

### sys.exec

`sys.exec(path: str, args: List<str>, disown: Optional<bool>, env_vars: Option<Dict<str, str>>, input: Option<str>) -> Dict`

The **sys.exec** method executes a program specified with `path` and passes the `args` list.
On *nix systems disown will run the process in the background disowned from the agent. This is done through double forking.
On Windows systems disown will run the process with detached stdin and stdout such that it won't block the tomes execution.
The `env_vars` will be a map of environment variables to be added to the process of the execution.

```python
sys.exec("/bin/bash",["-c", "whoami"])
{
    "stdout":"root\n",
    "stderr":"",
    "status":0,
}
sys.exec("/bin/bash",["-c", "ls /nofile"])
{
    "stdout":"",
    "stderr":"ls: cannot access '/nofile': No such file or directory\n",
    "status":2,
}
```

### sys.get_env

`sys.get_env() -> Dict`

The **sys.get_env** method returns a dictionary that describes the current process's environment variables.
An example is below:

```json
{
    "FOO": "BAR",
    "CWD": "/"
}
```

### sys.get_ip

`sys.get_ip() -> List<Dict>`

The **sys.get_ip** method returns a list of network interfaces as a dictionary. An example is available below:

```json
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
```

### sys.get_os

`sys.get_os() -> Dict`

The **sys.get_os** method returns a dictionary that describes the current system's OS.
An example is below:

```json
{
    "arch": "x86_64",
    "desktop_env": "Unknown: Unknown",
    "distro": "Debian GNU/Linux 10 (buster)",
    "platform": "PLATFORM_LINUX"
}
```

### sys.get_pid

`sys.get_pid() -> int`

The **sys.get_pid** method returns the process ID of the current process.
An example is below:

```python
$> sys.get_pid()
123456
```

### sys.get_reg

`sys.get_reg(reghive: str, regpath: str) -> Dict`

The **sys.get_reg** method returns the registry values at the requested registry path.
An example is below:

```python
$> sys.get_reg("HKEY_LOCAL_MACHINE","SOFTWARE\\Microsoft\\Windows\\CurrentVersion")
{
    "ProgramFilesDir": "C:\\Program Files",
    "CommonFilesDir": "C:\\Program Files\\Common Files",
    "ProgramFilesDir (x86)": "C:\\Program Files (x86)",
    "CommonFilesDir (x86)": "C:\\Program Files (x86)\\Common Files",
    "CommonW6432Dir": "C:\\Program Files\\Common Files",
    "DevicePath": "%SystemRoot%\\inf",
    "MediaPathUnexpanded": "%SystemRoot%\\Media",
    "ProgramFilesPath": "%ProgramFiles%",
    "ProgramW6432Dir": "C:\\Program Files",
    "SM_ConfigureProgramsName": "Set Program Access and Defaults",
    "SM_GamesName": "Games"
}
```

### sys.get_user

`sys.get_user() -> Dict`

The **sys.get_user** method returns a dictionary that describes the current process's running user.
On *Nix, will return UID, EUID, GID, EGID, and detailed user info for the UID and EUID mappings.
For users, will return name and groups of user.

```json
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
```

### sys.hostname

`sys.hostname() -> String`

The **sys.hostname** method returns a String containing the host's hostname.

### sys.is_bsd

`sys.is_bsd() -> bool`

The **sys.is_bsd** method returns `True` if on a `freebsd`, `netbsd`, or `openbsd` system and `False` on everything else.

### sys.is_linux

`sys.is_linux() -> bool`

The **sys.is_linux** method returns `True` if on a linux system and `False` on everything else.

### sys.is_macos

`sys.is_macos() -> bool`

The **sys.is_macos** method returns `True` if on a mac os system and `False` on everything else.

### sys.is_windows

`sys.is_windows() -> bool`

The **sys.is_windows** method returns `True` if on a windows system and `False` on everything else.

### sys.list_users

`sys.list_users() -> List<Dict>`

The **sys.list_users** method lists all user accounts on the system.

### sys.shell

`sys.shell(cmd: str) -> Dict`

The **sys.shell** Given a string run it in a native interpreter. On MacOS, Linux, and *nix/bsd systems this is `/bin/bash -c <your command>`. On Windows this is `cmd /C <your command>`. Stdout, stderr, and the status code will be returned to you as a dictionary with keys: `stdout`, `stderr`, `status`. For example:

```python
sys.shell("whoami")
{
    "stdout":"root\n",
    "stderr":"",
    "status":0,
}
sys.shell("ls /nofile")
{
    "stdout":"",
    "stderr":"ls: cannot access '/nofile': No such file or directory\n",
    "status":2,
}
```

### sys.write_reg_hex

`sys.write_reg_hex(reghive: str, regpath: str, regname: str, regtype: str, regvalue: str) -> Bool`

The **sys.write_reg_hex** method returns `True` if registry values are written to the requested registry path and accepts a hexstring as the value argument.
An example is below:

```python
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_SZ","deadbeef")
True
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_BINARY","deadbeef")
True
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_NONE","deadbeef")
True
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_EXPAND_SZ","deadbeef")
True
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_DWORD","deadbeef")
True
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_DWORD_BIG_ENDIAN","deadbeef")
True
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_LINK","deadbeef")
True
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_MULTI_SZ","dead,beef")
True
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_RESOURCE_LIST","deadbeef")
True
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_FULL_RESOURCE_DESCRIPTOR","deadbeef")
True
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_RESOURCE_REQUIREMENTS_LIST","deadbeef")
True
$> sys.write_reg_hex("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_QWORD","deadbeefdeadbeef")
True
```

### sys.write_reg_int

`sys.write_reg_int(reghive: str, regpath: str, regname: str, regtype: str, regvalue: int) -> Bool`

The **sys.write_reg_int** method returns `True` if registry values are written to the requested registry path and accepts an integer as the value argument.
An example is below:

```python
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_SZ",12345678)
True
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_BINARY",12345678)
True
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_NONE",12345678)
True
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_EXPAND_SZ",12345678)
True
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_DWORD",12345678)
True
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_DWORD_BIG_ENDIAN",12345678)
True
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_LINK",12345678)
True
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_MULTI_SZ",12345678)
True
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_RESOURCE_LIST",12345678)
True
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_FULL_RESOURCE_DESCRIPTOR",12345678)
True
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_RESOURCE_REQUIREMENTS_LIST",12345678)
True
$> sys.write_reg_int("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_QWORD",12345678)
True
```

### sys.write_reg_str

`sys.write_reg_str(reghive: str, regpath: str, regname: str, regtype: str, regvalue: str) -> Bool`

The **sys.write_reg_str** method returns `True` if registry values are written to the requested registry path and accepts a string as the value argument.
An example is below:

```python
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_SZ","BAR1")
True
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_BINARY","DEADBEEF")
True
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_NONE","DEADBEEF")
True
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_EXPAND_SZ","BAR2")
True
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_DWORD","12345678")
True
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_DWORD_BIG_ENDIAN","12345678")
True
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_LINK","A PLAIN STRING")
True
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_MULTI_SZ","BAR1,BAR2,BAR3")
True
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_RESOURCE_LIST","DEADBEEF")
True
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_FULL_RESOURCE_DESCRIPTOR","DEADBEEF")
True
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_RESOURCE_REQUIREMENTS_LIST","DEADBEEF")
True
$> sys.write_reg_str("HKEY_CURRENT_USER","SOFTWARE\\TEST1","FOO1","REG_QWORD","1234567812345678")
True
```

## Time

The `time` library contains general functions for obtaining and formatting the current system time. It allows you to convert between timestamps and readable strings, or introduce execution delays using sleep.

### time.format_to_epoch

`time.format_to_epoch(input: str, format: str) -> int`

The **time.format_to_epoch** method returns the seconds since epoch for the given UTC timestamp of the provided format. Input must include date and time components.

Some common formatting methods are:

- "%Y-%m-%d %H:%M:%S" (24 Hour Time)
- "%Y-%m-%d %I:%M:%S %P" (AM/PM)

For reference on all available format specifiers, see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>

### time.format_to_readable

`time.format_to_readable(input: int, format: str) -> str`

The **time.format_to_readable** method returns the timestamp in the provided format of the provided UTC timestamp.

Some common formatting methods are:

- "%Y-%m-%d %H:%M:%S" (24 Hour Time)
- "%Y-%m-%d %I:%M:%S %P" (AM/PM)

For reference on all available format specifiers, see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>

### time.now

`time.now() -> int`

The **time.now** method returns the time since UNIX EPOCH (Jan 01 1970). This uses the local system time.

### time.sleep

`time.sleep(secs: int)`

The **time.sleep** method sleeps the task for the given number of seconds.
