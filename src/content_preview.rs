//! Bounded, lazy previews. Decode on workers; upload and audition on the editor thread.
use crate::{
    assets,
    preview_audio::{Pcm, Player},
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Instant, SystemTime},
};

const LIMIT: usize = 96;
const EDGE: u32 = 192;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum PreviewMode {
    #[default]
    Source,
    TargetPsx,
}

struct AudioLoad {
    path: PathBuf,
    stamp: Option<Stamp>,
    generation: u64,
    mode: PreviewMode,
    receive: mpsc::Receiver<Result<Pcm, String>>,
    cancelled: Arc<AtomicBool>,
}

/// Both browser thumbnails and Inspector audition share this identity. Using
/// the whole index in one view would evict the other's preview every frame.
pub fn revision<'a>(root: &Path, index: &'a assets::Index, path: &Path) -> &'a str {
    index
        .assets
        .values()
        .flatten()
        .find(|r| r.path == path)
        .map(|r| r.revision.as_str())
        .or_else(|| {
            index
                .sources
                .get(&assets::path_string(root, path))
                .map(|s| s.hash.as_str())
        })
        .unwrap_or("")
}

pub fn source_kind(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("Texture"),
        "ttf" | "otf" => Some("Font"),
        "wav" | "mp3" | "flac" | "ogg" | "mid" | "midi" | "seq" | "sep" => Some("Audio"),
        "vab" | "vh" | "vb" | "sf2" | "sf3" => Some("SoundBank"),
        "fbx" | "obj" => Some("Mesh"),
        _ => None,
    }
}
/// Content dispatch within nominated source files. Signatures route malformed
/// Sony/MIDI data to their validating parser; they never constitute acceptance.
pub(crate) fn audio_content_kind(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"MThd") || bytes.starts_with(b"pQES") {
        Some("Sequence")
    } else if bytes.starts_with(b"pBAV") || crate::sf2::has_header(bytes) {
        Some("SoundBank")
    } else if crate::sequence::catalog_source(bytes, None).is_ok() {
        Some("Sequence") // No magic: converted must pass the complete structural parser.
    } else if crate::audio_decode::probe(bytes).is_ok() {
        Some("Audio")
    } else {
        None
    }
}
pub enum Preview {
    Image {
        rgba: Vec<u8>,
        width: u32,
        height: u32,
        texture: Option<imgui::TextureId>,
    },
    Audio {
        peaks: Vec<f32>,
        duration: f32,
    },
}
#[derive(Clone, PartialEq)]
struct Stamp {
    modified: Option<SystemTime>,
    length: u64,
    revision: String,
}
struct Entry {
    stamp: Stamp,
    checked: Instant,
    used: u64,
    preview: Result<Preview, String>,
}
struct Pending {
    path: PathBuf,
    stamp: Stamp,
    receive: mpsc::Receiver<Result<Preview, String>>,
    mode: PreviewMode,
    obsolete: bool,
}
#[derive(Default)]
pub struct Cache {
    edge: Option<u32>,
    limit: Option<usize>,
    entries: BTreeMap<PathBuf, Entry>,
    pending: Option<Pending>,
    retired: Vec<imgui::TextureId>,
    frame: u64,
    audition: Option<PathBuf>,
    audition_revision: Option<String>,
    audio_load: Option<AudioLoad>,
    generation: u64,
    mode: PreviewMode,
    player: Option<Player>,
    pub error: Option<String>,
    pub report: Option<String>,
    report_pending: bool,
    root: Option<PathBuf>,
    project_revision: String,
}
impl Cache {
    pub fn take_report(&mut self) -> Option<String> {
        if std::mem::take(&mut self.report_pending) {
            self.report.clone()
        } else {
            None
        }
    }
    pub fn configure(&mut self, root: &Path, index: &assets::Index) {
        let revision = format!(
            "{}:{:?}",
            index
                .assets
                .values()
                .flatten()
                .filter(|r| matches!(
                    r.meta.kind,
                    assets::Kind::AudioClip | assets::Kind::MusicSequence | assets::Kind::SoundBank
                ))
                .map(|r| format!("{}:{}:{}", r.meta.id, r.path.display(), r.revision))
                .collect::<Vec<_>>()
                .join(";"),
            crate::workspace::optional_manifest(root).map(|m| m.and_then(|m| m.default_sound_bank))
        );
        if self.root.as_deref() == Some(root) && self.project_revision == revision {
            return;
        }
        self.stop();
        self.root = Some(root.into());
        self.project_revision = revision;
        if let Some(pending) = &mut self.pending {
            pending.obsolete = true;
        }
        for path in self.entries.keys().cloned().collect::<Vec<_>>() {
            self.remove(&path);
        }
    }
    pub fn ensure_project(&mut self, root: &Path, index: &assets::Index) {
        if self.root.as_deref() != Some(root) {
            self.configure(root, index);
        }
    }
    pub fn mode(&self) -> PreviewMode {
        self.mode
    }
    pub fn set_mode(&mut self, mode: PreviewMode) {
        if self.mode == mode {
            return;
        }
        self.stop();
        self.mode = mode;
        self.error = None;
        for path in self.entries.keys().cloned().collect::<Vec<_>>() {
            self.remove(&path);
        }
    }
    pub fn mode_controls(&mut self, ui: &imgui::Ui) {
        let mut mode = self.mode;
        ui.radio_button("Source Preview", &mut mode, PreviewMode::Source);
        ui.same_line();
        ui.radio_button("Target Preview (PSX)", &mut mode, PreviewMode::TargetPsx);
        self.set_mode(mode);
    }
    pub fn inspector() -> Self {
        Self {
            edge: Some(1024),
            limit: Some(2),
            ..Default::default()
        }
    }
    pub fn request(&mut self, path: &Path, revision: &str) {
        if self.active(path) {
            if self
                .audition_revision
                .as_deref()
                .is_some_and(|old| old != revision)
            {
                self.stop();
            } else {
                self.audition_revision = Some(revision.into());
            }
        }
        if let Some(pending) = &mut self.pending
            && pending.path == path
            && pending.stamp.revision != revision
        {
            pending.obsolete = true;
        }
        if let Some(entry) = self.entries.get_mut(path) {
            entry.used = self.frame;
            if entry.stamp.revision == revision && entry.checked.elapsed().as_secs() < 2 {
                return;
            }
            entry.checked = Instant::now();
            if stamp(path, revision).as_ref() == Some(&entry.stamp) {
                return;
            }
            self.remove(path);
            if self.active(path) {
                self.stop();
            }
        }
        if self.pending.is_some() {
            return;
        }
        let Some(stamp) = stamp(path, revision) else {
            return;
        };
        let path = path.to_owned();
        let worker_path = path.clone();
        let (send, receive) = mpsc::sync_channel(1);
        let edge = self.edge.unwrap_or(EDGE);
        let mode = self.mode;
        std::thread::spawn(move || {
            let _ = send.send(decode_size_mode(&worker_path, edge, mode));
        });
        self.pending = Some(Pending {
            path,
            stamp,
            receive,
            mode,
            obsolete: false,
        });
    }
    pub fn get(&self, path: &Path) -> Option<&Preview> {
        self.entries.get(path)?.preview.as_ref().ok()
    }
    pub fn failure(&self, path: &Path) -> Option<&str> {
        self.entries
            .get(path)?
            .preview
            .as_ref()
            .err()
            .map(String::as_str)
    }
    fn remove(&mut self, path: &Path) {
        if let Some(Entry {
            preview: Ok(Preview::Image {
                texture: Some(id), ..
            }),
            ..
        }) = self.entries.remove(path)
        {
            self.retired.push(id);
        }
    }
    pub fn stop(&mut self) {
        if let Some(job) = &self.audio_load {
            job.cancelled.store(true, Ordering::Relaxed);
        }
        self.generation = self.generation.wrapping_add(1);
        self.audition = None;
        self.audition_revision = None;
        self.player = None;
    }
    pub fn active(&self, path: &Path) -> bool {
        self.audition.as_deref() == Some(path)
    }
    pub fn loading(&self, path: &Path) -> bool {
        self.active(path) && self.player.is_none()
    }
    pub fn progress(&self, path: &Path) -> f32 {
        if self.active(path) {
            self.player.as_ref().map_or(0., Player::progress)
        } else {
            0.
        }
    }
    pub fn toggle(&mut self, path: &Path) {
        if self.active(path) {
            self.stop();
            return;
        }
        self.stop();
        self.error = None;
        self.report = None;
        self.audition = Some(path.to_owned());
        self.audition_revision = self
            .entries
            .get(path)
            .map(|e| e.stamp.revision.clone())
            .or_else(|| {
                self.pending
                    .as_ref()
                    .filter(|p| p.path == path)
                    .map(|p| p.stamp.revision.clone())
            });
        // A previous decode finishes before starting the newest request, bounding memory.
        self.start_audio_load();
    }
    fn start_audio_load(&mut self) {
        if self.audio_load.is_some() || self.player.is_some() {
            return;
        }
        let Some(path) = self.audition.clone() else {
            return;
        };
        let (send, receive) = mpsc::sync_channel(1);
        let mode = self.mode;
        let worker_path = path.clone();
        let root = self.root.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = cancelled.clone();
        std::thread::spawn(move || {
            let _ = send.send(decode_audio_context(
                &worker_path,
                mode,
                root.as_deref(),
                &worker_cancelled,
            ));
        });
        self.audio_load = Some(AudioLoad {
            stamp: stamp(&path, ""),
            path,
            generation: self.generation,
            mode,
            receive,
            cancelled,
        });
    }
    pub fn poll(&mut self) {
        self.frame += 1;
        if let Some(result) = self
            .pending
            .as_ref()
            .and_then(|p| match p.receive.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Disconnected) => Some(Err("Preview worker stopped".into())),
                Err(mpsc::TryRecvError::Empty) => None,
            })
        {
            let pending = self.pending.take().unwrap();
            if !pending.obsolete
                && pending.mode == self.mode
                && stamp(&pending.path, &pending.stamp.revision).as_ref() == Some(&pending.stamp)
            {
                self.entries.insert(
                    pending.path,
                    Entry {
                        stamp: pending.stamp,
                        checked: Instant::now(),
                        used: self.frame,
                        preview: result,
                    },
                );
            }
        }
        while self.entries.len() > self.limit.unwrap_or(LIMIT) {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.used)
                .unwrap()
                .0
                .clone();
            self.remove(&oldest);
        }
        if let Some(result) =
            self.audio_load
                .as_ref()
                .and_then(|job| match job.receive.try_recv() {
                    Ok(result) => Some(result),
                    Err(mpsc::TryRecvError::Disconnected) => {
                        Some(Err("Audio preview worker stopped".into()))
                    }
                    Err(mpsc::TryRecvError::Empty) => None,
                })
        {
            let job = self.audio_load.take().unwrap();
            if self.active(&job.path) && job.generation == self.generation && job.mode == self.mode
            {
                let result = if stamp(&job.path, "") != job.stamp {
                    Err("Source changed during preview. Play again to use the new revision.".into())
                } else {
                    result
                };
                match result.and_then(|pcm| {
                    self.report = pcm.report.clone();
                    self.report_pending = true;
                    Player::start(pcm)
                }) {
                    Ok(player) => self.player = Some(player),
                    Err(error) => {
                        self.stop();
                        self.error = Some(format!(
                            "Audio preview failed for {}: {error}",
                            job.path.display()
                        ));
                    }
                }
            }
        }
        if self.player.as_ref().is_some_and(Player::finished) {
            self.stop();
        }
        self.start_audio_load();
    }
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut imgui_wgpu::Renderer,
    ) {
        self.poll();
        for id in self.retired.drain(..) {
            renderer.textures.remove(id);
        }
        for entry in self.entries.values_mut() {
            if let Ok(Preview::Image {
                rgba,
                width,
                height,
                texture,
            }) = &mut entry.preview
            {
                if texture.is_some() {
                    continue;
                }
                let image = imgui_wgpu::Texture::new(
                    device,
                    renderer,
                    imgui_wgpu::TextureConfig {
                        size: wgpu::Extent3d {
                            width: *width,
                            height: *height,
                            depth_or_array_layers: 1,
                        },
                        format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                        ..Default::default()
                    },
                );
                image.write(queue, rgba, *width, *height);
                *texture = Some(renderer.textures.insert(image));
                rgba.clear();
                rgba.shrink_to_fit();
            }
        }
    }
    pub fn clear(&mut self, renderer: &mut imgui_wgpu::Renderer) {
        self.stop();
        if let Some(pending) = &mut self.pending {
            pending.obsolete = true;
        }
        for path in self.entries.keys().cloned().collect::<Vec<_>>() {
            self.remove(&path);
        }
        for id in self.retired.drain(..) {
            renderer.textures.remove(id);
        }
    }
}
fn stamp(path: &Path, revision: &str) -> Option<Stamp> {
    let meta = path.metadata().ok()?;
    Some(Stamp {
        modified: meta.modified().ok(),
        length: meta.len(),
        revision: revision.into(),
    })
}
#[cfg(test)]
fn decode(path: &Path) -> Result<Preview, String> {
    decode_size(path, EDGE)
}
#[cfg(test)]
fn decode_size(path: &Path, edge: u32) -> Result<Preview, String> {
    decode_size_mode(path, edge, PreviewMode::Source)
}
fn decode_size_mode(path: &Path, edge: u32, mode: PreviewMode) -> Result<Preview, String> {
    if matches!(source_kind(path), Some("Audio" | "SoundBank")) {
        let bytes = assets::read_bounded(path)?;
        let kind = audio_content_kind(&bytes);
        if kind == Some("Sequence") {
            return sequence_plot(&bytes, &Default::default());
        }
        if kind == Some("SoundBank") || (kind.is_none() && source_kind(path) == Some("SoundBank")) {
            return Err("SoundBank source: inspect its programs/samples and explicitly select a VH/VB pair when needed. Waveform Hz and playback mapping are unresolved; no automatic audition.".into());
        }
        return waveform(decode_audio_mode(path, mode)?);
    }
    if path.extension().is_some_and(|e| e == "epokasset") {
        let package = assets::Package::load(path)?;
        match package.meta.kind {
            assets::Kind::AudioClip => waveform(decode_audio_mode(path, mode)?),
            assets::Kind::MusicSequence => {
                sequence_plot(&package.source, package.meta.settings.sequence()?)
            }
            assets::Kind::Texture => {
                let data = crate::texture::decode(&package.source)?;
                Ok(thumbnail(
                    data.rgba,
                    data.width as u32,
                    data.height as u32,
                    edge,
                ))
            }
            assets::Kind::Font => {
                let data =
                    crate::font_asset::decode(&package.source, package.meta.settings.font()?)?;
                Ok(thumbnail(
                    data.rgba,
                    u32::from(data.width),
                    u32::from(data.height),
                    edge,
                ))
            }
            _ => Err("No preview available for this asset type".into()),
        }
    } else {
        decode_png_size(&assets::read_bounded(path)?, edge)
    }
}
#[cfg(test)]
fn decode_png(bytes: &[u8]) -> Result<Preview, String> {
    decode_png_size(bytes, EDGE)
}
fn decode_png_size(bytes: &[u8], edge: u32) -> Result<Preview, String> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    decoder.set_limits(png::Limits {
        bytes: 64 * 1024 * 1024,
    });
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let info = reader.info();
    if info.width == 0
        || info.height == 0
        || info.width as u64 * info.height as u64 > 16 * 1024 * 1024
    {
        return Err("Image preview exceeds 16 megapixels".into());
    }
    let mut bytes = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut bytes).map_err(|e| e.to_string())?;
    let channels = info.color_type.samples();
    let mut rgba = Vec::with_capacity(info.width as usize * info.height as usize * 4);
    for p in bytes[..info.buffer_size()].chunks_exact(channels) {
        match info.color_type {
            png::ColorType::Rgba => rgba.extend_from_slice(p),
            png::ColorType::Rgb => rgba.extend_from_slice(&[p[0], p[1], p[2], 255]),
            png::ColorType::Grayscale => rgba.extend_from_slice(&[p[0], p[0], p[0], 255]),
            png::ColorType::GrayscaleAlpha => rgba.extend_from_slice(&[p[0], p[0], p[0], p[1]]),
            _ => return Err("Unsupported PNG preview format".into()),
        }
    }
    Ok(thumbnail(rgba, info.width, info.height, edge))
}
fn thumbnail(rgba: Vec<u8>, width: u32, height: u32, edge: u32) -> Preview {
    let scale = (edge as f32 / width.max(height) as f32).min(1.);
    let w = (width as f32 * scale).round().max(1.) as u32;
    let h = (height as f32 * scale).round().max(1.) as u32;
    let mut small = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let offset = (((y as u64 * height as u64 / h as u64) * width as u64
                + x as u64 * width as u64 / w as u64)
                * 4) as usize;
            small.extend_from_slice(&rgba[offset..offset + 4]);
        }
    }
    Preview::Image {
        rgba: small,
        width: w,
        height: h,
        texture: None,
    }
}
#[cfg(test)]
fn decode_audio(path: &Path) -> Result<Pcm, String> {
    decode_audio_mode(path, PreviewMode::Source)
}
fn decode_audio_mode(path: &Path, mode: PreviewMode) -> Result<Pcm, String> {
    decode_audio_context(path, mode, None, &AtomicBool::new(false))
}
pub(crate) fn decode_audio_context(
    path: &Path,
    mode: PreviewMode,
    root: Option<&Path>,
    cancelled: &AtomicBool,
) -> Result<Pcm, String> {
    if path.extension().is_some_and(|e| e == "epokasset") {
        let package = assets::Package::load(path)?;
        if package.meta.kind == assets::Kind::MusicSequence {
            return sequence_pcm(
                root,
                &package.source,
                package.meta.settings.sequence()?,
                mode,
                cancelled,
            );
        }
        if package.meta.kind != assets::Kind::AudioClip {
            return Err("Expected an AudioClip".into());
        }
        preview_pcm(&package.source, Some(package.meta.settings.audio()?), mode)
    } else {
        let bytes = assets::read_bounded(path)?;
        let kind = audio_content_kind(&bytes);
        if kind == Some("Sequence") {
            return sequence_pcm(root, &bytes, &Default::default(), mode, cancelled);
        }
        if kind == Some("SoundBank") || (kind.is_none() && source_kind(path) == Some("SoundBank")) {
            if crate::sf2::has_header(&bytes) {
                return Err("Instrument libraries contain samples and mappings. Import this SF2/SF3 as a SoundBank and select it for a MusicSequence to audition a song.".into());
            }
            return Err("Sony SoundBank playback is unavailable until source rate/tuning and instrument semantics are resolved. Inspect/import the bank; explicitly pair VH/VB. No waveform rate is assumed.".into());
        }
        preview_pcm(&bytes, None, mode)
    }
}
fn sequence_pcm(
    root: Option<&Path>,
    bytes: &[u8],
    settings: &crate::sequence::Settings,
    mode: PreviewMode,
    cancelled: &AtomicBool,
) -> Result<Pcm, String> {
    let root = root.ok_or("MIDI needs a SoundBank. Open a project and assign a SoundBank or Project Default SoundBank.")?;
    let (mut pcm, stats) = if mode == PreviewMode::TargetPsx {
        crate::sequence_preview::render_target(root, bytes, settings, cancelled)?
    } else {
        crate::sequence_preview::render(root, bytes, settings, cancelled)?
    };
    let pitch_policy = if settings.source_selection.is_some()
        || settings.midi_profile == crate::midi::MidiProfile::LegacyV1
    {
        "legacy ±2 semitones"
    } else {
        "source RPN sensitivity/tuning (default ±2 semitones)"
    };
    let report = format!(
        "Host audition: {} active voices peak / {} limit, {} steals, {} clipped channel samples. Pitch: {pitch_policy}. Loops cut tails and restore controllers; output loops are quantized to 44.1 kHz frames.",
        stats.peak,
        settings.voices(),
        stats.steals,
        stats.clipped
    );
    pcm.report = Some(
        pcm.report
            .map_or_else(|| report.clone(), |target| format!("{target} {report}")),
    );
    Ok(pcm)
}
fn sequence_plot(bytes: &[u8], settings: &crate::sequence::Settings) -> Result<Preview, String> {
    let ir = crate::sequence::decode_source(bytes, settings)?;
    let mut peaks = vec![0f32; 128];
    for e in &ir.events {
        if let crate::sequence_ir::EventKind::NoteOn { velocity, .. } = e.kind {
            let at =
                (ir.micros_at(e.tick) as u128 * 127 / ir.duration_micros.max(1) as u128) as usize;
            peaks[at] = peaks[at].max(velocity as f32 / 127.);
        }
    }
    Ok(Preview::Audio {
        peaks,
        duration: ir.duration_micros as f32 / 1_000_000.,
    })
}
fn preview_pcm(
    bytes: &[u8],
    settings: Option<&crate::audio_import::Settings>,
    mode: PreviewMode,
) -> Result<Pcm, String> {
    if bytes.get(..4) == Some(b"MThd") {
        crate::midi::parse(bytes)?;
        return Err("MIDI contains notes, not instruments. Assign a SoundBank to its MusicSequence or select a Project Default SoundBank to audition it.".into());
    }
    if mode == PreviewMode::Source {
        return audio_pcm(bytes, settings);
    }
    let defaults = crate::audio_import::Settings::default();
    let settings = settings.unwrap_or(&defaults);
    settings.validate_psx()?;
    if settings.is_streamed() {
        return Err("Target Preview cannot decode PSX XA yet. Select Source Preview to audition the original PCM.".into());
    }
    let cooked = crate::audio_import::convert(bytes, settings)?;
    Ok(Pcm {
        samples: crate::audio_import::decode_adpcm(&cooked.adpcm)?,
        channels: 1,
        rate: settings.rate(),
        loop_region: settings
            .looping
            .then_some((28, (cooked.adpcm.len() / 16 - 1) * 28)),
        report: None,
        timeline: None,
    })
}
// Audition the embedded import snapshot and its trim/channel/gain settings, before PSX encoding.
pub(crate) fn audio_pcm(
    bytes: &[u8],
    settings: Option<&crate::audio_import::Settings>,
) -> Result<Pcm, String> {
    let ir = crate::audio_ir::DecodedAudioIr::decode(bytes)?;
    let info = ir.info;
    let samples = ir.samples;
    let (start, end, channels, rate, normalize) = if let Some(s) = settings {
        s.validate()?;
        let (start, end) = crate::audio_import::trim_range(&info, s)?;
        (start, end, s.channels, info.sample_rate, s.normalize)
    } else {
        (0, info.frames, info.channels, info.sample_rate, false)
    };
    let frames = ((end - start) as u64 * rate as u64)
        .div_ceil(info.sample_rate as u64)
        .max(1) as usize;
    if frames * channels as usize > 52_920_000 {
        return Err("Audio preview is too long".into());
    }
    let source = &samples[start * info.channels as usize..end * info.channels as usize];
    let value = |frame: usize, channel: usize| {
        let at = frame as f64 * info.sample_rate as f64 / rate as f64;
        let a = (at as usize).min(end - start - 1);
        let b = (a + 1).min(end - start - 1);
        let sample = |index: usize| {
            let offset = index * info.channels as usize;
            if channels == 1 && info.channels == 2 {
                (source[offset] + source[offset + 1]) * 0.5
            } else {
                source[offset + channel.min(info.channels as usize - 1)]
            }
        };
        sample(a) + (sample(b) - sample(a)) * (at - at.floor()) as f32
    };
    // Normalize after channel conversion/resampling, matching the imported clip's gain.
    let mut peak = 0f32;
    if normalize {
        for frame in 0..frames {
            for channel in 0..channels as usize {
                peak = peak.max(value(frame, channel).abs());
            }
        }
    }
    let gain = if peak > 0. { 0.95 / peak } else { 1. };
    let mut pcm = Vec::with_capacity(frames * channels as usize);
    for frame in 0..frames {
        for channel in 0..channels as usize {
            pcm.push(
                (value(frame, channel) * gain * 32767.)
                    .round()
                    .clamp(-32768., 32767.) as i16,
            );
        }
    }
    Ok(Pcm {
        samples: pcm,
        channels,
        rate,
        loop_region: settings.is_some_and(|s| s.looping).then_some((0, frames)),
        report: None,
        timeline: None,
    })
}
fn waveform(pcm: Pcm) -> Result<Preview, String> {
    let peaks = pcm
        .samples
        .chunks(pcm.samples.len().div_ceil(128).max(1))
        .map(|chunk| {
            chunk
                .iter()
                .map(|s| (*s as f32 / 32768.).abs())
                .fold(0f32, f32::max)
        })
        .collect();
    Ok(Preview::Audio {
        peaks,
        duration: pcm.duration(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audio_source_target_preview_and_late_worker_identity() {
        let bytes = crate::audio_import::test_wav();
        let source = preview_pcm(&bytes, None, PreviewMode::Source).unwrap();
        let target = preview_pcm(&bytes, None, PreviewMode::TargetPsx).unwrap();
        let looped = preview_pcm(
            &bytes,
            Some(&crate::audio_import::Settings {
                looping: true,
                ..Default::default()
            }),
            PreviewMode::TargetPsx,
        )
        .unwrap();
        assert_eq!(looped.loop_region, Some((28, looped.samples.len())));
        assert_eq!(source.samples.len(), 2205);
        assert!(target.samples.len() > source.samples.len()); // SPU silent entry / block padding.
        assert!(target.samples.iter().any(|v| v.abs() > 1000));
        let settings = crate::audio_import::Settings {
            load_mode: crate::audio_import::LoadMode::Stream,
            sample_rate: 37800,
            ..Default::default()
        };
        assert!(preview_pcm(&bytes, Some(&settings), PreviewMode::Source).is_ok());
        assert!(
            preview_pcm(&bytes, Some(&settings), PreviewMode::TargetPsx)
                .unwrap_err()
                .contains("XA")
        );
        assert!(
            preview_pcm(&crate::midi::fixture(), None, PreviewMode::Source)
                .unwrap_err()
                .contains("SoundBank")
        );
        let mut cache = Cache::default();
        let path = PathBuf::from("missing.wav");
        let (send, receive) = mpsc::sync_channel(1);
        cache.audition = Some(path.clone());
        cache.audio_load = Some(AudioLoad {
            path: path.clone(),
            stamp: None,
            generation: 0,
            mode: PreviewMode::Source,
            receive,
            cancelled: Arc::new(AtomicBool::new(false)),
        });
        cache.stop();
        cache.audition = Some(path.clone()); // Same path, new audition, while the previous worker retires.
        send.send(Err("stale decode error".into())).unwrap();
        cache.poll();
        assert!(
            cache.error.is_none(),
            "Late results must not affect a new audition of the same path"
        );
        cache.set_mode(PreviewMode::TargetPsx);
        settle(&mut cache);
        assert!(cache.player.is_none());
        assert!(!cache.active(&path));
    }

    #[test]
    fn audio_all_sampled_formats_preview_before_and_after_import_without_cooking() {
        let root = crate::workspace::tests::temp("portable-audio-preview");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        for (ext, bytes) in [
            ("wav", crate::audio_import::test_wav()),
            (
                "mp3",
                include_bytes!("../tests/fixtures/stereo-tone.mp3").to_vec(),
            ),
            (
                "flac",
                include_bytes!("../tests/fixtures/portable-tone.flac").to_vec(),
            ),
            (
                "ogg",
                include_bytes!("../tests/fixtures/portable-tone.ogg").to_vec(),
            ),
        ] {
            let name = format!("assets/tone.{ext}");
            std::fs::write(root.join(&name), &bytes).unwrap();
            let source = decode_audio(&root.join(&name)).unwrap();
            assert!(source.samples.iter().any(|v| v.abs() > 500));
            for mode in [
                crate::audio_import::LoadMode::Resident,
                crate::audio_import::LoadMode::Stream,
            ] {
                let destination = format!("assets/{ext}-{mode:?}.epokasset");
                assets::commit(
                    assets::prepare(
                        &root,
                        &name,
                        &destination,
                        crate::audio_import::Settings {
                            load_mode: mode,
                            ..Default::default()
                        },
                        None,
                        false,
                    )
                    .unwrap(),
                )
                .unwrap();
                let pcm = decode_audio(&root.join(&destination)).unwrap();
                assert!(pcm.samples.iter().any(|v| v.abs() > 500));
                assert_eq!(pcm.rate, source.rate);
            }
        }
        for ext in ["mid", "midi"] {
            let path = root.join(format!("assets/sequence.{ext}"));
            std::fs::write(&path, crate::midi::fixture()).unwrap();
            assert_eq!(source_kind(&path), Some("Audio"));
            assert!(decode_audio(&path).unwrap_err().contains("SoundBank"));
        }
        let index = assets::scan(&root, &mut Default::default());
        assert_eq!(index.sources.len(), 6);
        assert_eq!(index.usable().count(), 8);
        assert!(
            !root.join(".epok/imported").exists(),
            "Detection, import and source preview never request PSX cooking"
        );
    }
    #[test]
    fn audio_late_waveform_and_audition_are_rejected_on_revision_change() {
        let root = crate::workspace::tests::temp("audio-stale-revision");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("tone.wav");
        std::fs::write(&path, crate::audio_import::test_wav()).unwrap();
        let (send, receive) = mpsc::sync_channel(1);
        let mut cache = Cache {
            pending: Some(Pending {
                path: path.clone(),
                stamp: stamp(&path, "old").unwrap(),
                mode: PreviewMode::Source,
                obsolete: false,
                receive,
            }),
            audition: Some(path.clone()),
            audition_revision: Some("old".into()),
            ..Default::default()
        };
        cache.request(&path, "new"); // A Refresh finds a timestamp-preserving edit.
        send.send(Err("old waveform".into())).ok().unwrap();
        cache.poll();
        assert!(!cache.active(&path));
        assert!(cache.get(&path).is_none());
        assert!(cache.failure(&path).is_none());
    }
    #[test]
    fn midi_default_bank_and_sample_revisions_cancel_pending_audition() {
        let root = crate::workspace::tests::temp("midi-default-bank");
        let (sample, bank) = crate::sequence_preview::tests::fixture(&root);
        let descriptor = root.join("Preview.epokproject");
        std::fs::write(&descriptor, serde_json::to_vec(&serde_json::json!({
            "format_version":1, "editor_version":env!("CARGO_PKG_VERSION"), "name":"Preview", "startup_scene":"assets/scenes/Main.epokmap", "auto_build":false,
            "default_sound_bank":bank
        })).unwrap()).unwrap();
        let path = root.join("assets/song.mid");
        std::fs::write(&path, crate::midi::fixture()).unwrap();
        let pcm = decode_audio_context(
            &path,
            PreviewMode::Source,
            Some(&root),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(pcm.samples.iter().any(|s| s.abs() > 1000));
        assert!(pcm.report.unwrap().contains("voices peak"));
        let index = assets::scan(&root, &mut Default::default());
        assert!(
            assets::dependencies(&root, bank)
                .unwrap()
                .contains(&"Project Default SoundBank".into())
        );
        let mut cache = Cache::default();
        cache.configure(&root, &index);
        cache.toggle(&path);
        let cancelled = cache.audio_load.as_ref().unwrap().cancelled.clone();
        let record = index.resolve(sample).unwrap();
        let mut settings = record.meta.settings.audio().unwrap().clone();
        settings.normalize = !settings.normalize;
        assets::commit(
            assets::prepare(
                &root,
                "",
                "assets/tone.epokasset",
                settings,
                Some(record),
                true,
            )
            .unwrap(),
        )
        .unwrap();
        cache.configure(&root, &assets::scan(&root, &mut Default::default()));
        assert!(cancelled.load(Ordering::Relaxed));
        assert!(!cache.active(&path));
        settle(&mut cache);
        assert!(cache.player.is_none());
        let before = cache.project_revision.clone();
        let mut manifest = crate::workspace::read_manifest(&root).unwrap();
        manifest.default_sound_bank = None;
        crate::workspace::save_manifest(&root, &manifest).unwrap();
        cache.configure(&root, &index);
        assert_ne!(cache.project_revision, before);
        assert!(
            decode_audio_context(
                &path,
                PreviewMode::Source,
                Some(&root),
                &AtomicBool::new(false)
            )
            .unwrap_err()
            .contains("SoundBank")
        );
    }
    #[test]
    fn compatibility_preview_uses_selection_and_common_host_target_paths() {
        let root = crate::workspace::tests::temp("compatibility-sequence-preview");
        let (_, bank) = crate::sequence_preview::tests::fixture(&root);
        let mut source = b"pQES\0\0\0\x01\0\x60\x07\xa1\x20\x04\x02".to_vec();
        source.extend([0, 0x90, 60, 100, 96, 0x80, 60, 0, 0, 0xff, 0x2f, 0]);
        assert!(sequence_plot(&source, &Default::default()).is_err());
        let settings = crate::sequence::Settings {
            sound_bank: Some(bank),
            source_selection: Some(
                crate::sequence::select_source(
                    &source,
                    crate::sequence::SourceProfile::SonySeqV1,
                    Some(0),
                    None,
                )
                .unwrap(),
            ),
            ..Default::default()
        };
        let Preview::Audio { peaks, duration } = sequence_plot(&source, &settings).unwrap() else {
            panic!()
        };
        assert_eq!(duration, 0.5);
        assert!(peaks.iter().any(|&peak| peak > 0.));
        for mode in [PreviewMode::Source, PreviewMode::TargetPsx] {
            let pcm = sequence_pcm(
                Some(&root),
                &source,
                &settings,
                mode,
                &AtomicBool::new(false),
            )
            .unwrap();
            assert!(pcm.samples.iter().any(|&s| s.unsigned_abs() > 500));
        }
        let mut blocked = source[..15].to_vec();
        blocked.extend([0, 0xb0, 0, 3, 0, 0xff, 0x2f, 0]);
        let mut settings = settings;
        settings.ignore_unsupported = true;
        for mode in [PreviewMode::Source, PreviewMode::TargetPsx] {
            assert!(
                sequence_pcm(
                    Some(&root),
                    &blocked,
                    &settings,
                    mode,
                    &AtomicBool::new(false)
                )
                .unwrap_err()
                .contains("cannot bypass")
            );
        }
    }
    #[test]
    fn raw_bank_sources_do_not_fabricate_a_waveform_rate_or_audition() {
        let root = crate::workspace::tests::temp("compatibility-bank-no-fallback");
        std::fs::create_dir_all(&root).unwrap();
        for extension in ["vab", "vh", "vb"] {
            let path = root.join(format!("source.{extension}"));
            std::fs::write(&path, b"unresolved bank source").unwrap();
            assert_eq!(source_kind(&path), Some("SoundBank"));
            assert!(
                decode_audio_context(
                    &path,
                    PreviewMode::Source,
                    Some(&root),
                    &AtomicBool::new(false)
                )
                .unwrap_err()
                .contains("No waveform rate is assumed")
            );
        }
    }
    #[test]
    fn compatibility_dispatch_follows_content_across_wrong_extensions() {
        let root = crate::workspace::tests::temp("compatibility-wrong-extension");
        std::fs::create_dir_all(&root).unwrap();
        let mut seq = b"pQES\0\0\0\x01\0\x60\x07\xa1\x20\x04\x02".to_vec();
        seq.extend([0, 0xff, 0x2f, 0]);
        let wrong_bank = root.join("song.vab");
        std::fs::write(&wrong_bank, &seq).unwrap();
        assert_eq!(audio_content_kind(&seq), Some("Sequence"));
        assert!(
            matches!(decode_size_mode(&wrong_bank, EDGE, PreviewMode::Source), Err(e) if e.contains("Select the source profile"))
        );
        let mut vab = vec![0_u8; 0xc30];
        vab[..4].copy_from_slice(b"pBAV");
        vab[4..8].copy_from_slice(&7_u32.to_le_bytes());
        vab[12..16].copy_from_slice(&0xc30_u32.to_le_bytes());
        for at in [18, 20, 22] {
            vab[at] = 1;
        }
        vab[32] = 1;
        vab[0x824] = 60;
        vab[0x827] = 127;
        vab[0x836] = 1;
        vab[0xa22] = 2;
        vab[0xc20] = 12;
        vab[0xc21] = 1;
        assert!(
            crate::vab_import::parse(
                crate::vab_import::Input {
                    bytes: &vab,
                    label: "original synthetic bank",
                    rights: "Epok test fixture"
                },
                None
            )
            .is_ok()
        );
        let wrong_sequence = root.join("bank.sep");
        std::fs::write(&wrong_sequence, &vab).unwrap();
        assert_eq!(audio_content_kind(&vab), Some("SoundBank"));
        assert!(
            matches!(decode_audio_context(&wrong_sequence, PreviewMode::Source, Some(&root), &AtomicBool::new(false)), Err(e) if e.contains("No waveform rate is assumed"))
        );
        let mut converted_seq = 16_u32.to_le_bytes().to_vec();
        converted_seq.extend(500000_u32.to_le_bytes());
        converted_seq.extend([96, 0, 4, 2, 0xff, 0x2f, 0, 0]);
        assert_eq!(audio_content_kind(&converted_seq), Some("Sequence"));
        let path = root.join("converted.seq");
        std::fs::write(&path, converted_seq).unwrap();
        assert!(
            matches!(decode_size_mode(&path, EDGE, PreviewMode::Source), Err(e) if e.contains("converted-seq-le32"))
        );
        assert_eq!(
            audio_content_kind(&crate::audio_import::test_wav()),
            Some("Audio")
        );
    }
    fn png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(rgba)
                .unwrap();
        }
        bytes
    }
    #[test]
    fn png_preview_preserves_aspect_alpha_and_limits_size() {
        let mut pixels = vec![0; 600 * 300 * 4];
        for p in pixels.chunks_exact_mut(4) {
            p.copy_from_slice(&[255, 80, 40, 128]);
        }
        let Preview::Image {
            width,
            height,
            rgba,
            ..
        } = decode_png(&png(600, 300, &pixels)).unwrap()
        else {
            panic!()
        };
        assert_eq!((width, height), (192, 96));
        assert_eq!(&rgba[..4], &[255, 80, 40, 128]);
        let Preview::Image { width, height, .. } =
            decode_png_size(&png(600, 300, &pixels), 1024).unwrap()
        else {
            panic!()
        };
        assert_eq!(
            (width, height),
            (600, 300),
            "Inspector previews preserve more detail than browser thumbnails"
        );
        assert!(decode_png(b"bad PNG").is_err());
        let mut bytes = Vec::new();
        png::Encoder::new(&mut bytes, 100_000, 100_000)
            .write_header()
            .unwrap();
        assert!(decode_png(&bytes).is_err());
    }
    #[test]
    fn imported_audio_uses_snapshot_and_saved_trim() {
        let root = std::env::temp_dir().join(format!("epok-preview-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(
            root.join("assets/tone.wav"),
            crate::audio_import::test_wav(),
        )
        .unwrap();
        let settings = crate::audio_import::Settings {
            trim_start: 0.02,
            trim_end: Some(0.07),
            normalize: true,
            ..Default::default()
        };
        assets::commit(
            assets::prepare(
                &root,
                "assets/tone.wav",
                "assets/tone.epokasset",
                settings,
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let index = assets::scan(&root, &mut Default::default());
        for name in ["assets/tone.wav", "assets/tone.epokasset"] {
            let path = root.join(name);
            let signature = revision(&root, &index, &path);
            assert!(!signature.is_empty());
            let mut cache = Cache::default();
            cache.request(&path, signature);
            settle(&mut cache);
            cache.audition = Some(path.clone());
            // Inspector and browser can request the same waveform every frame,
            // even while an unrelated asset changes the overall index revision.
            let mut changed = index.clone();
            changed.sources.clear();
            for _ in 0..10 {
                cache.request(&path, revision(&root, &index, &path));
                if name.ends_with("epokasset") {
                    cache.request(&path, revision(&root, &changed, &path));
                }
                assert!(cache.pending.is_none());
                assert!(cache.get(&path).is_some());
                assert!(cache.active(&path));
            }
            cache.stop();
        }
        std::fs::remove_file(root.join("assets/tone.wav")).unwrap();
        let pcm = decode_audio(&root.join("assets/tone.epokasset")).unwrap();
        assert!((pcm.duration() - 0.05).abs() < 0.001);
        assert_eq!(pcm.channels, 1);
        assert_eq!(pcm.rate, 22050);
        let Preview::Audio { peaks, duration } =
            decode(&root.join("assets/tone.epokasset")).unwrap()
        else {
            panic!()
        };
        assert!((duration - 0.05).abs() < 0.001);
        assert!(peaks.len() <= 128 && peaks.iter().any(|p| *p > 0.9));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn source_audio_keeps_stereo_and_corruption_falls_back() {
        let pcm = audio_pcm(include_bytes!("../tests/fixtures/stereo-tone.mp3"), None).unwrap();
        assert_eq!(pcm.channels, 2);
        assert!(pcm.samples.chunks_exact(2).any(|p| p[0] != p[1]));
        let settings = crate::audio_import::Settings {
            normalize: true,
            ..Default::default()
        };
        let mono = audio_pcm(
            include_bytes!("../tests/fixtures/stereo-tone.mp3"),
            Some(&settings),
        )
        .unwrap();
        assert_eq!(mono.channels, 1);
        let peak = mono
            .samples
            .iter()
            .map(|s| (*s as f32).abs() / 32767.)
            .fold(0f32, f32::max);
        assert!(
            (peak - 0.95).abs() < 0.0001,
            "Normalization follows the stereo-to-mono conversion"
        );
        assert!(audio_pcm(b"broken audio", None).is_err());
    }
    fn settle(cache: &mut Cache) {
        let start = Instant::now();
        while cache.pending.is_some() || cache.audio_load.is_some() {
            assert!(start.elapsed().as_secs() < 10);
            cache.poll();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
    #[test]
    fn cache_reimport_retry_eviction_and_cancelled_playback() {
        let root =
            std::env::temp_dir().join(format!("epok-preview-cache-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("picture.png");
        std::fs::write(&path, b"broken").unwrap();
        let mut cache = Cache::default();
        cache.request(&path, "first");
        settle(&mut cache);
        assert!(cache.failure(&path).is_some());
        std::fs::write(&path, png(1, 1, &[1, 2, 3, 255])).unwrap();
        cache.request(&path, "second");
        settle(&mut cache);
        assert!(matches!(cache.get(&path), Some(Preview::Image { .. })));
        for i in 0..LIMIT + 3 {
            cache.entries.insert(
                root.join(i.to_string()),
                Entry {
                    stamp: stamp(&path, "").unwrap(),
                    checked: Instant::now(),
                    used: i as u64,
                    preview: Err("unsupported".into()),
                },
            );
        }
        cache.poll();
        assert_eq!(cache.entries.len(), LIMIT);
        let audio = root.join("tone.wav");
        std::fs::write(&audio, crate::audio_import::test_wav()).unwrap();
        cache.toggle(&audio);
        assert!(cache.loading(&audio));
        cache.toggle(&audio);
        settle(&mut cache);
        assert!(!cache.active(&audio));
        assert!(cache.player.is_none());
        // A late result cannot start playing after another path was requested and cancelled.
        cache.toggle(&audio);
        cache.toggle(&root.join("other.wav"));
        cache.stop();
        settle(&mut cache);
        assert!(cache.player.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[cfg(windows)]
    #[ignore = "Plays a short quiet tone through the default Windows audio output"]
    fn native_audio_output_completes_and_can_stop_early() {
        for stop_early in [false, true] {
            let pcm = Pcm {
                channels: 2,
                rate: 44100,
                loop_region: None,
                report: None,
                timeline: None,
                samples: (0..8820)
                    .flat_map(|i| {
                        let v = ((i as f32 * 440. * std::f32::consts::TAU / 44100.).sin() * 600.)
                            as i16;
                        [v, v]
                    })
                    .collect(),
            };
            let player = Player::start(pcm).unwrap();
            if !stop_early {
                let start = Instant::now();
                while !player.finished() {
                    assert!(start.elapsed().as_secs() < 5);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                assert!(player.progress() > 0.8);
            }
            drop(player);
        }
    }
}
