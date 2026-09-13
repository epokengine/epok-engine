# RPG spell timeline and fireball example

Play the project to cast a reusable seven-layer Fireball: casting sigil, moving
core, travelling embers, impact flash, sparks, aftermath smoke and residual glow.
Its embedded TimelineAsset owns the `Cast`, `Launch`, `Impact` and `Aftermath`
markers. Eleven typed property/event tracks use the same compiler and director
as scene sequences. The startup scene is `Combat.epokmap`. `BP_Fireball` spawns
the effect directly, waits for its embedded timeline's `Impact`, applies damage
once to the captured target, then waits for the particle tail to drain. Native
code supplies input and HUD updates; the Blueprint owns the spell choreography's
connection to combat. The target starts at 100 HP and each Impact deals 25 damage.

The first spell casts automatically. **Cross** casts again after completion,
**Circle** cancels the current effect, **Square** reloads the combat scene, and
**Start** pauses/resumes. Cancelling before Impact leaves HP unchanged. Destroying
the caster or replacing the scene cancels its continuation without running damage
or completion gameplay. The target's authoring UUID becomes a generation-checked
handle captured at cast time; changing the next cast's target cannot redirect an
already suspended Impact.

Open `assets/blueprints/BP_Fireball.epokbp` to inspect the cast and cancellation
graphs. `tests/fixtures/create_timeline_combat.py` explicitly regenerates that
Blueprint and scene with stable IDs; native declarations remain in `SpellCombat`.

Open `assets/Effects/Fireball.particle-effect.json` to edit density, violence,
scale, brightness, chaos, direction and duration, then preview with pause,
restart, seek, camera/background controls and visible particle/drop counters.
The atlas is the original MIT-licensed artwork from `rpg-2-5d-demo`. The explicit
fixture generator `tests/fixtures/create_timeline_fireball.py` recreates the
effect/texture source with stable IDs; it overwrites those example files when run.

The final authored burst uses 32 sparks. PCSX measured a peak of 51 particles,
76 particles emitted over the whole effect, no dropped particles/sprite work,
and complete cleanup. The largest completed-frame simulation sample was 182
scanlines. This is an emulator fixture measurement, not a hardware or 60 FPS
guarantee. Host/MIPS comparison covers all 350 steps, including sprite values.

The separate `Main.epokmap` retains the original presentation harness and scene
TimelineComponent example. Open `Timeline Spell.epokproject`, then double-click `Cast.timeline.json` in the
Project browser. Scrub the preview: charge rises from zero to one at Impact,
falls to 0.25 at Aftermath, and reaches zero at completion. Validate reports the
empty required Caster binding; choose the scene's Caster in Test scene binding.
These test bindings are temporary and never enter the reusable asset.

The Reset Charge event calls an explicitly exposed idempotent presentation
action at tick 4096. Its Fixed argument cooks to raw Q12. Scrubbing previews
property values and never invokes C++ actions or gameplay code.

The Caster has a TimelineComponent bound to this asset and to its own authored
UUID. Play runs the compiled charge curve and Reset Charge event. Inspect the
component to change its asset, bindings, enable state, or automatic playback.
The charge property currently has no visual output; it remains a small scene
sequence example alongside the Fireball combat scene.

From the engine repository after building both binaries:

```powershell
target/debug/epok-editor.exe --project examples/timeline-spell --compile-timelines
target/debug/epok-editor.exe --project examples/timeline-spell --open-timeline assets/Timelines/Cast.timeline.json
target/debug/epok-editor.exe --project examples/timeline-spell --open-effect assets/Effects/Fireball.particle-effect.json
target/debug/epok-editor.exe --project examples/timeline-spell --open-blueprint assets/blueprints/BP_Fireball.epokbp
target/debug/epok-editor.exe --project examples/timeline-spell --build-psx
```

The scene binding's class UUID and the track's property UUID refer to the
annotated `SpellPresentation` declaration. Renaming labels preserves those
identities. Removing `TimelineAnimatable` must fail validation, even though the
property remains editable.
