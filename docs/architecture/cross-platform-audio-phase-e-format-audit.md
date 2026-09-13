# Phase E preparation: Sony and converted SEQ/SEP audio source formats

Status: **historical format investigation and isolated parser preparation**,
2026-09-13. This evidence was collected while D was being implemented; the
preparation itself did not change production or pass a gate. Subsequent production
integration, verified subsets and remaining fidelity limits are recorded in
[the implementation ledger](cross-platform-audio-progress.md) and
[the user guide](../assets.md#sony-and-converted-sequence-source-imports).

The user authorized a subagent to advance independent E work. The parent limited
this work to this new report and ignored `artifacts/audio-phase-e-preparation/`.
The RTK instructions, three architecture documents and A–C ledger were read.
The dirty working tree was inspected before writing; its changes belong to the
ongoing A–D implementation. No existing tracked file was edited by this subtask.

## Evidence and ownership

The following primary evidence was inspected:

- Sony, [File Formats](https://psx.arthus.net/sdk/Psy-Q/DOCS/Devrefs/Filefrmt.pdf),
  November 1998, physical PDF pages 210–214, printed pages 4-3 through 4-7:
  sequence/container diagrams, bank tables and sample-size units. Retrieved with
  Python after web extraction timed out; 864,963 bytes, SHA-256
  `0f1a35952cc93405a22ded78cd19ef270b09f3accc35f4a6bfadd32e4a9b17a7`.
- Sony, [Run-Time Library Reference](https://psx.arthus.net/sdk/Psy-Q/DOCS/Devrefs/Libref.pdf),
  physical PDF pages 856–857, printed pages 14-10 and 14-11: `VabHdr` and `VagAtr`
  field definitions. The downloaded PDF was read in memory, not redistributed.
- [PSX-SPX original format research](https://psx-spx.consoledev.net/cdromfileformats/#cdrom-file-audio-sequences-seqsep-sony)
  was used for byte ordering and independently cross-checked against the local
  corpus. Its tentative description of VAB program count as count-minus-one is
  contradicted by both Sony's `VabHdr` description and all 107 local headers.
- [loveemu's original SEQ research](https://loveemu.hatenablog.com/entry/20060630/PSX_SEQ_Format)
  supplies the score-specific meta/running-status distinctions and Sony NRPN
  interpretation. No converter source code was copied.
- Original local format evidence in
  `<private-corpus>/game/sound/sound.txt:24`,
  `game/prog/SOUNDDEF.H:351`, and `game/prog/sound.c:3812`, `:1919`, `:3307`:
  converter workflow, converted header layout, record traversal, score startup
  and event consumption. These files were read only. Their source notices name
  their original publisher and reserve rights; they are evidence, not code available for reuse.

No game was launched, no third-party audio was played, and no game source or
samples were copied into Epok. The adjacent repository remains read-only. Its
generated audio documentation was treated as a lead, not as an oracle: the loop
values listed there conflict with original constants and actual score bytes.

The local corpus is **private verification input with unverified redistribution
rights**. Paths, sizes, hashes, numeric format metadata and diagnostics are
recorded under ignored artifacts. Original synthetic fixtures and parser code
written for this task are suitable for repository tests under Epok's license.
Compatibility support does not imply permission to bundle a game's instruments.

## Reproducible preparation artifacts

All paths below are relative to `D:/GitProjects/GameEngines/Epok/EpokEngine`.

| Artifact | Purpose |
| --- | --- |
| `artifacts/audio-phase-e-preparation/audit_formats.py` | Original bounded structural audit and 17 synthetic assertions; no decoder/player |
| `artifacts/audio-phase-e-preparation/corpus-report.json` | Hashes and numeric analysis of 163 local sources; two strict-profile bank errors retained |
| `artifacts/audio-phase-e-preparation/sequence_compat.rs` | Prepared Sony/converted SEQ envelope and event adapter to the existing `SequenceIr`; isolated from production |
| `artifacts/audio-phase-e-preparation/sequence-compat-review.patch` | Reviewable addition of that module to `src/sequence_compat.rs`; **not applied** |
| `artifacts/audio-phase-e-preparation/Cargo.toml`, `Cargo.lock`, `harness.rs` | Separate offline host harness using actual `src/sequence_ir.rs`; separate target directory |
| `artifacts/audio-phase-e-preparation/rust-test.log`, `rust-build.log`, `rust-clippy.log`, `rust-corpus.log` | Complete command results / exits and per-song parser outcomes |

The patch adds a module and its tests only. It deliberately does not edit
`main.rs`, asset package dispatch, scanners, import settings, UI, audio staging
or runtime files. The main agent must review and integrate it after gate D.

## Identity and envelope rules

Extensions are hints for discovery only. A raw VB has no identifying header and
must be paired explicitly with its VH; it cannot be identified safely by bytes
or a guessed neighboring filename. Unknown revisions stay unsupported until
separately specified and fixture-tested.

| Proposed source profile | Structural identity | Evidence coverage |
| --- | --- | --- |
| `sony-vab-v7` | `70 42 41 56` (`pBAV`), LE u32 version 7; complete validated tables and body | 107 headers; 105 pass the strict tone-reference profile |
| `sony-vh-v7-vb` | Same validated header, exact VH boundary, explicitly supplied VB with matching lengths | Synthetic split equivalence; no real standalone pair in this corpus |
| `sony-seq-v1` | `70 51 45 53` (`pQES`), BE u32 version 1, valid score to EOT | Sony documentation and original synthetic fixtures; no real Sony SEQ found locally |
| `sony-sep-v0` | `pQES`, BE u16 version 0, bounded independent entry records | Sony documentation and original synthetic fixtures; no real Sony SEP found locally |
| `converted-seq-le32-v1` | No magic; complete chain of validated LE size/timing records and post-event-delta scores | 56 files / 88 songs parse completely |

The first eight bytes of a Sony SEQ v1 also match a Sony SEP v0 whose first song
ID is 1. Therefore `pQES 00 00 00 01` alone cannot disambiguate them. The prepared
detector validates both structures and reports ambiguity if both succeed. A
manual source-profile selection still validates the selected complete grammar.
Converted SEQ/SEP structural detection is a candidate classification, not a cryptographic
signature; it must never publish an asset simply because a file looks plausible.

### Sony SEQ and SEP fields

The offset tables below describe the prepared byte reader. Multi-byte values
are read explicitly; no native struct casts or host endianness are used.

| Sony SEQ offset | Bytes | Meaning |
| --- | ---: | --- |
| 0 | 4 | `pQES` |
| 4 | 4 | BE revision 1 |
| 8 | 2 | BE PPQN |
| 10 | 3 | BE microseconds per quarter note |
| 13 | 1 | Time-signature numerator |
| 14 | 1 | Denominator exponent |
| 15 | variable | Initial delta, then score; final EOT required |

Sony SEP begins with six bytes (`pQES`, BE u16 zero). Every following entry has
its own song ID and timing; entries are **independent songs**, not simultaneous
tracks to merge as SMF type 1.

| Sony SEP entry-relative offset | Bytes | Meaning |
| --- | ---: | --- |
| 0 | 2 | BE song ID; duplicate IDs rejected by the prepared profile |
| 2 | 2 | BE PPQN |
| 4 | 3 | BE initial tempo |
| 7 | 2 | Numerator, denominator exponent |
| 9 | 4 | BE score-byte count including EOT |
| 13 | declared length | Score; must consume that slice exactly |

No alignment bytes are assumed between Sony entries. Checked addition precedes
every slice. Empty containers, truncated fields, nonzero/unknown versions,
zero tempo/PPQN, SMPTE-bit PPQN and invalid rhythm fields fail with offsets.

### Converted SEQ/SEP (LE32) records

`converted-seq-le32-v1` is an Epok descriptive profile name. Sony's documented
SEQ/SEP family is shared across games; the exact converted layout below has only
been verified against this private corpus. Neither exclusivity nor an official
name for this variant has been established. The workflow's SEQCOMB/SEPCOMB tool
names are provenance observations, not evidence that these are Sony SDK tools.
`<private-corpus>` below refers to the external root recorded in the ignored
`artifacts/audio-phase-e-preparation/corpus-report.json`.

Every song has a 12-byte header: LE u32 record size at 0, LE u32 tempo at 4,
LE u16 PPQN at 8, numerator at 10, denominator exponent at 11. The size includes
the header, score and zero padding to four bytes. The next record starts exactly
at the previous record's end. There is no outer magic, song count, ID table or
Sony-style score-size field. IDs are source record ordinals.

The first event starts immediately at byte 12 with an explicit status and an
implicit initial tick zero. Subsequent delta VLQs follow each event; EOT has no
following delta. All 88 local records satisfy full-score consumption plus 0–3
zero alignment bytes. `o_clr0.seq` uses this same converted format despite its
`.seq` extension. The original workflow describes SEQCOMB conversion followed
by SEPCOMB concatenation; neither executable is needed by the prepared parser.

## Score grammar and semantic boundaries

The prepared reader enforces 4 MiB per container, 256 independent songs, 65,536
source events per container, 65,536 lowered events per song and 4,096 diagnostics
per song. VLQs consume at most four bytes; ticks use checked u32 accumulation.
All channel parameters are seven-bit. Original score order is retained in
`Event.order`, with header tempo/rhythm preceding score events at tick zero.

Supported event lowering covers note on/off (including velocity-zero note-on),
program, volume/pan/expression/sustain controls, bend, tempo and EOT. `FF 51`
is followed directly by three tempo bytes: it has **no SMF length byte**. EOT
is `FF 2F 00`. Running status can include `FF` in these score formats. The
prepared reader tests repeated tempo/EOT under that status; all 15 real corpus
tempo events happen to be followed by an explicit channel status instead.

Aftertouch and known-length unsupported channel controllers produce retained
diagnostics with offsets and original numeric values. Unknown system/meta
messages fail explicitly because blindly using SMF's length grammar would lose
synchronization. No mark callback can enter gameplay from imported data.
Bank changes, dynamic VAB mutations, reverb commands, RPN/NRPN and finite repeat
counts are not silently interpreted as supported Epok operations.

### Loop facts versus playback policy

Sony and converted SEQ/SEP both use **20 / 30 decimal = 0x14 / 0x1e**, carried by CC99, for
loop start/end. This is the same numeric value written in different bases.
The local generated description's `0x60` is not supported by the original
constants or corpus. All 77 observed looped songs use CC6=127, CC99=20 and
CC99=30; count 127 represents the infinite-loop case. In 76 songs these are
on one channel. `o_me1d.sep` starts on channel 8 and ends on channel 9 (one-based).
The prepared strict adapter diagnoses that cross-channel pattern rather than
guessing compatibility with its initial single-channel loop rule.

For the converted profile, the original runtime records the byte pointer after reading the loop
control's following delta. The first repeated event is 1 tick after the start
control in 74 songs, at the same tick in 2, and 120 ticks later in 1. Using the
control tick as the loop boundary would add a tick to most repeated periods.
The prepared adapter records both ticks, source offsets, count and channel in
`SourceLoop`, and places a converted region at the first repeated event's tick.

This does **not** prove equivalence of libSnd/converted-sequence looping to Epok's controller
snapshots and tail cutting. Every lowered compatibility loop carries an
unsupported/acknowledgement diagnostic explaining that difference. A finite
count is retained as intent and diagnosed, without being converted to infinite
playback. Nested, partial, multiple or malformed loop controls also remain
diagnosed. Sony's exact first-delta replay semantics still need a libSnd trace.

Two real songs (`o_me1d.sep`, `o_mt01.sep`) have events after their end-loop
control. These provide mandatory future fixtures for outro/loop-disabled
behavior. Ordinary MIDI loop behavior is not changed by this preparation.

## Sony VAB v7 facts established by the corpus

All 107 bank headers contain `pBAV`, revision 7 and a total byte size equal to
their actual file length. Header programs and tones are **counts**, not last
indices. Program count equals the number of nonempty program rows; total tone
count equals the sum of their active tone counts. The sample-size table plus
header accounts for the complete file in every bank.

| VAB offset | Bytes | Meaning used by the audit |
| --- | ---: | --- |
| 0 | 4 | `pBAV` |
| 4, 8, 12 | 4 each | LE version, source bank ID, total VH+VB bytes |
| 16 | 2 | Reserved, retained without requiring a fixed value |
| 18, 20, 22 | 2 each | LE program, tone, sample counts |
| 24, 25 | 1 each | Master gain and pan |
| 26, 27, 28 | 1, 1, 4 | Source attributes and reserved data |
| 32 | 128 × 16 | Fixed program table |
| 0x820 | program_count × 512 | Compact active-program tone rows, 16 slots × 32 bytes |
| 0x820 + program_count × 512 | 512 | 256 LE u16 sample sizes, multiplied by 8 |
| 0xa20 + program_count × 512 | remainder | Raw SPU-ADPCM samples, without VAG headers |

In this corpus sample table slot 0 is zero, samples occupy slots 1 through the
header's sample count, and later entries are zero. Tone sample references are
1-based. A universal zero-based interpretation would address the wrong sample.
Treat other table-occupancy conventions as separately unverified variants.

A program row contains tone count at 0, gain at 1, priority at 2, mode at 3,
pan at 4, reserved byte at 5, LE attributes at 6 and reserved bytes through 15.
Its fixed table index is the MIDI program number; the corresponding tone block
is indexed by compact rank among active programs, not by that sparse number.

A tone row contains priority/mode/gain/pan at 0–3, center note/fine correction
at 4–5, inclusive key limits at 6–7, vibrato at 8–9, portamento at 10–11, bend
limits at 12–13, reserved bytes at 14–15, LE ADSR words at 16 and 18, LE owner
program/sample IDs at 20 and 22, then reserved bytes. Reserved/unknown values
must survive in original source and compatibility metadata.

The stricter Python tone-reference profile accepts 105/107 banks. It reports
`M1c.VAB` at 0x20b4 and `M2a.VAB` at 0x1d54: their active tone references sample
ID 0. This is preserved as an unresolved source convention, not repaired by
using sample 1. The sizes/header analyses themselves still agree for both.

Among the 105 accepted banks: 2,405 tones, 1,401 sample entries and 13,982,416
encoded sample bytes were enumerated. These are corpus totals, **not one game's
resident budget**. Banks range from 8,912 to 407,296 total bytes. No accepted
sample has a filter above 4 or shift above 12, and all sample lengths are
multiples of 16. This audit does not decode PCM or measure sound quality.

### Data the current portable bank must not silently flatten

- **Layering:** 61/105 accepted banks contain overlapping tone key ranges.
  They need multiple physical tones per note or an explicit rejection. C's
  single-zone-match policy cannot represent them by simply selecting the first.
- **Reverb:** 1,208 tones have mode 4; 1,197 have mode 0. Preserve the source mode
  and diagnose unsupported effects. Dry playback is an explicit approximation.
- **Bend:** 2,396 tones use limits (0,0), six (12,0), two (2,0), one (12,12).
  Substituting the current fixed ±2 policy is not faithful source import.
- **Fine correction:** observed values include 100. The Sony reference identifies
  the field as cents, but its sign in the center-note pitch formula and original
  sample-frequency convention must be verified before final bank lowering.
  Positive root correction is not automatically positive playback detune.
- **ADSR:** raw SPU envelope words encode shapes/rates that do not exactly equal
  the current host linear attack/decay/release model. Preserve both raw source
  parameters and any documented neutral approximation; report the difference.
- **Channel 10:** Sony scores are not inherently General MIDI percussion. The
  portable kernel currently requests explicit drum mappings on that channel;
  imported bank mappings must be explicit and verified, never an OS/GM fallback.
- **Loops:** source sample block flags and exact repeat/start/end positions must
  be decoded. Most corpus samples have stop/terminal blocks; eight have repeating
  end flags. Treat all samples as looping only because they are instruments would
  be incorrect. Preserve full compressed source even when removing terminal
  padding from decoded PCM.

## Source packages, lowering and future integration

The prepared sequence adapter returns one `SequenceIr` per independent song.
A multi-song import requires an explicit selection/import-all transaction;
preview must identify the selected song. Retain the full original container,
profile, original song ID/ordinal, offset and source hash. Reimport selects the
same identity and fails if that member disappeared; it must not silently move
an existing asset UUID to a different song.

For VAB/VH+VB, retain original component bytes and their individual hashes in a
versioned source bundle. The current SoundBank source validator accepts a
mapping document, so compatibility dispatch/source-bundle validation is an
actual integration task. A missing VB must fail before publishing anything.
Pair selection is persisted; raw absolute paths are not a portable dependency.

Decode original ADPCM into a neutral PCM source only when no original PCM master
is available, retaining a lossy-origin warning. The resulting sample child
assets need stable UUIDs and explicit ownership/provenance; avoid content hashes
as persistent identity. Bank reimport must recertify all child publications and
preserve the previous valid packages on failure. Existing A–C imports must keep
their UUIDs, unknown-field round trips and original-source snapshots.

Program/tone mapping, ranges, gain, pan, root correction and loops can then
lower to portable `sound_bank::Settings` plus explicit compatibility records.
Layering, envelope shapes, per-tone bend and unsupported effects require either
additional verified IR semantics or an actionable unsupported profile result.
The prepared parser does not pretend those bank/runtime features exist.

Cook identity must include source profile/revision, selected container member,
all source components, sample dependencies, mapping policy, approximation
acknowledgements, decoder version and PSX profile. The production D cooker must
consume the same neutral IR as MIDI. No libSnd dependency, proprietary converter
execution or Sony-format runtime payload is introduced by E.

## Executed validation and remaining gates

Commands were run with the required `rtk proxy` prefix. The separate Cargo
harness uses its own manifest, lock and target directory; it does not build or
modify Epok's production binaries.

- Python structural/synthetic audit: 17 assertions pass; 163 files enumerated,
  56 converted sources fully consumed, 105 VAB strict-profile successes and the two
  sample-ID-zero diagnostics retained. Script exit 0 means the audit completed,
  **not** that every real source is supported.
- Isolated Rust tests: **8 passed, 0 failed, 0 ignored**. Tests cover equal
  Sony/converted SEQ note semantics, running FF status/tempo, independent SEP songs,
  exact bounds/IDs, converted first-repeated-event timing, finite-loop diagnosis,
  unsupported messages, malformed headers/VLQs/padding, event/tick capacities
  and mutation/truncation panic resistance.
- Offline locked Rust harness build: passed. Clippy `--all-targets -- -D warnings`:
  passed. The first clippy run found a manual multiple-of check; it was corrected
  before the final build/tests. No warning suppression was added.
- Rust adapter against all 56 real converted sources: **88 songs, 191,424 lowered
  events, 0 parse failures**. Original source event count was 191,327; the lowered
  count also includes header tempo/rhythm and recognized loop markers, while
  removing recognized loop-control messages from the playable event list.
  Compatibility loops remain explicitly diagnosed, so this is parsing/IR
  evidence, not a blanket fidelity or playback claim.

The Rust preparation recognizes 76 region loops. `o_me1d.sep` retains three
unsupported controller diagnostics for its cross-channel loop. Its notes,
tempo, programs and ordinary controllers still parse, but faithful looping is
not implemented by this strict preparation profile.

No host audition, PSX executable, emulator, physical hardware or new resource
budget was exercised by this subtask. Decoder PCM costs, PSX voices/RAM, musical
timing, layering and envelope fidelity remain unmeasured here. The separate
phase-D agent owns runtime and emulator qualification.

Before E completion: gate D must pass; source-package/UI transactions and source
selection must be integrated; VAB decoding/mapping must be implemented with the
listed semantics preserved or diagnosed; Sony conformance needs independent
real/libSnd-produced fixtures or equivalent verified traces; compatibility
preview/reimport/cache/lifecycle tests must pass. The patch is a reviewable
starting point and does not discharge those obligations.

Representative private fixtures (full corpus hashes are in the JSON):

| Relative to `<private-corpus>/game/sound/` | Bytes | SHA-256 |
| --- | ---: | --- |
| `sep/o_c0.sep` | 2,328 | `3eccd7a197e50828864637609014f8eefd5f08cc6b08bd00e42c00ffb45cffb3` |
| `sep/o_g2.sep` | 7,876 | `9449feb43cd429cb2c6443f7d02afa1c6147ea7065081cbccbe941274199bad2` |
| `sep/o_clr0.seq` | 3,820 | `590bfd96c43d5433f54381d428e895156907429f39e77e3eb058c2691c976394` |
| `vab/clr0.VAB` | 152,448 | `2c59c19112e82f48ccbb3fa1fd6bde65ff0cbdcf55569c49756aaf5d8e41125c` |
| `vab/M1c.VAB` | 233,648 | `a9a2926d370eadb6146b8019d6812ce8ea63d9c5479ea1a87c7a839a4afc3eed` |
| `vab/M2a.VAB` | 255,648 | `234283420e1acdb0e3d24deac677c2a6c0bdeaa9e01b95a7ced194ee61846968` |
