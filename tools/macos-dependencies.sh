# shellcheck shell=bash
# Shared paths for a source checkout and a Finder bundle.

macos_tools=".tools/macos"
macos_redux_app="$macos_tools/redux/PCSX-Redux.app"
macos_libclang="$macos_tools/libclang/libclang-18.1.1.data/platlib/clang/native"

nugget_revision() {
    python3 -c 'import json, sys; print(json.load(sys.stdin)["nugget"]["revision"])' \
        < "$1/tools/dependencies.json"
}

prepare_nugget() {
    local root="$1" revision="$2" sdk="$1/third_party/nugget" file gitlink
    if [[ -e "$root/.git" ]]; then
        gitlink="$(git -C "$root" ls-files --stage -- third_party/nugget | awk '$1 == 160000 && $3 == 0 {print $2}')"
        [[ "$gitlink" == "$revision" ]] || {
            echo "Nugget gitlink and tools/dependencies.json disagree." >&2
            return 1
        }
        git -C "$root" -c submodule.recurse=false submodule update --init --checkout -- third_party/nugget
    fi

    if [[ -e "$sdk/.git" ]]; then
        [[ "$(git -C "$sdk" rev-parse HEAD)" == "$revision" ]] || {
            echo "Nugget is not at the pinned revision $revision." >&2
            return 1
        }
        # The mirror contains unrelated nested submodules that cannot all be initialized.
        git -C "$sdk" -c submodule.recurse=false submodule update --init --checkout -- third_party/psxlua
    elif [[ ! -f "$sdk/.epok-pinned-revision" || "$(<"$sdk/.epok-pinned-revision")" != "$revision" ]]; then
        echo "Missing Nugget $revision. Run setup from a checkout or rebuild the app with make app." >&2
        return 1
    fi

    for file in psyqo/psyqo.mk common.mk third_party/EASTL/include/EASTL/array.h \
        third_party/EABase/include/Common/EABase/eabase.h third_party/psxlua/src/lparser.c; do
        [[ -f "$sdk/$file" ]] || { echo "Incomplete Nugget SDK: $file" >&2; return 1; }
    done
    echo "Verified Nugget $revision and psxlua."
}

macos_toolchain_bin() {
    local compiler bin prefix
    if compiler="$(command -v mipsel-none-elf-g++)"; then
        bin="$(dirname "$compiler")"
        if [[ -x "$bin/mipsel-none-elf-objcopy" ]]; then
            printf '%s\n' "$bin"
            return
        fi
    fi
    if command -v brew >/dev/null 2>&1; then
        prefix="$(brew --prefix)" || return
        printf '%s/bin\n' "$prefix"
    else
        printf '%s\n' /opt/homebrew/bin
    fi
}

redux_executable() {
    local name path
    for name in PCSX-Redux pcsx-redux; do
        path="$1/Contents/MacOS/$name"
        if [[ -f "$path" && -x "$path" ]]; then
            printf '%s\n' "$path"
            return
        fi
    done
    return 1
}

write_macos_config() {
    python3 - "$1" "$2" "$3" "$macos_tools" "$macos_libclang" <<'PY'
import json
import os
import sys
import tempfile
from pathlib import Path

path, toolchain, emulator, tools, libclang = sys.argv[1:]
destination = Path(path)
config = {
    "make": "make",
    "toolchain_bin": toolchain,
    "nugget": "third_party/nugget",
    "emulator": emulator,
    "psxavenc": f"{tools}/bin/psxavenc",
    "mkpsxiso": f"{tools}/bin/mkpsxiso",
    "libclang": libclang,
    "code": "",
    "web_port": 8077,
    "auto_build": True,
}
with tempfile.NamedTemporaryFile(
    mode="w", encoding="utf-8", dir=destination.parent, prefix=".epokconfig-", delete=False
) as draft:
    temporary = Path(draft.name)
    try:
        for key, value in config.items():
            draft.write(f"{key}: {json.dumps(value, ensure_ascii=False)}\n")
        draft.close()
        os.replace(temporary, destination)
    finally:
        temporary.unlink(missing_ok=True)
PY
}
