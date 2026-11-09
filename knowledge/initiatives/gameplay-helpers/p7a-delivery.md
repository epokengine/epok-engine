# P7A Basic and Sample delivery

Status: validated. P7B-P10 remain open.
Evidence: `artifacts/gameplay-helpers/p7/small-starters-validation.json` and
`p7/record-small-starters.py`.

## Starter classes

Basic generates an ordinary, editable `GameRoot : Actor3D` class in C++, Blueprint
or Lua. The scene contains its camera and the inert actor with one inherited
3D root component. It adds no world mesh or active gameplay helper.

Sample retains its selected-language Spinner rule and common world. Its speed
property is editable per component instance. The C++ class now respects runtime
class identity when inherited. The same derivation predicate serves the compiler
and class picker, including supported Lua parents.

`ProjectSettings/Template.epoksettings` records the installation choice and the
`game_root` or `spinner` role using existing ClassReference UUIDs. It defines no
classes or alternate class catalog. Opening validates and preserves the file;
projects can mix authoring systems afterwards. Missing installation metadata
continues to work for older projects. Invalid edits remain on disk for repair.

The Hub and CLI describe the emitted class and source location. Public workflows
are documented in [starter classes](../../../docs/starter-templates.md) and
[projects](../../../docs/projects.md).

## Acceptance evidence

| Check | Result |
| --- | --- |
| Selected source ownership and basic composition | Passed in all three providers |
| Role identity, native ancestry, optional-helper exclusion and reopen | Passed |
| Malformed installation/version handling | Rejected without rewriting the authored file |
| Native Play | Six template/flavor cases passed; live instance speed changes and independent owners agree |
| PSX execution | Ten cases passed: two templates x C++, Blueprint, Lua Native C++, Lua bytecode and Lua source |
| Semantic traces | All five modes match exactly within each template |
| Authored world comparison | Matches across flavors after normalizing local identities and the selected starter class UUID; geometry/properties/bindings remain compared |
| Derived Spinner | Authored in the selected language, inherits the speed property and behavior, accepts an independent instance override and retains runtime class identity |
| Actor renaming | All fixture actors renamed before playback; UUID bindings remain valid |
| Standalone export | Six exports rebuild byte-identically to the played executable |
| Full workspace | 1,296 passed; 88 ignored for their declared environment requirements |
| Clippy / format / whitespace | Passed, with Clippy warnings denied |

The fixture observes actual starter rules. It adds a native observer to the
camera and, for Sample, a project-derived Spinner on the second cube with speed
90 while the original stays at 45. It supplies no rotation implementation.
After 120 ticks the original/derived angles are 90/180 degrees in every mode,
with matching state hashes and valid distinct runtime class identities.

The first full regression found two stale test fixtures: MCP replaced the startup
class catalog with synthetic definitions, and the Hub expected Basic to contain
only one actor. The MCP operation fixture now starts with its own empty scene;
the real Hub test requires camera and GameRoot. The original failed log is retained
as `p7-starter-workspace.log`. The final full suite, including both tests and
Native Play, passes in `p7-starter-workspace-final.log`.

Execution and export use frozen editor SHA-256
`ee1a39b58a27fc0af4232973223bcab84ffff3a8338c85faea75ee2be62d0645`.
Per-case executable hashes, static allocations, authored-source hashes and export
paths are stored in the evidence. Allocation rows describe the acceptance
fixtures, including observation/derived components. They are not runtime
heap/stack peaks or final hardware/performance acceptance.

P7B next separates reusable player/camera construction from arena construction
and records Third Person roles. Platformer pickup policy and Dungeon helper
regeneration remain P7C/P7D. Complete-game decomposition, ports and the final
30-case template acceptance remain P8-P10.
