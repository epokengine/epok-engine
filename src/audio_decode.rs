//! Desktop-only decoding. Runtime representations are generated from PCM, never MP3.
use crate::audio_ir::Info;
use std::io::Cursor;
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, errors::Error, formats::FormatOptions,
    io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
};

fn open_with_gapless(
    bytes: &[u8],
    gapless: bool,
) -> Result<Box<dyn symphonia::core::formats::FormatReader>, String> {
    let stream = MediaSourceStream::new(Box::new(Cursor::new(bytes.to_vec())), Default::default());
    symphonia::default::get_probe()
        .format(
            &Hint::new(),
            stream,
            &FormatOptions {
                enable_gapless: gapless,
                ..Default::default()
            },
            &MetadataOptions::default(),
        )
        .map(|p| p.format)
        .map_err(|e| format!("Unsupported or invalid audio: {e}"))
}
fn open(bytes: &[u8]) -> Result<Box<dyn symphonia::core::formats::FormatReader>, String> {
    // Preserve the existing AudioClip import recipe, including encoder-delay trimming.
    open_with_gapless(bytes, true)
}

pub fn probe(bytes: &[u8]) -> Result<(), String> {
    let format = open(bytes)?;
    let track = format.default_track().ok_or("No audio track")?;
    symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn decode(bytes: &[u8]) -> Result<(Info, Vec<f32>), String> {
    decode_with_budget(bytes, crate::audio_ir::MAX_SAMPLES, None)
}

/// Embedded instrument samples share the decoder, with a smaller remaining PCM
/// budget and packet-level cancellation. The sampled AudioClip recipe is unchanged.
pub fn decode_with_budget(
    bytes: &[u8],
    sample_limit: usize,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> Result<(Info, Vec<f32>), String> {
    decode_limited(bytes, sample_limit, cancelled, true, true, None)
}

/// Decode one self-contained SF3 Vorbis stream. Ogg's final granule position is
/// its authoritative decoded-frame count, so this intentionally enables Symphonia's
/// packet trimming but does not apply the AudioClip amplitude clamp.
pub fn decode_soundfont_vorbis_with_budget(
    bytes: &[u8],
    sample_limit: usize,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> Result<(Info, Vec<f32>), String> {
    if !bytes.starts_with(b"OggS") {
        return Err("Expected a self-contained Ogg Vorbis stream".into());
    }
    let terminal_frames = ogg_terminal_frames(bytes, cancelled)?;
    let sample_limit = sample_limit.min(crate::audio_ir::MAX_SAMPLES);
    if terminal_frames > sample_limit {
        return Err(format!(
            "Decoded instrument exceeds its remaining {sample_limit}-sample PCM budget"
        ));
    }
    let (info, samples) = decode_limited(
        bytes,
        sample_limit,
        cancelled,
        true,
        false,
        Some(terminal_frames),
    )?;
    if info.frames != terminal_frames {
        return Err(format!(
            "Vorbis decoder produced {} frames before the Ogg final granule {terminal_frames}",
            info.frames,
        ));
    }
    Ok((info, samples))
}

fn ogg_terminal_frames(
    bytes: &[u8],
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> Result<usize, String> {
    let mut position = 0usize;
    let mut serial = None;
    let mut sequence = None;
    let mut terminal = None;
    while position < bytes.len() {
        if cancelled.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed)) {
            return Err("Audio decode cancelled".into());
        }
        let header = bytes
            .get(position..position + 27)
            .ok_or("Truncated Ogg page header")?;
        if &header[..4] != b"OggS" || header[4] != 0 {
            return Err("Invalid Ogg page header".into());
        }
        if header[5] & !0x07 != 0 {
            return Err("Invalid Ogg page flags".into());
        }
        let page_serial = u32::from_le_bytes(header[14..18].try_into().unwrap());
        if serial
            .replace(page_serial)
            .is_some_and(|previous| previous != page_serial)
        {
            return Err("SF3 sample contains multiple Ogg logical streams".into());
        }
        let page_sequence = u32::from_le_bytes(header[18..22].try_into().unwrap());
        if sequence
            .replace(page_sequence.wrapping_add(1))
            .is_some_and(|expected| expected != page_sequence)
        {
            return Err("Ogg page sequence is discontinuous".into());
        }
        if position == 0 {
            if header[5] & 0x02 == 0 || header[5] & 0x01 != 0 {
                return Err("SF3 Ogg stream has no valid beginning-of-stream page".into());
            }
        } else if header[5] & 0x02 != 0 {
            return Err("SF3 Ogg stream has multiple beginning-of-stream pages".into());
        }
        let segments = usize::from(header[26]);
        let lacing_start = position
            .checked_add(27)
            .ok_or("Ogg lacing offset overflows")?;
        let lacing_end = lacing_start
            .checked_add(segments)
            .ok_or("Ogg lacing size overflows")?;
        let lacing = bytes
            .get(lacing_start..lacing_end)
            .ok_or("Truncated Ogg lacing table")?;
        let payload = lacing
            .iter()
            .try_fold(0usize, |total, &value| {
                total.checked_add(usize::from(value)).ok_or(())
            })
            .map_err(|_| "Ogg page payload overflows")?;
        let end = lacing_end
            .checked_add(payload)
            .ok_or("Ogg page size overflows")?;
        if end > bytes.len() {
            return Err("Truncated Ogg page payload".into());
        }
        let expected_crc = u32::from_le_bytes(header[22..26].try_into().unwrap());
        if ogg_crc(&bytes[position..end]) != expected_crc {
            return Err("Ogg page CRC is invalid".into());
        }
        if header[5] & 0x04 != 0 {
            if end != bytes.len() || terminal.is_some() {
                return Err("Ogg end-of-stream page is not terminal".into());
            }
            let granule = u64::from_le_bytes(header[6..14].try_into().unwrap());
            terminal = Some(
                usize::try_from(granule).map_err(|_| "Ogg final granule exceeds this platform")?,
            );
        }
        position = end;
    }
    terminal.ok_or("SF3 Ogg stream has no terminal granule".into())
}

fn ogg_crc(page: &[u8]) -> u32 {
    let mut crc = 0u32;
    for (index, &byte) in page.iter().enumerate() {
        let byte = if (22..26).contains(&index) { 0 } else { byte };
        crc ^= u32::from(byte) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 {
                (crc << 1) ^ 0x04c1_1db7
            } else {
                crc << 1
            };
        }
    }
    crc
}

fn decode_limited(
    bytes: &[u8],
    sample_limit: usize,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
    gapless: bool,
    clamp: bool,
    terminal_frames: Option<usize>,
) -> Result<(Info, Vec<f32>), String> {
    let sample_limit = sample_limit.min(crate::audio_ir::MAX_SAMPLES);
    if bytes.get(..4) == Some(b"RIFF")
        && (bytes.get(8..12) != Some(b"WAVE") || u32_at(bytes, 4)? as usize + 8 != bytes.len())
    {
        return Err("Incomplete WAV or unexpected data after RIFF.".into());
    }
    let mut format = if gapless {
        open(bytes)?
    } else {
        open_with_gapless(bytes, false)?
    };
    let track = format.default_track().ok_or("No audio track")?;
    let id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;
    let mut samples = Vec::new();
    let mut spec = None;
    loop {
        if cancelled.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed)) {
            return Err("Audio decode cancelled".into());
        }
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(format!("Audio packet: {e}")),
        };
        if packet.track_id() != id {
            continue;
        }
        let decoded = decoder
            .decode(&packet)
            .map_err(|e| format!("Audio decode: {e}"))?;
        let current = *decoded.spec();
        let channels = current.channels.count();
        if !(1..=2).contains(&channels) || !(8000..=192000).contains(&current.rate) {
            return Err("Audio must be mono/stereo at 8–192 kHz".into());
        }
        if spec.is_some_and(|old| old != current) {
            return Err("Audio format changes mid-stream".into());
        }
        spec = Some(current);
        let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, current);
        buffer.copy_interleaved_ref(decoded);
        let endpoint = terminal_frames
            .map(|frames| {
                frames
                    .checked_mul(channels)
                    .ok_or("Ogg final granule overflows PCM frame count")
            })
            .transpose()?;
        if endpoint.is_none() && buffer.samples().len() > sample_limit.saturating_sub(samples.len())
        {
            if sample_limit < crate::audio_ir::MAX_SAMPLES {
                return Err(format!(
                    "Decoded instrument exceeds its remaining {sample_limit}-sample PCM budget"
                ));
            }
            return Err(
                "Decoded audio exceeds the 202 MiB import limit. Shorten the source.".into(),
            );
        }
        let keep = endpoint.map_or(buffer.samples().len(), |end| {
            end.saturating_sub(samples.len())
                .min(buffer.samples().len())
        });
        if endpoint.is_some_and(|end| end > sample_limit) {
            return Err(format!(
                "Decoded instrument exceeds its remaining {sample_limit}-sample PCM budget"
            ));
        }
        for (index, &sample) in buffer.samples()[..keep].iter().enumerate() {
            if index.is_multiple_of(4096)
                && cancelled.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed))
            {
                return Err("Audio decode cancelled".into());
            }
            if !sample.is_finite() {
                return Err("Audio contains non-finite samples".into());
            }
            samples.push(if clamp { sample.clamp(-1., 1.) } else { sample });
        }
    }
    let spec = spec.ok_or("Audio is empty")?;
    let channels = spec.channels.count();
    if let Some(frames) = terminal_frames
        && samples.len()
            != frames
                .checked_mul(channels)
                .ok_or("Ogg final granule overflows PCM frame count")?
    {
        return Err(format!(
            "Vorbis decoder ended before the Ogg final granule {frames}"
        ));
    }
    if samples.is_empty() {
        return Err("Audio is empty".into());
    }
    Ok((
        Info {
            sample_rate: spec.rate,
            channels: channels as u16,
            frames: samples.len() / channels,
        },
        samples,
    ))
}

fn u16_at(bytes: &[u8], i: usize) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        bytes
            .get(i..i + 2)
            .ok_or("Truncated WAV")?
            .try_into()
            .unwrap(),
    ))
}
fn u32_at(bytes: &[u8], i: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        bytes
            .get(i..i + 4)
            .ok_or("Truncated WAV")?
            .try_into()
            .unwrap(),
    ))
}
pub fn decode_wav(bytes: &[u8]) -> Result<(Info, Vec<f32>), String> {
    if bytes.len() > crate::audio_import::MAX_SOURCE
        || bytes.get(..4) != Some(b"RIFF")
        || bytes.get(8..12) != Some(b"WAVE")
    {
        return Err("Expected a RIFF WAV file (maximum 32 MiB).".into());
    }
    let end = u32_at(bytes, 4)? as usize + 8;
    if end != bytes.len() {
        return Err("Incomplete WAV or unexpected data after RIFF.".into());
    }
    let (mut format, mut data, mut offset) = (None, None, 12);
    while offset + 8 <= end {
        let size = u32_at(bytes, offset + 4)? as usize;
        let chunk = bytes
            .get(offset + 8..offset + 8 + size)
            .ok_or("Truncated WAV chunk")?;
        match &bytes[offset..offset + 4] {
            b"fmt " if format.replace(chunk).is_some() => return Err("Duplicate WAV format".into()),
            b"data" if data.replace(chunk).is_some() => {
                return Err("Multiple WAV data chunks are not supported".into());
            }
            _ => {}
        }
        offset += 8 + size + (size & 1);
    }
    if offset != end {
        return Err("Invalid WAV chunk padding".into());
    }
    let fmt = format.ok_or("WAV has no format chunk")?;
    let data = data.ok_or("WAV has no audio data")?;
    let (encoding, channels, rate, align, bits) = (
        u16_at(fmt, 0)?,
        u16_at(fmt, 2)?,
        u32_at(fmt, 4)?,
        u16_at(fmt, 12)?,
        u16_at(fmt, 14)?,
    );
    if !(1..=2).contains(&channels)
        || !(8000..=192000).contains(&rate)
        || !(encoding == 1 && [8, 16, 24, 32].contains(&bits) || encoding == 3 && bits == 32)
        || align != channels * (bits / 8)
        || u32_at(fmt, 8)? != rate * u32::from(align)
    {
        return Err(
            "Supported WAV: mono/stereo PCM 8/16/24/32-bit or IEEE float32, 8-192 kHz.".into(),
        );
    }
    if data.is_empty() || !data.len().is_multiple_of(align as usize) {
        return Err("Incomplete WAV sample frame".into());
    }
    let mut samples = Vec::with_capacity(data.len() / (bits / 8) as usize);
    for frame in data.chunks_exact(align as usize) {
        for sample in frame.chunks_exact((bits / 8) as usize) {
            let v = match (encoding, bits) {
                (3, 32) => f32::from_le_bytes(sample.try_into().unwrap()),
                (_, 8) => (sample[0] as f32 - 128.) / 128.,
                (_, 16) => i16::from_le_bytes(sample.try_into().unwrap()) as f32 / 32768.,
                (_, 24) => {
                    (i32::from_le_bytes([0, sample[0], sample[1], sample[2]]) >> 8) as f32
                        / 8388608.
                }
                (_, 32) => i32::from_le_bytes(sample.try_into().unwrap()) as f32 / 2147483648.,
                _ => unreachable!(),
            };
            if !v.is_finite() {
                return Err("WAV contains non-finite samples".into());
            }
            samples.push(v.clamp(-1., 1.));
        }
    }
    Ok((
        Info {
            sample_rate: rate,
            channels,
            frames: samples.len() / channels as usize,
        },
        samples,
    ))
}

#[cfg(test)]
mod tests {
    use crate::audio_import::{self, Settings};
    use std::sync::atomic::AtomicBool;
    const MP3: &[u8] = include_bytes!("../tests/fixtures/stereo-tone.mp3");

    fn ogg_page(serial: u32, sequence: u32, flags: u8, granule: u64, payload: &[u8]) -> Vec<u8> {
        assert!(payload.len() <= 255);
        let mut page = b"OggS\0".to_vec();
        page.push(flags);
        page.extend_from_slice(&granule.to_le_bytes());
        page.extend_from_slice(&serial.to_le_bytes());
        page.extend_from_slice(&sequence.to_le_bytes());
        page.extend_from_slice(&0u32.to_le_bytes());
        page.push(1);
        page.push(payload.len() as u8);
        page.extend_from_slice(payload);
        let checksum = super::ogg_crc(&page);
        page[22..26].copy_from_slice(&checksum.to_le_bytes());
        page
    }

    #[test]
    fn sf3_ogg_endpoint_requires_one_terminal_granule() {
        let mut stream = ogg_page(7, 0, 2, 0, b"header");
        stream.extend(ogg_page(7, 1, 4, 37, b"audio"));
        assert_eq!(super::ogg_terminal_frames(&stream, None).unwrap(), 37);
        assert!(super::ogg_terminal_frames(&stream[..stream.len() - 1], None).is_err());
        let mut multiple = ogg_page(7, 0, 6, 1, b"a");
        multiple.extend(ogg_page(7, 1, 4, 2, b"b"));
        assert!(super::ogg_terminal_frames(&multiple, None).unwrap_err().contains("not terminal"));
        let mut skipped = ogg_page(7, 0, 2, 0, b"header");
        skipped.extend(ogg_page(7, 2, 4, 37, b"audio"));
        assert!(super::ogg_terminal_frames(&skipped, None).unwrap_err().contains("discontinuous"));
        assert!(super::decode_soundfont_vorbis_with_budget(&stream, 36, None).unwrap_err().contains("36-sample PCM budget"));
        let mut damaged = stream.clone();
        *damaged.last_mut().unwrap() ^= 1;
        assert!(
            super::ogg_terminal_frames(&damaged, None)
                .unwrap_err()
                .contains("CRC")
        );
        let no_bos = ogg_page(7, 0, 0, 37, b"audio");
        assert!(
            super::ogg_terminal_frames(&no_bos, None)
                .unwrap_err()
                .contains("beginning-of-stream")
        );
        assert!(
            super::ogg_terminal_frames(&stream, Some(&AtomicBool::new(true)))
                .unwrap_err()
                .contains("cancelled")
        );
    }
    #[test]
    fn mp3_decode_retains_channels_and_supports_sfx_trimming() {
        let (info, pcm) = super::decode(MP3).unwrap();
        assert_eq!((info.sample_rate, info.channels), (44100, 2));
        assert!((132300..=137000).contains(&info.frames));
        assert!(pcm.chunks_exact(2).any(|f| (f[0] - f[1]).abs() > 0.1));
        let settings = Settings {
            trim_start: 0.5,
            trim_end: Some(0.6),
            ..Default::default()
        };
        let result = audio_import::convert(MP3, &settings).unwrap();
        assert_eq!(result.adpcm.len(), (2205_usize.div_ceil(28) + 2) * 16);
        assert!(super::decode(b"not an mp3").is_err());
        assert!(super::decode(&MP3[..12]).is_err());
        assert!(
            audio_import::convert(
                MP3,
                &Settings {
                    trim_end: Some(100.),
                    ..settings
                }
            )
            .is_err()
        );
    }
    #[test]
    fn legacy_settings_remain_sfx() {
        let settings: Settings =
            serde_json::from_str(r#"{"sample_rate":22050,"normalize":false,"looping":true}"#)
                .unwrap();
        assert_eq!(settings.role, audio_import::AudioRole::Sfx);
        assert_eq!(settings.load_mode, audio_import::LoadMode::Resident);
        assert_eq!(settings.channels, 1);
        assert!(settings.validate().is_ok());
    }
}
