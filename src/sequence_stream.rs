//! Bounded musical commands shared by host audition and target cookers.
use crate::{
    sequence::{LoopMode, Settings},
    sequence_ir::{EventKind, SequenceIr},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub tick: u32,
    pub op: u8,
    pub channel: u8,
    pub a: u8,
    pub b: u8,
    pub value: u32,
}

pub fn events(ir: &SequenceIr, settings: &Settings) -> Result<Vec<Event>, String> {
    settings.validate_playback(ir)?;
    let region = match settings.loop_mode {
        LoopMode::Off => None,
        LoopMode::Whole => Some([0, ir.duration_ticks]),
        LoopMode::Markers => ir.loop_region,
    };
    if region.is_some_and(|[a, b]| a >= b) {
        return Err("A sequence loop must have positive duration".into());
    }
    let mut output = Vec::new();
    for source in &ir.events {
        let (op, channel, a, b, value) = match source.kind {
            EventKind::NoteOn {
                channel,
                key,
                velocity,
            } => (0, channel, key, velocity, 0),
            EventKind::NoteOff { channel, key } => (1, channel, key, 0, 0),
            EventKind::Program { channel, program } => (2, channel, program, 0, 0),
            EventKind::BankProgram {
                channel,
                bank,
                program,
            } => (10, channel, program, 0, bank as u32),
            EventKind::Parameter {
                channel,
                parameter,
                value,
            } => (9, channel, parameter, 0, value as u32),
            EventKind::Control {
                channel,
                controller,
                value,
            } => (3, channel, controller, value, 0),
            EventKind::Bend { channel, value } => (4, channel, 0, 0, value as u32),
            EventKind::Tempo(value) => (5, 0, 0, 0, value),
            _ => continue,
        };
        if region.is_none_or(|[_, end]| source.tick < end) {
            output.push(Event {
                tick: source.tick,
                op,
                channel,
                a,
                b,
                value,
            });
        }
    }
    let marker = |tick, op| Event {
        tick,
        op,
        channel: 0,
        a: 0,
        b: 0,
        value: 0,
    };
    if let Some([start, end]) = region {
        // Snapshot precedes simultaneous loop_start events; loop_end is exclusive.
        output.insert(output.partition_point(|e| e.tick < start), marker(start, 6));
        output.push(marker(end, 7));
    } else {
        output.push(marker(ir.duration_ticks, 8));
    }
    if output.len() > crate::sequence_ir::MAX_EVENTS {
        return Err("Sequence exceeds the bounded kernel event capacity".into());
    }
    Ok(output)
}

/// v1 keeps its exact legacy bytes. Only streams requiring extended semantics use v2.
pub fn payload_version(events: &[Event]) -> u16 {
    if events
        .iter()
        .any(|event| event.op > 8 || (event.op == 3 && ![7, 10, 11, 64].contains(&event.a)))
    {
        2
    } else {
        1
    }
}
