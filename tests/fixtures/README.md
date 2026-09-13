# Audio fixture

`portable-tone.flac` and `portable-tone.ogg` are original 100 ms stereo sine
signals (440/660 Hz, 22,050 Hz, amplitudes 0.25/0.20), under the repository MIT
license. Generated with FFmpeg's `aevalsrc`, FLAC and libvorbis (`-q:a 4`), with
no recordings or instrument samples. Both compressed fixtures exercise the real
decoder, import snapshot and host preview paths; FFmpeg is not a test dependency.

`audio-legacy-golden.json` records SHA-256 results captured at `58e370f` before
the portable audio schema migration, using `audio_legacy_golden_outputs`.
Inputs are the original Epok-generated tone in `audio_import::test_wav`, an
explicit v1 package and synthetic XA sectors. It covers the package, resident
ADPCM (one-shot, loop and trimmed/normalized/resampled), bank declarations and
all four XA interleave profiles. The signals contain no third-party samples.

`stereo-tone.mp3` is a generated three-second test signal: 440 Hz left and 660 Hz right, 44.1 kHz stereo, encoded at 128 kbps using LAME (lameenc 1.8.4). It contains no third-party recording. The fixture is provided under the repository's MIT license. Encoder delay/padding is intentional; tests do not assume this MP3 contains gapless metadata.


`resources/models/EpokMannequin.fbx` is original MIT-licensed Epok sample content (96 vertices, 144 triangles, Idle/Walk). Recreate it with Blender using `create_skeletal_fixture.py`. The importer/tests use the checked-in FBX and do not require Blender.
