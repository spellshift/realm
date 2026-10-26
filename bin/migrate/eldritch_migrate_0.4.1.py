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
    # crypto -> aes, base64, utf16le, json, hash
    ("crypto.aes_decrypt_file", "aes.decrypt_file"),
    ("crypto.aes_decrypt", "aes.decrypt"),
    ("crypto.aes_encrypt_file", "aes.encrypt_file"),
    ("crypto.aes_encrypt", "aes.encrypt"),
    ("crypto.decode_b64", "base64.decode"),
    ("crypto.decode_utf16le", "utf16le.decode"),
    ("crypto.encode_b64", "base64.encode"),
    ("crypto.encode_utf16le", "utf16le.encode"),
    ("crypto.from_json", "json.decode"),
    ("crypto.hash_file", "hash.file"),
    ("crypto.is_json", "json.is_valid"),
    ("crypto.md5", "hash.md5"),
    ("crypto.sha1", "hash.sha1"),
    ("crypto.sha256", "hash.sha256"),
    ("crypto.to_json", "json.encode"),
    # file -> gzip
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
    # process -> net
    ("process.netstat", "net.sockets"),
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


def split_top_level_args(arg_str: str) -> List[str]:
    """Split a function call's argument string on top-level commas."""
    args: List[str] = []
    current: List[str] = []
    depth = 0
    in_quote = None
    escape = False

    for ch in arg_str:
        if escape:
            current.append(ch)
            escape = False
            continue
        if ch == "\\":
            current.append(ch)
            escape = True
            continue
        if in_quote:
            current.append(ch)
            if ch == in_quote:
                in_quote = None
            continue
        if ch in ("'", '"'):
            in_quote = ch
            current.append(ch)
            continue
        if ch in ("(", "[", "{"):
            depth += 1
            current.append(ch)
            continue
        if ch in (")", "]", "}"):
            depth = max(0, depth - 1)
            current.append(ch)
            continue
        if ch == "," and depth == 0:
            args.append("".join(current))
            current = []
            continue
        current.append(ch)

    if current or arg_str.strip():
        args.append("".join(current))
    return args


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
