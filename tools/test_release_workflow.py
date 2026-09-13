import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import build_state
import versioning


def git(root: Path, *args: str) -> None:
    subprocess.run(["git", "-C", str(root), *args], check=True, capture_output=True)


class SemVerTests(unittest.TestCase):
    def test_semver_precedence(self) -> None:
        ordered = [
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0-alpha.beta",
            "1.0.0-beta",
            "1.0.0-beta.2",
            "1.0.0-beta.11",
            "1.0.0-rc.1",
            "1.0.0",
            "1.1.0",
            "2.0.0",
        ]
        parsed = [versioning.Version.parse(value) for value in ordered]
        self.assertEqual(parsed, sorted(reversed(parsed)))

    def test_rejects_leading_zero(self) -> None:
        with self.assertRaises(ValueError):
            versioning.Version.parse("01.0.0")


class BuildStateTests(unittest.TestCase):
    def test_counts_successes_and_resets_on_new_base(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "VERSION").write_text("0.1.0\n", encoding="utf-8")
            git(root, "init", "-b", "develop")
            git(root, "config", "user.name", "Epok Test")
            git(root, "config", "user.email", "epok@example.invalid")
            git(root, "add", "VERSION")
            git(root, "commit", "-m", "Initial version")
            state_path = root / ".epok" / "build-state.json"
            command = [sys.executable, "-c", "pass"]

            self.assertEqual(build_state.record(root, state_path, "develop", command), 0)
            self.assertEqual(build_state.record(root, state_path, "develop", command), 0)
            state = json.loads(state_path.read_text(encoding="utf-8"))
            self.assertEqual(state["build_count"], 2)

            (root / "next.txt").write_text("next\n", encoding="utf-8")
            git(root, "add", "next.txt")
            git(root, "commit", "-m", "Advance develop")
            self.assertEqual(build_state.record(root, state_path, "develop", command), 0)
            state = json.loads(state_path.read_text(encoding="utf-8"))
            self.assertEqual(state["build_count"], 1)


if __name__ == "__main__":
    unittest.main()
