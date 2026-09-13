#!/usr/bin/env python3
"""Prepare the verified offline serial payload when assembling an Epok distribution.

The editor can download the same payload itself; end users never need Python.
Keep .tools/serial-bundle alongside the editor's tools/ and runtime/ folders.
"""
import hashlib
import json
from pathlib import Path
import urllib.request


def main():
    root = Path(__file__).resolve().parent.parent
    manifest = json.loads((root / "tools/serial-dependency.json").read_text())
    destination = root / ".tools/serial-bundle"
    destination.mkdir(parents=True, exist_ok=True)
    for entry in manifest["files"]:
        name = entry["name"]
        if Path(name).name != name or "/" in name or "\\" in name or name.startswith("."):
            raise ValueError("Invalid package filename")
        output = destination / name
        data = output.read_bytes() if output.is_file() else b""
        if len(data) != entry["size"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
            with urllib.request.urlopen(manifest["base_url"] + name, timeout=30) as response:
                data = response.read(entry["size"] + 1)
            if len(data) != entry["size"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
                raise ValueError(f"Checksum mismatch: {name}")
            temporary = output.with_suffix(output.suffix + ".tmp")
            temporary.write_bytes(data)
            temporary.replace(output)
        print(f"Verified {name}")
    (destination / "SOURCE.txt").write_text(
        f"Unmodified NOTPSXSerial\nSource: {manifest['source']}\n"
        "See LICENSE and THIRD_PARTY_NOTICES.txt.\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()
