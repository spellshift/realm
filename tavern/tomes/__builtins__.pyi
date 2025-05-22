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

from typing import List, Dict, Any, Optional, Callable, Iterable, TypedDict

# --- Eldritch Modules and Functions ---


class Agent:
    """
    Used for meta-style interactions with the agent itself.
    """

    def eval(self, script: str) -> None:
        """
        The agent.eval method takes an arbitrary eldritch payload string and
        executes it in the runtime environment of the executing tome.
        """
        ...

    def set_callback_interval(self, new_interval: int) -> None:
        """
        The agent.set_callback_interval method takes an unsigned int and changes the
        running agent’s callback interval to the passed value.
        This configuration change will not persist across agent reboots.
        """
        ...

    def set_callback_uri(self, new_uri: str) -> None:
        """
        The agent.set_callback_uri method takes a string and changes the
        running agent’s callback uri to the passed value.
        This configuration change will not persist across agent reboots.
        """
        ...


agent: Agent = ...  # Global instance of the Agent module


class Assets:
    """
    Used to interact with files stored natively in the agent.
    """

    def copy(self, src: str, dst: str) -> None:
        """
        The assets.copy method copies an embedded file from the agent to disk.
        """
        ...

    def list(self) -> List[str]:
        """
        The assets.list method returns a list of asset names that the agent is aware of.
        """
        ...

    # Documented as List<u32>, mapping to List[int]
    def read_binary(self, src: str) -> List[int]:
        """
        The assets.read_binary method returns a list of u32 numbers representing the asset files bytes.
        """
        ...

    def read(self, src: str) -> str:
        """
        The assets.read method returns a UTF-8 string representation of the asset file.
        """
        ...


assets: Assets = ...  # Global instance of the Assets module


class Crypto:
    """
    Used to encrypt/decrypt or hash data.
    """

    def aes_decrypt_file(self, src: str, dst: str, key: str) -> None:
        """
        The crypto.aes_decrypt_file method decrypts the given src file using the given key and writes it to disk at the dst location.
        Key must be 16 Bytes (Characters).
        """
        ...

    def aes_encrypt_file(self, src: str, dst: str, key: str) -> None:
        """
        The crypto.aes_encrypt_file method encrypts the given src file, encrypts it using the given key and writes it to disk at the dst location.
        Key must be 16 Bytes (Characters).
        """
        ...

    def encode_b64(self, content: str, encode_type: Optional[str] = None) -> str:
        """
        The crypto.encode_b64 method encodes the given text using the given base64 encoding method.
        Valid methods: STANDARD (default), STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD.
        """
        ...

    def decode_b64(self, content: str, decode_type: Optional[str] = None) -> str:
        """
        The crypto.decode_b64 method encodes the given text using the given base64 decoding method.
        Valid methods: STANDARD (default), STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD.
        """
        ...

    # 'Value' type in Starlark, maps to Any in Python typing
    def from_json(self, content: str) -> Any:
        """
        The crypto.from_json method converts JSON text to an object of correct type.
        """
        ...

    def hash_file(self, file: str, algo: str) -> str:
        """
        The crypto.hash_file method will produce the hash of the given file’s contents.
        Valid algorithms: MD5, SHA1, SHA256, SHA512.
        """
        ...

    # 'Value' type in Starlark, maps to Any in Python typing
    def to_json(self, content: Any) -> str:
        """
        The crypto.to_json method converts given type to JSON text.
        """
        ...


crypto: Crypto = ...  # Global instance of the Crypto module


class FileStat(TypedDict):
    """
    Represents file status information as returned by file.list.
    """
    file_name: str
    absolute_path: str
    size: int
    owner: str
    group: str
    permissions: str
    modified: str  # Documented as "2023-07-09 01:35:40 UTC", so string
    type: str  # "Directory" or "File"


class File:
    """
    Used to interact with files on the system.
    """

    def append(self, path: str, content: str) -> None:
        """
        The file.append method appends the `content` to file at `path`.
        If no file exists at path create the file with the content.
        """
        ...

    def compress(self, src: str, dst: str) -> None:
        """
        The file.compress method compresses a file using the gzip algorithm.
        """
        ...

    def copy(self, src: str, dst: str) -> None:
        """
        The file.copy method copies a file from `src` path to `dst` path.
        """
        ...

    def exists(self, path: str) -> bool:
        """
        The file.exists method checks if a file or directory exists at the path specified.
        """
        ...

    def follow(self, path: str, fn: Callable[[str], Any]) -> None:
        """
        The file.follow method will call `fn(line)` for any new `line` that is added to the file.
        """
        ...

    def is_dir(self, path: str) -> bool:
        """
        The file.is_dir method checks if a path exists and is a directory.
        """
        ...

    def is_file(self, path: str) -> bool:
        """
        The file.is_file method checks if a path exists and is a file.
        """
        ...

    def list(self, path: str) -> List[FileStat]:
        """
        The file.list method returns a list of files at the specified path.
        """
        ...

    def mkdir(self, path: str, parent: Optional[bool] = None) -> None:
        """
        The file.mkdir method will make a new directory at `path`.
        """
        ...

    def moveto(self, src: str, dst: str) -> None:
        """
        The file.moveto method moves a file or directory.
        """
        ...

    def parent_dir(self, path: str) -> str:
        """
        Returns the parent directory of the given path.
        """
        ...

    def read(self, path: str) -> str:
        """
        Reads the content of a file. Can return as string or list of integers (bytes).
        """
        ...

    def remove(self, path: str) -> None:
        """
        Removes a file or directory.
        """
        ...

    def replace(self, path: str, pattern: str, value: str) -> None:
        """
        Replaces the first occurrence of `pattern` with `value` in the file at `path`.
        """
        ...

    def replace_all(self, path: str, pattern: str, value: str) -> None:
        """
        Replaces all occurrences of `pattern` with `value` in the file at `path`.
        """
        ...

    def temp_file(self, name: Optional[str] = None) -> str:
        """
        Creates a temporary file and returns its path.
        """
        ...

    def template(self, template_path: str, dst: str, args: Dict[str, Any], autoescape: bool) -> None:
        """
        Processes a template file using provided arguments and writes the result to `dst`.
        """
        ...

    def timestomp(self, src: str, dst: str) -> None:
        """
        Copies timestamps from `src` file to `dst` file.
        """
        ...

    def write(self, path: str, content: str) -> None:
        """
        Writes `content` to a file at `path`. Overwrites if file exists.
        """
        ...

    def find(self, path: str, name: Optional[str] = None, file_type: Optional[str] = None, permissions: Optional[int] = None, modified_time: Optional[int] = None, create_time: Optional[int] = None) -> List[str]:
        """
        Finds files based on various criteria.
        """
        ...


file: File = ...  # Global instance of the File module


class Http:
    """
    Used to make http(s) requests from the agent.
    """

    def download(self, uri: str, dst: str, allow_insecure: Optional[bool] = None) -> None:
        """
        Downloads content from a URI to a destination path.
        """
        ...

    def get(self, uri: str, query_params: Optional[Dict[str, str]] = None, headers: Optional[Dict[str, str]] = None, allow_insecure: Optional[bool] = None) -> str:
        """
        Performs an HTTP GET request.
        """
        ...

    def post(self, uri: str, body: Optional[str] = None, form: Optional[Dict[str, str]] = None, headers: Optional[Dict[str, str]] = None, allow_insecure: Optional[bool] = None) -> str:
        """
        Performs an HTTP POST request.
        """
        ...


http: Http = ...  # Global instance of the Http module


class Pivot:
    """
    Used to identify and move between systems.
    """

    def arp_scan(self, target_cidrs: List[str]) -> List[str]:
        """
        Performs an ARP scan on target CIDRs.
        """
        ...

    def bind_proxy(self, listen_address: str, listen_port: int, username: str, password: str) -> None:
        """
        Binds a proxy for pivot.
        """
        ...

    def ncat(self, address: str, port: int, data: str, protocol: str) -> str:
        """
        Performs ncat-like network operations.
        """
        ...

    def port_forward(self, listen_address: str, listen_port: int, forward_address: str, forward_port: int, protocol: str) -> None:
        """
        Forwards a port.
        """
        ...

    def port_scan(self, target_cidrs: List[str], ports: List[int], protocol: str, timeout: int) -> List[str]:
        """
        Performs a port scan on target CIDRs and ports.
        """
        ...

    def reverse_shell_pty(self, cmd: Optional[str] = None) -> None:
        """
        Establishes a reverse shell with PTY.
        """
        ...

    def smb_exec(self, target: str, port: int, username: str, password: str, hash: str, command: str) -> str:
        """
        Executes a command over SMB.
        """
        ...

    def ssh_copy(self, target: str, port: int, src: str, dst: str, username: str, password: Optional[str] = None, key: Optional[str] = None, key_password: Optional[str] = None, timeout: Optional[int] = None) -> str:
        """
        Copies files over SSH.
        """
        ...

    def ssh_exec(self, target: str, port: int, command: str, username: str, password: Optional[str] = None, key: Optional[str] = None, key_password: Optional[str] = None, timeout: Optional[int] = None) -> List[Dict[str, Any]]:
        """
        Executes a command over SSH.
        """
        ...


pivot: Pivot = ...  # Global instance of the Pivot module


class Process:
    """
    Used to interact with processes on the system.
    """

    def info(self, pid: Optional[int] = None) -> Dict[str, Any]:
        """
        Returns information about a process or the current process.
        """
        ...

    def kill(self, pid: int) -> None:
        """
        Kills a process by its PID.
        """
        ...

    def list(self) -> List[Dict[str, Any]]:
        """
        Lists running processes.
        """
        ...

    def name(self, pid: int) -> str:
        """
        Returns the name of the process given its PID.
        """
        ...

    def netstat(self) -> List[Dict[str, Any]]:
        """
        Returns network connection statistics for processes.
        """
        ...


process: Process = ...  # Global instance of the Process module


class Random:
    """
    Used to generate cryptographically secure random values.
    """

    def bool(self) -> bool:
        """
        Generates a random boolean value.
        """
        ...

    def int(self, min: int, max: int) -> int:
        """
        Generates a random integer within a specified range.
        """
        ...

    def string(self, length: int, charset: Optional[str] = None) -> str:
        """
        Generates a random string of specified length and optional charset.
        """
        ...


random: Random = ...  # Global instance of the Random module


class Regex:
    """
    Regular expression capabilities for operating on strings.
    """

    def match_all(self, haystack: str, pattern: str) -> List[str]:
        """
        Finds all non-overlapping matches of `pattern` in `haystack`.
        """
        ...

    # This signature implies a single match, not Optional.
    def match(self, haystack: str, pattern: str) -> str:
        """
        Attempts to match `pattern` at the beginning of `haystack`.
        """
        ...

    def replace_all(self, haystack: str, pattern: str, value: str) -> str:
        """
        Replaces all occurrences of `pattern` in `haystack` with `value`.
        """
        ...

    def replace(self, haystack: str, pattern: str, value: str) -> str:
        """
        Replaces the first occurrence of `pattern` in `haystack` with `value`.
        """
        ...


regex: Regex = ...  # Global instance of the Regex module


class Report:
    """
    Structured data reporting capabilities.
    """

    def file(self, path: Optional[str] = None) -> None:
        """
        Reports file information. Can be called with or without a path.
        """
        ...

    def process_list(self, list: List[Dict[str, Any]]) -> None:
        """
        Reports a list of processes.
        """
        ...

    def ssh_key(self, username: str, key: str) -> None:
        """
        Reports an SSH key associated with a username.
        """
        ...

    def user_password(self, username: str, password: str) -> None:
        """
        Reports a user password.
        """
        ...


report: Report = ...  # Global instance of the Report module


class Sys:
    """
    General system capabilities can include loading libraries, or information about the current context.
    """

    def dll_inject(self, dll_path: str, pid: int) -> None:
        """
        Injects a DLL into a process.
        """
        ...

    def dll_reflect(self, dll_bytes: List[int], pid: int, function_name: str) -> None:
        """
        Reflectively loads and executes a function from DLL bytes in a process.
        """
        ...

    def exec(self, path: str, args: List[str], disown: Optional[bool] = None, env_vars: Optional[Dict[str, str]] = None) -> Dict[str, Any]:
        """
        Executes an external command.
        Returns a dictionary with 'stdout', 'stderr', and 'status'.
        """
        ...

    def get_env(self) -> Dict[str, str]:
        """
        Returns a dictionary of environment variables.
        """
        ...

    def get_ip(self) -> List[Dict[str, Any]]:
        """
        Returns a list of IP addresses.
        """
        ...

    def get_os(self) -> Dict[str, Any]:
        """
        Returns operating system information.
        """
        ...

    def get_pid(self) -> int:
        """
        Returns the process ID of the current Eldritch script.
        """
        ...

    def get_reg(self, reghive: str, regpath: str) -> Dict[str, Any]:
        """
        Retrieves a registry key value.
        """
        ...

    def get_user(self) -> Dict[str, Any]:
        """
        Returns current user information.
        """
        ...

    def hostname(self) -> str:
        """
        Returns the hostname of the system.
        """
        ...

    def is_bsd(self) -> bool:
        """
        Checks if the operating system is BSD.
        """
        ...

    def is_linux(self) -> bool:
        """
        Checks if the operating system is Linux.
        """
        ...

    def is_macos(self) -> bool:
        """
        Checks if the operating system is macOS.
        """
        ...

    def is_windows(self) -> bool:
        """
        Checks if the operating system is Windows.
        """
        ...

    def shell(self, cmd: str) -> Dict[str, Any]:
        """
        Executes a command in the system shell.
        Returns a dictionary with 'stdout', 'stderr', and 'status'.
        """
        ...

    def write_reg_hex(self, reghive: str, regpath: str, regname: str, regtype: str, regvalue: str) -> bool:
        """
        Writes a hexadecimal value to the registry.
        """
        ...

    def write_reg_int(self, reghive: str, regpath: str, regname: str, regtype: str, regvalue: int) -> bool:
        """
        Writes an integer value to the registry.
        """
        ...

    def write_reg_str(self, reghive: str, regpath: str, regname: str, regtype: str, regvalue: str) -> bool:
        """
        Writes a string value to the registry.
        """
        ...


sys: Sys = ...  # Global instance of the Sys module


class Time:
    """
    General functions for obtaining and formatting time, also add delays into code.
    """

    def format_to_epoch(self, input: str, format: str) -> int:
        """
        Formats a time string to epoch timestamp.
        """
        ...

    def format_to_readable(self, input: int, format: str) -> str:
        """
        Formats an epoch timestamp to a readable string.
        """
        ...

    def now(self) -> int:
        """
        Returns the current epoch timestamp.
        """
        ...

    def sleep(self, secs: float) -> None:
        """
        Pauses execution for a specified number of seconds.
        """
        ...


time: Time = ...  # Global instance of the Time module


# --- Global Starlark Built-in Functions (Explicitly mentioned as supported) ---
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
