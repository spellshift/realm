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
            # Regular read cast to str() only when needed
            ('res = file.read(path)', 'res = str(file.read(path))'),
            ('print(assets.read("a.sh"))', 'print(str(assets.read("a.sh")))'),
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
            # Regular write/append cast to bytes() only when needed
            ('file.write(path, content)', 'file.write(path, bytes(content))'),
            ('file.append(path, "line\\n")', 'file.append(path, bytes("line\\n"))'),
            # Already bytes -> no extra cast
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
        ]
        for original, expected in cases:
            with self.subTest(original=original):
                migrated = migration.migrate_content(original)
                self.assertEqual(migrated, expected)
                if "_binary" not in original and expected != "file.read(path)":
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
