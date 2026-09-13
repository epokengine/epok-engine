//! Explicit loop-seam analysis and repair for PSX instrument cooking.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

pub const MAX_CROSSFADE_FRAMES: u16 = 256;
const MAX_PCM_FRAMES: usize = 8_388_636;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoopQuality {
    /// Absolute decoded sample step at the loop boundary, normalized by 32768.
    pub step: f64,
    /// RMS difference between the loop's final window and the PCM window that
    /// naturally precedes its start, normalized by 32768.
    pub boundary_window_rms: Option<f64>,
    /// Largest adjacent decoded-sample slope across the final window, boundary,
    /// and initial window, normalized by 32768.
    pub maximum_slope: f64,
    /// Actual boundary-slope window after applying the loop-length bound.
    pub window_frames: u16,
    /// Frames compared to PCM preceding the start. This can be shorter than
    /// `window_frames`, and is zero when a loop starts at frame zero.
    pub comparison_frames: u16,
}

/// Crossfades the final loop window toward the PCM immediately preceding the
/// loop start. Endpoints and period stay fixed, and samples after `loop_region`
/// (including an UntilRelease tail) are never changed. Returns the actual frame
/// count after bounding it by the available prefix and one quarter of the loop.
pub fn crossfade(
    samples: &mut [f32],
    loop_region: [usize; 2],
    frames: u16,
    cancelled: &AtomicBool,
) -> Result<u16, String> {
    let [start, end] = validate_region(samples.len(), loop_region)?;
    validate_frame_count(frames)?;
    check_cancelled(cancelled)?;
    if frames == 0 {
        return Ok(0);
    }
    if start == 0 {
        return Err("Loop crossfade requires PCM before loop start 0; padding is not inferred".into());
    }
    let count = usize::from(frames).min(start).min((end - start) / 4);
    if count == 0 {
        return Ok(0);
    }

    // Calculate into bounded scratch storage first. Cancellation and invalid
    // inputs therefore leave the caller's PCM unchanged.
    let mut replacement = Vec::with_capacity(count);
    for index in 0..count {
        check_cancelled(cancelled)?;
        let original = f64::from(samples[end - count + index]);
        let predecessor = f64::from(samples[start - count + index]);
        if !original.is_finite() || !predecessor.is_finite() {
            return Err("Loop crossfade input contains a non-finite sample".into());
        }
        let weight = if count == 1 {
            1.0
        } else {
            let position = index as f64 / (count - 1) as f64;
            0.5 - 0.5 * (std::f64::consts::PI * position).cos()
        };
        let value = original * (1.0 - weight) + predecessor * weight;
        let value = value as f32;
        if !value.is_finite() {
            return Err("Loop crossfade produced a non-finite sample".into());
        }
        replacement.push(value);
    }
    check_cancelled(cancelled)?;
    samples[end - count..end].copy_from_slice(&replacement);
    Ok(count as u16)
}

/// Measures a decoded ADPCM loop. Coordinates are decoded-frame coordinates;
/// callers account for the encoder's initial silent block before calling.
pub fn analyze(
    decoded: &[i16],
    loop_region: [usize; 2],
    window_frames: u16,
    cancelled: &AtomicBool,
) -> Result<LoopQuality, String> {
    let [start, end] = validate_region(decoded.len(), loop_region)?;
    validate_frame_count(window_frames)?;
    check_cancelled(cancelled)?;
    let step = normalized_difference(decoded[start], decoded[end - 1]);
    let count = usize::from(window_frames).min((end - start) / 4);
    if count == 0 {
        return Ok(LoopQuality {
            step,
            boundary_window_rms: None,
            maximum_slope: step,
            window_frames: 0,
            comparison_frames: 0,
        });
    }

    let comparison_count = count.min(start);
    let boundary_window_rms = if comparison_count == 0 {
        None
    } else {
        let mut squared_difference = 0_u64;
        for index in 0..comparison_count {
            check_cancelled(cancelled)?;
            let difference = i64::from(decoded[end - comparison_count + index])
                - i64::from(decoded[start - comparison_count + index]);
            squared_difference = squared_difference
                .checked_add((difference * difference) as u64)
                .ok_or("Loop quality RMS accumulator overflow")?;
        }
        Some(((squared_difference as f64 / comparison_count as f64).sqrt()) / 32768.0)
    };

    let mut maximum_slope = 0.0_f64;
    let mut previous = decoded[end - count];
    for &sample in &decoded[end - count + 1..end] {
        check_cancelled(cancelled)?;
        maximum_slope = maximum_slope.max(normalized_difference(sample, previous));
        previous = sample;
    }
    maximum_slope = maximum_slope.max(step);
    previous = decoded[start];
    for &sample in &decoded[start + 1..start + count] {
        check_cancelled(cancelled)?;
        maximum_slope = maximum_slope.max(normalized_difference(sample, previous));
        previous = sample;
    }
    check_cancelled(cancelled)?;
    Ok(LoopQuality {
        step,
        boundary_window_rms,
        maximum_slope,
        window_frames: count as u16,
        comparison_frames: comparison_count as u16,
    })
}

fn validate_region(length: usize, loop_region: [usize; 2]) -> Result<[usize; 2], String> {
    if length > MAX_PCM_FRAMES {
        return Err(format!("Loop quality input exceeds {MAX_PCM_FRAMES} frames"));
    }
    let [start, end] = loop_region;
    if start >= end || end > length {
        return Err("Loop quality region must be a non-empty range inside the PCM input".into());
    }
    Ok(loop_region)
}

fn validate_frame_count(frames: u16) -> Result<(), String> {
    if frames > MAX_CROSSFADE_FRAMES {
        return Err(format!(
            "Loop quality window exceeds the {MAX_CROSSFADE_FRAMES}-frame limit"
        ));
    }
    Ok(())
}

fn normalized_difference(a: i16, b: i16) -> f64 {
    (i32::from(a) - i32::from(b)).unsigned_abs() as f64 / 32768.0
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Relaxed) {
        Err("PSX loop quality processing cancelled".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active() -> AtomicBool {
        AtomicBool::new(false)
    }

    fn decoded(samples: &[f32]) -> Vec<i16> {
        samples
            .iter()
            .map(|sample| (f64::from(*sample) * 32767.0).round() as i16)
            .collect()
    }

    #[test]
    fn crossfade_improves_a_boundary_and_reports_actual_window() {
        let mut samples = vec![0.0_f32; 64];
        for (index, sample) in samples[8..17].iter_mut().enumerate() {
            *sample = -0.4 + index as f32 * 0.05;
        }
        for (index, sample) in samples[16..48].iter_mut().enumerate() {
            *sample += (index as f32 * 0.19).sin() * 0.2;
        }
        samples[40..48].copy_from_slice(&[0.7, 0.75, 0.8, 0.85, 0.9, 0.85, 0.8, 0.75]);
        let prior = analyze(&decoded(&samples), [16, 48], 8, &active()).unwrap();
        let applied = crossfade(&mut samples, [16, 48], 8, &active()).unwrap();
        let final_quality = analyze(&decoded(&samples), [16, 48], 8, &active()).unwrap();
        assert_eq!(applied, 8);
        assert_eq!(final_quality.window_frames, 8);
        assert!(final_quality.step < prior.step, "{prior:?} -> {final_quality:?}");
        assert!(final_quality.boundary_window_rms.unwrap() < prior.boundary_window_rms.unwrap(),
            "{prior:?} -> {final_quality:?}");
    }

    #[test]
    fn crossfade_changes_only_the_final_loop_window() {
        let mut samples: Vec<f32> = (0..80).map(|index| index as f32 / 100.0).collect();
        let original = samples.clone();
        let applied = crossfade(&mut samples, [12, 60], 32, &active()).unwrap();
        assert_eq!(applied, 12, "start bounds the requested window");
        assert_eq!(&samples[..48], &original[..48]);
        assert_eq!(&samples[60..], &original[60..], "UntilRelease tail must survive");
        assert_eq!(samples[48], original[48], "half-cosine begins at weight zero");
        assert_eq!(samples[59], original[11], "half-cosine ends at weight one");
    }

    #[test]
    fn short_loops_bound_the_crossfade_to_one_quarter() {
        let mut samples = vec![0.0; 32];
        assert_eq!(crossfade(&mut samples, [3, 11], 256, &active()).unwrap(), 2);
        assert_eq!(crossfade(&mut samples, [3, 6], 256, &active()).unwrap(), 0);
    }

    #[test]
    fn invalid_inputs_and_cancellation_do_not_mutate_pcm() {
        let original = vec![0.25_f32; 32];
        for (region, frames, cancelled) in [
            ([8, 8], 4, false),
            ([8, 33], 4, false),
            ([0, 16], 4, false),
            ([8, 24], 257, false),
            ([8, 24], 4, true),
        ] {
            let mut samples = original.clone();
            let result = crossfade(&mut samples, region, frames, &AtomicBool::new(cancelled));
            assert!(result.is_err());
            assert_eq!(samples, original);
        }
        let mut non_finite = original.clone();
        non_finite[23] = f32::NAN;
        let before = non_finite.clone();
        assert!(crossfade(&mut non_finite, [8, 24], 4, &active()).is_err());
        assert!(non_finite.iter().zip(before).all(|(a, b)| a.to_bits() == b.to_bits()));
        assert!(analyze(&[0; 32], [8, 24], 257, &active()).is_err());
        assert!(analyze(&[0; 32], [8, 24], 4, &AtomicBool::new(true)).is_err());
        let start_zero = analyze(&[0, 100, 200, 300, 400, 500, 600, 700], [0, 8], 4, &active()).unwrap();
        assert_eq!(start_zero.boundary_window_rms, None);
        assert_eq!((start_zero.window_frames, start_zero.comparison_frames), (2, 0));
        assert!(start_zero.step > 0.0 && start_zero.maximum_slope >= start_zero.step);
    }
}
