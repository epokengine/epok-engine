//! Musical time and events, independent of console payloads and sample codecs.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};

pub const MAX_EVENTS: usize = 65_536;
pub const MAX_DIAGNOSTICS: usize = 4096;
/// Bound host work while interpreting pedal and channel-mode controller passes.
pub const MAX_NOTE_STATE_OPERATIONS: usize = 2_000_000;

fn consume_note_state_operations(total: &mut usize, count: usize) -> Result<(), String> {
    *total = total
        .checked_add(count)
        .ok_or("MIDI note-state operation counter overflow")?;
    if *total > MAX_NOTE_STATE_OPERATIONS {
        return Err("MIDI note-state operation budget exceeded (2000000)".into());
    }
    Ok(())
}

/// Lossless semantic view of a verified source event, including operations the
/// current playback profile cannot execute. Original wire bytes remain in the
/// authoritative asset snapshot; this ledger never guesses unknown lengths.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceEvent {
    pub tick: u32,
    /// SMF track number. Legacy compatibility sources deserialize as track zero.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub track: u16,
    pub order: u32,
    /// Absolute byte offset in the original source file, including the delta-time.
    pub offset: usize,
    pub status: u8,
    pub explicit_status: bool,
    /// Channel parameters or the meta type followed by its exact parameter bytes.
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum EventKind {
    NoteOn {
        channel: u8,
        key: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        key: u8,
    },
    Program {
        channel: u8,
        program: u8,
    },
    /// A Program Change whose preceding Bank Select was non-zero.
    BankProgram {
        channel: u8,
        bank: u16,
        program: u8,
    },
    Control {
        channel: u8,
        controller: u8,
        value: u8,
    },
    /// 0..16383, center 8192. Playback starts at +/-2 semitones until RPN 0 changes it.
    Bend {
        channel: u8,
        value: u16,
    },
    /// Normalized RPN state: 0=bend sensitivity in cents, 1=fine tuning,
    /// 2=coarse tuning. Selectors and raw Data Entry stay in SourceEvent.
    Parameter {
        channel: u8,
        parameter: u8,
        value: u16,
    },
    Tempo(u32),
    TimeSignature {
        numerator: u8,
        denominator_power: u8,
        clocks: u8,
        thirty_seconds: u8,
    },
    LoopStart,
    LoopEnd,
    EndTrack,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Event {
    pub tick: u32,
    pub track: u16,
    pub order: u32,
    pub kind: EventKind,
}

#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub track: u16,
    pub tick: u32,
    pub message: String,
    /// Unsupported musical operations require explicit acknowledgement before audition/cook.
    pub unsupported: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct TempoPoint {
    pub tick: u32,
    pub micros_per_quarter: u32,
    /// Exact time numerator with PPQN as the denominator (no accumulated rounding).
    pub time_numerator: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct InstrumentUse {
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bank: u16,
    pub program: u8,
    /// Current MIDI profile: channel 10 needs explicit drum zones. Compatibility
    /// sources without that policy carry a hard blocker and retain their ledger.
    pub drum_key: Option<u8>,
}

fn is_zero(value: &u16) -> bool {
    *value == 0
}

#[derive(Clone, Debug, Serialize)]
pub struct SequenceIr {
    pub ppqn: u16,
    pub events: Vec<Event>,
    pub diagnostics: Vec<Diagnostic>,
    pub tempo_map: Vec<TempoPoint>,
    pub duration_ticks: u32,
    pub duration_micros: u64,
    pub loop_region: Option<[u32; 2]>,
    pub instruments: BTreeSet<InstrumentUse>,
    /// Logical overlapping note lifetimes, including sustain; bank release tails are separate.
    pub peak_polyphony: u32,
    pub channels: BTreeSet<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_profile: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_events: Vec<SourceEvent>,
    /// Integrity/fidelity requirements which Ignore unsupported MIDI cannot bypass.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub playback_blockers: Vec<Diagnostic>,
}

impl SequenceIr {
    pub fn analyze(
        ppqn: u16,
        mut events: Vec<Event>,
        mut diagnostics: Vec<Diagnostic>,
    ) -> Result<Self, String> {
        if ppqn == 0
            || ppqn & 0x8000 != 0
            || events.len() > MAX_EVENTS
            || diagnostics.len() > MAX_DIAGNOSTICS
        {
            return Err("Invalid sequence PPQN or event/diagnostic capacity".into());
        }
        for event in &events {
            let valid = match event.kind {
                EventKind::NoteOn {
                    channel,
                    key,
                    velocity,
                } => channel < 16 && key < 128 && (1..=127).contains(&velocity),
                EventKind::NoteOff { channel, key } => channel < 16 && key < 128,
                EventKind::Program { channel, program } => channel < 16 && program < 128,
                EventKind::BankProgram {
                    channel,
                    bank,
                    program,
                } => channel < 16 && bank < 16384 && program < 128,
                EventKind::Control {
                    channel,
                    controller,
                    value,
                } => {
                    channel < 16
                        && [1, 7, 10, 11, 64, 66, 91, 92, 93, 95, 120, 121, 123]
                            .contains(&controller)
                        && value < 128
                        && (!([92, 93, 95, 120, 121, 123].contains(&controller)) || value == 0)
                }
                EventKind::Bend { channel, value } => channel < 16 && value < 16384,
                EventKind::Parameter {
                    channel,
                    parameter,
                    value,
                } => {
                    channel < 16
                        && match parameter {
                            0 => value <= 12_827,
                            1 => value < 16_384,
                            2 => value <= 127,
                            _ => false,
                        }
                }
                EventKind::Tempo(value) => (1..=0xffffff).contains(&value),
                EventKind::TimeSignature {
                    numerator,
                    denominator_power,
                    ..
                } => numerator != 0 && denominator_power <= 7,
                _ => true,
            };
            if !valid {
                return Err("Invalid neutral sequence event".into());
            }
        }
        // Explicit total order: absolute tick, source track, then original event ordinal.
        events.sort_by_key(|e| (e.tick, e.track, e.order));
        let mut ir = Self {
            ppqn,
            events,
            diagnostics: vec![],
            tempo_map: vec![TempoPoint {
                tick: 0,
                micros_per_quarter: 500_000,
                time_numerator: 0,
            }],
            duration_ticks: 0,
            duration_micros: 0,
            loop_region: None,
            instruments: BTreeSet::new(),
            peak_polyphony: 0,
            channels: BTreeSet::new(),
            source_profile: None,
            source_events: Vec::new(),
            playback_blockers: Vec::new(),
        };
        let mut programs = [0; 16];
        let mut banks = [0; 16];
        let mut sustain = [false; 16];
        let mut sostenuto = [false; 16];
        #[derive(Clone, Copy)]
        struct HeldNote {
            down: bool,
            sustained: bool,
            sostenuto: bool,
            active: bool,
        }
        let mut notes = Vec::<HeldNote>::new();
        // Each note-off consumes its matching key-down in FIFO order. The
        // active set keeps controller passes proportional to live voices, not
        // to every note which has appeared earlier in a long SMF.
        let mut queued: [[VecDeque<usize>; 128]; 16] =
            std::array::from_fn(|_| std::array::from_fn(|_| VecDeque::new()));
        let mut active_notes: [BTreeSet<usize>; 16] = std::array::from_fn(|_| BTreeSet::new());
        let mut note_state_operations = 0_usize;
        let (mut active, mut time, mut tick, mut tempo) = (0_u32, 0_u64, 0_u32, 500_000_u32);
        let (mut start, mut end) = (None, None);
        for e in &ir.events {
            time = time
                .checked_add(u64::from(e.tick - tick) * u64::from(tempo))
                .ok_or("MIDI duration overflow")?;
            tick = e.tick;
            match e.kind {
                EventKind::Tempo(value) => {
                    tempo = value;
                    ir.tempo_map.push(TempoPoint {
                        tick,
                        micros_per_quarter: value,
                        time_numerator: time,
                    });
                }
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
                EventKind::NoteOn { channel, key, .. } => {
                    consume_note_state_operations(&mut note_state_operations, 1)?;
                    ir.channels.insert(channel);
                    ir.instruments.insert(InstrumentUse {
                        bank: banks[channel as usize],
                        program: programs[channel as usize],
                        drum_key: (channel == 9).then_some(key),
                    });
                    let index = notes.len();
                    notes.push(HeldNote {
                        down: true,
                        sustained: false,
                        sostenuto: false,
                        active: true,
                    });
                    queued[channel as usize][key as usize].push_back(index);
                    active_notes[channel as usize].insert(index);
                    active += 1;
                    ir.peak_polyphony = ir.peak_polyphony.max(active);
                }
                EventKind::NoteOff { channel, key } => {
                    consume_note_state_operations(&mut note_state_operations, 1)?;
                    if let Some(index) = queued[channel as usize][key as usize].pop_front() {
                        let note = &mut notes[index];
                        note.down = false;
                        note.sustained = sustain[channel as usize];
                        if !note.active || note.sustained || note.sostenuto {
                            // Pedals retain this voice until their corresponding release.
                        } else {
                            note.active = false;
                            active_notes[channel as usize].remove(&index);
                            active -= 1;
                        }
                    } else if diagnostics.len() < MAX_DIAGNOSTICS {
                        diagnostics.push(Diagnostic {
                            track: e.track,
                            tick,
                            message: format!(
                                "Note-off without note-on: channel {}, key {key}",
                                channel + 1
                            ),
                            unsupported: false,
                        });
                    } else {
                        return Err("MIDI diagnostic limit exceeded (4096)".into());
                    }
                }
                EventKind::Control {
                    channel,
                    controller: 64,
                    value,
                } => {
                    sustain[channel as usize] = value >= 64;
                    if value < 64 {
                        consume_note_state_operations(
                            &mut note_state_operations,
                            active_notes[channel as usize].len(),
                        )?;
                        for index in active_notes[channel as usize]
                            .iter()
                            .copied()
                            .collect::<Vec<_>>()
                        {
                            let note = &mut notes[index];
                            if note.down {
                                continue;
                            }
                            note.sustained = false;
                            if !note.sostenuto {
                                note.active = false;
                                active_notes[channel as usize].remove(&index);
                                active -= 1;
                            }
                        }
                    }
                }
                EventKind::Control {
                    channel,
                    controller: 66,
                    value,
                } => {
                    if value >= 64 && !sostenuto[channel as usize] {
                        consume_note_state_operations(
                            &mut note_state_operations,
                            active_notes[channel as usize].len(),
                        )?;
                        for index in active_notes[channel as usize].iter().copied() {
                            if notes[index].down {
                                notes[index].sostenuto = true;
                            }
                        }
                    }
                    sostenuto[channel as usize] = value >= 64;
                    if value < 64 {
                        consume_note_state_operations(
                            &mut note_state_operations,
                            active_notes[channel as usize].len(),
                        )?;
                        for index in active_notes[channel as usize]
                            .iter()
                            .copied()
                            .collect::<Vec<_>>()
                        {
                            let note = &mut notes[index];
                            note.sostenuto = false;
                            if !note.down && !note.sustained {
                                note.active = false;
                                active_notes[channel as usize].remove(&index);
                                active -= 1;
                            }
                        }
                    }
                }
                EventKind::Control {
                    channel,
                    controller: 120,
                    ..
                } => {
                    consume_note_state_operations(
                        &mut note_state_operations,
                        active_notes[channel as usize].len(),
                    )?;
                    for index in active_notes[channel as usize]
                        .iter()
                        .copied()
                        .collect::<Vec<_>>()
                    {
                        notes[index].active = false;
                    }
                    active -= active_notes[channel as usize].len() as u32;
                    active_notes[channel as usize].clear();
                }
                EventKind::Control {
                    channel,
                    controller: 123,
                    ..
                } => {
                    consume_note_state_operations(
                        &mut note_state_operations,
                        active_notes[channel as usize].len(),
                    )?;
                    for index in active_notes[channel as usize]
                        .iter()
                        .copied()
                        .collect::<Vec<_>>()
                    {
                        let note = &mut notes[index];
                        if !note.down {
                            continue;
                        }
                        note.down = false;
                        note.sustained = sustain[channel as usize];
                        if !note.sustained && !note.sostenuto {
                            note.active = false;
                            active_notes[channel as usize].remove(&index);
                            active -= 1;
                        }
                    }
                }
                EventKind::Control {
                    channel,
                    controller: 121,
                    ..
                } => {
                    consume_note_state_operations(
                        &mut note_state_operations,
                        active_notes[channel as usize].len(),
                    )?;
                    sustain[channel as usize] = false;
                    sostenuto[channel as usize] = false;
                    for index in active_notes[channel as usize]
                        .iter()
                        .copied()
                        .collect::<Vec<_>>()
                    {
                        let note = &mut notes[index];
                        note.sustained = false;
                        note.sostenuto = false;
                        if note.down {
                            continue;
                        }
                        note.active = false;
                        active_notes[channel as usize].remove(&index);
                        active -= 1;
                    }
                }
                EventKind::LoopStart | EventKind::LoopEnd => {
                    let (marker, name) = if matches!(e.kind, EventKind::LoopStart) {
                        (&mut start, "loop_start")
                    } else {
                        (&mut end, "loop_end")
                    };
                    if marker.is_some() {
                        return Err(format!("MIDI has multiple {name} markers"));
                    }
                    *marker = Some(tick);
                }
                _ => {}
            }
        }
        ir.duration_ticks = tick;
        ir.duration_micros = time / u64::from(ppqn);
        ir.loop_region = match (start, end) {
            (None, None) => None,
            (Some(a), Some(b)) if a < b => Some([a, b]),
            _ => return Err("MIDI loops require one loop_start before one loop_end".into()),
        };
        if active != 0 {
            diagnostics.push(Diagnostic {
                track: 0,
                tick,
                message: format!(
                    "{active} notes remain at song end; playback releases them at the end boundary"
                ),
                unsupported: false,
            });
        }
        if diagnostics.len() > MAX_DIAGNOSTICS {
            return Err("MIDI diagnostic limit exceeded (4096)".into());
        }
        ir.diagnostics = diagnostics;
        Ok(ir)
    }

    pub fn micros_at(&self, tick: u32) -> u64 {
        let tempo = self
            .tempo_map
            .iter()
            .rfind(|point| point.tick <= tick)
            .unwrap();
        (tempo.time_numerator + u64::from(tick - tempo.tick) * u64::from(tempo.micros_per_quarter))
            / u64::from(self.ppqn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(tick: u32, order: u32, kind: EventKind) -> Event {
        Event {
            tick,
            track: 0,
            order,
            kind,
        }
    }

    #[test]
    fn sostenuto_release_while_key_is_down_does_not_latch_the_following_note() {
        let ir = SequenceIr::analyze(
            96,
            vec![
                event(
                    0,
                    0,
                    EventKind::NoteOn {
                        channel: 0,
                        key: 60,
                        velocity: 1,
                    },
                ),
                event(
                    0,
                    1,
                    EventKind::Control {
                        channel: 0,
                        controller: 66,
                        value: 127,
                    },
                ),
                event(
                    0,
                    2,
                    EventKind::Control {
                        channel: 0,
                        controller: 66,
                        value: 0,
                    },
                ),
                event(
                    1,
                    3,
                    EventKind::NoteOff {
                        channel: 0,
                        key: 60,
                    },
                ),
                event(
                    2,
                    4,
                    EventKind::NoteOn {
                        channel: 0,
                        key: 61,
                        velocity: 1,
                    },
                ),
                event(
                    3,
                    5,
                    EventKind::NoteOff {
                        channel: 0,
                        key: 61,
                    },
                ),
                event(3, 6, EventKind::EndTrack),
            ],
            vec![],
        )
        .unwrap();
        assert_eq!(ir.peak_polyphony, 1);
        assert!(
            !ir.diagnostics
                .iter()
                .any(|d| d.message.contains("remain at song end"))
        );
    }

    #[test]
    fn reset_controllers_clears_sostenuto_latches_on_keys_still_down() {
        let ir = SequenceIr::analyze(
            96,
            vec![
                event(
                    0,
                    0,
                    EventKind::NoteOn {
                        channel: 0,
                        key: 60,
                        velocity: 1,
                    },
                ),
                event(
                    0,
                    1,
                    EventKind::Control {
                        channel: 0,
                        controller: 66,
                        value: 127,
                    },
                ),
                event(
                    0,
                    2,
                    EventKind::Control {
                        channel: 0,
                        controller: 121,
                        value: 0,
                    },
                ),
                event(
                    1,
                    3,
                    EventKind::NoteOff {
                        channel: 0,
                        key: 60,
                    },
                ),
                event(1, 4, EventKind::EndTrack),
            ],
            vec![],
        )
        .unwrap();
        assert!(
            !ir.diagnostics
                .iter()
                .any(|d| d.message.contains("remain at song end"))
        );
    }

    #[test]
    fn sequential_note_offs_are_fifo_and_do_not_scan_historical_notes() {
        // Keep the parser's entire valid event capacity on the O(1) note-off
        // path: 65_534 note messages, one neutral controller, and EndTrack.
        let pairs = 32_767_u32;
        let mut events = Vec::with_capacity(MAX_EVENTS);
        for tick in 0..pairs {
            events.push(event(
                tick,
                tick * 2,
                EventKind::NoteOn {
                    channel: 0,
                    key: 60,
                    velocity: 1,
                },
            ));
            events.push(event(
                tick,
                tick * 2 + 1,
                EventKind::NoteOff {
                    channel: 0,
                    key: 60,
                },
            ));
        }
        events.push(event(
            pairs,
            pairs * 2,
            EventKind::Control {
                channel: 0,
                controller: 1,
                value: 0,
            },
        ));
        events.push(event(pairs, pairs * 2 + 1, EventKind::EndTrack));
        assert_eq!(events.len(), MAX_EVENTS);
        let ir = SequenceIr::analyze(96, events, vec![]).unwrap();
        assert_eq!(ir.peak_polyphony, 1);
        assert!(
            !ir.diagnostics
                .iter()
                .any(|d| d.message.contains("remain at song end"))
        );
    }

    #[test]
    fn pedal_passes_stop_at_the_explicit_host_operation_budget() {
        let active_voices = 60_000_u32;
        let mut events = Vec::with_capacity((active_voices + 34) as usize);
        for order in 0..active_voices {
            events.push(event(
                0,
                order,
                EventKind::NoteOn {
                    channel: 0,
                    key: 60,
                    velocity: 1,
                },
            ));
        }
        for step in 0..33_u32 {
            events.push(event(
                0,
                active_voices + step,
                EventKind::Control {
                    channel: 0,
                    controller: 66,
                    value: if step % 2 == 0 { 127 } else { 0 },
                },
            ));
        }
        assert!(
            SequenceIr::analyze(96, events, vec![])
                .unwrap_err()
                .contains("MIDI note-state operation budget exceeded (2000000)")
        );
    }
}
