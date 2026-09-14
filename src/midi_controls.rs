//! Musical MIDI controller lowering after SMF tracks have a stable total order.
use crate::sequence_ir::{Diagnostic, Event, EventKind, MAX_DIAGNOSTICS};

#[derive(Clone, Debug)]
pub(crate) enum StagedKind {
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
    Control {
        channel: u8,
        controller: u8,
        value: u8,
    },
    Bend {
        channel: u8,
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

#[derive(Clone, Debug)]
pub(crate) struct StagedEvent {
    pub tick: u32,
    pub track: u16,
    pub order: u32,
    pub kind: StagedKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Rpn,
    Nrpn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Selected {
    None,
    Partial(Family),
    Rpn(u16),
    Nrpn(u16),
}

#[derive(Clone, Copy)]
struct Channel {
    bank_msb: u8,
    bank_lsb: u8,
    rpn_msb: Option<u8>,
    rpn_lsb: Option<u8>,
    nrpn_msb: Option<u8>,
    nrpn_lsb: Option<u8>,
    family: Option<Family>,
    /// RPN 0 uses cents, RPN 1 is unsigned 14-bit, RPN 2 is semitones.
    parameters: [u16; 3],
    rpn0_msb: u8,
    rpn0_lsb: u8,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            bank_msb: 0,
            bank_lsb: 0,
            rpn_msb: None,
            rpn_lsb: None,
            nrpn_msb: None,
            nrpn_lsb: None,
            family: None,
            parameters: [200, 8192, 64],
            rpn0_msb: 2,
            rpn0_lsb: 0,
        }
    }
}

impl Channel {
    fn rpn0_value(&self) -> u16 {
        u16::from(self.rpn0_msb) * 100 + u16::from(self.rpn0_lsb)
    }

    fn update_rpn0(&mut self) {
        self.parameters[0] = self.rpn0_value();
    }

    fn set_rpn0_normalized(&mut self, value: u16) {
        // RPN 0 represents cents. Its wire MSB remains a seven-bit data byte,
        // so the top semantic value 12_827 is encoded as 127 + 127 cents.
        let msb = (value / 100).min(127);
        self.rpn0_msb = msb as u8;
        self.rpn0_lsb = (value - msb * 100) as u8;
        self.parameters[0] = value;
    }

    fn increment_rpn0(&mut self) {
        self.set_rpn0_normalized(self.rpn0_value().saturating_add(1).min(12_827));
    }

    fn decrement_rpn0(&mut self) {
        self.set_rpn0_normalized(self.rpn0_value().saturating_sub(1));
    }
}

impl Channel {
    fn bank(self) -> u16 {
        u16::from(self.bank_msb) << 7 | u16::from(self.bank_lsb)
    }

    fn select_rpn_msb(&mut self, value: u8) {
        self.family = Some(Family::Rpn);
        self.rpn_msb = Some(value);
    }
    fn select_rpn_lsb(&mut self, value: u8) {
        self.family = Some(Family::Rpn);
        self.rpn_lsb = Some(value);
    }
    fn select_nrpn_msb(&mut self, value: u8) {
        self.family = Some(Family::Nrpn);
        self.nrpn_msb = Some(value);
    }
    fn select_nrpn_lsb(&mut self, value: u8) {
        self.family = Some(Family::Nrpn);
        self.nrpn_lsb = Some(value);
    }

    fn selected(self) -> Selected {
        let (family, msb, lsb) = match self.family {
            None => return Selected::None,
            Some(Family::Rpn) => (Family::Rpn, self.rpn_msb, self.rpn_lsb),
            Some(Family::Nrpn) => (Family::Nrpn, self.nrpn_msb, self.nrpn_lsb),
        };
        let (Some(msb), Some(lsb)) = (msb, lsb) else {
            return Selected::Partial(family);
        };
        if msb == 127 && lsb == 127 {
            Selected::None
        } else if family == Family::Rpn {
            Selected::Rpn(u16::from(msb) << 7 | u16::from(lsb))
        } else {
            Selected::Nrpn(u16::from(msb) << 7 | u16::from(lsb))
        }
    }

    fn reset_selectors(&mut self) {
        self.rpn_msb = None;
        self.rpn_lsb = None;
        self.nrpn_msb = None;
        self.nrpn_lsb = None;
        self.family = None;
    }
}

fn diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    event: &StagedEvent,
    message: String,
) -> Result<(), String> {
    if diagnostics.len() == MAX_DIAGNOSTICS {
        return Err("MIDI exceeds 4096 diagnostics".into());
    }
    diagnostics.push(Diagnostic {
        track: event.track,
        tick: event.tick,
        message,
        unsupported: true,
    });
    Ok(())
}

fn parameter_event(channel: u8, parameter: usize, value: u16) -> EventKind {
    EventKind::Parameter {
        channel,
        parameter: parameter as u8,
        value,
    }
}

fn edit_parameter(
    channel: &mut Channel,
    event: &StagedEvent,
    data_controller: u8,
    data_value: u8,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<EventKind>, String> {
    let channel_number = event_channel(event) + 1;
    let number = match channel.selected() {
        Selected::None => return Ok(None),
        Selected::Partial(Family::Rpn) => {
            diagnostic(
                diagnostics,
                event,
                format!(
                    "RPN selection is incomplete; CC {data_controller}={data_value} on channel {channel_number} was retained but cannot execute"
                ),
            )?;
            return Ok(None);
        }
        Selected::Partial(Family::Nrpn) => {
            diagnostic(
                diagnostics,
                event,
                format!(
                    "NRPN selection is incomplete; CC {data_controller}={data_value} on channel {channel_number} was retained but cannot execute"
                ),
            )?;
            return Ok(None);
        }
        Selected::Rpn(number @ 0..=2) => number as usize,
        Selected::Rpn(number) => {
            diagnostic(
                diagnostics,
                event,
                format!(
                    "Unsupported RPN {number}; CC {data_controller}={data_value} on channel {channel_number} was retained but cannot execute"
                ),
            )?;
            return Ok(None);
        }
        Selected::Nrpn(number) => {
            diagnostic(
                diagnostics,
                event,
                format!(
                    "Unsupported NRPN {number}; CC {data_controller}={data_value} on channel {channel_number} was retained but cannot execute"
                ),
            )?;
            return Ok(None);
        }
    };
    match data_controller {
        6 => match number {
            0 => {
                channel.rpn0_msb = data_value;
                channel.update_rpn0();
            }
            1 => channel.parameters[1] = u16::from(data_value) << 7 | channel.parameters[1] & 127,
            2 => channel.parameters[2] = u16::from(data_value),
            _ => unreachable!(),
        },
        38 => match number {
            0 => {
                channel.rpn0_lsb = data_value;
                channel.update_rpn0();
            }
            1 => channel.parameters[1] = channel.parameters[1] & !127 | u16::from(data_value),
            2 => return Ok(None), // RPN 2 uses its MSB only; the raw CC remains in the ledger.
            _ => unreachable!(),
        },
        96 => match number {
            0 => channel.increment_rpn0(),
            1 => channel.parameters[1] = channel.parameters[1].saturating_add(1).min(16_383),
            2 => channel.parameters[2] = channel.parameters[2].saturating_add(1).min(127),
            _ => unreachable!(),
        },
        97 => match number {
            0 => channel.decrement_rpn0(),
            _ => channel.parameters[number] = channel.parameters[number].saturating_sub(1),
        },
        _ => unreachable!(),
    }
    Ok(Some(parameter_event(
        event_channel(event),
        number,
        channel.parameters[number],
    )))
}

fn event_channel(event: &StagedEvent) -> u8 {
    match &event.kind {
        StagedKind::Control { channel, .. } => *channel,
        _ => unreachable!("only controller events edit RPN state"),
    }
}

/// Resolve raw channel state after merge order `(tick, track, order)` is fixed.
pub(crate) fn resolve(
    mut staged: Vec<StagedEvent>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<Vec<Event>, String> {
    staged.sort_by_key(|event| (event.tick, event.track, event.order));
    let mut channels = [Channel::default(); 16];
    let mut output = Vec::with_capacity(staged.len());
    for staged_event in staged {
        let event = &staged_event;
        let kind = match event.kind.clone() {
            StagedKind::NoteOn {
                channel,
                key,
                velocity,
            } => Some(EventKind::NoteOn {
                channel,
                key,
                velocity,
            }),
            StagedKind::NoteOff { channel, key } => Some(EventKind::NoteOff { channel, key }),
            StagedKind::Bend { channel, value } => Some(EventKind::Bend { channel, value }),
            StagedKind::Tempo(value) => Some(EventKind::Tempo(value)),
            StagedKind::TimeSignature {
                numerator,
                denominator_power,
                clocks,
                thirty_seconds,
            } => Some(EventKind::TimeSignature {
                numerator,
                denominator_power,
                clocks,
                thirty_seconds,
            }),
            StagedKind::LoopStart => Some(EventKind::LoopStart),
            StagedKind::LoopEnd => Some(EventKind::LoopEnd),
            StagedKind::EndTrack => Some(EventKind::EndTrack),
            StagedKind::Program { channel, program } => {
                let bank = channels[channel as usize].bank();
                Some(if bank == 0 {
                    EventKind::Program { channel, program }
                } else {
                    EventKind::BankProgram {
                        channel,
                        bank,
                        program,
                    }
                })
            }
            StagedKind::Control {
                channel,
                controller,
                value,
            } => {
                let state = &mut channels[channel as usize];
                match controller {
                    0 => {
                        state.bank_msb = value;
                        None
                    }
                    32 => {
                        state.bank_lsb = value;
                        None
                    }
                    101 => {
                        state.select_rpn_msb(value);
                        None
                    }
                    100 => {
                        state.select_rpn_lsb(value);
                        None
                    }
                    99 => {
                        state.select_nrpn_msb(value);
                        None
                    }
                    98 => {
                        state.select_nrpn_lsb(value);
                        None
                    }
                    6 | 38 | 96 | 97 => {
                        edit_parameter(state, event, controller, value, diagnostics)?
                    }
                    1 | 7 | 10 | 11 | 64 | 66 => Some(EventKind::Control {
                        channel,
                        controller,
                        value,
                    }),
                    120 | 123 => Some(EventKind::Control {
                        channel,
                        controller,
                        value: 0,
                    }),
                    121 => {
                        state.reset_selectors();
                        Some(EventKind::Control {
                            channel,
                            controller,
                            value: 0,
                        })
                    }
                    91 => Some(EventKind::Control {
                        channel,
                        controller,
                        value,
                    }),
                    92 | 93 | 95 if value == 0 => Some(EventKind::Control {
                        channel,
                        controller,
                        value,
                    }),
                    92 | 93 | 95 => {
                        diagnostic(
                            diagnostics,
                            event,
                            format!(
                                "Unsupported effect CC {controller}={value} on channel {}; only explicit zero is representable",
                                channel + 1
                            ),
                        )?;
                        None
                    }
                    _ => {
                        diagnostic(
                            diagnostics,
                            event,
                            format!(
                                "Unsupported controller CC {controller}={value} on channel {}",
                                channel + 1
                            ),
                        )?;
                        None
                    }
                }
            }
        };
        if let Some(kind) = kind {
            output.push(Event {
                tick: event.tick,
                track: event.track,
                order: event.order,
                kind,
            });
        }
    }
    Ok(output)
}
