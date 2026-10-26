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
        let player = crate::preview_audio::Player::start_with_gain(pcm, self.gain as f32 / 4096.)
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
                player.set_gain(gain as f32 / 4096.)?;
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
    let resources = crate::scene_bank::resources(&banks);
    let catalog = input
        .native_play_catalog
        .clone()
        .map_or_else(|| crate::scripts::catalog(root), Ok)?;
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
                Ok(crate::pipeline::Control::Stop | crate::pipeline::Control::StopAndReset) | Err(mpsc::TryRecvError::Disconnected) => {
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
                Ok(crate::pipeline::Control::Stop | crate::pipeline::Control::StopAndReset) | Err(mpsc::TryRecvError::Disconnected) => {
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
