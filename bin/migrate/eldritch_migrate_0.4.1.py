#!/usr/bin/env python3
"""
Migration script for Eldritch v0.4.1 standard library function renames.

Usage:
    python3 bin/migrate/eldritch_migrate_0.4.1.py <path> [<path> ...] [--dry-run]

Given one or more `.eldritch` files or directories containing `.eldritch` files,
this script updates deprecated function calls to their new v0.4.1 library names.
"""

import argparse
import os
import re
import sys
from pathlib import Path
from typing import List, Tuple

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
    # file -> gzip, process, template
    ("file.compress", "gzip.compress_file"),
    ("file.decompress", "gzip.decompress_file"),
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
    ("pivot.port_scan", "scan.port"),
    ("pivot.smb_exec", "smb.exec"),
    ("pivot.ssh_copy", "ssh.copy"),
    ("pivot.ssh_deploy", "ssh.deploy"),
    ("pivot.ssh_exec", "ssh.exec"),
    # process / net -> net.conns
    ("process.netstat", "net.conns"),
    ("net.sockets", "net.conns"),
    # sys -> windows, process, net, sys, user, file
    ("sys.dll_inject", "windows.dll_inject"),
    ("sys.dll_reflect", "windows.dll_reflect"),
    ("sys.get_env", "process.env"),
    ("sys.get_ip", "net.interfaces"),
    ("sys.get_os", "sys.info"),
    ("sys.get_pid", "process.pid"),
    ("sys.get_reg", "windows.read_reg"),
    ("sys.get_user", "user.info"),
    ("sys.impersonate", "windows.impersonate"),
    ("sys.list_users", "user.list"),
    ("sys.tokens", "windows.tokens"),
    ("sys.use_token", "windows.use_token"),
    ("sys.whereis", "file.which"),
    ("sys.write_reg", "windows.write_reg"),
]


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


BYTES_RETURNING_PREFIXES = (
    "bytes(",
    "file.read(",
    "file.read_binary(",
    "assets.read(",
    "assets.read_binary(",
    "gzip.compress(",
    "gzip.decompress(",
    "aes.encrypt(",
    "aes.decrypt(",
    "crypto.aes_encrypt(",
    "crypto.aes_decrypt(",
    "random.bytes(",
)


def is_bytes_expr(expr: str, known_bytes_vars: set) -> bool:
    """Return True if `expr` is already guaranteed to evaluate to `bytes`."""
    s = expr.strip()
    if not s:
        return False
    # Bytes literal: b"...", b'...', rb"...", br"...", etc.
    if re.match(r"^(?:b|rb|br)['\"]", s, re.IGNORECASE):
        return True
    for prefix in BYTES_RETURNING_PREFIXES:
        if s.startswith(prefix) and find_matching_delim(s, len(prefix) - 1, "(", ")") == len(s) - 1:
            return True
    # Any expression ending with `.encode(...)`
    m = re.search(r"\.encode\s*\([^()]*\)\s*$", s)
    if m:
        return True
    if s in known_bytes_vars:
        return True
    return False


def collect_known_bytes_vars(content: str) -> set:
    """Find variable names assigned from known bytes expressions in the script."""
    known: set = set()
    assign_re = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+)$", re.MULTILINE)
    for m in assign_re.finditer(content):
        var_name = m.group(1)
        rhs = m.group(2).strip()
        # Strip trailing comment if any
        if "#" in rhs:
            rhs = rhs.split("#", 1)[0].strip()
        if (
            re.match(r"^(?:b|rb|br)['\"]", rhs, re.IGNORECASE)
            or rhs.startswith("bytes(")
            or rhs.startswith("file.read_binary(")
            or rhs.startswith("assets.read_binary(")
            or rhs.startswith("gzip.compress(")
            or rhs.startswith("gzip.decompress(")
            or rhs.startswith("aes.encrypt(")
            or rhs.startswith("aes.decrypt(")
            or rhs.startswith("crypto.aes_encrypt(")
            or rhs.startswith("crypto.aes_decrypt(")
            or rhs.startswith("random.bytes(")
            or re.search(r"\.encode\s*\([^()]*\)\s*$", rhs)
        ):
            known.add(var_name)
    return known


def is_inside_bytes_or_str_cast(prefix: str, suffix: str, bytes_return_Check: bool = False) -> bool:
    """Check if the current call at the boundary of `prefix` and `suffix` needs no `str(...)` cast."""
    stripped_suffix = suffix.lstrip()
    # Followed immediately by `.decode(`
    if re.match(r"^\.decode\s*\(", stripped_suffix):
        return True
    # Preceded immediately by `str(`, `bytes(`, `gzip.compress(`, `gzip.decompress(`, `hash.md5(`, `hash.sha1(`, `hash.sha256(`, `crypto.md5(`, `crypto.sha1(`, `crypto.sha256(`
    if re.search(
        r"\b(?:str|bytes|gzip\.compress|gzip\.decompress|hash\.md5|hash\.sha1|hash\.sha256|crypto\.md5|crypto\.sha1|crypto\.sha256)\s*\(\s*$",
        prefix,
    ):
        return True
    # 2nd arg of aes.encrypt / aes.decrypt / file.write / file.append
    if re.search(
        r"\b(?:aes\.encrypt|aes\.decrypt|crypto\.aes_encrypt|crypto\.aes_decrypt|file\.write|file\.append|file\.write_binary|file\.append_binary)\s*\([^,()]+,\s*$",
        prefix,
    ):
        return True
    if bytes_return_Check and re.search(r"\breturn\s+$", prefix):
        return True
    return False


def has_bytes_return_context(content: str) -> bool:
    """Check if a function in `content` returns bytes for `ssh.deploy` or explicitly documents returning bytes."""
    if "as bytes" in content or "into bytes" in content:
        return True
    return False


def migrate_binary_and_io_calls(
    content: str, known_bytes_vars: set = None, bytes_return_ctx: bool = None
) -> str:
    """
    Remove `*_binary` methods and update `file.read`, `assets.read`, `file.write`, `file.append`
    to work with `bytes`, casting with `str(...)` or `bytes(...)` only when necessary.
    """
    if known_bytes_vars is None:
        known_bytes_vars = collect_known_bytes_vars(content)
    if bytes_return_ctx is None:
        bytes_return_ctx = has_bytes_return_context(content)

    # Simplify `file.read(x).encode()` / `assets.read(x).encode()` (default utf8) to `file.read(x)` / `assets.read(x)`
    # before wrapping `file.read` / `assets.read` in `str(...)`.
    pattern = re.compile(
        r"\b(file\.read_binary|assets\.read_binary|file\.write_binary|file\.append_binary|file\.read|assets\.read|file\.write|file\.append)\s*\("
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

        if fn_name in ("file.read_binary", "assets.read_binary"):
            new_fn = "file.read" if fn_name == "file.read_binary" else "assets.read"
            migrated_args = [migrate_binary_and_io_calls(a, known_bytes_vars) for a in args]
            out.append(f"{new_fn}({','.join(migrated_args)})")
            idx = close_paren_idx + 1
            continue

        if fn_name in ("file.write_binary", "file.append_binary"):
            new_fn = "file.write" if fn_name == "file.write_binary" else "file.append"
            migrated_args = [migrate_binary_and_io_calls(a, known_bytes_vars) for a in args]
            out.append(f"{new_fn}({','.join(migrated_args)})")
            idx = close_paren_idx + 1
            continue

        if fn_name in ("file.read", "assets.read"):
            migrated_args = [migrate_binary_and_io_calls(a, known_bytes_vars) for a in args]
            call_str = f"{fn_name}({','.join(migrated_args)})"
            # Check if followed by `.encode()` or `.encode("utf8")` / `.encode("utf-8")`
            enc_match = re.match(
                r"^\s*\.encode\s*\(\s*(?:['\"]utf-?8['\"]\s*)?\)",
                suffix_after,
                re.IGNORECASE,
            )
            if enc_match:
                out.append(call_str)
                idx = close_paren_idx + 1 + enc_match.end()
                continue
            if is_inside_bytes_or_str_cast(prefix_so_far, suffix_after, bytes_return_ctx):
                out.append(call_str)
            else:
                out.append(f"str({call_str})")
            idx = close_paren_idx + 1
            continue

        if fn_name in ("file.write", "file.append"):
            if len(args) >= 2:
                arg0 = migrate_binary_and_io_calls(args[0], known_bytes_vars)
                raw_arg1 = args[1]
                # If raw_arg1 is a direct `file.read(...)` or `assets.read(...)`, it already returns bytes!
                stripped_arg1 = raw_arg1.strip()
                if re.match(r"^(?:file|assets)\.read(?:_binary)?\s*\(", stripped_arg1):
                    open_p = stripped_arg1.find("(")
                    if find_matching_delim(stripped_arg1, open_p, "(", ")") == len(stripped_arg1) - 1:
                        inner_read = stripped_arg1[open_p + 1 : -1]
                        inner_read_migrated = migrate_binary_and_io_calls(inner_read, known_bytes_vars)
                        read_fn = "file.read" if stripped_arg1.startswith("file.") else "assets.read"
                        leading_ws = raw_arg1[: len(raw_arg1) - len(raw_arg1.lstrip())]
                        trailing_ws = raw_arg1[len(raw_arg1.rstrip()) :]
                        arg1 = f"{leading_ws}{read_fn}({inner_read_migrated}){trailing_ws}"
                    else:
                        arg1 = migrate_binary_and_io_calls(raw_arg1, known_bytes_vars)
                else:
                    migrated_arg1 = migrate_binary_and_io_calls(raw_arg1, known_bytes_vars)
                    if is_bytes_expr(raw_arg1, known_bytes_vars) or is_bytes_expr(migrated_arg1, known_bytes_vars):
                        arg1 = migrated_arg1
                    else:
                        leading_ws = migrated_arg1[: len(migrated_arg1) - len(migrated_arg1.lstrip())]
                        trailing_ws = migrated_arg1[len(migrated_arg1.rstrip()) :]
                        arg1 = f"{leading_ws}bytes({migrated_arg1.strip()}){trailing_ws}"
                rest_args = [migrate_binary_and_io_calls(a, known_bytes_vars) for a in args[2:]]
                all_args = [arg0, arg1] + rest_args
                out.append(f"{fn_name}({','.join(all_args)})")
            else:
                migrated_args = [migrate_binary_and_io_calls(a, known_bytes_vars) for a in args]
                out.append(f"{fn_name}({','.join(migrated_args)})")
            idx = close_paren_idx + 1
            continue

    result = "".join(out)
    # Replace any remaining bare references (e.g. in comments or function references)
    for old_bin, new_bin in (
        ("file.read_binary", "file.read"),
        ("assets.read_binary", "assets.read"),
        ("file.write_binary", "file.write"),
        ("file.append_binary", "file.append"),
    ):
        result = re.sub(r"\b" + re.escape(old_bin) + r"\b", new_bin, result)
    return result


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
    # Sort by old name length descending to avoid partial prefix matches
    for old_name, new_name in sorted(RENAMES, key=lambda x: len(x[0]), reverse=True):
        pattern = re.compile(r"\b" + re.escape(old_name) + r"\b")
        updated = pattern.sub(new_name, updated)
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
        original = file_path.read_text(encoding="utf-8")
        migrated = migrate_content(original)
        if migrated != original:
            changed_count += 1
            if args.dry_run:
                print(f"[DRY-RUN] Would update {file_path}")
            else:
                file_path.write_text(migrated, encoding="utf-8")
                print(f"Updated {file_path}")

    print(f"Migration complete ({changed_count} file(s) {'would be ' if args.dry_run else ''}updated).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
