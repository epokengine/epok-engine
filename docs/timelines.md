# Timeline assets and scene directors

For step-by-step effect authoring, use [Using the VFX editor](vfx-editor.md).
[Connect a spell to Blueprint gameplay](spell-tutorial.md) covers spawning,
markers and cancellation. This page describes scene directors and the shared
timeline reference, including limits, bindings and cache behavior.

TimelineAsset creates and validates reusable source assets and previews their
property values. TimelineComponent binds an asset to scene entities and plays
its generated typed tables on PSX. ParticleEffect layers and transient effects
use the same TimelineAsset; Blueprint marker and completion waits use bounded
continuations. Existing Blueprint scalar timelines remain available.

Choose **New Timeline** in the Project browser, or double-click an existing
`.timeline.json` file. Add a typed binding slot, select a reflected class, and
choose an explicitly animatable property. Add two to four keys and markers,
then scrub **Preview tick**. Values use integer Q12 evaluation, including on
the host. There are 4096 ticks per second. The [spell example](../examples/timeline-spell/README.md)
contains a charge curve and Cast, Impact, and Aftermath markers.

Use **Validate** to cook immutable tables and refresh the preview cache. Source
files remain JSON below the project's `assets/`. Derived tables and host source
maps live below `.epok/timelines/`. Add **Timeline Component** in the entity
Inspector, select an asset, and assign its typed binding slots. **Play on start**
starts the sequence once the owner is active. The scene stores asset and entity
UUIDs; the build resolves generation-checked handles and stages typed accessors.

Opening a sequence preserves the current workspace layout. The dedicated
Scene/Sequencer layout remains an explicit Layout menu option. In the Sequencer
track list, expand vector tracks to see X/Y/Z channels; each value follows the
playhead. Editing a value creates or updates a key at that time. The adjacent
controls jump to the previous/next key or add one. Value edits support Undo/Redo.

With **Scene preview** (the eye button) enabled, scrubbing and Sequencer Play
preview TimelineTransform position/rotation/scale and TimelineCamera field of
view in Scene. The preview uses the same Q12 curves and section timing as cooked
playback, on a disposable scene copy: saving/building the map keeps its authored
values. Disable the eye or close the sequence to restore the authored view.
Preview targets appear beside binding rows and can be changed there; bindings
from a matching scene TimelineComponent take precedence over automatic selection.
Automatic selection only chooses an unambiguous compatible object/component.
These preview-only choices do not modify the reusable asset or its scene player.
Missing bindings are reported in the Sequencer status row. Other adapter fields,
custom script properties and event execution still require Game/runtime playback.

## Component adapters

Choose **Install Timeline Adapters** in the Project browser, or run
`--project <project> --install-timeline-adapters`. This adds the version 1
`assets/scripts/TimelineAdapters.hpp` source library. Repeating installation
preserves existing project edits. Attach an adapter through the ordinary script
Inspector or derive a Blueprint from it, then select its class in a timeline slot.
The component itself must already exist on the bound entity.

| Adapter | Animated properties and actions |
| --- | --- |
| TimelineTransform | Local position, rotation and scale |
| TimelineCamera | Transform, field of view, activate camera |
| TimelineAudio | Volume, pitch, play and stop the configured clip |
| TimelineLight | Transform, intensity, range and color |
| TimelinePalette | Cycle speed, reverse direction and reset |
| TimelineEmitter | Transform, rate, lifetime, velocity, spread, gravity, start/end size, play, stop and burst |
| TimelineRect | HUD position and size |
| TimelineText | Rect properties, color and wrapping |
| TimelineImage | Rect properties and color |
| TimelineProgress | Rect properties, progress value, foreground/background colors |

Configure initial values on the entity's actual components. Timeline playback
captures their live values; the adapter's script-field defaults do not initialize
components. Each write applies immediately, before crossing events, and changes
only its addressed property. RestoreInitial writes the captured component value
back immediately. A Blueprint may override gameplay events without losing this
runtime synchronization. As with other native scripts, source changes rebuild
and restart Play. These adapters use the existing one-script-per-entity model;
they do not add a second component or binding system.

Colors use normalized Q12 RGB with deterministic rounding to bytes. Writes clamp
field of view to 25–120 degrees, volume/progress/color to 0–1, pitch to 0.25–4,
light intensity to 0–2 and range to 0.01–128, palette speed to 0–60, emitter rate
to 0–512, lifetime to 1/60–60 seconds, particle sizes to 0–32, velocity/gravity to
±128 and spread to 0–128. HUD position/size use ±1024, matching existing component
validation (including signed sizes). Fractional limits use their nearest Q12
values: 0.01 is 41 raw and 1/60 is 68 raw. Palette speed zero pauses cycling.
Transforms retain raw Q12 values. Existing particle budgets still
apply; these bounds do not promise that the maximum settings meet a frame budget.
Emitter Burst takes an explicit count (zero is a no-op), accepts at most the
remaining space in the existing 256-request queue, and adds rejected requests to
the saturating particle dropped counter. Later particle-pool drops count only
accepted requests that cannot spawn.

The reflected `TimelineRequires=Camera` (or other documented component name)
class annotation declares a component requirement. Requirements accumulate through
the existing native/Blueprint ancestry. A required binding to a missing or
disabled component fails cooking; optional work is skipped with the director's
bounded diagnostics. Runtime checks are repeated after handle/generation and
activation checks, including after component removal. HUD Text/Image/Progress
adapters also require RectTransform. An ordinary Transform-typed slot requires
only the transform, even when the attached class derives from TimelineCamera.

On an **actor** the same requirement is read differently: an actor has no
component members of its own, so the requirement is satisfied by the component
class reserved for it being present in the actor's component set — `Camera` by
`epok::Camera3DComponent`, `AudioSource` by `epok::AudioComponent`, `Light` by
`epok::Light3DComponent`, and the HUD requirements by
`epok::RectTransformComponent`, `epok::TextComponent`, `epok::ImageComponent` and
`epok::ProgressBarComponent`. Neither check replaces the other: a legacy entity
keeps the entity-member check above, an actor gets the component-set one, and the
set of requirements is identical, so nothing that cooks today stops cooking.
`PaletteAnimator` and `ParticleEmitter` have no reserved actor component yet and
are checked on legacy entities only. See
`knowledge/initiatives/actor-architecture/p9-services.md`.

The reusable library passes a dedicated MIPS/standalone-export/PCSX integration
that inspects actual component values, plus PsyQo host runtime tests. This
supplements the original gate/camera lifecycle fixture. Audible output, pixel
parity and physical-console performance are outside that component-state check.

## Supported profile

Native C++ properties must explicitly declare `TimelineAnimatable`:

```cpp
EPOK_PROPERTY(EditAnywhere, TimelineAnimatable, Id="82b3be44-fc06-43f7-ae08-6e73ec1ec8e4")
epok::Fixed charge = 0.0;
```

Writable Fixed, Int32, UInt32, and Fixed[2]/Fixed[3] properties support Linear,
Step, Smoothstep, EaseIn, and EaseOut interpolation with Absolute or Additive
blend policies. Bool and signed 32-bit enums support Step/Absolute only. Easing
quantizes alpha to Q12; linear interpolation uses a wide integer numerator and
division toward zero. Unsigned curves preserve all 32 bits. Other types,
ReadOnly fields, and ordinary editable fields fail validation. Every native
profile explicitly declares its supported interpolation and blend modes.
LeaveFinal keeps the final output; RestoreInitial restores the value captured
before the first write on completion or cancellation, if the target is still
valid and active. Inactive optional targets capture their initial value when
they first become usable. Additive tracks always use a captured baseline.

Use **Add event track** for a public void function marked `TimelineCallable`
(crossing event) or `TimelineAction` (idempotent action). Each event key has up to
four typed arguments: scalar/vector literals, Texture/AudioClip asset references,
or compatible binding slots using the existing Blueprint EntityRef rules.
Mutable reference parameters are rejected. Blueprint variables expose the
**Timeline animatable** checkbox and functions expose **Timeline exposure**;
Blueprint callability by itself does not grant timeline access. Event and marker
tables share chronological order with persistent key/marker IDs as tie breakers.
Property scrubbing has no native side effects.

Limits are 32 authored assets per project, eight binding slots, 16 combined
property/event tracks, four keys per curve, 64 event keys, and 64 markers. These are conservative authoring limits,
independent of the runtime's eight director instances and eight components.
Pool exhaustion skips the new request and increments dropped-work diagnostics.
Existing particle limits
remain 64 emitters, 128 particles per emitter and 256 globally. The native sprite
triangle budget defaults to 2048 per frame and is configurable from 64 to 2048
in Project Settings → Rendering → Geometry.

## Identity, editing, and validation

Assets, slots, tracks, keys, and markers have persistent UUIDs. Property references
use the existing reflection IDs; slots use Blueprint's typed EntityRef contract.
Labels and display order never supply a fallback identity. Keys cook by time,
tracks by priority, Absolute before Additive, then ID, and markers by time then ID. Curves hold their
endpoint values outside the authored key interval, so equal-priority absolute
writes to the same property conflict throughout the asset duration. Two slots
bound to the same entity are also checked for conflicting writes.
Tracks sharing a property must agree on restoration policy.

**Test scene binding** checks UUIDs and class compatibility against the current
scene without changing it. Required broken bindings report errors; optional
broken or inactive bindings report diagnostics. Deleting or reordering an entity
cannot bind a track to a different entity by name or array position. Persistent
scene bindings compile through the existing Blueprint EntityRef resolver.

The director freezes while its owner is inactive or paused. Destroying an owner
or replacing a scene cancels its sequences. Optional invalid targets skip only
their properties/events; a stale handle never follows a reused entity slot.
Markers and events fire once per crossing in chronological order. Forward seeking
reconstructs explicitly idempotent actions and skips crossing events/markers.
Backward runtime seeking is rejected; restart to replay from the beginning.
Each advance accepts at most one asset duration; excess catch-up time is counted
and dropped. Ordinary fixed-step loop remainder is preserved. Concurrent
instances cannot claim the same entity/property; conflicting play requests fail.
`epok::sequence_stats` exposes active/peak instances, sampled properties, events,
markers, completion, cancellation, skipped work, pool/conflict drops and
diagnostic overflow. The diagnostic ring holds at most 32 entries.

Undo/Redo preserves IDs. Saving uses the existing asset transaction and blocks
when the file changed externally. Unknown fields and orphaned references remain
in the document for repair; unsupported document versions are rejected without
rewriting the file. There is no automatic conversion of old particle emitters or
Blueprint scalar timelines. Timeline source v1 migrates to v2 and Blueprint
sources v1/v2 migrate to v3 in memory without rewriting files or changing IDs.
Legacy metadata defaults to no timeline permission; reflection schema v6
regenerates caches before use. The
prototype reflection v2 Fixed/Linear/Absolute profile remains readable with its
original narrower permissions. Scenes using TimelineComponent or ParticleEffectComponent migrate to scene
v4; scenes without it retain v3. Template capture, placement, duplication, and
refresh remap binding UUIDs. Template bindings outside the captured subtree must
be cleared or replaced. Dynamic template components register after typed
initializers and before start callbacks, sharing the existing spawn rollback.

## ParticleEffect assets and lifecycle

ParticleEffect source v1 embeds this same TimelineAsset
and adds up to eight Sprite/Emitter layers. EffectLayerRef stores internal layer
UUIDs and the existing Clang catalog exposes the SDK's explicit layer controls.
The director uses a typed entity/layer handle binding; layers never occupy scene
entity slots. Source creation (`--new-particle-effect Name`) and validation
(`--validate-particle-effects`) are available. Scene source v4 accepts a
`particle_effect` component containing the effect asset UUID, external slot-to-entity
UUID bindings, enabled/play-on-start flags, and a seed override (zero uses the
asset seed). Builds cook reachable effects into immutable layer definitions and
the shared timeline tables. Layer order is canonicalized by UUID so authoring
reorder does not change deterministic seed assignment. Required broken external
bindings fail cooking; scene bindings cannot replace internal effect layers.
Template capture, placement and refresh remap external effect binding UUIDs.

The PSX runtime supports eight persistent or transient effect instances, sharing
the director's eight sequence slots. Local effect pause freezes curves, sprite
animation and particle age while retaining the rendered state. Inactive owners
hide and freeze their effects. Owner destruction and scene replacement cancel
owned effects; detached effects survive entity destruction but still cancel on
scene replacement. Normal completion stops emission, hides sprite layers and
drains existing particles for a bounded lifetime; explicit stop releases them.
Reusing a completed sequence's director slot does not cancel its particle tail.
`epok::effect_stats` reports active/peak effects and layers, spawned/dropped plays,
completion/cancellation, dropped burst requests and clamped tick work.
## Effect authoring and instance overrides

Choose **New Particle Effect** in Project, or double-click an effect source to
open its layers and embedded Timeline in the same editor. Fire, Smoke, Sparks,
Impact, Projectile, Aura and Rune presets create ordinary shared timeline tracks
and Sprite/Emitter settings. Layer controls reuse the existing sprite/emitter
Inspector, including atlas selection, flipbooks, blend modes and particle
budgets. Layer removal also removes its slot and target tracks; Undo restores
the entire effect with its original IDs. Effect and timeline edits share one
64-entry undo/redo history and one guarded atomic save. Artistic density,
violence/speed, scale, brightness, chaos/spread and direction controls edit those
same payloads. Burst density edits the existing event keys. Duration/retime
rescales the embedded timeline while preserving IDs, and rejects a duration
that would merge distinct curve keys. Advanced sprite/emission controls remain
available. The preview transport and counters stay visible while authoring scrolls.

Add **Particle Effect Component** in the entity Inspector to select the asset,
seed, play-on-start policy and typed external bindings. Component v2 adds explicit
per-layer overrides keyed by layer and reflected property UUIDs, using the shared
typed literal format. They initialize each instance before timeline capture;
animation tracks then take precedence. Other plays retain asset defaults. V1
components without overrides migrate without changing asset or binding IDs.
Orphaned layers, removed exposure and changed types fail cooking with preserved
source for repair. Scene/template undo captures the entire component.
**Open Particle Effect**
navigates to the source. `--open-effect assets/Effects/Fire.particle-effect.json`
opens it directly. The visual effect preview runs the linked production C++
kernels with typed compiled data, supports an eight-second loop, pause/restart
and seeking, and displays particle/drop/event/marker counters. It reuses the
host GPU sprite renderer, textures and blend modes with neutral asset lighting.
Required external scene bindings need the PSX Game view; absent optional work
uses the director's bounded skip diagnostics. Source or reflection errors stop
the preview and remain visible. Asset changes reconstruct data from the seed;
no native library is hot-reloaded. The asset preview displays asset defaults; per-instance overrides
are visible in the scene's PSX Game view.

## Blueprint playback nodes

Blueprint's **Sequence** and **Effect** node groups provide component Play,
Stop, Pause and Resume. The returned `SequenceHandle` and `EffectHandle` are
distinct transient types; their default is null and they cannot be instance
overrides or animation targets. **Effect / Sequence handle** accesses the shared
timeline for marker waits. **Burst enabled emitter layers** queues the requested
count on each enabled, playing emitter layer with the existing bounded queues
and dropped-work counters. It rejects stopped, draining and stale effects.

**Sequence / Play asset** and **Effect / Spawn asset** select a source asset by
UUID. The node exposes that asset's external binding slots as typed EntityRef
pins, keyed by slot UUID; labels can be renamed without reconnecting wires.
Required bindings must be connected and cannot be literal null. A checked cast
provides a class-constrained entity pin. Missing optional bindings safely skip
their work. Internal effect-layer bindings come from the effect pool.
These nodes specialize their typed interface at compile time; they do not accept
a dynamically chosen asset variable. Cooked adapters and referenced textures are
included even when the scene has no sequence or effect components.

Spawn takes an initial Transform, seed and optional owner. **Get Transform** and
**Make Transform** provide typed values. Spawn interprets the Transform in world
space; Get Transform reads an entity's local Transform, so it can be used directly
for a scene root. The owner controls lifetime/activation; direct spawned effects
keep their supplied placement, while persistent effect components follow their
entity's world transform. Null owner creates a detached effect;
zero seed uses the asset seed. Each direct Play/Spawn creates an independent
instance. Retriggering a Blueprint event replaces its continuation frame, but
does not stop a previously started direct play. Stop that handle before replacing
a sequence which writes the same properties; conflicting plays are rejected.

### Marker and completion waits

**Wait for marker** selects a timeline and marker by persistent identity, including
an effect's embedded timeline. Reached runs once for its first crossing; Completed
runs if playback finishes before reaching it, and Cancelled handles unavailable,
stopped or incompatible playback. **Wait for sequence completion** ends when
the timeline finishes; **Wait for effect completion** also waits for particle
drain. Next runs after the selected outcome branch. All waits suspend the current
event through its existing continuation slot; even an already reached result
resumes on a later tick. Inactivity retains results until dispatch can resume.
Destroying the consuming behaviour or changing scenes cancels its frame without
executing an outcome branch. Retriggering the event replaces its pending frame.

### Marker subscriptions

**Subscribe to marker** listens for future crossings of the selected marker,
starting after the revision observed at registration. Next continues immediately;
the listener owns a captured frame in the same eight-slot continuation table.
Reached runs once for each subsequent crossing, at most once per simulation tick,
then re-arms. Its branch
may contain Delay or another playback wait. A compact revision backlog in the
graph frame preserves crossings during that branch, inactivity or paused
dispatch, including after a terminal playback slot is reused. Completed or
Cancelled runs after the captured crossings have been delivered. Return from a
listener branch ends that subscription. Registering the same node replaces its
captured frame; consumer destruction and scene replacement cancel it. Stop the
playback handle to cancel the producer. No listener pool is allocated.
A subscription stores 40 bytes of playback state on MIPS, compared with 32 bytes
for an ordinary playback wait, plus captured arguments/temporaries and the
compiler's program counter and reentry epoch. The eight-slot continuation table
remains 204 bytes. Each subscriber preserves its marker's crossing order; gameplay in
separate behaviours follows the normal behaviour dispatch order.

For reproducible headless comparisons, run `--project <folder>
--preview-particle-effect <source-path> --steps 96`. The JSON trace contains raw
Q12 particle state, sprite output and counters, with at most 482 fixed steps.

## Dependencies, rebuilds and stale output

The cache signature includes the timeline's semantic data, compiler/schema
versions, consumed class ancestry, selected property/function signatures, and
referenced imported-asset hashes. Unused properties and defaults do not affect
the cooked curve signature. Label/layout/reorder changes retain table
timestamps. Reflected renames keep IDs while invalidating typed accessor metadata.
Unrelated classes do not invalidate the asset. Failed validation retains the last
valid artifact only with an explicit stale flag and diagnostics; compile commands
always validate current sources. Native edits still require rebuild/restart Play.
The source watcher stops an outdated Play session through the existing worker.
With Auto compile enabled, it waits for that worker to finish and for source edits
to settle before rebuilding and relaunching. Invalid sources leave Play stopped;
repairing them retries the requested Play target. An explicit Stop cancels that
automatic restart. Save or discard an open modified Blueprint, Timeline or effect
before Build/Play. Native code and component state are never patched in place.

The host-only `.epok/ArtifactDependencies.json` records the compiler's existing
IDs, dependency signatures and stale reasons. Reflection changes and valid source
edits/deletions invalidate affected timeline previews even when their windows are
closed. Removing a dependency retains its previous consumer edges and last valid
signature for diagnosis. Restoring a source requires fresh validation of its
outputs. Existing preview-cache footprints migrate into this graph without
changing authored IDs, and its identity data survives moving the project.
Persisted Blueprint compilations also publish the exact snapshots used to
generate their code, including direct plays, marker references, reflected calls
and imported resources. Source changes invalidate those recorded consumers;
layout-only canvas edits do not. Unsaved canvas validation never publishes a
project artifact.
Class reference defaults and literal values record the chosen class ancestry as
well as the declared base. Reparenting that class invalidates the Blueprints
that use the choice; recompilation diagnoses incompatible references.
Generated timeline/effect headers also record their content and dependencies for
each build or export directory. Changing an input marks affected headers stale;
rebuilding one destination does not certify earlier exports. Effect headers track
the texture revisions and shared-table mappings they actually reference. Failed
staging preserves previous signatures as stale diagnostic data. The
`stage-playback:*` records cover these headers, not the complete executable.
Generated `scene.hh` files record their actual scene inputs, registered maps,
resources, script metadata, Blueprint sources and effect override contracts.
An export of a saved scene and a build of an unsaved editor scene retain distinct
provenance. Editing the open scene or a consumed registered map requires a fresh
build; closing a document marks its former in-memory results stale. A completed
lighting bake updates a derived cache and does not restart Play by itself.
Native source bytes and generated Blueprint code are recorded per staged file.
The stage manifest includes successful writes, including unchanged files, and
excludes abandoned files left in the output directory. A build checks that
manifest before and after compilation; changed native sources, build options or
staged files prevent the result from being certified for Play. Starting another
compilation marks the previous executable stale until a new build succeeds.
Exports record their own staged inputs and documentation. Rebuilding Play does
not certify a previously exported source bundle.

Generated audio payloads record only the imported clip snapshot used to produce
their bytes. Scene transforms, native script edits and other clips do not
invalidate an unchanged payload. Reimporting or removing the clip invalidates
each recorded build/export copy. Audio staging uses the project's selected asset
index and rejects a package whose ID or revision changed since selection.
The generated display header records the validated output of Project Settings;
settings used only by geometry cooking do not invalidate it. The existing scene
and executable dependencies still require a fresh build when their inputs change.

Open **Window > Artifact Dependencies** to inspect these records. Search by ID,
path or diagnostic text, or enable **Stale only**. Select a node to see its
inputs, direct consumers and further consumers. Click a dependency to navigate;
hover a further consumer to see one shortest path. **Back** returns through the
last 64 selections, and **Copy ID** copies the complete identity. Stale reasons
link to the originating dependency, including missing records. Signatures remain
visible as diagnostic history when an artifact becomes stale.
The panel reads the graph every second while open; **Refresh** reads it
immediately. It never compiles, saves or marks artifacts valid. A read error
keeps the last snapshot with an explicit warning. **Recorded** means no known
invalidation in that snapshot, not an independent validation of current files.

Timeline/effect source observation retains independent valid documents when
another file is unreadable or has an ambiguous UUID. Scene components and typed
Blueprint playback/marker nodes load the assets they reference; unrelated
malformed timeline/effect files do not prevent their cooking. Missing or ambiguous
required references still fail. The all-source `--compile-timelines` and
`--validate-particle-effects` commands remain strict project validation.
The open asset window diagnoses its own catalog identity without disabling an
independent preview. Component and Blueprint pickers retain valid choices while
showing catalog errors; ambiguous identities cannot be selected. Refreshing these
choices never rewrites authored references. A changed selected playback asset
requests validation of the open Blueprint, including unsaved nodes.

Auto compile follows the selected build destination's recorded dependencies for
timeline/effect changes. An unrelated malformed file does not stop Play. A changed
or broken used source stops the old worker; required errors fail its replacement
build, and repairing the source requests another build. Source history remains
independent of preview caching. Build and export also recheck current playback
sources before certification, so edits during compilation cannot certify stale
output. Project capacity violations and failed dependency-graph reads still
block automatic rebuilding until corrected.
Build and export certification also reread Blueprint sources through the existing
loader, independently of the editor watcher. Edits, removals, unreadable sources
and changes to the compiled class set invalidate old output. This includes adding
the first Blueprint to a previously empty project. Scene preparation and script
generation must agree on that class set. Older staged provenance without this
coverage requires regeneration; restoring source files alone never certifies an
old executable or export. Layout-only Blueprint changes keep their existing
semantic behavior and do not require code regeneration.

Before compilation, after compilation and before publishing an export, the same
checks reread consumed saved scenes, the map registry and rendering settings.
Imported packages are scanned afresh and validated, including source checksums;
changed, removed, corrupt or duplicate used assets invalidate their consumers.
Unrelated broken packages do not invalidate independent outputs. Editing a raw
import source requires reimport before it changes the authoritative package.
These checks preserve submitted unsaved editor scenes: a worker cannot replace
them with disk content or infer that their documents have closed. The editor's
document observer still owns changes to those in-memory inputs. Restoring disk
inputs requires fresh staging before retained output is certified again.
Play records source selection from the original submitted scene even when linked
Blueprint instances are refreshed before lighting preparation. Updated inherited
resources are cooked from the prepared scene and retain their own dependencies.

Audio banks use a separate catalog signature derived through the existing
reflection registry's inherited property resolution. Blueprint numeric variable
declarations/defaults and editor exposure flags do not change bank selection.
AudioClip declarations retain their IDs, names and defaults, including null:
their names determine which scene overrides supply clips. Changed or removed clip
declarations invalidate each affected build/export bank independently. A failed
catalog leaves retained banks stale; repairing the catalog requires successful
staging before they become current. Template and linked-instance component
overrides use the same typed applicator as template resolution: spatial changes,
color, activation and audio playback controls do not change bank selection.
Audio clip removal, script replacement and selected timeline/effect assets remain
observable. Unknown component members and invalid resource-bearing values retain
conservative signatures and their original validation errors. Inherited property
defaults use the Blueprint compiler's own declaration and ancestry pass: numeric
and other non-audio defaults do not invalidate audio banks, while explicit
AudioClip overrides (including null) remain dependencies. Unknown IDs or invalid
declarations retain all raw defaults for observation and still fail compilation.
This does not use retained reflection as a fallback or certify executable graphs.
Script values in scenes, templates and linked-instance overrides use that same
bound-class/inherited-property resolution. Known non-audio values and their
override bookkeeping do not change bank selection; unknown classes or members
remain observable. Persistent class IDs never fall back to a matching name.
Before build certification, the observer rereads native declarations and
Blueprint source declarations to refresh audio selection, including projects
with no Blueprints. It does not compile graphs or trust an old registry.

Native catalog reads now contribute metadata provenance to scene staging.
Legacy `.epokscript` files and their membership, validated Clang include snapshots,
extractor/compiler/libclang binaries, SDK make fragments and reflection settings
are rechecked before and after a build and before export publication. Repeated
reads during staging must agree. These checks reuse the current extractor and
its validated cache; they do not infer declarations from cached output.
Project inputs use relative provenance paths. External SDK/tool/include paths
describe host dependencies only and are never serialized into runtime tables.
The editor watches recorded external inputs in one background reader, hashing
their contents at most once per second without blocking the UI on SDK/tool files.
Changes, missing files and repairs follow the existing selected build/stage
dependencies and stop affected Play sessions before rebuilding. Preserved file
timestamps do not hide edits. Repeated missing files do not repeatedly restart
Play. Native inputs first discovered by a compiler query become watched after
that query; this is not a scan of arbitrary external source folders.
Retained scene staging without metadata coverage must be regenerated. Metadata
dependencies remain inputs to scene staging and native builds. Audio banks
depend on the current typed audio catalog and source selections, so changing a
native numeric default or implementation does not by itself invalidate a bank.
Invalid native metadata invalidates the catalog and dependent banks; repairing
it never certifies retained output without successful staging.

Native application builds query the configured Make rules and GCC's dependency
output before and after compilation. The captured inputs include transitive
C/C++ headers, generated playback tables, Make files, compiler options, linker
scripts, consumed archives and selected compiler/linker executables. Dependency
queries use the same flag order and target-specific optimization as object
compilation. Changed inputs force application recompilation even when file dates
were preserved; changes detected during compilation prevent launch. An unchanged
successful build permits Make to reuse its objects. This host provenance belongs
to the executable, so it does not invalidate generated debug sources merely by
being recorded. Standalone Make builds do not require the editor's capture files.

SDK construction now has its own input certificate. The configured Nugget Make
rules supply the source list and build options; GCC and the assembler supply
transitive C/C++ includes, assembly includes and binary inclusions. Changed inputs
rebuild SDK objects in a private directory and construct a fresh archive owned by
the application build. Existing shared SDK objects/libraries are not overwritten.
Missing sources, failed compilation or changes during compilation keep the old
archive stale and prevent Play. Unchanged certified archives are reused.
SDK capture/build commands take a nonblocking host lock; a competing editor SDK
build reports that the SDK is busy instead of sharing partial objects.

Standalone launchers build private SDK objects/archives from current sources and
force an application rebuild, without the editor or its certificates. A plain
standalone `make` dispatches to `build.ps1` on Windows or `build.sh` on Unix hosts.
Nested custom linker scripts, dynamically loaded tool components and external
source packaging remain unfinished coverage. Changed native input signatures
currently recompile the whole application, rather than individual objects.

The shared audio bank records its selected packages and scene resource inputs.
Moving an entity or changing AudioSource volume, pitch or playback controls does
not invalidate the bank. Changing a clip assignment does; saved-scene inputs and
the open editor document remain separate, and repairing a source does not certify
an old bank until staging succeeds again.
Standalone and embedded effect timelines contribute their typed AudioClip
references through the same resource collector. Curve, marker and event timing
edits leave that selection unchanged; adding the first clip or removing the last
one invalidates the bank. Missing, unreadable or ambiguous timeline sources keep
their dependent banks stale until successful staging. Changing a scene's selected
timeline/effect asset also updates its resource-selection dependency.
Blueprint graph literals use the same typed resource traversal as cooking.
Changing scalar logic or moving an AudioClip from an inline pin to a literal node
keeps the bank current when its clip selection is unchanged. Adding/removing clips
or changing directly played timeline/effect assets updates that selection.
Inherited defaults, declarations, template components and script-instance values
have typed selection projections; graph isolation does not bypass validation.

Unknown or invalid declarations retain conservative dependencies. Shared HUD
budgets, geometry and build lists also retain scene/project inputs rather than
promising per-field incremental generation. The graph does not bypass
current-source build validation. Supported compiler inputs and relocation checks
are described above; arbitrary external-source packaging is not provided.

Timeline/VFX projects can be copied with their caches and rebuilt at the new
location. The generated script manifest records project-relative authoring
dependencies, while build files reference their staged sources. Standalone
exports include the Timeline and Blueprint guides and can be built from a
separate folder without the original project, using the configured MIPS tools
and Nugget SDK. The Windows relocation check covers paths with spaces, identical
executables and unchanged authored files; it does not validate a different SDK
version or a macOS host.

## Command line

Headless authoring commands:

```powershell
target/debug/epok-editor.exe --project MyGame --new-timeline Cast
target/debug/epok-editor.exe --project MyGame --compile-timelines
target/debug/epok-editor.exe --project MyGame --new-particle-effect Fire --preset Fire
target/debug/epok-editor.exe --project MyGame --validate-particle-effects
```
