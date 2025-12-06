---
title: Eldritch V2 Language Guide
permalink: /user-guide/eldritchv2
---

Eldritch V2 is a Starlark-like domain specific language used for scripting implant behaviors. It is designed to be familiar to Python users while remaining simple, safe, and easily embeddable.

## Quick Start

You can try the language in the [interactive REPL demo](/assets/eldritch-repl/index.html).

### Hello World

```python
print("Hello, World!")
```

### Variables and Types

Eldritch V2 is dynamically typed.

```python
x = 10              # int
y = 3.14            # float
name = "Eldritch"   # string
is_active = True    # bool
data = b"\x00\x01"  # bytes
items = [1, 2, 3]   # list
config = {"a": 1}   # dict
point = (10, 20)    # tuple
unique = {1, 2, 3}  # set
```

## Language Reference

### Control Flow

**If Statements**

```python
if x > 10:
    print("Big")
elif x == 10:
    print("Ten")
else:
    print("Small")
```

**Loops**

```python
# For loop
for i in range(5):
    print(i)

# While loop
while x > 0:
    x -= 1
    if x == 5:
        break
```

**Ternary Operator**

```python
status = "Active" if is_running else "Inactive"
```

### Functions

Functions are defined using `def`. They support positional arguments, keyword arguments, default values, `*args`, and `**kwargs`.

```python
def greet(name, greeting="Hello"):
    return "%s, %s!" % (greeting, name)

print(greet("World"))
```

### Modules

Eldritch V2 supports loading modules. Standard library modules (like `file`, `sys`) are available globally or can be imported if configured. In the standard environment, they are pre-loaded global objects.

```python
import file
# or just use the global object directly if available
file.list(".")
```

## Built-in Functions

Eldritch V2 provides a rich set of built-in functions available in the global scope.

### Core

*   **`print(*args)`**: Prints objects to the standard output.
*   **`pprint(object)`**: Pretty-prints an object.
*   **`len(s)`**: Returns the length of an object (string, list, dict, etc.).
*   **`type(object)`**: Returns the type of the object.
*   **`dir([object])`**: Returns a list of valid attributes for the object.
*   **`libs()`**: Lists all registered libraries.
*   **`builtins()`**: Lists all built-in functions.
*   **`fail(message)`**: Aborts execution with an error message.
*   **`assert(condition)`**: Aborts if the condition is false.
*   **`assert_eq(a, b)`**: Aborts if `a` is not equal to `b`.

### Type Constructors & Conversion

*   **`bool(x)`**: Converts a value to a Boolean.
*   **`int(x)`**: Converts a number or string to an integer.
*   **`float(x)`**: Converts a number or string to a floating point number.
*   **`str(object)`**: Returns a string containing a nicely printable representation of an object.
*   **`bytes(source)`**: Creates a bytes object.
*   **`list([iterable])`**: Creates a list.
*   **`dict(**kwargs)`** or **`dict(iterable)`**: Creates a dictionary.
*   **`set([iterable])`**: Creates a set.
*   **`tuple([iterable])`**: Creates a tuple.

### Math & Logic

*   **`abs(x)`**: Returns the absolute value of a number.
*   **`max(iterable)`**: Returns the largest item in an iterable or the largest of two or more arguments.
*   **`min(iterable)`**: Returns the smallest item in an iterable.
*   **`range(start, stop[, step])`**: Returns a sequence of numbers.

### Iteration

*   **`all(iterable)`**: Returns True if all elements of the iterable are true.
*   **`any(iterable)`**: Returns True if any element of the iterable is true.
*   **`enumerate(iterable)`**: Returns an enumerate object yielding pairs of (index, value).
*   **`reversed(seq)`**: Returns a reverse iterator.
*   **`sorted(iterable)`**: Returns a new sorted list from the items in iterable.
*   **`zip(*iterables)`**: Returns an iterator of tuples, where the i-th tuple contains the i-th element from each of the argument sequences.

---

## Standard Library

The standard library provides powerful capabilities for interacting with the host system.

### Agent

The `agent` module interacts with the running implant itself.

*   **`agent.get_config() -> Dict`**
    Returns the current agent configuration.

*   **`agent.get_id() -> String`**
    Returns the unique identifier of the agent.

*   **`agent.get_platform() -> String`**
    Returns the platform the agent is running on (e.g., "linux", "windows").

*   **`agent.kill()`**
    Terminates the agent process.

*   **`agent.set_config(config: Dict)`**
    Updates the agent configuration.

*   **`agent.sleep(secs: int)`**
    Sleeps for the specified number of seconds.

### Assets

The `assets` module provides access to embedded files within the agent.

*   **`assets.get(name: String) -> Bytes`**
    Returns the content of the named asset as bytes.

*   **`assets.list() -> List<String>`**
    Returns a list of available asset names.

### Crypto

The `crypto` module provides basic cryptographic primitives.

*   **`crypto.aes_decrypt(key: Bytes, iv: Bytes, data: Bytes) -> Bytes`**
    Decrypts data using AES-CBC. Key must be 16, 24, or 32 bytes.

*   **`crypto.aes_encrypt(key: Bytes, iv: Bytes, data: Bytes) -> Bytes`**
    Encrypts data using AES-CBC.

*   **`crypto.md5(data: Bytes) -> String`**
    Returns the MD5 hash of the data as a hex string.

*   **`crypto.sha1(data: Bytes) -> String`**
    Returns the SHA1 hash of the data as a hex string.

*   **`crypto.sha256(data: Bytes) -> String`**
    Returns the SHA256 hash of the data as a hex string.

### File

The `file` module allows interaction with the filesystem.

*   **`file.append(path: String, content: String)`**
    Appends content to a file.

*   **`file.compress(src: String, dst: String)`**
    Compresses a file or directory to a gzip archive.

*   **`file.copy(src: String, dst: String)`**
    Copies a file from source to destination.

*   **`file.decompress(src: String, dst: String)`**
    Decompresses a gzip archive.

*   **`file.exists(path: String) -> Bool`**
    Checks if a path exists.

*   **`file.follow(path: String, callback: Function)`**
    Follows a file (tail) and calls the callback function for each new line.

*   **`file.is_dir(path: String) -> Bool`**
    Checks if path is a directory.

*   **`file.is_file(path: String) -> Bool`**
    Checks if path is a file.

*   **`file.list(path: String) -> List<Dict>`**
    Lists files in a directory. Returns a list of dictionaries containing file metadata.

*   **`file.mkdir(path: String, parent: Option<Bool>)`**
    Creates a directory. Set `parent` to True to create parent directories as needed.

*   **`file.move(src: String, dst: String)`**
    Moves or renames a file.

*   **`file.parent_dir(path: String) -> String`**
    Returns the parent directory of the path.

*   **`file.read(path: String) -> String`**
    Reads a file as a UTF-8 string.

*   **`file.read_binary(path: String) -> Bytes`**
    Reads a file as bytes.

*   **`file.remove(path: String)`**
    Deletes a file or directory.

*   **`file.replace(path: String, pattern: String, value: String)`**
    Replaces the first occurrence of a regex pattern in a file.

*   **`file.replace_all(path: String, pattern: String, value: String)`**
    Replaces all occurrences of a regex pattern in a file.

*   **`file.temp_file(name: Option<String>) -> String`**
    Creates a temporary file and returns its path.

*   **`file.template(template_path: String, dst: String, args: Dict, autoescape: Bool)`**
    Renders a template file to a destination.

*   **`file.timestomp(src: String, dst: String)`**
    Copies timestamp metadata from source to destination.

*   **`file.write(path: String, content: String)`**
    Writes content to a file, overwriting it.

*   **`file.find(path: String, name: Option<String>, file_type: Option<String>, permissions: Option<Int>, modified_time: Option<Int>, create_time: Option<Int>) -> List<String>`**
    Finds files matching criteria. `file_type` can be "file" or "dir".

### HTTP

The `http` module performs HTTP requests.

*   **`http.download(url: String, path: String)`**
    Downloads a file from a URL to a local path.

*   **`http.get(url: String, headers: Option<Dict>) -> Dict`**
    Performs a GET request. Returns a dictionary with `status` (int), `headers` (dict), and `body` (bytes).

*   **`http.post(url: String, body: Option<Bytes>, headers: Option<Dict>) -> Dict`**
    Performs a POST request.

### Pivot

The `pivot` module manages network pivoting and connections.

*   **`pivot.list() -> List<Dict>`**
    Lists active pivots.

*   **`pivot.start_tcp(bind_addr: String) -> String`**
    Starts a TCP listener for pivoting.

*   **`pivot.stop(id: String)`**
    Stops a pivot by ID.

*   **`pivot.arp_scan(target_cidrs: List<String>) -> List<Dict>`**
    Scans for hosts using ARP.

*   **`pivot.bind_proxy(listen_address: String, listen_port: Int, username: String, password: String)`**
    Starts a SOCKS5 proxy server.

*   **`pivot.ncat(address: String, port: Int, data: String, protocol: String) -> String`**
    Sends data to a host via TCP/UDP.

*   **`pivot.port_forward(listen_address: String, listen_port: Int, forward_address: String, forward_port: Int, protocol: String)`**
    Forwards traffic from a local port to a remote port.

*   **`pivot.port_scan(target_cidrs: List<String>, ports: List<Int>, protocol: String, timeout: Int) -> List<Dict>`**
    Scans TCP/UDP ports.

*   **`pivot.reverse_shell_pty(cmd: Option<String>)`**
    Spawns a reverse shell in a PTY.

*   **`pivot.smb_exec(target: String, port: Int, username: String, password: String, hash: String, command: String) -> String`**
    Executes a command over SMB.

*   **`pivot.ssh_copy(target: String, port: Int, src: String, dst: String, username: String, password: Option<String>, key: Option<String>, key_password: Option<String>, timeout: Option<Int>) -> String`**
    Copies a file via SSH.

*   **`pivot.ssh_exec(target: String, port: Int, command: String, username: String, password: Option<String>, key: Option<String>, key_password: Option<String>, timeout: Option<Int>) -> List<Dict>`**
    Executes a command via SSH.

### Process

The `process` module interacts with system processes.

*   **`process.info(pid: Option<Int>) -> Dict`**
    Returns information about a process.

*   **`process.kill(pid: Int)`**
    Kills a process.

*   **`process.list() -> List<Dict>`**
    Lists running processes.

*   **`process.name(pid: Int) -> String`**
    Returns the name of a process.

*   **`process.netstat() -> List<Dict>`**
    Returns network connection information.

### Random

The `random` module generates random data.

*   **`random.bool() -> Bool`**
    Returns a random boolean.

*   **`random.bytes(len: Int) -> Bytes`**
    Returns random bytes of specified length.

*   **`random.int(min: Int, max: Int) -> Int`**
    Returns a random integer in the range [min, max).

*   **`random.string(len: Int, charset: Option<String>) -> String`**
    Returns a random string.

*   **`random.uuid() -> String`**
    Returns a random UUIDv4.

### Regex

The `regex` module provides regular expression matching.

*   **`regex.match(haystack: String, pattern: String) -> String`**
    Returns the first match of the pattern in the string.

*   **`regex.match_all(haystack: String, pattern: String) -> List<String>`**
    Returns all matches of the pattern.

*   **`regex.replace(haystack: String, pattern: String, value: String) -> String`**
    Replaces the first match with value.

*   **`regex.replace_all(haystack: String, pattern: String, value: String) -> String`**
    Replaces all matches with value.

### Report

The `report` module sends structured data back to the C2 server.

*   **`report.file(path: String)`**
    Exfiltrates a file.

*   **`report.process_list(list: List<Dict>)`**
    Reports a process list snapshot.

*   **`report.ssh_key(username: String, key: String)`**
    Reports a captured SSH key.

*   **`report.user_password(username: String, password: String)`**
    Reports captured credentials.

### Sys

The `sys` module provides system information and low-level interaction.

*   **`sys.dll_inject(dll_path: String, pid: Int)`**
    Injects a DLL from disk into a process.

*   **`sys.dll_reflect(dll_bytes: Bytes, pid: Int, function_name: String)`**
    Reflectively loads a DLL from memory into a process.

*   **`sys.exec(path: String, args: List<String>, disown: Option<Bool>, env_vars: Option<Dict>) -> Dict`**
    Executes a process.

*   **`sys.get_env() -> Dict`**
    Returns environment variables.

*   **`sys.get_ip() -> List<Dict>`**
    Returns network interface information.

*   **`sys.get_os() -> Dict`**
    Returns OS information.

*   **`sys.get_pid() -> Int`**
    Returns the current process ID.

*   **`sys.get_reg(reghive: String, regpath: String) -> Dict`**
    Reads registry keys.

*   **`sys.get_user() -> Dict`**
    Returns current user information.

*   **`sys.hostname() -> String`**
    Returns the hostname.

*   **`sys.is_bsd() -> Bool`**
    Returns True if running on BSD.

*   **`sys.is_linux() -> Bool`**
    Returns True if running on Linux.

*   **`sys.is_macos() -> Bool`**
    Returns True if running on macOS.

*   **`sys.is_windows() -> Bool`**
    Returns True if running on Windows.

*   **`sys.shell(cmd: String) -> Dict`**
    Executes a shell command.

*   **`sys.write_reg_hex(reghive: String, regpath: String, regname: String, regtype: String, regvalue: String) -> Bool`**
    Writes a HEX registry value.

*   **`sys.write_reg_int(reghive: String, regpath: String, regname: String, regtype: String, regvalue: Int) -> Bool`**
    Writes an integer registry value.

*   **`sys.write_reg_str(reghive: String, regpath: String, regname: String, regtype: String, regvalue: String) -> Bool`**
    Writes a string registry value.

### Time

The `time` module handles time operations.

*   **`time.format_to_epoch(input: String, format: String) -> Int`**
    Parses a time string to epoch seconds.

*   **`time.format_to_readable(input: Int, format: String) -> String`**
    Formats epoch seconds to a string.

*   **`time.now() -> Int`**
    Returns current epoch seconds.

*   **`time.sleep(secs: Int)`**
    Sleeps for a duration.
