# PSX portable audio implementation ledger

Scope: phases A–D, then PSX compatibility importers only after the MIDI path
passes its gates. No additional console backend is part of this work.

## Baseline — 2026-09-13

Repository: `58e370f` (`Show procedural 2D UI automatically in the scene editor`).
`git status --short`, staged and unstaged diffs were empty before changes.
The three architecture documents were read before editing. No ancestor or
repository AGENTS.md was found; the supplied RTK instructions apply.

Executed before changing production audio behavior:

| Command (all prefixed with `rtk proxy`) | Result |
| --- | --- |
| `cargo test --locked audio -- --nocapture` | exit 0; 15 passed, 1 ignored (real Windows output) |
| `cargo test --locked audio_legacy_golden_outputs -- --nocapture` | exit 0; captured the golden hashes; first fixture compile had a corrected type-name typo |
| `python tests/runtime/verify_spatial.py` | exit 0; 19 C++ executables, including audio, lifecycle, scene transitions and CD/XA ownership |
| `python tests/integration/verify_bgm.py` | exit 0; MP3 resident/XA import, source loss/move, reimport recovery, BIN/CUE boot, stereo XA capture, concurrent SFX, loop, priority, stop/retrigger/switch and mono completion |
| `python tests/integration/verify_audio_assets.py` | exit 0; UUID moves/conflicts, cache regeneration, actual emulated SPU capture, loop/one-shot, stop/play, volume/pitch and 24-voice priority |

The native host suite emitted existing PsyQo C4244 numeric-conversion warnings.
These are not suppressed or counted as new audio warnings. The baseline
emulator runs use the installed PCSX-Redux and existing harnesses serially.
Raw integration evidence is retained under ignored `artifacts/` and fixture
projects under `.epok/audio-verify-*` and `.epok/bgm-verify-*`.

`tests/fixtures/audio-legacy-golden.json` records hashes obtained from production
functions before migration. `audio_legacy_golden_outputs` reconstructs an actual
legacy untagged package, tests its load path and asserts exact ADPCM, generated
bank-header and all four XA-spacing outputs. Golden changes require an explicit
format/behavior decision, not regeneration to accommodate a regression.

Baseline costs: the 100 ms mono tone contains 2,205 source frames, 1,296 encoded
bytes, 1,344 bytes after 64-byte DMA alignment. Its main-memory copy and SPU copy
are separate costs. PSX has 24 sample voices; the existing sample budget is
520,192 bytes, with 4,096 bytes reserved for capture. Emulated SPU capture peak
was 12,294 in the audio-assets fixture. This is amplitude evidence, not latency.
Heap/stack peaks and hardware timing remain unknown. Physical PSX validation
has not been executed by this task.

## Phase A — compatibility gate passed

Authoring introduces AudioRole (Sfx/Music/Ambience/Dialogue), LoadMode
(Auto/Resident/Stream), and Quality. Legacy settings retain their exact rate,
channels, normalization and looping. `Sfx` migrates to Sfx/Resident and `Music`
to Music/Stream, both with Custom quality. Settings without Usage retain the
old Sfx/Resident default. The package envelope stays EPOKAS01; audio settings
use schema_version 1 and importer version 3. No UUID/source bytes change.

Custom PSX rates serialize under `target_overrides.psx.sample_rate`. Unknown
audio options, target namespaces and package metadata are retained. Other
namespaces are authoring data only; they do not advertise implemented targets.
Profile `psx-legacy-audio-v3` resolves Auto to Resident for Sfx and Stream for
the other roles. Explicit residency never changes due to rate or budget errors.
Quality presets map to the existing converter rates. The profile is included
in the disposable cook identity. A new key requires recooking, not source or
UUID migration.

Verified after migration:

- `cargo test --locked audio -- --nocapture --test-threads=1`: exit 0,
  19 passed, 2 ignored. Log: `artifacts/audio-phase-a-focused-final.log`.
- The ignored `audio_import_dialog_clicks_publish_package` test was then run
  explicitly and passed (exit 0). It operates production ImGui controls.
- `verify_spatial.py`: exit 0, including added free-slot/priority/oldest-age,
  disable, clip mutation and owner-stop assertions in `audio_disabled.cpp`.
- `cargo build --locked --bins`: exit 0. An earlier concurrent build could not
  replace a header-tool executable used by tests; the later build ran cleanly.
- `verify_bgm.py` and `verify_audio_assets.py`: both exit 0 again using the
  rebuilt editor; logs `audio-phase-a-bgm.log` and `audio-phase-a-spu.log`.
- Full tests with four threads: 348 passed, 2 failed, 31 ignored. Serial run:
  349 passed, 1 failed, 31 ignored. The remaining failure is the existing
  `gui::interaction_tests::scene_clicks_and_hierarchy_context_menu_use_real_imgui_events`
  assertion "User can select Console". It fails identically in a detached clean
  `58e370f` checkout at `.epok/audio-phase-a-baseline` (SDK/tool junctions only).
  The other failure was an ImGui context collision and passes serially.
- `cargo clippy --locked --all-targets -- -D warnings`: exit 1, with the same
  18 diagnostics as the clean baseline; both complete logs are retained. The
  one new large Cached variant warning was fixed by boxing its host Record.
- `git diff --check`: exit 0. Formatting was limited to changed files; unrelated
  formatter hunks in main.rs and memory.rs were removed.

The source/metadata/UI changes are in audio_import, import_settings, assets,
audio, music, content_preview, asset_ui, asset_inspector, main and memory. Other
Metadata constructors only initialize the new unknown-field map. Runtime code
is unchanged. Actual XA baseline hashes and the installed converter fingerprint
are saved in `artifacts/audio-baseline-hashes.json`; original evidence was copied
to `artifacts/audio-baseline/` before rerunning the emulator tests. The baseline
emulator editor was the pre-existing debug binary (hash
`7f88cb357bfff0422f66951e64b92725cb1b74306d1aa0a9bd77c937431f124a`);
the byte oracles were compiled from the inspected source revision.

Hardware remains pending. Phase B is enabled. Sequence RAM, bank costs,
sequencer timing and emulator/hardware MIDI evidence are still pending.

## Phase B — compatibility gate passed

`audio_ir::DecodedAudioIr` retains original-rate interleaved PCM, channel/frame
metadata, trim, normalization intent and loop range. The strict legacy resident
WAV decoder moved to `audio_decode` without changing its sample math. Source
audition and XA retain the original Symphonia decoder path; both PSX cookers
consume original PCM through the neutral boundary. This avoids narrowing the
stream decoder to the resident WAV profile's format subset.

Import validates source/portable edits but does not cook. A stereo Resident
AudioClip can be saved and auditioned from source while PSX cooking reports its
mono-only target error. Cook identities now include target/profile, source and
settings, importer, embedded implementation/Cargo.lock and the actual selected
psxavenc executable SHA-256. Generated audio resources depend on `audio-cook:UUID`;
observation invalidates these when their cook inputs change. A small last-cook
report supplies measured encoded/sample-copy/SPU bytes without hashing tools
or running a converter in the UI frame. XA working memory remains unknown.

Scanner, picker, external drop and audio filters/icons recognize MID/MIDI.
The bounded SMF probe inspects format 0/1, track lengths/count and PPQN; malformed
files, format 2 and SMPTE are diagnosed. Discovery never creates a package.
At this gate MIDI audition explains the required SoundBank; complete sequence
import and playback belong to C.

Source Preview applies saved trim/channels/normalization at the source rate.
Target Preview (PSX) encodes and decodes resident SPU blocks, including padding.
XA target decoding is explicitly unavailable. Modes appear in Inspector and
the browser filter popup. The shared audition uses generation, mode, file stamp
and source revision checks. Late same-path results, timestamp-preserving edits
reported by Refresh, mode changes and obsolete waveform workers are retired.
Only one audio worker and one thumbnail worker exist per bounded cache.

Validation (all shell commands prefixed with `rtk proxy`):

- `cargo test --locked audio -- --nocapture --test-threads=1`: 24 passed,
  2 intentionally ignored; `artifacts/audio-phase-b-focused-final.log`.
- Import-dialog real ImGui test and actual Windows output completion/early-stop
  test executed explicitly: both passed, `audio-phase-b-dialog.log` and
  `audio-phase-b-winmm.log`.
- Full serial suite: 352 passed, 2 failed, 32 ignored. One failure was the known
  baseline Console-selection assertion. The other recovery test assumed import
  created a cache; it now explicitly requests a cook before deleting the cache,
  and passes separately (`audio-phase-b-recovery.log`). The new stale-revision
  test initially omitted its temporary directory; corrected and passed, then
  included in the final focused run.
- `verify_spatial.py`: all 19 native executables passed. Existing SDK conversion
  warnings remain visible (`audio-phase-b-native.log`). No runtime changes in B.
- `cargo build --locked --bins`: passed. `verify_bgm.py` and
  `verify_audio_assets.py` passed with that rebuilt editor. They now verify that
  import does not cook, explicitly build, then remove the resulting cache and
  prove regeneration. All prior playback/UUID/priority/stop/XA checks remain.
- Emulated SPU capture peak remains 12,294. XA mono/stereo playback, loop, stop,
  retrigger, switching and concurrent SFX passed. Logs: `audio-phase-b-bgm.log`,
  `audio-phase-b-spu.log`; fixture `.epok/bgm-verify-1789229816478526700`.
- Final clippy `-D warnings`: the same 18 baseline diagnostics (20 error lines
  including compile summaries); one new collapsible-if diagnostic was corrected.
  Exact error-family comparison with baseline passed. Log:
  `audio-phase-b-clippy-final.log`.
- `git diff --check`: passed. Formatting stayed within the audio files changed
  by this task; main/project_browser/timeline_compile were not bulk formatted.

The FLAC/OGG fixtures are original stereo sine signals: 13,017 and 4,016 source
bytes respectively. All four sampled formats pass raw and imported playback
buffer tests with both explicit residency choices, without cooking. The legacy
100 ms tone remains 1,296 encoded / 1,344 aligned sample bytes. Host decode is
bounded to 52,920,000 float samples; host peak heap/stack are not yet measured.

### XA baseline qualification

Full XA file hashes are **not reproducible in the pinned legacy converter**.
The original baseline, A and B runs differ in bytes within each audio sector's
20 unused bytes after its 18 sound groups, and the resulting EDC. No differing
audio/header/layout byte was found. All three runs have these identical
defined-audio-plus-layout hashes (excluding only those 24 tail bytes on actual
audio sectors; keeping every interleave/end-marker sector):

- 226,592 bytes, 3 mono audio sectors:
  `ac2493ce9bd07196681f893036354964c56eb5e8b689870d82d4f470ee3dd24c`.
- 1,067,552 bytes, 57 stereo audio sectors:
  `902402226a74a767e07af03aa6a1a70f7a49d1f25e4455d2c780520d0a201788`.

The full hashes/tails remain in `artifacts/audio-phase-b-xa-comparison.json`.
`verify_bgm.py::verify_xa_golden` now asserts every defined byte against this
baseline and passed on B's actual cooked files. This does not claim full-file
determinism. The unchanged synthetic layout goldens still compare full bytes.
Upstream [psxavenc v0.3.1 XA encoder](https://github.com/WonderfulToolchain/psxavenc/blob/v0.3.1/libpsxav/adpcm.c#L268)
writes the 18 sound groups and calculates the sector checksum; the observed
tail variance is retained as a legacy tool limitation, not normalized silently.
A future correction requires an explicit XA profile/output revision.

No hardware validation has been executed. C/D timing, banks and their measured
costs remain pending; no additional console backend has been introduced.

Final build after the stale-revision guard and controlled formatting also passed
(`audio-phase-b-build-final.log`). C is enabled. The native host/PSX kernel will
share neutral C++ code through the existing host-native build boundary; it does
not introduce an unproven Rust-to-PSX ABI or bypass the roadmap's shared-runtime
prerequisites. Legacy PSX remains the production runtime.

## Phase C — authoring and host audition gate passed

MusicSequence and SoundBank are versioned package kinds, with original source
snapshots and UUID dependencies. `sequence.rs`, `sequence_ir.rs`, `midi.rs` and
`sound_bank.rs` supply portable settings, bounded SMF 0/1 parsing, analysis,
bank mappings and transactions. `assets.rs`, `import_settings.rs`,
`asset_manager.rs`, `main.rs` and `mcp_tools.rs` integrate import/reimport,
duplicate/move/relink, unknown fields and deletion protection. New projects and
old projects without `default_sound_bank` start with no bank; this optional
manifest field requires no destructive migration.

Content/Imports can create and edit banks, assign sequence banks and display
MIDI analysis. Project Settings > Description > Audio selects the project
default. UI changes are in `asset_ui.rs`, `asset_inspector.rs`,
`project_browser.rs`, `settings_ui.rs` and `editor.rs`; the manifest test
constructor in `staging_files.rs` was updated. The original sample's clip Load
Mode does not choose bank residency. Missing programs, percussion mappings,
sample ranges and unsupported events have actionable diagnostics.

The new `runtime/sequence_kernel.hpp` has no SDK, allocation or device addresses.
It uses integer PPQN time, stable event ordering, controller/tempo loop snapshots,
FIFO note-offs, sustain and bounded service work. Stolen or naturally retired
notes retain their pending note-off identity so a late note-off cannot release
a replacement. Loop ends are exclusive, cut tails and restore pre-loop state.
Pitch bend is fixed at ±2 semitones; unsupported RPN/NRPN/SysEx/etc. block playback
unless acknowledged explicitly. This is a portable C++ header shared through the
existing native build boundary, not an unproven Rust PSX runtime or a second
console backend. The 12-byte kernel event record is not yet a PSX file format.

`native/sequence_preview.{h,cpp}`, `src/sequence_preview.rs` and `build.rs` provide
the actual host software mixer. Original float PCM, saved channel/trim/gain
settings, zone ADSR/pan/tuning and sample loops feed it. Runtime render chunks
allocate nothing; preparation is bounded. Source Preview renders the intro and
one steady loop, then WinMM repeats the pinned PCM until Stop. Output loops are
quantized to 44.1 kHz frames; the kernel retains integer musical time. The
playhead maps the repeated buffer back to the source timeline. `preview_audio.rs`
retires all prepared headers after reset. `content_preview.rs` retains one audio
worker, generation cancellation, dependency/default-bank invalidation, revision
rechecks before/after rendering and a cheap MIDI note-activity thumbnail.

Content can generate Retro Starter, program 0 only, with an original integer
triangle wave and explicit MIT provenance. No OS instruments, external samples,
GM/percussion substitution or automatic default selection is used. Source WAV:
2,205 mono frames at 22,050 Hz, 4,454 bytes,
SHA256 `ea07a2fee216b8d554b358743b0184ce2ea4ff5fe243761d4ec88b8c02f47b4c`.
Publishing the two original assets preserves already-created originals and
reports their paths if a later publication fails. `docs/assets.md` documents
the actual GUI, CLI, policy, recovery and limits.

Validation, with full command output/exit results in `artifacts/`:

- MIDI focused suite: 11 passed, 2 intentionally ignored
  (`audio-phase-c-final-focused.log`). Parser tests cover format 0/1, stable
  simultaneous ordering, PPQN/tempo, running status, sustain, bend, percussion,
  loops, malformed/truncated/resource limits and unsupported diagnostics.
- Real ImGui bank selection/import and actual WinMM MIDI completion/repeating
  loop/stop both passed explicitly (`audio-phase-c-midi-dialog.log`,
  `audio-phase-c-winmm.log`); the final interactive rerun is recorded in
  `audio-phase-c-interactive-final.log`.
- Full serial suite: 364 passed, 1 baseline failure, 34 ignored
  (`audio-phase-c-full.log`). The only failure remains the Console-selection
  assertion in `gui::interaction_tests::scene_clicks_and_hierarchy_context_menu_use_real_imgui_events`,
  already reproduced on unchanged `58e370f`. Legacy audio goldens and all source
  preview/reimport/cache recovery tests pass in this run.
- `verify_spatial.py`: 20 native executables passed
  (`audio-phase-c-kernel.log`), including the new shared-kernel timing, service
  chunk independence, sustain, loop restoration, late note-off and overflow
  tests. Existing SDK C4244 warnings remain visible.
- `cargo build --locked --bins` passed (`audio-phase-c-build.log`).
  `verify_midi_authoring.py` passed against those executables: generated bank,
  project default, independent role/residency, moved asset/missing MIDI snapshot
  reimport, UUID retention, failed reimport preserving bytes and no target cook.
  Log: `audio-phase-c-cli.log`; fixture:
  `.epok/midi-authoring-1789233868539085600`.
- Final clippy still fails on exactly the 18 baseline diagnostics (20 error
  lines with compiler summaries), with identical error-family counts. Two new
  warnings found during C were fixed; no warnings were suppressed. See
  `audio-phase-c-clippy-final.log`. `git diff --check` passes.

Measured host fixture costs: 6,488-byte kernel, 12-byte event records; the
half-second sequence plus release/silence produces 22,977 stereo frames /
91,908 PCM bytes. Peak is one active voice, zero steals and zero clipped channel
samples for that fixture. Whole-loop PCM repeats frames 22,050..44,100. The
starter-bank fixture produces 109,548 PCM bytes. These are host measurements,
not PSX RAM figures. Peak host heap/stack, output-driver underruns, PSX code/RAM,
IRQ CPU time and timing jitter remain unknown until measured.

The first host test exposed a release-tail buffer ending one frame too early;
the buffer now includes the complete fractional-millisecond release plus 1 ms
silence, and the test passes. Two temporary-fixture setup errors (a missing
manifest field and a startup path outside assets/scenes) were corrected without
changing unrelated production behavior. No MIDI emulator/hardware evidence is
claimed in C. D is now enabled; physical hardware remains pending.

## Phase D — usable emulator milestone passed; hardware pending

The complete A–C checkpoint was snapshotted with hashes in
`artifacts/audio-phase-c-snapshot/manifest.json` (49 files). No commit was made.
The following D work passed its host/emulator gates with the known baseline
failures explicitly retained. Physical PSX validation is pending:

- Shared event lowering moved from the host mixer to `src/sequence_stream.rs`.
  The 11 MIDI tests passed after extraction (2 interactive tests excluded).
- `src/psx_sequence.rs` writes little-endian EPSQ/EPSB v1 payloads with bounded
  event/sample/zone counts, resident budgets, bank/sample dependency identity,
  implementation fingerprints, checksum-protected bank cache and build reports.
  Bank samples start from original PCM; exact compatible samples deduplicate
  within a bank. Independent banks remain separate. Loops round outward to
  28-frame ADPCM blocks and report the change; loop entrances use filter zero.
  The old resident encoder wrapper retains its golden output.
- `runtime/sequence_data.hpp`, `sequence_service.hpp`, `sequence_clock.hpp`,
  `sequence_lock.hpp` and `sequence_tables.hpp` implement payload validation,
  four start mailboxes, bounded physical starts, shared SFX priority/age,
  generation checks, delayed key-on and bank retirement pins. The proposed
  profile uses linear software ADSR in millisecond steps, with fixed-point
  pitch/pan. It preserves the retail BIOS and GPU Timer 1; Timer 0 requests
  service and Timer 2 measures elapsed CPU/8 ticks. Clock behavior is measured
  in PCSX-Redux as recorded below; physical hardware remains pending.
- AudioSource selection, scene drop, typed Blueprint references and staging now
  accept MusicSequence. Legacy runtime `AssetRef<AudioClip>` remains a playable
  handle alias; bank sample references stay strictly AudioClip. Inspector and
  linked memory reports include sequence/bank costs. Cooked-sample Target Preview explicitly
  does not emulate SPU interpolation, IRQ latency or contention.

Checks so far: `audio-phase-d-staging-regression-r2.log` reports 24 focused audio
tests passed, 3 ignored; legacy golden bytes pass. `audio-phase-d-cooker-preview.log`
reports 6 focused tests passed, including target audition, staging dependencies,
cache reconstruction and source-change invalidation. `audio-phase-d-native-service-r2.log`
reports 21 native executables passed, including retirement, priority exhaustion,
disable, XA transitions and clock-fault behavior. Host measurements: instance
pool 26,112 bytes; physical state 1,152 bytes; stats 64 bytes. The tiny bank uses
1,344 SPU bytes / 1,472 payload bytes; the non-loop sequence uses 76 bytes.
These are not measurements of PSX runtime CPU/stack or emulator audio output.

The first MIPS build failed due to SDK register macro/header collisions and a
missing event header. Both were corrected. An intermediate emulator fixture
assigned MIDI to the wrong entity; that run is retained as a failed test and
is not evidence of sequenced audio. The corrected final run
(`audio-phase-d-emulator-final.log`, exit 0) captures voice 1, verifies its
MIDI C4 pitch register (1217), positive/negative decoded samples (peak 7937),
looping, stop/restart and disable. A silent SFX reserves capture voice 0 to
prevent misidentifying legacy SFX as MIDI output. Maximum observed service cost
was 751.134 us. `audio-phase-d-emulator-evidence.json` contains the counters and
payload report; the fixture is `.epok/psx-sequence-1789237982613488500`.

`audio-phase-d-stress-test.log` and
`audio-phase-d-stress-1789237066937521300/report.json` record the six-phase
emulator stress pass: 25-note chord constrained to 16 logical voices, 24 competing
priority-255 SFX, paged geometry reads, XA takeover, sequence restart and scene
reload. Maximum observed service cost was 2922.572 us, gap 3024 us; 16 physical
steals and 75 priority-denied notes were visible, with no clock/capacity errors.
Geometry read count remained 2 during stationary sequence playback, then advanced
to 5 under movement and scene reload. Sequence playback itself reads no CD data.
These are measured maxima from these workloads, not worst-case guarantees.

Final focused tests: 5 PSX cooker tests passed, including a combined SFX/bank
overflow above 520192 bytes that preserves source bytes and explicit Resident.
21 native executables passed, including the exact 4096-command service bound
plus at most 128 emergency cut commands. Full serial Rust suite: 369 passed,
34 ignored, the single previously reproduced GUI baseline failure; no new
failure (`audio-phase-d-full-final.log`, exit 1). Clippy initially caught one
new redundant cast, corrected; final `audio-phase-d-clippy-final-r2.log` has
exactly the 18 baseline diagnostics and exit 1. `cargo build --bins` passed
(`audio-phase-d-build-final.log`). Outputs retain the existing SDK C4244 warnings.

Final small fixture: 76 sequence bytes, 1472 bank payload bytes including 1344
sample bytes, 1344 bank SPU bytes, 6991 authoring package/dependency bytes. Linked
static main RAM is 871232 bytes for the entire fixture game; SPU use is 6784
bytes including capture reserve and a separate 1344-byte legacy SFX. Payloads
are embedded in the EXE, not additional disc files. Heap/stack peaks and physical
PSX timing remain unknown. Inspector and build reports distinguish these costs;
the HUD now includes the bank uploads. Source and target previews share the
sequence kernel; target preview's SPU interpolation/latency limitations are
explicit. Evidence and failures remain under `artifacts/audio-phase-d-*`.

Final legacy emulator regressions also pass: `audio-phase-d-legacy-spu-final.log`
captures peak 12294 with unchanged one-shot/loop/pitch/24-voice priority;
`audio-phase-d-legacy-xa-final.log` passes BIN/CUE boot, stereo XA decode,
concurrent SFX, stop/restart/retrigger, priority and mono one-shot switch. Both
commands exited 0. These close the remaining D regression gate.

At the user's request a subagent prepared E format evidence, diagnostics,
synthetic fixtures and separate reviewable parser patches in parallel. Production
integration starts only after this D checkpoint. No additional console is enabled.

## Phase E — verified source-import and sequence subset gate passed

Production integration began after the 70-file D snapshot at
`artifacts/audio-phase-d-snapshot/manifest.json`. The user explicitly authorized
the format subagent. Its isolated preparation did not advance a production gate.

`sequence_compat.rs` implements separate verified Sony SEQ v1, Sony SEP v0 and
converted SEQ/SEP (LE32) parsers, with exact source offsets, bounded counts, complete
typed event ledgers and source loop intent. `sequence.rs` centralizes catalog,
selection and decode. Sony selection retains song IDs; converted SEQ/SEP retains ordinal
plus record hash and requires reselection on changed/reordered input. The entire
container is authoritative. Unsupported Sony semantics are hard playback errors
independent of the MIDI acknowledgment flag. The common preview/kernel/cooker
path remains the only playback implementation; no libSnd or format-specific
console player was added.

`vab_import.rs` retains complete combined VAB or explicitly paired VH/VB sources,
sparse programs, up to 2048 source tones, all overlapping layers, raw envelopes,
tuning/gain/pan/modulation, sample IDs, first-pass PCM, ADPCM blocks, flags and
predictor histories. `bank_compat.rs` provides the common logical tone view,
source inspection, validated source-part manifests and a two-input transaction.
Unknown Hz and unresolved playback parameters are represented as unknown.
All imported Sony banks currently return a precise playback compatibility error:
reference-rate/tuning, native ADSR and staged gain/pan/channel policy have no
verified conversion to the initial portable playback profile. Reverb/layers and
predictive loops add source-specific requirements. This is **not** full Sony
instrument fidelity or a claim that these banks play in the D runtime.

The source package and IR remain useful independently of those target limits.
The initial 128-zone PSX payload profile was not widened to hide incompatibility.
No empty native bank, sample substitution, discarded layer, implicit PCM rate or
silent envelope approximation is emitted. No compatibility exporter is enabled.
Only PSX-only source waveforms are decoded by this adapter, with the additional
re-encoding quality warning; no future-console cookers were introduced.

Import settings stay schema 1 with optional versioned source descriptors, omitted
for existing MIDI/native banks. Compatibility packages use importer version 2;
version 1 still loads. Source manifests reconstruct original VH and VB exactly.
UUIDs, metadata/settings extras and descriptor/part extras survive reimport.
Both observed source hashes are checked before publication. Changing only VB
queues one grouped reimport. Snapshot updates work after both source files vanish;
pair relinking is explicit rather than guessed from filenames. Clearing the
companion field explicitly selects a complete VAB on reimport.

`assets.rs`, Manager, Inspector, preview, import dialog, project browser, CLI and
MCP share the new routing/validation. Extensions nominate candidates; structural
identity decides the parser, including wrong-extension tests. The dialog offers
Auto or explicit profile inspection and a required song selection. CLI exposes
`--inspect-audio-source`, `--sequence-profile`, `--song-id`, `--song-index`,
`--import-sound-bank` and `--vb`; MCP accepts the serialized sequence selection
and `vb_source` through the same transaction.

Checks already executed in production: 15 focused compatibility tests passed
(`audio-phase-e-compat-initial.log`); 12 VAB/ADPCM tests passed, including all
2048 tones and source-IR serialization/reload (`audio-phase-e-vab-initial.log`);
7 shared decoder/preview/selection tests passed (`audio-phase-e-shared-preview.log`);
one grouped-source watcher test passed (`audio-phase-e-pair-watcher.log`). Actual
ImGui Sony SEP selection/import passed (`audio-phase-e-ui.log`), and the two
existing MIDI ImGui/WinMM output tests passed (`audio-phase-e-midi-ui-winmm.log`).
These focused filters overlap and are not summed as unique test counts.

The read-only private-corpus preparation parsed 107 VAB banks, 2468 tones and
1431 samples; 61 banks contain layers, two contain unresolved sample-zero tones,
and six of eight repeating samples have predictive entries. No proprietary
corpus asset or game implementation was copied into the repository. Isolated
release host parse/decode/hash/source-copy time summed to 81.789 ms, median
671 us/bank; this is one observed run, not integrated editor latency or target
CPU cost. Peak host RSS and physical-console fidelity remain unmeasured.

Final production verification:

- `audio-phase-e-bank-final.log`: pair source checks, snapshot recovery, explicit
  paired-to-combined reimport and identity preservation passed.
- `audio-phase-e-cli-test.log` and `audio-phase-e-cli-1789256271765566900/`:
  real CLI catalog/import, wrong extensions, two-song selection, guarded converted
  reimport, complete VAB/VH+VB snapshots and an unbypassable build error for an
  unresolved Sony bank passed. Corrupt reimport retained the previous package.
- `audio-phase-e-sony-emulator-test.log` and
  `audio-phase-e-emulator-sony-1789256285758459500/`: Sony SEQ with the original
  portable triangle bank passed actual SPU capture, loops, stop/restart and
  disable. Sequence 88 bytes, bank payload 1472 bytes, SPU samples 1344 bytes,
  authoring packages 7097 bytes. Decoded peak 7927; maximum measured service
  755.858 us and gap 1986 us; no runtime or clock errors.
- The converted-sequence emulator run (original paths retained in the phase-E
  snapshot): the independent converted SEQ/SEP parser
  through the same runtime passed those checks. Sequence 88 bytes, same portable
  bank costs, authoring packages 7168 bytes; decoded peak 7937, maximum service
  755.858 us and gap 1986 us, no runtime or clock errors. Neither fixture uses
  proprietary game music or claims Sony bank instrument fidelity.
- `audio-phase-e-full-final.log`: 399 passed, 35 ignored, one unchanged GUI
  baseline failure, exit 1. The 35 include the three audio UI/WinMM tests already
  run explicitly and passed as reported above.
- `audio-phase-e-clippy-final.log`: exactly the 18 baseline diagnostics, exit 1;
  multiset comparison has zero delta (`audio-phase-e-clippy-comparison.json`).
  An earlier E run found an unused declared source limit; the parser now uses
  that limit explicitly. No warning suppression was added.
- `audio-phase-e-build-final.log`: `cargo build --bins` passed, exit 0.
- `audio-phase-e-legacy-spu-final.log` and `audio-phase-e-legacy-xa-final.log`:
  both final-executable integration suites passed, exit 0. SPU peak 12294,
  unchanged 24-voice behavior; XA stereo/mono, loop, priority, SFX coexistence,
  source moves, cache recovery, CD boot and stop/restart/retrigger all passed.
- A final independent read-only review found no concrete loss, hash/CAS bypass,
  wrong-song reimport or unsupported-playback bypass in the E paths.

The verified E gate covers complete source/IR preservation, strict separate
format profiles, common preview/runtime behavior for supported sequence events,
and explicit rejection of unresolved bank/sequence semantics. Faithful Sony bank
playback, native Sony ADSR/reverb/layers, predictive-loop repetition and physical
PSX validation remain **pending**, not silently accepted. The first usable
portable MIDI/SoundBank PSX milestone is complete in the emulator. The original
GUI and clippy baseline issues remain outside this audio change. No N64, PS2,
GameCube, compatibility exporter or later-roadmap work was started.

Final changed-file inventory, hashes and copies are retained in
`artifacts/audio-phase-e-snapshot/manifest.json`; earlier A/B/C/D snapshots remain
available for phase review. No commit, staging or destructive reset was made.

## Neutral source profile naming — 2026-09-13

The converted source adapter now uses `ConvertedSeqLe32V1`, displays
**Converted SEQ/SEP (LE32)** and serializes `converted-seq-le32-v1`. This is a
descriptive Epok profile name; neither an official name nor exclusivity of this
exact layout to one game has been established. Code, CLI examples, diagnostics,
tests and current documentation use the neutral name. Private corpus provenance
and historical test artifacts remain available in the ignored audit records.

The initial experimental metadata identifier remains only as a read alias and
in migration fixtures. Both metadata loading and CLI profile parsing use the
same alias definition. The next save/reimport writes the canonical identifier
while retaining source bytes, UUID, selected ordinal/hash and unknown metadata.
Reordered songs still require explicit reselection. The target cook key changes
with the implementation/settings; EPSQ/EPSB versions and playback commands do not.

Additional final verification, all exit 0:

- `cargo test compatibility -- --test-threads=1`: 8 passed, including migration,
  exact canonical metadata roundtrip, equal PSX payloads and selection guards;
  `artifacts/audio-profile-naming-tests.log`.
- `cargo test sequence_compat -- --test-threads=1`: 8 passed;
  `artifacts/audio-profile-naming-parser-tests.log`.
- The explicitly selected ignored Sony SEP ImGui interaction test: 1 passed;
  `artifacts/audio-profile-naming-ui-test.log`.
- `cargo build --bins`: passed; `artifacts/audio-profile-naming-build.log`.
- `python tests/integration/verify_audio_compatibility.py`: real CLI catalog,
  canonical profile import and old-package snapshot migration passed, preserving
  UUID/source/selection; guarded reimport and bank fidelity blockers still pass.
  Evidence: `artifacts/audio-profile-naming-cli-test.log` and
  `artifacts/audio-phase-e-cli-1789259231377852500/`.
- `python tests/integration/verify_psx_sequence.py converted`: PCSX-Redux passed
  sequenced SPU capture, IRQ clock, loops, stop/restart and disable. The owned
  single-note fixture uses an 88-byte sequence, 1472-byte bank, 1344 SPU sample
  bytes and 7166 package bytes; measured maximum service 755.858 us, gap 1986 us
  and captured peak 7927. Log: `artifacts/audio-profile-naming-emulator-test.log`.

These targeted checks supplement the phase-E full-suite and legacy SPU/XA
evidence above. The known GUI/clippy baseline failures and pending physical PSX
and Sony instrument fidelity checks are unchanged. The reviewed task contains
75 files; its final local copies/hashes are in
`artifacts/audio-profile-naming-snapshot/manifest.json`. The user subsequently
authorized committing and pushing this complete task to the existing branch.
