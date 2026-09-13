//! EPSB v2 serialization for cooked, song-specific PSX instrument libraries.
//!
//! This module only writes the bounded host payload. The runtime reader validates
//! it independently; no JSON or authoring representation is embedded in EPSB.
use crate::{
    instrument_voice::{Curve, Destination, Input, LoopMode, ModSource, Sustain, Voice},
    psx_library::{Cooked, Sample, Zone, MAX_SAMPLES, MAX_ZONES},
    psx_music_settings::MAX_RATE,
};

pub const VERSION: u16 = 2;
pub const HEADER_BYTES: usize = 48;
pub const SAMPLE_BYTES: usize = 24;
pub const ZONE_BYTES: usize = 144;
pub const MOD_BYTES: usize = 16;
pub const MAX_MODULATORS_PER_ZONE: usize = 32;
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;
const PROFILE: u32 = 2;

/// Writes the fixed-layout EPSB v2 payload. An over-budget `Cooked` remains
/// serializable for analysis; publication is gated by its report elsewhere.
pub fn encode(cooked: &Cooked) -> Result<Vec<u8>, String> {
    cooked.report.recipe.validate()?;
    if cooked.samples.is_empty() || cooked.zones.is_empty() {
        return Err("EPSB v2 requires non-empty sample and zone tables".into());
    }
    if cooked.samples.len() > MAX_SAMPLES || cooked.zones.len() > MAX_ZONES {
        return Err("EPSB v2 exceeds the 128 sample or zone wire limit".into());
    }
    // Validate every existing allocation and calculate the complete wire extent
    // before allocating writer tables or the output payload.
    for sample in &cooked.samples {
        validate_sample(sample)?;
    }
    let mut mod_count = 0_usize;
    for (index, zone) in cooked.zones.iter().enumerate() {
        validate_zone(zone, &cooked.samples)?;
        if cooked.report.recipe.effects == crate::psx_music_settings::Effects::Dry
            && (zone.voice.reverb_permille != 0 || zone.voice.modulations.iter().any(|m| m.destination == Destination::ReverbPermille && m.amount != 0))
        {
            return Err("EPSB v2 Dry recipe retains a non-dry reverb send".into());
        }
        if zone.voice.modulations.len() > MAX_MODULATORS_PER_ZONE {
            return Err(format!(
                "EPSB v2 zone {index} has more than {MAX_MODULATORS_PER_ZONE} modulators"
            ));
        }
        for modulation in &zone.voice.modulations {
            validate_modulator(modulation)?;
        }
        mod_count = checked_add(mod_count, zone.voice.modulations.len())?;
    }
    if mod_count > MAX_ZONES * MAX_MODULATORS_PER_ZONE {
        return Err("EPSB v2 modulator table exceeds its bounded zone spans".into());
    }
    let sample_offset = HEADER_BYTES;
    let zone_offset = checked_add(
        sample_offset,
        checked_mul(cooked.samples.len(), SAMPLE_BYTES)?,
    )?;
    let mod_offset = checked_add(zone_offset, checked_mul(cooked.zones.len(), ZONE_BYTES)?)?;
    let data_offset = align64(checked_add(mod_offset, checked_mul(mod_count, MOD_BYTES)?)?)?;
    let mut total = data_offset;
    for sample in &cooked.samples {
        total = checked_add(total, sample.bytes.len())?;
    }
    if total > MAX_PAYLOAD_BYTES {
        return Err(format!(
            "EPSB v2 analysis payload exceeds {MAX_PAYLOAD_BYTES} bytes"
        ));
    }
    let total_u32 = as_u32(total, "EPSB v2 payload")?;
    let mut sample_offsets = Vec::with_capacity(cooked.samples.len());
    let mut offset = data_offset;
    for sample in &cooked.samples {
        sample_offsets.push(offset);
        offset = checked_add(offset, sample.bytes.len())?;
    }
    debug_assert_eq!(offset, total);
    let mut encoded_modulators = Vec::with_capacity(mod_count);
    for zone in &cooked.zones {
        for modulation in &zone.voice.modulations {
            encoded_modulators.push(encode_modulator(modulation)?);
        }
    }
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"EPSB");
    u16le(&mut out, VERSION);
    u16le(&mut out, HEADER_BYTES as u16);
    u16le(&mut out, cooked.samples.len() as u16);
    u16le(&mut out, cooked.zones.len() as u16);
    u32le(&mut out, as_u32(sample_offset, "EPSB v2 sample table")?);
    u32le(&mut out, as_u32(zone_offset, "EPSB v2 zone table")?);
    u32le(&mut out, as_u32(mod_offset, "EPSB v2 modulator table")?);
    u32le(&mut out, as_u32(data_offset, "EPSB v2 sample data")?);
    u32le(&mut out, total_u32);
    u32le(&mut out, as_u32(mod_count, "EPSB v2 modulator count")?);
    u32le(&mut out, PROFILE);
    u32le(&mut out, u32::from(cooked.report.recipe.effects == crate::psx_music_settings::Effects::Room));
    u32le(&mut out, u32::from(cooked.report.recipe.reverb_depth_q15()));
    debug_assert_eq!(out.len(), HEADER_BYTES);

    for (sample, &offset) in cooked.samples.iter().zip(&sample_offsets) {
        u32le(&mut out, as_u32(offset, "EPSB v2 sample offset")?);
        u32le(
            &mut out,
            as_u32(sample.bytes.len(), "EPSB v2 sample length")?,
        );
        u32le(&mut out, sample.rate);
        u32le(&mut out, sample.frames);
        u32le(&mut out, sample.loop_region.map_or(0, |region| region[0]));
        u32le(&mut out, sample.loop_region.map_or(0, |region| region[1]));
    }
    debug_assert_eq!(out.len(), zone_offset);

    let mut mod_begin = 0_usize;
    for zone in &cooked.zones {
        let count = zone.voice.modulations.len();
        encode_zone(&mut out, zone, mod_begin, count)?;
        mod_begin += count;
    }
    debug_assert_eq!(out.len(), mod_offset);
    for modulation in &encoded_modulators {
        out.extend_from_slice(modulation);
    }
    out.resize(data_offset, 0);
    for (sample, &offset) in cooked.samples.iter().zip(&sample_offsets) {
        debug_assert_eq!(out.len(), offset);
        out.extend_from_slice(&sample.bytes);
    }
    if out.len() != total {
        return Err("EPSB v2 internal payload size mismatch".into());
    }
    Ok(out)
}

fn validate_sample(sample: &Sample) -> Result<(), String> {
    if !(400..=MAX_RATE).contains(&sample.rate)
        || sample.frames < 56
        || sample.frames % 28 != 0
        || sample.bytes.len() < 64
        || !sample.bytes.len().is_multiple_of(64)
    {
        return Err("EPSB v2 sample has invalid rate, frame count, or 64-byte ADPCM extent".into());
    }
    let [loop_start, loop_end] = sample.loop_region.unwrap_or([0, 0]);
    if loop_end != 0 {
        let start = loop_start;
        let end = loop_end;
        if start < 28 || start >= end || end > sample.frames || start % 28 != 0 || end % 28 != 0 {
            return Err("EPSB v2 sample loop is not an in-range 28-frame exclusive region".into());
        }
    } else if loop_start != 0 {
        return Err("EPSB v2 non-looping sample retains a loop start".into());
    }
    let blocks = sample.frames / 28;
    let extent = usize::try_from(u64::from(blocks) + 1)
        .ok()
        .and_then(|blocks| blocks.checked_mul(16))
        .ok_or("EPSB v2 ADPCM extent overflows")?;
    if extent > sample.bytes.len() || sample.bytes[..16].iter().any(|byte| *byte != 0) {
        return Err("EPSB v2 sample has truncated ADPCM blocks or lacks initial silence".into());
    }
    for block in 1..blocks {
        let header = sample.bytes[block as usize * 16];
        let flags = sample.bytes[block as usize * 16 + 1];
        if header >> 4 > 4 || header & 15 > 12 || flags & !7 != 0 {
            return Err("EPSB v2 sample has invalid ADPCM header or flags".into());
        }
        let mut expected = if block == blocks - 1 { 1 } else { 0 };
        if loop_end != 0 {
            if block == loop_end / 28 - 1 {
                expected = 3;
            }
            if block == loop_start / 28 {
                expected |= 4;
            }
        }
        let manual_repeat = loop_end != 0 && block == loop_start / 28 && flags == (expected & !4);
        if flags != expected && !manual_repeat {
            return Err("EPSB v2 sample ADPCM flags do not match its loop region".into());
        }
    }
    let terminal = blocks as usize * 16;
    if sample.bytes[terminal] != 0
        || sample.bytes[terminal + 1] != 7
        || sample.bytes[terminal + 2..].iter().any(|byte| *byte != 0)
    {
        return Err("EPSB v2 sample lacks a valid terminal silence block or zero padding".into());
    }
    Ok(())
}

fn validate_zone(zone: &Zone, samples: &[Sample]) -> Result<(), String> {
    let voice = &zone.voice;
    let sample = samples
        .get(usize::from(zone.sample))
        .ok_or("EPSB v2 zone points to an invalid cooked sample")?;
    if voice.sample != zone.sample {
        return Err("EPSB v2 zone points to an invalid cooked sample".into());
    }
    if zone.instrument.bank > 16_383
        || zone.instrument.program > 127
        || voice.key_range[0] > voice.key_range[1]
        || voice.key_range[1] > 127
        || voice.velocity_range[0] > voice.velocity_range[1]
        || voice.velocity_range[1] > 127
        || voice.root_key > 127
        || voice.fixed_key.is_some_and(|key| key > 127)
        || voice.fixed_velocity.is_some_and(|velocity| velocity > 127)
        || voice.exclusive_class > 127
        || !within(voice.tune_cents, -14_000, 14_000)
        || !within(voice.scale_cents_per_key, 0, 1_200)
        || !within(voice.attenuation_centibels, 0, 1_440)
        || !within(voice.pan_permille, -500, 500)
        || !within(voice.mod_env_pitch_cents, -12_000, 12_000)
    {
        return Err("EPSB v2 zone has an invalid MIDI range, key, velocity, or bank".into());
    }
    if voice.start_offset != 0 || voice.end_offset != 0 {
        return Err("EPSB v2 requires cooked source offsets to be zero".into());
    }
    validate_baked_filter(voice)?;
    if !valid_envelope(&voice.volume_envelope, true)
        || !valid_envelope(&voice.modulation_envelope, false)
        || !valid_lfo(&voice.modulation_lfo, false)
        || !valid_lfo(&voice.vibrato_lfo, true)
    {
        return Err("EPSB v2 zone envelope or LFO is outside runtime limits".into());
    }
    if !(0..=1000).contains(&voice.reverb_permille) || voice.chorus_permille != 0 {
        return Err("EPSB v2 requires a bounded reverb send and dry chorus".into());
    }
    match voice.loop_mode {
        LoopMode::Off
            if voice.loop_start != 0 || voice.loop_end != 0 || sample.loop_region.is_some() =>
        {
            return Err("EPSB v2 non-looping zone retains loop coordinates".into());
        }
        LoopMode::Continuous | LoopMode::UntilRelease
            if sample
                .loop_region
                .map(|[start, end]| (i64::from(start), i64::from(end)))
                != Some((voice.loop_start, voice.loop_end))
                || voice.loop_start < 28
                || voice.loop_start >= voice.loop_end
                || (voice.loop_mode == LoopMode::Continuous
                    && voice.loop_end != i64::from(sample.frames)) =>
        {
            return Err("EPSB v2 looping zone has invalid cooked loop coordinates".into());
        }
        _ => {}
    }
    if voice.loop_mode == LoopMode::UntilRelease
        && sample.bytes[(voice.loop_start / 28 * 16 + 1) as usize] & 4 != 0
    {
        return Err("EPSB v2 UntilRelease requires a software-owned repeat address".into());
    }
    if voice.loop_mode == LoopMode::Continuous
        && sample.bytes[(voice.loop_start / 28 * 16 + 1) as usize] & 4 == 0
    {
        return Err("EPSB v2 Continuous requires its ADPCM loop-start flag".into());
    }
    Ok(())
}

fn validate_baked_filter(voice: &Voice) -> Result<(), String> {
    if voice.filter_cents != 13_500
        || voice.filter_centibels != 0
        || voice.mod_env_filter_cents != 0
        || voice.modulation_lfo.filter_cents != 0
        || voice.vibrato_lfo.filter_cents != 0
    {
        return Err("EPSB v2 requires filter fields baked and removed before serialization".into());
    }
    Ok(())
}

fn within(value: i32, low: i32, high: i32) -> bool {
    (low..=high).contains(&value)
}

fn encode_zone(
    out: &mut Vec<u8>,
    zone: &Zone,
    mod_begin: usize,
    mod_count: usize,
) -> Result<(), String> {
    let begin = out.len();
    let voice = &zone.voice;
    u16le(out, zone.sample);
    u16le(out, zone.instrument.bank);
    out.push(zone.instrument.program);
    out.push(u8::from(zone.instrument.percussion));
    out.extend_from_slice(&[
        voice.key_range[0],
        voice.key_range[1],
        voice.velocity_range[0],
        voice.velocity_range[1],
    ]);
    out.push(voice.root_key);
    out.push(voice.fixed_key.unwrap_or(255));
    out.push(voice.fixed_velocity.unwrap_or(255));
    out.push(loop_mode(voice.loop_mode));
    u16le(out, voice.exclusive_class);
    i32le(out, voice.tune_cents);
    i32le(out, voice.scale_cents_per_key);
    i32le(out, voice.attenuation_centibels);
    i32le(out, voice.pan_permille);
    i32le(out, voice.mod_env_pitch_cents);
    i32le(out, voice.reverb_permille);
    encode_envelope(out, &voice.volume_envelope, true)?;
    encode_envelope(out, &voice.modulation_envelope, false)?;
    encode_lfo(out, &voice.modulation_lfo, true)?;
    encode_lfo(out, &voice.vibrato_lfo, false)?;
    u16le(
        out,
        u16::try_from(mod_begin).map_err(|_| "EPSB v2 modulator begin exceeds u16")?,
    );
    u16le(
        out,
        u16::try_from(mod_count).map_err(|_| "EPSB v2 modulator count exceeds u16")?,
    );
    u32le(out, 0);
    debug_assert_eq!(out.len() - begin, ZONE_BYTES);
    Ok(())
}

fn encode_envelope(
    out: &mut Vec<u8>,
    envelope: &crate::instrument_voice::Envelope,
    volume: bool,
) -> Result<(), String> {
    if !valid_envelope(envelope, volume) {
        return Err("EPSB v2 envelope is outside runtime limits".into());
    }
    for value in [
        envelope.delay.0,
        envelope.attack.0,
        envelope.hold.0,
        envelope.decay.0,
        match (volume, envelope.sustain) {
            (true, Sustain::AttenuationCentibels(value))
            | (false, Sustain::ReductionPermille(value)) => value,
            _ => return Err("EPSB v2 envelope sustain has incompatible neutral units".into()),
        },
        envelope.release.0,
        envelope.hold_cents_per_key,
        envelope.decay_cents_per_key,
    ] {
        i32le(out, value);
    }
    Ok(())
}

fn encode_lfo(
    out: &mut Vec<u8>,
    lfo: &crate::instrument_voice::Lfo,
    modulation: bool,
) -> Result<(), String> {
    if !valid_lfo(lfo, !modulation) {
        return Err("EPSB v2 LFO is outside runtime limits".into());
    }
    if !modulation && lfo.filter_cents != 0 {
        return Err("EPSB v2 vibrato LFO cannot carry a filter field".into());
    }
    i32le(out, lfo.delay.0);
    i32le(out, lfo.frequency_cents);
    i32le(out, lfo.pitch_cents);
    i32le(out, if modulation { lfo.volume_centibels } else { 0 });
    Ok(())
}

fn valid_time(value: i32, maximum: i32, zero_sentinel: bool) -> bool {
    (zero_sentinel && value == -32_768) || within(value, -12_000, maximum)
}

fn valid_envelope(envelope: &crate::instrument_voice::Envelope, volume: bool) -> bool {
    let sustain = match envelope.sustain {
        Sustain::AttenuationCentibels(value) if volume => within(value, 0, 1_440),
        Sustain::ReductionPermille(value) if !volume => within(value, 0, 1_000),
        _ => false,
    };
    valid_time(envelope.delay.0, 5_000, true)
        && valid_time(envelope.attack.0, 8_000, true)
        && valid_time(envelope.hold.0, 5_000, true)
        && valid_time(envelope.decay.0, 8_000, false)
        && valid_time(envelope.release.0, 8_000, false)
        && sustain
        && within(envelope.hold_cents_per_key, -1_200, 1_200)
        && within(envelope.decay_cents_per_key, -1_200, 1_200)
}

fn valid_lfo(lfo: &crate::instrument_voice::Lfo, vibrato: bool) -> bool {
    valid_time(lfo.delay.0, 5_000, true)
        && within(lfo.frequency_cents, -16_000, 4_500)
        && within(lfo.pitch_cents, -12_000, 12_000)
        && if vibrato {
            lfo.volume_centibels == 0
        } else {
            within(lfo.volume_centibels, -960, 960)
        }
}

fn encode_modulator(
    modulation: &crate::instrument_voice::Modulation,
) -> Result<[u8; MOD_BYTES], String> {
    validate_modulator(modulation)?;
    let destination = destination_code(modulation.destination, modulation.amount)?;
    let amount = modulation.amount as i32;
    let mut out = Vec::with_capacity(MOD_BYTES);
    u16le(&mut out, source_bits(modulation.source)?);
    u16le(&mut out, source_bits(modulation.amount_source)?);
    u16le(&mut out, destination);
    u16le(&mut out, u16::from(modulation.absolute));
    i32le(&mut out, amount);
    u32le(&mut out, 0);
    Ok(out.try_into().unwrap())
}

fn validate_modulator(modulation: &crate::instrument_voice::Modulation) -> Result<(), String> {
    source_bits(modulation.source)?;
    source_bits(modulation.amount_source)?;
    destination_code(modulation.destination, modulation.amount)?;
    i32::try_from(modulation.amount).map_err(|_| "EPSB v2 modulation amount exceeds i32")?;
    Ok(())
}

/// Stable EPSB v2 neutral destination codes. Filter and sample-position
/// destinations are rejected above rather than silently dropped.
fn destination_code(destination: Destination, amount: i64) -> Result<u16, String> {
    let code = match destination {
        Destination::PitchCents => 1,
        Destination::ScaleCentsPerKey => 2,
        Destination::AttenuationCentibels => 3,
        Destination::PanPermille => 4,
        Destination::ModEnvPitchCents => 5,
        Destination::ModLfoPitchCents => 6,
        Destination::VibLfoPitchCents => 7,
        Destination::ModLfoVolumeCentibels => 8,
        Destination::ModLfoDelayCents => 9,
        Destination::ModLfoFrequencyCents => 10,
        Destination::VibLfoDelayCents => 11,
        Destination::VibLfoFrequencyCents => 12,
        Destination::VolEnvDelayCents => 13,
        Destination::VolEnvAttackCents => 14,
        Destination::VolEnvHoldCents => 15,
        Destination::VolEnvDecayCents => 16,
        Destination::VolEnvSustainCentibels => 17,
        Destination::VolEnvReleaseCents => 18,
        Destination::VolEnvHoldKeyCents => 19,
        Destination::VolEnvDecayKeyCents => 20,
        Destination::ModEnvDelayCents => 21,
        Destination::ModEnvAttackCents => 22,
        Destination::ModEnvHoldCents => 23,
        Destination::ModEnvDecayCents => 24,
        Destination::ModEnvSustainPermille => 25,
        Destination::ModEnvReleaseCents => 26,
        Destination::ModEnvHoldKeyCents => 27,
        Destination::ModEnvDecayKeyCents => 28,
        Destination::ReverbPermille => 29,
        Destination::ChorusPermille => 30,
        Destination::StartFrames
        | Destination::EndFrames
        | Destination::LoopStartFrames
        | Destination::LoopEndFrames => {
            return Err("EPSB v2 cannot serialize sample position modulation".into())
        }
        Destination::FilterCents
        | Destination::FilterCentibels
        | Destination::ModLfoFilterCents
        | Destination::ModEnvFilterCents => {
            return Err("EPSB v2 cannot serialize unbaked filter modulation".into())
        }
    };
    if matches!(
        destination,
        Destination::ChorusPermille
    ) && amount != 0
    {
        return Err("EPSB v2 requires dry effect modulation sends".into());
    }
    Ok(code)
}

/// Source bits are stable and compatible with SoundFont's source shape: input
/// index (or controller bit 7), reversed bit 8, bipolar bit 9, curve bits 10–11.
fn source_bits(source: ModSource) -> Result<u16, String> {
    let input = match source.input {
        Input::Constant => 0,
        Input::Velocity => 2,
        Input::Key => 3,
        Input::PolyPressure => 10,
        Input::ChannelPressure => 13,
        Input::PitchWheel => 14,
        Input::PitchWheelRange => 16,
        Input::Controller(index) if index < 120 && !matches!(index, 0 | 6 | 32..=63 | 98..=101) => 0x80 | u16::from(index),
        Input::Controller(_) => {
            return Err("EPSB v2 controller source exceeds MIDI CC range".into())
        }
    };
    let curve = match source.curve {
        Curve::Linear => 0,
        Curve::Concave => 1,
        Curve::Convex => 2,
        Curve::Switch => 3,
    };
    Ok(
        input
            | (u16::from(source.reversed) << 8)
            | (u16::from(source.bipolar) << 9)
            | (curve << 10),
    )
}

fn loop_mode(mode: LoopMode) -> u8 {
    match mode {
        LoopMode::Off => 0,
        LoopMode::Continuous => 1,
        LoopMode::UntilRelease => 3,
    }
}

fn checked_add(left: usize, right: usize) -> Result<usize, String> {
    left.checked_add(right)
        .ok_or("EPSB v2 size overflow".into())
}

fn checked_mul(left: usize, right: usize) -> Result<usize, String> {
    left.checked_mul(right)
        .ok_or("EPSB v2 size overflow".into())
}

fn align64(value: usize) -> Result<usize, String> {
    checked_add(value, 63).map(|value| value / 64 * 64)
}

fn as_u32(value: usize, name: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("{name} exceeds the EPSB v2 u32 limit"))
}

fn u16le(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn u32le(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn i32le(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        instrument_selection::Instrument, instrument_voice::Modulation, psx_music_settings::Recipe,
    };

    fn fixture() -> Cooked {
        let library = crate::sf2::parse(&crate::sf2::fixture()).unwrap();
        let mut voice =
            crate::instrument_voice::from_soundfont(&library, &library.presets[0].regions[0])
                .unwrap();
        voice.sample = 0;
        voice.start_offset = 0;
        voice.end_offset = 0;
        voice.loop_start = 0;
        voice.loop_end = 0;
        voice.loop_mode = LoopMode::Off;
        voice.filter_cents = 13_500;
        voice.filter_centibels = 0;
        voice.mod_env_filter_cents = 0;
        voice.modulation_lfo.filter_cents = 0;
        voice.vibrato_lfo.filter_cents = 0;
        voice.reverb_permille = 0;
        voice.chorus_permille = 0;
        voice.modulations.clear();
        let recipe = Recipe::default();
        let mut sample_bytes = vec![0; 64];
        sample_bytes[17] = 1;
        sample_bytes[33] = 7;
        Cooked {
            samples: vec![Sample {
                source_sample: 0,
                bytes: sample_bytes,
                rate: 22_050,
                frames: 56,
                loop_region: None,
                squared_error: 0,
                input_frames: 28,
                improved_encoder: false,
            }],
            zones: vec![Zone {
                sample: 0,
                instrument: Instrument {
                    bank: 0,
                    program: 7,
                    percussion: false,
                },
                voice,
            }],
            report: crate::psx_library::Report {
                profile: "test".into(),
                recipe,
                note_events: 0,
                regions: 1,
                samples: 1,
                song_peak_layers_per_note: 1,
                sample_spu_bytes: 64,
                reverb_spu_bytes: 0,
                other_resident_bytes: 0,
                available_bank_bytes: crate::audio_import::SPU_BUDGET as u32,
                fits_sample_budget: true,
                adaptations: vec![],
                squared_error: 0,
                encoded_input_frames: 28,
                maximum_loop_step: 0.,
                accounting: Default::default(), loops: vec![],
            },
        }
    }

    fn read_u32(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    #[test]
    fn writer_emits_fixed_header_sample_zone_and_aligned_data() {
        let bytes = encode(&fixture()).unwrap();
        assert_eq!(&bytes[..8], b"EPSB\x02\0\x30\0");
        assert_eq!(read_u32(&bytes, 12), 48);
        assert_eq!(read_u32(&bytes, 16), 72);
        assert_eq!(read_u32(&bytes, 20), 216);
        assert_eq!(read_u32(&bytes, 24), 256);
        assert_eq!(read_u32(&bytes, 28), 320);
        assert_eq!(read_u32(&bytes, 36), 2);
        assert_eq!(read_u32(&bytes, 48), 256);
        assert_eq!(read_u32(&bytes, 52), 64);
        assert_eq!(u16::from_le_bytes(bytes[72..74].try_into().unwrap()), 0);
        assert_eq!(bytes[76], 7);
        assert_eq!(bytes.len(), 320);
    }

    #[test]
    fn stable_source_and_destination_codes_preserve_all_modulation_bits() {
        let source = ModSource {
            input: Input::Controller(74),
            reversed: true,
            bipolar: true,
            curve: Curve::Switch,
        };
        assert_eq!(source_bits(source).unwrap(), 0x0FCA);
        assert_eq!(destination_code(Destination::PitchCents, 1).unwrap(), 1);
        assert_eq!(
            destination_code(Destination::ModEnvDecayKeyCents, 1).unwrap(),
            28
        );
        assert!(destination_code(Destination::FilterCents, 0).is_err());
    }

    #[test]
    fn rejects_modulator_amount_overflow_and_non_dry_sends() {
        let source = ModSource {
            input: Input::Constant,
            reversed: false,
            bipolar: false,
            curve: Curve::Linear,
        };
        let overflow = Modulation {
            source,
            amount_source: source,
            destination: Destination::PitchCents,
            amount: i64::from(i32::MAX) + 1,
            absolute: false,
        };
        assert!(encode_modulator(&overflow).is_err());
        assert_eq!(destination_code(Destination::ReverbPermille, 1).unwrap(), 29);
    }

    #[test]
    fn rejects_misaligned_or_truncated_sample_extents() {
        let sample = Sample {
            source_sample: 0,
            bytes: vec![0; 63],
            rate: 22_050,
            frames: 56,
            loop_region: None,
            squared_error: 0,
            input_frames: 28,
            improved_encoder: false,
        };
        assert!(validate_sample(&sample).is_err());
    }
}
