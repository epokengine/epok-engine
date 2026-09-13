//! Target-neutral instrument voices. Source adapters resolve their hierarchy before
//! constructing this model; console encodings and SPU registers do not belong here.
use crate::instrument_ir::{LibraryIr, RegionIr, SampleIr};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopMode { Off, Continuous, UntilRelease }

/// Absolute timecents: 1200 * log2(seconds). The source's -32768 sentinel
/// conventionally means zero for delay/attack/hold. Source adapters apply the
/// parameter's legal range; decay/release do not use this sentinel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeCents(pub i32);
impl TimeCents {
    pub fn seconds(self) -> f64 {
        if self.0 == -32768 { 0. } else { 2_f64.powf(self.0 as f64 / 1200.) }
    }
    fn for_key(self, cents_per_key: i32, key: u8, maximum: i32) -> Self {
        let value = i64::from(self.0) + i64::from(cents_per_key) * (60 - i64::from(key));
        // SoundFont applies key offsets before checking the hold-only sentinel.
        if maximum == 5000 && value <= -32768 { Self(-32768) }
        else { Self(value.clamp(-12000, i64::from(maximum)) as i32) }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub delay: TimeCents, pub attack: TimeCents, pub hold: TimeCents,
    pub decay: TimeCents, pub release: TimeCents,
    pub hold_cents_per_key: i32, pub decay_cents_per_key: i32,
    pub sustain: Sustain,
}
impl Envelope {
    pub fn hold_for_key(&self, key: u8) -> TimeCents { self.hold.for_key(self.hold_cents_per_key, key, 5000) }
    pub fn decay_for_key(&self, key: u8) -> TimeCents { self.decay.for_key(self.decay_cents_per_key, key, 8000) }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sustain {
    /// Volume decreases by 0.1 dB per unit; this is not a linear amplitude.
    AttenuationCentibels(i32),
    /// Modulation envelope decreases by 0.1% of its full amplitude per unit.
    ReductionPermille(i32),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lfo {
    pub delay: TimeCents,
    /// Absolute cents from MIDI key zero (8.175798915643707 Hz).
    pub frequency_cents: i32,
    pub pitch_cents: i32, pub filter_cents: i32, pub volume_centibels: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Input { Constant, Velocity, Key, PolyPressure, ChannelPressure, PitchWheel, PitchWheelRange, Controller(u8) }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Curve { Linear, Concave, Convex, Switch }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModSource { pub input: Input, pub reversed: bool, pub bipolar: bool, pub curve: Curve }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Destination {
    StartFrames, EndFrames, LoopStartFrames, LoopEndFrames,
    ModLfoPitchCents, VibLfoPitchCents, ModEnvPitchCents,
    FilterCents, FilterCentibels, ModLfoFilterCents, ModEnvFilterCents,
    ModLfoVolumeCentibels, ChorusPermille, ReverbPermille, PanPermille,
    ModLfoDelayCents, ModLfoFrequencyCents, VibLfoDelayCents, VibLfoFrequencyCents,
    ModEnvDelayCents, ModEnvAttackCents, ModEnvHoldCents, ModEnvDecayCents,
    ModEnvSustainPermille, ModEnvReleaseCents, ModEnvHoldKeyCents, ModEnvDecayKeyCents,
    VolEnvDelayCents, VolEnvAttackCents, VolEnvHoldCents, VolEnvDecayCents,
    VolEnvSustainCentibels, VolEnvReleaseCents, VolEnvHoldKeyCents, VolEnvDecayKeyCents,
    AttenuationCentibels, PitchCents, ScaleCentsPerKey,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modulation {
    pub source: ModSource, pub amount_source: ModSource,
    pub destination: Destination, pub amount: i64, pub absolute: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voice {
    pub sample: u16,
    pub key_range: [u8; 2], pub velocity_range: [u8; 2],
    pub root_key: u8, pub fixed_key: Option<u8>, pub fixed_velocity: Option<u8>,
    pub tune_cents: i32, pub scale_cents_per_key: i32,
    /// Offsets relative to the decoded sample. An end offset is applied to its
    /// decoded frame count; loop positions below are already sample-relative.
    pub start_offset: i64, pub end_offset: i64,
    pub loop_start: i64, pub loop_end: i64, pub loop_mode: LoopMode,
    pub attenuation_centibels: i32, pub pan_permille: i32,
    pub filter_cents: i32, pub filter_centibels: i32,
    pub mod_env_pitch_cents: i32, pub mod_env_filter_cents: i32,
    pub reverb_permille: i32, pub chorus_permille: i32,
    pub volume_envelope: Envelope, pub modulation_envelope: Envelope,
    pub modulation_lfo: Lfo, pub vibrato_lfo: Lfo,
    /// Scoped to a channel/preset by playback, never a global percussion kill.
    pub exclusive_class: u16,
    pub modulations: Vec<Modulation>,
}

fn mod_source(bits: u16) -> Result<ModSource, String> {
    let index = (bits & 127) as u8;
    let input = if bits & 128 != 0 { Input::Controller(index) } else { match index {
        0 => Input::Constant, 2 => Input::Velocity, 3 => Input::Key, 10 => Input::PolyPressure,
        13 => Input::ChannelPressure, 14 => Input::PitchWheel, 16 => Input::PitchWheelRange,
        _ => return Err(format!("Unresolved modulation input {bits:#06x}")),
    }};
    let curve = match bits >> 10 {
        0 => Curve::Linear, 1 => Curve::Concave, 2 => Curve::Convex, 3 => Curve::Switch,
        _ => return Err(format!("Unresolved modulation curve {bits:#06x}")),
    };
    Ok(ModSource { input, curve, reversed: bits & 256 != 0, bipolar: bits & 512 != 0 })
}

fn destination(op: u16) -> Result<(Destination, i64), String> {
    use Destination::*;
    Ok(match op {
        0 => (StartFrames, 1), 1 => (EndFrames, 1), 2 => (LoopStartFrames, 1), 3 => (LoopEndFrames, 1),
        4 => (StartFrames, 32768), 5 => (ModLfoPitchCents, 1), 6 => (VibLfoPitchCents, 1),
        7 => (ModEnvPitchCents, 1), 8 => (FilterCents, 1), 9 => (FilterCentibels, 1),
        10 => (ModLfoFilterCents, 1), 11 => (ModEnvFilterCents, 1), 12 => (EndFrames, 32768),
        13 => (ModLfoVolumeCentibels, 1), 15 => (ChorusPermille, 1), 16 => (ReverbPermille, 1), 17 => (PanPermille, 1),
        21 => (ModLfoDelayCents, 1), 22 => (ModLfoFrequencyCents, 1), 23 => (VibLfoDelayCents, 1), 24 => (VibLfoFrequencyCents, 1),
        25 => (ModEnvDelayCents, 1), 26 => (ModEnvAttackCents, 1), 27 => (ModEnvHoldCents, 1), 28 => (ModEnvDecayCents, 1),
        29 => (ModEnvSustainPermille, 1), 30 => (ModEnvReleaseCents, 1), 31 => (ModEnvHoldKeyCents, 1), 32 => (ModEnvDecayKeyCents, 1),
        33 => (VolEnvDelayCents, 1), 34 => (VolEnvAttackCents, 1), 35 => (VolEnvHoldCents, 1), 36 => (VolEnvDecayCents, 1),
        37 => (VolEnvSustainCentibels, 1), 38 => (VolEnvReleaseCents, 1), 39 => (VolEnvHoldKeyCents, 1), 40 => (VolEnvDecayKeyCents, 1),
        45 => (LoopStartFrames, 32768), 48 => (AttenuationCentibels, 1), 50 => (LoopEndFrames, 32768),
        51 => (PitchCents, 100), 52 | 59 => (PitchCents, 1), 56 => (ScaleCentsPerKey, 1),
        _ => return Err(format!("Unresolved modulation destination {op}")),
    })
}

/// SoundFont adapter. Unknown necessary operations stay errors; interpreting this
/// IR does not by itself assert that a target can execute its effects.
pub fn from_soundfont(library: &LibraryIr, region: &RegionIr) -> Result<Voice, String> {
    if let Some(issue) = library.blockers.first().or(region.blockers.first()) {
        return Err(format!("{}: {}", issue.code, issue.message));
    }
    let sample: &SampleIr = library.samples.get(region.sample as usize).ok_or("Instrument sample index is invalid")?;
    let g = |op| region.effective_generators.get(&op).copied().unwrap_or(0);
    let root = if g(58) >= 0 { g(58) } else { i32::from(sample.root_key) };
    if !(0..=127).contains(&root) { return Err("Instrument requires an explicit sample root key".into()); }
    let loop_mode = match g(54) { 0 => LoopMode::Off, 1 => LoopMode::Continuous, 3 => LoopMode::UntilRelease,
        value => return Err(format!("Unresolved instrument loop mode {value}")) };
    let envelope = |base, sustain| Envelope {
        delay: TimeCents(g(base)), attack: TimeCents(g(base + 1)), hold: TimeCents(g(base + 2)),
        decay: TimeCents(g(base + 3)), release: TimeCents(g(base + 5)),
        hold_cents_per_key: g(base + 6), decay_cents_per_key: g(base + 7), sustain,
    };
    let offset = |fine, coarse| i64::from(g(fine)) + 32768 * i64::from(g(coarse));
    let loop_base = if sample.sample_type & 0x10 != 0 { 0 } else { i64::from(sample.start) };
    let mut modulations = Vec::with_capacity(region.effective_modulators.len());
    for source in &region.effective_modulators {
        let (destination, scale) = destination(source.destination)?;
        if !matches!(source.transform, 0 | 2) { return Err(format!("Unresolved modulation transform {}", source.transform)); }
        modulations.push(Modulation { source: mod_source(source.source)?, amount_source: mod_source(source.amount_source)?,
            destination, amount: i64::from(source.amount) * scale, absolute: source.transform == 2 });
    }
    Ok(Voice {
        sample: region.sample, key_range: region.key_range, velocity_range: region.velocity_range,
        root_key: root as u8, fixed_key: (g(46) >= 0).then_some(g(46) as u8), fixed_velocity: (g(47) >= 0).then_some(g(47) as u8),
        tune_cents: g(51) * 100 + g(52) + i32::from(sample.pitch_correction), scale_cents_per_key: g(56),
        start_offset: offset(0, 4), end_offset: offset(1, 12),
        loop_start: i64::from(sample.loop_start) - loop_base + offset(2, 45),
        loop_end: i64::from(sample.loop_end) - loop_base + offset(3, 50), loop_mode,
        attenuation_centibels: g(48), pan_permille: g(17), filter_cents: g(8), filter_centibels: g(9),
        mod_env_pitch_cents: g(7), mod_env_filter_cents: g(11), reverb_permille: g(16), chorus_permille: g(15),
        volume_envelope: envelope(33, Sustain::AttenuationCentibels(g(37))),
        modulation_envelope: envelope(25, Sustain::ReductionPermille(g(29))),
        modulation_lfo: Lfo { delay: TimeCents(g(21)), frequency_cents: g(22), pitch_cents: g(5), filter_cents: g(10), volume_centibels: g(13) },
        vibrato_lfo: Lfo { delay: TimeCents(g(23)), frequency_cents: g(24), pitch_cents: g(6), filter_cents: 0, volume_centibels: 0 },
        exclusive_class: region.exclusive_class, modulations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn soundfont_voice_preserves_timing_pitch_loops_and_default_modulation_units() {
        let mut library = crate::sf2::parse(&crate::sf2::fixture()).unwrap();
        library.samples[0].start = 1000;
        library.samples[0].loop_start = 1010;
        library.samples[0].loop_end = 1100;
        library.samples[0].pitch_correction = -7;
        let mut region = library.presets[0].regions[0].clone();
        region.effective_generators.extend([(51, 2), (52, 3), (54, 3), (2, 4), (3, -2), (37, 60), (29, 250)]);
        let voice = from_soundfont(&library, &region).unwrap();
        assert_eq!(voice.tune_cents, 196);
        assert_eq!((voice.loop_start, voice.loop_end, voice.loop_mode), (14, 98, LoopMode::UntilRelease));
        assert_eq!(voice.volume_envelope.sustain, Sustain::AttenuationCentibels(60));
        assert_eq!(voice.modulation_envelope.sustain, Sustain::ReductionPermille(250));
        assert_eq!(voice.modulations.len(), 10);
        let wheel = voice.modulations.iter().find(|m| m.source.input == Input::PitchWheel).unwrap();
        assert_eq!((wheel.destination, wheel.amount, wheel.amount_source.input), (Destination::PitchCents, 12700, Input::PitchWheelRange));
        assert!(wheel.source.bipolar);
        library.samples[0].sample_type = 0x11;
        let sf3 = from_soundfont(&library, &region).unwrap();
        assert_eq!((sf3.loop_start, sf3.loop_end), (1014, 1098), "compressed byte start is never subtracted from PCM loops");
        assert_eq!(TimeCents(0).seconds(), 1.);
        assert_eq!(TimeCents(0).for_key(100, 72, 8000).seconds(), 0.5);
        assert_eq!(TimeCents(-32768).for_key(100, 20, 5000), TimeCents(-12000));
        assert_eq!(TimeCents(-32768).for_key(100, 100, 5000).seconds(), 0.);
        assert_eq!(TimeCents(-32768).for_key(0, 60, 8000), TimeCents(-12000));
        let mut envelope = voice.volume_envelope.clone();
        envelope.hold = TimeCents(4900); envelope.decay = TimeCents(4900);
        envelope.hold_cents_per_key = 100; envelope.decay_cents_per_key = 100;
        assert_eq!(envelope.hold_for_key(58), TimeCents(5000));
        assert_eq!(envelope.decay_for_key(58), TimeCents(5100));
        region.blockers.push(crate::instrument_ir::ImportDiagnostic { code: "unsupported".into(), message: "required effect".into() });
        assert!(from_soundfont(&library, &region).unwrap_err().contains("required effect"));
    }
}
