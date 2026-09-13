//! Bounded, identified Sony SEQ/SEP and converted SEQ/SEP (LE32) source adapters.
//! Original implementation; see the phase-E format audit for evidence and limits.
use crate::sequence_ir::{Diagnostic, Event, EventKind, MAX_DIAGNOSTICS, MAX_EVENTS, SequenceIr};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const MAX_BYTES: usize = 4 * 1024 * 1024;
const MAX_SEQUENCES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Profile {
    #[serde(rename = "sony-seq-v1")]
    SonySeqV1,
    #[serde(rename = "sony-sep-v0")]
    SonySepV0,
    // Read the initial experimental metadata identifier; always write the neutral name.
    #[serde(rename = "converted-seq-le32-v1", alias = "legacy-seqcomb-sepcomb-v1")]
    ConvertedSeqLe32V1,
}

impl Profile {
    pub fn from_id(value: &str) -> Result<Self, String> {
        Self::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value))
            .map_err(|_| format!("Unknown sequence source profile {value}"))
    }
    pub fn id(self) -> &'static str {
        match self {
            Self::SonySeqV1 => "sony-seq-v1",
            Self::SonySepV0 => "sony-sep-v0",
            Self::ConvertedSeqLe32V1 => "converted-seq-le32-v1",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Entry {
    pub id: u16,
    pub offset: usize,
    pub length: usize,
    pub header: Header,
    pub ir: SequenceIr,
    /// Numeric source facts survive independently of the chosen playback loop policy.
    pub source_loop: Option<SourceLoop>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLoop {
    pub count: u8,
    pub channel: u8,
    pub command_start_tick: u32,
    pub first_repeated_tick: u32,
    pub end_tick: u32,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Serialize)]
pub struct Container {
    pub profile: Profile,
    /// Independent songs. A SEP is not a set of simultaneous SMF tracks.
    pub entries: Vec<Entry>,
}

fn fail(at: usize, message: impl std::fmt::Display) -> String {
    format!("Sequence source at offset 0x{at:x}: {message}")
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    base: usize,
}

impl<'a> Reader<'a> {
    fn offset(&self) -> usize {
        self.base + self.at
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self
            .at
            .checked_add(count)
            .ok_or_else(|| fail(self.offset(), "length overflow"))?;
        let result = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| fail(self.offset(), "truncated data"))?;
        self.at = end;
        Ok(result)
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn data(&mut self) -> Result<u8, String> {
        let at = self.offset();
        let value = self.byte()?;
        if value > 127 {
            return Err(fail(at, "channel parameter has its status bit set"));
        }
        Ok(value)
    }
    fn be16(&mut self) -> Result<u16, String> {
        let b = self.take(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }
    fn be24(&mut self) -> Result<u32, String> {
        let b = self.take(3)?;
        Ok(u32::from_be_bytes([0, b[0], b[1], b[2]]))
    }
    fn be32(&mut self) -> Result<u32, String> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn le16(&mut self) -> Result<u16, String> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn le32(&mut self) -> Result<u32, String> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn vlq(&mut self) -> Result<u32, String> {
        let at = self.offset();
        let mut result = 0;
        for _ in 0..4 {
            let value = self.byte()?;
            result = (result << 7) | u32::from(value & 127);
            if value < 128 {
                return Ok(result);
            }
        }
        Err(fail(at, "VLQ exceeds four bytes"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    pub ppqn: u16,
    pub tempo: u32,
    pub numerator: u8,
    pub denominator: u8,
}

impl Header {
    fn sony(input: &mut Reader<'_>) -> Result<Self, String> {
        let value = Self {
            ppqn: input.be16()?,
            tempo: input.be24()?,
            numerator: input.byte()?,
            denominator: input.byte()?,
        };
        value.validate(input.offset())?;
        Ok(value)
    }
    fn validate(&self, at: usize) -> Result<(), String> {
        if self.ppqn == 0
            || self.ppqn & 0x8000 != 0
            || !(1..=0xffffff).contains(&self.tempo)
            || self.numerator == 0
            || self.denominator > 7
        {
            return Err(fail(at, "invalid PPQN, tempo or time signature"));
        }
        Ok(())
    }
}

/// A full structural probe, using no extension. It never chooses arbitrarily
/// between the overlapping SEQ-v1 and SEP-v0 / first-ID-1 byte prefixes.
pub fn detect(bytes: &[u8]) -> Result<Profile, String> {
    let candidates: &[Profile] = if bytes.starts_with(b"pQES") {
        &[Profile::SonySeqV1, Profile::SonySepV0]
    } else {
        &[Profile::ConvertedSeqLe32V1]
    };
    let mut accepted = Vec::new();
    let mut errors = Vec::new();
    for &profile in candidates {
        match parse(bytes, profile) {
            Ok(_) => accepted.push(profile),
            Err(error) => errors.push(format!("{}: {error}", profile.id())),
        }
    }
    match accepted.as_slice() {
        [profile] => Ok(*profile),
        [] => Err(errors.join("; ")),
        _ => Err(fail(
            0,
            "Ambiguous Sony sequence structure; choose the verified source profile explicitly",
        )),
    }
}

pub fn parse(bytes: &[u8], profile: Profile) -> Result<Container, String> {
    if bytes.is_empty() || bytes.len() > MAX_BYTES {
        return Err(fail(0, "empty source or source exceeds 4 MiB"));
    }
    let mut input = Reader {
        bytes,
        at: 0,
        base: 0,
    };
    let mut entries = Vec::new();
    let mut ids = BTreeSet::new();
    let mut total_events = 0;
    match profile {
        Profile::SonySeqV1 => {
            if input.take(4)? != b"pQES" || input.be32()? != 1 {
                return Err(fail(0, "expected pQES and big-endian SEQ version 1"));
            }
            let header = Header::sony(&mut input)?;
            let start = input.at;
            let (ir, source_loop, consumed) =
                score(&bytes[start..], start, header, profile, &mut total_events)?;
            if start + consumed != bytes.len() {
                return Err(fail(start + consumed, "trailing Sony SEQ bytes"));
            }
            entries.push(Entry {
                id: 0,
                offset: 0,
                length: bytes.len(),
                header,
                ir,
                source_loop,
            });
            input.at = bytes.len();
        }
        Profile::SonySepV0 => {
            if input.take(4)? != b"pQES" || input.be16()? != 0 {
                return Err(fail(0, "expected pQES and big-endian SEP version 0"));
            }
            while input.at < bytes.len() {
                if entries.len() == MAX_SEQUENCES {
                    return Err(fail(input.at, "more than 256 independent sequences"));
                }
                let offset = input.at;
                let id = input.be16()?;
                if !ids.insert(id) {
                    return Err(fail(offset, "duplicate Sony SEP sequence ID"));
                }
                let header = Header::sony(&mut input)?;
                let size = input.be32()? as usize;
                let start = input.at;
                let source = input.take(size)?;
                let (ir, source_loop, consumed) =
                    score(source, start, header, profile, &mut total_events)?;
                if consumed != size {
                    return Err(fail(
                        start + consumed,
                        "trailing data inside Sony SEP score",
                    ));
                }
                entries.push(Entry {
                    id,
                    offset,
                    length: input.at - offset,
                    header,
                    ir,
                    source_loop,
                });
            }
        }
        Profile::ConvertedSeqLe32V1 => {
            while input.at < bytes.len() {
                if entries.len() == MAX_SEQUENCES {
                    return Err(fail(input.at, "more than 256 independent sequences"));
                }
                let offset = input.at;
                let size = input.le32()? as usize;
                if size < 16 || !size.is_multiple_of(4) {
                    return Err(fail(
                        offset,
                        "Converted SEQ/SEP record size must be at least 16 and divisible by four",
                    ));
                }
                let header = Header {
                    tempo: input.le32()?,
                    ppqn: input.le16()?,
                    numerator: input.byte()?,
                    denominator: input.byte()?,
                };
                header.validate(offset + 4)?;
                let start = input.at;
                let source = input.take(size - 12)?;
                let (ir, source_loop, consumed) =
                    score(source, start, header, profile, &mut total_events)?;
                let padding = &source[consumed..];
                if padding.len() > 3 || padding.iter().any(|&b| b != 0) {
                    return Err(fail(
                        start + consumed,
                        "Converted SEQ/SEP permits only zero alignment padding up to three bytes",
                    ));
                }
                entries.push(Entry {
                    id: entries.len() as u16,
                    offset,
                    length: size,
                    header,
                    ir,
                    source_loop,
                });
            }
        }
    }
    if entries.is_empty() {
        return Err(fail(input.at, "sequence container is empty"));
    }
    Ok(Container { profile, entries })
}

#[derive(Clone)]
struct Control {
    tick: u32,
    order: u32,
    offset: usize,
    next_tick: u32,
    channel: u8,
    number: u8,
    value: u8,
}

fn diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    tick: u32,
    offset: usize,
    message: impl std::fmt::Display,
) -> Result<(), String> {
    if diagnostics.len() == MAX_DIAGNOSTICS {
        return Err(fail(offset, "diagnostic capacity exceeds 4096"));
    }
    diagnostics.push(Diagnostic {
        track: 0,
        tick,
        message: fail(offset, message),
        unsupported: true,
    });
    Ok(())
}

fn push(
    events: &mut Vec<Event>,
    tick: u32,
    order: u32,
    kind: EventKind,
    offset: usize,
) -> Result<(), String> {
    if events.len() == MAX_EVENTS {
        return Err(fail(offset, "neutral event capacity exceeds 65536"));
    }
    events.push(Event {
        tick,
        track: 0,
        order,
        kind,
    });
    Ok(())
}

fn score(
    bytes: &[u8],
    base: usize,
    header: Header,
    profile: Profile,
    total: &mut usize,
) -> Result<(SequenceIr, Option<SourceLoop>, usize), String> {
    let mut input = Reader { bytes, at: 0, base };
    let mut events = Vec::new();
    let mut diagnostics = Vec::new();
    let mut controls: Vec<Control> = Vec::new();
    let mut source_events = Vec::new();
    push(&mut events, 0, 0, EventKind::Tempo(header.tempo), base)?;
    push(
        &mut events,
        0,
        1,
        EventKind::TimeSignature {
            numerator: header.numerator,
            denominator_power: header.denominator,
            clocks: 24,
            thirty_seconds: 8,
        },
        base,
    )?;
    let (mut tick, mut order, mut running) = (0_u32, 2_u32, None);
    let mut ended = false;
    let post_delta = profile == Profile::ConvertedSeqLe32V1;
    while input.at < bytes.len() {
        *total += 1;
        if *total > MAX_EVENTS {
            return Err(fail(
                input.offset(),
                "container exceeds 65536 source events",
            ));
        }
        if !post_delta {
            tick = tick
                .checked_add(input.vlq()?)
                .ok_or_else(|| fail(input.offset(), "tick overflow"))?;
            if let Some(control) = controls.last_mut().filter(|c| c.order + 1 == order) {
                control.next_tick = tick;
            }
        }
        let at = input.offset();
        let first = input.byte()?;
        let status = if first >= 128 {
            running = Some(first);
            first
        } else {
            input.at -= 1;
            running.ok_or_else(|| fail(at, "running status before explicit status"))?
        };
        let data_start = input.at;
        let kind = match status {
            0x80..=0xef => {
                let channel = status & 15;
                let a = input.data()?;
                let b = if matches!(status >> 4, 12 | 13) {
                    0
                } else {
                    input.data()?
                };
                match status >> 4 {
                    8 => Some(EventKind::NoteOff { channel, key: a }),
                    9 if b == 0 => Some(EventKind::NoteOff { channel, key: a }),
                    9 => Some(EventKind::NoteOn {
                        channel,
                        key: a,
                        velocity: b,
                    }),
                    11 if [7, 10, 11, 64].contains(&a) => Some(EventKind::Control {
                        channel,
                        controller: a,
                        value: b,
                    }),
                    11 => {
                        controls.push(Control {
                            tick,
                            order,
                            offset: at,
                            next_tick: tick,
                            channel,
                            number: a,
                            value: b,
                        });
                        None
                    }
                    12 => Some(EventKind::Program {
                        channel,
                        program: a,
                    }),
                    14 => Some(EventKind::Bend {
                        channel,
                        value: u16::from(a) | (u16::from(b) << 7),
                    }),
                    _ => {
                        diagnostic(
                            &mut diagnostics,
                            tick,
                            at,
                            format!("unsupported aftertouch status {status:02x}, values {a}, {b}"),
                        )?;
                        None
                    }
                }
            }
            0xff => match input.byte()? {
                0x51 => {
                    let value = input.be24()?;
                    if value == 0 {
                        return Err(fail(at, "zero score tempo"));
                    }
                    Some(EventKind::Tempo(value))
                }
                0x2f => {
                    if input.byte()? != 0 {
                        return Err(fail(at, "EOT terminator must be zero"));
                    }
                    ended = true;
                    Some(EventKind::EndTrack)
                }
                meta => {
                    return Err(fail(
                        at,
                        format!(
                            "unsupported score meta {meta:02x}; length is not SMF-encoded and cannot be guessed"
                        ),
                    ));
                }
            },
            _ => {
                return Err(fail(
                    at,
                    format!("unsupported system status {status:02x}; encoded length is unverified"),
                ));
            }
        };
        source_events.push(crate::sequence_ir::SourceEvent {
            tick,
            track: 0,
            order,
            offset: at,
            status,
            explicit_status: first >= 128,
            data: bytes[data_start..input.at].to_vec(),
        });
        if let Some(kind) = kind {
            push(&mut events, tick, order, kind, at)?;
        }
        if ended {
            break;
        }
        if post_delta {
            tick = tick
                .checked_add(input.vlq()?)
                .ok_or_else(|| fail(input.offset(), "tick overflow"))?;
            if let Some(control) = controls.last_mut().filter(|c| c.order == order) {
                control.next_tick = tick;
            }
        }
        order += 1;
    }
    if !ended {
        return Err(fail(input.offset(), "missing end-of-track"));
    }
    let source_loop = lower_loop(&controls, &mut events, &mut diagnostics, profile)?;
    if source_events
        .iter()
        .any(|e| e.status == 0x99 && e.data.get(1).is_some_and(|v| *v != 0))
    {
        diagnostic(
            &mut diagnostics,
            0,
            base,
            "Sony channel 10 has no declared GM percussion policy. Supply an explicit mapping before playback; the MIDI drum fallback is not applied.",
        )?;
    }
    let blockers = diagnostics
        .iter()
        .filter(|d| d.unsupported)
        .cloned()
        .collect();
    let mut ir =
        SequenceIr::analyze(header.ppqn, events, diagnostics).map_err(|e| fail(base, e))?;
    ir.source_profile = Some(profile.id().into());
    ir.source_events = source_events;
    ir.playback_blockers = blockers;
    Ok((ir, source_loop, input.at))
}

fn lower_loop(
    controls: &[Control],
    events: &mut Vec<Event>,
    diagnostics: &mut Vec<Diagnostic>,
    profile: Profile,
) -> Result<Option<SourceLoop>, String> {
    let starts: Vec<_> = controls
        .iter()
        .filter(|c| c.number == 99 && c.value == 20)
        .collect();
    let ends: Vec<_> = controls
        .iter()
        .filter(|c| c.number == 99 && c.value == 30)
        .collect();
    let data: Vec<_> = controls.iter().filter(|c| c.number == 6).collect();
    let candidate = match (starts.as_slice(), ends.as_slice(), data.as_slice()) {
        ([start], [end], [count])
            if start.channel == end.channel
                && start.channel == count.channel
                && start.tick == count.tick
                && start.order < end.order
                && start.order.abs_diff(count.order) == 1 =>
        {
            Some((*start, *end, *count))
        }
        _ => None,
    };
    let mut consumed = BTreeSet::new();
    let mut result = None;
    if let Some((start, end, count)) = candidate {
        let first = if profile == Profile::ConvertedSeqLe32V1 {
            // The converter/runtime points past the control AND its following
            // delta. Recording the CC tick would add 1 tick in 74 corpus loops.
            if start.order > count.order {
                start.next_tick
            } else {
                count.next_tick
            }
        } else {
            // Sony compatibility needs a separate libSnd trace before claiming
            // exact controller, tail and first-delta replay semantics.
            start.tick
        };
        if first < end.tick {
            let facts = SourceLoop {
                count: count.value,
                channel: start.channel,
                command_start_tick: start.tick,
                first_repeated_tick: first,
                end_tick: end.tick,
                start_offset: start.offset,
                end_offset: end.offset,
            };
            result = Some(facts);
            if count.value == 127 {
                push(
                    events,
                    first,
                    start.order,
                    EventKind::LoopStart,
                    start.offset,
                )?;
                push(events, end.tick, end.order, EventKind::LoopEnd, end.offset)?;
                diagnostic(
                    diagnostics,
                    start.tick,
                    start.offset,
                    "Compatibility loop would use Epok's cut-tail/controller-reset policy. Its source semantics are preserved; playback requires a verified loop resolver and cannot use Ignore unsupported MIDI.",
                )?;
            } else {
                diagnostic(
                    diagnostics,
                    start.tick,
                    start.offset,
                    format!(
                        "Finite compatibility loop count {} is preserved as source intent but cannot execute in the current infinite-region kernel",
                        count.value
                    ),
                )?;
            }
            consumed.extend([start.offset, end.offset, count.offset]);
        }
    }
    for control in controls.iter().filter(|c| !consumed.contains(&c.offset)) {
        diagnostic(
            diagnostics,
            control.tick,
            control.offset,
            format!(
                "Unsupported compatibility CC {}={}, channel {}; bank mutation/mark callbacks/reverb/RPN/NRPN are not discarded silently",
                control.number,
                control.value,
                control.channel + 1
            ),
        )?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn seq(score: &[u8]) -> Vec<u8> {
        let mut bytes = b"pQES\0\0\0\x01\0\x60\x07\xa1\x20\x04\x02".to_vec();
        bytes.extend_from_slice(score);
        bytes
    }
    fn converted_seq(score: &[u8]) -> Vec<u8> {
        let size = (12 + score.len() + 3) & !3;
        let mut bytes = (size as u32).to_le_bytes().to_vec();
        bytes.extend_from_slice(&500_000_u32.to_le_bytes());
        bytes.extend_from_slice(&[96, 0, 4, 2]);
        bytes.extend_from_slice(score);
        bytes.resize(size, 0);
        bytes
    }
    fn sep(id: u16, score: &[u8]) -> Vec<u8> {
        let mut bytes = b"pQES\0\0".to_vec();
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes.extend_from_slice(&[0, 96, 7, 0xa1, 0x20, 4, 2]);
        bytes.extend_from_slice(&(score.len() as u32).to_be_bytes());
        bytes.extend_from_slice(score);
        bytes
    }

    #[test]
    fn sony_and_converted_notes_lower_to_equal_ir() {
        let a = parse(
            &seq(&[0, 0x90, 60, 100, 96, 60, 0, 0, 0xff, 0x2f, 0]),
            Profile::SonySeqV1,
        )
        .unwrap();
        let b = parse(
            &converted_seq(&[0x90, 60, 100, 96, 60, 0, 0, 0xff, 0x2f, 0]),
            Profile::ConvertedSeqLe32V1,
        )
        .unwrap();
        assert_eq!(a.entries[0].ir.events, b.entries[0].ir.events);
        assert_eq!(a.entries[0].ir.duration_micros, 500_000);
        assert_eq!(b.entries[0].ir.peak_polyphony, 1);
        assert_eq!(
            detect(&seq(&[0, 0xff, 0x2f, 0])).unwrap(),
            Profile::SonySeqV1
        );
        assert_eq!(
            detect(&converted_seq(&[0xff, 0x2f, 0])).unwrap(),
            Profile::ConvertedSeqLe32V1
        );
    }

    #[test]
    fn sony_meta_running_status_and_no_length_byte() {
        let source = seq(&[
            0, 0xff, 0x51, 7, 0xa1, 0x20, 96, 0x51, 15, 0x42, 0x40, 96, 0x2f, 0,
        ]);
        let parsed = parse(&source, Profile::SonySeqV1).unwrap();
        assert_eq!(parsed.entries[0].ir.duration_micros, 1_500_000);
        assert_eq!(parsed.entries[0].ir.duration_ticks, 192);
    }

    #[test]
    fn sep_songs_are_independent_and_bounds_are_exact() {
        let first = sep(1, &[0, 0xff, 0x2f, 0]);
        assert_eq!(detect(&first).unwrap(), Profile::SonySepV0);
        let mut bytes = first.clone();
        bytes.extend_from_slice(&sep(8, &[0, 0x90, 60, 1, 96, 60, 0, 0, 0xff, 0x2f, 0])[6..]);
        let parsed = parse(&bytes, Profile::SonySepV0).unwrap();
        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].id, 1);
        assert_eq!(parsed.entries[1].id, 8);
        assert_eq!(parsed.entries[1].offset, first.len());
        assert_eq!(parsed.entries[1].ir.duration_ticks, 96);
        let mut duplicate = first.clone();
        duplicate.extend_from_slice(&first[6..]);
        assert!(
            parse(&duplicate, Profile::SonySepV0)
                .unwrap_err()
                .contains("duplicate")
        );
        assert!(parse(&bytes[..bytes.len() - 1], Profile::SonySepV0).is_err());
    }

    #[test]
    fn converted_loop_uses_first_repeated_event_and_requires_acknowledgement() {
        let bytes = converted_seq(&[
            0xb0, 6, 127, 0, 99, 20, 1, 0x90, 60, 100, 95, 60, 0, 0, 0xb0, 99, 30, 0, 0xff, 0x2f, 0,
        ]);
        let parsed = parse(&bytes, Profile::ConvertedSeqLe32V1).unwrap();
        let entry = &parsed.entries[0];
        assert_eq!(entry.ir.loop_region, Some([1, 96]));
        assert_eq!(entry.source_loop.as_ref().unwrap().command_start_tick, 0);
        assert_eq!(entry.source_loop.as_ref().unwrap().first_repeated_tick, 1);
        assert!(
            entry
                .ir
                .diagnostics
                .iter()
                .any(|d| d.unsupported && d.message.contains("cut-tail"))
        );
        let finite = converted_seq(&[
            0xb0, 6, 2, 0, 99, 20, 1, 0x90, 60, 100, 95, 60, 0, 0, 0xb0, 99, 30, 0, 0xff, 0x2f, 0,
        ]);
        let parsed = parse(&finite, Profile::ConvertedSeqLe32V1).unwrap();
        assert!(parsed.entries[0].ir.loop_region.is_none());
        assert_eq!(parsed.entries[0].source_loop.as_ref().unwrap().count, 2);
        assert!(
            parsed.entries[0]
                .ir
                .diagnostics
                .iter()
                .any(|d| d.unsupported && d.message.contains("Finite"))
        );
    }

    #[test]
    fn unsupported_musical_parameters_remain_actionable() {
        let bytes = seq(&[0, 0xb0, 0, 3, 0, 0xa0, 60, 64, 0, 0xff, 0x2f, 0]);
        let parsed = parse(&bytes, Profile::SonySeqV1).unwrap();
        assert_eq!(parsed.entries[0].ir.diagnostics.len(), 2);
        assert!(
            parsed.entries[0]
                .ir
                .diagnostics
                .iter()
                .all(|d| d.unsupported && d.message.contains("offset 0x"))
        );
        assert!(
            parse(&seq(&[0, 0xff, 1, 0]), Profile::SonySeqV1)
                .unwrap_err()
                .contains("cannot be guessed")
        );
    }

    #[test]
    fn malformed_headers_vlq_status_and_padding_are_rejected() {
        for data in [
            vec![],
            vec![0; 16],
            converted_seq(&[60, 100, 0xff, 0x2f, 0]),
            converted_seq(&[0x90, 60, 128]),
            seq(&[0x80, 0x80, 0x80, 0x80, 0, 0xff, 0x2f, 0]),
            seq(&[0, 0xff, 0x2f, 1]),
        ] {
            assert!(detect(&data).is_err());
        }
        let mut corrupt = converted_seq(&[0xff, 0x2f, 0]);
        *corrupt.last_mut().unwrap() = 1;
        assert!(
            parse(&corrupt, Profile::ConvertedSeqLe32V1)
                .unwrap_err()
                .contains("padding")
        );
        let mut invalid_ppqn = seq(&[0, 0xff, 0x2f, 0]);
        invalid_ppqn[8] = 0x80;
        assert!(parse(&invalid_ppqn, Profile::SonySeqV1).is_err());
    }

    #[test]
    fn source_event_limit_and_tick_overflow_fail_explicitly() {
        let mut events = Vec::new();
        for _ in 0..MAX_EVENTS {
            events.extend_from_slice(&[0, 0xc0, 0]);
        }
        events.extend_from_slice(&[0, 0xff, 0x2f, 0]);
        assert!(
            parse(&seq(&events), Profile::SonySeqV1)
                .unwrap_err()
                .contains("65536")
        );
        let mut overflow = Vec::new();
        for _ in 0..17 {
            overflow.extend_from_slice(&[0xff, 0xff, 0xff, 0x7f, 0xc0, 0]);
        }
        overflow.extend_from_slice(&[0, 0xff, 0x2f, 0]);
        assert!(
            parse(&seq(&overflow), Profile::SonySeqV1)
                .unwrap_err()
                .contains("tick overflow")
        );
    }

    #[test]
    fn bounded_corrupt_inputs_do_not_panic() {
        let original = converted_seq(&[0x90, 60, 100, 96, 60, 0, 0, 0xff, 0x2f, 0]);
        for length in 0..original.len() {
            let _ = detect(&original[..length]);
        }
        for i in 0..original.len() {
            for value in [0, 1, 127, 128, 255] {
                let mut bytes = original.clone();
                bytes[i] = value;
                let _ = detect(&bytes);
            }
        }
    }
}
