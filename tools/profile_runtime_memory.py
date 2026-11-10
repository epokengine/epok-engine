"""Read PsyQo's pinned allocator layout and decode linked MIPS stack writes.

These are observer-side helpers: they allocate no console state and do not
change an executable's RAM budget. Heap sizes include allocator block headers.
"""
from dataclasses import dataclass
from pathlib import Path
import json
import re
import struct
import subprocess

MASK = 0x1fffffff


def heap_snapshot(ram, metadata_address, heap_start, ram_top=0x200000):
    def address(value, endpoint=False):
        physical = value & MASK
        if physical > len(ram) or physical == len(ram) and not endpoint:
            raise ValueError("Allocator pointer outside main RAM")
        return physical

    def word(offset):
        if offset < 0 or offset + 4 > len(ram):
            raise ValueError("Allocator metadata outside main RAM")
        return struct.unpack_from("<I", ram, offset)[0]

    meta = address(metadata_address)
    marker = meta + 16
    head, bottom, top, maximum = (word(meta + i * 4) for i in range(4))
    if not head:
        if bottom or top or maximum:
            raise ValueError("Inconsistent uninitialized allocator")
        return {"initialized": False, "live_bytes": 0, "footprint_bytes": 0,
                "sdk_maximum_bytes": 0, "free_blocks": 0, "allocated_end": heap_start}
    bottom, top = address(bottom), address(top, True)
    if bottom < heap_start or bottom % 8 or top % 8 or not bottom < top <= ram_top:
        raise ValueError("Invalid allocator extent")
    current, cursor, free, blocks = address(head), bottom, 0, 0
    allocated_end = bottom
    footprint_end = bottom + 8  # The initial free-list header itself is live.
    while current != marker:
        blocks += 1
        if blocks > (top - bottom) // 8:
            raise ValueError("Allocator free-list cycle")
        if current < cursor or current % 8 or current + 8 > top:
            raise ValueError("Unordered or out-of-range allocator free block")
        next_block, size = word(current), word(current + 4)
        if size < 8 or size % 8 or current + size > top:
            raise ValueError("Invalid allocator free-block size")
        if cursor < current:
            allocated_end = current
        footprint_end = max(footprint_end, current + 8)
        free += size
        cursor = current + size
        current = address(next_block)
    if cursor < top:
        allocated_end = top
    footprint_end = max(footprint_end, allocated_end)
    maximum = address(maximum, True) if maximum else bottom
    if not bottom <= maximum <= top:
        raise ValueError("Invalid allocator maximum")
    return {"initialized": True, "live_bytes": top - bottom - free,
            "footprint_bytes": footprint_end - bottom,
            "sdk_maximum_bytes": maximum - bottom, "free_blocks": blocks,
            "allocated_end": allocated_end, "heap_start": bottom, "heap_top": top}


@dataclass(frozen=True)
class StackWrite:
    pc: int
    word: int
    kind: str
    rs: int
    rt: int
    immediate: int


def decode_stack_write(pc, instruction):
    op, rs, rt = instruction >> 26, instruction >> 21 & 31, instruction >> 16 & 31
    rd, function = instruction >> 11 & 31, instruction & 63
    immediate = instruction & 0xffff
    if op == 0 and rd == 29:
        kinds = {0: "sll", 2: "srl", 3: "sra", 4: "sllv", 6: "srlv", 7: "srav",
                 0x20: "add", 0x21: "add", 0x22: "sub", 0x23: "sub",
                 0x24: "and", 0x25: "or", 0x26: "xor", 0x27: "nor"}
        if function not in kinds:
            raise ValueError(f"Unsupported SP writer at {pc:#x}: {instruction:#x}")
        return StackWrite(pc, instruction, kinds[function], rs, rt, instruction >> 6 & 31)
    if op in (8, 9, 12, 13, 14, 15, 32, 33, 35, 36, 37) and rt == 29:
        kinds = {8: "addi", 9: "addi", 12: "andi", 13: "ori", 14: "xori", 15: "lui",
                 32: "lb", 33: "lh", 35: "lw", 36: "lbu", 37: "lhu"}
        if op in (8, 9, 32, 33, 35, 36, 37) and immediate & 0x8000:
            immediate -= 0x10000
        return StackWrite(pc, instruction, kinds[op], rs, rt, immediate)
    if op in (16, 18) and rs in (0, 2) and rt == 29:
        raise ValueError(f"Unsupported coprocessor SP writer at {pc:#x}")
    if rt == 29 and op in (10, 11, 34, 38, 48, 56):
        raise ValueError(f"Unsupported SP writer at {pc:#x}: {instruction:#x}")
    return None


def linked_stack_writes(disassembly):
    writes = []
    for match in re.finditer(r"^\s*([0-9a-fA-F]+):\s+([0-9a-fA-F]{8})\s", disassembly, re.M):
        value = decode_stack_write(int(match[1], 16), int(match[2], 16))
        if value:
            writes.append(value)
    return writes


def predicted_stack_pointer(write, registers, read_memory):
    a, b, n, kind = registers[write.rs] & 0xffffffff, registers[write.rt] & 0xffffffff, write.immediate, write.kind
    signed = lambda value: value - 0x100000000 if value & 0x80000000 else value
    if kind in ("add", "addi"):
        result = a + (b if kind == "add" else n)
    elif kind == "sub":
        result = a - b
    elif kind in ("and", "andi", "or", "ori", "xor", "xori", "nor"):
        right = n if kind.endswith("i") else b
        result = a & right if kind.startswith("and") else a ^ right if kind.startswith("xor") else a | right
        if kind == "nor":
            result = ~result
    elif kind == "lui":
        result = n << 16
    elif kind in ("sll", "srl", "sra", "sllv", "srlv", "srav"):
        shift = a & 31 if kind.endswith("v") else n
        result = b << shift if kind.startswith("sll") else signed(b) >> shift if kind.startswith("sra") else b >> shift
    elif kind in ("lb", "lh", "lw", "lbu", "lhu"):
        size = {"lb": 1, "lh": 2, "lw": 4, "lbu": 1, "lhu": 2}[kind]
        result = int.from_bytes(read_memory((a + n) & 0xffffffff, size), "little", signed=kind in ("lb", "lh"))
    else:
        raise ValueError(f"Unsupported SP operation {kind}")
    return result & 0xffffffff


def elf_probe_config(elf, tool_directory):
    """Inspect the exact executable, failing if its allocator/stack is unknown."""
    suffix = ".exe" if (Path(tool_directory) / "mipsel-none-elf-nm.exe").exists() else ""
    def run(tool, *arguments):
        return subprocess.check_output([str(Path(tool_directory) / ("mipsel-none-elf-" + tool + suffix)),
                                        *arguments, str(elf)], text=True)
    symbols = {}
    for line in run("nm", "-S", "--defined-only").splitlines():
        match = re.fullmatch(r"([0-9a-fA-F]+) (?:[0-9a-fA-F]+ )?\w (.+)", line)
        if match:
            name, value = match[2], int(match[1], 16)
            if name in symbols and symbols[name] != value:
                # Duplicate local symbols are irrelevant unless requested below.
                symbols[name] = None
            elif name not in symbols:
                symbols[name] = value
    def require(name):
        value = symbols.get(name)
        if value is None:
            raise ValueError(f"Missing or ambiguous ELF symbol: {name}")
        return value
    heap_start, ram_top = require("__heap_start") & MASK, require("__stack_start") & MASK
    metadata = require("s_heap_metadata") & MASK
    if not 0x10000 <= metadata < metadata + 24 <= heap_start < ram_top == 0x200000:
        raise ValueError("Unsupported allocator metadata or RAM layout")
    allocators = [symbols[name] for name in ("psyqo_malloc", "psyqo_realloc", "psyqo_free") if symbols.get(name)]
    if not allocators:
        raise ValueError("No PsyQo allocator entry points")
    writes = linked_stack_writes(run("objdump", "-d"))
    if not writes:
        raise ValueError("No linked stack writes")
    config = {"start": require("_start"), "heap_start": heap_start, "ram_top": ram_top,
              "metadata": metadata, "allocators": allocators, "stack_write_sites": len(writes),
              "image_start": require("__ftext_start") & MASK,
              "lua_stats": (symbols.get("_ZN4epok3lua5statsE") or 0) & MASK}
    return config, writes


def lua_probe(config, writes):
    """Generate an external observer; no target instructions or BSS are added."""
    def literal(value):
        if isinstance(value, str):
            return json.dumps(value)
        if isinstance(value, (list, tuple)):
            return "{" + ",".join(map(literal, value)) + "}"
        if isinstance(value, dict):
            return "{" + ",".join(f"[{literal(key)}]={literal(item)}" for key, item in value.items()) + "}"
        return str(value)
    config = dict(config, writes=[[w.pc, w.kind, w.rs, w.rt, w.immediate] for w in writes])
    source = Path(__file__).with_name("runtime_memory_probe.lua").read_text(encoding="utf-8")
    return "\ndo\nlocal config=" + literal(config) + "\n" + source + "\nend\n"
