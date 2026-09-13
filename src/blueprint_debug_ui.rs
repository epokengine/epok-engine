//! Source-mapped node breakpoints operate on the owned emulator, never host offsets.
use crate::{blueprint_debug as debug, editor::Editor};
use std::path::PathBuf;

#[derive(Default)]
pub struct State {
    root: Option<PathBuf>,
    points: Vec<debug::Breakpoint>,
    error: Option<String>,
}
fn trace_id(id: &str) -> u32 {
    crate::blueprint_refs::compact_id(id) as u32
}
pub fn draw(ui: &imgui::Ui, editor: &mut Editor) {
    if !editor.blueprint_debug_enabled {
        return;
    }
    let mut state = std::mem::take(&mut editor.blueprint_debug_ui);
    if state.root.as_ref() != Some(&editor.root) {
        state = State {
            root: Some(editor.root.clone()),
            ..Default::default()
        };
        let path = editor.root.join("UserSettings/Breakpoints.epokprefs");
        if path.is_file() {
            match std::fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|bytes| {
                    crate::document::from_slice::<Vec<debug::Breakpoint>>(&bytes)
                        .map_err(|e| e.to_string())
                }) {
                Ok(points) if points.len() <= 32 => state.points = points,
                Ok(_) => state.error = Some("Breakpoint file exceeds 32 entries.".into()),
                Err(error) => state.error = Some(error),
            }
        }
    }
    let bridge = editor
        .job
        .as_ref()
        .and_then(|job| job.bridge.as_ref())
        .map(|bridge| bridge.debug.clone());
    let mut command = None;
    let mut jump = None;
    let mut save = false;
    ui.window("Blueprint Debugger").size([440.,440.],imgui::Condition::FirstUseEver).build(|| {
        ui.text_wrapped("Instrumented MIPS execution. Node breakpoints preserve the native stack. Release builds omit instrumentation.");
        let selected=editor.blueprint_editor.asset.as_ref().and_then(|asset|editor.blueprint_editor.selected_node().map(|(_,node)|debug::Breakpoint{class_id:trace_id(&asset.id),node_id:trace_id(&node),instance:None}));
        {
            let _disabled=ui.begin_disabled(selected.is_none() || state.points.len()>=32);
            if ui.button("Break on selected node") {let point=selected.unwrap();if !state.points.contains(&point){state.points.push(point);save=true;}command=Some(debug::Command::Add(point));}
        }
        ui.same_line();if ui.button("Clear breakpoints"){state.points.clear();save=true;command=Some(debug::Command::Clear);}
        let mut remove=None;
        for (index,point) in state.points.iter().enumerate() {
            let label=editor.blueprint_editor.asset.as_ref().filter(|asset|trace_id(&asset.id)==point.class_id).and_then(|asset|asset.functions.iter().find_map(|graph|graph.nodes.iter().find(|node|trace_id(&node.id)==point.node_id).map(|_|format!("{} / {}",asset.name,graph.name))))
                .unwrap_or_else(||format!("Class {:08x} / node {:08x}",point.class_id,point.node_id));
            ui.bullet_text(label);ui.same_line();if ui.small_button(format!("Remove##bp{index}")){remove=Some(index);}
        }
        if let Some(index)=remove {command=Some(debug::Command::Remove(state.points.remove(index)));save=true;}
        let Some(bridge)=&bridge else {ui.text_wrapped("Select a graph node to set a breakpoint, then Play. Saved all-instance breakpoints also catch Start events.");return;};
        let mut live=bridge.lock().unwrap();
        if live.maps.is_empty() && live.available && let Err(error)=live.load_maps(&editor.root.join(".epok/build-blueprint-debug")){state.error=Some(error);}
        if live.at_node {editor.paused=true;}
        if ui.button("Pause at next node"){command=Some(debug::Command::Pause);}
        ui.same_line();{let _disabled=ui.begin_disabled(!live.paused);if ui.button("Step node"){command=Some(debug::Command::Step);}}
        ui.same_line();if ui.button("Continue"){command=Some(debug::Command::Resume);}
        ui.text(if !live.available{"Waiting for instrumented emulator..."}else if live.at_node{"Paused at Blueprint node"}else if live.paused{"Paused"}else{"Running"});
        if let Some(snapshot)=&live.snapshot {
            ui.text(format!("Instance slot {} / generation {}",snapshot.owner,snapshot.generation));
            if let Some(map)=live.maps.get(&snapshot.class_id) {
                if let Some((graph,node))=map.node(snapshot.node_id) {
                    ui.text(format!("{} / {}",map.source.display(),graph.name));
                    editor.blueprint_editor.debug_node=if editor.blueprint_editor.asset.as_ref().is_some_and(|asset|asset.id==map.class && asset.semantic_hash()==map.semantic_hash) {live.at_node.then(||node.id.clone())}else{None};
                    if ui.button("Show current node"){jump=Some((map.clone(),graph.id.clone(),node.id.clone()));}
                }
                for value in &snapshot.values {
                    if let Some(member)=map.member(value.member) {
                        ui.text(format!("{} = {}",member.name,value.display()));
                        if ui.is_item_hovered(){ui.tooltip_text(format!("{}\n{}",member.id,member.value_type.label()));}
                    }
                }
            }
            if ui.button("Break here for this instance only") {command=Some(debug::Command::Add(debug::Breakpoint{class_id:snapshot.class_id,node_id:snapshot.node_id,instance:Some((snapshot.owner,snapshot.generation))}));}
        }
        ui.separator();ui.text(format!("Recent nodes: {} / dropped {}",live.traces.len(),live.dropped));
        if ui.small_button("Clear trace history"){live.clear_traces();}
        ui.child_window("blueprint-trace-history").size([0.,120.]).build(|| {
            for (i,trace) in live.traces.iter().rev().take(64).enumerate() {
                if let Some(map)=live.maps.get(&trace.class_id) && let Some((graph,node))=map.node(trace.node_id)
                    && ui.selectable(format!("{} [{}:{}] {}##trace{i}",graph.name,trace.owner,trace.generation,node.id)) {jump=Some((map.clone(),graph.id.clone(),node.id.clone()));}
            }
        });
        if let Some(error)=&live.error {ui.text_colored([1.,0.4,0.3,1.],error);}
    });
    if save
        && let Err(error) = crate::settings::save_document(
            &editor.root.join("UserSettings/Breakpoints.epokprefs"),
            &state.points,
        )
    {
        state.error = Some(error);
    }
    if let Some(command) = command
        && let Some(bridge) = bridge
    {
        if matches!(command, debug::Command::Resume | debug::Command::Step) {
            editor.paused = false;
            editor.blueprint_editor.debug_node = None;
        }
        if let Err(error) = bridge.lock().unwrap().request(command) {
            state.error = Some(error);
        }
    }
    if let Some((map, graph, node)) = jump {
        let path = crate::assets::inside(
            &editor.root,
            &map.source.to_string_lossy().replace('\\', "/"),
        );
        let result=path.and_then(|path| {
            let current=crate::blueprint_asset::load(&path)?;
            if current.semantic_hash()!=map.semantic_hash {return Err("Blueprint changed since this debug build; rebuild before navigating live nodes.".into());}
            if editor.blueprint_editor.path.as_ref()==Some(&path) && editor.blueprint_editor.asset.as_ref().is_some_and(|asset|asset.semantic_hash()!=map.semantic_hash) {return Err("The open Blueprint draft differs from the debug build; save and rebuild before live navigation.".into());}
            if editor.blueprint_editor.path.as_ref()!=Some(&path){editor.blueprint_editor.open(&path)?;}
            editor.blueprint_editor.focus_node(&graph,&node)
        });
        if let Err(error) = result {
            state.error = Some(error);
        }
    }
    if let Some(error) = state.error.take() {
        editor.log(format!("Blueprint debugger: {error}"));
    }
    editor.blueprint_debug_ui = state;
}
