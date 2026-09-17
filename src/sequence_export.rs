//! Offline sequence rendering for the streamed-XA music workflow.
//!
//! A `MusicSequence` is deliberately resident on the PSX: it is an event
//! stream plus a SoundBank.  XA is rendered audio instead, so exporting the
//! host/source audition to a conventional WAV is the explicit, reproducible
//! bridge between the two asset types.
use crate::{assets, preview_audio::Pcm};
use std::{path::Path, sync::atomic::AtomicBool};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedWav {
    pub frames: usize,
    pub rate: u32,
    pub channels: u16,
}

/// Render the original SoundBank interpretation of an imported MIDI/sequence.
/// This intentionally does not use the PSX target cook: callers use this WAV
/// as the master for a separate streamed BGM/XA AudioClip.
pub fn render_source_wav(
    root: &Path,
    sequence_path: &Path,
    destination: &Path,
) -> Result<RenderedWav, String> {
    let package = assets::Package::load(sequence_path)?;
    if package.meta.kind != assets::Kind::MusicSequence {
        return Err("WAV export requires a MusicSequence asset".into());
    }
    let mut settings = package.meta.settings.sequence()?.clone();
    // Preview renders a second pass for loop audition, but XA owns looping at
    // playback time. Export exactly one authored pass so importing the WAV as
    // a looping BGM never duplicates the song.
    settings.loop_mode = crate::sequence::LoopMode::Off;
    let authored = crate::sequence::decode_source(&package.source, &settings)?;
    let cancelled = AtomicBool::new(false);
    let (mut pcm, stats) =
        crate::sequence_preview::render(root, &package.source, &settings, &cancelled)?;
    if stats.error > 0 {
        return Err(format!(
            "MusicSequence source render reported {} synthesis errors; correct the sequence before exporting BGM",
            stats.error
        ));
    }
    // The host audition appends the longest potential instrument release so
    // manually played notes can decay. A whole-song loop restarts at the MIDI
    // endpoint, however, and XA must match that endpoint rather than add an
    // often-silent release reservation to every loop iteration.
    let authored_frames = (authored.duration_micros as u128 * u128::from(pcm.rate))
        .div_ceil(1_000_000)
        .max(1) as usize;
    pcm.samples
        .truncate(authored_frames * usize::from(pcm.channels));
    let result = RenderedWav {
        frames: pcm.samples.len() / usize::from(pcm.channels),
        rate: pcm.rate,
        channels: pcm.channels,
    };
    assets::atomic_write(destination, &wav_bytes(&pcm), None)?;
    Ok(result)
}

fn wav_bytes(pcm: &Pcm) -> Vec<u8> {
    let sample_bytes = pcm
        .samples
        .len()
        .checked_mul(std::mem::size_of::<i16>())
        .expect("bounded PCM byte length");
    let data_bytes = u32::try_from(sample_bytes).expect("bounded PCM fits WAV data chunk");
    let riff_bytes = data_bytes
        .checked_add(36)
        .expect("bounded PCM fits WAV RIFF chunk");
    let block_align = pcm.channels.checked_mul(2).expect("valid PCM channels");
    let byte_rate = pcm
        .rate
        .checked_mul(u32::from(block_align))
        .expect("bounded PCM byte rate");
    let mut bytes = Vec::with_capacity(sample_bytes + 44);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_bytes.to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&pcm.channels.to_le_bytes());
    bytes.extend_from_slice(&pcm.rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for sample in &pcm.samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcm_wav_is_standard_signed_16_bit_little_endian() {
        let bytes = wav_bytes(&Pcm {
            samples: vec![-32768, 0, 32767, -1],
            rate: 44_100,
            channels: 2,
            loop_region: None,
            report: None,
            timeline: None,
        });
        assert_eq!(&bytes[..4], b"RIFF");
        assert_eq!(&bytes[8..16], b"WAVEfmt ");
        assert_eq!(u16::from_le_bytes(bytes[20..22].try_into().unwrap()), 1);
        assert_eq!(u16::from_le_bytes(bytes[22..24].try_into().unwrap()), 2);
        assert_eq!(
            u32::from_le_bytes(bytes[24..28].try_into().unwrap()),
            44_100
        );
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(&bytes[44..], &[0, 128, 0, 0, 255, 127, 255, 255]);
    }
}
