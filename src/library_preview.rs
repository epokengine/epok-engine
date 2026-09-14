//! Instrument-library audition from immutable source snapshots and actual cooks.
use crate::{assets, preview_audio::Pcm, sequence::Settings, sequence_ir::SequenceIr};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub fn render(
    root: &Path,
    ir: &SequenceIr,
    settings: &Settings,
    record: &assets::Record,
    target: bool,
    cancelled: &AtomicBool,
) -> Result<(Pcm, crate::sequence_preview::Stats), String> {
    let package = assets::Package::load(&record.path)?;
    if package.meta.id != record.meta.id
        || assets::cache_key(&package.meta) != assets::cache_key(&record.meta)
    {
        return Err(
            "Instrument library changed while preparing preview; retry with the current revision"
                .into(),
        );
    }
    let bank_settings = package.meta.settings.sound_bank()?;
    bank_settings.validate()?;
    if bank_settings.library.is_none() {
        return Err("Instrument preview requires a source library SoundBank".into());
    }
    if bank_settings.load_mode == crate::audio_import::LoadMode::Stream {
        return Err(
            "PSX instrument banks require Resident or Auto; sequence residency is independent"
                .into(),
        );
    }
    let recipe = crate::psx_music_settings::Recipe::from_settings(settings)?;
    let selection = if target {
        recipe.selection
    } else {
        crate::psx_music_settings::Selection::Reachable
    };
    let prepared = crate::psx_library::prepare_selection(
        &package.source,
        ir,
        &settings.instrument_mappings,
        selection,
        cancelled,
    )?;
    let result = if target {
        let cooked = crate::psx_library::cook(&prepared, &recipe, cancelled)?;
        if !cooked.report.fits_sample_budget {
            return Err(format!(
                "PSX music samples require {} bytes; {} available after SFX and reverb reservations. Choose a smaller recipe or Optimize to budget",
                cooked.report.sample_spu_bytes,
                recipe.available_bytes()
            ));
        }
        render_target(ir, settings, &cooked, cancelled)?
    } else {
        let (frames, loops) = layout(ir, settings, prepared.regions.iter().map(|r| &r.voice))?;
        let events = crate::sequence_stream::events(ir, settings)?;
        let (mut pcm, stats) = crate::instrument_source_preview::render(
            &events,
            ir.ppqn,
            &prepared,
            frames,
            recipe.headroom_centibels,
            cancelled,
        )?;
        set_timeline(&mut pcm, ir, loops, frames);
        pcm.report = Some(format!(
            "{} Comparison headroom: {} dB. Observed physical peak: {}; clipped output samples: {}.",
            pcm.report.take().unwrap_or_default(),
            f32::from(recipe.headroom_centibels) / 10.,
            stats.physical_peak,
            stats.clipped
        ));
        (
            pcm,
            crate::sequence_preview::Stats {
                error: stats.error,
                peak: stats.physical_peak,
                steals: stats.steals,
                loops: stats.loops,
                clipped: stats.clipped,
            },
        )
    };
    crate::psx_library_asset::verify_record(record, cancelled)?;
    let index = assets::scan(root, &mut Default::default());
    if crate::sequence::resolve_bank(root, settings, &index)?
        .meta
        .id
        != record.meta.id
    {
        return Err(
            "Project Default SoundBank changed during audition; retry with the current selection"
                .into(),
        );
    }
    Ok(result)
}

pub fn render_target(
    ir: &SequenceIr,
    settings: &Settings,
    cooked: &crate::psx_library::Cooked,
    cancelled: &AtomicBool,
) -> Result<(Pcm, crate::sequence_preview::Stats), String> {
    let (events, _) = crate::psx_sequence::library_payload(ir, settings, uuid::Uuid::nil())?;
    let payload = crate::psx_library_wire::encode(cooked)?;
    let (frames, loops) = layout(ir, settings, cooked.zones.iter().map(|z| &z.voice))?;
    let (mut pcm, stats) = crate::instrument_preview::render(
        &events,
        ir.ppqn,
        &payload,
        &cooked.samples,
        settings.voices(),
        frames,
        cancelled,
    )?;
    set_timeline(&mut pcm, ir, loops, frames);
    pcm.report = Some(format!(
        "{} Samples: {} SPU bytes; global reverb reservation: {} bytes; bank main RAM: {} bytes. Observed logical/physical peaks: {}/{}; denied notes: {}; steals: {}; clipped output samples: {}. Reverb wet audio is not simulated; use an emulator capture for Room. {} explicit conversion adaptations.",
        pcm.report.take().unwrap_or_default(),
        cooked.report.sample_spu_bytes,
        cooked.report.reverb_spu_bytes,
        payload.len(),
        stats.logical_peak,
        stats.physical_peak,
        stats.denied,
        stats.steals,
        stats.clipped,
        cooked.report.adaptations.len()
    ));
    if cancelled.load(Ordering::Relaxed) {
        return Err("Instrument preview cancelled".into());
    }
    Ok((
        pcm,
        crate::sequence_preview::Stats {
            error: stats.error,
            peak: stats.physical_peak,
            steals: stats.steals,
            loops: stats.loops,
            clipped: stats.clipped,
        },
    ))
}

fn layout<'a>(
    ir: &SequenceIr,
    settings: &Settings,
    voices: impl Iterator<Item = &'a crate::instrument_voice::Voice>,
) -> Result<(usize, Option<(u64, u64)>), String> {
    // A release-time modulator can lengthen a tail. Conservatively include its
    // full admitted excursion; the source/target renderer never truncates it.
    let tail = voices
        .map(|voice| {
            let increase = voice
                .modulations
                .iter()
                .filter(|m| {
                    m.destination == crate::instrument_voice::Destination::VolEnvReleaseCents
                })
                .map(|m| m.amount.unsigned_abs().min(80_000))
                .sum::<u64>()
                .min(80_000) as i64;
            let tc = (i64::from(voice.volume_envelope.release.0) + increase).clamp(-12_000, 8_000);
            (2_f64.powf(tc as f64 / 1200.) * 1_000_000.).ceil() as u64
        })
        .max()
        .unwrap_or(0);
    let loops = match settings.loop_mode {
        crate::sequence::LoopMode::Off => None,
        crate::sequence::LoopMode::Whole => Some((0, ir.duration_micros)),
        crate::sequence::LoopMode::Markers => ir
            .loop_region
            .map(|[a, b]| (ir.micros_at(a), ir.micros_at(b))),
    };
    let end = loops.map_or(
        ir.duration_micros.saturating_add(tail).saturating_add(1000),
        |(a, b)| b.saturating_add(b - a),
    );
    let frames = (u128::from(end) * 44100).div_ceil(1_000_000).max(1);
    if frames * 2 > crate::audio_ir::MAX_SAMPLES as u128 {
        return Err("Instrument audition exceeds the bounded ten-minute stereo PCM budget".into());
    }
    Ok((frames as usize, loops))
}
fn set_timeline(pcm: &mut Pcm, ir: &SequenceIr, loops: Option<(u64, u64)>, frames: usize) {
    pcm.loop_region = loops.map(|(_, b)| ((b * 44100).div_ceil(1_000_000) as usize, frames));
    pcm.timeline = Some(crate::preview_audio::Timeline {
        duration: ir.duration_micros as f32 / 1_000_000.,
        loop_region: loops.map(|(a, b)| (a as f32 / 1_000_000., b as f32 / 1_000_000.)),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn library_preview_source_target_snapshot_cancellation_and_room_budget() {
        let root = crate::workspace::tests::temp("library-preview");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(root.join("assets/library.sf2"), crate::sf2::tone_fixture()).unwrap();
        let id = assets::commit(
            crate::soundfont_asset::prepare(
                &root,
                "assets/library.sf2",
                "assets/library.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let before = std::fs::read(root.join("assets/library.epokasset")).unwrap();
        let midi = crate::midi::fixture();
        let mut settings = Settings {
            sound_bank: Some(id),
            loop_mode: crate::sequence::LoopMode::Off,
            ..Default::default()
        };
        let cancel = AtomicBool::new(false);
        let (source, source_stats) =
            crate::sequence_preview::render(&root, &midi, &settings, &cancel).unwrap();
        let (target, target_stats) =
            crate::sequence_preview::render_target(&root, &midi, &settings, &cancel).unwrap();
        assert!(source.samples.iter().any(|s| *s != 0) && target.samples.iter().any(|s| *s != 0));
        assert_eq!((source_stats.error, target_stats.error), (0, 0));
        assert!(
            source
                .report
                .as_ref()
                .unwrap()
                .contains("original decoded PCM")
        );
        assert!(
            target
                .report
                .as_ref()
                .unwrap()
                .contains("decoded cooked SPU ADPCM")
        );
        assert_eq!(
            std::fs::read(root.join("assets/library.epokasset")).unwrap(),
            before
        );
        cancel.store(true, Ordering::Relaxed);
        assert!(
            crate::sequence_preview::render_target(&root, &midi, &settings, &cancel)
                .unwrap_err()
                .contains("cancelled")
        );
        cancel.store(false, Ordering::Relaxed);
        let mut recipe = crate::psx_music_settings::Recipe::default();
        recipe.preset = crate::psx_music_settings::Preset::Custom;
        recipe.effects = crate::psx_music_settings::Effects::Room;
        assert_eq!(
            recipe.available_bytes(),
            crate::audio_import::SPU_BUDGET as u32 - 9920
        );
        recipe.store(&mut settings).unwrap();
        let (room, _) =
            crate::sequence_preview::render_target(&root, &midi, &settings, &cancel).unwrap();
        assert!(
            room.report
                .unwrap()
                .contains("Reverb wet audio is not simulated")
        );
    }
    fn write_wav(path: &Path, pcm: &Pcm) {
        use std::io::Write;
        let mut file = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
        let size = (pcm.samples.len() * 2) as u32;
        file.write_all(b"RIFF").unwrap();
        file.write_all(&(size + 36).to_le_bytes()).unwrap();
        file.write_all(b"WAVEfmt \x10\0\0\0\x01\0\x02\0").unwrap();
        file.write_all(&pcm.rate.to_le_bytes()).unwrap();
        file.write_all(&(pcm.rate * 4).to_le_bytes()).unwrap();
        file.write_all(b"\x04\0\x10\0data").unwrap();
        file.write_all(&size.to_le_bytes()).unwrap();
        for sample in &pcm.samples {
            file.write_all(&sample.to_le_bytes()).unwrap();
        }
        file.flush().unwrap();
    }
    #[test]
    #[ignore = "External hash-pinned song/library; set EPOK_MIDI_SOURCE, EPOK_SOUNDFONT_LIBRARY and EPOK_P4_REPORT_DIR"]
    fn library_preview_ironwood_full_source_and_actual_target() {
        let midi = std::fs::read(std::env::var_os("EPOK_MIDI_SOURCE").unwrap()).unwrap();
        let library = std::fs::read(std::env::var_os("EPOK_SOUNDFONT_LIBRARY").unwrap()).unwrap();
        assert_eq!(
            assets::hash(&midi),
            "5709b0b1b24d93b7c83199b168d573f1e915add144b340633502da2e4b299efe"
        );
        assert_eq!(
            assets::hash(&library),
            "cda013d8c370a48ae8dad271e761078d2e77455488dabdedbfbe5fc76a38c682"
        );
        let output = std::path::PathBuf::from(std::env::var_os("EPOK_P4_REPORT_DIR").unwrap());
        std::fs::create_dir_all(&output).unwrap();
        let cancel = AtomicBool::new(false);
        let ir = crate::midi::parse(&midi).unwrap();
        let prepared = crate::psx_library::prepare(&library, &ir, &[], &cancel).unwrap();
        let mut recipe = crate::psx_music_settings::Recipe::default();
        recipe.preset = crate::psx_music_settings::Preset::Custom;
        recipe.max_sample_rate = 10208;
        recipe.other_resident_bytes = 4672;
        recipe.maximum_release_ms = std::env::var("EPOK_P4_RELEASE_MS")
            .ok()
            .map(|s| s.parse().unwrap());
        let cooked = crate::psx_library::cook(&prepared, &recipe, &cancel).unwrap();
        assert!(cooked.report.fits_sample_budget);
        let voices: u16 = std::env::var("EPOK_P4_VOICE_LIMIT")
            .ok()
            .map(|s| s.parse().unwrap())
            .unwrap_or(24);
        let mut settings = Settings {
            loop_mode: crate::sequence::LoopMode::Off,
            voice_limit: Some(voices),
            ..Default::default()
        };
        recipe.store(&mut settings).unwrap();
        let events = crate::sequence_stream::events(&ir, &settings).unwrap();
        let (frames, _) =
            layout(&ir, &settings, prepared.regions.iter().map(|r| &r.voice)).unwrap();
        let start = std::time::Instant::now();
        let (source, ss) = crate::instrument_source_preview::render(
            &events,
            ir.ppqn,
            &prepared,
            frames,
            recipe.headroom_centibels,
            &cancel,
        )
        .unwrap();
        let source_ms = start.elapsed().as_secs_f64() * 1000.;
        let payload = crate::psx_library_wire::encode(&cooked).unwrap();
        let start = std::time::Instant::now();
        let (target_events, sequence) =
            crate::psx_sequence::library_payload(&ir, &settings, uuid::Uuid::nil()).unwrap();
        let (target, ts) = crate::instrument_preview::render(
            &target_events,
            ir.ppqn,
            &payload,
            &cooked.samples,
            voices,
            frames,
            &cancel,
        )
        .unwrap();
        let target_ms = start.elapsed().as_secs_f64() * 1000.;
        write_wav(&output.join("opening_02-source-dry.wav"), &source);
        write_wav(&output.join("opening_02-target-dry.wav"), &target);
        std::fs::write(output.join("opening_02.epsb"), payload).unwrap();
        std::fs::write(output.join("opening_02.epsq"), sequence).unwrap();
        let report = serde_json::json!({"source_ms":source_ms,"target_ms":target_ms,"frames":frames,
            "source":{"logical_peak":ss.logical_peak,"physical_peak":ss.physical_peak,"steals":ss.steals,"denied":ss.denied,"clipped":ss.clipped},
            "target":{"logical_peak":ts.logical_peak,"physical_peak":ts.physical_peak,"steals":ts.steals,"denied":ts.denied,"clipped":ts.clipped},"cook":cooked.report});
        std::fs::write(
            output.join("report.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("{report}");
        assert_eq!((ss.error, ts.error), (0, 0));
        assert_eq!((ss.denied, ts.denied), (0, 0));
        if recipe.maximum_release_ms.is_some() {
            assert_eq!(
                ts.steals, 0,
                "explicit adapted recipe must fit physical voices"
            );
        }
        assert!(source.samples.iter().any(|s| *s != 0) && target.samples.iter().any(|s| *s != 0));
    }
}
