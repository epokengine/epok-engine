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
# shellcheck source=tools/macos-dependencies.sh
source "$root/tools/macos-dependencies.sh"
tools="$root/$macos_tools"

# Finder starts the editor with a system PATH. Use Apple Silicon Homebrew
# there as well as in shells that already have their own Homebrew on PATH.
export PATH="$PATH:/opt/homebrew/bin:/opt/homebrew/sbin"

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

sdk_revision="$(nugget_revision "$root")"
toolchain_bin="$(macos_toolchain_bin)"
emulator="$(
    redux_executable "$root/$macos_redux_app" || redux_executable /Applications/PCSX-Redux.app || true
)"
emulator="${emulator#"$root/"}"
config="$root/Local.epokconfig"
if [[ ! -e "$config" ]]; then
    # These paths become usable as each tool is installed, including on a
    # partial first run. Keep any existing configuration until the tools are ready.
    write_macos_config "$config" "$toolchain_bin" "${emulator:-$macos_redux_app/Contents/MacOS/PCSX-Redux}"
fi

prepare_nugget "$root" "$sdk_revision"

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

mkdir -p "$tools/src" "$tools/bin"

if [[ ! -x "$toolchain_bin/mipsel-none-elf-g++" || ! -x "$toolchain_bin/mipsel-none-elf-objcopy" ]]; then
    echo "Installing the PCSX-Redux MIPS toolchain through Homebrew..."
    brew install pcsx-redux/mips/mipsel-none-elf-gcc
    brew_prefix="$(brew --prefix)"
    toolchain_bin="$brew_prefix/bin"
    [[ -x "$toolchain_bin/mipsel-none-elf-g++" && -x "$toolchain_bin/mipsel-none-elf-objcopy" ]] || {
        echo "Homebrew did not link the MIPS compiler and binutils in $toolchain_bin." >&2
        exit 1
    }
fi

source_at() {
    local url="$2" ref="$3" directory="$tools/src/$1"
    if [[ ! -e "$directory/.git" ]]; then
        git clone --depth 1 --branch "$ref" "$url" "$directory"
    elif $repair || ! git -C "$directory" rev-parse --verify --quiet "refs/tags/$ref" >/dev/null; then
        git -C "$directory" fetch --depth 1 origin "refs/tags/$ref:refs/tags/$ref"
    fi
    git -C "$directory" checkout --detach "refs/tags/$ref"
    # A previous run can have cloned the parent repository before a nested
    # download failed. Finish the checkout every time we need to build it.
    git -C "$directory" submodule update --init --checkout --recursive
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

install_redux() (
    # PCSX-Redux publishes this catalog for its own macOS updater. Individual
    # nightly URLs expire, so resolve the download from the catalog each time.
    local host="https://distrib.app" catalog build metadata asset sha3
    catalog="$host/storage/manifests/pcsx-redux/dev-macos-arm"
    build="$(curl -fsSL --retry 2 "$catalog/manifest.json" | python3 -c '
import json, sys
print(max(int(build["id"]) for build in json.load(sys.stdin)["builds"]))
')"
    metadata="$(curl -fsSL --retry 2 "$catalog/manifest-$build.json" | python3 -c '
import json, re, sys
build = json.load(sys.stdin)
path, sha3 = build["path"], build["hashes"]["sha3"]
if not re.fullmatch(r"/storage/assets/[0-9a-f/]+/PCSX-Redux-[0-9a-f]{8}-Arm\.dmg", path):
    raise SystemExit("PCSX-Redux catalog did not contain a macOS Arm disk image.")
if not re.fullmatch(r"[0-9a-f]{64}", sha3):
    raise SystemExit("PCSX-Redux catalog did not contain a valid SHA3-256 checksum.")
print(path, sha3)
')"
    read -r asset sha3 <<< "$metadata"

    local destination="$root/$macos_redux_app" work mountpoint mounted=false
    mkdir -p "$(dirname "$destination")"
    work="$(mktemp -d "$(dirname "$destination")/.install.XXXXXX")"
    mountpoint="$work/mount"
    cleanup_redux() {
        if $mounted && ! hdiutil detach "$mountpoint" >/dev/null; then
            return 1
        fi
        rm -rf "$work"
    }
    trap cleanup_redux EXIT

    local archive="$work/PCSX-Redux.dmg" staged_app="$work/PCSX-Redux.app"
    echo "Downloading ${asset##*/} (build $build)..."
    curl --fail --location --retry 2 --progress-bar "$host$asset" --output "$archive"
    python3 - "$archive" "$sha3" <<'PY'
import hashlib
import sys

digest = hashlib.sha3_256()
with open(sys.argv[1], "rb") as archive:
    for chunk in iter(lambda: archive.read(1024 * 1024), b""):
        digest.update(chunk)
if digest.hexdigest() != sys.argv[2]:
    raise SystemExit("PCSX-Redux download failed its SHA3-256 integrity check.")
PY
    mkdir "$mountpoint"
    hdiutil attach -nobrowse -readonly -mountpoint "$mountpoint" "$archive" >/dev/null
    mounted=true
    ditto "$mountpoint/PCSX-Redux.app" "$staged_app"
    hdiutil detach "$mountpoint" >/dev/null
    mounted=false
    redux_executable "$staged_app" >/dev/null || {
        echo "PCSX-Redux disk image did not contain a usable executable." >&2
        exit 1
    }
    rm -rf "$destination"
    mv "$staged_app" "$destination"
)

# Keep an existing executable during setup and repair so emulator timings
# remain comparable across runs of the same installation.
if [[ -z "$emulator" ]]; then
    install_redux
    emulator="$(redux_executable "$root/$macos_redux_app")"
    emulator="${emulator#"$root/"}"
fi

# C++ script reflection accepts only the pinned libclang 18.1.1, so the one
# Xcode or Homebrew happens to ship is never a substitute: the header tool
# rejects any other version. The pinned wheel is the same one the other hosts
# use, verified by hash and extracted as an archive.
libclang_dir="$tools/libclang"
if [[ ! -f "$root/$macos_libclang/libclang.dylib" || $repair == true ]]; then
    echo "Downloading pinned libclang 18.1.1 for macOS Apple Silicon..."
    libclang_temp="$(mktemp -d "${TMPDIR:-/tmp}/epok-libclang.XXXXXX")"
    wheel="$libclang_temp/libclang-18.1.1-py2.py3-none-macosx_11_0_arm64.whl"
    curl --fail --location --retry 2 --silent --show-error \
        "https://files.pythonhosted.org/packages/db/ed/1df62b44db2583375f6a8a5e2ca5432bbdc3edb477942b9b7c848c720055/libclang-18.1.1-py2.py3-none-macosx_11_0_arm64.whl" \
        --output "$wheel"
    [[ "$(shasum -a 256 "$wheel" | awk '{print $1}')" == 83ce5045d101b669ac38e6da8e58765f12da2d3aafb3b9b98d88b286a60964d8 ]] || {
        rm -rf "$libclang_temp"
        echo "libclang download failed its SHA-256 integrity check." >&2
        exit 1
    }
    rm -rf "$libclang_dir"
    mkdir -p "$libclang_dir"
    ditto -x -k "$wheel" "$libclang_dir"
    rm -rf "$libclang_temp"
fi
[[ -f "$root/$macos_libclang/libclang.dylib" ]] || {
    echo "Could not locate the pinned libclang.dylib for C++ script reflection." >&2
    exit 1
}

write_macos_config "$config" "$toolchain_bin" "$emulator"
# Include the same verified offline payload used by Windows distributions.
python3 "$root/tools/bundle-serial.py"
echo "Ready. Run make run or ./tools/run-macos.sh, then Play."
