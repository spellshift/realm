import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("eldritch_migrate_0.4.1.py")
SPEC = importlib.util.spec_from_file_location("eldritch_migrate", SCRIPT)
migration = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(migration)


class AdditionalRenamesTest(unittest.TestCase):
    RENAMES = {
        "file.temp_file": "file.tmp",
        "time.format_to_epoch": "time.to_epoch",
        "time.format_to_readable": "time.format",
        "agent.get_config": "agent.config",
        "agent.get_transport": "agent.transport",
        "agent.get_callback_interval": "agent.callback_interval",
        "file.pwd": "process.cwd",
        "file.template": "template.render_file",
        "file.template_str": "template.render",
        "process.netstat": "net.conns",
        "net.sockets": "net.conns",
        "pivot.port_scan": "scan.ports",
        "scan.port": "scan.ports",
        "file.moveto": "file.move",
        "file.exist": "file.exists",
        "assets.read_bytes": "assets.read",
    }

    def test_calls_and_references(self):
        for old, new in self.RENAMES.items():
            with self.subTest(old=old):
                self.assertEqual(migration.migrate_content(f"{old}()"), f"{new}()")
                self.assertEqual(migration.migrate_content(f"fn = {old}"), f"fn = {new}")
                self.assertEqual(migration.migrate_content(f"{old}_suffix()"), f"{old}_suffix()")

    def test_encoding_methods_migration(self):
        cases = [
            ('crypto.encode_utf16le("hello")', '"hello".encode("utf16le")'),
            ('utf16le.encode(text)', 'text.encode("utf16le")'),
            ('utf16le.encode(a + b)', '(a + b).encode("utf16le")'),
            ('crypto.decode_utf16le(data)', 'data.decode("utf16le")'),
            ('utf16le.decode(b"h\\x00i\\x00")', 'b"h\\x00i\\x00".decode("utf16le")'),
            ('utf8.encode("hello")', '"hello".encode()'),
            ('utf8.decode(data)', 'data.decode()'),
            ('utf8.decode(data, True)', 'data.decode()'),
            (
                'utf8.decode(gzip.decompress(gzip.compress(utf8.encode("hi"))))',
                'gzip.decompress(gzip.compress("hi".encode())).decode()',
            ),
        ]
        for original, expected in cases:
            with self.subTest(original=original):
                migrated = migration.migrate_content(original)
                self.assertEqual(migrated, expected)
                self.assertEqual(migration.migrate_content(migrated), expected)

    def test_binary_removal_and_smart_casting(self):
        cases = [
            # *_binary removed without casting
            ('file.read_binary("/tmp/a")', 'file.read("/tmp/a")'),
            ('assets.read_binary("pkg/bin")', 'assets.read("pkg/bin")'),
            ('file.write_binary("/tmp/a", data)', 'file.write("/tmp/a", data)'),
            ('file.append_binary("/tmp/a", data)', 'file.append("/tmp/a", data)'),
            # Regular read only cast to str() when string methods/operators are used
            ('res = file.read(path)', 'res = file.read(path)'),
            ('res = file.read(path)\nlines = res.splitlines()', 'res = str(file.read(path))\nlines = res.splitlines()'),
            ('file.read(path).split("\\n")', 'str(file.read(path)).split("\\n")'),
            ('print(assets.read("a.sh"))', 'print(assets.read("a.sh"))'),
            # Already decoded or cast -> no extra cast
            ('file.read(path).decode()', 'file.read(path).decode()'),
            ('file.read(path).decode("utf16le")', 'file.read(path).decode("utf16le")'),
            ('str(file.read(path))', 'str(file.read(path))'),
            ('bytes(file.read(path))', 'bytes(file.read(path))'),
            ('file.read(path).encode()', 'file.read(path)'),
            ('utf8.encode(file.read(path))', 'file.read(path)'),
            # Used directly as bytes argument -> no extra cast
            ('gzip.compress(file.read(path))', 'gzip.compress(file.read(path))'),
            ('hash.sha256(file.read(path))', 'hash.sha256(file.read(path))'),
            ('file.write(dst, file.read(src))', 'file.write(dst, file.read(src))'),
            # Regular write/append accept str and Bytes interchangeably -> no extra cast
            ('file.write(path, content)', 'file.write(path, content)'),
            ('file.append(path, "line\\n")', 'file.append(path, "line\\n")'),
            ('file.write(path, b"raw")', 'file.write(path, b"raw")'),
            ('file.write(path, bytes(content))', 'file.write(path, bytes(content))'),
            ('file.write(path, content.encode())', 'file.write(path, content.encode())'),
            (
                'file.write(path, crypto.encode_utf16le(content))',
                'file.write(path, content.encode("utf16le"))',
            ),
            (
                'data = file.read_binary(src)\nfile.write(dst, data)',
                'data = file.read(src)\nfile.write(dst, data)',
            ),
            # Variable-based `in` / `not in` and `+` string operations trigger str() cast
            (
                'content = file.read(filepath)\nif install_path not in content:\n    file.write(filepath, content + startup_cmd)',
                'content = str(file.read(filepath))\nif install_path not in content:\n    file.write(filepath, content + startup_cmd)',
            ),
            # Helper returning file.read() and "" gets str() cast
            (
                'def safe_read(path):\n    if file.is_file(path):\n        return file.read(path)\n    return ""\n',
                'def safe_read(path):\n    if file.is_file(path):\n        return str(file.read(path))\n    return ""\n',
            ),
            # Explicit bytes(...) membership check preserves raw Bytes without str() cast
            (
                'contents = file.read(path)\nif bytes(password) in contents:\n    print("ok")',
                'contents = file.read(path)\nif bytes(password) in contents:\n    print("ok")',
            ),
        ]
        for original, expected in cases:
            with self.subTest(original=original):
                migrated = migration.migrate_content(original)
                self.assertEqual(migrated, expected)
                if "_binary" not in original and expected != "file.read(path)":
                    self.assertEqual(migration.migrate_content(migrated), expected)

    def test_shadowed_stdlib_variables_and_params(self):
        cases = [
            # `user = sys.get_user()` -> `cur_user = user.info()` without overwriting `user` module
            (
                'def pre_flight():\n'
                '    user = sys.get_user()\n'
                '    if user["euid"]["uid"] != 0:\n'
                '        return False\n'
                '    return True\n',
                'def pre_flight():\n'
                '    cur_user = user.info()\n'
                '    if cur_user["euid"]["uid"] != 0:\n'
                '        return False\n'
                '    return True\n',
            ),
            # `windows = sys.is_windows()` -> `is_windows = sys.is_windows()`
            (
                'windows = sys.is_windows()\n'
                'def can_read(f):\n'
                '    if windows:\n'
                '        return True\n'
                '    return False\n',
                'is_windows = sys.is_windows()\n'
                'def can_read(f):\n'
                '    if is_windows:\n'
                '        return True\n'
                '    return False\n',
            ),
            # Loop variable and f-string with multi-byte UTF-8 emojis
            (
                'for user in candidate_users:\n'
                '    print(f"[INFO] ❌ Trying \'{user}\' ✅")\n'
                '    res = test_ssh(user)\n',
                'for cur_user in candidate_users:\n'
                '    print(f"[INFO] ❌ Trying \'{cur_user}\' ✅")\n'
                '    res = test_ssh(cur_user)\n',
            ),
            # Function parameters and `file` variable shadowing alongside `file.is_file` calls
            (
                'def deploy(ip, user):\n'
                '    file = input_params["FILE"]\n'
                '    if file.is_file(file):\n'
                '        pivot.ssh_copy(target=ip, src=file, username=user)\n',
                'def deploy(ip, cur_user):\n'
                '    target_file = input_params["FILE"]\n'
                '    if file.is_file(target_file):\n'
                '        ssh.copy(target=ip, src=target_file, username=cur_user)\n',
            ),
        ]
        for original, expected in cases:
            with self.subTest(original=original):
                migrated = migration.migrate_content(original)
                self.assertEqual(migrated, expected)
                self.assertEqual(migration.migrate_content(migrated), expected)

    def test_legacy_multi_arg_registry_calls(self):
        cases = [
            (
                'winlogon = sys.get_reg(\n'
                '    "HKEY_LOCAL_MACHINE",\n'
                '    "SOFTWARE\\\\Microsoft\\\\Windows NT\\\\CurrentVersion\\\\Winlogon",\n'
                ')\n',
                'winlogon = registry.read(\n'
                '    "HKEY_LOCAL_MACHINE\\\\SOFTWARE\\\\Microsoft\\\\Windows NT\\\\CurrentVersion\\\\Winlogon",\n'
                ')\n',
            ),
            (
                'vals = sys.get_reg(HIVE, reg_path)\n'
                'ok1 = sys.write_reg_str(HIVE, reg_path, "ImagePath", "REG_EXPAND_SZ", bin_path)\n'
                'ok2 = sys.write_reg_int(HIVE, reg_path, "Start", "REG_DWORD", 4)\n',
                'vals = registry.read(HIVE + "\\\\" + reg_path)\n'
                'ok1 = registry.write(HIVE + "\\\\" + reg_path, "ImagePath", "REG_EXPAND_SZ", bin_path)\n'
                'ok2 = registry.write(HIVE + "\\\\" + reg_path, "Start", "REG_DWORD", 4)\n',
            ),
        ]
        for original, expected in cases:
            with self.subTest(original=original):
                migrated = migration.migrate_content(original)
                self.assertEqual(migrated, expected)
                self.assertEqual(migration.migrate_content(migrated), expected)

    def test_nested_calls_and_idempotence(self):
        original = (
            'time.format_to_readable(time.format_to_epoch("2026-01-01 00:00:00", '
            '"%Y-%m-%d %H:%M:%S"), "%Y")\n'
            'file.template_str("{{ cwd }}", {"cwd": file.pwd()}, True)\n'
            'file.template("source.j2", file.temp_file(), agent.get_config(), False)\n'
        )
        expected = (
            'time.format(time.to_epoch("2026-01-01 00:00:00", '
            '"%Y-%m-%d %H:%M:%S"), "%Y")\n'
            'template.render("{{ cwd }}", {"cwd": process.cwd()}, True)\n'
            'template.render_file("source.j2", file.tmp(), agent.config(), False)\n'
        )
        self.assertEqual(migration.migrate_content(original), expected)
        self.assertEqual(migration.migrate_content(expected), expected)

    def test_cli_dry_run_and_directory_migration(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tome = root / "example"
            tome.mkdir()
            script = tome / "main.eldritch"
            original = "\n".join(f"{old}()" for old in self.RENAMES)
            expected = "\n".join(f"{new}()" for new in self.RENAMES.values())
            script.write_text(original, encoding="utf-8")
            metadata = tome / "metadata.yml"
            metadata.write_text("name: file.template\n", encoding="utf-8")

            subprocess.run(
                [sys.executable, str(SCRIPT), "--dry-run", str(root)],
                check=True, capture_output=True, text=True,
            )
            self.assertEqual(script.read_text(encoding="utf-8"), original)
            subprocess.run(
                [sys.executable, str(SCRIPT), str(root)],
                check=True, capture_output=True, text=True,
            )
            self.assertEqual(script.read_text(encoding="utf-8"), expected)
            self.assertEqual(metadata.read_text(encoding="utf-8"), "name: file.template\n")


if __name__ == "__main__":
    unittest.main()
