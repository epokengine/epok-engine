#!/usr/bin/env bash
# Build Epok as a portable Finder application bundle for macOS.
set -euo pipefail

[[ "$(uname -s)" == "Darwin" ]] || {
    echo "make app is available on macOS only." >&2
    exit 1
}

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=tools/macos-dependencies.sh
source "$root/tools/macos-dependencies.sh"
cargo="$root/tools/cargo-macos.sh"
output="$root/target/release/EpokEngine.app"
work="$(mktemp -d "${TMPDIR:-/tmp}/epok-app.XXXXXX")"
bundle="$work/EpokEngine.app"
home="$bundle/Contents/Resources/Epok"
desktop="$HOME/Desktop/EpokEngine.app"

cleanup() {
    rm -rf "$work"
}
trap cleanup EXIT

require_file() {
    [[ -f "$1" ]] || {
        echo "Missing $2: $1" >&2
        exit 1
    }
}

sdk="$root/third_party/nugget"
sdk_revision="$(nugget_revision "$root")"
prepare_nugget "$root" "$sdk_revision"

echo "Compiling the optimized Epok editor..."
"$cargo" build --locked --release --bins

editor="$root/target/release/epok-editor"
extractor="$root/target/release/epok-header-tool"
require_file "$editor" "the release editor binary"
require_file "$extractor" "the release reflection extractor"

mkdir -p "$bundle/Contents/MacOS" "$home/third_party/nugget" "$home/$macos_tools/bin"
install -m 755 "$editor" "$bundle/Contents/MacOS/epok-editor"
install -m 755 "$extractor" "$bundle/Contents/MacOS/epok-header-tool"

# Include each available local tool. The same relative paths also let the
# bundle discover tools installed later from the Dependencies panel.
for binary in psxavenc mkpsxiso; do
    if [[ -x "$root/$macos_tools/bin/$binary" ]]; then
        install -m 755 "$root/$macos_tools/bin/$binary" "$home/$macos_tools/bin/$binary"
    fi
done
if [[ -d "$root/$macos_redux_app" ]]; then
    mkdir -p "$home/$macos_tools/redux"
    ditto "$root/$macos_redux_app" "$home/$macos_redux_app"
fi
if [[ -f "$root/$macos_libclang/libclang.dylib" ]]; then
    ditto "$root/$macos_tools/libclang" "$home/$macos_tools/libclang"
fi

# Finder launches do not inherit a shell's Homebrew PATH. Record the compiler
# location so native Play works from the copied app too.
toolchain_bin="$(macos_toolchain_bin)"
emulator="$(redux_executable "$home/$macos_redux_app" || redux_executable /Applications/PCSX-Redux.app || true)"
emulator="${emulator#"$home/"}"
write_macos_config "$home/Editor.epokconfig" "$toolchain_bin" \
    "${emulator:-$macos_redux_app/Contents/MacOS/PCSX-Redux}"
chmod 644 "$home/Editor.epokconfig"

# Archive the pinned trees so the app can move without their Git metadata.
sdk_snapshot="$home/third_party/nugget"
git -C "$sdk" archive "$sdk_revision" | tar -xf - -C "$sdk_snapshot"
psxlua_revision="$(git -C "$sdk" rev-parse "$sdk_revision:third_party/psxlua")"
mkdir -p "$sdk_snapshot/third_party/psxlua"
git -C "$sdk/third_party/psxlua" archive "$psxlua_revision" | tar -xf - -C "$sdk_snapshot/third_party/psxlua"
printf '%s\n' "$sdk_revision" > "$sdk_snapshot/.epok-pinned-revision"
install -m 644 "$root/LICENSE" "$home/LICENSE"
mkdir -p "$home/tools"
install -m 755 "$root/tools/setup-macos.sh" "$home/tools/setup-macos.sh"
install -m 755 "$root/tools/bundle-serial.py" "$home/tools/bundle-serial.py"
install -m 644 "$root/tools/macos-dependencies.sh" "$home/tools/macos-dependencies.sh"
install -m 644 "$root/tools/dependencies.json" "$home/tools/dependencies.json"
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
