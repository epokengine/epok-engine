//! Evaluation of target-neutral SoundFont modulation sources.
//!
//! Values returned by [`evaluate`] remain in the units named by each
//! [`Destination`]. Applying them to a target voice is deliberately a separate
//! cooking step.

use crate::instrument_voice::{Curve, Destination, Input, ModSource, Voice};

/// P3's bounded per-voice modulation profile.
pub const MAX_MODULATIONS_PER_VOICE: usize = 32;

/// MIDI state needed to evaluate the SoundFont controller model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Controls {
    pub cc: [u8; 128],
    pub poly_pressure: u8,
    pub channel_pressure: u8,
    /// Fourteen-bit MIDI pitch wheel value, centered at 8192.
    pub bend: u16,
    /// Current pitch-wheel sensitivity, expressed in cents.
    pub bend_range_cents: u16,
}

impl Default for Controls {
    fn default() -> Self {
        let mut cc = [0; 128];
        cc[7] = 100;
        cc[10] = 64;
        cc[11] = 127;
        Self {
            cc,
            poly_pressure: 0,
            channel_pressure: 0,
            bend: 8192,
            bend_range_cents: 200,
        }
    }
}

/// Evaluates and sums a voice's modulators. The first occurrence of a
/// destination determines its position in the result.
pub fn evaluate(
    voice: &Voice,
    key: u8,
    velocity: u8,
    controls: &Controls,
) -> Result<Vec<(Destination, f64)>, String> {
    if voice.modulations.len() > MAX_MODULATIONS_PER_VOICE {
        return Err(format!(
            "Voice has {} modulators; the P3 cooking limit is {MAX_MODULATIONS_PER_VOICE}",
            voice.modulations.len()
        ));
    }
    validate_inputs(key, velocity, controls)?;
    let actual_key = voice.fixed_key.unwrap_or(key);
    let actual_velocity = voice.fixed_velocity.unwrap_or(velocity);
    if actual_key > 127 || actual_velocity > 127 {
        return Err("Fixed key and velocity must be seven-bit MIDI values".into());
    }

    let mut sums: Vec<(Destination, f64)> = Vec::with_capacity(voice.modulations.len());
    for modulation in &voice.modulations {
        let primary = source_value(modulation.source, actual_key, actual_velocity, controls)?;
        let amount_source =
            source_value(modulation.amount_source, actual_key, actual_velocity, controls)?;
        let mut value = modulation.amount as f64 * primary * amount_source;
        if modulation.absolute {
            value = value.abs();
        }
        if !value.is_finite() {
            return Err("A modulation produced a non-finite result".into());
        }
        if let Some((_, sum)) = sums
            .iter_mut()
            .find(|(destination, _)| *destination == modulation.destination)
        {
            *sum += value;
            if !sum.is_finite() {
                return Err("Summed modulation value is non-finite".into());
            }
        } else {
            sums.push((modulation.destination, value));
        }
    }
    Ok(sums)
}

fn validate_inputs(key: u8, velocity: u8, controls: &Controls) -> Result<(), String> {
    if key > 127 || velocity > 127 {
        return Err("Key and velocity must be seven-bit MIDI values".into());
    }
    if controls.poly_pressure > 127
        || controls.channel_pressure > 127
        || controls.cc.iter().any(|value| *value > 127)
    {
        return Err("Controller values must be seven-bit MIDI values".into());
    }
    if controls.bend > 16383 {
        return Err("Pitch wheel must be a fourteen-bit MIDI value".into());
    }
    if controls.bend_range_cents > 12827 {
        return Err("Pitch-wheel sensitivity exceeds MIDI RPN 0's 127 semitones plus 127 cents".into());
    }
    Ok(())
}

fn source_value(
    source: ModSource,
    key: u8,
    velocity: u8,
    controls: &Controls,
) -> Result<f64, String> {
    // The zero source is the SoundFont constant multiplier. Its other source
    // flags have no effect.
    if source.input == Input::Constant {
        return Ok(1.0);
    }

    let (raw, range) = match source.input {
        Input::Constant => unreachable!(),
        Input::Velocity => (f64::from(velocity), 128.0),
        Input::Key => (f64::from(key), 128.0),
        Input::PolyPressure => (f64::from(controls.poly_pressure), 128.0),
        Input::ChannelPressure => (f64::from(controls.channel_pressure), 128.0),
        Input::PitchWheel => (f64::from(controls.bend), 16384.0),
        // The SF2 default pitch modulator has amount 12700. Expressing the RPN
        // value over that same span makes the product equal the configured
        // range without applying it a second time.
        Input::PitchWheelRange => (f64::from(controls.bend_range_cents), 12700.0),
        Input::Controller(index) => (
            f64::from(*controls.cc.get(index as usize).ok_or_else(|| {
                format!("Modulation controller index {index} is outside the MIDI CC range")
            })?),
            128.0,
        ),
    };
    Ok(map_source(raw, range, source))
}

fn map_source(raw: f64, range: f64, source: ModSource) -> f64 {
    let normal = raw / range;
    let inverse = 1.0 - 1.0 / range - normal;
    let directed = if source.reversed { inverse } else { normal };
    let maximum = (range - 1.0) / range;

    if !source.bipolar {
        return match source.curve {
            Curve::Linear => directed,
            Curve::Switch => f64::from(directed >= 0.5),
            Curve::Concave => concave(directed, maximum).min(maximum),
            Curve::Convex => convex(directed, maximum).min(maximum),
        };
    }

    // SF2 controller ranges are half-open. Preserve the positive endpoint
    // specified for the maximum native value while retaining an exact center.
    let bipolar = if source.input != Input::PitchWheel && directed == maximum {
        maximum
    } else {
        -1.0 + 2.0 * directed
    };
    match source.curve {
        Curve::Linear => bipolar,
        Curve::Switch => if bipolar >= 0.0 { 1.0 } else { -1.0 },
        Curve::Concave if bipolar >= 0.0 => concave(bipolar, maximum).min(maximum),
        Curve::Concave => -concave(-bipolar, maximum),
        Curve::Convex if bipolar >= 0.0 => convex(bipolar, maximum).min(maximum),
        Curve::Convex => -convex(-bipolar, maximum),
    }
}

/// SoundFont 2.04 section 8.2.4's 96 dB concave characteristic. The native
/// maximum locates the explicit endpoint in each controller's half-open range.
fn concave(value: f64, maximum: f64) -> f64 {
    if value <= 0.0 {
        return 0.0;
    }
    if value >= maximum {
        return 1.0;
    }
    let normalized = value / maximum;
    (-(40.0 / 96.0) * (1.0 - normalized).log10()).min(1.0)
}

fn convex(value: f64, maximum: f64) -> f64 {
    if value <= 0.0 {
        return 0.0;
    }
    if value >= maximum {
        return 1.0;
    }
    1.0 - concave(maximum - value, maximum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instrument_voice::{
        Envelope, Lfo, LoopMode, Modulation, Sustain, TimeCents,
    };

    fn source(input: Input, reversed: bool, bipolar: bool, curve: Curve) -> ModSource {
        ModSource { input, reversed, bipolar, curve }
    }

    fn voice_with(modulations: Vec<Modulation>) -> Voice {
        let envelope = || Envelope {
            delay: TimeCents(-12000), attack: TimeCents(-12000), hold: TimeCents(-12000),
            decay: TimeCents(-12000), release: TimeCents(-12000),
            hold_cents_per_key: 0, decay_cents_per_key: 0,
            sustain: Sustain::AttenuationCentibels(0),
        };
        let lfo = || Lfo {
            delay: TimeCents(-12000), frequency_cents: 0, pitch_cents: 0,
            filter_cents: 0, volume_centibels: 0,
        };
        Voice {
            sample: 0, key_range: [0, 127], velocity_range: [0, 127], root_key: 60,
            fixed_key: None, fixed_velocity: None, tune_cents: 0, scale_cents_per_key: 100,
            start_offset: 0, end_offset: 0, loop_start: 0, loop_end: 0, loop_mode: LoopMode::Off,
            attenuation_centibels: 0, pan_permille: 0, filter_cents: 13500,
            filter_centibels: 0, mod_env_pitch_cents: 0, mod_env_filter_cents: 0,
            reverb_permille: 0, chorus_permille: 0,
            volume_envelope: envelope(), modulation_envelope: envelope(),
            modulation_lfo: lfo(), vibrato_lfo: lfo(), exclusive_class: 0, modulations,
        }
    }

    #[test]
    fn controls_defaults_are_midi_defaults() {
        let controls = Controls::default();
        assert_eq!((controls.cc[7], controls.cc[10], controls.cc[11]), (100, 64, 127));
        assert_eq!((controls.poly_pressure, controls.channel_pressure), (0, 0));
        assert_eq!((controls.bend, controls.bend_range_cents), (8192, 200));
        assert_eq!(controls.cc.iter().filter(|value| **value != 0).count(), 3);
    }

    #[test]
    fn pitch_wheel_uses_the_published_8192_denominator_once() {
        let modulation = Modulation {
            source: source(Input::PitchWheel, false, true, Curve::Linear),
            amount_source: source(Input::PitchWheelRange, false, false, Curve::Linear),
            destination: Destination::PitchCents,
            amount: 12700,
            absolute: false,
        };
        let voice = voice_with(vec![modulation]);
        let mut controls = Controls { bend_range_cents: 1200, ..Controls::default() };
        controls.bend = 8832;
        assert_eq!(evaluate(&voice, 60, 100, &controls).unwrap()[0].1, 93.75);
        controls.bend = 9600;
        assert_eq!(evaluate(&voice, 60, 100, &controls).unwrap()[0].1, 206.25);
        controls.bend = 0;
        assert_eq!(evaluate(&voice, 60, 100, &controls).unwrap()[0].1, -1200.0);
        controls.bend = 16383;
        assert_eq!(evaluate(&voice, 60, 100, &controls).unwrap()[0].1, 1200.0 * 8191.0 / 8192.0);
        controls.bend_range_cents = 12827;
        controls.bend = 8192 + 640;
        assert_eq!(evaluate(&voice, 60, 100, &controls).unwrap()[0].1, 12827.0 * 640.0 / 8192.0);
    }

    #[test]
    fn fixed_inputs_curves_amount_source_absolute_and_sums_are_applied() {
        let constant = source(Input::Constant, true, true, Curve::Switch);
        let key = source(Input::Key, false, false, Curve::Linear);
        let velocity = source(Input::Velocity, true, false, Curve::Linear);
        let mods = vec![
            Modulation { source: key, amount_source: constant, destination: Destination::PanPermille, amount: 128, absolute: false },
            Modulation { source: velocity, amount_source: constant, destination: Destination::PanPermille, amount: 128, absolute: false },
            Modulation { source: source(Input::PitchWheel, false, true, Curve::Linear), amount_source: constant,
                destination: Destination::PitchCents, amount: 100, absolute: true },
        ];
        let mut voice = voice_with(mods);
        voice.fixed_key = Some(12);
        voice.fixed_velocity = Some(100);
        let controls = Controls { bend: 4096, ..Controls::default() };
        let values = evaluate(&voice, 90, 20, &controls).unwrap();
        assert_eq!(values, vec![(Destination::PanPermille, 39.0), (Destination::PitchCents, 50.0)]);

        let maximum = 127.0 / 128.0;
        assert_eq!(map_source(0.0, 128.0, source(Input::Key, false, false, Curve::Concave)), 0.0);
        assert_eq!(map_source(127.0, 128.0, source(Input::Key, false, false, Curve::Concave)), maximum);
        assert_eq!(map_source(64.0, 128.0, source(Input::Key, false, true, Curve::Linear)), 0.0);
        assert_eq!(map_source(63.0, 128.0, source(Input::Key, false, true, Curve::Switch)), -1.0);
        assert_eq!(map_source(64.0, 128.0, source(Input::Key, false, true, Curve::Switch)), 1.0);
        let x = 32.0 / 128.0;
        let concave_x = map_source(32.0, 128.0, source(Input::Key, false, false, Curve::Concave));
        let convex_mirror = map_source(95.0, 128.0, source(Input::Key, false, false, Curve::Convex));
        assert!(concave_x < x);
        assert!((concave_x + convex_mirror - 1.0).abs() < 1e-12);
        assert_eq!(map_source(0.0, 128.0, source(Input::Key, false, true, Curve::Concave)), -1.0);
    }

    #[test]
    fn pressure_cc_and_pitch_range_sources_use_their_native_units() {
        let constant = source(Input::Constant, false, false, Curve::Linear);
        let voice = voice_with(vec![
            Modulation { source: source(Input::PolyPressure, false, false, Curve::Linear), amount_source: constant,
                destination: Destination::FilterCents, amount: 128, absolute: false },
            Modulation { source: source(Input::ChannelPressure, false, false, Curve::Linear), amount_source: constant,
                destination: Destination::VibLfoPitchCents, amount: 128, absolute: false },
            Modulation { source: source(Input::Controller(1), false, false, Curve::Linear), amount_source: constant,
                destination: Destination::ModLfoPitchCents, amount: 128, absolute: false },
            Modulation { source: source(Input::PitchWheelRange, false, false, Curve::Linear), amount_source: constant,
                destination: Destination::PitchCents, amount: 12700, absolute: false },
        ]);
        let mut controls = Controls {
            poly_pressure: 32,
            channel_pressure: 64,
            bend_range_cents: 300,
            ..Controls::default()
        };
        controls.cc[1] = 96;
        assert_eq!(evaluate(&voice, 60, 100, &controls).unwrap(), vec![
            (Destination::FilterCents, 32.0),
            (Destination::VibLfoPitchCents, 64.0),
            (Destination::ModLfoPitchCents, 96.0),
            (Destination::PitchCents, 300.0),
        ]);
    }

    #[test]
    fn validation_and_profile_limit_fail_before_evaluation() {
        let modulation = Modulation {
            source: source(Input::Controller(1), false, false, Curve::Linear),
            amount_source: source(Input::Constant, false, false, Curve::Linear),
            destination: Destination::PitchCents,
            amount: 1,
            absolute: false,
        };
        let voice = voice_with(vec![modulation.clone(); MAX_MODULATIONS_PER_VOICE + 1]);
        assert!(evaluate(&voice, 60, 100, &Controls::default()).unwrap_err().contains("32"));
        let voice = voice_with(vec![modulation]);
        let mut controls = Controls::default();
        controls.cc[1] = 128;
        assert!(evaluate(&voice, 60, 100, &controls).is_err());
        controls.cc[1] = 0;
        controls.bend = 16384;
        assert!(evaluate(&voice, 60, 100, &controls).is_err());
        controls.bend = 8192;
        controls.bend_range_cents = 12828;
        assert!(evaluate(&voice, 60, 100, &controls).is_err());
        let mut invalid_controller = voice.clone();
        invalid_controller.modulations[0].source.input = Input::Controller(255);
        assert!(evaluate(&invalid_controller, 60, 100, &Controls::default()).is_err());
        assert!(evaluate(&voice, 128, 100, &Controls::default()).is_err());
    }
}
