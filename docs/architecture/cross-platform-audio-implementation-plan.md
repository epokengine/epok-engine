# Epok PSX-First Portable Audio Implementation Plan

Status: proposed implementation plan, 2026-09-12.

Implementation checkpoints and validation evidence are tracked in
[cross-platform-audio-progress.md](cross-platform-audio-progress.md).

This plan specializes the source/cook/runtime boundaries defined by
`multi-platform-architecture.md`. Its implementation scope is sampled audio,
streamed audio and MIDI-like sequenced music for PSX. It preserves source and
runtime boundaries suitable for later N64, PS2 and GameCube work without
implementing any of those backends in this initiative.

## 1. Outcome and decisions

Epok will expose portable audio intent to the user and produce a different
cooked representation for each build target.

The principal decisions are:

1. WAV, FLAC, Ogg Vorbis and MP3 are source formats. They do not determine
   whether the runtime keeps the sound resident or streams it.
2. MIDI is a sequence source. A MIDI asset depends on a `SoundBank` containing
   the instruments and samples needed to render it.
3. `Load Mode` is a portable setting with `Auto`, `Resident` and `Stream`.
   `Auto` resolves deterministically from a versioned target profile and the
   resolved choice is visible before building.
4. Sequence event data is resident in the first implementation. It is normally
   small enough that streaming the event list would add complexity without a
   useful saving. Its `SoundBank` has a separate residency policy.
5. XA, SPU ADPCM, VAB, SEQ and SEP are PSX cooked or compatibility formats, not
   universal project formats. Future consoles must be able to produce their own
   payloads independently from the original source or neutral host IR, but such
   cookers and runtimes are outside this plan's implementation scope.
6. Import preserves the original source snapshot and settings in the
   `.epokasset`. Target cooking is cached but disposable. No target is cooked
   from another target's lossy output.
7. Explorer preview uses host-readable source/IR. A separate target-preview
   option may audition decoded cooked output. Preview never requires a running
   console or emulator.
8. Audio capacity failures are explicit. A build never silently changes
   `Resident` to `Stream`, drops MIDI events, changes polyphony or substitutes
   an unsupported codec.

### 1.1 Scope boundary

In scope now:

- portable authoring settings and neutral host IR where needed to avoid PSX
  data becoming the permanent project representation;
- editor detection, import, reimport, Inspector and preview;
- existing PSX resident SPU-ADPCM and streamed XA behavior;
- MIDI, `MusicSequence`, `SoundBank` and PSX sequenced playback;
- optional Sony/converted SEQ PSX compatibility importers after the normal MIDI path.

Explicitly out of scope:

- N64 audio cooking or runtime playback;
- PS2 audio cooking or runtime playback;
- GameCube audio cooking or runtime playback;
- SDK selection, codec integration, mixer work, packaging, profiling or hardware
  validation for those future consoles;
- UI controls that claim those targets are available.

Multiplatform readiness in this plan means preserving original source data,
using semantic settings, keeping target overrides namespaced and preventing PSX
addresses/formats from entering the authoring schema. It does not mean building
a second console backend now.

## 2. Current state and architectural fit

The current implementation already supplies several useful pieces:

- Automatic source discovery and manual import for WAV, MP3, FLAC and OGG.
- Authoritative `.epokasset` packages containing the source snapshot, UUID and
  import settings.
- Background conversion and cache publication.
- Explorer and Inspector waveform preview plus host PCM audition.
- PSX resident mono SPU-ADPCM SFX with a checked sample-memory budget.
- PSX XA music cooking, disc packaging and one asynchronous CD music stream.
- A 24-voice PSX allocator with priority/age stealing and scene ownership.

The current `Usage::Sfx | Music` setting combines three independent concerns:
semantic role, storage policy and PSX encoding. `Music` means XA/streamed while
`Sfx` means SPU/resident. That coupling cannot describe resident music,
streamed ambience, sequenced music or another console's mixer.

The multi-console architecture already requires:

- source audio IR containing PCM, loop markers, channels and intended role;
- target-specific audio cooking;
- logical audio commands above backend-owned voices, streams and buffers;
- platform/profile capability validation;
- independent target payload and cache identities, with only PSX implemented by
  this plan;
- bounded callbacks, resource pins and explicit retirement.

This plan extends those boundaries rather than introducing a second asset or
build pipeline.

## 3. User-facing model

### 3.1 Source files

The first supported inputs are:

| Source | Imported asset | Purpose |
|---|---|---|
| `.wav`, `.flac`, `.ogg`, `.mp3` | `AudioClip` | Sampled SFX, ambience, dialogue or rendered music |
| `.mid`, `.midi` | `MusicSequence` | Notes, controllers, tempo and loop markers |
| WAV/FLAC sample collection plus mappings | `SoundBank` | Portable instruments for a `MusicSequence` |
| `.sf2` | `SoundBank` | Planned convenience importer after the explicit bank model is stable |
| Sony `.vab` or `.vh` + `.vb` | `SoundBank` | Optional PSX compatibility import, with portability/quality warnings |
| Sony `.seq`/`.sep` and converted LE32 `.sep` | `MusicSequence` | Optional compatibility import through separate identified parsers |

OGG and MP3 remain convenient inputs, including for resident sounds. They are
decoded on the host and cooked to the PSX representation selected by the
current profile. The schema does not imply that a future console runtime decodes
the original MP3 or OGG; a later named target profile would have to implement
and budget that decoder explicitly.

WAV or FLAC should be recommended for reusable masters and `SoundBank`
samples. Re-encoding a lossy source to XA, SPU ADPCM or another console codec
adds another lossy stage, so the Inspector should show a warning without
rejecting the asset.

### 3.2 AudioClip Inspector

Selecting an imported sampled-audio asset should show:

```text
AudioClip
  Source                 Music.ogg
  Role                   Music | SFX | Ambience | Dialogue
  Load Mode              Auto | Resident | Stream
  Channels               Preserve | Mono | Stereo
  Quality                Low | Medium | High | Custom
  Trim                    start / end
  Normalize              on / off
  Loop                    Off | Whole clip | Region
  Loop Region             start / end or imported markers

  Target                  PSX
  Resolved Representation XA stream
  Estimated Cost          disc, RAM buffers, voices and media requirements
  Compatibility           Supported / warning / error

  [Play] [Stop] [Source Preview | Target Preview]
  [Advanced Target Overrides]
```

`Role` provides defaults and bus routing; it does not force storage. `Load Mode`
controls residency. A WAV can be streamed and an MP3 can be resident.

Recommended `Auto` defaults are:

- short SFX and UI clips: resident;
- long music, dialogue and ambience: stream where the target supports it;
- short music stingers: target/profile decision;
- unsupported or over-budget resolution: a visible error, not a silent change.

The size threshold, selected codec/profile and quality mapping are versioned
cooker inputs. The Inspector displays the resolved result for every enabled
target so `Auto` is predictable rather than magical.

### 3.3 MusicSequence Inspector

Selecting a MIDI-derived asset should show:

```text
MusicSequence
  Source                 StageTheme.mid
  SoundBank              RetroBank.epokasset
  Role / Bus             Music
  Loop                    MIDI markers | Whole song | Off
  Music Voice Limit      Auto or explicit bounded value
  Tempo                   imported tempo map
  Duration               calculated
  MIDI Channels          used channels and programs
  Peak Polyphony         estimated from note lifetimes
  Unsupported Events     none or actionable list
  Bank Residency         Auto | Scene | Permanent

  Target                  PSX
  Sequence RAM           estimated bytes
  Sound RAM              bank/sample estimate
  Voice Budget           peak / configured / target maximum
  Compatibility           Supported / warning / error

  [Play] [Stop] [Source Preview | Target Preview]
```

MIDI contains no recorded instrument sound. Preview and builds therefore
require a `SoundBank`. A project may select a default bank so importing ordinary
MIDI files is a one-step experience. Epok should eventually include an
Epok-owned, redistributable small retro starter bank with explicit provenance;
until then the UI must request a bank rather than using an OS synthesizer whose
sound cannot match the console.

The starter bank need not reproduce the full General MIDI library. It may map
the 128 General MIDI programs to a smaller documented set of original Epok
samples, but every fallback mapping must be shown in the import report.

### 3.4 SoundBank Inspector

A portable bank contains programs and one or more sample zones per program:

- source sample dependency;
- MIDI program and optional drum key;
- key and velocity ranges;
- root key and fine tuning;
- volume and pan;
- ADSR envelope;
- sample loop start/end;
- priority and optional effect-send intent.

The Inspector shows missing programs used by selected MIDI assets, sample
deduplication, per-target encoded size, resident-memory cost and maximum tones
triggered by one note. Target constraints are diagnostics, not limits placed on
the source model.

## 4. Detection, import, reimport and preview

The intended flow is:

```text
copy/drop source into assets/ or use Import Content
    -> scanner identifies source kind and hashes stable bytes
    -> background probe produces metadata and a disposable preview
    -> user accepts defaults or edits import settings
    -> atomic .epokasset publication preserves source snapshot + settings
    -> host IR/preview cache is generated
    -> build cooks only requested target/profile outputs
    -> package includes only referenced cooked resources
```

Detection remains non-destructive. It may probe and preview a raw source, but it
must not publish an authoritative asset or cook every console before the user
chooses settings. Manual import and an optional future auto-import preference
use the same preparation/validation/atomic-commit path.

Explorer behavior:

1. Raw WAV/FLAC/OGG/MP3 can be played before import using the source decoder.
2. Imported `AudioClip` plays from its source snapshot with saved trim,
   normalization, channels and loop settings.
3. Raw MIDI can be parsed and displayed before import. It plays through the
   project's default preview bank if one is configured; otherwise Play explains
   that a `SoundBank` is required.
4. Imported `MusicSequence` plays through its assigned bank and the common
   sequence interpreter.
5. Source preview is immediate and platform-independent. Target preview uses a
   cached selected-target cook and decodes or emulates its result on the host.
6. Only one Explorer audition owns the host output at a time. Cancellation and
   late worker completion retain the existing bounded behavior.

Changing settings invalidates host IR where relevant and all affected target
caches. Unrelated target outputs remain reusable. Reimport keeps the asset UUID
and dependencies; a failed conversion leaves the previous package intact.

## 5. Authoring schema and neutral IR

The precise Rust module placement follows the staged workspace extraction in
the multi-console architecture. The logical schema should be equivalent to:

```rust
enum AudioRole { Sfx, Music, Ambience, Dialogue }
enum LoadMode { Auto, Resident, Stream }
enum LoopMode { Off, Whole, Region { start: Sample, end: Sample } }

struct AudioClipSettings {
    role: AudioRole,
    load_mode: LoadMode,
    channels: ChannelPolicy,
    quality: QualityProfile,
    trim: TrimRegion,
    normalize: bool,
    looping: LoopMode,
    target_overrides: TargetOverrideMap,
}

struct MusicSequenceSettings {
    sound_bank: AssetId,
    loop_mode: SequenceLoopMode,
    voice_limit: AutoOrU16,
    bank_residency: BankResidency,
    target_overrides: TargetOverrideMap,
}
```

This is a semantic sketch, not a commitment to the final serialized Rust
layout. Persisted enums and records require explicit schema versions and legacy
migration.

Neutral host IRs are:

- `DecodedAudioIr`: source sample rate, channel layout, normalized PCM access,
  trim/loop markers and provenance;
- `SequenceIr`: PPQN, tempo map, deterministic ordered events, loop regions,
  used programs and analysis results;
- `SoundBankIr`: programs, zones, source sample references, envelopes and
  tuning.

The authoring model never stores PSX SPU addresses, XA filenames, N64 mixer
records, device pointers, native padding or target-endian structs.

## 6. MIDI and sequence semantics

The first MIDI parser/compiler supports:

- Standard MIDI File format 0 and 1;
- PPQN timing; SMPTE time division is rejected initially;
- note on/off, including velocity-zero note-on;
- program change;
- channel volume, pan, expression and sustain pedal;
- pitch bend and a documented pitch-bend range policy;
- tempo and time-signature meta events;
- channel 10 percussion through explicit bank mappings;
- running status;
- end-of-track;
- deterministic loop markers named `loop_start` and `loop_end`.

Format-1 tracks are merged through absolute tick positions with a documented
stable ordering for simultaneous events. SysEx, aftertouch and unsupported RPN,
NRPN or meta events are listed by the import report. A setting may allow known
safe events to be ignored, but required musical behavior never disappears
silently.

The common sequence kernel owns:

- integer/fixed-point conversion from ticks and tempo to the audio clock;
- event cursor and loop state;
- per-channel program/controllers/bend/sustain state;
- note lifetime bookkeeping;
- bounded command emission and diagnostics.

Scheduling uses the audio service clock/callback cadence, not the rendered game
frame. Gameplay starts or stops a sequence as one logical operation; it does not
push individual MIDI notes every frame.

The kernel performs no normal-playback allocation. A target backend maps its
bounded note commands to hardware voices or native mixer channels and retains
ownership of DMA, callbacks, sample memory and voice retirement.

## 7. PSX target cooking

| Role | PSX cooked representation |
|---|---|
| Resident sampled audio | Current mono SPU ADPCM bank, with later PSX profile extensions only when verified |
| Streamed sampled audio | Current XA/CD path |
| Sequenced music | Epok PSX sequence payload plus SPU sample bank; optional Sony-format compatibility |

The interfaces and cache keys retain an explicit target/profile identity so a
future backend can be added without migrating project assets. This initiative
must not add placeholder N64, PS2 or GameCube cookers or successful no-op
implementations.

Each cook output records:

- target and profile IDs;
- payload kind/version and endianness;
- source/settings/dependency hashes;
- converter and tool fingerprints;
- encoded and uncompressed sizes;
- alignment and residency class;
- voice/mixer/channel requirements;
- stream buffers, decoder work memory and media bandwidth;
- warnings and hard capability failures.

### 7.1 PSX sequence format policy

Sony VAB/SEQ/SEP support is useful interoperability, but the core should not
depend on `libSnd` or call every compiled sequence a SEP.

The PSX implementation should distinguish:

1. Sony VAB/VH+VB input/output compatibility;
2. Sony SEQ/SEP input compatibility;
3. converted SEQ/SEP (LE32) input compatibility;
4. Epok's versioned PSX sequence and bank payload used by the native backend.

The three import parsers lower into `SequenceIr`/`SoundBankIr`. The normal MIDI
path compiles directly from those IRs and does not invoke old proprietary tools.
An optional Sony-compatible exporter may be added after byte-level fixtures and
runtime tests exist. The Epok runtime may use the existing sample uploader and
voice manager rather than reproducing the complete Sony VAB/`libSnd` API.

The first PSX sequence profile keeps event data in main RAM and sample data in
SPU RAM. It has a configurable music voice ceiling, initially defaulting to 16
to leave capacity for effects, but uses the same priority/ownership policy as
other logical voices. The selected policy and worst observed contention are
reported.

## 8. Runtime contracts

The existing logical audio design is extended without exposing codecs:

```text
AudioSource / Blueprint / game code
    -> PlayableAudio handle + gain/pitch/loop/priority/bus
    -> bounded AudioCommand queue
    -> Audio service
         sampled resident player
         stream player
         common sequence kernel
    -> platform backend
         physical voices/mixer
         sample/device memory
         media reads and decode buffers
         interrupts/callbacks and retirement
```

`PlayableAudio` resolves to an `AudioClip` or `MusicSequence`. A sequence pins
its event payload and `SoundBank` until all notes and backend commands retire.
Generation-checked handles prevent a stopped/destroyed source from controlling a
new voice. Scene teardown stops owners, drains asynchronous work and only then
releases stream/bank resources.

The runtime preserves:

- explicit play/stop/update/status results;
- priority and age policy;
- one owner for every physical device and media channel;
- bounded command/event queues;
- underrun, dropped-command, voice-steal and peak-residency counters;
- no gameplay callbacks from an interrupt or native audio callback;
- no successful no-op for an unsupported required feature.

For PSX, XA continues to leave all sample voices available but competes for the
CD reader. Sequenced music avoids continuous CD access but consumes SPU sample
RAM and voices. The Inspector and build report must expose this trade-off.

Future backends must supply their own voice, mixer, memory and media policies;
they must not inherit the PSX 24-voice model through this abstraction. No such
backend work is part of the current implementation.

## 9. Compatibility and budget presentation

The Inspector contains a target table for every enabled build profile:

| Target | Resolved mode | Encoded/package | Runtime memory | Voices | Media | Result |
|---|---:|---:|---:|---:|---:|---|
| PSX | XA stream | value | buffers | 0 SPU sample voices | CD bandwidth | Supported |

The current Inspector only needs to present PSX. Its underlying result record
retains target identity so later consoles can add rows when they actually have
implemented and selected profiles.

Unknown is not zero. Estimates become measured values after cooking, and the UI
labels which is shown. Global build diagnostics aggregate:

- resident and scene-resident audio bytes by memory region;
- stream buffers and decoder work areas;
- package/disc/ROM size and estimated media rate;
- sequence bytes and bank dependencies;
- configured and estimated peak polyphony;
- physical/logical voice pressure and steals;
- stream count, underruns and media-arbitration conflicts.

An asset may remain valid authoring data while being unsupported by a selected
target. The project browser marks that target error, and that target's build
fails with the asset UUID/path and corrective options. Other supported targets
can still cook.

## 10. Implementation phases

Each phase is reviewable and preserves unrelated working-tree changes. Existing
PSX SFX/XA behavior is protected by golden outputs and runtime tests before its
schema or service boundary changes.

### A — Specify portable audio intent and migrations

1. Record current PSX audio package bytes, generated bank declarations, XA
   staging, disc layout and runtime behavior as fixtures.
2. Add source-level `AudioRole` and `LoadMode`; migrate legacy `Sfx` to
   `role=Sfx, load=Resident` and legacy `Music` to
   `role=Music, load=Stream` for the PSX legacy profile.
3. Separate portable settings from target overrides and add audio capability
   records to the build profile/cook fingerprint.
4. Update Inspector mock/tests to show resolved target representation and
   budget. Preserve the current import dialog until package migration tests pass.

Done when legacy projects round-trip unchanged in behavior, old PSX derived
bytes remain equal, and the new schema can express resident music and streamed
ambience without mentioning XA.

### B — Extract shared source audio IR and improve preview

1. Move decoding, trimming, normalization, channel policy and loop markers into
   a target-neutral host adapter.
2. Make the current PSX SFX and XA converters consume the same original-source
   IR while preserving their exact outputs.
3. Extend detection/file-picker/source-kind tables to `.mid` and `.midi`.
4. Keep raw and imported sampled-audio audition working; add source vs selected
   target preview identity to the cache key and UI.
5. Ensure background probe/import cancellation cannot publish stale metadata or
   start late playback.

Done when WAV/FLAC/OGG/MP3 detection, manual import, reimport, waveform and Play
work as today, while load mode and source extension are independent.

### C — Add MusicSequence and SoundBank authoring

1. Implement bounded SMF 0/1 parsing into `SequenceIr` with deterministic event
   ordering and structured unsupported-event diagnostics.
2. Introduce `MusicSequence` and `SoundBank` package kinds, UUID dependencies,
   duplication/move/relink behavior and target cache invalidation.
3. Add a minimal SoundBank editor for sample zones, programs, drums, tuning,
   ADSR and loops. Reuse imported lossless `AudioClip` sources rather than
   duplicating raw sample bytes where package ownership permits.
4. Implement the allocation-free common sequence kernel and a host software
   backend for Explorer preview.
5. Add project-default SoundBank selection and create/license an Epok-owned
   retro starter bank with recorded provenance before enabling it by default.
6. Add MIDI duration, tempo, program, missing-instrument and peak-polyphony
   analysis to Inspector.

Done when a copied or manually imported MIDI can be assigned a bank, auditioned
in Explorer, moved/reimported safely and diagnosed without any console tools.

### D — Implement PSX sequenced playback

1. Define the versioned PSX sequence/bank payloads and golden byte fixtures.
2. Cook SoundBank samples from original PCM to SPU ADPCM; deduplicate identical
   samples and report header/main-RAM/SPU-RAM costs separately.
3. Cook MIDI events into a bounded event stream supporting the MVP semantics in
   section 6.
4. Refactor the PSX allocator only as necessary to represent SFX and sequence
   note ownership under one priority/age policy; preserve existing SFX results.
5. Service the common sequence kernel from an audio-time source with measured
   jitter, not from the rendered-frame tick.
6. Integrate play/stop/loop/volume/pitch/status, scene teardown and bank pins.
7. Test sequence music with concurrent SFX, XA disabled/enabled transitions,
   geometry/CD activity, voice exhaustion and SPU budget failures.

Done when one MIDI plus portable bank plays on emulator and physical PSX,
performs no continuous CD reads during sequence playback, loops correctly,
survives scene changes, and reports measured RAM/SPU/voice costs.

### E — Add compatibility importers

1. Specify and fixture official Sony VAB/VH+VB, SEQ and SEP variants accepted by
   Epok; reject ambiguous/corrupt variants with offsets and reasons.
2. Implement the converted LE32 SEP parser as a separately selected/detected
   compatibility profile.
3. Lower all accepted inputs into the same IR and preview/runtime paths.
4. Decode PSX-only samples for other target cooks only when no original PCM is
   available; show the re-encoding quality warning.
5. Add optional exporter work only after importer/runtime conformance is proven.

Done when format identity is never inferred only from `.sep`/`.vab` extension
and every compatibility fixture round-trips semantically through the IR.

### Work after E — explicitly not authorized by this plan

Stop after the PSX phases and their documentation/validation are complete. N64,
PS2 and GameCube implementation remains in the separate multi-console roadmap
and requires a future explicit task. Do not add their cookers, runtimes, SDK
dependencies, codecs, capability promises, preview modes or packaging here.

The completed PSX work must merely leave these invariants:

- original source remains available for an independent future cook;
- portable authoring does not contain PSX addresses or encoded blobs;
- PSX target settings are namespaced;
- runtime-facing audio commands describe logical playback rather than Sony file
  formats;
- adding a target does not require changing existing asset UUIDs.

## 11. Validation matrix

Minimum automated coverage includes:

- legacy package migration and unknown-field preservation;
- source hash/provenance and deterministic cook keys;
- MP3/OGG/FLAC/WAV decode, trim, normalization, channels and loop regions;
- resident/stream `Auto` resolution and explicit override rejection;
- MIDI format 0/1, running status, tempo, sustain, bend, percussion and loops;
- malformed/truncated/oversized source limits and checked arithmetic;
- deterministic simultaneous MIDI event ordering;
- SoundBank missing programs, overlapping zones and sample deduplication;
- preview cancellation, one-audition ownership and stale worker results;
- voice allocation/stealing, note-off, sustain release and scene teardown;
- stream underrun/recovery and resource pin retirement;
- per-target endian/alignment/format validation;
- current PSX SFX/XA golden bytes and runtime behavior;
- combined PSX audio/graphics/CD stress;
- hardware gates kept pending until actually executed.

## 12. Risks and controls

| Risk | Control |
|---|---|
| Source format becomes mistaken for runtime format | Separate source, intent, resolved representation and cooked payload in schema/UI |
| MIDI sounds different or silent | Required/default SoundBank, missing-program report and target preview |
| Music starves SFX voices | Configured sequence ceiling, shared priority policy and steal/peak counters |
| Bank exceeds device memory | Per-target pre-cook estimates and hard build validation |
| Stream interrupts geometry or underruns | Backend media owner, bounded queues and combined stress fixtures |
| Frame-based MIDI timing jitters | Audio-clock sequence service with measured timing |
| Lossy source is repeatedly transcoded | Always cook from preserved original source; show quality warnings |
| `Auto` changes unexpectedly | Versioned profile rules, visible resolution and cook fingerprint |
| Official SEP and game-specific SEP are confused | Header-based identified parsers and separate compatibility profiles |
| Future console assumptions leak into authoring | Capability descriptors and namespaced target overrides |
| Bundled bank has unclear redistribution rights | Epok-owned/generated samples or recorded permissive license/provenance |

## 13. Recommended delivery order

The authorized PSX delivery order is:

1. A and B: decouple role/residency and make the current importer portable while
   preserving PSX output.
2. C: deliver MIDI + SoundBank import and Explorer preview entirely on the host.
3. D: deliver PSX sequenced playback and measured trade-offs.
4. E compatibility importers may proceed after the neutral IR is stable; they
   are not required for ordinary MIDI authoring.
5. Stop. Future console audio begins only under a separate explicit task.

This ordering gives users a stable portable workflow early while keeping legacy
PSX SFX/XA production usable at every checkpoint.
