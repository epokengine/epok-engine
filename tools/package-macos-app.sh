#!/usr/bin/env bash
# Build Epok as a portable Finder application bundle for macOS.
set -euo pipefail

[[ "$(uname -s)" == "Darwin" ]] || {
    echo "make app is available on macOS only." >&2
    exit 1
}

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo="$root/tools/cargo-macos.sh"
output="$root/target/release/Epok Engine.app"
work="$(mktemp -d "${TMPDIR:-/tmp}/epok-app.XXXXXX")"
bundle="$work/Epok Engine.app"
home="$bundle/Contents/Resources/Epok"
desktop="$HOME/Desktop/Epok Engine.app"

cleanup() {
    rm -rf "$work"
}
trap cleanup EXIT

require_directory() {
    [[ -d "$1" ]] || {
        echo "Missing $2: $1" >&2
        exit 1
    }
}

require_file() {
    [[ -f "$1" ]] || {
        echo "Missing $2: $1" >&2
        exit 1
    }
}

require_directory "$root/third_party/nugget" "the pinned Nugget SDK"

echo "Compiling the optimized Epok editor..."
"$cargo" build --locked --release --bins

editor="$root/target/release/epok-editor"
extractor="$root/target/release/epok-header-tool"
require_file "$editor" "the release editor binary"
require_file "$extractor" "the release reflection extractor"

mkdir -p "$bundle/Contents/MacOS" "$home/third_party" "$home/.tools/macos"
install -m 755 "$editor" "$bundle/Contents/MacOS/epok-editor"
install -m 755 "$extractor" "$bundle/Contents/MacOS/epok-header-tool"

# Keep optional local tools inside the bundle when setup has produced them. The
# editor otherwise resolves the usual command names from the user's PATH.
psxavenc="psxavenc"
mkpsxiso="mkpsxiso"
if [[ -x "$root/.tools/macos/bin/psxavenc" ]]; then
    ditto "$root/.tools/macos/bin" "$home/.tools/macos/bin"
    psxavenc=".tools/macos/bin/psxavenc"
    mkpsxiso=".tools/macos/bin/mkpsxiso"
fi
if [[ -d "$root/.tools/macos/redux/PCSX-Redux.app" ]]; then
    ditto "$root/.tools/macos/redux" "$home/.tools/macos/redux"
fi

# Finder launches do not inherit a shell's Homebrew PATH. Record the compiler
# location when it is available so native Play works from the copied app too.
toolchain_bin=""
if compiler="$(command -v mipsel-none-elf-g++ 2>/dev/null)"; then
    toolchain_bin="$(dirname "$compiler")"
fi
emulator="pcsx-redux"
for name in pcsx-redux PCSX-Redux; do
    if [[ -x "$home/.tools/macos/redux/PCSX-Redux.app/Contents/MacOS/$name" ]]; then
        emulator=".tools/macos/redux/PCSX-Redux.app/Contents/MacOS/$name"
        break
    fi
done
for candidate in \
    /Applications/PCSX-Redux.app/Contents/MacOS/pcsx-redux \
    /Applications/PCSX-Redux.app/Contents/MacOS/PCSX-Redux; do
    if [[ "$emulator" == "pcsx-redux" && -x "$candidate" ]]; then
        emulator="$candidate"
        break
    fi
done

# editor_home() recognizes this standard application resource location, so
# relative paths remain valid even after Finder moves the bundle.
cat > "$home/Editor.epokconfig" <<EOF
make: make
toolchain_bin: "$toolchain_bin"
nugget: third_party/nugget
emulator: "$emulator"
psxavenc: $psxavenc
mkpsxiso: $mkpsxiso
code: ""
web_port: 8077
auto_build: true
EOF

ditto "$root/third_party/nugget" "$home/third_party/nugget"
rm -rf "$home/third_party/nugget/.git"
printf '%s\n' '6186b131aacc5853a9161fb076ed34ffe504552d' > "$home/third_party/nugget/.epok-pinned-revision"
install -m 644 "$root/LICENSE" "$home/LICENSE"
mkdir -p "$home/tools"
install -m 755 "$root/tools/setup-macos.sh" "$home/tools/setup-macos.sh"
install -m 755 "$root/tools/bundle-serial.py" "$home/tools/bundle-serial.py"
install -m 644 "$root/tools/serial-dependency.json" "$home/tools/serial-dependency.json"

# macOS's native converter produces the ICNS directly from the checked-in
# transparent application icon. This is more portable across Xcode versions
# than assuming an iconutil iconset layout.
sips -s format icns "$root/resources/branding/epok.png" \
    --out "$bundle/Contents/Resources/EpokEngine.icns" >/dev/null

cat > "$bundle/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleDisplayName</key><string>Epok Engine</string>
  <key>CFBundleExecutable</key><string>epok-editor</string>
  <key>CFBundleIconFile</key><string>EpokEngine</string>
  <key>CFBundleIdentifier</key><string>com.epok.engine</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleName</key><string>Epok Engine</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$(awk -F '"' '/^version = / { print $2; exit }' "$root/Cargo.toml")</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
</dict></plist>
EOF

# An ad-hoc signature lets macOS verify the local bundle after it is copied.
codesign --force --deep --sign - "$bundle" >/dev/null

rm -rf "$output"
mv "$bundle" "$output"
echo "Bundle created: $output"

if [[ -t 0 ]]; then
    read -r -p "¿Copiar o reemplazar '$desktop'? [s/N] " answer
    case "$answer" in
        s|S|si|SI|sí|SÍ|y|Y|yes|YES)
            rm -rf "$desktop"
            ditto "$output" "$desktop"
            echo "Installed on Desktop: $desktop"
            ;;
        *)
            echo "The bundle remains at: $output"
            ;;
    esac
else
    echo "No interactive terminal; the bundle remains at: $output"
fi
