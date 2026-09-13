//! Bounded Sony VAB v7 / explicitly paired VH+VB import.
//! Original implementation and tests for Epok. No SDK or game source copied.
//! See docs/architecture/cross-platform-audio-phase-e-format-audit.md for evidence.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_SOURCE_PROGRAMS: usize = 128;
pub const MAX_SOURCE_TONES: usize = MAX_SOURCE_PROGRAMS * 16;
pub const MAX_SOURCE_SAMPLES: usize = 254;
pub const MAX_DECODED_FRAMES: usize = MAX_SOURCE_BYTES / 16 * 28;
/// Describes the current destination, NEVER a source parser limit.
pub const CURRENT_SOUNDBANK_ZONES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Error {
    pub part: usize,
    pub offset: usize,
    pub reason: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "source {} offset 0x{:x}: {}",
            self.part, self.offset, self.reason
        )
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn fail<T>(part: usize, offset: usize, reason: &str) -> Result<T> {
    Err(Error {
        part,
        offset,
        reason: reason.into(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub part: usize,
    pub offset: usize,
    pub length: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// User-supplied location and rights statement; neither establishes ownership.
    pub label: String,
    pub rights: String,
    /// Computed over precisely these bytes, never trusted from the caller.
    pub sha256: String,
    pub bytes: Vec<u8>,
}
pub struct Input<'a> {
    pub bytes: &'a [u8],
    pub label: &'a str,
    pub rights: &'a str,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Source,
    Authoring,
    Playback,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub scope: Scope,
    pub code: String,
    pub at: Span,
    pub reason: String,
}
fn diag(out: &mut Vec<Diagnostic>, scope: Scope, code: &str, at: Span, reason: &str) {
    out.push(Diagnostic {
        scope,
        code: code.into(),
        at,
        reason: reason.into(),
    });
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    pub raw: [u8; 32],
    pub version: u32,
    pub bank_id: u32,
    pub declared_bytes: u32,
    pub program_count: u16,
    pub tone_count: u16,
    pub sample_count: u16,
    pub master_gain: u8,
    pub master_pan: u8,
    pub attributes: [u8; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tone {
    pub at: Span,
    pub raw: [u8; 32],
    pub slot: u8,
    pub priority: u8,
    pub mode: u8,
    pub reverb_requested: bool,
    pub gain: u8,
    pub pan: u8,
    pub root_key: u8,
    /// Raw Sony VagAtr.shift byte. No unverified sign/centering conversion.
    pub tuning_raw: u8,
    pub keys: [u8; 2],
    pub vibrato: [u8; 2],
    pub portamento: [u8; 2],
    pub bend_down_up: [u8; 2],
    pub adsr_words: [u16; 2],
    /// Both table program and embedded owner survive even when inconsistent.
    pub embedded_program: u16,
    pub sample_id: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub at: Span,
    pub raw: [u8; 16],
    pub id: u8,
    pub compact_slot: u8,
    pub gain: u8,
    pub priority: u8,
    pub mode: u8,
    pub pan: u8,
    pub attributes: u16,
    pub tones: Vec<Tone>,
    pub overlapping_tone_pairs: Vec<[u8; 2]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub byte_offset: usize,
    pub filter: u8,
    pub shift: u8,
    pub flags: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Termination {
    EndMute,
    EndRepeat,
    /// Source bytes end without an END flag; do not infer hardware stop.
    MissingEnd,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Loop {
    /// PCM frame positions, half-open, including the complete end block.
    pub frames: [u32; 2],
    pub start_block: u32,
    pub end_block: u32,
    pub history_at_first_entry: [i16; 2],
    pub history_after_end: [i16; 2],
    pub entry_filter: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decoded {
    /// One forward pass through first END, inclusive. No looping/resampling.
    pub pcm: Vec<i16>,
    /// Every source block, including unreachable trailers, remains inspectable.
    pub blocks: Vec<Block>,
    pub termination: Termination,
    pub loop_region: Option<Loop>,
    pub trailing_bytes: usize,
    pub diagnostics: Vec<Diagnostic>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sample {
    pub id: u16,
    pub encoded: Span,
    pub encoded_sha256: String,
    /// Raw VAB/VB has no waveform rate. Assignment requires an import policy.
    pub original_sample_rate_hz: Option<u32>,
    pub decoded: Decoded,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BankImportIr {
    pub schema_version: u32,
    pub profile: String,
    pub sources: Vec<Source>,
    pub header: Header,
    pub programs: Vec<Program>,
    pub samples: Vec<Sample>,
    pub diagnostics: Vec<Diagnostic>,
}

fn u16le(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn u32le(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(
        bytes[at..at + 4]
            .try_into()
            .expect("validated fixed header"),
    )
}
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Parse structurally safe source even when source references cannot be played.
/// All semantic incompatibilities remain explicit diagnostics with source offsets.
pub fn parse(header_input: Input<'_>, body_input: Option<Input<'_>>) -> Result<BankImportIr> {
    let h = header_input.bytes;
    let total = h
        .len()
        .checked_add(body_input.as_ref().map_or(0, |b| b.bytes.len()))
        .ok_or_else(|| Error {
            part: 0,
            offset: 0,
            reason: "source size overflow".into(),
        })?;
    if total > MAX_SOURCE_BYTES || h.len() < 32 {
        return fail(
            0,
            0,
            "source must contain 32-byte header and total at most 4 MiB",
        );
    }
    for (part, input) in std::iter::once(&header_input)
        .chain(body_input.iter())
        .enumerate()
    {
        if input.label.len() > 4096 || input.rights.len() > 4096 {
            return fail(part, 0, "provenance field exceeds 4096 bytes");
        }
    }
    if &h[..4] != b"pBAV" {
        return fail(0, 0, "expected pBAV signature");
    }
    if u32le(h, 4) != 7 {
        return fail(0, 4, "only verified Sony VAB version 7 is supported");
    }
    let np = u16le(h, 18) as usize;
    let nt = u16le(h, 20) as usize;
    let ns = u16le(h, 22) as usize;
    if !(1..=MAX_SOURCE_PROGRAMS).contains(&np)
        || nt == 0
        || nt > (np * 16).min(MAX_SOURCE_TONES)
        || !(1..=MAX_SOURCE_SAMPLES).contains(&ns)
    {
        return fail(
            0,
            18,
            "source program/tone/sample counts outside verified bounds",
        );
    }
    let header_bytes = 0xa20 + np * 512;
    if h.len() < header_bytes {
        return fail(0, h.len(), "truncated VAB attributes/size table");
    }
    let (body, part, body_offset) = if let Some(b) = &body_input {
        if h.len() != header_bytes {
            return fail(0, header_bytes, "paired VH must end after size table");
        }
        (b.bytes, 1, 0)
    } else {
        (&h[header_bytes..], 0, header_bytes)
    };
    if u32le(h, 12) as usize != total {
        return fail(
            0,
            12,
            "declared combined size differs from exact VH+VB bytes",
        );
    }
    let live: Vec<_> = (0..128).filter(|&p| h[32 + p * 16] != 0).collect();
    if live.len() != np
        || live.iter().any(|&p| h[32 + p * 16] > 16)
        || live.iter().map(|&p| h[32 + p * 16] as usize).sum::<usize>() != nt
    {
        return fail(
            0,
            18,
            "active fixed program rows disagree with exact header counts",
        );
    }
    let table = 0x820 + np * 512;
    if u16le(h, table) != 0 || (ns + 1..256).any(|i| u16le(h, table + i * 2) != 0) {
        return fail(
            0,
            table,
            "unverified sample size table occupancy (1-based IDs required)",
        );
    }
    let sizes: Vec<_> = (1..=ns)
        .map(|i| u16le(h, table + i * 2) as usize * 8)
        .collect();
    if let Some(i) = sizes
        .iter()
        .position(|&size| size == 0 || !size.is_multiple_of(16))
    {
        return fail(
            0,
            table + (i + 1) * 2,
            "sample size must be positive and 16-byte aligned",
        );
    }
    if sizes.iter().sum::<usize>() != body.len() {
        return fail(0, table, "sample sizes do not cover VB exactly");
    }
    let mut diagnostics = Vec::new();
    let mut programs = Vec::with_capacity(np);
    for (compact, &id) in live.iter().enumerate() {
        let at = Span {
            part: 0,
            offset: 32 + id * 16,
            length: 16,
        };
        let raw: [u8; 16] = h[at.offset..at.offset + 16].try_into().unwrap();
        let mut tones = Vec::with_capacity(raw[0] as usize);
        for slot in 0..raw[0] {
            let tone_at = Span {
                part: 0,
                offset: 0x820 + compact * 512 + slot as usize * 32,
                length: 32,
            };
            let t: [u8; 32] = h[tone_at.offset..tone_at.offset + 32].try_into().unwrap();
            let tone = Tone {
                at: tone_at,
                raw: t,
                slot,
                priority: t[0],
                mode: t[1],
                reverb_requested: t[1] & 4 != 0,
                gain: t[2],
                pan: t[3],
                root_key: t[4],
                tuning_raw: t[5],
                keys: [t[6], t[7]],
                vibrato: [t[8], t[9]],
                portamento: [t[10], t[11]],
                bend_down_up: [t[12], t[13]],
                adsr_words: [u16le(&t, 16), u16le(&t, 18)],
                embedded_program: u16le(&t, 20),
                sample_id: u16le(&t, 22),
            };
            if tone.embedded_program as usize != id {
                diag(
                    &mut diagnostics,
                    Scope::Source,
                    "tone-owner",
                    tone_at,
                    "embedded program differs from fixed table row; both retained",
                );
            }
            if tone.sample_id == 0 || tone.sample_id as usize > ns {
                diag(
                    &mut diagnostics,
                    Scope::Source,
                    "sample-reference",
                    tone_at,
                    "sample ID has no supplied waveform; no replacement or silent tone chosen",
                );
            }
            if tone.root_key > 127
                || tone.keys[0] > tone.keys[1]
                || tone.keys[1] > 127
                || tone.gain > 127
                || tone.pan > 127
            {
                diag(
                    &mut diagnostics,
                    Scope::Source,
                    "tone-range",
                    tone_at,
                    "tone numeric ranges need source-specific interpretation; original values retained",
                );
            }
            tones.push(tone);
        }
        let mut overlaps = Vec::new();
        for (i, a) in tones.iter().enumerate() {
            for b in &tones[..i] {
                if a.keys[0] <= b.keys[1] && b.keys[0] <= a.keys[1] {
                    overlaps.push([b.slot, a.slot]);
                }
            }
        }
        programs.push(Program {
            at,
            raw,
            id: id as u8,
            compact_slot: compact as u8,
            gain: raw[1],
            priority: raw[2],
            mode: raw[3],
            pan: raw[4],
            attributes: u16le(&raw, 6),
            tones,
            overlapping_tone_pairs: overlaps,
        });
    }
    let mut samples = Vec::with_capacity(ns);
    let mut cursor = 0;
    let mut decoded_frames = 0;
    for (i, size) in sizes.into_iter().enumerate() {
        let at = Span {
            part,
            offset: body_offset + cursor,
            length: size,
        };
        let data = &body[cursor..cursor + size];
        let decoded = decode_spu_adpcm(data, at, MAX_DECODED_FRAMES - decoded_frames)?;
        decoded_frames += decoded.pcm.len();
        samples.push(Sample {
            id: (i + 1) as u16,
            encoded: at,
            encoded_sha256: sha256(data),
            original_sample_rate_hz: None,
            decoded,
        });
        cursor += size;
    }
    let header = Header {
        raw: h[..32].try_into().unwrap(),
        version: 7,
        bank_id: u32le(h, 8),
        declared_bytes: u32le(h, 12),
        program_count: np as u16,
        tone_count: nt as u16,
        sample_count: ns as u16,
        master_gain: h[24],
        master_pan: h[25],
        attributes: [h[26], h[27]],
    };
    let sources = std::iter::once(header_input)
        .chain(body_input)
        .map(|i| Source {
            label: i.label.into(),
            rights: i.rights.into(),
            sha256: sha256(i.bytes),
            bytes: i.bytes.to_vec(),
        })
        .collect();
    Ok(BankImportIr {
        schema_version: 1,
        profile: "sony-vab-v7".into(),
        sources,
        header,
        programs,
        samples,
        diagnostics,
    })
}

/// Integer first-pass decode, zero initial history, no gain/ADSR/pitch/interpolation.
/// Caller supplies an origin span; limits are enforced independently of VAB parsing.
pub fn decode_spu_adpcm(bytes: &[u8], origin: Span, max_frames: usize) -> Result<Decoded> {
    if bytes.is_empty()
        || bytes.len() > MAX_SOURCE_BYTES
        || !bytes.len().is_multiple_of(16)
        || origin.length != bytes.len()
        || origin.offset.checked_add(bytes.len()).is_none()
    {
        return fail(
            origin.part,
            origin.offset,
            "invalid bounded ADPCM source span",
        );
    }
    let max_frames = max_frames.min(MAX_DECODED_FRAMES);
    let mut blocks = Vec::with_capacity(bytes.len() / 16);
    for (i, b) in bytes.chunks_exact(16).enumerate() {
        if b[0] >> 4 > 4 || b[0] & 15 > 12 {
            return fail(
                origin.part,
                origin.offset + i * 16,
                "unsupported ADPCM filter or reserved shift",
            );
        }
        blocks.push(Block {
            byte_offset: i * 16,
            filter: b[0] >> 4,
            shift: b[0] & 15,
            flags: b[1],
        });
    }
    let terminal = blocks.iter().position(|b| b.flags & 1 != 0);
    let forward_blocks = terminal.map_or(blocks.len(), |i| i + 1);
    let frames = forward_blocks * 28;
    if frames > max_frames {
        return fail(
            origin.part,
            origin.offset,
            "decoded frame budget exceeded before allocation",
        );
    }
    let mut pcm = Vec::with_capacity(frames);
    let mut history = [0_i32; 2];
    let mut loop_start = None;
    let mut loop_history = [0_i16; 2];
    let mut diagnostics = Vec::new();
    if blocks.iter().any(|b| b.flags & !7 != 0) {
        diag(
            &mut diagnostics,
            Scope::Playback,
            "adpcm-unknown-flags",
            origin,
            "upper flag bits are preserved but their hardware meaning is unverified",
        );
    }
    for (i, b) in blocks[..forward_blocks].iter().enumerate() {
        if b.flags & 4 != 0 {
            loop_start = Some(i);
            loop_history = [history[0] as i16, history[1] as i16];
        }
        let coefficients = [(0, 0), (60, 0), (115, -52), (98, -55), (122, -60)][b.filter as usize];
        for packed in &bytes[i * 16 + 2..i * 16 + 16] {
            for nibble in [packed & 15, packed >> 4] {
                let signed = if nibble < 8 {
                    nibble as i32
                } else {
                    nibble as i32 - 16
                };
                let residual = (signed * 4096) >> b.shift;
                let predicted =
                    (history[0] * coefficients.0 + history[1] * coefficients.1 + 32) >> 6;
                let value = (residual + predicted).clamp(-32768, 32767);
                pcm.push(value as i16);
                history = [value, history[0]];
            }
        }
    }
    let termination = match terminal {
        None => Termination::MissingEnd,
        Some(i) if blocks[i].flags & 2 != 0 => Termination::EndRepeat,
        Some(_) => Termination::EndMute,
    };
    let loop_region = if termination == Termination::EndRepeat {
        if let Some(start) = loop_start {
            let region = Loop {
                frames: [(start * 28) as u32, frames as u32],
                start_block: start as u32,
                end_block: (forward_blocks - 1) as u32,
                history_at_first_entry: loop_history,
                history_after_end: [history[0] as i16, history[1] as i16],
                entry_filter: blocks[start].filter,
            };
            // Even equal first-pass histories do not prove future loop stability.
            if region.entry_filter != 0 {
                diag(
                    &mut diagnostics,
                    Scope::Playback,
                    "adpcm-loop-history",
                    origin,
                    "predictive loop entry requires stateful repeated decode or verified rendering policy; static PCM loop may differ",
                );
            }
            Some(region)
        } else {
            diag(
                &mut diagnostics,
                Scope::Playback,
                "adpcm-external-repeat",
                origin,
                "END+REPEAT has no in-sample LOOP START; external repeat address is unknown",
            );
            None
        }
    } else {
        None
    };
    if termination == Termination::MissingEnd {
        diag(
            &mut diagnostics,
            Scope::Playback,
            "adpcm-missing-end",
            origin,
            "no END flag before waveform boundary; hardware would continue outside this sample",
        );
    }
    Ok(Decoded {
        pcm,
        blocks,
        termination,
        loop_region,
        trailing_bytes: bytes.len() - forward_blocks * 16,
        diagnostics,
    })
}

/// Review obligations for the current SoundBank and PSX sequencer. This is not a
/// converter, an acknowledgment mechanism or a claim that an empty list suffices.
pub fn assess_current_sound_bank(bank: &BankImportIr) -> Vec<Diagnostic> {
    let at = Span {
        part: 0,
        offset: 0,
        length: 32,
    };
    let mut out = bank.diagnostics.clone();
    let tones: usize = bank.programs.iter().map(|p| p.tones.len()).sum();
    if tones > CURRENT_SOUNDBANK_ZONES {
        diag(
            &mut out,
            Scope::Authoring,
            "authoring-zone-limit",
            at,
            "source exceeds current 128-zone authoring validator; retain source IR, never truncate",
        );
        diag(
            &mut out,
            Scope::Playback,
            "psx-zone-limit",
            at,
            "current PSX profile requires at most 128 cooked zones; raising authoring alone does not raise this profile",
        );
    }
    for (code, scope, message) in [
        (
            "sample-rate-tuning",
            Scope::Authoring,
            "raw waveform has no original Hz; choose and verify reference-rate and Sony root/fine tuning conversion explicitly",
        ),
        (
            "gain-pan-law",
            Scope::Playback,
            "master/program/tone gain and pan stages require a verified combination law; preserve all stages",
        ),
        (
            "adsr-law",
            Scope::Playback,
            "raw SPU ADSR words do not map exactly to current portable linear milliseconds/sustain envelope",
        ),
        (
            "channel-10-policy",
            Scope::Authoring,
            "Sony banks do not assert GM channel-10 percussion; existing SoundBank drum-key lookup needs explicit sequence mapping",
        ),
    ] {
        diag(&mut out, scope, code, at, message);
    }
    for p in &bank.programs {
        if !p.overlapping_tone_pairs.is_empty() {
            diag(
                &mut out,
                Scope::Authoring,
                "authoring-layers",
                p.at,
                "current authoring validation rejects overlapping zones; every source tone remains in IR",
            );
            diag(
                &mut out,
                Scope::Playback,
                "psx-layers",
                p.at,
                "current playback selects one zone; implementing layers requires voice accounting and all-tone note-off behavior",
            );
        }
        if p.mode != 0 || p.priority != 0 || p.attributes != 0 {
            diag(
                &mut out,
                Scope::Playback,
                "program-attributes",
                p.at,
                "program mode, priority or attributes require verified playback mapping",
            );
        }
        for t in &p.tones {
            if t.reverb_requested {
                diag(
                    &mut out,
                    Scope::Playback,
                    "reverb",
                    t.at,
                    "source requests reverb but current bank has no matching reverb routing/preset",
                );
            }
            if t.bend_down_up != [2, 2] {
                diag(
                    &mut out,
                    Scope::Playback,
                    "bend-range",
                    t.at,
                    "source asymmetric bend differs from current fixed +/-2 semitones",
                );
            }
            if t.mode & !4 != 0 || t.priority != 0 || t.vibrato != [0, 0] || t.portamento != [0, 0]
            {
                diag(
                    &mut out,
                    Scope::Playback,
                    "tone-modulation-priority",
                    t.at,
                    "tone mode, priority, vibrato or portamento require explicit playback support",
                );
            }
        }
    }
    for s in &bank.samples {
        out.extend(s.decoded.diagnostics.iter().cloned());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn put16(b: &mut [u8], at: usize, n: u16) {
        b[at..at + 2].copy_from_slice(&n.to_le_bytes());
    }
    fn put32(b: &mut [u8], at: usize, n: u32) {
        b[at..at + 4].copy_from_slice(&n.to_le_bytes());
    }
    fn block(header: u8, flags: u8, fill: u8) -> Vec<u8> {
        let mut b = vec![fill; 16];
        b[0] = header;
        b[1] = flags;
        b
    }
    fn decode(b: &[u8]) -> Result<Decoded> {
        decode_spu_adpcm(
            b,
            Span {
                part: 1,
                offset: 0,
                length: b.len(),
            },
            1024,
        )
    }
    fn input(b: &[u8]) -> Input<'_> {
        Input {
            bytes: b,
            label: "original synthetic fixture",
            rights: "Epok repository license; original generated test bytes",
        }
    }
    fn fixture(counts: &[(u8, u8)], samples: &[Vec<u8>]) -> Vec<u8> {
        let size = 0xa20 + counts.len() * 512;
        let mut b = vec![0; size];
        b[..4].copy_from_slice(b"pBAV");
        put32(&mut b, 4, 7);
        put32(&mut b, 8, 123);
        put32(
            &mut b,
            12,
            (size + samples.iter().map(Vec::len).sum::<usize>()) as u32,
        );
        put16(&mut b, 18, counts.len() as u16);
        put16(&mut b, 20, counts.iter().map(|p| p.1 as u16).sum());
        put16(&mut b, 22, samples.len() as u16);
        b[24] = 127;
        b[25] = 64;
        for (compact, &(id, count)) in counts.iter().enumerate() {
            let p = 32 + id as usize * 16;
            b[p] = count;
            b[p + 1] = 127;
            b[p + 4] = 64;
            for tone in 0..count as usize {
                let t = 0x820 + compact * 512 + tone * 32;
                b[t + 2] = 127;
                b[t + 3] = 64;
                b[t + 4] = 60;
                b[t + 7] = 127;
                b[t + 12] = 2;
                b[t + 13] = 2;
                put16(&mut b, t + 20, id as u16);
                put16(&mut b, t + 22, 1);
            }
        }
        for (i, s) in samples.iter().enumerate() {
            put16(
                &mut b,
                0x820 + counts.len() * 512 + (i + 1) * 2,
                (s.len() / 8) as u16,
            );
        }
        for sample in samples {
            b.extend(sample);
        }
        b
    }
    #[test]
    fn preserves_sparse_ids_all_fields_and_exact_source() {
        let mut b = fixture(&[(5, 2), (127, 1)], &[block(12, 1, 0)]);
        b[0x820..0x830]
            .copy_from_slice(&[3, 4, 99, 21, 71, 57, 40, 90, 2, 3, 4, 5, 12, 0, 0xaa, 0xbb]);
        put16(&mut b, 0x830, 0x89ab);
        put16(&mut b, 0x832, 0xcdef);
        let ir = parse(input(&b), None).unwrap();
        assert_eq!(ir.sources[0].bytes, b);
        assert_eq!(ir.sources[0].sha256, sha256(&b));
        assert_eq!(
            ir.programs.iter().map(|p| p.id).collect::<Vec<_>>(),
            [5, 127]
        );
        let t = &ir.programs[0].tones[0];
        assert_eq!(
            (
                t.priority,
                t.reverb_requested,
                t.tuning_raw,
                t.bend_down_up,
                t.adsr_words,
                t.embedded_program,
                t.sample_id
            ),
            (3, true, 57, [12, 0], [0x89ab, 0xcdef], 5, 1)
        );
        assert_eq!(ir.programs[0].overlapping_tone_pairs, [[0, 1]]);
        assert_eq!(ir.samples[0].original_sample_rate_hz, None);
    }
    #[test]
    fn explicit_split_pair_has_equal_musical_data_and_correct_provenance() {
        let b = fixture(&[(4, 1)], &[block(12, 1, 0x21)]);
        let a = parse(input(&b), None).unwrap();
        let s = parse(input(&b[..0xc20]), Some(input(&b[0xc20..]))).unwrap();
        assert_eq!(a.header, s.header);
        assert_eq!(a.programs, s.programs);
        assert_eq!(a.samples[0].decoded, s.samples[0].decoded);
        assert_eq!(a.samples[0].encoded_sha256, s.samples[0].encoded_sha256);
        assert_eq!(
            s.samples[0].encoded,
            Span {
                part: 1,
                offset: 0,
                length: 16
            }
        );
        assert_eq!(s.sources.len(), 2);
        assert!(parse(input(&b[0xc20..]), None).is_err());
        assert!(parse(input(&b[..0xc20 + 1]), Some(input(&b[0xc20 + 1..]))).is_err());
    }
    #[test]
    fn source_2048_tones_survive_current_128_zone_profile() {
        let counts: Vec<_> = (0..128).map(|id| (id, 16)).collect();
        let ir = parse(input(&fixture(&counts, &[block(12, 1, 0)])), None).unwrap();
        assert_eq!(
            ir.programs.iter().map(|p| p.tones.len()).sum::<usize>(),
            MAX_SOURCE_TONES
        );
        assert_eq!(
            ir.programs
                .iter()
                .map(|p| p.overlapping_tone_pairs.len())
                .sum::<usize>(),
            15360
        );
        let d = assess_current_sound_bank(&ir);
        assert!(
            d.iter()
                .any(|d| d.code == "authoring-zone-limit" && d.scope == Scope::Authoring)
        );
        assert!(
            d.iter()
                .any(|d| d.code == "psx-zone-limit" && d.scope == Scope::Playback)
        );
    }
    #[test]
    fn unresolved_references_remain_visible_and_never_remapped() {
        let mut b = fixture(&[(0, 1)], &[block(12, 1, 0)]);
        put16(&mut b, 0x834, 8);
        put16(&mut b, 0x836, 0);
        b[0x826] = 127;
        b[0x827] = 3;
        let ir = parse(input(&b), None).unwrap();
        assert_eq!(ir.programs[0].tones[0].sample_id, 0);
        assert_eq!(
            ir.diagnostics
                .iter()
                .map(|d| d.code.as_str())
                .collect::<Vec<_>>(),
            ["tone-owner", "sample-reference", "tone-range"]
        );
    }
    #[test]
    fn nibble_order_sign_shift_and_saturation_have_known_answers() {
        let d = decode(&block(12, 1, 0x87)).unwrap();
        assert_eq!(&d.pcm[..4], &[7, -8, 7, -8]);
        let d = decode(&block(0, 1, 0x87)).unwrap();
        assert_eq!(&d.pcm[..2], &[28672, -32768]);
        let d = decode(&block(0x10, 1, 0x77)).unwrap();
        assert_eq!(&d.pcm[..3], &[28672, 32767, 32767]);
        let d = decode(&block(0x10, 1, 0x88)).unwrap();
        assert_eq!(&d.pcm[..3], &[-32768, -32768, -32768]);
    }
    #[test]
    fn filters_and_negative_rounding_preserve_cross_block_history() {
        // Hand-derived from histories [1,2], residual zero, (a*h0+b*h1+32)>>6.
        for (filter, expected) in [
            (0, [0, 0]),
            (1, [1, 1]),
            (2, [0, -1]),
            (3, [0, -1]),
            (4, [0, -1]),
        ] {
            let mut a = block(12, 0, 0);
            a[15] = 0x12;
            a.extend(block((filter << 4) | 12, 1, 0));
            let d = decode(&a).unwrap();
            assert_eq!(&d.pcm[28..30], &expected);
        }
    }
    #[test]
    fn end_block_is_decoded_trailers_retained_and_samples_reset_history() {
        let mut s = block(12, 1, 0x77);
        s.extend(block(12, 7, 0));
        let d = decode(&s).unwrap();
        assert_eq!(d.pcm.len(), 28);
        assert_eq!(d.trailing_bytes, 16);
        assert_eq!(d.blocks.len(), 2);
        assert_eq!(d.termination, Termination::EndMute);
        assert!(d.loop_region.is_none());
        let b = fixture(&[(0, 1)], &[s, block(0x1c, 1, 0)]);
        let ir = parse(input(&b), None).unwrap();
        assert!(ir.samples[1].decoded.pcm.iter().all(|&v| v == 0));
    }
    #[test]
    fn loop_flags_keep_latest_start_half_open_end_and_predictor_warning() {
        let mut b = block(12, 4, 0x11);
        b.extend(block(0x1c, 6, 0x22));
        b.extend(block(12, 3, 0x33));
        let d = decode(&b).unwrap();
        let l = d.loop_region.unwrap();
        assert_eq!(l.frames, [28, 84]);
        assert_eq!(l.start_block, 1);
        assert_eq!(l.end_block, 2);
        assert_eq!(l.history_at_first_entry, [1, 1]);
        assert_eq!(l.history_after_end, [3, 3]);
        assert!(d.diagnostics.iter().any(|d| d.code == "adpcm-loop-history"));
        let single = decode(&block(12, 7, 0)).unwrap();
        assert_eq!(single.loop_region.unwrap().frames, [0, 28]);
    }
    #[test]
    fn incomplete_loop_and_unknown_flags_diagnose_without_guessing() {
        let d = decode(&block(12, 3, 0)).unwrap();
        assert!(d.loop_region.is_none());
        assert_eq!(d.diagnostics[0].code, "adpcm-external-repeat");
        let d = decode(&block(12, 0x82, 0)).unwrap();
        assert_eq!(d.termination, Termination::MissingEnd);
        assert_eq!(d.diagnostics.len(), 2);
    }
    #[test]
    fn structural_counts_sizes_versions_and_truncations_are_rejected() {
        let good = fixture(&[(0, 1)], &[block(12, 1, 0)]);
        for length in 0..good.len() {
            assert!(parse(input(&good[..length]), None).is_err());
        }
        for (offset, value) in [
            (0, b'V'),
            (4, 6),
            (18, 0),
            (19, 1),
            (20, 0),
            (22, 255),
            (32, 17),
            (0xa20, 1),
            (0xa22, 1),
            (0xa24, 2),
        ] {
            let mut b = good.clone();
            b[offset] = value;
            assert!(parse(input(&b), None).is_err(), "offset {offset:x}");
        }
        let mut b = good.clone();
        b.push(0);
        let n = b.len() as u32;
        put32(&mut b, 12, n);
        assert!(parse(input(&b), None).is_err());
        assert!(parse(input(&vec![0; MAX_SOURCE_BYTES + 1]), None).is_err());
    }
    #[test]
    fn decoder_bounds_bad_coding_and_unreachable_tail_are_checked() {
        assert!(decode(&[]).is_err());
        assert!(decode(&[0; 15]).is_err());
        for h in [0x50, 13, 14, 15] {
            assert!(decode(&block(h, 1, 0)).is_err());
        }
        let mut b = block(12, 1, 0);
        b.extend(block(0x50, 7, 0));
        assert!(decode(&b).is_err());
        let b = block(12, 1, 0);
        let span = Span {
            part: 0,
            offset: 0,
            length: 16,
        };
        assert!(decode_spu_adpcm(&b, span, 27).is_err());
        assert!(decode_spu_adpcm(&b, span, 28).is_ok());
        assert!(
            decode_spu_adpcm(
                &b,
                Span {
                    offset: usize::MAX,
                    ..span
                },
                28
            )
            .is_err()
        );
    }
    #[test]
    fn deterministic_serialization_and_bounded_corruptions_do_not_panic() {
        let good = fixture(&[(0, 2)], &[block(12, 7, 0)]);
        let first = parse(input(&good), None).unwrap();
        let json = serde_json::to_vec(&first).unwrap();
        let restored: BankImportIr = serde_json::from_slice(&json).unwrap();
        assert_eq!(restored, first);
        assert_eq!(
            json,
            serde_json::to_vec(&parse(input(&good), None).unwrap()).unwrap()
        );
        assert_eq!(
            assess_current_sound_bank(&first),
            assess_current_sound_bank(&first)
        );
        let mut rng = 0x5eed_u32;
        for _ in 0..2048 {
            let mut b = good.clone();
            for _ in 0..4 {
                rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                let at = rng as usize % b.len();
                b[at] ^= (rng >> 24) as u8;
            }
            let _ = parse(input(&b), None);
        }
    }
}
