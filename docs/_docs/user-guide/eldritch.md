---
title: Eldritch
tags: 
 - User Guide
description: Eldritch User Guide
permalink: user-guide/eldritch
---
# Overview
![/assets/img/coming-soon.gif](/assets/img/coming-soon.gif)

## Data types
Eldritch currently only supports the [default starlark data types.](https://github.com/facebookexperimental/starlark-rust/blob/main/docs/types.md)

## Error handling
Eldritch doesn't implement any form of error handling. If a function fails it will stop the tome from completing execution. There is no way to recover after a function has errored.

If you're using a functions that has a chance to error (functions that do file / network IO) test preemptively with function like `is_file`, `is_dir`, `is_windows`, etc.

For example:
```Python
if is_linux():
    if is_file("/etc/passwd"):
        file.read("/etc/passwd")
```


# Standard Library
The Standard Library is very cool, and will be even cooler when Nick documents it.

### file.append
`file.append(path: str, content: str) -> None`

The <b>file.append</b> Append content str to file at path. If no file exists at path create the file with the contents content.

### file.copy
`file.copy(src: str, dst: str) -> None`

The <b>file.copy</b> copies a file from src path to dst path. If dst file doesn't exist it will be created.

### file.compress
`file.compress(src: str, dst: str) -> None`

The <b>file.compress</b> function compresses a file using the gzip algorithm. If the destination file doesn't exist it will be created. If the source file doesn't exist an error will be thrown. If the source path is a directory the contents will be placed in a tar archive and then compressed.

### file.download
`file.download(uri: str, dst: str) -> None`

The <b>file.download</b> method downloads a file at the URI specified in `uri` to the path specified in `dst`. If a file already exists at that location, it will be overwritten. This currently only supports `http` & `https` protocols.

### file.exists
`file.exists(path: str) -> bool`

The <b>file.exists</b> checks if a file or directory exists at the path specified.

### file.hash
`file.hash(path: str) -> str`

The <b>file.hash</b> takes a sha256 hash of the file specified in path.

### file.is_dir
`file.is_dir(path: str) -> bool`

The <b>file.is_dir</b> checks if a path exists and is a directory. If it doesn't exist or is not a directory it will return false.

### file.is_file
`file.is_file(path: str) -> bool`

The <b>file.is_file</b> checks if a path exists and is a file. If it doesn't exist or is not a file it will return false.

### file.mkdir
`file.mkdir(path: str) -> None`

The <b>file.mkdir</b> method is very cool, and will be even cooler when Nick documents it.

### file.read
`file.read(path: str) -> str`

The <b>file.read</b> method will read the contents of a file. If the file or directory doesn't exist the method will error to avoid this ensure the file exists, and you have permission to read it.

### file.remove
`file.remove(path: str) -> None`

The <b>file.remove</b> method will delete a file or directory (and it's contents) specified by path.

### file.moveto
`file.moveto(src: str, dst: str) -> None`

The <b>file.moveto</b> method will move a file or directory from src to dst. If the dst directory or file exists it will be deleted before being replaced. To ensure consistency across systems.

### file.replace
`file.replace(path: str, pattern: str, value: str) -> None`
The <b>file.replace</b> method is very cool, and will be even cooler when Nick documents it.

### file.replace_all
`file.replace_all(path: str, pattern: str, value: str) -> None`

The <b>file.replace_all</b> method finds all strings matching a regex pattern in the specified file and replaces them with the value.

### file.timestomp
`file.timestomp(src: str, dst: str) -> None`

The <b>file.timestomp</b> method is very cool, and will be even cooler when Nick documents it.

### file.write
`file.write(path: str, content: str) -> None`

The <b>file.write</b> method is very cool, and will be even cooler when Nick documents it.

### process.kill
`process.kill(pid: int) -> None`

The <b>process.kill</b> will kill a process using the KILL signal given its process id.

### process.list
`process.list() -> List<Dict>`

The <b>process.list</b> method will return a list of dictionarys that describe each process. The dictionaries follow the schema:
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

The <b>process.name</b> method is very cool, and will be even cooler when Nick documents it.

### sys.dll_inject
`sys.dll_inject(dll_path: str, pid: int) -> None`

The <b>sys.dll_inject</b> method will attempt to inject a dll on disk into a remote process by using the `CreateRemoteThread` function call.

### sys.exec
`sys.exec(path: str, args: List<str>, disown: bool) -> Dict`

The <b>sys.exec</b> method executes a program specified with `path` and passes the `args` list.
Disown will run the process in the background disowned from the agent. This is done through double forking and only works on *nix systems.

When disown is true the returned dictionary will be `{"stdout":"","stderr":"","status":0}`.
If disown is not used stdout from the process will be returned stdout, stderr, and the status code will be returned to you as a dictionary with keys: `stdout`, `stderr`, `status`. For example:

```python
sys.execute("/bin/bash",["-c", "whoami"])
{
    "stdout":"root\n",
    "stderr":"",
    "status":0,
}
sys.execute("/bin/bash",["-c", "ls /nofile"])
{
    "stdout":"",
    "stderr":"ls: cannot access '/nofile': No such file or directory\n",
    "status":2,
}
```

### sys.is_linux
`sys.is_linux() -> bool`

The <b>sys.is_linux</b> method returns true if on a linux system and fales on everything else.

### sys.is_windows
`sys.is_windows() -> bool`

The <b>sys.is_windows</b> method returns true if on a windows system and fales on everything else.

### sys.is_macos
`sys.is_macos() -> bool`

The <b>sys.is_macos</b> method returns true if on a mac os system and false on everything else.

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


### pivot.ssh_exec
`pivot.ssh_exec(target: str, port: int, username: str, password: str, key: str, command: str, shell_path: str) -> List<str>`

The <b>pivot.ssh_exec</b> method is being proposed to allow users a way to move between hosts running ssh.

### pivot.ssh_password_spray
`pivot.ssh_password_spray(targets: List<str>, port: int, credentials: List<str>, keys: List<str>, command: str, shell_path: str) -> List<str>`

The <b>pivot.ssh_password_spray</b> method is being proposed to allow users a way to test found credentials against neighboring targets. It will iterate over the targets list and try each credential set. Credentials will be a formatted list of usernames and passwords Eg. "username:password". The function will return a formatted list of "target:username:password". command and shell_path is intended to give more felxiblity but may be adding complexity.

### pivot.smb_exec
`pivot.smb_exec(target: str, port: int, username: str, password: str, hash: str, command: str) -> str`

The <b>pivot.smb_exec</b> method is being proposed to allow users a way to move between hosts running smb.

### pivot.port_scan
`pivot.port_scan(target_cidrs: List<str>, ports: List<int>, protocol: str, timeout: int) -> List<str>`

The <b>pivot.port_scan</b> method allows users to scan TCP/UDP ports within the eldritch language.
Inputs:
- `target_cidrs` must be in a CIDR format eg. `127.0.0.1/32`. Domains and single IPs `example.com` / `127.0.0.1` cannot be passed.
- `ports` can be a list of any number of integers between 1 and 65535.
- `protocol` must be: `tcp` or `udp`. These are the only supported options.
- `timeout` is the number of seconds a scan will wait without a response before it's marked as `timeout`

Results will be in the format:
```JSON
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
| timeout | udp        | Connection was refused, dropped, or didn't respond   |

Each IP in the specified CIDR will be returned regardless of if it returns any open ports.
Be mindful of this when scanning large CIDRs as it may create largo return objects.

NOTE: Windows scans against `localhost`/`127.0.0.1` can behave unexpectedly or even treat the action as malicious. Eg. scanning ports 1-65535 against windows localhost may cause the stack to overflow or process to hang indefinitely.

### pivot.arp_scan
`pivot.arp_scan(target_cidrs: List<str>) -> List<str>`

The <b>pivot.arp_scan</b> method is being proposed to allow users to enumerate hosts on their network  without using TCP connect or ping.

### pivot.port_forward
`pivot.port_forward(listen_address: str, listen_port: int, forward_address: str, forward_port:  int, str: portocol  ) -> None`

The <b>pivot.port_forward</b> method is being proposed to providde socat like functionality by forwarding traffic from a port on a local machine to a port on a different machine allowing traffic to be relayed.

### pivot.ncat
`pivot.ncat(address: str, port: int, data: str, str: portocol ) -> str`

The <b>pivot.ncat</b> method allows a user to send arbitrary data over TCP/UDP to a host. If the server responds that response will be returned.

### pivot.bind_proxy
`pivot.bind_proxy(listen_address: str, listen_port: int, username: str, password: str ) -> None`

The <b>pivot.bind_proxy</b> method is being proposed to provide users another option when trying to connect and pivot within an environment. This function will start a SOCKS5 proxy on the specificed port and interface, with the specificed username and password (if provided).
