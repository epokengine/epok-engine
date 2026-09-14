//! Small, target-neutral DSP operations used while cooking instrument samples.

use std::sync::atomic::{AtomicBool, Ordering};

const MAX_SAMPLE_RATE: u32 = 768_000;
const SINC_RADIUS: i64 = 20;
pub const MAX_SAMPLE_FRAMES: usize = 8_388_608;

/// Static two-pole low-pass parameters selected by the cooking policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilterSpec {
    pub cutoff_hz: f64,
    pub resonance_centibels: f64,
}

/// Downsamples mono PCM with a Blackman-windowed sinc low-pass kernel.
/// Upsampling is outside the bounded P3 profile.
pub fn resample(
    samples: &[f32],
    source_rate: u32,
    target_rate: u32,
    cancelled: &AtomicBool,
) -> Result<Vec<f32>, String> {
    validate_rate(source_rate)?;
    validate_rate(target_rate)?;
    if target_rate > source_rate {
        return Err("P3 resampling only accepts target rates at or below the source rate".into());
    }
    validate_samples(samples, cancelled)?;
    check_cancelled(cancelled)?;
    if samples.is_empty() {
        return Ok(Vec::new());
    }
    if target_rate == source_rate {
        let output = samples.to_vec();
        check_cancelled(cancelled)?;
        return Ok(output);
    }

    let numerator = (samples.len() as u128)
        .checked_mul(u128::from(target_rate))
        .ok_or("Resampled length overflow")?;
    let output_len_u128 = numerator
        .checked_add(u128::from(source_rate - 1))
        .ok_or("Resampled length overflow")?
        / u128::from(source_rate);
    let output_len = usize::try_from(output_len_u128)
        .map_err(|_| "Resampled output cannot be represented on this platform")?;
    if output_len > samples.len() {
        return Err("Resampled output exceeds its bounded source length".into());
    }

    let rate_ratio = f64::from(target_rate) / f64::from(source_rate);
    let mut output = Vec::with_capacity(output_len);
    for output_index in 0..output_len {
        if output_index & 255 == 0 {
            check_cancelled(cancelled)?;
        }
        let source_position = output_index as f64 / rate_ratio;
        let center = source_position.floor() as i64;
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        for source_index in (center - SINC_RADIUS + 1)..=(center + SINC_RADIUS) {
            if source_index < 0 || source_index >= samples.len() as i64 {
                continue;
            }
            let distance = source_index as f64 - source_position;
            let weight = sinc_kernel(distance, rate_ratio);
            weighted_sum += f64::from(samples[source_index as usize]) * weight;
            weight_sum += weight;
        }
        if !weighted_sum.is_finite() || !weight_sum.is_finite() || weight_sum.abs() < 1e-12 {
            return Err("Resampling produced an invalid filter state".into());
        }
        let value = weighted_sum / weight_sum;
        if !value.is_finite() || value < f64::from(f32::MIN) || value > f64::from(f32::MAX) {
            return Err("Resampling produced a non-finite sample".into());
        }
        output.push(value as f32);
    }
    check_cancelled(cancelled)?;
    Ok(output)
}

/// Applies a static two-pole low-pass to mono PCM. Zero resonance is adapted
/// to a Butterworth Q; positive SoundFont centibels raise Q by their amplitude
/// ratio. Values that would exceed the bounded Q profile are reported so the
/// caller can choose and record an explicit adaptation.
pub fn low_pass(
    samples: &[f32],
    sample_rate: u32,
    spec: FilterSpec,
    cancelled: &AtomicBool,
) -> Result<Vec<f32>, String> {
    validate_rate(sample_rate)?;
    validate_samples(samples, cancelled)?;
    if !spec.cutoff_hz.is_finite()
        || spec.cutoff_hz <= 0.0
        || spec.cutoff_hz >= f64::from(sample_rate) * 0.5
    {
        return Err("Low-pass cutoff must be finite and between zero and Nyquist".into());
    }
    if !spec.resonance_centibels.is_finite() || !(0.0..=960.0).contains(&spec.resonance_centibels) {
        return Err("Low-pass resonance must be finite and within 0..=960 centibels".into());
    }
    check_cancelled(cancelled)?;

    let omega = 2.0 * std::f64::consts::PI * spec.cutoff_hz / f64::from(sample_rate);
    let (sin_omega, cos_omega) = omega.sin_cos();
    let q = std::f64::consts::FRAC_1_SQRT_2 * 10.0_f64.powf(spec.resonance_centibels / 200.0);
    if !q.is_finite() || q > 64.0 {
        return Err(format!(
            "Low-pass resonance requires Q {q:.3}, above the bounded P3 maximum of 64; select an explicit cooking adaptation"
        ));
    }
    let alpha = sin_omega / (2.0 * q);
    let a0 = 1.0 + alpha;
    // SF2 generator 9 lowers DC gain by half the specified resonance in dB.
    let dc_gain = 10.0_f64.powf(-spec.resonance_centibels / 400.0);
    let b0 = dc_gain * ((1.0 - cos_omega) * 0.5) / a0;
    let b1 = dc_gain * (1.0 - cos_omega) / a0;
    let b2 = b0;
    let a1 = (-2.0 * cos_omega) / a0;
    let a2 = (1.0 - alpha) / a0;
    if [b0, b1, b2, a1, a2].iter().any(|value| !value.is_finite()) {
        return Err("Low-pass coefficients are non-finite".into());
    }

    let mut output = Vec::with_capacity(samples.len());
    let mut state1 = 0.0;
    let mut state2 = 0.0;
    for (index, sample) in samples.iter().enumerate() {
        if index & 255 == 0 {
            check_cancelled(cancelled)?;
        }
        let input = f64::from(*sample);
        let value = b0 * input + state1;
        state1 = b1 * input - a1 * value + state2;
        state2 = b2 * input - a2 * value;
        if !value.is_finite() || !state1.is_finite() || !state2.is_finite() {
            return Err("Low-pass processing produced a non-finite state".into());
        }
        if value < f64::from(f32::MIN) || value > f64::from(f32::MAX) {
            return Err("Low-pass processing exceeded the finite f32 range".into());
        }
        let output_sample = value as f32;
        if !output_sample.is_finite() {
            return Err("Low-pass f32 output is non-finite".into());
        }
        output.push(output_sample);
    }
    check_cancelled(cancelled)?;
    Ok(output)
}

fn validate_rate(rate: u32) -> Result<(), String> {
    if rate == 0 || rate > MAX_SAMPLE_RATE {
        return Err(format!(
            "Sample rate must be within 1..={MAX_SAMPLE_RATE} Hz"
        ));
    }
    Ok(())
}

fn validate_samples(samples: &[f32], cancelled: &AtomicBool) -> Result<(), String> {
    if samples.len() > MAX_SAMPLE_FRAMES {
        return Err(format!(
            "Sample has {} frames; the cooking limit is {MAX_SAMPLE_FRAMES}",
            samples.len()
        ));
    }
    for (index, sample) in samples.iter().enumerate() {
        if index & 4095 == 0 {
            check_cancelled(cancelled)?;
        }
        if !sample.is_finite() {
            return Err(format!("Input sample {index} is non-finite"));
        }
    }
    Ok(())
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Relaxed) {
        Err("Instrument sample processing was cancelled".into())
    } else {
        Ok(())
    }
}

fn sinc_kernel(distance: f64, cutoff: f64) -> f64 {
    if distance.abs() >= SINC_RADIUS as f64 {
        return 0.0;
    }
    let phase = std::f64::consts::PI * cutoff * distance;
    let sinc = if phase.abs() < 1e-12 {
        cutoff
    } else {
        cutoff * phase.sin() / phase
    };
    let window_phase = std::f64::consts::PI * distance / SINC_RADIUS as f64;
    let blackman = 0.42 + 0.5 * window_phase.cos() + 0.08 * (2.0 * window_phase).cos();
    sinc * blackman
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active() -> AtomicBool {
        AtomicBool::new(false)
    }

    #[test]
    fn sinc_downsampling_has_bounded_length_and_preserves_dc() {
        let input = vec![0.25; 4801];
        let output = resample(&input, 48_000, 32_000, &active()).unwrap();
        assert_eq!(output.len(), 3201);
        assert!(output.iter().all(|sample| (*sample - 0.25).abs() < 1e-5));
    }

    #[test]
    fn sinc_filter_rejects_source_nyquist_energy() {
        let input: Vec<f32> = (0..4096)
            .map(|index| if index & 1 == 0 { 1.0 } else { -1.0 })
            .collect();
        let output = resample(&input, 48_000, 24_000, &active()).unwrap();
        let interior = &output[32..output.len() - 32];
        let peak = interior
            .iter()
            .fold(0.0_f32, |peak, value| peak.max(value.abs()));
        assert!(peak < 1e-4, "peak={peak}");
    }

    #[test]
    fn low_pass_is_finite_and_attenuates_high_frequency() {
        let input: Vec<f32> = (0..8192)
            .map(|index| if index & 1 == 0 { 1.0 } else { -1.0 })
            .collect();
        let output = low_pass(
            &input,
            48_000,
            FilterSpec {
                cutoff_hz: 4_000.0,
                resonance_centibels: 0.0,
            },
            &active(),
        )
        .unwrap();
        assert_eq!(output.len(), input.len());
        assert!(output.iter().all(|sample| sample.is_finite()));
        let tail_peak = output[1024..]
            .iter()
            .fold(0.0_f32, |peak, value| peak.max(value.abs()));
        assert!(tail_peak < 0.08, "peak={tail_peak}");
    }

    #[test]
    fn soundfont_resonance_reduces_dc_gain_by_half_its_decibels() {
        let input = vec![0.25; 8192];
        let output = low_pass(
            &input,
            44100,
            FilterSpec {
                cutoff_hz: 4000.,
                resonance_centibels: 200.,
            },
            &active(),
        )
        .unwrap();
        let expected = 0.25 * 10_f32.powf(-10. / 20.);
        assert!((output[8191] - expected).abs() < 1e-6);
    }

    #[test]
    fn invalid_parameters_samples_and_cancellation_are_errors() {
        let cancelled = AtomicBool::new(true);
        assert!(
            resample(&[0.0], 48_000, 24_000, &cancelled)
                .unwrap_err()
                .contains("cancelled")
        );
        assert!(
            low_pass(
                &[0.0],
                48_000,
                FilterSpec {
                    cutoff_hz: 1_000.0,
                    resonance_centibels: 0.0
                },
                &cancelled,
            )
            .unwrap_err()
            .contains("cancelled")
        );
        assert!(resample(&[0.0], 24_000, 48_000, &active()).is_err());
        assert!(resample(&[f32::NAN], 48_000, 24_000, &active()).is_err());
        assert!(
            low_pass(
                &[0.0],
                48_000,
                FilterSpec {
                    cutoff_hz: 24_000.0,
                    resonance_centibels: 0.0
                },
                &active(),
            )
            .is_err()
        );
        assert!(
            low_pass(
                &[0.0],
                48_000,
                FilterSpec {
                    cutoff_hz: 1_000.0,
                    resonance_centibels: 960.0
                },
                &active(),
            )
            .unwrap_err()
            .contains("explicit cooking adaptation")
        );
    }
}
