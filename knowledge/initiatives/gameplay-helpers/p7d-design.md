# Dungeon migration implementation notes

Status: P7C is validated. Shared pickup/flag/door rules and semantic roles are
implemented in all three providers. Ten console policies, five foundation routes,
three Native Play providers, three standalone exports and exact 180-frame motion
comparisons pass. Fresh generation preserves the original world/resources in
all three providers. Installation rollback and world-domain validation pass;
the final debug build, formatting and Clippy pass. P7D is validated with
1,310 workspace tests / 88 ignored; see the delivery notes and receipt for
detailed provenance.

Fresh Dungeon classes now use Pickup, GameplayFlag and Door contracts and consume
the template's semantic class references. Existing project sources retain their
old contract. New ownership state is independent of actor visibility.

## Project rules and generated composition

- `DungeonKey` derives from `PickupComponent`. The generated instance binds
  `accepted_initiator` to the player, enables `auto_collect`, and points
  `collected_flag` at a `GameplayFlagComponent` on that same key actor. Its
  selected-language `collected` callback hides the key. The `lock_id` value remains
  project data. The flag remains readable while its actor is inactive.
- `DungeonDoor` derives from `DoorComponent`. Its `requirement_flag` names that
  key's flag component; its collider reference names the existing door collider.
  Project code requests opening when ownership becomes available and implements
  the visual result through the normal door events. Parent lifecycle and queued
  collision transitions remain active. Automatic opening remains the default.
- Extra state uses components on existing actors. No extra organizational actors
  are required. Preserve the recipe, style, generated actor transforms, resource
  payloads, handmade-room contents, lighting and navigation settings.
- C++, Blueprint and Lua own the same rule hooks. Generated C++ from Blueprint
  or Lua remains compiler output. Each fresh project includes only its selected
  authored implementation, plus native SDK services and shared resources.

## Identity and compatibility

Add typed `key`, `door` and `key_state` roles to the existing
`ProjectSettings/Template.epoksettings` beside the player/camera roles. Resolve
these through the ordinary reflected class registry. Regeneration consumes the
UUIDs, including project replacements and renamed classes, rather than guessing
the provider from source paths.

Keep component and flag identities deterministic from the generator's existing
derived identities so doors bind to the same regenerated key state. Validate
owner domains, required inherited properties, collider references and component
budgets before committing scene changes.

Existing authored key/door implementations need an explicit compatibility path.
Preserve edited files and their legacy actor-reference properties. Do not turn
their `player`/`key` fields into new component references without also migrating
the implementation and saved placements. Keep any old identity/file detection
inside that compatibility path; new role metadata is authoritative, and malformed
or unresolved role metadata must fail without silently installing another flavor.
Even untouched legacy sources remain preserved; upgrading a source/property
contract requires an explicit migration. Installation journals newly created
sources and changed identity settings so a rejected role resolution can restore
the previous installation without discarding concurrent user edits.

## Validation gates

1. Compile all three fresh providers, verify common native parents and inspect
   editable rule graphs/source. Check inherited owner contracts and property
   budgets with real reflection and staging.
2. Generate both 2D and 3D recipes, including multiple locks, unbound players,
   handmade rooms and regeneration. Verify deterministic component references,
   shared flags, existing visuals/resources and player-only automatic pickup.
3. Rename project classes and actors, replace semantic roles with compatible
   derived classes, save/reopen and regenerate. Add alternate-flavor files to
   prove that recorded roles continue selecting the intended implementation.
4. Cover invalid/missing roles, edited legacy scripts, obsolete unedited scripts
   and generation failure. Preserve files and saved scene data on rejection.
5. Exercise ownership independently of key visibility, repeat after an explicit
   pickup/flag/door reset, and verify locked/unlocked collider behavior. Run these
   policies in all five execution combinations and compare the established
   character/key/door route with P5, explaining any intended timing difference.
6. Rebuild standalone exports, measure linked RAM/reserve, run the workspace and
   runtime lifecycle suites, then record a separate P7D receipt. Complete-game
   refactoring and the full final matrix remain P8–P10.
