//! Bounded, read-only Blueprint snapshots and commands for the emulator debugger.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, VecDeque},
    path::{Path, PathBuf},
};

pub const SNAPSHOT_BYTES: usize = 476;
pub const MAX_TRACES: usize = 128;
pub const MAX_COMMANDS: usize = 32;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugValue {
    pub member: u32,
    pub value_type: u32,
    pub length: u32,
    pub words: [u32; 4],
}
impl DebugValue {
    pub fn display(&self) -> String {
        let signed = |word: u32| word as i32;
        match self.value_type {
            1 => {
                if self.words[0] == 0 {
                    "false".into()
                } else {
                    "true".into()
                }
            }
            2 | 5 => signed(self.words[0]).to_string(),
            3 => self.words[0].to_string(),
            4 => format!("{:.4}", f64::from(signed(self.words[0])) / 4096.),
            6 | 7 | 11 => format!(
                "[{}]",
                self.words[..self.length as usize]
                    .iter()
                    .map(|&word| format!("{:.4}", f64::from(signed(word)) / 4096.))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            8 => format!("slot {} / generation {}", self.words[0], self.words[1]),
            9 | 10 => format!(
                "{:016x}",
                u64::from(self.words[0]) | (u64::from(self.words[1]) << 32)
            ),
            _ => "Unsupported debug value".into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub class_id: u32,
    pub node_id: u32,
    pub owner: u16,
    pub generation: u32,
    pub values: Vec<DebugValue>,
}
pub fn snapshot(bytes: &[u8]) -> Result<Snapshot, String> {
    if bytes.len() != SNAPSHOT_BYTES {
        return Err("Invalid Blueprint snapshot size.".into());
    }
    let words = bytes
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .collect::<Vec<_>>();
    if words[0] != 0x55425144 || words[1] != 1 || words[4] > 65535 || words[6] > 16 {
        return Err("Unsupported Blueprint snapshot header.".into());
    }
    let mut values = vec![];
    let mut ids = std::collections::BTreeSet::new();
    for i in 0..words[6] as usize {
        let entry = &words[7 + i * 7..7 + (i + 1) * 7];
        let expected = match entry[1] {
            1..=5 => 1,
            6 | 8 | 9 | 10 => 2,
            7 => 3,
            11 => 4,
            _ => return Err("Unsupported Blueprint value type.".into()),
        };
        if entry[2] != expected
            || !ids.insert(entry[0])
            || (entry[1] == 1 && entry[3] > 1)
            || (entry[1] == 8 && entry[3] > 65535)
        {
            return Err("Invalid Blueprint member layout or duplicate identity.".into());
        }
        values.push(DebugValue {
            member: entry[0],
            value_type: entry[1],
            length: entry[2],
            words: entry[3..7].try_into().unwrap(),
        });
    }
    Ok(Snapshot {
        class_id: words[2],
        node_id: words[3],
        owner: words[4] as u16,
        generation: words[5],
        values,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trace {
    pub class_id: u32,
    pub node_id: u32,
    pub owner: u16,
    pub generation: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Breakpoint {
    pub class_id: u32,
    pub node_id: u32,
    pub instance: Option<(u16, u32)>,
}
#[derive(Clone, Debug)]
pub enum Command {
    Add(Breakpoint),
    Remove(Breakpoint),
    Clear,
    Pause,
    Step,
    Resume,
}
impl Command {
    pub fn wire(&self) -> String {
        match self {
            Self::Add(point) | Self::Remove(point) => {
                let (owner, generation) = point.instance.unwrap_or((65535, 0));
                format!(
                    "{} {} {} {} {}",
                    if matches!(self, Self::Add(_)) {
                        "B"
                    } else {
                        "X"
                    },
                    point.class_id,
                    point.node_id,
                    owner,
                    generation
                )
            }
            Self::Clear => "C".into(),
            Self::Pause => "P".into(),
            Self::Step => "N".into(),
            Self::Resume => "R".into(),
        }
    }
}
#[derive(Default)]
pub struct State {
    pub available: bool,
    pub paused: bool,
    pub at_node: bool,
    pub snapshot: Option<Snapshot>,
    pub traces: VecDeque<Trace>,
    pub dropped: u64,
    pub error: Option<String>,
    pub maps: BTreeMap<u32, SourceMap>,
    commands: VecDeque<Command>,
}
impl State {
    pub fn request(&mut self, command: Command) -> Result<(), String> {
        if self.commands.len() >= MAX_COMMANDS {
            return Err("Blueprint debugger command queue is full.".into());
        }
        self.commands.push_back(command);
        Ok(())
    }
    pub fn next_command(&mut self) -> Option<Command> {
        self.commands.pop_front()
    }
    pub fn push_trace(&mut self, trace: Trace) {
        if self.traces.len() == MAX_TRACES {
            self.traces.pop_front();
            self.dropped = self.dropped.saturating_add(1);
        }
        self.traces.push_back(trace);
    }
    pub fn clear_traces(&mut self) {
        self.traces.clear();
        self.dropped = 0;
    }
    pub fn load_maps(&mut self, build: &Path) -> Result<(), String> {
        let directory = build.join("blueprints");
        let mut maps = BTreeMap::new();
        if directory.is_dir() {
            for entry in std::fs::read_dir(directory).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if !entry.file_type().map_err(|e| e.to_string())?.is_file()
                    || !path.to_string_lossy().ends_with(".epokdebug")
                {
                    continue;
                }
                if maps.len() >= 64
                    || entry.metadata().map_err(|e| e.to_string())?.len() > 2_000_000
                {
                    return Err("Blueprint debug maps exceed the bounded host budget.".into());
                }
                let map: SourceMap =
                    crate::document::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;
                if map.version != 1
                    || map.debug_abi.version != 1
                    || map.debug_abi.words != 119
                    || map.debug_abi.entries != 16
                    || map.debug_abi.entry_words != 7
                {
                    return Err("Unsupported Blueprint debug source-map ABI.".into());
                }
                if maps.insert(map.trace_id, map).is_some() {
                    return Err("Blueprint debug class identity collision.".into());
                }
            }
        }
        self.maps = maps;
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize)]
pub struct SourceMap {
    pub version: u32,
    pub class: String,
    pub trace_id: u32,
    pub source: PathBuf,
    pub semantic_hash: String,
    pub debug_abi: Abi,
    pub members: Vec<Member>,
    pub graphs: Vec<Graph>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Abi {
    pub version: u32,
    pub words: u32,
    pub entries: u32,
    pub entry_words: u32,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Member {
    pub id: String,
    pub trace_id: u32,
    pub name: String,
    pub value_type: crate::reflection_schema::Type,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Graph {
    pub id: String,
    pub name: String,
    pub nodes: Vec<Node>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Node {
    pub id: String,
    pub trace_id: u32,
}
impl SourceMap {
    pub fn node(&self, id: u32) -> Option<(&Graph, &Node)> {
        self.graphs.iter().find_map(|graph| {
            graph
                .nodes
                .iter()
                .find(|node| node.trace_id == id)
                .map(|node| (graph, node))
        })
    }
    pub fn member(&self, id: u32) -> Option<&Member> {
        self.members.iter().find(|member| member.trace_id == id)
    }
}
fn ram_offset(address: u32, length: u32) -> Option<u32> {
    let offset = match address {
        0..=0x1fffff => address,
        0x80000000..=0x801fffff => address - 0x80000000,
        0xa0000000..=0xa01fffff => address - 0xa0000000,
        _ => return None,
    };
    (offset % 4 == 0 && offset.checked_add(length)? <= 0x200000).then_some(offset)
}
pub fn write_config(root: &Path, hook: u32, snapshot: u32) -> Result<(), String> {
    if ram_offset(hook, 4).is_none() || ram_offset(snapshot, SNAPSHOT_BYTES as u32).is_none() {
        return Err("Blueprint debugger symbols are outside aligned PSX RAM.".into());
    }
    let path = root.join("UserSettings/Breakpoints.epokprefs");
    let points: Vec<Breakpoint> = if path.is_file() {
        if std::fs::metadata(&path)
            .map_err(|error| error.to_string())?
            .len()
            > 65536
        {
            return Err("Saved Blueprint breakpoints exceed the bounded file limit.".into());
        }
        crate::document::from_slice(&std::fs::read(&path).map_err(|error| error.to_string())?)
            .map_err(|error| format!("Invalid saved Blueprint breakpoints: {error}"))?
    } else {
        vec![]
    };
    if points.len() > 32 || points.iter().any(|point| point.instance.is_some()) {
        return Err("Save at most 32 all-instance Blueprint breakpoints; generation-specific breakpoints belong to the current run only.".into());
    }
    let points = points
        .iter()
        .map(|point| {
            format!(
                "{{class={},node={},owner=65535,generation=0}}",
                point.class_id, point.node_id
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let directory = root.join(".epok/emulator");
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    std::fs::write(
        directory.join("blueprint-debug.lua"),
        format!("return {{version=1,hook={hook},snapshot={snapshot},breakpoints={{{points}}}}}\n"),
    )
    .map_err(|e| e.to_string())
}
pub fn clear_config(root: &Path) -> Result<(), String> {
    let path = root.join(".epok/emulator/blueprint-debug.lua");
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bytes(words: &[u32]) -> Vec<u8> {
        words.iter().flat_map(|word| word.to_le_bytes()).collect()
    }
    #[test]
    fn snapshot_validates_layout_and_formats_target_values() {
        let mut words = [0u32; 119];
        words[..7].copy_from_slice(&[0x55425144, 1, 10, 20, 3, 9, 2]);
        words[7..14].copy_from_slice(&[1, 4, 1, (-6144i32) as u32, 0, 0, 0]);
        words[14..21].copy_from_slice(&[2, 8, 2, 4, 99, 0, 0]);
        let state = snapshot(&bytes(&words)).unwrap();
        assert_eq!(state.values[0].display(), "-1.5000");
        assert_eq!(state.values[1].display(), "slot 4 / generation 99");
        words[6] = 17;
        assert!(snapshot(&bytes(&words)).is_err());
        words[6] = 2;
        words[16] = 4;
        assert!(snapshot(&bytes(&words)).is_err());
        assert!(snapshot(&[0; 12]).is_err());
    }
    #[test]
    fn queues_and_symbol_ranges_are_bounded() {
        let mut state = State::default();
        for _ in 0..32 {
            state.request(Command::Step).unwrap();
        }
        assert!(state.request(Command::Resume).is_err());
        assert!(state.next_command().is_some());
        for node in 0..200 {
            state.push_trace(Trace {
                class_id: 1,
                node_id: node,
                owner: 0,
                generation: 1,
            });
        }
        assert_eq!(state.traces.len(), 128);
        assert_eq!(state.dropped, 72);
        assert_eq!(ram_offset(0x80001000, 476), Some(4096));
        assert!(ram_offset(0x801ffffc, 476).is_none());
        assert!(ram_offset(0x1f800000, 4).is_none());
        assert_eq!(
            Command::Add(Breakpoint {
                class_id: 1,
                node_id: 2,
                instance: None
            })
            .wire(),
            "B 1 2 65535 0"
        );
    }
    #[test]
    #[ignore = "requires EPOK_BP_DEBUG_PROJECT, pinned SDK and exclusive real PCSX-Redux session"]
    fn live_blueprint_breakpoint_snapshot_step_resume_and_cleanup() {
        use std::{
            fs, thread,
            time::{Duration, Instant},
        };
        let root = PathBuf::from(
            std::env::var_os("EPOK_BP_DEBUG_PROJECT")
                .expect("Set EPOK_BP_DEBUG_PROJECT to the isolated Blueprint acceptance project."),
        );
        let scene =
            crate::scene::Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
        let files = crate::blueprint_asset::load_all(&root).unwrap();
        let (asset,graph)=files.iter().filter(|file|scene.actors.iter().any(|entity|entity.class.class_id.as_ref()==Some(&file.asset.id)||entity.components.iter().any(|c|c.class.class_id.as_ref()==Some(&file.asset.id))))
            .find_map(|file|file.asset.functions.iter().find(|graph|graph.name=="on_ready").map(|graph|(&file.asset,graph)))
            .expect("Acceptance scene needs an attached Blueprint overriding on_ready with health 80 or 150.");
        let point = Breakpoint {
            class_id: crate::blueprint_refs::compact_id(&asset.id) as u32,
            node_id: crate::blueprint_refs::compact_id(&graph.entry) as u32,
            instance: None,
        };
        struct Restore {
            path: PathBuf,
            original: Option<Vec<u8>>,
        }
        impl Drop for Restore {
            fn drop(&mut self) {
                if let Some(bytes) = &self.original {
                    let _ = fs::write(&self.path, bytes);
                } else {
                    let _ = fs::remove_file(&self.path);
                }
            }
        }
        let path = root.join("UserSettings/Breakpoints.epokprefs");
        let original = match fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => panic!("{error}"),
        };
        let _restore = Restore {
            path: path.clone(),
            original,
        };
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, serde_json::to_vec(&vec![point]).unwrap()).unwrap();
        let overall_started = Instant::now();
        let job = crate::pipeline::Job::start_with_debug(root.clone(), scene, true, true);
        let bridge = job.bridge.as_ref().expect("Blueprint bridge startup");
        let wait = |predicate: &dyn Fn(&State) -> bool| {
            loop {
                while let Ok(event) = job.events.try_recv() {
                    match event {
                        crate::pipeline::Event::Finished(Err(error)) => {
                            panic!("Blueprint play failed: {error}")
                        }
                        crate::pipeline::Event::Log(line) => println!("{line}"),
                        _ => {}
                    }
                }
                let video = bridge.state.lock().unwrap();
                assert!(video.error.is_none(), "Bridge failed: {:?}", video.error);
                drop(video);
                let mut state = bridge.debug.lock().unwrap();
                if state.available && state.maps.is_empty() {
                    state
                        .load_maps(&root.join(".epok/build-blueprint-debug"))
                        .unwrap();
                }
                assert!(
                    state.error.is_none(),
                    "Debugger command failed: {:?}",
                    state.error
                );
                if predicate(&state) {
                    return state.snapshot.clone();
                }
                drop(state);
                assert!(
                    overall_started.elapsed() < Duration::from_secs(120),
                    "Timed out waiting for real Blueprint debugger state."
                );
                thread::sleep(Duration::from_millis(20));
            }
        };
        let first = wait(&|state| {
            state.available
                && state.paused
                && state.at_node
                && state.snapshot.as_ref().is_some_and(|snapshot| {
                    snapshot.class_id == point.class_id && snapshot.node_id == point.node_id
                })
        })
        .unwrap();
        {
            let state = bridge.debug.lock().unwrap();
            let map = &state.maps[&first.class_id];
            let health = map
                .members
                .iter()
                .find(|member| member.name == "health")
                .expect("Reflected health source map");
            let value = first
                .values
                .iter()
                .find(|value| value.member == health.trace_id)
                .expect("Live health value");
            assert_eq!(value.value_type, 4);
            assert!(
                [80 * 4096, 150 * 4096].contains(&(value.words[0] as i32)),
                "Unexpected live health {}",
                value.display()
            );
        }
        bridge.debug.lock().unwrap().request(Command::Step).unwrap();
        let next = wait(&|state| {
            state.paused
                && state.at_node
                && state
                    .snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.node_id != first.node_id)
        })
        .unwrap();
        assert_eq!(
            (next.owner, next.generation),
            (first.owner, first.generation)
        );
        assert_ne!(
            next.node_id, first.node_id,
            "Step must cross a graph node, not repeat the hook."
        );
        // Replace the persisted all-instance point with a runtime owner filter.
        {
            let mut state = bridge.debug.lock().unwrap();
            state.request(Command::Clear).unwrap();
            state
                .request(Command::Add(Breakpoint {
                    class_id: first.class_id,
                    node_id: first.node_id,
                    instance: Some((first.owner, first.generation)),
                }))
                .unwrap();
            state.request(Command::Resume).unwrap();
        }
        wait(&|state| !state.paused);
        // Pause-next is a real instruction hook, independent of rendered frames.
        bridge
            .debug
            .lock()
            .unwrap()
            .request(Command::Pause)
            .unwrap();
        let paused = wait(&|state| state.paused && state.at_node).unwrap();
        let (traces, dropped) = {
            let state = bridge.debug.lock().unwrap();
            (state.traces.len(), state.dropped)
        };
        println!(
            "PASS real Blueprint breakpoint + health snapshot at {}:{}, node step {} -> {}, pause-next at {}:{}; traces={traces}, dropped={dropped}",
            first.owner,
            first.generation,
            first.node_id,
            next.node_id,
            paused.owner,
            paused.generation
        );
        drop(job);
    }
}
