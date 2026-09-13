#!/usr/bin/env python3
"""Run an Epok build and record its successful local build count."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def git(root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(root), *args],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    )
    return result.stdout.strip() if result.returncode == 0 else ""


def resolve_base(root: Path, base_branch: str) -> str:
    candidates = (
        f"refs/heads/{base_branch}",
        f"refs/remotes/origin/{base_branch}",
        base_branch,
    )
    for candidate in candidates:
        commit = git(root, "merge-base", "HEAD", candidate)
        if commit:
            return commit
    commit = git(root, "rev-parse", "HEAD")
    if not commit:
        raise RuntimeError("the repository has no Git commit to use as build base")
    return commit


def load_state(path: Path) -> dict[str, object]:
    if not path.exists():
        return {}
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise RuntimeError(f"cannot read {path}: {error}") from error
    if not isinstance(value, dict):
        raise RuntimeError(f"{path} must contain a JSON object")
    return value


def save_state(path: Path, state: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(
        json.dumps(state, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    os.replace(temporary, path)


def record(root: Path, path: Path, base_branch: str, command: list[str]) -> int:
    if not command:
        raise RuntimeError("record requires a build command after --")
    result = subprocess.run(command, cwd=root, check=False)
    if result.returncode:
        return result.returncode

    version = (root / "VERSION").read_text(encoding="utf-8").strip()
    base_commit = resolve_base(root, base_branch)
    previous = load_state(path)
    same_base = (
        previous.get("base_branch") == base_branch
        and previous.get("base_commit") == base_commit
        and previous.get("current_version") == version
    )
    count = int(previous.get("build_count", 0)) + 1 if same_base else 1
    state: dict[str, object] = {
        "schema_version": 1,
        "current_version": version,
        "base_branch": base_branch,
        "base_commit": base_commit,
        "build_count": count,
        "last_build_commit": git(root, "rev-parse", "HEAD"),
        "last_build_utc": datetime.now(timezone.utc).isoformat(),
        "last_command": command,
    }
    save_state(path, state)
    print(f"Epok local build {version}+{count} (base {base_branch}@{base_commit[:12]})")
    return 0


def main() -> int:
    root_default = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=root_default)
    parser.add_argument("--state", type=Path)
    subparsers = parser.add_subparsers(dest="action", required=True)
    build = subparsers.add_parser("record", help="Run a command and record success")
    build.add_argument("--base", default="develop")
    build.add_argument("command", nargs=argparse.REMAINDER)
    subparsers.add_parser("show", help="Print the current local state")
    subparsers.add_parser("reset", help="Delete the current local state")
    args = parser.parse_args()

    root = args.repo.resolve()
    state_path = (args.state or root / ".epok" / "build-state.json").resolve()
    try:
        if args.action == "record":
            command = args.command[1:] if args.command[:1] == ["--"] else args.command
            return record(root, state_path, args.base, command)
        if args.action == "show":
            state = load_state(state_path)
            if not state:
                print(f"No local build state at {state_path}")
                return 0
            print(json.dumps(state, indent=2, sort_keys=True))
            return 0
        if state_path.exists():
            state_path.unlink()
            print(f"Removed {state_path}")
        else:
            print(f"No local build state at {state_path}")
    except (OSError, RuntimeError, ValueError) as error:
        print(f"build-state error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
