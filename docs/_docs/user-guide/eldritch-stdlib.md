---
title: Eldritch Standard Library
tags:
 - User Guide
description: Eldritch Standard Library Documentation
permalink: user-guide/eldritch-stdlib
---

# Standard Library

The following libraries are available in Eldritch.

## Agent Library
The `agent` library provides capabilities for interacting with the agent's internal state, configuration, and task management.

It allows you to:
- Modify agent configuration (callback intervals, transports).
- Manage background tasks.
- Control agent execution (termination).

### agent._terminate_this_process_clowntown
`agent._terminate_this_process_clowntown() -> None`
**DANGER**: Terminates the agent process immediately.

This method calls `std::process::exit(0)`, effectively killing the agent.
Use with extreme caution.

**Returns**
- `None` (Does not return as the process exits).

**Errors**
- This function is unlikely to return an error, as it terminates the process.

### agent.get_callback_interval
`agent.get_callback_interval() -> int`
Returns the current callback interval in seconds.

**Returns**
- `int`: The interval in seconds.

**Errors**
- Returns an error string if the interval cannot be retrieved.

### agent.get_config
`agent.get_config() -> Dict`
Returns the current configuration of the agent as a dictionary.

**Returns**
- `Dict<String, Value>`: A dictionary containing configuration keys and values.

**Errors**
- Returns an error string if the configuration cannot be retrieved or is not implemented.

### agent.get_transport
`agent.get_transport() -> str`
Returns the name of the currently active transport.

**Returns**
- `str`: The name of the transport (e.g., "http", "grpc").

**Errors**
- Returns an error string if the transport cannot be identified.

### agent.list_tasks
`agent.list_tasks() -> List<TaskWrapper>`
Lists the currently running or queued background tasks on the agent.

**Returns**
- `List<Task>`: A list of task objects.

**Errors**
- Returns an error string if the task list cannot be retrieved.

### agent.list_transports
`agent.list_transports() -> List<str>`
Returns a list of available transport names.

**Returns**
- `List<str>`: A list of transport names.

**Errors**
- Returns an error string if the list cannot be retrieved.

### agent.set_active_callback_uri
`agent.set_active_callback_uri(uri: str) -> None`
Sets the active callback URI for the agent.

**Parameters**
- `uri` (`str`): The new URI to callback to

**Returns**
- `None`

**Errors**
- Returns an error string if the active callback uri cannot be set.

### agent.set_callback_interval
`agent.set_callback_interval(interval: int) -> None`
Sets the callback interval for the agent.

This configuration change is typically transient and may not persist across reboots.

**Parameters**
- `interval` (`int`): The new interval in seconds.

**Returns**
- `None`

**Errors**
- Returns an error string if the interval cannot be set.

### agent.stop_task
`agent.stop_task(task_id: int) -> None`
Stops a specific background task by its ID.

**Parameters**
- `task_id` (`int`): The ID of the task to stop.

**Returns**
- `None`

**Errors**
- Returns an error string if the task cannot be stopped or does not exist.

## Assets Library
The `assets` library provides access to files embedded directly within the agent binary or fetched remotely.

### assets.copy
`assets.copy(src: str, dest: str) -> None`
Copies an embedded asset to a destination path on the disk.

**Parameters**
- `src` (`str`): The name/path of the source asset.
- `dest` (`str`): The destination file path on the local system.

**Returns**
- `None`

**Errors**
- Returns an error string if the asset does not exist or the file cannot be written (e.g., permission denied).

### assets.fetch
`assets.fetch(name: str) -> Bytes`
Fetches a remote asset from the C2 server.

**Parameters**
- `name` (`str`): The name of the asset to fetch.

**Returns**
- `List<int>`: The asset content as a list of bytes.

**Errors**
- Returns an error string if the fetch fails.

### assets.list
`assets.list() -> List<str>`
Returns a list of all available asset names.

**Returns**
- `List<str>`: A list of asset names available in the agent.

**Errors**
- Returns an error string if the asset list cannot be retrieved.

### assets.read
`assets.read(name: str) -> str`
Reads the content of an embedded asset as a UTF-8 string.

**Parameters**
- `name` (`str`): The name/path of the asset to read.

**Returns**
- `str`: The asset content as a string.

**Errors**
- Returns an error string if the asset does not exist or contains invalid UTF-8 data.

### assets.read_binary
`assets.read_binary(name: str) -> Bytes`
Reads the content of an embedded asset as a list of bytes.

**Parameters**
- `name` (`str`): The name/path of the asset to read.

**Returns**
- `List<int>`: The asset content as a list of bytes (u8).

**Errors**
- Returns an error string if the asset does not exist.

## Crypto Library
The `crypto` library provides cryptographic primitives, hashing, encoding, and JSON handling utilities.

It supports:
- AES encryption and decryption.
- Hashing (MD5, SHA1, SHA256) for data and files.
- Base64 encoding and decoding.
- JSON serialization and deserialization.

### crypto.aes_decrypt
`crypto.aes_decrypt(key: Bytes, iv: Bytes, data: Bytes) -> Bytes`
Decrypts data using AES (CBC mode).

**Parameters**
- `key` (`Bytes`): The decryption key (must be 16, 24, or 32 bytes).
- `iv` (`Bytes`): The initialization vector (must be 16 bytes).
- `data` (`Bytes`): The encrypted data to decrypt.

**Returns**
- `Bytes`: The decrypted data.

**Errors**
- Returns an error string if decryption fails (e.g., invalid padding, incorrect key length).

### crypto.aes_decrypt_file
`crypto.aes_decrypt_file(src: str, dst: str, key: str) -> None`
Decrypts a file using AES.

**Parameters**
- `src` (`str`): The source file path.
- `dst` (`str`): The destination file path.
- `key` (`str`): The decryption key.

**Returns**
- `None`

**Errors**
- Returns an error string if decryption fails or file operations fail.

### crypto.aes_encrypt
`crypto.aes_encrypt(key: Bytes, iv: Bytes, data: Bytes) -> Bytes`
Encrypts data using AES (CBC mode).

**Parameters**
- `key` (`Bytes`): The encryption key (must be 16, 24, or 32 bytes).
- `iv` (`Bytes`): The initialization vector (must be 16 bytes).
- `data` (`Bytes`): The data to encrypt.

**Returns**
- `Bytes`: The encrypted data.

**Errors**
- Returns an error string if encryption fails (e.g., incorrect key length).

### crypto.aes_encrypt_file
`crypto.aes_encrypt_file(src: str, dst: str, key: str) -> None`
Encrypts a file using AES.

**Parameters**
- `src` (`str`): The source file path.
- `dst` (`str`): The destination file path.
- `key` (`str`): The encryption key.

**Returns**
- `None`

**Errors**
- Returns an error string if encryption fails or file operations fail.

### crypto.decode_b64
`crypto.decode_b64(content: str, encode_type: Option<str>) -> str`
Decodes a Base64 encoded string.

**Parameters**
- `content` (`str`): The Base64 string to decode.
- `encode_type` (`Option<str>`): The decoding variant (matches encoding options).
- "STANDARD" (default)
- "STANDARD_NO_PAD"
- "URL_SAFE"
- "URL_SAFE_NO_PAD"

**Returns**
- `str`: The decoded string.

**Errors**
- Returns an error string if decoding fails or the variant is invalid.

### crypto.encode_b64
`crypto.encode_b64(content: str, encode_type: Option<str>) -> str`
Encodes a string to Base64.

**Parameters**
- `content` (`str`): The string content to encode.
- `encode_type` (`Option<str>`): The encoding variant. Valid options:
- "STANDARD" (default)
- "STANDARD_NO_PAD"
- "URL_SAFE"
- "URL_SAFE_NO_PAD"

**Returns**
- `str`: The Base64 encoded string.

**Errors**
- Returns an error string if the encoding type is invalid.

### crypto.hash_file
`crypto.hash_file(file: str, algo: str) -> str`
Calculates the hash of a file on disk.

**Parameters**
- `file` (`str`): The path to the file.
- `algo` (`str`): The hashing algorithm to use ("MD5", "SHA1", "SHA256", "SHA512").

**Returns**
- `str`: The hexadecimal representation of the hash.

**Errors**
- Returns an error string if the file cannot be read or the algorithm is not supported.

### crypto.is_json
`crypto.is_json(content: str) -> bool`
Checks if a string is valid JSON.

**Parameters**
- `content` (`str`): The string to check.

**Returns**
- `bool`: `True` if valid JSON, `False` otherwise.

### crypto.md5
`crypto.md5(data: Bytes) -> str`
Calculates the MD5 hash of the provided data.

**Parameters**
- `data` (`Bytes`): The input data.

**Returns**
- `str`: The hexadecimal representation of the hash.

**Errors**
- Returns an error string if hashing fails.

### crypto.sha1
`crypto.sha1(data: Bytes) -> str`
Calculates the SHA1 hash of the provided data.

**Parameters**
- `data` (`Bytes`): The input data.

**Returns**
- `str`: The hexadecimal representation of the hash.

**Errors**
- Returns an error string if hashing fails.

### crypto.sha256
`crypto.sha256(data: Bytes) -> str`
Calculates the SHA256 hash of the provided data.

**Parameters**
- `data` (`Bytes`): The input data.

**Returns**
- `str`: The hexadecimal representation of the hash.

**Errors**
- Returns an error string if hashing fails.

### crypto.to_json
`crypto.to_json(content: Value) -> str`
Serializes an Eldritch value into a JSON string.

**Parameters**
- `content` (`Value`): The value to serialize.

**Returns**
- `str`: The JSON string representation.

**Errors**
- Returns an error string if serialization fails (e.g., circular references, unsupported types).

## File Library
The `file` library provides comprehensive filesystem operations.

It supports:
- reading and writing files (text and binary).
- file manipulation (copy, move, remove).
- directory operations (mkdir, list).
- compression and decompression (gzip).
- content searching and replacement.

### file.append
`file.append(path: str, content: str) -> None`
Appends content to a file.

If the file does not exist, it will be created.

**Parameters**
- `path` (`str`): The path to the file.
- `content` (`str`): The string content to append.

**Returns**
- `None`

**Errors**
- Returns an error string if the file cannot be opened or written to.

### file.compress
`file.compress(src: str, dst: str) -> None`
Compresses a file or directory using GZIP.

If `src` is a directory, it will be archived (tar) before compression.

**Parameters**
- `src` (`str`): The source file or directory path.
- `dst` (`str`): The destination path for the compressed file (e.g., `archive.tar.gz`).

**Returns**
- `None`

**Errors**
- Returns an error string if the source doesn't exist or compression fails.

### file.copy
`file.copy(src: str, dst: str) -> None`
Copies a file from source to destination.

If the destination exists, it will be overwritten.

**Parameters**
- `src` (`str`): The source file path.
- `dst` (`str`): The destination file path.

**Returns**
- `None`

**Errors**
- Returns an error string if the source doesn't exist or copy fails.

### file.decompress
`file.decompress(src: str, dst: str) -> None`
Decompresses a GZIP file.

If the file is a tar archive, it will be extracted to the destination directory.

**Parameters**
- `src` (`str`): The source compressed file path.
- `dst` (`str`): The destination path (file or directory).

**Returns**
- `None`

**Errors**
- Returns an error string if decompression fails.

### file.exists
`file.exists(path: str) -> bool`
Checks if a file or directory exists at the given path.

**Parameters**
- `path` (`str`): The path to check.

**Returns**
- `bool`: `True` if it exists, `False` otherwise.

### file.find
`file.find(path: str, name: Option<str>, file_type: Option<str>, permissions: Option<int>, modified_time: Option<int>, create_time: Option<int>) -> List<str>`
Finds files matching specific criteria.

**Parameters**
- `path` (`str`): The base directory to start searching from.
- `name` (`Option<str>`): Filter by filename (substring match).
- `file_type` (`Option<str>`): Filter by type ("file" or "dir").
- `permissions` (`Option<int>`): Filter by permissions (Unix octal e.g., 777, Windows readonly check).
- `modified_time` (`Option<int>`): Filter by modification time (epoch seconds).
- `create_time` (`Option<int>`): Filter by creation time (epoch seconds).

**Returns**
- `List<str>`: A list of matching file paths.

**Errors**
- Returns an error string if the search encounters issues.

### file.follow
`file.follow(path: str, fn_val: Value) -> None`
Follows a file (tail -f) and executes a callback function for each new line.

This is useful for monitoring logs.

**Parameters**
- `path` (`str`): The file path to follow.
- `fn` (`function(str)`): A callback function that takes a string (the new line) as an argument.

**Returns**
- `None` (This function may block indefinitely or until interrupted).

**Errors**
- Returns an error string if the file cannot be opened.

### file.is_dir
`file.is_dir(path: str) -> bool`
Checks if the path exists and is a directory.

**Parameters**
- `path` (`str`): The path to check.

**Returns**
- `bool`: `True` if it is a directory, `False` otherwise.

### file.is_file
`file.is_file(path: str) -> bool`
Checks if the path exists and is a file.

**Parameters**
- `path` (`str`): The path to check.

**Returns**
- `bool`: `True` if it is a file, `False` otherwise.

### file.list
`file.list(path: Option<str>) -> List<Dict>`
Lists files and directories in the specified path.

Supports globbing patterns (e.g., `/home/*/*.txt`).

**Parameters**
- `path` (`Option<str>`): The directory path or glob pattern. Defaults to current working directory.

**Returns**
- `List<Dict>`: A list of dictionaries containing file details:
- `file_name` (`str`)
- `absolute_path` (`str`)
- `size` (`int`)
- `owner` (`str`)
- `group` (`str`)
- `permissions` (`str`)
- `modified` (`str`)
- `type` (`str`: "File" or "Directory")

**Errors**
- Returns an error string if listing fails.

### file.mkdir
`file.mkdir(path: str, parent: Option<bool>) -> None`
Creates a new directory.

**Parameters**
- `path` (`str`): The directory path to create.
- `parent` (`Option<bool>`): If `True`, creates parent directories as needed (like `mkdir -p`). Defaults to `False`.

**Returns**
- `None`

**Errors**
- Returns an error string if creation fails.

### file.move
`file.move(src: str, dst: str) -> None`
Moves or renames a file or directory.

**Parameters**
- `src` (`str`): The source path.
- `dst` (`str`): The destination path.

**Returns**
- `None`

**Errors**
- Returns an error string if the move fails.

### file.parent_dir
`file.parent_dir(path: str) -> str`
Returns the parent directory of the given path.

**Parameters**
- `path` (`str`): The file or directory path.

**Returns**
- `str`: The parent directory path.

**Errors**
- Returns an error string if the path is invalid or has no parent.

### file.read
`file.read(path: str) -> str`
Reads the entire content of a file as a string.

Supports globbing; if multiple files match, reads the first one (or behavior may vary, usually reads specific file).
*Note*: v1 docs say it errors if a directory matches.

**Parameters**
- `path` (`str`): The file path.

**Returns**
- `str`: The file content.

**Errors**
- Returns an error string if the file cannot be read or contains invalid UTF-8.

### file.read_binary
`file.read_binary(path: str) -> Bytes`
Reads the entire content of a file as binary data.

**Parameters**
- `path` (`str`): The file path.

**Returns**
- `List<int>`: The file content as a list of bytes (u8).

**Errors**
- Returns an error string if the file cannot be read.

### file.remove
`file.remove(path: str) -> None`
Deletes a file or directory recursively.

**Parameters**
- `path` (`str`): The path to remove.

**Returns**
- `None`

**Errors**
- Returns an error string if removal fails.

### file.replace
`file.replace(path: str, pattern: str, value: str) -> None`
Replaces the first occurrence of a regex pattern in a file with a replacement string.

**Parameters**
- `path` (`str`): The file path.
- `pattern` (`str`): The regex pattern to match.
- `value` (`str`): The replacement string.

**Returns**
- `None`

**Errors**
- Returns an error string if the file cannot be modified or the regex is invalid.

### file.replace_all
`file.replace_all(path: str, pattern: str, value: str) -> None`
Replaces all occurrences of a regex pattern in a file with a replacement string.

**Parameters**
- `path` (`str`): The file path.
- `pattern` (`str`): The regex pattern to match.
- `value` (`str`): The replacement string.

**Returns**
- `None`

**Errors**
- Returns an error string if the file cannot be modified or the regex is invalid.

### file.temp_file
`file.temp_file(name: Option<str>) -> str`
Creates a temporary file and returns its path.

**Parameters**
- `name` (`Option<str>`): Optional preferred filename. If None, a random name is generated.

**Returns**
- `str`: The absolute path to the temporary file.

**Errors**
- Returns an error string if creation fails.

### file.template
`file.template(template_path: str, dst: str, args: Dict, autoescape: bool) -> None`
Renders a Jinja2 template file to a destination path.

**Parameters**
- `template_path` (`str`): Path to the source template file.
- `dst` (`str`): Destination path for the rendered file.
- `args` (`Dict<str, Value>`): Variables to substitute in the template.
- `autoescape` (`bool`): Whether to enable HTML auto-escaping (OWASP recommendations).

**Returns**
- `None`

**Errors**
- Returns an error string if the template cannot be read, parsed, or written.

### file.timestomp
`file.timestomp(path: str, mtime: Option<Value>, atime: Option<Value>, ctime: Option<Value>, ref_file: Option<str>) -> None`
Timestomps a file.

Modifies the timestamps (modified, access, creation) of a file.
Can use a reference file or specific values.

**Parameters**
- `path` (`str`): The target file to modify.
- `mtime` (`Option<Value>`): New modification time (Int epoch or String).
- `atime` (`Option<Value>`): New access time (Int epoch or String).
- `ctime` (`Option<Value>`): New creation time (Int epoch or String). Windows only.
- `ref_file` (`Option<str>`): Path to a reference file to copy timestamps from.

**Returns**
- `None`

**Errors**
- Returns an error string if the operation fails or input is invalid.

### file.write
`file.write(path: str, content: str) -> None`
Writes content to a file, overwriting it if it exists.

**Parameters**
- `path` (`str`): The file path.
- `content` (`str`): The string content to write.

**Returns**
- `None`

**Errors**
- Returns an error string if writing fails.

## Http Library
The `http` library enables the agent to make HTTP requests.

It supports:
- GET and POST requests.
- File downloading.
- Custom headers.

**Note**: TLS validation behavior depends on the underlying agent configuration and may not be exposed per-request in this version of the library (unlike v1 which had `allow_insecure` arg).

### http.download
`http.download(url: str, path: str) -> None`
Downloads a file from a URL to a local path.

**Parameters**
- `url` (`str`): The URL to download from.
- `path` (`str`): The local destination path.

**Returns**
- `None`

**Errors**
- Returns an error string if the download fails.

### http.get
`http.get(url: str, headers: Option<Dict<str, str>>) -> Dict`
Performs an HTTP GET request.

**Parameters**
- `url` (`str`): The target URL.
- `headers` (`Option<Dict<str, str>>`): Optional custom HTTP headers.

**Returns**
- `Dict`: A dictionary containing the response:
- `status_code` (`int`): HTTP status code.
- `body` (`Bytes`): The response body.
- `headers` (`Dict<str, str>`): Response headers.

**Errors**
- Returns an error string if the request fails.

### http.post
`http.post(url: str, body: Option<Bytes>, headers: Option<Dict<str, str>>) -> Dict`
Performs an HTTP POST request.

**Parameters**
- `url` (`str`): The target URL.
- `body` (`Option<Bytes>`): The request body.
- `headers` (`Option<Dict<str, str>>`): Optional custom HTTP headers.

**Returns**
- `Dict`: A dictionary containing the response:
- `status_code` (`int`): HTTP status code.
- `body` (`Bytes`): The response body.
- `headers` (`Dict<str, str>`): Response headers.

**Errors**
- Returns an error string if the request fails.

## Pivot Library
The `pivot` library provides tools for lateral movement, scanning, and tunneling.

### pivot.arp_scan
`pivot.arp_scan(target_cidrs: List<str>) -> List<Dict>`
Performs an ARP scan to discover live hosts on the local network.

### pivot.ncat
`pivot.ncat(address: str, port: int, data: str, protocol: str) -> str`
Sends arbitrary data to a host via TCP or UDP and waits for a response.

### pivot.port_scan
`pivot.port_scan(target_cidrs: List<str>, ports: List<int>, protocol: str, timeout: int, fd_limit: Option<int>) -> List<Dict>`
Scans TCP/UDP ports on target hosts.

### pivot.reverse_shell_pty
`pivot.reverse_shell_pty(cmd: Option<str>) -> None`
Spawns a reverse shell with a PTY (Pseudo-Terminal) attached.

### pivot.reverse_shell_repl
`pivot.reverse_shell_repl(interp: &mut Interpreter) -> None`
Spawns a basic REPL-style reverse shell.

### pivot.ssh_copy
`pivot.ssh_copy(target: str, port: int, src: str, dst: str, username: str, password: Option<str>, key: Option<str>, key_password: Option<str>, timeout: Option<int>) -> str`
Copies a file to a remote host via SSH (SCP/SFTP).

### pivot.ssh_exec
`pivot.ssh_exec(target: str, port: int, command: str, username: str, password: Option<str>, key: Option<str>, key_password: Option<str>, timeout: Option<int>) -> Dict`
Executes a command on a remote host via SSH.

## Process Library
The `process` library allows interaction with system processes.

It supports:
- Listing running processes.
- Retrieving process details (info, name).
- Killing processes.
- Inspecting network connections (netstat).

### process.info
`process.info(pid: Option<int>) -> Dict`
Returns detailed information about a specific process.

**Parameters**
- `pid` (`Option<int>`): The process ID to query. If `None`, returns info for the current agent process.

**Returns**
- `Dict`: Dictionary with process details (pid, name, cmd, exe, environ, cwd, memory_usage, user, etc.).

**Errors**
- Returns an error string if the process is not found or cannot be accessed.

### process.kill
`process.kill(pid: int) -> None`
Terminates a process by its ID.

**Parameters**
- `pid` (`int`): The process ID to kill.

**Returns**
- `None`

**Errors**
- Returns an error string if the process cannot be killed (e.g., permission denied).

### process.list
`process.list() -> List<Dict>`
Lists all currently running processes.

**Returns**
- `List<Dict>`: A list of process dictionaries containing `pid`, `ppid`, `name`, `path`, `username`, `command`, `cwd`, etc.

**Errors**
- Returns an error string if the process list cannot be retrieved.

### process.name
`process.name(pid: int) -> str`
Returns the name of a process given its ID.

**Parameters**
- `pid` (`int`): The process ID.

**Returns**
- `str`: The process name.

**Errors**
- Returns an error string if the process is not found.

### process.netstat
`process.netstat() -> List<Dict>`
Returns a list of active network connections (TCP/UDP/Unix).

**Returns**
- `List<Dict>`: A list of connection details including socket type, local/remote address/port, and associated PID.

**Errors**
- Returns an error string if network information cannot be retrieved.

## Random Library
The `random` library provides cryptographically secure random value generation.

### random.bool
`random.bool() -> bool`
Generates a random boolean value.

**Returns**
- `bool`: True or False.

### random.bytes
`random.bytes(len: int) -> Bytes`
Generates a list of random bytes.

**Parameters**
- `len` (`int`): Number of bytes to generate.

**Returns**
- `List<int>`: The random bytes.

### random.int
`random.int(min: int, max: int) -> int`
Generates a random integer within a range.

**Parameters**
- `min` (`int`): Minimum value (inclusive).
- `max` (`int`): Maximum value (exclusive).

**Returns**
- `int`: The random integer.

### random.string
`random.string(len: int, charset: Option<str>) -> str`
Generates a random string.

**Parameters**
- `len` (`int`): Length of the string.
- `charset` (`Option<str>`): Optional string of characters to use. If `None`, defaults to alphanumeric.

**Returns**
- `str`: The random string.

### random.uuid
`random.uuid() -> str`
Generates a random UUID (v4).

**Returns**
- `str`: The UUID string.

## Regex Library
The `regex` library provides regular expression capabilities using Rust's `regex` crate syntax.

**Note**: Currently, it primarily supports a single capture group. Multi-group support might be limited.

### regex.match
`regex.match(haystack: str, pattern: str) -> str`
Returns the first substring matching the pattern.

**Parameters**
- `haystack` (`str`): The string to search.
- `pattern` (`str`): The regex pattern.

**Returns**
- `str`: The matching string.

**Errors**
- Returns an error string if no match is found or the regex is invalid.

### regex.match_all
`regex.match_all(haystack: str, pattern: str) -> List<str>`
Returns all substrings matching the pattern in the haystack.

If the pattern contains capture groups, returns the captured string for each match.

**Parameters**
- `haystack` (`str`): The string to search.
- `pattern` (`str`): The regex pattern.

**Returns**
- `List<str>`: A list of matching strings.

**Errors**
- Returns an error string if the regex is invalid.

### regex.replace
`regex.replace(haystack: str, pattern: str, value: str) -> str`
Replaces the first occurrence of the pattern with the value.

**Parameters**
- `haystack` (`str`): The string to modify.
- `pattern` (`str`): The regex pattern to match.
- `value` (`str`): The replacement string.

**Returns**
- `str`: The modified string.

**Errors**
- Returns an error string if the regex is invalid.

### regex.replace_all
`regex.replace_all(haystack: str, pattern: str, value: str) -> str`
Replaces all occurrences of the pattern with the value.

**Parameters**
- `haystack` (`str`): The string to modify.
- `pattern` (`str`): The regex pattern to match.
- `value` (`str`): The replacement string.

**Returns**
- `str`: The modified string.

**Errors**
- Returns an error string if the regex is invalid.

## Report Library
The `report` library handles structured data reporting to the C2 server.

It allows you to:
- Exfiltrate files (in chunks).
- Report process snapshots.
- Report captured credentials (passwords, SSH keys).

### report.file
`report.file(path: str) -> None`
Reports (exfiltrates) a file from the host to the C2 server.

The file is sent asynchronously in chunks.

**Parameters**
- `path` (`str`): The path of the file to exfiltrate.

**Returns**
- `None`

**Errors**
- Returns an error string if the file cannot be read or queued for reporting.

### report.process_list
`report.process_list(list: List<Dict>) -> None`
Reports a snapshot of running processes.

This updates the process list view in the C2 UI.

**Parameters**
- `list` (`List<Dict>`): The list of process dictionaries (typically from `process.list()`).

**Returns**
- `None`

### report.ssh_key
`report.ssh_key(username: str, key: str) -> None`
Reports a captured SSH private key.

**Parameters**
- `username` (`str`): The associated username.
- `key` (`str`): The SSH key content.

**Returns**
- `None`

### report.user_password
`report.user_password(username: str, password: str) -> None`
Reports a captured user password.

**Parameters**
- `username` (`str`): The username.
- `password` (`str`): The password.

**Returns**
- `None`

## Sys Library
The `sys` library provides general system interaction capabilities.

It supports:
- Process execution (`exec`, `shell`).
- System information (`get_os`, `get_ip`, `get_user`, `hostname`).
- Registry operations (Windows).
- DLL injection and reflection.
- Environment variable access.

### sys.dll_inject
`sys.dll_inject(dll_path: str, pid: int) -> None`
Injects a DLL from disk into a remote process.

**Parameters**
- `dll_path` (`str`): Path to the DLL on disk.
- `pid` (`int`): Target process ID.

**Returns**
- `None`

**Errors**
- Returns an error string if injection fails.

### sys.dll_reflect
`sys.dll_reflect(dll_bytes: Bytes, pid: int, function_name: str) -> None`
Reflectively injects a DLL from memory into a remote process.

**Parameters**
- `dll_bytes` (`List<int>`): Content of the DLL.
- `pid` (`int`): Target process ID.
- `function_name` (`str`): Exported function to call.

**Returns**
- `None`

**Errors**
- Returns an error string if injection fails.

### sys.exec
`sys.exec(path: str, args: List<str>, disown: Option<bool>, env_vars: Option<Dict<str, str>>) -> Dict`
Executes a program directly (without a shell).

**Parameters**
- `path` (`str`): Path to the executable.
- `args` (`List<str>`): List of arguments.
- `disown` (`Option<bool>`): If `True`, runs in background/detached.
- `env_vars` (`Option<Dict<str, str>>`): Environment variables to set.

**Returns**
- `Dict`: Output containing `stdout`, `stderr`, and `status` (exit code).

### sys.get_env
`sys.get_env() -> Dict`
Returns the current process's environment variables.

**Returns**
- `Dict<str, str>`: Map of environment variables.

### sys.get_ip
`sys.get_ip() -> List<Dict>`
Returns network interface information.

**Returns**
- `List<Dict>`: List of interfaces with `name` and `ip`.

### sys.get_os
`sys.get_os() -> Dict`
Returns information about the operating system.

**Returns**
- `Dict`: Details like `arch`, `distro`, `platform`.

### sys.get_pid
`sys.get_pid() -> int`
Returns the current process ID.

**Returns**
- `int`: The PID.

### sys.get_reg
`sys.get_reg(reghive: str, regpath: str) -> Dict`
Reads values from the Windows Registry.

**Parameters**
- `reghive` (`str`): The registry hive (e.g., "HKEY_LOCAL_MACHINE").
- `regpath` (`str`): The registry path.

**Returns**
- `Dict<str, str>`: A dictionary of registry keys and values.

### sys.get_user
`sys.get_user() -> Dict`
Returns information about the current user.

**Returns**
- `Dict`: User details (uid, gid, name, groups).

### sys.hostname
`sys.hostname() -> str`
Returns the system hostname.

**Returns**
- `str`: The hostname.

### sys.is_bsd
`sys.is_bsd() -> bool`
Checks if the OS is BSD.

**Returns**
- `bool`: True if BSD.

### sys.is_linux
`sys.is_linux() -> bool`
Checks if the OS is Linux.

**Returns**
- `bool`: True if Linux.

### sys.is_macos
`sys.is_macos() -> bool`
Checks if the OS is macOS.

**Returns**
- `bool`: True if macOS.

### sys.is_windows
`sys.is_windows() -> bool`
Checks if the OS is Windows.

**Returns**
- `bool`: True if Windows.

### sys.shell
`sys.shell(cmd: str) -> Dict`
Executes a command via the system shell (`/bin/sh` or `cmd.exe`).

**Parameters**
- `cmd` (`str`): The command string to execute.

**Returns**
- `Dict`: Output containing `stdout`, `stderr`, and `status`.

### sys.write_reg_hex
`sys.write_reg_hex(reghive: str, regpath: str, regname: str, regtype: str, regvalue: str) -> bool`
Writes a hex value to the Windows Registry.

**Parameters**
- `reghive` (`str`)
- `regpath` (`str`)
- `regname` (`str`)
- `regtype` (`str`): e.g., "REG_BINARY".
- `regvalue` (`str`): Hex string.

**Returns**
- `bool`: True on success.

### sys.write_reg_int
`sys.write_reg_int(reghive: str, regpath: str, regname: str, regtype: str, regvalue: int) -> bool`
Writes an integer value to the Windows Registry.

**Parameters**
- `reghive` (`str`)
- `regpath` (`str`)
- `regname` (`str`)
- `regtype` (`str`): e.g., "REG_DWORD".
- `regvalue` (`int`)

**Returns**
- `bool`: True on success.

### sys.write_reg_str
`sys.write_reg_str(reghive: str, regpath: str, regname: str, regtype: str, regvalue: str) -> bool`
Writes a string value to the Windows Registry.

**Parameters**
- `reghive` (`str`)
- `regpath` (`str`)
- `regname` (`str`)
- `regtype` (`str`): e.g., "REG_SZ".
- `regvalue` (`str`)

**Returns**
- `bool`: True on success.

## Time Library
The `time` library provides time measurement, formatting, and sleep capabilities.

### time.format_to_epoch
`time.format_to_epoch(input: str, format: str) -> int`
Converts a formatted time string to a Unix timestamp (epoch seconds).

**Parameters**
- `input` (`str`): The time string (e.g., "2023-01-01 12:00:00").
- `format` (`str`): The format string (e.g., "%Y-%m-%d %H:%M:%S").

**Returns**
- `int`: The timestamp.

**Errors**
- Returns an error string if parsing fails.

### time.format_to_readable
`time.format_to_readable(input: int, format: str) -> str`
Converts a Unix timestamp to a readable string.

**Parameters**
- `input` (`int`): The timestamp (epoch seconds).
- `format` (`str`): The desired output format.

**Returns**
- `str`: The formatted time string.

### time.now
`time.now() -> int`
Returns the current time as a Unix timestamp.

**Returns**
- `int`: Current epoch seconds.

### time.sleep
`time.sleep(secs: int) -> None`
Pauses execution for the specified number of seconds.

**Parameters**
- `secs` (`int`): Seconds to sleep.

**Returns**
- `None`

