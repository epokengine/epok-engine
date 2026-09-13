# Epok document formats

Epok uses type-specific extensions and YAML for editor-owned text documents.
The PSX runtime consumes generated C++ and binary data; it does not parse YAML.

| Document | Extension / location |
| --- | --- |
| Project identity and settings | `Game.epokproject` at the project root |
| Scene / map | `assets/scenes/Main.epokmap` |
| Visual Blueprint | `assets/scripts/Enemy.epokbp` |
| Legacy C++ property metadata | `assets/scripts/Spinner.epokscript` |
| Resource package | `Stone.epokasset` (binary) |
| Additional map registry | `ProjectSettings/Maps.epoksettings` |
| Editor tool configuration | `Editor.epokconfig` |
| Machine/project tool override | `Local.epokconfig` |
| User preferences | `Editor.epokprefs`, `RecentProjects.epokprefs` |
| Local project state | `UserSettings/SceneView.epokprefs`, `ImportState.epokprefs`, `Breakpoints.epokprefs` |
| Reflection/import caches | `.epok/reflection/Reflection.epokcache`, `.epok/imported/<hash>/Import.epokcache` |
| Extractor request | `.epok/reflection/Reflection.epokrequest` |
| Exported script manifest | `Scripts.epokmanifest` |
| Generated Blueprint debug map | `blueprints/<id>.epokdebug` |

For example:

```yaml
format_version: 1
editor_version: 0.1.0
name: My Game
startup_scene: assets/scenes/Main.epokmap
auto_build: true
```

The document layer retains the existing versioned data schemas, UUIDs, numeric
types, validation, and plain mappings/sequences. Writers produce deterministic
YAML with stable field ordering and no language-specific enum tags. Unknown
Blueprint fields are retained. Comments and hand-chosen formatting are not
preserved when the editor saves a document. Duplicate keys and multiple YAML
documents in one file are rejected. JSON syntax remains accepted as a subset for
legacy fixtures, but ordinary Epok saves write YAML.

`.epokasset` packages use the eight-byte `EPOKAS01` signature, followed by the
existing metadata/source lengths and binary payload. Their embedded metadata
remains JSON; the package is not a text document.

JSON remains intentional for MCP and extractor stdout, VS Code configuration,
Clang's `compile_commands.json`, PCSX-Redux configuration, the dependency bootstrap
manifest `tools/dependencies.json`, and explicitly JSON diagnostic reports.
These do not acquire Epok extensions merely because they are in this checkout.

`Scripts.epokmanifest` v2 is generated provenance. `files` and `native_sources`
are relative to the manifest directory (`file_base: manifest`); `dependencies`
are relative to the authoring project (`dependency_base: project`). All use
forward slashes. Dependencies describe the original inputs; standalone builds
use the staged files and do not require those authoring paths. Regenerating a
build/export replaces generated v1 host-path manifests with v2 without changing
source files. Provider dependencies outside the project are rejected before
writing artifacts.

## Document versions

| Contract | Current version | Accepted on load |
| --- | --- | --- |
| Scene / map (`.epokmap`) | 5 | 1–5 |
| Blueprint asset (`.epokbp`) | 4 | 1–4 |
| Reflection manifest and cache | 8 | 8, plus 7 read with default actor metadata |

A document whose version is **above** the current one is refused with a message
naming both versions, and is never read as an empty document or re-saved over. A
scene with version 0 keeps the older "unsupported scene version" refusal. Older
versions are migrated in memory and the original bytes are preserved until you save.

### Scene document version 5

Version 5 adds two optional members to the scene document: `actors` and
`scene_script`. Both are omitted when empty, so a map with neither is written
exactly as version 3 or 4 was. The version rises to 5 only when actor content
actually exists in the document being saved. See
[Actors and components](actors.md) and
[Migrating an existing project](migration-actors.md).

The shape, shown as JSON (the file on disk is the same structure encoded as YAML):

```json
{
  "version": 5,
  "name": "SampleScene",
  "entities": [ /* unchanged legacy entities */ ],
  "actors": [
    {
      "id": "7d0f1c02-9f3a-4c0e-9d3a-6b1c2f7d4a11",
      "class": { "name": "epok::Actor3D", "class_id": "fc24ce9b-558c-49de-bc35-e040f350e486" },
      "name": "Hero",
      "active": true,
      "components": [
        {
          "id": "1a2b3c4d-0000-4000-8000-000000000001",
          "class": { "name": "epok::SceneComponent3D", "class_id": "ed73d249-b6cb-4a3c-a0e8-696de55e286f" },
          "name": "Transform",
          "root": true,
          "properties": { "position": [0.0, 0.5, 0.0], "rotation": [0.0, 0.0, 0.0], "scale": [1.0, 1.0, 1.0] },
          "overrides": ["position", "rotation", "scale"]
        },
        {
          "id": "1a2b3c4d-0000-4000-8000-000000000002",
          "class": { "name": "epok::AudioComponent", "class_id": "7f0eb028-5301-4ac7-b93b-5665fab12b20" },
          "name": "Footsteps"
        }
      ],
      "properties": { "speed": 2.5 },
      "overrides": ["speed"],
      "legacy_entity": "2f1d7b60-4c8a-4d2e-8f10-9a4b6c1d2e30"
    },
    {
      "id": "7d0f1c02-9f3a-4c0e-9d3a-6b1c2f7d4a12",
      "class": { "name": "epok::UIActor", "class_id": "b09bd2fa-8b09-4c0f-a33a-c3ca08b21d8f" },
      "name": "Health",
      "active": true,
      "logical_parent": "7d0f1c02-9f3a-4c0e-9d3a-6b1c2f7d4a11",
      "components": [
        {
          "id": "1a2b3c4d-0000-4000-8000-000000000003",
          "class": { "name": "epok::RectTransformComponent", "class_id": "dc805165-6c65-48dc-8ff8-4a638a5d21df" },
          "name": "RectTransform",
          "root": true
        }
      ]
    }
  ],
  "scene_script": {
    "parent": { "name": "epok::SceneScriptActor", "class_id": "b4c08aa0-fa85-4abf-8f45-7501e1c8a040" },
    "blueprint": { "version": 4, "name": "SampleScene_SceneScript", "…": "a whole Blueprint asset" }
  }
}
```

- `properties` holds authored values and `overrides` records which of them are
  explicit. A document with no `overrides` entry treats every persisted property as
  an override, so values from a class that changed shape survive a load/save cycle
  instead of being dropped.
- `logical_parent` is the hierarchy you see; the separate `attach` member is spatial
  attachment, and an absent `attach.component` means the target actor's root.
- `legacy_entity` is provenance only: it names an entity of the same document that
  this actor came from.
- `scene_script.blueprint` is a complete Blueprint asset embedded in the map. The map
  file is therefore the Blueprint's own source: diagnostics point at the map, and
  saving the Blueprint saves the map.

### Blueprint asset version 4

Version 4 adds one optional `family` hint to a `.epokbp` document. The hint is
advisory — the compiled class tree always wins, and a hint that disagrees with it is
a compile diagnostic naming both families. Because the field is omitted when unset, a
version 3 asset is written as exactly the bytes it had, so upgrading does not make
any existing Blueprint look stale.

## Migrating a UniQo game

The original engine checkout and existing games are not automatically modified.
Copy a game to a new destination using the migration tool:

```sh
python -m pip install -r tools/requirements.txt
python tools/migrate_project.py path/to/old-game path/to/new-epok-game
```

The tool validates the source, copies authoring files, converts JSON documents to
YAML, moves the project descriptor to the root, updates references and C++ API
names (`epok::`, `epok.hpp`, `EPOK_CLASS`, `EPOK_PROPERTY`, `EPOK_FUNCTION`), and
changes resource package signatures without changing asset UUIDs or payloads.
It refuses occupied destinations and never writes into the original game.
It omits build caches, exports, Git metadata, and machine-specific tool overrides.
Macro calls produced by custom code generators must also use the new API names.

For native reflected C++ members, Clang identities include namespaces. The tool
updates serialized qualified names; compile the copied project to diagnose any
custom identities or external provider references needing manual adjustment.

The editor's `--migrate-project` command only relocates an already compatible
legacy `ProjectSettings/project.json` manifest; it is not a substitute for the
full cross-engine migration tool above.

## Working copy

This refactor started from upstream `main` commit
`811fcb4` (Replace Blueprint promo screenshot with cursor-free native capture).
Epok is presented as an engine for the original PlayStation. The source repository
is https://github.com/franadoriv/epok-engine. The website will be linked when published.
