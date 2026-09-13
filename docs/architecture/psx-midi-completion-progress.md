# MIDI completion: implementation checkpoints

Started 2026-09-13 against `26cdef3810250c434550710802a6042f5a1d102e` on `develop`.
Execution follows [the completion plan](psx-midi-completion-plan.md); phase gates below
record evidence, not intended functionality. Physical PSX validation remains pending.

**Current delivery decision (2026-09-13):** after reviewing the audio, the user
explicitly requested deployment of the usable feature, basic verification and
push, with remaining extensive tests documented to control token expenditure.
This supersedes earlier requirements below to hold deployment until all P4/P5/P6
gates close. It does not mark unrun or failed tests as passed. See
[the scoped delivery record](psx-midi-delivery.md) for current behavior, evidence,
Ironwood migration and remaining work. Earlier checkpoint entries are historical.

## P0 — Baseline and acceptance contract

The pre-existing Console preview fix and the completion plan are preserved. Baseline
snapshots, command logs and the external song audit live under
`artifacts/midi-completion/p0/`. The independent instrument-library investigation is
preparation for P2 and does not enable an import or playback capability.

### Musical interpretation and migration decisions

- Keep an explicit `LegacyV1` interpretation for existing MIDI settings that omit
  the interpretation field. It retains the original supported subset and the
  existing explicit unsupported-event acknowledgment policy.
- Newly imported sources default to `MusicalV2`. Existing sources can explicitly
  select it without changing their UUID or source snapshot. Include interpretation
  in source/target preview and cooker identity. Sony compatibility profiles retain
  their separately verified interpretation and fidelity blockers.
- Parse and retain a bounded source-event ledger with track, absolute offset,
  wire status and original data. Merge by tick, track and source ordinal before
  interpreting channel state, including when different tracks share a channel.
- Resolve RPN selection/Data Entry on the host into neutral channel parameter
  events: bend sensitivity in cents, fine tuning as its centered 14-bit value,
  and coarse tuning as its centered 7-bit value. The original RPN operations remain
  in the source ledger. The runtime consumes resolved musical operations rather
  than a second independent RPN decoder. Parameter changes update active notes.
- Resolve bank selection at Program Change, retaining bank identity on the
  resolved program event. Selecting a bank alone does not retarget sounding notes.
  A bank without the requested mapping must fail before playback.
- Preserve source loop state and restore all musical parameters at the boundary.
  The default bend is 200 cents; integer bend displacement uses
  `(value - 8192) * range_cents / 8192`. Values 8832 and 9600 at range 1200
  correspond to +93.75 and +206.25 cents. These independent rational values are
  the acceptance oracle for the external song's pitch operations.
- Existing EPSQ v1 event streams retain their bytes. New parameter/bank operations
  require EPSQ v2 with an explicitly validated opcode contract; v1 is still read.
  No new opcode can be treated as a v1 no-op. EPSB changes follow the bank contract
  in P2/P3 and preserve v1 loading.

### Capability matrix to close incrementally

| Operation | Source/IR contract | Playback gate |
|---|---|---|
| Notes, tempo, program, volume, pan, expression, sustain, bends | Existing baseline plus verified normalization | P1 regression; P4 fidelity |
| RPN 0/1/2, null selection, data increments/decrements | Resolve channel state and retain original messages | P1 parameter oracle and runtime update |
| Bank MSB/LSB | Latch on Program Change; exact mapping identity | P1 identity; P2 nonzero-bank coverage |
| Effect-depth zero | Explicit dry/disabled state | P1; never infer support for nonzero depth |
| Modulation, pedals, reset, note/sound off | Typed musical controls with state/lifetime semantics | P1 interpretation; P4 instrument destinations |
| SF2 envelopes, layers, exclusive groups and modulators | Bounded source catalog; no first-zone or missing-generator fallback | P2 corpus audit; P3 cook; P4 fidelity |
| Sony bank/sequence semantics | Existing complete source records and hard blockers | P7 verified native fidelity |
| Other RPN/NRPN, SysEx, pressure without a declared destination | Preserve and diagnose exact operation | No advertised support without a tested implementation |

### Measurable acceptance limits

These are test requirements selected before runtime changes, not measured results:

- Song duration stays exactly 112,000,000 microseconds in the rational tempo model;
  event order and all 1508 Note On operations remain intact. Chunking the host
  clock must not alter the event trace or accumulate loop drift.
- Parameter arithmetic matches the independent cents oracle within 0.01 cent
  before device pitch quantization. PSX calibration tones must match the expected
  frequency within 2 cents plus the error of one SPU pitch-register step. Unexpected
  pitch clamps fail acceptance rather than silently widening this tolerance.
- For the actual Title workload, measured service maximum must be at most 2000 µs,
  maximum service gap at most 3000 µs, and note-on timing error at most 3100 µs.
  Measure at least two complete loops, including ordinary menu SFX. The elapsed
  musical clock must not drift by more than one service gap relative to the
  independent console clock over this interval.
- For the pre-existing deliberate exhaustion/geometry/XA stress, maximum service
  is at most 4000 µs and gap at most 5000 µs, with zero clock/capacity faults and
  all steals/denials counted. Baseline maxima were 2922.572/3024 µs. Stress is not
  the musical-fidelity acceptance case. Report average CPU as well as maxima.
- All linked and dynamic memory, stack/heap peaks and SPU reservations must fit
  their real regions. Unknown measurements cannot satisfy this requirement.
  Music alone and normal menu SFX must introduce no unintended note loss; an
  explicitly exhausted fixture has separate expected stealing behavior.

If a requirement cannot be met, retain the failed evidence, diagnose the workload
and revise the implementation before closing the phase. A proposed requirement
change must record its reason and measured effect rather than silently replacing
these limits.

P0 gate passed: source hash/metadata and the 280 diagnostics reproduced; the
independent 123-byte synthetic fixture and cents oracle are recorded in
`artifacts/midi-completion/p0/fixture-oracle.md`. Serial baseline checks:
`rtk cargo test --locked midi -- --test-threads=1` passed 13 tests (3 ignored),
and `rtk cargo test --locked audio_contract -- --test-threads=1` passed 5
(2 ignored), both exit 0. The RTK-filtered logs retain totals; source test names
are available in the unchanged baseline revision. No emulator/hardware run was
needed for this read-only baseline. P1 is enabled.

## P1 — Musical interpretation

Gate passed after integration review and targeted regression fixes. New SMF imports
use Musical v2; existing settings without `midi_profile` keep Legacy v1 on
round-trip. The import dialog and `--midi-interpretation` expose an explicit upgrade.
UUID, source snapshot, unknown fields and the legacy acknowledgment are preserved.
Musical v2 unsupported operations remain actionable errors even if a saved legacy
acknowledgment is true. The disabled UI option explains this prerequisite.

RPN 0/1/2, partial/raw Data Entry updates, cents carry, saturation, null selectors,
bank/program identity and channel controls are resolved after stable track merge.
The source ledger retains the wire data. Host analysis uses FIFO note-off queues
and a 2,000,000-operation limit for active-note/pedal passes. The shared kernel
updates active notes and restores tuning at loops, handles both pedals/reset and
channel-mode retirement, and bounds extreme pitch arithmetic before saturation.
EPSQ v2 adds opcodes 9/10 and extended controllers; the original subset retains
EPSQ v1 bytes. EPSB v1 still has bank 0 only and rejects other bank identities.

Integration review caught and fixed partial RPN-byte loss, stale sostenuto latches,
short-payload reads, a missing-braces pedal branch, fine-tune unit scaling and
pre-division pitch overflow. Their regression cases are retained. The first real
Console test exposed an unactionable v2 error message; the final test now verifies
the path, operation, track/tick and corrective action once per Play attempt.

Validation (commands run serially with raw output under `artifacts/midi-completion/`):

| Check | Result | Log |
|---|---|---|
| Isolated parser/IR harness | 12 passed | `p0/p1-harness-tests.raw.log` |
| `cargo test --locked midi -- --test-threads=1 --nocapture` | 19 passed, 3 ignored | `p1-midi-final.raw.log` |
| `cargo test --locked sequence -- --test-threads=1` | 31 passed, 2 ignored | `p1-sequence.raw.log` |
| `cargo test --locked audio_contract -- --test-threads=1` | 5 passed, 2 ignored | `p1-audio_contract.raw.log` |
| `cargo test --locked content_preview -- --test-threads=1` | 11 passed, 1 ignored | `p1-content_preview.raw.log` |
| `cargo test --locked midi -- --ignored --test-threads=1` | 3 passed: ImGui import, grid/list Console, real Windows output/stop | `p1-midi-gui-output-final.raw.log` |
| Native runtime runner | All 21 executables passed, including legacy SPU/XA and extended sequence cases | `p1-native.log` |

These filters overlap; the counts must not be summed as unique tests. Cargo was
invoked through `rtk proxy pwsh -NoProfile -Command` and raw redirection. Initial
failed Console output remains in `p1-midi-gui-output.raw.log`; final exit-code
files distinguish its corrected result. No emulator/hardware claim follows from
host runtime tests using emulated register storage.

The external MIDI still hashes to the P0 source and produces 1508 Note On events,
112,000,000 µs and logical peak 11. Musical v2 reports zero unsupported events;
Legacy v1 reproduces the original 280 (`p0/p1-opening-{musical,legacy}.json`).
The independent 440 Hz calibration waveform measures 464.483854 and 495.669499 Hz
for the song's +93.75/+206.25-cent bends; new-profile error across bend/fine/coarse/
reset cases is below 0.000124 cent. Legacy waveform error is below 0.00505 cent,
including its documented integer rounding. Native tests independently check a
zone's +50-cent SPU register setting within one register step and extreme clamps.

Host ABI sizes: kernel 7768 bytes (baseline 6488), four-instance pool 31232 bytes
(baseline 26112), physical-state array 1152 bytes and stats 64 bytes unchanged.
Linked PSX sizes, hardware CPU/heap/stack and the real song's bank/SPU/voice costs
remain pending. P1 does not provide missing instruments or declare Ironwood
playback ready. P2 is now enabled.

## P2 — Authoritative instrument library and source interpretation

Gate passed. SF2/SF3 import, Inspector catalog/coverage, CLI, MCP and the existing
import worker use one authoritative SoundBank snapshot, including large sources,
UUID, unknown metadata, source relinking and snapshot reimport. Embedded samples
are derived data, not separate AudioClips. New library banks use settings schema 2;
legacy portable banks remain schema 1. Empty new sequence mapping overrides are
omitted on serialization, preserving old settings; explicit overrides persist and
are validated. Playback remains explicitly gated until P3/P4 support the library.

The selected reference is FluidR3Mono GM 2.315 from Debian 2.315-7, 23,614,353
bytes, SHA-256 `cda013d8c370a48ae8dad271e761078d2e77455488dabdedbfbe5fc76a38c682`.
`resources/audio/fluidr3mono-2.315.json` records the version/source. Its complete
MIT notice is `LICENSE-fluidr3mono.txt` (SHA-256
`8fff5f4a88a956f806feef0c2dce196f0bb818ac719ef14162810deebd4700c8`), copied exactly
from the distribution notice. Import recognizes the complete source checksum and
preserves that notice in metadata. The descriptor does not install a binary.

The source model resolves instrument/preset global/local hierarchy, all matching
layers, effective default/override/additive modulators, key/velocity ranges,
pitch/root, envelopes, loops and percussion exclusion. Typed voice parameters use
neutral units, not PSX registers. Unknown required semantics remain blockers on
their affected regions; valid range clamps are reported and raw values preserved.
Review corrected defaults, sentinel ordering and hold versus decay key scaling.

Bounds are checked before allocations/expanded clones: 256 MiB source, 4096 RIFF
chunks, 2048 presets, 4096 instruments, 8192 samples, 65536 raw zones, 262144 raw
generator/modulator records, 65536 expanded regions and 1,000,000 expanded records.
Selection is cancelable and limited to 8,000,000 region matches. Decode has per
sample/aggregate PCM limits and cancellation, and validates Ogg CRC/BOS/sequence/EOS.

The real source has 197 presets, 1037 samples and 10738 regions. Its converter
uses half-open compressed byte ranges: all 1036 SHDR adjacencies and the final
SMPL endpoint agree. The FluidSynth wiki/+1 loader differs from that converter;
the independent oracle was regenerated without the extra next-stream byte.
All 52 slices consume complete CRC-valid Ogg pages. Their final granules define
PCM endpoints; six Symphonia terminal blocks contain 1953 excess frames, removed
at those endpoints without padding PCM or changing source loops. The previously
suspect clarinet/percussion loop endpoints fit this decoded source.

The hash-pinned Ironwood acceptance covers 1508 notes and all seven melodic
programs/seven percussion keys: 55 reachable regions, 52 samples and up to two
layers per note. Automatic selection does not fall back to program 0. The 52
decoded sources contain 2,976,759 frames / 11,907,036 float32 bytes, peak amplitude
0.9040142. Against the complete independent libsndfile oracle, frame counts match
and maximum absolute PCM difference is 6.25849e-7 (below one PCM16 LSB). Analysis
took 152.949 ms and decode 1075.007 ms in the final Debug acceptance run; these
exclude disk reads/hash verification and are host observations, not guarantees.
Full host peak working set, cooked main/SPU RAM and runtime voice/timing costs
remain unmeasured at this gate. The earlier FFmpeg memory estimate is provisional.

Validation through `rtk proxy pwsh -NoProfile -Command`, serial Cargo commands:

| Filter / check | Result | Artifact |
|---|---|---|
| `sf2` | 7 passed; malformed Hydra/RIFF, limits, layers, defaults, stereo/exclusion and sentinel cases | `p2-final-sf2.raw.log` |
| `soundfont` | 6 passed, 2 explicitly ignored external/UI cases at this checkpoint | `p2-final-soundfont.raw.log` |
| `soundfont -- --ignored` with source/oracle paths | 2 passed; actual ImGui import and all real source notes/PCM | `p2-final-gui-real.raw.log` |
| `audio_decode` | 3 passed; corrected synthetic Ogg sequence and corrupt CRC | `p2-verified-audio_decode.raw.log` |
| `audio_contract` | 5 passed, 2 ignored | `p2-verified-audio_contract.raw.log` |
| `assets::tests` | 9 passed, including staged payload and size contracts | `p2-verified-assets-tests.raw.log` |
| `sequence` | 31 passed, 2 ignored | `p2-verified-sequence.raw.log` |
| `instrument_` | 5 passed, 2 ignored | `p2-verified-instrument_.raw.log` |
| `asset_manager` | 2 passed | `p2-verified-asset_manager.raw.log` |
| `soundfont_mcp` | 1 passed; detected and explicitly configured library imports | `p2-mcp-import.raw.log` |

Filters overlap; counts are not unique totals. Every final exit code is 0.
Earlier compile/fixture failures remain in their original logs: Candidate Debug
assertion, stale-source fixture type and Ogg fixture page sequence were corrected.
The 33 MiB valid-library test also covers load, move, relink and reimport while
AudioClip limits stay unchanged. Original stereo FluidR3 corpus independently
parses as 189 presets, 1418 samples, 16912 regions, zero source-wide blockers.
Own per-instrument MIDI fixtures and stereo FluidSynth reference renders are under
`artifacts/midi-completion/reference/`; the full reference is 112 seconds plus
4.413243 seconds of release. No PSX emulator/hardware or subjective listening
equivalence is claimed by this source gate. P3 is enabled.

## P3 conversion checkpoint — 2026-09-13

The conversion/data gate is complete. Runtime synthesis, audition/final preset
acceptance and the user workflow are P4/P5; the source-library playback blocker
remains until that integration is verified. No Ironwood authoring file changed.

New host modules: `psx_library`, `psx_music_settings`, `psx_music_optimizer`,
`psx_library_wire`, `psx_library_asset`, `psx_loop_quality`, `spu_encoder`,
`instrument_dsp`, `instrument_modulation`; independent reader
`runtime/instrument_bank.hpp` and `tests/runtime/instrument_bank.cpp`.
`psx_sequence` bridges library derivatives and retains its original v1 emitter.

Recipes are versioned under `target_overrides.psx.music_conversion`. Missing
recipes default to Balanced; old portable SoundBanks and SFX still take their
original conversion path. Presets keep source rates below their maximum,
preserve layers/tempo/notes and do not change Role or Load Mode. Changing a
quality value requires Custom. Unknown target and nested optimization values
round-trip. Reachable is the default; FullLibrary preserves complete ranges
within the 128-region target bound and rejects absent/conflicting mappings.
Nonzero percussion-bank aliases require an explicit mapping, not a melodic
canonical-bank assumption. Reported layer peak is explicitly the song peak.

EPSB v2 uses a 48-byte header, 24-byte sample, 144-byte zone and 16-byte modulator
record, little endian and relative offsets. Caps are 128 samples/regions and 32
modulators per region; analysis payloads are bounded to 64 MiB. The independent
C++ reader checks table adjacency, ranges, envelopes, LFOs, every ADPCM block's
header/flags, initial/terminal silence, padding and loops before playback. V1
bytes are never reinterpreted as v2. UntilRelease retains PCM after the loop;
the hardware repeat-address behavior is still work for P4.

The reference library needs explicitly reported static filter baking at sustain
and a reference velocity. Animated LFO filters and sample-position modulation
are rejected. Dry removes base and modulated sends with an adaptation record.
No filter/control no-op is advertised as executable. Filters above source
Nyquist retain source-band PCM with a report; resonance Q above 64 remains an
error. These declared approximations require Source/Target audition acceptance,
not merely a successful binary export.

| Actual Ironwood conversion | Sample SPU bytes | EPSB/main-RAM copy | Debug cook |
|---|---:|---:|---:|
| Compact, maximum 11025 Hz | 556416 | 571904 | 2806.705 ms |
| Balanced, maximum 22050 Hz | 1109312 | 1124800 | 5190.171 ms |
| High, maximum 44100 Hz | 1778560 | 1794048 | 4772.622 ms |
| Proposed Custom, maximum 10208 Hz | 515520 | 531008 | See search report |

All have 1508 notes, 55 regions, 55 encoded variants from 52 sources and 385
retained modulation records. Metadata/table alignment contributes 15488 bytes
to the single immutable main-RAM EPSB copy. Sequence, kernel/pools, other assets,
linked total and peak heap/stack are separate costs, not included in that column.
Preparation took 1342.203 ms. During this Debug test process, an OS snapshot
reported peak working set 84488192 bytes; it includes test/runtime overhead and
is not a measured standalone cooker allocation maximum or PSX cost.

The read-only inventory of every resident Ironwood AudioClip is 4672 aligned
SPU bytes. The deterministic optimizer tested 13 fully cooked candidates in
34500.780 ms and proposed 10208 Hz: 515520 + 4672 = 520192 bytes, exactly the
post-capture budget. No spare SPU bytes are promised. This is a conservative
asset inventory, not the linked Title/build measurement; a P4 reverb reservation
or other bank changes the available budget and requires a new proposal. The
optimizer returns a proposal and never saves a recipe or changes residency.

Thorough encoding is optional and compares whole-sample SSE with Fast, selecting
Thorough only when smaller. Three source-rate sample audits reduced SSE by
approximately 21–35% at equal byte size, with longer cook times; details are in
`p3-encoder/audit.csv`. The legacy AudioClip encoder was not changed.

There are 48 sample loops. Alignment uses integer rational arithmetic and each
loop records original/aligned coordinates, post-ADPCM step, window RMS and
maximum slope. Explicit crossfade blends at most 256 pre-end frames toward the
PCM before the loop start without moving its period/endpoints or release tail.
It fails for a requested crossfade at start zero. It is not enabled by presets.
At 10208 Hz a 128-frame request improved many RMS comparisons but increased the
maximum boundary step from 0.257477 to 0.312500; the report preserves this result
and makes no universal quality-improvement claim. Audition decides its use.

Reduced bank identity includes source song/selection, authoring asset revisions,
library, effective recipe, profile and all implementation inputs. It uses a
derived UUID v8, never the shared library UUID. Staging inputs are namespaced per
song and source/dependency revisions are checked again after cooking. Cache
verification streams the source rather than duplicating the complete library.
One atomic cache envelope contains metadata and checksum; corrupt or semantically
invalid caches rebuild. Cache reads are capped during reading, not just stat.
Cancellation before cache commit aborts; an atomic completed commit wins a late
cancellation. P5 must discard late worker results and use authoring CAS on Apply.
The existing synchronous sequence-summary writes are not a transactional UI
Apply; `cook_cancelled` supplies the token path for the upcoming worker integration.

P3 validation (all commands prefixed `rtk`; filters overlap):

- `spu_encoder`: 7 passed; legacy bytes, loops/tails, Thorough SSE and cancellation.
- `instrument_modulation`: 5 passed; fixed/curved/pressure/controller sources,
  exact 8192 pitch denominator and bounded modulation state.
- `instrument_dsp`: 4 passed; resampling DC/aliasing and bounded filter behavior.
- `psx_loop_quality`: 4 passed; boundary analysis, prefix/tail preservation, invalid
  input and cancellation without partial PCM mutation.
- `psx_library_wire`: 4 passed; fixed layout and malformed/range failures.
- `psx_music_optimizer`: 4 passed; proposals, limits, disabled search and cancellation.
- `psx_library_asset_tests`: 3 passed, including same-checksum semantic corruption,
  source/UUID/unknown preservation, distinct song/recipe identity and failure/CAS.
- `library_full_selection`: passed, including free aliases, missing targets and
  canonical mapping conflicts versus explicit nonzero percussion-bank aliases.
- `psx_sequence`: 6 passed; resident budget/staging and legacy payload checks.
- `sequence`: 32 passed, 2 ignored; `audio_contract`: 5 passed, 2 ignored.
- Hash-pinned real conversion/optimization test explicitly invoked: passed; all
  presets and the fitting proposal serialized and were validated independently
  by the C++ reader. Its hand-built malformed/truncated fixtures also passed.

Logs: `artifacts/midi-completion/p3-*-verified.raw.log`, `p3-*-gate.raw.log`,
`p3-*-cache-final.raw.log`, `p3-full-selection-final.raw.log` and
`p3-native/`. Real recipe/cost/loop reports and payloads:
`p3-presets-verified/`. Early fixture compilation errors (shadowed helper,
tuple/array mismatch) remain in earlier logs and were corrected before final
checks. Physical PSX evidence, runtime musical fidelity and full-song listening
are not claimed by this conversion checkpoint. P4 is enabled.

## P4 — In progress (not a closed gate)

EPSB v2 now has a bounded integer parameter state shared by source/target host
audition and the PSX service. Matching layers start as an atomic logical group;
the 24-voice allocator applies aggregate and per-instance ceilings, priority,
exclusive classes and deferred key-on. UntilRelease samples retain their tail
and do not let a later ADPCM loop-start flag undo an early release. EPSB v1 and
the SFX/XA paths remain separate versioned contracts.

Source Preview consumes original decoded PCM and instrument definitions. Target
Preview consumes the encoded/decoded SPU samples and target definitions; both
report the comparison headroom and observed voices/clipping. The host uses linear
interpolation and does not simulate wet PSX reverb. Room has a global exclusive
lease, explicit binary per-voice sends, CC91 zero handling and a 9,920-byte SPU
reservation. Its single-DMA clearing buffer also costs 9,920 bytes of main BSS.
Mocked reservation/ownership/transfer failures pass. The owned-tone emulator
fixture demonstrated a nonzero Room work area and measured pitch (details below).
Host State size is now 752 bytes, not a console CPU timing claim.

The first complete original-library song audition observed 89 physical voices
(88 logical including release), with no source stealing/denial/clipping. The
independent FluidSynth 2.6.0 public API audit observed **91 physical voices** at
103.5015 seconds, chiefly releases. Thus preserving all these library tails does
not fit the PSX's 24 voices. The original target test observed 489 steals and is
not acceptance evidence. Scripts/reports: `reference/audit_fluid_voice_demand.py`,
`reference/fluid-voice-demand.json`, `p4-ironwood-preview/` under the artifact root.

An explicit optional `maximum_release_ms` target adaptation now limits the
full-scale volume-release duration, preserves the source and reports each changed
region. Named quality presets default to Preserve; the limiter requires Custom.
Nonzero release-time modulators currently block this adaptation rather than
silently being discarded. A full-song run at Custom 10,208 Hz / 500 ms observed
**21 physical voices / 20 logical, zero steals, zero denials and zero clipped
samples**. There were 36 release adaptations. Its sample budget remained 515,520
bytes, plus 4,672 reserved for the inventoried Ironwood SFX. This is a measured
conversion option, not a claim of identical original musical tails or a runtime
SFX reservation. Source/target renders took 11.566 s / 2.994 s on this host.
Evidence: `p4-ironwood-release500/`, `p4-release500.raw.log`. Full-song verification
must be repeated after the later resonance/parameter corrections and on PSX.

Review corrected modulation-envelope convex attack, tremolo polarity and the
filter's DC attenuation with positive resonance against SF2 generators 26, 13
and 9. Formula-based native tests passed; source-filter control signals use Q15
the neutral value 32768. Source/target processing remains distinct. The PSX build probe also
caught two integration errors before console execution: memory reporting treated
a derived bank UUID as an authoring asset, and the dependency observer omitted
library-specific cook inputs. Both paths now understand the versioned derivation;
a regression asserts observer/cooker identity equality.

Latest focused Rust `library_` run: 14 passed, 3 external tests ignored
(`p4-library-integration-tests.raw.log`). Focused native synth/source-preview/
service/reverb run passed (`p4-synthesis-reverb-regression.raw.log`). The new
owned-fixture emulator test is `tests/integration/verify_psx_library.py`; its
initial memory/identity failures are retained in the raw logs. P4 still requires
successful emulator timing, pitch/reverb capture, full-song/stress and legacy
regression evidence before P5 begins. Ironwood has not yet been edited.

### P4 runtime profiling and preparation (still an open gate)

The immutable library stream now prepares initial voice states and exact initial
SPU pitch on the main thread. The actual song needs 97 distinct states and 1,676
layer references. The event map removes IRQ library searches. A deterministic
target-only compaction removes 120 redundant assignments, preserving original IR,
explicit zero reverb, notes, reset commands, ordering and loop state. EPSQ is
36,928 bytes; EPSB remains 531,008 bytes, including 515,520 SPU sample bytes.
The linked preparation cache costs 90,896 main-RAM bytes in the first full-song
build. That isolated build uses 1,627,488 / 2,097,152 main-RAM bytes; it is not an
Ironwood scene-memory claim, and heap/stack peaks still require evidence.

Allocation reuses a per-service view of free voices, supports complete-group
exclusion in its fast path and retains the shared priority allocator on overflow.
Per-note physical masks avoid repeated full-pool scans. Immutable initial synth
parameters are copied lazily; aligned word copies avoid the SDK's bytewise copy
helpers. Envelope updates return state by reference, preserve exact reciprocal
correction, and avoid zero-time release work and unchanged pitch reconstruction.
Library key-ons can be sent in the same IRQ after at least 128 Timer-2 ticks
(over 30 microseconds) since hardware key-off. Legacy keeps its deferred path.
Note admission retains an absolute deadline; a fixed six-start slicing experiment
was rejected because it delayed notes by over 6 ms. No gate was widened.

A regression exposed retirement of a younger repeated key before an older held
key. Per-key issued/consumed ordinals now preserve FIFO lifetime even when sample
durations differ. Pending-key masks bound loop cleanup; a 70,000-loop test covers
serial wrap. Host Kernel is 12,208 bytes; four instances use 51,040 bytes, physical
states 19,584 bytes, and legacy statistics 64 bytes. Additional timing statistics
record actual key-on lateness and cumulative service ticks without changing that
legacy statistics layout. Runtime paths still perform no allocation.

Evidence under `artifacts/midi-completion/`:

- `p4-lifetimes-all-native.raw.log`: all 28 native executables passed.
- `p4-pitch-serial-regression.raw.log`: independent initial pitch calculation and
  70,000-loop FIFO wrap passed. The younger-retirement failure is retained in
  `p4-younger-retirement-reproduction.raw.log`.
- `p4-ironwood-final-source-target/report.json`: complete host Source/Target after
  resonance fixes; source 89 physical / 88 logical, target 21 / 20, no denials,
  steals or clipping. Host renders took 14.851 s / 3.557 s. Subsequent runtime-only
  optimizations and the FIFO correction still require final host regression.
- Owned tone `p4-psx-library-1789295582482256600`: actual SPU pitch 495.452 Hz,
  -0.7599 cents against the independent oracle, nonzero Room memory; CPU maximum
  1,964.994 microseconds and gap 2,000 microseconds. Repeat against final headers.
- `p4-runtime-1789301496943170100/evidence.json`: 12-second real-song excerpt passed
  with 1,842.167 microseconds maximum service, 2,003 maximum gap and 2,517 maximum
  key-on delay; zero steals, denied notes, clamps or clock faults. This is an
  excerpt, not two-loop acceptance.
- The first full-song run `p4-psx-song-1789301683209467300` has so far observed
  2,217.734 microseconds maximum service, 3,051 gap and 3,028 key-on delay,
  23 physical voices and zero musical loss. Average service consumed roughly
  52 percent of the measured guest CPU interval. Its CPU gate fails; profiling
  continues. No P4 closure, P5 UI, Ironwood integration or hardware validation is
  claimed by this update.

### P4 final timing work — 450 ms candidate (gate still open)

The complete 500 ms recipe reached 24 physical voices in the real backend, leaving
no margin for menu SFX. The explicit candidate is now Custom, 10,208 Hz, Dry,
maximum full-scale release 450 ms, music ceiling 21, and 4,672 additional resident
SPU bytes. The original MIDI/library and Source Preview remain unchanged. This is
an explicit envelope adaptation, not a named preset or an automatic optimizer
change. Samples still occupy 515,520 SPU bytes; 36 zones report a release cap.

The first 450 ms / 21-voice two-loop run had no steals, denials, clamps or clock
faults, but failed the CPU gate at 2,158.919 us (gap 3,000 us, key-on delay 2,895 us).
Evidence: `p4-psx-song-1789303927154388100` and
`p4-psx-song-release450.raw.log`. Intrusive profiling then located the maximum at
the loop boundary, rather than the dense climax. The ordinary climax was below
2 ms. Failed experiments remain evidence; their timings are not acceptance.

Current implementation adds indexed per-key FIFO links, fixed aligned channel
snapshots, direct note initialization, reuse of the deadline for same-tick events,
and cached reverb sends / direct software-envelope gain at hardware key-on. The
library cooker may capture a Whole-loop snapshot after its initial zero-tick
setup prefix, before any note; no musical event changes order or disappears.
Marker loops and legacy payloads retain their existing paths. An independent
native fixture compares deadlines, bends, pedals and channel state over ten loops.
Target Preview uses the same bounded sequence payload builder as the library cook.

Current host sizes: State 728 bytes, Kernel 16,384 bytes, four service instances
67,744 bytes, physical pool 19,200 bytes. These supersede the earlier intermediate
sizes above; linked MIPS sizes and dynamic peaks are recorded separately.

- `p4-current-all-native.raw.log`: all 28 native executables passed before the last
  deadline/key-on optimizations. Focused regressions after those changes passed in
  `p4-deadline-native.raw.log` and `p4-start-registers-native.raw.log`.
- `p4-final-rust-sequence.raw.log`: 33 passed, 2 ignored. The later loop-prefix
  focused suite passed 8 tests in `p4-loop-setup-rust.raw.log`.
- `p4-ironwood-release450/report.json`: complete Source / Target renders had
  physical peaks 89 / 21, logical peaks 88 / 20, and zero steals, denials or
  clipping. Rendering took 16.468 / 4.289 seconds on this host.
- `p4-loop-start-registers.raw.log`: a diagnostic seek near the end of an isolated
  build reproduced the loop in six seconds and passed at 1,920.824 us service,
  2,003 us gap and 2,765 us key-on delay. It is not full-song acceptance.
- `verify_psx_music_observer.py` independently observes guest CPU cycles, executable
  stack adjustments and the SDK heap high-water mark. Its five-second smoke test
  passed with 509.064 us maximum clock drift, 33,256 bytes of executable stack and
  a 16-byte heap extent. Full-loop measurement remains required. BIOS-private
  stacks are explicitly outside that observation; heap extent includes overhead.

The clean two-loop song test, full independent observation, final SPU tone and
legacy/library stress regressions are required before closing P4. No P5 UI,
Ironwood edits, physical-hardware validation or push is claimed here.
