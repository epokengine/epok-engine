#!/usr/bin/env python3
"""Validate Epok's release version against its release branch and Git tags."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


SEMVER_RE = re.compile(
    r"^(?P<major>0|[1-9]\d*)\."
    r"(?P<minor>0|[1-9]\d*)\."
    r"(?P<patch>0|[1-9]\d*)"
    r"(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*)"
    r"(?:\.(?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*))*))?"
    r"(?:\+(?P<build>[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$"
)


@dataclass(frozen=True)
class Version:
    major: int
    minor: int
    patch: int
    prerelease: tuple[str, ...] = ()

    @classmethod
    def parse(cls, value: str) -> "Version":
        match = SEMVER_RE.fullmatch(value.strip())
        if not match:
            raise ValueError(f"invalid SemVer version: {value!r}")
        prerelease = tuple((match.group("prerelease") or "").split("."))
        if prerelease == ("",):
            prerelease = ()
        return cls(
            int(match.group("major")),
            int(match.group("minor")),
            int(match.group("patch")),
            prerelease,
        )

    def __str__(self) -> str:
        base = f"{self.major}.{self.minor}.{self.patch}"
        return base if not self.prerelease else f"{base}-{'.'.join(self.prerelease)}"

    def __lt__(self, other: "Version") -> bool:
        ours = (self.major, self.minor, self.patch)
        theirs = (other.major, other.minor, other.patch)
        if ours != theirs:
            return ours < theirs
        if not self.prerelease:
            return False
        if not other.prerelease:
            return True
        for left, right in zip(self.prerelease, other.prerelease):
            if left == right:
                continue
            left_numeric = left.isdigit()
            right_numeric = right.isdigit()
            if left_numeric and right_numeric:
                return int(left) < int(right)
            if left_numeric != right_numeric:
                return left_numeric
            return left < right
        return len(self.prerelease) < len(other.prerelease)


def run_git(root: Path, *args: str, check: bool = True) -> str:
    result = subprocess.run(
        ["git", "-C", str(root), *args],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if check and result.returncode:
        detail = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(f"git {' '.join(args)} failed: {detail}")
    return result.stdout.strip() if result.returncode == 0 else ""


def read_current(root: Path) -> tuple[str, Version]:
    value = (root / "VERSION").read_text(encoding="utf-8").strip()
    return value, Version.parse(value)


def cargo_version(root: Path) -> str:
    manifest = (root / "Cargo.toml").read_text(encoding="utf-8")
    package = manifest.split("[package]", 1)[1].split("\n[", 1)[0]
    match = re.search(r'^version\s*=\s*"([^"]+)"\s*$', package, re.MULTILINE)
    if not match:
        raise ValueError("Cargo.toml [package] does not declare a version")
    return match.group(1)


def version_from_ref(root: Path, ref: str) -> Version | None:
    value = run_git(root, "show", f"{ref}:VERSION", check=False)
    return Version.parse(value) if value else None


def merged_versions(root: Path, ref: str) -> list[tuple[str, Version]]:
    tags = run_git(root, "tag", "--merged", ref, "--list", "v*").splitlines()
    parsed: list[tuple[str, Version]] = []
    for tag in tags:
        try:
            parsed.append((tag, Version.parse(tag.removeprefix("v"))))
        except ValueError:
            continue
    return parsed


def validate_release(root: Path, base_ref: str) -> None:
    current_text, current = read_current(root)
    manifest_version = cargo_version(root)
    if current_text != manifest_version:
        raise ValueError(
            f"VERSION ({current_text}) must match Cargo.toml package version "
            f"({manifest_version})"
        )

    floors: list[tuple[str, Version]] = []
    base_version = version_from_ref(root, base_ref)
    if base_version is not None:
        floors.append((f"{base_ref}:VERSION", base_version))
    floors.extend(merged_versions(root, base_ref))
    if not floors:
        print(f"Release version {current_text} is valid; no previous version exists.")
        return

    source, floor = max(floors, key=lambda item: item[1])
    if not floor < current:
        raise ValueError(
            f"release version {current_text} must be greater than {floor} ({source})"
        )
    print(f"Release version {current_text} is greater than {floor} ({source}).")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo", type=Path, default=Path(__file__).resolve().parents[1]
    )
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("current", help="Print the current repository version")
    validate = subparsers.add_parser(
        "validate-release", help="Validate a candidate release version"
    )
    validate.add_argument("--base-ref", default="origin/release")
    args = parser.parse_args()

    root = args.repo.resolve()
    try:
        if args.command == "current":
            current, parsed = read_current(root)
            manifest_version = cargo_version(root)
            if current != manifest_version:
                raise ValueError(
                    f"VERSION ({current}) does not match Cargo.toml ({manifest_version})"
                )
            print(parsed)
        else:
            validate_release(root, args.base_ref)
    except (OSError, RuntimeError, ValueError) as error:
        print(f"version error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
