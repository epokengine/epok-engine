//! One editor audition at a time. The driver owns neither the header nor PCM memory.

// Only the Windows driver reads the authored timeline; elsewhere `Player` refuses to start.
#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Clone, Debug)]
pub struct Timeline {
    pub duration: f32,
    pub loop_region: Option<(f32, f32)>,
}
#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Clone, Debug)]
pub struct Pcm {
    pub samples: Vec<i16>,
    pub rate: u32,
    pub channels: u16,
    /// Half-open output frame range; the driver keeps its PCM pinned until reset completes.
    pub loop_region: Option<(usize, usize)>,
    pub report: Option<String>,
    pub timeline: Option<Timeline>,
}
impl Pcm {
    pub fn duration(&self) -> f32 {
        self.samples.len() as f32 / self.channels as f32 / self.rate as f32
    }
}

#[cfg(windows)]
mod win {
    use super::*;
    use std::time::Instant;
    use std::{ffi::c_void, ptr};
    #[repr(C, packed)]
    struct Format {
        tag: u16,
        channels: u16,
        rate: u32,
        bytes: u32,
        align: u16,
        bits: u16,
        extra: u16,
    }
    #[repr(C)]
    struct Header {
        data: *mut i16,
        length: u32,
        recorded: u32,
        user: usize,
        flags: u32,
        loops: u32,
        next: *mut Header,
        reserved: usize,
    }
    #[link(name = "winmm")]
    unsafe extern "system" {
        fn waveOutOpen(
            out: *mut *mut c_void,
            device: u32,
            format: *const Format,
            callback: usize,
            instance: usize,
            flags: u32,
        ) -> u32;
        fn waveOutPrepareHeader(out: *mut c_void, header: *mut Header, size: u32) -> u32;
        fn waveOutWrite(out: *mut c_void, header: *mut Header, size: u32) -> u32;
        fn waveOutPause(out: *mut c_void) -> u32;
        fn waveOutRestart(out: *mut c_void) -> u32;
        fn waveOutReset(out: *mut c_void) -> u32;
        fn waveOutUnprepareHeader(out: *mut c_void, header: *mut Header, size: u32) -> u32;
        fn waveOutClose(out: *mut c_void) -> u32;
    }
    pub struct Player {
        device: *mut c_void,
        headers: Box<[Header]>,
        // Keep both allocations stable until reset/unprepare completes.
        _samples: Box<[i16]>,
        started: Instant,
        duration: f32,
        loop_region: Option<(f32, f32)>,
    }
    impl Player {
        pub fn start(pcm: Pcm) -> Result<Self, String> {
            if pcm.samples.is_empty()
                || !(1..=2).contains(&pcm.channels)
                || !(8000..=192000).contains(&pcm.rate)
                || pcm.samples.len() * 2 > u32::MAX as usize
                || !pcm.samples.len().is_multiple_of(pcm.channels as usize)
                || pcm
                    .loop_region
                    .is_some_and(|(a, b)| a >= b || b > pcm.samples.len() / pcm.channels as usize)
            {
                return Err("Invalid audio preview buffer".into());
            }
            let format = Format {
                tag: 1,
                channels: pcm.channels,
                rate: pcm.rate,
                bytes: pcm.rate * pcm.channels as u32 * 2,
                align: pcm.channels * 2,
                bits: 16,
                extra: 0,
            };
            let mut device = ptr::null_mut();
            let result = unsafe { waveOutOpen(&mut device, u32::MAX, &format, 0, 0, 0) };
            if result != 0 {
                return Err(format!("Cannot open audio output (Windows error {result})"));
            }
            let duration = pcm
                .timeline
                .as_ref()
                .map_or(pcm.duration(), |t| t.duration.max(0.000_001));
            let loop_region = pcm
                .timeline
                .as_ref()
                .map(|t| t.loop_region)
                .unwrap_or_else(|| {
                    pcm.loop_region
                        .map(|(a, b)| (a as f32 / pcm.rate as f32, b as f32 / pcm.rate as f32))
                });
            let frames = pcm.samples.len() / pcm.channels as usize;
            let channels = pcm.channels as usize;
            let mut samples = pcm.samples.into_boxed_slice();
            let header = |start: usize, end: usize, looping: bool| Header {
                data: unsafe { samples.as_mut_ptr().add(start * channels) },
                length: ((end - start) * channels * 2) as u32,
                recorded: 0,
                user: 0,
                flags: if looping { 4 | 8 } else { 0 }, // WHDR_BEGINLOOP | WHDR_ENDLOOP
                loops: if looping { u32::MAX } else { 0 },
                next: ptr::null_mut(),
                reserved: 0,
            };
            let mut header = header;
            let headers = if let Some((start, end)) = pcm.loop_region {
                let mut headers = Vec::with_capacity(2);
                if start > 0 {
                    headers.push(header(0, start, false));
                }
                headers.push(header(start, end, true));
                headers
            } else {
                vec![header(0, frames, false)]
            }
            .into_boxed_slice();
            let mut player = Self {
                device,
                headers,
                _samples: samples,
                started: Instant::now(),
                duration,
                loop_region,
            };
            let size = std::mem::size_of::<Header>() as u32;
            for header in &mut player.headers {
                let result = unsafe { waveOutPrepareHeader(device, header, size) };
                if result != 0 {
                    return Err(format!("Cannot prepare audio preview ({result})"));
                }
                let result = unsafe { waveOutWrite(device, header, size) };
                if result != 0 {
                    return Err(format!("Cannot play audio preview ({result})"));
                }
            }
            Ok(player)
        }
        pub fn finished(&self) -> bool {
            // WHDR_DONE is updated by WinMM while the prepared header remains allocated.
            unsafe {
                ptr::read_volatile(ptr::addr_of!(self.headers.last().unwrap().flags)) & 1 != 0
            }
        }
        pub fn pause(&self, paused: bool) {
            unsafe {
                if paused {
                    waveOutPause(self.device);
                } else {
                    waveOutRestart(self.device);
                }
            }
        }
        pub fn progress(&self) -> f32 {
            let mut elapsed = self.started.elapsed().as_secs_f32();
            if let Some((start, end)) = self.loop_region
                && elapsed >= end
            {
                elapsed = start + (elapsed - start) % (end - start);
            }
            (elapsed / self.duration).min(1.)
        }
    }
    impl Drop for Player {
        fn drop(&mut self) {
            unsafe {
                waveOutReset(self.device);
                for header in &mut self.headers {
                    waveOutUnprepareHeader(
                        self.device,
                        header,
                        std::mem::size_of::<Header>() as u32,
                    );
                }
                waveOutClose(self.device);
            }
        }
    }
}
#[cfg(windows)]
pub use win::Player;
#[cfg(not(windows))]
pub struct Player;
#[cfg(not(windows))]
impl Player {
    pub fn start(_: Pcm) -> Result<Self, String> {
        Err("Audio audition is currently available on Windows.".into())
    }
    pub fn finished(&self) -> bool {
        true
    }
    pub fn progress(&self) -> f32 {
        0.
    }
    pub fn pause(&self, _: bool) {}
}
