---
title: Eldritch
tags:
 - User Guide
description: Eldritch User Guide
permalink: user-guide/eldritch
---
# Overview

Eldritch is a pythonic red team Domain Specific Language (DSL) based on [starlark](https://github.com/facebookexperimental/starlark-rust).

Eldritch is a small interpreter that can be embedded into a c2 agent as it is with Golem and Imix.
By embedding the interpreter into the agent conditional logic can be quickly evaluated without requiring multiple callbacks.

Eldritch is currently under active development to help delineate methods in development the description contains the phrase `X method will`.

## Data types

Eldritch currently only supports the [default starlark data types.](https://github.com/facebookexperimental/starlark-rust/blob/main/docs/types.md)

## Error handling

Eldritch doesn't implement any form of error handling. If a function fails it will stop the tome from completing execution. There is no way to recover after a function has errored.

If you're using a functions that has a chance to error (functions that do file / network IO) test preemptively with function like `is_file`, `is_dir`, `is_windows`, etc.

For example:

```python
def read_passwd():
    if is_linux():
        if is_file("/etc/passwd"):
            file.read("/etc/passwd")
read_passwd()
```

```python
def write_systemd_service():
    if is_linux():
        if is_dir("/lib/systemd/system/"):
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

# Standard Library

The standard library is the default functionality that eldritch provides.

It currently contains five modules:

- `assets` - Used to interact with files stored natively in the agent.
- `file` - Used to interact with files on the system.
- `pivot` - Used to identify and move between systems.
- `process` - Used to interact with processes on the system.
- `sys` - General system capabilities can include loading libraries, or information about the current context.
- `crypto` - Used to encrypt/decrypt or hash data.

Functions fall into one of these five modules. This is done to improve clarity about function use.

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

## Assets

### assets.copy

`assets.copy(src: str, dst: str) -> None`

The <b>assets.copy</b> method copies an embedded file from the agent to disk.
The `srt` variable will be the path from the `embed_files_golem_prod` as the root dir.
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

The <b>assets.list</b> method returns a list of asset names that the agent is aware of.

### assets.read_binary

`assets.read_binary(src: str) -> List<int>`

The <b>assets.read_binary</b> method returns a list of u32 numbers representing the asset files bytes.

### assets.read

`assets.read(src: str) -> str`

The <b>assets.read</b> method returns a UTF-8 string representation of the asset file.

---

## File

### file.append

`file.append(path: str, content: str) -> None`

The <b>file.append</b> method appends the `content` to file at `path`. If no file exists at path create the file with the contents content.

### file.compress

`file.compress(src: str, dst: str) -> None`

The <b>file.compress</b> method compresses a file using the gzip algorithm. If the destination file doesn't exist it will be created. If the source file doesn't exist an error will be thrown. If the source path is a directory the contents will be placed in a tar archive and then compressed.

### file.copy

`file.copy(src: str, dst: str) -> None`

The <b>file.copy</b> method copies a file from `src` path to `dst` path. If `dst` file doesn't exist it will be created.

### file.download

`file.download(uri: str, dst: str) -> None`

The <b>file.download</b> method downloads a file at the URI specified in `uri` to the path specified in `dst`. If a file already exists at that location, it will be overwritten. This currently only supports `http` & `https` protocols.

### file.exists

`file.exists(path: str) -> bool`

The <b>file.exists</b> method checks if a file or directory exists at the path specified.

### file.hash

`file.hash(path: str) -> str`

The <b>file.hash</b> method returns a sha256 hash of the file specified in `path`.

### file.is_dir

`file.is_dir(path: str) -> bool`

The <b>file.is_dir</b> method checks if a path exists and is a directory. If it doesn't exist or is not a directory it will return `False`.

### file.is_file

`file.is_file(path: str) -> bool`

The <b>file.is_file</b> method checks if a path exists and is a file. If it doesn't exist or is not a file it will return `False`.

### file.list

`file.list(path: str) -> List<Dict>`

The <b>file.list</b> method returns a list of files at the specified path. The path is relative to your current working directory and can be traversed with `../`.
Each file is represented by a Dict type.
Here is an example of the Dict layout:

```json
[
    {
        "file_name": "implants",
        "size": 4096,
        "owner": "root",
        "group": "0",
        "permissions": "40755",
        "modified": "2023-07-09 01:35:40 UTC",
        "type": "Directory"
    },
    {
        "file_name": "README.md",
        "size": 750,
        "owner": "root",
        "group": "0",
        "permissions": "100644",
        "modified": "2023-07-08 02:49:47 UTC",
        "type": "File"
    },
    {
        "file_name": ".git",
        "size": 4096,
        "owner": "root",
        "group": "0",
        "permissions": "40755",
        "modified": "2023-07-10 21:14:06 UTC",
        "type": "Directory"
    }
]
```

### file.mkdir

`file.mkdir(path: str) -> None`

The <b>file.mkdir</b> method is make a new dirctory at `path`. If the parent directory does not exist or the directory cannot be otherwise be created, it will creat an error.

### file.moveto

`file.moveto(src: str, dst: str) -> None`

The <b>file.moveto</b> method moves a file or directory from src to `dst`. If the `dst` directory or file exists it will be deleted before being replaced to ensure consistency across systems.

### file.read

`file.read(path: str) -> str`

The <b>file.read</b> method will read the contents of a file. If the file or directory doesn't exist the method will error to avoid this ensure the file exists, and you have permission to read it.

### file.remove

`file.remove(path: str) -> None`

The <b>file.remove</b> method deletes a file or directory (and it's contents) specified by path.

### file.replace

`file.replace(path: str, pattern: str, value: str) -> None`
The <b>file.replace</b> method is very cool, and will be even cooler when Nick documents it.

### file.replace_all

`file.replace_all(path: str, pattern: str, value: str) -> None`

The <b>file.replace_all</b> method finds all strings matching a regex pattern in the specified file and replaces them with the value.

### file.template

`file.template(template_path: str, dst: str, args: Dict<String, Value>, autoescape: bool) -> None`

The <b>file.template</b> method reads a Jinja2 template file from disk, fill in the variables using `args` and then write it to the destination specified.
If the destination file doesn't exist it will be created (if the parent directory exists). If the destination file does exist it will be overwritten.
The `args` dictionary currently supports values of: `int`, `str`, and `List`.
`autoescape` when `True` will perform HTML character escapes according to the [OWASP XSS guidelines](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)

### file.timestomp

`file.timestomp(src: str, dst: str) -> None`

The <b>file.timestomp</b> method is very cool, and will be even cooler when Nick documents it.

### file.write

`file.write(path: str, content: str) -> None`

The <b>file.write</b> method writes to a given file path with the given content.
If a file or directory already exists at this path, the method will fail.

---

## Pivot

### pivot.arp_scan

`pivot.arp_scan(target_cidrs: List<str>) -> List<str>`

The <b>pivot.arp_scan</b> method is being proposed to allow users to enumerate hosts on their network without using TCP connect or ping.

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

The <b>pivot.bind_proxy</b> method is being proposed to provide users another option when trying to connect and pivot within an environment. This function will start a SOCKS5 proxy on the specified port and interface, with the specified username and password (if provided).

### pivot.ncat

`pivot.ncat(address: str, port: int, data: str, protocol: str ) -> str`

The <b>pivot.ncat</b> method allows a user to send arbitrary data over TCP/UDP to a host. If the server responds that response will be returned.

`protocol` must be `tcp`, or `udp` anything else will return an error `Protocol not supported please use: udp or tcp.`.

### pivot.port_scan

`pivot.port_scan(target_cidrs: List<str>, ports: List<int>, protocol: str, timeout: int) -> List<str>`

The <b>pivot.port_scan</b> method allows users to scan TCP/UDP ports within the eldritch language.
Inputs:

- `target_cidrs` must be in a CIDR format eg. `127.0.0.1/32`. Domains and single IPs `example.com` / `127.0.0.1` cannot be passed.
- `ports` can be a list of any number of integers between 1 and 65535.
- `protocol` must be: `tcp` or `udp`. These are the only supported options.
- `timeout` is the number of seconds a scan will wait without a response before it's marked as `timeout`

Results will be in the format:

```json
[
    { "ip": "127.0.0.1", "port": 22, "protocol": "tcp", "status": "open"},
    { "ip": "127.0.0.1", "port": 21, "protocol": "tcp", "status": "closed"},
    { "ip": "127.0.0.1", "port": 80, "protocol": "tcp", "status": "timeout"},
]
```

A ports status can be open, closed, or timeout:

|**State**|**Protocol**| **Meaning**                                          |
|---------|------------|------------------------------------------------------|
| open    | tcp        | Connection successful.                               |
| close   | tcp        | Connection refused.                                  |
| timeout | tcp        | Connection dropped or didn't respond.                |
| open    | udp        | Connection returned some data.                       |
| timeout | udp        | Connection was refused, dropped, or didn't respond.  |

Each IP in the specified CIDR will be returned regardless of if it returns any open ports.
Be mindful of this when scanning large CIDRs as it may create largo return objects.

NOTE: Windows scans against `localhost`/`127.0.0.1` can behave unexpectedly or even treat the action as malicious. Eg. scanning ports 1-65535 against windows localhost may cause the stack to overflow or process to hang indefinitely.

### pivot.port_forward

`pivot.port_forward(listen_address: str, listen_port: int, forward_address: str, forward_port:  int, str: protocol  ) -> None`

The <b>pivot.port_forward</b> method is being proposed to provide socat like functionality by forwarding traffic from a port on a local machine to a port on a different machine allowing traffic to be relayed.

### pivot.smb_exec

`pivot.smb_exec(target: str, port: int, username: str, password: str, hash: str, command: str) -> str`

The <b>pivot.smb_exec</b> method is being proposed to allow users a way to move between hosts running smb.

### pivot.ssh_copy

`pivot.ssh_copy(target: str, port: int, src: str, dst: str, username: str, password: Optional<str>, key: Optional<str>, key_password: Optional<str>, timeout: Optional<int>) -> None`

The <b>pivot.ssh_copy</b> method copies a local file to a remote system. If no password or key is specified the function will error out with:
`Failed to run handle_ssh_exec: Failed to authenticate to host`
If the connection is successful but the copy writes a file error will be returend.

ssh_copy will first delete the remote file and then write to it's location.
The file directory the `dst` file exists in must exist in order for ssh_copy to work.

### pivot.ssh_exec

`pivot.ssh_exec(target: str, port: int, command: str, username: str, password: Optional<str>, key: Optional<str>, key_password: Optional<str>, timeout: Optional<int>) -> List<Dict>`

The <b>pivot.ssh_exec</b> method executes a command string on the remote host using the default shell. If no password or key is specified the function will error out with:
`Failed to run handle_ssh_exec: Failed to authenticate to host`
If the connection is successful but the command fails no output will be returned but the status code will be set.
Not returning stderr is a limitation of the way we're performing execution. Since it's not using the SSH shell directive we're limited on the return output we can capture.

```json
{
    "stdout": "uid=1000(kali) gid=1000(kali) groups=1000(kali),24(cdrom),25(floppy),27(sudo),29(audio),30(dip),44(video),46(plugdev),109(netdev),118(bluetooth),128(lpadmin),132(scanner),143(docker)\n",
    "status": 0
}
```

### pivot.ssh_password_spray

`pivot.ssh_password_spray(targets: List<str>, port: int, credentials: List<str>, keys: List<str>, command: str, shell_path: str) -> List<str>`

The <b>pivot.ssh_password_spray</b> method is being proposed to allow users a way to test found credentials against neighboring targets. It will iterate over the targets list and try each credential set. Credentials will be a formatted list of usernames and passwords Eg. "username:password". The function will return a formatted list of "target:username:password". command and shell_path is intended to give more flexibility but may be adding complexity.

## Process

### process.kill

`process.kill(pid: int) -> None`

The <b>process.kill</b> method will kill a process using the KILL signal given its process id.

### process.list

`process.list() -> List<Dict>`

The <b>process.list</b> method returns a list of dictionaries that describe each process. The dictionaries follow the schema:

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

The <b>process.name</b> method returns the name of the process from it's given process id.

### process.info
`process.info(pid: Optional<int>) -> Dict`

The <b>process.info</b> method returns all information on a given process ID. Default is the current process.

```json
{
  "pid": 1286574,
  "name": "golem",
  "cmd": [
    "./target/debug/golem",
    "-i"
  ],
  "exe": "/home/user/realm/implants/target/debug/golem",
  "environ": [
    "USER=user",
    "HOME=/home/user",
    "PATH=/home/user/.cargo/bin:/usr/local/bin:/usr/bin:/bin:/usr/local/games:/usr/games:/snap/bin:/home/user/.dotnet/tools",
    "SHELL=/bin/zsh",
    "TERM=xterm-256color",
    "SSH_TTY=/dev/pts/0",
    "SHLVL=1",
    "PWD=/home/user",
    "OLDPWD=/home/user",
    "XDG_DATA_DIRS=/usr/local/share:/usr/share:/var/lib/snapd/desktop",
    "P9K_TTY=old",
    "_P9K_TTY=/dev/pts/0",
    "ZSH=/home/user/.oh-my-zsh",
  ],
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

### process.netstat
`process.netstat() -> Vec<Dict>`

The <b>process.netstat</b> method returns all information on TCP, UDP, and Unix sockets on the system. Will also return PID and Process Name of attached process, if one exists.

```json
[
    {
        "socket_type": "TCP",
        "local_address": "127.0.0.1",
        "local_port": 46341,
        "remote_address": "0.0.0.0",
        "remote_port": 0,
        "state": "LISTEN",
        "pids": [
            2359037
        ]
    },
    ...
]
```

## Sys

### sys.dll_inject

`sys.dll_inject(dll_path: str, pid: int) -> None`

The <b>sys.dll_inject</b> method will attempt to inject a dll on disk into a remote process by using the `CreateRemoteThread` function call.

### sys.exec

`sys.exec(path: str, args: List<str>, disown: Optional<bool>) -> Dict`

The <b>sys.exec</b> method executes a program specified with `path` and passes the `args` list.
Disown will run the process in the background disowned from the agent. This is done through double forking and only works on *nix systems.

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


### sys.hostname

`sys.hostname() -> String`

The <b>sys.hostname</b> method returns a String containing the host's hostname.

### sys.get_env

`sys.get_env() -> Dict`

The <b>sys.get_env</b> method returns a dictionary that describes the current process's environment variables.
An example is below:

```json
{
    "FOO": "BAR",
    "CWD": "/"
}
```

### sys.get_ip

`sys.get_ip() -> List<Dict>`

The <b>sys.get_ip</b> method returns a list of network interfaces as a dictionary. An example is available below:

```json
[
    {
        "name": "eth0",
        "ips": [
            "172.17.0.2"
        ],
        "mac": "02:42:ac:11:00:02"
    },
    {
        "name": "lo",
        "ips": [
            "127.0.0.1"
        ],
        "mac": "00:00:00:00:00:00"
    }
]
```

### sys.get_os

`sys.get_os() -> Dict`

The <b>sys.get_os</b> method returns a dictionary that describes the current systems OS.
An example is below:

```json
{
    "arch": "x86_64",
    "desktop_env": "Unknown: Unknown",
    "distro": "Debian GNU/Linux 10 (buster)",
    "platform": "Linux"
}
```

### sys.get_pid

`sys.get_pid() -> int`

The <b>sys.get_pid</b> method returns the process ID of the current process.
An example is below:

```python
$> sys.get_pid()
123456
```

### sys.get_user

`sys.get_user() -> Dict`

The <b>sys.get_user</b> method returns a dictionary that describes the current process's running user.
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

### sys.is_linux

`sys.is_linux() -> bool`

The <b>sys.is_linux</b> method returns `True` if on a linux system and `False` on everything else.

### sys.is_macos

`sys.is_macos() -> bool`

The <b>sys.is_macos</b> method returns `True` if on a mac os system and `False` on everything else.

### sys.is_windows

`sys.is_windows() -> bool`

The <b>sys.is_windows</b> method returns `True` if on a windows system and `False` on everything else.

### sys.shell

`sys.shell(cmd: str) -> Dict`

The <b>sys.shell</b> Given a string run it in a native interpreter. On MacOS, Linux, and *nix/bsd systems this is `/bin/bash -c <your command>`. On Windows this is `cmd /C <your command>`. Stdout, stderr, and the status code will be returned to you as a dictionary with keys: `stdout`, `stderr`, `status`. For example:

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

## Crypto

### crypto.aes_encrypt_file

`crypto.aes_encrypt_file(src: str, dst: str, key: str) -> None`

The <b>crypto.aes_encrypt_file</b> method encrypts the given src file, encrypts it using the given key and writes it to disk at the dst location.

Key must be 16 Bytes (Characters)

### crypto.aes_decrypt_file

`crypto.aes_decrypt_file(src: str, dst: str, key: str) -> None`

The <b>crypto.aes_decrypt_file</b> method decrypts the given src file using the given key and writes it to disk at the dst location.

Key must be 16 Bytes (Characters)

### crypto.hash_file

`crypto.hash_file(file: str, algo: str) -> str`

The <b>crypto.hash_file</b> method will produce the hash of the given file's contents. Valid algorithms include:

- MD5
- SHA1
- SHA256
- SHA512

### crypto.encode_b64

`crypto.encode_b64(content: str, encode_type: Optional<str>) -> str`

The <b>crypto.encode_b64</b> method encodes the given text using the given base64 encoding method. Valid methods include:

- STANDARD (default)
- STANDARD_NO_PAD
- URL_SAFE
- URL_SAFE_NO_PAD

### crypto.decode_b64

`crypto.decode_b64(content: str, decode_type: Optional<str>) -> str`

The <b>crypto.decode_b64</b> method encodes the given text using the given base64 decoding method. Valid methods include:

- STANDARD (default)
- STANDARD_NO_PAD
- URL_SAFE
- URL_SAFE_NO_PAD

### crypto.from_json

`crypto.from_json(content: str) -> Value`

The <b>crypto.from_json</b> method converts JSON text to an object of correct type.

```python
crypto.from_json("{\"foo\":\"bar\"}")
{
    "foo": "bar"
}
```

### crypto.to_json

`crypto.to_json(content: Value) -> str`

The <b>crypto.to_json</b> method converts given type to JSON text.

```python
crypto.to_json({"foo": "bar"})
"{\"foo\":\"bar\"}"
```
