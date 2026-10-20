"""Verify the actual Windows portable ZIP before publishing it."""
import argparse
from pathlib import Path, PurePosixPath
import re
import zipfile


def verify(archive: Path, version: str) -> None:
    prefix = f"Epok-Engine-v{version}-Windows-x64-Portable/"
    required = {
        "Epok Engine.exe", "epok-header-tool.exe", "Editor.epokconfig", "VERSION",
        "LICENSE", "README.md", "THIRD_PARTY_NOTICES.md", "PORTABLE.txt",
        "tools/setup.ps1", "tools/dependencies.json", "tools/serial-dependency.json",
        "tools/register-project.ps1", "runtime/epok.hpp", "runtime/data_table.hpp",
        "runtime/data_table_service.hpp", "resources/code-editor/index.html",
        "resources/code-editor/app.js", "resources/code-editor/editor.worker.js",
        "resources/code-editor/LICENSE-monaco.txt", "templates/ThirdPersonController.lua",
        "examples/sample-game/Epok Sample.epokproject",
        "examples/gameplay-api-parity/blueprint-only/Blueprint Gameplay Parity.epokproject",
        "examples/gameplay-api-parity/lua-only/Lua Gameplay Parity.epokproject",
    }
    with zipfile.ZipFile(archive) as package:
        names = package.namelist()
        assert len(names) == len(set(names)), "Duplicate archive entries"
        for name in names:
            path = PurePosixPath(name)
            assert "\\" not in name and not path.is_absolute() and ".." not in path.parts, f"Unsafe archive entry: {name}"
            assert name.startswith(prefix), f"Unexpected package root: {name}"
            assert not any(part in {".git", ".epok", ".tools", "target", "node_modules"} for part in path.parts), f"Local build state in archive: {name}"
        missing = sorted(prefix + name for name in required if prefix + name not in names)
        assert not missing, f"Missing portable files: {missing}"
        assert all(package.getinfo(prefix + name).file_size > 0 for name in required), "Empty required portable file"
        assert package.read(prefix + "VERSION").decode("utf-8-sig").strip() == version, "Packaged VERSION mismatch"
        config = package.read(prefix + "Editor.epokconfig").decode("utf-8-sig")
        for field in ("make", "toolchain_bin", "nugget", "emulator", "psxavenc", "mkpsxiso"):
            match = re.search(rf"^{field}:\s*(.+)$", config, re.M)
            assert match, f"Missing portable configuration field: {field}"
            value = match[1].strip().strip("\"'")
            assert value.startswith((".tools/", "third_party/")) and ".." not in value.split("/"), f"Nonportable {field}: {value}"
        projects = [name for name in names if name.endswith(".epokproject") and name.startswith(prefix + "examples/")]
        for project in projects:
            descriptor = package.read(project).decode("utf-8-sig")
            match = re.search(r"^editor_version:\s*(\S+)\s*$", descriptor, re.M)
            assert match and match[1].strip("\"'") == version, f"Example version mismatch: {project}"
        assert package.testzip() is None, "Archive CRC failure"
    print(f"Verified Windows portable v{version}: {len(required)} required files and {len(projects)} example descriptors")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("--version", required=True)
    args = parser.parse_args()
    verify(args.archive, args.version)
