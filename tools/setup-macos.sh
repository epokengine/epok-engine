#!/usr/bin/env bash
# Provision Epok's host dependencies on macOS Apple Silicon.
set -euo pipefail

[[ "$(uname -s)" == Darwin ]] || { echo "This script must run on macOS." >&2; exit 1; }
[[ "$(uname -m)" == arm64 ]] || { echo "macOS setup supports Apple Silicon (arm64) only." >&2; exit 1; }

repair=false
case ${1:-} in
    "") ;;
    --repair) repair=true ;;
    *) echo "Usage: $0 [--repair]" >&2; exit 2 ;;
esac

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tools="$root/.tools/macos"
sdk="$root/third_party/nugget"
sdk_revision="6186b131aacc5853a9161fb076ed34ffe504552d"
sdk_marker="$sdk/.epok-pinned-revision"
psxlua="$sdk/third_party/psxlua"
psxlua_revision="abed030e686b4e34987851bd0a028c93b1f73967"

require() {
    command -v "$1" >/dev/null 2>&1 || { echo "Missing required command: $1" >&2; exit 1; }
}

require brew
require git
require xcrun
require curl
require python3
require shasum
require hdiutil
require ditto
xcrun --find clang >/dev/null

# NOTPSXSerial's managed executable needs Mono on macOS. The editor also checks
# this at first Serial Play and installs it through an existing Homebrew setup.
if ! command -v mono >/dev/null 2>&1; then
    brew install mono
fi

if ! command -v cargo >/dev/null 2>&1 && [[ ! -x "$(brew --prefix rustup)/bin/cargo" ]]; then
    brew install rustup
    "$(brew --prefix rustup)/bin/rustup" toolchain install 1.97.1 --profile minimal \
        --component rustfmt --component clippy
fi

if [[ -d "$root/.git" && ! -e "$sdk/.git" ]]; then
    git -C "$root" -c submodule.recurse=false submodule update --init -- third_party/nugget
fi
if [[ -e "$sdk/.git" ]]; then
    [[ "$(git -C "$sdk" rev-parse HEAD)" == "$sdk_revision" ]] || {
        echo "Nugget is not at the pinned revision $sdk_revision." >&2
        exit 1
    }
elif [[ -f "$sdk_marker" && "$(<"$sdk_marker")" == "$sdk_revision" ]]; then
    # Application bundles contain a verified SDK snapshot, not its original
    # checkout metadata. The marker is written only by package-macos-app.sh.
    echo "Verified bundled Nugget $sdk_revision"
else
    echo "Nugget is missing; initialize third_party/nugget first." >&2
    exit 1
fi
for file in psyqo/psyqo.mk common.mk third_party/EASTL/include/EASTL/array.h third_party/EABase/include/Common/EABase/eabase.h; do
    [[ -f "$sdk/$file" ]] || { echo "Incomplete Nugget SDK: $file" >&2; exit 1; }
done

# The Lua VM execution modes build their interpreter from psxlua, a nested
# submodule of the pinned Nugget mirror. Only this one is initialized: the
# mirror carries other nested submodules whose paths are not all valid after
# mirroring, which is why the SDK itself is never initialized recursively.
if [[ -e "$sdk/.git" && ! -f "$psxlua/src/lparser.c" ]]; then
    git -C "$sdk" -c submodule.recurse=false submodule update --init -- third_party/psxlua
fi
[[ -f "$psxlua/src/lparser.c" ]] || {
    echo "psxlua is missing; initialize third_party/nugget/third_party/psxlua first." >&2
    exit 1
}
if [[ -e "$psxlua/.git" ]]; then
    [[ "$(git -C "$psxlua" rev-parse HEAD)" == "$psxlua_revision" ]] || {
        echo "psxlua is not at the pinned revision $psxlua_revision." >&2
        exit 1
    }
fi
echo "Verified psxlua $psxlua_revision"

mkdir -p "$tools/src" "$tools/bin"

if ! command -v mipsel-none-elf-g++ >/dev/null 2>&1; then
    echo "Installing the PCSX-Redux MIPS toolchain through Homebrew..."
    brew install nikitabobko/tap/brew-install-path
    formulas="$tools/pcsx-redux-formulas"
    if [[ ! -d "$formulas/.git" ]]; then
        git clone --depth 1 https://github.com/grumpycoders/pcsx-redux.git "$formulas"
    elif $repair; then
        git -C "$formulas" pull --ff-only
    fi
    sed -i '' 's|https://ftpmirror.gnu.org/gnu/|https://mirrors.kernel.org/gnu/|' \
        "$formulas/tools/macos-mips/mipsel-none-elf-binutils.rb" \
        "$formulas/tools/macos-mips/mipsel-none-elf-gcc.rb"
    brew install-path "$formulas/tools/macos-mips/mipsel-none-elf-binutils.rb"
    brew install-path "$formulas/tools/macos-mips/mipsel-none-elf-gcc.rb"
fi

source_at() {
    local name="$1" url="$2" ref="$3" directory="$tools/src/$1"
    if [[ ! -d "$directory/.git" ]]; then
        git clone --depth 1 --branch "$ref" --recurse-submodules "$url" "$directory"
    elif $repair; then
        git -C "$directory" fetch --depth 1 origin "$ref"
        git -C "$directory" checkout --detach FETCH_HEAD
        git -C "$directory" submodule update --init --recursive
    fi
}

if [[ ! -x "$tools/bin/psxavenc" || $repair == true ]]; then
    brew install meson ninja pkg-config ffmpeg
    source_at psxavenc https://github.com/WonderfulToolchain/psxavenc.git v0.3.1
    # FFmpeg 7 stopped pulling libavutil/mem.h in transitively, which psxavenc
    # v0.3.1 still relies on for av_malloc/av_freep in mdec.c.
    mdec="$tools/src/psxavenc/psxavenc/mdec.c"
    if ! grep -q 'libavutil/mem.h' "$mdec"; then
        sed -i '' '/^#include <libavcodec\/avdct.h>$/a\
#include <libavutil/mem.h>
' "$mdec"
        grep -q 'libavutil/mem.h' "$mdec" || {
            echo "Could not add the libavutil/mem.h include to $mdec." >&2
            exit 1
        }
    fi
    meson setup "$tools/src/psxavenc/build" "$tools/src/psxavenc" --buildtype release --wipe
    meson compile -C "$tools/src/psxavenc/build"
    install -m 755 "$tools/src/psxavenc/build/psxavenc" "$tools/bin/psxavenc"
fi

if [[ ! -x "$tools/bin/mkpsxiso" || $repair == true ]]; then
    brew install cmake
    source_at mkpsxiso https://github.com/Lameguy64/mkpsxiso.git v2.30
    cmake -S "$tools/src/mkpsxiso" -B "$tools/src/mkpsxiso/build" -DCMAKE_BUILD_TYPE=Release
    cmake --build "$tools/src/mkpsxiso/build"
    install -m 755 "$tools/src/mkpsxiso/build/mkpsxiso" "$tools/bin/mkpsxiso"
fi

redux_dir="$tools/redux"
redux_config_path=""
redux_binary=""
for name in pcsx-redux PCSX-Redux; do
    candidate=".tools/macos/redux/PCSX-Redux.app/Contents/MacOS/$name"
    if [[ -x "$root/$candidate" ]]; then
        redux_config_path="$candidate"
        redux_binary="$root/$candidate"
        break
    fi
done
if [[ -z "$redux_binary" ]]; then
    echo "Downloading PCSX-Redux for macOS Apple Silicon..."
    redux_temp="$(mktemp -d "${TMPDIR:-/tmp}/epok-redux.XXXXXX")"
    cleanup_redux() { rm -rf "$redux_temp"; }
    trap cleanup_redux EXIT
    catalog="$redux_temp/catalog.json"
    curl --fail --location --retry 2 --silent --show-error \
        "https://distrib.app/storage/manifests/pcsx-redux/dev-macos-arm/manifest.json" \
        --output "$catalog"
    build_id="$(python3 - "$catalog" <<'PY'
import json, sys
builds = json.load(open(sys.argv[1], encoding="utf-8")).get("builds", [])
if not builds or not isinstance(builds[0].get("id"), int):
    raise SystemExit("PCSX-Redux catalog has no usable macOS Arm build")
print(builds[0]["id"])
PY
)"
    manifest="$redux_temp/manifest.json"
    curl --fail --location --retry 2 --silent --show-error \
        "https://distrib.app/storage/manifests/pcsx-redux/dev-macos-arm/manifest-${build_id}.json" \
        --output "$manifest"
    mapfile_output="$(python3 - "$manifest" <<'PY'
import json, sys
data = json.load(open(sys.argv[1], encoding="utf-8"))
path = data.get("path")
sha1 = data.get("hashes", {}).get("sha1")
if not isinstance(path, str) or not path.startswith("/") or not isinstance(sha1, str):
    raise SystemExit("PCSX-Redux build manifest is incomplete")
print("https://distrib.app" + path)
print(sha1)
PY
)"
    redux_url="$(printf '%s\n' "$mapfile_output" | sed -n '1p')"
    redux_sha1="$(printf '%s\n' "$mapfile_output" | sed -n '2p')"
    archive="$redux_temp/PCSX-Redux.dmg"
    # Keep curl's actual transfer meter in the editor's live install log.
    curl --fail --location --retry 2 --progress-bar "$redux_url" --output "$archive"
    actual_sha1="$(shasum -a 1 "$archive" | awk '{print $1}')"
    [[ "$actual_sha1" == "$redux_sha1" ]] || {
        echo "PCSX-Redux download failed its SHA-1 integrity check." >&2
        exit 1
    }
    mount_plist="$redux_temp/mount.plist"
    hdiutil attach -nobrowse -readonly -plist "$archive" > "$mount_plist"
    mountpoint="$(python3 - "$mount_plist" <<'PY'
import plistlib, sys
for entity in plistlib.load(open(sys.argv[1], "rb")).get("system-entities", []):
    point = entity.get("mount-point")
    if point:
        print(point)
        break
else:
    raise SystemExit("PCSX-Redux disk image did not mount")
PY
)"
    redux_app="$(find "$mountpoint" -maxdepth 3 -type d -name 'PCSX-Redux.app' -print -quit)"
    [[ -n "$redux_app" ]] || {
        hdiutil detach "$mountpoint" >/dev/null
        echo "PCSX-Redux disk image did not contain PCSX-Redux.app." >&2
        exit 1
    }
    staged_app="$redux_temp/PCSX-Redux.app"
    ditto "$redux_app" "$staged_app"
    hdiutil detach "$mountpoint" >/dev/null
    mkdir -p "$redux_dir"
    rm -rf "$redux_dir/PCSX-Redux.app"
    mv "$staged_app" "$redux_dir/PCSX-Redux.app"
    for name in pcsx-redux PCSX-Redux; do
        candidate=".tools/macos/redux/PCSX-Redux.app/Contents/MacOS/$name"
        if [[ -x "$root/$candidate" ]]; then
            redux_config_path="$candidate"
            redux_binary="$root/$candidate"
            break
        fi
    done
fi

if [[ -n "$redux_binary" ]]; then
    emulator="$redux_config_path"
else
    emulator=""
    for candidate in \
        /Applications/PCSX-Redux.app/Contents/MacOS/pcsx-redux \
        /Applications/PCSX-Redux.app/Contents/MacOS/PCSX-Redux; do
        if [[ -x "$candidate" ]]; then
            emulator="$candidate"
            break
        fi
    done
    [[ -n "$emulator" ]] || { echo "PCSX-Redux installation did not produce an executable." >&2; exit 1; }
fi

# C++ script reflection loads libclang.dylib from a directory. Xcode and the
# Command Line Tools both ship one, and Homebrew's llvm is the fallback.
libclang=""
for candidate in \
    "$(xcode-select -p 2>/dev/null)/Toolchains/XcodeDefault.xctoolchain/usr/lib" \
    "$(xcode-select -p 2>/dev/null)/usr/lib" \
    "$(brew --prefix llvm 2>/dev/null)/lib"; do
    if [[ -f "$candidate/libclang.dylib" ]]; then
        libclang="$candidate"
        break
    fi
done
if [[ -z "$libclang" ]]; then
    brew install llvm
    libclang="$(brew --prefix llvm)/lib"
fi
[[ -f "$libclang/libclang.dylib" ]] || {
    echo "Could not locate libclang.dylib for C++ script reflection." >&2
    exit 1
}

cat > "$root/Local.epokconfig" <<EOF
make: make
toolchain_bin: ""
nugget: third_party/nugget
emulator: "$emulator"
psxavenc: .tools/macos/bin/psxavenc
mkpsxiso: .tools/macos/bin/mkpsxiso
libclang: "$libclang"
code: ""
web_port: 8077
auto_build: true
EOF

# Include the same verified offline payload used by Windows distributions.
python3 "$root/tools/bundle-serial.py"
echo "Ready. Run make run or ./tools/run-macos.sh, then Play."
