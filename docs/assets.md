# Assets, import and audio

Authored **EditableMesh** assets use the same UUID package and move/conflict rules, with editable geometry as their embedded source. They open in [Blockout](blockout.md) and do not enter audio import/reimport. **Terrain** assets follow the same rules with a compact binary height grid as their source, and open in [Terrain](terrain.md). An entity references the asset UUID and optional material-slot overrides; saved-scene dependency checks include these references.

Imported assets are portable `.epokasset` files under the game's `assets/` directory. Each contains a UUID, importer settings, a source checksum and a complete snapshot of the original file. Scenes reference the UUID. No neighboring `.meta` file or editor database is required to recover an imported asset.

The sampled-audio importer supports WAV, MP3, FLAC and Ogg Vorbis. MID/MIDI imports as MusicSequence and references a SoundBank. The working source is optional after import: the saved snapshot can regenerate derived data or be converted with different settings.

## Import workflow

1. Copy supported audio files anywhere under the project's `assets/` directory using the OS file explorer.
2. The **Files detected** notification offers **Import...** or **Omit**. Import opens the inbox; select a file to open its import settings.
3. Choose a `.epokasset` destination, **Role** (Sfx, Music, Ambience or Dialogue), **Load Mode** (Auto, Resident or Stream), quality, channels, trimming, normalization and whole-clip looping. Role and residency are independent: Music can be resident and Ambience can stream. The resolved PSX representation and compatibility are displayed. Source validation runs in the background; a validated package is published on success. Import does not cook a target. Valid authoring settings can be unsupported by PSX; the build reports that target error without changing the package.
4. Select the AudioClip in Project to see its waveform, derived size, source state and actions. Add **Audio Source** to an entity, select the clip in Inspector and use Play.

Project shows raw sources, imported assets and native scenes/scripts. Raw WAV,
FLAC, OGG and MP3 can be auditioned before import. `.mid` and `.midi` are detected
in the scanner, file picker and drag/drop, with SMF format/tracks/PPQN shown in
Inspector. MIDI contains no instruments and requires a SoundBank for audition.
Invalid imports display their error without producing an asset. Omitted files
and failed imports remain accessible across sessions. A changed source version
can produce a new notification.

**Source Preview** plays the original PCM at its original rate, applying saved
trim, channel conversion and normalization for imported clips. **Target Preview
(PSX)** encodes and decodes resident SPU-ADPCM, including block padding. XA target
decoding is currently unavailable and reports a diagnostic; Source Preview is
available. Select the preview mode in Inspector or the browser filter popup.
Both views share one audition. Stop, another audition, revision and mode changes
retire late worker results. Preview never publishes a package.

Failed audio auditions write one Console entry per Play attempt, including the
asset path and complete diagnostic. The message does not persist in the browser
footer or repeat on subsequent frames; trying Play again can log a new failure.

## MIDI, MusicSequence and SoundBank

Use **Content > Add > SoundBank...** (or **Imports > Create SoundBank...**) to
create a bank. Add a program mapping and select an imported AudioClip for each
zone. Edit key/velocity ranges, root key, fine tuning in cents, gain/pan, ADSR
in milliseconds and sample loops in original source frames. Ranges must not
overlap: this initial bank model selects one zone per note. Program and drum
key numbers are MIDI's 0–127 values. Channel 10 needs an explicit drum-key
mapping for its selected program. Missing mappings never select another tone.
Bank sample residency is independent of the referenced AudioClip's Load Mode;
a clip imported for streaming can still supply its original PCM to a bank.

**Content > Add > Retro Starter SoundBank** generates an original triangle
sample and a bank containing program 0. Both assets record MIT provenance;
there are no external recordings, OS instruments or General MIDI/percussion
fallbacks. This action preserves existing files and does not change project
defaults. If publication fails after the sample was saved, the diagnostic lists
that retained original asset for recovery.

Import a MID/MIDI and choose its SoundBank in **MusicSequence / SoundBank**.
Alternatively, select **Project Settings > Description > Audio > Project
Default SoundBank** and Apply; raw MIDI and sequences without an explicit bank
use that default. Original MIDI bytes, asset UUID, settings and sample/bank
dependencies survive moves, duplication and reimport. Referenced samples/banks
cannot be deleted through the asset manager while saved dependencies exist.

Inspector reports SMF format/tracks/PPQN, duration, tempo changes, used programs,
missing mappings, loop markers and logical peak polyphony. The MIDI graph shows
note activity, not a PCM waveform. Peak polyphony excludes bank release tails;
audition reports its observed active-voice peak, steals and clipped samples.
Source Preview uses the shared sequence interpreter and the original sample
snapshots. Explorer and Inspector share one Windows output and one bounded
audition worker. Changing a bank, sample, default bank or source retires stale
work; Stop resets output before freeing prepared audio buffers.

Supported SMF 0/1 behavior: PPQN timing, notes (including velocity-zero note-off),
programs, CC7 volume, CC10 pan, CC11 expression, CC64 sustain,
tempo/time signature and exact `loop_start` / `loop_end` markers.
Tracks merge by absolute tick, source track index, then original event ordinal.
Repeated notes use FIFO note-offs, including when an older voice was stolen.
New MIDI imports select **Musical v2** interpretation. It resolves RPN 0 bend
sensitivity (default ±2 semitones), RPN 1 fine tuning, RPN 2 coarse tuning,
Data Entry/increment/decrement, null selectors, Bank Select at Program Change,
sostenuto, Reset All Controllers, All Notes Off and All Sound Off. Controller
reset centers bend and expression but retains the configured RPN tuning.
CC91/92/93/95 at zero explicitly disable their effects; nonzero effects remain
unsupported in this checkpoint. CC1 modulation state is retained; instrument
destinations are part of the pending instrument-fidelity profile, not general
GM effect support. A nonzero bank must have an exact mapping; existing portable
banks provide bank 0 and report missing mappings instead of substituting it.

Existing packages with no interpretation field retain **Legacy v1**, including
fixed ±2-semitone bends and their saved unsupported-event acknowledgment.
Choose **MIDI interpretation → Musical v2** in import settings to upgrade
explicitly; reimport preserves UUID and source. The headless equivalent is
`--reimport-asset assets/Audio/Theme.epokasset --snapshot --midi-interpretation musical-v2`.
The interpretation participates in preview/cook identity. EPSQ v1 bytes remain
unchanged for the original event subset; extended operations require EPSQ v2.

SMPTE is rejected. SysEx, aftertouch, unknown RPN/NRPN, unsupported controllers
and unknown meta events appear in the report and block Musical v2 playback until
resolved in the source. Sequencer-specific meta `0x7F` is retained as opaque
authoring metadata but is advisory: it does not describe playable music and does
not block preview or conversion. **Ignore reported unsupported MIDI events** is available
only for the Legacy v1 interpretation. The original event ledger retains track,
tick, offset and message data; the legacy exception cannot bypass source-integrity
or bank-fidelity blockers.

Event Load Mode Auto resolves to Resident. The initial host profile also needs
a Resident/Auto bank. Whole and marker loops cut notes at the exclusive end and
restore programs/controllers/tempo from immediately before the loop start.
The host renders a bounded PCM audition with a reusable loop; WinMM loop lengths
are quantized to 44.1 kHz frames. Limits: 4 MiB SMF, 256 tracks, 65,536 events,
4,096 diagnostics, 2,000,000 note-state analysis operations, 128 zones and up to 128 host voices (Auto: 16). Audition PCM
is limited to 52,920,000 samples. Capacity overflow is a diagnostic, never silent
event loss. PSX sequence cooking/playback uses the separately validated profile
below; Source Preview works without console tools.

Headless examples (all destinations are project-relative):

```powershell
epok-editor.exe --project "D:/Games/My Game" --create-starter-bank "assets/Audio/Retro.epokasset"
epok-editor.exe --project "D:/Games/My Game" --create-sound-bank "assets/Audio/Piano.epokasset" --sample <AudioClip-UUID> --program 0 --root-key 60 --provenance "Own recording"
epok-editor.exe --project "D:/Games/My Game" --default-sound-bank <SoundBank-UUID>
epok-editor.exe --project "D:/Games/My Game" --import-audio assets/Audio/Theme.mid --sound-bank <SoundBank-UUID> --voice-limit 16 --sequence-loop whole
epok-editor.exe --project "D:/Games/My Game" --reimport-asset assets/Audio/Theme.epokasset --snapshot --sound-bank default --sequence-loop markers
```

`--import-sound-bank` accepts a portable JSON/YAML bank mapping document whose
zones reference AudioClip UUIDs. `--default-sound-bank none` clears the default.
`--ignore-unsupported` is the CLI's explicit acknowledgement. Sequence settings
use schema version 1; old projects simply have no default bank. Unknown settings
and namespaced target overrides are retained on round-trip.

Native filesystem events trigger reconciliation, with a 750 ms source-stability interval before offering imports. File size and modification-time caching avoids rehashing unchanged files; reported writes invalidate it even when timestamps are preserved. Opening a project starts a fresh scan; **Refresh** also discards that cache. Symlinks and directory junctions under `assets/` are skipped.

## External moves and recovery

The same rules apply when the editor was closed during the changes:

| Change | Result on the next scan |
| --- | --- |
| Move/rename a `.epokasset` or its folder within `assets/` | Its UUID is unchanged; scene references resolve at the new path. |
| Move an unchanged source and remove its old path | A unique matching hash reconnects it. The new source path is saved so later source edits are detected. |
| Move and modify a source before scanning | Identity cannot be inferred reliably. Use **Reimport / Locate source...** to enter its new project-relative path. The imported snapshot remains usable. |
| Delete a source or move it outside `assets/` | Playback and conversion from the snapshot still work. To reconnect an external file, first place it under `assets/`. |
| Copy a `.epokasset` using the OS explorer | Both copies have the same UUID. **Conflicts / Problems** requires an explicit resolution; neither copy is selected arbitrarily. |
| Delete/move a `.epokasset` outside `assets/` | Scene references keep the missing UUID and build reports it. Restore the package anywhere under `assets/` to recover them. |
| Move the whole project | Open its new folder in the Hub. Asset paths and source hints remain relative. |
| Delete `.epok/` while the project is closed | Build and import caches regenerate from the packages. |

Hash matching is a recovery hint, not identity. Several matching sources require manual selection. For duplicate packages, choose **Give this copy a new UUID** on copies intended to be independent. Existing references retain the old UUID: leave exactly one original package or reassign those references.

Editor **Move / Rename** preserves the UUID; **Duplicate** creates a new one. Destinations must be unused `.epokasset` paths. **Used by...** lists saved scene dependencies in Console. **Delete...** refuses assets referenced by the open scene or saved scenes, and retains deleted packages in `UserSettings/AssetTrash/`. Restore the last deletion through the window; older trash entries can be moved back under `assets/` with the OS explorer.

## Storage and consistency

```text
assets/Audio/Hit.wav                 # Optional working source
assets/Audio/Hit.epokasset          # Authoritative identity, settings and original bytes
assets/scenes/Main.epokmap        # AudioSource.clip stores a UUID
.epok/imported/<content-key>/       # Disposable SPU-ADPCM or XA, checksum and waveform report
UserSettings/ImportState.epokprefs      # Local omitted/error decisions
UserSettings/AssetTrash/             # Local recoverable deleted packages
```

Track `.epokasset` files in version control. They are binary and contain the source; retaining the working source too duplicates those bytes. Derived caches, trash and import decisions are local.

The package starts with `EPOKAS01`, two little-endian 32-bit lengths, JSON metadata and source bytes. Package/importer versions and the SHA-256 source checksum are validated. Audio settings schema 1 stores role/load mode independently and custom rates under `target_overrides.psx.sample_rate`. Legacy Sfx migrates to Sfx/Resident and Music to Music/Stream, retaining its custom rate and channels. Unknown audio options, target namespaces and metadata are preserved. Derived cache keys include source content, settings, importer version and target profile. Corrupt conversions regenerate.

PSX audio profile v3 resolves **Auto** to Resident for Sfx and Stream for Music,
Ambience and Dialogue. This rule does not inspect the extension or available
memory. Explicit modes never switch to fit a budget. Low/Medium/High resident
quality selects 11,025/22,050/44,100 Hz; streaming selects 18,900/37,800/37,800 Hz.
Custom uses the saved PSX rate override. PSX resident stereo is currently a target
error; select Mono to use that profile. No additional console is advertised.

Workers prepare candidates; only the owning editor session publishes packages.
Before publishing, it checks that the source and replaced package still match
the revisions it read. Failed source validation and stale candidates retain the
old asset. A later target cook failure leaves the saved authoring package intact.
Cook keys also include implementation/dependency versions and the actual selected
encoder binary hash. Target caches are disposable; a last-cook report distinguishes
encoded bytes, main RAM sample copies and SPU samples. Unknown costs stay unknown.
Temporary files reside beside their destinations; publication never overwrites
an existing destination for a new asset. Windows uses a same-directory
[MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)
rename without replacement. Closing drops the candidate receiver before releasing
the project lock. External applications do not honor that lock; allow concurrent
writes to finish before retrying an operation.

## Effects (SFX)

- Inputs include MP3, FLAC and Ogg Vorbis, decoded on the PC with Symphonia. Maximum source size is 32 MiB; decoded audio is limited to 52,920,000 floating-point samples (about 202 MiB).
- SFX RIFF WAV: mono/stereo PCM 8/16/24/32-bit or IEEE float32, input rates 8–192 kHz, maximum source size 32 MiB. Compressed WAV and WAVE_FORMAT_EXTENSIBLE are not supported yet.
- Stereo mixes down to mono. Offline resampling produces 11,025, 22,050 or 44,100 Hz, with optional normalization and PSX SPU-ADPCM encoding.
- Referenced clips reside in SPU RAM. Each UUID uploads once. Builds reject banks above 508 KiB; 4 KiB is reserved for capture buffers and reverb is disabled.
- Up to 24 concurrent voices, play/stop, whole-clip loops, volume 0–1, pitch 0.25–4 and priority 0–255. Higher priorities can replace lower ones; equal priorities replace the oldest voice. Lower-priority requests are dropped when all voices are occupied.
- ADPCM stores 28 samples per 16-byte block. Partial final blocks are padded; author loop boundaries accordingly. Loop entry resets prediction history for repeatable decoding.

There is no spatial attenuation, Audio Listener, mixer bus or effect chain yet.
Explorer and Inspector provide host audition. Console playback is verified in
PCSX-Redux; physical-console validation is pending. The runtime follows the
[PSX SPU register, ADPCM and memory documentation](https://psx-spx.consoledev.net/soundprocessingunitspu/).

In a C++ Behaviour with an assigned Audio Source:

```cpp
if (auto* sound = entity().get<epok::AudioSource>()) {
    sound->volume = 0.5;
    sound->pitch = 1.0;
    sound->play();
    // sound->stop();
    // bool active = sound->is_playing();
}
```

The generated bank uses compact indices after resolving scene UUIDs. The console resolves generated music filenames through ISO9660; it does no asset indexing, hashing or source audio conversion. Exports include the generated bank and native runtime.

## Sequenced PSX music

Assign a MusicSequence to AudioSource or a Blueprint audio reference just as with a sampled clip. Its events resolve to Resident; the SoundBank owns a separate residency setting. Stream is currently a precise target error for both sequences and banks. The PSX profile accepts 1–24 music voices (default 16), sharing the 24 physical voices with SFX according to priority and age. Four simultaneous sequence instances are available. Volume, sample pitch, autoplay, enable/disable, looping and scene teardown are supported. Sample pitch leaves musical tempo unchanged.

Inspector displays the last valid cook's sequence RAM, bank payload in main RAM, sample bytes in SPU RAM, authoring package size, configured voices and analyzed polyphony. A changed bank, sample, setting or project default invalidates that report. Build emits `audio/sequence-report.json` with input identities and resolved costs. EPSQ/EPSB v1 are little-endian PSX payloads embedded in the executable; they do not add separate disc files. Build's linked memory report includes their main RAM copies. Runtime heap/stack peaks are not inferred from these static sizes.

The current profile preloads all referenced banks. Combined SFX and bank samples must fit 520,192 SPU bytes; excess is an error and never changes residency. Exact compatible samples deduplicate inside a bank. Separate banks and standalone SFX use separate allocations. Sequence data is limited to 256 KiB per song. Bank loops round outward to 28-frame ADPCM boundaries, with explicit source/resolved loop diagnostics. Portable linear ADSR is quantized to milliseconds, tuning to hundredths of a cent, gain to Q12 and pan to Q14.

Target Preview cooks and decodes the PSX bank through the shared sequence kernel. Its report identifies the limitations: it does not reproduce SPU Gaussian interpolation, physical IRQ/key-on latency or contention with runtime SFX. Source Preview continues using original PCM. Neither mode claims native Sony ADSR/reverb fidelity. The initial emulator stress measurement observed a 2.923 ms maximum service cost and a 3.024 ms maximum service gap; these are observed maxima, not hard real-time guarantees. Physical PSX measurements remain pending.

## Sony and converted sequence source imports

`.seq`, `.sep`, `.vab`, `.vh` and `.vb` are import candidates. File contents
determine the verified profile: Sony SEQ v1, Sony SEP v0, the converted LE32
SEQ/SEP layout, or Sony VAB v7. Wrong extensions do not change format identity.

The converted profile is displayed as **Converted SEQ/SEP (LE32)** and saved as
`converted-seq-le32-v1`. This is Epok's descriptive name for the verified layout,
not an official Sony format designation or a claim of exclusivity to any game.
Its size and tempo are little-endian 32-bit fields; the complete 12-byte header,
post-event deltas and alignment are checked. Earlier experimental profile metadata
is accepted on read and uses the neutral identifier on the next save/reimport;
the source snapshot, asset UUID, selected ordinal and record hash are preserved.
Ambiguous, truncated or unsupported layouts fail with source offsets and reasons.
The accepted layouts and independent evidence are described in the
[format audit](architecture/cross-platform-audio-phase-e-format-audit.md).

In the MusicSequence import dialog, inspect the source and choose its song.
SEP entries are independent songs. Sony selection stores the original song ID;
Converted SEQ/SEP stores an ordinal and record hash. A changed or reordered converted entry requires
explicit reselection on reimport. The entire original container stays embedded.
The common IR also retains a typed source event ledger, including events that
cannot play in the initial profile. Supported notes/controllers/tempo use the
same source preview, target preview and PSX cooker as MIDI, with an explicitly
assigned portable bank. Sony loops, unsupported controls and channel-10 policy
can produce hard compatibility errors; the MIDI Ignore unsupported checkbox
cannot bypass these errors. Source loop commands and exact destination ticks
remain inspectable even when playback is blocked.

VAB imports create a **SoundBank with unresolved source parameters**. For split
files select the VH as Source and its VB companion explicitly; no filename pairing
is inferred. The package stores both original parts, their paths, offsets and
SHA-256 hashes. Both inputs are checked again before atomic publication. A VB-only
change queues one bank reimport. Missing working sources do not prevent inspection
or settings-only reimport from the snapshot. Relocating a pair requires explicit
source paths; Epok never guesses another companion.

The imported bank IR preserves up to 2048 tones, including overlapping layers,
all program/tone gain and pan stages, native ADSR words, bend ranges, modulation,
sample IDs, ADPCM blocks and predictor loop histories. Inspector lists all tones
and samples; unresolved values remain unknown in the common logical bank view.
**Sony bank playback is currently unavailable** until those parameters have a
verified conversion to the portable playback profile. Native ADSR, reverb,
layering and reference-rate/tuning are not silently approximated. The original
VB does not contain a waveform sample rate, so audition never invents one.
First-pass PCM is available in the source IR; predictive loops require additional
stateful fidelity work. Re-encoding this decoded lossy source would add loss.
No sample child assets or empty playable bank are fabricated.

These source adapters do not change EPSQ/EPSB payloads or add another console.
Legacy MusicSequence and native SoundBank settings remain schema 1; optional,
versioned source descriptors are added only when used, with importer version 2
for compatibility sources. Version-1 packages stay readable. Source snapshots,
UUIDs, unknown fields and namespaced overrides survive save/reimport. Cache
identity includes source selection and adapter implementation versions.

```powershell
epok-editor.exe --project D:/Games/MyGame --inspect-audio-source assets/Score.sep
epok-editor.exe --project D:/Games/MyGame --import-audio assets/Score.sep --song-id 19 --sound-bank BANK_UUID
epok-editor.exe --project D:/Games/MyGame --import-audio assets/Converted.sep --sequence-profile converted-seq-le32-v1 --song-index 0 --sound-bank BANK_UUID
epok-editor.exe --project D:/Games/MyGame --import-sound-bank assets/Bank.vab --provenance "Source rights statement"
epok-editor.exe --project D:/Games/MyGame --import-sound-bank assets/Bank.vh --vb assets/Bank.vb
epok-editor.exe --project D:/Games/MyGame --reimport-asset assets/Bank.epokasset --snapshot
```

Use `--sequence-profile sony-seq-v1|sony-sep-v0|converted-seq-le32-v1` to
select an explicit structural parser, including during inspection. Sony SEQ's
single song has ID 0 and must also be selected explicitly. MCP `asset_import`
accepts `sequence_settings.source_selection` and `vb_source`, using the same
validated import transactions as the UI.

## SoundFont instrument libraries

Import a compatible SF2 or SF3 from the Content browser as a **SoundBank**.
The library keeps its complete source snapshot in one asset; embedded samples
do not become individual AudioClips. Reimport, source relinking and snapshot
reimport retain its UUID. Library source files may be up to 256 MiB; this does
not raise AudioClip limits or PSX memory limits.

The Inspector shows the catalog and, for a MusicSequence using that library,
the exact bank/program, note/velocity coverage and matching layers. Missing
instruments are reported explicitly. MIDI percussion bank 0 maps to SoundFont
bank 128; other mappings require an explicit saved override.

The selected reference library is FluidR3Mono GM 2.315, identified by its complete
checksum in `resources/audio/fluidr3mono-2.315.json`. Its MIT notice is preserved
when importing that exact library. In the MusicSequence import settings, **Install
reference instrument library** installs the bundled source as one SoundBank and
selects it. Existing different files are never overwritten.

Choose **Musical v2** for MIDI RPN tuning and bank selection, then assign the
library. **PSX music conversion** offers Compact/Balanced/High presets and Custom
sample rate, ADPCM effort, sample-loop alignment/crossfade, release cap, filter,
effects, headroom and memory budgets. Hover an option or its `?` for two seconds
for help; clicking or focusing `?` shows it immediately. The converter preserves
required layers and reports adaptations and missing mappings.

**Analyze conversion** reports a draft's exact cooked costs. **Optimize to budget**
performs a bounded rate search; **Adopt proposed recipe** only changes the draft.
Save settings / Reimport preserves the asset UUID and source snapshot. Compare
**Source Preview** and **PSX Target Preview** in Explorer or Inspector after saving.
Target Preview decodes the actual EPSB samples and EPSQ sequence, with linear host
interpolation and no wet Room reverb; PSX playback renders the hardware result.
Draft audition before saving and broader fidelity/stress acceptance remain pending.

### Epok Pulse music driver

**Music driver → Epok Pulse** is the default for SoundFont-derived sequences.
The editor compiles MIDI ordering, pedals, tuning and modulation to EPSQ v3:
timestamped SPU commands and deduplicated instrument-start templates. The SPU
performs ADSR and sample playback; the console does not evaluate a SoundFont
envelope or LFO on every audio interrupt. Pitch/gain automation is evaluated
offline at 250 Hz plus MIDI event boundaries and emitted only when it changes.
EPSB v2 sample banks and the existing AudioComponent/Blueprint play API remain.

Epok Pulse is an original driver. Native-only builds omit the old software
synthesizer's per-voice states and MIDI FIFO storage.
Projects mixing native sequences with legacy/reference sequences keep both paths.

Hardware ADSR is an explicit target adaptation, not an exact SoundFont renderer:
rates/sustain levels are quantized, delay is omitted, and hold is folded into
decay. The cook and Target Preview report affected layers and offline voice steals.
Controllers that retime an already-playing volume envelope fail conversion rather
than silently losing their effect. **Software reference** preserves the previous
runtime path for fidelity comparisons. Source Preview remains unchanged.

EPSQ v3 is limited to 256 KiB, 32,768 commands and ten minutes per authored pass;
loops can repeat indefinitely. Excessive automation fails with a conversion
diagnostic. Target Preview uses the cooked command stream and a hardware-envelope
model; Gaussian interpolation, key-on delay, SFX contention and wet reverb still
require emulator/hardware verification. The authored source snapshot is preserved.

For Ironwood's `opening_02`, the delivered recipe is Custom, maximum 10,208 Hz,
450 ms release cap, Dry, 6 dB headroom, 21 physical music voices and 4,672 bytes
reserved for other resident samples. These are explicit target adaptations, not
universal defaults. Whole-song looping is separate from instrument sustain loops.
The Title AudioSource keeps autoplay and volume 0.7 and references the existing
MusicSequence UUID. See [delivery evidence and pending work](architecture/psx-midi-delivery.md).

## Background music (BGM / XA)

Choose **Load Mode: Stream** to decode the source to PCM, apply trimming/normalization and convert it to 4-bit XA-ADPCM at 37,800 or 18,900 Hz, mono or stereo. This applies to every Role. This is rendered audio: importing MP3 does not recover instruments, MIDI or a sequenced soundtrack. Lossy MP3-to-XA conversion adds another compression stage; WAV/FLAC are preferable masters when available.

A `MusicSequence` itself is always resident, so it cannot become an XA stream by changing its load mode. To make a disc version of a MIDI, render its **Source** interpretation with its assigned SoundBank to a WAV, then import that WAV as a separate streamed BGM AudioClip. This keeps the compact sequence available for EXE builds while the rendered master supplies the CD build. The render is deterministic from the saved sequence snapshot and SoundBank; it is not a recording of the editor audio device.

The same source can produce independent SFX and BGM assets with different destinations/settings. The original source snapshot and UUID stay in the package, including after reimport, moves or cache deletion. Version-1 WAV packages remain readable with their original SFX defaults.

Referenced BGMs automatically produce `.epok/build/epok.bin` and `epok.cue`, alongside the executable. **Play** boots that disc in PCSX-Redux. Keep BIN and CUE together; distribute the disc image for music-enabled games. A standalone PS-X EXE cannot supply its XA sectors. Exports carry `music/`, `disc.xml` and `SYSTEM.CNF`; see the runtime build instructions.

The runtime finds the generated file through PsyQo's ISO9660 parser, starts filtered XA playback at double speed, and routes CD audio into the SPU. It keeps music outside the sample bank and leaves all 24 sample voices available for effects. Initialization, seeking, stopping and switching are asynchronous. A small final data marker ends each track.

- One BGM stream at a time. Only one BGM Audio Source may have **Play on start** enabled. Other tracks can be started from C++.
- `play()`, `stop()`, `is_playing()` and volume work with both profiles. Music pitch is fixed at 1.0; builds reject authored pitch changes. Runtime pitch changes have no effect on XA.
- Higher/equal-priority BGM requests replace the current request; lower-priority requests are dropped. The SFX and music priority pools are separate. Calling `play()` on the active BGM restarts it.
- Loop repeats the imported, trimmed clip. Switching/restarting/looping requires a CD seek and can leave a gap. MP3 encoder padding and XA sector rounding can also add silence. Seamless loops, intro/loop regions, crossfades and SPU-streamed music are future work.
- Maximum trimmed BGM duration is ten minutes, additionally constrained by source/decoded-memory limits. The generated disc is conservatively capped at 74-minute CD capacity.
- This implementation inserts filtered padding sectors to maintain real-time XA spacing. Disc consumption is approximately 21 MB per minute regardless of mono/stereo/rate; lower settings reduce encoded audio payload, but do not reduce this initial disc layout's size. Sharing those unused sectors with other tracks/data is future work.
- The music service owns the CD controller. Arbitrary parallel CD file loading is not supported while playing; the current scene, scripts and effects are already resident.

`epok::music_stats` exposes state, starts, ends, loops, errors and the current clip index. States: 0 unused, 1 initializing, 2 idle, 3 seeking, 4 playing, 5 stopping, 6 error. Error codes: 1 reset, 2 ISO initialization, 3 file lookup, 4 playback command, 5 invalid end marker or marker DMA timeout. A bad end marker increments errors and stops the track. These counters support game diagnostics; they are not timing guarantees.

Setup installs pinned [psxavenc 0.3.1](https://github.com/WonderfulToolchain/psxavenc/tree/v0.3.1) and [mkpsxiso 2.30](https://github.com/Lameguy64/mkpsxiso/tree/v2.30), checked by archive hashes. They run only on the development PC. SFX conversion needs neither utility. XA sector layout and playback follow the [CD format](https://psx-spx.consoledev.net/cdromformat/) and [CD controller documentation](https://psx-spx.consoledev.net/cdromdrive/). Physical-console testing remains pending.

## Command line

Commands use the same project validation and exclusive lock as the editor. Source/destination paths are project-relative:

```powershell
epok-editor.exe --project "D:/Games/My Game" --import-audio assets/Audio/Hit.wav
epok-editor.exe --project "D:/Games/My Game" --import-audio assets/Audio/Ambience.wav --asset assets/Audio/Wind.epokasset --rate 11025 --loop --normalize
epok-editor.exe --project "D:/Games/My Game" --reimport-asset assets/Audio/Hit.epokasset
epok-editor.exe --project "D:/Games/My Game" --reimport-asset assets/Audio/Hit.epokasset --import-audio assets/Audio/NewHit.wav
epok-editor.exe --project "D:/Games/My Game" --reimport-asset assets/Audio/Hit.epokasset --snapshot --rate 22050
epok-editor.exe --project "D:/Games/My Game" --import-audio assets/Audio/Theme.mp3 --audio-usage bgm --rate 37800 --channels 2 --loop
epok-editor.exe --project "D:/Games/My Game" --render-music-sequence assets/Audio/Theme.epokasset --output assets/Audio/Theme-disc.wav
epok-editor.exe --project "D:/Games/My Game" --import-audio assets/Audio/Theme-disc.wav --asset assets/Audio/Theme-disc.epokasset --audio-usage bgm --loop
epok-editor.exe --project "D:/Games/My Game" --duplicate-music-sequence assets/Audio/Theme.epokasset --asset assets/Audio/Theme-compact.epokasset --bank-budget 110000 --minimum-sample-rate 400
epok-editor.exe --project "D:/Games/My Game" --import-audio assets/Audio/Theme.mp3 --asset assets/Audio/Stinger.epokasset --audio-usage sfx --trim-start 2 --trim-end 3
epok-editor.exe --project "D:/Games/My Game" --scan-assets
```

`--audio-role sfx|music|ambience|dialogue` and
`--load-mode auto|resident|stream` set independent authoring choices.
`--audio-usage sfx|bgm` remains a legacy shortcut that sets both choices and the
old default custom rate/channels. `--rate` explicitly selects Custom quality.

`--scan-assets` emits read-only JSON with package IDs, paths, sources, usability and scan problems. Reimport preserves the UUID/settings unless overridden; the UI also supports disabling normalization/looping. The interactive reconciliation service persists recovered source paths; a headless reimport can also save the recovered path.

AudioClip, MusicSequence and SoundBank use the same UUID package contract. Native scenes retain their path-based startup selection and C++ bindings remain name-based: arbitrary external script renames are not repaired.


FBX characters use separate Skeleton, SkeletalMesh, AnimationClip and Material assets. See [skeletal characters](skeletal.md) for import, preview, model reimport rules and PSX playback. The audio workflow below remains supported.
