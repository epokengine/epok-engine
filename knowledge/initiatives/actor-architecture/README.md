# Actor architecture initiative

Implementation context for the plan in `knowledge/initiatives/bp-scene-optimization.mdy`
(native classes, Actors, 3D/2D/UI domains and the per-map scene Blueprint).
The plan was written on another machine; its `D:/GitProjects/GameEngines/Epok/EpokEngine`
paths correspond to this repository root.

- [Design contract](design.md) — names, stable IDs, annotation syntax, schema and runtime
  contracts every phase implements against. Read this before touching any phase.
- [Integration map](integration-map.md) — the P4–P7 survey of the call sites each phase
  had to touch. Identifiers, not line numbers.

## Phase reports

| Phase | Report | Scope |
| --- | --- | --- |
| P0 | [p0-baseline.md](p0-baseline.md) | Reference commit, isolation, versions, pre-existing test failures |
| P1 | [p1-registry.md](p1-registry.md) | Reflection schema 8, annotation grammar, host `Model` |
| P2/P3 | [p2-p3-runtime.md](p2-p3-runtime.md) | `runtime/object_model.hpp`, lifecycle, components, legacy bridge |
| P4 | [p4-documents.md](p4-documents.md) | Scene document version 5, `actor_document`, legacy mapping, redirects |
| P5 | [p5-blueprints.md](p5-blueprints.md) | Actor/Component Blueprints, asset version 4, typed refs, `object_classes[]` |
| P6 | [p6-scene-blueprint.md](p6-scene-blueprint.md) | Per-map scene Blueprint: discovery, compilation, editing, map scoping |
| P7 | [p7-editor.md](p7-editor.md) | 3D/2D/UI modes, filtered Hierarchy, Actor creation, Map Settings, scene undo |
| P8 | [p8-world2d-runtime.md](p8-world2d-runtime.md) | `runtime/world2d.hpp` — the 2D world, runtime half |
| P9 | [p9-services.md](p9-services.md) | Audio contract and quarantine, target capabilities, preview protocol v2 |
| P10 | [p10-cook-runtime.md](p10-cook-runtime.md) | Cooked actor tables, runtime wiring, MCP/CLI tools, counters |
| P11 | [p11-delivery.md](p11-delivery.md) | Closing matrix: phases, test coverage, unrun validations, host sizes, deviations |

## Closing context

- [p11-delivery.md](p11-delivery.md) — what was delivered, what is only partially
  covered by tests, the exact commands the first SDK machine must run, and every
  recorded deviation from the design contract.
- [legacy-compat-inventory.md](legacy-compat-inventory.md) — every compatibility
  adapter, derived view and placeholder, with where it lives, which fixtures need it
  and the condition that lets it be deleted.

- [p11-runtime-integration.md](p11-runtime-integration.md) and [p11-editor.md](p11-editor.md) — the
  two P11 code deliveries (runtime/cook gaps; editor actor and component operations).

Nothing in this folder is user documentation. User-facing behavior belongs in `docs/`;
the initiative's user documentation is [`docs/actors.md`](../../../docs/actors.md) and
[`docs/migration-actors.md`](../../../docs/migration-actors.md).
