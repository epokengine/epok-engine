# Release installers and local publication plan

Prepare Epok for version 1 with a Windows installer, a macOS disk image and an
initial Debian package. Build and package through repository scripts that can
run locally or through a manually started GitHub Actions workflow. Keep the
existing `develop` to `release` promotion rule.

Planning baseline: 20 October 2026. The engine checkout was on `develop` at
`e6f3018` with ongoing local changes. Published behavior below refers to
`v0.7.0`; dependencies introduced by the local code editor work must be checked
against the commit selected for each future release. This proposal adds no
installer implementation or publication changes.

The current [publication workflow](../../.github/workflows/tag-release.yml)
starts on pushes to `release`, runs setup and tests, compiles the release
binaries, creates an annotated tag and publishes one Windows portable ZIP.
The [v0.7.0 release](https://github.com/epokengine/epok-engine/releases/tag/v0.7.0)
contains a single approximately 69 MiB ZIP. Its
[publication job](https://github.com/epokengine/epok-engine/actions/runs/38734331467)
took about 34 minutes. The required
[`release-policy` check](../../.github/workflows/release-policy.yml) also builds
and tests the release PR; branch protection currently requires that check,
including for administrators.

There is already a [macOS application bundler](../../tools/package-macos-app.sh),
but it is not part of the release workflow. It creates an application with an
ad hoc signature, and its configuration can contain paths from the builder's
machine. The [macOS setup](../../tools/setup-macos.sh) supports Apple Silicon;
[Linux setup](../../tools/setup-linux.sh) supports x86_64. Broader architecture
support needs its own dependency and validation work.

Use this initial delivery matrix:

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

1. **Make installed resources independent of writable state.**

   Audit [`editor_home()` and `user_data()`](../../src/workspace.rs) and
   [dependency installation](../../src/dependencies.rs). Keep application files
   and bundled defaults immutable. Put preferences, dependency downloads, logs,
   tool configuration and WebView profiles in platform user directories. Keep
   project files and generated project data under the user's project directory.
   Use macOS Application Support and the appropriate Windows and XDG locations.
   Migrate existing preferences without deleting the old data.

   Installed builds must resolve resources from their installed layout without
   relying on the builder's `CARGO_MANIFEST_DIR`, shell working directory or
   absolute Homebrew paths. Ship a valid distribution configuration marker.
   Installing or repairing tools must not modify a signed `.app`, a read-only
   AppImage or a system package directory.

   Define an explicit packaging inventory for the editor, mandatory
   `epok-header-tool`, required resources, embedded assets, templates, runtime
   material, setup helpers, SDK content and license notices. Inventory the
   existing Windows ZIP and macOS bundle independently; they currently package
   different sets of files. Exclude builder configuration, Git metadata,
   downloads, caches and local projects. Copy bundled examples into a writable
   location before editing them.

   Separate maintainer build provisioning from end-user tool installation.
   End users must be able to open Epok without installing Rust or checking out
   the repository. Choose and document which PSX tools ship in the package and
   which download on demand, with pinned versions and verified checksums.

2. **Extract shared local build and publication commands.**

   Move packaging and GitHub publication out of YAML into repository tools.
   Proposed interfaces are a platform build/package command, a verification
   command, and a coordinator that creates or resumes a draft release. These
   interfaces do not exist yet. Give the coordinator a dry-run mode that changes
   no Git refs or GitHub resources.

   Resolve the selected merged `release` commit once and record its full SHA.
   Build clean source at that SHA on every host. Check `VERSION`, `Cargo.toml`,
   `Cargo.lock`, pinned SDK revisions and submodules before building. Use the
   pinned Rust toolchain and `--locked`; record package-tool versions too.
   Package only outputs produced for this candidate, so an old binary cannot
   satisfy a missing build result.

   Inspect Cargo configuration and `CARGO_TARGET_DIR`, check disk space and cache
   size, and reuse the canonical `target/` directory with consistent profiles
   and flags. Use release builds for distribution and debug builds for routine
   development. Review caches above 15 GiB before adding build outputs, and apply
   the repository's verified, selective cleanup rules when needed.

   If the selected commit includes the embedded web editor, build its JavaScript
   assets from its lockfile before Cargo embeds them. Account for WebView2 on
   Windows and GTK/WebKitGTK on Linux.
   [Wry platform requirements](https://docs.rs/wry/0.57.0/wry/)
   describe these native engines; Microsoft's
   [WebView2 distribution guidance](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)
   provides online and offline deployment options. Determine runtime package
   dependencies separately from build development packages.

   Emit per-platform manifests containing version, source SHA, architecture,
   dependency versions, filenames and checksums. The coordinator must reject
   mixed commits, missing required packages and conflicting existing tags.
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

   Check native DLL requirements and, if the web editor is included, install or
   locate WebView2. Support upgrading an existing installation while retaining
   preferences, tool downloads and projects. Detect an open editor before
   replacement. Add Authenticode signing for the executables and installer as
   part of the version 1 distribution work.

   Make the installer the primary download. Keep the portable ZIP as a secondary
   option during at least the transition, generated from the same inventory.

4. **Deliver the macOS disk image.**

   Make the current bundler noninteractive and separate compiling from assembling
   the bundle. Remove machine-specific paths and include the complete inventory.
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

Local readiness on the inspected PC: Cargo, Rust, GitHub CLI and Docker CLI are
installed. WSL lists only `docker-desktop`; it does not provide a general Linux
development distribution. Container execution was not tested. Inno Setup and
SignTool were not found on PATH or the checked Inno installation paths. Set up
these packaging tools as part of the Windows increment, and identify the Mac and
signing identities before the macOS increment.

About GitHub usage: `epokengine/epok-engine` is currently public and its workflow
uses `windows-latest`. GitHub says standard runners in public repositories are
free; private repository minutes are charged to the repository owner.
[GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions)
therefore does not establish that this repository exhausted a paid minute quota.
Check the actual account/organization alert and billing period before assigning
a cause to the user's earlier failures.

The repository currently has eight active Actions artifacts totaling about
510 MiB and no Actions caches. The workflow duplicates the portable ZIP in an
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

Accept the migration when a fresh user can install, launch from the desktop,
create and reopen a project, use the advertised scripting workflow, provision
PSX tools and build a sample without a source checkout. Also verify an upgrade
from the prior installer, preservation of projects and preferences during
uninstall, paths containing spaces/non-ASCII characters, and operation without
write permission to application resources. Test an interrupted upload and
resumption, a tag collision, a mismatched platform manifest and the manually
started fallback. macOS acceptance includes Gatekeeper verification of the
downloaded distribution and an intact signature after normal use. Record which
platforms and optional workflows actually passed before advertising them.
