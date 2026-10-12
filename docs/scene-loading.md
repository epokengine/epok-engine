# Scene loading and disposable editor artifacts

Opening a map is not an import/build request. The authoring view continues to
resolve the entire map; PSX residency and baked-visibility simulation remain
separate and opt-in. No authored scene, resource identity, or PSX budget is
rewritten to improve loading time.

## Architecture

`source_cache` stores immutable decoded inputs in a process-wide, estimated
128 MiB LRU (also capped at 2,048 entries). Open documents can retain their own
`Arc`s after eviction. Decoding runs outside the cache lock.

Every lookup reads and SHA-256 hashes the actual source bytes. A timestamp-only
hit is not enough: external edits with preserved file size/mtime, changed asset
packages, and deleted files must remain observable. The existing asset watcher
and build dependency graph still own import discovery and build invalidation.

Artifacts under `.epok/editor-cache/v1/` contain a versioned importer identity,
source checksum and payload checksum. The payload is compact typed JSON, not
YAML and not a memory-mapped or PSX format. Writes use unique temporary files
and atomic rename. An absent, unreadable, truncated or checksum-invalid cache
falls back to source decoding. Unwritable cache storage does not prevent opening
a valid document. This directory is disposable, not source-controlled content.

Cached input projections:

- **Scene:** parsed/migrated, unresolved document. Each open gets an independent
  clone and validates it; unsaved changes never mutate the shared cached source.
  YAML is parsed once on a miss, not once for the version and again for the map.
- **Scene catalog:** validation result/name only. Listing unopened maps no
  longer instantiates every visibility track after their initial import.
- **Scene script index:** embedded script plus actor/component reference IDs.
  Class discovery does not reconstruct geometry or baked tracks on a cache hit.
- **Editable mesh:** validated document, shared across actors and scene switches.
- **Texture:** decoded pixels, palette and PSX words, shared across scene switches.

Cache keys for mesh/texture projections include the expected asset identity;
replacing a package cannot satisfy an old reference even if its index is stale.
Source bytes include package metadata/settings, so reimported resources invalidate
their projection. These caches intentionally do not store dependency-dependent
build products; those require the existing dependency graph, not a single-file
cache key. Increment each projection's version when its derived semantics change.

Blueprint instance refresh is skipped only if the scene has no linked instances.
Linked instances retain the existing refresh/validation path. Resources prepared
by the project loader are not resolved again merely to establish an observation
baseline. Saved-lighting validation is performed on the scene-loading worker,
and the result travels with that exact resolved scene to the UI.

Scene equality compares bulk bake/navigation/visibility data structurally while
retaining the existing serialized Actor/component-projection semantics for the
rest. A background observation must not expand millions of visibility indices
into JSON values on the UI thread just to check that its snapshot is current.

## Verification

Regular tests cover in-memory/disk reuse, same-size/same-mtime source edits,
source deletion, corrupt caches, importer version changes, unwritable caches,
immutable mesh/texture sharing, reimported packages, changed asset identities,
embedded script scopes, and invalid external scene edits. Existing scene
navigation, undo, dependency certification, Blueprint and asset tests remain
applicable.

For real projects, run the ignored `editor::tests::profile_scene_open` test with
`EPOK_IDLE_PROJECT` pointing to an isolated project copy and optionally
`EPOK_PROFILE_SCENE` naming a relative scene path. `EPOK_PROFILE_STARTUP=1` emits
UI handoff stage timings.

`editor::tests::profile_scene_switch_sequence` accepts semicolon-separated
relative paths in `EPOK_PROFILE_SCENES`. It opens each twice, measures total and
maximum UI tick, compares authored data with uncached parsing, checks resolved
meshes/textures and unrestricted visibility, and verifies source files did not
change or trigger an automatic build. Measure both Debug and Release; report
first-import and restart/warm-cache results separately. These tests exclude GPU
rendering/presentation, which must additionally be checked in the real editor.

## Reference architecture

The design follows three established asset-pipeline ideas: keep source assets,
derived import artifacts and their dependency records separate; answer metadata
queries from a registry without loading full objects; and reuse prepared data
from a derived-data cache. These are architectural
principles, not benchmark results or claims that Epok implements any complete
asset system.
