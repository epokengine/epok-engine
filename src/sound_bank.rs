//! Portable instrument mappings. Samples reference original AudioClip snapshots by UUID.
use crate::{assets, audio_import::LoadMode, sequence_ir::SequenceIr};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use uuid::Uuid;

pub const MAX_ZONES: usize = 128;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Envelope {
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub sustain: f32,
    pub release_ms: f32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack_ms: 5.,
            decay_ms: 100.,
            sustain: 0.8,
            release_ms: 120.,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub sample: Uuid,
    pub key_range: [u8; 2],
    pub velocity_range: [u8; 2],
    pub root_key: u8,
    pub fine_tune_cents: f32,
    pub gain: f32,
    /// -1 left, 0 center, +1 right.
    pub pan: f32,
    pub envelope: Envelope,
    /// Original source sample frames, half-open. Independent of target block alignment.
    pub sample_loop: Option<[u32; 2]>,
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
impl Zone {
    pub fn new(sample: Uuid) -> Self {
        Self {
            sample,
            key_range: [0, 127],
            velocity_range: [1, 127],
            root_key: 60,
            fine_tune_cents: 0.,
            gain: 1.,
            pan: 0.,
            envelope: Envelope::default(),
            sample_loop: None,
            extra: BTreeMap::new(),
        }
    }
    pub fn matches(&self, key: u8, velocity: u8) -> bool {
        (self.key_range[0]..=self.key_range[1]).contains(&key)
            && (self.velocity_range[0]..=self.velocity_range[1]).contains(&velocity)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub program: u8,
    pub drum_key: Option<u8>,
    pub zones: Vec<Zone>,
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub schema_version: u32,
    /// None is the original portable mapping schema. Imported definitions keep
    /// unresolved source tones separate from the resolved playback mappings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported: Option<crate::bank_compat::Definition>,
    /// An authoritative SF2/SF3 snapshot. Its catalog is parsed from source,
    /// never expanded into one AudioClip asset per embedded sample.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library: Option<crate::soundfont_asset::Definition>,
    pub load_mode: LoadMode,
    pub programs: Vec<Program>,
    /// Human-readable origin/license of the mapping and samples. No bundled bank is assumed.
    pub provenance: String,
    pub target_overrides: BTreeMap<String, serde_json::Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
    #[serde(skip)]
    pub envelope_extra: BTreeMap<String, serde_json::Value>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            imported: None,
            library: None,
            load_mode: LoadMode::Resident,
            programs: vec![],
            provenance: String::new(),
            target_overrides: BTreeMap::new(),
            extra: BTreeMap::new(),
            envelope_extra: BTreeMap::new(),
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 && !(self.schema_version == 2 && self.library.is_some()) {
            return Err("Unsupported SoundBank settings version".into());
        }
        if self.schema_version == 1 && self.library.is_some() {
            return Err("SoundFont SoundBanks require settings schema version 2".into());
        }
        if let Some(imported) = &self.imported {
            imported.validate()?;
            if !self.programs.is_empty() {
                return Err("Imported SoundBank source tones cannot be replaced by native zones. Create a separate portable mapping after resolving the source semantics.".into());
            }
        }
        if let Some(library) = &self.library {
            library.validate()?;
            if self.imported.is_some() || !self.programs.is_empty() {
                return Err("SoundFont source snapshots cannot be combined with Sony imports or portable zones. Resolve the library into a separate target bank.".into());
            }
        }
        if self.programs.len() > MAX_ZONES
            || self.programs.iter().map(|p| p.zones.len()).sum::<usize>() > MAX_ZONES
        {
            return Err("SoundBank exceeds 128 zones/program mappings".into());
        }
        if self.provenance.len() > 4096 || self.target_overrides.values().any(|v| !v.is_object()) {
            return Err("Invalid SoundBank provenance/target override metadata".into());
        }
        let mut keys = BTreeSet::new();
        for p in &self.programs {
            if p.program > 127
                || p.drum_key.is_some_and(|key| key > 127)
                || p.zones.is_empty()
                || !keys.insert((p.program, p.drum_key))
            {
                return Err(
                    "Bank program/drum mappings must be unique, 0–127, and contain a zone".into(),
                );
            }
            for (i, z) in p.zones.iter().enumerate() {
                if z.sample.is_nil()
                    || z.root_key > 127
                    || z.key_range[0] > z.key_range[1]
                    || z.key_range[1] > 127
                    || z.velocity_range[0] == 0
                    || z.velocity_range[0] > z.velocity_range[1]
                    || z.velocity_range[1] > 127
                    || !(-100. ..=100.).contains(&z.fine_tune_cents)
                    || !(0. ..=4.).contains(&z.gain)
                    || !(-1. ..=1.).contains(&z.pan)
                    || !(0. ..=1.).contains(&z.envelope.sustain)
                    || [
                        z.envelope.attack_ms,
                        z.envelope.decay_ms,
                        z.envelope.release_ms,
                    ]
                    .iter()
                    .any(|v| !(0. ..=60_000.).contains(v))
                    || z.sample_loop.is_some_and(|[start, end]| start >= end)
                {
                    return Err(format!(
                        "Invalid zone {} in program {}: check sample, ranges, tuning, gain/pan, ADSR and loop",
                        i + 1,
                        p.program
                    ));
                }
                if p.zones[..i].iter().any(|other| {
                    z.key_range[0] <= other.key_range[1]
                        && other.key_range[0] <= z.key_range[1]
                        && z.velocity_range[0] <= other.velocity_range[1]
                        && other.velocity_range[0] <= z.velocity_range[1]
                }) {
                    return Err(format!(
                        "Program {} has overlapping zones; initial bank profile requires one matching zone per note",
                        p.program
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn dependencies(&self) -> BTreeSet<Uuid> {
        if self.library.is_some() {
            return BTreeSet::new();
        }
        self.programs
            .iter()
            .flat_map(|p| p.zones.iter().map(|z| z.sample))
            .collect()
    }
    pub fn zone(&self, channel: u8, program: u8, key: u8, velocity: u8) -> Option<&Zone> {
        self.programs
            .iter()
            .find(|p| p.program == program && p.drum_key == (channel == 9).then_some(key))?
            .zones
            .iter()
            .find(|zone| zone.matches(key, velocity))
    }
    pub fn validate_sequence(&self, ir: &SequenceIr, index: &assets::Index) -> Result<(), String> {
        self.validate()?;
        self.validate_playback()?;
        let mut programs = [0; 16];
        let mut banks = [0; 16];
        let mut missing = BTreeSet::new();
        for e in &ir.events {
            match e.kind {
                crate::sequence_ir::EventKind::Program { channel, program } => {
                    programs[channel as usize] = program;
                    banks[channel as usize] = 0;
                }
                crate::sequence_ir::EventKind::BankProgram {
                    channel,
                    bank,
                    program,
                } => {
                    programs[channel as usize] = program;
                    banks[channel as usize] = bank;
                }
                crate::sequence_ir::EventKind::NoteOn {
                    channel,
                    key,
                    velocity,
                } => {
                    let program = programs[channel as usize];
                    if banks[channel as usize] != 0 {
                        missing.insert(format!(
                            "bank MSB {}/LSB {}, program {program}, key {key}, velocity {velocity}; this bank provides bank 0 only",
                            banks[channel as usize] >> 7, banks[channel as usize] & 127
                        ));
                    } else if self.zone(channel, program, key, velocity).is_none() {
                        missing.insert(format!(
                            "program {program}, {}key {key}, velocity {velocity}",
                            if channel == 9 { "drum " } else { "" }
                        ));
                    }
                }
                _ => {}
            }
        }
        if !missing.is_empty() {
            return Err(format!(
                "SoundBank is missing {} instrument/zone mappings. Add: {}",
                missing.len(),
                missing.into_iter().take(16).collect::<Vec<_>>().join("; ")
            ));
        }
        for id in self.dependencies() {
            if index.resolve(id)?.meta.kind != assets::Kind::AudioClip {
                return Err(format!("SoundBank sample {id} must be an AudioClip"));
            }
        }
        Ok(())
    }
    pub fn validate_playback(&self) -> Result<(), String> {
        if self.imported.is_some() {
            return Err(crate::bank_compat::PLAYBACK_BLOCKER.into());
        }
        if self.library.is_some() {
            return Err(crate::soundfont_asset::PLAYBACK_BLOCKER.into());
        }
        Ok(())
    }
}

pub fn create(root: &Path, destination: &str, settings: Settings) -> Result<Uuid, String> {
    assets::commit(prepare_new(root, destination, settings)?)
}
pub fn prepare_new(
    root: &Path,
    destination: &str,
    settings: Settings,
) -> Result<assets::Candidate, String> {
    if settings.library.is_some() {
        return Err(
            "Use soundfont_asset::prepare to import an authoritative SF2/SF3 snapshot".into(),
        );
    }
    settings.validate()?;
    let path = assets::inside(root, destination)?;
    if path.extension().is_none_or(|e| e != "epokasset") {
        return Err("SoundBank destination must end in .epokasset".into());
    }
    let source = crate::document::to_vec(&settings).map_err(|e| e.to_string())?;
    let id = Uuid::new_v4();
    let package = assets::Package {
        meta: assets::Metadata {
            version: 2,
            id,
            kind: assets::Kind::SoundBank,
            importer_version: 1,
            source: assets::path_string(root, &path.with_extension("epokbank")),
            source_hash: assets::hash(&source),
            settings: crate::import_settings::Settings::SoundBank(settings),
            extra: Default::default(),
        },
        source,
    };
    package.bytes()?;
    Ok(assets::Candidate {
        destination: path,
        source_hash: package.meta.source_hash.clone(),
        package,
        expected: None,
        source_path: None,
    })
}

/// Two reviewable original-source assets; generated locally and never selected as a default silently.
pub fn starter_candidates(
    root: &Path,
    destination: &str,
) -> Result<Vec<assets::Candidate>, String> {
    let bank_path = assets::inside(root, destination)?;
    let stem = bank_path
        .file_stem()
        .ok_or("Invalid starter bank destination")?
        .to_string_lossy();
    let sample_path = bank_path.with_file_name(format!("{stem}-triangle.epokasset"));
    if bank_path.exists() || sample_path.exists() {
        return Err("Starter bank/sample destination already exists. Choose a new name; existing assets are preserved.".into());
    }
    let rate = 22050u32;
    let frames = 2205u32; // Exactly 44 periods of A4 at the original rate.
    let mut source = b"RIFF".to_vec();
    source.extend((36 + frames * 2).to_le_bytes());
    source.extend(b"WAVEfmt ");
    source.extend(16u32.to_le_bytes());
    source.extend(1u16.to_le_bytes());
    source.extend(1u16.to_le_bytes());
    source.extend(rate.to_le_bytes());
    source.extend((rate * 2).to_le_bytes());
    source.extend(2u16.to_le_bytes());
    source.extend(16u16.to_le_bytes());
    source.extend(b"data");
    source.extend((frames * 2).to_le_bytes());
    for frame in 0..frames {
        let phase = ((frame * 440 % rate) * 65536 / rate) as i32;
        let triangle = (phase - 32768).abs() * 2 - 32768;
        source.extend((triangle / 4).to_le_bytes()[..2].iter().copied());
    }
    let sample_id = Uuid::new_v4();
    let package = assets::Package {
        meta: assets::Metadata { version: 2, id: sample_id, kind: assets::Kind::AudioClip,
            importer_version: crate::audio_import::IMPORTER_VERSION,
            source: assets::path_string(root, &sample_path.with_extension("wav")), source_hash: assets::hash(&source),
            settings: crate::import_settings::Settings::Audio(Default::default()),
            extra: BTreeMap::from([("provenance".into(), "Epok integer triangle generator v1. Original generated sample, MIT license; no recordings or external instruments.".into())]),
        }, source,
    };
    package.bytes()?;
    let sample = assets::Candidate {
        destination: sample_path,
        source_hash: package.meta.source_hash.clone(),
        package,
        expected: None,
        source_path: None,
    };
    let mut zone = Zone::new(sample_id);
    zone.root_key = 69;
    zone.sample_loop = Some([0, frames]);
    zone.gain = 0.5;
    let bank = prepare_new(root, destination, Settings {
        programs: vec![Program { program: 0, drum_key: None, zones: vec![zone], extra: Default::default() }],
        provenance: "Epok Retro Triangle v1: original integer-generated triangle, mapping and sample licensed MIT. Program 0 only; no General MIDI or percussion fallback.".into(),
        ..Default::default()
    })?;
    Ok(vec![sample, bank])
}

pub fn publish_starter(candidates: Vec<assets::Candidate>) -> Result<Uuid, String> {
    let mut published = Vec::new();
    for candidate in candidates {
        let path = candidate.destination.display().to_string();
        match assets::commit(candidate) {
            Ok(id) => published.push((path, id)),
            Err(error) => {
                return Err(format!(
                    "Starter bank: {error}. Already published original assets retained for recovery: {}",
                    published
                        .iter()
                        .map(|(p, _)| p.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }
    published
        .last()
        .map(|(_, id)| *id)
        .ok_or("No starter bank candidates".into())
}
