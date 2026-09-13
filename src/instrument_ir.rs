//! Neutral, source-preserving SoundFont library definitions.
use crate::sf2::SourceFormat;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ops::Range};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportDiagnostic {
    pub code: String,
    pub message: String,
}

/// The source INFO/ifil version. It remains part of the neutral library so
/// versioned SoundFont compatibility rules are reproducible after import.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionIr {
    pub major: u16,
    pub minor: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generator {
    pub operator: u16,
    /// Signed generator amount. `raw_amount` preserves range/ID bit patterns.
    pub amount: i16,
}

impl Generator {
    pub fn raw_amount(&self) -> u16 {
        self.amount as u16
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModulatorIr {
    pub source: u16,
    pub destination: u16,
    pub amount: i16,
    pub amount_source: u16,
    pub transform: u16,
}

/// A modulator after SoundFont hierarchy resolution. The signed 32-bit amount
/// can represent the sum of identical preset- and instrument-level records.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectiveModulatorIr {
    pub source: u16,
    pub destination: u16,
    pub amount: i32,
    pub amount_source: u16,
    pub transform: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneIr {
    pub bag_index: usize,
    pub global: bool,
    pub generators: Vec<Generator>,
    pub modulators: Vec<ModulatorIr>,
}

/// A raw Hydra definition. Its zones retain global/local records exactly enough
/// to reproduce the SF2 generator-stack semantics in a later synthesis phase.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    pub name: String,
    pub zones: Vec<ZoneIr>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SampleIr {
    pub name: String,
    /// Native SoundFont header offsets. In SF2 these are PCM frame offsets in
    /// `smpl`. In SF3, samples with type bit `0x10` use compressed byte
    /// offsets; uncompressed samples retain the SF2 frame-offset meaning.
    pub start: u32,
    pub end: u32,
    pub loop_start: u32,
    pub loop_end: u32,
    pub sample_rate: u32,
    pub root_key: u8,
    pub pitch_correction: i8,
    pub link: u16,
    pub sample_type: u16,
    /// Absolute byte range in the original source. SF2 points at PCM16 frames;
    /// compressed SF3 points at one encoded stream. Ranges are half-open even
    /// though native SHDR fields remain available above.
    pub data_range: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionIr {
    pub key_range: [u8; 2],
    pub velocity_range: [u8; 2],
    pub sample: u16,
    /// Instrument values are resolved over the SoundFont defaults, then
    /// preset offsets are added and known generator ranges are clamped.
    /// Missing entries have the SoundFont default value zero.
    pub effective_generators: BTreeMap<u16, i32>,
    /// Raw source modulators in preset-global, preset-local,
    /// instrument-global, instrument-local order. These remain available for
    /// source inspection; use `effective_modulators` for playback semantics.
    pub modulators: Vec<ModulatorIr>,
    /// Default instrument modulators with instrument overrides applied, plus
    /// the effective preset modulators using SoundFont identity/addition rules.
    pub effective_modulators: Vec<EffectiveModulatorIr>,
    pub preset_zone: usize,
    pub instrument_zone: usize,
    pub exclusive_class: u16,
    /// Necessary source semantics that this region retains but cannot safely
    /// execute. Selection can reject a used region without blocking unrelated
    /// presets in the same library.
    pub blockers: Vec<ImportDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetIr {
    pub bank: u16,
    pub program: u8,
    pub name: String,
    pub regions: Vec<RegionIr>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawChunk {
    pub id: [u8; 4],
    pub data_range: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryIr {
    pub format: SourceFormat,
    pub version: VersionIr,
    pub presets: Vec<PresetIr>,
    pub samples: Vec<SampleIr>,
    pub sample_data: Range<usize>,
    /// Optional SF2 `sm24` extension retained as an absolute source range. It
    /// is not silently combined with `smpl` by this PCM16-only phase.
    pub sm24_data: Option<Range<usize>>,
    pub raw_presets: Vec<Definition>,
    pub raw_instruments: Vec<Definition>,
    pub unknown_chunks: Vec<RawChunk>,
    pub diagnostics: Vec<ImportDiagnostic>,
    /// Source-wide blockers. Region-specific blockers live on `RegionIr` so an
    /// unused preset cannot make an otherwise usable library unselectable.
    pub blockers: Vec<ImportDiagnostic>,
}
