//! Bounded, pull-based video transport. One frame in flight; latest frame replaces the previous.
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU16, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub sequence: u64,
    pub buttons: u16,
    pub cycles: u64,
    pub vsyncs: u64,
    pub rgba: Vec<u8>,
}
#[derive(Default)]
pub struct State {
    pub frame: Option<Arc<Frame>>,
    pub connected: bool,
    pub error: Option<String>,
}
pub struct Bridge {
    pub script: PathBuf,
    pub state: Arc<Mutex<State>>,
    pub buttons: Arc<AtomicU16>,
    pub step: Arc<AtomicBool>,
    pub debug: Arc<Mutex<crate::blueprint_debug::State>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Bridge {
    pub fn start(root: &Path) -> Result<Self, String> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let token = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let script = root.join(".epok/emulator/bridge.lua");
        std::fs::create_dir_all(script.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(
            &script,
            format!(
                "EPOK_PORT={}\nEPOK_TOKEN={token:?}\nEPOK_DEBUG_CONFIG={:?}\nEPOK_ANALOG_CONTROLLER={}\n{}",
                listener.local_addr().unwrap().port(),
                root.join(".epok/emulator/blueprint-debug.lua")
                    .to_string_lossy(),
                crate::play::Profile::load(root)?.analog_controller,
                include_str!("../integrations/pcsx-redux/bridge.lua")
            ),
        )
        .map_err(|e| e.to_string())?;
        let state = Arc::new(Mutex::new(State::default()));
        let buttons = Arc::new(AtomicU16::new(0));
        let step = Arc::new(AtomicBool::new(false));
        let debug = Arc::new(Mutex::new(crate::blueprint_debug::State::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let (shared, input, stopping) = (state.clone(), buttons.clone(), stop.clone());
        let stepping = step.clone();
        let debugging = debug.clone();
        let worker = thread::spawn(move || {
            while !stopping.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let result = receive(
                            stream, &token, &shared, &input, &stopping, &stepping, &debugging,
                        );
                        let mut state = shared.lock().unwrap();
                        state.connected = false;
                        if !stopping.load(Ordering::Relaxed) {
                            state.error = result.err();
                        }
                        break;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20))
                    }
                    Err(e) => {
                        shared.lock().unwrap().error = Some(e.to_string());
                        break;
                    }
                }
            }
        });
        Ok(Self {
            script,
            state,
            buttons,
            step,
            debug,
            stop,
            worker: Some(worker),
        })
    }
}
impl Drop for Bridge {
    fn drop(&mut self) {
        self.buttons.store(0, Ordering::Relaxed);
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn line(reader: &mut BufReader<TcpStream>) -> Result<String, String> {
    let mut text = String::new();
    reader
        .take(128)
        .read_line(&mut text)
        .map_err(|e| e.to_string())?;
    if !text.ends_with('\n') {
        return Err("Incomplete bridge header".into());
    }
    Ok(text)
}
fn receive(
    stream: TcpStream,
    token: &str,
    state: &Mutex<State>,
    buttons: &AtomicU16,
    stop: &AtomicBool,
    step: &AtomicBool,
    debug: &Mutex<crate::blueprint_debug::State>,
) -> Result<(), String> {
    stream.set_nonblocking(false).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_millis(500)))
        .map_err(|e| e.to_string())?;
    stream.set_nodelay(true).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(stream);
    if line(&mut reader)?.trim() != format!("EPOK {token}") {
        return Err("Bridge session mismatch".into());
    }
    state.lock().unwrap().connected = true;
    while !stop.load(Ordering::Relaxed) {
        let started = Instant::now();
        let command = debug.lock().unwrap().next_command();
        if let Some(command) = command {
            writeln!(reader.get_mut(), "{}", command.wire()).map_err(|error| error.to_string())?;
            let response = line(&mut reader)?;
            match response.trim(){
                "EPKA1 1"=>debug.lock().unwrap().error=None,
                "EPKA1 0"=>debug.lock().unwrap().error=Some("Blueprint debugger command rejected: instrumented interpreter/debugger required, or breakpoint limit reached.".into()),
                _=>return Err("Invalid Blueprint debugger command acknowledgement.".into()),
            }
        }
        writeln!(
            reader.get_mut(),
            "{} {}",
            if step.swap(false, Ordering::Relaxed) {
                "S"
            } else {
                "F"
            },
            buttons.load(Ordering::Relaxed)
        )
        .map_err(|e| e.to_string())?;
        let mut header = line(&mut reader)?;
        if header.starts_with("EPKD1 ") {
            receive_debug(&mut reader, &header, debug)?;
            header = line(&mut reader)?;
        }
        if header.trim() == "EPKF0" {
            thread::sleep(Duration::from_millis(16));
            continue;
        }
        let parts: Vec<_> = header.split_whitespace().collect();
        if parts.len() != 8 || parts[0] != "EPKF1" {
            return Err("Invalid frame header".into());
        }
        let values: Vec<u64> = parts[1..]
            .iter()
            .map(|v| v.parse::<u64>().map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let (width, height, depth) = (values[0], values[1], values[2]);
        if width == 0
            || height == 0
            || width > 1024
            || height > 512
            || ![2, 3].contains(&depth)
            || values[4] > 65535
        {
            return Err(format!(
                "Unsupported frame dimensions or format: {}",
                header.trim()
            ));
        }
        let mut data = vec![0; (width * height * depth) as usize];
        reader.read_exact(&mut data).map_err(|e| e.to_string())?;
        let rgba = decode(&data, depth as usize);
        state.lock().unwrap().frame = Some(Arc::new(Frame {
            width: width as u32,
            height: height as u32,
            sequence: values[3],
            buttons: values[4] as u16,
            cycles: values[5],
            vsyncs: values[6],
            rgba,
        }));
        if let Some(wait) = Duration::from_millis(16).checked_sub(started.elapsed()) {
            thread::sleep(wait);
        }
    }
    let _ = writeln!(reader.get_mut(), "F 0");
    Ok(())
}
fn receive_debug(
    reader: &mut impl Read,
    header: &str,
    state: &Mutex<crate::blueprint_debug::State>,
) -> Result<(), String> {
    let parts = header.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 7 {
        return Err("Invalid Blueprint debug packet header.".into());
    }
    let values = parts[1..]
        .iter()
        .map(|value| value.parse::<u32>().map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    if values[..3].iter().any(|&value| value > 1)
        || values[4] > 64
        || ![0, crate::blueprint_debug::SNAPSHOT_BYTES as u32].contains(&values[5])
    {
        return Err("Blueprint debug packet exceeds its bounded format.".into());
    }
    let snapshot = if values[5] != 0 {
        let mut bytes = [0u8; crate::blueprint_debug::SNAPSHOT_BYTES];
        reader
            .read_exact(&mut bytes)
            .map_err(|error| error.to_string())?;
        Some(crate::blueprint_debug::snapshot(&bytes)?)
    } else {
        None
    };
    let mut traces = vec![];
    for _ in 0..values[4] {
        let mut bytes = [0u8; 16];
        reader
            .read_exact(&mut bytes)
            .map_err(|error| error.to_string())?;
        let words = bytes
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
            .collect::<Vec<_>>();
        if words[2] > 65535 {
            return Err("Invalid Blueprint trace owner.".into());
        }
        traces.push(crate::blueprint_debug::Trace {
            class_id: words[0],
            node_id: words[1],
            owner: words[2] as u16,
            generation: words[3],
        });
    }
    let mut state = state.lock().unwrap();
    state.available = values[0] != 0;
    state.paused = values[1] != 0;
    state.at_node = values[2] != 0;
    state.dropped = state.dropped.saturating_add(u64::from(values[3]));
    if snapshot.is_some() {
        state.snapshot = snapshot;
    }
    for trace in traces {
        state.push_trace(trace);
    }
    Ok(())
}
fn decode(data: &[u8], depth: usize) -> Vec<u8> {
    // Allocate exactly once. Avoid a per-channel flat_map/collect iterator
    // stack: it is expensive for 640x480 video in the unoptimized Debug editor.
    let mut rgba = vec![255; (data.len() / depth) * 4];
    if depth == 3 {
        for (pixel, output) in data.chunks_exact(3).zip(rgba.chunks_exact_mut(4)) {
            output[..3].copy_from_slice(pixel);
        }
        return rgba;
    }
    for (p, output) in data.chunks_exact(2).zip(rgba.chunks_exact_mut(4)) {
        let p = u16::from_le_bytes([p[0], p[1]]);
        output[0] = ((p & 31) << 3 | (p & 31) >> 2) as u8;
        output[1] = (((p >> 5) & 31) << 3 | ((p >> 5) & 31) >> 2) as u8;
        output[2] = (((p >> 10) & 31) << 3 | ((p >> 10) & 31) >> 2) as u8;
    }
    rgba
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blueprint_packets_are_bounded_and_do_not_overwrite_video_state() {
        let state = Mutex::new(crate::blueprint_debug::State::default());
        let mut words = [0u32; 119];
        words[..7].copy_from_slice(&[0x55425144, 1, 1, 2, 3, 4, 0]);
        let mut bytes = words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect::<Vec<_>>();
        bytes.extend([1u32, 2, 3, 4].iter().flat_map(|word| word.to_le_bytes()));
        receive_debug(
            &mut std::io::Cursor::new(bytes),
            "EPKD1 1 1 1 7 1 476\n",
            &state,
        )
        .unwrap();
        let debug = state.lock().unwrap();
        assert!(debug.available && debug.paused && debug.at_node);
        assert_eq!(debug.dropped, 7);
        assert_eq!(debug.traces.len(), 1);
        assert_eq!(debug.snapshot.as_ref().unwrap().owner, 3);
        drop(debug);
        assert!(receive_debug(&mut std::io::empty(), "EPKD1 1 1 1 0 65 0\n", &state).is_err());
        assert!(receive_debug(&mut std::io::empty(), "EPKD1 1 1 1 0 0 477\n", &state).is_err());
    }
    #[test]
    fn psx_color_channels_and_mask_bit() {
        assert_eq!(
            decode(&[31, 128, 224, 3, 0, 124], 2),
            [255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255]
        );
        assert_eq!(decode(&[12, 34, 56], 3), [12, 34, 56, 255]);
    }
    #[test]
    #[ignore = "requires local MIPS toolchain and PCSX-Redux; launches a real game"]
    fn live_video_input_pause_step_and_cleanup() {
        use crate::{pipeline, project, scene};
        use std::collections::HashSet;
        use std::hash::{Hash, Hasher};
        let root = std::env::var_os("EPOK_LIVE_PROJECT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/sample-game"));
        let scene_path = root.join(
            std::env::var("EPOK_LIVE_SCENE")
                .unwrap_or_else(|_| "assets/scenes/SampleScene.epokmap".into()),
        );
        let scene = scene::Scene::load(&scene_path).unwrap();
        let config = project::Config::load(&root).unwrap();
        let resolution = crate::settings::rendering(&root).unwrap();
        let profile = crate::workspace::optional_manifest(&root)
            .unwrap()
            .map(|m| m.play)
            .unwrap_or_default();
        let input = crate::play::input(&root, scene_path, scene, profile, false).unwrap();
        let job = pipeline::Job::start_with_debug(root.clone(), input, true, false);
        let bridge = job.bridge.as_ref().expect("bridge startup");
        let runtime_ready = std::cell::Cell::new(false);
        let wait_frame = |after: u64| {
            let start = Instant::now();
            loop {
                while let Ok(event) = job.events.try_recv() {
                    match event {
                        pipeline::Event::Finished(Err(e)) => panic!("{e}"),
                        pipeline::Event::Log(line) => {
                            if line.contains("EPOK: runtime ready") {
                                runtime_ready.set(true);
                            }
                            eprintln!("{line}");
                        }
                        pipeline::Event::Running(pid) => eprintln!("Emulator PID: {pid}"),
                        _ => {}
                    }
                }
                let state = bridge.state.lock().unwrap();
                assert!(state.error.is_none(), "{:?}", state.error);
                if let Some(frame) = &state.frame
                    && frame.sequence > after
                {
                    return frame.clone();
                }
                drop(state);
                assert!(
                    start.elapsed() < Duration::from_secs(if after == 0 { 180 } else { 15 }),
                    "No video frame"
                );
                thread::sleep(Duration::from_millis(10));
            }
        };
        let hash = |f: &Frame| {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            f.rgba.hash(&mut h);
            h.finish()
        };
        let mut f = wait_frame(0);
        let mut distinct = HashSet::new();
        let started = Instant::now();
        let first_vsync = f.vsyncs;
        for _ in 0..100 {
            f = wait_frame(f.sequence);
            distinct.insert(hash(&f));
        }
        assert!(distinct.len() > 10, "Video is frozen");
        assert!(
            runtime_ready.get(),
            "Runtime logs must reach the editor Console"
        );
        assert_eq!(f.width, u32::from(resolution.width));
        assert_eq!(f.height, u32::from(resolution.height));
        println!(
            "Video: {} unique frames, {:.1} received fps, {}x{}",
            distinct.len(),
            100. / started.elapsed().as_secs_f32(),
            f.width,
            f.height
        );
        let emulated_hz = (f.vsyncs - first_vsync) as f64 / started.elapsed().as_secs_f64();
        println!("Emulated vblank rate: {emulated_hz:.1} Hz");
        assert!(
            emulated_hz < 75.,
            "Headless emulator must not run unthrottled"
        );
        if let Some(path) = std::env::var_os("EPOK_LIVE_CAPTURE") {
            let mut encoder =
                png::Encoder::new(std::fs::File::create(path).unwrap(), f.width, f.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&f.rgba)
                .unwrap();
        }
        bridge
            .buttons
            .store((1 << 4) | (1 << 14), Ordering::Relaxed);
        let mut acknowledged = false;
        for _ in 0..20 {
            f = wait_frame(f.sequence);
            if f.buttons & (1 << 4 | 1 << 14) == (1 << 4 | 1 << 14) {
                acknowledged = true;
                break;
            }
        }
        assert!(acknowledged, "Pad override not acknowledged");
        bridge.buttons.store(0, Ordering::Relaxed);
        for _ in 0..10 {
            f = wait_frame(f.sequence);
            if f.buttons == 0 {
                break;
            }
        }
        assert_eq!(f.buttons, 0, "Pad buttons stuck");
        pipeline::request(config.web_port, "pause").unwrap();
        thread::sleep(Duration::from_millis(100));
        f = wait_frame(f.sequence);
        let paused = hash(&f);
        for _ in 0..10 {
            f = wait_frame(f.sequence);
            assert_eq!(hash(&f), paused, "Video changed while paused");
        }
        let before_vsync = f.vsyncs;
        let before_cycles = f.cycles;
        bridge.step.store(true, Ordering::Relaxed);
        for _ in 0..15 {
            f = wait_frame(f.sequence);
        }
        assert_eq!(
            f.vsyncs,
            before_vsync + 1,
            "Step did not advance exactly one vblank"
        );
        assert!(f.cycles > before_cycles, "CPU did not advance");
        let stepped = hash(&f);
        let stepped_cycles = f.cycles;
        for _ in 0..10 {
            f = wait_frame(f.sequence);
            assert_eq!(f.cycles, stepped_cycles, "Step did not pause CPU");
            assert_eq!(hash(&f), stepped, "Step did not stop at vblank");
        }
        pipeline::request(config.web_port, "resume").unwrap();
        let mut changed = false;
        for _ in 0..15 {
            f = wait_frame(f.sequence);
            changed |= hash(&f) != stepped;
        }
        assert!(changed, "Resume did not animate");
        drop(job);
        let socket =
            TcpListener::bind(("127.0.0.1", config.web_port)).expect("Emulator not reaped");
        drop(socket);
        println!("PASS video, input press/release, pause, vblank step, resume and process cleanup");
    }
}
