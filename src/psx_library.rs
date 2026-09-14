//! Song-specific PSX instrument conversion. The authoritative library is never pruned.
use crate::{
    instrument_selection::{Coverage, Instrument, Mapping, NoteUse, RegionId},
    instrument_voice::{Destination, LoopMode, Sustain, Voice},
    psx_music_settings::{Effects, FilterPolicy, Recipe, SampleLoops},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, Ordering},
};

pub const MAX_ZONES: usize = 128;
pub const MAX_SAMPLES: usize = 128;

pub struct Region {
    pub id: RegionId,
    pub instrument: Instrument,
    pub notes: BTreeSet<NoteUse>,
    pub voice: Voice,
}
pub struct Prepared {
    pub source_hash: String,
    pub coverage: Coverage,
    pub regions: Vec<Region>,
    pub pcm: BTreeMap<u16, crate::instrument_samples::SamplePcm>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Adaptation {
    pub region: RegionId,
    pub code: String,
    pub detail: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sample {
    pub source_sample: u16,
    pub bytes: Vec<u8>,
    pub rate: u32,
    pub frames: u32,
    /// Exclusive decoded PCM coordinates, including the initial 28-frame block.
    pub loop_region: Option<[u32; 2]>,
    pub squared_error: u64,
    pub input_frames: usize,
    pub improved_encoder: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Zone {
    pub sample: u16,
    pub instrument: Instrument,
    pub voice: Voice,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Accounting {
    pub source_samples: usize,
    pub encoded_variants: usize,
    pub modulation_records: usize,
    /// One immutable EPSB copy is retained in the linked runtime's main RAM.
    pub bank_main_ram_bytes: usize,
    pub metadata_and_alignment_bytes: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoopReport {
    pub region: RegionId,
    pub source_loop: [i64; 2],
    pub aligned_loop: [u32; 2],
    pub crossfade_frames: u16,
    pub before: crate::psx_loop_quality::LoopQuality,
    pub after: crate::psx_loop_quality::LoopQuality,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub profile: String,
    pub recipe: Recipe,
    pub note_events: usize,
    pub regions: usize,
    pub samples: usize,
    pub song_peak_layers_per_note: usize,
    pub sample_spu_bytes: usize,
    pub other_resident_bytes: u32,
    pub available_bank_bytes: u32,
    #[serde(default)]
    pub reverb_spu_bytes: u32,
    pub fits_sample_budget: bool,
    pub adaptations: Vec<Adaptation>,
    pub squared_error: u64,
    pub encoded_input_frames: usize,
    pub maximum_loop_step: f64,
    #[serde(default)]
    pub accounting: Accounting,
    #[serde(default)]
    pub loops: Vec<LoopReport>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cooked {
    pub samples: Vec<Sample>,
    pub zones: Vec<Zone>,
    pub report: Report,
}

/// A reduced bank belongs to this selection and source song, never just to the
/// library asset UUID. The implementation identity covers the target pipeline.
pub fn identity(
    prepared: &Prepared,
    recipe: &Recipe,
    sequence_identity: &str,
) -> Result<String, String> {
    let regions = prepared
        .regions
        .iter()
        .map(|r| (&r.id, &r.instrument, &r.notes, &r.voice))
        .collect::<Vec<_>>();
    let inputs = BTreeMap::from([
        ("library-source".into(), prepared.source_hash.clone()),
        ("sequence-selection".into(), sequence_identity.into()),
        (
            "reachable-regions".into(),
            crate::assets::hash(&serde_json::to_vec(&regions).map_err(|e| e.to_string())?),
        ),
        (
            "recipe".into(),
            serde_json::to_string(recipe).map_err(|e| e.to_string())?,
        ),
        ("profile".into(), crate::psx_music_settings::PROFILE.into()),
    ]);
    Ok(crate::psx_sequence::identity(&inputs))
}

pub fn prepare(
    source: &[u8],
    sequence: &crate::sequence_ir::SequenceIr,
    mappings: &[Mapping],
    cancelled: &AtomicBool,
) -> Result<Prepared, String> {
    prepare_selection(
        source,
        sequence,
        mappings,
        crate::psx_music_settings::Selection::Reachable,
        cancelled,
    )
}
pub fn prepare_selection(
    source: &[u8],
    sequence: &crate::sequence_ir::SequenceIr,
    mappings: &[Mapping],
    selection: crate::psx_music_settings::Selection,
    cancelled: &AtomicBool,
) -> Result<Prepared, String> {
    let library = crate::sf2::parse(source)?;
    let mut coverage =
        crate::instrument_selection::resolve(sequence, &library, mappings, cancelled)?;
    coverage.require_complete()?;
    if selection == crate::psx_music_settings::Selection::FullLibrary {
        if library
            .presets
            .iter()
            .map(|p| p.regions.len())
            .sum::<usize>()
            > MAX_ZONES
        {
            return Err("Full Library exceeds this PSX profile's 128 regions; choose Reachable explicitly or a smaller library. The original library was preserved".into());
        }
        validate_full_mappings(&library, mappings)?;
        for (preset_id, preset) in library.presets.iter().enumerate() {
            for (region_id, region) in preset.regions.iter().enumerate() {
                check_cancelled(cancelled)?;
                let id = RegionId {
                    preset: preset_id,
                    region: region_id,
                };
                let mut instruments = mappings
                    .iter()
                    .filter(|m| {
                        m.library_bank == preset.bank && m.library_program == preset.program
                    })
                    .map(|m| m.source)
                    .collect::<BTreeSet<_>>();
                let default = Instrument {
                    bank: if preset.bank == 128 { 0 } else { preset.bank },
                    program: preset.program,
                    percussion: preset.bank == 128,
                };
                if !mappings.iter().any(|m| m.source == default) {
                    instruments.insert(default);
                }
                for instrument in instruments {
                    // These references select a documented bake velocity. The full
                    // key/velocity ranges remain in the Voice and are not narrowed.
                    coverage.regions.entry(id).or_default().insert(NoteUse {
                        instrument,
                        key: region.key_range[0],
                        velocity: region.velocity_range[1],
                    });
                }
                coverage.samples.insert(region.sample);
            }
        }
    }
    let mut regions = Vec::new();
    for (&id, notes) in &coverage.regions {
        check_cancelled(cancelled)?;
        let source_region = &library.presets[id.preset].regions[id.region];
        let voice = crate::instrument_voice::from_soundfont(&library, source_region)?;
        // An explicit alias may use the same preset from two source MIDI banks.
        // Both runtime identities survive; sample data may still be deduplicated.
        let instruments = notes
            .iter()
            .map(|note| note.instrument)
            .collect::<BTreeSet<_>>();
        for instrument in instruments {
            if regions.len() == MAX_ZONES {
                return Err("PSX library selection exceeds 128 physical voice regions; no layers were discarded".into());
            }
            regions.push(Region {
                id,
                instrument,
                voice: voice.clone(),
                notes: notes
                    .iter()
                    .filter(|note| note.instrument == instrument)
                    .copied()
                    .collect(),
            });
        }
    }
    if regions.is_empty() {
        return Err("The sequence has no reachable instrument notes".into());
    }
    let pcm = crate::instrument_samples::decode_selected(source, &library, &coverage, cancelled)?;
    Ok(Prepared {
        source_hash: crate::assets::hash(source),
        coverage,
        regions,
        pcm,
    })
}
fn validate_full_mappings(
    library: &crate::instrument_ir::LibraryIr,
    mappings: &[Mapping],
) -> Result<(), String> {
    for mapping in mappings {
        if !library
            .presets
            .iter()
            .any(|p| (p.bank, p.program) == (mapping.library_bank, mapping.library_program))
        {
            return Err("Full Library has an explicit mapping to a missing library preset".into());
        }
        let canonical_bank = if mapping.source.percussion {
            128
        } else {
            mapping.source.bank
        };
        let has_canonical = !mapping.source.percussion || mapping.source.bank == 0;
        if has_canonical
            && library
                .presets
                .iter()
                .any(|p| (p.bank, p.program) == (canonical_bank, mapping.source.program))
            && (canonical_bank, mapping.source.program)
                != (mapping.library_bank, mapping.library_program)
        {
            return Err("Full Library mapping conflicts with a canonical preset identity; choose Reachable for substitutions or use a free source bank/program alias".into());
        }
    }
    Ok(())
}
fn check_cancelled(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Relaxed) {
        Err("PSX library conversion cancelled".into())
    } else {
        Ok(())
    }
}
fn adaptation(output: &mut Vec<Adaptation>, region: &Region, code: &str, detail: String) {
    output.push(Adaptation {
        region: region.id,
        code: code.into(),
        detail,
    });
}
fn static_filter(
    region: &Region,
    recipe: &Recipe,
    adaptations: &mut Vec<Adaptation>,
) -> Result<crate::instrument_dsp::FilterSpec, String> {
    let voice = &region.voice;
    if voice.modulation_lfo.filter_cents != 0
        || voice
            .modulations
            .iter()
            .any(|m| m.destination == Destination::ModLfoFilterCents && m.amount != 0)
    {
        return Err(format!(
            "Region {:?} needs an animated LFO filter; this PSX recipe cannot bake that operation",
            region.id
        ));
    }
    if recipe.filter_policy == FilterPolicy::RequireStatic && voice.mod_env_filter_cents != 0 {
        return Err(format!(
            "Region {:?} uses a filter envelope. Choose the explicit Bake Sustain adaptation or a library with static filters",
            region.id
        ));
    }
    let reference = region
        .notes
        .iter()
        .next()
        .ok_or("Selected region has no reference note")?;
    let changes = crate::instrument_modulation::evaluate(
        voice,
        reference.key,
        reference.velocity,
        &Default::default(),
    )?;
    let delta = |destination| {
        changes
            .iter()
            .find(|(d, _)| *d == destination)
            .map_or(0., |(_, value)| *value)
    };
    let envelope_amount = voice.mod_env_filter_cents as f64 + delta(Destination::ModEnvFilterCents);
    if delta(Destination::ModLfoFilterCents) != 0. {
        return Err("A modulator requires an animated LFO filter outside this PSX recipe".into());
    }
    if recipe.filter_policy == FilterPolicy::RequireStatic
        && (envelope_amount != 0.
            || voice.modulations.iter().any(|m| {
                matches!(
                    m.destination,
                    Destination::FilterCents | Destination::FilterCentibels
                ) && m.amount != 0
                    && (m.source.input != crate::instrument_voice::Input::Constant
                        || m.amount_source.input != crate::instrument_voice::Input::Constant)
            }))
    {
        return Err("Require Static cannot represent this note/controller-dependent filter; choose the explicit Bake Sustain adaptation".into());
    }
    let sustain = match voice.modulation_envelope.sustain {
        Sustain::ReductionPermille(value) => value as f64,
        _ => return Err("Modulation envelope uses incompatible sustain units".into()),
    };
    let sustain =
        1. - ((sustain + delta(Destination::ModEnvSustainPermille)) / 1000.).clamp(0., 1.);
    let cents =
        (voice.filter_cents as f64 + delta(Destination::FilterCents) + envelope_amount * sustain)
            .clamp(1500., 13500.);
    if envelope_amount != 0.
        || voice.modulations.iter().any(|m| {
            m.amount != 0
                && matches!(
                    m.destination,
                    Destination::FilterCents
                        | Destination::FilterCentibels
                        | Destination::ModEnvFilterCents
                        | Destination::ModEnvSustainPermille
                )
        })
    {
        adaptation(
            adaptations,
            region,
            "filter_baked_at_sustain",
            format!(
                "Static filter at {cents:.3} absolute cents, reference key {} velocity {}. Filter-envelope motion and velocity differences are approximated; sample transposition also transposes this filter. Original parameters remain in the source library.",
                reference.key, reference.velocity
            ),
        );
    }
    Ok(crate::instrument_dsp::FilterSpec {
        cutoff_hz: 8.175798915643707 * 2_f64.powf(cents / 1200.),
        resonance_centibels: (voice.filter_centibels as f64 + delta(Destination::FilterCentibels))
            .clamp(0., 960.),
    })
}

/// Produces actual encoded samples even for an over-budget report. Publication
/// and playback must require `fits_sample_budget`; there is no streaming fallback.
pub fn cook(
    prepared: &Prepared,
    recipe: &Recipe,
    cancelled: &AtomicBool,
) -> Result<Cooked, String> {
    recipe.validate()?;
    let mut samples = Vec::new();
    let mut zones = Vec::new();
    let mut dedup = BTreeMap::new();
    let mut adaptations = Vec::new();
    let mut maximum_loop_step = 0_f64;
    let mut loops = Vec::new();
    for region in &prepared.regions {
        check_cancelled(cancelled)?;
        let voice = &region.voice;
        if voice.modulations.iter().any(|m| {
            matches!(
                m.destination,
                Destination::StartFrames
                    | Destination::EndFrames
                    | Destination::LoopStartFrames
                    | Destination::LoopEndFrames
            )
        }) {
            return Err(format!(
                "Region {:?} modulates sample positions; this PSX profile cannot apply source frame offsets after resampling and block alignment",
                region.id
            ));
        }
        let source = prepared
            .pcm
            .get(&voice.sample)
            .ok_or("Selected source PCM is missing")?;
        let start = voice.start_offset;
        let end = source.samples.len() as i64 + voice.end_offset;
        if start < 0 || start >= end || end > source.samples.len() as i64 {
            return Err(format!(
                "Region {:?} sample offsets exceed the decoded source window; no adjacent sample data was substituted",
                region.id
            ));
        }
        let source_pcm = &source.samples[start as usize..end as usize];
        let filter = static_filter(region, recipe, &mut adaptations)?;
        let filtered = if filter.cutoff_hz >= source.rate as f64 * 0.5 {
            adaptation(
                &mut adaptations,
                region,
                "filter_above_source_band",
                format!(
                    "Filter cutoff {:.3} Hz reaches/exceeds source Nyquist at {} Hz; the static bake retains source-band PCM (including no resonant peak outside that band)",
                    filter.cutoff_hz, source.rate
                ),
            );
            source_pcm.to_vec()
        } else {
            crate::instrument_dsp::low_pass(source_pcm, source.rate, filter, cancelled)?
        };
        let rate = recipe.max_sample_rate.min(source.rate);
        let mut pcm = crate::instrument_dsp::resample(&filtered, source.rate, rate, cancelled)?;
        let blocks = pcm.len().div_ceil(28);
        let loop_blocks = if voice.loop_mode == LoopMode::Off {
            None
        } else {
            let a = voice.loop_start - start;
            let b = voice.loop_end - start;
            if a < 0 || a >= b || b > end - start {
                return Err(format!(
                    "Region {:?} sample loop lies outside its decoded source",
                    region.id
                ));
            }
            let numerator_a = a as u64 * rate as u64;
            let numerator_b = b as u64 * rate as u64;
            let denominator = source.rate as u64 * 28;
            let a = (numerator_a / denominator) as usize;
            let b = numerator_b.div_ceil(denominator) as usize;
            let scaled_a = numerator_a as f64 / source.rate as f64;
            let scaled_b = numerator_b as f64 / source.rate as f64;
            if a >= b || b > blocks {
                return Err("Sample loop is outside the encoded block range".into());
            }
            if numerator_a % denominator != 0 || numerator_b % denominator != 0 {
                if recipe.sample_loops == SampleLoops::ExactBlocks {
                    return Err(format!(
                        "Region {:?} loop requires ADPCM block alignment; choose Align Outward explicitly",
                        region.id
                    ));
                }
                adaptation(
                    &mut adaptations,
                    region,
                    "loop_block_alignment",
                    format!(
                        "Source loop {}..{} becomes {scaled_a:.4}..{scaled_b:.4} frames at {rate} Hz; encoded loop {}..{} (exclusive, before initial silence)",
                        voice.loop_start,
                        voice.loop_end,
                        a * 28,
                        b * 28
                    ),
                );
            }
            if voice.loop_mode == LoopMode::Continuous {
                pcm.resize(b * 28, 0.);
            }
            // UntilRelease retains all frames after the loop. Playback switches
            // the voice's repeat address to that tail when the note is released.
            Some([a, b])
        };
        let quantize = |pcm: &[f32]| -> Result<Vec<i16>, String> {
            let gain = recipe.gain();
            let mut integer_pcm = Vec::with_capacity(pcm.len());
            for (i, &value) in pcm.iter().enumerate() {
                if i % 4096 == 0 {
                    check_cancelled(cancelled)?;
                }
                let value = value as f64 * gain;
                if !value.is_finite() || value.abs() > 1. {
                    return Err(format!(
                        "Region {:?} clips after filtering; increase Headroom explicitly",
                        region.id
                    ));
                }
                integer_pcm.push((value * 32767.).round() as i16);
            }
            Ok(integer_pcm)
        };
        let mut integer_pcm = quantize(&pcm)?;
        let mut encoded = crate::spu_encoder::encode_region(
            &integer_pcm,
            loop_blocks,
            recipe.encoder_effort,
            cancelled,
        )?;
        let frames = (integer_pcm.len().div_ceil(28) as u32 + 1) * 28;
        let loop_region = loop_blocks.map(|[a, b]| [(a as u32 + 1) * 28, (b as u32 + 1) * 28]);
        if let Some([a, b]) = loop_blocks {
            let decoded = decode_contiguous(&encoded.bytes, frames)?;
            let before =
                crate::psx_loop_quality::analyze(&decoded[28..], [a * 28, b * 28], 64, cancelled)?;
            let mut actual = 0;
            if recipe.loop_crossfade_frames > 0 {
                if pcm.len() < b * 28 {
                    pcm.resize(b * 28, 0.);
                }
                actual = crate::psx_loop_quality::crossfade(
                    &mut pcm,
                    [a * 28, b * 28],
                    recipe.loop_crossfade_frames,
                    cancelled,
                )?;
                integer_pcm = quantize(&pcm)?;
                encoded = crate::spu_encoder::encode_region(
                    &integer_pcm,
                    loop_blocks,
                    recipe.encoder_effort,
                    cancelled,
                )?;
                adaptation(
                    &mut adaptations,
                    region,
                    "loop_crossfade",
                    format!(
                        "Crossfaded {actual} frames before loop end at {rate} Hz; loop coordinates, period and release tail retained"
                    ),
                );
            }
            let decoded = decode_contiguous(&encoded.bytes, frames)?;
            let after =
                crate::psx_loop_quality::analyze(&decoded[28..], [a * 28, b * 28], 64, cancelled)?;
            maximum_loop_step = maximum_loop_step.max(after.step);
            loops.push(LoopReport {
                region: region.id,
                source_loop: [voice.loop_start, voice.loop_end],
                aligned_loop: loop_region.unwrap(),
                crossfade_frames: actual,
                before,
                after,
            });
        }
        let mut bytes = encoded.bytes;
        if voice.loop_mode == LoopMode::UntilRelease {
            if let Some([start, _]) = loop_region {
                // Repeat address is owned by the service: a loop-start flag
                // would undo release before the first traversal reaches it.
                bytes[(start / 28 * 16 + 1) as usize] &= !4;
            }
        }
        bytes.resize(bytes.len().div_ceil(64) * 64, 0);
        let key = crate::assets::hash(
            format!(
                "{rate}:{frames}:{loop_region:?}:{}",
                crate::assets::hash(&bytes)
            )
            .as_bytes(),
        );
        let sample = if let Some(&index) = dedup.get(&key) {
            index
        } else {
            if samples.len() == MAX_SAMPLES {
                return Err(
                    "PSX library requires more than 128 encoded samples; no samples were discarded"
                        .into(),
                );
            }
            let index = samples.len() as u16;
            dedup.insert(key, index);
            samples.push(Sample {
                source_sample: voice.sample,
                bytes,
                rate,
                frames,
                loop_region,
                squared_error: encoded.squared_error,
                input_frames: integer_pcm.len(),
                improved_encoder: encoded.improved,
            });
            index
        };
        let mut target_voice = voice.clone();
        if let Some(maximum_ms) = recipe.maximum_release_ms {
            if voice
                .modulations
                .iter()
                .any(|m| m.destination == Destination::VolEnvReleaseCents && m.amount != 0)
            {
                return Err(format!(
                    "Region {:?} modulates release time; the explicit release limit cannot preserve this modulation. Choose Preserve release or another instrument mapping",
                    region.id
                ));
            }
            let limit = (1200. * (f64::from(maximum_ms) / 1000.).log2()).floor() as i32;
            if target_voice.volume_envelope.release.0 > limit {
                let original = target_voice.volume_envelope.release.seconds() * 1000.;
                target_voice.volume_envelope.release.0 = limit;
                adaptation(
                    &mut adaptations,
                    region,
                    "release_limit",
                    format!(
                        "Full-scale volume release reduced from {original:.3} ms to {:.3} ms (requested maximum {maximum_ms} ms). This shortens note tails and reduces physical voice demand; MIDI notes, attack, sustain and source envelopes are preserved",
                        target_voice.volume_envelope.release.seconds() * 1000.
                    ),
                );
            }
        }
        target_voice.sample = sample;
        target_voice.start_offset = 0;
        target_voice.end_offset = 0;
        target_voice.loop_start = loop_region.map_or(0, |r| i64::from(r[0]));
        target_voice.loop_end = loop_region.map_or(0, |r| i64::from(r[1]));
        // These are explicit recipe adaptations, not discarded source metadata.
        target_voice.mod_env_filter_cents = 0;
        target_voice.filter_cents = 13500;
        target_voice.filter_centibels = 0;
        target_voice.modulations.retain(|m| {
            !matches!(
                m.destination,
                Destination::FilterCents
                    | Destination::FilterCentibels
                    | Destination::ModEnvFilterCents
                    | Destination::ModLfoFilterCents
            )
        });
        let sends = voice
            .modulations
            .iter()
            .filter(|m| {
                m.amount != 0
                    && matches!(
                        m.destination,
                        Destination::ReverbPermille | Destination::ChorusPermille
                    )
            })
            .count();
        if recipe.effects == Effects::Dry
            && (voice.reverb_permille != 0 || voice.chorus_permille != 0 || sends != 0)
        {
            adaptation(
                &mut adaptations,
                region,
                "dry_effects",
                format!(
                    "Dry recipe: library reverb {}‰, chorus {}‰ and {sends} potentially nonzero send modulators disabled",
                    voice.reverb_permille, voice.chorus_permille
                ),
            );
        }
        if recipe.effects == Effects::Room {
            adaptation(
                &mut adaptations,
                region,
                "psx_room_reverb",
                format!(
                    "Global PSX Room reverb: {} bytes reserved, output {}‰; positive reverb sends become binary hardware sends. Explicit CC91=0 disables the channel send; one sequence owns the resource at a time. Chorus {}‰ is disabled. Target Preview omits wet audio; use an emulator capture to assess this effect",
                    recipe.reverb_bytes(),
                    recipe.reverb_depth_permille,
                    voice.chorus_permille
                ),
            );
        } else {
            target_voice.reverb_permille = 0;
        }
        target_voice.chorus_permille = 0;
        target_voice.modulations.retain(|m| {
            m.destination != Destination::ChorusPermille
                && (recipe.effects == Effects::Room || m.destination != Destination::ReverbPermille)
        });
        zones.push(Zone {
            sample,
            instrument: region.instrument,
            voice: target_voice,
        });
    }
    let sample_spu_bytes = samples.iter().map(|s| s.bytes.len()).sum::<usize>();
    let modulation_records = zones
        .iter()
        .map(|z| z.voice.modulations.len())
        .sum::<usize>();
    let metadata_and_alignment_bytes = (crate::psx_library_wire::HEADER_BYTES
        + samples.len() * crate::psx_library_wire::SAMPLE_BYTES
        + zones.len() * crate::psx_library_wire::ZONE_BYTES
        + modulation_records * crate::psx_library_wire::MOD_BYTES)
        .div_ceil(64)
        * 64;
    let report = Report {
        profile: crate::psx_music_settings::PROFILE.into(),
        recipe: recipe.clone(),
        note_events: prepared.coverage.note_on_events,
        regions: zones.len(),
        samples: samples.len(),
        song_peak_layers_per_note: prepared.coverage.peak_layers_per_note,
        sample_spu_bytes,
        other_resident_bytes: recipe.other_resident_bytes,
        available_bank_bytes: recipe.available_bytes(),
        reverb_spu_bytes: recipe.reverb_bytes(),
        fits_sample_budget: sample_spu_bytes <= recipe.available_bytes() as usize,
        adaptations,
        squared_error: samples.iter().map(|s| s.squared_error).sum(),
        encoded_input_frames: samples.iter().map(|s| s.input_frames).sum(),
        maximum_loop_step,
        accounting: Accounting {
            source_samples: prepared.pcm.len(),
            encoded_variants: samples.len(),
            modulation_records,
            bank_main_ram_bytes: metadata_and_alignment_bytes + sample_spu_bytes,
            metadata_and_alignment_bytes,
        },
        loops,
    };
    Ok(Cooked {
        samples,
        zones,
        report,
    })
}

/// Decode a cooked sample's linear body, including an UntilRelease tail. The
/// regular AudioClip decoder still stops at its first ADPCM end flag.
pub fn decode_contiguous(bytes: &[u8], frames: u32) -> Result<Vec<i16>, String> {
    if frames < 56 || frames % 28 != 0 {
        return Err("Invalid bank decoded frame count".into());
    }
    let extent = (frames as usize / 28 + 1)
        .checked_mul(16)
        .ok_or("Bank ADPCM extent overflows")?;
    let mut linear = bytes
        .get(..extent)
        .ok_or("Incomplete bank ADPCM extent")?
        .to_vec();
    let last = linear.len() / 16 - 2;
    for (index, block) in linear.chunks_exact_mut(16).enumerate() {
        if index < last {
            block[1] &= !1;
        } else if index == last {
            block[1] |= 1;
        }
    }
    crate::audio_import::decode_adpcm(&linear)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::psx_music_settings::Preset;

    pub(crate) fn prepared_fixture() -> Prepared {
        let source = crate::sf2::fixture();
        let library = crate::sf2::parse(&source).unwrap();
        let mut voice =
            crate::instrument_voice::from_soundfont(&library, &library.presets[0].regions[0])
                .unwrap();
        voice.modulations.clear();
        voice.loop_mode = LoopMode::UntilRelease;
        voice.loop_start = 56;
        voice.loop_end = 224;
        let instrument = Instrument {
            bank: 0,
            program: 0,
            percussion: false,
        };
        let id = RegionId {
            preset: 0,
            region: 0,
        };
        let notes = BTreeSet::from([NoteUse {
            instrument,
            key: 60,
            velocity: 100,
        }]);
        Prepared {
            source_hash: crate::assets::hash(&source),
            coverage: Coverage {
                note_on_events: 1,
                regions: BTreeMap::from([(id, notes.clone())]),
                samples: BTreeSet::from([0]),
                missing: BTreeSet::new(),
                peak_layers_per_note: 1,
                matching_operations: 1,
            },
            regions: vec![Region {
                id,
                instrument,
                notes,
                voice,
            }],
            pcm: BTreeMap::from([(
                0,
                crate::instrument_samples::SamplePcm {
                    rate: 11025,
                    samples: (0..560).map(|i| (i as f32 * 0.15).sin() * 0.3).collect(),
                },
            )]),
        }
    }

    #[test]
    fn library_release_limit_is_explicit_reports_changes_and_preserves_source() {
        let mut prepared = prepared_fixture();
        prepared.regions[0].voice.volume_envelope.release.0 = 1200;
        let cancel = AtomicBool::new(false);
        let recipe = Recipe::default();
        let preserved = cook(&prepared, &recipe, &cancel).unwrap();
        let mut limited_recipe = recipe.clone();
        limited_recipe.preset = Preset::Custom;
        limited_recipe.maximum_release_ms = Some(500);
        let limited = cook(&prepared, &limited_recipe, &cancel).unwrap();
        assert_eq!(prepared.regions[0].voice.volume_envelope.release.0, 1200);
        assert_eq!(preserved.zones[0].voice.volume_envelope.release.0, 1200);
        assert_eq!(limited.zones[0].voice.volume_envelope.release.0, -1200);
        assert_eq!(limited.samples[0].bytes, preserved.samples[0].bytes);
        assert_eq!(limited.report.note_events, preserved.report.note_events);
        assert!(
            limited
                .report
                .adaptations
                .iter()
                .any(|a| a.code == "release_limit")
        );
        let constant = crate::instrument_voice::ModSource {
            input: crate::instrument_voice::Input::Constant,
            reversed: false,
            bipolar: false,
            curve: crate::instrument_voice::Curve::Linear,
        };
        let modulation = crate::instrument_voice::Modulation {
            source: constant,
            amount_source: constant,
            destination: Destination::VolEnvReleaseCents,
            amount: 100,
            absolute: false,
        };
        prepared.regions[0].voice.modulations.push(modulation);
        assert!(
            cook(&prepared, &limited_recipe, &cancel)
                .err()
                .unwrap()
                .contains("modulates release")
        );
        limited_recipe.maximum_release_ms = Some(0);
        assert!(limited_recipe.validate().is_err());
        limited_recipe.select_preset(Preset::Balanced);
        assert_eq!(limited_recipe.maximum_release_ms, None);
    }

    #[test]
    fn library_cook_preserves_release_tail_layers_and_source_and_is_deterministic() {
        let mut prepared = prepared_fixture();
        let first = &prepared.regions[0];
        let mut layered = first.voice.clone();
        layered.pan_permille = 100;
        prepared.regions.push(Region {
            id: first.id,
            instrument: first.instrument,
            notes: first.notes.clone(),
            voice: layered,
        });
        let before = serde_json::to_vec(&prepared.regions[0].voice).unwrap();
        let pcm_before = prepared.pcm[&0].samples.clone();
        let recipe = Recipe::default();
        let cancel = AtomicBool::new(false);
        let cooked = cook(&prepared, &recipe, &cancel).unwrap();
        let again = cook(&prepared, &recipe, &cancel).unwrap();
        assert_eq!(
            cooked.samples.len(),
            1,
            "compatible sample data deduplicates, independent layers survive"
        );
        assert_eq!(cooked.zones.len(), 2);
        assert_eq!(
            serde_json::to_vec(&cooked.report).unwrap(),
            serde_json::to_vec(&again.report).unwrap()
        );
        assert_eq!(cooked.samples[0].bytes, again.samples[0].bytes);
        let sample = &cooked.samples[0];
        assert_eq!(sample.rate, 11025, "a maximum quality rate never upsamples");
        assert_eq!(sample.loop_region, Some([84, 252]));
        assert_eq!(sample.frames, 588);
        assert_eq!(
            sample.bytes[(84 / 28) * 16 + 1] & 4,
            0,
            "early release cannot be overwritten by a loop-start flag"
        );
        assert_eq!(
            sample.bytes[(252 / 28 - 1) * 16 + 1],
            3,
            "loop end has end/repeat flags"
        );
        let decoded = decode_contiguous(&sample.bytes, sample.frames).unwrap();
        assert_eq!(decoded.len(), 588);
        assert!(
            decoded[252..].iter().any(|v| *v != 0),
            "release tail is present after the loop end"
        );
        assert_eq!(
            before,
            serde_json::to_vec(&prepared.regions[0].voice).unwrap()
        );
        assert_eq!(prepared.pcm[&0].samples, pcm_before);
        assert_eq!(cooked.report.sample_spu_bytes, sample.bytes.len());
        let key = identity(&prepared, &recipe, "song A").unwrap();
        assert_ne!(key, identity(&prepared, &recipe, "song B").unwrap());
        prepared.regions[0].voice.tune_cents = 1;
        assert_ne!(key, identity(&prepared, &recipe, "song A").unwrap());
        prepared.regions[0].voice.tune_cents = 0;
        let mut changed_recipe = recipe.clone();
        changed_recipe.select_preset(Preset::High);
        assert_ne!(key, identity(&prepared, &changed_recipe, "song A").unwrap());
        prepared.source_hash.push('0');
        assert_ne!(key, identity(&prepared, &recipe, "song A").unwrap());
    }

    #[test]
    fn library_cook_reports_alignment_and_overbudget_and_rejects_unsupported_or_cancelled() {
        let mut prepared = prepared_fixture();
        prepared.regions[0].voice.loop_start = 57;
        let mut recipe = Recipe::default();
        recipe.bank_budget_bytes = 64;
        recipe.other_resident_bytes = crate::audio_import::SPU_BUDGET as u32 - 128;
        let cancel = AtomicBool::new(false);
        let cooked = cook(&prepared, &recipe, &cancel).unwrap();
        assert!(!cooked.report.fits_sample_budget);
        assert_eq!(cooked.report.available_bank_bytes, 64);
        assert!(
            cooked
                .report
                .adaptations
                .iter()
                .any(|a| a.code == "loop_block_alignment")
        );
        recipe.preset = Preset::Custom;
        recipe.sample_loops = SampleLoops::ExactBlocks;
        assert!(
            cook(&prepared, &recipe, &cancel)
                .err()
                .unwrap()
                .contains("block alignment")
        );
        recipe.sample_loops = SampleLoops::AlignOutward;
        prepared.regions[0].voice.modulation_lfo.filter_cents = 1;
        assert!(
            cook(&prepared, &recipe, &cancel)
                .err()
                .unwrap()
                .contains("animated LFO filter")
        );
        assert!(
            cook(&prepared, &recipe, &AtomicBool::new(true))
                .err()
                .unwrap()
                .contains("cancelled")
        );
    }

    #[test]
    fn library_full_selection_preserves_ranges_and_validates_unused_aliases() {
        let source = crate::sf2::fixture();
        let sequence = crate::midi::parse(&crate::midi::fixture()).unwrap();
        let alias = Mapping {
            source: Instrument {
                bank: 1,
                program: 42,
                percussion: false,
            },
            library_bank: 0,
            library_program: 0,
        };
        let full = crate::psx_music_settings::Selection::FullLibrary;
        let cancel = AtomicBool::new(false);
        let prepared =
            prepare_selection(&source, &sequence, &[alias.clone()], full, &cancel).unwrap();
        assert_eq!(
            prepared.regions.len(),
            2,
            "canonical identity and explicit free alias coexist"
        );
        assert_eq!(prepared.pcm.len(), 1);
        assert!(
            prepared
                .regions
                .iter()
                .all(|r| r.voice.key_range == [0, 127] && r.voice.velocity_range == [0, 127])
        );
        let mut missing = alias;
        missing.library_program = 1;
        assert!(
            prepare_selection(&source, &sequence, &[missing], full, &cancel)
                .err()
                .unwrap()
                .contains("missing library preset")
        );
        let mut recipe = Recipe::default();
        recipe.selection = full;
        let cooked = cook(&prepared, &recipe, &cancel).unwrap();
        assert_eq!(cooked.zones.len(), 2);
        assert_eq!(cooked.samples.len(), 1);
        crate::psx_library_wire::encode(&cooked).unwrap();
        let mut library = crate::sf2::parse(&source).unwrap();
        let mut second = library.presets[0].clone();
        second.bank = 1;
        library.presets.push(second);
        let mut override_mapping = Mapping {
            source: Instrument {
                bank: 1,
                program: 0,
                percussion: false,
            },
            library_bank: 0,
            library_program: 0,
        };
        assert!(
            validate_full_mappings(&library, &[override_mapping.clone()])
                .unwrap_err()
                .contains("conflicts with a canonical")
        );
        override_mapping.source.percussion = true;
        validate_full_mappings(&library, &[override_mapping]).unwrap();
    }

    #[test]
    #[ignore = "External hash-pinned Ironwood MIDI and reference library; set EPOK_MIDI_SOURCE, EPOK_SOUNDFONT_LIBRARY, EPOK_P3_REPORT_DIR"]
    fn library_cook_ironwood_presets_measure_actual_encoded_costs() {
        let midi =
            std::fs::read(std::env::var_os("EPOK_MIDI_SOURCE").expect("EPOK_MIDI_SOURCE")).unwrap();
        let bank = std::fs::read(
            std::env::var_os("EPOK_SOUNDFONT_LIBRARY").expect("EPOK_SOUNDFONT_LIBRARY"),
        )
        .unwrap();
        assert_eq!(
            crate::assets::hash(&midi),
            "5709b0b1b24d93b7c83199b168d573f1e915add144b340633502da2e4b299efe"
        );
        assert_eq!(
            crate::assets::hash(&bank),
            "cda013d8c370a48ae8dad271e761078d2e77455488dabdedbfbe5fc76a38c682"
        );
        let output = std::path::PathBuf::from(
            std::env::var_os("EPOK_P3_REPORT_DIR").expect("EPOK_P3_REPORT_DIR"),
        );
        std::fs::create_dir_all(&output).unwrap();
        let sequence = crate::midi::parse(&midi).unwrap();
        assert!(sequence.diagnostics.iter().all(|d| !d.unsupported));
        let cancel = AtomicBool::new(false);
        let began = std::time::Instant::now();
        let prepared = prepare(&bank, &sequence, &[], &cancel).unwrap();
        let prepare_ms = began.elapsed().as_secs_f64() * 1000.;
        let mut timings = Vec::new();
        for preset in [Preset::Compact, Preset::Balanced, Preset::High] {
            let mut recipe = Recipe::default();
            recipe.select_preset(preset);
            let began = std::time::Instant::now();
            let cooked = cook(&prepared, &recipe, &cancel).unwrap();
            let cook_ms = began.elapsed().as_secs_f64() * 1000.;
            assert_eq!(cooked.report.note_events, 1508);
            assert_eq!(cooked.report.regions, 55);
            assert!(
                cooked.report.samples >= 52,
                "no source sample was discarded"
            );
            assert!(
                cooked
                    .samples
                    .iter()
                    .all(|s| s.rate <= recipe.max_sample_rate)
            );
            let payload = crate::psx_library_wire::encode(&cooked).unwrap();
            assert_eq!(payload.len(), cooked.report.accounting.bank_main_ram_bytes);
            std::fs::write(output.join(format!("{preset:?}.epsb")), &payload).unwrap();
            for (index, sample) in cooked.samples.iter().enumerate() {
                std::fs::write(
                    output.join(format!("{preset:?}-{index:03}.adpcm")),
                    &sample.bytes,
                )
                .unwrap();
            }
            std::fs::write(
                output.join(format!("{preset:?}.json")),
                serde_json::to_vec_pretty(&cooked.report).unwrap(),
            )
            .unwrap();
            timings.push(serde_json::json!({"preset":preset, "cook_milliseconds":cook_ms,
                "sample_spu_bytes":cooked.report.sample_spu_bytes, "fits":cooked.report.fits_sample_budget}));
            println!(
                "{preset:?}: {} encoded bytes, fits {}, {cook_ms:.3} ms, max loop step {:.6}",
                cooked.report.sample_spu_bytes,
                cooked.report.fits_sample_budget,
                cooked.report.maximum_loop_step
            );
        }
        if std::env::var_os("EPOK_P3_OPTIMIZE").is_some() {
            // Read-only inventory of all resident AudioClips is a conservative
            // project reservation. P6 separately measures the actual linked set.
            let project = std::path::PathBuf::from(
                std::env::var_os("EPOK_IRONWOOD_ROOT").expect("EPOK_IRONWOOD_ROOT"),
            );
            let index = crate::assets::scan(&project, &mut Default::default());
            let mut resident_bytes = 0;
            let mut resident_clips = Vec::new();
            for (&id, records) in &index.assets {
                if records.len() != 1 || records[0].meta.kind != crate::assets::Kind::AudioClip {
                    continue;
                }
                let package = crate::assets::Package::load(&records[0].path).unwrap();
                let settings = package.meta.settings.audio().unwrap();
                if settings.is_streamed() {
                    continue;
                }
                let size = crate::audio_import::convert(&package.source, settings)
                    .unwrap()
                    .adpcm
                    .len()
                    .div_ceil(64)
                    * 64;
                resident_bytes += size;
                resident_clips.push(serde_json::json!({"id":id,"bytes":size}));
            }
            let mut recipe = Recipe::default();
            recipe.select_preset(Preset::Compact);
            recipe.other_resident_bytes = resident_bytes as u32;
            let began = std::time::Instant::now();
            let proposal =
                crate::psx_music_optimizer::propose(&prepared, &recipe, &cancel).unwrap();
            let optimize_ms = began.elapsed().as_secs_f64() * 1000.;
            std::fs::write(output.join("optimization.json"), serde_json::to_vec_pretty(&serde_json::json!({
                "proposal":proposal,"optimize_milliseconds":optimize_ms,"resident_audio_clips":resident_clips,
                "scope":"all resident AudioClip assets; conservative reservation, not an actual linked build measurement"})).unwrap()).unwrap();
            if let Some(mut proposed) = proposal.proposed_recipe {
                let cooked = cook(&prepared, &proposed, &cancel).unwrap();
                assert!(cooked.report.fits_sample_budget);
                let payload = crate::psx_library_wire::encode(&cooked).unwrap();
                std::fs::write(output.join("Optimized.epsb"), payload).unwrap();
                println!(
                    "Optimized: {} Hz, {} samples bytes + {resident_bytes} other resident bytes, {} candidates in {optimize_ms:.3} ms",
                    proposed.max_sample_rate,
                    cooked.report.sample_spu_bytes,
                    proposal.candidates.len()
                );
                proposed.loop_crossfade_frames = 128;
                let adjusted = cook(&prepared, &proposed, &cancel).unwrap();
                std::fs::write(
                    output.join("Crossfade.json"),
                    serde_json::to_vec_pretty(&adjusted.report).unwrap(),
                )
                .unwrap();
                std::fs::write(
                    output.join("Crossfade.epsb"),
                    crate::psx_library_wire::encode(&adjusted).unwrap(),
                )
                .unwrap();
                println!(
                    "Explicit 128-frame crossfade: max step {:.6} -> {:.6}",
                    cooked.report.maximum_loop_step, adjusted.report.maximum_loop_step
                );
            } else {
                println!(
                    "Optimization has no fitting proposal: {:?}",
                    proposal.reason
                );
            }
        }
        std::fs::write(output.join("timings.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "prepare_milliseconds":prepare_ms, "presets":timings, "scope":"host debug conversion; no runtime cost inferred"})).unwrap()).unwrap();
    }
}
