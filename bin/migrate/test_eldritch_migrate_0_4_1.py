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
    }

    def test_calls_and_references(self):
        for old, new in self.RENAMES.items():
            with self.subTest(old=old):
                self.assertEqual(migration.migrate_content(f"{old}()"), f"{new}()")
                self.assertEqual(migration.migrate_content(f"fn = {old}"), f"fn = {new}")
                self.assertEqual(migration.migrate_content(f"{old}_suffix()"), f"{old}_suffix()")

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
