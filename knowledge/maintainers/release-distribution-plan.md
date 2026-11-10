# Release installers and local publication plan

Prepare Epok for version 1 with a Windows installer, a macOS disk image and an
initial Debian package. Build and package through repository scripts that can
run locally or through a manually started GitHub Actions workflow. Keep the
existing `develop` to `release` promotion rule.

Revised baseline: 6 November 2026, replacing the 20 October assessment. The
engine checkout is on `develop` at
`fb542cd167fdc297425c0b15fedb901b7c4e0c08`; `VERSION` and Cargo report `0.9.0`.
The working tree also contains ongoing gameplay, dialogue, runtime and template
changes, including untracked sources. Those changes inform the candidate's
distribution requirements; they are not all part of the published `v0.9.0`.
Release builds must use a complete, committed candidate. This document updates
the implementation plan without changing packaging, setup or workflows.

The current [publication workflow](../../.github/workflows/tag-release.yml)
starts on pushes to `release`, runs setup and tests, compiles the release
binaries, creates an annotated tag and publishes one Windows portable ZIP.
The latest [v0.9.0 release](https://github.com/epokengine/epok-engine/releases/tag/v0.9.0)
contains a single 85,227,071-byte Windows ZIP, approximately 81 MiB. Its
[publication workflow run](https://github.com/epokengine/epok-engine/actions/runs/40616783446)
completed in approximately 36 minutes on 28 October. The required
[`release-policy` check](../../.github/workflows/release-policy.yml) also builds
and tests the release PR; branch protection currently requires that check,
including for administrators, as confirmed again on 6 November. Both workflows
currently run tests with `--test-threads=1`; the local `make check` uses Cargo's
default test scheduling. Any consolidation must preserve the chosen validation
policy explicitly.

The implementation has advanced since the first assessment:

| Area | Current evidence | Consequence for this plan |
| --- | --- | --- |
| Windows portable | The workflow now requires both binaries, copies tracked content, and includes `Editor.epokconfig`, `VERSION`, setup scripts and manifests | Reuse this relocatable distribution rather than redesigning its inventory from scratch |
| ZIP verification | [verify_portable.py](../../tools/verify_portable.py) checks required files, relative tool paths, example versions, archive safety and exclusion of local build state before tagging | Extend this verifier into package-specific checks and installed-product tests |
| Code editor | Monaco is integrated; generated assets are tracked and embedded. Language servers and WebView profiles already use user data | WebView dependencies are part of the normal product; preserve the existing user storage split |
| Native Play and HUD simulation | [hud_simulation.rs](../../src/hud_simulation.rs) compiles project runners with MSVC on Windows or `CXX`/`c++` elsewhere | A host C++ compiler is an end-user prerequisite for these features, separately from the MIPS compiler |
| Templates and game helpers | The working tree has six templates, dialogue and shared character/world helpers, plus project-owned template installation records | Expand acceptance to the supported language combinations and preserve project identities during upgrades |
| Application data | [workspace.rs](../../src/workspace.rs) already provides a user profile, while PSX dependency setup still writes below the editor installation | Finish this separation instead of migrating all preferences and tools indiscriminately |
| Installer and local release tooling | Packaging/publication still lives in YAML; the macOS script builds an ad hoc signed application | Shared release commands, Windows installer, notarized DMG and Linux package remain pending |

There is already a [macOS application bundler](../../tools/package-macos-app.sh),
but it is not part of the release workflow. It creates an application with an
ad hoc signature, and its configuration can contain paths from the builder's
machine. The [macOS setup](../../tools/setup-macos.sh) supports Apple Silicon;
[Linux setup](../../tools/setup-linux.sh) supports x86_64. Broader architecture
support needs its own dependency and validation work.

Use this delivery matrix, enabling each platform only when its own installation
checks pass. The first local release can require Windows only; add macOS and
Linux to the required asset set as they become ready.

| Platform | Primary download | Installation experience | Build environment |
| --- | --- | --- | --- |
| Windows x64 | `Epok-Engine-vX.Y.Z-Windows-x64-Setup.exe` | Install for the current user, add Start Menu entry, optionally associate `.epokproject`, uninstall through Windows settings | Windows with MSVC and a pinned Inno Setup compiler |
| macOS arm64 | `Epok-Engine-vX.Y.Z-macOS-arm64.dmg` | Open the disk image and drag `Epok Engine.app` to Applications | A Mac with the required Rust, Xcode and signing tools |
| Linux x86_64 | `epok-engine_X.Y.Z-1_amd64.deb` | Install through the package manager with desktop entry, icon and declared runtime dependencies | A pinned Linux build container or VM |

[Inno Setup](https://jrsoftware.org/isinfo.php) provides EXE installers,
shortcuts and uninstall support. Its
[`PrivilegesRequired=lowest`](https://jrsoftware.org/ishelp/topic_setup_privilegesrequired.htm)
supports installation without requesting elevation. Use it for the first Windows
installer; add an MSI only if enterprise deployment becomes a requirement.

Use the existing application bundle as the basis of the macOS disk image. Apple
documents the [disk image layout and signing process](https://developer.apple.com/documentation/xcode/packaging-mac-software-for-distribution),
including an Applications link. A public distribution needs Developer ID
signatures, hardened runtime, notarization and a stapled ticket; the existing ad
hoc signature cannot provide that distribution path.
[Apple notarization requirements](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)
apply to the bundled executables too.

Use [cargo-deb](https://github.com/kornelski/cargo-deb) to package the first Linux
build. Name the supported Ubuntu/Debian versions after validating their actual
library and GPU requirements; a `.deb` does not imply support for every Linux
distribution. Consider AppImage later for a portable download. Its
[compatibility requirements](https://docs.appimage.org/reference/best-practices.html)
still require a suitable build baseline and tests on the supported systems.
Defer RPM and Flatpak until there is demand. In particular, Flatpak would need
additional design for Epok's external compilers, emulators and hardware access
because of its [sandbox permissions](https://docs.flatpak.org/en/latest/sandbox-permissions.html).

Implement the transition through these six increments:

1. **Finish installed paths and define runtime prerequisites.**

   Preserve the existing preferences, `CodeTools` downloads and `CodeWebView`
   profile under [`user_data()`](../../src/workspace.rs). Its production path is
   currently `LOCALAPPDATA`, then `XDG_DATA_HOME`, then `~/.local/share/Epok`;
   tests now get an isolated profile. Add the macOS Application Support location
   with a migration from the old path. Retain project-owned caches and settings
   under the project.

   Separate installation resources from the mutable owner used by
   [`Config`](../../src/project.rs) and
   [dependency installation](../../src/dependencies.rs). Without a project
   override, the owner is still `editor_home()`: setup uses its `.tools/`,
   `Local.epokconfig` and `.epok/dependencies` for downloads, configuration, logs
   and locks. Move these mutable responsibilities to user storage, while
   preserving explicit project overrides and portable behavior. Define the
   installed/portable choice through distribution metadata or a marker, not
   through a failed write followed by an implicit change of mode.

   Installed builds must resolve resources from their installed layout without
   relying on the builder's `CARGO_MANIFEST_DIR`, shell working directory or
   absolute Homebrew paths. Ship a valid distribution configuration marker.
   Installing or repairing tools must not modify a signed `.app`, a read-only
   AppImage or a system package directory.

   Extend the current Windows tracked-file inventory and verifier. Distinguish
   content embedded in the binaries from files required beside them: Monaco and
   Dungeon Complete are embedded, and `project::stage_runtime()` writes an
   embedded runtime snapshot into build staging. The code editor also references
   the installed `runtime/` for C++ tooling, so dropping that directory is not
   justified by embedding alone. Keep both binaries, required raw resources,
   setup helpers, defaults and license notices in a declared inventory. Check
   the macOS bundle against the same functional inventory; it currently omits
   raw runtime/content directories and libclang configuration provided through
   the macOS setup path. Copy examples to user projects before editing them.

   Add a prerequisite table to the product and package verifier:

   | Capability | Current dependency | Distribution requirement |
   | --- | --- | --- |
   | Open editor and integrated code view | Native graphics backend plus WebView2 on Windows, WKWebView on macOS, GTK/WebKitGTK on Linux | Check native runtime availability without requiring Rust or Node |
   | Reflect C++ and resolve project classes | Companion `epok-header-tool` and pinned libclang | Verify the helper belongs to the same candidate and the library resolves after relocation |
   | Native Play and simulated HUD | MSVC Build Tools and Windows SDK on Windows; C++20 compiler on macOS/Linux | Detect and explain the supported host compiler before launching a runner |
   | Build/export PSX games | MIPS compiler, Make, Nugget/psxlua, audio/disc utilities and relevant emulator workflow | Install verified tools to writable user storage or distribute a verified SDK snapshot |
   | Completion and optional Copilot | Existing LuaLS, clangd and Copilot packages in `CodeTools` | Retain verified downloads; keep optional services and account setup independent of basic installation |

   Windows currently discovers `cl.exe` through `cc::windows_registry`, rather
   than using an arbitrary bundled compiler. Prefer detecting an existing
   supported MSVC installation and providing its prerequisite setup path for
   the first installer. Supporting another host compiler is a separate code
   change. macOS must check command line compiler availability when launched
   from Finder; a packaged editor does not provide Xcode automatically.

   Resolve the path constraint before advertising normal desktop installation:
   [getting-started.md](../../docs/getting-started.md) still requires a checkout
   path without spaces because of upstream MIPS Makefiles. Exercise spaces and
   non-ASCII characters in application, user-tool and project paths. Fix or
   explicitly constrain the affected PSX build path; installer quoting alone
   does not resolve Makefile path handling.

   Separate maintainer build provisioning from end-user tool installation.
   End users must be able to open Epok without installing Rust or checking out
   the repository. The current Windows SDK setup still requires Git, clones
   Nugget when absent, and refuses an existing SDK directory without Git
   metadata. An installed SDK snapshot therefore needs a separate verified
   provider; removing `.git` from a copy is not sufficient. Keep maintainer
   submodule provisioning separate from this end-user route. The current
   Nugget gitlink and dependency manifest agree on
   `c702235f366fb1cd8c18e0a5a5392e0acae8a079`; record it and the pinned nested
   psxlua revision in the candidate manifest. Never initialize all nested
   submodules recursively.

2. **Extract shared local build and publication commands.**

   Move packaging and GitHub publication out of YAML into repository tools.
   Proposed interfaces are a platform build/package command, a verification
   command, and a coordinator that creates or resumes a draft release. These
   interfaces do not exist yet. Give the coordinator a dry-run mode that changes
   no Git refs or GitHub resources.

   Extract the existing Windows build/stage/ZIP/verify path first and exercise it
   locally with publication disabled. Reuse `tools/verify_portable.py` and the
   current workflow's tracked-file enumeration. Add a separate installed-package
   policy: its defaults need user storage rather than the portable `.tools/`
   layout. Retain the portable verifier's existing guarantees and add coverage
   for its rejection cases. Replace its Python `assert` checks with explicit
   validation failures so interpreter optimization cannot disable verification.
   Give each candidate a fresh staging subdirectory
   below `dist/`, independently of the reusable Cargo cache, so packaging cannot
   pick up files left by an older candidate.

   Resolve the selected merged `release` commit once and record its full SHA.
   Build clean source at that SHA on every host. Check `VERSION`, `Cargo.toml`,
   `Cargo.lock`, pinned SDK revisions and submodules before building. Use the
   pinned Rust toolchain and `--locked`; record package-tool versions too.
   Package only outputs produced for this candidate, so an old binary cannot
   satisfy a missing build result.

   `VERSION`, root Cargo metadata and the published release currently report
   `0.9.0`. A new production release must increment the authoritative version
   and synchronize shipped example descriptors through the existing version
   policy. Require all new runtime/helper/template sources to be tracked in
   the selected commit; the current dirty working tree cannot be the production
   build input. Ensure the psxlua sources are present before Cargo builds the
   Lua bytecode cooker: `build.rs` otherwise permits a build without `epok_luac`,
   leaving bytecode cooking unavailable despite successful compilation.

   Inspect Cargo configuration and `CARGO_TARGET_DIR`, check disk space and cache
   size, and reuse the canonical `target/` directory with consistent profiles
   and flags. Use release builds for distribution and debug builds for routine
   development. Review caches above 15 GiB before adding build outputs, and apply
   the repository's verified, selective cleanup rules when needed.

   The embedded web editor is now part of the committed product, not an optional
   future addition. Use the tracked generated assets for ordinary release
   builds, which do not require Node or frontend network access. Run the pinned
   `npm ci`, frontend checks and build in a separate asset-consistency check,
   requiring its output to match the candidate before distribution. Compare
   reproducible bytes or hashes against the candidate rather than creating a
   silently modified source tree on each host. Preserve worker files, fonts and
   generated license notices, beyond the three web files named in the current
   portable verifier. Account for WebView2 on Windows and GTK/WebKitGTK on Linux.
   [Wry platform requirements](https://docs.rs/wry/0.57.0/wry/)
   describe these native engines; Microsoft's
   [WebView2 distribution guidance](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)
   provides online and offline deployment options. Determine runtime package
   dependencies separately from build development packages.

   Emit per-platform manifests containing version, source SHA, architecture,
   dependency versions, enabled capabilities, filenames and checksums. Record
   the candidate's required platform set and validation results. The coordinator
   must reject mixed commits, missing required packages and conflicting tags.
   Produce `SHA256SUMS` after all signing and notarization steps have finished.

   After candidate verification, create and push the annotated version tag at
   the exact built SHA. Use `gh release create --draft --verify-tag`, upload the
   artifacts, verify their names and checksums, and publish the completed draft.
   [GitHub CLI supports local release creation and asset upload](https://cli.github.com/manual/gh_release_create).
   Resume partial uploads only in a matching draft. Avoid overwriting published
   artifacts with `--clobber`; corrections should receive a new version.

3. **Deliver the Windows installer first.**

   Add a pinned Inno Setup recipe with stable application identity, version
   metadata, icon, user installation directory, Start Menu entry and uninstall
   support. Reuse the existing
   [project association helper](../../tools/register-project.ps1) where suitable,
   with uninstall ownership checks. Make desktop shortcuts and project
   associations optional.

   Check native DLL requirements and install or locate WebView2. Validate the
   host compiler prerequisite separately from the MIPS toolchain and libclang.
   Support upgrading an existing installation while retaining
   preferences, tool downloads and projects. Detect an open editor before
   replacement. Add Authenticode signing for the executables and installer as
   part of the version 1 distribution work.

   Make the installer the primary download. Keep the portable ZIP as a secondary
   option during at least the transition, generated from the same inventory.

4. **Deliver the macOS disk image.**

   Make the current bundler noninteractive and separate compiling from assembling
   the bundle. Remove machine-specific paths and include the complete inventory.
   Replace opportunistic copying from the builder's `.tools/macos` with a
   declared package policy. Include or provision pinned libclang and the
   required compiler/tools without asking end users to run the developer setup
   inside the signed bundle. Preserve any optional serial support requirements
   as an explicitly tested capability.
   Derive bundle version metadata and the minimum macOS version from the chosen
   support policy and verified dependencies.

   Sign nested code and helpers with Developer ID, sign the completed application
   with hardened runtime, and build the disk image with an Applications link.
   Sign, notarize and staple the final distribution, and verify it after download
   on another Mac. Keep signing credentials in the operating system's credential
   storage, outside the repository and manifests.

   Start with arm64 because that matches the existing setup. Add Intel or a
   Universal build only after its complete SDK/tool inventory and installation
   path pass the same checks.

5. **Deliver the Linux package.**

   Pin the build distribution/container and test against explicitly supported
   Ubuntu/Debian versions. Install the binary and helpers with resources under a
   defined package layout, and provide a launcher if resource discovery requires
   it. Add a desktop entry, application icon, MIME association and package
   metadata. Declare the runtime libraries identified from the final binaries.

   Keep Linux experimental until the installed package passes desktop and
   workflow checks. Ubuntu 24.04 is available locally as a development/test
   environment; choose and pin the oldest supported build baseline after
   checking Rust, native libraries and the advertised target distributions.
   A package built under WSL is not evidence of graphics, audio or serial support
   on a user's desktop. Provide the host C++ compiler and PSX tool setup paths
   without writing into `/usr` application resources.

   Test installation with dependency resolution, upgrade, removal, desktop
   launching and project association. Exercise X11 and a Wayland session through
   the XWayland path currently selected by the local editor implementation.
   Validate graphics, audio and any advertised serial/device workflow on a real
   desktop as well as package structure in a container.

6. **Switch release publication after a full rehearsal.**

   Run the new commands in dry-run mode first. Then replace the packaging
   workflow's automatic `push` trigger with `workflow_dispatch`, available on
   the default `develop` branch, accepting an explicit release commit or tag.
   The fallback must verify that its selected commit belongs to `release` and
   invoke the same repository scripts as the local path. GitHub documents the
   [manual workflow requirements](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow).

   Keep `release-policy` and its branch protection working during this change.
   A normal merge into `release` should continue validating promotion but should
   no longer start the binary publication job. Transfer annotated-tag creation
   to the shared coordinator; it currently lives inside the publication job.
   Avoid using `[skip ci]` as the normal control because it can leave required
   checks pending, as described by
   [GitHub's skip behavior](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/skip-workflow-runs).

   Include the trigger change in the release promotion that activates local
   publication, and verify the resulting `release` workflow before running the
   coordinator. An earlier release revision still has automatic publication
   enabled. Rehearse the transition so the next promotion uses the intended
   manual workflow and cannot race a local run. Prevent the local
   coordinator and the manual fallback from publishing the same candidate
   concurrently. A single coordinator must own tag creation and draft completion.

   The first production run should assemble a draft from the selected commit,
   complete installation checks for the advertised platforms, then publish once
   all required assets are present. Update
   [release instructions](../../docs/release-process.md), download links and
   website documentation to match the delivered formats.

The normal release sequence will be: merge `develop` into `release`, freeze its
SHA, build and test the candidate on the relevant hosts, verify manifests, create
the annotated tag, upload the installers to a draft, and publish the complete
release. Each platform can upload from its own host; the coordinator enforces a
single source revision. A Windows PC can cover Windows and a Linux container or
VM. Use a Mac for the macOS build, signing and installation validation. A hybrid
mode can retain GitHub-hosted macOS builds when no Mac is available.

Local readiness rechecked on 6 November: Cargo, Rust, GitHub CLI and Docker CLI
are installed. WSL now lists `Ubuntu-24.04` as well as `docker-desktop`, and an
Ubuntu shell reports x86_64. Linux packaging dependencies and desktop acceptance
still need validation; container execution was not tested. Inno Setup and
SignTool are not on PATH, and no Inno Setup directory was found under the checked
Program Files locations. `CARGO_TARGET_DIR` is unset in this session; inspect
Cargo configuration again before implementation builds. Identify the Mac and
signing identities before the macOS increment.

About GitHub usage: `epokengine/epok-engine` is currently public and its workflow
uses `windows-latest`. GitHub says standard runners in public repositories are
free; private repository minutes are charged to the repository owner.
[GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions)
therefore does not establish that this repository exhausted a paid minute quota.
Check the actual account/organization alert and billing period before assigning
a cause to the user's earlier failures.

On 6 November the repository has ten active Actions artifacts totaling about
663 MiB and no Actions caches. The workflow duplicates the portable ZIP in an
Actions artifact and a GitHub Release asset. The new local path should upload
directly to Releases, while any fallback workflow should retain intermediate
artifacts only briefly. This storage observation does not identify the earlier
failure cause. Release assets are a separate distribution mechanism with their
own [limits](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases).

Local scripts require no Actions runner minutes for compiling, packaging or
uploading. The retained PR validation still runs in Actions. If the goal later
expands to eliminating all Actions execution, redesign the required check and
branch protection explicitly. Self-hosted runners are an optional later route
for automatic orchestration; direct local scripts satisfy the current request
with less runner administration.

The release acceptance matrix must exercise the final installed artifact with
the source checkout unavailable and an isolated user profile. A structural ZIP
check, a developer-machine launch or the default Cargo test suite alone cannot
establish these outcomes:

| Gate | Required installed-product evidence |
| --- | --- |
| Relocation and startup | Start through the OS shortcut from another working directory, without write access to application resources; check spaces/non-ASCII paths and missing WebView behavior |
| Authoring templates | Create/reopen Basic, Sample, Third Person, Platformer 2D and Dungeon in C++, Blueprint and Lua; create Dungeon Complete in C++, its currently supported flavor. This is 16 creation cases across six templates |
| Native execution | Build fresh Native Play/HUD runners with the supported host compiler; verify actionable behavior without that compiler and reuse after restarting |
| Gameplay content | Exercise character/camera helpers, interaction, dialogue/widgets and project template role records where included by the selected candidate; use the relevant [gameplay helper acceptance matrix](../initiatives/gameplay-helpers/plan.md) |
| Lua and export | Exercise the advertised Lua Native C++, VM bytecode and VM source modes, PSX build/export, and external scene files where used; ensure bytecode cooking is available |
| Integrated editor | Open the embedded code editor, load workers and fonts, edit/save, resolve runtime headers and install/use LuaLS and clangd through the existing [acceptance path](../../docs/code-editor.md); keep Copilot optional |
| Dependencies | Install/repair the pinned SDK and tools without changing application files; verify interrupted downloads, checksums and operation after moving the installation |
| Upgrade and removal | Migrate preferences, retain project class UUIDs and template role bindings, preserve user modifications, handle an open editor, and uninstall without deleting projects or shared tools |
| Platform trust | Verify signed Windows products and macOS Gatekeeper behavior after download; macOS signatures remain valid after tool installation and normal use |
| Publication recovery | Reject a conflicting tag or mixed source manifests, resume interrupted uploads into a matching draft, verify the required asset set, and exercise the manual fallback |

The 16 creation cases describe the current `Template::supports()` policy. The
[gameplay helpers initiative](../initiatives/gameplay-helpers/plan.md) targets
18 creation cases and 30 execution combinations once Dungeon Complete also
supports Blueprint and Lua. At this assessment that initiative records P8 in
progress and P9/P10 pending, including a pre-existing complete-game frame-time
failure and missing runtime heap/stack peaks. Preserve those completion and
budget gates when selecting the version 1 candidate. Packaging readiness does
not establish gameplay parity or clear its outstanding acceptance results.

Use existing tests as inputs, including
[portable verification](../../tools/verify_portable.py),
[release version/build-state tests](../../tools/test_release_workflow.py),
[small starter checks](../../tests/integration/verify_small_starters.py),
[Lua mode checks](../../tests/integration/verify_lua_modes.py) and
[Dungeon Complete checks](../../tests/integration/verify_dungeon_complete.py).
Some of the new integration sources are untracked in the assessed working tree;
include them in the committed candidate before relying on them. Adapt harnesses
that currently launch `target/debug/epok-editor.exe` so they can exercise the
installed release executable. Run the relevant ignored Rust tests and Python
integration checks explicitly; `cargo test --workspace` does not cover all of
them. Record which
platforms and optional capabilities actually passed before advertising them.
