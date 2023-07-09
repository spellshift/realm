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

Functions fall into one of these five modules. This is done to improve clarity about function use.

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

### assets.read
`assets.read(src: str) -> str`

The <b>assets.read</b> method returns a UTF-8 string representation of the file. 

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

### file.mkdir
`file.mkdir(path: str) -> None`

The <b>file.mkdir</b> method is very cool, and will be even cooler when Nick documents it.

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

The <b>file.write</b> method is very cool, and will be even cooler when Nick documents it.

---

## Pivot
### pivot.arp_scan
`pivot.arp_scan(target_cidrs: List<str>) -> List<str>`

The <b>pivot.arp_scan</b> method is being proposed to allow users to enumerate hosts on their network without using TCP connect or ping.

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

### pivot.ssh_exec
`pivot.ssh_exec(target: str, port: int, username: str, password: str, key: str, command: str, shell_path: str) -> List<str>`

The <b>pivot.ssh_exec</b> method is being proposed to allow users a way to move between hosts running ssh.

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

The <b>process.name</b> method is very cool, and will be even cooler when Nick documents it.

## Sys
### sys.dll_inject
`sys.dll_inject(dll_path: str, pid: int) -> None`

The <b>sys.dll_inject</b> method will attempt to inject a dll on disk into a remote process by using the `CreateRemoteThread` function call.

### sys.exec
`sys.exec(path: str, args: List<str>, disown: bool) -> Dict`

The <b>sys.exec</b> method executes a program specified with `path` and passes the `args` list.
Disown will run the process in the background disowned from the agent. This is done through double forking and only works on *nix systems.


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

