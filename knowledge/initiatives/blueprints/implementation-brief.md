# Blueprint implementation kickoff

Expanded implementation update (2026-09-08): the user subsequently requested the
full bounded BP feature, native visual fidelity and publication with an actual
editor screenshot. Data-only/executable assets, templates, spawning, timelines
and node debugging now have working implementations and native acceptance.
Use [validation status](validation-status.md) and [completion gates](completion-work.md)
as the current state. The older kickoff and foundation paragraph below are
historical instructions, not a request to stop at or restart that milestone.

Implementation update (2026-09-08): the foundation described below has working code and MIPS/emulator acceptance. See [foundation status](foundation-status.md) for the delivered contract, test evidence, and limits. The kickoff prompt remains here as historical scope; do not restart or overwrite the implementation. Continue with data-only Blueprint assets and a minimal typed executable graph after addressing any explicitly recorded follow-up. Live Windows creation, inherited Inspector values, attachment Undo/Redo, saving overrides and Reset to Inherited passed after unlocking. Native picker input timed out in desktop automation; registered double-click remains unverified. No Lua implementation or performance claims were added.

Copy the following prompt into a new session opened in this repository:

```text
Begin implementing the Blueprint system in Epok. Deliver the first working foundation milestone, not another research-only response.

Repository: D:/GitProjects/GameEngines/PSX/Epok
Design: D:/GitProjects/GameEngines/PSX/Epok/knowledge/initiatives/blueprints/research.md

Read the applicable AGENTS.md instructions, C:/Users/Adolfo/.codex/RTK.md, CONTRIBUTING.md, the design document, and the architecture, scripting, runtime-services, and testing guides. Revalidate the design against the current code. Preserve all existing uncommitted work, including streaming and rendering changes. Keep documentation, comments, diagnostics, and UI text in English.

Implement Phase 1 (classes and reflection), performing the focused Phase 0 feasibility checks needed to validate it. The long-term goal is class Blueprints with inheritance, defaults, event overrides, Call Parent, and graphs compiled to native C++ for PSX. Do not attempt full feature parity in this milestone.

Required deliverables:
1. A shared, versioned class/type registry with stable class/member IDs, parent relationships, typed properties, function/event signatures, source locations, and eligibility flags. Separate authoring provider from execution backend. Use one registry for class selection and the Inspector, designed to feed the future Blueprint node catalog.
2. Working extraction from annotated C++ headers, initially using the proposed EPOK_CLASS/EPOK_PROPERTY/EPOK_FUNCTION approach and a host Clang-based tool. Validate against real PsyQo headers and the effective MIPS build configuration. Reject unsupported declarations clearly; do not substitute regex parsing or hand-maintained duplicate metadata for reflection. Integrate pinned dependencies and transitive cache invalidation.
3. A transactional C++ script creation dialog with name, folder, searchable parent-class tree, inherited-member information, and Create / Create and Attach actions. Support Behaviour and eligible user C++ bases. Do not generate final by default or suppress inherited update behaviour with an empty override. Validate abstract/final/access rules, identifiers, path collisions, and attachment undo.
4. Typed Inspector values and correct inherited defaults versus explicit overrides, with Reset to Inherited. Preserve existing .epokscript scripts and scene values through a legacy adapter and versioned migration. Preserve legacy serialized values as explicit overrides; do not infer user intent from equality with defaults or silently delete orphaned fields.
5. Build, scene-bank reset, and standalone export integration. Preserve PSX C++20, Q12, no RTTI/exceptions, independent instance state, and existing lifecycle semantics. Generated exports must rebuild without the editor or host reflection extractor.
6. The Lua extension contracts from section 12: provider capabilities, non-C++ artifact preparation, typed invocation boundaries, unavailable-provider diagnostics, and backend-owned instance lifecycle. Do not implement or link Lua now. Validate these boundaries using a small fake provider where appropriate. Games without Lua must acquire no Lua dependency.
7. The project descriptor workflow from section 14, as part of this foundation milestone. Introduce a root-level MyGame.epokproject file containing versioned JSON, with one authoritative owner for the current manifest settings. Share a canonical project resolver across the Hub, CLI, reflection, build, and export; support folder and descriptor opening, --project <file-or-folder>, and positional descriptor launch. Update new-project templates and settings writes, retain legacy ProjectSettings/project.json reading, and implement recoverable versioned migration without silently rewriting on read or maintaining duplicate writable manifests. Preserve relative asset paths, canonical-root locks, deduplicated recents, and project-switch save/cancel behavior. Include Windows file association/icon and double-click support through distribution setup, while portable builds work without registration. Do not change scene or asset extensions or bundle assets inside the descriptor.

Acceptance: create Enemy and Boss : Enemy through the editor; reflect inherited properties and event signatures; verify inherited behaviour, independent overrides, save/load, bank reset, MIPS build, and standalone export. Check reflection failures, inheritance validation, migration, and dependency invalidation with meaningful tests. Run the repository's required checks and relevant emulator validation when available; report unavailable checks honestly. Measure the native prototype where practical without inventing performance results.

Keep the full graph editor, latent actions, dynamic class spawning, construction scripts, hot reload, and actual Lua support for later milestones. Complete the current milestone autonomously, update the design with decisions and implementation status, and finish with changed behaviour, validation results, remaining risks, and a concrete next milestone for data-only Blueprints and executable graphs.

Project-opening acceptance: verify equivalent startup settings and class discovery through folder, descriptor, and CLI paths; verify desktop launch where available. Test legacy migration and recovery, conflicting/multiple descriptors, unsupported versions, malformed data, cancellation, paths with spaces/non-ASCII characters, complete-folder relocation, recents deduplication, and locking across file/folder aliases. Rebuild and export after migration. Update the project guides in English and report any unavailable installer or desktop checks honestly.
```
