"""Execute the actual cooked DataTable code on the host, and check MIPS syntax.

python tests/runtime/verify_data_tables.py --scene Game/.epok/build/scene.hh
"""
import argparse
import hashlib
from pathlib import Path
import re
import subprocess
import tempfile
from verify_blueprint_runtime import host_compiler, ROOT


def compact(uuid):
    return int.from_bytes(hashlib.sha256(uuid.encode()).digest()[:8], "little")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--scene", type=Path, required=True)
    args = parser.parse_args()
    text = args.scene.read_text(encoding="utf-8")
    table = re.search(r"namespace epok::bp \{\nstruct DataTableRow.*?\n\}\n", text, re.S)
    assert table, "No cooked DataTable definitions in scene header"
    rows = re.findall(r'\{UINT64_C\((\d+)\),("(?:[^"\\]|\\.)*"),UINT64_C\((\d+)\),(\d+)\}', table.group())
    assert rows, "The scene needs at least one populated table"
    table_id, key, asset_id, kind = rows[0]
    wrong = 1 if kind != "1" else 2
    program = f'''#include <stdint.h>
#include <stddef.h>
#ifdef __mips__
#define assert(value) ((value)?void(0):__builtin_trap())
#else
#include <assert.h>
#endif
#include "data_table.hpp"
{table.group()}
namespace epok::bp {{ int asset_index(uint64_t,uint32_t){{return -1;}} }}
#include "data_table_service.hpp"
int main() {{
    using namespace epok;
    const uint64_t table=UINT64_C({table_id}), asset=UINT64_C({asset_id});
    assert(assets::find(table,{key})==asset);
    assert(assets::find(table,{key},assets::Kind({kind}))==asset);
    assert(assets::find(table,{key},assets::Kind({wrong}))==0);
    assert(assets::find(table,"__missing_key__")==0);
    assert(assets::find(0,{key})==0);
    assert(assets::find(table,nullptr)==0);
    assert(assets::cast(asset,assets::Kind({kind}))==asset);
    assert(assets::cast(asset,assets::Kind({wrong}))==0);
    assert(assets::cast(0,assets::Kind({kind}))==0);
    // Verify every cooked row, including aliases and distinct tables.
    for(const auto& row:bp::data_table_rows) {{
        assert(assets::find(row.table,row.key,assets::Kind(row.kind))==row.asset);
    }}
}}
'''
    compiler, env = host_compiler()
    with tempfile.TemporaryDirectory(prefix="epok-data-table-runtime-") as temp:
        temp = Path(temp)
        source, binary = temp / "probe.cpp", temp / "probe.exe"
        source.write_text(program, encoding="utf-8")
        if Path(compiler).name.lower() == "cl.exe":
            command = [compiler, "/nologo", "/std:c++20", "/EHsc", str(source), f"/Fe:{binary}", f"/Fo:{temp / 'probe.obj'}", f"/I{ROOT / 'runtime'}"]
        else:
            command = [compiler, "-std=c++20", str(source), "-o", str(binary), f"-I{ROOT / 'runtime'}"]
        subprocess.run(command, cwd=temp, env=env, check=True)
        subprocess.run([str(binary)], check=True, timeout=15)
        mips = ROOT / ".tools/mips/bin/mipsel-none-elf-g++.exe"
        if mips.is_file():
            subprocess.run([str(mips), "-std=c++20", "-mips1", "-mabi=32", "-EL", "-msoft-float", "-ffreestanding", "-fsyntax-only", str(source), f"-I{ROOT / 'runtime'}"], check=True)
    print(f"PASS cooked DataTable lookup/cast/missing/wrong-type behavior for {len(rows)} rows; host execution and MIPS syntax")


if __name__ == "__main__":
    main()
