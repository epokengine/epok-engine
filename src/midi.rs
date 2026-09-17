//! Bounded Standard MIDI File discovery. Parsing never publishes an asset.
#[path = "midi_controls.rs"]
mod controls;

use crate::sequence_ir::{
    Diagnostic, Event, EventKind, MAX_DIAGNOSTICS, MAX_EVENTS, SequenceIr, SourceEvent,
};
use serde::{Deserialize, Serialize};

pub const MAX_TRACKS: usize = 256;
pub const MAX_BYTES: usize = 4 * 1024 * 1024;

/// Old assets retain their whitelist; new imports resolve normal MIDI channel state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiProfile {
    LegacyV1,
    #[default]
    MusicalV2,
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn byte(&mut self) -> Result<u8, String> {
        let v = *self.bytes.get(self.at).ok_or("Truncated MIDI event")?;
        self.at += 1;
        Ok(v)
    }
    fn data(&mut self) -> Result<u8, String> {
        let v = self.byte()?;
        if v >= 128 {
            Err("MIDI data byte has its status bit set".into())
        } else {
            Ok(v)
        }
    }
    fn vlq(&mut self) -> Result<u32, String> {
        let mut v = 0;
        for _ in 0..4 {
            let b = self.byte()?;
            v = v << 7 | u32::from(b & 127);
            if b & 128 == 0 {
                return Ok(v);
            }
        }
        Err("MIDI variable-length quantity exceeds four bytes".into())
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(n).ok_or("MIDI event size overflow")?;
        let out = self
            .bytes
            .get(self.at..end)
            .ok_or("Truncated MIDI event data")?;
        self.at = end;
        Ok(out)
    }
}

pub fn parse(bytes: &[u8]) -> Result<SequenceIr, String> {
    parse_with_profile(bytes, MidiProfile::MusicalV2)
}

pub fn parse_with_profile(bytes: &[u8], profile: MidiProfile) -> Result<SequenceIr, String> {
    let header = probe(bytes)?;
    let mut at = 8 + u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let (mut events, mut staged, mut diagnostics, mut ledger) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut count = 0;
    for track in 0..header.tracks {
        let len = u32::from_be_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
        parse_track(
            &bytes[at + 8..at + 8 + len],
            at + 8,
            track,
            profile,
            &mut events,
            &mut staged,
            &mut diagnostics,
            &mut ledger,
            &mut count,
        )
        .map_err(|e| format!("MIDI track {}: {e}", track + 1))?;
        at += 8 + len;
    }
    let events = if profile == MidiProfile::LegacyV1 {
        events
    } else {
        controls::resolve(staged, &mut diagnostics)?
    };
    let blockers = if profile == MidiProfile::MusicalV2 {
        diagnostics
            .iter()
            .filter(|d| d.unsupported)
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    let mut ir = SequenceIr::analyze(header.ppqn, events, diagnostics)?;
    ir.source_events = ledger;
    if profile == MidiProfile::MusicalV2 {
        ir.source_profile = Some("midi-musical-v2".into());
        ir.playback_blockers = blockers;
    }
    Ok(ir)
}

#[allow(clippy::too_many_arguments)]
fn parse_track(
    bytes: &[u8],
    base: usize,
    track: u16,
    profile: MidiProfile,
    events: &mut Vec<Event>,
    staged: &mut Vec<controls::StagedEvent>,
    diagnostics: &mut Vec<Diagnostic>,
    ledger: &mut Vec<SourceEvent>,
    count: &mut usize,
) -> Result<(), String> {
    let mut input = Reader { bytes, at: 0 };
    let (mut tick, mut order, mut running, mut ended) = (0_u32, 0_u32, None, false);
    while input.at < bytes.len() {
        let event_start = input.at;
        *count += 1;
        if *count > MAX_EVENTS {
            return Err("MIDI exceeds 65536 source events".into());
        }
        tick = tick
            .checked_add(input.vlq()?)
            .ok_or("MIDI absolute tick overflow")?;
        let first = input.byte()?;
        let status = if first < 128 {
            input.at -= 1;
            running
                .ok_or("Running status without a preceding channel status (meta/SysEx cancel it)")?
        } else {
            first
        };
        let data_start = input.at;
        let mut diagnostic: Option<(String, bool)> = None;
        let (legacy, musical) = if status < 0xf0 {
            running = Some(status);
            let channel = status & 15;
            let a = input.data()?;
            let b = if matches!(status >> 4, 0xc | 0xd) {
                0
            } else {
                input.data()?
            };
            match status >> 4 {
                8 => (
                    Some(EventKind::NoteOff { channel, key: a }),
                    Some(controls::StagedKind::NoteOff { channel, key: a }),
                ),
                9 if b == 0 => (
                    Some(EventKind::NoteOff { channel, key: a }),
                    Some(controls::StagedKind::NoteOff { channel, key: a }),
                ),
                9 => (
                    Some(EventKind::NoteOn {
                        channel,
                        key: a,
                        velocity: b,
                    }),
                    Some(controls::StagedKind::NoteOn {
                        channel,
                        key: a,
                        velocity: b,
                    }),
                ),
                0xb => {
                    let old = [7, 10, 11, 64].contains(&a).then_some(EventKind::Control {
                        channel,
                        controller: a,
                        value: b,
                    });
                    if old.is_none() && profile == MidiProfile::LegacyV1 {
                        let name = if [6, 38, 96, 97, 98, 99, 100, 101].contains(&a) {
                            "RPN/NRPN or data-entry"
                        } else {
                            "controller"
                        };
                        diagnostic = Some((
                            format!("Unsupported {name} CC {a}={b} on channel {}", channel + 1),
                            true,
                        ));
                    }
                    (
                        old,
                        Some(controls::StagedKind::Control {
                            channel,
                            controller: a,
                            value: b,
                        }),
                    )
                }
                0xc => (
                    Some(EventKind::Program {
                        channel,
                        program: a,
                    }),
                    Some(controls::StagedKind::Program {
                        channel,
                        program: a,
                    }),
                ),
                0xe => {
                    let value = u16::from(a) | u16::from(b) << 7;
                    (
                        Some(EventKind::Bend { channel, value }),
                        Some(controls::StagedKind::Bend { channel, value }),
                    )
                }
                0xa | 0xd => {
                    diagnostic = Some((
                        format!("Unsupported aftertouch on channel {}", channel + 1),
                        true,
                    ));
                    (None, None)
                }
                _ => return Err(format!("Invalid channel status 0x{status:02X}")),
            }
        } else {
            running = None;
            match status {
                0xf0 | 0xf7 => {
                    let len = input.vlq()? as usize;
                    input.take(len)?;
                    diagnostic = Some((
                        format!("Unsupported SysEx/escape 0x{status:02X} ({len} bytes)"),
                        true,
                    ));
                    (None, None)
                }
                0xff => {
                    let meta = input.data()?;
                    let len = input.vlq()? as usize;
                    let data = input.take(len)?;
                    match meta {
                        0x2f if len == 0 => {
                            ended = true;
                            if input.at != bytes.len() {
                                return Err("Data follows MIDI end-of-track".into());
                            };
                            (
                                Some(EventKind::EndTrack),
                                Some(controls::StagedKind::EndTrack),
                            )
                        }
                        0x2f => return Err("MIDI end-of-track must have length zero".into()),
                        0x51 if len >= 3 => {
                            let tempo = u32::from_be_bytes([0, data[0], data[1], data[2]]);
                            if tempo == 0 {
                                return Err("MIDI tempo cannot be zero".into());
                            }
                            if len > 3 {
                                diagnostic = Some((
                                    format!(
                                        "Tempo meta has {} additional bytes; preserved in source",
                                        len - 3
                                    ),
                                    false,
                                ));
                            }
                            (
                                Some(EventKind::Tempo(tempo)),
                                Some(controls::StagedKind::Tempo(tempo)),
                            )
                        }
                        0x58 if len >= 4 => {
                            if data[0] == 0 || data[1] > 7 {
                                return Err("Invalid MIDI time signature".into());
                            }
                            if len > 4 {
                                diagnostic = Some((
                                    format!(
                                        "Time signature has {} additional bytes; preserved in source",
                                        len - 4
                                    ),
                                    false,
                                ));
                            }
                            let legacy = EventKind::TimeSignature {
                                numerator: data[0],
                                denominator_power: data[1],
                                clocks: data[2],
                                thirty_seconds: data[3],
                            };
                            let musical = controls::StagedKind::TimeSignature {
                                numerator: data[0],
                                denominator_power: data[1],
                                clocks: data[2],
                                thirty_seconds: data[3],
                            };
                            (Some(legacy), Some(musical))
                        }
                        0x51 | 0x58 => {
                            return Err(format!("Truncated MIDI meta event 0x{meta:02X}"));
                        }
                        0x06 if data == b"loop_start" => (
                            Some(EventKind::LoopStart),
                            Some(controls::StagedKind::LoopStart),
                        ),
                        0x06 if data == b"loop_end" => (
                            Some(EventKind::LoopEnd),
                            Some(controls::StagedKind::LoopEnd),
                        ),
                        0x01..=0x07 | 0x00 | 0x59 => {
                            diagnostic = Some((
                                format!(
                                    "Annotation meta 0x{meta:02X} ({len} bytes), preserved in source"
                                ),
                                false,
                            ));
                            (None, None)
                        }
                        // SMF sequencer-specific data is opaque authoring metadata. It
                        // does not describe a note, controller, tempo, or any other
                        // operation that Epok would reproduce. Preserve it in the
                        // source ledger, but do not make a normal MIDI unplayable just
                        // because its DAW wrote a private three-byte marker at tick 0.
                        0x7f => {
                            diagnostic = Some((
                                format!(
                                    "Sequencer-specific meta 0x7F ({len} bytes), preserved in source"
                                ),
                                false,
                            ));
                            (None, None)
                        }
                        _ => {
                            diagnostic = Some((
                                format!(
                                    "Unsupported meta 0x{meta:02X} ({len} bytes), preserved in source"
                                ),
                                true,
                            ));
                            (None, None)
                        }
                    }
                }
                _ => {
                    return Err(format!(
                        "Status 0x{status:02X} is not legal in an SMF track"
                    ));
                }
            }
        };
        ledger.push(SourceEvent {
            tick,
            track,
            order,
            offset: base + event_start,
            status,
            explicit_status: first >= 128,
            data: bytes[data_start..input.at].to_vec(),
        });
        match profile {
            MidiProfile::LegacyV1 => {
                if let Some(kind) = legacy {
                    events.push(Event {
                        tick,
                        track,
                        order,
                        kind,
                    });
                }
            }
            MidiProfile::MusicalV2 => {
                if let Some(kind) = musical {
                    staged.push(controls::StagedEvent {
                        tick,
                        track,
                        order,
                        kind,
                    });
                }
            }
        }
        if let Some((message, unsupported)) = diagnostic {
            if diagnostics.len() == MAX_DIAGNOSTICS {
                return Err("MIDI exceeds 4096 diagnostics".into());
            }
            diagnostics.push(Diagnostic {
                track,
                tick,
                message,
                unsupported,
            });
        }
        order += 1;
    }
    if !ended {
        return Err("MIDI track has no end-of-track event".into());
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
pub struct Header {
    pub format: u16,
    pub tracks: u16,
    pub ppqn: u16,
}
pub fn probe(bytes: &[u8]) -> Result<Header, String> {
    if bytes.len() > MAX_BYTES || bytes.get(..4) != Some(b"MThd") {
        return Err("Expected a Standard MIDI File (MThd, maximum 4 MiB)".into());
    }
    let u16be = |at| -> Result<u16, String> {
        Ok(u16::from_be_bytes(
            bytes
                .get(at..at + 2)
                .ok_or("Truncated MIDI header")?
                .try_into()
                .unwrap(),
        ))
    };
    let u32be = |at| -> Result<u32, String> {
        Ok(u32::from_be_bytes(
            bytes
                .get(at..at + 4)
                .ok_or("Truncated MIDI chunk")?
                .try_into()
                .unwrap(),
        ))
    };
    let header_size = u32be(4)? as usize;
    if header_size < 6 || header_size > bytes.len().saturating_sub(8) {
        return Err("Invalid MIDI header length".into());
    }
    let header = Header {
        format: u16be(8)?,
        tracks: u16be(10)?,
        ppqn: u16be(12)?,
    };
    if header.format > 1
        || header.tracks == 0
        || header.tracks as usize > MAX_TRACKS
        || (header.format == 0 && header.tracks != 1)
    {
        return Err("Supported MIDI: format 0 (one track) or 1, at most 256 tracks".into());
    }
    if header.ppqn & 0x8000 != 0 {
        return Err("SMPTE MIDI timing is unsupported; export PPQN timing".into());
    }
    if header.ppqn == 0 {
        return Err("MIDI PPQN cannot be zero".into());
    }
    let mut at = 8 + header_size;
    for _ in 0..header.tracks {
        if bytes.get(at..at + 4) != Some(b"MTrk") {
            return Err("Missing MIDI MTrk chunk".into());
        }
        let len = u32be(at + 4)? as usize;
        at = at
            .checked_add(8)
            .and_then(|v| v.checked_add(len))
            .ok_or("MIDI track length overflow")?;
        if len == 0 || at > bytes.len() {
            return Err("Empty or truncated MIDI track".into());
        }
    }
    if at != bytes.len() {
        return Err("Unexpected data after MIDI tracks".into());
    }
    Ok(header)
}

#[cfg(test)]
pub fn fixture() -> Vec<u8> {
    b"MThd\0\0\0\x06\0\0\0\x01\0\x60MTrk\0\0\0\x0c\0\x90\x3c\x64\x60\x80\x3c\0\0\xff\x2f\0".to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn smf(ppqn: u16, tracks: &[&[u8]]) -> Vec<u8> {
        let mut b = b"MThd\0\0\0\x06".to_vec();
        b.extend(if tracks.len() == 1 { 0_u16 } else { 1 }.to_be_bytes());
        b.extend((tracks.len() as u16).to_be_bytes());
        b.extend(ppqn.to_be_bytes());
        for t in tracks {
            b.extend(b"MTrk");
            b.extend((t.len() as u32).to_be_bytes());
            b.extend(*t);
        }
        b
    }
    #[test]
    fn midi_notes_controllers_sustain_bend_and_exact_tempo_analysis() {
        let ir = parse(&smf(
            96,
            &[&[
                0, 0xc0, 5, 0, 0xb0, 7, 100, 0, 64, 127, 0, 0x90, 60, 100, 96, 60, 0, 0, 0xff,
                0x51, 3, 0x0f, 0x42, 0x40, 0, 0x90, 64, 90, 96, 0x80, 64, 0, 0, 0xb0, 64, 0, 0,
                0xe0, 0, 96, 0, 0xff, 0x2f, 0,
            ]],
        ))
        .unwrap();
        assert_eq!(ir.duration_micros, 1_500_000);
        assert_eq!(ir.micros_at(144), 1_000_000);
        assert_eq!(ir.peak_polyphony, 2);
        assert!(ir.diagnostics.is_empty());
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::Bend {
                channel: 0,
                value: 12288
            }));
    }

    #[test]
    fn sequencer_specific_metadata_is_advisory_in_musical_v2() {
        let ir = parse(&smf(
            96,
            &[&[
                0, 0xff, 0x7f, 3, 1, 2, 3, 0, 0x90, 60, 100, 96, 0x80, 60, 0, 0, 0xff, 0x2f, 0,
            ]],
        ))
        .unwrap();
        assert!(
            ir.events
                .iter()
                .any(|event| matches!(event.kind, EventKind::NoteOn { .. }))
        );
        assert!(ir.diagnostics.iter().any(|diagnostic| {
            diagnostic.message.contains("Sequencer-specific meta 0x7F") && !diagnostic.unsupported
        }));
        assert!(ir.playback_blockers.is_empty());
    }
    #[test]
    fn musical_merge_rpn_banks_and_wire_ledger_are_deterministic() {
        let ir = parse(&smf(
            96,
            &[
                &[
                    0, 0xb0, 100, 0, 0, 101, 0, 0, 0xb0, 0, 1, 0, 32, 2, 0, 0xc0, 44, 0, 0xff,
                    0x2f, 0,
                ],
                &[
                    0, 0xb0, 6, 12, 0, 38, 0, 0, 0x90, 60, 1, 96, 0x80, 60, 0, 0, 0xff, 0x2f, 0,
                ],
            ],
        ))
        .unwrap();
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::Parameter {
                channel: 0,
                parameter: 0,
                value: 1200
            }));
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::BankProgram {
                channel: 0,
                bank: 130,
                program: 44
            }));
        assert_eq!(ir.source_events.len(), 11);
        assert!(ir.source_events.iter().all(|e| e.offset >= 22));
        assert_eq!(ir.source_events[0].track, 0);
    }
    #[test]
    fn rpn_increment_limits_effects_and_partial_selectors_are_actionable() {
        let ir = parse(&smf(
            96,
            &[&[
                0, 0xb0, 101, 0, 0, 100, 0, 0, 6, 127, 0, 38, 127, 0, 96, 0, 0, 97, 0, 0, 91, 0, 0,
                95, 0, 0, 91, 1, 0, 120, 99, 0, 121, 88, 0, 101, 3, 0, 100, 0, 0, 6, 1, 0, 0xb1,
                101, 3, 0, 0xb1, 6, 1, 0, 0xff, 0x2f, 0,
            ]],
        ))
        .unwrap();
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::Parameter {
                channel: 0,
                parameter: 0,
                value: 12827
            }));
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::Parameter {
                channel: 0,
                parameter: 0,
                value: 12826
            }));
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::Control {
                channel: 0,
                controller: 95,
                value: 0
            }));
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::Control {
                channel: 0,
                controller: 120,
                value: 0
            }));
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::Control {
                channel: 0,
                controller: 121,
                value: 0
            }));
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::Control {
                channel: 0,
                controller: 91,
                value: 1
            }));
        assert!(
            !ir.diagnostics
                .iter()
                .any(|d| d.message.contains("effect CC 91=1"))
        );
        assert!(
            ir.diagnostics
                .iter()
                .any(|d| d.message.contains("RPN selection is incomplete"))
        );
        assert!(
            ir.diagnostics
                .iter()
                .any(|d| d.message.contains("Unsupported RPN 384"))
        );
    }

    #[test]
    fn rpn_zero_carries_cents_and_other_registered_parameters_increment_in_their_units() {
        let ir = parse(&smf(
            96,
            &[&[
                0, 0xb0, 101, 0, 0, 100, 0, 0, 6, 1, 0, 38, 99, 0, 96, 0, 0, 101, 0, 0, 100, 1, 0,
                6, 64, 0, 38, 0, 0, 96, 0, 0, 101, 0, 0, 100, 2, 0, 6, 64, 0, 96, 0, 0, 0xff, 0x2f,
                0,
            ]],
        ))
        .unwrap();
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::Parameter {
                channel: 0,
                parameter: 0,
                value: 200
            }));
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::Parameter {
                channel: 0,
                parameter: 1,
                value: 8193
            }));
        assert!(ir.events.iter().any(|e| e.kind
            == EventKind::Parameter {
                channel: 0,
                parameter: 2,
                value: 65
            }));
    }

    #[test]
    fn rpn_zero_retains_raw_lsb_until_data_increment_normalizes_cents() {
        let ir = parse(&smf(
            96,
            &[&[
                0, 0xb0, 101, 0, 0, 100, 0, 0, 6, 12, 0, 38, 127, 0, 6, 13, 0, 96, 0, 0, 6, 127, 0,
                38, 127, 0, 96, 0, 0, 97, 0, 0, 38, 0, 0, 97, 0, 0, 0xff, 0x2f, 0,
            ]],
        ))
        .unwrap();
        let values = ir
            .events
            .iter()
            .filter_map(|event| match event.kind {
                EventKind::Parameter {
                    channel: 0,
                    parameter: 0,
                    value,
                } => Some(value),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            [
                1200, 1327, 1427, 1428, 12728, 12827, 12827, 12826, 12700, 12699
            ]
        );
    }
    #[test]
    fn pedals_channel_modes_and_repeated_notes_leave_no_hanging_voice() {
        let ir = parse(&smf(
            96,
            &[&[
                0, 0x90, 60, 1, 0, 60, 1, 0, 0xb0, 66, 127, 0, 0x80, 60, 0, 0, 0x80, 60, 0, 0,
                0x90, 61, 1, 0, 0xb0, 64, 127, 0, 0x80, 61, 0, 0, 0xb0, 120, 0, 0, 0x90, 62, 1, 0,
                0xb0, 123, 0, 0, 0xb0, 121, 0, 0, 0xff, 0x2f, 0,
            ]],
        ))
        .unwrap();
        assert_eq!(ir.peak_polyphony, 3);
        assert!(
            !ir.diagnostics
                .iter()
                .any(|d| d.message.contains("remain at song end"))
        );
    }
    #[test]
    fn legacy_profile_preserves_unsupported_rpn_oracle() {
        let bytes = smf(96, &[&[0, 0xb0, 101, 0, 0, 0xff, 0x2f, 0]]);
        assert_eq!(
            parse_with_profile(&bytes, MidiProfile::LegacyV1)
                .unwrap()
                .diagnostics
                .iter()
                .filter(|d| d.unsupported)
                .count(),
            1
        );
        assert!(parse(&bytes).unwrap().diagnostics.is_empty());
    }
    #[test]
    fn loop_validation_and_malformed_inputs_remain_bounded() {
        let start = b"\0\xff\x06\x0aloop_start\x60\xff\x06\x08loop_end\0\xff\x2f\0";
        assert_eq!(
            parse(&smf(96, &[start])).unwrap().loop_region,
            Some([0, 96])
        );
        assert!(parse(&smf(96, &[&[0, 60, 100]])).is_err());
        let large = [0, 0xc0, 1].repeat(MAX_EVENTS + 1);
        assert!(parse(&smf(96, &[&large])).unwrap_err().contains("65536"));
    }
}
