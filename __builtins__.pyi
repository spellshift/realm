# __builtins__.pyi
#
# Type stub file for Eldritch-specific functions and modules,
# based on the provided list of code blocks from the Eldritch User Guide.
#
# See this documentation for more details
# https://github.com/microsoft/pyright/blob/main/docs/builtins.md
#
# to improve code, add this to .vscode/settings.json
# {
#     "files.associations": {
#         "*.eldritch": "python",
#         "*.eldr": "python",
#     },
#     "python.analysis.diagnosticSeverityOverrides": {
#         "reportPossiblyUnboundVariable": "none",
#         "reportInvalidStringEscapeSequence": "none",
#     }
# }

from typing import List, Dict, Any, Optional, Callable, Iterable, TypedDict, Literal


class Agent:
    """
    Used for meta-style interactions with the agent itself.
    """
    @staticmethod
    def eval(script: str) -> None:
        """
        The **agent.eval** method takes an arbitrary eldritch payload string and
        executes it in the runtime environment of the executing tome. This means that
        any `print`s or `eprint`s or output from the script will be merged with that
        of the broader tome.
        """
        ...

    @staticmethod
    def set_callback_interval(new_interval: int) -> None:
        """
        The **agent.set_callback_interval** method takes an unsigned int and changes the
        running agent's callback interval to the passed value. This configuration change will
        not persist across agent reboots.
        """
        ...

    @staticmethod
    def set_callback_uri(new_uri: str) -> None:
        """
        The **agent.set_callback_uri** method takes an string and changes the
        running agent's callback uri to the passed value. This configuration change will
        not persist across agent reboots. NOTE: please ensure the passed URI path is correct
        for the underlying `Transport` being used, as a URI can take many forms and we make no
        assumptions on `Transport` requirements no gut checks are applied to the passed string.
        """
        ...


class Assets:
    """
    Used to interact with files stored natively in the agent.
    """
    @staticmethod
    def copy(src: str, dst: str) -> None:
        """
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
        """
        ...

    @staticmethod
    def list() -> List[str]:
        """
        The **assets.list** method returns a list of asset names that the agent is aware of.
        """
        ...

    @staticmethod
    def read_binary(src: str) -> List[int]:
        """
        The **assets.read_binary** method returns a list of u32 numbers representing the asset files bytes.
        """
        ...

    @staticmethod
    def read(src: str) -> str:
        """
        The **assets.read** method returns a UTF-8 string representation of the asset file.
        """
        ...


class Crypto:
    """
    Used to encrypt/decrypt, decode, or hash data.
    """
    @staticmethod
    def aes_decrypt_file(src: str, dst: str, key: str) -> None:
        """
        The **crypto.aes_decrypt_file** method decrypts the given src file using the given key and writes it to disk at the dst location.

        Key must be 16 Bytes (Characters)
        """
        ...

    @staticmethod
    def aes_encrypt_file(src: str, dst: str, key: str) -> None:
        """
        The **crypto.aes_encrypt_file** method encrypts the given src file, encrypts it using the given key and writes it to disk at the dst location.

        Key must be 16 Bytes (Characters)
        """
        ...

    @staticmethod
    def encode_b64(
        content: str,
        encode_type: Optional[
            Literal[
                "STANDARD",
                "STANDARD_NO_PAD",
                "URL_SAFE",
                "URL_SAFE_NO_PAD",
            ]
        ] = None,
    ) -> str:
        """
        The **crypto.encode_b64** method encodes the given text using the given base64 encoding method. Valid methods include:

        - STANDARD (default)
        - STANDARD_NO_PAD
        - URL_SAFE
        - URL_SAFE_NO_PAD
        """
        ...

    @staticmethod
    def decode_b64(
        content: str,
        decode_type: Optional[
            Literal[
                "STANDARD",
                "STANDARD_NO_PAD",
                "URL_SAFE",
                "URL_SAFE_NO_PAD",
            ]
        ] = None,
    ) -> str:
        """
        The **crypto.decode_b64** method encodes the given text using the given base64 decoding method. Valid methods include:

        - STANDARD (default)
        - STANDARD_NO_PAD
        - URL_SAFE
        - URL_SAFE_NO_PAD
        """
        ...

    @staticmethod
    def is_json(content: str) -> Any:
        """
        The **crypto.is_json** method checks if the given input is valid JSON.

        ```python
        crypto.from_json("{\"foo\":\"bar\"}")
        True
        ```
        """
        ...

    @staticmethod
    def from_json(content: str) -> Any:
        """
        The **crypto.from_json** method converts JSON text to an object of correct type.

        ```python
        crypto.from_json("{\"foo\":\"bar\"}")
        {
            "foo": "bar"
        }
        ```
        """
        ...

    @staticmethod
    def hash_file(file: str, algo: str) -> str:
        """
        The **crypto.hash_file** method will produce the hash of the given file's contents. Valid algorithms include:

        - MD5
        - SHA1
        - SHA256
        - SHA512
        """
        ...

    @staticmethod
    def to_json(content: Any) -> str:
        """
        The **crypto.to_json** method converts given type to JSON text.

        ```python
        crypto.to_json({"foo": "bar"})
        "{\"foo\":\"bar\"}"
        ```
        """
        ...


class FileStat(TypedDict):
    """
    Represents file status information as returned by file.list.
    """
    file_name: str
    """The name of the file or directory."""
    absolute_path: str
    """The absolute path to the file or directory."""
    size: int
    """The size of the file in bytes."""
    owner: str
    """The owner of the file or directory."""
    group: str
    """The group owner of the file or directory."""
    permissions: str
    """The file permissions in a string format (e.g., 'rwxr-xr-x')."""
    modified: str
    """The last modification timestamp of the file or directory, in 'YYYY-MM-DD HH:MM:SS UTC' format."""
    type: str
    """The type of the file system entry ('Directory' or 'File')."""


class File:
    """
    Used to interact with files on the system.
    """
    @staticmethod
    def append(path: str, content: str) -> None:
        """
        The **file.append** method appends the `content` to file at `path`. If no file exists at path create the file with the content.
        """
        ...

    @staticmethod
    def compress(src: str, dst: str) -> None:
        """
        The **file.compress** method compresses a file using the gzip algorithm. If the destination file doesn't exist it will be created. If the source file doesn't exist an error will be thrown. If the source path is a directory the contents will be placed in a tar archive and then compressed.
        """
        ...

    @staticmethod
    def copy(src: str, dst: str) -> None:
        """
        The **file.copy** method copies a file from `src` path to `dst` path. If `dst` file doesn't exist it will be created.
        """
        ...

    @staticmethod
    def exists(path: str) -> bool:
        """
        The **file.exists** method checks if a file or directory exists at the path specified.
        """
        ...

    @staticmethod
    def follow(path: str, fn: Callable[[str], Any]) -> None:
        """
        The **file.follow** method will call `fn(line)` for any new `line` that is added to the file (such as from `bash_history` and other logs).

        ```python
        # Print every line added to bob's bash history
        file.follow('/home/bob/.bash_history', print)
        ```
        """
        ...

    @staticmethod
    def is_dir(path: str) -> bool:
        """
        The **file.is_dir** method checks if a path exists and is a directory. If it doesn't exist or is not a directory it will return `False`.
        """
        ...

    @staticmethod
    def is_file(path: str) -> bool:
        """
        The **file.is_file** method checks if a path exists and is a file. If it doesn't exist or is not a file it will return `False`.
        """
        ...

    @staticmethod
    def list(path: str) -> List[FileStat]:
        """
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
        """
        ...

    @staticmethod
    def mkdir(path: str, parent: Optional[bool] = None) -> None:
        """
        The **file.mkdir** method will make a new directory at `path`. If the parent directory does not exist or the directory cannot be created, it will error; unless the `parent` parameter is passed as `True`.
        """
        ...

    @staticmethod
    def moveto(src: str, dst: str) -> None:
        """
        The **file.moveto** method moves a file or directory from `src` to `dst`. If the `dst` directory or file exists it will be deleted before being replaced to ensure consistency across systems.
        """
        ...

    @staticmethod
    def parent_dir(path: str) -> str:
        """
        The **file.parent_dir** method returns the parent directory of a give path. Eg `/etc/ssh/sshd_config` -> `/etc/ssh`
        """
        ...

    @staticmethod
    def read(path: str) -> str:
        """
        The **file.read** method will read the contents of a file. If the file or directory doesn't exist the method will error to avoid this ensure the file exists, and you have permission to read it.
        This function supports globbing with `*` for example:

        ```python
        file.read("/home/*/.bash_history") # Read all files called .bash_history in sub dirs of `/home/`
        file.read("/etc/*ssh*") # Read the contents of all files that have `ssh` in the name. Will error if a dir is found.
        file.read("\\\\127.0.0.1\\c$\\Windows\\Temp\\metadata.yml") # Read file over Windows UNC
        ```
        """
        ...

    @staticmethod
    def read_binary(path: str) -> List[int]:
        """
        The **file.read_binary** method will read the contents of a file, **returning as a list of bytes**. If the file or directory doesn't exist the method will error to avoid this ensure the file exists, and you have permission to read it.
        This function supports globbing with `*` for example:

        ```python
        file.read_binary("/home/*/.bash_history") # Read all files called .bash_history in sub dirs of `/home/`
        file.read_binary("/etc/*ssh*") # Read the contents of all files that have `ssh` in the name. Will error if a dir is found.
        file.read_binary("\\\\127.0.0.1\\c$\\Windows\\Temp\\metadata.yml") # Read file over Windows UNC
        ```
        """
        ...

    @staticmethod
    def remove(path: str) -> None:
        """
        The **file.remove** method deletes a file or directory (and it's contents) specified by path.
        """
        ...

    @staticmethod
    def replace(path: str, pattern: str, value: str) -> None:
        """
        The **file.replace** method finds the first string matching a regex pattern in the specified file and replaces them with the value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.
        """
        ...

    @staticmethod
    def replace_all(path: str, pattern: str, value: str) -> None:
        """
        The **file.replace_all** method finds all strings matching a regex pattern in the specified file and replaces them with the value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.
        """
        ...

    @staticmethod
    def temp_file(name: Optional[str] = None) -> str:
        """
        The **file.temp** method returns the path of a new temporary file with a random filename or the optional filename provided as an argument.
        """
        ...

    @staticmethod
    def template(template_path: str, dst: str, args: Dict[str, Any], autoescape: bool) -> None:
        """
        The **file.template** method reads a Jinja2 template file from disk, fill in the variables using `args` and then write it to the destination specified.
        If the destination file doesn't exist it will be created (if the parent directory exists). If the destination file does exist it will be overwritten.
        The `args` dictionary currently supports values of: `int`, `str`, and `List`.
        `autoescape` when `True` will perform HTML character escapes according to the [OWASP XSS guidelines](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)
        """
        ...

    @staticmethod
    def timestomp(src: str, dst: str) -> None:
        """
        Unimplemented.
        """
        ...

    @staticmethod
    def write(path: str, content: str) -> None:
        """
        The **file.write** method writes to a given file path with the given content.
        If a file already exists at this path, the method will overwite it. If a directory
        already exists at the path the method will error.
        """
        ...

    @staticmethod
    def find(
        path: str,
        name: Optional[str] = None,
        file_type: Optional[Literal["file", "dir"]] = None,
        permissions: Optional[int] = None,
        modified_time: Optional[int] = None,
        create_time: Optional[int] = None
    ) -> List[str]:
        """
        The **file.find** method finds all files matching the used parameters. Returns file path for all matching items.

        - name: Checks if file name contains provided input
        - file_type: Checks for 'file' or 'dir' for files or directories, respectively.
        - permissions: On UNIX systems, takes numerical input of standard unix permissions (rwxrwxrwx == 777). On Windows, takes 1 or 0, 1 if file is read only.
        - modified_time: Checks if last modified time matches input specified in time since EPOCH
        - create_time: Checks if last modified time matches input specified in time since EPOCH
        """
        ...


class HTTP:
    """
    Used to make http(s) requests from the agent. The HTTP library also allows the user to allow the http client to ignore TLS validation via the `allow_insecure` optional parameter (defaults to `false`).
    """
    @staticmethod
    def download(uri: str, dst: str, allow_insecure: Optional[bool] = None) -> None:
        """
        The **http.download** method downloads a file at the URI specified in `uri` to the path specified in `dst`. If a file already exists at that location, it will be overwritten.
        """
        ...

    @staticmethod
    def get(
        uri: str,
        query_params: Optional[Dict[str, str]] = None,
        headers: Optional[Dict[str, str]] = None,
        allow_insecure: Optional[bool] = None
    ) -> str:
        """
        The **http.get** method sends an HTTP GET request to the URI specified in `uri` with the optional query paramters specified in `query_params` and headers specified in `headers`, then return the response body as a string. Note: in order to conform with HTTP2+ all header names are transmuted to lowercase.
        """
        ...

    @staticmethod
    def post(
        uri: str,
        body: Optional[str] = None,
        form: Optional[Dict[str, str]] = None,
        headers: Optional[Dict[str, str]] = None,
        allow_insecure: Optional[bool] = None
    ) -> str:
        """
        The **http.post** method sends an HTTP POST request to the URI specified in `uri` with the optional request body specified by `body`, form paramters specified in `form`, and headers specified in `headers`, then return the response body as a string. Note: in order to conform with HTTP2+ all header names are transmuted to lowercase. Other Note: if a `body` and a `form` are supplied the value of `body` will be used.
        """
        ...


class ARPTableEntry(TypedDict):
    """
    An entry in the ARP table, mapping an IP address to a MAC address.
    """
    ip: str
    """The IP address."""
    mac: str
    """The MAC address."""
    interface: str
    """The network interface associated with this entry."""


class PortScanResult(TypedDict):
    """
    The result of a port scan for a single port.
    """
    ip: str
    """The IP address that was scanned."""
    port: int
    """The port number that was scanned."""
    protocol: str
    """The protocol used for the scan (e.g., 'tcp', 'udp')."""
    status: str
    """The status of the port (e.g., 'open', 'closed', 'timeout')."""


class Pivot:
    """
    Used to identify and move between systems.
    """
    @staticmethod
    def arp_scan(target_cidrs: List[str]) -> List[ARPTableEntry]:
        """
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
        """
        ...

    @staticmethod
    def bind_proxy(listen_address: str, listen_port: int, username: str, password: str) -> None:
        """
        The **pivot.bind_proxy** method is being proposed to provide users another option when trying to connect and pivot within an environment. This function will start a SOCKS5 proxy on the specified port and interface, with the specified username and password (if provided).
        """
        ...

    @staticmethod
    def ncat(address: str, port: int, data: str, protocol: Literal["tcp", "udp"]) -> str:
        """
        The **pivot.ncat** method allows a user to send arbitrary data over TCP/UDP to a host. If the server responds that response will be returned.

        `protocol` must be `tcp`, or `udp` anything else will return an error `Protocol not supported please use: udp or tcp.`.
        """
        ...

    @staticmethod
    def port_forward(
        listen_address: str,
        listen_port: int,
        forward_address: str,
        forward_port: int,
        protocol: Literal["tcp", "udp"]
    ) -> None:
        """
        The **pivot.port_forward** method is being proposed to provide socat like functionality by forwarding traffic from a port on a local machine to a port on a different machine allowing traffic to be relayed.
        """
        ...

    @staticmethod
    def port_scan(
        target_cidrs: List[str],
        ports: List[int],
        protocol: Literal["tcp", "udp"],
        timeout: int
    ) -> List[PortScanResult]:
        """
        The **pivot.port_scan** method allows users to scan TCP/UDP ports within the eldritch language.
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

        |**State**|**Protocol**| **Meaning** |
        |---------|------------|------------------------------------------------------|
        | open    | tcp        | Connection successful.                               |
        | close   | tcp        | Connection refused.                                  |
        | timeout | tcp        | Connection dropped or didn't respond.                |
        | open    | udp        | Connection returned some data.                       |
        | timeout | udp        | Connection was refused, dropped, or didn't respond.  |

        Each IP in the specified CIDR will be returned regardless of if it returns any open ports.
        Be mindful of this when scanning large CIDRs as it may create large return objects.

        NOTE: Windows scans against `localhost`/`127.0.0.1` can behave unexpectedly or even treat the action as malicious. Eg. scanning ports 1-65535 against windows localhost may cause the stack to overflow or process to hang indefinitely.
        """
        ...

    @staticmethod
    def reverse_shell_pty(cmd: Optional[str] = None) -> None:
        """
        The **pivot.reverse_shell_pty** method spawns the provided command in a cross-platform PTY and opens a reverse shell over the agent's current transport (e.g. gRPC). If no command is provided, Windows will use `cmd.exe`. On other platforms, `/bin/bash` is used as a default, but if it does not exist then `/bin/sh` is used.
        """
        ...

    @staticmethod
    def smb_exec(target: str, port: int, username: str, password: str, hash: str, command: str) -> str:
        """
        The **pivot.smb_exec** method is being proposed to allow users a way to move between hosts running smb.
        """
        ...

    @staticmethod
    def ssh_copy(
        target: str,
        port: int,
        src: str,
        dst: str,
        username: str,
        password: Optional[str] = None,
        key: Optional[str] = None,
        key_password: Optional[str] = None,
        timeout: Optional[int] = None
    ) -> str:
        """
        The **pivot.ssh_copy** method copies a local file to a remote system.
        ssh_copy will return `"Sucess"` if successful and `"Failed to run handle_ssh_copy: ..."` on failure.
        If the connection is successful but the copy writes a file error will be returned.
        ssh_copy will overwrite the remote file if it exists.
        The file directory the `dst` file exists in must exist in order for ssh_copy to work.
        """
        ...

    @staticmethod
    def ssh_exec(
        target: str,
        port: int,
        command: str,
        username: str,
        password: Optional[str] = None,
        key: Optional[str] = None,
        key_password: Optional[str] = None,
        timeout: Optional[int] = None
    ) -> ShellResult:
        """
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
        """
        ...


class ProcessInfo(TypedDict):
    """
    Detailed information about a process.
    """
    pid: int
    """The process ID."""
    name: str
    """The name of the process."""
    cmd: List[str]
    """The command and arguments used to start the process."""
    exe: str
    """The path to the process executable."""
    environ: List[str]
    """A list of environment variables for the process."""
    cwd: str
    """The current working directory of the process."""
    root: str
    """The root directory of the process."""
    memory_usage: int
    """The resident set size (RSS) memory usage of the process in bytes."""
    virtual_memory_usage: int
    """The virtual memory size (VMS) usage of the process in bytes."""
    ppid: int
    """The parent process ID."""
    status: str
    """The current status of the process (e.g., 'Running', 'Sleeping', 'Stopped')."""
    start_time: int
    """The process start time as a Unix timestamp."""
    run_time: int
    """The total CPU time the process has consumed in seconds."""
    uid: int
    """The real user ID of the process."""
    euid: int
    """The effective user ID of the process."""
    gid: int
    """The real group ID of the process."""
    egid: int
    """The effective group ID of the process."""
    sid: int
    """The session ID of the process."""


class ProcessInfoSimple(TypedDict):
    """
    A simplified view of process information.
    """
    pid: str
    """The process ID as a string."""
    ppid: str
    """The parent process ID as a string."""
    status: str
    """The current status of the process (e.g., 'Sleeping', 'Running')."""
    name: str
    """The name of the process."""
    path: str
    """The path to the process executable."""
    username: str
    """The username of the process owner."""
    command: str
    """The full command line used to start the process."""
    cwd: str
    """The current working directory of the process."""
    environ: str
    """A string containing the environment variables of the process."""


class SocketInfo(TypedDict):
    """
    Information about an open socket.
    """
    socket_type: str
    """The type of socket (e.g., 'TCP', 'UDP')."""
    local_address: str
    """The local IP address the socket is bound to."""
    local_port: int
    """The local port number the socket is using."""
    pid: int
    """The process ID that owns the socket."""


class Process:
    """
    Used to interact with processes on the system.
    """
    @staticmethod
    def info(pid: Optional[int] = None) -> ProcessInfo:
        """
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
        """
        ...

    @staticmethod
    def kill(pid: int) -> None:
        """
        The **process.kill** method will kill a process using the KILL signal given its process id.
        """
        ...

    @staticmethod
    def list() -> List[ProcessInfoSimple]:
        """
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
        """
        ...

    @staticmethod
    def name(pid: int) -> str:
        """
        The **process.name** method returns the name of the process from it's given process id.
        """
        ...

    @staticmethod
    def netstat() -> List[SocketInfo]:
        """
        The **process.netstat** method returns all information on TCP, UDP, and Unix sockets on the system. Will also return PID and Process Name of attached process, if one exists.

        > Currently only shows LISTENING TCP connections

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
        """
        ...


class Random:
    """
    The random library is designed to enable generation of cryptographically secure random values. None of these functions will be blocking.
    """
    @staticmethod
    def bool() -> bool:
        """
        The **random.bool** method returns a randomly sourced boolean value.
        """
        ...

    @staticmethod
    def int(min: int, max: int) -> int:
        """
        The **random.int** method returns randomly generated integer value between a specified range. The range is inclusive on the minimum and exclusive on the maximum.
        """
        ...

    @staticmethod
    def string(length: int, charset: Optional[str] = None) -> str:
        """
        The **random.string** method returns a randomly generated string of the specified length. If `charset` is not provided defaults to [Alphanumeric](https://docs.rs/rand_distr/latest/rand_distr/struct.Alphanumeric.html). Warning, the string is stored entirely in memory so exceptionally large files (multiple megabytes) can lead to performance issues.
        """
        ...


class Regex:
    """
    The regex library is designed to enable basic regex operations on strings. Be aware as the underlying implementation is written
    in Rust we rely on the Rust Regex Syntax as talked about [here](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html). Further, we only support a single capture group currently, defining more/less than one will cause the tome to error.
    """
    @staticmethod
    def match_all(haystack: str, pattern: str) -> List[str]:
        """
        The **regex.match_all** method returns a list of capture group strings that matched the given pattern within the given haystack. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.
        """
        ...

    @staticmethod
    def match(haystack: str, pattern: str) -> str:
        """
        The **regex.match** method returns the first capture group string that matched the given pattern within the given
        haystack. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.
        """
        ...

    @staticmethod
    def replace_all(haystack: str, pattern: str, value: str) -> str:
        """
        The **regex.replace_all** method returns the given haystack with all the capture group strings that matched the given pattern replaced with the given value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.
        """
        ...

    @staticmethod
    def replace(haystack: str, pattern: str, value: str) -> str:
        """
        The **regex.replace** method returns the given haystack with the first capture group string that matched the given pattern replaced with the given value. Please consult the [Rust Regex Docs](https://rust-lang-nursery.github.io/rust-cookbook/text/regex.html) for more information on pattern matching.
        """
        ...


class Report:
    """
    The report library is designed to enable reporting structured data to Tavern. It's API is still in the active development phase, so **future versions of Eldritch may break tomes that rely on this API**.
    """
    @staticmethod
    def file(path: str) -> None:
        """
        The **report.file** method reports a file from the host that an Eldritch Tome is being evaluated on (e.g. a compromised system) to Tavern. It has a 1GB size limit, and will report the file in 1MB chunks. This process happens asynchronously, so after `report.file()` returns **there are no guarantees about when this file will be reported**. This means that if you delete the file immediately after reporting it, it may not be reported at all (race condition).
        """
        ...

    @staticmethod
    def process_list(list: List[Dict[str, Any]]) -> None:
        """
        The **report.process_list** method reports a snapshot of the currently running processes on the host system. This should only be called with the entire process list (e.g. from calling `process.list()`), as it will replace Tavern's current list of processes for the host with this new snapshot.
        """
        ...

    @staticmethod
    def ssh_key(username: str, key: str) -> None:
        """
        The **report.ssh_key** method reports a captured SSH Key credential to Tavern. It will automatically be associated with the host that the Eldritch Tome was being evaluated on.
        """
        ...

    @staticmethod
    def user_password(username: str, password: str) -> None:
        """
        The **report.user_password** method reports a captured username & password combination to Tavern. It will automatically be associated with the host that the Eldritch Tome was being evaluated on.
        """
        ...


class OSInfo(TypedDict):
    """
    Detailed information about the operating system.
    """
    arch: str
    """The architecture of the operating system (e.g., 'x86_64')."""
    desktop_env: str
    """The desktop environment in use (e.g., 'GNOME', 'KDE', or 'Unknown')."""
    distro: str
    """The distribution of the operating system (e.g., 'Debian GNU/Linux 10 (buster)')."""
    platform: str
    """The general platform of the operating system (e.g., 'PLATFORM_LINUX', 'PLATFORM_WINDOWS')."""


class UserDetail(TypedDict):
    """
    Detailed information about a user.
    """
    uid: int
    """The user ID."""
    name: str
    """The username."""
    gid: int
    """The primary group ID of the user."""
    groups: List[str]
    """A list of groups the user belongs to."""


class UserInfo(TypedDict):
    """
    Information about the current process's running user.
    """
    uid: UserDetail
    """Details for the real user ID."""
    euid: UserDetail
    """Details for the effective user ID."""
    gid: int
    """The real group ID of the process."""
    egid: int
    """The effective group ID of the process."""


class ShellResult(TypedDict):
    """
    The result of a shell command execution.
    """
    stdout: str
    """The standard output from the command."""
    stderr: str
    """The standard error from the command."""
    status: int
    """The exit status code of the command."""


class NetworkInterface(TypedDict):
    """
    Information about a single network interface.
    """
    name: str
    """The name of the network interface (e.g., 'eth0', 'lo')."""
    ips: List[str]
    """A list of IP addresses (with CIDR notation) assigned to the interface."""
    mac: str
    """The MAC address of the network interface."""


class Sys:
    """
    General system capabilities can include loading libraries, or information about the current context.
    """
    @staticmethod
    def dll_inject(dll_path: str, pid: int) -> None:
        """
        The **sys.dll_inject** method will attempt to inject a dll on disk into a remote process by using the `CreateRemoteThread` function call.
        """
        ...

    @staticmethod
    def dll_reflect(dll_bytes: List[int], pid: int, function_name: str) -> None:
        """
        The **sys.dll_reflect** method will attempt to inject a dll from memory into a remote process by using the loader defined in `realm/bin/reflective_loader`.

        The ints in dll_bytes will be cast down from int u32 ---> u8 in rust.
        If your dll_bytes array contains a value greater than u8::MAX it will cause the function to fail. If you're doing any decryption in starlark make sure to be careful of the u8::MAX bound for each byte.
        """
        ...

    @staticmethod
    def exec(
        path: str,
        args: List[str],
        disown: Optional[bool] = None,
        env_vars: Optional[Dict[str, str]] = None
    ) -> ShellResult:
        """
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
        """
        ...

    @staticmethod
    def get_env() -> Dict[str, str]:
        """
        The **sys.get_env** method returns a dictionary that describes the current process's environment variables.
        An example is below:

        ```json
        {
            "FOO": "BAR",
            "CWD": "/"
        }
        ```
        """
        ...

    @staticmethod
    def get_ip() -> List[NetworkInterface]:
        """
        The **sys.get_ip** method returns a list of network interfaces as a dictionary. An example is available below:

        ```json
        [
            {
                "name": "eth0",
                "ips": [
                    "172.17.0.2/24"
                ],
                "mac": "02:42:ac:11:00:02"
            },
            {
                "name": "lo",
                "ips": [
                    "127.0.0.1/8"
                ],
                "mac": "00:00:00:00:00:00"
            }
        ]
        ```
        """
        ...

    @staticmethod
    def get_os() -> OSInfo:
        """
        The **sys.get_os** method returns a dictionary that describes the current systems OS.
        An example is below:

        ```json
        {
            "arch": "x86_64",
            "desktop_env": "Unknown: Unknown",
            "distro": "Debian GNU/Linux 10 (buster)",
            "platform": "PLATFORM_LINUX"
        }
        ```
        """
        ...

    @staticmethod
    def get_pid() -> int:
        """
        The **sys.get_pid** method returns the process ID of the current process.
        An example is below:

        ```python
        $> sys.get_pid()
        123456
        ```
        """
        ...

    @staticmethod
    def get_reg(reghive: str, regpath: str) -> Dict[str, Any]:
        """
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
        """
        ...

    @staticmethod
    def get_user() -> UserInfo:
        """
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
        """
        ...

    @staticmethod
    def hostname() -> str:
        """
        The **sys.hostname** method returns a String containing the host's hostname.
        """
        ...

    @staticmethod
    def is_bsd() -> bool:
        """
        The **sys.is_bsd** method returns `True` if on a `freebsd`, `netbsd`, or `openbsd` system and `False` on everything else.
        """
        ...

    @staticmethod
    def is_linux() -> bool:
        """
        The **sys.is_linux** method returns `True` if on a linux system and `False` on everything else.
        """
        ...

    @staticmethod
    def is_macos() -> bool:
        """
        The **sys.is_macos** method returns `True` if on a mac os system and `False` on everything else.
        """
        ...

    @staticmethod
    def is_windows() -> bool:
        """
        The **sys.is_windows** method returns `True` if on a windows system and `False` on everything else.
        """
        ...

    @staticmethod
    def shell(cmd: str) -> ShellResult:
        """
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
        """
        ...

    @staticmethod
    def write_reg_hex(
        reghive: str, regpath: str, regname: str, regtype: str, regvalue: str
    ) -> bool:
        """
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
        """
        ...

    @staticmethod
    def write_reg_int(
        reghive: str, regpath: str, regname: str, regtype: str, regvalue: int
    ) -> bool:
        """
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
        """
        ...

    @staticmethod
    def write_reg_str(
        reghive: str, regpath: str, regname: str, regtype: str, regvalue: str
    ) -> bool:
        """
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
        """
        ...


class Time:
    """
    General functions for obtaining and formatting time, also add delays into code.
    """
    @staticmethod
    def format_to_epoch(input: str, format: str) -> int:
        """
        The **time.format_to_epoch** method returns the seconds since epoch for the given UTC timestamp of the provided format. Input must include date and time components.

        Some common formatting methods are:

        - "%Y-%m-%d %H:%M:%S" (24 Hour Time)
        - "%Y-%m-%d %I:%M:%S %P" (AM/PM)

        For reference on all available format specifiers, see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
        """
        ...

    @staticmethod
    def format_to_readable(input: int, format: str) -> str:
        """
        The **time.format_to_readable** method returns the timestamp in the provided format of the provided UTC timestamp.

        Some common formatting methods are:

        - "%Y-%m-%d %H:%M:%S" (24 Hour Time)
        - "%Y-%m-%d %I:%M:%S %P" (AM/PM)

        For reference on all available format specifiers, see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
        """
        ...

    @staticmethod
    def now() -> int:
        """
        The **time.now** method returns the time since UNIX EPOCH (Jan 01 1970). This uses the local system time.
        """
        ...

    @staticmethod
    def sleep(secs: float) -> None:
        """
        The **time.sleep** method sleeps the task for the given number of seconds.
        """
        ...


# Used for meta-style interactions with the agent itself.
agent: Agent = ...
assets: Assets = ...    # Used to interact with files stored natively in the agent.
crypto: Crypto = ...    # Used to encrypt/decrypt, hash, or encode data.
file: File = ...        # Used to interact with files on the system.
http: HTTP = ...        # Used to make http(s) requests from the agent.
pivot: Pivot = ...      # Used to identify targets and move between systems.
process: Process = ...  # Used to interact with processes on the system.
random: Random = ...    # Used to generate cryptographically secure random values.
regex: Regex = ...      # Used to perform regular expression functions on strings.
report: Report = ...    # Used to report structured data to Tavern
sys: Sys = ...          # Used to run general system capabilities, such as loading external libraries or to gather information about the system.
# Used to obtain and format time values, and to introduce delays or pauses within code execution.
time: Time = ...


# --- Global Starlark Built-in Functions ---
# These are standard Starlark functions that Eldritch supports.

def any(iterable: Iterable[Any]) -> bool:
    """
    Returns True if any element of the iterable is true.
    """
    ...


def dir(obj: Any) -> List[str]:
    """
    Returns a list of the names of the attributes of the given object.
    """
    ...


def sorted(iterable: Iterable[Any], key: Optional[Callable[[Any], Any]] = None, reverse: bool = False) -> List[Any]:
    """
    Returns a new sorted list from the items in iterable.
    """
    ...


def print(*values: Any) -> None:
    """
    Prints values to the console.
    """
    ...


def pprint(*values: Any) -> None:
    """
    Pretty-prints values to the console.
    """
    ...


def eprint(*values: Any) -> None:
    """
    Prints error values to the console.
    """
    ...


# Parameters passed to the tome from the UI
input_params: Dict[str, Any]
