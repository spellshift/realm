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

The <b>file.remove</b> method is very cool, and will be even cooler when Nick documents it.

### file.rename
`file.rename(src: str, dst: str) -> None`

The <b>file.rename</b> method is very cool, and will be even cooler when Nick documents it.

### file.replace
`file.replace(path: str, pattern: str, value: str) -> 
None`
The <b>file.replace</b> method is very cool, and will be even cooler when Nick documents it.

### file.replace_all
`file.replace_all(path: str, pattern: str, value:` str) -> None

The <b>file.replace_all</b> method is very cool, and will be even cooler when Nick documents it.

### file.timestomp
`file.timestomp(src: str, dst: str) -> None`

The <b>file.timestomp</b> method is very cool, and will be even cooler when Nick documents it.

### file.write
`file.write(path: str, content: str) -> None`

The <b>file.write</b> method is very cool, and will be even cooler when Nick documents it.

### process.kill
`process.kill(pid: int) -> None`

The <b>process.kill</b> method is very cool, and will be even cooler when Nick documents it.

### process.list
`process.list() -> List<int>`

The <b>process.list</b> method is very cool, and will be even cooler when Nick documents it.

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

### sys.shell
`sys.shell(cmd: str) -> str`

The <b>sys.shell</b> method is very cool, and will be even cooler when Nick documents it.
