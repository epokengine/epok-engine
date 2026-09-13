#!/usr/bin/env bash
# Provision Epok's verified Linux x86_64 dependencies without changing PATH or
# installing system packages. The MIPS compiler is built into .tools/linux.
set -euo pipefail

[[ "$(uname -s)" == Linux ]] || { echo "This script must run on Linux." >&2; exit 1; }
[[ "$(uname -m)" == x86_64 ]] || { echo "Linux setup currently supports x86_64 only." >&2; exit 1; }

repair=false
dependency=""
while (($#)); do
    case "$1" in
        --repair) repair=true ;;
        --dependency)
            shift
            dependency=${1:-}
            [[ -n "$dependency" ]] || { echo "--dependency needs a package name." >&2; exit 2; }
            ;;
        *) echo "Usage: $0 [--repair] [--dependency mips|redux|psxavenc|mkpsxiso|libclang|nugget]" >&2; exit 2 ;;
    esac
    shift
done

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tools="$root/.tools/linux"
archives="$tools/archives"
sdk="$root/third_party/nugget"
sdk_revision="6186b131aacc5853a9161fb076ed34ffe504552d"

require() {
    command -v "$1" >/dev/null 2>&1 || { echo "Missing required command: $1" >&2; exit 1; }
}
for command in bash curl git make g++ tar xz bzip2 python3 sha256sum; do require "$command"; done
mkdir -p "$archives"

selected() {
    [[ -z "$dependency" || "$dependency" == "$1" ]]
}

download() {
    local name="$1" url="$2" hash="$3"
    local archive="$archives/$name"
    if [[ ! -f "$archive" ]]; then
        local temporary="$archive.download"
        echo "Downloading $name" >&2
        curl --fail --location --retry 3 --silent --show-error "$url" -o "$temporary"
        printf '%s  %s\n' "$hash" "$temporary" | sha256sum --check --status || {
            echo "Checksum mismatch: $name" >&2
            exit 1
        }
        mv "$temporary" "$archive"
    fi
    printf '%s  %s\n' "$hash" "$archive" | sha256sum --check --status || {
        echo "Cached archive checksum mismatch: $archive" >&2
        exit 1
    }
    printf '%s\n' "$archive"
}

# Zip extraction is deliberately checked entry-by-entry. It never follows
# links, writes outside the package directory, or removes user-added files.
install_zip() {
    local archive="$1" destination="$2"
    python3 - "$archive" "$destination" "$repair" <<'PY'
import hashlib
import os
import stat
import sys
import zipfile

archive, destination, repair = sys.argv[1], os.path.abspath(sys.argv[2]), sys.argv[3] == "true"
if os.path.islink(destination):
    raise SystemExit(f"Package destination is a symbolic link: {destination}")
fresh = not os.path.exists(destination)
with zipfile.ZipFile(archive) as package:
    entries = [entry for entry in package.infolist() if not entry.is_dir()]
    for entry in entries:
        name = entry.filename
        if not name or name.startswith(("/", "\\")) or ".." in name.replace("\\", "/").split("/"):
            raise SystemExit(f"Archive entry escapes its destination: {name}")
        mode = entry.external_attr >> 16
        if stat.S_ISLNK(mode):
            raise SystemExit(f"Archive contains a symbolic link: {name}")
        target = os.path.abspath(os.path.join(destination, name))
        if os.path.commonpath((destination, target)) != destination:
            raise SystemExit(f"Archive entry escapes its destination: {name}")
        expected = hashlib.sha256(package.read(entry)).digest()
        if os.path.islink(target):
            raise SystemExit(f"Package file is a symbolic link: {target}")
        matches = os.path.isfile(target) and hashlib.file_digest(open(target, "rb"), "sha256").digest() == expected
        if matches:
            continue
        if not (fresh or repair):
            raise SystemExit(f"Changed or missing package file: {name}. Run with --repair.")
        parent = os.path.dirname(target)
        ancestor = parent
        while ancestor != destination:
            if os.path.islink(ancestor):
                raise SystemExit(f"Package path crosses a symbolic link: {ancestor}")
            ancestor = os.path.dirname(ancestor)
        os.makedirs(parent, exist_ok=True)
        with package.open(entry) as source, open(target, "wb") as output:
            output.write(source.read())
        os.chmod(target, mode & 0o777 or 0o644)
print(f"Verified {os.path.basename(destination)}")
PY
}

install_nugget() {
    if [[ -d "$root/.git" && ! -e "$sdk/.git" ]]; then
        git -C "$root" -c submodule.recurse=false submodule update --init -- third_party/nugget
    fi
    [[ -e "$sdk/.git" ]] || { echo "Nugget is missing; initialize third_party/nugget first." >&2; exit 1; }
    [[ "$(git -C "$sdk" rev-parse HEAD)" == "$sdk_revision" ]] || {
        echo "Nugget is not at the pinned revision $sdk_revision." >&2
        exit 1
    }
    [[ -z "$(git -C "$sdk" status --porcelain --untracked-files=normal)" ]] || {
        echo "Nugget has local source changes. Preserve them before setup." >&2
        exit 1
    }
    for file in psyqo/psyqo.mk common.mk third_party/EASTL/include/EASTL/array.h third_party/EABase/include/Common/EABase/eabase.h; do
        [[ -f "$sdk/$file" ]] || { echo "Incomplete Nugget checkout: $file" >&2; exit 1; }
    done
    echo "Verified Nugget $sdk_revision"
}

build_mips() {
    local compiler="$tools/mips/bin/mipsel-none-elf-g++"
    [[ -x "$compiler" && "$repair" == false ]] && return
    local work stage build
    work="$(mktemp -d "$tools/mips-source.XXXXXX")"
    stage="$(mktemp -d "$tools/mips-stage.XXXXXX")"
    build="$work/gcc-build"
    trap 'rm -rf "$work" "$stage"' RETURN
    echo "Building the pinned MIPS compiler locally. This can take several minutes."
    curl --fail --location --retry 3 --silent --show-error \
        https://mirrors.kernel.org/gnu/binutils/binutils-2.47.tar.gz -o "$work/binutils.tar.gz"
    printf '%s  %s\n' \
        15f5abd0db9cf8ea116f516a9ce12bc1cefef491ef9e50942868093d8f92a6f3 \
        "$work/binutils.tar.gz" | sha256sum --check --status
    curl --fail --location --retry 3 --silent --show-error \
        https://mirrors.kernel.org/gnu/gcc/gcc-16.2.0/gcc-16.2.0.tar.gz -o "$work/gcc.tar.gz"
    printf '%s  %s\n' \
        071d00a097579e5ef7ce97fc4a9e58e73fd3503c0a013c765c970370a5a53b9b \
        "$work/gcc.tar.gz" | sha256sum --check --status
    tar -C "$work" -xf "$work/binutils.tar.gz"
    tar -C "$work" -xf "$work/gcc.tar.gz"
    (
        cd "$work/gcc-16.2.0"
        ./contrib/download_prerequisites --directory="$work/gcc-16.2.0" --no-isl
    )
    mkdir "$work/binutils-build" "$build"
    (
        cd "$work/binutils-build"
        "$work/binutils-2.47/configure" --target=mipsel-none-elf --disable-multilib --disable-nls --disable-werror --prefix="$stage"
        make -j"$(nproc)"
        make install-strip
    )
    (
        cd "$build"
        "$work/gcc-16.2.0/configure" --target=mipsel-none-elf --without-isl --disable-nls --disable-threads --disable-shared --disable-libssp --disable-libstdcxx-pch --disable-libgomp --disable-werror --without-headers --disable-hosted-libstdcxx --with-as="$stage/bin/mipsel-none-elf-as" --with-ld="$stage/bin/mipsel-none-elf-ld" --enable-languages=c,c++ --prefix="$stage"
        make -j"$(nproc)" all-gcc
        make install-strip-gcc
        make -j"$(nproc)" all-target-libgcc all-target-libstdc++-v3
        make install-strip-target-libgcc install-strip-target-libstdc++-v3
    )
    [[ -x "$stage/bin/mipsel-none-elf-g++" ]] || { echo "MIPS compiler build did not produce mipsel-none-elf-g++." >&2; exit 1; }
    if [[ -e "$tools/mips" ]]; then
        local backup="$tools/mips.backup-$(date +%s)"
        mv "$tools/mips" "$backup"
        echo "Preserved previous MIPS package as $backup"
    fi
    mv "$stage" "$tools/mips"
    trap - RETURN
    rm -rf "$work"
    echo "Verified MIPS compiler 16.2.0"
}

if selected nugget; then install_nugget; fi
if selected mips; then build_mips; fi
if selected psxavenc; then
    install_zip "$(download psxavenc-linux.zip https://github.com/WonderfulToolchain/psxavenc/releases/download/v0.3.1/psxavenc-linux.zip ad22b887683631149fb8c7c85ecea8bb760dd482f9e18cce2b35b11319e95e51)" "$tools/psxavenc"
fi
if selected mkpsxiso; then
    install_zip "$(download mkpsxiso-2.30-Linux.zip https://github.com/Lameguy64/mkpsxiso/releases/download/v2.30/mkpsxiso-2.30-Linux.zip 8e387e6db51dee3eb7cb08f4d59293a24c48f58bd541631acf9301d78015a1fc)" "$tools/mkpsxiso"
fi
if selected redux; then
    install_zip "$(download pcsx-redux-dd0b35d1-linux-x86_64.zip https://distrib.app/storage/assets/aa6/6fc/129/f58473c46a3b85efd5e6c86660166a8093780728652a7e0b8e21f6d/PCSX-Redux-dd0b35d1-linux-x86_64.zip 018dcadaaeb62b60b567479e0e2add67bac13e158ae1e295527bb6181131c422)" "$tools/redux"
fi
if selected libclang; then
    install_zip "$(download libclang-18.1.1-manylinux2010-x86_64.whl https://files.pythonhosted.org/packages/1d/fc/716c1e62e512ef1c160e7984a73a5fc7df45166f2ff3f254e71c58076f7c/libclang-18.1.1-py2.py3-none-manylinux2010_x86_64.whl c533091d8a3bbf7460a00cb6c1a71da93bffe148f172c7d03b1c31fbf8aa2a0b)" "$tools/libclang"
fi

if [[ -z "$dependency" ]]; then
    cat > "$root/Local.epokconfig" <<'EOF'
make: make
toolchain_bin: .tools/linux/mips/bin
nugget: third_party/nugget
emulator: .tools/linux/redux/PCSX-Redux-HEAD-x86_64.AppImage
psxavenc: .tools/linux/psxavenc/bin/psxavenc
mkpsxiso: .tools/linux/mkpsxiso/mkpsxiso-2.30-Linux/bin/mkpsxiso
libclang: .tools/linux/libclang/clang/native
code: ''
web_port: 8077
auto_build: true
EOF
    echo "Ready. Run make run, then Play."
fi
