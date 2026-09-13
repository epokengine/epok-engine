//! Deterministic PSX ADPCM encoding for SoundFont-derived banks.
//!
//! `Fast` intentionally mirrors the legacy AudioClip encoder. `Thorough` searches
//! every legal filter/shift pair, then keeps the result only when whole-sample
//! decoded SSE is strictly lower than Fast's. The fallback makes the quality
//! promise independent of local, block-by-block choices.
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

pub const MAX_PCM_FRAMES: usize = 8_388_608;
const FILTERS: [[i32; 2]; 5] = [[0, 0], [60, 0], [115, -52], [98, -55], [122, -60]];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effort {
    #[default]
    Fast,
    Thorough,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Encoded {
    pub bytes: Vec<u8>,
    pub squared_error: u64,
    /// Number of PCM blocks examined by the thorough search.
    pub searched_blocks: u32,
    /// True only when Thorough's final decoded SSE beat Fast's SSE.
    pub improved: bool,
}

pub fn encode(
    pcm: &[i16],
    loop_start_block: Option<usize>,
    effort: Effort,
    cancelled: &AtomicBool,
) -> Result<Encoded, String> {
    let loop_blocks = loop_start_block.map(|start| [start, pcm.len().div_ceil(28)]);
    encode_region(pcm, loop_blocks, effort, cancelled)
}

/// Encodes an optional repeat region `[start_block, end_block)`. Blocks after a
/// repeat end are an UntilRelease tail, reached by changing the SPU repeat
/// address after note release.
pub fn encode_region(
    pcm: &[i16],
    loop_blocks: Option<[usize; 2]>,
    effort: Effort,
    cancelled: &AtomicBool,
) -> Result<Encoded, String> {
    validate(pcm, loop_blocks)?;
    check_cancelled(cancelled)?;

    let fast = encode_fast(pcm, loop_blocks, cancelled)?;
    let fast_error = squared_error(&fast, pcm, cancelled)?;
    if effort == Effort::Fast {
        return Ok(Encoded {
            bytes: fast,
            squared_error: fast_error,
            searched_blocks: 0,
            improved: false,
        });
    }

    let thorough = encode_thorough(pcm, loop_blocks, cancelled)?;
    let thorough_error = squared_error(&thorough, pcm, cancelled)?;
    let improved = thorough_error < fast_error;
    Ok(Encoded {
        bytes: if improved { thorough } else { fast },
        squared_error: if improved { thorough_error } else { fast_error },
        searched_blocks: pcm.len().div_ceil(28) as u32,
        improved,
    })
}

fn validate(pcm: &[i16], loop_blocks: Option<[usize; 2]>) -> Result<(), String> {
    if pcm.is_empty() {
        return Err("PSX ADPCM encoder requires at least one PCM frame".into());
    }
    validate_pcm_len(pcm.len())?;
    let blocks = pcm.len().div_ceil(28);
    if let Some([start, end]) = loop_blocks {
        if start >= end || end == 0 || end > blocks {
            return Err("PSX ADPCM repeat region lies outside PCM blocks".into());
        }
    }
    Ok(())
}

fn validate_pcm_len(len: usize) -> Result<(), String> {
    if len > MAX_PCM_FRAMES {
        return Err(format!(
            "PSX ADPCM encoder accepts at most {MAX_PCM_FRAMES} PCM frames"
        ));
    }
    Ok(())
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Relaxed) {
        Err("PSX ADPCM encoding cancelled".into())
    } else {
        Ok(())
    }
}

fn predict(history: [i32; 2], filter: [i32; 2]) -> i32 {
    (history[0] * filter[0] + history[1] * filter[1] + 32) >> 6
}

/// Exact copy of the legacy candidate order and f64 rounding. Keep this separate
/// from Thorough so AudioClip golden bytes remain a meaningful compatibility test.
fn encode_fast(
    pcm: &[i16],
    loop_blocks: Option<[usize; 2]>,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, String> {
    let mut out = vec![0_u8; 16];
    let mut history = [0_i32; 2];
    let blocks = pcm.len().div_ceil(28);
    for (index, chunk) in pcm.chunks(28).enumerate() {
        check_cancelled(cancelled)?;
        let mut block = [0_i16; 28];
        block[..chunk.len()].copy_from_slice(chunk);
        let mut best = (u64::MAX, [0_u8; 16], history);
        let filter_count = if loop_blocks.is_some_and(|region| region[0] == index) {
            1
        } else {
            FILTERS.len()
        };
        for (filter_index, filter) in FILTERS.iter().enumerate().take(filter_count) {
            let mut h = history;
            let mut peak = 0;
            for sample in block {
                peak = peak.max((i32::from(sample) - predict(h, *filter)).abs());
                h = [sample as i32, h[0]];
            }
            let mut shift: u32 = 0;
            while shift < 12 && peak <= (7 * (4096 >> (shift + 1))) {
                shift += 1;
            }
            for shift in [shift, shift.saturating_sub(1)] {
                let step = 4096 >> shift;
                let mut h = history;
                let mut error = 0_u64;
                let mut encoded = [0_u8; 16];
                encoded[0] = ((filter_index as u8) << 4) | shift as u8;
                for (i, sample) in block.iter().enumerate() {
                    let p = predict(h, *filter);
                    let q =
                        (((*sample as i32 - p) as f64 / step as f64).round() as i32).clamp(-8, 7);
                    let decoded = (p + q * step).clamp(-32768, 32767);
                    error = add_square(error, i64::from(*sample) - i64::from(decoded))?;
                    h = [decoded, h[0]];
                    encoded[2 + i / 2] |= ((q as u8) & 15) << ((i % 2) * 4);
                }
                if error < best.0 {
                    best = (error, encoded, h);
                }
            }
        }
        history = best.2;
        best.1[1] = if loop_blocks.is_some_and(|region| region[1] == index + 1) {
            3
        } else if index + 1 == blocks {
            1
        } else {
            0
        };
        if loop_blocks.is_some_and(|region| region[0] == index) {
            best.1[1] |= 4;
        }
        out.extend_from_slice(&best.1);
    }
    let mut silence = [0_u8; 16];
    silence[1] = 7;
    out.extend_from_slice(&silence);
    Ok(out)
}

fn encode_thorough(
    pcm: &[i16],
    loop_blocks: Option<[usize; 2]>,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, String> {
    let mut out = vec![0_u8; 16];
    let mut history = [0_i32; 2];
    let blocks = pcm.len().div_ceil(28);
    for (index, chunk) in pcm.chunks(28).enumerate() {
        check_cancelled(cancelled)?;
        let mut block = [0_i16; 28];
        block[..chunk.len()].copy_from_slice(chunk);
        let filter_count = if loop_blocks.is_some_and(|region| region[0] == index) {
            1
        } else {
            FILTERS.len()
        };
        let mut best = (u64::MAX, [0_u8; 16], history);
        for (filter_index, filter) in FILTERS.iter().enumerate().take(filter_count) {
            for shift in 0..=12 {
                check_cancelled(cancelled)?;
                let candidate =
                    encode_integer_candidate(block, history, *filter, filter_index, shift)?;
                // Strictly lower retains the ascending filter/shift order as a stable tie-break.
                if candidate.0 < best.0 {
                    best = candidate;
                }
            }
        }
        history = best.2;
        best.1[1] = if loop_blocks.is_some_and(|region| region[1] == index + 1) {
            3
        } else if index + 1 == blocks {
            1
        } else {
            0
        };
        if loop_blocks.is_some_and(|region| region[0] == index) {
            best.1[1] |= 4;
        }
        out.extend_from_slice(&best.1);
    }
    let mut silence = [0_u8; 16];
    silence[1] = 7;
    out.extend_from_slice(&silence);
    Ok(out)
}

fn encode_integer_candidate(
    block: [i16; 28],
    history: [i32; 2],
    filter: [i32; 2],
    filter_index: usize,
    shift: u32,
) -> Result<(u64, [u8; 16], [i32; 2]), String> {
    let step = 4096 >> shift;
    let mut h = history;
    let mut error = 0_u64;
    let mut encoded = [0_u8; 16];
    encoded[0] = ((filter_index as u8) << 4) | shift as u8;
    for (i, sample) in block.iter().enumerate() {
        let predicted = predict(h, filter);
        let q = rounded_div(i32::from(*sample) - predicted, step).clamp(-8, 7);
        let decoded = (predicted + q * step).clamp(-32768, 32767);
        error = add_square(error, i64::from(*sample) - i64::from(decoded))?;
        h = [decoded, h[0]];
        encoded[2 + i / 2] |= ((q as u8) & 15) << ((i % 2) * 4);
    }
    Ok((error, encoded, h))
}

/// Division rounded to nearest with half values away from zero, matching
/// `f64::round()` for the small integer operands used by PSX ADPCM.
fn rounded_div(value: i32, divisor: i32) -> i32 {
    debug_assert!(divisor > 0);
    if value >= 0 {
        (value + divisor / 2) / divisor
    } else {
        -((-value + divisor / 2) / divisor)
    }
}

fn add_square(total: u64, difference: i64) -> Result<u64, String> {
    let square = difference
        .checked_mul(difference)
        .and_then(|value| u64::try_from(value).ok())
        .ok_or("PSX ADPCM squared error overflow")?;
    total
        .checked_add(square)
        .ok_or("PSX ADPCM squared error overflow".into())
}

/// Decodes contiguously to compare all source PCM after the mandatory initial
/// silent block. A repeat flag can precede an UntilRelease tail, so it cannot
/// terminate this quality calculation.
fn squared_error(bytes: &[u8], pcm: &[i16], cancelled: &AtomicBool) -> Result<u64, String> {
    if bytes.len() < 48 || !bytes.len().is_multiple_of(16) {
        return Err("Invalid PSX ADPCM block size".into());
    }
    let mut history = [0_i32; 2];
    let mut frame = 0_usize;
    let mut error = 0_u64;
    let end = 28_usize
        .checked_add(pcm.len())
        .ok_or("PSX ADPCM frame count overflow")?;
    // Exclude the terminal silence block; decode every useful block regardless
    // of its repeat/end flags so that the tail contributes to SSE.
    for block in bytes[..bytes.len() - 16].chunks_exact(16) {
        check_cancelled(cancelled)?;
        let filter = *FILTERS
            .get((block[0] >> 4) as usize)
            .ok_or("Invalid PSX ADPCM filter")?;
        let shift = block[0] & 15;
        if shift > 12 {
            return Err("Invalid PSX ADPCM shift".into());
        }
        for i in 0..28 {
            if frame.is_multiple_of(4096) {
                check_cancelled(cancelled)?;
            }
            let nibble = (block[2 + i / 2] >> ((i % 2) * 4)) & 15;
            let signed = ((nibble as i8) << 4) >> 4;
            let decoded =
                (((signed as i32 * 4096) >> shift) + predict(history, filter)).clamp(-32768, 32767);
            if (28..end).contains(&frame) {
                error = add_square(error, i64::from(pcm[frame - 28]) - i64::from(decoded))?;
            }
            history = [decoded, history[0]];
            frame += 1;
        }
    }
    if frame < end {
        return Err("PSX ADPCM stream ended before all PCM frames".into());
    }
    Ok(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active() -> AtomicBool {
        AtomicBool::new(false)
    }

    #[test]
    fn fast_is_the_legacy_golden_path() {
        let pcm = (0..113)
            .map(|i| ((i * 977 % 20_000) as i16) - 10_000)
            .collect::<Vec<_>>();
        let encoded = encode(&pcm, Some(1), Effort::Fast, &active()).unwrap();
        assert_eq!(
            encoded.bytes,
            crate::audio_import::encode_blocks(&pcm, Some(1))
        );
        assert!(!encoded.improved);
        assert_eq!(encoded.searched_blocks, 0);
    }

    #[test]
    fn thorough_is_deterministic_and_can_improve_a_real_pcm_fixture() {
        // Quiet changing PCM makes the legacy peak heuristic omit useful shifts.
        let pcm = (0..280)
            .map(|i| (((i * 37 % 223) as i16) - 111) * 23)
            .collect::<Vec<_>>();
        let first = encode(&pcm, None, Effort::Thorough, &active()).unwrap();
        let second = encode(&pcm, None, Effort::Thorough, &active()).unwrap();
        let fast = encode(&pcm, None, Effort::Fast, &active()).unwrap();
        assert_eq!(first, second);
        assert!(first.squared_error <= fast.squared_error);
        assert!(first.improved, "fixture must exercise the thorough search");
        assert!(first.squared_error < fast.squared_error);
        assert_eq!(first.searched_blocks, 10);
    }

    #[test]
    fn loop_entry_is_filter_zero_and_decodes_with_initial_offset() {
        let pcm = (0..84).map(|i| (i as i16 - 42) * 501).collect::<Vec<_>>();
        let encoded = encode(&pcm, Some(1), Effort::Thorough, &active()).unwrap();
        assert_eq!(encoded.bytes[16 + 16] >> 4, 0);
        assert_eq!(encoded.bytes[16 + 16 + 1] & 4, 4);
        let loop_block = &encoded.bytes[32..48];
        assert_eq!(
            decode_block(loop_block, [0, 0]).unwrap(),
            decode_block(loop_block, [12_345, -9_876]).unwrap()
        );
        assert_eq!(
            squared_error(&encoded.bytes, &pcm, &active()).unwrap(),
            encoded.squared_error
        );
    }

    #[test]
    fn until_release_tail_remains_decodable_and_loop_state_is_stable() {
        let pcm = (0..112)
            .map(|i| (((i * 719) % 14_000) as i16) - 7_000)
            .collect::<Vec<_>>();
        let encoded = encode_region(&pcm, Some([1, 3]), Effort::Thorough, &active()).unwrap();
        // Initial silence, four data blocks, then terminal silence.
        assert_eq!(encoded.bytes[32] >> 4, 0);
        assert_eq!(encoded.bytes[33] & 4, 4);
        assert_eq!(encoded.bytes[49] & 3, 3);
        assert_eq!(encoded.bytes[65] & 1, 1);
        let decoded = decode_contiguous(&encoded.bytes).unwrap();
        assert_eq!(decoded.len(), 28 + pcm.len());
        assert!(decoded[28 + 84..].iter().any(|sample| *sample != 0));
        assert_eq!(
            squared_error(&encoded.bytes, &pcm, &active()).unwrap(),
            encoded.squared_error
        );

        let once = decode_blocks(&encoded.bytes, 1, 3, [7_777, -6_666]).unwrap();
        let twice = decode_blocks(&encoded.bytes, 1, 3, once).unwrap();
        assert_eq!(once, twice);

        let legacy = encode(&pcm, Some(1), Effort::Fast, &active()).unwrap();
        let whole_region = encode_region(&pcm, Some([1, 4]), Effort::Fast, &active()).unwrap();
        assert_eq!(legacy.bytes, whole_region.bytes);
    }

    #[test]
    fn rejects_invalid_input_and_honours_cancellation() {
        let cancelled = AtomicBool::new(true);
        assert!(encode(&[1], None, Effort::Fast, &cancelled).is_err());
        assert!(encode(&[], None, Effort::Fast, &active()).is_err());
        assert!(encode(&[1; 28], Some(1), Effort::Fast, &active()).is_err());
        assert!(encode_region(&[1; 56], Some([1, 1]), Effort::Fast, &active()).is_err());
        assert!(encode_region(&[1; 56], Some([0, 0]), Effort::Fast, &active()).is_err());
        assert!(encode_region(&[1; 56], Some([0, 3]), Effort::Fast, &active()).is_err());
        assert!(validate_pcm_len(MAX_PCM_FRAMES).is_ok());
        assert!(validate_pcm_len(MAX_PCM_FRAMES + 1).is_err());
    }

    #[test]
    fn thorough_keeps_fast_bytes_when_total_sse_ties() {
        let pcm = vec![0; 56];
        let fast = encode(&pcm, None, Effort::Fast, &active()).unwrap();
        let thorough = encode(&pcm, None, Effort::Thorough, &active()).unwrap();
        assert_eq!(thorough.squared_error, 0);
        assert!(!thorough.improved);
        assert_eq!(thorough.bytes, fast.bytes);
    }

    #[test]
    fn integer_rounding_and_error_accumulation_do_not_wrap() {
        assert_eq!(rounded_div(3, 2), 2);
        assert_eq!(rounded_div(-3, 2), -2);
        assert!(add_square(u64::MAX, 1).is_err());
        assert!(add_square(0, i64::MAX).is_err());
    }

    fn decode_block(block: &[u8], history: [i32; 2]) -> Result<Vec<i16>, String> {
        Ok(decode_block_state(block, history)?.0)
    }

    fn decode_block_state(
        block: &[u8],
        mut history: [i32; 2],
    ) -> Result<(Vec<i16>, [i32; 2]), String> {
        let filter = *FILTERS
            .get((block[0] >> 4) as usize)
            .ok_or("invalid filter")?;
        let shift = block[0] & 15;
        let mut output = Vec::new();
        for i in 0..28 {
            let nibble = (block[2 + i / 2] >> ((i % 2) * 4)) & 15;
            let signed = ((nibble as i8) << 4) >> 4;
            let decoded =
                (((signed as i32 * 4096) >> shift) + predict(history, filter)).clamp(-32768, 32767);
            history = [decoded, history[0]];
            output.push(decoded as i16);
        }
        Ok((output, history))
    }

    fn decode_blocks(
        bytes: &[u8],
        start: usize,
        end: usize,
        mut history: [i32; 2],
    ) -> Result<[i32; 2], String> {
        for index in start..end {
            (_, history) = decode_block_state(&bytes[16 + index * 16..32 + index * 16], history)?;
        }
        Ok(history)
    }

    fn decode_contiguous(bytes: &[u8]) -> Result<Vec<i16>, String> {
        let mut output = Vec::new();
        let mut history = [0, 0];
        for block in bytes[..bytes.len() - 16].chunks_exact(16) {
            let (samples, next) = decode_block_state(block, history)?;
            output.extend(samples);
            history = next;
        }
        Ok(output)
    }
}
