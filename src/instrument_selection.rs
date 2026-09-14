//! Reachable instrument regions, without target sample conversion or asset publication.
use crate::{
    instrument_ir::LibraryIr,
    sequence_ir::{EventKind, SequenceIr},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, Ordering},
};

pub const MAX_REGION_MATCH_OPERATIONS: usize = 8_000_000;
pub const MAX_INSTRUMENT_MAPPINGS: usize = 2048;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Instrument {
    pub bank: u16,
    pub program: u8,
    pub percussion: bool,
}

/// Explicit authoring override; the source's identity remains in the coverage report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mapping {
    pub source: Instrument,
    pub library_bank: u16,
    pub library_program: u8,
}

pub fn validate_mappings(mappings: &[Mapping]) -> Result<(), String> {
    if mappings.len() > MAX_INSTRUMENT_MAPPINGS {
        return Err("At most 2048 instrument mappings are supported".into());
    }
    let mut identities = BTreeSet::new();
    for mapping in mappings {
        if mapping.source.bank > 16383
            || mapping.source.program > 127
            || mapping.library_bank > 16383
            || mapping.library_program > 127
            || !identities.insert(mapping.source)
        {
            return Err(
                "Instrument mappings require valid, unique MIDI bank/program identities".into(),
            );
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RegionId {
    pub preset: usize,
    pub region: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct NoteUse {
    pub instrument: Instrument,
    pub key: u8,
    pub velocity: u8,
}

#[derive(Clone, Debug, Serialize)]
pub struct Coverage {
    pub note_on_events: usize,
    /// Every matching layer is retained, including stereo and velocity layers.
    #[serde(serialize_with = "serialize_regions")]
    pub regions: BTreeMap<RegionId, BTreeSet<NoteUse>>,
    pub samples: BTreeSet<u16>,
    pub missing: BTreeSet<NoteUse>,
    pub peak_layers_per_note: usize,
    pub matching_operations: usize,
}

fn serialize_regions<S: serde::Serializer>(
    regions: &BTreeMap<RegionId, BTreeSet<NoteUse>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    let mut sequence = serializer.serialize_seq(Some(regions.len()))?;
    for entry in regions {
        sequence.serialize_element(&entry)?;
    }
    sequence.end()
}

impl Coverage {
    pub fn require_complete(&self) -> Result<(), String> {
        if self.missing.is_empty() {
            return Ok(());
        }
        Err(format!(
            "Instrument library is missing {} note/velocity mappings: {}. Select a library containing them or add an explicit instrument mapping.",
            self.missing.len(),
            self.missing
                .iter()
                .take(16)
                .map(|n| format!(
                    "{} bank MSB {}/LSB {}, program {} (0-based), key {}, velocity {}",
                    if n.instrument.percussion {
                        "percussion"
                    } else {
                        "melodic"
                    },
                    n.instrument.bank >> 7,
                    n.instrument.bank & 127,
                    n.instrument.program,
                    n.key,
                    n.velocity
                ))
                .collect::<Vec<_>>()
                .join("; ")
        ))
    }
}

pub fn resolve(
    song: &SequenceIr,
    library: &LibraryIr,
    overrides: &[Mapping],
    cancelled: &AtomicBool,
) -> Result<Coverage, String> {
    validate_mappings(overrides)?;
    let mut remap = BTreeMap::new();
    for mapping in overrides {
        remap.insert(
            mapping.source,
            (mapping.library_bank, mapping.library_program),
        );
    }
    let mut presets = BTreeMap::new();
    for (index, preset) in library.presets.iter().enumerate() {
        if presets
            .insert((preset.bank, preset.program), index)
            .is_some()
        {
            return Err(format!(
                "Ambiguous library bank {}, program {}",
                preset.bank, preset.program
            ));
        }
    }
    let mut report = Coverage {
        note_on_events: 0,
        regions: BTreeMap::new(),
        samples: BTreeSet::new(),
        missing: BTreeSet::new(),
        peak_layers_per_note: 0,
        matching_operations: 0,
    };
    let mut banks = [0; 16];
    let mut programs = [0; 16];
    let mut visited = BTreeSet::new();
    for event in &song.events {
        if cancelled.load(Ordering::Relaxed) {
            return Err("Instrument analysis cancelled".into());
        }
        match event.kind {
            EventKind::Program { channel, program } => {
                banks[channel as usize] = 0;
                programs[channel as usize] = program;
            }
            EventKind::BankProgram {
                channel,
                bank,
                program,
            } => {
                banks[channel as usize] = bank;
                programs[channel as usize] = program;
            }
            EventKind::NoteOn {
                channel,
                key,
                velocity,
            } => {
                report.note_on_events += 1;
                let instrument = Instrument {
                    bank: banks[channel as usize],
                    program: programs[channel as usize],
                    percussion: channel == 9,
                };
                let note = NoteUse {
                    instrument,
                    key,
                    velocity,
                };
                if !visited.insert(note) {
                    continue;
                }
                // SF2 defines bank 128 as the standard percussion bank. Arbitrary MIDI
                // percussion-bank extensions need an explicit mapping, never a bank-0 fallback.
                let selected = remap.get(&instrument).copied().or_else(|| {
                    if !instrument.percussion {
                        Some((instrument.bank, instrument.program))
                    } else if instrument.bank == 0 {
                        Some((128, instrument.program))
                    } else {
                        None
                    }
                });
                let Some(preset_index) = selected.and_then(|id| presets.get(&id)).copied() else {
                    report.missing.insert(note);
                    continue;
                };
                let preset = &library.presets[preset_index];
                report.matching_operations = report
                    .matching_operations
                    .checked_add(preset.regions.len())
                    .ok_or("Instrument region matching counter overflow")?;
                if report.matching_operations > MAX_REGION_MATCH_OPERATIONS {
                    return Err(
                        "Instrument matching exceeds the 8,000,000-region analysis budget".into(),
                    );
                }
                let mut layers = 0;
                for (region_index, region) in preset.regions.iter().enumerate() {
                    if (region.key_range[0]..=region.key_range[1]).contains(&key)
                        && (region.velocity_range[0]..=region.velocity_range[1]).contains(&velocity)
                    {
                        layers += 1;
                        report
                            .regions
                            .entry(RegionId {
                                preset: preset_index,
                                region: region_index,
                            })
                            .or_default()
                            .insert(note);
                        report.samples.insert(region.sample);
                    }
                }
                report.peak_layers_per_note = report.peak_layers_per_note.max(layers);
                if layers == 0 {
                    report.missing.insert(note);
                }
            }
            _ => {}
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequence_ir::{Event, EventKind};

    fn song(kinds: Vec<EventKind>) -> SequenceIr {
        SequenceIr::analyze(
            480,
            kinds
                .into_iter()
                .enumerate()
                .map(|(i, kind)| Event {
                    tick: i as u32,
                    track: 0,
                    order: i as u32,
                    kind,
                })
                .collect(),
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn soundfont_selection_preserves_layers_and_uses_exact_bank_and_drum_identity() {
        let mut library = crate::sf2::parse(&crate::sf2::fixture()).unwrap();
        let original = library.presets[0].regions[0].clone();
        let mut second = original.clone();
        second.velocity_range = [80, 127];
        library.presets[0].regions.push(second);
        let mut drums = library.presets[0].clone();
        drums.bank = 128;
        drums.regions = vec![original];
        drums.regions[0].key_range = [87, 87];
        library.presets.push(drums);
        let sequence = song(vec![
            EventKind::NoteOn {
                channel: 0,
                key: 60,
                velocity: 95,
            },
            EventKind::NoteOn {
                channel: 0,
                key: 60,
                velocity: 95,
            },
            EventKind::BankProgram {
                channel: 0,
                bank: 130,
                program: 0,
            },
            EventKind::NoteOn {
                channel: 0,
                key: 60,
                velocity: 95,
            },
            EventKind::NoteOn {
                channel: 9,
                key: 87,
                velocity: 95,
            },
            EventKind::NoteOn {
                channel: 9,
                key: 84,
                velocity: 95,
            },
            EventKind::Program {
                channel: 0,
                program: 0,
            },
            EventKind::NoteOn {
                channel: 0,
                key: 60,
                velocity: 76,
            },
        ]);
        let coverage = resolve(&sequence, &library, &[], &AtomicBool::new(false)).unwrap();
        assert_eq!(coverage.note_on_events, 6);
        assert_eq!(coverage.peak_layers_per_note, 2);
        assert_eq!(coverage.regions.len(), 3);
        assert_eq!(
            coverage.samples.len(),
            1,
            "shared PCM does not eliminate independent voice layers"
        );
        assert_eq!(coverage.missing.len(), 2);
        let error = coverage.require_complete().unwrap_err();
        assert!(
            error.contains("bank MSB 1/LSB 2")
                && error.contains("percussion")
                && error.contains("key 84")
        );
        assert_eq!(
            coverage.regions[&RegionId {
                preset: 0,
                region: 0
            }]
                .len(),
            2
        );
        assert_eq!(
            coverage.regions[&RegionId {
                preset: 0,
                region: 1
            }]
                .len(),
            1
        );
        assert!(
            serde_json::to_string(&coverage)
                .unwrap()
                .contains("note_on_events")
        );
        let mapping = Mapping {
            source: Instrument {
                bank: 130,
                program: 0,
                percussion: false,
            },
            library_bank: 0,
            library_program: 0,
        };
        let remapped = resolve(
            &sequence,
            &library,
            &[mapping.clone()],
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(
            remapped.missing.len(),
            1,
            "only the explicitly mapped bank may change"
        );
        assert!(
            resolve(
                &sequence,
                &library,
                &[mapping.clone(), mapping],
                &AtomicBool::new(false)
            )
            .is_err()
        );
        assert!(
            resolve(&sequence, &library, &[], &AtomicBool::new(true))
                .unwrap_err()
                .contains("cancelled")
        );
    }

    #[test]
    fn soundfont_library_coverage_never_falls_back_to_program_zero() {
        let library = crate::sf2::parse(&crate::sf2::fixture()).unwrap();
        let sequence = song(vec![
            EventKind::Program {
                channel: 0,
                program: 71,
            },
            EventKind::NoteOn {
                channel: 0,
                key: 60,
                velocity: 95,
            },
        ]);
        let coverage = resolve(&sequence, &library, &[], &AtomicBool::new(false)).unwrap();
        assert!(coverage.regions.is_empty());
        assert!(
            coverage
                .require_complete()
                .unwrap_err()
                .contains("program 71")
        );
    }

    #[test]
    #[ignore = "External, hash-pinned Ironwood MIDI and FluidR3Mono source; set EPOK_MIDI_SOURCE and EPOK_SOUNDFONT_LIBRARY"]
    fn soundfont_ironwood_all_notes_layers_and_voice_semantics_are_resolved() {
        let source =
            std::fs::read(std::env::var_os("EPOK_MIDI_SOURCE").expect("EPOK_MIDI_SOURCE")).unwrap();
        let bank = std::fs::read(
            std::env::var_os("EPOK_SOUNDFONT_LIBRARY").expect("EPOK_SOUNDFONT_LIBRARY"),
        )
        .unwrap();
        assert_eq!(
            crate::assets::hash(&source),
            "5709b0b1b24d93b7c83199b168d573f1e915add144b340633502da2e4b299efe"
        );
        assert_eq!(
            crate::assets::hash(&bank),
            "cda013d8c370a48ae8dad271e761078d2e77455488dabdedbfbe5fc76a38c682"
        );
        let began = std::time::Instant::now();
        let library = crate::sf2::parse(&bank).unwrap();
        let sequence = crate::midi::parse(&source).unwrap();
        let coverage = resolve(&sequence, &library, &[], &AtomicBool::new(false)).unwrap();
        coverage.require_complete().unwrap();
        assert_eq!(coverage.note_on_events, 1508);
        assert_eq!(coverage.samples.len(), 52);
        let mut melodic = BTreeSet::new();
        let mut drums = BTreeSet::new();
        let mut voices = Vec::new();
        for (id, notes) in &coverage.regions {
            for note in notes {
                if note.instrument.percussion {
                    drums.insert(note.key);
                } else {
                    melodic.insert(note.instrument.program);
                }
            }
            let region = &library.presets[id.preset].regions[id.region];
            let voice =
                crate::instrument_voice::from_soundfont(&library, region).unwrap_or_else(|error| {
                    panic!(
                        "preset {} region {} sample {}: {error}",
                        id.preset, id.region, region.sample
                    )
                });
            voices.push((id, voice));
        }
        assert_eq!(melodic, BTreeSet::from([44, 52, 70, 71, 75, 104, 105]));
        assert_eq!(drums, BTreeSet::from([52, 70, 73, 74, 84, 86, 87]));
        let analysis_ms = began.elapsed().as_secs_f64() * 1000.;
        let decode_began = std::time::Instant::now();
        let pcm = crate::instrument_samples::decode_selected(
            &bank,
            &library,
            &coverage,
            &AtomicBool::new(false),
        )
        .unwrap();
        let decode_ms = decode_began.elapsed().as_secs_f64() * 1000.;
        let oracle_dir =
            std::path::PathBuf::from(std::env::var_os("EPOK_SOUNDFONT_ORACLE_DIR").expect(
                "EPOK_SOUNDFONT_ORACLE_DIR: complete independent libsndfile float32 samples",
            ));
        let mut max_error = 0_f32;
        for (id, decoded) in &pcm {
            let reference =
                std::fs::read(oracle_dir.join(format!("sample-{id:04}.f32le"))).unwrap();
            assert_eq!(
                reference.len(),
                decoded.samples.len() * 4,
                "sample {id} frame count"
            );
            for (&value, expected) in decoded.samples.iter().zip(reference.chunks_exact(4)) {
                let expected = f32::from_le_bytes(expected.try_into().unwrap());
                assert!(expected.is_finite() && value.is_finite());
                max_error = max_error.max((value - expected).abs());
            }
        }
        assert!(
            max_error <= 1. / 32768.,
            "independent Vorbis decoders differ by more than one PCM16 LSB: {max_error}"
        );
        let frames = pcm
            .values()
            .map(|sample| sample.samples.len())
            .sum::<usize>();
        let report = serde_json::json!({"coverage": coverage, "voices": voices,
            "analysis_milliseconds": analysis_ms, "decode_milliseconds": decode_ms,
            "source_pcm_frames": frames, "source_pcm_float32_bytes": frames * 4,
            "independent_pcm_max_abs_difference": max_error,
            "source_pcm_peak": pcm.values().flat_map(|sample| &sample.samples).fold(0_f32, |peak, sample| peak.max(sample.abs())),
            "source_bytes": bank.len(), "library_presets": library.presets.len(),
            "library_samples": library.samples.len(), "scope": "source analysis; no PSX RAM or runtime cost inferred"});
        if let Some(path) = std::env::var_os("EPOK_INSTRUMENT_REPORT") {
            std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
        println!(
            "Ironwood: 1508 note events, 14 instrument/drum assignments, {} regions, 52 samples, {} maximum layers; all source voice semantics resolved",
            coverage.regions.len(),
            coverage.peak_layers_per_note
        );
    }
}
