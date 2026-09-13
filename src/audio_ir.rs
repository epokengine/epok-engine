//! Neutral decoded source. No console addresses, encoded blocks or residency decisions.
use serde::{Deserialize, Serialize};

pub const MAX_SAMPLES: usize = 52_920_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Info {
    pub sample_rate: u32,
    pub channels: u16,
    pub frames: usize,
}

pub struct DecodedAudioIr {
    pub info: Info,
    /// Finite, normalized, interleaved PCM at the original source rate.
    pub samples: Vec<f32>,
    /// Half-open source frame range. Processing is deferred to each consumer.
    pub trim: std::ops::Range<usize>,
    pub normalize: bool,
    pub loop_region: Option<std::ops::Range<usize>>,
}

impl DecodedAudioIr {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let (info, samples) = crate::audio_decode::decode(bytes)?;
        Self::from_pcm(info, samples)
    }

    pub fn from_pcm(info: Info, samples: Vec<f32>) -> Result<Self, String> {
        if !(1..=2).contains(&info.channels)
            || !(8000..=192000).contains(&info.sample_rate)
            || info.frames == 0
            || samples.len() > MAX_SAMPLES
            || info.frames.checked_mul(info.channels as usize) != Some(samples.len())
            || samples.iter().any(|v| !v.is_finite())
        {
            return Err("Invalid or oversized decoded PCM".into());
        }
        let trim = 0..info.frames;
        Ok(Self {
            info,
            samples,
            trim,
            normalize: false,
            loop_region: None,
        })
    }

    pub fn with_edits(
        mut self,
        start: f32,
        end: Option<f32>,
        normalize: bool,
        looping: bool,
    ) -> Result<Self, String> {
        let (start, end) = trim_range(&self.info, start, end)?;
        self.trim = start..end;
        self.normalize = normalize;
        self.loop_region = looping.then_some(start..end);
        Ok(self)
    }

    pub fn trimmed_samples(&self) -> &[f32] {
        let channels = self.info.channels as usize;
        &self.samples[self.trim.start * channels..self.trim.end * channels]
    }

    pub fn mono(&self) -> Vec<f32> {
        self.trimmed_samples()
            .chunks_exact(self.info.channels as usize)
            .map(|frame| frame.iter().sum::<f32>() / self.info.channels as f32)
            .collect()
    }
}

pub fn trim_range(info: &Info, start: f32, end: Option<f32>) -> Result<(usize, usize), String> {
    if !start.is_finite() || start < 0. || end.is_some_and(|v| !v.is_finite() || v < 0.) {
        return Err("Trim times must be finite and nonnegative".into());
    }
    let start = (start as f64 * info.sample_rate as f64).round() as usize;
    let end = end.map_or(info.frames, |end| {
        (end as f64 * info.sample_rate as f64).round() as usize
    });
    if start >= end || end > info.frames {
        return Err("Trim must lie inside the source duration".into());
    }
    Ok((start, end))
}
