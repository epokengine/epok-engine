//! Bounded host audition of an authoritative SoundFont library selection.
//!
//! This intentionally consumes `psx_library::Prepared` before its PSX recipe:
//! `SamplePcm` is passed by pointer in its original frame/rate coordinates and
//! source voice offsets, loops, modulators and dynamic filter remain live.
use crate::{
    instrument_voice::{Curve, Destination, Input, LoopMode, ModSource, Sustain, Voice},
    preview_audio::Pcm,
    psx_library,
    sequence_stream::Event,
};
use std::{
    collections::BTreeMap,
    ffi::c_void,
    sync::atomic::{AtomicBool, Ordering},
};

pub const RATE: u32 = 44_100;
const MAX_FRAMES: usize = crate::audio_ir::MAX_SAMPLES / 2;

#[repr(C)]
struct NativePcm {
    framesdata: *const f32,
    frames: u32,
    rate: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct NativeEnvelope {
    delay: i32,
    attack: i32,
    hold: i32,
    decay: i32,
    sustain: i32,
    release: i32,
    hold_key: i32,
    decay_key: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct NativeLfo {
    delay: i32,
    frequency: i32,
    pitch: i32,
    filter: i32,
    volume: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct NativeModulation {
    source: u16,
    amount_source: u16,
    destination: u16,
    flags: u16,
    amount: i64,
}
#[repr(C)]
struct NativeRegion {
    sample: u16,
    bank: u16,
    program: u8,
    percussion: u8,
    key_lo: u8,
    key_hi: u8,
    velocity_lo: u8,
    velocity_hi: u8,
    root_key: u8,
    fixed_key: u8,
    fixed_velocity: u8,
    loop_mode: u8,
    exclusive_class: u16,
    tune: i32,
    scale: i32,
    attenuation: i32,
    pan: i32,
    filter_cents: i32,
    filter_centibels: i32,
    mod_env_pitch: i32,
    mod_env_filter: i32,
    reverb: i32,
    chorus: i32,
    start_offset: i64,
    end_offset: i64,
    loop_start: i64,
    loop_end: i64,
    volume_envelope: NativeEnvelope,
    modulation_envelope: NativeEnvelope,
    modulation_lfo: NativeLfo,
    vibrato_lfo: NativeLfo,
    modulations: *const NativeModulation,
    modulation_count: u16,
    reserved: u16,
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
    fn epok_source_instrument_create(
        events: *const Event,
        count: u32,
        ppqn: u16,
        regions: *const NativeRegion,
        region_count: u32,
        samples: *const NativePcm,
        sample_count: u32,
        rate: u32,
    ) -> *mut c_void;
    fn epok_source_instrument_destroy(handle: *mut c_void);
    fn epok_source_instrument_render(
        handle: *mut c_void,
        stereo_output: *mut i16,
        frames: u32,
    ) -> i32;
    fn epok_source_instrument_stats(handle: *const c_void) -> Stats;
    fn epok_source_instrument_set_gain(handle: *mut c_void, gain: f32) -> bool;
}

struct Native(*mut c_void);
impl Drop for Native {
    fn drop(&mut self) {
        unsafe { epok_source_instrument_destroy(self.0) }
    }
}

fn cancelled(flag: &AtomicBool) -> Result<(), String> {
    if flag.load(Ordering::Relaxed) {
        Err("Source instrument preview cancelled".into())
    } else {
        Ok(())
    }
}

fn source_bits(source: ModSource) -> Result<u16, String> {
    let (index, controller) = match source.input {
        Input::Constant => (0, false),
        Input::Velocity => (2, false),
        Input::Key => (3, false),
        Input::PolyPressure => (10, false),
        Input::ChannelPressure => (13, false),
        Input::PitchWheel => (14, false),
        Input::PitchWheelRange => (16, false),
        Input::Controller(index) => (index, true),
    };
    if controller
        && (index == 0
            || index == 6
            || (32..=63).contains(&index)
            || (98..=101).contains(&index)
            || index >= 120)
    {
        return Err(format!(
            "Source preview received unsupported controller source {index}"
        ));
    }
    let curve = match source.curve {
        Curve::Linear => 0,
        Curve::Concave => 1,
        Curve::Convex => 2,
        Curve::Switch => 3,
    };
    Ok(u16::from(index)
        | (u16::from(controller) << 7)
        | (u16::from(source.reversed) << 8)
        | (u16::from(source.bipolar) << 9)
        | (curve << 10))
}

fn destination(destination: Destination) -> u16 {
    use Destination::*;
    match destination {
        StartFrames => 1,
        EndFrames => 2,
        LoopStartFrames => 3,
        LoopEndFrames => 4,
        ModLfoPitchCents => 5,
        VibLfoPitchCents => 6,
        ModEnvPitchCents => 7,
        FilterCents => 8,
        FilterCentibels => 9,
        ModLfoFilterCents => 10,
        ModEnvFilterCents => 11,
        ModLfoVolumeCentibels => 12,
        ChorusPermille => 13,
        ReverbPermille => 14,
        PanPermille => 15,
        ModLfoDelayCents => 16,
        ModLfoFrequencyCents => 17,
        VibLfoDelayCents => 18,
        VibLfoFrequencyCents => 19,
        ModEnvDelayCents => 20,
        ModEnvAttackCents => 21,
        ModEnvHoldCents => 22,
        ModEnvDecayCents => 23,
        ModEnvSustainPermille => 24,
        ModEnvReleaseCents => 25,
        ModEnvHoldKeyCents => 26,
        ModEnvDecayKeyCents => 27,
        VolEnvDelayCents => 28,
        VolEnvAttackCents => 29,
        VolEnvHoldCents => 30,
        VolEnvDecayCents => 31,
        VolEnvSustainCentibels => 32,
        VolEnvReleaseCents => 33,
        VolEnvHoldKeyCents => 34,
        VolEnvDecayKeyCents => 35,
        AttenuationCentibels => 36,
        PitchCents => 37,
        ScaleCentsPerKey => 38,
    }
}

fn envelope(source: &crate::instrument_voice::Envelope) -> NativeEnvelope {
    let sustain = match source.sustain {
        Sustain::AttenuationCentibels(value) | Sustain::ReductionPermille(value) => value,
    };
    NativeEnvelope {
        delay: source.delay.0,
        attack: source.attack.0,
        hold: source.hold.0,
        decay: source.decay.0,
        sustain,
        release: source.release.0,
        hold_key: source.hold_cents_per_key,
        decay_key: source.decay_cents_per_key,
    }
}
fn lfo(source: &crate::instrument_voice::Lfo) -> NativeLfo {
    NativeLfo {
        delay: source.delay.0,
        frequency: source.frequency_cents,
        pitch: source.pitch_cents,
        filter: source.filter_cents,
        volume: source.volume_centibels,
    }
}
fn region(
    voice: &Voice,
    instrument: crate::instrument_selection::Instrument,
    sample: u16,
) -> NativeRegion {
    NativeRegion {
        sample,
        bank: instrument.bank,
        program: instrument.program,
        percussion: u8::from(instrument.percussion),
        key_lo: voice.key_range[0],
        key_hi: voice.key_range[1],
        velocity_lo: voice.velocity_range[0],
        velocity_hi: voice.velocity_range[1],
        root_key: voice.root_key,
        fixed_key: voice.fixed_key.unwrap_or(255),
        fixed_velocity: voice.fixed_velocity.unwrap_or(255),
        loop_mode: match voice.loop_mode {
            LoopMode::Off => 0,
            LoopMode::Continuous => 1,
            LoopMode::UntilRelease => 3,
        },
        exclusive_class: voice.exclusive_class,
        tune: voice.tune_cents,
        scale: voice.scale_cents_per_key,
        attenuation: voice.attenuation_centibels,
        pan: voice.pan_permille,
        filter_cents: voice.filter_cents,
        filter_centibels: voice.filter_centibels,
        mod_env_pitch: voice.mod_env_pitch_cents,
        mod_env_filter: voice.mod_env_filter_cents,
        reverb: voice.reverb_permille,
        chorus: voice.chorus_permille,
        start_offset: voice.start_offset,
        end_offset: voice.end_offset,
        loop_start: voice.loop_start,
        loop_end: voice.loop_end,
        volume_envelope: envelope(&voice.volume_envelope),
        modulation_envelope: envelope(&voice.modulation_envelope),
        modulation_lfo: lfo(&voice.modulation_lfo),
        vibrato_lfo: lfo(&voice.vibrato_lfo),
        modulations: std::ptr::null(),
        modulation_count: 0,
        reserved: 0,
    }
}

/// Renders the selected source library at 44.1 kHz with a linear host mixer.
/// It is intentionally dry: reverb/chorus sends are retained in the native
/// input for diagnostics but no effect bus is invented for an audition.
pub fn render(
    events: &[Event],
    ppqn: u16,
    prepared: &psx_library::Prepared,
    output_frames: usize,
    headroom_centibels: u16,
    cancelled_flag: &AtomicBool,
) -> Result<(Pcm, Stats), String> {
    cancelled(cancelled_flag)?;
    if events.is_empty()
        || events.len() > u32::MAX as usize
        || ppqn == 0
        || prepared.regions.is_empty()
        || prepared.regions.len() > 128
        || prepared.pcm.is_empty()
        || prepared.pcm.len() > 128
        || output_frames == 0
        || headroom_centibels > 960
        || output_frames > MAX_FRAMES
    {
        return Err("Invalid bounded source instrument preview input".into());
    }
    let mut sample_ids = BTreeMap::new();
    let mut native_pcm = Vec::with_capacity(prepared.pcm.len());
    let mut total_frames = 0usize;
    for (&id, pcm) in &prepared.pcm {
        cancelled(cancelled_flag)?;
        let frames = pcm.samples.len();
        total_frames = total_frames
            .checked_add(frames)
            .ok_or("Source instrument PCM frame count overflows")?;
        if frames == 0
            || frames > u32::MAX as usize
            || !(400..=192_000).contains(&pcm.rate)
            || total_frames > crate::audio_ir::MAX_SAMPLES
            || pcm.samples.iter().any(|sample| !sample.is_finite())
        {
            return Err("Source instrument PCM exceeds the bounded preview budget".into());
        }
        sample_ids.insert(id, native_pcm.len() as u16);
        native_pcm.push(NativePcm {
            framesdata: pcm.samples.as_ptr(),
            frames: frames as u32,
            rate: pcm.rate,
        });
    }
    let mut modulations = Vec::<Vec<NativeModulation>>::with_capacity(prepared.regions.len());
    for selected in &prepared.regions {
        cancelled(cancelled_flag)?;
        if selected.voice.modulations.len() > 32 {
            return Err("Source instrument region exceeds its 32-modulator preview bound".into());
        }
        let mut converted = Vec::with_capacity(selected.voice.modulations.len());
        for modulation in &selected.voice.modulations {
            let amount = i32::try_from(modulation.amount).map_err(
                |_| "Source instrument modulator amount exceeds the native preview range",
            )?;
            converted.push(NativeModulation {
                source: source_bits(modulation.source)?,
                amount_source: source_bits(modulation.amount_source)?,
                destination: destination(modulation.destination),
                flags: u16::from(modulation.absolute),
                amount: i64::from(amount),
            });
        }
        modulations.push(converted);
    }
    let mut regions = Vec::with_capacity(prepared.regions.len());
    for (index, selected) in prepared.regions.iter().enumerate() {
        let sample = sample_ids
            .get(&selected.voice.sample)
            .copied()
            .ok_or("Selected source region references missing decoded PCM")?;
        let mut converted = region(&selected.voice, selected.instrument, sample);
        converted.modulations = modulations[index].as_ptr();
        converted.modulation_count = modulations[index].len() as u16;
        regions.push(converted);
    }
    let native = Native(unsafe {
        epok_source_instrument_create(
            events.as_ptr(),
            events.len() as u32,
            ppqn,
            regions.as_ptr(),
            regions.len() as u32,
            native_pcm.as_ptr(),
            native_pcm.len() as u32,
            RATE,
        )
    });
    if native.0.is_null() {
        return Err("Cannot initialize source instrument preview; verify source PCM, voice ranges and modulators".into());
    }
    let has_sends = prepared.regions.iter().any(|region| {
        region.voice.reverb_permille != 0
            || region.voice.chorus_permille != 0
            || region.voice.modulations.iter().any(|modulation| {
                matches!(
                    modulation.destination,
                    Destination::ReverbPermille | Destination::ChorusPermille
                ) && modulation.amount != 0
            })
    });
    if !unsafe {
        epok_source_instrument_set_gain(
            native.0,
            10_f32.powf(-f32::from(headroom_centibels) / 200.),
        )
    } {
        return Err("Invalid Source Preview comparison gain".into());
    }
    let mut pcm = Pcm {
        samples: vec![
            0;
            output_frames
                .checked_mul(2)
                .ok_or("Source instrument preview output size overflows")?
        ],
        rate: RATE,
        channels: 2,
        loop_region: None,
        report: Some(if has_sends {
            "Source SoundFont preview: original decoded PCM, source-rate cursors and dynamic two-pole filters; the audition is explicitly dry, so SoundFont reverb/chorus sends are not rendered.".into()
        } else {
            "Source SoundFont preview: original decoded PCM, source-rate cursors and dynamic two-pole filters; no PSX conversion was applied.".into()
        }),
        timeline: None,
    };
    for chunk in pcm.samples.chunks_mut(4096 * 2) {
        cancelled(cancelled_flag)?;
        let error = unsafe {
            epok_source_instrument_render(native.0, chunk.as_mut_ptr(), (chunk.len() / 2) as u32)
        };
        if error != 0 {
            return Err(format!(
                "Source instrument preview stopped with diagnostic {error}"
            ));
        }
    }
    cancelled(cancelled_flag)?;
    let stats = unsafe { epok_source_instrument_stats(native.0) };
    if stats.error != 0 {
        return Err(format!(
            "Source instrument preview reported diagnostic {}",
            stats.error
        ));
    }
    Ok((pcm, stats))
}
