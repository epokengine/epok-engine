//! Native PC Play transport. The generated C++ gameplay runs out of process;
//! only bounded actor/HUD snapshots cross back into the editor renderer.
use crate::{hud_native, scene::Scene};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant},
};

const MAGIC: u32 = 0x344e_5045; // "EPN4"
const ACTOR_WORDS: usize = 16;
const AUDIO_WORDS: usize = 6;
const AUDIO_LIMIT: u32 = 256;
fn next_rumble_session() -> u64 {
    static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);
    NEXT_SESSION.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Debug)]
struct ActorState {
    alive: bool,
    active: bool,
    parent: i32,
    position: [i32; 3],
    rotation: [i32; 3],
    scale: [i32; 3],
    fov: i32,
    mirror_x: bool,
    animation_ticks: u32,
    animation_clip: i32,
    animation_enabled: bool,
    animation_looping: bool,
}

#[derive(Clone, Copy, Debug)]
struct AudioEvent {
    source: u64,
    play: bool,
    clip: i32,
    volume: i32,
    pitch: i32,
}

#[derive(Clone)]
pub struct Frame {
    pub scene: Scene,
    pub number: u64,
    pub scene_bank: u32,
    pub scene_generation: u32,
    pub transition_phase: u32,
    pub buttons: u16,
    pub actor_count: u32,
    pub camera: Option<usize>,
    pub hud_rgba: Vec<u8>,
    pub hud_size: [u32; 2],
    /// 2D layers drawn under or inside the 3D image, each `hud_size`.
    pub screen_layers: Vec<hud_native::ScreenLayer>,
    pub stats: [u32; 5],
    pub audio_voices: u32,
    pub audio_pending: u32,
    /// Active visibility track sample, when a track is playing.
    pub track_sample: Option<u32>,
}
// Runtime projection uses width/2 horizontally and height*2/3 vertically.
// Their normalized ratio is 4:3 for every supported video mode, irrespective
// of the raw framebuffer aspect (PSX pixels are not necessarily square).
pub const GAME_PROJECTION_ASPECT: f32 = 4. / 3.;
impl Frame {
    pub fn view(&self) -> crate::viewport::View {
        let Some(index) = self.camera.filter(|index| *index < self.scene.actors.len()) else {
            return crate::viewport::View::default();
        };
        let camera = &self.scene.actors[index];
        let matrix = self.scene.world_matrix(index);
        let mut forward = matrix.vector([0., 0., 1.]);
        let length = forward
            .iter()
            .map(|value| value * value)
            .sum::<f32>()
            .sqrt();
        if length > 1e-6 {
            for value in &mut forward {
                *value /= length;
            }
        } else {
            forward = [0., 0., 1.];
        }
        let eye = matrix.point([0.; 3]);
        let distance = 1.;
        crate::viewport::View {
            yaw: forward[0].atan2(forward[2]),
            pitch: (-forward[1]).clamp(-1., 1.).asin(),
            center: std::array::from_fn(|axis| eye[axis] + forward[axis] * distance),
            // SceneGpu's orbit shader multiplies vertical coordinates by 2.9.
            // Match the runtime's normalized vertical focal length, 4/3*cot.
            zoom: (1. / (camera.camera_fov.clamp(25., 120.).to_radians() * 0.5).tan())
                * GAME_PROJECTION_ASPECT
                / 2.9,
            distance,
            phase: self.number as f32 / 60.,
            fly_speed: 5.,
            camera_speed_scalar: 1.,
            distance_scaled_speed: false,
            mirror_x: camera.camera_mirror_x,
            lighting: true,
        }
    }
}

#[derive(Default)]
pub struct State {
    pub frame: Option<Arc<Frame>>,
    pub connected: bool,
    pub error: Option<String>,
    pub rumble: Option<crate::rumble::Snapshot>,
}

#[derive(Clone)]
pub struct Bridge {
    pub state: Arc<Mutex<State>>,
    pub pads: Arc<Mutex<[crate::controls::PadState; 4]>>,
    pub session: u64,
    pub output_enabled: Arc<AtomicBool>,
}
impl Bridge {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State::default())),
            pads: Arc::new(Mutex::new(Default::default())),
            session: next_rumble_session(),
            output_enabled: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[derive(Debug)]
struct Packet {
    rumble: [crate::rumble::Command; 4],
    number: u32,
    fade: u8,
    actor_count: u32,
    authored_count: u32,
    camera: Option<usize>,
    actors: Vec<ActorState>,
    commands: Vec<hud_native::Command>,
    audio: Vec<AudioEvent>,
    scene_bank: u32,
    scene_generation: u32,
    transition_phase: u32,
    audio_gain: u32,
    loading_text: String,
    loading_color: [u8; 3],
    loading_size: [u32; 2],
    loading_pixels: Vec<u16>,
    stats: [u32; 5],
    track_sample: Option<u32>,
}

fn word(input: &mut impl Read) -> Result<u32, String> {
    let mut bytes = [0; 4];
    input
        .read_exact(&mut bytes)
        .map_err(|error| format!("Native PC runtime ended: {error}"))?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_packet(input: &mut impl Read) -> Result<Packet, String> {
    if word(input)? != MAGIC {
        return Err("Invalid Native PC Play protocol".into());
    }
    let number = word(input)?;
    let fade = word(input)?;
    let actor_count = word(input)?;
    let authored_count = word(input)?;
    let camera = match word(input)? {
        u32::MAX => None,
        value => Some(value as usize),
    };
    let command_count = word(input)?;
    let text_length = word(input)?;
    let audio_count = word(input)?;
    let mut stats = [0; 5];
    for value in &mut stats {
        *value = word(input)?;
    }
    let track_sample = match word(input)? {
        u32::MAX => None,
        value => Some(value),
    };
    let scene_bank = word(input)?;
    let scene_generation = word(input)?;
    let transition_phase = word(input)?;
    let audio_gain = word(input)?;
    let loading_color = word(input)?.to_le_bytes();
    let loading_size = [word(input)?, word(input)?];
    if scene_bank >= 16
        || transition_phase > 4
        || audio_gain > 4096
        || loading_size.iter().any(|size| *size > 64)
        || loading_size[0] % 2 != 0
        || (loading_size[0] == 0) != (loading_size[1] == 0)
        || actor_count > 1024
        || authored_count > actor_count
        || command_count > 8192
        || audio_count > AUDIO_LIMIT
        || text_length > 95
        || fade > 255
    {
        return Err("Native PC Play packet exceeds protocol bounds".into());
    }
    let mut rumble = [crate::rumble::Command::default(); 4];
    for command in &mut rumble {
        let identity = word(input)?;
        let revision = word(input)?;
        let large = word(input)?;
        let small = word(input)?;
        let remaining_us = word(input)?;
        if large > 255 || small > 1 || remaining_us > 60_000_000 {
            return Err("Native PC rumble packet exceeds protocol bounds".into());
        }
        *command = crate::rumble::Command {
            identity,
            revision,
            large: large as u8,
            small: small != 0,
            remaining_us,
        };
    }
    let mut actors = Vec::with_capacity(actor_count as usize);
    for _ in 0..actor_count {
        let mut values = [0u32; ACTOR_WORDS];
        for value in &mut values {
            *value = word(input)?;
        }
        actors.push(ActorState {
            alive: values[0] != 0,
            active: values[1] != 0,
            parent: values[2] as i32,
            position: [values[3] as i32, values[4] as i32, values[5] as i32],
            rotation: [values[6] as i32, values[7] as i32, values[8] as i32],
            scale: [values[9] as i32, values[10] as i32, values[11] as i32],
            fov: values[12] as i32,
            mirror_x: values[15] & 8 != 0,
            animation_ticks: values[13],
            animation_clip: values[14] as i32,
            animation_enabled: values[15] & 1 != 0,
            animation_looping: values[15] & 4 != 0,
        });
    }
    let mut commands = vec![hud_native::Command::default(); command_count as usize];
    for command in &mut commands {
        for value in &mut command.0 {
            *value = word(input)? as i32;
        }
    }
    let mut audio = Vec::with_capacity(audio_count as usize);
    for _ in 0..audio_count {
        let mut values = [0u32; AUDIO_WORDS];
        for value in &mut values {
            *value = word(input)?;
        }
        audio.push(AudioEvent {
            source: u64::from(values[0]) | (u64::from(values[1]) << 32),
            play: values[2] != 0,
            clip: values[3] as i32,
            volume: values[4] as i32,
            pitch: values[5] as i32,
        });
    }
    let mut text = vec![0; text_length as usize];
    input
        .read_exact(&mut text)
        .map_err(|error| error.to_string())?;
    let mut loading_pixels = Vec::with_capacity((loading_size[0] * loading_size[1]) as usize);
    for _ in 0..loading_size[0] * loading_size[1] {
        let mut bytes = [0; 2];
        input.read_exact(&mut bytes).map_err(|e| e.to_string())?;
        loading_pixels.push(u16::from_le_bytes(bytes));
    }
    Ok(Packet {
        rumble,
        number,
        fade: fade as u8,
        actor_count,
        authored_count,
        camera,
        actors,
        commands,
        audio,
        scene_bank,
        scene_generation,
        transition_phase,
        audio_gain,
        loading_text: String::from_utf8_lossy(&text).into_owned(),
        loading_color: [loading_color[0], loading_color[1], loading_color[2]],
        loading_size,
        loading_pixels,
        stats,
        track_sample,
    })
}

fn apply(
    mut scene: Scene,
    resources: &Scene,
    packet: &Packet,
    buttons: u16,
    audio_voices: usize,
    audio_pending: usize,
) -> Frame {
    let fixed = |raw: i32| raw as f32 / 4096.;
    let count = scene
        .actors
        .len()
        .min(packet.authored_count as usize)
        .min(packet.actors.len());
    for (actor, state) in scene.actors.iter_mut().zip(&packet.actors).take(count) {
        actor.active = state.alive && state.active;
        actor.parent = (state.parent >= 0).then_some(state.parent as usize);
        actor.position = state.position.map(fixed);
        actor.rotation = state.rotation.map(fixed);
        actor.scale = state.scale.map(fixed);
        actor.camera_mirror_x = state.mirror_x;
        if state.fov > 0 {
            actor.camera_fov = fixed(state.fov);
        }
        if let Some(skeletal) = &mut actor.skeletal_mesh {
            skeletal.clip = state
                .animation_enabled
                .then_some(state.animation_clip)
                .filter(|clip| *clip >= 0)
                .and_then(|clip| {
                    skeletal
                        .model
                        .as_ref()
                        .and_then(|model| model.clips.get(clip as usize))
                        .map(|(id, _)| *id)
                });
            skeletal.time = state.animation_ticks as f32 / 60.;
            skeletal.looping = state.animation_looping;
        }
    }
    let (mut hud_rgba, screen_layers) = hud_native::render_overlay_layers(
        &scene,
        resources,
        &packet.commands,
        packet.number as f32 / 60.,
        packet.fade,
    );
    if matches!(packet.transition_phase, 2 | 4) {
        render_loading(&mut hud_rgba, scene.display_size, packet);
    }
    Frame {
        scene_bank: packet.scene_bank,
        scene_generation: packet.scene_generation,
        transition_phase: packet.transition_phase,
        hud_size: scene.display_size.map(u32::from),
        scene,
        number: u64::from(packet.number),
        buttons,
        actor_count: packet.actor_count,
        camera: packet.camera.filter(|index| *index < count),
        hud_rgba,
        screen_layers,
        stats: packet.stats,
        audio_voices: audio_voices as u32,
        audio_pending: audio_pending as u32,
        track_sample: packet.track_sample,
    }
}

// Use the console loading layout: fixed 8x16 glyphs, right/bottom margin 5%,
// with the optional resident RGB555 image above the text.
fn render_loading(pixels: &mut [u8], size: [u16; 2], packet: &Packet) {
    let [width, height] = size.map(i32::from);
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0, 0, 0, 255]);
    }
    let mut put = |x: i32, y: i32, rgb: [u8; 3]| {
        if x >= 0 && y >= 0 && x < width && y < height {
            let i = ((y * width + x) * 4) as usize;
            pixels[i..i + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    };
    let columns = ((width - width / 10) / 8).max(1) as usize;
    let text = packet.loading_text.as_bytes();
    let rows = text.len().div_ceil(columns) as i32;
    let right = width - width / 20;
    let top = height - height / 20 - rows * 16;
    for (row, line) in text.chunks(columns).enumerate() {
        let left = right - line.len() as i32 * 8;
        for (column, ch) in line.iter().enumerate() {
            let ch = if (32..=126).contains(ch) { *ch } else { b'?' };
            for (y, bits) in crate::bitmap_font::glyph(char::from(ch))
                .unwrap_or([0; 16])
                .iter()
                .enumerate()
            {
                for x in 0..8 {
                    if bits & (1 << x) != 0 {
                        put(
                            left + column as i32 * 8 + x,
                            top + row as i32 * 16 + y as i32,
                            packet.loading_color,
                        );
                    }
                }
            }
        }
    }
    let [w, h] = packet.loading_size;
    for (i, color) in packet.loading_pixels.iter().enumerate() {
        if *color == 0 {
            continue;
        }
        let rgb = [0, 5, 10].map(|shift| {
            let c = ((color >> shift) & 31) as u8;
            (c << 3) | (c >> 2)
        });
        put(
            right - w as i32 + (i as u32 % w) as i32,
            top - if rows > 0 { 4 } else { 0 } - h as i32 + (i as u32 / w) as i32,
            rgb,
        );
    }
}

/// Camera recording for the visibility bake: while set, every Native PC Play
/// frame with an active visibility track appends the rendered camera, in the
/// runtime's Q12 camera space, as one JSON line (see visibility_bake).
pub struct CameraRecorder {
    file: std::io::BufWriter<std::fs::File>,
    display: [u16; 2],
    pub frames: usize,
}
static RECORDER: Mutex<Option<CameraRecorder>> = Mutex::new(None);
/// Starts recording into `path` (replacing it), or stops with `None`.
pub fn record_cameras(path: Option<&Path>, display: [u16; 2]) -> Result<(), String> {
    let mut recorder = RECORDER.lock().unwrap();
    if let Some(mut old) = recorder.take() {
        old.file.flush().map_err(|e| e.to_string())?;
    }
    if let Some(path) = path {
        let file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
        *recorder = Some(CameraRecorder {
            file: std::io::BufWriter::new(file),
            display,
            frames: 0,
        });
    }
    Ok(())
}
pub fn recorded_camera_frames() -> usize {
    RECORDER.lock().unwrap().as_ref().map_or(0, |r| r.frames)
}
impl Frame {
    /// The camera as the runtime renders it: basis rows (right, up, forward)
    /// and origin from the camera's world transform, and row_scale folding the
    /// projection (horizontal FOV across the display, mirror) into rows 0, 1.
    pub fn runtime_camera(&self, display: [u16; 2]) -> Option<serde_json::Value> {
        let index = self.camera.filter(|i| *i < self.scene.actors.len())?;
        let sample = self.track_sample?;
        let camera = &self.scene.actors[index];
        let world = self.scene.world_matrix(index);
        // Exactly the runtime's camera space, including a camera scale (which
        // scales depth, and so the runtime's near and far limits).
        let basis = world.inverse().ok()?;
        let cot = 1. / (camera.camera_fov.clamp(25., 120.).to_radians() * 0.5).tan();
        let horizontal = f32::from(display[0]) / 2. / (f32::from(display[1]) * 2. / 3.);
        let q12 = |v: f32| (v * 4096.).round() as i64;
        Some(serde_json::json!({
            "sample": sample, "frame": self.number,
            "scene": self.scene.name, "scene_bank": self.scene_bank, "scene_generation": self.scene_generation,
            "basis": (0..3).map(|r| (0..3).map(|c| q12(basis.0[r][c])).collect::<Vec<_>>()).collect::<Vec<_>>(),
            "origin": (0..3).map(|r| q12(world.0[r][3])).collect::<Vec<_>>(),
            "row_scale": [q12(cot * horizontal * if camera.camera_mirror_x { -1. } else { 1. }), q12(cot)],
        }))
    }
}
fn record_frame(frame: &Frame) {
    // One recording feeds one visibility bake. Never mix another bank's sample
    // numbers into the initial scene's file, or record a frozen loading pose.
    if frame.scene_bank != 0 || frame.transition_phase != 0 {
        return;
    }
    let mut recorder = RECORDER.lock().unwrap();
    let Some(recorder) = recorder.as_mut() else {
        return;
    };
    if let Some(camera) = frame.runtime_camera(recorder.display)
        && writeln!(recorder.file, "{camera}").is_ok()
    {
        recorder.frames += 1;
        if recorder.frames % 60 == 0 {
            let _ = recorder.file.flush();
        }
    }
}

struct Session {
    child: Child,
    input: ChildStdin,
    packets: Receiver<Result<Packet, String>>,
    pending: bool,
    log: PathBuf,
}
impl Drop for Session {
    fn drop(&mut self) {
        crate::pipeline::stop(&mut self.child);
    }
}
impl Session {
    fn launch(exe: &Path) -> Result<Self, String> {
        let log = exe.parent().unwrap().join("runtime.log");
        let mut command = Command::new(exe);
        crate::pipeline::quiet(&mut command);
        command
            .current_dir(exe.parent().unwrap())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(fs::File::create(&log).map_err(|error| error.to_string())?);
        let mut child = command
            .spawn()
            .map_err(|error| format!("Start Native PC runtime: {error}"))?;
        let input = child.stdin.take().ok_or("Native PC runtime has no input")?;
        let mut output = child
            .stdout
            .take()
            .ok_or("Native PC runtime has no output")?;
        let (send, packets) = mpsc::sync_channel(2);
        std::thread::spawn(move || {
            loop {
                let packet = read_packet(&mut output);
                let failed = packet.is_err();
                if send.send(packet).is_err() || failed {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input,
            packets,
            pending: true,
            log,
        })
    }
    fn request(
        &mut self,
        elapsed: u32,
        pads: [crate::controls::PadState; 4],
        completed: &[u64],
    ) -> Result<(), String> {
        if self.pending {
            return Ok(());
        }
        if completed.len() > AUDIO_LIMIT as usize {
            return Err("Native PC audio completion batch exceeds protocol bounds".into());
        }
        self.input
            .write_all(&MAGIC.to_le_bytes())
            .and_then(|_| self.input.write_all(&elapsed.to_le_bytes()))
            .and_then(|_| {
                for pad in pads {
                    self.input
                        .write_all(&u32::from(pad.buttons).to_le_bytes())?;
                    self.input.write_all(&u32::from(pad.analog).to_le_bytes())?;
                    self.input
                        .write_all(&u32::from_le_bytes(pad.axes).to_le_bytes())?;
                    let flags = u32::from(pad.connected)
                        | (u32::from(pad.rumble_supported) << 1)
                        | (u32::from(pad.rumble_enabled) << 2);
                    self.input.write_all(&flags.to_le_bytes())?;
                    self.input.write_all(&pad.identity.to_le_bytes())?;
                }
                Ok(())
            })
            .and_then(|_| {
                self.input
                    .write_all(&(completed.len() as u32).to_le_bytes())
            })
            .and_then(|_| {
                for source in completed {
                    self.input.write_all(&(*source as u32).to_le_bytes())?;
                    self.input
                        .write_all(&((*source >> 32) as u32).to_le_bytes())?;
                }
                Ok(())
            })
            .and_then(|_| self.input.flush())
            .map_err(|error| error.to_string())?;
        self.pending = true;
        Ok(())
    }
    fn receive(&mut self, timeout: Duration) -> Result<Packet, String> {
        let packet = self.packets.recv_timeout(timeout).map_err(|error| {
            format!(
                "Native PC runtime did not return a frame: {error}\n{}",
                fs::read_to_string(&self.log).unwrap_or_default()
            )
        })??;
        self.pending = false;
        Ok(packet)
    }
}

type AudioCacheKey = (PathBuf, String);
type CachedAudio = Result<Arc<crate::preview_audio::Pcm>, String>;
static AUDIO_CACHE: OnceLock<Mutex<HashMap<AudioCacheKey, CachedAudio>>> = OnceLock::new();
static AUDIO_LOADING: OnceLock<Mutex<HashSet<AudioCacheKey>>> = OnceLock::new();

/// Automated runs set `EPOK_MUTE_AUDIO`. Playback still starts and completes on
/// schedule, so audio events behave exactly as they do when heard.
fn audible(gain: u32) -> f32 {
    static MUTED: OnceLock<bool> = OnceLock::new();
    if *MUTED.get_or_init(|| std::env::var_os("EPOK_MUTE_AUDIO").is_some()) {
        0.
    } else {
        gain as f32 / 4096.
    }
}

struct HostClip {
    key: AudioCacheKey,
    path: PathBuf,
    pcm: Option<Arc<crate::preview_audio::Pcm>>,
}
struct HostAudio {
    root: PathBuf,
    clips: Vec<HostClip>,
    players: HashMap<u64, crate::preview_audio::Player>,
    pending: HashMap<u64, AudioEvent>,
    gain: u32,
}
impl HostAudio {
    fn new(root: &Path, scene: &Scene, index: &crate::assets::Index) -> Result<Self, String> {
        let mut clips = Vec::new();
        let ids = crate::audio::clip_ids(scene);
        let asset_revision = index.fingerprint();
        for id in &ids {
            let record = index.resolve(*id)?;
            let key = (
                record.path.clone(),
                format!("{}:{asset_revision}", record.revision),
            );
            let pcm = AUDIO_CACHE
                .get_or_init(Default::default)
                .lock()
                .unwrap()
                .get(&key)
                .and_then(|result| result.as_ref().ok().cloned());
            clips.push(HostClip {
                key,
                path: record.path.clone(),
                pcm,
            });
        }
        let mut result = Self {
            root: root.to_path_buf(),
            clips,
            players: HashMap::new(),
            pending: HashMap::new(),
            gain: 4096,
        };
        for clip in scene
            .actors
            .iter()
            .enumerate()
            .filter(|(index, _)| scene.is_active(*index))
            .map(|(_, actor)| actor)
            .flat_map(crate::audio::sources)
            .filter(|source| source.play_on_start)
            .filter_map(|source| source.clip)
            .filter_map(|id| ids.iter().position(|candidate| *candidate == id))
            .collect::<HashSet<_>>()
        {
            result.load(clip);
        }
        Ok(result)
    }
    fn load(&mut self, clip: usize) {
        let Some(slot) = self.clips.get_mut(clip) else {
            return;
        };
        if slot.pcm.is_some()
            || AUDIO_CACHE
                .get_or_init(Default::default)
                .lock()
                .unwrap()
                .contains_key(&slot.key)
        {
            return;
        }
        let root = self.root.clone();
        let path = slot.path.clone();
        let key = slot.key.clone();
        if !AUDIO_LOADING
            .get_or_init(Default::default)
            .lock()
            .unwrap()
            .insert(key.clone())
        {
            return;
        }
        std::thread::spawn(move || {
            let decode = |mode| {
                crate::content_preview::decode_audio_context(
                    &path,
                    mode,
                    Some(&root),
                    &AtomicBool::new(false),
                )
            };
            let result = decode(crate::content_preview::PreviewMode::TargetPsx)
                .or_else(|target_error| {
                    if target_error.contains("cannot decode PSX XA") {
                        decode(crate::content_preview::PreviewMode::Source)
                    } else {
                        Err(target_error)
                    }
                })
                .map(Arc::new);
            AUDIO_CACHE
                .get_or_init(Default::default)
                .lock()
                .unwrap()
                .insert(key.clone(), result);
            AUDIO_LOADING
                .get_or_init(Default::default)
                .lock()
                .unwrap()
                .remove(&key);
        });
    }
    fn start(&mut self, event: AudioEvent) -> Result<bool, String> {
        let Some(source) = self
            .clips
            .get(event.clip.max(0) as usize)
            .and_then(|clip| clip.pcm.clone())
        else {
            return Ok(false);
        };
        let mut pcm = (*source).clone();
        let volume = (event.volume as f32 / 4096.).clamp(0., 1.);
        for sample in &mut pcm.samples {
            *sample = (*sample as f32 * volume)
                .round()
                .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        }
        let pitch = (event.pitch as f32 / 4096.).clamp(0.25, 4.);
        pcm.rate = (pcm.rate as f32 * pitch).round().clamp(8_000., 192_000.) as u32;
        let player = crate::preview_audio::Player::start_with_gain(pcm, audible(self.gain))
            .map_err(|error| format!("Native PC audio: {error}"))?;
        self.players.insert(event.source, player);
        Ok(true)
    }
    fn update(&mut self, events: &[AudioEvent]) -> Result<Vec<u64>, String> {
        let completed = self
            .players
            .iter()
            .filter_map(|(source, player)| player.finished().then_some(*source))
            .collect::<Vec<_>>();
        for source in &completed {
            self.players.remove(source);
        }
        {
            let cache = AUDIO_CACHE.get_or_init(Default::default).lock().unwrap();
            for clip in &mut self.clips {
                if clip.pcm.is_none()
                    && let Some(Ok(pcm)) = cache.get(&clip.key)
                {
                    clip.pcm = Some(pcm.clone());
                }
            }
        }
        for event in events {
            self.players.remove(&event.source);
            self.pending.remove(&event.source);
            if !event.play || event.clip < 0 {
                continue;
            }
            if event.clip as usize >= self.clips.len() {
                return Err(format!(
                    "Native PC audio clip {} is outside the prepared bank",
                    event.clip
                ));
            }
            if !self.start(*event)? {
                self.load(event.clip as usize);
                self.pending.insert(event.source, *event);
            }
        }
        let ready = self
            .pending
            .iter()
            .filter_map(|(source, event)| {
                self.clips
                    .get(event.clip as usize)
                    .is_some_and(|clip| clip.pcm.is_some())
                    .then_some(*source)
            })
            .collect::<Vec<_>>();
        if let Some(error) = self.pending.values().find_map(|event| {
            let clip = self.clips.get(event.clip as usize)?;
            AUDIO_CACHE
                .get_or_init(Default::default)
                .lock()
                .unwrap()
                .get(&clip.key)
                .and_then(|result| result.as_ref().err().cloned())
        }) {
            return Err(format!("Native PC audio: {error}"));
        }
        for source in ready {
            let event = self.pending.remove(&source).unwrap();
            self.start(event)?;
        }
        Ok(completed)
    }
    fn set_gain(&mut self, gain: u32) -> Result<(), String> {
        if self.gain != gain {
            for player in self.players.values() {
                player.set_gain(audible(gain))?;
            }
            self.gain = gain;
        }
        Ok(())
    }
    fn reset(&mut self) {
        self.players.clear();
        self.pending.clear();
    }
    fn pause(&self, paused: bool) {
        for player in self.players.values() {
            player.pause(paused);
        }
    }
    fn status(&self) -> (usize, usize) {
        (self.players.len(), self.pending.len())
    }
}

struct Prepared {
    exe: PathBuf,
    banks: Vec<Scene>,
    resources: Scene,
}
fn prepare_input(
    root: &Path,
    input: &crate::scene_dependencies::Input,
    cancel: &AtomicBool,
) -> Result<Prepared, String> {
    let mut banks =
        crate::scene_bank::load_selected(root, &input.scene, &input.origin, Some(input))?.scenes;
    let assets = crate::assets::scan(root, &mut Default::default());
    let rendering = crate::settings::rendering(root)?;
    for scene in &mut banks {
        if cancel.load(Ordering::Relaxed) {
            return Err("Native PC Play cancelled".into());
        }
        crate::project::refresh_linked_scene(root, scene)?;
        crate::mesh::resolve(scene, &assets)?;
        crate::skeletal::resolve(scene, &assets)?;
        crate::texture::resolve(scene, &assets)?;
        crate::hud::resolve_fonts(scene, &assets)?;
        scene.display_size = [rendering.width, rendering.height];
        // Ordered draw is a PSX packet/streaming optimization. Native geometry
        // uses the editor renderer; rail/visibility-track gameplay remains live.
        scene.ordered_draw = None;
    }
    let catalog = input
        .native_play_catalog
        .clone()
        .map_or_else(|| crate::scripts::catalog(root), Ok)?;
    let registry = crate::blueprint::native_registry(root, &catalog)?;
    let blueprint_files = crate::blueprint_asset::load_all(root)?;
    let templates = crate::blueprint_spawn::prepare(root, &catalog, &assets)?;
    let mut resource_scenes = banks.clone();
    resource_scenes.extend(templates.into_iter().map(|t| t.scene));
    let referenced = crate::blueprint_refs::resources(
        &blueprint_files,
        &resource_scenes,
        &registry,
        &assets,
        &[],
    )?;
    resource_scenes.push(referenced);
    let resources = crate::scene_bank::resources(&resource_scenes);
    // Always compute the complete bank hash: an inactive map or its resources
    // may have changed without changing the editor's current scene revision.
    let exe = crate::hud_simulation::build_runner(
        root,
        &resources,
        &catalog,
        cancel,
        crate::hud_simulation::Runner::NativePlay,
        Some(&banks),
    )?;
    Ok(Prepared {
        exe,
        banks,
        resources,
    })
}
#[cfg(test)]
fn prepare(
    root: &Path,
    scene: Scene,
    _scene_ready: bool,
    catalog: Option<Vec<crate::scripts::Script>>,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    let mut input = crate::scene_dependencies::Input::from(scene);
    input.play = Some(crate::play::Profile {
        content: crate::play::Content::CurrentScene,
        ..Default::default()
    });
    input.native_play_catalog = catalog;
    Ok(prepare_input(root, &input, cancel)?.exe)
}

pub fn execute(
    root: &Path,
    input: &crate::scene_dependencies::Input,
    tx: &std::sync::mpsc::Sender<crate::pipeline::Event>,
    controls: &Receiver<crate::pipeline::Control>,
    bridge: &Bridge,
) -> Result<(), String> {
    struct ClearRumble(Bridge);
    impl Drop for ClearRumble {
        fn drop(&mut self) {
            let mut state = self.0.state.lock().unwrap();
            state.rumble = None;
            state.connected = false;
        }
    }
    let _rumble_guard = ClearRumble(bridge.clone());
    let _ = tx.send(crate::pipeline::Event::Stage(
        "Preparing Native PC scene banks".into(),
    ));
    let cancel = Arc::new(AtomicBool::new(false));
    let (done_tx, done_rx) = mpsc::channel();
    let root_owned = root.to_path_buf();
    let input = input.clone();
    let compile_cancel = cancel.clone();
    let worker = std::thread::spawn(move || {
        let _ = done_tx.send(prepare_input(&root_owned, &input, &compile_cancel));
    });
    let prepared = loop {
        match done_rx.recv_timeout(Duration::from_millis(30)) {
            Ok(result) => break result?,
            Err(mpsc::RecvTimeoutError::Timeout) => match controls.try_recv() {
                Ok(crate::pipeline::Control::Stop | crate::pipeline::Control::StopAndReset)
                | Err(mpsc::TryRecvError::Disconnected) => {
                    cancel.store(true, Ordering::Relaxed);
                    let _ = worker.join();
                    return Err("Native PC Play cancelled".into());
                }
                _ => {}
            },
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("Native PC compilation worker ended".into());
            }
        }
    };
    let Prepared {
        exe,
        banks,
        resources,
    } = prepared;
    let assets = crate::assets::scan(root, &mut Default::default());
    let mut audio = HostAudio::new(root, &resources, &assets)?;
    let _ = tx.send(crate::pipeline::Event::Built(exe.clone()));
    let mut generation = None;
    let mut session = Session::launch(&exe)?;
    let mut rumble_session = bridge.session;
    let _ = tx.send(crate::pipeline::Event::Running(session.child.id()));
    bridge.state.lock().unwrap().connected = true;
    let mut paused = false;
    let mut next = Instant::now();
    let mut sampled_pads = [crate::controls::PadState::default(); 4];
    loop {
        let packet = session.receive(Duration::from_secs(5))?;
        bridge.state.lock().unwrap().rumble = Some(crate::rumble::Snapshot {
            session: rumble_session,
            generation: packet.scene_generation,
            commands: packet.rumble,
            received: Instant::now(),
        });
        let scene = banks
            .get(packet.scene_bank as usize)
            .ok_or("Native PC returned an unknown scene bank")?;
        if generation != Some(packet.scene_generation) {
            audio.reset();
            generation = Some(packet.scene_generation);
        }
        audio.set_gain(packet.audio_gain)?;
        let completed_audio = audio.update(&packet.audio)?;
        let (audio_voices, audio_pending) = audio.status();
        let mut pads = *bridge.pads.lock().unwrap();
        for pad in &mut pads {
            pad.rumble_enabled &= bridge.output_enabled.load(Ordering::Relaxed);
        }
        let frame = apply(
            scene.clone(),
            &resources,
            &packet,
            sampled_pads[0].buttons,
            audio_voices,
            audio_pending,
        );
        let frame_number = frame.number;
        record_frame(&frame);
        bridge.state.lock().unwrap().frame = Some(Arc::new(frame));
        let mut reset = false;
        loop {
            match controls.try_recv() {
                Ok(crate::pipeline::Control::Stop | crate::pipeline::Control::StopAndReset)
                | Err(mpsc::TryRecvError::Disconnected) => {
                    return Ok(());
                }
                Ok(crate::pipeline::Control::Pause) => {
                    paused = true;
                    bridge.state.lock().unwrap().rumble = None;
                    audio.pause(true);
                    let _ = tx.send(crate::pipeline::Event::Paused(true));
                }
                Ok(crate::pipeline::Control::Resume) => {
                    paused = false;
                    audio.pause(false);
                    next = Instant::now();
                    let _ = tx.send(crate::pipeline::Event::Paused(false));
                }
                Ok(crate::pipeline::Control::Reset) => {
                    reset = true;
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
            }
        }
        if reset {
            bridge.state.lock().unwrap().rumble = None;
            rumble_session = next_rumble_session();
            session = Session::launch(&exe)?;
            generation = None;
            audio.reset();
            bridge.state.lock().unwrap().frame = None;
            paused = false;
            sampled_pads = Default::default();
            next = Instant::now();
            let _ = tx.send(crate::pipeline::Event::Running(session.child.id()));
            let _ = tx.send(crate::pipeline::Event::Paused(false));
            continue;
        }
        if paused {
            audio.pause(true);
            std::thread::sleep(Duration::from_millis(10));
            session.request(0, pads, &completed_audio)?;
            sampled_pads = pads;
            continue;
        }
        let elapsed =
            (((frame_number + 1) * 1_000_000 / 60) - (frame_number * 1_000_000 / 60)) as u32;
        next += Duration::from_micros(u64::from(elapsed));
        if next > Instant::now() {
            std::thread::sleep(next - Instant::now());
        }
        session.request(elapsed, pads, &completed_audio)?;
        sampled_pads = pads;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wire_header() -> [u32; 42] {
        let mut header = [0u32; 42];
        header[..22].copy_from_slice(&[
            MAGIC,
            0,
            0,
            0,
            0,
            u32::MAX,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            u32::MAX,
            0,
            0,
            0,
            4096,
            0xffffff,
            0,
            0,
        ]);
        header
    }
    #[test]
    fn transition_packet_limits_and_truncated_images_are_rejected() {
        for (slot, value) in [
            (15, 16),
            (17, 5),
            (18, 4097),
            (20, 66),
            (20, 3),
            (21, 65),
            (7, 96),
        ] {
            let mut words = wire_header();
            words[slot] = value;
            let bytes: Vec<u8> = words.into_iter().flat_map(u32::to_le_bytes).collect();
            assert!(read_packet(&mut &bytes[..]).unwrap_err().contains("bounds"));
        }
        let mut words = wire_header();
        words[20] = 2;
        words[21] = 1;
        let bytes: Vec<u8> = words.into_iter().flat_map(u32::to_le_bytes).collect();
        assert!(read_packet(&mut &bytes[..]).is_err());
    }
    #[test]
    fn loading_overlay_preserves_text_and_rgb555_image_over_black() {
        let mut words = wire_header();
        words[17] = 2;
        words[7] = 1;
        words[20] = 2;
        words[21] = 1;
        let mut bytes: Vec<u8> = words.into_iter().flat_map(u32::to_le_bytes).collect();
        bytes.extend_from_slice(b"A");
        bytes.extend_from_slice(&31u16.to_le_bytes());
        bytes.extend_from_slice(&992u16.to_le_bytes());
        let packet = read_packet(&mut &bytes[..]).unwrap();
        let mut pixels = vec![127; 320 * 240 * 4];
        render_loading(&mut pixels, [320, 240], &packet);
        assert_eq!(&pixels[..4], &[0, 0, 0, 255]);
        let image = ((228 - 16 - 4 - 1) * 320 + 304 - 2) * 4;
        assert_eq!(&pixels[image..image + 8], &[255, 0, 0, 255, 0, 255, 0, 255]);
        assert!(pixels.chunks_exact(4).any(|p| p == [255, 255, 255, 255]));
    }
    fn scene_fixture(root: &Path) -> crate::scene_dependencies::Input {
        let scripts = root.join("assets/scripts");
        fs::create_dir_all(&scripts).unwrap();
        fs::write(
            scripts.join("SceneProbe.hpp"),
            include_bytes!("../tests/fixtures/native-scenes/SceneProbe.hpp"),
        )
        .unwrap();
        fs::write(
            scripts.join("SceneProbe.cpp"),
            include_bytes!("../tests/fixtures/native-scenes/SceneProbe.cpp"),
        )
        .unwrap();
        let mut a = Scene {
            name: "A".into(),
            actors: vec![crate::scene::Actor::cube("Probe".into())],
            ..Default::default()
        };
        a.actors[0]
            .components
            .push(crate::actor_document::ComponentInstance::new(
                uuid::Uuid::new_v4(),
                crate::actor_document::ClassReference::new(
                    "SceneProbe",
                    "582e4f6b-711a-487b-a3f4-831ba03c757e",
                ),
                "Probe",
            ));
        let mut b = a.clone();
        b.name = "B".into();
        b.actors.push(crate::scene::Actor::cube("Only B".into()));
        for (scene, color) in [(&mut a, [255u8, 0, 0]), (&mut b, [0, 255, 0])] {
            let source = format!("assets/{}.png", scene.name);
            let file = fs::File::create(root.join(&source)).unwrap();
            let mut encoder = png::Encoder::new(file, 4, 4);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&color.repeat(16))
                .unwrap();
            let id = crate::assets::commit(
                crate::texture::prepare(
                    root,
                    &source,
                    &format!("assets/{}.epokasset", scene.name),
                    None,
                    false,
                )
                .unwrap(),
            )
            .unwrap();
            let canvas_index = scene.actors.len();
            let mut canvas = crate::scene::Actor::cube("Canvas".into());
            canvas.kind = "Empty".into();
            canvas.canvas = Some(Default::default());
            scene.actors.push(canvas);
            let mut image = crate::scene::Actor::cube("Image".into());
            image.kind = "Empty".into();
            image.parent = Some(canvas_index);
            image.rect = Some(crate::hud::RectTransform {
                anchor_min: [0., 1.],
                anchor_max: [0., 1.],
                pivot: [0., 1.],
                size: [4., 4.],
                ..Default::default()
            });
            image.image = Some(crate::hud::Image {
                texture: Some(id),
                color: [1.; 3],
                ..Default::default()
            });
            scene.actors.push(image);
        }
        let ap = root.join("assets/scenes/A.epokmap");
        a.save(&ap).unwrap();
        b.save(&root.join("assets/scenes/B.epokmap")).unwrap();
        let mut excluded = a.clone();
        excluded.name = "Excluded".into();
        excluded
            .save(&root.join("assets/scenes/Excluded.epokmap"))
            .unwrap();
        let mut input = crate::scene_dependencies::Input::load(&ap).unwrap();
        input.play = Some(crate::play::Profile {
            runtime: crate::play::Runtime::NativePc,
            content: crate::play::Content::SelectedScenes,
            selected_scenes: vec![
                "assets/scenes/A.epokmap".into(),
                "assets/scenes/B.epokmap".into(),
            ],
            ..Default::default()
        });
        input
    }
    fn step(
        session: &mut Session,
        elapsed: u32,
        button: Option<crate::controls::Button>,
        completed: &[u64],
    ) -> Packet {
        let mut pads = [crate::controls::PadState::default(); 4];
        for pad in &mut pads {
            pad.connected = true;
        }
        if let Some(button) = button {
            pads[0].buttons = 1 << button as u32;
        }
        session.request(elapsed, pads, completed).unwrap();
        session.receive(Duration::from_secs(10)).unwrap()
    }
    #[test]
    #[ignore = "requires a native C++ compiler and epok-header-tool"]
    fn scene_banks_round_trip_pause_reset_callbacks_and_stale_audio() {
        let root =
            std::env::temp_dir().join(format!("epok-native-scenes-{}", uuid::Uuid::new_v4()));
        let input = scene_fixture(&root);
        let prepared = prepare_input(&root, &input, &AtomicBool::new(false)).unwrap();
        assert_eq!(prepared.banks.len(), 2);
        let mut session = Session::launch(&prepared.exe).unwrap();
        let mut packet = session.receive(Duration::from_secs(10)).unwrap();
        assert_eq!((packet.scene_bank, packet.scene_generation), (0, 0));
        let mut old_audio = packet.audio[0].source;
        use crate::controls::Button;
        for turn in 0..24 {
            let expected = (turn + 1) % 2;
            let before = packet.scene_generation;
            // Release the previous button before requesting the next bank.
            step(&mut session, 16667, None, &[]);
            step(
                &mut session,
                16667,
                Some(if expected == 1 {
                    Button::Cross
                } else {
                    Button::Circle
                }),
                &[],
            );
            let mut loading = false;
            let mut intermediate_gain = false;
            let mut new_audio = None;
            for _ in 0..30 {
                packet = step(&mut session, 16667, None, &[old_audio]);
                intermediate_gain |= packet.audio_gain > 0 && packet.audio_gain < 4096;
                if packet.transition_phase == 2 {
                    loading = true;
                    assert_eq!(packet.scene_generation, before);
                    assert_eq!(packet.loading_text, "Scene probe");
                    assert_eq!(packet.loading_pixels, vec![31, 992]);
                }
                if matches!(packet.transition_phase, 1..=3) {
                    let frozen = (
                        packet.number,
                        packet.scene_bank,
                        packet.scene_generation,
                        packet.transition_phase,
                        packet.audio_gain,
                    );
                    for _ in 0..3 {
                        let p = step(&mut session, 0, None, &[]);
                        assert_eq!(
                            (
                                p.number,
                                p.scene_bank,
                                p.scene_generation,
                                p.transition_phase,
                                p.audio_gain
                            ),
                            frozen
                        );
                    }
                }
                for audio in &packet.audio {
                    if audio.play {
                        new_audio = Some(audio.source);
                        assert_ne!(audio.source, old_audio);
                    }
                }
                if packet.scene_generation == before + 1 && packet.transition_phase == 0 {
                    break;
                }
            }
            assert!(loading);
            if expected == 1 {
                assert!(intermediate_gain);
            }
            assert_eq!(
                (packet.scene_bank, packet.scene_generation),
                (expected, before + 1)
            );
            assert_eq!(packet.authored_count, if expected == 1 { 4 } else { 3 });
            packet = step(&mut session, 16667, None, &[old_audio]);
            assert_eq!(
                packet.actors[0].position[2], 4096,
                "stale audio completion stopped the new bank"
            );
            let frame = apply(
                prepared.banks[expected as usize].clone(),
                &prepared.resources,
                &packet,
                0,
                0,
                0,
            );
            assert_eq!(frame.scene.name, if expected == 1 { "B" } else { "A" });
            assert_eq!(frame.scene.actors.len(), packet.authored_count as usize);
            assert_eq!(
                &frame.hud_rgba[..4],
                if expected == 1 {
                    &[0, 255, 0, 255]
                } else {
                    &[255, 0, 0, 255]
                }
            );
            old_audio = new_audio.unwrap();
        }
        for button in [Button::Square, Button::Triangle] {
            let before = packet.scene_generation;
            step(&mut session, 16667, Some(button), &[]);
            for _ in 0..50 {
                packet = step(&mut session, 16667, None, &[]);
                if packet.scene_generation == before + 2 && packet.transition_phase == 0 {
                    break;
                }
            }
            assert_eq!(
                (packet.scene_bank, packet.scene_generation),
                (0, before + 2)
            );
        }
        // Stop while a request is queued, then Reset by launching the same artifact.
        step(&mut session, 16667, Some(Button::Cross), &[]);
        drop(session);
        let mut reset = Session::launch(&prepared.exe).unwrap();
        let first = reset.receive(Duration::from_secs(10)).unwrap();
        assert_eq!(
            (
                first.scene_bank,
                first.scene_generation,
                first.actors[0].position[0]
            ),
            (0, 0, 4096)
        );
        drop(reset);
        // All included banks participate in the artifact key, including unsaved
        // inactive maps. The editor's old fast-path artifact must not hide edits.
        let mut modified = input.clone();
        modified.native_play_cache = Some(prepared.exe.clone());
        let path = root.join("assets/scenes/B.epokmap");
        let mut changed = Scene::load_unresolved(&path).unwrap();
        changed.actors[1].scale = [2.; 3];
        modified.editor_override = Some((path, changed));
        let updated = prepare_input(&root, &modified, &AtomicBool::new(false)).unwrap();
        assert_ne!(prepared.exe, updated.exe);
        assert_eq!(updated.banks[1].actors[1].scale, [2.; 3]);
        assert_eq!(
            prepare_input(&root, &modified, &AtomicBool::new(false))
                .unwrap()
                .exe,
            updated.exe
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn packet_bounds_are_rejected_before_allocation() {
        let mut bytes = Vec::new();
        for value in [
            MAGIC,
            0,
            0,
            2048,
            0,
            u32::MAX,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            u32::MAX,
            0,
            0,
            0,
            4096,
            0xffffff,
            0,
            0,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        assert!(read_packet(&mut &bytes[..]).unwrap_err().contains("bounds"));
    }
    #[test]
    fn actor_animation_state_round_trips_through_the_protocol() {
        let mut bytes = Vec::new();
        for value in [
            MAGIC,
            7,
            0,
            1,
            1,
            u32::MAX,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            0,
            12,
            0,
            0,
            0,
            4096,
            0xffffff,
            0,
            0,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&[0u8; 80]); // Four EPN4 motor records.
        for value in [
            1,
            1,
            u32::MAX,
            0,
            0,
            0,
            0,
            0,
            0,
            4096,
            4096,
            4096,
            60 * 4096,
            18,
            3,
            1 | 2 | 4 | 8,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [0x89ab_cdef_u32, 0x0123_4567, 1, 2, 3072, 6144] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let packet = read_packet(&mut &bytes[..]).unwrap();
        assert_eq!(packet.track_sample, Some(12));
        let actor = &packet.actors[0];
        assert_eq!(actor.animation_ticks, 18);
        assert_eq!(actor.animation_clip, 3);
        assert!(actor.animation_enabled && actor.animation_looping);
        assert!(actor.mirror_x);
        let mut scene = Scene::default();
        scene.actors.truncate(1);
        let frame = apply(scene.clone(), &scene, &packet, 0, 0, 0);
        assert!(frame.scene.actors[0].camera_mirror_x);
        assert_eq!(packet.audio.len(), 1);
        assert_eq!(packet.audio[0].source, 0x0123_4567_89ab_cdef);
        assert!(packet.audio[0].play);
    }
    #[test]
    #[ignore = "requires a native C++ compiler and epok-header-tool"]
    fn spatial_batches_publish_changes_and_match_individual_queries() {
        let root =
            std::env::temp_dir().join(format!("epok-native-spatial-{}", uuid::Uuid::new_v4()));
        let scripts = root.join("assets/scripts");
        fs::create_dir_all(&scripts).unwrap();
        fs::write(
            scripts.join("SpatialProbe.hpp"),
            include_bytes!("../tests/fixtures/native-spatial/SpatialProbe.hpp"),
        )
        .unwrap();
        fs::write(
            scripts.join("SpatialProbe.cpp"),
            include_bytes!("../tests/fixtures/native-spatial/SpatialProbe.cpp"),
        )
        .unwrap();
        let mut scene = Scene {
            actors: (0..4)
                .map(|index| crate::scene::Actor::cube(format!("Actor{index}")))
                .collect(),
            ..Default::default()
        };
        scene.actors[0]
            .components
            .push(crate::actor_document::ComponentInstance::new(
                uuid::Uuid::new_v4(),
                crate::actor_document::ClassReference::new(
                    "SpatialProbe",
                    "482e4f6b-711a-487b-a3f4-831ba03c757e",
                ),
                "SpatialProbe",
            ));
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        let packet = session.receive(Duration::from_secs(10)).unwrap();
        assert_eq!(packet.actors[0].position[1], 42 * 4096);
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn gameplay_control_capture_runs_before_native_play_simulation() {
        let root = crate::workspace::tests::temp("gameplay-control-native-play");
        crate::workspace::create(&root, "Capture Native", crate::workspace::Template::Basic)
            .unwrap();
        fs::write(root.join("assets/scripts/WorldCapture.hpp"), r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="b1b7fa5c-9cde-5419-a21e-ac2374cec7a1") WorldCapture:public epok::ActorComponent {
    epok::InputCapture token{};
public:
    using Controls=epok::GameplayInputLibrary;
    void begin_play() override {token=Controls::capture(id(),Controls::mask(epok::GameplayAction::Confirm),100,0);}
    void on_frame(uint32_t) override {
        if(Controls::context_take_pressed(token,epok::GameplayAction::Confirm,true)){
            get_owner()->data()->transform.position[0]+=epok::Fixed(1.0);Controls::release(token);
        }
    }
    void tick(epok::Fixed) override {
        if(Controls::button_pressed(epok::Button::Cross,0))get_owner()->data()->transform.position[0]+=epok::Fixed(100.0);
    }
};
"#).unwrap();
        let mut actor = crate::scene::Actor::cube("Control".into());
        actor.position = [0.; 3];
        crate::actor_components::sync(&mut actor);
        actor
            .components
            .push(crate::actor_document::ComponentInstance::new(
                uuid::Uuid::new_v4(),
                crate::actor_document::ClassReference::new(
                    "WorldCapture",
                    "b1b7fa5c-9cde-5419-a21e-ac2374cec7a1",
                ),
                "WorldCapture",
            ));
        let scene = Scene {
            actors: vec![actor],
            ..Default::default()
        };
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        assert_eq!(
            session.receive(Duration::from_secs(10)).unwrap().actors[0].position[0],
            0
        );
        use crate::controls::Button;
        for (elapsed, button, expected) in [
            (10_000, Some(Button::Cross), 1),
            (10_000, Some(Button::Cross), 1),
            (16_667, None, 1),
            (16_667, Some(Button::Cross), 101),
        ] {
            assert_eq!(
                step(&mut session, elapsed, button, &[]).actors[0].position[0],
                expected * 4096
            );
        }
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn interaction_routes_frame_input_and_preserves_typed_bindings() {
        let root = crate::workspace::tests::temp("interaction-native-play");
        crate::workspace::create(
            &root,
            "Interaction Native",
            crate::workspace::Template::Basic,
        )
        .unwrap();
        fs::write(root.join("assets/scripts/InteractionProbe.hpp"),r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="765fdcfe-a818-57fd-be80-a245e301601e") InteractionProbe:public epok::InteractableComponent {
public:
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId expected_source{};
    bool can_interact(epok::ObjectId source) const override{return source==expected_source;}
    void interaction_started(epok::ObjectId,uint32_t) override{auto& z=get_owner()->data()->transform.position[2];z+=epok::Fixed(1.0);if(z>=2.0)epok::LocalizationLibrary::set_language(1);}
};
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="1e7d12b5-c1a2-58e3-a536-8cd6bef7620c") PlayerInteractionProbe:public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::InteractableComponent") epok::ObjectId target{};
    void begin_play() override{if(!epok::active_object_registry->resolve<epok::InteractableComponent>(target))get_owner()->data()->transform.position[2]=epok::Fixed(999.0);}
    void tick(epok::Fixed) override{if(epok::GameplayInputLibrary::button_pressed(epok::Button::Cross,0))get_owner()->data()->transform.position[0]+=epok::Fixed(100.0);}
};
"#).unwrap();
        let mut player = crate::scene::Actor::cube("Renamed Player".into());
        player.position = [0.; 3];
        let mut target = crate::scene::Actor::cube("Renamed Object".into());
        target.position = [0.; 3];
        crate::actor_components::sync(&mut player);
        crate::actor_components::sync(&mut target);
        use crate::actor_document::{ClassReference, ComponentInstance};
        let mut interactive = ComponentInstance::new(
            uuid::Uuid::new_v4(),
            ClassReference::new("InteractionProbe", "765fdcfe-a818-57fd-be80-a245e301601e"),
            "Object Rules",
        );
        interactive
            .properties
            .insert("expected_source".into(), serde_json::json!(player.id));
        interactive.overrides.insert("expected_source".into());
        interactive
            .properties
            .insert("prompt_key".into(), serde_json::json!(0));
        interactive.overrides.insert("prompt_key".into());
        let mut player_rules = ComponentInstance::new(
            uuid::Uuid::new_v4(),
            ClassReference::new(
                "PlayerInteractionProbe",
                "1e7d12b5-c1a2-58e3-a536-8cd6bef7620c",
            ),
            "Player Rules",
        );
        player_rules
            .properties
            .insert("target".into(), serde_json::json!(interactive.id));
        player_rules.overrides.insert("target".into());
        player.components.push(ComponentInstance::new(
            uuid::Uuid::new_v4(),
            ClassReference::new(
                "epok::InteractorComponent",
                "ebe82fcd-e9db-5de5-89a7-ed42f3a6d29a",
            ),
            "Interactor",
        ));
        player.components.push(player_rules);
        let scanner_id = player
            .components
            .iter()
            .find(|c| c.class.class_id.as_deref() == Some("ebe82fcd-e9db-5de5-89a7-ed42f3a6d29a"))
            .unwrap()
            .id;
        target.components.push(interactive);
        let mut scene = Scene {
            actors: vec![player, target],
            ..Default::default()
        };
        crate::localization::create(
            &root,
            "assets/Prompts.epokasset",
            &crate::localization::StringTable {
                languages: vec!["English".into(), "Spanish".into()],
                entries: vec![crate::localization::Entry {
                    key: "interaction.examine".into(),
                    values: vec!["Examine".into(), "Examinar".into()],
                }],
                ..Default::default()
            },
        )
        .unwrap();
        let registry =
            crate::blueprint::native_registry(&root, &crate::scripts::catalog(&root).unwrap())
                .unwrap();
        let prompt = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &registry.model().unwrap(),
            "b68bfec7-d1e2-5a99-8afd-14536a1196c3",
        )
        .unwrap();
        let presenter = scene.actors[prompt]
            .components
            .iter_mut()
            .find(|c| c.class.class_id.as_deref() == Some("b7e56800-3c6f-5e1d-a165-848c122b51c5"))
            .unwrap();
        presenter
            .properties
            .insert("interactor".into(), serde_json::json!(scanner_id));
        presenter.overrides.insert("interactor".into());
        let path = crate::workspace::startup_scene(&root).unwrap();
        fs::write(&path, crate::document::to_vec(&scene).unwrap()).unwrap();
        let scene = Scene::load(&path).unwrap();
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        let first = session.receive(Duration::from_secs(10)).unwrap();
        assert_eq!(first.actors[0].position, [0; 3]);
        let focused = step(&mut session, 10_000, None, &[]);
        assert_eq!(focused.stats[1], 7, "localized Examine prompt is drawn");
        assert_eq!(focused.stats[4], 0);
        use crate::controls::Button;
        for (elapsed, button, expected) in [
            (10_000, Some(Button::Cross), 1),
            (10_000, Some(Button::Cross), 1),
            (16_667, None, 1),
            (16_667, Some(Button::Cross), 2),
        ] {
            let frame = step(&mut session, elapsed, button, &[]);
            assert_eq!(frame.actors[0].position, [0; 3]);
            assert_eq!(frame.actors[1].position[2], expected * 4096);
            assert_eq!(frame.stats[1], if expected >= 2 { 8 } else { 7 });
            assert_eq!(frame.stats[4], 0);
        }
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native_camera_projection_uses_console_lens_and_clip_boundaries() {
        let root = crate::workspace::tests::temp("native-camera-projection");
        crate::workspace::create(
            &root,
            "Camera Projection",
            crate::workspace::Template::Basic,
        )
        .unwrap();
        fs::write(root.join("assets/scripts/ProjectionProbe.hpp"), r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Id="82367884-c392-48b6-9662-3a69252e33d8") ProjectionProbe:public epok::Actor3D {
 bool checked=false;
public:
 static constexpr uint64_t static_class_id=epok::detail::compact_class_id("82367884-c392-48b6-9662-3a69252e33d8");
 uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
 void on_frame(uint32_t)override{
  if(checked)return;checked=true;unsigned passed=0,total=0;
  auto check=[&](bool good){++total;if(good)++passed;};
  auto project=[](epok::Fixed x,epok::Fixed y,epok::Fixed z){return epok::SceneLibrary::project({x,y,z});};
  auto point=project(0.0,0.0,1.0);check(point.success&&point.x.raw()==epok::display_width/2*4096&&point.y.raw()==epok::display_height/2*4096);
  check(project(0.0,0.0,0.25).success);check(!project(0.0,0.0,epok::Fixed(1023,epok::Fixed::RAW)).success);
  check(!project(0.0,0.0,-1.0).success);check(project(0.0,0.0,127.0).success);check(!project(0.0,0.0,128.0).success);
  point=project(-1.0,0.0,1.0);check(point.success&&point.x.raw()==0);check(!project(1.0,0.0,1.0).success);
  point=project(0.0,0.75,1.0);check(point.success&&point.y.raw()==0);check(!project(0.0,-0.75,1.0).success);
  auto* camera=epok::active_camera().get();check(camera&&camera->owner);
  if(camera&&camera->owner){
   camera->camera_settings.mirror_x=true;point=project(0.5,0.0,1.0);check(point.success&&point.x.raw()==epok::display_width/4*4096);camera->camera_settings.mirror_x=false;
   camera->camera_settings.field_of_view=25.0;check(!project(0.5,0.0,1.0).success);
   camera->camera_settings.field_of_view=120.0;check(project(0.5,0.0,1.0).success);
   camera->camera_settings.field_of_view=90.0;camera->owner->set_active(false);check(!project(0.0,0.0,1.0).success);camera->owner->set_active(true);check(project(0.0,0.0,1.0).success);
  }
  data()->transform.position[0]=epok::Fixed(int(passed),0);data()->transform.position[1]=epok::Fixed(int(total),0);
 }
};
"#).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.truncate(1);
        scene.actors[0].position = [0.; 3];
        scene.actors[0].rotation = [0.; 3];
        scene.actors[0].camera_fov = 90.;
        scene.sync_actor_components();
        let probe = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            "82367884-c392-48b6-9662-3a69252e33d8",
        )
        .unwrap();
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        session.receive(Duration::from_secs(10)).unwrap();
        let frame = step(&mut session, 16_667, None, &[]);
        assert_eq!(frame.actors[probe].position[0], 16 * 4096);
        assert_eq!(frame.actors[probe].position[1], 16 * 4096);
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn native_locomotion_drivers_apply_project_policies_after_both_motors() {
        use crate::actor_document::{ClassReference, ComponentInstance};
        for dimension in ["3D", "2D"] {
            let root = crate::workspace::tests::temp("native-locomotion-driver");
            crate::workspace::create(&root, "Locomotion", crate::workspace::Template::Basic)
                .unwrap();
            let source = r#"#pragma once
#include "epok.hpp"
BANK
class EPOK_CLASS(Blueprintable,Id="46043bb7-5c77-5646-baf9-6c3c50103593") NativeAnimationPolicy:public epok::DRIVER {
public:
 void begin_play()override{epok::DRIVER::begin_play();clips={0,1,2,3,4,5};INITIALIZE}
 epok::LocomotionSelection select_animation(epok::CharacterMotionSampleDIM motion,epok::LocomotionState previous,bool finished)override{
  auto value=epok::DRIVER::select_animation(motion,previous,finished);value.rate=1.25;return value;
 }
};
class EPOK_CLASS(Blueprintable,Domain=DOMAIN,Owners=DOMAIN,Id="be52ada1-5f96-56e1-ac76-b6d29ec3c339") NativeAnimationReporter:public epok::ActorComponent {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::PostPhysics;
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId observer{};
 void tick(epok::Fixed)override{
  auto* actor=epok::active_object_registry->resolve<epok::CharacterDIM>(owner_id());
  auto* data=epok::gameplay_actor_data(observer);if(!actor||!data)return;
  auto* policy=epok::world2d::find<NativeAnimationPolicy>(actor);if(!policy||policy->sample().status!=epok::LocomotionStatus::Applied)return;
  data->transform.position[0]=policy->sample().rate;data->transform.position[1]=actor->motion().displacement.x;
  data->transform.position[2]=PLAYBACK;
 }
};
"#;
            let source=if dimension=="3D" {source.replace("DRIVER","SkeletalLocomotionComponent")
                .replace("BANK","inline const epok::AnimationClip native_clips[]={{nullptr,nullptr,nullptr,6,nullptr},{nullptr,nullptr,nullptr,8,nullptr},{nullptr,nullptr,nullptr,10,nullptr},{nullptr,nullptr,nullptr,5,nullptr},{nullptr,nullptr,nullptr,4,nullptr},{nullptr,nullptr,nullptr,3,nullptr}};\ninline const epok::SkeletalMesh native_model={nullptr,nullptr,nullptr,nullptr,nullptr,0,native_clips,6,{}};")
                .replace("INITIALIZE","auto* data=get_owner()->data();data->animator.enabled=true;data->animator.model=&native_model;")
                .replace("PLAYBACK","epok::Fixed(int32_t(actor->data()->animator.ticks+1),0)")
            } else {source.replace("DRIVER","SpriteLocomotion2DComponent")
                .replace("BANK","inline const epok::SpriteFrame native_frames[]={{{0,0,8,8},epok::Fixed(0.05),7},{{8,0,8,8},epok::Fixed(0.05),8}};\ninline const epok::SpriteClip native_clips[]={{native_frames,2,true,nullptr},{native_frames,2,true,nullptr},{native_frames,2,true,nullptr},{native_frames,2,false,nullptr},{native_frames,2,false,nullptr},{native_frames,2,false,nullptr}};")
                .replace("INITIALIZE","auto* sprite=epok::world2d::find<epok::Sprite2DComponent>(get_owner());sprite->clips=native_clips;sprite->clip_count=6;")
                .replace("PLAYBACK","epok::world2d::find<epok::Sprite2DComponent>(actor)->speed_scale")
            }.replace("DIM",dimension).replace("DOMAIN",if dimension=="3D" {"World3D"} else {"World2D"});
            fs::write(
                root.join("assets/scripts/NativeAnimationPolicy.hpp"),
                source,
            )
            .unwrap();
            let registry =
                crate::blueprint::native_registry(&root, &crate::scripts::catalog(&root).unwrap())
                    .unwrap();
            let model = registry.model().unwrap();
            let mut scene = Scene::default();
            scene.actors.truncate(1);
            let observer = scene.actors[0].id;
            let character = registry
                .named(&format!("epok::Character{dimension}"))
                .unwrap();
            let index =
                crate::editor::place_class(&mut scene, &root, &registry, &model, &character.id)
                    .unwrap();
            for (name, cls, properties) in [
                (
                    "Visual",
                    format!(
                        "epok::{}",
                        if dimension == "3D" {
                            "Mesh3DComponent"
                        } else {
                            "Sprite2DComponent"
                        }
                    ),
                    serde_json::json!({}),
                ),
                (
                    "Commands",
                    format!("epok::ScriptedCommand{dimension}Component"),
                    serde_json::json!({"desired":if dimension=="3D" {serde_json::json!({"velocity":{"x":4.0,"y":0.0},"heading":0.0,"turn":false,"jump":false})} else {serde_json::json!({"velocity":{"x":4.0,"y":0.0},"jump":false})}}),
                ),
                (
                    "Animation",
                    "NativeAnimationPolicy".into(),
                    serde_json::json!({}),
                ),
                (
                    "Report",
                    "NativeAnimationReporter".into(),
                    serde_json::json!({"observer":observer}),
                ),
            ] {
                let class = registry.named(&cls).unwrap();
                let mut part = ComponentInstance::new(
                    uuid::Uuid::new_v4(),
                    ClassReference::new(&class.cpp_name, &class.id),
                    name,
                );
                part.properties = serde_json::from_value(properties).unwrap();
                part.overrides = part.properties.keys().cloned().collect();
                scene.actors[index].components.push(part);
            }
            assert!(scene.actors[index].components.len() <= 8);
            let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
            let mut session = Session::launch(&exe).unwrap();
            session.receive(Duration::from_secs(10)).unwrap();
            let frame = step(&mut session, 16_667, None, &[]);
            assert_eq!(frame.actors[0].position[0], 5120, "{dimension}");
            assert!(frame.actors[0].position[1] > 0, "{dimension}");
            assert!(frame.actors[0].position[2] > 0, "{dimension}");
            drop(session);
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn native_authored_command_records_and_npc_goals_reach_both_motors() {
        use crate::actor_document::{ClassReference, ComponentInstance};
        for dimension in ["3D", "2D"] {
            let root = crate::workspace::tests::temp("native-authored-character-commands");
            crate::workspace::create(
                &root,
                "Authored Commands",
                crate::workspace::Template::Basic,
            )
            .unwrap();
            let source=r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Id="6d914f8c-4285-5ff4-858f-a26399a06664") AuthoredCommandReporter:public epok::ActorComponent {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::PostPhysics;
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::ScriptedCommandDIMComponent") epok::ObjectId source{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::NPCDIM") epok::ObjectId npc{};
 void tick(epok::Fixed)override{
  auto* policy=epok::active_object_registry->resolve<epok::ScriptedCommandDIMComponent>(source);
  auto* actor=epok::active_object_registry->resolve<epok::NPCDIM>(npc);
  if(!policy||!actor||!get_owner()->data())return;
  const auto command=policy->sample().command;auto& out=get_owner()->data()->transform;
  out.position[0]=command.velocity.x;out.position[1]=SECOND;
  out.position[2]=actor->motion().displacement.x;
 }
};
"#.replace("DIM",dimension).replace("SECOND",if dimension=="3D" {"command.turn&&command.jump?command.heading:epok::Fixed(0.0)"} else {"command.jump?command.velocity.y:epok::Fixed(0.0)"});
            fs::write(
                root.join("assets/scripts/AuthoredCommandReporter.hpp"),
                source,
            )
            .unwrap();
            let registry =
                crate::blueprint::native_registry(&root, &crate::scripts::catalog(&root).unwrap())
                    .unwrap();
            let model = registry.model().unwrap();
            let mut scene = Scene::default();
            scene.actors.truncate(1);
            let character = registry
                .named(&format!("epok::Character{dimension}"))
                .unwrap();
            let character_index =
                crate::editor::place_class(&mut scene, &root, &registry, &model, &character.id)
                    .unwrap();
            let source_class = registry
                .named(&format!("epok::ScriptedCommand{dimension}Component"))
                .unwrap();
            let mut policy = ComponentInstance::new(
                uuid::Uuid::new_v4(),
                ClassReference::new(&source_class.cpp_name, &source_class.id),
                "Authored route",
            );
            let policy_id = policy.id;
            let desired = if dimension == "3D" {
                serde_json::json!({"velocity":{"x":4.0,"y":-3.0},"heading":132.5,"turn":true,"jump":true})
            } else {
                serde_json::json!({"velocity":{"x":4.0,"y":-3.0},"jump":true})
            };
            policy.properties.insert("desired".into(), desired);
            policy.overrides.insert("desired".into());
            scene.actors[character_index].components.push(policy);
            let npc_class = registry.named(&format!("epok::NPC{dimension}")).unwrap();
            let npc_index =
                crate::editor::place_class(&mut scene, &root, &registry, &model, &npc_class.id)
                    .unwrap();
            scene.actors[npc_index].position[0] = 20.0;
            crate::actor_components::sync(&mut scene.actors[npc_index]);
            scene.actors[npc_index].components[0]
                .overrides
                .insert("position".into());
            let mover_class = registry
                .named(&format!("epok::MoveTo{dimension}Component"))
                .unwrap();
            let mover = scene.actors[npc_index]
                .components
                .iter_mut()
                .find(|c| c.class.class_id.as_deref() == Some(mover_class.id.as_str()))
                .unwrap();
            mover
                .properties
                .insert("use_point".into(), serde_json::json!(true));
            mover.overrides.insert("use_point".into());
            mover.properties.insert(
                "goal".into(),
                if dimension == "3D" {
                    serde_json::json!({"x":30.0,"y":0.0,"z":0.0})
                } else {
                    serde_json::json!({"x":30.0,"y":0.0})
                },
            );
            mover.overrides.insert("goal".into());
            let mut reporter = ComponentInstance::new(
                uuid::Uuid::new_v4(),
                ClassReference::new(
                    "AuthoredCommandReporter",
                    "6d914f8c-4285-5ff4-858f-a26399a06664",
                ),
                "Observe authored values",
            );
            for (name, value) in [("source", policy_id), ("npc", scene.actors[npc_index].id)] {
                reporter
                    .properties
                    .insert(name.into(), serde_json::json!(value));
                reporter.overrides.insert(name.into());
            }
            scene.actors[0].components.push(reporter);
            let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
            let mut session = Session::launch(&exe).unwrap();
            session.receive(Duration::from_secs(10)).unwrap();
            let result = step(&mut session, 16_667, None, &[]);
            assert_eq!(result.actors[0].position[0], 4 * 4096, "{dimension}");
            assert_eq!(
                result.actors[0].position[1],
                if dimension == "3D" {
                    132 * 4096 + 2048
                } else {
                    -3 * 4096
                },
                "{dimension}"
            );
            assert!(result.actors[0].position[2] > 0, "{dimension}");
            drop(session);
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn native_character_command_sources_keep_capture_and_physics_independent() {
        use crate::actor_document::{ClassReference, ComponentInstance};
        for dimension in ["3D", "2D"] {
            let root = crate::workspace::tests::temp("native-character-command-input");
            crate::workspace::create(&root, "Command Input", crate::workspace::Template::Basic)
                .unwrap();
            let source = r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Id="6a81b411-6321-56f6-b244-43a94b85522d") CommandInputReporter:public epok::ActorComponent {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::PostPhysics;
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::PlayerDIM") epok::ObjectId target{};
 void on_frame(uint32_t)override{
  if(epok::GameplayInputLibrary::frame_sample(epok::GameplayAction::Confirm,0).pressed)
   epok::GameplayInputLibrary::capture(id(),epok::GameplayInputLibrary::movement_actions(),100,0);
 }
 void tick(epok::Fixed)override{
  auto* actor=epok::active_object_registry->resolve<epok::PlayerDIM>(target);auto* owner=get_owner();
  if(!actor||!owner||!owner->data())return;
  owner->data()->transform.position[0]=actor->input_source.sample().command.velocity.x;
  owner->data()->transform.position[1]=actor->motion().velocity.y;
  owner->data()->transform.position[2]=actor->motion().displacement.x;
 }
};
"#.replace("DIM", dimension);
            fs::write(root.join("assets/scripts/CommandInputReporter.hpp"), source).unwrap();
            let catalog = crate::scripts::catalog(&root).unwrap();
            let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
            let model = registry.model().unwrap();
            let class = registry.named(&format!("epok::Player{dimension}")).unwrap();
            let mut scene = Scene::default();
            scene.actors.truncate(1);
            let index = crate::editor::place_class(&mut scene, &root, &registry, &model, &class.id)
                .unwrap();
            let mut reporter = ComponentInstance::new(
                uuid::Uuid::new_v4(),
                ClassReference::new(
                    "CommandInputReporter",
                    "6a81b411-6321-56f6-b244-43a94b85522d",
                ),
                "Observe commands",
            );
            reporter
                .properties
                .insert("target".into(), serde_json::json!(scene.actors[index].id));
            reporter.overrides.insert("target".into());
            scene.actors[0].components.push(reporter);
            let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
            let mut session = Session::launch(&exe).unwrap();
            session.receive(Duration::from_secs(10)).unwrap();
            let moved = step(
                &mut session,
                16_667,
                Some(crate::controls::Button::Right),
                &[],
            );
            assert!(moved.actors[0].position[0] > 0, "{dimension}");
            assert!(moved.actors[0].position[2] > 0, "{dimension}");
            let mut pads = [crate::controls::PadState::default(); 4];
            for pad in &mut pads {
                pad.connected = true;
            }
            pads[0].buttons = (1 << crate::controls::Button::Right as u32)
                | (1 << crate::controls::Button::Cross as u32);
            session.request(16_667, pads, &[]).unwrap();
            let captured = session.receive(Duration::from_secs(10)).unwrap();
            assert_eq!(captured.actors[0].position[0], 0, "{dimension}");
            if dimension == "3D" {
                assert!(captured.actors[0].position[1] < moved.actors[0].position[1]);
            } else {
                assert!(captured.actors[0].position[1] > moved.actors[0].position[1]);
            }
            drop(session);
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn native_character_classes_run_physics_before_presentation_in_both_domains() {
        use crate::actor_document::{ClassReference, ComponentInstance};
        for (parent, command) in [
            ("Character3D", "CharacterCommand3D"),
            ("Character2D", "CharacterCommand2D"),
        ] {
            let root = crate::workspace::tests::temp("native-character-stages");
            crate::workspace::create(&root, "Character Stages", crate::workspace::Template::Basic)
                .unwrap();
            let source = r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Id="497da2f7-604c-4824-bd68-5a5ce3fb8e22") CharacterStageProbe:public epok::CHARACTER {
public:void tick(epok::Fixed)override{epok::COMMAND command;command.velocity.x=2.0;submit_command(command);}
};
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Id="98fd7cdf-645e-46fc-9d25-e6d399894517") CharacterStageReporter:public epok::ActorComponent {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::PostPhysics;
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::CHARACTER") epok::ObjectId target{};
 void tick(epok::Fixed)override{
  auto* actor=epok::active_object_registry->resolve<epok::CHARACTER>(target);
  auto* owner=get_owner();if(!actor||!owner||!owner->data())return;
  const auto view=actor->motion();owner->data()->transform.position[0]=view.displacement.x;
  owner->data()->transform.position[1]=view.status==epok::CharacterMotionStatus::Applied?epok::Fixed(1.0):epok::Fixed(0.0);
 }
};
"#.replace("CHARACTER", parent).replace("COMMAND", command);
            fs::write(root.join("assets/scripts/CharacterStageProbe.hpp"), source).unwrap();
            let catalog = crate::scripts::catalog(&root).unwrap();
            let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
            let model = registry.model().unwrap();
            let mut scene = Scene::default();
            scene.actors.truncate(1);
            let index = crate::editor::place_class(
                &mut scene,
                &root,
                &registry,
                &model,
                "497da2f7-604c-4824-bd68-5a5ce3fb8e22",
            )
            .unwrap();
            let mut reporter = ComponentInstance::new(
                uuid::Uuid::new_v4(),
                ClassReference::new(
                    "CharacterStageReporter",
                    "98fd7cdf-645e-46fc-9d25-e6d399894517",
                ),
                "Observe before player",
            );
            reporter
                .properties
                .insert("target".into(), serde_json::json!(scene.actors[index].id));
            reporter.overrides.insert("target".into());
            scene.actors[0].components.push(reporter);
            let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
            let mut session = Session::launch(&exe).unwrap();
            session.receive(Duration::from_secs(10)).unwrap();
            let frame = step(&mut session, 16_667, None, &[]);
            assert!(frame.actors[0].position[0] > 0, "{parent}");
            assert_eq!(frame.actors[0].position[1], 4096, "{parent}");
            drop(session);
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn native_camera_helper_uses_optional_header_and_inherited_components() {
        let root = crate::workspace::tests::temp("native-camera-helper");
        crate::workspace::create(&root, "Camera Helper", crate::workspace::Template::Basic)
            .unwrap();
        fs::write(root.join("assets/scripts/CameraProbe.hpp"), r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Id="288b78b4-0bb4-47a9-95b3-88039a571112") CameraProbe:public epok::ThirdPersonCamera {
public:
 void begin_play()override{ThirdPersonCamera::begin_play();automatic_tick=false;follow_target=false;make_active();}
 epok::CameraRigSample calculate_view(epok::Fixed dt)override{auto view=ThirdPersonCamera::calculate_view(dt);view.position={1.0,2.0,3.0};view.roll=11.0;return view;}
};
"#).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        let probe = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            "288b78b4-0bb4-47a9-95b3-88039a571112",
        )
        .unwrap();
        assert_eq!(scene.actors[probe].components.len(), 3);
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        let frame = session.receive(Duration::from_secs(10)).unwrap();
        assert_eq!(frame.actors[probe].position, [4096, 8192, 12288]);
        assert_eq!(frame.actors[probe].rotation[2], 11 * 4096);
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn native_ui_anchors_follow_world_targets_without_interaction() {
        use crate::actor_document::{ClassReference, ComponentInstance};
        let root = crate::workspace::tests::temp("native-ui-anchor");
        crate::workspace::create(&root, "UI Anchor", crate::workspace::Template::Basic).unwrap();
        fs::write(
            root.join("assets/scripts/AnchorProbe.hpp"),
            include_str!("../tests/fixtures/native-scenes/AnchorProbe.hpp"),
        )
        .unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.truncate(1);
        scene.actors[0].position = [0.; 3];
        scene.actors[0].rotation = [0.; 3];
        scene.sync_actor_components();
        let widget = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            crate::object_model::UI_ACTOR_ID,
        )
        .unwrap();
        scene.actors[widget].canvas = Some(Default::default());
        let content = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            crate::object_model::UI_ACTOR_ID,
        )
        .unwrap();
        scene.actors[content].parent = Some(widget);
        scene.actors[content].rect = Some(crate::hud::RectTransform {
            anchor_min: [0.; 2],
            anchor_max: [0.; 2],
            pivot: [0.5, 0.],
            size: [100., 40.],
            ..Default::default()
        });
        let rect = scene.actors[content]
            .components
            .iter()
            .find(|c| {
                c.class.class_id.as_deref()
                    == Some(crate::object_model::RECT_TRANSFORM_COMPONENT_ID)
            })
            .unwrap()
            .id;
        let probe = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            "3c892c5a-4eb4-50c2-9466-8d6257f2ae74",
        )
        .unwrap();
        scene.actors[probe].position = [0.5, 0., 2.];
        let anchor_id = uuid::Uuid::new_v4();
        let mut anchor = ComponentInstance::new(
            anchor_id,
            ClassReference::new(
                "epok::ScreenAnchorComponent",
                crate::game_helpers::UI_CLASSES[0].1,
            ),
            "Anchor",
        );
        anchor
            .properties
            .insert("content".into(), serde_json::json!(rect));
        anchor
            .properties
            .insert("target".into(), serde_json::json!(scene.actors[probe].id));
        scene.actors[widget].components.push(anchor);
        scene.actors[probe]
            .properties
            .insert("anchor".into(), serde_json::json!(anchor_id));
        scene.sync_actor_components();
        let header = crate::blueprint_spawn::object_class_table_for_scenes(
            &registry,
            std::slice::from_ref(&scene),
        )
        .unwrap();
        assert!(header.contains("EPOK_UI_ANCHOR_USED"));
        assert!(!header.contains("EPOK_INTERACTION_USED"));
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        session.receive(Duration::from_secs(10)).unwrap();
        let frame = step(&mut session, 16_667, None, &[]);
        assert_eq!(frame.actors[probe].rotation[0], 8 * 4096);
        assert_eq!(frame.actors[probe].rotation[1], 8 * 4096);
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_session_updates_unscaled_reveal_and_consumes_closing_input() {
        let root = crate::workspace::tests::temp("dialogue-native-session");
        crate::workspace::create(&root, "Dialogue Session", crate::workspace::Template::Basic)
            .unwrap();
        use crate::{dialogue_asset as da, localization as loc};
        let table = loc::create(
            &root,
            "assets/dialogue-text.epokasset",
            &loc::StringTable {
                pageable: true,
                languages: vec!["English".into(), "Español".into()],
                entries: vec![loc::Entry {
                    key: "greeting".into(),
                    values: vec!["Á[color=#ff0000]BC[/color]\nD".into(), "ÓBC\nD".into()],
                }],
                ..Default::default()
            },
        )
        .unwrap();
        let line = uuid::Uuid::from_u128(1);
        let end = uuid::Uuid::from_u128(2);
        let branch = uuid::Uuid::from_u128(3);
        let event = uuid::Uuid::from_u128(4);
        let role = uuid::Uuid::from_u128(10);
        let condition = uuid::Uuid::from_u128(11);
        let action = uuid::Uuid::from_u128(12);
        let source = da::DialogueAsset {
            version: da::VERSION,
            entry: line,
            participants: vec![da::Participant {
                id: role,
                label: "Rules".into(),
                display_name: None,
                portrait: None,
            }],
            bindings: vec![
                da::Binding {
                    id: condition,
                    label: "Available".into(),
                    participant: role,
                    kind: da::BindingKind::Condition,
                    class: "310e882c-2353-55bc-bba3-f3d54277f671".into(),
                    member: "fb9c1c09-be15-5b42-a069-4480e8e389f2".into(),
                },
                da::Binding {
                    id: action,
                    label: "Accepted".into(),
                    participant: role,
                    kind: da::BindingKind::Action,
                    class: "310e882c-2353-55bc-bba3-f3d54277f671".into(),
                    member: "e66c2703-4f26-5bfc-b269-2f0c0eefbb8f".into(),
                },
            ],
            nodes: vec![
                da::Node {
                    id: line,
                    kind: da::NodeKind::Line {
                        speaker: None,
                        text: da::TextKey {
                            table,
                            key: "greeting".into(),
                        },
                        next: branch,
                        policy: Some(da::LinePolicy {
                            milliseconds_per_glyph: 10,
                            auto_advance_ms: None,
                        }),
                    },
                },
                da::Node {
                    id: end,
                    kind: da::NodeKind::End,
                },
                da::Node {
                    id: branch,
                    kind: da::NodeKind::Branch {
                        condition,
                        yes: event,
                        no: end,
                    },
                },
                da::Node {
                    id: event,
                    kind: da::NodeKind::Event { action, next: end },
                },
            ],
        };
        let conversation = da::create(&root, "assets/conversation.epokasset", &source).unwrap();
        let style = da::create_style(
            &root,
            "assets/dialogue-style.epokasset",
            &da::DialogueStyle::default(),
        )
        .unwrap();
        fs::write(root.join("assets/scripts/DialogueProbe.hpp"),r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Owners=UI,Id="23a292e7-7d93-5e4c-afb7-e4134800cc24") DialogueProbe:public epok::ActorComponent {
public:
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId initiator{};
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::InteractableComponent") epok::ObjectId rules{};
 EPOK_PROPERTY(EditAnywhere,AssetKind=Dialogue) uint64_t conversation=0;
 EPOK_PROPERTY(EditAnywhere,AssetKind=DialogueStyle) uint64_t style=0;
 static void entered(epok::ObjectId source,uint32_t){auto* component=epok::active_object_registry->resolve<epok::ActorComponent>(source);if(component)component->get_owner()->data()->transform.rotation[0]+=epok::Fixed(1.0);}
 void begin_play() override {
   namespace d=epok::dialogue;
   const auto* resource=d::resource(conversation,style);
   d::Start start;start.owner=id();start.initiator=initiator;start.presenter=owner_id();start.entered=&entered;
   const d::Assignment roles[]={{COOKED_ROLE,rules}};start.roles=roles;start.role_count=1;
   auto* data=get_owner()->data();
   if(!resource){data->transform.rotation[1]=epok::Fixed(int(d::Result::InvalidData),0);return;}
   start.timing=resource->presentation->timing;
   data->transform.rotation[1]=epok::Fixed(int(d::session().start(*resource->table,start)),0);
   data->text.set_text(d::session().text());data->text.visible_glyphs=0;
 }
 void on_frame(uint32_t) override {
   auto* data=get_owner()->data();auto& session=epok::dialogue::session();
   data->text.enabled=session.active();data->text.key=-1;data->text.set_text(session.text());data->text.visible_glyphs=uint16_t(session.snapshot().revealed);
   if(epok::input.held(epok::Button::Square))epok::time.set_paused(true);
   if(epok::input.held(epok::Button::Triangle))epok::time.set_paused(false);
 }
 void tick(epok::Fixed) override {if(epok::GameplayInputLibrary::take_pressed(epok::GameplayAction::Jump,0))get_owner()->data()->transform.rotation[2]+=epok::Fixed(1.0);}
};
"#.replace("COOKED_ROLE",&crate::dialogue_compile::compact(role).to_string())).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.clear();
        let player = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            crate::object_model::ACTOR3D_ID,
        )
        .unwrap();
        let label = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            crate::object_model::UI_ACTOR_ID,
        )
        .unwrap();
        use crate::actor_document::{ClassReference, ComponentInstance};
        let rules = uuid::Uuid::new_v4();
        scene.actors[player].components.push(ComponentInstance::new(
            rules,
            ClassReference::new(
                "epok::InteractableComponent",
                "310e882c-2353-55bc-bba3-f3d54277f671",
            ),
            "Rules",
        ));
        scene.actors[label].components.push(ComponentInstance::new(
            uuid::Uuid::new_v4(),
            ClassReference::new(
                "epok::TextComponent",
                crate::object_model::TEXT_COMPONENT_ID,
            ),
            "Text",
        ));
        let mut probe = ComponentInstance::new(
            uuid::Uuid::new_v4(),
            ClassReference::new("DialogueProbe", "23a292e7-7d93-5e4c-afb7-e4134800cc24"),
            "Dialogue",
        );
        probe.properties.insert(
            "initiator".into(),
            serde_json::json!(scene.actors[player].id),
        );
        probe.overrides.insert("initiator".into());
        probe
            .properties
            .insert("rules".into(), serde_json::json!(rules));
        probe.overrides.insert("rules".into());
        probe
            .properties
            .insert("conversation".into(), serde_json::json!(conversation));
        probe.overrides.insert("conversation".into());
        probe
            .properties
            .insert("style".into(), serde_json::json!(style));
        probe.overrides.insert("style".into());
        scene.actors[label].components.push(probe);
        scene.actors[label].refresh_components();
        scene.actors[label].rect.as_mut().unwrap().size = [64., 64.];
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        let initial = session.receive(Duration::from_secs(10)).unwrap();
        assert_eq!(initial.actors[label].rotation[1], 0, "session accepted");
        assert_eq!(initial.stats[1], 0);
        assert_eq!(step(&mut session, 9_000, None, &[]).stats[1], 0);
        assert_eq!(step(&mut session, 1_000, None, &[]).stats[1], 1);
        use crate::controls::Button;
        assert_eq!(
            step(&mut session, 10_000, Some(Button::Square), &[]).stats[1],
            2
        );
        assert_eq!(
            step(&mut session, 10_000, None, &[]).stats[1],
            3,
            "reveal uses unscaled frame time while paused"
        );
        assert_eq!(
            step(&mut session, 10_000, Some(Button::Cross), &[]).stats[1],
            4,
            "first confirm completes the page"
        );
        step(&mut session, 16_667, None, &[]);
        let closed = step(&mut session, 16_667, Some(Button::Cross), &[]);
        assert_eq!(closed.stats[1], 0);
        assert_eq!(
            closed.actors[label].rotation[0], 16384,
            "one Line, Branch, Event and End entry from cooked assets"
        );
        assert_eq!(
            closed.actors[label].rotation[2], 0,
            "closing confirm cannot jump"
        );
        step(&mut session, 16_667, Some(Button::Triangle), &[]);
        let resumed = step(&mut session, 16_667, Some(Button::Cross), &[]);
        assert_eq!(
            resumed.actors[label].rotation[2], 4096,
            "new press reaches gameplay after closure"
        );
        assert_eq!(resumed.stats[4], 0);
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dialogue_helpers_open_from_interaction_present_choices_and_release_control() {
        let root = crate::workspace::tests::temp("dialogue-helper-workflow");
        crate::workspace::create(&root, "Dialogue Helpers", crate::workspace::Template::Basic)
            .unwrap();
        fs::write(root.join("assets/scripts/HelperProbe.hpp"),r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="52a44e11-ad01-5c94-a6b3-87f7da1b84ef") DialogueTargetProbe:public epok::DialogueInteractionComponent {
public:
 void dialogue_started(epok::ObjectId initiator,uint32_t port,epok::DialogueHandle handle) override {epok::DialogueInteractionComponent::dialogue_started(initiator,port,handle);get_owner()->data()->transform.rotation[0]+=epok::Fixed(1.0);}
 void dialogue_node_entered(uint32_t node) override {epok::DialogueInteractionComponent::dialogue_node_entered(node);get_owner()->data()->transform.rotation[1]+=epok::Fixed(1.0);}
 void dialogue_finished(epok::ObjectId initiator,uint32_t port,epok::DialogueResult result) override {epok::DialogueInteractionComponent::dialogue_finished(initiator,port,result);get_owner()->data()->transform.rotation[2]+=epok::Fixed(1.0);}
};
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="8062ff45-182b-5805-85c8-90aa15ab0119") DialoguePlayerProbe:public epok::ActorComponent {
public:
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::DialogueInteractionComponent") epok::ObjectId target{};
 void on_frame(uint32_t) override {
  if(epok::GameplayInputLibrary::take_frame_pressed(epok::GameplayAction::Attack,0)){
   const auto started=epok::DialogueSessionLibrary::start(target,owner_id(),0);
   get_owner()->data()->transform.rotation[0]=epok::Fixed(int(started.result),0);
  }
  if(epok::input.held(epok::Button::Triangle))if(auto* source=epok::active_object_registry->resolve<epok::DialogueInteractionComponent>(target))source->set_enabled(false);
  get_owner()->data()->transform.rotation[1]=epok::Fixed(epok::DialogueSessionLibrary::valid(epok::DialogueSessionLibrary::owner_session(target))?1:0,0);
 }
 void tick(epok::Fixed) override {if(epok::GameplayInputLibrary::take_pressed(epok::GameplayAction::Jump,0))get_owner()->data()->transform.rotation[2]+=epok::Fixed(1.0);}
};
"#).unwrap();
        use crate::{
            actor_document::{ClassReference, ComponentInstance},
            dialogue_asset as da, localization as loc,
        };
        let table = loc::create(
            &root,
            "assets/strings.epokasset",
            &loc::StringTable {
                pageable: true,
                entries: std::iter::once(loc::Entry {
                    key: "line".into(),
                    values: vec!["Hola".into()],
                })
                .chain((0..6).map(|i| loc::Entry {
                    key: format!("choice{i}"),
                    values: vec![format!("A{i}")],
                }))
                .collect(),
                ..Default::default()
            },
        )
        .unwrap();
        let line = uuid::Uuid::new_v4();
        let choice = uuid::Uuid::new_v4();
        let end = uuid::Uuid::new_v4();
        let conversation = da::create(
            &root,
            "assets/conversation.epokasset",
            &da::DialogueAsset {
                version: da::VERSION,
                entry: line,
                participants: vec![],
                bindings: vec![],
                nodes: vec![
                    da::Node {
                        id: line,
                        kind: da::NodeKind::Line {
                            speaker: None,
                            text: da::TextKey {
                                table,
                                key: "line".into(),
                            },
                            next: choice,
                            policy: Some(da::LinePolicy {
                                milliseconds_per_glyph: 10,
                                auto_advance_ms: None,
                            }),
                        },
                    },
                    da::Node {
                        id: choice,
                        kind: da::NodeKind::Choice {
                            speaker: None,
                            prompt: None,
                            options: (0..6)
                                .map(|i| da::ChoiceOption {
                                    id: uuid::Uuid::new_v4(),
                                    text: da::TextKey {
                                        table,
                                        key: format!("choice{i}"),
                                    },
                                    next: end,
                                    condition: None,
                                })
                                .collect(),
                        },
                    },
                    da::Node {
                        id: end,
                        kind: da::NodeKind::End,
                    },
                ],
            },
        )
        .unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.clear();
        let player = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            crate::object_model::ACTOR3D_ID,
        )
        .unwrap();
        let box_actor = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            crate::object_model::ACTOR3D_ID,
        )
        .unwrap();
        let presenter = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            "d40dc688-f7c0-50a3-9af8-7beaf1e3eea3",
        )
        .unwrap();
        let target_id = uuid::Uuid::new_v4();
        let mut target = ComponentInstance::new(
            target_id,
            ClassReference::new(
                "DialogueTargetProbe",
                "52a44e11-ad01-5c94-a6b3-87f7da1b84ef",
            ),
            "Dialogue",
        );
        target
            .properties
            .insert("conversation".into(), serde_json::json!(conversation));
        target.overrides.insert("conversation".into());
        target.properties.insert(
            "presenter".into(),
            serde_json::json!(scene.actors[presenter].id),
        );
        target.overrides.insert("presenter".into());
        scene.actors[box_actor].components.push(target);
        scene.actors[player].components.push(ComponentInstance::new(
            uuid::Uuid::new_v4(),
            ClassReference::new(
                "epok::InteractorComponent",
                "ebe82fcd-e9db-5de5-89a7-ed42f3a6d29a",
            ),
            "Interactor",
        ));
        let mut probe = ComponentInstance::new(
            uuid::Uuid::new_v4(),
            ClassReference::new(
                "DialoguePlayerProbe",
                "8062ff45-182b-5805-85c8-90aa15ab0119",
            ),
            "Probe",
        );
        probe
            .properties
            .insert("target".into(), serde_json::json!(target_id));
        probe.overrides.insert("target".into());
        scene.actors[player].components.push(probe);
        let mut insufficient = scene.clone();
        insufficient.hud_budget.glyphs = 1;
        assert!(
            prepare(&root, insufficient, false, None, &AtomicBool::new(false))
                .unwrap_err()
                .contains("Dialogue HUD budget")
        );
        let mut render_scene = scene.clone();
        let rendering = crate::settings::rendering(&root).unwrap();
        render_scene.display_size = [rendering.width, rendering.height];
        let capture = |name: &str, packet: &Packet| {
            if let Some(dir) = std::env::var_os("EPOK_DIALOGUE_EVIDENCE_DIR") {
                let dir = PathBuf::from(dir);
                fs::create_dir_all(&dir).unwrap();
                let rgba = crate::hud_native::render(&render_scene, &packet.commands, 0., 0);
                let file = fs::File::create(dir.join(format!("{name}.png"))).unwrap();
                let mut encoder = png::Encoder::new(
                    file,
                    u32::from(render_scene.display_size[0]),
                    u32::from(render_scene.display_size[1]),
                );
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(&rgba)
                    .unwrap();
            }
        };
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        let initial = session.receive(Duration::from_secs(10)).unwrap();
        assert_eq!(initial.stats[1], 0);
        use crate::controls::Button;
        let opened = step(&mut session, 16_667, Some(Button::Cross), &[]);
        assert_eq!(opened.actors[box_actor].rotation, [4096, 4096, 0]);
        assert_eq!(
            opened.actors[player].rotation[2], 0,
            "opening interaction cannot jump"
        );
        assert_eq!(step(&mut session, 10_000, None, &[]).stats[1], 1);
        let revealed = step(&mut session, 10_000, Some(Button::Cross), &[]);
        assert_eq!(
            revealed.stats[1], 5,
            "line plus the authored continuation glyph"
        );
        capture("classic-line", &revealed);
        step(&mut session, 10_000, None, &[]);
        let choices = step(&mut session, 10_000, Some(Button::Cross), &[]);
        assert_eq!(choices.stats[1], 8, "four labels");
        capture("classic-choices", &choices);
        step(&mut session, 16_667, None, &[]);
        let mut page = choices;
        for _ in 0..4 {
            page = step(&mut session, 16_667, Some(Button::Down), &[]);
            step(&mut session, 16_667, None, &[]);
        }
        assert_eq!(page.stats[1], 4, "second page holds two labels");
        capture("classic-choice-page-two", &page);
        let closed = step(&mut session, 16_667, Some(Button::Cross), &[]);
        assert_eq!(closed.actors[box_actor].rotation, [4096, 12288, 4096]);
        assert_eq!(closed.stats[1], 0);
        assert_eq!(closed.actors[player].rotation[2], 0, "closing cannot jump");
        step(&mut session, 16_667, None, &[]);
        let from_api = step(&mut session, 16_667, Some(Button::Square), &[]);
        assert_eq!(
            from_api.actors[player].rotation[0], 0,
            "reflected start accepted"
        );
        assert_eq!(from_api.actors[box_actor].rotation[0], 8192);
        let cancelled = step(&mut session, 16_667, Some(Button::Circle), &[]);
        assert_eq!(cancelled.actors[box_actor].rotation[2], 8192);
        assert_eq!(cancelled.stats[1], 0);
        step(&mut session, 16_667, Some(Button::Triangle), &[]);
        let resumed = step(&mut session, 16_667, Some(Button::Cross), &[]);
        assert_eq!(
            resumed.actors[player].rotation[2], 4096,
            "new gameplay press works after capture release"
        );
        assert_eq!(resumed.stats[4], 0);
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn authored_widget_focus_links_keep_identity_in_placed_and_spawned_classes() {
        let (root, scene, probe) = crate::class_designer::tests::focus_project();
        if let Some(destination) = std::env::var_os("EPOK_WIDGET_FOCUS_EVIDENCE_DIR") {
            fn copy(source: &Path, destination: &Path) {
                fs::create_dir_all(destination).unwrap();
                for entry in fs::read_dir(source).unwrap() {
                    let entry = entry.unwrap();
                    if entry.file_name() == ".epok" {
                        continue;
                    }
                    let target = destination.join(entry.file_name());
                    let kind = entry.file_type().unwrap();
                    assert!(!kind.is_symlink());
                    if kind.is_dir() {
                        copy(&entry.path(), &target);
                    } else {
                        fs::copy(entry.path(), target).unwrap();
                    }
                }
            }
            let destination = PathBuf::from(destination);
            assert!(!destination.exists());
            copy(&root, &destination);
        }
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        session.receive(Duration::from_secs(10)).unwrap();
        let mut frame = step(&mut session, 16_667, None, &[]);
        for _ in 0..3 {
            frame = step(&mut session, 16_667, None, &[]);
        }
        assert_eq!(frame.actors[probe].rotation, [7 * 4096, 9 * 4096, 4096]);
        assert_eq!(frame.stats[1], 18);
        assert_eq!(frame.stats[4], 0);
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn widget_focus_clears_destroyed_targets_before_slot_reuse_in_all_providers() {
        let (root, scene, probe) = crate::class_designer::tests::focus_lifetime_project();
        if let Some(destination) = std::env::var_os("EPOK_WIDGET_FOCUS_LIFETIME_EVIDENCE_DIR") {
            fn copy(source: &Path, destination: &Path) {
                fs::create_dir_all(destination).unwrap();
                for entry in fs::read_dir(source).unwrap() {
                    let entry = entry.unwrap();
                    if entry.file_name() == ".epok" {
                        continue;
                    }
                    let target = destination.join(entry.file_name());
                    let kind = entry.file_type().unwrap();
                    assert!(!kind.is_symlink());
                    if kind.is_dir() {
                        copy(&entry.path(), &target);
                    } else {
                        fs::copy(entry.path(), target).unwrap();
                    }
                }
            }
            let destination = PathBuf::from(destination);
            assert!(!destination.exists());
            copy(&root, &destination);
        }
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        session.receive(Duration::from_secs(10)).unwrap();
        let mut frame = step(&mut session, 16_667, None, &[]);
        for _ in 0..7 {
            frame = step(&mut session, 16_667, None, &[]);
        }
        assert_eq!(frame.actors[probe].rotation, [7 * 4096, 3 * 4096, 4096]);
        assert_eq!(frame.stats[1], 15);
        assert_eq!(frame.stats[4], 0);
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn authored_widget_components_keep_properties_and_typed_bindings_in_native_play() {
        let (root, scene, driver) = crate::class_designer::tests::component_project();
        if let Some(destination) = std::env::var_os("EPOK_WIDGET_COMPONENT_EVIDENCE_DIR") {
            fn copy(source: &Path, destination: &Path) {
                fs::create_dir_all(destination).unwrap();
                for entry in fs::read_dir(source).unwrap() {
                    let entry = entry.unwrap();
                    if entry.file_name() == ".epok" {
                        continue;
                    }
                    let target = destination.join(entry.file_name());
                    let kind = entry.file_type().unwrap();
                    assert!(!kind.is_symlink());
                    if kind.is_dir() {
                        copy(&entry.path(), &target);
                    } else {
                        fs::copy(entry.path(), target).unwrap();
                    }
                }
            }
            copy(&root, &PathBuf::from(destination));
        }
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        session.receive(Duration::from_secs(10)).unwrap();
        let frame = step(&mut session, 16_667, None, &[]);
        assert_eq!(frame.actors[driver].rotation, [7 * 4096, 6 * 4096, 4096]);
        assert_eq!(frame.stats[1], 18);
        assert_eq!(frame.stats[4], 0);
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn nested_widget_classes_keep_their_own_behavior_and_control_bindings_in_native_play() {
        use crate::{blueprint_templates as templates, class_composition as compositions};
        let (root, _, _) = compositions::tests::fixture();
        const BASE: &str = "301a291b-4772-4ab6-ae77-d6f973639242";
        const HOST: &str = "eb34d3af-eed3-41bd-a1be-315508df1318";
        const CPP: &str = "c1ee278c-27ae-562e-8738-b930f01e3c94";
        const DRIVER: &str = "7886bf3f-bfa0-575e-9f99-e1f28109c3da";
        fs::write(root.join("assets/scripts/CppNestedWidget.hpp"), r#"#pragma once
#include "WidgetClasses.hpp"
class EPOK_CLASS(Blueprintable,Id="c1ee278c-27ae-562e-8738-b930f01e3c94") CppNestedWidget:public CppWidgetBase {
public:
 void on_frame(uint32_t)override {if(auto* text=epok::active_object_registry->resolve<epok::TextComponent>(label))text->set_number(101);}
};
"#).unwrap();
        let base_header = "WidgetClasses.hpp";
        fs::write(
            root.join("assets/scripts/LuaWidget.lua"),
            r#"---@class LuaWidget : CppWidgetBase
local LuaWidget = CppWidgetBase:extend()
---@override
function LuaWidget:on_frame(frame_microseconds)
    self.label:set_number(303)
end
return LuaWidget
"#,
        )
        .unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let frame = registry
            .ancestry("CppWidgetBase")
            .into_iter()
            .flat_map(|class| class.functions.iter())
            .find(|f| f.name == "on_frame")
            .unwrap()
            .clone();
        let setter = registry
            .ancestry("epok::TextComponent")
            .into_iter()
            .flat_map(|class| class.functions.iter())
            .find(|f| f.name == "set_number")
            .unwrap()
            .clone();
        let path = root.join("assets/Blueprints/BP_Widget.epokbp");
        let mut bp = crate::blueprint_asset::load(&path).unwrap();
        let [entry, label, set, done] =
            std::array::from_fn::<_, 4, _>(|_| uuid::Uuid::new_v4().to_string());
        let mut graph = serde_json::json!({"id":uuid::Uuid::new_v4(),"name":"on_frame","override_id":frame.id,"parameters":frame.parameters,"returns":frame.returns,"entry":entry,"nodes":[
            {"id":entry,"kind":{"kind":"entry"},"outputs":{"next":[set]}},
            {"id":label,"kind":{"kind":"get_variable","member":"47ec3265-ddab-4d28-87c5-6915dd3d7a32"}},
            {"id":set,"kind":{"kind":"call_on","class":crate::object_model::TEXT_COMPONENT_ID,"function":setter.id},"inputs":{"__target":{"kind":"link","node":label,"pin":"value"}},"outputs":{"next":[done]}},
            {"id":done,"kind":{"kind":"return"}}
        ]});
        graph["nodes"][2]["inputs"][&setter.parameters[0].name] = serde_json::json!({"kind":"literal","value_type":setter.parameters[0].value_type,"value":202});
        bp.functions.push(serde_json::from_value(graph).unwrap());
        fs::write(path, crate::document::to_vec(&bp).unwrap()).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let base = compositions::tests::base_template(&root, &registry);
        let parent = base.actors[0].entity.id;
        compositions::create(
            &root,
            "assets/WidgetBase.epokclass",
            &compositions::Document::new(BASE.into(), base),
            &registry,
        )
        .unwrap();
        let model = registry.model().unwrap();
        let classes = [
            CPP.to_owned(),
            bp.id,
            registry.named("LuaWidget").unwrap().id.clone(),
        ];
        let mut template = templates::Template::default();
        let mut sites = vec![];
        for (n, class) in classes.iter().enumerate() {
            sites.push(
                crate::blueprint_editor::template_editor::add_class_instance(
                    &mut template,
                    &model,
                    parent,
                    class,
                    &format!("Nested {n}"),
                )
                .unwrap(),
            );
        }
        let sources = compositions::sources(&root, &registry).unwrap();
        let binding = || crate::scene::ClassDefaults {
            name: "CppWidget".into(),
            class_id: Some(HOST.into()),
            ..Default::default()
        };
        let inherited = compositions::inherited(&root, &registry, BASE).unwrap();
        let mut layers = inherited.clone();
        layers.push(template.clone());
        let scene = sources
            .resolve_layers(&registry, HOST, &layers.iter().collect::<Vec<_>>())
            .unwrap()
            .scene(binding(), &registry)
            .unwrap();
        for (n, site) in sites.iter().enumerate() {
            let original = scene.actors.iter().find(|a| a.id == *site).unwrap();
            let mut edited = original.clone();
            let rect = edited.rect.as_mut().unwrap();
            rect.anchor_min = [0.; 2];
            rect.anchor_max = [0.; 2];
            rect.pivot = [0.; 2];
            rect.position = [10. + n as f32 * 200., 40.];
            rect.size = [160., 100.];
            crate::actor_components::sync(&mut edited);
            crate::blueprint_editor::template_editor::apply_entity_changes(
                &mut template,
                original,
                &edited,
            )
            .unwrap();
        }
        compositions::create(
            &root,
            "assets/CppWidget.epokclass",
            &compositions::Document::new(HOST.into(), template),
            &registry,
        )
        .unwrap();
        let ids = classes
            .iter()
            .map(|id| format!("UINT64_C({})", crate::blueprint_refs::compact_id(id)))
            .collect::<Vec<_>>()
            .join(",");
        let driver=r#"#pragma once
#include "Widget.hpp"
extern "C" { inline uint32_t nested_widget_probe[5]={}; }
class EPOK_CLASS(Blueprintable,Id="7886bf3f-bfa0-575e-9f99-e1f28109c3da") NestedWidgetProbe:public epok::Actor3D {
 static bool equal(const char* a,const char* b){while(*a&&*a==*b){++a;++b;}return *a==*b;}
public:
 void on_frame(uint32_t)override {
  auto* level=epok::active_object_registry->resolve<epok::Level>(level_id());const uint64_t classes[]={CLASSES};
  const char* labels[]={"101","202","303"};epok::ObjectId controls[8];unsigned count=0,mask=0;bool valid=true;
  for(size_t i=0;i<level->actor_count();++i){auto* actor=epok::active_object_registry->resolve<epok::Actor>(level->actor_at(i));if(!actor)continue;
   for(unsigned n=0;n<3;++n)if(actor->class_id()==classes[n]){auto* widget=static_cast<CppWidgetBase*>(actor);auto* control=epok::active_object_registry->resolve<epok::TextComponent>(widget->label);auto* owner=control?control->get_owner():nullptr;
    if(owner&&equal(owner->data()->text.value,labels[n]))mask|=1u<<n;
    valid&=owner&&owner->data()->parent==epok::entity_index(widget->data())&&widget->data()->rect.size[0]==160.0&&widget->data()->rect.size[1]==100.0;
    for(unsigned j=0;j<count;++j)valid&=controls[j]!=widget->label;if(count<8)controls[count++]=widget->label;else valid=false;
   }
  }
  data()->transform.rotation[0]=epok::Fixed(int(mask),0);data()->transform.rotation[1]=epok::Fixed(int(count),0);data()->transform.rotation[2]=epok::Fixed(valid?1:0,0);
  nested_widget_probe[0]=mask;nested_widget_probe[1]=count;nested_widget_probe[2]=valid;
  nested_widget_probe[3]=epok::hud_stats.glyphs;nested_widget_probe[4]=epok::hud_stats.dropped;
 }
};
"#.replace("Widget.hpp",base_header).replace("CLASSES",&ids);
        fs::write(root.join("assets/scripts/NestedWidgetProbe.hpp"), driver).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.truncate(1);
        let host = crate::editor::place_class(&mut scene, &root, &registry, &model, HOST).unwrap();
        scene.duplicate_actor_branch(scene.actors[host].id).unwrap();
        let driver =
            crate::editor::place_class(&mut scene, &root, &registry, &model, DRIVER).unwrap();
        if let Some(destination) = std::env::var_os("EPOK_WIDGET_EVIDENCE_DIR") {
            // Preserve ordinary project sources and the placed scene for the
            // target/export acceptance harness, without copying native caches.
            fn copy_sources(source: &Path, destination: &Path) {
                fs::create_dir_all(destination).unwrap();
                for entry in fs::read_dir(source).unwrap() {
                    let entry = entry.unwrap();
                    if entry.file_name() == ".epok" {
                        continue;
                    }
                    let target = destination.join(entry.file_name());
                    let kind = entry.file_type().unwrap();
                    assert!(!kind.is_symlink());
                    if kind.is_dir() {
                        copy_sources(&entry.path(), &target);
                    } else {
                        fs::copy(entry.path(), target).unwrap();
                    }
                }
            }
            scene
                .save(&crate::workspace::startup_scene(&root).unwrap())
                .unwrap();
            copy_sources(&root, &PathBuf::from(destination));
        }
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        session.receive(Duration::from_secs(10)).unwrap();
        let mut frame = step(&mut session, 16_667, None, &[]);
        for _ in 0..3 {
            if frame.actors[driver].rotation == [7 * 4096, 6 * 4096, 4096] {
                break;
            }
            frame = step(&mut session, 16_667, None, &[]);
        }
        assert_eq!(frame.actors[driver].rotation, [7 * 4096, 6 * 4096, 4096]);
        assert_eq!(frame.stats[1], 46);
        assert_eq!(frame.stats[4], 0);
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_widget_classes_spawn_with_authored_pagination_and_anchors_in_native_play() {
        use crate::{
            actor_document::{ClassReference, ComponentInstance},
            dialogue_asset as da,
            workspace::GameplayFlavor,
        };
        let temporary = crate::workspace::tests::temp("dialogue-widget-spawn");
        let root = temporary.join("Nested/Authoring/LongProject");
        fs::create_dir_all(root.parent().unwrap()).unwrap();
        crate::workspace::create(
            &root,
            "Dialogue Widget Spawn",
            crate::workspace::Template::Basic,
        )
        .unwrap();
        let mut ids = vec![];
        for (n, flavor) in GameplayFlavor::ALL.into_iter().enumerate() {
            let id = crate::dialogue_widget::create(
                &root,
                &format!("Widget{}", flavor.key()),
                flavor,
                da::Presentation::SpeechBubble,
                [320, 240],
            )
            .unwrap();
            let edit = |template: &mut crate::blueprint_templates::Template| {
                let control = template
                    .actors
                    .iter_mut()
                    .find(|a| a.entity.name == "Dialogue body")
                    .unwrap();
                let rect = control.entity.rect.as_mut().unwrap();
                rect.anchor_max = [0.; 2];
                rect.position = [24. + n as f32 * 8., 104.];
                rect.size = [88. + n as f32 * 8., 16.];
                crate::actor_components::sync(&mut control.entity);
            };
            if flavor == GameplayFlavor::Blueprint {
                let path = root.join(format!("assets/Blueprints/Widget{}.epokbp", flavor.key()));
                let mut file = crate::blueprint_asset::load(&path).unwrap();
                edit(&mut file.template);
                fs::write(&path, crate::document::to_vec(&file).unwrap()).unwrap();
            } else {
                let source = crate::class_composition::load(&root)
                    .unwrap()
                    .sources
                    .remove(&id)
                    .unwrap();
                let mut file = source.document.clone();
                edit(&mut file.template);
                let catalog = crate::scripts::catalog(&root).unwrap();
                let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
                crate::class_composition::save(&root, &source, &file, &registry).unwrap();
            }
            ids.push(crate::blueprint_refs::compact_id(&id));
        }
        let driver=r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="a25e4552-1930-41cd-9501-8cb0e038ad3d") WidgetSpawnProbe:public epok::ActorComponent {
 unsigned phase=0,index=0;epok::ObjectId widget{};epok::DialogueHandle handle{};
public:
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::DialogueInteractionComponent") epok::ObjectId target{};
 void on_frame(uint32_t) override {
  if(index>=3)return;const uint64_t classes[]={CLASSES};
  auto* registry=epok::active_object_registry;auto* level=registry->resolve<epok::Level>(get_owner()->level_id());
  auto* source=registry->resolve<epok::DialogueInteractionComponent>(target);
  if(phase==0){level->spawn_actor(*epok::find_object_class(classes[index]),nullptr);phase=1;return;}
  if(phase==1){for(size_t i=0;i<level->actor_count();++i){auto* value=registry->resolve<epok::Actor>(level->actor_at(i));if(value&&value->class_id()==classes[index]){widget=value->id();source->presenter=widget;const auto result=epok::DialogueSessionLibrary::start(target,owner_id(),0);handle=result.session;get_owner()->data()->transform.rotation[0]=epok::Fixed(int(result.result),0);phase=2;break;}}return;}
  if(epok::input.held(epok::Button::Square)){epok::DialogueSessionLibrary::cancel(handle);level->destroy_actor(widget);phase=0;++index;get_owner()->data()->transform.rotation[1]=epok::Fixed(int(index),0);}
 }
};
"#.replace("CLASSES",&ids.iter().map(|id|format!("UINT64_C({id})")).collect::<Vec<_>>().join(","));
        fs::write(root.join("assets/scripts/WidgetSpawnProbe.hpp"), driver).unwrap();
        let table = crate::localization::create(
            &root,
            "assets/Words.epokasset",
            &crate::localization::StringTable {
                pageable: true,
                entries: vec![crate::localization::Entry {
                    key: "line".into(),
                    values: vec!["A".repeat(40)],
                }],
                ..Default::default()
            },
        )
        .unwrap();
        let line = uuid::Uuid::new_v4();
        let end = uuid::Uuid::new_v4();
        let conversation = da::create(
            &root,
            "assets/Talk.epokasset",
            &da::DialogueAsset {
                version: da::VERSION,
                entry: line,
                participants: vec![],
                bindings: vec![],
                nodes: vec![
                    da::Node {
                        id: line,
                        kind: da::NodeKind::Line {
                            speaker: None,
                            text: da::TextKey {
                                table,
                                key: "line".into(),
                            },
                            next: end,
                            policy: Some(da::LinePolicy {
                                milliseconds_per_glyph: 0,
                                auto_advance_ms: None,
                            }),
                        },
                    },
                    da::Node {
                        id: end,
                        kind: da::NodeKind::End,
                    },
                ],
            },
        )
        .unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.clear();
        let owner = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            crate::object_model::ACTOR3D_ID,
        )
        .unwrap();
        let target = uuid::Uuid::new_v4();
        let mut component = ComponentInstance::new(
            target,
            ClassReference::new(
                "epok::DialogueInteractionComponent",
                "aa343ddf-5098-5b32-b993-d8b9b619d455",
            ),
            "Dialogue",
        );
        component
            .properties
            .insert("conversation".into(), serde_json::json!(conversation));
        component
            .properties
            .insert("automatic_input".into(), serde_json::json!(false));
        component
            .overrides
            .extend(component.properties.keys().cloned());
        scene.actors[owner].components.push(component);
        let mut probe = ComponentInstance::new(
            uuid::Uuid::new_v4(),
            ClassReference::new("WidgetSpawnProbe", "a25e4552-1930-41cd-9501-8cb0e038ad3d"),
            "Probe",
        );
        probe
            .properties
            .insert("target".into(), serde_json::json!(target));
        probe.overrides.insert("target".into());
        scene.actors[owner].components.push(probe);
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        session.receive(Duration::from_secs(10)).unwrap();
        for (n, glyphs) in [11, 12, 13].into_iter().enumerate() {
            let mut packet = step(&mut session, 16_667, None, &[]);
            for _ in 0..4 {
                if packet.stats[1] == glyphs + 1 {
                    break;
                }
                packet = step(&mut session, 16_667, None, &[]);
            }
            assert_eq!(
                packet.actors[owner].rotation[0], 0,
                "start accepted for provider {n}"
            );
            assert_eq!(
                packet.stats[1],
                glyphs + 1,
                "widget {n} uses its own cooked control width; slots {}, source {:?}, HUD {:?}",
                packet.actor_count,
                packet.actors[owner].rotation,
                packet.stats
            );
            assert_eq!(packet.stats[4], 0);
            let closed = step(
                &mut session,
                16_667,
                Some(crate::controls::Button::Square),
                &[],
            );
            assert_eq!(closed.stats[1], 0);
            assert_eq!(closed.actors[owner].rotation[1], (n as i32 + 1) * 4096);
        }
        drop(session);
        fs::remove_dir_all(temporary).unwrap();
    }

    #[test]
    fn legacy_controller_oracles_compile_independently_of_the_shared_starter() {
        use crate::workspace::{GameplayFlavor, Template};
        for flavor in GameplayFlavor::ALL {
            let root = crate::workspace::tests::temp("isolated-controller-oracle");
            crate::workspace::create(&root, "Controller Oracle", Template::Basic).unwrap();
            match flavor {
                GameplayFlavor::Cpp => {
                    for (name, source) in [
                        (
                            "ThirdPersonController.hpp",
                            include_str!("../templates/ThirdPersonController.hpp"),
                        ),
                        (
                            "ThirdPersonController.cpp",
                            include_str!("../templates/ThirdPersonController.cpp"),
                        ),
                        (
                            "CharacterMotion.hpp",
                            include_str!("../templates/CharacterMotion.hpp"),
                        ),
                        (
                            "CharacterClips.hpp",
                            include_str!("../templates/CharacterClips.hpp"),
                        ),
                    ] {
                        fs::write(root.join("assets/scripts").join(name), source).unwrap();
                    }
                }
                GameplayFlavor::Blueprint => {
                    let path = root.join(crate::third_person_blueprint::ASSET_PATH);
                    fs::create_dir_all(path.parent().unwrap()).unwrap();
                    crate::blueprint_asset::create(&path, &crate::third_person_blueprint::asset())
                        .unwrap();
                }
                GameplayFlavor::Lua => {
                    fs::write(
                        root.join(crate::third_person::LUA_CONTROLLER_PATH),
                        include_str!("../templates/ThirdPersonController.lua"),
                    )
                    .unwrap();
                    let mut identities = crate::lua_identity::Registry::default();
                    identities.classes.insert(
                        crate::third_person::LUA_CONTROLLER_PATH.into(),
                        crate::third_person::LUA_CONTROLLER_ID.into(),
                    );
                    identities.save(&root).unwrap();
                }
            }
            let scene = Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
            assert_eq!(
                scene.actors.len(),
                2,
                "the fixture is independent of Third Person scene generation"
            );
            let exe = prepare(&root, scene, false, None, &AtomicBool::new(false))
                .unwrap_or_else(|e| panic!("{flavor:?}: {e}"));
            let mut session = Session::launch(&exe).unwrap();
            session.receive(Duration::from_secs(10)).unwrap();
            let packet = step(&mut session, 16_667, None, &[]);
            assert!(packet.actors[1].alive);
            drop(session);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn dungeon_project_rules_use_explicit_ownership_once_and_reset_in_every_provider() {
        use crate::actor_document::{ClassReference, ComponentInstance};
        use crate::workspace::{CreateOptions, GameplayFlavor, Template};
        for flavor in GameplayFlavor::ALL {
            let root = crate::workspace::tests::temp("dungeon-policy-native");
            crate::workspace::create_with_options(
                &root,
                "Dungeon Policy",
                CreateOptions::new(Template::Dungeon).with_gameplay(flavor),
            )
            .unwrap();
            fs::write(
                root.join("assets/scripts/DungeonPolicyProbe.hpp"),
                include_str!("../tests/fixtures/native-scenes/DungeonPolicyProbe.hpp")
                    .replace("DUNGEON_POLICY_MAGIC", "1778384896"),
            )
            .unwrap();
            let installed = crate::template_install::read(&root).unwrap().unwrap();
            let mut scene = Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
            let generator = &scene.dungeons[0];
            let player = generator.spawn_actor.unwrap();
            let observer = scene
                .actors
                .iter()
                .position(|a| a.id == generator.root)
                .unwrap();
            let pickup = scene
                .actors
                .iter()
                .flat_map(|a| &a.components)
                .find(|c| c.class == installed.roles["key"])
                .unwrap();
            let flag = pickup.properties["collected_flag"].clone();
            let door = scene
                .actors
                .iter()
                .flat_map(|a| &a.components)
                .find(|c| {
                    c.class == installed.roles["door"] && c.properties["requirement_flag"] == flag
                })
                .unwrap()
                .id;
            let mut probe = ComponentInstance::new(
                uuid::Uuid::new_v4(),
                ClassReference::new("DungeonPolicyProbe", "795a13bd-6614-5b87-a537-a1a658aa2edf"),
                "Policy observer",
            );
            probe.properties.extend([
                ("player".into(), serde_json::json!(player)),
                ("pickup".into(), serde_json::json!(pickup.id)),
                ("flag".into(), flag),
                ("door".into(), serde_json::json!(door)),
            ]);
            probe.overrides.extend(probe.properties.keys().cloned());
            scene.actors[observer].components.push(probe);
            for (i, actor) in scene.actors.iter_mut().enumerate() {
                actor.name = format!("Renamed Dungeon actor {i}");
            }
            let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
            let mut session = Session::launch(&exe).unwrap();
            session.receive(Duration::from_secs(10)).unwrap();
            let mut packet = step(&mut session, 16_667, None, &[]);
            for _ in 0..11 {
                packet = step(&mut session, 16_667, None, &[]);
            }
            assert_eq!(
                &packet.actors[observer].rotation[..2],
                &[0, 4 * 4096],
                "{flavor:?}"
            );
            drop(session);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn platformer_pickups_validate_the_player_and_repeat_project_policy_after_reset() {
        use crate::actor_document::{ClassReference, ComponentInstance};
        use crate::workspace::{CreateOptions, GameplayFlavor, Template};
        for flavor in GameplayFlavor::ALL {
            let root = crate::workspace::tests::temp("platformer-pickup-native");
            crate::workspace::create_with_options(
                &root,
                "Pickup Policy",
                CreateOptions::new(Template::Platformer2D).with_gameplay(flavor),
            )
            .unwrap();
            fs::write(
                root.join("assets/scripts/PlatformerPickupPolicyProbe.hpp"),
                include_str!("../tests/fixtures/native-scenes/PlatformerPickupPolicyProbe.hpp"),
            )
            .unwrap();
            let installed = crate::template_install::read(&root).unwrap().unwrap();
            let mut scene = Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
            let player = scene
                .actors
                .iter()
                .find(|a| a.class == installed.roles["player"])
                .unwrap()
                .id;
            let pickup = scene
                .actors
                .iter()
                .flat_map(|a| &a.components)
                .find(|c| c.class == installed.roles["pickup"])
                .unwrap()
                .id;
            for (i, actor) in scene.actors.iter_mut().enumerate() {
                actor.name = format!("Renamed platformer actor {i}");
            }
            let mut marker = crate::world2d::create(
                crate::world2d::Preset::Node2D,
                "Policy observation",
                [0., 0.],
            );
            let mut probe = ComponentInstance::new(
                uuid::Uuid::new_v4(),
                ClassReference::new(
                    "PlatformerPickupPolicyProbe",
                    "061e2973-0830-5447-a720-172e7d7bfe8f",
                ),
                "Probe",
            );
            probe.properties.extend([
                ("player".into(), serde_json::json!(player)),
                ("pickup".into(), serde_json::json!(pickup)),
            ]);
            probe.overrides.extend(probe.properties.keys().cloned());
            marker.components.push(probe);
            marker.refresh_components();
            let index = scene.actors.len();
            scene.actors.push(marker);
            let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
            let mut session = Session::launch(&exe).unwrap();
            session.receive(Duration::from_secs(10)).unwrap();
            let packet = step(&mut session, 16_667, None, &[]);
            assert_eq!(
                &packet.actors[index].position[..2],
                &[11 * 4096, 11 * 4096],
                "{flavor:?}"
            );
            drop(session);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn third_person_template_roles_keep_native_play_movement_jump_and_camera_after_renaming() {
        use crate::workspace::{CreateOptions, GameplayFlavor, Template};
        let mut outcomes = Vec::new();
        for flavor in GameplayFlavor::ALL {
            let root = crate::workspace::tests::temp("third-person-roles-native");
            let project = crate::workspace::create_with_options(
                &root,
                "Shared Player",
                CreateOptions::new(Template::ThirdPerson).with_gameplay(flavor),
            )
            .unwrap();
            let installed = crate::template_install::read(&root).unwrap().unwrap();
            let mut scene = Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
            let player = scene
                .actors
                .iter()
                .position(|a| a.class == installed.roles["player"])
                .unwrap();
            let camera = scene
                .actors
                .iter()
                .position(|a| a.class == installed.roles["camera"])
                .unwrap();
            for (i, actor) in scene.actors.iter_mut().enumerate() {
                actor.name = format!("Renamed native actor {i}");
            }
            let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
            let mut session = Session::launch(&exe).unwrap();
            session.receive(Duration::from_secs(10)).unwrap();
            let initial = step(&mut session, 16_667, None, &[]);
            let start = initial.actors[player].position;
            let mut moved = step(&mut session, 16_667, Some(crate::controls::Button::Up), &[]);
            for _ in 1..30 {
                moved = step(&mut session, 16_667, Some(crate::controls::Button::Up), &[]);
            }
            assert!(moved.actors[player].alive && moved.actors[camera].alive);
            assert!(
                moved.actors[player].position[2] > start[2] + 4096,
                "{flavor:?}: movement reaches the shared motor"
            );
            assert_ne!(
                moved.actors[camera].position, initial.actors[camera].position,
                "camera follows its typed player reference"
            );
            let mut jumping = step(
                &mut session,
                16_667,
                Some(crate::controls::Button::Cross),
                &[],
            );
            for _ in 0..9 {
                jumping = step(&mut session, 16_667, None, &[]);
            }
            assert!(
                jumping.actors[player].position[1] > start[1] + 819,
                "{flavor:?}: jump reaches the shared motor"
            );
            outcomes.push((
                moved.actors[player].position,
                jumping.actors[player].position,
                jumping.actors[camera].position,
            ));
            drop(session);
            drop(project);
            fs::remove_dir_all(root).unwrap();
        }
        assert!(
            outcomes.iter().all(|value| *value == outcomes[0]),
            "{outcomes:?}"
        );
    }

    #[test]
    fn small_starters_run_selected_classes_and_instance_tuning_in_native_play() {
        use crate::workspace::{CreateOptions, GameplayFlavor, Template};
        for template in [Template::Basic, Template::Sample] {
            let mut outcomes = Vec::new();
            for flavor in GameplayFlavor::ALL {
                let root = crate::workspace::tests::temp("small-starter-native-play");
                let project = crate::workspace::create_with_options(
                    &root,
                    "Small Starter",
                    CreateOptions::new(template).with_gameplay(flavor),
                )
                .unwrap();
                let installed = crate::template_install::read(&root).unwrap().unwrap();
                let role = &installed.roles[if template == Template::Basic {
                    "game_root"
                } else {
                    "spinner"
                }];
                let mut scene =
                    Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
                let target = if template == Template::Basic {
                    scene
                        .actors
                        .iter()
                        .position(|a| a.class.class_id == role.class_id)
                        .unwrap()
                } else {
                    let index = scene
                        .actors
                        .iter()
                        .position(|a| {
                            a.components
                                .iter()
                                .any(|c| c.class.class_id == role.class_id)
                        })
                        .unwrap();
                    let spinner = scene.actors[index]
                        .components
                        .iter_mut()
                        .find(|c| c.class.class_id == role.class_id)
                        .unwrap();
                    spinner
                        .properties
                        .insert("speed".into(), serde_json::json!(90.0));
                    spinner.overrides.insert("speed".into());
                    index
                };
                let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
                let mut session = Session::launch(&exe).unwrap();
                session.receive(Duration::from_secs(10)).unwrap();
                let mut packet = step(&mut session, 16_667, None, &[]);
                for _ in 1..30 {
                    packet = step(&mut session, 16_667, None, &[]);
                }
                assert!(packet.actors[target].alive);
                let rotation = packet.actors[target].rotation;
                if template == Template::Basic {
                    assert_eq!(packet.actors[target].position, [0; 3]);
                    assert_eq!(rotation, [0; 3]);
                } else {
                    assert_eq!([rotation[0], rotation[2]], [0; 2]);
                    assert!(
                        (40 * 4096..50 * 4096).contains(&rotation[1]),
                        "{flavor:?}: {rotation:?}"
                    );
                    assert_eq!(
                        packet.actors[2].rotation, [0; 3],
                        "the untuned second cube remains independent"
                    );
                }
                outcomes.push(rotation);
                drop(session);
                drop(project);
                fs::remove_dir_all(root).unwrap();
            }
            assert!(
                outcomes.iter().all(|value| *value == outcomes[0]),
                "{template:?}: {outcomes:?}"
            );
        }
    }

    #[test]
    fn world_pickups_collect_once_through_native_physics_in_both_domains() {
        let root = crate::workspace::tests::temp("world-pickup-physics");
        crate::workspace::create(&root, "Pickup Physics", crate::workspace::Template::Basic)
            .unwrap();
        fs::write(root.join("assets/scripts/AutoPickupProbe.hpp"), r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="593ae57a-2319-51fa-a766-33ff587b912c") AutoPickupProbe:public epok::PickupComponent {
public:
 void collected(epok::ObjectId initiator,uint32_t)override{
  auto* actor=epok::active_object_registry->resolve<epok::Actor>(initiator);
  if(actor)actor->data()->transform.rotation[0]+=epok::Fixed(1.0);
 }
};
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="908929b6-1426-53d5-84f9-173d8a787338") PickupStateProbe:public epok::ActorComponent {
public:
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::GameplayFlagComponent") epok::ObjectId flag{};
 void on_frame(uint32_t)override{
  auto* state=epok::active_object_registry->resolve<epok::GameplayFlagComponent>(flag);
  get_owner()->data()->transform.rotation[1]=state&&state->value()?epok::Fixed(1.0):epok::Fixed(0.0);
 }
};
"#).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        for domain in [3, 2] {
            let mut scene = Scene::default();
            scene.actors.clear();
            let class = if domain == 3 {
                crate::object_model::ACTOR3D_ID
            } else {
                crate::object_model::ACTOR2D_ID
            };
            let collider = if domain == 3 {
                crate::object_model::COLLIDER3D_COMPONENT_ID
            } else {
                "aad9ca37-7c63-4ca9-8a23-5f4ff49e35fe"
            };
            let mut actors = Vec::new();
            for _ in 0..3 {
                let index = crate::editor::place_class(&mut scene, &root, &registry, &model, class)
                    .unwrap();
                crate::actor_components::add(&mut scene.actors[index], &model, collider).unwrap();
                actors.push(index);
            }
            let (source, foreign, pickup) = (actors[0], actors[1], actors[2]);
            let flag = crate::actor_components::add(
                &mut scene.actors[source],
                &model,
                "epok::GameplayFlagComponent",
            )
            .unwrap();
            let observer =
                crate::actor_components::add(&mut scene.actors[source], &model, "PickupStateProbe")
                    .unwrap();
            let endpoint =
                crate::actor_components::add(&mut scene.actors[pickup], &model, "AutoPickupProbe")
                    .unwrap();
            let source_id = scene.actors[source].id;
            let probe = scene.actors[source]
                .components
                .iter_mut()
                .find(|c| c.id == observer)
                .unwrap();
            probe
                .properties
                .insert("flag".into(), serde_json::json!(flag));
            probe.overrides.insert("flag".into());
            let probe = scene.actors[pickup]
                .components
                .iter_mut()
                .find(|c| c.id == endpoint)
                .unwrap();
            for (key, value) in [
                ("accepted_initiator", serde_json::json!(source_id)),
                ("collected_flag", serde_json::json!(flag)),
                ("auto_collect", serde_json::json!(true)),
                ("deactivate_on_collect", serde_json::json!(true)),
            ] {
                probe.properties.insert(key.into(), value);
                probe.overrides.insert(key.into());
            }
            if domain == 3 {
                for actor in &mut scene.actors {
                    actor.collider.as_mut().unwrap().enabled = true;
                }
                scene.actors[pickup].collider.as_mut().unwrap().trigger = true;
            } else {
                let shape = scene.actors[pickup]
                    .components
                    .iter_mut()
                    .find(|c| c.class.class_id.as_deref() == Some(collider))
                    .unwrap();
                crate::world2d::write(
                    shape,
                    &crate::world2d::Collider2D {
                        trigger: true,
                        ..Default::default()
                    },
                );
            }
            scene.sync_actor_components();
            let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
            let mut session = Session::launch(&exe).unwrap();
            session.receive(Duration::from_secs(10)).unwrap();
            let mut packet = step(&mut session, 16_667, None, &[]);
            for _ in 0..4 {
                packet = step(&mut session, 16_667, None, &[]);
            }
            assert_eq!(
                packet.actors[source].rotation[0], 4096,
                "{domain}D automatic collection rewards once"
            );
            assert_eq!(
                packet.actors[source].rotation[1], 4096,
                "{domain}D collected flag survives pickup deactivation"
            );
            assert_eq!(
                packet.actors[foreign].rotation[0], 0,
                "{domain}D foreign overlap receives no reward"
            );
            drop(session);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn interaction_sensors_route_native_physics_in_both_domains() {
        let root = crate::workspace::tests::temp("interaction-sensors");
        crate::workspace::create(
            &root,
            "Interaction Sensors",
            crate::workspace::Template::Basic,
        )
        .unwrap();
        fs::write(root.join("assets/scripts/SensorProbe.hpp"),r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="a65a7d6e-d488-525b-8a9b-93913b0f154d") SensorTargetProbe:public epok::InteractableComponent {
public:
 void interaction_started(epok::ObjectId,uint32_t) override{get_owner()->data()->transform.rotation[0]+=epok::Fixed(1.0);}
};
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="4fb70742-6a37-527d-b6dc-a83c8aee211b") SensorPlayerProbe:public epok::ActorComponent {
public:
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::InteractorComponent") epok::ObjectId scanner{};
 void on_frame(uint32_t) override{
   auto* owner=get_owner();auto* interactor=epok::active_object_registry->resolve<epok::InteractorComponent>(scanner);
   if(epok::input.held(epok::Button::Square)){
     if(auto* root=epok::active_object_registry->resolve<epok::SceneComponent2D>(owner->root_id()))root->set_position(128.0,0.0);
     else if(auto* root=epok::active_object_registry->resolve<epok::SceneComponent3D>(owner->root_id()))root->set_local_position(4.0,0.0,0.0);
   }
   owner->data()->transform.rotation[1]=interactor&&interactor->focused_target().valid()?epok::Fixed(1.0):epok::Fixed(0.0);
 }
};
"#).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        use crate::actor_document::{ClassReference, ComponentInstance};
        for domain in [3, 2] {
            let mut scene = Scene::default();
            scene.actors.clear();
            let actor = if domain == 3 {
                crate::object_model::ACTOR3D_ID
            } else {
                crate::object_model::ACTOR2D_ID
            };
            let source =
                crate::editor::place_class(&mut scene, &root, &registry, &model, actor).unwrap();
            let target =
                crate::editor::place_class(&mut scene, &root, &registry, &model, actor).unwrap();
            let sensor = crate::editor::place_class(
                &mut scene,
                &root,
                &registry,
                &model,
                if domain == 3 {
                    "177ad606-c610-5ca6-ba99-a6126b85595d"
                } else {
                    "1d128dde-89a7-516e-bc39-2b4c2eada9c5"
                },
            )
            .unwrap();
            let mut scanner = ComponentInstance::new(
                uuid::Uuid::new_v4(),
                ClassReference::new(
                    "epok::InteractorComponent",
                    "ebe82fcd-e9db-5de5-89a7-ed42f3a6d29a",
                ),
                "Scanner",
            );
            scanner
                .properties
                .insert("proximity".into(), serde_json::json!(false));
            scanner.overrides.insert("proximity".into());
            let scanner_id = scanner.id;
            let endpoint = ComponentInstance::new(
                uuid::Uuid::new_v4(),
                ClassReference::new("SensorTargetProbe", "a65a7d6e-d488-525b-8a9b-93913b0f154d"),
                "Examine",
            );
            let target_id = endpoint.id;
            scene.actors[target].components.push(endpoint);
            scene.actors[source].components.push(scanner);
            let mut probe = ComponentInstance::new(
                uuid::Uuid::new_v4(),
                ClassReference::new("SensorPlayerProbe", "4fb70742-6a37-527d-b6dc-a83c8aee211b"),
                "Probe",
            );
            probe
                .properties
                .insert("scanner".into(), serde_json::json!(scanner_id));
            probe.overrides.insert("scanner".into());
            scene.actors[source].components.push(probe);
            scene.actors[source].components.push(ComponentInstance::new(
                uuid::Uuid::new_v4(),
                ClassReference::new(
                    if domain == 3 {
                        "epok::Collider3DComponent"
                    } else {
                        "epok::Collider2DComponent"
                    },
                    if domain == 3 {
                        crate::object_model::COLLIDER3D_COMPONENT_ID
                    } else {
                        "aad9ca37-7c63-4ca9-8a23-5f4ff49e35fe"
                    },
                ),
                "Collider",
            ));
            scene.actors[source].refresh_components();
            let source_id = scene.actors[source].id;
            let sensor_component = scene.actors[sensor]
                .components
                .iter_mut()
                .find(|c| !c.root)
                .unwrap();
            sensor_component
                .properties
                .insert("target".into(), serde_json::json!(target_id));
            sensor_component.overrides.insert("target".into());
            sensor_component
                .properties
                .insert("accepted_initiator".into(), serde_json::json!(source_id));
            sensor_component
                .overrides
                .insert("accepted_initiator".into());
            let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
            let mut session = Session::launch(&exe).unwrap();
            session.receive(Duration::from_secs(10)).unwrap();
            step(&mut session, 16_667, None, &[]);
            let entered = step(&mut session, 10_000, None, &[]);
            assert_eq!(
                entered.actors[source].rotation[1], 4096,
                "{domain}D trigger registers focus"
            );
            let accepted = step(
                &mut session,
                10_000,
                Some(crate::controls::Button::Cross),
                &[],
            );
            assert_eq!(
                accepted.actors[target].rotation[0], 4096,
                "{domain}D trigger interaction"
            );
            step(
                &mut session,
                16_667,
                Some(crate::controls::Button::Square),
                &[],
            );
            let exited = step(&mut session, 16_667, None, &[]);
            assert_eq!(
                exited.actors[source].rotation[1], 0,
                "{domain}D exit clears focus"
            );
            let unavailable = step(
                &mut session,
                10_000,
                Some(crate::controls::Button::Cross),
                &[],
            );
            assert_eq!(unavailable.actors[target].rotation[0], 4096);
            drop(session);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "requires EPOK_NATIVE_PROJECT, a native C++ compiler, and project assets"]
    fn project_runner_compiles_and_returns_a_frame() {
        let root = PathBuf::from(std::env::var_os("EPOK_NATIVE_PROJECT").unwrap());
        let path = std::env::var_os("EPOK_NATIVE_SCENE")
            .map(PathBuf::from)
            .unwrap_or_else(|| crate::workspace::startup_scene(&root).unwrap());
        let scene = Scene::load(&path).unwrap();
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe).unwrap();
        let packet = session.receive(Duration::from_secs(5)).unwrap();
        assert!(packet.authored_count > 0);
        assert_eq!(packet.actors.len(), packet.actor_count as usize);
        if std::env::var_os("EPOK_NATIVE_EXPECT_AUDIO").is_some() {
            assert!(!packet.audio.is_empty(), "initial frame has no audio event");
        }
        let first_frame = packet.number;
        let mut last_frame = first_frame;
        for _ in 0..120 {
            session
                .request(16667, [crate::controls::PadState::default(); 4], &[])
                .unwrap();
            let packet = session.receive(Duration::from_secs(5)).unwrap();
            assert!(
                packet.scene_bank == 0,
                "project requested a scene transition"
            );
            last_frame = packet.number;
        }
        assert_eq!(last_frame, first_frame + 120);
        println!(
            "Native PC project compiled and advanced 120 frames: {}",
            root.display()
        );
    }
    #[test]
    #[ignore = "requires EPOK_NATIVE_PROJECT and an EPOK_NATIVE_SCENE with RailProbe and rail aim"]
    fn project_runner_advances_rail_camera_and_target_aim() {
        let root = PathBuf::from(std::env::var_os("EPOK_NATIVE_PROJECT").unwrap());
        let path = PathBuf::from(std::env::var_os("EPOK_NATIVE_SCENE").unwrap());
        let scene = Scene::load(&path).unwrap();
        let nav = scene
            .visibility_track
            .as_ref()
            .unwrap()
            .navigation
            .as_ref()
            .unwrap();
        assert!(nav.aim.is_some());
        let target = scene
            .actors
            .iter()
            .position(|a| a.id == nav.target)
            .unwrap();
        let exe = prepare(&root, scene, false, None, &AtomicBool::new(false)).unwrap();
        let mut still = Session::launch(&exe).unwrap();
        let mut moving = Session::launch(&exe).unwrap();
        let mut a = still.receive(Duration::from_secs(10)).unwrap();
        let mut b = moving.receive(Duration::from_secs(10)).unwrap();
        let camera = a.camera.unwrap();
        let initial_position = a.actors[camera].position;
        let mut pads = [crate::controls::PadState::default(); 4];
        for pad in &mut pads {
            pad.connected = true;
        }
        pads[0].buttons = (1u16 << crate::controls::Button::R1 as u32)
            | (1u16 << crate::controls::Button::R2 as u32);
        for _ in 0..240 {
            still
                .request(33334, [crate::controls::PadState::default(); 4], &[])
                .unwrap();
            moving.request(33334, pads, &[]).unwrap();
            a = still.receive(Duration::from_secs(5)).unwrap();
            b = moving.receive(Duration::from_secs(5)).unwrap();
        }
        assert_ne!(
            a.actors[camera].position, initial_position,
            "Native rail never advanced"
        );
        assert!(
            b.actors[target].position[1] > a.actors[target].position[1],
            "Probe input never reached the target"
        );
        assert_eq!(
            a.actors[camera].position, b.actors[camera].position,
            "Vertical aim must not move the rail"
        );
        assert_ne!(
            a.actors[camera].rotation[0], b.actors[camera].rotation[0],
            "Camera did not aim at the changed target height"
        );
        println!(
            "Native rail/aim: 240 paired frames; camera {:?}; pitch still={} moving={}; target Y still={} moving={}",
            a.actors[camera].position,
            a.actors[camera].rotation[0],
            b.actors[camera].rotation[0],
            a.actors[target].position[1],
            b.actors[target].position[1]
        );
    }
}
