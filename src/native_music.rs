//! Offline MIDI/SoundFont compilation to a bounded SPU register stream.
use crate::sequence_stream::Event;
use std::{
    ffi::c_void,
    sync::atomic::{AtomicBool, Ordering},
};
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, serde::Serialize)]
pub struct Stats {
    pub error: u32,
    pub commands: u32,
    pub tones: u32,
    pub peak: u32,
    pub steals: u32,
    pub adapted: u32,
    pub automation: u32,
}
unsafe extern "C" {
    fn epok_native_music_compile(
        events: *const Event,
        count: u32,
        ppqn: u16,
        voices: u16,
        bank: *const u8,
        size: u32,
    ) -> *mut c_void;
    fn epok_native_music_bytes(handle: *mut c_void, size: *mut u32, stats: *mut Stats)
    -> *const u8;
    fn epok_native_music_destroy(handle: *mut c_void);
}
struct Native(*mut c_void);
impl Drop for Native {
    fn drop(&mut self) {
        unsafe {
            epok_native_music_destroy(self.0);
        }
    }
}
pub fn compile(
    events: &[Event],
    ppqn: u16,
    voices: u16,
    bank: &[u8],
    id: uuid::Uuid,
    cancel: &AtomicBool,
) -> Result<(Vec<u8>, Stats), String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("Epok Pulse conversion cancelled".into());
    }
    if events.is_empty() || events.len() > 65536 || bank.len() > 4 * 1024 * 1024 {
        return Err("Epok Pulse input exceeds conversion bounds".into());
    }
    let handle = Native(unsafe {
        epok_native_music_compile(
            events.as_ptr(),
            events.len() as u32,
            ppqn,
            voices,
            bank.as_ptr(),
            bank.len() as u32,
        )
    });
    if handle.0.is_null() {
        return Err("Epok Pulse compiler could not allocate its workspace".into());
    }
    let mut size = 0;
    let mut stats = Stats::default();
    let ptr = unsafe { epok_native_music_bytes(handle.0, &mut size, &mut stats) };
    if stats.error != 0 || ptr.is_null() || !(40..=256 * 1024).contains(&size) {
        return Err(format!(
            "Epok Pulse conversion failed (code {}). Code 1: compiled automation exceeds 256 KiB; 3: missing instrument or a note exceeds the voice budget; 5: timeline exceeds ten minutes; 9: a controller dynamically retimes a playing volume envelope, which requires SoftwareReference. Source assets are preserved; SoftwareReference remains available.",
            stats.error
        ));
    }
    if cancel.load(Ordering::Relaxed) {
        return Err("Epok Pulse conversion cancelled".into());
    }
    let mut bytes = unsafe { std::slice::from_raw_parts(ptr, size as usize) }.to_vec();
    bytes[24..40].copy_from_slice(id.as_bytes());
    Ok((bytes, stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_compiles_and_auditions_real_wire_data() {
        let cancel = AtomicBool::new(false);
        let ir = crate::midi::parse(&crate::midi::fixture()).unwrap();
        let prepared =
            crate::psx_library::prepare(&crate::sf2::tone_fixture(), &ir, &[], &cancel).unwrap();
        let cooked = crate::psx_library::cook(&prepared, &Default::default(), &cancel).unwrap();
        let bank = crate::psx_library_wire::encode(&cooked).unwrap();
        let settings = crate::sequence::Settings {
            loop_mode: crate::sequence::LoopMode::Off,
            ..Default::default()
        };
        let events = crate::sequence_stream::events(&ir, &settings).unwrap();
        let (stream, stats) =
            compile(&events, ir.ppqn, 16, &bank, uuid::Uuid::nil(), &cancel).unwrap();
        assert_eq!(&stream[..8], b"EPSQ\x03\0\x28\0");
        assert_eq!(stats.error, 0);
        assert!(stats.tones > 0 && stats.peak > 0);
        let (pcm, audition) = crate::instrument_preview::render_native(
            &events,
            ir.ppqn,
            &bank,
            &cooked.samples,
            16,
            44100,
            &cancel,
            &stream,
        )
        .unwrap();
        assert_eq!(audition.error, 0);
        assert!(pcm.samples.iter().any(|s| s.abs() > 100));
        // The compiler resolves pedals and tempo offline, while bend and pan
        // become sparse register writes rather than live synthesis work.
        let event = |tick, op, a, b, value| Event {
            tick,
            op,
            channel: 0,
            a,
            b,
            value,
        };
        let controlled = vec![
            event(0, 6, 0, 0, 0),
            event(0, 0, 60, 100, 0),
            event(1, 3, 64, 127, 0),
            event(4, 4, 0, 0, 12000),
            event(5, 3, 10, 0, 0),
            event(10, 1, 60, 0, 0),
            event(11, 5, 0, 0, 250000),
            event(20, 3, 64, 0, 0),
            event(96, 7, 0, 0, 0),
        ];
        let (stream, stats) =
            compile(&controlled, 96, 16, &bank, uuid::Uuid::nil(), &cancel).unwrap();
        assert!(stats.automation >= 2);
        let count = u32::from_le_bytes(stream[12..16].try_into().unwrap()) as usize;
        let release = stream[40..40 + count * 8]
            .chunks_exact(8)
            .find(|c| c[4] == 2)
            .unwrap();
        // 11 ticks at 500000 us/quarter, then 9 at 250000: 80.729 ms.
        let release_us = u32::from_le_bytes(release[..4].try_into().unwrap());
        assert!((80729..=80730).contains(&release_us));
        let (_, audition) = crate::instrument_preview::render_native(
            &controlled,
            96,
            &bank,
            &cooked.samples,
            16,
            44100,
            &cancel,
            &stream,
        )
        .unwrap();
        assert!(audition.loops >= 2);
        let mut invalid = controlled.clone();
        invalid.pop();
        assert!(compile(&invalid, 96, 16, &bank, uuid::Uuid::nil(), &cancel).is_err());
        cancel.store(true, Ordering::Relaxed);
        assert!(
            compile(&events, ir.ppqn, 16, &bank, uuid::Uuid::nil(), &cancel)
                .unwrap_err()
                .contains("cancelled")
        );
    }
    #[test]
    #[ignore = "Set EPOK_NATIVE_SONG_PROJECT and EPOK_NATIVE_SONG_ASSET for an installed song"]
    fn native_real_song_report() {
        let root = std::path::PathBuf::from(std::env::var_os("EPOK_NATIVE_SONG_PROJECT").unwrap());
        let asset = root.join(std::env::var_os("EPOK_NATIVE_SONG_ASSET").unwrap());
        let package = crate::assets::Package::load(&asset).unwrap();
        let index = crate::assets::scan(&root, &mut Default::default());
        let cooked = crate::psx_sequence::cook(&root, &package, &index).unwrap();
        println!("{}", serde_json::to_string_pretty(&cooked.report).unwrap());
        assert_eq!(cooked.report.profile, "psx-native-spu-v3");
        assert_eq!(cooked.report.prepared_start_states, 0);
        assert!(cooked.report.native_driver.unwrap().commands > 100);
    }
}
