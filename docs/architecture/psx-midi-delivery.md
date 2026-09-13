# PSX MIDI usable delivery — 2026-09-13

## Scope and authorization

After listening to the result, the user requested deployment and push with the
remaining extensive tests documented, to reduce time and token expenditure.
This delivery does not claim 100% completion of the original P0–P7 plan. No new
subagents were used for the deployment work.

**Delivery limit:** the engine builds and the real Title scene plays MIDI in an
isolated project. Ironwood with Title plus the two Forest scenes currently fails
the PSX link budget by **878,264 bytes (857.68 KiB)** using its saved serial/host
Play profile. The ordinary build also fails, by 875,896 bytes. Reducing the complete
game's main RAM use is deferred; this is not a successful full-game deployment.

## Observable behavior

- MIDI MusicSequence can use a complete SF2/SF3 SoundBank, including the bundled
  MIT-licensed FluidR3Mono GM 2.315 reference library. Required melodic/drum regions
  and layers are selected per song without destructively pruning the source bank.
- Musical v2 interprets RPN bend range/fine/coarse tuning, bank selection and
  supported channel controllers. Unsupported meaningful operations remain errors.
  The 280 old diagnostics in `opening_02` no longer block its supported operations.
- PSX conversion has quality presets and editable parameters with delayed help,
  asynchronous cancellable analysis and a bounded rate optimizer whose proposals
  require adoption. Saving preserves the sequence UUID and source snapshot.
- Source and PSX Target audition share the existing single-preview controls and
  error-to-Console behavior. Target uses the actual cooked EPSQ/EPSB payloads.
- PSX playback uses sequenced SPU voices; existing AudioSource autoplay, gain,
  stop, priority and SFX coexistence retain their interfaces. XA stays available.
- Role and Load Mode remain independent. Neutral IR and target-specific recipes
  remain the architecture; no N64, PS2 or GameCube playback support was added.

## Ironwood migration

Local project: `../EpokDemos/Ironwood` (not a Git repository, not part of the engine push).

- Original MIDI SHA-256 remains
  `5709b0b1b24d93b7c83199b168d573f1e915add144b340633502da2e4b299efe`.
- `assets/sounds/bgm/opening_02.epokasset` retains UUID
  `84ad8004-1cd4-44ac-8acb-3ccd06cd84f2`; snapshot reimport explicitly selects
  MusicalV2, Music, Resident, Whole and 21 physical music voices.
- Added `assets/AudioLibraries/FluidR3Mono_GM.sf3`, its `.epokasset`, MIT notice and
  `opening_02.psx-music.json`. SoundBank UUID is
  `5d07793f-7c7a-4038-8402-7be2adacf19b` and also becomes the project default.
- Recipe: Custom, 10,208 Hz maximum, 450 ms maximum full-scale release, Dry,
  default 6 dB headroom, 4,672 other resident sample bytes. 36 regions have a
  reported release adaptation. Source Preview retains the authored tails.
- Title Music entity `15430168-5b56-57a4-973e-ae0079f54b36` keeps its name, ID,
  volume 0.7 and autoplay; its clip reference changes from Field1 to opening_02.
- Play's initial scene becomes Title, added alongside the existing selected
  Forest scenes. The serial target and host data preference remain unchanged.
- Pre-edit manifest, Title, sequence asset and MIDI backups plus hashes are in
  `artifacts/midi-completion/deployment-backup/`. The original Field1 asset is intact.
- Existing MIDI with no interpretation field remains LegacyV1. Other projects
  are not automatically migrated. EPSQ v1 / EPSB v1 remain readable; new musical
  operations and library banks use validated v2 contracts.
- Audio diagnostic counters are now `epok::music_sequence_stats`; Timeline keeps
  its existing `epok::sequence_stats`. This fixes their coexistence in Ironwood.

## Completed evidence

Evidence paths below are local ignored artifacts, not bundled game sources.

| Check | Result / evidence |
|---|---|
| Editor Debug build | Passed; deployment-editor-build.raw.log; 6 unused-code warnings |
| Delayed-help timer | 1 passed; deployment-tooltip.raw.log |
| Actual conversion UI | Visually inspected; deployment-ui.png shows migrated settings and parameter help |
| Isolated original Title build | Passed in 35.700 s; deployment-title-isolated-build.raw.log |
| Title autoplay in PCSX-Redux | 12.092 s, 153 hardware note starts, one active sequence, zero steals/denials/clamps/capacity/clock errors; deployment-title-smoke/evidence.json |
| Full Ironwood build | Failed RAM budget; deployment-title-profile-build.raw.log; explicitly deferred |
| Final native audio service | sequence_service and instrument_service passed; deployment-native-final.raw.log |
| Native Room service | Passed; deployment-native.raw.log |
| Previous complete native suite | 28 passed; p4-final-all-native.raw.log |
| Previous Rust sequence suite | 33 passed, 2 ignored; p4-final-rust-sequence.raw.log |
| Previous library suite | 15 passed, 3 ignored; p4-final-library-tests.raw.log |
| Final host whole-song render | Source 12.627 s / Target 3.172 s; zero steals, denied notes or clipping; p4-final-host-payload/report.json |
| PCSX-Redux full song | 228 s, two loops, 3,408 hardware note starts, no steals/denials/clamps/capacity/clock errors; p4-psx-song-1789306112256969300 |
| Independent observer | Failed its drift assertion: 2,579.172 us maximum; other metrics retained in p4-observer-1789306438973577300/observations.json |

The full normal song run measured 1,889.409 us maximum audio service, 2,054 us
maximum service gap, 2,729 us maximum actual key-on delay and 486.399 us average
service (48.537% of measured guest CPU). Physical peak was 21 voices. These are
isolated song measurements, not complete Ironwood frame-performance guarantees.

The same isolated executable used 1,663,688 / 2,097,152 main RAM bytes. Cooked
samples consume 515,520 SPU bytes, plus 4,096 capture bytes; the sequence is
36,928 bytes and bank metadata 15,488 bytes. The independent observer measured
33,360 bytes of executable stack and 16 bytes of SDK heap extent; BIOS-private
stacks are outside that observation. Its drift failure remains unresolved.

The final Title-only executable is in
`.epok/ironwood-midi-title-delivery/.epok/build/epok.ps-exe`, SHA-256
`74649dd37e3d086b89263f01038ba6fdbb9e635d1f8f652294dfa79d2452fd86`.
Its measured static main RAM is **1,704,248 / 2,097,152 bytes**. The 12-second smoke
measured 1,618.717 us maximum service, 2,001 us maximum gap, 2,292 us maximum actual
key-on delay and 19 physical voices. All three actual menu SFX are resident in
this build; this smoke did not press menu buttons or change scenes.

To use MIDI now, reopen the rebuilt editor and use opening_02's Source / Target
Preview. For a Title-only PSX Play build, select only Title in Play's Selected
Scenes, with Title as the initial scene. Adding the Forest scenes currently
reintroduces the documented RAM blocker. The isolated validation project keeps
that narrower scope without removing the user's Forest selection from Ironwood.

## Deferred acceptance and limitations

### Physical-console follow-up

The user subsequently reported that Title sounds correct and usually reaches
60 FPS in the emulator, but the serial-loaded console plays only a few voices,
flickers and fluctuates between 60 and 12 FPS. This is a hardware regression
report; previous emulator passes do not invalidate it.

A first correction batches chord/layer KON writes into one mask per hardware
register per flush. Sample DMA now acknowledges Stop before changing its address,
acknowledges DMA Write before starting, drains the transfer FIFO, and returns to
acknowledged Stop. Every wait has a bounded timeout. The shared upload path also
covers resident SFX. Transfer ordering follows the documented
[SPU DMA procedure](https://psx-spx.consoledev.net/soundprocessingunitspu/#spu-ram-dma-write).

Regression evidence: `hardware-fix-native.raw.log` passes disabled audio, legacy
sequence service, layered service and delayed transfer acknowledgement/timeout
tests. `hardware-fix-spu.raw.log` passes the emulator's owned layered tone,
RPN pitch, Room send and lifecycle test.

Physical follow-up uses the configured NOPS tool on COM12 with `/fast`, without
PING, against UNIROM v8.0.K. The user confirmed that all instruments now sound
on the console after the KON/DMA fix (`f3a5745`). A halted snapshot after 27.895 s
recorded 19 peak voices, 363 key-ons, zero denied notes and zero clock/runtime
errors. Audio service consumed 63.31% CPU, averaging 682.54 us per service with
a 2374.34 us maximum; the HUD reported 12 FPS. Evidence is
`artifacts/midi-completion/hardware-audio-01.{bin,json}` and
`hardware-hud-01.bin`. This confirms the instrument correction, but also the
remaining performance regression on real hardware.

The next correction holds library envelope/modulation output between 250 Hz
control updates while keeping the MIDI scheduler on every approximately 1 kHz
IRQ. The first envelope step follows the note start at the next service.
Controllers and release advance the old state to the event time before applying
the new operation, preserving elapsed time across control ticks. Legacy EPSB v1
and SFX paths keep their cadence. This adds 192 bytes to the physical voice pool;
Title uses 1,712,936 bytes of static RAM. Native service tests pass, including
controller/release timing between ticks. The emulator's layered pitch/Room and
lifecycle capture passes (`hardware-control-spu.raw.log`, evidence
`p4-psx-library-1789311347446158600`, measured pitch error -1.406 cents).
The first warm reload stopped before runtime initialization (all counters zero),
so `hardware-audio-02.bin` is not performance evidence. After a clean UNIROM boot,
the optimized executable ran for 31.726 s on the physical console: 60 FPS in the
HUD, 26.07% audio service CPU, average service 268.22 us, 19 peak voices, 417
key-ons, and zero steals, denials, capacity errors, pitch clamps or clock/runtime
errors. The user confirmed stable video and complete audio. Evidence:
`hardware-audio-03.{bin,json}`, `hardware-hud-03.bin`; executable SHA-256
`5e2f16356559ba17bdea1317c417a891ba0dc2e9e7700cfbdda101f3e03c36fe`.
The baseline and candidate snapshots cover similar but not identical song
intervals (27.895 s versus 31.726 s). This is a short hardware smoke acceptance,
not a full-song timing guarantee: maximum service was 2748.49 us, maximum service
gap 3042 us and maximum key-on delay 3315 us. Worst-case IRQ latency remains open.
The console was halted to collect the snapshots and COM12 was released; resuming
after a long debugger pause is not a valid clock-continuity test.

- Fit Title plus the Forest scenes in main RAM before claiming the complete game
  deploys. The full-game link failure is an integration blocker, not an audio
  quality test that passed. Keep source library snapshots on the authoring host;
  design a measured residency solution instead of silently removing instruments.
- Investigate the independent full-loop guest-clock drift failure and rerun the
  observer. Do not claim it passed merely because the normal song run passed.
- Complete final full-song Title plus simultaneous menu SFX, priority pressure,
  scene unload/transition, CD geometry and XA regressions. The isolated two-loop
  test reserves SFX bytes but does not instantiate the actual menu SFX.
- Rerun final SPU pitch/Room capture and full legacy/library stress matrices after
  deployment integration. The earlier owned tone measured 495.452 Hz, -0.760 cents
  from its independent RPN oracle, with nonzero Room memory; it is earlier evidence.
- Complete draft Source/Target A/B before saving, exhaustive UI/cancellation/cache
  interactions and broader conversion-quality/performance acceptance from P5/P6.
- Source Preview preserves SoundFont filters; target BakeSustain is an explicit
  reported approximation. Animated filters outside supported policy remain errors.
  Host Target Preview uses linear interpolation and omits wet PSX reverb.
- Sony SEQ/SEP/VAB fidelity adapters (P7) remain blocked until their prerequisites
  and evidence exist. No title-specific format names or fabricated support.
- Physical PSX full-song/performance acceptance and disc deployment remain open;
  the short serial hardware run above confirms restored instruments and stable
  Title video at 60 FPS in the observed interval, with the latency limits above.
  Other console backends remain unverified.
- The earlier full Rust baseline had 399 passes, 35 ignored and one existing GUI
  failure; Clippy had 18 baseline diagnostics. No all-tests-green claim is made.

Commands for remaining checks are in `tests/integration/verify_psx_library.py`,
`verify_psx_library_song.py --release-ms 450 --voice-limit 21`,
`verify_psx_music_observer.py`, `verify_psx_sequence_stress.py --library`,
`verify_audio_assets.py`, `verify_streaming_xa.py` and `verify_serial_xa.py`.
Check each script's prerequisites and keep emulators/MIPS builds serialized.

## Changed engine files

The pre-existing Console-preview edits in docs/assets.md, src/asset_inspector.rs, src/content_preview.rs and src/project_browser.rs were preserved. The delivery contains the following engine files; Ironwood files are listed separately above.

- build.rs
- docs/architecture/psx-midi-completion-plan.md
- docs/architecture/psx-midi-completion-progress.md
- docs/architecture/psx-midi-delivery.md
- docs/assets.md
- native/instrument_preview.cpp
- native/instrument_preview.h
- native/instrument_source_preview.cpp
- native/instrument_source_preview.h
- native/sequence_preview.cpp
- resources/audio/FluidR3Mono_GM.sf3
- resources/audio/fluidr3mono-2.315.json
- resources/audio/LICENSE-fluidr3mono.txt
- runtime/instrument_allocator.hpp
- runtime/instrument_bank.hpp
- runtime/instrument_preparation.hpp
- runtime/instrument_reverb.hpp
- runtime/instrument_synth.hpp
- runtime/main.cpp
- runtime/README.md
- runtime/sequence_clock.hpp
- runtime/sequence_data.hpp
- runtime/sequence_instrument_service.hpp
- runtime/sequence_kernel.hpp
- runtime/sequence_service.hpp
- src/asset_inspector.rs
- src/asset_manager.rs
- src/asset_ui.rs
- src/assets.rs
- src/audio_decode.rs
- src/content_preview.rs
- src/instrument_dsp.rs
- src/instrument_ir.rs
- src/instrument_modulation.rs
- src/instrument_preview.rs
- src/instrument_samples.rs
- src/instrument_selection.rs
- src/instrument_source_preview.rs
- src/instrument_voice.rs
- src/library_preview.rs
- src/main.rs
- src/mcp_tests.rs
- src/mcp_tools.rs
- src/memory.rs
- src/midi_controls.rs
- src/midi.rs
- src/music_conversion_ui.rs
- src/project_browser.rs
- src/project.rs
- src/psx_library_asset_tests.rs
- src/psx_library_asset.rs
- src/psx_library_wire.rs
- src/psx_library.rs
- src/psx_loop_quality.rs
- src/psx_music_optimizer.rs
- src/psx_music_settings.rs
- src/psx_sequence.rs
- src/sequence_compat.rs
- src/sequence_ir.rs
- src/sequence_preview.rs
- src/sequence_stream.rs
- src/sequence.rs
- src/sf2.rs
- src/sound_bank.rs
- src/soundfont_asset.rs
- src/spu_encoder.rs
- tests/integration/spu_capture_tone.py
- tests/integration/verify_psx_library_song.py
- tests/integration/verify_psx_library.py
- tests/integration/verify_psx_music_observer.py
- tests/integration/verify_psx_sequence_stress.py
- tests/integration/verify_psx_sequence.py
- tests/runtime/audio_transport_stub.hpp
- tests/runtime/instrument_allocator.cpp
- tests/runtime/instrument_bank.cpp
- tests/runtime/instrument_preview.cpp
- tests/runtime/instrument_reverb.cpp
- tests/runtime/instrument_service.cpp
- tests/runtime/instrument_source_preview.cpp
- tests/runtime/instrument_synth.cpp
- tests/runtime/sequence_kernel.cpp
- tests/runtime/sequence_service.cpp
- tests/runtime/verify_spatial.py
