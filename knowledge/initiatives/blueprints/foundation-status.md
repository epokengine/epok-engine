# Blueprint foundation status

Validated implementation snapshot: 2026-09-08. This supersedes the earlier scaffold status; it does not claim completion of the visual graph system.

## Delivered foundation

- Semantic libclang 18.1.1 extraction in the isolated `epok-header-tool` process, pinned Cargo/download dependencies, real embedded runtime/PsyQo/MIPS include configuration, actual typed signatures and source locations, UUID/USR identities, duplicate/unsupported declaration diagnostics, concrete construction/reset checks, and transitive content-hash invalidation. Successful caches are retained on errors but cannot authorize stale builds.
- One registry drives the parent hierarchy, inherited-member browser, and typed Inspector. Authoring provider and execution backend have separate versioned identifiers. Native creation publishes only original header/source files; manual JSON remains a legacy adapter, not reflection.
- Name/folder/searchable-parent creation, abstract-base eligibility with concrete-attachment validation, retained inherited behavior, exclusive file creation/rollback, and guarded attachment Undo/Redo. The direct Behaviour template implements its required pure update; derived user classes receive no empty overrides.
- Bool, Int32, UInt32, Q12 Fixed, enums and bounded Fixed vectors; explicit per-instance overrides, Reset to Inherited, retained orphan data, class/member IDs and provider/backend persistence. Version-1 scene reads migrate in memory; only Save writes version 2. Legacy values remain explicit regardless of equality with defaults.
- Recursive native source artifacts, staging/export, backend-owned native declaration/binding/reset generation, and independent instance state. Exported build.ps1 works without editor/reflection tools. A fake non-C++ authoring provider supplies typed declarations, resource-only artifacts and a different reset/release policy; tests cover unavailable-provider persistence, parent filtering and checked invocation boundaries. No Lua VM/runtime dependencies are enabled.
- Authoritative root project descriptors; canonical folder/file/positional CLI resolution; file/folder Hub browsers; canonical locks and recents; explicit locked migration and recovery preserving the losing manifest bytes; version/malformed/ambiguity checks. Optional per-user Windows registration/icon/quoted open command and guarded uninstall are part of setup, not portable startup.

## Validation actually run

- Required Rust formatting, locked unit tests, strict all-target Clippy, and locked host builds. The current default suite contains 114 passing tests and four explicitly ignored environment/desktop/native tests.
- Explicit real-ImGui creation test: Enemy, Boss in Enemies/Bosses, inherited properties/events, concrete attachment, stable identity, Undo/Redo, save/load, duplicate transaction rejection, and abstract-child attachment rollback versus Create success. This exercises production controls with mouse events and real Clang, without relying on Windows input injection.
- `tests/integration/verify_reflection.py --emulator`: typed semantic extraction, transitive shared-include changes, error/cache retention, final/collision rejection, independent overrides, Unicode project paths, nested source directories, MIPS compilation, independent standalone rebuild, relocation, and inherited execution/two bank resets measured from PCSX-Redux RAM. Local evidence: `artifacts/blueprint-foundation/native-validation.json`.
- `tests/integration/verify_projects.py`: folder/descriptor/positional builds, incremental source objects, external relocation, startup/settings changes, old-format reads, migration/recovery, ambiguity/version rejection and export after migration.
- `tests/integration/verify_project_registration.py`: actual registration script in a UUID-isolated HKCU subtree; quoted commands/icon, preservation of existing defaults, repeat installation and uninstall. The test does not change live file associations.
- App-rendered 1280x720 creation-dialog screenshot inspected. Found/fixed its first-frame popup ordering. Local evidence: `artifacts/blueprint-foundation/script-dialog.png`.
- Live Windows editor click-through after unlocking: created Enemy and nested Boss : Enemy, inspected inherited property/event signatures, attached Boss, exercised attachment Undo/Redo, changed an inherited bool, saved its explicit member-ID override, and used Reset to Inherited. The saved scene then contained no instance overrides and the Inspector displayed the parent default again. Local project: `artifacts/blueprint-foundation/desktop-game`. The Hub opened its native descriptor picker with the `.epokproject` filter.
- Dependency setup verified pinned archives/SDK. No fresh-checkout rebuild or performance benchmark is claimed.

## Supported limits and unverified checks

- Native picker selection/cancellation could not be completed through the desktop automation helper: input calls timed out on the owned Windows dialog. Hub/editor controls worked before it opened. Actual registered double-click launch remains unverified; the live file associations were intentionally unchanged. Positional descriptor launching and the registration script were tested separately.
- Unicode native build/export paths currently require an ASCII Windows short-path alias. On a volume with 8.3 names disabled and no suitable alias, an actionable diagnostic requires an ASCII project/export directory. SDK/tool installation directories remain ASCII without spaces.
- Script folder components are portable identifiers, not arbitrary filenames. Reflected constructors must be implicit/defaulted, fields declaratively initialized, and inheritance public/single/nonvirtual. At most 16 exposed fields per class hierarchy. Nested/template classes and arbitrary record/container/pointer fields are unsupported. These are diagnosed rather than guessed.
- Member identities without explicit UUIDs change on rename; mismatched/orphaned stored values require deliberate migration. No general member-rename migration UI is delivered.
- Provider extension contracts are compiled-in host interfaces. An operational third-party provider loader, interpreted execution, Lua creation and cross-language inheritance remain unavailable.
- Physical PSX hardware validation and performance/memory measurements were not performed.

## Next milestone (not part of this foundation)

Implement a versioned data-only Blueprint asset referencing a parent ClassId, typed default overrides, dependency/cycle validation, and generated native subclasses. Then add a minimal executable graph: typed IR, one callable damage function, an overridable event and explicit Call Parent, generated C++ plus node-mapped diagnostics, and the same standalone/MIPS/emulator acceptance. Full graph catalogs, latent actions, construction scripts, dynamic spawning and hot reload remain later work.
