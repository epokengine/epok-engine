use crate::audio_import::{Converted, Settings};
use std::{fs, io::Write, path::Path, process::Command};
pub const SECTOR: usize = 2336;
// Allow rounding the final audio sector at the sparsest (32-sector) spacing.
pub const MAX_BYTES: usize = (150 * 600 + 32 + 1) * SECTOR;

pub fn convert(root: &Path, source: &[u8], settings: &Settings) -> Result<Converted, String> {
    settings.validate_psx()?;
    if !settings.is_streamed() {
        return Err("Expected BGM import settings".into());
    }
    let ir = crate::audio_ir::DecodedAudioIr::decode(source)?.with_edits(
        settings.trim_start,
        settings.trim_end,
        settings.normalize,
        settings.looping,
    )?;
    let info = ir.info.clone();
    if ir.trim.len() > info.sample_rate as usize * 600 {
        return Err("BGM clips are limited to ten minutes".into());
    }
    let pcm = ir.trimmed_samples();
    let peak = pcm.iter().copied().fold(0_f32, |a, b| a.max(b.abs()));
    let gain = if ir.normalize && peak > 0. {
        0.95 / peak
    } else {
        1.
    };
    let waveform = pcm
        .chunks(pcm.len().div_ceil(256))
        .map(|c| c.iter().fold(0_f32, |a, b| a.max((b * gain).abs())))
        .collect();
    let job = root
        .join(".epok/audio-jobs")
        .join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&job).map_err(|e| e.to_string())?;
    let input = job.join("input.wav");
    let output = job.join("encoded.xa");
    let result = (|| {
        let mut file =
            std::io::BufWriter::new(fs::File::create(&input).map_err(|e| e.to_string())?);
        let bytes = pcm.len() as u32 * 2;
        let align = info.channels * 2;
        let mut header = b"RIFF".to_vec();
        header.extend((bytes + 36).to_le_bytes());
        header.extend(b"WAVEfmt \x10\0\0\0\x01\0");
        header.extend(info.channels.to_le_bytes());
        header.extend(info.sample_rate.to_le_bytes());
        header.extend((info.sample_rate * align as u32).to_le_bytes());
        header.extend(align.to_le_bytes());
        header.extend(16_u16.to_le_bytes());
        header.extend(b"data");
        header.extend(bytes.to_le_bytes());
        file.write_all(&header).map_err(|e| e.to_string())?;
        for sample in pcm {
            file.write_all(
                &((sample * gain * 32767.).round().clamp(-32768., 32767.) as i16).to_le_bytes(),
            )
            .map_err(|e| e.to_string())?;
        }
        file.flush().map_err(|e| e.to_string())?;
        drop(file);
        crate::disc::run(
            Command::new(crate::disc::tool(root, "psxavenc")?)
                .args([
                    "-q",
                    "-t",
                    "xa",
                    "-f",
                    &settings.rate().to_string(),
                    "-c",
                    &settings.channels.to_string(),
                    "-b",
                    "4",
                    "-F",
                    "0",
                    "-C",
                    "0",
                ])
                .arg(&input)
                .arg(&output),
            &job.join("encode.log"),
            || false,
        )?;
        let raw = fs::read(output).map_err(|e| e.to_string())?;
        let adpcm = interleave(&raw, settings)?;
        Ok(Converted {
            adpcm,
            info,
            waveform,
        })
    })();
    // This is a freshly created importer-owned directory, never a source path.
    let _ = fs::remove_dir_all(&job);
    result
}
pub fn interleave(raw: &[u8], settings: &Settings) -> Result<Vec<u8>, String> {
    settings.validate_psx()?;
    if !settings.is_streamed() {
        return Err("Expected BGM import settings".into());
    }
    if raw.is_empty() || !raw.len().is_multiple_of(SECTOR) {
        return Err("Encoder returned invalid XA sectors".into());
    }
    let stride =
        8 * (2 / settings.channels as usize) * if settings.rate() == 18900 { 2 } else { 1 };
    let size = raw
        .len()
        .checked_mul(stride)
        .and_then(|n| n.checked_add(SECTOR))
        .ok_or("XA too large")?;
    if size > MAX_BYTES {
        return Err("BGM exceeds ten-minute disc allocation".into());
    }
    let mut result = Vec::with_capacity(size);
    let mut padding = [0_u8; SECTOR];
    padding[..8].copy_from_slice(&[0, 31, 0x64, 0, 0, 31, 0x64, 0]);
    for sector in raw.chunks_exact(SECTOR) {
        if sector[..4] != sector[4..8]
            || sector[0] != 0
            || sector[1] != 0
            || sector[2] & 0x64 != 0x64
        {
            return Err("Invalid XA audio subheader".into());
        }
        result.extend_from_slice(sector);
        for _ in 1..stride {
            result.extend_from_slice(&padding);
        }
    }
    // One ordinary data sector signals completion; padding sectors are filtered by hardware.
    let mut end = [0_u8; SECTOR];
    end[..8].copy_from_slice(&[0, 0, 0x89, 0, 0, 0, 0x89, 0]);
    end[8..16].copy_from_slice(b"EPOKEND1");
    result.extend_from_slice(&end);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xa_spacing_and_end_marker_cover_all_profiles() {
        for (rate, channels, stride) in [
            (37800, 2, 8),
            (37800, 1, 16),
            (18900, 2, 16),
            (18900, 1, 32),
        ] {
            let settings = Settings {
                role: crate::audio_import::AudioRole::Music,
                load_mode: crate::audio_import::LoadMode::Stream,
                sample_rate: rate,
                channels,
                ..Default::default()
            };
            let mut raw = vec![0; SECTOR * 2];
            for s in raw.chunks_exact_mut(SECTOR) {
                s[..8].copy_from_slice(&[0, 0, 0x64, 0, 0, 0, 0x64, 0]);
            }
            let output = interleave(&raw, &settings).unwrap();
            assert_eq!(output.len(), (2 * stride + 1) * SECTOR);
            assert_eq!(&output[..SECTOR], &raw[..SECTOR]);
            assert_eq!(
                &output[stride * SECTOR..(stride + 1) * SECTOR],
                &raw[..SECTOR]
            );
            assert_eq!(output[SECTOR + 1], 31);
            assert_eq!(
                &output[output.len() - SECTOR + 8..output.len() - SECTOR + 16],
                b"EPOKEND1"
            );
            assert!(interleave(&raw[..raw.len() - 1], &settings).is_err());
            raw[0] = 1;
            assert!(interleave(&raw, &settings).is_err());
        }
        assert!(
            interleave(
                &vec![0; SECTOR],
                &Settings {
                    role: crate::audio_import::AudioRole::Music,
                    load_mode: crate::audio_import::LoadMode::Stream,
                    channels: 0,
                    ..Default::default()
                }
            )
            .is_err()
        );
    }
}
