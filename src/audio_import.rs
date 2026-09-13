//! Offline WAV decoding and PSX SPU-ADPCM conversion. Nothing here runs on the console.
use serde::{Deserialize, Serialize};

pub const IMPORTER_VERSION: u32 = 3;
pub const PSX_PROFILE: &str = "psx-legacy-audio-v3";
pub const SPU_BUDGET: usize = 512 * 1024 - 4096; // Capture area reserved; reverb disabled.
pub const MAX_SOURCE: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, Serialize)]
pub struct TargetResult {
    pub target: &'static str,
    pub profile: &'static str,
    pub resolved_mode: LoadMode,
    pub representation: &'static str,
    pub sample_rate: u32,
    pub sample_voices: u16,
    pub encoded_bytes: Option<u64>,
    pub main_ram_bytes: Option<u64>,
    pub spu_ram_bytes: Option<u64>,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum AudioRole {
    #[default]
    Sfx,
    Music,
    Ambience,
    Dialogue,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum LoadMode {
    #[default]
    Auto,
    Resident,
    Stream,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Quality {
    Low,
    #[default]
    Medium,
    High,
    Custom,
}

/// Portable authoring. The explicit rate is a PSX override on disk; it is kept
/// here as an editor convenience, never used to choose residency.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub role: AudioRole,
    pub load_mode: LoadMode,
    pub quality: Quality,
    pub channels: u16,
    pub trim_start: f32,
    pub trim_end: Option<f32>,
    pub sample_rate: u32,
    pub normalize: bool,
    pub looping: bool,
    pub target_overrides: std::collections::BTreeMap<String, serde_json::Value>,
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
    // Unknown fields around the tagged importer envelope are owned by its serializer.
    pub envelope_extra: std::collections::BTreeMap<String, serde_json::Value>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            role: AudioRole::Sfx,
            load_mode: LoadMode::Resident,
            quality: Quality::Custom,
            channels: 1,
            trim_start: 0.,
            trim_end: None,
            sample_rate: 22050,
            normalize: false,
            looping: false,
            target_overrides: Default::default(),
            extra: Default::default(),
            envelope_extra: Default::default(),
        }
    }
}
impl Settings {
    pub fn target_result(&self) -> TargetResult {
        let streamed = self.is_streamed();
        TargetResult {
            target: "psx",
            profile: PSX_PROFILE,
            resolved_mode: if streamed {
                LoadMode::Stream
            } else {
                LoadMode::Resident
            },
            representation: if streamed {
                "XA stream"
            } else {
                "SPU ADPCM resident"
            },
            sample_rate: self.rate(),
            sample_voices: if streamed { 0 } else { 1 },
            encoded_bytes: None,
            main_ram_bytes: None,
            spu_ram_bytes: streamed.then_some(0),
            error: self.validate_psx().err(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=2).contains(&self.channels) || !(8000..=192000).contains(&self.sample_rate) {
            return Err("Audio requires mono/stereo and an 8–192 kHz custom sample rate".into());
        }
        if self.target_overrides.values().any(|v| !v.is_object()) {
            return Err("Target overrides must be namespaced objects".into());
        }
        if !self.trim_start.is_finite()
            || self.trim_start < 0.
            || self
                .trim_end
                .is_some_and(|end| !end.is_finite() || end <= self.trim_start)
        {
            return Err("Invalid audio trim range".into());
        }
        Ok(())
    }
    /// Profile v3: Auto selects Resident for SFX and Stream for the other roles.
    /// It never changes the choice in response to budget pressure or file type.
    pub fn is_streamed(&self) -> bool {
        match self.load_mode {
            LoadMode::Resident => false,
            LoadMode::Stream => true,
            LoadMode::Auto => self.role != AudioRole::Sfx,
        }
    }
    pub fn rate(&self) -> u32 {
        match (self.is_streamed(), self.quality) {
            (_, Quality::Custom) => self.sample_rate,
            (true, Quality::Low) => 18900,
            (true, _) => 37800,
            (false, Quality::Low) => 11025,
            (false, Quality::Medium) => 22050,
            (false, Quality::High) => 44100,
        }
    }
    pub fn validate_psx(&self) -> Result<(), String> {
        self.validate()?;
        let rates: &[u32] = if self.is_streamed() {
            &[18900, 37800]
        } else {
            &[11025, 22050, 44100]
        };
        if !rates.contains(&self.rate()) || (!self.is_streamed() && self.channels != 1) {
            return Err("PSX: Resident requires mono 11025/22050/44100 Hz; Stream requires mono/stereo 18900/37800 Hz. Choose a quality preset or edit the PSX override; Load Mode was not changed.".into());
        }
        Ok(())
    }
}

impl Serialize for Settings {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = serde_json::to_value(&self.extra).map_err(serde::ser::Error::custom)?;
        let mut overrides = self.target_overrides.clone();
        let psx = overrides
            .entry("psx".into())
            .or_insert_with(|| serde_json::json!({}));
        let psx = psx
            .as_object_mut()
            .ok_or_else(|| serde::ser::Error::custom("Invalid PSX overrides"))?;
        psx.insert("sample_rate".into(), self.sample_rate.into());
        let fields = serde_json::json!({
            "schema_version":1,"role":self.role,"load_mode":self.load_mode,
            "quality":self.quality,"channels":self.channels,"trim_start":self.trim_start,
            "trim_end":self.trim_end,"normalize":self.normalize,"looping":self.looping,
            "target_overrides":overrides
        });
        value
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        value.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Settings {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let mut v = serde_json::Value::deserialize(d)?;
        let map = v
            .as_object_mut()
            .ok_or_else(|| D::Error::custom("Audio settings must be an object"))?;
        let modern = map.contains_key("schema_version");
        let mut s = Self::default();
        if modern {
            if map.remove("schema_version") != Some(serde_json::json!(1)) {
                return Err(D::Error::custom(
                    "Unsupported audio settings schema version",
                ));
            }
            s.role = serde_json::from_value(
                map.remove("role")
                    .ok_or_else(|| D::Error::custom("Missing audio role"))?,
            )
            .map_err(D::Error::custom)?;
            s.load_mode = serde_json::from_value(
                map.remove("load_mode")
                    .ok_or_else(|| D::Error::custom("Missing audio load_mode"))?,
            )
            .map_err(D::Error::custom)?;
        } else {
            let usage = map
                .remove("usage")
                .map(serde_json::from_value::<String>)
                .transpose()
                .map_err(D::Error::custom)?;
            match usage.as_deref() {
                None | Some("Sfx") => {}
                Some("Music") => {
                    s.role = AudioRole::Music;
                    s.load_mode = LoadMode::Stream;
                }
                _ => return Err(D::Error::custom("Invalid legacy audio usage")),
            }
            if !map.contains_key("sample_rate") {
                return Err(D::Error::custom(
                    "Legacy audio settings require sample_rate",
                ));
            }
            if map.contains_key("role") || map.contains_key("load_mode") {
                return Err(D::Error::custom(
                    "Portable audio intent requires schema_version",
                ));
            }
        }
        macro_rules! read { ($($field:ident),*) => { $(if let Some(value) = map.remove(stringify!($field)) {
            s.$field = serde_json::from_value(value).map_err(D::Error::custom)?;
        })* }; }
        read!(
            channels,
            quality,
            trim_start,
            trim_end,
            normalize,
            looping,
            sample_rate,
            target_overrides
        );
        if let Some(psx) = s.target_overrides.get_mut("psx") {
            let psx = psx
                .as_object_mut()
                .ok_or_else(|| D::Error::custom("Invalid PSX overrides"))?;
            if let Some(rate) = psx.remove("sample_rate") {
                s.sample_rate = serde_json::from_value(rate).map_err(D::Error::custom)?;
            }
            if psx.is_empty() {
                s.target_overrides.remove("psx");
            }
        }
        s.extra = map.clone().into_iter().collect();
        s.validate().map_err(D::Error::custom)?;
        Ok(s)
    }
}
pub use crate::audio_ir::Info;
pub struct Converted {
    pub adpcm: Vec<u8>,
    pub info: Info,
    pub waveform: Vec<f32>,
}

pub fn decode(bytes: &[u8]) -> Result<(Info, Vec<f32>), String> {
    // Preserve the resident profile's strict legacy WAV formats and exact PCM
    // conversion. Streaming/source audition retain their existing decoder path.
    let ir = if bytes.get(..4) == Some(b"RIFF") {
        let (info, samples) = crate::audio_decode::decode_wav(bytes)?;
        crate::audio_ir::DecodedAudioIr::from_pcm(info, samples)?
    } else {
        crate::audio_ir::DecodedAudioIr::decode(bytes)?
    };
    let mono = ir.mono();
    Ok((ir.info, mono))
}

pub fn convert(source: &[u8], settings: &Settings) -> Result<Converted, String> {
    settings.validate_psx()?;
    if settings.is_streamed() {
        return Err("Use the XA converter for BGM".into());
    }
    let (info, samples) = decode(source)?;
    let (start, end) = trim_range(&info, settings)?;
    let samples = &samples[start..end];
    let count =
        (samples.len() as u64 * settings.rate() as u64).div_ceil(info.sample_rate as u64) as usize;
    if (count.div_ceil(28) + 2) * 16 > SPU_BUDGET {
        return Err(
            "Clip exceeds the PSX sample-memory budget. Shorten it or lower the sample rate."
                .into(),
        );
    }
    let pcm = resample_pcm(samples, info.sample_rate, settings.rate());
    let peak = pcm.iter().fold(0_f32, |a, b| a.max(b.abs()));
    let gain = if settings.normalize && peak > 0. {
        0.95 / peak
    } else {
        1.
    };
    let waveform = pcm
        .chunks(pcm.len().div_ceil(256))
        .map(|chunk| {
            chunk
                .iter()
                .fold(0_f32, |a, b| a.max((b * gain).abs()))
                .min(1.)
        })
        .collect();
    let pcm = pcm
        .iter()
        .map(|s| (s * gain * 32767.).round().clamp(-32768., 32767.) as i16)
        .collect::<Vec<_>>();
    Ok(Converted {
        adpcm: encode(&pcm, settings.looping),
        info,
        waveform,
    })
}

/// Shared PSX resampler; callers check their output budget before allocating.
pub fn resample_pcm(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    let count = (samples.len() as u64 * target_rate as u64).div_ceil(source_rate as u64) as usize;
    let mut pcm = Vec::with_capacity(count);
    let ratio = source_rate as f64 / target_rate as f64;
    let cutoff = (1. / ratio).min(1.) * 0.90;
    for i in 0..count {
        let pos = i as f64 * ratio;
        if target_rate == source_rate {
            pcm.push(samples[i]);
            continue;
        }
        let mut sum = 0.;
        let mut weight = 0.;
        // Windowed sinc low-pass prevents downsampling from aliasing high frequencies.
        for tap in -16..=16 {
            let index = pos.floor() as isize + tap;
            let distance = pos - index as f64;
            let x = std::f64::consts::PI * distance * cutoff;
            let sinc = if x.abs() < 1e-8 { 1. } else { x.sin() / x };
            let w = sinc * (0.5 + 0.5 * (std::f64::consts::PI * distance / 17.).cos());
            sum += samples[index.clamp(0, samples.len() as isize - 1) as usize] as f64 * w;
            weight += w;
        }
        pcm.push((sum / weight) as f32);
    }
    pcm
}

pub fn trim_range(info: &Info, settings: &Settings) -> Result<(usize, usize), String> {
    crate::audio_ir::trim_range(info, settings.trim_start, settings.trim_end)
}

const FILTERS: [[i32; 2]; 5] = [[0, 0], [60, 0], [115, -52], [98, -55], [122, -60]];
/// Decode cooked SPU blocks for explicit Target Preview. Never used as a source master.
pub fn decode_adpcm(bytes: &[u8]) -> Result<Vec<i16>, String> {
    if bytes.len() < 48 || !bytes.len().is_multiple_of(16) {
        return Err("Invalid PSX ADPCM block size".into());
    }
    let mut samples = Vec::with_capacity(bytes.len() / 16 * 28);
    let mut history = [0_i32; 2];
    for block in bytes.chunks_exact(16) {
        let filter = *FILTERS
            .get((block[0] >> 4) as usize)
            .ok_or("Invalid PSX ADPCM filter")?;
        let shift = block[0] & 15;
        if shift > 12 {
            return Err("Invalid PSX ADPCM shift".into());
        }
        for i in 0..28 {
            let nibble = (block[2 + i / 2] >> ((i % 2) * 4)) & 15;
            let signed = ((nibble as i8) << 4) >> 4;
            let value =
                (((signed as i32 * 4096) >> shift) + predict(history, filter)).clamp(-32768, 32767);
            history = [value, history[0]];
            samples.push(value as i16);
        }
        if block[1] & 1 != 0 {
            break;
        }
    }
    Ok(samples)
}
fn predict(history: [i32; 2], filter: [i32; 2]) -> i32 {
    (history[0] * filter[0] + history[1] * filter[1] + 32) >> 6
}
pub fn encode(pcm: &[i16], looping: bool) -> Vec<u8> {
    encode_blocks(pcm, looping.then_some(0))
}
/// The caller quantizes a bank loop to complete 28-frame blocks and truncates
/// PCM at its exclusive end. Filter zero makes every loop entry independent.
pub fn encode_blocks(pcm: &[i16], loop_start_block: Option<usize>) -> Vec<u8> {
    let mut out = vec![0_u8; 16]; // Initial silent block, as expected by the SPU.
    let mut history = [0_i32; 2];
    let blocks = pcm.len().div_ceil(28);
    for (index, chunk) in pcm.chunks(28).enumerate() {
        let mut block = [0_i16; 28];
        block[..chunk.len()].copy_from_slice(chunk);
        let mut best = (u64::MAX, [0_u8; 16], history);
        // The loop entry must decode independently of the previous iteration's history.
        let filter_count = if loop_start_block == Some(index) {
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
                    error += (i64::from(*sample) - i64::from(decoded)).pow(2) as u64;
                    h = [decoded, h[0]];
                    encoded[2 + i / 2] |= ((q as u8) & 15) << ((i % 2) * 4);
                }
                if error < best.0 {
                    best = (error, encoded, h);
                }
            }
        }
        history = best.2;
        best.1[1] = if index + 1 == blocks {
            if loop_start_block.is_some() { 3 } else { 1 }
        } else {
            0
        };
        if loop_start_block == Some(index) {
            best.1[1] |= 4;
        }
        out.extend_from_slice(&best.1);
    }
    let mut silence = [0_u8; 16];
    silence[1] = 7;
    out.extend_from_slice(&silence);
    out
}

#[cfg(test)]
pub fn test_wav() -> Vec<u8> {
    let pcm = (0..2205)
        .flat_map(|i| {
            (((i as f32 * 440. * std::f32::consts::TAU / 22050.).sin() * 12000.) as i16)
                .to_le_bytes()
        })
        .collect::<Vec<_>>();
    let mut wav = b"RIFF".to_vec();
    wav.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt \x10\0\0\0\x01\0\x01\0");
    wav.extend_from_slice(&22050_u32.to_le_bytes());
    wav.extend_from_slice(&44100_u32.to_le_bytes());
    wav.extend_from_slice(b"\x02\0\x10\0data");
    wav.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    wav.extend(pcm);
    wav
}
#[cfg(test)]
mod tests {
    use super::*;
    fn wav(encoding: u16, bits: u16, channels: u16, data: Vec<u8>) -> Vec<u8> {
        let mut bytes = test_wav();
        bytes.truncate(44);
        bytes[20..22].copy_from_slice(&encoding.to_le_bytes());
        bytes[22..24].copy_from_slice(&channels.to_le_bytes());
        let align = channels * bits / 8;
        bytes[28..32].copy_from_slice(&(22050 * u32::from(align)).to_le_bytes());
        bytes[32..34].copy_from_slice(&align.to_le_bytes());
        bytes[34..36].copy_from_slice(&bits.to_le_bytes());
        bytes[40..44].copy_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend(data);
        if !bytes.len().is_multiple_of(2) {
            bytes.push(0);
        }
        let size = bytes.len() as u32 - 8;
        bytes[4..8].copy_from_slice(&size.to_le_bytes());
        bytes
    }
    #[test]
    fn supported_sample_formats_stereo_downmix_and_resampling() {
        for (encoding, bits, data) in [
            (1, 8, vec![0, 255]),
            (1, 16, vec![0, 128, 255, 127]),
            (1, 24, vec![0, 0, 128, 255, 255, 127]),
            (1, 32, vec![0, 0, 0, 128, 255, 255, 255, 127]),
            (
                3,
                32,
                [-1_f32, 1.]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        ] {
            let (_, samples) = decode(&wav(encoding, bits, 1, data.clone())).unwrap();
            assert_eq!(samples[0], -1.);
            assert!(samples[1] > 0.99);
            let (_, stereo) = decode(&wav(encoding, bits, 2, data)).unwrap();
            assert_eq!(stereo.len(), 1);
            assert!(stereo[0].abs() < 0.004);
        }
        assert!(decode(&wav(3, 32, 1, f32::NAN.to_le_bytes().to_vec())).is_err());
        assert!(decode(&wav(6, 16, 1, vec![0; 8])).is_err());
        let converted = convert(
            &test_wav(),
            &Settings {
                sample_rate: 11025,
                normalize: true,
                looping: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(converted.adpcm.len(), (1103_usize.div_ceil(28) + 2) * 16);
        assert!(converted.waveform.iter().copied().fold(0_f32, f32::max) > 0.9);
        let oversized = wav(1, 16, 1, vec![0; 2 * 920000]);
        assert!(
            convert(&oversized, &Settings::default())
                .err()
                .unwrap()
                .contains("budget")
        );
    }
    #[test]
    fn wav_validation_and_spu_decode_quality() {
        let wav = test_wav();
        let (_, pcm) = decode(&wav).unwrap();
        assert!(decode(&wav[..wav.len() - 1]).is_err());
        let converted = convert(&wav, &Settings::default()).unwrap();
        let mut h = [0_i32; 2];
        let mut decoded = Vec::new();
        for block in converted.adpcm[16..converted.adpcm.len() - 16].chunks_exact(16) {
            for i in 0..28 {
                let nibble = (block[2 + i / 2] >> ((i % 2) * 4)) & 15;
                let signed = ((nibble as i8) << 4) >> 4;
                let sample = (((signed as i32 * 4096) >> (block[0] & 15))
                    + predict(h, FILTERS[(block[0] >> 4) as usize]))
                .clamp(-32768, 32767);
                h = [sample, h[0]];
                decoded.push(sample as f32 / 32768.);
            }
        }
        let rms = (pcm
            .iter()
            .zip(decoded)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            / pcm.len() as f32)
            .sqrt();
        assert!(rms < 0.015, "ADPCM error {rms}");
        assert_eq!(converted.adpcm[converted.adpcm.len() - 31], 1);
        let looped = convert(
            &wav,
            &Settings {
                looping: true,
                ..Settings::default()
            },
        )
        .unwrap();
        assert_eq!(looped.adpcm[17] & 4, 4);
        assert_eq!(looped.adpcm[16] >> 4, 0);
        assert_eq!(looped.adpcm[looped.adpcm.len() - 31] & 3, 3);
    }
}
