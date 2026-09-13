//! Source PCM for reachable SoundFont samples. No resampling, downmix or target cook.
use crate::{instrument_ir::LibraryIr, instrument_selection::Coverage};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
};

pub const MAX_SAMPLE_FRAMES: usize = 8_388_608;

#[derive(Clone, Debug)]
pub struct SamplePcm {
    pub rate: u32,
    pub samples: Vec<f32>,
}

pub fn decode_selected(
    source: &[u8],
    library: &LibraryIr,
    selection: &Coverage,
    cancelled: &AtomicBool,
) -> Result<BTreeMap<u16, SamplePcm>, String> {
    selection.require_complete()?;
    let mut output = BTreeMap::new();
    let mut remaining = crate::audio_ir::MAX_SAMPLES;
    for &id in &selection.samples {
        if cancelled.load(Ordering::Relaxed) {
            return Err("Instrument sample decode cancelled".into());
        }
        let header = library
            .samples
            .get(id as usize)
            .ok_or("Selected sample is outside its library")?;
        if header.sample_type & 0x8000 != 0 {
            return Err(format!(
                "Sample {id} ({}) requires external ROM data",
                header.name
            ));
        }
        let bytes = source
            .get(header.data_range.clone())
            .ok_or("Instrument sample extends past the authoritative source")?;
        let limit = remaining.min(MAX_SAMPLE_FRAMES);
        let samples = if header.sample_type & 0x10 != 0 {
            if !bytes.starts_with(b"OggS") {
                return Err(format!("SF3 sample {id} is not a Vorbis stream"));
            }
            let (info, pcm) = crate::audio_decode::decode_soundfont_vorbis_with_budget(
                bytes,
                limit,
                Some(cancelled),
            )
            .map_err(|error| format!("SF3 sample {id} ({}): {error}", header.name))?;
            if info.channels != 1 || info.sample_rate != header.sample_rate {
                return Err(format!(
                    "SF3 sample {id} decoded channels/rate disagree with its mono SoundFont definition"
                ));
            }
            pcm
        } else {
            if bytes.len() % 2 != 0 || bytes.len() / 2 > limit {
                return Err(format!(
                    "PCM sample {id} exceeds the remaining {limit}-frame instrument budget or has an incomplete frame"
                ));
            }
            bytes
                .chunks_exact(2)
                .enumerate()
                .map(|(frame, pair)| {
                    if frame.is_multiple_of(4096) && cancelled.load(Ordering::Relaxed) {
                        return Err("Instrument sample decode cancelled".to_string());
                    }
                    Ok(i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32768.)
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        if samples.is_empty() {
            return Err(format!("Instrument sample {id} is empty"));
        }
        remaining -= samples.len();
        output.insert(
            id,
            SamplePcm {
                rate: header.sample_rate,
                samples,
            },
        );
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::{BTreeMap, BTreeSet},
        env, fs,
    };

    #[test]
    fn soundfont_decodes_only_selected_source_pcm_and_preserves_cancellation() {
        let bytes = crate::sf2::fixture();
        let library = crate::sf2::parse(&bytes).unwrap();
        let song = crate::midi::parse(&crate::midi::fixture()).unwrap();
        let selection =
            crate::instrument_selection::resolve(&song, &library, &[], &AtomicBool::new(false))
                .unwrap();
        let pcm = decode_selected(&bytes, &library, &selection, &AtomicBool::new(false)).unwrap();
        assert_eq!(pcm.len(), 1);
        let header = &library.samples[0];
        assert_eq!(pcm[&0].samples.len(), header.data_range.len() / 2);
        for (actual, pair) in pcm[&0]
            .samples
            .iter()
            .zip(bytes[header.data_range.clone()].chunks_exact(2))
        {
            assert_eq!(
                *actual,
                i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32768.
            );
        }
        assert!(
            decode_selected(&bytes, &library, &selection, &AtomicBool::new(true))
                .unwrap_err()
                .contains("cancelled")
        );
    }

    #[test]
    fn embedded_vorbis_enforces_budget_before_accumulating_large_pcm() {
        let bytes = include_bytes!("../tests/fixtures/portable-tone.ogg");
        assert!(
            crate::audio_decode::decode_with_budget(bytes, 100, None)
                .unwrap_err()
                .contains("100-sample PCM budget")
        );
        assert!(
            crate::audio_decode::decode_with_budget(bytes, 1000, Some(&AtomicBool::new(true)))
                .unwrap_err()
                .contains("cancelled")
        );
    }

    #[test]
    #[ignore = "External FluidR3Mono corpus; set EPOK_SF3_AUDIT_SOURCE explicitly"]
    fn optional_fluidr3mono_sf3_acceptance_uses_ogg_granule_frames() {
        let path = env::var("EPOK_SF3_AUDIT_SOURCE").expect("EPOK_SF3_AUDIT_SOURCE");
        let source = fs::read(path).unwrap();
        let library = crate::sf2::parse(&source).unwrap();
        let id = 481u16; // Clarinet A#5(L), a final-granule/non-block-boundary probe.
        let selection = Coverage {
            note_on_events: 1,
            regions: BTreeMap::new(),
            samples: BTreeSet::from([id]),
            missing: BTreeSet::new(),
            peak_layers_per_note: 1,
            matching_operations: 1,
        };
        let output =
            decode_selected(&source, &library, &selection, &AtomicBool::new(false)).unwrap();
        assert_eq!(output[&id].samples.len(), 4122);
        assert_eq!(output[&id].rate, 44100);
    }
}
