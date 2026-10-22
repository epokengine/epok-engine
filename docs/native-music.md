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

Native-only builds schedule the next command, deferred key-on or retirement on
Timer 2's compare register. The counter stays free-running at CPU/8; only its
target moves. Idle gaps are capped at 7 ms, below the counter's wrap guard.
Recovery is measured from the actual SPU key-off and waits at least 128 CPU/8
ticks (about 30 microseconds). This avoids polling unchanged SPU voices at 529 Hz
and lets cooled free SFX voices start without a redundant busy wait.

Mixed/legacy builds retain Timer 0 with target 64000 (about 529 Hz). Its old DIV8 flag
did not divide Timer 0, so the old configuration was approximately 2000 Hz, not
250 Hz. Target 65535 was rejected by emulator testing: target/overflow coincidence
did not deliver audio callbacks in the installed Redux core. Timer 2 still measures
elapsed CPU/8 time and Timer 1 supplies the wrap guard.

`python tests/runtime/verify_spatial.py native_music_clock native_music_service
native_music_mixed sequence_service audio_disabled spu_transfer` checks compare
deadlines across 16-bit wrap, overdue events, timer routing, key-off recovery,
source changes, layered stealing and the legacy transport. Hardware timer
behavior follows the [timer register specification](https://psx-spx.consoledev.net/timers/).

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

The native Timer 2 scheduler leaves at least 1 ms between its final live counter
read and the next compare, allowing BIOS acknowledgement/return and competing
controller/card interrupts. Compares within 512 ticks of either side of counter
overflow move to target 512 after the wrap. A target of 1 could be stepped over
by Redux's overflow handling, adding a full counter wrap to the expected gap.
Elapsed time still comes from the free-running counter, rather than the requested
interval. A delayed IRQ spanning at most 1,600 HSync lines (about 102 ms) uses
Timer 1 to identify the unique number of Timer 2 wraps. The elapsed CPU ticks
remain exact; HSync does not replace them with an estimated tempo. The admissible
NTSC/PAL timing window is narrower than one wrap. Inconsistent counter pairs and
longer gaps still stop the sequence with an error. `sequence_clock_diagnostics`
records recovered/rejected gaps and the first interrupted PC for hardware checks.
See the [timer register specification](https://psx-spx.consoledev.net/timers/) and
[GPU clock timings](https://psx-spx.consoledev.net/graphicsprocessingunitgpu/#gpu-timings).

PCDrv geometry reads, external scene-file chunks and the runtime's BIOS startup
messages explicitly pause the sequence clock, including file open, seek and close.
Unirom can hold interrupts while
forwarding TTY text as well as file data.
The outermost operation resynchronizes the Timer 2 and HSync baselines before
playback continues and rearms the next native compare, preventing a missed
transport-time compare from adding a full counter wrap. This keeps serial I/O from becoming an
ambiguous elapsed-time fault. Nested operations retain the pause. Unexpected
interrupt gaps outside these storage operations use the bounded recovery above;
unresolvable gaps still report a clock fault.

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
