//! Host software audition using the same bounded musical kernel as the PSX backend.
use crate::{
    assets,
    preview_audio::Pcm,
    sequence::{LoopMode, Settings},
    sequence_stream::{Event, events},
};
use std::{
    ffi::c_void,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

#[repr(C)]
struct Zone {
    samples: *const f32,
    frames: u32,
    rate: u32,
    loop_start: u32,
    loop_end: u32,
    channels: u16,
    program: u8,
    drum_key: u8,
    key_lo: u8,
    key_hi: u8,
    velocity_lo: u8,
    velocity_hi: u8,
    root_key: u8,
    reserved: u8,
    cents: f32,
    gain: f32,
    pan: f32,
    attack_ms: f32,
    decay_ms: f32,
    sustain: f32,
    release_ms: f32,
}
#[repr(C)]
#[derive(Debug, Default)]
pub struct Stats {
    pub error: u32,
    pub peak: u32,
    pub steals: u32,
    pub loops: u32,
    pub clipped: u32,
}
unsafe extern "C" {
    fn epok_sequence_create(
        events: *const Event,
        count: u32,
        ppqn: u16,
        voices: u16,
        zones: *const Zone,
        zone_count: u32,
        rate: u32,
    ) -> *mut c_void;
    fn epok_sequence_destroy(handle: *mut c_void);
    fn epok_sequence_render(handle: *mut c_void, output: *mut i16, frames: u32) -> i32;
    fn epok_sequence_stats(handle: *const c_void) -> Stats;
}
struct Native(*mut c_void);
impl Drop for Native {
    fn drop(&mut self) {
        unsafe { epok_sequence_destroy(self.0) }
    }
}

pub fn render(
    root: &Path,
    bytes: &[u8],
    settings: &Settings,
    cancelled: &AtomicBool,
) -> Result<(Pcm, Stats), String> {
    render_mode(root, bytes, settings, cancelled, false)
}
pub fn render_target(
    root: &Path,
    bytes: &[u8],
    settings: &Settings,
    cancelled: &AtomicBool,
) -> Result<(Pcm, Stats), String> {
    render_mode(root, bytes, settings, cancelled, true)
}
fn render_mode(
    root: &Path,
    bytes: &[u8],
    settings: &Settings,
    cancelled: &AtomicBool,
    target: bool,
) -> Result<(Pcm, Stats), String> {
    let ir = crate::sequence::decode_source(bytes, settings)?;
    let events = events(&ir, settings)?;
    let index = assets::scan(root, &mut Default::default());
    let record = crate::sequence::resolve_bank(root, settings, &index)?;
    let bank = record.meta.settings.sound_bank()?;
    if bank.library.is_some() {
        return crate::library_preview::render(root, &ir, settings, record, target, cancelled);
    }
    if bank.library.is_none()
        && crate::psx_music_settings::Recipe::from_settings(settings)?.effects
            != crate::psx_music_settings::Effects::Dry
    {
        return Err("Room reverb requires an instrument library SoundBank; portable v1 banks retain their dry playback contract".into());
    }
    if bank.library.is_none() && !settings.instrument_mappings.is_empty() {
        return Err("Explicit library mappings require an instrument library SoundBank; portable v1 banks use their saved zones".into());
    }
    bank.validate_sequence(&ir, &index)?;
    let mut dependencies = vec![record];
    for id in bank.dependencies() {
        dependencies.push(index.resolve(id)?);
    }
    let verify_dependencies = || -> Result<(), String> {
        for record in &dependencies {
            if assets::hash(&assets::read_bounded(&record.path)?) != record.revision {
                return Err(format!(
                    "SoundBank/sample changed during audition preparation: {}. Play again to use the current revision.",
                    record.path.display()
                ));
            }
        }
        if crate::sequence::resolve_bank(root, settings, &index)?
            .meta
            .id
            != record.meta.id
        {
            return Err(
                "Project Default SoundBank changed during audition preparation. Play again.".into(),
            );
        }
        Ok(())
    };
    verify_dependencies()?;
    if bank.load_mode == crate::audio_import::LoadMode::Stream {
        return Err("Host audition currently requires a Resident/Auto SoundBank. Its sample residency is independent of the sequence.".into());
    }
    let target_bank = if target {
        crate::psx_sequence::sequence_payload(&ir, settings, record.meta.id)?;
        Some(crate::psx_sequence::bank(root, record, &index)?)
    } else {
        None
    };
    let mut target_samples = Vec::new();
    // Decode each original snapshot once, even when several zones reference it.
    let mut samples = std::collections::BTreeMap::new();
    let mut total = 0;
    for id in bank.dependencies().into_iter().filter(|_| !target) {
        if cancelled.load(Ordering::Relaxed) {
            return Err("Audio preview cancelled".into());
        }
        let sample = assets::Package::load(&index.resolve(id)?.path)?;
        let s = sample.meta.settings.audio()?;
        let original = crate::audio_ir::DecodedAudioIr::decode(&sample.source)?;
        s.validate()?;
        let (trim_start, trim_end) = crate::audio_import::trim_range(&original.info, s)?;
        let mut pcm = Vec::with_capacity((trim_end - trim_start) * s.channels as usize);
        for frame in original.samples[trim_start * original.info.channels as usize
            ..trim_end * original.info.channels as usize]
            .chunks_exact(original.info.channels as usize)
        {
            if s.channels == 1 {
                pcm.push(frame.iter().sum::<f32>() / frame.len() as f32);
            } else {
                pcm.extend_from_slice(&[frame[0], frame[frame.len() - 1]]);
            }
        }
        if s.normalize {
            let peak = pcm.iter().fold(0f32, |a, b| a.max(b.abs()));
            if peak > 0. {
                for value in &mut pcm {
                    *value *= 0.95 / peak;
                }
            }
        }
        total += pcm.len();
        if total > crate::audio_ir::MAX_SAMPLES {
            return Err("SoundBank host PCM exceeds the 52,920,000 sample audition budget".into());
        }
        samples.insert(id, (pcm, original.info.sample_rate, s.channels, trim_start));
    }
    let mut zones = Vec::new();
    let mut tail_ms = 0f32;
    if let Some(bank) = &target_bank {
        for sample in &bank.samples {
            if cancelled.load(Ordering::Relaxed) {
                return Err("Audio preview cancelled".into());
            }
            target_samples.push(
                crate::audio_import::decode_adpcm(&sample.bytes)?
                    .into_iter()
                    .map(|v| v as f32 / 32768.)
                    .collect::<Vec<_>>(),
            );
        }
        for z in &bank.zones {
            let sample = &bank.samples[z.sample as usize];
            let pcm = &target_samples[z.sample as usize];
            tail_ms = tail_ms.max(z.release_ms as f32);
            zones.push(Zone {
                samples: pcm.as_ptr(),
                frames: pcm.len() as u32,
                rate: sample.rate,
                loop_start: sample.loop_region.map_or(0, |r| r[0]),
                loop_end: sample.loop_region.map_or(0, |r| r[1]),
                channels: 1,
                program: z.program,
                drum_key: z.drum_key,
                key_lo: z.key_range[0],
                key_hi: z.key_range[1],
                velocity_lo: z.velocity_range[0],
                velocity_hi: z.velocity_range[1],
                root_key: z.root_key,
                reserved: 0,
                cents: z.cents as f32 / 100.,
                gain: z.gain as f32 / 4096.,
                pan: z.pan as f32 / 16384.,
                attack_ms: z.attack_ms as f32,
                decay_ms: z.decay_ms as f32,
                sustain: z.sustain as f32 / 32767.,
                release_ms: z.release_ms as f32,
            });
        }
    }
    for p in bank.programs.iter().filter(|_| !target) {
        for z in &p.zones {
            let (pcm, rate, channels, trim_start) = &samples[&z.sample];
            let frames = (pcm.len() / *channels as usize) as u32;
            let [loop_start, loop_end] = if let Some([start, end]) = z.sample_loop {
                if (start as usize) < *trim_start || (end as usize) > *trim_start + frames as usize
                {
                    return Err(format!(
                        "SoundBank sample {} loop lies outside its saved source trim",
                        z.sample
                    ));
                }
                [start - *trim_start as u32, end - *trim_start as u32]
            } else {
                [0, 0]
            };
            tail_ms = tail_ms.max(z.envelope.release_ms);
            zones.push(Zone {
                samples: pcm.as_ptr(),
                frames,
                rate: *rate,
                loop_start,
                loop_end,
                channels: *channels,
                program: p.program,
                drum_key: p.drum_key.unwrap_or(255),
                key_lo: z.key_range[0],
                key_hi: z.key_range[1],
                velocity_lo: z.velocity_range[0],
                velocity_hi: z.velocity_range[1],
                root_key: z.root_key,
                reserved: 0,
                cents: z.fine_tune_cents,
                gain: z.gain,
                pan: z.pan,
                attack_ms: z.envelope.attack_ms,
                decay_ms: z.envelope.decay_ms,
                sustain: z.envelope.sustain,
                release_ms: z.envelope.release_ms,
            });
        }
    }
    let rate = 44100u32;
    let loop_times = match settings.loop_mode {
        LoopMode::Off => None,
        LoopMode::Whole => Some((0, ir.duration_micros)),
        LoopMode::Markers => ir
            .loop_region
            .map(|[a, b]| (ir.micros_at(a), ir.micros_at(b))),
    };
    // Keep the first pass (including any intro tails), then repeat the steady second pass.
    // WinMM loops are quantized to output frames; the kernel itself retains exact PPQN time.
    let end_micros = loop_times.map_or(
        ir.duration_micros + (tail_ms as f64 * 1000.).ceil() as u64 + 1000,
        |(a, b)| b + b - a,
    );
    let frames = (end_micros as u128 * rate as u128)
        .div_ceil(1_000_000)
        .max(1);
    if frames * 2 > crate::audio_ir::MAX_SAMPLES as u128 {
        return Err("Sequence audition exceeds the 10-minute stereo PCM budget; shorten the sequence/loop for preview".into());
    }
    let mut pcm = Pcm {
        samples: vec![0; frames as usize * 2],
        rate,
        channels: 2,
        loop_region: loop_times.map(|(_, b)| {
            (
                (b * rate as u64).div_ceil(1_000_000) as usize,
                frames as usize,
            )
        }),
        report: target_bank.as_ref().map(|bank| format!("PSX cooked sample audition: {} SPU bytes, {} bank payload bytes. Uses decoded ADPCM and quantized bank settings; the host mixer does not emulate SPU Gaussian interpolation, IRQ/key-on latency or physical SFX contention. {}", bank.samples.iter().map(|s| s.bytes.len()).sum::<usize>(), bank.payload.len(), bank.warnings.join(" "))),
        timeline: Some(crate::preview_audio::Timeline {
            duration: ir.duration_micros as f32 / 1_000_000.,
            loop_region: loop_times.map(|(a, b)| (a as f32 / 1_000_000., b as f32 / 1_000_000.)),
        }),
    };
    // Rust owns and pins events, zones and PCM vectors until Native is destroyed. Render allocates nothing.
    let native = Native(unsafe {
        epok_sequence_create(
            events.as_ptr(),
            events.len() as u32,
            ir.ppqn,
            settings.voices(),
            zones.as_ptr(),
            zones.len() as u32,
            rate,
        )
    });
    if native.0.is_null() {
        return Err("Cannot initialize the bounded sequence preview backend".into());
    }
    for chunk in pcm.samples.chunks_mut(4096 * 2) {
        if cancelled.load(Ordering::Relaxed) {
            return Err("Audio preview cancelled".into());
        }
        let error =
            unsafe { epok_sequence_render(native.0, chunk.as_mut_ptr(), (chunk.len() / 2) as u32) };
        if error != 0 {
            return Err(format!(
                "Sequence kernel stopped with diagnostic {error} (1 input, 2 service capacity, 3 missing instrument, 4 clock overflow)"
            ));
        }
    }
    let stats = unsafe { epok_sequence_stats(native.0) };
    verify_dependencies()?;
    Ok((pcm, stats))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    pub fn fixture(root: &Path) -> (uuid::Uuid, uuid::Uuid) {
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(
            root.join("assets/tone.wav"),
            crate::audio_import::test_wav(),
        )
        .unwrap();
        let sample = assets::commit(
            assets::prepare(
                root,
                "assets/tone.wav",
                "assets/tone.epokasset",
                crate::audio_import::Settings {
                    load_mode: crate::audio_import::LoadMode::Stream,
                    ..Default::default()
                },
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let mut zone = crate::sound_bank::Zone::new(sample);
        zone.sample_loop = Some([0, 2205]);
        zone.envelope.release_ms = 20.;
        let bank = crate::sound_bank::create(
            root,
            "assets/bank.epokasset",
            crate::sound_bank::Settings {
                programs: vec![crate::sound_bank::Program {
                    program: 0,
                    drum_key: None,
                    zones: vec![zone],
                    extra: Default::default(),
                }],
                provenance: "Epok-owned generated test tone, MIT".into(),
                ..Default::default()
            },
        )
        .unwrap();
        (sample, bank)
    }
    #[test]
    fn midi_host_renders_original_bank_deterministically_and_loops_with_bounded_voices() {
        let root = crate::workspace::tests::temp("midi-host");
        let (_, bank) = fixture(&root);
        let mut settings = Settings {
            sound_bank: Some(bank),
            ..Default::default()
        };
        let bytes = crate::midi::fixture();
        let (first, stats) = render(&root, &bytes, &settings, &AtomicBool::new(false)).unwrap();
        assert_eq!(first.channels, 2);
        assert_eq!(stats.error, 0);
        assert_eq!(stats.peak, 1);
        assert_eq!(stats.steals, 0);
        assert!(first.samples.iter().any(|s| s.abs() > 1000));
        assert_eq!(&first.samples[first.samples.len() - 80..], &[0; 80]);
        let (second, _) = render(&root, &bytes, &settings, &AtomicBool::new(false)).unwrap();
        assert_eq!(first.samples, second.samples);
        settings.loop_mode = LoopMode::Whole;
        let (looped, stats) = render(&root, &bytes, &settings, &AtomicBool::new(false)).unwrap();
        let (start, end) = looped.loop_region.unwrap();
        assert_eq!(stats.loops, 1);
        assert_eq!(start * 2, end);
        assert_eq!(&looped.samples[..start * 2], &looped.samples[start * 2..]);
        assert!(
            !root.join(".epok/imported").exists(),
            "Source bank audition must not invoke a target cooker"
        );
        assert!(
            render(&root, &bytes, &settings, &AtomicBool::new(true))
                .unwrap_err()
                .contains("cancelled")
        );
        println!(
            "Host sequence: {} frames, {} PCM bytes, peak {}, steals {}, clipped {}; loop {}..{} frames",
            first.samples.len() / 2,
            first.samples.len() * 2,
            stats.peak,
            stats.steals,
            stats.clipped,
            start,
            end
        );
    }
    #[test]
    fn midi_rendered_pitch_matches_independent_rpn_and_legacy_frequency_oracles() {
        let root = crate::workspace::tests::temp("midi-pitch-oracle");
        let (_, bank) = fixture(&root); // The owned sample is a 440 Hz sine, rooted at key 60.
        let mut track = vec![
            0, 0x90, 60, 100, 0, 0xb0, 101, 0, 0, 0xb0, 100, 0, 0, 0xb0, 6, 12,
        ];
        // PPQN 480 with the default 500000 us tempo: delta 960 is exactly one second.
        track.extend_from_slice(&[0x87, 0x40, 0xe0, 0, 69]); // 8832: +93.75 cents at range 12.
        track.extend_from_slice(&[0x87, 0x40, 0xe0, 0, 75]); // 9600: +206.25 cents.
        track.extend_from_slice(&[0x87, 0x40, 0xb0, 100, 1, 0, 0xb0, 6, 96]); // Fine +50 cents.
        track.extend_from_slice(&[0x87, 0x40, 0xb0, 100, 2, 0, 0xb0, 6, 65]); // Coarse +100 cents.
        track.extend_from_slice(&[0x87, 0x40, 0xb0, 121, 0]); // Center bend; retain RPN tuning.
        track.extend_from_slice(&[0x87, 0x40, 0x80, 60, 0, 0, 0xff, 0x2f, 0]);
        let mut midi = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
        midi.extend_from_slice(&(track.len() as u32).to_be_bytes());
        midi.extend(track);
        for (profile, cents) in [
            (
                crate::midi::MidiProfile::MusicalV2,
                [0., 93.75, 206.25, 256.25, 356.25, 150.],
            ),
            (
                crate::midi::MidiProfile::LegacyV1,
                [0., 15.625, 34.375, 34.375, 34.375, 34.375],
            ),
        ] {
            let (pcm, stats) = render(
                &root,
                &midi,
                &Settings {
                    sound_bank: Some(bank),
                    midi_profile: profile,
                    ignore_unsupported: profile == crate::midi::MidiProfile::LegacyV1,
                    ..Default::default()
                },
                &AtomicBool::new(false),
            )
            .unwrap();
            assert_eq!((stats.error, stats.steals, stats.clipped), (0, 0, 0));
            for (second, cents) in cents.into_iter().enumerate() {
                // Measure the emitted waveform, independently of the kernel's pitch calculation.
                let start = second * pcm.rate as usize + pcm.rate as usize / 10;
                let end = (second + 1) * pcm.rate as usize - pcm.rate as usize / 10;
                let mut crossings = Vec::new();
                for frame in start..end {
                    let a = pcm.samples[frame * 2] as f64;
                    let b = pcm.samples[(frame + 1) * 2] as f64;
                    if a <= 0. && b > 0. {
                        crossings.push(frame as f64 - a / (b - a));
                    }
                }
                assert!(crossings.len() > 300);
                let measured = (crossings.len() - 1) as f64 * pcm.rate as f64
                    / (crossings.last().unwrap() - crossings[0]);
                let expected = 440. * 2_f64.powf(cents / 1200.);
                let error_cents = 1200. * (measured / expected).log2();
                println!(
                    "{profile:?} second {second}: {measured:.6} Hz, expected {expected:.6}, error {error_cents:.6} cents"
                );
                assert!(
                    error_cents.abs() < 0.2,
                    "{profile:?} second {second}: pitch error {error_cents} cents"
                );
            }
        }
    }
    #[test]
    fn midi_host_reports_missing_banks_zones_and_invalid_sample_loops() {
        let root = crate::workspace::tests::temp("midi-host-errors");
        let (_, bank) = fixture(&root);
        let bytes = crate::midi::fixture();
        assert!(
            render(&root, &bytes, &Settings::default(), &AtomicBool::new(false))
                .unwrap_err()
                .contains("SoundBank")
        );
        let index = assets::scan(&root, &mut Default::default());
        let record = index.resolve(bank).unwrap();
        let mut settings = record.meta.settings.sound_bank().unwrap().clone();
        settings.programs[0].zones[0].sample_loop = Some([0, 2206]);
        assets::commit(
            assets::prepare_portable(
                &root,
                "",
                "assets/bank.epokasset",
                crate::import_settings::Settings::SoundBank(settings),
                Some(record),
                true,
            )
            .unwrap(),
        )
        .unwrap();
        assert!(
            render(
                &root,
                &bytes,
                &Settings {
                    sound_bank: Some(bank),
                    ..Default::default()
                },
                &AtomicBool::new(false)
            )
            .unwrap_err()
            .contains("outside")
        );
    }
    #[test]
    fn midi_starter_bank_has_owned_pcm_provenance_and_no_implicit_default() {
        let root = crate::workspace::tests::temp("midi-starter");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let candidates =
            crate::sound_bank::starter_candidates(&root, "assets/Retro.epokasset").unwrap();
        let hash = candidates[0].package.meta.source_hash.clone();
        let again = crate::sound_bank::starter_candidates(&root, "assets/Other.epokasset").unwrap();
        assert_eq!(hash, again[0].package.meta.source_hash);
        assert_eq!(
            crate::audio_decode::decode(&candidates[0].package.source)
                .unwrap()
                .0
                .frames,
            2205
        );
        let bank = crate::sound_bank::publish_starter(candidates).unwrap();
        assert!(crate::sound_bank::starter_candidates(&root, "assets/Retro.epokasset").is_err());
        let index = assets::scan(&root, &mut Default::default());
        let settings = index
            .resolve(bank)
            .unwrap()
            .meta
            .settings
            .sound_bank()
            .unwrap();
        assert!(settings.provenance.contains("MIT"));
        assert_eq!(settings.dependencies().len(), 1);
        assert!(crate::sequence::resolve_bank(&root, &Settings::default(), &index).is_err());
        let (pcm, stats) = render(
            &root,
            &crate::midi::fixture(),
            &Settings {
                sound_bank: Some(bank),
                ..Default::default()
            },
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(pcm.samples.iter().any(|v| v.abs() > 1000));
        assert_eq!(stats.clipped, 0);
        println!(
            "Owned starter source SHA256 {hash}; {} bytes stereo audition",
            pcm.samples.len() * 2
        );
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "Plays and stops real sequenced MIDI on the Windows output, including a repeating loop"]
    fn midi_native_output_completes_loops_and_retires_buffers_on_stop() {
        let root = crate::workspace::tests::temp("midi-native-output");
        let (_, bank) = fixture(&root);
        for loop_mode in [LoopMode::Off, LoopMode::Whole] {
            let (pcm, _) = render(
                &root,
                &crate::midi::fixture(),
                &Settings {
                    sound_bank: Some(bank),
                    loop_mode,
                    ..Default::default()
                },
                &AtomicBool::new(false),
            )
            .unwrap();
            let player = crate::preview_audio::Player::start(pcm).unwrap();
            let start = std::time::Instant::now();
            if loop_mode == LoopMode::Off {
                while !player.finished() {
                    assert!(start.elapsed().as_secs() < 5);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            } else {
                std::thread::sleep(std::time::Duration::from_millis(1250));
                assert!(
                    !player.finished(),
                    "WinMM must repeat the sequence until Stop"
                );
            }
            drop(player); // Reset completes before any prepared header or PCM allocation is freed.
        }
    }
}
