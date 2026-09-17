# Epok Pulse — SPU music driver implementation and verification

## Authoring

SoundFont-derived MusicSequences default to **Music driver → Epok Pulse** in
PSX music conversion. Existing recipes without a driver field select this path
when cooked again. Keep the existing SoundBank reference and AudioComponent or
Blueprint `play` call. Source snapshots and UUIDs are preserved. Choosing
**Software reference** restores the previous runtime synthesis implementation.

## Execution model

The editor resolves MIDI ordering, pedals, note lifetimes, instrument regions,
layer admission, pitch/gain modulation and PSX ADSR rates. EPSQ v3 contains
absolute microsecond timestamps, atomic note groups, starts, release/cut,
pitch/volume/send changes and loop boundaries. Identical tone templates are
deduplicated. EPSB v2 remains the sample-bank format.

The IRQ dispatches commands to the SPU and shares the 24 hardware voices with
SFX. It performs neither SoundFont envelope/LFO evaluation nor MIDI note-off
matching. Hardware ADSR and ADPCM playback run in the SPU. Native-only games omit
the reference synthesizer's working states and MIDI FIFO. Legacy/mixed games
retain the compatibility service. All paths retain bounded priorities and
whole-layer stealing, deferred key-on, source activation, bank pins and reverb
ownership.

Epok Pulse is an original driver built around editor-compiled commands and
hardware SPU envelopes. Register/timer behavior was checked against
[PSX SPU documentation](https://psx-spx.consoledev.net/soundprocessingunitspu/) and
[timer documentation](https://psx-spx.consoledev.net/timers/).

Timer 0 now uses system clock with target 64000 (about 529 Hz). Its old DIV8 flag
did not divide Timer 0, so the old configuration was approximately 2000 Hz, not
250 Hz. Target 65535 was rejected by emulator testing: target/overflow coincidence
did not deliver audio callbacks in the installed Redux core. Timer 2 still measures
elapsed CPU/8 time and Timer 1 supplies the wrap guard.

## Measured result, 2026-09-17

Installed PCSX-Redux, interpreter, 2 MiB RAM; an isolated copy of Denise's arena
and `hm-bonneforces-exe`. Seven emulated seconds measured after warm-up. Same
scene/camera/song/48,000-byte sample budget in all cases; profiling HUD and motion
interpolation disabled **in all three copies** to let the reference driver pass
the minimum heap/stack guard. The benchmark did not modify the source demo.

| Mode | FPS | Measured sequence-service CPU | Maximum observed service |
| --- | ---: | ---: | ---: |
| Software reference | 29.36 | 15.52% | 7.61 ms |
| Epok Pulse | 34.49 | 3.49% | 1.89 ms |
| Native, no playback | 35.35 | 1.43% | 0.027 ms |

This is about 77.5% less measured service CPU, not a zero-cost audio claim.
Playing the native song adds about 2.06 percentage points over its idle service.
Counters exclude BIOS interrupt-dispatch overhead; all three cases use the same
corrected timer. No clock fault or runtime diagnostic occurred. These numbers
are emulator evidence for this workload, not a console-wide or hardware guarantee.

With the original HUD/interpolation settings, a separate native/silent run gave
33.50/34.36 FPS. On MIPS, native instance/physical working arrays occupy 2,320 +
960 bytes; this excludes bank/sequence data, audio infrastructure and stack.
The tested native sequence is 120,824 bytes plus a 55,552-byte bank (47,936 sample
bytes uploaded to SPU RAM), without prepared software note-state tables.

Reproduce with `python tests/integration/verify_native_music.py PATH_TO_DENISE_PROJECT` after
building the editor. It copies inputs to a new owned `.epok/` directory and starts
only its own headless emulator on an unused port; it does not change the input
project or terminate an existing emulator.

## Fidelity and bounds

- Hardware ADSR is quantized. Source delay is omitted and hold is folded into
  decay; conversion and Target Preview report affected layers. Source Preview
  remains the reference for the authored SoundFont behavior.
- Controllers that dynamically retime an active volume envelope fail native
  conversion with a Software reference diagnostic. Pan, gain, pitch bend, tuning,
  tempo and sustain/sostenuto semantics are resolved offline.
- Pitch/gain automation is sampled at 250 Hz plus MIDI event boundaries and only
  changed register values are emitted. Excessive data fails the 256 KiB / 32,768
  command budget; an authored pass is limited to ten minutes. Loops can repeat.
- The actual song reaches the 16-physical-voice ceiling including release tails;
  2,613 offline voice steals are reported for the whole song. Its analyzed
  11-note MIDI peak is **not** a measurement of physical voices including tails.
  The new driver does not silently shorten the source's release setting.
- Target Preview decodes cooked ADPCM and the compiled commands with a hardware
  ADSR model. It is not a bit-exact SPU emulator: linear interpolation replaces
  Gaussian interpolation; wet reverb, hardware key-on latency and SFX contention
  are not simulated.
- Real console validation and a subjective full-song listening review remain.

Host regression coverage includes conversion/audition, pedal release timing
across tempo changes, bend/pan automation, repeated loops, malformed/truncated
input, cancellation, ADSR fitting/chunk invariance, native/mixed playback,
atomic layers, SFX stealing, source disable, gain, bank retirement, legacy
sequence playback and the existing instrument allocator/bank tests.
