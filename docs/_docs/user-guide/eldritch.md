---
title: Eldritch
tags: 
 - User Guide
description: Eldritch User Guide
permalink: user-guide/eldritch
---
# Overview
![/assets/img/coming-soon.gif](/assets/img/coming-soon.gif)

# Examples
![/assets/img/coming-soon.gif](/assets/img/coming-soon.gif)

# Standard Library
The Standard Library is very cool, and will be even cooler when Nick documents it.

### file.append
`file.append(path: str, content: str) -> None`

The <b>file.append</b> Append content str to file at path. If no file exists at path create the file with the contents content.

### file.copy
`file.copy(src: str, dst: str) -> None`

The <b>file.copy</b> copies a file from src path to dst path. If dst path doesn't exist it will be created.

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
`process.list() -> List<str>`

The <b>process.list</b> method will return a list of JSON strings representing the current process list. Eg. `"{pid:9,ppid:0,status:\"Sleeping\",username:\"root\",path:\"/bin/dash\",command:\"/bin/sh\",cwd:\"/\",environ:\"TERM_PROGRAM_VERSION=1.65.2 USER=root\",name:\"sh\"}"`

### process.name
`process.name(pid: int) -> str`

The <b>process.name</b> method is very cool, and will be even cooler when Nick documents it.### Sys Library
The Sys Library is very cool, and will be even cooler when Nick documents it.

### sys.exec
`sys.exec(path: str, args: List<str>, ?disown: bool) -> str`

The <b>sys.exec</b> method is very cool, and will be even cooler when Nick documents it.

### sys.is_linux
`sys.is_linux() -> bool`

The <b>sys.is_linux</b> method is very cool, and will be even cooler when Nick documents it.

### sys.is_windows
`sys.is_windows() -> bool`

The <b>sys.is_windows</b> method is very cool, and will be even cooler when Nick documents it.

### sys.is_macos
`sys.is_macos() -> bool`

The <b>sys.is_macos</b> method returns true if on a mac os system and fales on everything else.

### sys.shell
`sys.shell(cmd: str) -> str`

The <b>sys.shell</b> Given a string run it in a native interpreter. On MacOS, Linux, and *nix/bsd systems this is `/bin/bash -c <your command>`. On Windows this is `cmd /C <your command>`. Stdout from the process will be returned. If your command errors the error will be ignored and not passed back to you.

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
`pivot.port_scan(target_cidrs: List<str>, ports: List<int>, portocol: str) -> List<str>`

The <b>pivot.port_scan</b> method is being proposed to allow users to scan the network for open ports. It will take a list of CIDRs and/or IPs and return the results in a grepable format similar to nmap.

### pivot.arp_scan
`pivot.arp_scan(target_cidrs: List<str>) -> List<str>`

The <b>pivot.arp_scan</b> method is being proposed to allow users to enumerate hosts on their network  without using TCP connect or ping.

### pivot.port_forward
`pivot.port_forward(listen_address: str, listen_port: int, forward_address: str, forward_port:  int, str: portocol  ) -> None`

The <b>pivot.port_forward</b> method is being proposed to providde socat like functionality by forwarding traffic from a port on a local machine to a port on a different machine allowing traffic to be relayed.

### pivot.ncat
`pivot.ncat(address: str, port: int, data: str, str: portocol, timeout: int ) -> String`

The <b>pivot.ncat</b> method is being proposed to allow arbitrary data to be sent to a host. The results will be reutrned.

### pivot.bind_proxy
`pivot.bind_proxy(listen_address: str, listen_port: int, username: str, password: str ) -> None`

The <b>pivot.bind_proxy</b> method is being proposed to provide users another option when trying to connect and pivot within an environment. This function will start a SOCKS5 proxy on the specificed port and interface, with the specificed username and password (if provided).
