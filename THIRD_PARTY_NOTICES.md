# Third-party notices

Epok's original source code is MIT licensed. This does not replace the licenses of dependencies, fonts, icons or user-authored game content.

## Editor resources

| Resource | Attribution and license | Included notice |
| --- | --- | --- |
| Codicons, version 0.0.46-24 | Microsoft and contributors; icon/content license CC BY 4.0, repository code MIT | [Codicons license](resources/editor/LICENSE-codicons.txt) |
| Roboto Regular and Bold | Google and contributors; Apache License 2.0 | [Roboto license](resources/editor/LICENSE-roboto.txt) |
| Material Design Icons function glyph, version 7.4.47 | Pictogrammers and contributors; Apache License 2.0 | [MDI license](resources/editor/LICENSE-blueprint-function-icon.txt) |
| Font Awesome Free desktop font | Fonticons, Inc.; SIL Open Font License 1.1 for the font | [Font Awesome license](resources/editor/LICENSE-fontawesome.txt) |
| mig68000 HUD bitmap font | Zingot Games; attribution retained from the pinned PsyQo source, author lists FontPack as CC BY 4.0 | [HUD font notice](resources/editor/LICENSE-psx-font.txt) |

Codicons legal notices distinguish content from code: https://github.com/microsoft/vscode-codicons#legal-notices

The included Codicons font is a subset of version 0.0.46-24 containing the glyphs used by the editor. Unused glyphs and glyph-name metadata were removed with fonttools 4.60.1; the retained designs are unchanged. See [resource maintenance](knowledge/maintainers/resources.md) for regeneration.

Font Awesome's distribution includes different licenses for different formats. Epok uses the TTF font: https://fontawesome.com/license/free

The Blueprint action menu uses unmodified Roboto fonts from googlefonts/roboto-2, revision `38062f4b4a0be4346d07a928408da21602545e9e`. Its function icon is the unmodified `function` design from `@mdi/font` 7.4.47, subset to one glyph and remapped from U+F0295 to U+E900 for ImGui's 16-bit font atlas. These open fonts and icons are independent of Epic's artwork. Codicons was regenerated for the action-menu symbols on 2026-09-10.

The HUD bitmap is decoded from the pinned PsyQo system font for matching editor/console glyphs. Its representation was converted; credit remains with Zingot Games. Author: https://www.zingot.com/ and https://zingot.itch.io/fontpack

Segoe UI is loaded from the Windows installation when available. Its font file is not distributed in this repository.

## Runtime and SDK

Nugget is referenced as an upstream submodule at the revision in [dependencies.json](tools/dependencies.json). PsyQo uses its common runtime code and included EASTL/EABase sources. See [runtime notices](runtime/THIRD_PARTY_NOTICES.md) and the original license texts in [runtime/licenses](runtime/licenses).

The repository does not distribute a proprietary PlayStation BIOS or game images.

## External development tools

Host C++ reflection uses libclang 18.1.1 (Apache-2.0 WITH LLVM-exception), distributed through the pinned `libclang` wheel. Setup extracts it as an archive; Python is not required to execute the extractor. Preserve `libclang-18.1.1.dist-info/LICENSE.TXT` with binary distributions. The DLL is loaded only by `epok-header-tool`, never by a PSX game or standalone export. The Rust `clang` 2.0.0 wrapper and `clang-sys` are pinned in Cargo.lock and carry their own package notices.

PCSX-Redux and the MIPS toolchain are downloaded separately by setup and remain outside version control. PCSX-Redux is GPL-2.0 licensed and includes additional third-party notices. OpenBIOS is included in that upstream distribution. Preserve those distributions' license files when preparing a binary package.

- PCSX-Redux: https://github.com/grumpycoders/pcsx-redux
- Distribution notices: https://github.com/grumpycoders/pcsx-redux/blob/main/LICENSES.md
- MIPS toolchain setup: https://github.com/grumpycoders/pcsx-redux/blob/main/mips.ps1

## Rust dependencies

Direct dependencies are imgui, imgui-winit-support, imgui-wgpu, wgpu, pollster, winit, raw-window-handle, serde, serde_json, png, sha2, uuid, symphonia, ufbx, rmcp, tokio, tokio-util, axum, base64, serde_ignored and arboard. Exact direct/transitive versions are recorded in Cargo.lock; source packages carry their own license texts. Dear ImGui is provided by the imgui bindings and is MIT licensed.

This source repository does not vendor those Rust packages. Binary distributions must include the notices applicable to the resolved libraries and embedded resources.

- Dear ImGui: https://github.com/ocornut/imgui
- Rust bindings: https://github.com/imgui-rs/imgui-rs
- wgpu: https://github.com/gfx-rs/wgpu
- Official Rust MCP SDK (rmcp): https://github.com/modelcontextprotocol/rust-sdk - linked into the desktop editor for local AI connections; not included in PSX games.

## Audio import and disc tools

- Symphonia 0.5.5 (MPL-2.0): https://github.com/pdeljanov/Symphonia/tree/v0.5.5 — linked into the desktop editor for audio decoding; not linked into the console runtime.
- psxavenc 0.3.1 (zlib license, Ben Russell and Adrian Siekierka): https://github.com/WonderfulToolchain/psxavenc/tree/v0.3.1 — separate desktop conversion executable. Its distribution also includes FFmpeg components, which retain their respective licenses.
- mkpsxiso 2.30 (GPL-2.0): https://github.com/Lameguy64/mkpsxiso/tree/v2.30 — separate desktop disc authoring executable; the downloaded distribution retains its `doc/LICENSE.md` and dependency notices.

The external tools are pinned in `tools/dependencies.json` and installed under `.tools/`; they are not embedded into exported games. Preserve upstream notices/source-availability requirements when redistributing the tools or editor dependencies. The synthetic MP3 fixture is original test data described in `tests/fixtures/README.md`.


## FBX importer and sample character

- ufbx Rust bindings 0.11.3: https://github.com/ufbx/ufbx-rust - MIT (the Rust bindings also offer PDDL-1.0); includes the ufbx C parser under its MIT license. It is linked into the desktop editor only. Preserve the upstream license text in binary editor distributions. No ufbx code is linked into PSX games.
- `resources/models/EpokMannequin.fbx` is original Epok content under the repository MIT license. See [skeletal characters](docs/skeletal.md) for its generation script and usage.
