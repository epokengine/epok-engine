# Contributing

Epok is an early PlayStation editor and runtime. Keep changes focused and explain the observable behavior, the reason for the change and the local checks performed.

## Development

Follow [Getting started](docs/getting-started.md). Supported environments are Windows x64 and macOS 11+ on Apple Silicon. Run the default checks from [Testing](knowledge/maintainers/testing.md) before submitting code; run the relevant emulator checks for runtime, exporter or integration changes. GitHub Actions enforces the release version policy; full validation is still performed locally.

Use `develop` for ongoing integration. Published versions reach `release` only
through a pull request from `develop`; see the [release process](docs/release-process.md)
for version validation, tags, and the local build counter.

Keep original sources separate from generated output. Game scripts and scenes live in each project's `assets/`; Lua classes (`.lua`) live in `assets/scripts/` beside C++ sources; the bundled sample is `examples/sample-game/`. Editor resources belong in `resources/editor/`. Never edit staged copies under a game's `.epok/build/` as the source of a change.

Preserve scene compatibility when changing serialization. Update the editor, generator and C++ runtime together when their shared representation changes.

## Dependencies and documentation

Commit `Cargo.lock` for dependency changes. Update the Nugget gitlink and dependency manifest together, and test the pinned revision. Do not submit compiled SDK libraries, downloaded executables or unrelated nested repositories.

Write documentation, code comments and commit descriptions in English. Document current behavior and limits. Keep user guides separate from local verification logs. Preserve third-party attribution when adding or updating resources.

Organize `docs/` around reusable engine features, APIs, workflows and limits. Game-specific designs, implementation reports and genre roadmaps belong to their game projects. Included examples document their own behavior in their example directories. README showcase images can illustrate games built with Epok without including or documenting their gameplay code in the engine repository.

Use the repository's formatting and line-ending settings. Avoid broad refactors mixed with behavior changes. Unit tests can remain beside Rust modules; compiler/emulator verification belongs in `tests/integration/`.

## Change descriptions

Describe the problem and resulting behavior, give a short reproduction when useful, and list checks actually run. Mention checks skipped because required hardware or tools were unavailable. Include product screenshots for visible UI changes after reviewing them for local information.

Original contributions are provided under the project's [MIT license](LICENSE). Third-party resources retain their own license terms.
