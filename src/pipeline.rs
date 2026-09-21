use crate::project::{self, Config};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::Duration,
};

pub enum Event {
    Log(String),
    Stage(String),
    Progress(f32),
    MemoryReport(Box<crate::memory::Report>),
    BuildSummary(Box<crate::build_report::Summary>),
    SerialPorts(Vec<crate::serial_support::Port>),
    SerialConfigured(crate::play::Serial),
    SerialStatus(String),
    SerialIssue(String),
    LightingBaked(crate::lighting::Bake),
    Built(PathBuf),
    Running(u32),
    SerialConnected,
    SerialCommandPending(bool),
    Paused(bool),
    Finished(Result<(), String>),
}
pub enum Control {
    Stop,
    Pause,
    Resume,
    Reset,
}
pub struct Job {
    pub bridge: Option<crate::bridge::Bridge>,
    pub native: Option<crate::native_play::Bridge>,
    pub events: Receiver<Event>,
    controls: Sender<Control>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Job {
    pub fn report(root: PathBuf, mut summary: crate::build_report::Summary) -> Self {
        let (tx, events) = mpsc::channel();
        let (controls, rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let result = (|| {
                let _ = tx.send(Event::Stage("Generating asset and memory report".into()));
                summary.verify_inputs(&root)?;
                let build = crate::build_report::directory(&root, summary.debug);
                let config = Config::load(&root)?;
                let report = crate::memory::analyze(
                    &root,
                    &build,
                    summary.profile.clone(),
                    summary.debug,
                    &config,
                )?;
                summary.verify_inputs(&root)?;
                if matches!(
                    rx.try_recv(),
                    Ok(Control::Stop) | Err(mpsc::TryRecvError::Disconnected)
                ) {
                    return Err("Report generation cancelled.".into());
                }
                summary.record_report(&root)?;
                summary.save(&root)?;
                let _ = tx.send(Event::MemoryReport(Box::new(report)));
                let _ = tx.send(Event::BuildSummary(Box::new(summary)));
                Ok(())
            })();
            let _ = tx.send(Event::Finished(result));
        });
        Self {
            bridge: None,
            native: None,
            events,
            controls,
            worker: Some(worker),
        }
    }
    pub fn serial_setup(
        mut settings: crate::play::Serial,
        action: crate::serial_ui::Action,
    ) -> Self {
        let (tx, events) = mpsc::channel();
        let (controls, rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let task = crate::serial_support::Task { tx: &tx, rx: &rx };
            let result: Result<(), String> = (|| {
                if action != crate::serial_ui::Action::Scan {
                    crate::serial_support::prepare(&settings, &task)?;
                    settings.executable.clear();
                }
                let found = crate::serial_support::discover(&mut settings, &task);
                if action == crate::serial_ui::Action::Test {
                    found?;
                    crate::serial_support::probe(&settings, &task)?;
                } else if let Err(message) = found {
                    let _ = tx.send(Event::SerialStatus(format!(
                        "{}{message}",
                        if action == crate::serial_ui::Action::Prepare {
                            "Tools ready. "
                        } else {
                            ""
                        }
                    )));
                    return Ok(());
                }
                let _ = tx.send(Event::SerialStatus(
                    if action == crate::serial_ui::Action::Test {
                        "Unirom responded. Ready to Play."
                    } else if action == crate::serial_ui::Action::Prepare {
                        "Tools ready. Choose your adapter and start Unirom, then Play."
                    } else {
                        "Adapter list refreshed. Start the Unirom loader, then Play."
                    }
                    .into(),
                ));
                Ok(())
            })();
            if let Err(e) = &result {
                let _ = tx.send(Event::SerialIssue(e.clone()));
            }
            let _ = tx.send(Event::Finished(result));
        });
        Self {
            bridge: None,
            native: None,
            events,
            controls,
            worker: Some(worker),
        }
    }
    #[cfg(test)]
    pub fn test_channels() -> (Self, Sender<Event>, Receiver<Control>) {
        let (sender, events) = mpsc::channel();
        let (controls, receiver) = mpsc::channel();
        (
            Self {
                bridge: None,
                native: None,
                events,
                controls,
                worker: None,
            },
            sender,
            receiver,
        )
    }
    pub fn start_with_debug(
        root: PathBuf,
        scene: impl Into<crate::scene_dependencies::Input>,
        run: bool,
        debug: bool,
    ) -> Self {
        Self::start_with_target(root, scene, run, debug, false)
    }
    pub fn start_with_target(
        root: PathBuf,
        scene: impl Into<crate::scene_dependencies::Input>,
        run: bool,
        debug: bool,
        physical_disc: bool,
    ) -> Self {
        Self::start_request(root, scene.into(), run, debug, physical_disc, false)
    }
    pub fn analyze(root: PathBuf, scene: crate::scene_dependencies::Input, debug: bool) -> Self {
        Self::start_request(root, scene, false, debug, false, true)
    }
    fn start_request(
        root: PathBuf,
        scene: crate::scene_dependencies::Input,
        run: bool,
        debug: bool,
        physical_disc: bool,
        analyze: bool,
    ) -> Self {
        let (tx, events) = mpsc::channel();
        let (controls, rx) = mpsc::channel();
        let serial = scene.play.as_ref().is_some_and(|p| {
            p.runtime == crate::play::Runtime::PlayStation
                && p.target == crate::play::Target::Serial
        });
        let native = run
            && scene
                .play
                .as_ref()
                .is_some_and(|p| p.runtime == crate::play::Runtime::NativePc);
        let bridge = if run && !serial && !native {
            crate::bridge::Bridge::start(&root).map(Some)
        } else {
            Ok(None)
        };
        let (bridge, bridge_error) = match bridge {
            Ok(b) => (b, None),
            Err(e) => (None, Some(e)),
        };
        let script = bridge.as_ref().map(|b| b.script.clone());
        let native_bridge = native.then(crate::native_play::Bridge::new);
        let worker_native = native_bridge.clone();
        let worker = thread::spawn(move || {
            let result = if let Some(e) = bridge_error {
                Err(e)
            } else if let Some(native) = &worker_native {
                crate::native_play::execute(&root, &scene, &tx, &rx, native)
            } else {
                execute(
                    &root,
                    &scene,
                    run,
                    debug,
                    physical_disc,
                    analyze,
                    script.as_deref(),
                    &tx,
                    &rx,
                )
            };
            if let Err(error) = &result {
                // Preparation can fail before the native compiler writes any output.
                // Preserve that diagnostic in the same log advertised by Build / Play.
                if let Ok(mut log) = fs::OpenOptions::new()
                    .append(true)
                    .open(root.join(".epok/Build.log"))
                {
                    let _ = writeln!(log, "Build / Play failed: {error}");
                }
            }
            let _ = tx.send(Event::Finished(result));
        });
        Self {
            bridge,
            native: native_bridge,
            events,
            controls,
            worker: Some(worker),
        }
    }
    pub fn control(&self, control: Control) {
        let _ = self.controls.send(control);
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        let _ = self.controls.send(Control::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

pub fn quiet(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let _ = command;
}
pub(crate) fn stop(child: &mut Child) {
    #[cfg(windows)]
    {
        let mut command = Command::new("taskkill");
        command
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        quiet(&mut command);
        let _ = command.status();
    }
    #[cfg(unix)]
    {
        // quiet() creates a dedicated process group, including shell/compiler children.
        let _ = Command::new("/bin/kill")
            .args(["-KILL", "--", &format!("-{}", child.id())])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}
fn output(
    child: &mut Child,
    tx: &Sender<Event>,
    capture_inputs: bool,
    build_log: Option<&Arc<Mutex<fs::File>>>,
) -> Vec<thread::JoinHandle<String>> {
    let pipes: Vec<Box<dyn Read + Send>> = vec![
        Box::new(child.stdout.take().unwrap()),
        Box::new(child.stderr.take().unwrap()),
    ];
    pipes
        .into_iter()
        .map(|pipe| {
            let tx = tx.clone();
            let build_log = build_log.cloned();
            thread::spawn(move || {
                let mut captured = String::new();
                for line in BufReader::new(pipe).lines() {
                    match line {
                        Ok(line) => {
                            if let Some(log) = &build_log {
                                let _ = writeln!(log.lock().unwrap(), "{line}");
                            }
                            if capture_inputs
                                && let Some(record) =
                                    line.strip_prefix(crate::build_inputs::REPORT_PREFIX)
                            {
                                captured.push_str(record);
                                captured.push('\n');
                                continue;
                            }
                            if build_log.is_some() && native_command_echo(&line) {
                                continue;
                            }
                            let _ = tx.send(Event::Log(line.chars().take(2000).collect()));
                        }
                        Err(_) => break,
                    }
                }
                captured
            })
        })
        .collect()
}
/// Hide command echoes only; compiler diagnostics (including tool-prefixed
/// failures such as `mipsel-none-elf-g++: error:`) must remain visible.
fn native_command_echo(line: &str) -> bool {
    let Some(tool) = line.split_whitespace().next() else {
        return false;
    };
    let tool = tool
        .trim_matches('"')
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(tool);
    matches!(
        tool.trim_end_matches(".exe"),
        "mipsel-none-elf-g++"
            | "mipsel-none-elf-gcc"
            | "mipsel-none-elf-gcc-ar"
            | "mipsel-none-elf-ar"
            | "mipsel-none-elf-objcopy"
    )
}
pub fn request(port: u16, function: &str) -> Result<(), String> {
    if !["pause", "resume"].contains(&function) {
        return Err("Unsupported emulator command".into());
    }
    let address = ([127, 0, 0, 1], port).into();
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_millis(800))
        .map_err(|e| format!("Emulator control connection: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(800)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_millis(800)))
        .map_err(|e| e.to_string())?;
    write!(stream,"POST /api/v1/execution-flow?function={function} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").map_err(|e|e.to_string())?;
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .map_err(|e| e.to_string())?;
    if line.split_whitespace().nth(1) != Some("200") {
        return Err(format!("Emulator rejected command: {}", line.trim()));
    }
    Ok(())
}
fn compile_command(
    mut command: Command,
    tx: &Sender<Event>,
    rx: &Receiver<Control>,
    build_log: &Arc<Mutex<fs::File>>,
) -> Result<String, String> {
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    quiet(&mut command);
    let mut child = command
        .spawn()
        .map_err(|e| format!("Cannot launch native build command: {e}"))?;
    let readers = output(&mut child, tx, true, Some(build_log));
    loop {
        if matches!(
            rx.try_recv(),
            Ok(Control::Stop) | Err(mpsc::TryRecvError::Disconnected)
        ) {
            stop(&mut child);
            return Err("Build cancelled".into());
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut captured = String::new();
                for reader in readers {
                    captured.push_str(
                        &reader
                            .join()
                            .map_err(|_| "Native build output reader failed")?,
                    );
                }
                return if status.success() {
                    Ok(captured)
                } else {
                    Err(format!(
                        "C++ compilation failed ({status}); no executable launched"
                    ))
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(40)),
            Err(e) => {
                stop(&mut child);
                return Err(e.to_string());
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn execute(
    root: &Path,
    input: &crate::scene_dependencies::Input,
    run: bool,
    debug: bool,
    physical_disc: bool,
    analyze: bool,
    bridge_script: Option<&Path>,
    tx: &Sender<Event>,
    rx: &Receiver<Control>,
) -> Result<(), String> {
    let scene = &input.scene;
    let profile = input.play.as_ref();
    let serial = run && profile.is_some_and(|p| p.target == crate::play::Target::Serial);
    let mut serial_settings = serial
        .then(|| crate::settings::Preferences::load().map(|p| p.serial))
        .transpose()?;
    if let Some(settings) = &mut serial_settings {
        if debug {
            return Err("Blueprint debugging is available only in the emulator. Disable it before Serial Play.".into());
        }
        let task = crate::serial_support::Task { tx, rx };
        (|| {
            crate::serial_support::prepare(settings, &task)?;
            settings.executable.clear();
            crate::serial_support::discover(settings, &task)?;
            crate::serial_support::check_port(settings)
        })()
        .inspect_err(|e| {
            let _ = tx.send(Event::SerialIssue(e.clone()));
        })?;
    }
    let _ = tx.send(Event::Stage("Building PSX program".into()));
    let config = Config::load(root)?;
    let log_path = root.join(".epok/Build.log");
    fs::create_dir_all(log_path.parent().unwrap()).map_err(|e| format!("Build log: {e}"))?;
    let build_log = Arc::new(Mutex::new(
        fs::File::create(&log_path).map_err(|e| format!("Build log: {e}"))?,
    ));
    let _ = tx.send(Event::Log(format!(
        "Full build log: {}",
        log_path.display()
    )));
    let mut timing = crate::scene_loading::Progress::new();
    let mut report = |message: crate::scene_loading::Message| {
        if let Ok(mut log) = build_log.lock() {
            let _ = writeln!(log, "{}", message.text);
        }
        let _ = tx.send(Event::Log(message.text));
    };
    let build = root.join(if debug {
        ".epok/build-blueprint-debug"
    } else {
        ".epok/build"
    });
    let receipt_request = (!physical_disc)
        .then(|| crate::play_cache::request(root, input, &config, debug))
        .transpose()?;
    timing.stage("Checking previous PSX build", &mut report);
    let cached = if let Some(receipt) = receipt_request
        .as_deref()
        .and_then(|request| crate::play_cache::Receipt::load(&build, request))
    {
        let invocation = crate::build_inputs::Invocation::new(root, &build, &config)?;
        let _ = tx.send(Event::Log("Validating cached PSX build...".into()));
        let native =
            crate::build_inputs::prepare(root, &build, &config, &invocation, &mut |arguments| {
                compile_command(invocation.command(arguments), tx, rx, &build_log)
            })?;
        report(crate::scene_loading::Message::new(format!(
            "PsyQo SDK: {}.",
            native.sdk_preparation()
        )));
        if native.requires_rebuild() {
            None
        } else {
            match receipt.reuse(root, &build) {
                Ok(outputs) => {
                    native.record_reused(&build)?;
                    let _ = tx.send(Event::Log(
                        "Reusing verified PSX build; no compilation or disc generation needed."
                            .into(),
                    ));
                    Some(outputs)
                }
                Err(reason) => {
                    let _ = tx.send(Event::Log(format!("Cached build needs refresh: {reason}")));
                    None
                }
            }
        }
    } else {
        None
    };
    let (exe, disc) = if let Some(outputs) = cached {
        outputs
    } else {
        timing.stage("Preparing scene and build files", &mut report);
        let nugget = Config::path(root, &config.nugget);
        if !nugget.join("psyqo/psyqo.mk").is_file() {
            return Err(
                "PsyQo missing: run the host setup tool or configure nugget in Editor.epokconfig"
                    .into(),
            );
        }
        let _ = tx.send(Event::Log(
            if crate::lighting::valid_bake(scene) {
                "Using cached vertex lighting"
            } else {
                "Preparing vertex lighting / static shadows on PC..."
            }
            .into(),
        ));
        let mut prepared = scene.clone();
        project::refresh_linked_scene(root, &mut prepared)?;
        if prepared
            .actors
            .iter()
            .any(|e| e.editable_mesh.is_some() || e.terrain.is_some())
        {
            let index = crate::assets::scan(root, &mut Default::default());
            crate::mesh::resolve(&mut prepared, &index)?;
            // The bake reads the terrain surface, so an unresolved grid would
            // silently bake no colours for it rather than failing.
            crate::terrain::resolve(&mut prepared, &index)?;
        }
        // Report a terrain over its limits before the generic lighting budget
        // does: this message names the actor and says which limit it is.
        crate::terrain::validate_scene(&prepared)?;
        if !crate::lighting::valid_bake(&prepared) {
            let bake = crate::lighting::bake(&prepared)?;
            let _ = tx.send(Event::LightingBaked(bake.clone()));
            prepared.bake = Some(bake);
        }
        project::stage_prepared(root, input, &prepared, &build)?;
        let patches = crate::staging_files::Capture::begin(&build)?;
        if debug {
            let sources =
                fs::read_to_string(build.join("sources.mk")).map_err(|e| e.to_string())?;
            if !sources.contains("-DEPOK_BLUEPRINTS") {
                return Err("Blueprint debugging requires at least one Blueprint class.".into());
            }
            project::write_changed(
                &build.join("sources.mk"),
                format!("{sources}CPPFLAGS += -DEPOK_BLUEPRINT_TRACE=1\n").as_bytes(),
            )?;
        }
        if physical_disc {
            let settings = crate::disc::Settings::load(root)?;
            crate::disc::physical_manifest(root, &build, &settings)?;
            let _ = tx.send(Event::Log(format!(
                "Disc target: {} / {}",
                settings.region.label(),
                settings.format.label()
            )));
        }
        crate::staging_files::patch(
            root,
            &build,
            patches.finish(),
            crate::scene_dependencies::hash((&config, debug, physical_disc, profile)),
        )?;
        let invocation = crate::build_inputs::Invocation::new(root, &build, &config)?;
        let mut run_make = |arguments: &[std::ffi::OsString]| {
            if arguments.iter().any(|a| a == "epok-build-inputs") {
                let scope = if arguments.iter().any(|a| a == "-C") {
                    "SDK"
                } else {
                    "game"
                };
                let _ = tx.send(Event::Log(format!(
                    "Checking {scope} compiler dependencies..."
                )));
            } else if arguments.iter().any(|a| a == "epok-sdk-archive") {
                let _ = tx.send(Event::Log("Compiling PsyQo SDK...".into()));
            }
            compile_command(invocation.command(arguments), tx, rx, &build_log)
        };
        let _ = tx.send(Event::Log(
            "Preparing native compiler and linker inputs...".into(),
        ));
        timing.stage("Preparing native dependencies", &mut report);
        let native =
            crate::build_inputs::prepare(root, &build, &config, &invocation, &mut run_make)?;
        report(crate::scene_loading::Message::new(format!(
            "PsyQo SDK: {}. Application rebuild: {}.",
            native.sdk_preparation(),
            if native.forces_full_recompile() {
                "full"
            } else if native.requires_rebuild() {
                "incremental"
            } else {
                "not required"
            }
        )));
        timing.stage("Validating staged build inputs", &mut report);
        let ticket = crate::staging_files::BuildTicket::begin_native(root, &build)?;
        timing.stage("Compiling and linking game", &mut report);
        let _ = tx.send(Event::Log("Compiling C++ / PsyQo for MIPS...".into()));
        run_make(&native.arguments())?;
        let exe = build.join("epok.ps-exe");
        let bytes = fs::read(&exe).map_err(|e| format!("Missing PSX executable: {e}"))?;
        if !bytes.starts_with(b"PS-X EXE") {
            return Err("Build output is not a PS-X EXE".into());
        }
        let _ = tx.send(Event::Log("Verifying compiled inputs...".into()));
        timing.stage("Verifying native compiler inputs", &mut report);
        native.verify(root, &build, &config, &invocation, &mut run_make)?;
        timing.stage("Certifying build sources and output", &mut report);
        let certificate = ticket.clone();
        ticket.complete(root, &build, &bytes)?;
        timing.stage("Preparing disc image", &mut report);
        let disc = if profile.is_some_and(|p| p.data == crate::play::DataSource::Host) {
            None
        } else {
            crate::disc::build(root, &build, || {
                matches!(
                    rx.try_recv(),
                    Ok(Control::Stop) | Err(mpsc::TryRecvError::Disconnected)
                )
            })?
        };
        if let Some(request) = &receipt_request {
            crate::play_cache::Receipt::save(
                &build,
                request.clone(),
                certificate,
                disc.as_deref(),
            )?;
        }
        (exe, disc)
    };
    for warning in crate::memory::build_warnings(&build)? {
        let line = format!("Warning: {warning}");
        if let Ok(mut log) = build_log.lock() {
            let _ = writeln!(log, "{line}");
        }
        let _ = tx.send(Event::Log(line));
    }
    let automatic_report =
        crate::workspace::optional_manifest(root)?.is_none_or(|m| m.build.generate_asset_report);
    let mut summary = crate::build_report::Summary::capture(
        &build,
        profile.cloned().unwrap_or_default(),
        debug,
        receipt_request,
        disc.as_deref(),
        automatic_report,
    )?;
    // The linker's fit check does not reserve heap, stack, or PsyQo callback
    // state. Always inspect the linked image before declaring a PSX build safe.
    let _ = tx.send(Event::Stage("Validating PSX runtime memory budget".into()));
    let generated_report =
        crate::memory::analyze(root, &build, summary.profile.clone(), debug, &config)?;
    crate::memory::validate_runtime_headroom(&generated_report)?;
    if analyze || automatic_report {
        summary.verify_inputs(root)?;
        summary.record_report(root)?;
    }
    summary.save(root)?;
    let _ = tx.send(Event::MemoryReport(Box::new(generated_report)));
    let _ = tx.send(Event::Log(format!("Build sizes: {}", summary.status())));
    let _ = tx.send(Event::BuildSummary(Box::new(summary)));
    let _ = tx.send(Event::Built(disc.clone().unwrap_or_else(|| exe.clone())));
    timing.finish("PSX build ready", &mut report);
    if !run {
        return Ok(());
    }
    if let Some(settings) = &serial_settings {
        return crate::serial::run(
            &build,
            &exe,
            settings,
            profile.is_some_and(|p| p.data == crate::play::DataSource::Host),
            tx,
            rx,
        );
    }
    if debug {
        let map = fs::read_to_string(build.join("epok.map"))
            .map_err(|e| format!("Blueprint debug link map: {e}"))?;
        let symbol = |name: &str| -> Result<u32, String> {
            map.lines()
                .find_map(|line| {
                    let words = line.split_whitespace().collect::<Vec<_>>();
                    if words.last() != Some(&name) {
                        return None;
                    }
                    words.iter().find_map(|word| {
                        word.strip_prefix("0x")
                            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                    })
                })
                .ok_or_else(|| format!("Instrumented build is missing debugger symbol {name}"))
        };
        crate::blueprint_debug::write_config(
            root,
            symbol("epok_blueprint_debug_hook")?,
            symbol("epok_blueprint_debug_snapshot")?,
        )?;
    } else {
        crate::blueprint_debug::clear_config(root)?;
    }
    if matches!(
        rx.try_recv(),
        Ok(Control::Stop) | Err(mpsc::TryRecvError::Disconnected)
    ) {
        return Err("Play cancelled".into());
    }
    let reservation = TcpListener::bind(("127.0.0.1", config.web_port)).map_err(|e| {
        format!(
            "Port {} is busy: {e}. Choose a free web_port",
            config.web_port
        )
    })?;
    let portable = root.join(".epok/emulator");
    fs::create_dir_all(&portable).map_err(|e| e.to_string())?;
    // Windows GUI-subsystem binaries may leave CRT stdout detached even when
    // ConPTY supplies a console. Redux's own logfile preserves MIPS diagnostics.
    let runtime_log = portable.join("runtime.log");
    fs::File::create(&runtime_log).map_err(|e| format!("Runtime log: {e}"))?;
    let mut runtime_output = fs::File::open(&runtime_log).map_err(|e| e.to_string())?;
    crate::settings::configure_emulator(&portable, &crate::settings::Preferences::load()?)?;
    let mut emulator = Config::executable(root, &config.emulator);
    // The Windows distribution's .exe is an updater that spawns .main. Launch
    // the installed binary directly so this PID also owns the debugger window.
    if cfg!(windows) && emulator.file_name().is_some_and(|n| n == "pcsx-redux.exe") {
        let direct = emulator.with_file_name("pcsx-redux.main");
        if direct.is_file() {
            emulator = direct;
        }
    }
    let mut command = Command::new(&emulator);
    command
        .current_dir(&portable)
        .arg("-portable")
        .arg(&portable)
        .arg("-logfile")
        .arg(&runtime_log)
        .args([
            "-run",
            "-stdout",
            "-fastboot",
            "-noupdate",
            "-interpreter",
            "-softgpu",
            "-2mb",
            "-no-gdb",
            "-webserver",
            "-webserver-port",
        ])
        .arg(config.web_port.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    if let Some(cue) = &disc {
        command.arg("-iso").arg(cue);
    }
    // Mount the CD for on-demand assets, but enter the compiled program
    // directly instead of waiting through the BIOS's CD boot animation.
    command.arg("-loadexe").arg(&exe);
    if profile.is_none_or(|p| p.target == crate::play::Target::Embedded) {
        command.arg("-no-ui");
    }
    if debug {
        command.arg("-debugger");
    }
    if profile.is_some_and(|p| p.data == crate::play::DataSource::Host) {
        command.arg("-pcdrv").arg("-pcdrvbase").arg(&build);
    } else {
        command.arg("-no-pcdrv");
    }
    if let Some(parent) = emulator.parent() {
        let fast = parent.join("openbios-fastboot.bin");
        let bios = if fast.is_file() {
            fast
        } else {
            parent.join("openbios.bin")
        };
        if bios.is_file() {
            command.arg("-bios").arg(bios);
        }
    }
    quiet(&mut command);
    drop(reservation);
    if let Some(script) = bridge_script {
        command.arg("-dofile").arg(script);
    }
    // Redux calls AllocConsole itself for stdout/no-ui, defeating
    // CREATE_NO_WINDOW. An already-attached private ConPTY/PTY prevents that
    // extra window and retains game diagnostics. Embedded Play has no GUI.
    let mut child = crate::serial_terminal::Terminal::spawn(command)
        .map_err(|e| format!("Cannot launch {}: {e}", emulator.display()))?;
    let pid = child.process_id().ok_or("Emulator process has no PID")?;
    let _ = tx.send(Event::Running(pid));
    let mut pending_log = Vec::new();
    let mut terminal_diagnostics = String::new();
    loop {
        // Keep terminal-only startup errors for an unsuccessful exit; ordinary
        // log lines come from one source, avoiding duplicates on Unix PTYs.
        for chunk in child.output.try_iter() {
            if terminal_diagnostics.len() < 16000 {
                terminal_diagnostics.push_str(&chunk);
            }
        }
        let mut buffer = [0u8; 8192];
        // Bound each poll so a very chatty game cannot prevent Stop/Pause.
        for _ in 0..8 {
            let n = runtime_output
                .read(&mut buffer)
                .map_err(|e| format!("Runtime log: {e}"))?;
            if n == 0 {
                break;
            }
            pending_log.extend_from_slice(&buffer[..n]);
            while let Some(end) = pending_log.iter().position(|&b| b == b'\n') {
                let bytes: Vec<_> = pending_log.drain(..=end).collect();
                let line = String::from_utf8_lossy(&bytes);
                let line = line.trim_end_matches(['\r', '\n']);
                if !line.is_empty() {
                    let _ = tx.send(Event::Log(line.chars().take(2000).collect()));
                }
            }
            if pending_log.len() > 16000 {
                pending_log.clear();
            }
        }
        match rx.try_recv() {
            Ok(Control::Stop) | Err(mpsc::TryRecvError::Disconnected) => {
                return Ok(());
            }
            Ok(Control::Pause) => match request(config.web_port, "pause") {
                Ok(()) => {
                    let _ = tx.send(Event::Paused(true));
                }
                Err(e) => {
                    let _ = tx.send(Event::Log(e));
                }
            },
            Ok(Control::Resume) => match request(config.web_port, "resume") {
                Ok(()) => {
                    let _ = tx.send(Event::Paused(false));
                }
                Err(e) => {
                    let _ = tx.send(Event::Log(e));
                }
            },
            Ok(Control::Reset) => {} // Reset is a physical PSX session control.
            Err(mpsc::TryRecvError::Empty) => {}
        }
        match child.poll() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    if !terminal_diagnostics.is_empty() {
                        let _ = tx.send(Event::Log(terminal_diagnostics));
                    }
                    Err(format!("Emulator exited: {status}"))
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(40)),
            Err(e) => {
                return Err(e.to_string());
            }
        }
    }
}

#[cfg(test)]
mod output_tests {
    use super::native_command_echo;
    #[test]
    #[ignore = "requires a disposable Ironwood copy and native tools; builds without launching an emulator"]
    fn profile_play_preparation() {
        let root = std::path::PathBuf::from(std::env::var_os("EPOK_IDLE_PROJECT").unwrap());
        let mut editor =
            crate::editor::Editor::open(crate::workspace::Project::open(&root).unwrap()).unwrap();
        editor
            .open_scene(root.join("assets/scenes/ForestClearing.epokmap"))
            .unwrap();
        if std::env::var_os("EPOK_TEST_DEBUG_HUD").is_some() {
            let mut manifest = crate::workspace::read_manifest(&root).unwrap();
            manifest.debug = crate::settings::DebugHud {
                fps: true,
                cpu: true,
                gte: true,
                gpu: true,
                spu_ram: true,
            };
            editor.apply_project_settings(manifest).unwrap();
        }
        for iteration in 1..=2 {
            let input = crate::play::input(
                &root,
                editor.scene_path(),
                editor.scene.clone(),
                editor.play_profile.clone(),
                false,
            )
            .unwrap();
            let started = std::time::Instant::now();
            let job = super::Job::start_with_debug(root.clone(), input, false, false);
            let mut sdk_builds = 0;
            let mut reused = false;
            let mut game_builds = 0;
            loop {
                assert!(started.elapsed().as_secs() < 180, "Build timed out");
                match job.events.recv_timeout(std::time::Duration::from_secs(1)) {
                    Ok(super::Event::Log(line)) => {
                        if line == "Compiling PsyQo SDK..." {
                            sdk_builds += 1;
                        }
                        if line.starts_with("Reusing verified PSX build") {
                            reused = true;
                        }
                        if line == "Compiling C++ / PsyQo for MIPS..." {
                            game_builds += 1;
                        }
                        eprintln!(
                            "[build {iteration}, {:.3}s] {line}",
                            started.elapsed().as_secs_f64()
                        );
                    }
                    Ok(super::Event::LightingBaked(bake)) => editor.scene.bake = Some(bake),
                    Ok(super::Event::Finished(result)) => {
                        result.unwrap();
                        break;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        panic!("Build worker disconnected")
                    }
                    _ => {}
                }
            }
            eprintln!("Play build {iteration}: {:?}", started.elapsed());
            if iteration == 2 {
                assert_eq!(sdk_builds, 0, "Unchanged SDK must be reused");
                assert_eq!(game_builds, 0, "Unchanged game must not recompile");
                assert!(reused, "Unchanged game/disc must use the certified receipt");
            }
        }
    }
    #[test]
    fn compiler_echo_filter_preserves_errors_warnings_and_unrecognized_output() {
        for line in [
            "mipsel-none-elf-g++ -std=c++20 -M -MF main.dep main.cpp",
            "mipsel-none-elf-gcc -o epok.elf main.o",
            "mipsel-none-elf-gcc-ar rcs libpsyqo.a scene.o",
            "C:/tools/mipsel-none-elf-objcopy.exe -O binary epok.elf epok.ps-exe",
        ] {
            assert!(native_command_echo(line), "{line}");
        }
        for line in [
            "mipsel-none-elf-g++: fatal error: no input files",
            "mipsel-none-elf-gcc.exe: error: unrecognized command-line option",
            "scripts/Actor.cpp:4: error: missing symbol",
            "scripts/Actor.cpp:8: warning: unused variable",
            "collect2.exe: error: ld returned 1 exit status",
            "make: *** [main.o] Error 1",
            "Build complete.",
            "",
        ] {
            assert!(!native_command_echo(line), "{line}");
        }
    }
}
