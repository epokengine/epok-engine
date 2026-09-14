//! Bounded SoundFont 2/SF3 RIFF and Hydra reader. It does not decode Vorbis.
use crate::instrument_ir::{
    Definition, EffectiveModulatorIr, Generator, ImportDiagnostic, LibraryIr, ModulatorIr,
    PresetIr, RawChunk, RegionIr, SampleIr, VersionIr, ZoneIr,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

pub const MAX_SOURCE_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_RIFF_CHUNKS: usize = 4096;
pub const MAX_PRESETS: usize = 2048;
pub const MAX_INSTRUMENTS: usize = 4096;
pub const MAX_SAMPLES: usize = 8192;
pub const MAX_RAW_ZONES: usize = 65536;
pub const MAX_GEN_MOD_RECORDS: usize = 262144;
pub const MAX_EXPANDED_REGIONS: usize = 65536;
pub const MAX_EXPANDED_GEN_MOD_RECORDS: usize = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceFormat {
    Sf2Pcm16,
    Sf3Vorbis,
}

pub fn has_header(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"sfbk"
}

fn u16_at(b: &[u8], n: usize) -> Result<u16, String> {
    b.get(n..n + 2)
        .and_then(|x| x.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or("truncated SoundFont record".into())
}
fn i16_at(b: &[u8], n: usize) -> Result<i16, String> {
    Ok(u16_at(b, n)? as i16)
}
fn u32_at(b: &[u8], n: usize) -> Result<u32, String> {
    b.get(n..n + 4)
        .and_then(|x| x.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or("truncated SoundFont record".into())
}
fn name(b: &[u8]) -> String {
    String::from_utf8_lossy(&b[..b.iter().position(|&x| x == 0).unwrap_or(b.len())]).into_owned()
}
fn id(b: &[u8], n: usize) -> Result<[u8; 4], String> {
    b.get(n..n + 4)
        .and_then(|x| x.try_into().ok())
        .ok_or("truncated RIFF chunk header".into())
}

#[derive(Clone)]
struct Bag {
    generator: usize,
    modu: usize,
}
#[derive(Clone)]
struct Header {
    name: String,
    a: u16,
    b: u16,
    bag: usize,
}

fn records<'a>(
    data: &'a [u8],
    entry: usize,
    cap: usize,
    label: &str,
) -> Result<Vec<&'a [u8]>, String> {
    record_count(data, entry, cap, label)?;
    Ok(data.chunks_exact(entry).collect())
}

fn record_count(data: &[u8], entry: usize, cap: usize, label: &str) -> Result<usize, String> {
    if data.len() % entry != 0 {
        return Err(format!("{label} chunk has a partial record"));
    }
    let count = data.len() / entry;
    if count > cap {
        return Err(format!("{label} record limit exceeded"));
    }
    Ok(count)
}
fn zone(
    bags: &[Bag],
    gens: &[Generator],
    mods: &[ModulatorIr],
    index: usize,
    global: bool,
) -> Result<ZoneIr, String> {
    let a = bags.get(index).ok_or("Hydra bag index out of range")?;
    let z = bags.get(index + 1).ok_or("Hydra bag terminal missing")?;
    if a.generator > z.generator
        || z.generator > gens.len()
        || a.modu > z.modu
        || z.modu > mods.len()
    {
        return Err("Hydra generator/modulator range is invalid".into());
    }
    Ok(ZoneIr {
        bag_index: index,
        global,
        generators: gens[a.generator..z.generator].to_vec(),
        modulators: mods[a.modu..z.modu].to_vec(),
    })
}
fn amount(z: &ZoneIr, op: u16) -> Option<u16> {
    z.generators
        .iter()
        .rev()
        .find(|g| g.operator == op)
        .map(Generator::raw_amount)
}
fn range(z: &ZoneIr, op: u16) -> Option<[u8; 2]> {
    amount(z, op).map(|v| [(v & 255) as u8, (v >> 8) as u8])
}
fn combine_range(a: [u8; 2], b: [u8; 2]) -> Option<[u8; 2]> {
    let r = [a[0].max(b[0]), a[1].min(b[1])];
    (r[0] <= r[1]).then_some(r)
}
fn merge_level(global: Option<&ZoneIr>, local: &ZoneIr) -> BTreeMap<u16, i32> {
    let mut out = BTreeMap::new();
    for z in global.into_iter().chain(std::iter::once(local)) {
        for g in &z.generators {
            if !matches!(g.operator, 41 | 43 | 44 | 53) {
                out.insert(g.operator, i32::from(g.amount));
            }
        }
    }
    out
}

fn generator_default(operator: u16) -> Option<i32> {
    match operator {
        8 => Some(13_500),
        21 | 23 | 25 | 26 | 27 | 28 | 30 | 33 | 34 | 35 | 36 | 38 => Some(-12_000),
        46 | 47 | 58 => Some(-1),
        56 => Some(100),
        _ => None,
    }
}

fn generator_range(operator: u16) -> Option<(i32, i32)> {
    match operator {
        0..=4 | 12 | 45 | 50 => Some((-32_768, 32_767)),
        5..=7 | 10 | 11 => Some((-12_000, 12_000)),
        8 => Some((1_500, 13_500)),
        9 => Some((0, 960)),
        13 => Some((-960, 960)),
        15 | 16 | 29 => Some((0, 1_000)),
        17 => Some((-500, 500)),
        21 | 23 | 25 | 27 | 33 | 35 => Some((-12_000, 5_000)),
        22 | 24 => Some((-16_000, 4_500)),
        26 | 28 | 30 | 34 | 36 | 38 => Some((-12_000, 8_000)),
        31 | 32 | 39 | 40 => Some((-1_200, 1_200)),
        37 | 48 => Some((0, 1_440)),
        46 | 47 | 58 => Some((-1, 127)),
        51 => Some((-120, 120)),
        52 => Some((-99, 99)),
        54 => Some((0, 3)),
        56 => Some((0, 1_200)),
        57 => Some((0, 127)),
        _ => None,
    }
}

fn allows_zero_time_sentinel(operator: u16) -> bool {
    matches!(operator, 21 | 23 | 25 | 26 | 27 | 33 | 34 | 35)
}

fn valid_value_generator(operator: u16) -> bool {
    matches!(
        operator,
        0..=13 | 15..=17 | 21..=40 | 45..=48 | 50..=52 | 54 | 56..=58
    )
}

fn allowed_preset_generator(operator: u16) -> bool {
    matches!(operator, 5..=11 | 13 | 15..=17 | 21..=40 | 48 | 51 | 52 | 56)
}

fn merge_stack(
    preset_global: Option<&ZoneIr>,
    preset_local: &ZoneIr,
    instrument_global: Option<&ZoneIr>,
    instrument_local: &ZoneIr,
) -> (
    BTreeMap<u16, i32>,
    Vec<ImportDiagnostic>,
    Vec<ImportDiagnostic>,
) {
    let preset = merge_level(preset_global, preset_local);
    let instrument = merge_level(instrument_global, instrument_local);
    let mut out = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let mut blockers = Vec::new();
    for operator in 0..=60 {
        if let Some(value) = generator_default(operator) {
            out.insert(operator, value);
        }
    }
    for (&operator, &value) in &instrument {
        if valid_value_generator(operator) {
            out.insert(operator, value);
        }
    }
    for (&operator, &value) in &preset {
        if allowed_preset_generator(operator) {
            *out.entry(operator)
                .or_insert_with(|| generator_default(operator).unwrap_or(0)) += value;
        } else if valid_value_generator(operator) {
            blockers.push(ImportDiagnostic {
                code: "sf2.invalid_preset_generator".into(),
                message: format!(
                    "generator {operator} is not legal at preset level and was not applied"
                ),
            });
        }
    }
    for (&operator, value) in &mut out {
        if let Some((minimum, maximum)) = generator_range(operator) {
            let unclamped = *value;
            if !(allows_zero_time_sentinel(operator) && unclamped == i32::from(i16::MIN)) {
                *value = unclamped.clamp(minimum, maximum);
            }
            if *value != unclamped {
                diagnostics.push(ImportDiagnostic {
                    code: "sf2.generator_clamped".into(),
                    message: format!(
                        "generator {operator} resolved to {unclamped}, outside {minimum}..={maximum}; effective value is {}",
                        *value
                    ),
                });
            }
        }
    }
    (out, diagnostics, blockers)
}

fn effective_generator_count(
    preset_global: Option<&ZoneIr>,
    preset_local: &ZoneIr,
    instrument_global: Option<&ZoneIr>,
    instrument_local: &ZoneIr,
) -> usize {
    let mut present = [false; 61];
    for operator in 0..=60 {
        present[operator] = generator_default(operator as u16).is_some();
    }
    for zone in instrument_global
        .into_iter()
        .chain(std::iter::once(instrument_local))
    {
        for generator in &zone.generators {
            if valid_value_generator(generator.operator) {
                present[generator.operator as usize] = true;
            }
        }
    }
    for zone in preset_global
        .into_iter()
        .chain(std::iter::once(preset_local))
    {
        for generator in &zone.generators {
            if allowed_preset_generator(generator.operator) {
                present[generator.operator as usize] = true;
            }
        }
    }
    present.into_iter().filter(|value| *value).count()
}

fn mod_identity(modulator: &ModulatorIr) -> (u16, u16, u16, u16) {
    (
        modulator.source,
        modulator.destination,
        modulator.amount_source,
        modulator.transform,
    )
}

fn effective_identity(modulator: &EffectiveModulatorIr) -> (u16, u16, u16, u16) {
    (
        modulator.source,
        modulator.destination,
        modulator.amount_source,
        modulator.transform,
    )
}

fn default_modulators(version: VersionIr) -> Vec<EffectiveModulatorIr> {
    // SF2.01 §8.4.2 used velocity as a negative-unipolar switch secondary
    // source. SF2.04 removed that erroneous switch; SF3 3.1 follows the
    // later SF2.04 controller model.
    let velocity_filter_amount_source = if version.major == 2 && version.minor < 4 {
        0x0c02
    } else {
        0x0000
    };
    [
        (0x0502, 48, 960, 0x0000),
        (0x0102, 8, -2400, velocity_filter_amount_source),
        (0x000d, 6, 50, 0x0000),
        (0x0081, 6, 50, 0x0000),
        (0x0587, 48, 960, 0x0000),
        (0x028a, 17, 1000, 0x0000),
        (0x058b, 48, 960, 0x0000),
        (0x00db, 16, 200, 0x0000),
        (0x00dd, 15, 200, 0x0000),
        (0x020e, 59, 12_700, 0x0010),
    ]
    .into_iter()
    .map(
        |(source, destination, amount, amount_source)| EffectiveModulatorIr {
            source,
            destination,
            amount,
            amount_source,
            transform: 0,
        },
    )
    .collect()
}

fn override_modulators(target: &mut Vec<EffectiveModulatorIr>, source: &[ModulatorIr]) {
    for modulator in source {
        let replacement = EffectiveModulatorIr {
            source: modulator.source,
            destination: modulator.destination,
            amount: i32::from(modulator.amount),
            amount_source: modulator.amount_source,
            transform: modulator.transform,
        };
        if let Some(existing) = target
            .iter_mut()
            .find(|candidate| effective_identity(candidate) == mod_identity(modulator))
        {
            *existing = replacement;
        } else {
            target.push(replacement);
        }
    }
}

fn merge_modulators(
    preset_global: Option<&ZoneIr>,
    preset_local: &ZoneIr,
    instrument_global: Option<&ZoneIr>,
    instrument_local: &ZoneIr,
    version: VersionIr,
) -> Vec<EffectiveModulatorIr> {
    let mut instrument = default_modulators(version);
    if let Some(global) = instrument_global {
        override_modulators(&mut instrument, &global.modulators);
    }
    override_modulators(&mut instrument, &instrument_local.modulators);

    let mut preset = Vec::new();
    if let Some(global) = preset_global {
        override_modulators(&mut preset, &global.modulators);
    }
    override_modulators(&mut preset, &preset_local.modulators);
    for modulator in preset {
        if let Some(existing) = instrument
            .iter_mut()
            .find(|candidate| effective_identity(candidate) == effective_identity(&modulator))
        {
            existing.amount += modulator.amount;
        } else {
            instrument.push(modulator);
        }
    }
    instrument
}

fn controller_source_is_valid(index: u16) -> bool {
    // SF2.04 §8.2.1 reserves MSB/LSB selector, data-entry, NRPN and channel
    // mode controllers. A controller LSB is not an independent modulator
    // source in this profile; accepting one would silently lose its pairing.
    !matches!(index, 0 | 6 | 32 | 38 | 33..=63 | 98..=101 | 120..=127)
}

fn reserved_controller_source(source: u16) -> Option<u16> {
    let index = source & 0x7f;
    (source >> 10 <= 3 && source & 0x80 != 0 && !controller_source_is_valid(index)).then_some(index)
}

fn source_is_valid(source: u16) -> bool {
    let index = source & 0x7f;
    let is_cc = source & 0x80 != 0;
    let source_type = source >> 10;
    source_type <= 3
        && if is_cc {
            controller_source_is_valid(index)
        } else {
            matches!(index, 0 | 2 | 3 | 10 | 13 | 14 | 16)
        }
}

fn diagnostics_for(zones: &[&ZoneIr]) -> Vec<ImportDiagnostic> {
    let mut unknown = BTreeSet::new();
    let mut reserved_controllers = BTreeSet::new();
    let mut invalid_modulator = false;
    for z in zones {
        for g in &z.generators {
            if !valid_value_generator(g.operator) && !matches!(g.operator, 41 | 43 | 44 | 53) {
                unknown.insert(g.operator);
            }
        }
        for modulator in &z.modulators {
            for source in [modulator.source, modulator.amount_source] {
                if let Some(controller) = reserved_controller_source(source) {
                    reserved_controllers.insert(controller);
                }
            }
            let linked = modulator.destination & 0x8000 != 0;
            let valid_destination = modulator.destination == 59
                || (valid_value_generator(modulator.destination)
                    && !matches!(modulator.destination, 46 | 47 | 54 | 57 | 58));
            invalid_modulator |= linked
                || !valid_destination
                || !source_is_valid(modulator.source)
                || !source_is_valid(modulator.amount_source)
                || !matches!(modulator.transform, 0 | 2);
        }
    }
    let mut blockers = Vec::new();
    for op in unknown {
        blockers.push(ImportDiagnostic {
            code: "sf2.unknown_generator".into(),
            message: format!("generator {op} is retained but unresolved"),
        });
    }
    for controller in reserved_controllers {
        blockers.push(ImportDiagnostic {
            code: "sf2.reserved_controller_source".into(),
            message: format!(
                "MIDI CC {controller} is reserved and cannot be used as a SoundFont modulator source"
            ),
        });
    }
    if invalid_modulator {
        blockers.push(ImportDiagnostic {
            code: "sf2.unsupported_modulator".into(),
            message: "a linked or invalid modulator is retained in the raw zone records".into(),
        });
    }
    blockers
}

fn sample_blockers(
    sample_index: usize,
    samples: &[SampleIr],
    format: SourceFormat,
    source: &[u8],
) -> Vec<ImportDiagnostic> {
    let sample = &samples[sample_index];
    let mut blockers = Vec::new();
    let kind = sample.sample_type & 0x0f;
    let compressed = sample.sample_type & 0x10 != 0;
    if !matches!(kind, 1 | 2 | 4 | 8) || sample.sample_type & !0x801f != 0 {
        blockers.push(ImportDiagnostic {
            code: "sf2.unknown_sample_type".into(),
            message: format!(
                "sample type {:#06x} contains an unsupported source encoding",
                sample.sample_type
            ),
        });
    }
    if sample.sample_type & 0x8000 != 0 {
        blockers.push(ImportDiagnostic {
            code: "sf2.rom_sample".into(),
            message: "ROM sample metadata is retained but has no source payload to import".into(),
        });
    }
    if compressed && matches!(format, SourceFormat::Sf2Pcm16) {
        blockers.push(ImportDiagnostic {
            code: "sf2.sample_encoding_mismatch".into(),
            message: "compressed sample flag requires INFO/ifil major version 3".into(),
        });
    }
    if compressed && kind != 1 {
        blockers.push(ImportDiagnostic {
            code: "sf3.compressed_sample_not_mono".into(),
            message: "SF3 compressed streams must decode to one mono channel".into(),
        });
    }
    if compressed
        && source.get(sample.data_range.start..sample.data_range.start.saturating_add(4))
            != Some(b"OggS")
    {
        blockers.push(ImportDiagnostic {
            code: "sf3.unknown_compression".into(),
            message: "compressed sample does not begin with a supported Ogg stream".into(),
        });
    }
    if kind == 1 && sample.link != 0 {
        blockers.push(ImportDiagnostic {
            code: "sf2.invalid_sample_link".into(),
            message: "mono sample has a nonzero link".into(),
        });
    } else if matches!(kind, 2 | 4 | 8) {
        let Some(linked) = samples.get(sample.link as usize) else {
            blockers.push(ImportDiagnostic {
                code: "sf2.invalid_sample_link".into(),
                message: format!("linked sample index {} is out of range", sample.link),
            });
            return blockers;
        };
        let linked_kind = linked.sample_type & 0x0f;
        let complementary = matches!((kind, linked_kind), (2, 4) | (4, 2) | (8, 8));
        if linked.link as usize != sample_index || !complementary {
            blockers.push(ImportDiagnostic {
                code: "sf2.invalid_sample_link".into(),
                message: format!(
                    "sample link {} is not reciprocal/complementary (types {kind}/{linked_kind}, back-link {})",
                    sample.link, linked.link
                ),
            });
        }
    }
    blockers
}

fn push_unique(target: &mut Vec<ImportDiagnostic>, diagnostic: &ImportDiagnostic) {
    if !target.iter().any(|candidate| candidate == diagnostic) {
        target.push(diagnostic.clone());
    }
}

/// Parse only the container and Hydra metadata. `Sf3Vorbis` retains compressed
/// byte ranges and loop metadata; actual Vorbis decoding is a later explicit step.
pub fn parse(bytes: &[u8]) -> Result<LibraryIr, String> {
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err("SoundFont source exceeds 256 MiB".into());
    }
    if !has_header(bytes) {
        return Err("not a RIFF sfbk SoundFont".into());
    }
    let declared = u32_at(bytes, 4)? as usize;
    if declared.checked_add(8) != Some(bytes.len()) {
        return Err("RIFF size does not match source length".into());
    }
    let mut chunks = 0usize;
    let mut pos = 12usize;
    let mut smpl = None;
    let mut sm24 = None;
    let mut ifil = None;
    let mut ifil_major = None;
    let mut pdta: BTreeMap<[u8; 4], Range<usize>> = BTreeMap::new();
    let mut list_types = BTreeSet::new();
    let mut unknown = Vec::new();
    while pos < bytes.len() {
        if pos.checked_add(8).filter(|&x| x <= bytes.len()).is_none() {
            return Err("truncated RIFF chunk".into());
        }
        chunks += 1;
        if chunks > MAX_RIFF_CHUNKS {
            return Err("SoundFont exceeds 4096 RIFF chunks".into());
        }
        let tag = id(bytes, pos)?;
        let len = u32_at(bytes, pos + 4)? as usize;
        let start = pos + 8;
        let end = start.checked_add(len).ok_or("RIFF chunk size overflow")?;
        if end > bytes.len() {
            return Err("RIFF chunk extends past source".into());
        };
        if tag != *b"LIST" {
            pos = end
                .checked_add(len & 1)
                .filter(|&next| next <= bytes.len())
                .ok_or("RIFF chunk padding truncated")?;
            unknown.push(RawChunk {
                id: tag,
                data_range: start..end,
            });
            continue;
        }
        if len < 4 {
            return Err("RIFF LIST is missing type".into());
        };
        let typ = id(bytes, start)?;
        if !list_types.insert(typ) {
            return Err(format!(
                "duplicate SoundFont LIST/{}",
                String::from_utf8_lossy(&typ)
            ));
        }
        let sf3_odd_sdta = len & 1 != 0
            && typ == *b"sdta"
            && ifil_major == Some(3)
            && end
                .checked_add(12)
                .filter(|&next| next <= bytes.len())
                .is_some()
            && bytes.get(end..end + 4) == Some(b"LIST")
            && bytes.get(end + 8..end + 12) == Some(b"pdta");
        pos = if sf3_odd_sdta {
            end
        } else {
            end.checked_add(len & 1)
                .filter(|&next| next <= bytes.len())
                .ok_or("RIFF chunk padding truncated")?
        };
        let mut p = start + 4;
        while p < end {
            if p + 8 > end {
                return Err("truncated RIFF subchunk".into());
            };
            chunks += 1;
            if chunks > MAX_RIFF_CHUNKS {
                return Err("SoundFont exceeds 4096 RIFF chunks".into());
            };
            let sub = id(bytes, p)?;
            let n = u32_at(bytes, p + 4)? as usize;
            let a = p + 8;
            let e = a.checked_add(n).ok_or("RIFF subchunk size overflow")?;
            if e > end {
                return Err("RIFF subchunk extends past LIST".into());
            };
            p = e
                .checked_add(n & 1)
                .ok_or("RIFF subchunk padding overflow")?;
            if p > end {
                if sf3_odd_sdta && sub == *b"smpl" && n & 1 != 0 && e == end {
                    p = e;
                } else {
                    return Err("RIFF subchunk padding truncated".into());
                }
            }
            match (typ, sub) {
                (x, y) if x == *b"sdta" && y == *b"smpl" => {
                    if smpl.replace(a..e).is_some() {
                        return Err("duplicate SoundFont smpl chunk".into());
                    }
                }
                (x, y) if x == *b"sdta" && y == *b"sm24" => {
                    if sm24.replace(a..e).is_some() {
                        return Err("duplicate SoundFont sm24 chunk".into());
                    }
                }
                (x, y) if x == *b"INFO" && y == *b"ifil" => {
                    if ifil.replace(a..e).is_some() {
                        return Err("duplicate SoundFont ifil chunk".into());
                    }
                    if n == 4 {
                        ifil_major = Some(u16_at(bytes, a)?);
                    }
                }
                (x, y) if x == *b"pdta" => {
                    if pdta.insert(y, a..e).is_some() {
                        return Err("duplicate SoundFont Hydra chunk".into());
                    }
                }
                _ => unknown.push(RawChunk {
                    id: sub,
                    data_range: a..e,
                }),
            }
        }
    }
    let sample_data = smpl.ok_or("SoundFont has no sdta/smpl chunk")?;
    let ifil = ifil.ok_or("SoundFont missing INFO/ifil version")?;
    if ifil.len() != 4 || !matches!(u16_at(bytes, ifil.start)?, 2 | 3) {
        return Err("unsupported SoundFont ifil version".into());
    }
    let version = VersionIr {
        major: u16_at(bytes, ifil.start)?,
        minor: u16_at(bytes, ifil.start + 2)?,
    };
    let format = match version.major {
        2 => SourceFormat::Sf2Pcm16,
        3 => SourceFormat::Sf3Vorbis,
        _ => unreachable!(),
    };
    let need = |tag: &[u8; 4]| {
        pdta.get(tag)
            .map(|r| &bytes[r.clone()])
            .ok_or_else(|| format!("SoundFont missing pdta/{}", String::from_utf8_lossy(tag)))
    };
    let phdr_data = need(b"phdr")?;
    let pbag_data = need(b"pbag")?;
    let pmod_data = need(b"pmod")?;
    let pgen_data = need(b"pgen")?;
    let inst_data = need(b"inst")?;
    let ibag_data = need(b"ibag")?;
    let imod_data = need(b"imod")?;
    let igen_data = need(b"igen")?;
    let shdr_data = need(b"shdr")?;
    let pbag_count = record_count(pbag_data, 4, MAX_RAW_ZONES + 1, "pbag")?;
    let ibag_count = record_count(ibag_data, 4, MAX_RAW_ZONES + 1, "ibag")?;
    if pbag_count
        .saturating_sub(1)
        .checked_add(ibag_count.saturating_sub(1))
        .filter(|count| *count <= MAX_RAW_ZONES)
        .is_none()
    {
        return Err("SoundFont raw zone limit exceeded".into());
    }
    let pmod_count = record_count(pmod_data, 10, MAX_GEN_MOD_RECORDS, "pmod")?;
    let pgen_count = record_count(pgen_data, 4, MAX_GEN_MOD_RECORDS, "pgen")?;
    let imod_count = record_count(imod_data, 10, MAX_GEN_MOD_RECORDS, "imod")?;
    let igen_count = record_count(igen_data, 4, MAX_GEN_MOD_RECORDS, "igen")?;
    let raw_gen_mod_count = [pmod_count, pgen_count, imod_count, igen_count]
        .into_iter()
        .try_fold(0usize, |total, count| total.checked_add(count))
        .ok_or("SoundFont generator/modulator counter overflow")?;
    if raw_gen_mod_count > MAX_GEN_MOD_RECORDS {
        return Err("SoundFont generator/modulator limit exceeded".into());
    }
    let phdr = records(phdr_data, 38, MAX_PRESETS + 1, "phdr")?;
    let pbag = records(pbag_data, 4, MAX_RAW_ZONES + 1, "pbag")?;
    let pmod = records(pmod_data, 10, MAX_GEN_MOD_RECORDS, "pmod")?;
    let pgen = records(pgen_data, 4, MAX_GEN_MOD_RECORDS, "pgen")?;
    let inst = records(inst_data, 22, MAX_INSTRUMENTS + 1, "inst")?;
    let ibag = records(ibag_data, 4, MAX_RAW_ZONES + 1, "ibag")?;
    let imod = records(imod_data, 10, MAX_GEN_MOD_RECORDS, "imod")?;
    let igen = records(igen_data, 4, MAX_GEN_MOD_RECORDS, "igen")?;
    let shdr = records(shdr_data, 46, MAX_SAMPLES + 1, "shdr")?;
    if phdr.len() < 1
        || inst.len() < 1
        || shdr.len() < 1
        || pbag.len() < 1
        || ibag.len() < 1
        || pgen.is_empty()
        || pmod.is_empty()
        || igen.is_empty()
        || imod.is_empty()
    {
        return Err("SoundFont Hydra terminal record missing".into());
    }
    if phdr.len() - 1 > MAX_PRESETS
        || inst.len() - 1 > MAX_INSTRUMENTS
        || shdr.len() - 1 > MAX_SAMPLES
    {
        return Err("SoundFont definition limit exceeded".into());
    }
    if pbag.len() - 1 + ibag.len() - 1 > MAX_RAW_ZONES {
        return Err("SoundFont raw zone limit exceeded".into());
    }
    if pgen.len() + igen.len() + pmod.len() + imod.len() > MAX_GEN_MOD_RECORDS {
        return Err("SoundFont generator/modulator limit exceeded".into());
    }
    if pgen.last().unwrap().iter().any(|&value| value != 0)
        || igen.last().unwrap().iter().any(|&value| value != 0)
        || pmod.last().unwrap().iter().any(|&value| value != 0)
        || imod.last().unwrap().iter().any(|&value| value != 0)
    {
        return Err("SoundFont Hydra generator/modulator terminal is not zero".into());
    }
    if phdr.last().unwrap()[20..24]
        .iter()
        .chain(&phdr.last().unwrap()[26..38])
        .any(|&value| value != 0)
        || shdr.last().unwrap()[20..].iter().any(|&value| value != 0)
    {
        return Err("SoundFont Hydra header terminal is not zero".into());
    }
    let bags = |v: Vec<&[u8]>| -> Result<Vec<Bag>, String> {
        v.into_iter()
            .map(|r| {
                Ok(Bag {
                    generator: u16_at(r, 0)? as usize,
                    modu: u16_at(r, 2)? as usize,
                })
            })
            .collect()
    };
    let pb = bags(pbag)?;
    let ib = bags(ibag)?;
    let gens = |v: Vec<&[u8]>| -> Result<Vec<Generator>, String> {
        v.into_iter()
            .map(|r| {
                Ok(Generator {
                    operator: u16_at(r, 0)?,
                    amount: i16_at(r, 2)?,
                })
            })
            .collect()
    };
    let pg = gens(pgen)?;
    let ig = gens(igen)?;
    let mods = |v: Vec<&[u8]>| -> Result<Vec<ModulatorIr>, String> {
        v.into_iter()
            .map(|r| {
                Ok(ModulatorIr {
                    source: u16_at(r, 0)?,
                    destination: u16_at(r, 2)?,
                    amount: i16_at(r, 4)?,
                    amount_source: u16_at(r, 6)?,
                    transform: u16_at(r, 8)?,
                })
            })
            .collect()
    };
    let pm = mods(pmod)?;
    let im = mods(imod)?;
    let headers = |v: Vec<&[u8]>, preset: bool| -> Result<Vec<Header>, String> {
        v.into_iter()
            .map(|r| {
                Ok(Header {
                    name: name(&r[..20]),
                    a: u16_at(r, 20)?,
                    b: if preset { u16_at(r, 22)? } else { 0 },
                    bag: u16_at(r, if preset { 24 } else { 20 })? as usize,
                })
            })
            .collect()
    };
    let ph = headers(phdr, true)?;
    let ih = headers(inst, false)?;
    let validate_bags = |bags: &[Bag], generators: usize, modulators: usize| {
        for pair in bags.windows(2) {
            if pair[0].generator > pair[1].generator
                || pair[1].generator >= generators
                || pair[0].modu > pair[1].modu
                || pair[1].modu >= modulators
            {
                return Err("SoundFont Hydra bag indices are not monotonic".to_string());
            }
        }
        Ok(())
    };
    validate_bags(&pb, pg.len(), pm.len())?;
    validate_bags(&ib, ig.len(), im.len())?;
    for pair in ph.windows(2) {
        if pair[0].bag > pair[1].bag || pair[1].bag >= pb.len() {
            return Err("SoundFont preset bag indices are not monotonic".into());
        }
    }
    for pair in ih.windows(2) {
        if pair[0].bag > pair[1].bag || pair[1].bag >= ib.len() {
            return Err("SoundFont instrument bag indices are not monotonic".into());
        }
    }
    if ph.last().unwrap().bag != pb.len() - 1
        || ih.last().unwrap().bag != ib.len() - 1
        || pb.last().unwrap().generator != pg.len() - 1
        || pb.last().unwrap().modu != pm.len() - 1
        || ib.last().unwrap().generator != ig.len() - 1
        || ib.last().unwrap().modu != im.len() - 1
    {
        return Err("SoundFont Hydra terminal indices are invalid".into());
    }
    let preset_terminal = &ph.last().unwrap().name;
    let instrument_terminal = &ih.last().unwrap().name;
    let sample_terminal = name(&shdr[shdr.len() - 1][..20]);
    if !matches!(preset_terminal.as_str(), "" | "EOP")
        || !matches!(instrument_terminal.as_str(), "" | "EOI")
        || !matches!(sample_terminal.as_str(), "" | "EOS")
    {
        return Err(format!(
            "SoundFont Hydra terminal names are invalid ({:?}/{:?}/{:?})",
            preset_terminal, instrument_terminal, sample_terminal
        ));
    }
    if matches!(format, SourceFormat::Sf2Pcm16) && sample_data.len() & 1 != 0 {
        return Err("PCM smpl chunk has an odd byte length".into());
    }
    let mut samples = Vec::new();
    for r in &shdr[..shdr.len() - 1] {
        let start = u32_at(r, 20)?;
        let end = u32_at(r, 24)?;
        let ls = u32_at(r, 28)?;
        let le = u32_at(r, 32)?;
        let sample_type = u16_at(r, 44)?;
        let compressed = sample_type & 0x10 != 0;
        let rom_or_unknown = sample_type & !0x801f != 0 || sample_type & 0x8000 != 0;
        if start >= end || ls > le || (!compressed && !rom_or_unknown && (ls < start || le > end)) {
            return Err("SoundFont sample/loop range is invalid".into());
        };
        let data_range = if rom_or_unknown {
            sample_data.start..sample_data.start
        } else if compressed {
            let a = sample_data
                .start
                .checked_add(start as usize)
                .ok_or("sample offset overflow")?;
            let e = sample_data
                .start
                .checked_add(end as usize)
                .ok_or("sample offset overflow")?;
            if e > sample_data.end {
                return Err("SF3 compressed sample range exceeds smpl".into());
            };
            a..e
        } else {
            let a = sample_data
                .start
                .checked_add(
                    (start as usize)
                        .checked_mul(2)
                        .ok_or("sample offset overflow")?,
                )
                .ok_or("sample offset overflow")?;
            let e = sample_data
                .start
                .checked_add(
                    (end as usize)
                        .checked_mul(2)
                        .ok_or("sample offset overflow")?,
                )
                .ok_or("sample offset overflow")?;
            if e > sample_data.end {
                return Err("PCM sample range exceeds smpl".into());
            };
            a..e
        };
        let sample_rate = u32_at(r, 36)?;
        if !(400..=50_000).contains(&sample_rate) {
            return Err("SoundFont sample rate is outside 400..=50000 Hz".into());
        }
        if r[40] > 127 {
            return Err("SoundFont sample root key is outside MIDI range".into());
        }
        samples.push(SampleIr {
            name: name(&r[..20]),
            start,
            end,
            loop_start: ls,
            loop_end: le,
            sample_rate,
            root_key: r[40],
            pitch_correction: r[41] as i8,
            link: u16_at(r, 42)?,
            sample_type,
            data_range,
        });
    }
    let defs = |hs: &[Header],
                bags: &[Bag],
                gs: &[Generator],
                ms: &[ModulatorIr],
                structural: u16|
     -> Result<Vec<Definition>, String> {
        let mut o = Vec::new();
        for i in 0..hs.len() - 1 {
            let a = hs[i].bag;
            let e = hs[i + 1].bag;
            if a > e || e >= bags.len() {
                return Err("SoundFont definition bag range is invalid".into());
            };
            let mut z = Vec::new();
            for n in a..e {
                z.push(zone(
                    bags,
                    gs,
                    ms,
                    n,
                    n == a && amount(&zone(bags, gs, ms, n, false)?, structural).is_none(),
                )?)
            }
            o.push(Definition {
                name: hs[i].name.clone(),
                zones: z,
            })
        }
        Ok(o)
    };
    let raw_presets = defs(&ph, &pb, &pg, &pm, 41)?;
    let raw_instruments = defs(&ih, &ib, &ig, &im, 53)?;
    let mut diagnostics = Vec::new();
    let mut blockers = Vec::new();
    let mut presets = Vec::new();
    let mut expanded_regions = 0usize;
    let mut expanded_gen_mod_records = 0usize;
    for (pi, h) in ph[..ph.len() - 1].iter().enumerate() {
        if h.a > 127 {
            return Err("SoundFont preset program is outside MIDI range".into());
        }
        let mut regions = Vec::new();
        let pdef = &raw_presets[pi];
        let pglobal = pdef.zones.first().filter(|z| z.global);
        for pz in pdef.zones.iter().filter(|z| !z.global) {
            let instrument =
                amount(pz, 41).ok_or("preset local zone lacks instrument generator")? as usize;
            if instrument >= raw_instruments.len() {
                return Err("preset instrument index out of range".into());
            };
            let idef = &raw_instruments[instrument];
            let iglobal = idef.zones.first().filter(|z| z.global);
            for iz in idef.zones.iter().filter(|z| !z.global) {
                let sample = amount(iz, 53).ok_or("instrument local zone lacks sampleID")?;
                if sample as usize >= samples.len() {
                    return Err("instrument sampleID out of range".into());
                };
                let stack = [pglobal, Some(pz), iglobal, Some(iz)];
                let refs = stack.iter().flatten().copied().collect::<Vec<_>>();
                let preset_key = range(pz, 43)
                    .or_else(|| pglobal.and_then(|global| range(global, 43)))
                    .unwrap_or([0, 127]);
                let instrument_key = range(iz, 43)
                    .or_else(|| iglobal.and_then(|global| range(global, 43)))
                    .unwrap_or([0, 127]);
                let Some(key) = combine_range(preset_key, instrument_key) else {
                    continue;
                };
                let preset_velocity = range(pz, 44)
                    .or_else(|| pglobal.and_then(|global| range(global, 44)))
                    .unwrap_or([0, 127]);
                let instrument_velocity = range(iz, 44)
                    .or_else(|| iglobal.and_then(|global| range(global, 44)))
                    .unwrap_or([0, 127]);
                let Some(vel) = combine_range(preset_velocity, instrument_velocity) else {
                    continue;
                };
                let raw_modulator_records = refs.iter().try_fold(0usize, |total, zone| {
                    total
                        .checked_add(zone.modulators.len())
                        .ok_or("SoundFont expanded generator/modulator counter overflow")
                })?;
                let effective_generator_records =
                    effective_generator_count(pglobal, pz, iglobal, iz);
                let effective_modulator_upper_bound =
                    refs.iter().try_fold(10usize, |total, zone| {
                        total
                            .checked_add(zone.modulators.len())
                            .ok_or("SoundFont expanded generator/modulator counter overflow")
                    })?;
                let added_records = raw_modulator_records
                    .checked_add(effective_generator_records)
                    .ok_or("SoundFont expanded generator/modulator counter overflow")?
                    .checked_add(effective_modulator_upper_bound)
                    .ok_or("SoundFont expanded generator/modulator counter overflow")?;
                expanded_gen_mod_records = expanded_gen_mod_records
                    .checked_add(added_records)
                    .ok_or("SoundFont expanded generator/modulator counter overflow")?;
                if expanded_gen_mod_records > MAX_EXPANDED_GEN_MOD_RECORDS {
                    return Err(
                        "SoundFont expanded generator/modulator limit exceeded (1000000)".into(),
                    );
                }
                expanded_regions = expanded_regions
                    .checked_add(1)
                    .ok_or("SoundFont expanded region counter overflow")?;
                if expanded_regions > MAX_EXPANDED_REGIONS {
                    return Err("SoundFont expanded region limit exceeded".into());
                }
                let (effective, region_diagnostics, mut region_blockers) =
                    merge_stack(pglobal, pz, iglobal, iz);
                let exclusive_class = effective.get(&57).copied().unwrap_or(0) as u16;
                let effective_modulators = merge_modulators(pglobal, pz, iglobal, iz, version);
                let mut modulators = Vec::new();
                for z in &refs {
                    modulators.extend(z.modulators.clone())
                }
                region_blockers.extend(diagnostics_for(&refs));
                region_blockers.extend(sample_blockers(sample as usize, &samples, format, bytes));
                if effective.get(&54) == Some(&2) {
                    region_blockers.push(ImportDiagnostic {
                        code: "sf2.reserved_sample_mode".into(),
                        message:
                            "sampleModes value 2 is reserved and cannot be interpreted as a loop"
                                .into(),
                    });
                }
                for diagnostic in region_diagnostics.iter().chain(&region_blockers) {
                    push_unique(&mut diagnostics, diagnostic);
                }
                regions.push(RegionIr {
                    key_range: key,
                    velocity_range: vel,
                    sample,
                    effective_generators: effective,
                    modulators,
                    effective_modulators,
                    preset_zone: pz.bag_index,
                    instrument_zone: iz.bag_index,
                    exclusive_class,
                    blockers: region_blockers,
                });
            }
        }
        presets.push(PresetIr {
            bank: h.b,
            program: h.a as u8,
            name: h.name.clone(),
            regions,
        })
    }
    if sm24.is_some() {
        let d = ImportDiagnostic {
            code: "sf2.sm24_retained".into(),
            message: "24-bit sample extension is retained but unresolved by the PCM16 import phase"
                .into(),
        };
        diagnostics.push(d.clone());
        blockers.push(d);
    }
    Ok(LibraryIr {
        format,
        version,
        presets,
        samples,
        sample_data,
        sm24_data: sm24,
        raw_presets,
        raw_instruments,
        unknown_chunks: unknown,
        diagnostics,
        blockers,
    })
}

#[cfg(test)]
/// A redistributable one-preset, one-region, one-PCM16-sample SF2 fixture.
pub fn fixture() -> Vec<u8> {
    tests::synthetic(
        2,
        vec![(vec![(41, 0)], vec![])],
        vec![(vec![(53, 0)], vec![])],
        vec![tests::TestSample::mono(0, 2)],
        vec![0; 4],
    )
}

#[cfg(test)]
pub fn tone_fixture() -> Vec<u8> {
    let frames = 2205_u32;
    let pcm = (0..frames)
        .flat_map(|i| {
            ((f64::from(i) * 440. * std::f64::consts::TAU / 22050.)
                .sin()
                .mul_add(12000., 0.)
                .round() as i16)
                .to_le_bytes()
        })
        .collect();
    tests::synthetic(
        2,
        vec![(vec![(41, 0)], vec![])],
        vec![(
            vec![
                (33, -32768),
                (34, -32768),
                (35, -32768),
                (38, -3600),
                (54, 1),
                (53, 0),
            ],
            vec![],
        )],
        vec![tests::TestSample::mono(0, frames)],
        pcm,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestModulator = (u16, u16, i16, u16, u16);

    #[derive(Clone, Copy)]
    pub(super) struct TestSample {
        start: u32,
        end: u32,
        loop_start: u32,
        loop_end: u32,
        link: u16,
        sample_type: u16,
    }

    impl TestSample {
        pub(super) fn mono(start: u32, end: u32) -> Self {
            Self {
                start,
                end,
                loop_start: start,
                loop_end: end,
                link: 0,
                sample_type: 1,
            }
        }
    }

    fn fixed(name: &[u8], size: usize) -> Vec<u8> {
        let mut value = vec![0; size];
        value[..name.len()].copy_from_slice(name);
        value
    }

    fn chunk(id: &[u8; 4], data: Vec<u8>) -> Vec<u8> {
        let mut value = Vec::new();
        value.extend_from_slice(id);
        value.extend_from_slice(&(data.len() as u32).to_le_bytes());
        value.extend_from_slice(&data);
        if data.len() & 1 != 0 {
            value.push(0);
        }
        value
    }

    fn list(kind: &[u8; 4], entries: Vec<Vec<u8>>) -> Vec<u8> {
        let mut data = kind.to_vec();
        for entry in entries {
            data.extend(entry);
        }
        chunk(b"LIST", data)
    }

    fn zones(zones: Vec<(Vec<(u16, i16)>, Vec<TestModulator>)>) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let mut bags = Vec::new();
        let mut generators = Vec::new();
        let mut modulators = Vec::new();
        let mut generator_index = 0u16;
        let mut modulator_index = 0u16;
        for (zone_generators, zone_modulators) in zones {
            bags.extend(generator_index.to_le_bytes());
            bags.extend(modulator_index.to_le_bytes());
            for (operator, amount) in zone_generators {
                generators.extend(operator.to_le_bytes());
                generators.extend(amount.to_le_bytes());
                generator_index += 1;
            }
            for (source, destination, amount, amount_source, transform) in zone_modulators {
                modulators.extend(source.to_le_bytes());
                modulators.extend(destination.to_le_bytes());
                modulators.extend(amount.to_le_bytes());
                modulators.extend(amount_source.to_le_bytes());
                modulators.extend(transform.to_le_bytes());
                modulator_index += 1;
            }
        }
        bags.extend(generator_index.to_le_bytes());
        bags.extend(modulator_index.to_le_bytes());
        generators.extend([0; 4]);
        modulators.extend([0; 10]);
        (bags, modulators, generators)
    }

    pub(super) fn synthetic(
        major: u16,
        preset_zones: Vec<(Vec<(u16, i16)>, Vec<TestModulator>)>,
        instrument_zones: Vec<(Vec<(u16, i16)>, Vec<TestModulator>)>,
        samples: Vec<TestSample>,
        sample_data: Vec<u8>,
    ) -> Vec<u8> {
        synthetic_version(
            major,
            4,
            preset_zones,
            instrument_zones,
            samples,
            sample_data,
        )
    }

    fn synthetic_version(
        major: u16,
        minor: u16,
        preset_zones: Vec<(Vec<(u16, i16)>, Vec<TestModulator>)>,
        instrument_zones: Vec<(Vec<(u16, i16)>, Vec<TestModulator>)>,
        samples: Vec<TestSample>,
        sample_data: Vec<u8>,
    ) -> Vec<u8> {
        let preset_zone_count = preset_zones.len() as u16;
        let instrument_zone_count = instrument_zones.len() as u16;
        let (pbag, pmod, pgen) = zones(preset_zones);
        let (ibag, imod, igen) = zones(instrument_zones);
        let mut phdr = Vec::new();
        for (label, bag_index) in [
            (b"Fixture".as_slice(), 0),
            (b"EOP".as_slice(), preset_zone_count),
        ] {
            phdr.extend(fixed(label, 20));
            phdr.extend(0_u16.to_le_bytes());
            phdr.extend(0_u16.to_le_bytes());
            phdr.extend(bag_index.to_le_bytes());
            phdr.extend([0; 12]);
        }
        let mut inst = Vec::new();
        for (label, bag_index) in [
            (b"Fixture instrument".as_slice(), 0),
            (b"EOI".as_slice(), instrument_zone_count),
        ] {
            inst.extend(fixed(label, 20));
            inst.extend(bag_index.to_le_bytes());
        }
        let mut shdr = Vec::new();
        for (index, sample) in samples.iter().enumerate() {
            shdr.extend(fixed(format!("Sample {index}").as_bytes(), 20));
            shdr.extend(sample.start.to_le_bytes());
            shdr.extend(sample.end.to_le_bytes());
            shdr.extend(sample.loop_start.to_le_bytes());
            shdr.extend(sample.loop_end.to_le_bytes());
            shdr.extend(22_050_u32.to_le_bytes());
            shdr.push(60);
            shdr.push(0);
            shdr.extend(sample.link.to_le_bytes());
            shdr.extend(sample.sample_type.to_le_bytes());
        }
        shdr.extend(fixed(b"EOS", 46));
        let pdta = list(
            b"pdta",
            vec![
                chunk(b"phdr", phdr),
                chunk(b"pbag", pbag),
                chunk(b"pmod", pmod),
                chunk(b"pgen", pgen),
                chunk(b"inst", inst),
                chunk(b"ibag", ibag),
                chunk(b"imod", imod),
                chunk(b"igen", igen),
                chunk(b"shdr", shdr),
            ],
        );
        let mut body = Vec::new();
        body.extend(list(
            b"INFO",
            vec![chunk(
                b"ifil",
                [major.to_le_bytes(), minor.to_le_bytes()].concat(),
            )],
        ));
        body.extend(list(b"sdta", vec![chunk(b"smpl", sample_data)]));
        body.extend(pdta);
        let mut result = b"RIFF".to_vec();
        result.extend(((body.len() + 4) as u32).to_le_bytes());
        result.extend(b"sfbk");
        result.extend(body);
        result
    }

    fn tag(source: &[u8], value: &[u8; 4]) -> usize {
        source
            .windows(4)
            .position(|window| window == value)
            .unwrap()
    }

    fn write_u32(source: &mut [u8], offset: usize, value: u32) {
        source[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn fixture_has_one_complete_layer_and_absolute_pcm_range() {
        let source = fixture();
        let library = parse(&source).unwrap();
        assert_eq!(library.format, SourceFormat::Sf2Pcm16);
        assert_eq!(library.version, VersionIr { major: 2, minor: 4 });
        assert_eq!(library.presets[0].regions.len(), 1);
        assert_eq!(library.presets[0].regions[0].sample, 0);
        assert_eq!(library.samples[0].data_range.len(), 4);
        assert_eq!(
            library.presets[0].regions[0].effective_generators[&8],
            13_500
        );
        assert_eq!(library.presets[0].regions[0].effective_generators[&56], 100);
        assert_eq!(library.presets[0].regions[0].effective_modulators.len(), 10);
    }

    #[test]
    fn versioned_velocity_filter_default_preserves_ifil_semantics() {
        let source = |major, minor| {
            synthetic_version(
                major,
                minor,
                vec![(vec![(41, 0)], vec![])],
                vec![(vec![(53, 0)], vec![])],
                vec![TestSample::mono(0, 2)],
                vec![0; 4],
            )
        };
        let amount_source = |library: &LibraryIr| {
            library.presets[0].regions[0]
                .effective_modulators
                .iter()
                .find(|modulator| modulator.source == 0x0102 && modulator.destination == 8)
                .unwrap()
                .amount_source
        };
        let sf201 = parse(&source(2, 1)).unwrap();
        assert_eq!(sf201.version, VersionIr { major: 2, minor: 1 });
        assert_eq!(amount_source(&sf201), 0x0c02);
        let sf204 = parse(&source(2, 4)).unwrap();
        assert_eq!(amount_source(&sf204), 0);
        let sf31 = parse(&source(3, 1)).unwrap();
        assert_eq!(sf31.format, SourceFormat::Sf3Vorbis);
        assert_eq!(sf31.version, VersionIr { major: 3, minor: 1 });
        assert_eq!(amount_source(&sf31), 0);
    }

    #[test]
    fn reserved_controller_sources_are_retained_with_strict_diagnostics() {
        for controller in [
            0, 6, 32, 38, 98, 99, 100, 101, 120, 121, 122, 123, 124, 125, 126, 127,
        ]
        .into_iter()
        .chain(33..=63)
        {
            assert!(!source_is_valid(0x80 | controller), "CC {controller}");
        }
        for controller in [1, 7, 10, 11, 64, 66, 91, 93, 95, 96, 97] {
            assert!(source_is_valid(0x80 | controller), "CC {controller}");
        }
        let source = synthetic(
            2,
            vec![(vec![(41, 0)], vec![])],
            vec![(vec![(53, 0)], vec![(0x0086, 48, 1, 0, 0)])],
            vec![TestSample::mono(0, 2)],
            vec![0; 4],
        );
        let library = parse(&source).unwrap();
        assert!(
            library.presets[0].regions[0]
                .blockers
                .iter()
                .any(|diagnostic| diagnostic.code == "sf2.reserved_controller_source")
        );
        assert!(
            library.presets[0].regions[0]
                .blockers
                .iter()
                .any(|diagnostic| diagnostic.code == "sf2.unsupported_modulator")
        );
        assert!(
            library
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "sf2.unsupported_modulator")
        );
    }

    #[test]
    fn malformed_riff_size_duplicate_chunks_and_bad_terminals_fail() {
        let mut source = fixture();
        source[4] = 0;
        assert!(parse(&source).unwrap_err().contains("RIFF size"));

        let mut source = fixture();
        let pgen = tag(&source, b"pgen");
        let size = u32_at(&source, pgen + 4).unwrap() as usize + 8;
        let duplicate = source[pgen..pgen + size].to_vec();
        let pdta_type = tag(&source, b"pdta");
        let pdta_header = pdta_type - 8;
        let old_pdta_size = u32_at(&source, pdta_header + 4).unwrap();
        source.splice(pgen + size..pgen + size, duplicate);
        write_u32(&mut source, pdta_header + 4, old_pdta_size + size as u32);
        let riff_size = source.len() as u32 - 8;
        write_u32(&mut source, 4, riff_size);
        assert!(
            parse(&source)
                .unwrap_err()
                .contains("duplicate SoundFont Hydra")
        );

        let mut source = fixture();
        let pgen = tag(&source, b"pgen");
        let size = u32_at(&source, pgen + 4).unwrap() as usize;
        source[pgen + 8 + size - 1] = 1;
        assert!(parse(&source).unwrap_err().contains("terminal is not zero"));

        let mut source = fixture();
        let marker = 41_u16.to_le_bytes();
        let position = source.windows(2).position(|v| v == marker).unwrap();
        source[position + 2] = 1;
        assert!(parse(&source).is_err());
    }

    #[test]
    fn hierarchy_uses_local_override_defaults_layers_and_modulator_identity() {
        let velocity_attenuation = |amount| (0x0502, 48, amount, 0, 0);
        let source = synthetic(
            2,
            vec![
                (
                    vec![(43, i16::from_le_bytes([0, 60])), (8, 200)],
                    vec![velocity_attenuation(100)],
                ),
                (
                    vec![
                        (43, i16::from_le_bytes([50, 90])),
                        (8, -500),
                        (25, 100),
                        (51, 100),
                        (41, 0),
                    ],
                    vec![velocity_attenuation(200)],
                ),
            ],
            vec![
                (
                    vec![
                        (43, i16::from_le_bytes([0, 100])),
                        (25, i16::MIN),
                        (26, i16::MIN),
                        (28, i16::MIN),
                        (51, 100),
                        (57, 7),
                    ],
                    vec![velocity_attenuation(500)],
                ),
                (
                    vec![(43, i16::from_le_bytes([40, 80])), (53, 0)],
                    vec![velocity_attenuation(400)],
                ),
                (
                    vec![(43, i16::from_le_bytes([70, 100])), (57, 9), (53, 1)],
                    vec![],
                ),
            ],
            vec![
                TestSample {
                    link: 1,
                    sample_type: 4,
                    ..TestSample::mono(0, 2)
                },
                TestSample {
                    link: 0,
                    sample_type: 2,
                    ..TestSample::mono(2, 4)
                },
            ],
            vec![0; 8],
        );
        let library = parse(&source).unwrap();
        let regions = &library.presets[0].regions;
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].key_range, [50, 80]);
        assert_eq!(regions[1].key_range, [70, 90]);
        assert_eq!(regions[0].effective_generators[&8], 13_000);
        assert_eq!(regions[0].effective_generators[&21], -12_000);
        assert_eq!(regions[0].effective_generators[&58], -1);
        assert_eq!(regions[0].effective_generators[&51], 120);
        assert_eq!(regions[0].effective_generators[&25], -12_000);
        assert_eq!(regions[0].effective_generators[&26], i32::from(i16::MIN));
        assert_eq!(regions[0].effective_generators[&28], -12_000);
        assert_eq!(regions[0].exclusive_class, 7);
        assert_eq!(regions[1].exclusive_class, 9);
        assert_eq!(regions[0].modulators.len(), 4);
        let effective = regions[0]
            .effective_modulators
            .iter()
            .find(|m| m.source == 0x0502 && m.destination == 48)
            .unwrap();
        assert_eq!(effective.amount, 600);
        assert!(regions.iter().all(|region| region.blockers.is_empty()));
        assert!(
            library
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "sf2.generator_clamped")
        );
    }

    #[test]
    fn disjoint_ranges_remain_raw_but_do_not_expand() {
        let source = synthetic(
            2,
            vec![(vec![(43, i16::from_le_bytes([0, 10])), (41, 0)], vec![])],
            vec![(vec![(43, i16::from_le_bytes([20, 30])), (53, 0)], vec![])],
            vec![TestSample::mono(0, 2)],
            vec![0; 4],
        );
        let library = parse(&source).unwrap();
        assert!(library.presets[0].regions.is_empty());
        assert_eq!(library.raw_presets[0].zones.len(), 1);
        assert_eq!(library.raw_instruments[0].zones.len(), 1);
    }

    #[test]
    fn sf3_padding_exception_is_exact_and_loops_are_pcm_relative() {
        let mut source = synthetic(
            3,
            vec![(vec![(41, 0)], vec![])],
            vec![(vec![(53, 0)], vec![])],
            vec![TestSample {
                loop_start: 4_000,
                loop_end: 5_000,
                sample_type: 0x11,
                ..TestSample::mono(0, 3)
            }],
            vec![1, 2, 3],
        );
        let smpl = tag(&source, b"smpl");
        let smpl_size = u32_at(&source, smpl + 4).unwrap() as usize;
        source.remove(smpl + 8 + smpl_size);
        let sdta = tag(&source, b"sdta") - 8;
        let sdta_size = u32_at(&source, sdta + 4).unwrap();
        write_u32(&mut source, sdta + 4, sdta_size - 1);
        let riff_size = source.len() as u32 - 8;
        write_u32(&mut source, 4, riff_size);
        let library = parse(&source).unwrap();
        assert_eq!(library.format, SourceFormat::Sf3Vorbis);
        assert_eq!(library.samples[0].data_range.len(), 3);
        assert_eq!(library.samples[0].loop_end, 5_000);

        let ifil = tag(&source, b"ifil");
        source[ifil + 8..ifil + 10].copy_from_slice(&2_u16.to_le_bytes());
        assert!(parse(&source).unwrap_err().contains("padding"));
    }

    #[test]
    fn invalid_stereo_link_and_reserved_sample_mode_block_only_the_region() {
        let source = synthetic(
            2,
            vec![(vec![(41, 0)], vec![])],
            vec![(vec![(54, 2), (53, 0)], vec![])],
            vec![TestSample {
                link: 0,
                sample_type: 4,
                ..TestSample::mono(0, 2)
            }],
            vec![0; 4],
        );
        let library = parse(&source).unwrap();
        assert!(library.blockers.is_empty());
        let codes = library.presets[0].regions[0]
            .blockers
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<BTreeSet<_>>();
        assert!(codes.contains("sf2.invalid_sample_link"));
        assert!(codes.contains("sf2.reserved_sample_mode"));
    }

    #[test]
    fn record_and_expansion_caps_fire_before_large_clones() {
        let mut source = fixture();
        let pgen = tag(&source, b"pgen");
        let old_size = u32_at(&source, pgen + 4).unwrap() as usize;
        let oversized = vec![0; (MAX_GEN_MOD_RECORDS + 1) * 4];
        let new_size = oversized.len();
        source.splice(pgen + 8..pgen + 8 + old_size, oversized);
        write_u32(&mut source, pgen + 4, new_size as u32);
        let pdta = tag(&source, b"pdta") - 8;
        let pdta_size = u32_at(&source, pdta + 4).unwrap();
        write_u32(
            &mut source,
            pdta + 4,
            pdta_size + (new_size - old_size) as u32,
        );
        let riff_size = source.len() as u32 - 8;
        write_u32(&mut source, 4, riff_size);
        assert!(parse(&source).unwrap_err().contains("pgen record limit"));

        let preset_zones = (0..257)
            .map(|_| (vec![(41, 0)], vec![]))
            .collect::<Vec<_>>();
        let instrument_zones = (0..256)
            .map(|_| (vec![(53, 0)], vec![]))
            .collect::<Vec<_>>();
        let source = synthetic(
            2,
            preset_zones,
            instrument_zones,
            vec![TestSample::mono(0, 2)],
            vec![0; 4],
        );
        let error = parse(&source).unwrap_err();
        assert!(
            error.contains("expanded region limit")
                || error.contains("expanded generator/modulator limit")
        );

        let mods = vec![(2, 48, 1, 0, 0); 4];
        let preset_zones = (0..250)
            .map(|_| (vec![(41, 0)], mods.clone()))
            .collect::<Vec<_>>();
        let instrument_zones = (0..250)
            .map(|_| (vec![(53, 0)], mods.clone()))
            .collect::<Vec<_>>();
        let source = synthetic(
            2,
            preset_zones,
            instrument_zones,
            vec![TestSample::mono(0, 2)],
            vec![0; 4],
        );
        assert!(
            parse(&source)
                .unwrap_err()
                .contains("expanded generator/modulator limit")
        );
    }
}
