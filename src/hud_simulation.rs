//! Isolated, native execution of authored HUD scenes. No emulator or MIPS code
//! runs here. The generated scene initializer and controller sources are shared.
use crate::{hud_native, scene::Scene, scripts::Script};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant},
};

/// The two phases of the native preview, and the boundary between them.
///
/// `Edit` is the automatic preview shown while authoring. It binds properties and
/// calls only public `editor_preview(Transform&)` construction hooks: no
/// BeginPlay/`start`, no `update`, no `frame_update`, no game clock, no input.
/// What it shows is construction, not behaviour.
///
/// `Simulate` is the optional **Interact** session. It runs the ordinary gameplay
/// lifecycle against the same generated scene.
///
/// Nothing but this flag decides which: the child process fixes its phase at
/// startup from `--edit` and reports it back in every frame header, so the editor
/// can never believe a frame came from a phase that did not produce it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum PreviewPhase {
    #[default]
    Edit,
    Simulate,
}
impl PreviewPhase {
    /// The boundary itself, as a predicate: `Edit` never runs BeginPlay.
    #[allow(dead_code)] // P9 states the contract ahead of the UI that displays it.
    pub fn runs_begin_play(self) -> bool {
        self == Self::Simulate
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Edit => "edit",
            Self::Simulate => "simulate",
        }
    }
}

/// Shown when a UI actor/entity is driven by a Blueprint class. The native
/// preview links C++ controllers only, so rendering it as if the Blueprint had
/// run would be a silent lie about behaviour that never executed.
pub const BLUEPRINT_UNSUPPORTED: &str = "Blueprint logic is not simulated in the native preview. \
This preview compiles C++ controllers only; use Play or the console build to run Blueprint classes.";

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Frame {
    pub number: u32,
    pub fade: u8,
    pub entities: u32,
    pub stats: [u32; 5],
    pub commands: Vec<hud_native::Command>,
    pub requested_scene: String,
    /// Reported by the child, not assumed by the editor.
    pub phase: PreviewPhase,
    /// Capability bits from the handshake; `blueprint_support()` is always false.
    pub capabilities: u32,
}
impl Frame {
    #[allow(dead_code)] // P9 states the contract ahead of the UI that displays it.
    pub fn blueprint_support(&self) -> bool {
        self.capabilities & hud_native::HUD_PREVIEW_CAP_BLUEPRINT != 0
    }
}
fn read_frame(input: &mut impl Read) -> Result<Frame, String> {
    fn word(input: &mut impl Read) -> Result<u32, String> {
        let mut b = [0; 4];
        input
            .read_exact(&mut b)
            .map_err(|e| format!("Native simulation ended: {e}"))?;
        Ok(u32::from_le_bytes(b))
    }
    if word(input)? != hud_native::HUD_PREVIEW_MAGIC {
        return Err("Invalid native HUD protocol".into());
    }
    // Checked on both sides: a cached executable built from an older header is
    // rejected here rather than decoded with the wrong field layout.
    let version = word(input)?;
    if version != hud_native::HUD_PREVIEW_PROTOCOL_VERSION {
        return Err(format!(
            "Native HUD preview protocol {version} does not match this editor's {}. Restart the preview to rebuild it.",
            hud_native::HUD_PREVIEW_PROTOCOL_VERSION
        ));
    }
    let capabilities = word(input)?;
    let phase = if capabilities & hud_native::HUD_PREVIEW_CAP_SIMULATE != 0 {
        PreviewPhase::Simulate
    } else {
        PreviewPhase::Edit
    };
    let number = word(input)?;
    let fade = word(input)?;
    let entities = word(input)?;
    let mut stats = [0; 5];
    for s in &mut stats {
        *s = word(input)?;
    }
    let count = word(input)?;
    let length = word(input)?;
    if count > 2560 || length > 4096 || fade > 255 || entities > 65535 {
        return Err("Invalid native HUD frame bounds".into());
    }
    let mut commands = vec![hud_native::Command::default(); count as usize];
    for c in &mut commands {
        for v in &mut c.0 {
            *v = word(input)? as i32;
        }
    }
    let mut text = vec![0; length as usize];
    input.read_exact(&mut text).map_err(|e| e.to_string())?;
    Ok(Frame {
        number,
        fade: fade as u8,
        entities,
        stats,
        commands,
        requested_scene: String::from_utf8_lossy(&text).into_owned(),
        phase,
        capabilities,
    })
}

pub struct Session {
    child: Child,
    input: ChildStdin,
    frames: Receiver<Result<Frame, String>>,
    pub scene: Scene,
    pub frame: Option<Frame>,
    pub log: PathBuf,
    pending: Option<Instant>,
    pub elapsed_us: u64,
    editing: bool,
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Session {
    fn launch(exe: &Path, scene: Scene) -> Result<Self, String> {
        Self::launch_mode(exe, scene, false)
    }
    fn launch_mode(exe: &Path, scene: Scene, editing: bool) -> Result<Self, String> {
        let log = exe
            .parent()
            .unwrap()
            .join(format!("session-{}.log", uuid::Uuid::new_v4()));
        let mut command = Command::new(exe);
        if editing {
            command.arg("--edit");
        }
        crate::pipeline::quiet(&mut command);
        command
            .current_dir(exe.parent().unwrap())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(fs::File::create(&log).map_err(|e| e.to_string())?);
        let mut child = command
            .spawn()
            .map_err(|e| format!("Start native simulation: {e}"))?;
        let input = child.stdin.take().unwrap();
        let mut output = child.stdout.take().unwrap();
        let (send, frames) = mpsc::sync_channel(2);
        std::thread::spawn(move || {
            loop {
                let frame = read_frame(&mut output);
                let failed = frame.is_err();
                if send.send(frame).is_err() || failed {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input,
            frames,
            scene,
            frame: None,
            log,
            pending: Some(Instant::now()),
            elapsed_us: 0,
            editing,
        })
    }
    /// The phase this session asked the child for. `Edit` never runs BeginPlay.
    pub fn phase(&self) -> PreviewPhase {
        if self.editing {
            PreviewPhase::Edit
        } else {
            PreviewPhase::Simulate
        }
    }
    /// The child reports its own phase; a disagreement means the executable is not
    /// the one this session launched, so the frame is rejected instead of shown.
    fn accept(&self, frame: Frame) -> Result<Frame, String> {
        if frame.phase != self.phase() {
            return Err(format!(
                "Native HUD preview reported the {} phase but this session requested {}.",
                frame.phase.label(),
                self.phase().label()
            ));
        }
        Ok(frame)
    }
    pub fn request(&mut self, elapsed: u32, buttons: u16) -> Result<(), String> {
        if self.pending.is_some() {
            return Ok(());
        }
        self.input
            .write_all(&elapsed.to_le_bytes())
            .and_then(|_| self.input.write_all(&u32::from(buttons).to_le_bytes()))
            .and_then(|_| self.input.flush())
            .map_err(|e| e.to_string())?;
        self.pending = Some(Instant::now());
        self.elapsed_us += u64::from(elapsed);
        Ok(())
    }
    pub fn poll(&mut self) -> Result<bool, String> {
        match self.frames.try_recv() {
            Ok(result) => {
                self.pending = None;
                let frame = result.map_err(|e| {
                    format!("{e}\n{}", fs::read_to_string(&self.log).unwrap_or_default())
                })?;
                self.frame = Some(self.accept(frame)?);
                Ok(true)
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                Err("Native simulation process disconnected".into())
            }
            Err(mpsc::TryRecvError::Empty) => {
                if self
                    .pending
                    .is_some_and(|t| t.elapsed() > Duration::from_secs(5))
                {
                    Err("Native script did not return within 5 seconds. Stop or restart the simulation.".into())
                } else {
                    Ok(false)
                }
            }
        }
    }
    pub fn wait_frame(&mut self) -> Result<&Frame, String> {
        let frame = self
            .frames
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())??;
        self.frame = Some(self.accept(frame)?);
        self.pending = None;
        Ok(self.frame.as_ref().unwrap())
    }
    pub fn pixels(&self) -> Option<Vec<u8>> {
        self.frame.as_ref().map(|f| {
            hud_native::render(
                &self.scene,
                &f.commands,
                self.elapsed_us as f32 / 1e6,
                f.fade,
            )
        })
    }
}

#[derive(Default)]
pub struct State {
    pub session: Option<Session>,
    building: Option<Receiver<Result<Session, String>>>,
    cancel: Option<Arc<AtomicBool>>,
    pub running: bool,
    pub error: Option<String>,
    pub pixels: Option<Vec<u8>>,
    accumulator: f64,
    pub buttons: u16,
    pub focused: bool,
    pending_steps: u32,
    pub interactive: bool,
    edit_key: Option<(Vec<u8>, [u16; 2], u64, u64)>,
    edit_checked: Option<Instant>,
    edit_pending: Option<(Vec<u8>, [u16; 2], u64, u64)>,
}
impl Drop for State {
    fn drop(&mut self) {
        self.stop();
    }
}
impl State {
    pub fn compiling(&self) -> bool {
        self.building.is_some()
    }
    pub fn stop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.session = None;
        self.building = None;
        self.running = false;
        self.pixels = None;
        self.accumulator = 0.;
        self.buttons = 0;
        self.focused = false;
        self.pending_steps = 0;
        self.interactive = false;
        self.edit_key = None;
        self.edit_checked = None;
        self.edit_pending = None;
    }
    pub fn start(&mut self, root: PathBuf, scene: Scene, catalog: Vec<Script>) {
        self.start_mode(root, scene, catalog, false);
    }
    fn start_mode(&mut self, root: PathBuf, scene: Scene, catalog: Vec<Script>, editing: bool) {
        self.stop();
        self.error = None;
        self.interactive = !editing;
        self.running = !editing;
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = Some(cancel.clone());
        let (send, receive) = mpsc::channel();
        self.building = Some(receive);
        std::thread::spawn(move || {
            let result = build(&root, &scene, &catalog, &cancel).and_then(|exe| {
                if cancel.load(Ordering::Relaxed) {
                    return Err("Simulation cancelled".into());
                }
                Session::launch_mode(&exe, scene, editing)
            });
            let _ = send.send(result);
        });
    }
    /// Rebuild procedural UI automatically from the current document. No game
    /// time or input runs in this mode. Briefly debounce continuous edits.
    pub fn ensure_edit(
        &mut self,
        root: &Path,
        scene: &Scene,
        catalog: &[Script],
        assets: u64,
        scripts: u64,
    ) {
        if self.interactive
            || self
                .edit_checked
                .is_some_and(|t| t.elapsed() < Duration::from_millis(250))
        {
            return;
        }
        self.edit_checked = Some(Instant::now());
        let Ok(document) = serde_json::to_vec(scene) else {
            return;
        };
        let key = (document, scene.display_size, assets, scripts);
        if self.edit_key.as_ref() == Some(&key) {
            self.edit_pending = None;
            return;
        }
        if self.edit_key.is_some() && self.edit_pending.as_ref() != Some(&key) {
            self.edit_pending = Some(key);
            return;
        }
        if scene.entities.iter().any(|e| e.script.is_some()) {
            self.start_mode(root.to_path_buf(), scene.clone(), catalog.to_vec(), true);
        } else {
            self.stop();
            self.error = None;
        }
        self.edit_key = Some(key);
        self.edit_checked = Some(Instant::now());
    }
    pub fn update(&mut self, delta: f32, step: bool) -> bool {
        self.pending_steps = (self.pending_steps + u32::from(step)).min(60);
        let mut changed = false;
        if let Some(job) = &self.building {
            match job.try_recv() {
                Ok(result) => {
                    self.building = None;
                    match result {
                        Ok(s) => self.session = Some(s),
                        Err(e) => {
                            self.error = Some(e);
                            self.running = false;
                        }
                    }
                    changed = true;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.building = None;
                    self.error = Some("Native compilation worker ended".into());
                    self.running = false;
                    changed = true;
                }
                _ => {}
            }
        }
        let result = (|| {
            let Some(session) = self.session.as_mut() else {
                return Ok::<_, String>(());
            };
            if session.poll()? {
                self.pixels = session.pixels();
                changed = true;
            }
            if session.editing {
                return Ok(());
            }
            if session
                .frame
                .as_ref()
                .is_some_and(|f| !f.requested_scene.is_empty())
            {
                self.running = false;
                return Ok(());
            }
            if self.running {
                self.accumulator = (self.accumulator + f64::from(delta)).min(0.1);
            } else {
                self.accumulator = 0.;
            }
            if session.pending.is_none() && (self.pending_steps > 0 || self.accumulator >= 1. / 60.)
            {
                // Step uses an exact 60 Hz clock. Running uses elapsed time so
                // a slower editor frame never slows the game's animations;
                // the shared Time service schedules fixed simulation ticks.
                let number = session.frame.as_ref().map_or(0, |f| u64::from(f.number));
                let us = if self.pending_steps > 0 {
                    self.pending_steps -= 1;
                    ((number + 1) * 1_000_000 / 60 - number * 1_000_000 / 60) as u32
                } else {
                    (self.accumulator * 1_000_000.).floor() as u32
                };
                session.request(us, self.buttons)?;
                self.accumulator = (self.accumulator - f64::from(us) / 1_000_000.).max(0.);
            }
            Ok(())
        })();
        if let Err(error) = result {
            let edit_key = self.edit_key.take();
            let interactive = self.interactive;
            self.error = Some(error);
            self.stop();
            self.edit_key = edit_key;
            self.interactive = interactive;
            changed = true;
        }
        changed
    }
}

/// Name of the first entity whose script binding is a Blueprint class, if any.
/// The preview links C++ controllers only; a Blueprint binding is the case that
/// would otherwise render as a working UI whose logic never ran.
pub fn blueprint_driven(scene: &Scene) -> Option<&str> {
    let blueprint = crate::script_backend::blueprint_provider();
    scene
        .entities
        .iter()
        .find(|e| {
            e.script
                .as_ref()
                .is_some_and(|s| s.provider.id == blueprint.id)
        })
        .map(|e| e.name.as_str())
}

pub fn build(
    root: &Path,
    scene: &Scene,
    catalog: &[Script],
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    scene.validate()?;
    // A Blueprint-driven entity gets its own diagnostic: the generic "unsupported
    // component" message reads as a missing feature, while this one has to say
    // that the logic does not run, so nothing is rendered as if it had.
    if let Some(entity) = blueprint_driven(scene) {
        return Err(format!(
            "{BLUEPRINT_UNSUPPORTED} `{entity}` is driven by a Blueprint class."
        ));
    }
    if scene.entities.iter().any(|e| {
        e.timeline.is_some()
            || e.particle_effect.is_some()
            || e.script.as_ref().is_some_and(|s| s.provider.id != "cpp")
    }) {
        return Err("Native HUD simulation currently supports C++ controllers and Canvas components. Use the dedicated preview for Timeline/Particle Effects.".into());
    }
    let mut names = BTreeSet::new();
    for binding in scene.entities.iter().filter_map(|e| e.script.as_ref()) {
        let mut script = Some(crate::script_backend::resolve(binding, catalog)?);
        while let Some(s) = script {
            if !names.insert(s.name.clone()) {
                break;
            }
            script = s
                .parent
                .as_ref()
                .and_then(|p| catalog.iter().find(|s| &s.name == p));
        }
    }
    let used: Vec<_> = catalog
        .iter()
        .filter(|s| names.contains(&s.name))
        .cloned()
        .collect();
    // Attached C++ sources are only a compile filter. Scene-owned actors still
    // require SDK declarations that are absent from those script ancestry chains.
    let registry = if scene.scene_script.is_some() || !scene.actors.is_empty() {
        crate::blueprint::native_registry(root, catalog)?
    } else {
        crate::blueprint::legacy_registry(root, catalog)
    };
    let header = crate::project::scene_header_with_registry(scene, &used, scene, true, scene, &registry)?;
    let mut files: BTreeMap<PathBuf, Vec<u8>> = crate::project::runtime_sources()
        .iter()
        .map(|(p, b)| (PathBuf::from(p), b.to_vec()))
        .collect();
    files.insert("scene.hh".into(), header.into_bytes());
    files.insert(
        "hud-config.hh".into(),
        scene.hud_budget.header()?.into_bytes(),
    );
    files.insert(
        "text.hpp".into(),
        include_bytes!("../runtime/text.hpp").to_vec(),
    );
    files.insert("display.hh".into(),format!("#pragma once\nnamespace epok {{inline constexpr int display_width={},display_height={};}}\n",scene.display_size[0],scene.display_size[1]).into_bytes());
    files.insert(
        "hud_runner.cpp".into(),
        include_bytes!("../native/hud_runner.cpp").to_vec(),
    );
    files.insert(
        "hud_commands.hpp".into(),
        include_bytes!("../native/hud_commands.hpp").to_vec(),
    );
    files.insert(
        "hud_preview.h".into(),
        include_bytes!("../native/hud_preview.h").to_vec(),
    );
    files.insert(
        "EASTL/functional.h".into(),
        b"#pragma once\n#include <functional>\nnamespace eastl {using std::function;}\n".to_vec(),
    );
    files.insert(
        "psyqo/xprintf.h".into(),
        b"#pragma once\n#include <cstdio>\n".to_vec(),
    );
    // Pin the native arithmetic to the same Nugget revision as the editor build.
    files.insert(
        "psyqo/fixed-point.hh".into(),
        include_bytes!("../third_party/nugget/psyqo/fixed-point.hh").to_vec(),
    );
    let mut sources = BTreeSet::new();
    for script in &used {
        let source = PathBuf::from("scripts")
            .join(script.header_path())
            .with_extension("cpp");
        if root.join("assets").join(&source).is_file() {
            sources.insert(source);
        }
    }
    // Snapshot all headers/local includes; compile only the attached classes and
    // their bases. Unavailable hardware services produce a visible linker error.
    let native = crate::script_backend::prepare_native(root)?;
    files.extend(native.files);
    let mut compiler = host_compiler()?;
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(format!("{:?}", compiler));
    for (path, bytes) in &files {
        hash.update(path.to_string_lossy().as_bytes());
        hash.update([0]);
        hash.update(bytes);
    }
    let key = format!("{:x}", hash.finalize());
    let cache = root.join(".epok/native-preview").join(key);
    let exe = cache.join(if cfg!(windows) {
        "runner.exe"
    } else {
        "runner"
    });
    if exe.is_file() {
        return Ok(exe);
    }
    let stage = cache.join(uuid::Uuid::new_v4().to_string());
    for (path, bytes) in files {
        let path = stage.join(path);
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(path, bytes).map_err(|e| e.to_string())?;
    }
    let output = stage.join(if cfg!(windows) {
        "runner.exe"
    } else {
        "runner"
    });
    compiler.current_dir(&stage);
    if cfg!(windows) {
        compiler.args([
            "/nologo",
            "/std:c++20",
            "/EHsc",
            "/O2",
            "/W0",
            "/D_CRT_SECURE_NO_WARNINGS",
            "/DEPOK_EDITOR_PREVIEW",
            "/I.",
            "/Iscripts",
        ]);
        compiler.arg(format!("/Fe:{}", output.display()));
    } else {
        compiler
            .args([
                "-std=c++20",
                "-O2",
                "-DEPOK_EDITOR_PREVIEW",
                "-I.",
                "-Iscripts",
                "-o",
            ])
            .arg(&output);
    }
    compiler.arg("hud_runner.cpp");
    compiler.args(sources);
    let log = stage.join("compile.log");
    let file = fs::File::create(&log).map_err(|e| e.to_string())?;
    compiler
        .stdout(file.try_clone().map_err(|e| e.to_string())?)
        .stderr(file);
    crate::pipeline::quiet(&mut compiler);
    let mut child = compiler
        .spawn()
        .map_err(|e| format!("Start native C++ compiler: {e}"))?;
    let started = Instant::now();
    loop {
        if cancel.load(Ordering::Relaxed) || started.elapsed() > Duration::from_secs(120) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Native compilation cancelled or timed out".into());
        }
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            if !status.success() {
                return Err(format!(
                    "Native HUD compilation failed. Controllers must use portable C++ and supported HUD services.\n{}\nLog: {}",
                    fs::read_to_string(&log).unwrap_or_default(),
                    log.display()
                ));
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(30));
    }
    // A concurrent preview may have published the same immutable artifact.
    if !exe.exists() {
        fs::rename(&output, &exe)
            .or_else(|e| if exe.exists() { Ok(()) } else { Err(e) })
            .map_err(|e| e.to_string())?;
    }
    Ok(exe)
}
fn host_compiler() -> Result<Command, String> {
    #[cfg(windows)]
    {
        cc::windows_registry::find_tool("x86_64-pc-windows-msvc", "cl.exe")
            .map(|tool| tool.to_command())
            .ok_or_else(|| {
                "Install Visual Studio C++ Build Tools to simulate native HUD scenes.".into()
            })
    }
    #[cfg(not(windows))]
    {
        Ok(Command::new(
            std::env::var_os("CXX").unwrap_or_else(|| "c++".into()),
        ))
    }
}

pub fn capture(
    root: &Path,
    path: &Path,
    frames: u32,
    output: &Path,
) -> Result<serde_json::Value, String> {
    let catalog = crate::scripts::catalog(root)?;
    let mut scene = Scene::load(path)?;
    let rendering = crate::settings::rendering(root)?;
    scene.display_size = [rendering.width, rendering.height];
    let exe = build(root, &scene, &catalog, &AtomicBool::new(false))?;
    let mut session = Session::launch(&exe, scene)?;
    session.wait_frame()?;
    for i in 0..frames {
        session.request(
            (((u64::from(i) + 1) * 1_000_000 / 60) - (u64::from(i) * 1_000_000 / 60)) as u32,
            0,
        )?;
        session.wait_frame()?;
    }
    let pixels = session.pixels().ok_or("Native simulation has no frame")?;
    let [w, h] = session.scene.display_size.map(u32::from);
    fs::write(output, crate::mcp::png(w, h, &pixels)?).map_err(|e| e.to_string())?;
    Ok(
        serde_json::json!({"frame":session.frame,"image":output,"compiler":"native","emulator":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn header(version: u32, capabilities: u32, commands: u32) -> Vec<u8> {
        let mut bytes = vec![];
        // magic, version, capabilities, frame, fade, entities, stats[5], commands, text
        for n in [
            hud_native::HUD_PREVIEW_MAGIC,
            version,
            capabilities,
            7,
            0,
            1,
            0,
            0,
            0,
            0,
            0,
            commands,
            0,
        ] {
            bytes.extend_from_slice(&u32::to_le_bytes(n));
        }
        bytes
    }
    #[test]
    fn malformed_frames_are_bounded() {
        assert!(read_frame(&mut &b"oops"[..]).is_err());
        // Command count far past the bound.
        let bytes = header(hud_native::HUD_PREVIEW_PROTOCOL_VERSION, 0, 999_999);
        assert!(read_frame(&mut &bytes[..]).is_err());
    }
    #[test]
    fn protocol_version_mismatch_is_rejected() {
        // A well-formed frame at the current version decodes.
        let bytes = header(hud_native::HUD_PREVIEW_PROTOCOL_VERSION, 0, 0);
        let frame = read_frame(&mut &bytes[..]).expect("current protocol decodes");
        assert_eq!(frame.number, 7);
        // The same bytes from an executable built before the bump are rejected,
        // not decoded with the wrong field layout.
        let stale = header(hud_native::HUD_PREVIEW_PROTOCOL_VERSION - 1, 0, 0);
        let error = read_frame(&mut &stale[..]).expect_err("stale protocol rejected");
        assert!(error.contains("protocol"), "{error}");
        let future = header(hud_native::HUD_PREVIEW_PROTOCOL_VERSION + 1, 0, 0);
        assert!(read_frame(&mut &future[..]).is_err());
    }
    #[test]
    fn phase_and_blueprint_support_come_from_the_child() {
        // Edit: no simulate bit, so no BeginPlay ran to produce this frame.
        let edit = read_frame(&mut &header(hud_native::HUD_PREVIEW_PROTOCOL_VERSION, 0, 0)[..])
            .expect("edit frame");
        assert_eq!(edit.phase, PreviewPhase::Edit);
        assert!(!edit.phase.runs_begin_play());
        assert!(!edit.blueprint_support());

        let simulate = read_frame(
            &mut &header(
                hud_native::HUD_PREVIEW_PROTOCOL_VERSION,
                hud_native::HUD_PREVIEW_CAP_SIMULATE,
                0,
            )[..],
        )
        .expect("simulate frame");
        assert_eq!(simulate.phase, PreviewPhase::Simulate);
        assert!(simulate.phase.runs_begin_play());
        // The runner never sets the Blueprint bit; the handshake reports false.
        assert!(!simulate.blueprint_support());
    }
    #[test]
    fn blueprint_driven_scene_reports_its_own_diagnostic() {
        let mut scene = Scene::default();
        scene.entities.clear();
        let mut entity = crate::scene::Entity::cube("Menu".into());
        entity.kind = "Empty".into();
        entity.script = Some(crate::scene::ScriptBinding {
            name: "MenuLogic".into(),
            provider: crate::script_backend::blueprint_provider(),
            ..Default::default()
        });
        scene.entities.push(entity);
        assert_eq!(blueprint_driven(&scene), Some("Menu"));
        let error = build(&std::env::temp_dir(), &scene, &[], &AtomicBool::new(false))
            .expect_err("Blueprint scenes are refused");
        // The author is told the logic did not run, not that a component is missing.
        assert!(
            error.contains("Blueprint logic is not simulated"),
            "{error}"
        );
        assert!(error.contains("Menu"), "{error}");

        // A C++ controller is not caught by this rule.
        scene.entities[0].script.as_mut().unwrap().provider =
            crate::reflection_schema::native_provider();
        assert_eq!(blueprint_driven(&scene), None);
    }
    #[test]
    #[ignore = "Requires pinned libclang/MIPS SDK, host C++ compiler and cargo build --bins"]
    fn native_preview_resolves_implicit_scene_script_without_attached_scripts() {
        let root = crate::workspace::tests::temp("hud-scene-script");
        crate::workspace::create(&root, "Scene script preview", crate::workspace::Template::Basic).unwrap();
        let mut scene = Scene::default();
        scene.entities.clear();
        scene.ensure_scene_script(None);
        let original = scene.clone();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let exe = build(&root, &scene, &catalog, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe, scene.clone()).unwrap();
        assert_eq!(session.wait_frame().unwrap().entities, 0);
        assert_eq!(scene, original);
        drop(session);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn native_controller_runs_real_lifecycle_input_and_property_overrides() {
        let root = std::env::temp_dir().join(format!("epok-hud-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("assets/scripts")).unwrap();
        fs::write(root.join("assets/scripts/Probe.hpp"),r#"#pragma once
#include "epok.hpp"
class Probe:public epok::Behaviour {public:
 int offset=3;epok::EntityHandle image;
 void construct();
 void start(epok::Transform&)override{construct();image.get()->rect.position[1]=-20.0;}
 void on_enable()override{if(!image.get())throw 7;}
 void editor_preview(epok::Transform&){epok::set_active(&entity(),false);epok::set_active(&entity(),true);construct();}
 void frame_update(epok::Transform&,uint32_t)override;
 void update(epok::Transform&,epok::Fixed)override{}
};
"#).unwrap();
        fs::write(root.join("assets/scripts/Probe.cpp"),r#"#include "Probe.hpp"
void Probe::construct(){
 auto* canvas=epok::create_entity("Canvas");canvas->canvas.enabled=true;
 auto* e=epok::create_entity("Dynamic image",canvas);image=epok::handle(e);
 e->rect.enabled=true;e->rect.anchor_min[0]=e->rect.anchor_max[0]=e->rect.pivot[0]=0.0;e->rect.anchor_min[1]=e->rect.anchor_max[1]=e->rect.pivot[1]=1.0;
 e->rect.position[0]=epok::Fixed(offset*4096,epok::Fixed::RAW);e->rect.position[1]=-10.0;e->rect.size[0]=32.0;e->rect.size[1]=16.0;
 e->image.enabled=true;e->image.color[0]=255;e->image.color[1]=e->image.color[2]=0;
}
void Probe::frame_update(epok::Transform&,uint32_t){if(epok::input.frame_pressed(epok::Button::Cross))image.get()->rect.position[0]+=epok::Fixed(1.0);}
"#).unwrap();
        let mut scene = Scene::default();
        scene.entities.clear();
        scene.display_size = [640, 480];
        let mut director = crate::scene::Entity::cube("Director".into());
        director.kind = "Empty".into();
        director.script = Some(crate::scene::ScriptBinding {
            name: "Probe".into(),
            properties: BTreeMap::from([("offset".into(), serde_json::json!(9))]),
            ..Default::default()
        });
        scene.entities.push(director);
        let catalog = vec![Script {
            name: "Probe".into(),
            properties: vec![crate::scripts::Property {
                name: "offset".into(),
                default: serde_json::json!(3),
                value_type: crate::reflection_schema::Type::Int32,
                id: String::new(),
            }],
            ..Default::default()
        }];
        let original = scene.clone();
        let exe = build(&root, &scene, &catalog, &AtomicBool::new(false)).unwrap();
        let mut session = Session::launch(&exe, scene.clone()).unwrap();
        let first = session.wait_frame().unwrap().clone();
        assert_eq!(first.entities, 3);
        assert_eq!(first.commands.len(), 1);
        assert_eq!(first.commands[0].0[3], 9);
        session.request(16666, 1 << 14).unwrap();
        assert_eq!(session.wait_frame().unwrap().commands[0].0[3], 10);
        session.request(16667, 1 << 14).unwrap();
        assert_eq!(session.wait_frame().unwrap().commands[0].0[3], 10);
        assert_eq!(scene, original);
        drop(session);
        let mut restarted = Session::launch(&exe, scene).unwrap();
        assert_eq!(restarted.wait_frame().unwrap().commands, first.commands);
        let mut state = State::default();
        state.session = Some(restarted);
        state.update(1., false);
        assert_eq!(
            state
                .session
                .as_ref()
                .unwrap()
                .frame
                .as_ref()
                .unwrap()
                .number,
            0
        );
        // Two clicks are preserved even while a child frame is in flight.
        state.update(0., true);
        state.update(0., true);
        let deadline = Instant::now() + Duration::from_secs(5);
        while state
            .session
            .as_ref()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .number
            < 2
        {
            assert!(Instant::now() < deadline);
            state.update(0., false);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(!state.running);
        assert_eq!(state.session.as_ref().unwrap().elapsed_us, 33333);
        state.running = true;
        state.update(1. / 30., false);
        assert_eq!(
            state.session.as_ref().unwrap().elapsed_us,
            66666,
            "30 Hz presentation must advance 33 ms, not slow the game to half speed"
        );
        drop(state);
        // The normal edit view must construct without Start, clicks or elapsed
        // game time, then automatically apply a changed Inspector property.
        let mut edit = State::default();
        edit.ensure_edit(&root, &original, &catalog, 0, 0);
        assert!(!edit.interactive && !edit.running);
        fn ready(state: &mut State) {
            let deadline = Instant::now() + Duration::from_secs(30);
            while state
                .session
                .as_ref()
                .and_then(|s| s.frame.as_ref())
                .is_none()
            {
                assert!(Instant::now() < deadline, "preview timed out");
                state.update(0., false);
                assert!(state.error.is_none(), "{:?}", state.error);
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        ready(&mut edit);
        let frame = edit
            .session
            .as_ref()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .clone();
        assert_eq!(frame.commands[0].0[3], 9);
        assert_eq!(
            frame.commands[0].0[4], 10,
            "Start must not run in edit mode"
        );
        assert_eq!(frame.number, 0);
        edit.buttons = 1 << 14;
        edit.update(1., true);
        assert_eq!(edit.session.as_ref().unwrap().elapsed_us, 0);
        assert_eq!(
            edit.session
                .as_ref()
                .unwrap()
                .frame
                .as_ref()
                .unwrap()
                .commands,
            frame.commands
        );
        let mut edited = original.clone();
        edited.entities[0]
            .script
            .as_mut()
            .unwrap()
            .properties
            .insert("offset".into(), serde_json::json!(19));
        for _ in 0..2 {
            edit.edit_checked = None;
            edit.ensure_edit(&root, &edited, &catalog, 0, 0);
        }
        ready(&mut edit);
        assert_eq!(
            edit.session
                .as_ref()
                .unwrap()
                .frame
                .as_ref()
                .unwrap()
                .commands[0]
                .0[3],
            19
        );
        // A disabled controller must not generate its UI during editing.
        edited.entities[0].active = false;
        for _ in 0..2 {
            edit.edit_checked = None;
            edit.ensure_edit(&root, &edited, &catalog, 0, 0);
        }
        ready(&mut edit);
        assert!(
            edit.session
                .as_ref()
                .unwrap()
                .frame
                .as_ref()
                .unwrap()
                .commands
                .is_empty()
        );
        drop(edit);
        fs::remove_dir_all(root).unwrap();
    }
}
