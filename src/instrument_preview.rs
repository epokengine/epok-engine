//! Bounded host audition of a cooked EPSB v2 instrument library.
//!
//! This module consumes the real wire payload and its cooked SPU-ADPCM samples.
//! It does not resolve assets, select regions, author settings, or publish a bank.
use crate::{preview_audio::Pcm, psx_library, sequence_stream::Event};
use std::{
    ffi::c_void,
    sync::atomic::{AtomicBool, Ordering},
};

pub const RATE: u32 = 44_100;
const MAX_FRAMES: usize = crate::audio_ir::MAX_SAMPLES / 2;
const MAX_TARGET_SAMPLE_FRAMES: usize = (crate::audio_import::SPU_BUDGET / 16) * 28;

#[repr(C)]
struct NativePcm {
    framesdata: *const f32,
    frames: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub error: u32,
    pub logical_peak: u32,
    pub physical_peak: u32,
    pub steals: u32,
    pub denied: u32,
    pub loops: u32,
    pub clipped: u32,
    pub sample_loops: u32,
}

unsafe extern "C" {
    fn epok_native_preview_create(
        stream: *const u8,
        length: u32,
        bank: *const u8,
        size: u32,
        samples: *const NativePcm,
        count: u32,
    ) -> *mut c_void;
    fn epok_native_preview_destroy(handle: *mut c_void);
    fn epok_native_preview_render(handle: *mut c_void, output: *mut i16, frames: u32) -> i32;
    fn epok_native_preview_stats(handle: *const c_void) -> Stats;
    fn epok_instrument_prepared_count(
        events: *const Event,
        count: u32,
        ppqn: u16,
        bytes: *const u8,
        size: u32,
        references: *mut u32,
    ) -> i32;
    fn epok_instrument_create(
        events: *const Event,
        count: u32,
        ppqn: u16,
        voice_limit: u16,
        bank_bytes: *const u8,
        bank_len: u32,
        samples: *const NativePcm,
        sample_count: u32,
        rate: u32,
    ) -> *mut c_void;
    fn epok_instrument_destroy(handle: *mut c_void);
    fn epok_instrument_render(handle: *mut c_void, stereo_output: *mut i16, frames: u32) -> i32;
    fn epok_instrument_stats(handle: *const c_void) -> Stats;
}

/// Exact number of immutable note-start states required by the resident stream.
/// Uses the same bounded control/loop traversal as preparation on the console.
pub fn prepared_count(events: &[Event], ppqn: u16, bank: &[u8]) -> Result<(u16, u32), String> {
    if events.is_empty() || events.len() > 65536 || bank.len() > u32::MAX as usize {
        return Err("Invalid PSX instrument preparation inputs".into());
    }
    let mut references = 0;
    let count = unsafe {
        epok_instrument_prepared_count(
            events.as_ptr(),
            events.len() as u32,
            ppqn,
            bank.as_ptr(),
            bank.len() as u32,
            &mut references,
        )
    };
    if count < 1 {
        return Err("PSX instrument preparation failed: missing mappings, invalid control stream, more than 1024 distinct note-start states or 65535 layer references. Split the sequence or simplify its instrument/control variation".into());
    }
    Ok((count as u16, references))
}

struct Native(*mut c_void, bool);
impl Drop for Native {
    fn drop(&mut self) {
        unsafe {
            if self.1 {
                epok_native_preview_destroy(self.0)
            } else {
                epok_instrument_destroy(self.0)
            }
        }
    }
}

fn cancelled(flag: &AtomicBool) -> Result<(), String> {
    if flag.load(Ordering::Relaxed) {
        Err("Instrument preview cancelled".into())
    } else {
        Ok(())
    }
}

/// Renders a prepared target preview at 44.1 kHz.
///
/// `output_frames` is selected by the caller from its sequence timeline and
/// release-tail policy. Keeping that policy outside this low-level renderer
/// makes its allocation bound explicit and keeps this function independent of
/// authoring and UI state.
pub fn render(
    events: &[Event],
    ppqn: u16,
    bank_bytes: &[u8],
    samples: &[psx_library::Sample],
    voice_limit: u16,
    output_frames: usize,
    cancelled_flag: &AtomicBool,
) -> Result<(Pcm, Stats), String> {
    render_impl(
        events,
        ppqn,
        bank_bytes,
        samples,
        voice_limit,
        output_frames,
        cancelled_flag,
        None,
    )
}
pub fn render_native(
    events: &[Event],
    ppqn: u16,
    bank_bytes: &[u8],
    samples: &[psx_library::Sample],
    voice_limit: u16,
    output_frames: usize,
    cancelled_flag: &AtomicBool,
    stream: &[u8],
) -> Result<(Pcm, Stats), String> {
    render_impl(
        events,
        ppqn,
        bank_bytes,
        samples,
        voice_limit,
        output_frames,
        cancelled_flag,
        Some(stream),
    )
}
fn render_impl(
    events: &[Event],
    ppqn: u16,
    bank_bytes: &[u8],
    samples: &[psx_library::Sample],
    voice_limit: u16,
    output_frames: usize,
    cancelled_flag: &AtomicBool,
    compiled: Option<&[u8]>,
) -> Result<(Pcm, Stats), String> {
    cancelled(cancelled_flag)?;
    if events.is_empty()
        || events.len() > u32::MAX as usize
        || ppqn == 0
        || !(1..=24).contains(&voice_limit)
        || bank_bytes.len() > u32::MAX as usize
        || samples.is_empty()
        || samples.len() > 128
        || output_frames == 0
        || output_frames > MAX_FRAMES
    {
        return Err("Invalid bounded instrument preview input".into());
    }

    // `decode_contiguous` deliberately continues after the UntilRelease loop
    // flag; the legacy AudioClip decoder stops at its first end/repeat flag and
    // would lose the retained release tail.
    let mut decoded = Vec::<Vec<f32>>::with_capacity(samples.len());
    let mut total_frames = 0usize;
    for sample in samples {
        cancelled(cancelled_flag)?;
        let frames = usize::try_from(sample.frames)
            .map_err(|_| "Cooked target sample frame count exceeds this platform")?;
        total_frames = total_frames
            .checked_add(frames)
            .ok_or("Cooked target sample frame count overflows")?;
        if !(400..=44_100).contains(&sample.rate)
            || frames < 56
            || frames % 28 != 0
            || total_frames > MAX_TARGET_SAMPLE_FRAMES
        {
            return Err("Cooked target sample exceeds the bounded EPSB preview PCM budget".into());
        }
        let source = psx_library::decode_contiguous(&sample.bytes, sample.frames)
            .map_err(|error| format!("Invalid cooked target ADPCM: {error}"))?;
        if source.len() != frames {
            return Err(
                "Cooked target ADPCM frame count disagrees with EPSB sample metadata".into(),
            );
        }
        let mut pcm = Vec::with_capacity(frames);
        for (index, value) in source.into_iter().enumerate() {
            if index.is_multiple_of(4096) {
                cancelled(cancelled_flag)?;
            }
            pcm.push(value as f32 / 32_768.0);
        }
        decoded.push(pcm);
    }
    let native_samples = decoded
        .iter()
        .zip(samples)
        .map(|(pcm, sample)| NativePcm {
            framesdata: pcm.as_ptr(),
            frames: sample.frames,
        })
        .collect::<Vec<_>>();

    let native = Native(
        unsafe {
            if let Some(stream) = compiled {
                epok_native_preview_create(
                    stream.as_ptr(),
                    stream.len() as u32,
                    bank_bytes.as_ptr(),
                    bank_bytes.len() as u32,
                    native_samples.as_ptr(),
                    native_samples.len() as u32,
                )
            } else {
                epok_instrument_create(
                    events.as_ptr(),
                    events.len() as u32,
                    ppqn,
                    voice_limit,
                    bank_bytes.as_ptr(),
                    bank_bytes.len() as u32,
                    native_samples.as_ptr(),
                    native_samples.len() as u32,
                    RATE,
                )
            }
        },
        compiled.is_some(),
    );
    if native.0.is_null() {
        return Err("Cannot initialize EPSB v2 target instrument preview; verify its wire payload and decoded target PCM".into());
    }
    let mut pcm = Pcm {
        samples: vec![0; output_frames.checked_mul(2).ok_or("Instrument preview output size overflows")?],
        rate: RATE,
        channels: 2,
        loop_region: None,
        report: Some("EPSB v2 target preview: decoded cooked SPU ADPCM with a linear host mixer; it does not emulate SPU Gaussian interpolation, key-on latency, DMA, or SFX contention.".into()),
        timeline: None,
    };
    for chunk in pcm.samples.chunks_mut(4096 * 2) {
        cancelled(cancelled_flag)?;
        let error = unsafe {
            if native.1 {
                epok_native_preview_render(native.0, chunk.as_mut_ptr(), (chunk.len() / 2) as u32)
            } else {
                epok_instrument_render(native.0, chunk.as_mut_ptr(), (chunk.len() / 2) as u32)
            }
        };
        if error != 0 {
            return Err(format!(
                "Target instrument preview stopped with diagnostic {error}"
            ));
        }
    }
    cancelled(cancelled_flag)?;
    let stats = unsafe {
        if native.1 {
            epok_native_preview_stats(native.0)
        } else {
            epok_instrument_stats(native.0)
        }
    };
    if native.1 {
        pcm.report=Some("Epok Pulse target preview: decoded cooked SPU ADPCM, compiled register commands and quantized hardware ADSR model. Linear host interpolation; no Gaussian interpolation, key-on latency or SFX contention.".into());
    }
    if stats.error != 0 {
        return Err(format!(
            "Target instrument preview reported diagnostic {}",
            stats.error
        ));
    }
    Ok((pcm, stats))
}
