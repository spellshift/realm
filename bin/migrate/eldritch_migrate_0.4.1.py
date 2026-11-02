#!/usr/bin/env python3
"""
Migration script for Eldritch v0.4.1 standard library function renames.

Usage:
    python3 bin/migrate/eldritch_migrate_0.4.1.py <path> [<path> ...] [--dry-run]

Given one or more `.eldritch` files or directories containing `.eldritch` files,
this script updates deprecated function calls to their new v0.4.1 library names.
"""

import argparse
import ast
import os
import re
import sys
import warnings
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

RENAMES: List[Tuple[str, str]] = [
    # crypto -> aes, base64, json, hash
    ("crypto.aes_decrypt_file", "aes.decrypt_file"),
    ("crypto.aes_decrypt", "aes.decrypt"),
    ("crypto.aes_encrypt_file", "aes.encrypt_file"),
    ("crypto.aes_encrypt", "aes.encrypt"),
    ("crypto.decode_b64", "base64.decode"),
    ("crypto.encode_b64", "base64.encode"),
    ("crypto.from_json", "json.decode"),
    ("crypto.hash_file", "hash.file"),
    ("crypto.is_json", "json.is_valid"),
    ("crypto.md5", "hash.md5"),
    ("crypto.sha1", "hash.sha1"),
    ("crypto.sha256", "hash.sha256"),
    ("crypto.to_json", "json.encode"),
    # assets / file -> gzip, process, template, file
    ("assets.read_bytes", "assets.read"),
    ("file.compress", "gzip.compress_file"),
    ("file.decompress", "gzip.decompress_file"),
    ("file.exist", "file.exists"),
    ("file.moveto", "file.move"),
    ("file.temp_file", "file.tmp"),
    ("file.pwd", "process.cwd"),
    ("file.template", "template.render_file"),
    ("file.template_str", "template.render"),
    # time and agent accessor names
    ("time.format_to_epoch", "time.to_epoch"),
    ("time.format_to_readable", "time.format"),
    ("agent.get_config", "agent.config"),
    ("agent.get_transport", "agent.transport"),
    ("agent.get_callback_interval", "agent.callback_interval"),
    # pivot -> scan, portal, smb, ssh
    ("pivot.arp_scan", "scan.arp"),
    ("pivot.create_portal", "portal.create"),
    ("pivot.port_scan", "scan.ports"),
    ("scan.port", "scan.ports"),
    ("pivot.smb_exec", "smb.exec"),
    ("pivot.ssh_copy", "ssh.copy"),
    ("pivot.ssh_deploy", "ssh.deploy"),
    ("pivot.ssh_exec", "ssh.exec"),
    # process / net -> net.conns
    ("process.netstat", "net.conns"),
    ("net.sockets", "net.conns"),
    # sys / windows -> windows, process, net, sys, registry, user, file
    ("sys.dll_inject", "windows.dll_inject"),
    ("sys.dll_reflect", "windows.dll_reflect"),
    ("sys.get_env", "process.env"),
    ("sys.get_ip", "net.interfaces"),
    ("sys.get_os", "sys.info"),
    ("sys.get_pid", "process.pid"),
    ("sys.get_reg", "registry.read"),
    ("windows.read_reg", "registry.read"),
    ("sys.get_user", "user.info"),
    ("sys.impersonate", "windows.impersonate"),
    ("sys.list_users", "user.list"),
    ("sys.tokens", "windows.tokens"),
    ("sys.use_token", "windows.use_token"),
    ("sys.whereis", "file.which"),
    ("sys.write_reg_str", "registry.write"),
    ("sys.write_reg_int", "registry.write"),
    ("sys.write_reg", "registry.write"),
    ("windows.write_reg_str", "registry.write"),
    ("windows.write_reg_int", "registry.write"),
    ("windows.write_reg", "registry.write"),
]

# All Eldritch v0.4.1 standard library modules and their known methods (including deprecated aliases).
STDLIB_METHODS: Dict[str, Set[str]] = {
    "aes": {"decrypt", "decrypt_file", "encrypt", "encrypt_file"},
    "agent": {
        "_terminate_this_process_clowntown",
        "callback_interval",
        "config",
        "get_callback_interval",
        "get_config",
        "get_transport",
        "list_tasks",
        "list_transports",
        "reset_transport",
        "set_callback_interval",
        "set_callback_uri",
        "stop_task",
        "transport",
    },
    "assets": {"copy", "list", "read", "read_binary", "read_bytes"},
    "base64": {"decode", "encode"},
    "chain": {"tcp"},
    "crypto": {
        "aes_decrypt",
        "aes_decrypt_file",
        "aes_encrypt",
        "aes_encrypt_file",
        "decode_b64",
        "decode_utf16le",
        "encode_b64",
        "encode_utf16le",
        "from_json",
        "hash_file",
        "is_json",
        "md5",
        "sha1",
        "sha256",
        "to_json",
    },
    "dns": {"list"},
    "file": {
        "append",
        "append_binary",
        "compress",
        "copy",
        "decompress",
        "exist",
        "exists",
        "find",
        "follow",
        "is_dir",
        "is_file",
        "list",
        "list_named_pipes",
        "list_recent",
        "mkdir",
        "move",
        "moveto",
        "parent_dir",
        "pwd",
        "read",
        "read_binary",
        "read_named_pipe",
        "remove",
        "replace",
        "replace_all",
        "temp_file",
        "template",
        "template_str",
        "timestomp",
        "tmp",
        "tmp_dir",
        "which",
        "write",
        "write_binary",
    },
    "gzip": {"compress", "compress_file", "decompress", "decompress_file"},
    "hash": {"file", "md5", "sha1", "sha256"},
    "http": {"download", "get", "post"},
    "json": {"decode", "encode", "is_valid"},
    "linux": {"uname"},
    "net": {"conns", "interfaces", "sockets", "tcp", "udp"},
    "pivot": {
        "arp_scan",
        "create_portal",
        "ncat",
        "port_scan",
        "smb_exec",
        "ssh_copy",
        "ssh_deploy",
        "ssh_exec",
    },
    "portal": {"create"},
    "process": {"cwd", "env", "info", "kill", "list", "name", "netstat", "pid", "signal"},
    "random": {"bool", "bytes", "int", "string", "uuid"},
    "regex": {"match", "match_all", "replace", "replace_all"},
    "registry": {"read", "write"},
    "report": {"file", "ntlm_hash", "process_list", "screenshot", "ssh_key", "user_password"},
    "scan": {"arp", "port", "ports"},
    "ssh": {"copy", "deploy", "exec"},
    "sys": {
        "dll_inject",
        "dll_reflect",
        "exec",
        "get_env",
        "get_ip",
        "get_os",
        "get_pid",
        "get_reg",
        "get_user",
        "hostname",
        "impersonate",
        "info",
        "is_bsd",
        "is_linux",
        "is_macos",
        "is_windows",
        "list_users",
        "shell",
        "tokens",
        "use_token",
        "whereis",
        "write_reg",
        "write_reg_int",
        "write_reg_str",
    },
    "template": {"render", "render_file"},
    "time": {"format", "format_to_epoch", "format_to_readable", "now", "sleep", "to_epoch"},
    "user": {"info", "list"},
    "windows": {
        "dll_inject",
        "dll_reflect",
        "impersonate",
        "read_reg",
        "tokens",
        "use_token",
        "write_reg",
        "write_reg_int",
        "write_reg_str",
    },
}

PREFERRED_SHADOW_RENAMES: Dict[str, List[str]] = {
    "user": ["cur_user", "target_user", "user_val"],
    "windows": ["is_windows", "win_os", "windows_val"],
    "linux": ["is_linux", "linux_os", "linux_val"],
    "file": ["target_file", "cur_file", "file_val"],
}


def find_matching_delim(content: str, open_idx: int, open_ch: str = "(", close_ch: str = ")") -> int:
    """Find matching closing delimiter, respecting single/triple strings and comments."""
    depth = 0
    i = open_idx
    n = len(content)

    while i < n:
        ch = content[i]
        if ch == "#":
            nl = content.find("\n", i)
            if nl == -1:
                return -1
            i = nl + 1
            continue
        if ch in ("'", '"'):
            if content[i : i + 3] == ch * 3:
                end_q = content.find(ch * 3, i + 3)
                if end_q == -1:
                    return -1
                i = end_q + 3
                continue
            i += 1
            while i < n:
                if content[i] == "\\":
                    i += 2
                    continue
                if content[i] == ch:
                    i += 1
                    break
                i += 1
            continue
        if ch == open_ch:
            depth += 1
        elif ch == close_ch:
            depth -= 1
            if depth == 0:
                return i
        i += 1

    return -1


def split_top_level_args(arg_str: str) -> List[str]:
    """Split a function call's argument string on top-level commas."""
    args: List[str] = []
    current: List[str] = []
    depth = 0
    i = 0
    n = len(arg_str)

    while i < n:
        ch = arg_str[i]
        if ch == "#":
            nl = arg_str.find("\n", i)
            if nl == -1:
                current.append(arg_str[i:])
                break
            current.append(arg_str[i : nl + 1])
            i = nl + 1
            continue
        if ch in ("'", '"'):
            if arg_str[i : i + 3] == ch * 3:
                end_q = arg_str.find(ch * 3, i + 3)
                if end_q == -1:
                    current.append(arg_str[i:])
                    break
                current.append(arg_str[i : end_q + 3])
                i = end_q + 3
                continue
            current.append(ch)
            i += 1
            while i < n:
                if arg_str[i] == "\\":
                    current.append(arg_str[i : i + 2])
                    i += 2
                    continue
                current.append(arg_str[i])
                if arg_str[i] == ch:
                    i += 1
                    break
                i += 1
            continue
        if ch in ("(", "[", "{"):
            depth += 1
            current.append(ch)
        elif ch in (")", "]", "}"):
            depth = max(0, depth - 1)
            current.append(ch)
        elif ch == "," and depth == 0:
            args.append("".join(current))
            current = []
        else:
            current.append(ch)
        i += 1

    if current or arg_str.strip():
        args.append("".join(current))
    return args


def is_simple_receiver(expr: str) -> bool:
    """Return True if expr can have `.encode()` / `.decode()` appended without parens."""
    s = expr.strip()
    if not s:
        return False
    # String / bytes literal
    if (s[0] in ("'", '"') and s[-1] == s[0]) or (
        len(s) >= 3 and s[:2].lower() in ("b'", 'b"', "r'", 'r"', "f'", 'f"') and s[-1] in ("'", '"')
    ):
        # Ensure it's a single literal and not `"a" + "b"`
        if s[0] in ("'", '"'):
            end_idx = find_matching_quote(s, 0)
            return end_idx == len(s)
        else:
            end_idx = find_matching_quote(s, 1)
            return end_idx == len(s)
    # Parenthesized expression
    if s[0] == "(" and find_matching_delim(s, 0, "(", ")") == len(s) - 1:
        return True
    # Identifier possibly followed by .attr, (...), [...]
    m = re.match(r"^[A-Za-z_][A-Za-z0-9_]*", s)
    if not m:
        return False
    i = m.end()
    while i < len(s):
        if s[i] == ".":
            m2 = re.match(r"^\.[A-Za-z_][A-Za-z0-9_]*", s[i:])
            if not m2:
                return False
            i += m2.end()
        elif s[i] == "(":
            close_idx = find_matching_delim(s, i, "(", ")")
            if close_idx == -1:
                return False
            i = close_idx + 1
        elif s[i] == "[":
            close_idx = find_matching_delim(s, i, "[", "]")
            if close_idx == -1:
                return False
            i = close_idx + 1
        else:
            return False
    return True


def find_matching_quote(s: str, quote_start: int) -> int:
    ch = s[quote_start]
    if s[quote_start : quote_start + 3] == ch * 3:
        end_q = s.find(ch * 3, quote_start + 3)
        return len(s) if end_q == -1 else end_q + 3
    i = quote_start + 1
    while i < len(s):
        if s[i] == "\\":
            i += 2
            continue
        if s[i] == ch:
            return i + 1
        i += 1
    return -1


def wrap_receiver(expr: str) -> str:
    s = expr.strip()
    return s if is_simple_receiver(s) else f"({s})"


def migrate_encoding_calls(content: str) -> str:
    """
    Rewrite UTF-8 and UTF-16LE library calls to `.encode(...)` and `.decode(...)` methods:
    - `crypto.encode_utf16le(x)` / `utf16le.encode(x)` -> `x.encode("utf16le")`
    - `crypto.decode_utf16le(x)` / `utf16le.decode(x)` -> `x.decode("utf16le")`
    - `utf8.encode(x)` -> `x.encode()`
    - `utf8.decode(x, ...)` -> `x.decode()`
    """
    pattern = re.compile(
        r"\b(crypto\.encode_utf16le|crypto\.decode_utf16le|utf16le\.encode|utf16le\.decode|utf8\.encode|utf8\.decode)\s*\("
    )

    changed = True
    while changed:
        changed = False
        idx = 0
        out: List[str] = []
        while True:
            match = pattern.search(content, idx)
            if not match:
                out.append(content[idx:])
                break
            out.append(content[idx : match.start()])
            fn_name = match.group(1)
            open_paren_idx = match.end() - 1
            close_paren_idx = find_matching_delim(content, open_paren_idx, "(", ")")
            if close_paren_idx == -1:
                out.append(content[match.start() : match.end()])
                idx = match.end()
                continue

            inner = content[open_paren_idx + 1 : close_paren_idx]
            args = split_top_level_args(inner)
            if not args:
                out.append(content[match.start() : close_paren_idx + 1])
                idx = close_paren_idx + 1
                continue

            first_arg = migrate_encoding_calls(args[0])
            recv = wrap_receiver(first_arg)
            if fn_name in ("crypto.encode_utf16le", "utf16le.encode"):
                out.append(f'{recv}.encode("utf16le")')
            elif fn_name in ("crypto.decode_utf16le", "utf16le.decode"):
                out.append(f'{recv}.decode("utf16le")')
            elif fn_name == "utf8.encode":
                out.append(f"{recv}.encode()")
            elif fn_name == "utf8.decode":
                out.append(f"{recv}.decode()")

            idx = close_paren_idx + 1
            changed = True
        content = "".join(out)

    return content


STRING_METHODS_RE = re.compile(
    r"^\s*\.\s*(?:split|splitlines|strip|lstrip|rstrip|lower|upper|capitalize|title|"
    r"startswith|endswith|removeprefix|removesuffix|find|index|rfind|rindex|count|"
    r"replace|join|format|partition|rpartition|rsplit|codepoints|elems|"
    r"isalnum|isalpha|isdigit|islower|isspace|istitle|isupper)\s*\("
)


def _is_bytes_expr(expr: str) -> bool:
    s = expr.strip()
    if not s:
        return False
    if re.match(r"^(?:bytes\s*\(|b['\"]|rb['\"]|br['\"])", s, re.IGNORECASE):
        return True
    if re.search(r"\.encode\s*\(\s*(?:['\"][^'\"]*['\"]\s*)?\)\s*$", s):
        return True
    if re.match(r"^(?:base64\.decode|aes\.decrypt|gzip\.decompress|crypto\.decode_b64|crypto\.aes_decrypt)\s*\(", s):
        return True
    if re.match(r"^\d+$", s):
        return True
    return False


def collect_vars_needing_str_cast(content: str) -> set:
    """
    Find variables assigned from `file.read(...)` or `assets.read(...)` (not `*_binary`/`*_bytes`)
    that are subsequently used with string-only methods, `+` string concatenation,
    `in` / `not in` substring checks, or string equality comparisons.
    """
    candidates: set = set()
    assign_re = re.compile(
        r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?:file|assets)\.read\s*\(",
        re.MULTILINE,
    )
    for m in assign_re.finditer(content):
        candidates.add(m.group(1))

    needs_str: set = set()
    for var in candidates:
        esc = re.escape(var)
        # If `bytes(...) in var` or `b"..." in var` is explicitly used on this variable,
        # the variable is intentionally used as `Bytes`.
        if re.search(rf"(?:\bbytes\s*\([^)]*\)|\bb['\"][^'\"]*['\"])\s+(?:not\s+)?in\s+{esc}\b", content):
            continue
        # Check for `var.<str_method>(`
        if re.search(
            rf"\b{esc}\s*\.\s*(?:split|splitlines|strip|lstrip|rstrip|lower|upper|capitalize|title|"
            rf"startswith|endswith|removeprefix|removesuffix|find|index|rfind|rindex|count|"
            rf"replace|join|format|partition|rpartition|rsplit|codepoints|elems|"
            rf"isalnum|isalpha|isdigit|islower|isspace|istitle|isupper)\s*\(",
            content,
        ):
            needs_str.add(var)
            continue
        # Check for string concatenation or comparison or `in`/`not in` with string literals
        if (
            re.search(rf"\b{esc}\s*(?:\+|==|!=)\s*(?:['\"]|f['\"]|r['\"])", content)
            or re.search(rf"(?:['\"]|f['\"]|r['\"])\s*(?:\+|==|!=|(?:not\s+)?\bin\b)\s*{esc}\b", content)
        ):
            needs_str.add(var)
            continue
        # Check for `lhs in var` / `lhs not in var` where `lhs` is an identifier/expression (not `bytes(...)` or `b"..."`)
        in_matches = re.findall(rf"([^\n:;,#]+?)\s+(?:not\s+)?in\s+{esc}\b", content)
        for lhs in in_matches:
            # Strip leading keywords like `if`, `elif`, `while`, `and`, `or`, `not`, `return`
            lhs_clean = re.sub(r"^\s*(?:if|elif|while|and|or|not|return|assert)\b\s*", "", lhs).strip()
            if lhs_clean and not _is_bytes_expr(lhs_clean):
                needs_str.add(var)
                break
        if var in needs_str:
            continue
        # Check for `var + rhs` or `lhs + var` where the other side is a variable/expression (not `bytes(...)` or `b"..."`)
        plus_rhs = re.findall(rf"\b{esc}\s*\+\s*([^\n:;,)\]]+)", content)
        for rhs in plus_rhs:
            if rhs.strip() and not _is_bytes_expr(rhs):
                needs_str.add(var)
                break
        if var in needs_str:
            continue
        plus_lhs = re.findall(rf"([^\n:;,(=\[]+?)\s*\+\s*{esc}\b", content)
        for lhs in plus_lhs:
            lhs_clean = re.sub(r"^\s*(?:return|print|eprint)\b\s*", "", lhs).strip()
            if lhs_clean and not _is_bytes_expr(lhs_clean):
                needs_str.add(var)
                break
    return needs_str


def collect_funcs_returning_str_read(content: str) -> Set[str]:
    """
    Find functions that have `return file.read(...)` or `return assets.read(...)`
    AND either also return a string literal (e.g. `return ""`) or are called with string methods.
    """
    funcs_needing_str: Set[str] = set()
    try:
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", SyntaxWarning)
            tree = ast.parse(content)
    except SyntaxError:
        return funcs_needing_str

    candidate_funcs: Set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef):
            has_read_return = False
            has_str_literal_return = False
            for sub in ast.walk(node):
                if isinstance(sub, ast.Return) and sub.value is not None:
                    val = sub.value
                    if (
                        isinstance(val, ast.Call)
                        and isinstance(val.func, ast.Attribute)
                        and isinstance(val.func.value, ast.Name)
                        and val.func.value.id in ("file", "assets")
                        and val.func.attr == "read"
                    ):
                        has_read_return = True
                    elif isinstance(val, ast.Constant) and isinstance(val.value, str):
                        has_str_literal_return = True
            if has_read_return:
                if has_str_literal_return:
                    funcs_needing_str.add(node.name)
                else:
                    candidate_funcs.add(node.name)

    for fn in candidate_funcs:
        esc = re.escape(fn)
        if re.search(
            rf"\b{esc}\s*\([^)]*\)\s*\.\s*(?:split|splitlines|strip|lstrip|rstrip|lower|upper|"
            rf"startswith|endswith|replace|find|count)\s*\(",
            content,
        ):
            funcs_needing_str.add(fn)

    return funcs_needing_str


def _enclosing_func_name(prefix: str) -> Optional[str]:
    """Return the name of the function immediately enclosing the end of `prefix`, if any."""
    lines = prefix.splitlines()
    if not lines:
        return None
    last_line = lines[-1]
    cur_indent = len(last_line) - len(last_line.lstrip(" \t"))
    for i in range(len(lines) - 1, -1, -1):
        line = lines[i]
        stripped = line.lstrip(" \t")
        if not stripped or stripped.startswith("#"):
            continue
        indent = len(line) - len(stripped)
        if indent < cur_indent:
            m = re.match(r"^def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", stripped)
            if m:
                return m.group(1)
            cur_indent = indent
    return None


def needs_str_cast_at_callsite(
    prefix: str,
    suffix: str,
    str_vars: set,
    str_return_funcs: Optional[Set[str]] = None,
) -> bool:
    """
    Return True only when a `file.read(...)` or `assets.read(...)` call requires `str(...)`
    because it is used directly with a string-only method, operator, or variable.
    """
    # Already wrapped in `str(` or `bytes(` or followed by `.decode(`
    if re.search(r"\b(?:str|bytes)\s*\(\s*$", prefix):
        return False
    if re.match(r"^\s*\.decode\s*\(", suffix):
        return False
    # Followed directly by a string method (e.g. `file.read(p).splitlines()`)
    if STRING_METHODS_RE.match(suffix):
        return True
    # Used in string concatenation, comparison, or `in`/`not in` with a string literal
    if re.match(r"^\s*(?:\+|==|!=)\s*(?:['\"]|f['\"]|r['\"])", suffix):
        return True
    if re.search(r"(?:['\"]|f['\"]|r['\"])\s*(?:\+|==|!=|(?:not\s+)?\bin\b)\s*$", prefix):
        return True
    # Used on the RHS of `<expr> in file.read(...)` / `<expr> not in file.read(...)` where `<expr>` is not bytes
    m_in_prefix = re.search(r"([^\n:;,#]+?)\s+(?:not\s+)?\bin\s*$", prefix)
    if m_in_prefix:
        lhs_clean = re.sub(r"^\s*(?:if|elif|while|and|or|not|return|assert)\b\s*", "", m_in_prefix.group(1)).strip()
        if lhs_clean and not _is_bytes_expr(lhs_clean):
            return True
    # Assigned to a variable that later uses string-only methods/operators
    m_assign = re.search(r"(?:^|\n)\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*$", prefix)
    if m_assign and m_assign.group(1) in str_vars:
        return True
    # Returned from a helper function that also returns a string literal (e.g. `safe_read`)
    if str_return_funcs and re.search(r"(?:^|\n)\s*return\s+$", prefix):
        fn_name = _enclosing_func_name(prefix)
        if fn_name and fn_name in str_return_funcs:
            return True
    return False


def migrate_binary_and_io_calls(
    content: str,
    str_vars: set = None,
    str_return_funcs: Optional[Set[str]] = None,
) -> str:
    """
    Remove `*_binary`/`*_bytes` methods and update `file.read`, `assets.read`, `file.write`, `file.append`.
    Since methods and builtins (such as `print`, `file.write`, `file.append`, etc.) accept
    either `bytes` or `str`, `str(...)` casts are added around `file.read`/`assets.read`
    only when string-specific methods or operators are used on the result.
    """
    if str_vars is None:
        str_vars = collect_vars_needing_str_cast(content)
    if str_return_funcs is None:
        str_return_funcs = collect_funcs_returning_str_read(content)

    pattern = re.compile(
        r"\b(file\.read_binary|assets\.read_binary|assets\.read_bytes|file\.write_binary|file\.append_binary|file\.read|assets\.read)\s*\("
    )

    idx = 0
    out: List[str] = []
    while True:
        match = pattern.search(content, idx)
        if not match:
            out.append(content[idx:])
            break

        out.append(content[idx : match.start()])
        fn_name = match.group(1)
        open_paren_idx = match.end() - 1
        close_paren_idx = find_matching_delim(content, open_paren_idx, "(", ")")
        if close_paren_idx == -1:
            out.append(content[match.start() : match.end()])
            idx = match.end()
            continue

        inner = content[open_paren_idx + 1 : close_paren_idx]
        args = split_top_level_args(inner)
        prefix_so_far = "".join(out)
        suffix_after = content[close_paren_idx + 1 :]

        if fn_name in ("file.read_binary", "assets.read_binary", "assets.read_bytes"):
            new_fn = "file.read" if fn_name == "file.read_binary" else "assets.read"
            migrated_args = [migrate_binary_and_io_calls(a, str_vars, str_return_funcs) for a in args]
            out.append(f"{new_fn}({','.join(migrated_args)})")
            idx = close_paren_idx + 1
            continue

        if fn_name in ("file.write_binary", "file.append_binary"):
            new_fn = "file.write" if fn_name == "file.write_binary" else "file.append"
            migrated_args = [migrate_binary_and_io_calls(a, str_vars, str_return_funcs) for a in args]
            out.append(f"{new_fn}({','.join(migrated_args)})")
            idx = close_paren_idx + 1
            continue

        if fn_name in ("file.read", "assets.read"):
            migrated_args = [migrate_binary_and_io_calls(a, str_vars, str_return_funcs) for a in args]
            call_str = f"{fn_name}({','.join(migrated_args)})"
            # Simplify `file.read(x).encode()` (default utf8) -> `file.read(x)`
            enc_match = re.match(
                r"^\s*\.encode\s*\(\s*(?:['\"]utf-?8['\"]\s*)?\)",
                suffix_after,
                re.IGNORECASE,
            )
            if enc_match:
                out.append(call_str)
                idx = close_paren_idx + 1 + enc_match.end()
                continue
            if needs_str_cast_at_callsite(prefix_so_far, suffix_after, str_vars, str_return_funcs):
                out.append(f"str({call_str})")
            else:
                out.append(call_str)
            idx = close_paren_idx + 1
            continue

    result = "".join(out)
    # Replace any remaining bare references (e.g. in comments or function references)
    for old_bin, new_bin in (
        ("file.read_binary", "file.read"),
        ("assets.read_binary", "assets.read"),
        ("assets.read_bytes", "assets.read"),
        ("file.write_binary", "file.write"),
        ("file.append_binary", "file.append"),
    ):
        result = re.sub(r"\b" + re.escape(old_bin) + r"\b", new_bin, result)
    return result


def _combine_registry_hive_and_path(hive_arg: str, path_arg: str) -> str:
    """Combine legacy (reghive, regpath) arguments into a single registry path argument."""
    lead_ws = hive_arg[: len(hive_arg) - len(hive_arg.lstrip())]
    h = hive_arg.strip()
    p = path_arg.strip()
    if (
        len(h) >= 2
        and len(p) >= 2
        and h[0] in ("'", '"')
        and h[-1] == h[0]
        and p[0] == h[0]
        and p[-1] == p[0]
        and find_matching_quote(h, 0) == len(h)
        and find_matching_quote(p, 0) == len(p)
    ):
        q = h[0]
        h_inner = h[1:-1].rstrip("\\")
        p_inner = p[1:-1].lstrip("\\")
        sep = "\\\\" if "\\\\" in p_inner or "\\\\" in h_inner or q in ('"', "'") else "\\"
        return f"{lead_ws}{q}{h_inner}{sep}{p_inner}{q}"
    return f'{lead_ws}{h} + "\\\\" + {p}'


def migrate_registry_calls(content: str) -> str:
    """
    Migrate legacy 2-argument `sys.get_reg(hive, path)` / `registry.read(hive, path)` calls to
    1-argument `registry.read(combined_path)`, and 5-argument `sys.write_reg*(hive, path, name, type, val)` /
    `registry.write(hive, path, name, type, val)` calls to 4-argument `registry.write(combined_path, name, type, val)`.
    """
    pattern = re.compile(
        r"\b(sys\.get_reg|windows\.read_reg|registry\.read|"
        r"sys\.write_reg_str|sys\.write_reg_int|sys\.write_reg|"
        r"windows\.write_reg_str|windows\.write_reg_int|windows\.write_reg|"
        r"registry\.write)\s*\("
    )
    idx = 0
    out: List[str] = []

    while True:
        match = pattern.search(content, idx)
        if not match:
            out.append(content[idx:])
            break

        out.append(content[idx : match.start()])
        fn_name = match.group(1)
        open_paren_idx = match.end() - 1
        close_paren_idx = find_matching_delim(content, open_paren_idx, "(", ")")
        if close_paren_idx == -1:
            out.append(content[match.start() : match.end()])
            idx = match.end()
            continue

        inner = content[open_paren_idx + 1 : close_paren_idx]
        raw_args = split_top_level_args(inner)
        # Filter out trailing empty argument caused by a trailing comma
        non_empty_args = [a for a in raw_args if a.strip()]
        has_trailing_comma = len(raw_args) > len(non_empty_args)
        trailing_ws = ("," + raw_args[-1]) if has_trailing_comma else ""

        is_read = fn_name in ("sys.get_reg", "windows.read_reg", "registry.read")
        if is_read:
            if len(non_empty_args) == 2:
                combined = _combine_registry_hive_and_path(non_empty_args[0], non_empty_args[1])
                out.append(f"registry.read({combined}{trailing_ws})")
            else:
                out.append(f"registry.read({inner})")
        else:
            if len(non_empty_args) == 5:
                combined = _combine_registry_hive_and_path(non_empty_args[0], non_empty_args[1])
                rest = ",".join(non_empty_args[2:])
                out.append(f"registry.write({combined},{rest}{trailing_ws})")
            else:
                out.append(f"registry.write({inner})")

        idx = close_paren_idx + 1

    return "".join(out)


def _is_stdlib_module_call(
    mod: str,
    attr: str,
    parent_node: ast.AST,
    grandparent_node: Optional[ast.AST],
) -> bool:
    """Return True if `mod.attr` is a call/reference to the Eldritch stdlib module `mod`."""
    methods = STDLIB_METHODS.get(mod)
    if not methods or attr not in methods:
        return False

    # Disambiguate the few method names that overlap with str/list methods
    if isinstance(grandparent_node, ast.Call) and grandparent_node.func is parent_node:
        num_args = len(grandparent_node.args) + len(grandparent_node.keywords)
        if mod in ("file", "regex") and attr == "replace":
            # str.replace(old, new) takes 2 args; file.replace / regex.replace take 3 args
            return num_args >= 3
        if mod == "file" and attr == "append":
            # list.append(item) takes 1 arg; file.append(path, content) takes 2 args
            return num_args >= 2
        if mod in ("base64", "json") and attr == "encode":
            # str.encode() takes 0 args or "utf8"/"utf16le"
            if num_args == 0:
                return False
            if (
                len(grandparent_node.args) == 1
                and isinstance(grandparent_node.args[0], ast.Constant)
                and isinstance(grandparent_node.args[0].value, str)
                and grandparent_node.args[0].value.lower().replace("-", "") in ("utf8", "utf16le")
            ):
                return False

    return True


def rename_shadowed_stdlib_vars(content: str) -> str:
    """
    Detect local/global variables, loop targets, and function parameters whose names
    collide with Eldritch v0.4.1 standard library modules (e.g. `user`, `windows`, `file`, `linux`)
    and rename them to safe identifiers (`cur_user`, `is_windows`, `target_file`, etc.)
    while preserving stdlib module calls (`user.info()`, `file.is_file(...)`, etc.).
    """
    try:
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", SyntaxWarning)
            tree = ast.parse(content)
    except SyntaxError:
        return content

    parent_map: Dict[ast.AST, ast.AST] = {}
    all_identifiers: Set[str] = set()
    shadowed_names: Set[str] = set()
    local_funcs_with_shadowed_param: Dict[str, Set[str]] = {}

    for parent in ast.walk(tree):
        for child in ast.iter_child_nodes(parent):
            parent_map[child] = parent
        if isinstance(parent, ast.Name):
            all_identifiers.add(parent.id)
            if isinstance(parent.ctx, (ast.Store, ast.Del)) and parent.id in STDLIB_METHODS:
                shadowed_names.add(parent.id)
        elif isinstance(parent, ast.arg):
            all_identifiers.add(parent.arg)
            if parent.arg in STDLIB_METHODS:
                shadowed_names.add(parent.arg)
        elif isinstance(parent, ast.FunctionDef):
            all_identifiers.add(parent.name)
            for a in parent.args.posonlyargs + parent.args.args + parent.args.kwonlyargs:
                if a.arg in STDLIB_METHODS:
                    local_funcs_with_shadowed_param.setdefault(parent.name, set()).add(a.arg)
            if parent.args.vararg and parent.args.vararg.arg in STDLIB_METHODS:
                local_funcs_with_shadowed_param.setdefault(parent.name, set()).add(parent.args.vararg.arg)
            if parent.args.kwarg and parent.args.kwarg.arg in STDLIB_METHODS:
                local_funcs_with_shadowed_param.setdefault(parent.name, set()).add(parent.args.kwarg.arg)

    if not shadowed_names:
        return content

    rename_map: Dict[str, str] = {}
    for mod in sorted(shadowed_names):
        candidates = list(PREFERRED_SHADOW_RENAMES.get(mod, [])) + [
            f"cur_{mod}",
            f"target_{mod}",
            f"{mod}_val",
            f"{mod}_var",
        ]
        for cand in candidates:
            if cand not in all_identifiers and cand not in STDLIB_METHODS:
                rename_map[mod] = cand
                all_identifiers.add(cand)
                break

    if not rename_map:
        return content

    # Collect UTF-8 byte spans `(lineno, start_col_byte, end_col_byte, old_name, new_name)`
    edits_by_line: Dict[int, List[Tuple[int, int, str, str]]] = {}

    for node in ast.walk(tree):
        if isinstance(node, ast.arg) and node.arg in rename_map:
            if node.lineno is not None and node.col_offset is not None:
                old_id = node.arg
                end_col = (
                    node.end_col_offset
                    if getattr(node, "end_col_offset", None) is not None
                    else node.col_offset + len(old_id.encode("utf-8"))
                )
                edits_by_line.setdefault(node.lineno, []).append(
                    (node.col_offset, end_col, old_id, rename_map[old_id])
                )
        elif isinstance(node, ast.Name) and node.id in rename_map:
            old_id = node.id
            parent = parent_map.get(node)
            grandparent = parent_map.get(parent) if parent is not None else None

            if isinstance(node.ctx, ast.Load):
                # Preserve `mod.method(...)` when it is a stdlib module call
                if isinstance(parent, ast.Attribute) and parent.value is node:
                    if _is_stdlib_module_call(old_id, parent.attr, parent, grandparent):
                        continue
                # Preserve `dir(mod)`
                if (
                    isinstance(parent, ast.Call)
                    and isinstance(parent.func, ast.Name)
                    and parent.func.id == "dir"
                    and node in parent.args
                ):
                    continue

            if node.lineno is not None and node.col_offset is not None:
                end_col = (
                    node.end_col_offset
                    if getattr(node, "end_col_offset", None) is not None
                    else node.col_offset + len(old_id.encode("utf-8"))
                )
                edits_by_line.setdefault(node.lineno, []).append(
                    (node.col_offset, end_col, old_id, rename_map[old_id])
                )
        elif isinstance(node, ast.keyword) and node.arg in rename_map:
            parent = parent_map.get(node)
            if (
                isinstance(parent, ast.Call)
                and isinstance(parent.func, ast.Name)
                and parent.func.id in local_funcs_with_shadowed_param
                and node.arg in local_funcs_with_shadowed_param[parent.func.id]
                and getattr(node, "lineno", None) is not None
                and getattr(node, "col_offset", None) is not None
            ):
                old_id = node.arg
                end_col = node.col_offset + len(old_id.encode("utf-8"))
                edits_by_line.setdefault(node.lineno, []).append(
                    (node.col_offset, end_col, old_id, rename_map[old_id])
                )

    if not edits_by_line:
        return content

    lines = content.splitlines(keepends=True)
    for lineno, line_edits in edits_by_line.items():
        idx = lineno - 1
        if idx < 0 or idx >= len(lines):
            continue
        line_bytes = lines[idx].encode("utf-8")
        # Deduplicate and sort reverse by start_col so earlier offsets stay valid
        unique_edits = sorted(set(line_edits), key=lambda x: x[0], reverse=True)
        for start_col, end_col, old_id, new_id in unique_edits:
            old_bytes = old_id.encode("utf-8")
            if line_bytes[start_col:end_col] == old_bytes:
                line_bytes = (
                    line_bytes[:start_col] + new_id.encode("utf-8") + line_bytes[end_col:]
                )
        lines[idx] = line_bytes.decode("utf-8")

    return "".join(lines)


def migrate_ncat_calls(content: str) -> str:
    """
    Rewrite `pivot.ncat(...)` calls to `net.tcp(...)` or `net.udp(...)`.
    `pivot.ncat(address, port, data, protocol)` -> `net.tcp(address, port, data)` or `net.udp(address, port, data)`.
    """
    pattern = re.compile(r"\bpivot\.ncat\s*\(")
    idx = 0
    out: List[str] = []

    while True:
        match = pattern.search(content, idx)
        if not match:
            out.append(content[idx:])
            break

        out.append(content[idx : match.start()])
        open_paren_idx = match.end() - 1

        # Find matching closing parenthesis
        depth = 0
        in_quote = None
        escape = False
        close_paren_idx = -1

        for i in range(open_paren_idx, len(content)):
            ch = content[i]
            if escape:
                escape = False
                continue
            if ch == "\\":
                escape = True
                continue
            if in_quote:
                if ch == in_quote:
                    in_quote = None
                continue
            if ch in ("'", '"'):
                in_quote = ch
                continue
            if ch == "(":
                depth += 1
            elif ch == ")":
                depth -= 1
                if depth == 0:
                    close_paren_idx = i
                    break

        if close_paren_idx == -1:
            # Unbalanced parens; fallback to net.tcp
            out.append("net.tcp(")
            idx = match.end()
            continue

        inner = content[open_paren_idx + 1 : close_paren_idx]
        args = split_top_level_args(inner)

        target_fn = "net.tcp"
        kept_args: List[str] = []

        for i, arg in enumerate(args):
            stripped = arg.strip()
            # Check keyword argument protocol=...
            kw_match = re.match(r"^protocol\s*=\s*['\"](tcp|udp)['\"]$", stripped, re.IGNORECASE)
            if kw_match:
                if kw_match.group(1).lower() == "udp":
                    target_fn = "net.udp"
                continue
            # Check 4th positional argument
            if i == 3:
                pos_match = re.match(r"^['\"](tcp|udp)['\"]$", stripped, re.IGNORECASE)
                if pos_match:
                    if pos_match.group(1).lower() == "udp":
                        target_fn = "net.udp"
                    continue
            kept_args.append(arg)

        new_inner = ",".join(kept_args)
        out.append(f"{target_fn}({new_inner})")
        idx = close_paren_idx + 1

    # Replace any remaining bare references to pivot.ncat
    result = "".join(out)
    result = re.sub(r"\bpivot\.ncat\b", "net.tcp", result)
    return result


def migrate_content(content: str) -> str:
    updated = migrate_ncat_calls(content)
    updated = migrate_encoding_calls(updated)
    updated = migrate_binary_and_io_calls(updated)
    updated = migrate_registry_calls(updated)
    # Sort by old name length descending to avoid partial prefix matches
    for old_name, new_name in sorted(RENAMES, key=lambda x: len(x[0]), reverse=True):
        pattern = re.compile(r"\b" + re.escape(old_name) + r"\b")
        updated = pattern.sub(new_name, updated)
    updated = rename_shadowed_stdlib_vars(updated)
    return updated


def collect_files(paths: List[str]) -> List[Path]:
    files: List[Path] = []
    for raw in paths:
        p = Path(raw)
        if p.is_file():
            files.append(p)
        elif p.is_dir():
            for root, _, filenames in os.walk(p):
                for fn in sorted(filenames):
                    if fn.endswith(".eldritch"):
                        files.append(Path(root) / fn)
        else:
            print(f"Warning: path not found: {raw}", file=sys.stderr)
    return files


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Migrate Eldritch tomes to v0.4.1 standard library function names."
    )
    parser.add_argument(
        "paths",
        nargs="*",
        default=["tavern/tomes"],
        help="Files or directories to migrate (default: tavern/tomes)",
    )
    parser.add_argument(
        "-n",
        "--dry-run",
        action="store_true",
        help="Show which files would be modified without writing changes",
    )
    args = parser.parse_args()

    target_files = collect_files(args.paths)
    changed_count = 0

    for file_path in target_files:
        original = file_path.read_bytes().decode("utf-8")
        migrated = migrate_content(original)
        if migrated != original:
            changed_count += 1
            if args.dry_run:
                print(f"[DRY-RUN] Would update {file_path}")
            else:
                file_path.write_bytes(migrated.encode("utf-8"))
                print(f"Updated {file_path}")

    print(f"Migration complete ({changed_count} file(s) {'would be ' if args.dry_run else ''}updated).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
