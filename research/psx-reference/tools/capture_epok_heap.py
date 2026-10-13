"""Read the PsyQo allocator's high-water address without changing emulator RAM.

Not a stack watermark or a proof of worst-case heap/stack separation.
The ABI is checked against the linked 24-byte s_heap_metadata symbol.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import urllib.request
from sample_epok_runtime import executable_sections, verify_running_code


def inspect(build, ram):
    elf = (build / "epok.elf").read_bytes()
    symbols = (build / "epok.map").read_text()
    match = re.search(r"\.bss\.s_heap_metadata\s+0x([0-9a-fA-F]+)\s+0x18\b", symbols)
    if not match:
        raise ValueError("Unsupported/missing PsyQo allocator metadata layout")
    def symbol(name):
        match = re.search(r"0x([0-9a-fA-F]+)\s+" + re.escape(name) + r"\b", symbols)
        if not match:
            raise ValueError(f"Missing {name}")
        return int(match[1], 16)
    if len(ram) != 0x200000:
        raise ValueError("Expected a complete 2 MiB RAM snapshot")
    verify_running_code(ram, executable_sections(elf))
    address = int(match[1], 16)
    head, bottom, top, maximum, marker_next, marker_size = struct.unpack_from("<6I", ram, address & 0x1fffffff)
    base, stack = symbol("__heap_start"), symbol("__stack_start")
    if maximum and not base <= maximum <= stack:
        raise ValueError("Allocator high-water address is out of bounds")
    report = {"scope": __doc__, "elf_sha256": hashlib.sha256(elf).hexdigest(),
              "ram_sha256": hashlib.sha256(ram).hexdigest(),
              "metadata_address": hex(address), "heap_start": hex(base), "stack_start": hex(stack),
              "allocator_words": [hex(v) for v in (head, bottom, top, maximum, marker_next, marker_size)],
              "heap_high_water_span_bytes": maximum - base if maximum else 0,
              "gap_from_heap_high_water_to_stack_start": stack - (maximum or base),
              "stack_high_water_measured": False}
    return report


def capture(build, output, port=None, ram_path=None):
    if ram_path is not None:
        ram = ram_path.read_bytes()
    else:
        with urllib.request.urlopen(f"http://127.0.0.1:{port}/api/v1/cpu/ram/raw", timeout=3) as response:
            ram = response.read()
    report = inspect(build, ram)
    output.mkdir(parents=True, exist_ok=False)
    (output / "ram.bin").write_bytes(ram)
    (output / "heap.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", type=Path)
    parser.add_argument("output", type=Path)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--port", type=int)
    source.add_argument("--ram", type=Path, help="Inspect an existing headless capture instead of a live emulator")
    args = parser.parse_args()
    print(json.dumps(capture(args.build, args.output, args.port, args.ram), indent=2))
