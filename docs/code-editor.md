# Integrated code editor

**Window > Code Editor** opens the Monaco workspace inside Epok. **Edit Lua
Class**, **Edit C++ Class**, source files in Project and source diagnostics open
the original file in this workspace. **Assets > Open Code Workspace** opens the
script tree. Editing does not require a separate VS Code installation.

The left tree lists Lua and C++ sources under `assets/scripts`, including nested
folders, and source files embedded in PocketStation apps. Filter by filename or
path. Files have independent tabs, undo histories and cursor positions. A dot
marks an unsaved tab; closing it offers Save, Discard and Cancel. SDK and generated
definitions reached through navigation are read-only.

**Save** / **Ctrl+S** writes the current file. **Save All** / **Ctrl+Shift+S** and
Epok's Save All write all open source buffers. The application and project close
dialogs include unsaved source. Saves check the on-disk revision and refuse to
overwrite an external change. Close and reopen a conflicted buffer after saving
or copying any text you want to retain.

## Language service setup

Click **Setup** in the Code toolbar. Each service supports **Install**, automatic
detection and an explicit executable path:

| Service | Features |
| --- | --- |
| Lua Language Server | Epok Lua and PocketLua completion, hover, signature help, definitions, references, rename, formatting and diagnostics. |
| clangd | C++ completion, navigation and diagnostics using the project's engine/SDK compilation options. |
| GitHub Copilot Language Server | Optional inline suggestions, integrated chat and GitHub sign-in. |

Install downloads an unmodified, versioned portable package from its official
GitHub release, verifies its pinned SHA-256 and stores it under the current
user's `Epok/CodeTools` directory. No global PATH change is needed. Apply saves
configuration in `CodeEditor.epokprefs` beside the other local Epok preferences
and restarts the services. These settings and tools do not travel with projects.

The editor and its Monaco assets are bundled with Epok and work offline. Language
services work offline after installation. Copilot requires internet access and
the user's GitHub Copilot entitlement; enable it, Apply and click **Sign in to
GitHub**. Complete the displayed device code flow in your browser. Signing out
is available from the same panel. Epok does not store a GitHub token in projects.

**Ctrl+Space** requests completion; **F12** opens a definition; the editor's context
menu provides references, rename and formatting. The Problems panel opens the
source position of a diagnostic. Copilot suggestions are accepted with Monaco's
inline suggestion controls, including Tab.

**Copilot Chat** opens a conversation beside Monaco. In compact panes the file
tree is hidden while chat is open; below 600 px chat fills the workspace. Close
chat to return to the source tree and editor.
Enable Copilot in Setup, click **Connect**, and use **Sign in** if needed. Chat
uses the same official server in ACP mode and can reuse an existing GitHub login.
**Include current file or selection** attaches the selected range, or the active
file when no range is selected. Turn it off for a question without a code
attachment. Attachments are limited to 128 KiB. Responses stream into the panel;
code blocks have Copy and Insert at cursor actions, and insertion uses Monaco's
normal Undo and save workflow. **Stop** cancels a response; **New chat** starts a
fresh conversation. Tool requests display their details with a decision for that
single request. No MCP servers or client shell capabilities are supplied.

**New** opens the engine's Lua/C++ class creation workflow. With a PocketStation
source selected, **New > Pocket Assembly** adds a companion Thumb source directly
to that app. **F5** saves source buffers and runs the current PocketStation app or
requests Play for gameplay sources.

## Engine and PocketStation APIs

Gameplay Lua uses the generated `.epok/lua/epok.d.lua` and the project's existing
`.luarc.json`. Definitions are regenerated from the same native, Lua and Blueprint
registry used by the engine. The Epok compiler remains the authority on the
supported Lua profile.

The Code tab in the PocketStation window hosts the same Monaco workspace.
PocketLua buffers save back into their `.epokasset`; source is not moved out of
the asset. Their files are mirrored under `.epok/code/pocket` solely for language
services. A separate LuaLS workspace and definitions generated from the PocketLua
compiler's builtin table keep `screen`, `button`, `sound`, `clock`, `save`, `link`,
`system` and its restricted `math` API separate from gameplay Lua. PocketStation
Build & Run saves code buffers first.

C++ uses `.epok/code/cpp/compile_commands.json`, derived from Epok's reflection
request and the actual project source paths, rather than the host compiler's
default include paths. Language tools, WebView and Monaco run only on the host;
they are not linked into PlayStation or PocketStation exports.

## Maintainer checks

Frontend sources live in `tools/code-editor`; generated, licensed assets live in
`resources/code-editor` and are compiled into the editor. Ordinary Rust builds
do not need Node.js or a network connection for the frontend.

```powershell
cd tools/code-editor
npm ci
npm run check
npm run build
node tests/native.mjs --install
node tests/native.mjs --chat --inline
node tests/native.mjs --pocket --chat --inline
```

The native acceptance runner uses `target/debug/epok-editor.exe`, an isolated
project and isolated WebView/preferences under `artifacts/code-editor/native`.
It connects to that WebView2's test-only DevTools port, exercises real UI actions,
retains screenshots and writes `report.json`. `--install` uses the same verified
portable installer as Setup. Subsequent runs reuse installed tools. Source buffer
and LSP framing regressions live in `script_workspace::tests` and
`script_lsp::tests`. Build debug before running desktop acceptance.

`--pocket` starts in the PocketStation Code tab and records a separate
`pocket-report.json`. Both runs cover adding embedded assembly and using F5 to
save and build the app. With the retained fixture, the headless
`python tests/integration/verify_code_language_services.py` checks all four real
server instances without browser automation or GitHub sign-in.

`--chat` additionally sends a synthetic prompt through the real Copilot ACP
server, verifies the streamed reply and code insertion/Undo, and retains
`copilot-chat.png` (prefixed `pocket-` for the embedded run). It checks that the
reply and compose controls fit within the actual native viewport. It requires
an authenticated Copilot account and does not attach project code to that prompt.

`--inline` verifies a real ghost-text suggestion, Tab acceptance, native source
save and Undo of the accepted suggestion, retaining `copilot-inline.png` with the
same prefix convention. `--pause-chat` holds the completed conversation for 30
seconds so the surrounding native window can also be inspected.

For a project containing the Lua third-person controller, run
`python tests/integration/verify_lua_api_diagnostics.py --project "path/to/project"`
after opening it with the current editor build. This checks every gameplay Lua
script under `assets/scripts` (or repeated `--source` paths), typed reference
completion and hover, class-name casts and the generated definitions with the
installed LuaLS. In-memory negative probes must still report incorrect argument
types, mesh-only calls on camera/actor references and nonexistent methods.
The runner preserves gameplay files and retains a JSON report under
`artifacts/code-editor`.
