//! Inspector playback of a cooked video, the way the console shows it: frames
//! decoded by the console's own decoder plus the MDEC model in a worker thread,
//! timed by the double-speed sector clock the CD delivers them on.
use crate::{assets, fmv, video_import};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    time::Instant,
};

pub struct Loaded {
    pub report: video_import::Report,
    /// When each frame has fully arrived, in seconds from the start.
    times: Vec<f32>,
    duration: f32,
    audio: Option<crate::preview_audio::Pcm>,
}
type Loading = mpsc::Receiver<Result<(Loaded, Arc<fmv::Stream>), String>>;
struct Worker {
    request: mpsc::Sender<usize>,
    frames: mpsc::Receiver<Result<(usize, Vec<u8>), String>>,
}

#[derive(Default)]
pub struct VideoPreview {
    path: Option<PathBuf>,
    loading: Option<Loading>,
    pub loaded: Option<Loaded>,
    worker: Option<Worker>,
    requested: Option<usize>,
    /// The frame on screen and its pixels waiting for upload.
    pub shown: Option<usize>,
    pixels: Option<Vec<u8>>,
    pub texture: Option<imgui::TextureId>,
    texture_size: [u32; 2],
    retired: Vec<imgui::TextureId>,
    pub playing: bool,
    started: Option<(Instant, f32)>,
    position: f32,
    player: Option<crate::preview_audio::Player>,
    pub error: Option<String>,
    /// Starts playing once loaded (screenshots of a running preview).
    pub autoplay: bool,
}

impl VideoPreview {
    /// Shows `path` (a Video package); a different asset starts over.
    pub fn select(&mut self, root: &Path, path: &Path) {
        if self.path.as_deref() == Some(path) {
            return;
        }
        self.reset();
        self.path = Some(path.to_owned());
        let (root, package) = (root.to_owned(), path.to_owned());
        let (send, receive) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let _ = send.send(load(&root, &package));
        });
        self.loading = Some(receive);
    }
    pub fn reset(&mut self) {
        self.stop_audio();
        if let Some(texture) = self.texture.take() {
            self.retired.push(texture);
        }
        let retired = std::mem::take(&mut self.retired);
        *self = Self {
            retired,
            autoplay: self.autoplay,
            ..Default::default()
        };
    }
    pub fn loading(&self) -> bool {
        self.loading.is_some()
    }
    pub fn duration(&self) -> f32 {
        self.loaded.as_ref().map_or(0., |l| l.duration)
    }
    pub fn position(&self) -> f32 {
        self.position
    }
    pub fn frames(&self) -> usize {
        self.loaded.as_ref().map_or(0, |l| l.times.len())
    }
    pub fn has_audio(&self) -> bool {
        self.loaded.as_ref().is_some_and(|l| l.audio.is_some())
    }
    fn stop_audio(&mut self) {
        self.player = None;
    }
    pub fn toggle(&mut self) {
        if self.playing {
            self.playing = false;
            self.started = None;
            self.stop_audio();
            return;
        }
        if self.position >= self.duration() {
            self.position = 0.;
        }
        self.playing = true;
        self.started = Some((Instant::now(), self.position));
        // Audio only from the start: the preview player has no seek.
        if self.position == 0.
            && let Some(audio) = self.loaded.as_ref().and_then(|l| l.audio.clone())
        {
            self.player = crate::preview_audio::Player::start(audio).ok();
        }
    }
    pub fn seek(&mut self, seconds: f32) {
        self.position = seconds.clamp(0., self.duration());
        self.stop_audio();
        if self.playing {
            self.started = Some((Instant::now(), self.position));
        }
    }
    /// Advances playback and asks the worker for the frame due now.
    pub fn tick(&mut self) {
        if let Some(receive) = &self.loading {
            match receive.try_recv() {
                Ok(Ok((loaded, stream))) => {
                    self.loading = None;
                    self.worker = Some(spawn(stream, loaded.report.tv_levels));
                    self.loaded = Some(loaded);
                    if std::mem::take(&mut self.autoplay) {
                        self.toggle();
                    }
                }
                Ok(Err(error)) => {
                    self.loading = None;
                    self.error = Some(error);
                }
                Err(mpsc::TryRecvError::Empty) => return,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.loading = None;
                    self.error = Some("Video preview stopped".into());
                }
            }
        }
        let Some(duration) = self.loaded.as_ref().map(|l| l.duration) else {
            return;
        };
        if let Some((start, from)) = self.started {
            self.position = match &self.player {
                Some(player) if !player.finished() => player.progress() * duration,
                _ => from + start.elapsed().as_secs_f32(),
            };
            if self.position >= duration {
                self.position = duration;
                self.playing = false;
                self.started = None;
                self.stop_audio();
            }
        }
        let Some(loaded) = &self.loaded else {
            return;
        };
        // The newest frame that has fully arrived by now, as on the console.
        let due = loaded
            .times
            .partition_point(|t| *t <= self.position)
            .saturating_sub(1);
        let Some(worker) = &self.worker else {
            return;
        };
        if self.requested != Some(due) && self.shown != Some(due) {
            // One request in flight keeps a slow debug decoder from queueing.
            if self.requested.is_none() {
                let _ = worker.request.send(due);
                self.requested = Some(due);
            }
        }
        while let Ok(result) = worker.frames.try_recv() {
            self.requested = None;
            match result {
                Ok((index, rgba)) => {
                    self.shown = Some(index);
                    self.pixels = Some(rgba);
                }
                Err(error) => self.error = Some(error),
            }
        }
    }
    pub fn picture_size(&self) -> Option<[u32; 2]> {
        let info = &self.loaded.as_ref()?.report.info;
        Some([info.width, info.height])
    }
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut imgui_wgpu::Renderer,
    ) {
        for id in self.retired.drain(..) {
            renderer.textures.remove(id);
        }
        let (Some(pixels), Some(size)) = (self.pixels.take(), self.picture_size()) else {
            return;
        };
        if self.texture.is_none() || self.texture_size != size {
            if let Some(id) = self.texture.take() {
                renderer.textures.remove(id);
            }
            let texture = imgui_wgpu::Texture::new(
                device,
                renderer,
                imgui_wgpu::TextureConfig {
                    size: wgpu::Extent3d {
                        width: size[0],
                        height: size[1],
                        depth_or_array_layers: 1,
                    },
                    format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                    sampler_desc: wgpu::SamplerDescriptor {
                        mag_filter: wgpu::FilterMode::Nearest,
                        min_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            self.texture = Some(renderer.textures.insert(texture));
            self.texture_size = size;
        }
        if let Some(texture) = self.texture.and_then(|id| renderer.textures.get(id)) {
            texture.write(queue, &pixels, size[0], size[1]);
        }
    }
}

fn load(root: &Path, path: &Path) -> Result<(Loaded, Arc<fmv::Stream>), String> {
    let package = assets::Package::load(path)?;
    let cooked = video_import::cook(root, &package, || false)?;
    let stream = fmv::parse(&cooked.stream)?;
    let times = stream
        .frames
        .iter()
        .map(|frame| {
            let chunks = frame.data.len().div_ceil(fmv::CHUNK).max(1) as f32;
            (frame.sector as f32 + chunks) / fmv::SECTORS_PER_SECOND as f32
        })
        .collect::<Vec<_>>();
    let duration = stream.info.seconds();
    let audio = fmv::audio_pcm(&cooked.stream).map(|(samples, format)| crate::preview_audio::Pcm {
        samples,
        rate: format.rate,
        channels: format.channels as u16,
        loop_region: None,
        report: None,
        timeline: Some(crate::preview_audio::Timeline {
            duration,
            loop_region: None,
        }),
    });
    Ok((
        Loaded {
            report: cooked.report,
            times,
            duration,
            audio,
        },
        Arc::new(stream),
    ))
}

fn spawn(stream: Arc<fmv::Stream>, tv_levels: bool) -> Worker {
    let (request, requests) = mpsc::channel::<usize>();
    let (send, frames) = mpsc::channel();
    std::thread::spawn(move || {
        let mdec = fmv::Mdec::new(tv_levels);
        let (width, height) = (stream.info.width, stream.info.height);
        while let Ok(mut index) = requests.recv() {
            // Only the newest request matters.
            while let Ok(newer) = requests.try_recv() {
                index = newer;
            }
            let Some(frame) = stream.frames.get(index) else {
                continue;
            };
            let result = fmv::decode(&frame.data, width, height, tv_levels)
                .and_then(|words| mdec.picture(&words, width, height))
                .map(|picture| (index, picture.rgba));
            if send.send(result).is_err() {
                break;
            }
        }
    });
    Worker { request, frames }
}

#[cfg(test)]
mod tests {
    #[test]
    fn worker_decodes_requested_frames_in_order_of_arrival() {
        let stream = std::sync::Arc::new(
            crate::fmv::parse(include_bytes!("../tests/fixtures/fmv-pattern-v3.str")).unwrap(),
        );
        let worker = super::spawn(stream, false);
        worker.request.send(2).unwrap();
        let (index, rgba) = worker.frames.recv().unwrap().unwrap();
        assert_eq!((index, rgba.len()), (2, 64 * 48 * 4));
        worker.request.send(99).unwrap();
        worker.request.send(0).unwrap();
        // Out-of-range requests are skipped, not answered with an error.
        assert_eq!(worker.frames.recv().unwrap().unwrap().0, 0);
    }
}
