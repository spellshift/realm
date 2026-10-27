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


STRING_METHODS_RE = re.compile(
    r"^\s*\.\s*(?:split|splitlines|strip|lstrip|rstrip|lower|upper|capitalize|title|"
    r"startswith|endswith|removeprefix|removesuffix|find|index|rfind|rindex|count|"
    r"replace|join|format|partition|rpartition|rsplit|codepoints|elems|"
    r"isalnum|isalpha|isdigit|islower|isspace|istitle|isupper)\s*\("
)


def collect_vars_needing_str_cast(content: str) -> set:
    """
    Find variables assigned from `file.read(...)` or `assets.read(...)` (not `*_binary`)
    that are subsequently used with string-only methods, `+` string concatenation,
    `in` substring checks, or string equality comparisons.
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
        # Check for string concatenation or comparison or `in` with string literals
        if (
            re.search(rf"\b{esc}\s*(?:\+|==|!=)\s*(?:['\"]|f['\"]|r['\"])", content)
            or re.search(rf"(?:['\"]|f['\"]|r['\"])\s*(?:\+|==|!=|\bin\b)\s*{esc}\b", content)
        ):
            needs_str.add(var)
    return needs_str


def needs_str_cast_at_callsite(prefix: str, suffix: str, str_vars: set) -> bool:
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
    # Used in string concatenation, comparison, or `in` with a string literal
    if re.match(r"^\s*(?:\+|==|!=)\s*(?:['\"]|f['\"]|r['\"])", suffix):
        return True
    if re.search(r"(?:['\"]|f['\"]|r['\"])\s*(?:\+|==|!=|\bin\b)\s*$", prefix):
        return True
    # Assigned to a variable that later uses string-only methods/operators
    m_assign = re.search(r"(?:^|\n)\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*$", prefix)
    if m_assign and m_assign.group(1) in str_vars:
        return True
    return False


def migrate_binary_and_io_calls(content: str, str_vars: set = None) -> str:
    """
    Remove `*_binary` methods and update `file.read`, `assets.read`, `file.write`, `file.append`.
    Since methods and builtins (such as `print`, `file.write`, `file.append`, etc.) accept
    either `bytes` or `str`, `str(...)` casts are added around `file.read`/`assets.read`
    only when string-specific methods or operators are used on the result.
    """
    if str_vars is None:
        str_vars = collect_vars_needing_str_cast(content)

    pattern = re.compile(
        r"\b(file\.read_binary|assets\.read_binary|file\.write_binary|file\.append_binary|file\.read|assets\.read)\s*\("
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
            migrated_args = [migrate_binary_and_io_calls(a, str_vars) for a in args]
            out.append(f"{new_fn}({','.join(migrated_args)})")
            idx = close_paren_idx + 1
            continue

        if fn_name in ("file.write_binary", "file.append_binary"):
            new_fn = "file.write" if fn_name == "file.write_binary" else "file.append"
            migrated_args = [migrate_binary_and_io_calls(a, str_vars) for a in args]
            out.append(f"{new_fn}({','.join(migrated_args)})")
            idx = close_paren_idx + 1
            continue

        if fn_name in ("file.read", "assets.read"):
            migrated_args = [migrate_binary_and_io_calls(a, str_vars) for a in args]
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
            if needs_str_cast_at_callsite(prefix_so_far, suffix_after, str_vars):
                if STRING_METHODS_RE.match(suffix_after):
                    out.append(f"str({call_str})")
                else:
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
