# Blueprint and VFX troubleshooting

Start with the first compile or validation error. Later messages may be consequences
of the same missing class, binding or asset. Preserve the source while repairing
references; deleting caches does not repair an invalid graph.

## Classes, graphs and connections

| Symptom | Check | Recovery |
| --- | --- | --- |
| No usable Blueprint parent | Windows reflection dependencies, extractor and native annotations | Repair setup, resolve native reflection errors and choose a Blueprintable, non-final class |
| A method does not appear in a graph | Its reflected visibility and Blueprint callability | Expose the intended method in its native declaration, then rebuild reflection |
| A property or function does not appear in a timeline | Explicit timeline permissions and supported types | Use TimelineAnimatable, TimelineCallable or TimelineAction as appropriate; editability alone is insufficient |
| A pin will not connect | Exact type, vector length and entity/class ancestry | Use compatible data or a checked cast; do not replace typed references with names |
| The class compiles but nothing runs | Scene attachment, entity activity and an event caller | Attach or spawn an instance and ensure an entry event actually invokes the graph |
| A new class default does not affect one entity | An explicit instance override | Reset that member's override to inherit again |
| A child event lost parent behaviour | Event override without Call Parent | Add Call Parent where parent execution is intended |
| A renamed or removed member leaves an error | Persistent member ID, parent and orphaned connections | Use the remap/reset controls; a same-named replacement is not the original member |

See [Your first Blueprint](blueprints-tutorial.md) for the workflow and
[Blueprint reference](blueprints.md) for the supported execution profile.

## Effects and timelines

| Symptom | Check | Recovery |
| --- | --- | --- |
| The preview is empty | Texture, pixel rectangle, size, opacity, enabled layer and emission | Isolate one layer, select a valid imported atlas and restart from the seed |
| The flash is hidden in Game view | Billboard depth and geometry sorting | Inspect from the actual camera; apply a modest depth bias or adjust placement |
| An icy effect looks yellow | The source atlas color multiplies the tint | Choose neutral artwork for freely tintable effects |
| Validate reports an empty required test binding | The editor's temporary Test scene binding | Select a compatible scene entity for validation; configure the persistent component binding separately |
| An effect works in the asset preview but fails in the scene | Required external bindings, component requirements or scene overrides | Repair the component or direct-node pins and test in PSX Game view |
| A curve no longer validates after retiming | Merged key times or keys outside duration | Separate key ticks or choose a duration that preserves distinct keys |
| Two curves conflict despite separated keys | Curves hold endpoints; equal-priority Absolute writers overlap | Remove the duplicate writer or deliberately choose compatible priority/blend/restoration policies |
| A direct Play fails on retrigger | A previous instance still claims the same property | Stop the earlier handle before starting the replacement |
| Marker gameplay never fires | Wrong timeline/marker identity, premature completion or disconnected Reached branch | Select the effect's embedded timeline, repair the marker selection and wire the outcome explicitly |
| Gameplay fires on cancellation | Success work was wired to common Next | Move hit/success logic to Reached or Completed as appropriate |
| The sequence finished but the effect is still active | Bounded particle lifetime tail | Wait for effect completion when full cleanup matters; Stop removes particles immediately |

Missing, inactive, destroyed or incompatible optional targets skip only their
work and report bounded diagnostics. Required broken bindings fail validation
or cooking. Entity UUIDs are saved; generation-checked handles are resolved at
runtime. Never repair a source file by inserting runtime indexes or generations.

## Preview, saving and stale output

**Compile/Validate** checks a draft. **Save** persists it. Build and Play require
open modified Blueprint, timeline and effect documents to be saved or discarded.
An externally edited file blocks a conflicting save; compare your draft with the
file before choosing Discard / Reload.

The dependency viewer reports why reflection, Blueprints, timelines, effects,
scenes, generated output or exports became stale. A last-valid artifact is retained
only for explicit stale preview/navigation. It cannot certify current Play or
export. Repair the source, reimport changed raw media when needed, then stage and
build again. Restoring a source file alone does not certify an older executable.

With Auto compile enabled, saved semantic changes stop old Play, rebuild and
restart from authored defaults after edits settle. A compile error leaves Play
stopped until repaired. Explicit Stop cancels automatic restart. There is no
native C++ hot reload or object-layout patching on PSX.

The dedicated effect preview uses production fixed-point simulation but a host
renderer and neutral lighting. It shows asset defaults, not per-instance overrides.
The older Scene view particle preview is not the effect parity reference. Use
Game view for gameplay callbacks, required scene bindings and console rendering.

## Capacity and performance

Inspect sequence, effect, particle and sprite counters together. A full pool
rejects new work safely; it does not promise the scene will maintain its target
frame rate. Layers and particles share budgets with ordinary scene emitters.

Reduce simultaneous plays, continuous emission, burst counts, lifetime or
overdraw when dropped work or frame cost rises. Compare at the intended display
resolution with scene geometry and HUD enabled. [Performance](performance.md)
describes native profiling; [Sprites and particles](sprites-particles.md) lists
the shared budgets. Check effect completion and cancellation rather than using
a guessed Delay to clean up a play.

## Build and export checklist

1. Compile current native reflection and Blueprints; validate the used timelines
   and effects. Resolve required binding and type errors.
2. Save the documents and scene. Build/Play from current sources.
3. Check the relevant gameplay, cancellation, pause and scene-change paths in
   the emulator. Confirm dropped-work counters at the intended load.
4. Export the standalone project and rebuild it with its build script. Do not
   edit generated sources as a substitute for fixing the original assets.
5. Record the platform actually checked. Emulator execution and host preview
   do not establish physical PSX timing or untested host-platform support.
