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
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant},
};

const MAGIC: u32 = 0x314e_5045;
const ACTOR_WORDS: usize = 16;
const AUDIO_WORDS: usize = 6;
const AUDIO_LIMIT: u32 = 256;

#[derive(Clone, Debug)]
struct ActorState {
    alive: bool,
    active: bool,
    parent: i32,
    position: [i32; 3],
    rotation: [i32; 3],
    scale: [i32; 3],
    fov: i32,
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
    pub buttons: u16,
    pub actor_count: u32,
    pub camera: Option<usize>,
    pub hud_rgba: Vec<u8>,
    pub hud_size: [u32; 2],
    pub stats: [u32; 5],
    pub audio_voices: u32,
    pub audio_pending: u32,
}
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
            zoom: (1. / (camera.camera_fov.clamp(25., 120.).to_radians() * 0.5).tan()) / 1.8125,
            distance,
            phase: self.number as f32 / 60.,
            fly_speed: 5.,
        }
    }
}

#[derive(Default)]
pub struct State {
    pub frame: Option<Arc<Frame>>,
    pub connected: bool,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct Bridge {
    pub state: Arc<Mutex<State>>,
    pub pads: Arc<Mutex<[crate::controls::PadState; 4]>>,
}
impl Bridge {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State::default())),
            pads: Arc::new(Mutex::new(Default::default())),
        }
    }
}

#[derive(Debug)]
struct Packet {
    number: u32,
    fade: u8,
    actor_count: u32,
    authored_count: u32,
    camera: Option<usize>,
    actors: Vec<ActorState>,
    commands: Vec<hud_native::Command>,
    audio: Vec<AudioEvent>,
    requested_scene: String,
    stats: [u32; 5],
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
    if actor_count > 1024
        || authored_count > actor_count
        || command_count > 2560
        || audio_count > AUDIO_LIMIT
        || text_length > 4096
        || fade > 255
    {
        return Err("Native PC Play packet exceeds protocol bounds".into());
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
    Ok(Packet {
        number,
        fade: fade as u8,
        actor_count,
        authored_count,
        camera,
        actors,
        commands,
        audio,
        requested_scene: String::from_utf8_lossy(&text).into_owned(),
        stats,
    })
}

fn apply(
    mut scene: Scene,
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
    let hud_rgba = hud_native::render_overlay(
        &scene,
        &packet.commands,
        packet.number as f32 / 60.,
        packet.fade,
    );
    Frame {
        hud_size: scene.display_size.map(u32::from),
        scene,
        number: u64::from(packet.number),
        buttons,
        actor_count: packet.actor_count,
        camera: packet.camera.filter(|index| *index < count),
        hud_rgba,
        stats: packet.stats,
        audio_voices: audio_voices as u32,
        audio_pending: audio_pending as u32,
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
            .write_all(&elapsed.to_le_bytes())
            .and_then(|_| {
                for pad in pads {
                    self.input
                        .write_all(&u32::from(pad.buttons).to_le_bytes())?;
                    self.input.write_all(&u32::from(pad.analog).to_le_bytes())?;
                    self.input
                        .write_all(&u32::from_le_bytes(pad.axes).to_le_bytes())?;
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
        let player = crate::preview_audio::Player::start(pcm)
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

fn prepare(
    root: &Path,
    mut scene: Scene,
    scene_ready: bool,
    catalog: Option<Vec<crate::scripts::Script>>,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    if !scene_ready {
        crate::project::refresh_linked_scene(root, &mut scene)?;
        let assets = crate::assets::scan(root, &mut Default::default());
        crate::mesh::resolve(&mut scene, &assets)?;
        crate::skeletal::resolve(&mut scene, &assets)?;
        crate::texture::resolve(&mut scene, &assets)?;
        crate::hud::resolve_fonts(&mut scene, &assets)?;
    }
    let rendering = crate::settings::rendering(root)?;
    scene.display_size = [rendering.width, rendering.height];
    let catalog = catalog.map_or_else(|| crate::scripts::catalog(root), Ok)?;
    crate::hud_simulation::build_runner(
        root,
        &scene,
        &catalog,
        cancel,
        crate::hud_simulation::Runner::NativePlay,
    )
}

pub fn execute(
    root: &Path,
    input: &crate::scene_dependencies::Input,
    tx: &std::sync::mpsc::Sender<crate::pipeline::Event>,
    controls: &Receiver<crate::pipeline::Control>,
    bridge: &Bridge,
) -> Result<(), String> {
    let assets = input
        .native_play_assets
        .clone()
        .unwrap_or_else(|| crate::assets::scan(root, &mut Default::default()));
    // Decode authored start-on-play audio alongside C++ preparation rather than
    // extending the time between a ready runtime and its first audible frame.
    let mut audio = HostAudio::new(root, &input.scene, &assets)?;
    let (exe, built) = if let Some(exe) = input
        .native_play_cache
        .as_ref()
        .filter(|path| path.is_file())
    {
        let _ = tx.send(crate::pipeline::Event::Stage(
            "Starting Native PC runtime".into(),
        ));
        let _ = tx.send(crate::pipeline::Event::Log(
            "Reusing current Native PC runtime.".into(),
        ));
        (exe.clone(), false)
    } else {
        let _ = tx.send(crate::pipeline::Event::Stage(
            "Building Native PC runtime".into(),
        ));
        let _ = tx.send(crate::pipeline::Event::Log(
            "Compiling shared C++ gameplay for Native PC...".into(),
        ));
        let cancel = Arc::new(AtomicBool::new(false));
        let (done_tx, done_rx) = mpsc::channel();
        let root_owned = root.to_path_buf();
        let scene = input.scene.clone();
        let scene_ready = input.native_play_scene_ready;
        let catalog = input.native_play_catalog.clone();
        let compile_cancel = cancel.clone();
        std::thread::spawn(move || {
            let _ = done_tx.send(prepare(
                &root_owned,
                scene,
                scene_ready,
                catalog,
                &compile_cancel,
            ));
        });
        let exe = loop {
            match done_rx.recv_timeout(Duration::from_millis(30)) {
                Ok(result) => break result?,
                Err(mpsc::RecvTimeoutError::Timeout) => match controls.try_recv() {
                    Ok(crate::pipeline::Control::Stop) | Err(mpsc::TryRecvError::Disconnected) => {
                        cancel.store(true, Ordering::Relaxed);
                        return Err("Native PC Play cancelled".into());
                    }
                    _ => {}
                },
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("Native PC compilation worker ended".into());
                }
            }
        };
        (exe, true)
    };
    if built {
        let _ = tx.send(crate::pipeline::Event::Built(exe.clone()));
    }
    let mut session = Session::launch(&exe)?;
    let _ = tx.send(crate::pipeline::Event::Running(session.child.id()));
    bridge.state.lock().unwrap().connected = true;
    let mut paused = false;
    let mut next = Instant::now();
    let mut sampled_pads = [crate::controls::PadState::default(); 4];
    loop {
        let packet = session.receive(Duration::from_secs(5))?;
        let completed_audio = audio.update(&packet.audio)?;
        let (audio_voices, audio_pending) = audio.status();
        let pads = *bridge.pads.lock().unwrap();
        let requested = packet.requested_scene.clone();
        let frame = apply(
            input.scene.clone(),
            &packet,
            sampled_pads[0].buttons,
            audio_voices,
            audio_pending,
        );
        let frame_number = frame.number;
        bridge.state.lock().unwrap().frame = Some(Arc::new(frame));
        if !requested.is_empty() {
            return Err(format!(
                "Native PC scene transition to `{requested}` is not available in this MVP. Use PlayStation runtime for multi-scene Play."
            ));
        }
        let mut reset = false;
        loop {
            match controls.try_recv() {
                Ok(crate::pipeline::Control::Stop) | Err(mpsc::TryRecvError::Disconnected) => {
                    return Ok(());
                }
                Ok(crate::pipeline::Control::Pause) => {
                    paused = true;
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
            session = Session::launch(&exe)?;
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
    #[test]
    fn packet_bounds_are_rejected_before_allocation() {
        let mut bytes = Vec::new();
        for value in [MAGIC, 0, 0, 2048, 0, u32::MAX, 0, 0, 0, 0, 0, 0, 0, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        assert!(read_packet(&mut &bytes[..]).unwrap_err().contains("bounds"));
    }
    #[test]
    fn actor_animation_state_round_trips_through_the_protocol() {
        let mut bytes = Vec::new();
        for value in [MAGIC, 7, 0, 1, 1, u32::MAX, 0, 0, 1, 0, 0, 0, 0, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
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
            1 | 2 | 4,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [0x89ab_cdef_u32, 0x0123_4567, 1, 2, 3072, 6144] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let packet = read_packet(&mut &bytes[..]).unwrap();
        let actor = &packet.actors[0];
        assert_eq!(actor.animation_ticks, 18);
        assert_eq!(actor.animation_clip, 3);
        assert!(actor.animation_enabled && actor.animation_looping);
        assert_eq!(packet.audio.len(), 1);
        assert_eq!(packet.audio[0].source, 0x0123_4567_89ab_cdef);
        assert!(packet.audio[0].play);
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
    }
}
