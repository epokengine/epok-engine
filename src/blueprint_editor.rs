//! Blueprint authoring uses the editor's existing ImGui context and draw lists.
//! No second native ImGui ABI is linked. The document, transactions and typed
//! sockets are independent of canvas rendering and exercised by unit tests.
use crate::{blueprint::Registry, blueprint_asset as asset, reflection_schema as schema};
use asset::{BlueprintAsset, Graph, Input, Node, NodeKind};
use imgui::{Condition, MouseButton, Ui};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
#[path = "blueprint_action_menu.rs"]
mod action_menu;
pub(crate) use action_menu::Fonts as ActionMenuFonts;
#[path = "blueprint_inline_values.rs"]
mod inline_values;
#[path = "blueprint_split_pins.rs"]
mod split_pins;
#[path = "blueprint_template_editor.rs"]
mod template_editor;

#[derive(Clone, Debug, PartialEq, Eq)]
enum SocketType {
    Exec,
    Value(schema::Type),
}
#[derive(Clone, Debug)]
struct Socket {
    node: String,
    pin: String,
    output: bool,
    ty: SocketType,
    label: String,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Clip {
    nodes: Vec<Node>,
    positions: BTreeMap<String, [f32; 2]>,
    comments: BTreeMap<String, String>,
    #[serde(default)]
    split_pins: BTreeSet<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Details {
    Class,
    Defaults,
    Variable,
    Function,
    Node,
    Template,
    CreateFunction,
    CreateVariable,
    BuildOptions,
}
pub struct BlueprintEditor {
    pub open: bool,
    pub maximized: bool,
    pub path: Option<PathBuf>,
    pub asset: Option<BlueprintAsset>,
    /// `Some` when the open document is a map's own Blueprint. `path` is then the
    /// `.epokmap`, the document lives in `Scene::scene_script` and never in a file
    /// of its own, and the scope carries the identities it may reference.
    embedded: Option<asset::MapScope>,
    /// Set by Save while an embedded document is open: the map owns the bytes, so
    /// saving is delegated to the scene save instead of writing a `.epokbp`.
    pub save_to_map: bool,
    saved: Vec<u8>,
    disk: Option<Vec<u8>>,
    undo: Vec<BlueprintAsset>,
    redo: Vec<BlueprintAsset>,
    graph: usize,
    prepare_events: bool,
    frame_events: bool,
    selected: BTreeSet<String>,
    clipboard: Option<Clip>,
    pan: [f32; 2],
    zoom: f32,
    drag: Option<String>,
    drag_before: Option<BlueprintAsset>,
    wire: Option<Socket>,
    wire_dragging: bool,
    inline_edit: Option<inline_values::Edit>,
    pin_menu: Option<Socket>,
    node_menu: Option<String>,
    action_menu: action_menu::State,
    catalog_position: Option<[f32; 2]>,
    member_search: String,
    details: Details,
    detail_member: String,
    variable_name: String,
    function_name: String,
    parameter_name: String,
    type_index: usize,
    comment: String,
    pub error: Option<String>,
    pub diagnostics: Vec<String>,
    diagnostic_nodes: BTreeMap<usize, (String, String)>,
    compile_stale: bool,
    pub generated: String,
    pub compile_requested: bool,
    compile_valid: bool,
    show_generated: bool,
    close_requested: bool,
    template_editor: template_editor::State,
    timeline_key: Option<usize>,
    pub focused: bool,
    pub place_requested: bool,
    pub capture_requested: bool,
    pub derived_requested: bool,
    pub debug_node: Option<String>,
    playback_timelines: Vec<(PathBuf, crate::timeline::TimelineAsset)>,
    playback_error: Option<String>,
    playback_effects: Vec<(PathBuf, crate::particle_effect::ParticleEffect)>,
}

impl Default for BlueprintEditor {
    fn default() -> Self {
        Self {
            open: false,
            maximized: false,
            path: None,
            asset: None,
            embedded: None,
            save_to_map: false,
            saved: vec![],
            disk: None,
            undo: vec![],
            redo: vec![],
            graph: 0,
            prepare_events: false,
            frame_events: false,
            selected: BTreeSet::new(),
            clipboard: None,
            pan: [50., 50.],
            zoom: 1.,
            drag: None,
            drag_before: None,
            wire: None,
            wire_dragging: false,
            inline_edit: None,
            pin_menu: None,
            node_menu: None,
            action_menu: Default::default(),
            catalog_position: None,
            member_search: String::new(),
            details: Details::Node,
            detail_member: String::new(),
            variable_name: "new_variable".into(),
            function_name: "new_function".into(),
            parameter_name: "argument".into(),
            type_index: 0,
            comment: String::new(),
            error: None,
            diagnostics: vec![],
            diagnostic_nodes: BTreeMap::new(),
            compile_stale: true,
            generated: String::new(),
            compile_requested: false,
            compile_valid: false,
            show_generated: false,
            close_requested: false,
            template_editor: Default::default(),
            timeline_key: None,
            focused: false,
            place_requested: false,
            capture_requested: false,
            derived_requested: false,
            debug_node: None,
            playback_timelines: vec![],
            playback_error: None,
            playback_effects: vec![],
        }
    }
}

fn bytes(doc: &BlueprintAsset) -> Vec<u8> {
    crate::document::to_vec(doc).expect("Blueprint serialization")
}
fn hit_socket(
    sockets: &[(Socket, [f32; 2])],
    mouse: [f32; 2],
    zoom: f32,
) -> Option<&(Socket, [f32; 2])> {
    let radius = (8. * zoom + 5.).max(11.);
    sockets
        .iter()
        .rev()
        .filter_map(|s| {
            let distance = (s.1[0] - mouse[0]).powi(2) + (s.1[1] - mouse[1]).powi(2);
            (distance <= radius * radius).then_some((s, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(s, _)| s)
}
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn button(ui: &Ui, label: &str) -> bool {
    let clicked = ui.button(label);
    record_control(ui, label);
    clicked
}
fn choose(ui: &Ui, label: &str) -> bool {
    let clicked = ui.selectable(label);
    record_control(ui, label);
    clicked
}
fn record_control(ui: &Ui, label: &str) {
    #[cfg(test)]
    CONTROLS.with(|controls| {
        let a = ui.item_rect_min();
        let b = ui.item_rect_max();
        controls
            .borrow_mut()
            .insert(label.into(), [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]);
    });
    #[cfg(not(test))]
    let _ = (ui, label);
}
#[cfg(test)]
thread_local! {
    static CONTROLS: std::cell::RefCell<BTreeMap<String,[f32;2]>> = const { std::cell::RefCell::new(BTreeMap::new()) };
    static SOCKETS: std::cell::RefCell<BTreeMap<String,[f32;2]>> = const { std::cell::RefCell::new(BTreeMap::new()) };
}
fn identifier(name: &str) -> bool {
    !name.is_empty()
        && name.as_bytes()[0].is_ascii_alphabetic()
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
fn default_value(ty: &schema::Type) -> Value {
    crate::script_values::default_value(ty)
}
fn primitive(index: usize) -> schema::Type {
    match index {
        0 => schema::Type::Fixed,
        1 => schema::Type::Bool,
        2 => schema::Type::Int32,
        3 => schema::Type::UInt32,
        4 => schema::Type::Vector { length: 2 },
        5 => schema::Type::Vector { length: 3 },
        6 => schema::Type::ObjectRef { class: None },
        7 => schema::Type::AssetRef {
            kind: "Texture".into(),
        },
        8 => schema::Type::AssetRef {
            kind: "AudioClip".into(),
        },
        _ => schema::Type::ClassRef {
            base: String::new(),
        },
    }
}
fn authoring_types(registry: &Registry) -> Vec<schema::Type> {
    let mut types: Vec<_> = (0..10).map(primitive).collect();
    types.extend([schema::Type::SequenceHandle, schema::Type::EffectHandle]);
    let mut enums = BTreeMap::new();
    for class in registry.classes.values() {
        for ty in
            class
                .properties
                .iter()
                .map(|p| &p.value_type)
                .chain(class.functions.iter().flat_map(|f| {
                    std::iter::once(&f.returns).chain(f.parameters.iter().map(|p| &p.value_type))
                }))
        {
            if let schema::Type::Enum { cpp_name, .. } = ty {
                enums.insert(cpp_name.clone(), ty.clone());
            }
        }
    }
    types.extend(enums.into_values());
    types
}
fn authoring_type(index: usize, registry: &Registry) -> schema::Type {
    authoring_types(registry)
        .get(index)
        .cloned()
        .unwrap_or(schema::Type::Fixed)
}
fn type_picker(ui: &Ui, label: &str, index: &mut usize, registry: &Registry) {
    let types = authoring_types(registry);
    let labels: Vec<_> = types
        .iter()
        .map(|ty| match ty {
            schema::Type::ClassRef { .. } => "Class reference".into(),
            schema::Type::ObjectRef { class: None } => "Actor reference".into(),
            _ => ty.label(),
        })
        .collect();
    *index = (*index).min(labels.len().saturating_sub(1));
    ui.combo_simple_string(label, index, &labels);
}
fn edit_value(
    ui: &Ui,
    label: &str,
    value: &mut Value,
    ty: &schema::Type,
    registry: &Registry,
) -> bool {
    if let schema::Type::ClassRef { base } = ty {
        let preview = value
            .as_str()
            .and_then(|id| registry.classes.get(id))
            .map(|c| c.cpp_name.as_str())
            .unwrap_or(if value.is_null() {
                "None"
            } else {
                "Missing class (preserved)"
            });
        if let Some(_combo) = ui.begin_combo(label, preview) {
            if ui.selectable("None") {
                *value = Value::Null;
                return true;
            }
            for class in registry.classes.values() {
                if !class.abstract_class
                    && registry
                        .ancestry(&class.cpp_name)
                        .iter()
                        .any(|c| c.id == *base)
                    && ui.selectable(&class.cpp_name)
                {
                    *value = json!(class.id);
                    return true;
                }
            }
        }
        false
    } else {
        crate::script_values::inspector(ui, label, value, ty)
    }
}

impl BlueprintEditor {
    pub(crate) fn set_action_menu_fonts(&mut self, fonts: ActionMenuFonts) {
        self.action_menu.fonts = Some(fonts);
    }
    pub fn selected_node(&self) -> Option<(String, String)> {
        Some((
            self.current()?.id.clone(),
            self.selected.iter().next()?.clone(),
        ))
    }
    pub fn focus_node(&mut self, graph_id: &str, node_id: &str) -> Result<(), String> {
        let doc = self.asset.as_ref().ok_or("No Blueprint open")?;
        let index = doc
            .functions
            .iter()
            .position(|g| g.id == graph_id)
            .ok_or("Debug graph is missing from this source")?;
        let graph = &doc.functions[index];
        if !graph.nodes.iter().any(|n| n.id == node_id) {
            return Err("Debug node is missing from this source".into());
        }
        let p = node_position(doc, graph, node_id);
        self.graph = index;
        self.details = Details::Node;
        self.selected = BTreeSet::from([node_id.into()]);
        self.pan = [60. - p[0] * self.zoom, 60. - p[1] * self.zoom];
        self.debug_node = Some(node_id.into());
        self.open = true;
        Ok(())
    }
    pub fn refresh_playback_sources(&mut self, root: &Path) {
        self.accept_playback_sources(crate::timeline::inspect_playback(root));
    }
    pub fn accept_playback_sources(
        &mut self,
        sources: Result<crate::timeline::PlaybackSources, String>,
    ) {
        let previous = self.selected_playback_sources();
        match sources {
            Ok(catalog) => {
                self.playback_timelines = catalog.timelines;
                self.playback_timelines.extend(
                    catalog
                        .effects
                        .iter()
                        .map(|(path, effect)| (path.clone(), effect.timeline.clone())),
                );
                self.playback_effects = catalog.effects;
                self.playback_error =
                    (!catalog.errors.is_empty()).then(|| catalog.errors.join("\n"));
            }
            Err(error) => {
                self.playback_timelines.clear();
                self.playback_effects.clear();
                self.playback_error = Some(error);
            }
        }
        if previous != self.selected_playback_sources() {
            self.compile_valid = false;
            self.compile_stale = true;
            self.compile_requested = true;
        }
    }
    // Schedule validation of the open document, including unsaved nodes. This
    // reads the same typed selections as the canvas; it never publishes output.
    fn selected_playback_sources(&self) -> BTreeMap<String, Option<String>> {
        self.asset
            .iter()
            .flat_map(|doc| &doc.functions)
            .flat_map(|graph| &graph.nodes)
            .filter_map(|node| {
                let (id, effect) = match &node.kind {
                    NodeKind::Builtin {
                        operation: asset::Builtin::PlayTimelineAsset { asset },
                    } => (asset, false),
                    NodeKind::Builtin {
                        operation: asset::Builtin::SpawnParticleEffect { asset },
                    } => (asset, true),
                    NodeKind::WaitPlayback {
                        condition:
                            asset::PlaybackCondition::Marker { timeline, .. }
                            | asset::PlaybackCondition::SubscribeMarker { timeline, .. },
                    } => (timeline, false),
                    _ => return None,
                };
                let signature = if effect {
                    self.playback_effects
                        .iter()
                        .find(|(_, source)| source.id.to_string() == *id)
                        .map(|(_, source)| source.semantic_hash())
                } else {
                    self.playback_timelines
                        .iter()
                        .find(|(_, source)| source.id.to_string() == *id)
                        .map(|(_, source)| source.semantic_hash())
                };
                Some((node.id.clone(), signature))
            })
            .collect()
    }
    pub fn compile(&mut self, root: &Path, registry: &Registry) -> bool {
        self.refresh_playback_sources(root);
        self.compile_requested = false;
        self.compile_valid = false;
        self.generated.clear();
        let mut diagnostic_nodes = BTreeMap::new();
        let result = (|| {
            let mut files = asset::load_all(root).map_err(|e| vec![e])?;
            if let (Some(path), Some(doc)) = (&self.path, &self.asset) {
                files.retain(|f| {
                    let existing =
                        std::fs::canonicalize(&f.path).unwrap_or_else(|_| f.path.clone());
                    let open = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
                    existing != open
                });
                files.push(asset::AssetFile {
                    path: path.clone(),
                    asset: doc.clone(),
                    source: match &self.embedded {
                        Some(scope) => asset::AssetSource::EmbeddedScene(scope.clone()),
                        None => asset::AssetSource::File,
                    },
                });
            }
            let mut native = registry.clone();
            native.classes.retain(|_, c| c.provider.id != "blueprint");
            crate::blueprint_compile::compile(root, &native, &files).map_err(|errors| {
                errors
                    .into_iter()
                    .enumerate()
                    .map(|(index, e)| {
                        let Some(file) = files.iter().find(|file| file.path == e.asset) else {
                            return e.to_string();
                        };
                        let mut label = file.asset.name.clone();
                        if let Some(graph) = file
                            .asset
                            .functions
                            .iter()
                            .find(|g| Some(&g.id) == e.graph.as_ref())
                        {
                            label.push_str(&format!(" / {}", display_port(&graph.name)));
                            if let Some(node) =
                                graph.nodes.iter().find(|n| Some(&n.id) == e.node.as_ref())
                            {
                                label.push_str(&format!(
                                    " / {}",
                                    node_label(node, &file.asset, registry)
                                ));
                                if self.path.as_ref() == Some(&file.path) {
                                    diagnostic_nodes
                                        .insert(index, (graph.id.clone(), node.id.clone()));
                                }
                            }
                        }
                        format!("{label}: {}", e.message)
                    })
                    .collect::<Vec<_>>()
            })
        })();
        match result {
            Ok(compilation) => {
                self.compile_valid = true;
                for (path, content) in &compilation.artifacts.files {
                    if path.extension().is_some_and(|e| e == "hpp" || e == "cpp") {
                        self.generated.push_str(&format!("// {}\n", path.display()));
                        self.generated.push_str(&String::from_utf8_lossy(content));
                        self.generated.push('\n');
                    }
                }
                self.diagnostics = vec![format!(
                    "Blueprint validation succeeded: {} visual classes ({} total registry classes). Generated C++ is a preview; Build verifies it with the target compiler.",
                    compilation.scripts.len(),
                    compilation.registry.classes.len()
                )];
                self.error = None;
            }
            Err(errors) => self.diagnostics = errors,
        }
        self.diagnostic_nodes = diagnostic_nodes;
        self.compile_stale = false;
        self.compile_valid
    }
    fn focus_diagnostic(&mut self, index: usize) {
        let Some((graph_id, node_id)) = self.diagnostic_nodes.get(&index) else {
            return;
        };
        let Some(doc) = &self.asset else { return };
        let Some((index, graph)) = doc
            .functions
            .iter()
            .enumerate()
            .find(|(_, g)| &g.id == graph_id)
        else {
            return;
        };
        if !graph.nodes.iter().any(|node| &node.id == node_id) {
            return;
        }
        let p = node_position(doc, graph, node_id);
        self.graph = index;
        self.details = Details::Node;
        self.selected = BTreeSet::from([node_id.clone()]);
        self.pan = [50. - p[0] * self.zoom, 50. - p[1] * self.zoom];
    }
    pub fn discard(&mut self) {
        *self = Self::default();
    }
    pub fn replace_template(
        &mut self,
        template: crate::blueprint_templates::Template,
    ) -> Result<(), String> {
        let before = self.asset.clone().ok_or("No Blueprint is open")?;
        self.asset.as_mut().unwrap().template = template;
        self.checkpoint(before);
        Ok(())
    }
    pub fn dirty(&self) -> bool {
        self.asset.as_ref().is_some_and(|a| bytes(a) != self.saved)
    }
    pub fn is_embedded(&self) -> bool {
        self.embedded.is_some()
    }
    /// The `.epokmap` whose own Blueprint is open, when one is.
    pub fn embedded_map(&self) -> Option<&Path> {
        self.embedded.as_ref()?;
        self.path.as_deref()
    }
    /// Opens a map's own Blueprint. The document is the map's: there is no file to
    /// read or write, no external-edit baseline, and every edit is handed back to
    /// the scene by [`Self::take_embedded_edit`], so dirty state, Undo and Save all
    /// belong to the map.
    pub fn open_embedded(&mut self, file: &asset::AssetFile) -> Result<(), String> {
        let scope = file
            .scope()
            .ok_or("This Blueprint is a file asset, not a map's own Blueprint.")?;
        if self.dirty() {
            return Err("Save or revert the open Blueprint before opening another asset.".into());
        }
        if self.embedded_map() == Some(file.path.as_path()) {
            self.open = true;
            return Ok(());
        }
        *self = Self {
            open: true,
            path: Some(file.path.clone()),
            embedded: Some(scope.clone()),
            saved: bytes(&file.asset),
            disk: None,
            asset: Some(file.asset.clone()),
            compile_requested: true,
            prepare_events: true,
            frame_events: true,
            clipboard: self.clipboard.take(),
            ..Self::default()
        };
        Ok(())
    }
    /// The edited embedded document when it differs from what the scene holds.
    /// Taking it marks the editor in sync, so the map's `dirty` is the only one.
    pub fn take_embedded_edit(&mut self) -> Option<BlueprintAsset> {
        let asset = self.asset.as_ref()?;
        if self.embedded.is_none() || bytes(asset) == self.saved {
            return None;
        }
        let asset = asset.clone();
        self.saved = bytes(&asset);
        Some(asset)
    }
    /// Replaces the open embedded document with the scene's, after an Undo or a
    /// scene reload. Graph edit state that names vanished nodes is dropped.
    pub fn adopt_embedded(&mut self, file: &asset::AssetFile) {
        let Some(scope) = file.scope() else { return };
        if self.embedded_map() != Some(file.path.as_path()) {
            return;
        }
        if self
            .asset
            .as_ref()
            .is_some_and(|open| bytes(open) == bytes(&file.asset))
        {
            return;
        }
        self.embedded = Some(scope.clone());
        self.asset = Some(file.asset.clone());
        self.saved = bytes(&file.asset);
        self.selected.clear();
        self.wire = None;
        self.inline_edit = None;
        self.drag = None;
        self.compile_requested = true;
    }
    pub fn open(&mut self, path: &Path) -> Result<(), String> {
        if self.path.as_deref().is_some_and(|old| {
            std::fs::canonicalize(old)
                .ok()
                .zip(std::fs::canonicalize(path).ok())
                .is_some_and(|(a, b)| a == b)
        }) {
            self.open = true;
            return Ok(());
        }
        if self.dirty() {
            return Err("Save or revert the open Blueprint before opening another asset.".into());
        }
        let data = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let doc = asset::load(path)?;
        *self = Self {
            open: true,
            path: Some(path.to_owned()),
            saved: bytes(&doc),
            disk: Some(data),
            asset: Some(doc),
            compile_requested: true,
            prepare_events: true,
            frame_events: true,
            clipboard: self.clipboard.take(),
            ..Self::default()
        };
        Ok(())
    }
    pub fn save(&mut self) -> Result<(), String> {
        use std::io::Write;
        // A map's own Blueprint has no file of its own. Saving it means saving the
        // map, which the editor does once for the whole document.
        if self.is_embedded() {
            self.save_to_map = true;
            self.error = None;
            return Ok(());
        }
        let path = self.path.as_ref().ok_or("No Blueprint file is open")?;
        let current = match std::fs::read(path) {
            Ok(data) => Some(data),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.to_string()),
        };
        if current != self.disk {
            return Err("Blueprint changed on disk. Save was blocked to preserve external edits. Revert to reload, or copy your work into another asset.".into());
        }
        let data = bytes(self.asset.as_ref().ok_or("No Blueprint document")?);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let temporary = path.with_extension(format!("{}.tmp", id()));
        let result = (|| {
            let mut output = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(|e| e.to_string())?;
            output
                .write_all(&data)
                .and_then(|()| output.sync_all())
                .map_err(|e| e.to_string())?;
            // Recheck after writing the temporary, before publishing it.
            let fresh = match std::fs::read(path) {
                Ok(data) => Some(data),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e.to_string()),
            };
            if fresh != self.disk {
                return Err("Blueprint changed during save; original preserved.".into());
            }
            if path.exists() {
                // A recoverable old copy makes the Windows replacement transaction explicit.
                let backup = path.with_extension(format!("{}.bak", id()));
                std::fs::rename(path, &backup).map_err(|e| e.to_string())?;
                if let Err(error) = std::fs::rename(&temporary, path) {
                    if let Err(restore) = std::fs::rename(&backup, path) {
                        return Err(format!(
                            "Save failed: {error}; restore failed: {restore}. Original: {}",
                            backup.display()
                        ));
                    }
                    return Err(error.to_string());
                }
                let _ = std::fs::remove_file(backup);
            } else {
                std::fs::rename(&temporary, path).map_err(|e| e.to_string())?;
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result?;
        self.disk = Some(data.clone());
        self.saved = data;
        self.error = None;
        Ok(())
    }
    fn checkpoint(&mut self, before: BlueprintAsset) {
        if self
            .asset
            .as_ref()
            .is_some_and(|a| bytes(a) != bytes(&before))
        {
            self.undo.push(before);
            if self.undo.len() > 128 {
                self.undo.remove(0);
            }
            self.redo.clear();
            self.compile_stale = true;
        }
    }
    pub fn undo(&mut self) {
        if let Some(old) = self.undo.pop() {
            self.compile_stale = true;
            if let Some(now) = self.asset.replace(old) {
                self.redo.push(now);
            }
            self.selected.clear();
            self.wire = None;
        }
    }
    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.compile_stale = true;
            if let Some(now) = self.asset.replace(next) {
                self.undo.push(now);
            }
            self.selected.clear();
            self.wire = None;
        }
    }
    fn revert(&mut self) -> Result<(), String> {
        if self.is_embedded() {
            return Err(
                "This Blueprint belongs to the map. Use Edit > Undo Scene Edit to step back, or reopen the map to discard every change."
                    .into(),
            );
        }
        let path = self.path.clone().ok_or("No Blueprint file")?;
        let data = std::fs::read(&path).map_err(|e| e.to_string())?;
        let doc = asset::load(&path)?;
        let old = self.asset.replace(doc);
        if let Some(old) = old {
            self.checkpoint(old);
        }
        self.saved = bytes(self.asset.as_ref().unwrap());
        self.disk = Some(data);
        self.selected.clear();
        self.wire = None;
        Ok(())
    }
    fn current(&self) -> Option<&Graph> {
        self.asset.as_ref()?.functions.get(self.graph)
    }
    fn select_node_graph(&mut self, node: &str) {
        if let Some(index) = self.asset.as_ref().and_then(|doc| {
            doc.functions
                .iter()
                .position(|graph| graph.nodes.iter().any(|n| n.id == node))
        }) && index != self.graph
        {
            self.graph = index;
            self.selected.clear();
        }
    }
    fn add_node(&mut self, kind: NodeKind) {
        let Some(doc) = &mut self.asset else {
            return;
        };
        let Some(graph) = doc.functions.get_mut(self.graph) else {
            return;
        };
        let key = id();
        let point = [
            50. - self.pan[0] / self.zoom + (graph.nodes.len() % 4) as f32 * 24.,
            80. - self.pan[1] / self.zoom + (graph.nodes.len() % 5) as f32 * 32.,
        ];
        let inputs = if matches!(kind, NodeKind::CallParent) {
            graph
                .parameters
                .iter()
                .map(|p| {
                    (
                        p.name.clone(),
                        Input::Parameter {
                            name: p.name.clone(),
                        },
                    )
                })
                .collect()
        } else {
            BTreeMap::new()
        };
        graph.nodes.push(Node {
            id: key.clone(),
            kind,
            inputs,
            outputs: BTreeMap::new(),
        });
        doc.layout.positions.insert(key.clone(), point);
        self.selected = BTreeSet::from([key]);
        self.details = Details::Node;
    }
    fn add_parent_call(&mut self, entry: &str, registry: &Registry) {
        self.select_node_graph(entry);
        let Some((doc, graph)) = self.asset.as_ref().zip(self.current()) else {
            return;
        };
        if graph.entry != entry || !can_call_parent(doc, graph, registry) {
            return;
        }
        let position = node_position(doc, graph, entry);
        self.add_node(NodeKind::CallParent);
        if let Some(key) = self.selected.iter().next().cloned() {
            self.asset
                .as_mut()
                .unwrap()
                .layout
                .positions
                .insert(key, [position[0] + 320., position[1] + 100.]);
        }
    }
    fn add_graph(&mut self, name: String, function: Option<&schema::Function>) {
        let Some(doc) = &mut self.asset else {
            return;
        };
        if doc.functions.iter().any(|g| g.name == name) {
            self.error = Some("A graph with this name already exists.".into());
            return;
        }
        let entry = id();
        doc.functions.push(Graph {
            timeline: None,
            id: id(),
            name,
            override_id: function.map(|f| f.id.clone()),
            parameters: function.map_or_else(Vec::new, |f| f.parameters.clone()),
            returns: function.map_or(schema::Type::Void, |f| f.returns.clone()),
            entry: entry.clone(),
            nodes: vec![Node {
                id: entry.clone(),
                kind: NodeKind::Entry,
                inputs: BTreeMap::new(),
                outputs: BTreeMap::new(),
            }],
        });
        doc.layout.positions.insert(entry, [0., 0.]);
        self.graph = doc.functions.len() - 1;
        self.selected.clear();
        self.pan = [50., 50.];
    }
    fn delete_selected(&mut self) {
        let Some(doc) = &mut self.asset else {
            return;
        };
        let Some(graph) = doc.functions.get_mut(self.graph) else {
            return;
        };
        self.selected.remove(&graph.entry);
        graph.nodes.retain(|n| !self.selected.contains(&n.id));
        for node in &mut graph.nodes {
            node.inputs.retain(|_, input| !matches!(input, Input::Link { node, .. } if self.selected.contains(node)));
            for targets in node.outputs.values_mut() {
                targets.retain(|target| !self.selected.contains(target));
            }
        }
        for key in &self.selected {
            doc.layout.positions.remove(key);
            doc.layout.comments.remove(key);
        }
        doc.layout.split_pins.retain(|key| {
            !self
                .selected
                .iter()
                .any(|id| key.starts_with(&format!("{id}:")))
        });
        self.selected.clear();
        self.wire = None;
    }
    fn copy_selected(&mut self) {
        let Some(graph) = self.current() else {
            return;
        };
        let doc = self.asset.as_ref().unwrap();
        self.clipboard = Some(Clip {
            split_pins: doc
                .layout
                .split_pins
                .iter()
                .filter(|key| {
                    self.selected
                        .iter()
                        .any(|id| key.starts_with(&format!("{id}:")))
                })
                .cloned()
                .collect(),
            nodes: graph
                .nodes
                .iter()
                .filter(|n| self.selected.contains(&n.id) && n.id != graph.entry)
                .cloned()
                .collect(),
            positions: doc
                .layout
                .positions
                .iter()
                .filter(|(key, _)| self.selected.contains(*key))
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
            comments: doc
                .layout
                .comments
                .iter()
                .filter(|(key, _)| self.selected.contains(*key))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        });
    }
    fn copy_to_clipboard(&mut self, ui: &Ui) {
        self.copy_selected();
        if let Some(clip) = &self.clipboard {
            ui.set_clipboard_text(format!(
                "Epok Blueprint Clipboard v1\n{}",
                serde_json::to_string(clip).expect("serializable nodes")
            ));
        }
    }
    fn paste_from_clipboard(&mut self, ui: &Ui) {
        if let Some(text) = ui.clipboard_text()
            && text.len() < 4 * 1024 * 1024
            && let Some(json) = text.strip_prefix("Epok Blueprint Clipboard v1\n")
        {
            match serde_json::from_str::<Clip>(json) {
                Ok(clip) if clip.nodes.len() <= 4096 => self.clipboard = Some(clip),
                _ => {
                    self.error = Some(
                        "Blueprint clipboard is invalid or too large. No nodes were changed."
                            .into(),
                    );
                    return;
                }
            }
        }
        self.paste();
    }
    fn paste(&mut self) {
        let Some(clip) = self.clipboard.clone() else {
            return;
        };
        let Some(doc) = &mut self.asset else {
            return;
        };
        let Some(graph) = doc.functions.get_mut(self.graph) else {
            return;
        };
        let remap: BTreeMap<_, _> = clip.nodes.iter().map(|n| (n.id.clone(), id())).collect();
        for key in &clip.split_pins {
            if let Some((old, suffix)) = key.split_once(':')
                && let Some(new) = remap.get(old)
            {
                doc.layout.split_pins.insert(format!("{new}:{suffix}"));
            }
        }
        self.selected.clear();
        for mut node in clip.nodes {
            let old = node.id.clone();
            node.id = remap[&old].clone();
            if let NodeKind::StopTimeline { node: target } = &mut node.kind
                && let Some(replacement) = remap.get(target)
            {
                *target = replacement.clone();
            }
            self.selected.insert(node.id.clone());
            node.inputs.retain(|_, input| match input {
                Input::Link { node, .. } => {
                    if let Some(new) = remap.get(node) {
                        *node = new.clone();
                        true
                    } else {
                        false
                    }
                }
                _ => true,
            });
            for targets in node.outputs.values_mut() {
                *targets = targets
                    .iter()
                    .filter_map(|v| remap.get(v).cloned())
                    .collect();
            }
            let p = clip.positions.get(&old).copied().unwrap_or([0., 0.]);
            doc.layout
                .positions
                .insert(node.id.clone(), [p[0] + 40., p[1] + 40.]);
            if let Some(text) = clip.comments.get(&old) {
                doc.layout.comments.insert(node.id.clone(), text.clone());
            }
            graph.nodes.push(node);
        }
    }

    /// Returns true when an authoring source was saved and catalogs must refresh.
    #[cfg(test)]
    pub fn draw(&mut self, ui: &Ui, registry: &Registry) -> bool {
        self.draw_with_options(ui, registry, |_| {})
    }
    /// Build settings belong to the project, but are presented inside this editor.
    pub fn draw_with_options(
        &mut self,
        ui: &Ui,
        registry: &Registry,
        build_options: impl FnOnce(&Ui),
    ) -> bool {
        if !self.open {
            self.focused = false;
            return false;
        }
        if self.prepare_events
            && let Some(before) = self.asset.clone()
            && registry.classes.contains_key(&before.parent)
        {
            self.prepare_events = false;
            if crate::blueprint_workflow::ensure_default_events(
                self.asset.as_mut().unwrap(),
                registry,
            ) {
                self.checkpoint(before);
            }
        }
        let mut visible = true;
        let mut saved = false;
        let name = self.asset.as_ref().map_or("Blueprint", |a| a.name.as_str());
        let title = format!(
            "Blueprint: {name}{}###BlueprintEditor",
            if self.dirty() { " *" } else { "" }
        );
        let window = ui
            .window(title)
            .opened(&mut visible)
            .size([1100., 720.], Condition::FirstUseEver);
        let window = if self.maximized {
            window.position([0., 22.], Condition::Always).size(
                [
                    ui.io().display_size[0],
                    (ui.io().display_size[1] - 22.).max(400.),
                ],
                Condition::Always,
            )
        } else {
            window
        };
        window.build(|| {
            self.focused =
                ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS);
            if button(ui, "Save") {
                match self.save() {
                    Ok(()) => {
                        saved = true;
                        self.compile_requested = true;
                    }
                    Err(e) => self.error = Some(e),
                }
            }
            ui.same_line();
            if button(ui, "Compile") {
                self.compile_requested = true;
            }
            ui.same_line();
            if button(ui, "Undo") {
                self.undo();
            }
            ui.same_line();
            if button(ui, "Redo") {
                self.redo();
            }
            ui.same_line();
            if button(ui, "Revert...") {
                ui.open_popup("Revert Blueprint");
            }
            ui.same_line();
            ui.checkbox("Generated C++", &mut self.show_generated);
            ui.same_line();
            if button(ui, "Class Settings") {
                self.details = Details::Class;
            }
            ui.same_line();
            if button(ui, "Class Defaults") {
                self.details = Details::Defaults;
                self.detail_member.clear();
            }
            if button(ui, "Place linked instance") {
                self.place_requested = true;
            }
            ui.same_line();
            if button(ui, "Capture selection as template...") {
                self.capture_requested = true;
            }
            ui.same_line();
            if button(ui, "Create derived Blueprint...") {
                self.derived_requested = true;
            }
            ui.same_line();
            if button(ui, "Build Options") {
                self.details = Details::BuildOptions;
            }
            if self.compile_requested {
                ui.text_disabled("Compiling Blueprint...");
            } else if self.compile_stale {
                ui.text_disabled("Compile to validate the current Blueprint.");
            } else if self.compile_valid {
                ui.text_colored([0.45, 0.85, 0.55, 1.], "Blueprint compilation succeeded.");
            } else {
                ui.text_colored(
                    [1., 0.4, 0.35, 1.],
                    format!(
                        "Blueprint compilation failed ({}). Build / Play is blocked.",
                        self.diagnostics.len()
                    ),
                );
                if let Some(index) = self.diagnostic_nodes.keys().next().copied() {
                    ui.same_line();
                    if button(ui, "Show error") {
                        self.focus_diagnostic(index);
                    }
                }
                if let Some(message) = self.diagnostics.first() {
                    let _color = ui.push_style_color(imgui::StyleColor::Text, [1., 0.6, 0.5, 1.]);
                    ui.text_wrapped(message);
                }
            }
            if let Some(_popup) = ui.begin_modal_popup("Revert Blueprint") {
                ui.text_wrapped(
                    "Reload the file on disk? Current edits remain recoverable with Undo.",
                );
                if button(ui, "Reload file") {
                    if let Err(e) = self.revert() {
                        self.error = Some(e);
                    }
                    ui.close_current_popup();
                }
                ui.same_line();
                if button(ui, "Cancel##revert") {
                    ui.close_current_popup();
                }
            }
            if ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS)
                && !ui.io().want_text_input
            {
                if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::S) {
                    match self.save() {
                        Ok(()) => {
                            saved = true;
                            self.compile_requested = true;
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::Z) {
                    if ui.io().key_shift {
                        self.redo();
                    } else {
                        self.undo();
                    }
                }
                if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::Y) {
                    self.redo();
                }
            }
            if let Some(error) = &self.error {
                ui.text_colored([1., 0.4, 0.35, 1.], error);
            }
            if let Some(doc) = &self.asset {
                let parent = registry.classes.get(&doc.parent);
                let chain = parent
                    .map(|p| {
                        registry
                            .ancestry(&p.cpp_name)
                            .iter()
                            .map(|c| c.cpp_name.clone())
                            .collect::<Vec<_>>()
                            .join(" > ")
                    })
                    .unwrap_or_else(|| format!("Missing parent: {}", doc.parent));
                ui.text_disabled(format!("{chain} > {}", doc.name));
            }
            ui.separator();
            let before = self.asset.clone();
            let available = ui.content_region_avail();
            let left = (available[0] * 0.18).clamp(185., 235.);
            let right = (available[0] * 0.22).clamp(245., 310.);
            let body = (available[1] - 110.).max(80.);
            let _header = ui.push_style_color(imgui::StyleColor::Header, [0.19, 0.19, 0.19, 1.]);
            let _header_hover =
                ui.push_style_color(imgui::StyleColor::HeaderHovered, [0.28, 0.28, 0.28, 1.]);
            let _header_active =
                ui.push_style_color(imgui::StyleColor::HeaderActive, [0.24, 0.34, 0.40, 1.]);
            ui.child_window("bp-navigation")
                .size([left, body])
                .build(|| {
                    ui.child_window("bp-components")
                        .size([0., (body * 0.43).max(130.)])
                        .border(true)
                        .build(|| {
                            ui.text_disabled("\u{eb29} Components");
                            ui.separator();
                            match template_editor::browser(
                                ui,
                                self.asset.as_mut().unwrap(),
                                registry,
                                &mut self.template_editor,
                            ) {
                                Ok(true) => {
                                    self.details = Details::Template;
                                    self.selected.clear();
                                }
                                Err(error) => self.error = Some(error),
                                _ => {}
                            }
                        });
                    ui.child_window("bp-members")
                        .size([0., 0.])
                        .border(true)
                        .build(|| self.member_browser(ui, registry));
                });
            ui.same_line();
            ui.child_window("bp-graph")
                .size([(available[0] - left - right - 16.).max(150.), body])
                .border(true)
                .build(|| {
                    if let Some(graph) = self.current() {
                        ui.text_disabled(format!(
                            "\u{eae9} {}  >  {}",
                            self.asset.as_ref().unwrap().name,
                            if graph.override_id.is_some() {
                                "Event Graph".into()
                            } else {
                                display_port(&graph.name)
                            }
                        ));
                    }
                    ui.separator();
                    self.graph_panel(ui, registry)
                });
            ui.same_line();
            ui.child_window("bp-details")
                .size([0., body])
                .border(true)
                .build(|| {
                    ui.text_disabled("\u{eae9} Details");
                    ui.separator();
                    ui.set_next_item_width(-1.);
                    if self.details == Details::BuildOptions {
                        ui.text("Build Options");
                        build_options(ui);
                    } else {
                        self.members(ui, registry);
                    }
                });
            if self.drag.is_none()
                && self.drag_before.is_none()
                && let Some(before) = before
            {
                self.checkpoint(before);
            }
            ui.separator();
            let mut focus_diagnostic = None;
            ui.child_window("bp-diagnostics").size([0., 0.]).build(|| {
                ui.text("Compiler diagnostics");
                if self.diagnostics.is_empty() {
                    ui.text_disabled("Compile to validate the document and inspect generated C++.");
                }
                for (index, diagnostic) in self.diagnostics.iter().enumerate() {
                    if self.diagnostic_nodes.contains_key(&index) {
                        if button(ui, &format!("Show node##bp-diagnostic-{index}")) {
                            focus_diagnostic = Some(index);
                        }
                        ui.same_line();
                    }
                    ui.text_wrapped(diagnostic);
                }
            });
            if let Some(index) = focus_diagnostic {
                self.focus_diagnostic(index);
            }
        });
        if !visible {
            if self.dirty() {
                self.close_requested = true;
            } else {
                self.open = false;
            }
        }
        if self.close_requested {
            ui.open_popup("Unsaved Blueprint");
            if let Some(_popup) = ui.begin_modal_popup("Unsaved Blueprint") {
                ui.text("Save Blueprint changes before closing?");
                if button(ui, "Save and close") {
                    match self.save() {
                        Ok(()) => {
                            saved = true;
                            self.open = false;
                            self.close_requested = false;
                            ui.close_current_popup();
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                ui.same_line();
                if button(ui, "Discard and close") {
                    self.asset = None;
                    self.open = false;
                    self.close_requested = false;
                    ui.close_current_popup();
                }
                ui.same_line();
                if button(ui, "Keep editing") {
                    self.close_requested = false;
                    ui.close_current_popup();
                }
            }
        }
        if self.show_generated {
            ui.window("Blueprint generated C++")
                .opened(&mut self.show_generated)
                .size([850., 600.], Condition::FirstUseEver)
                .build(|| {
                    ui.input_text_multiline("##generated-bp", &mut self.generated, [-1., -1.])
                        .read_only(true)
                        .build();
                });
        }
        saved
    }

    fn member_browser(&mut self, ui: &Ui, registry: &Registry) {
        ui.text_disabled("\u{eae9} My Blueprint");
        ui.separator();
        if button(ui, "\u{ea60} Add##member") {
            ui.open_popup("bp-member-add");
        }
        ui.same_line();
        ui.set_next_item_width(-1.);
        ui.input_text("##member-search", &mut self.member_search)
            .hint("Search")
            .build();
        record_control(ui, "bp-member-search");
        if let Some(_popup) = ui.begin_popup("bp-member-add") {
            if choose(ui, "Function") {
                self.details = Details::CreateFunction;
            }
            if choose(ui, "Variable") {
                self.details = Details::CreateVariable;
            }
            if let Some(_menu) = ui.begin_menu("Override event") {
                for function in functions(self.asset.as_ref().unwrap(), registry) {
                    if function.event
                        && !function.final_method
                        && choose(ui, &function_label(&function))
                    {
                        self.add_graph(function.name.clone(), Some(&function));
                        self.details = Details::Function;
                    }
                }
            }
        }
        ui.separator();
        let query = self.member_search.to_lowercase();
        for (category, event) in [("GRAPHS", true), ("FUNCTIONS", false)] {
            if ui.collapsing_header(category, imgui::TreeNodeFlags::DEFAULT_OPEN) {
                let graphs: Vec<_> = self
                    .asset
                    .as_ref()
                    .unwrap()
                    .functions
                    .iter()
                    .enumerate()
                    .filter(|(_, graph)| {
                        graph.override_id.is_some() == event
                            && graph.name.to_lowercase().contains(&query)
                    })
                    .map(|(index, graph)| (index, graph.id.clone(), graph.name.clone()))
                    .collect();
                if graphs.is_empty() {
                    ui.text_disabled(if query.is_empty() {
                        "  None"
                    } else {
                        "  No matches"
                    });
                }
                for (index, id, name) in graphs {
                    if ui
                        .selectable_config(format!("\u{eae9} {}##{id}", event_label(&name)))
                        .selected(self.graph == index && self.details == Details::Function)
                        .build()
                    {
                        self.graph = index;
                        self.details = Details::Function;
                        self.selected.clear();
                        self.wire = None;
                    }
                }
            }
        }
        if ui.collapsing_header("VARIABLES", imgui::TreeNodeFlags::DEFAULT_OPEN) {
            let mut values: Vec<_> = self
                .asset
                .as_ref()
                .unwrap()
                .variables
                .iter()
                .map(|value| (value.id.clone(), value.name.clone(), false))
                .collect();
            values.extend(
                properties(self.asset.as_ref().unwrap(), registry)
                    .into_iter()
                    .map(|value| (value.id, value.name, true)),
            );
            let mut shown = false;
            for (id, name, inherited) in values
                .into_iter()
                .filter(|(_, name, _)| name.to_lowercase().contains(&query))
            {
                shown = true;
                if ui
                    .selectable_config(format!(
                        "{} {}{}##{id}",
                        "\u{eae9}",
                        display_port(&name),
                        if inherited { " (inherited)" } else { "" }
                    ))
                    .selected(self.detail_member == id)
                    .build()
                {
                    self.detail_member = id.clone();
                    self.details = if inherited {
                        Details::Defaults
                    } else {
                        Details::Variable
                    };
                    self.selected.clear();
                }
                record_control(ui, &format!("bp-member:{id}"));
            }
            if !shown {
                ui.text_disabled(if query.is_empty() {
                    "  None"
                } else {
                    "  No matches"
                });
            }
        }
    }
    fn members(&mut self, ui: &Ui, registry: &Registry) {
        if self.details == Details::Template
            && let Err(error) = template_editor::draw(
                ui,
                self.asset.as_mut().unwrap(),
                registry,
                &mut self.template_editor,
            )
        {
            self.error = Some(error);
        }
        if self.details == Details::Class
            && ui.collapsing_header("Class settings", imgui::TreeNodeFlags::DEFAULT_OPEN)
        {
            let doc = self.asset.as_mut().unwrap();
            ui.input_text("Class name", &mut doc.name).build();
            ui.text_disabled(format!("Class ID: {}", doc.id));
            if let Ok(model) = registry.model()
                && let Some(parent) = model.class(&doc.parent)
                && parent.family == crate::reflection_schema::ClassFamily::Component
            {
                ui.text("Compatible Actors");
                let inherited = parent
                    .component
                    .as_ref()
                    .map(|c| c.owners.clone())
                    .unwrap_or_default();
                if ui.checkbox("Use parent compatibility", &mut doc.component.is_none()) {
                    if doc.component.is_some() {
                        doc.component = None;
                    } else {
                        doc.component = Some(crate::reflection_schema::ComponentContract {
                            owners: inherited.clone(),
                            ..Default::default()
                        });
                    }
                }
                if let Some(contract) = &mut doc.component {
                    for domain in [
                        crate::reflection_schema::Domain::World3D,
                        crate::reflection_schema::Domain::World2D,
                        crate::reflection_schema::Domain::UI,
                    ] {
                        if !inherited.is_empty() && !inherited.contains(&domain) {
                            continue;
                        }
                        let mut enabled = contract.owners.contains(&domain);
                        if ui.checkbox(domain.label(), &mut enabled) {
                            if enabled {
                                contract.owners.insert(domain);
                            } else if contract.owners.len() > 1 {
                                contract.owners.remove(&domain);
                            }
                        }
                    }
                }
            }

            let current = registry
                .classes
                .get(&doc.parent)
                .map(|c| c.cpp_name.as_str())
                .unwrap_or("Missing parent");
            let mut parent = None;
            if let Some(_combo) = ui.begin_combo("Reparent", current) {
                for class in registry.classes.values() {
                    if class.blueprintable
                        && !class.final_class
                        && class.backend.id == "native"
                        && class.id != doc.id
                        && !registry
                            .ancestry(&class.cpp_name)
                            .iter()
                            .any(|c| c.id == doc.id)
                        && ui.selectable(&class.cpp_name)
                    {
                        parent = Some(class.id.clone());
                    }
                }
            }
            ui.text_wrapped("Reparenting validates the complete project before accepting. No defaults, events or wires are deleted.");
            if let Some(parent) = parent {
                let root = self
                    .path
                    .as_ref()
                    .and_then(|path| {
                        path.ancestors()
                            .find(|p| p.file_name().is_some_and(|n| n == "assets"))
                    })
                    .and_then(Path::parent)
                    .map(Path::to_owned);
                if let Some(root) = root {
                    let old = self.asset.as_ref().unwrap().parent.clone();
                    // The object model is the single source of family/domain rules. Ask it
                    // before paying for a speculative compile, and surface its codes: they
                    // explain *why* a parent is impossible, which a compile error cannot.
                    let id = self.asset.as_ref().unwrap().id.clone();
                    let model_error = match registry.model() {
                        Ok(model)
                            if model.class(&id).is_some() && model.class(&parent).is_some() =>
                        {
                            model.validate_reparent(&id, &parent).err().map(|d| {
                                d.iter()
                                    .map(|d| format!("{}: {}", d.code, d.message))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            })
                        }
                        Ok(_) => None,
                        Err(diagnostics) => Some(
                            diagnostics
                                .iter()
                                .map(|d| format!("{}: {}", d.code, d.message))
                                .collect::<Vec<_>>()
                                .join("\n"),
                        ),
                    };
                    if let Some(message) = model_error {
                        self.error = Some(format!(
                            "Reparenting was rejected; the original parent and all graph data were retained.\n{message}"
                        ));
                    } else {
                        self.asset.as_mut().unwrap().parent = parent;
                        self.compile(&root, registry);
                        if !self.compile_valid {
                            self.asset.as_mut().unwrap().parent = old;
                            self.error=Some("Reparenting was rejected. Original parent and all graph data were retained; see diagnostics.".into());
                        }
                    }
                } else {
                    self.error = Some(
                        "Save the Blueprint below the project's assets folder before reparenting."
                            .into(),
                    );
                }
            }
        }
        if self.details == Details::CreateFunction
            && ui.collapsing_header("New function", imgui::TreeNodeFlags::DEFAULT_OPEN)
        {
            ui.input_text("Name##function", &mut self.function_name)
                .build();
            if button(ui, "Add function") {
                if identifier(&self.function_name) {
                    self.add_graph(self.function_name.clone(), None);
                    self.details = Details::Function;
                } else {
                    self.error = Some("Function name must be a portable identifier.".into());
                }
            }
            if let Some(_combo) = ui.begin_combo("Override event", "Choose inherited event...") {
                for f in functions(self.asset.as_ref().unwrap(), registry) {
                    if f.event && !f.final_method && ui.selectable(function_label(&f)) {
                        self.add_graph(f.name.clone(), Some(&f));
                    }
                }
            }
        }
        if self.details == Details::Defaults
            && ui.collapsing_header("Class defaults", imgui::TreeNodeFlags::DEFAULT_OPEN)
        {
            let props = properties(self.asset.as_ref().unwrap(), registry);
            for p in props {
                if !self.detail_member.is_empty() && p.id != self.detail_member {
                    continue;
                }
                let doc = self.asset.as_mut().unwrap();
                let mut value = doc
                    .defaults
                    .get(&p.id)
                    .cloned()
                    .unwrap_or(p.default.clone());
                ui.text(format!("{} : {}", p.name, p.value_type.label()));
                if edit_value(
                    ui,
                    &format!("##default-{}", p.id),
                    &mut value,
                    &p.value_type,
                    registry,
                ) {
                    doc.defaults.insert(p.id.clone(), value);
                }
                if doc.defaults.contains_key(&p.id)
                    && ui.small_button(format!("Reset to inherited##{}", p.id))
                {
                    doc.defaults.remove(&p.id);
                }
            }
            let known: BTreeSet<_> = properties(self.asset.as_ref().unwrap(), registry)
                .iter()
                .map(|p| p.id.clone())
                .collect();
            let orphans: Vec<_> = self
                .asset
                .as_ref()
                .unwrap()
                .defaults
                .iter()
                .filter(|(key, _)| !known.contains(*key))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for (key, value) in orphans {
                let _id = ui.push_id(&key);
                ui.text_colored([1., 0.6, 0.2, 1.], format!("Orphan preserved: {key}"));
                ui.text_wrapped(value.to_string());
                if let Some(_combo) =
                    ui.begin_combo("Remap override", "Choose a compatible member...")
                {
                    for property in properties(self.asset.as_ref().unwrap(), registry) {
                        if crate::script_values::valid(&value, &property.value_type)
                            && !self
                                .asset
                                .as_ref()
                                .unwrap()
                                .defaults
                                .contains_key(&property.id)
                            && ui.selectable(&property.name)
                        {
                            let doc = self.asset.as_mut().unwrap();
                            doc.defaults.remove(&key);
                            doc.defaults.insert(property.id, value.clone());
                        }
                    }
                }
                if ui.small_button("Discard orphan override") {
                    self.asset.as_mut().unwrap().defaults.remove(&key);
                }
            }
        }
        if matches!(self.details, Details::Variable | Details::CreateVariable)
            && ui.collapsing_header("Variable", imgui::TreeNodeFlags::DEFAULT_OPEN)
        {
            let doc = self.asset.as_mut().unwrap();
            let mut remove = None;
            for variable in &mut doc.variables {
                if self.details == Details::CreateVariable || variable.id != self.detail_member {
                    continue;
                }
                let _id = ui.push_id(&variable.id);
                ui.input_text("Name", &mut variable.name).build();
                if let Some(_combo) = ui.begin_combo("Type", variable.value_type.label()) {
                    for mut ty in authoring_types(registry) {
                        if let schema::Type::ClassRef { base } = &mut ty {
                            *base = doc.parent.clone();
                        }
                        if ui.selectable(ty.label()) {
                            if matches!(
                                ty,
                                schema::Type::SequenceHandle | schema::Type::EffectHandle
                            ) {
                                variable.editable = false;
                                variable.timeline_animatable = false;
                                variable.default = Value::Null;
                            }
                            variable.value_type = ty;
                        }
                    }
                }
                edit_value(
                    ui,
                    "Default",
                    &mut variable.default,
                    &variable.value_type,
                    registry,
                );
                if matches!(
                    variable.value_type,
                    schema::Type::SequenceHandle | schema::Type::EffectHandle
                ) {
                    ui.text_disabled("Runtime playback handle; assigned by a Play node.");
                } else {
                    ui.checkbox("Instance editable", &mut variable.editable);
                    ui.checkbox("Timeline animatable", &mut variable.timeline_animatable);
                }
                if !crate::script_values::valid(&variable.default, &variable.value_type)
                    && ui.small_button("Reset incompatible default")
                {
                    variable.default = default_value(&variable.value_type);
                }
                if ui.small_button("Remove declaration") {
                    remove = Some(variable.id.clone());
                }
                ui.separator();
            }
            if let Some(id) = remove {
                doc.variables.retain(|v| v.id != id);
                self.error=Some("Variable declaration removed. Existing graph references are preserved and will be diagnosed; Undo restores the declaration.".into());
            }
            if self.details == Details::CreateVariable {
                ui.input_text("Name##variable", &mut self.variable_name)
                    .build();
                type_picker(ui, "Type##variable", &mut self.type_index, registry);
                if button(ui, "Add variable") {
                    if identifier(&self.variable_name)
                        && !doc.variables.iter().any(|v| v.name == self.variable_name)
                    {
                        let mut ty = authoring_type(self.type_index, registry);
                        if let schema::Type::ClassRef { base } = &mut ty {
                            *base = doc.parent.clone();
                        }
                        self.detail_member = id();
                        doc.variables.push(asset::Variable {
                            timeline_animatable: false,
                            id: self.detail_member.clone(),
                            name: self.variable_name.clone(),
                            default: default_value(&ty),
                            editable: !matches!(
                                ty,
                                schema::Type::SequenceHandle | schema::Type::EffectHandle
                            ),
                            value_type: ty,
                        });
                        self.details = Details::Variable;
                    } else {
                        self.error = Some("Variable requires a unique portable identifier.".into());
                    }
                }
            }
        }
        let parent_id = self.asset.as_ref().unwrap().parent.clone();
        let mut remove_graph = false;
        if self.details == Details::Function
            && ui.collapsing_header("Function signature", imgui::TreeNodeFlags::DEFAULT_OPEN)
            && let Some(graph) = self.asset.as_mut().unwrap().functions.get_mut(self.graph)
        {
            let mut timeline = match graph.timeline {
                None => 0,
                Some(schema::TimelineCall::CrossingEvent) => 1,
                Some(schema::TimelineCall::IdempotentAction) => 2,
            };
            if ui.combo_simple_string(
                "Timeline exposure",
                &mut timeline,
                &["Not exposed", "Crossing event", "Idempotent action"],
            ) {
                graph.timeline = match timeline {
                    1 => Some(schema::TimelineCall::CrossingEvent),
                    2 => Some(schema::TimelineCall::IdempotentAction),
                    _ => None,
                };
            }
            if graph.override_id.is_some() {
                ui.text_wrapped("Inherited event signatures are owned by the parent.");
            } else {
                ui.input_text("Function name", &mut graph.name).build();
                if let Some(_combo) = ui.begin_combo("Return", graph.returns.label()) {
                    if ui.selectable("void") {
                        graph.returns = schema::Type::Void;
                    }
                    for mut ty in authoring_types(registry) {
                        if let schema::Type::ClassRef { base } = &mut ty {
                            *base = parent_id.clone();
                        }
                        if ui.selectable(ty.label()) {
                            graph.returns = ty;
                        }
                    }
                }
                let mut remove_parameter = None;
                for (index, parameter) in graph.parameters.iter_mut().enumerate() {
                    let _id = ui.push_id_usize(index);
                    ui.input_text("Argument name", &mut parameter.name).build();
                    ui.text_disabled(parameter.value_type.label());
                    let mut direction = match parameter.direction {
                        schema::Direction::Value => 0,
                        schema::Direction::ConstReference => 1,
                        schema::Direction::MutableReference => 2,
                    };
                    if ui.combo_simple_string(
                        "Direction",
                        &mut direction,
                        &["Value", "Const reference", "Mutable reference"],
                    ) {
                        parameter.direction = match direction {
                            0 => schema::Direction::Value,
                            1 => schema::Direction::ConstReference,
                            _ => schema::Direction::MutableReference,
                        };
                    }
                    if ui.small_button("Remove argument") {
                        remove_parameter = Some(index);
                    }
                }
                if let Some(index) = remove_parameter {
                    graph.parameters.remove(index);
                }
                ui.input_text("Parameter name", &mut self.parameter_name)
                    .build();
                type_picker(ui, "Parameter type", &mut self.type_index, registry);
                if button(ui, "Add parameter")
                    && identifier(&self.parameter_name)
                    && !graph
                        .parameters
                        .iter()
                        .any(|p| p.name == self.parameter_name)
                {
                    let mut value_type = authoring_type(self.type_index, registry);
                    if let schema::Type::ClassRef { base } = &mut value_type {
                        *base = parent_id.clone();
                    }
                    graph.parameters.push(schema::Parameter {
                        name: self.parameter_name.clone(),
                        value_type,
                        direction: schema::Direction::Value,
                    });
                }
            }
            remove_graph = button(ui, "Remove graph (Undo available)");
            ui.text_wrapped("Removing a graph or argument preserves references in other graphs so compilation can diagnose them.");
        }
        if remove_graph {
            let doc = self.asset.as_mut().unwrap();
            let removed = doc.functions.remove(self.graph);
            for node in removed.nodes {
                doc.layout.positions.remove(&node.id);
                doc.layout.comments.remove(&node.id);
            }
            self.graph = self.graph.min(doc.functions.len().saturating_sub(1));
            self.selected.clear();
            self.wire = None;
        }
        if self.details == Details::Node
            && ui.collapsing_header("Selected node", imgui::TreeNodeFlags::DEFAULT_OPEN)
        {
            self.node_inspector(ui, registry);
        }
    }

    fn graph_panel(&mut self, ui: &Ui, registry: &Registry) {
        if self.current().is_none() {
            ui.text_wrapped("Data-only Blueprint: parent behavior is inherited. Add a function or override an event to begin a graph.");
            return;
        }
        if button(ui, "Add node...") {
            self.catalog_position = None;
            self.action_menu.open(ui);
        }
        ui.same_line();
        if button(ui, "Frame all") || std::mem::take(&mut self.frame_events) {
            let available = ui.content_region_avail();
            let doc = self.asset.as_ref().unwrap();
            let graph = &doc.functions[self.graph];
            let mut minimum = [f32::INFINITY; 2];
            let mut maximum = [f32::NEG_INFINITY; 2];
            for graph in canvas_graphs(doc, self.graph) {
                for node in &graph.nodes {
                    let p = node_position(doc, graph, &node.id);
                    let pins = node_sockets_with_assets(
                        doc,
                        graph,
                        node,
                        registry,
                        &self.playback_timelines,
                        &self.playback_effects,
                    );
                    let count = pins
                        .iter()
                        .filter(|p| p.output)
                        .count()
                        .max(pins.iter().filter(|p| !p.output).count());
                    minimum[0] = minimum[0].min(p[0]);
                    minimum[1] = minimum[1].min(p[1] - 24.);
                    maximum[0] = maximum[0].max(p[0] + 230.);
                    maximum[1] = maximum[1].max(p[1] + 50. + count as f32 * 24.);
                }
            }
            if !graph.nodes.is_empty() {
                self.zoom = ((available[0] - 60.) / (maximum[0] - minimum[0]).max(1.))
                    .min((available[1] - 60.) / (maximum[1] - minimum[1]).max(1.))
                    .clamp(0.15, 1.2);
                self.pan = [30. - minimum[0] * self.zoom, 30. - minimum[1] * self.zoom];
            }
        }
        ui.same_line();
        ui.text_disabled("Middle drag: pan | Wheel: zoom | Double-click wire: reroute");
        self.catalog(ui, registry);
        let origin = ui.cursor_screen_pos();
        let size = ui.content_region_avail();
        ui.invisible_button("bp-canvas", [size[0].max(1.), size[1].max(1.)]);
        let hovered = ui.is_item_hovered();
        let mouse = ui.io().mouse_pos;
        if hovered && ui.io().mouse_wheel != 0. {
            let old = self.zoom;
            self.zoom = (self.zoom * 1.1f32.powf(ui.io().mouse_wheel)).clamp(0.3, 2.);
            self.pan = [
                mouse[0] - origin[0] - (mouse[0] - origin[0] - self.pan[0]) * self.zoom / old,
                mouse[1] - origin[1] - (mouse[1] - origin[1] - self.pan[1]) * self.zoom / old,
            ];
        }
        if hovered && ui.is_mouse_dragging(MouseButton::Middle) {
            self.pan[0] += ui.io().mouse_delta[0];
            self.pan[1] += ui.io().mouse_delta[1];
        }
        if hovered && !ui.io().want_text_input {
            if ui.is_key_pressed(imgui::Key::Delete) {
                self.delete_selected();
            }
            if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::C) {
                self.copy_to_clipboard(ui);
            }
            if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::V) {
                self.paste_from_clipboard(ui);
            }
            if ui.io().key_ctrl && ui.is_key_pressed(imgui::Key::D) {
                self.copy_selected();
                self.paste();
            }
            if ui.is_key_pressed(imgui::Key::Escape) {
                self.wire = None;
            }
        }
        let doc = self.asset.as_ref().unwrap();
        let Some(_) = doc.functions.get(self.graph) else {
            return;
        };
        let mut sockets = vec![];
        let mut boxes = vec![];
        let mut boolean_inputs = vec![];
        let mut numeric_inputs = vec![];
        let mut wires = vec![];
        // Match the compact graph language in the supplied visual reference:
        // neutral gray grid, almost-black bodies, semantic narrow headers,
        // white execution arrows and independently colored data sockets.
        ui.set_window_font_scale(0.85 * self.zoom);
        let draw = ui.get_window_draw_list();
        draw.with_clip_rect_intersect(origin, [origin[0] + size[0], origin[1] + size[1]], || {
            draw.add_rect(
                origin,
                [origin[0] + size[0], origin[1] + size[1]],
                [0.40, 0.40, 0.40, 1.],
            )
            .filled(true)
            .build();
            for (spacing, color) in [
                (16., [0.435, 0.435, 0.435, 1.]),
                (64., [0.46, 0.46, 0.46, 1.]),
            ] {
                let grid = spacing * self.zoom;
                let mut x = origin[0] + self.pan[0].rem_euclid(grid);
                while x < origin[0] + size[0] {
                    draw.add_line([x, origin[1]], [x, origin[1] + size[1]], color)
                        .build();
                    x += grid;
                }
                let mut y = origin[1] + self.pan[1].rem_euclid(grid);
                while y < origin[1] + size[1] {
                    draw.add_line([origin[0], y], [origin[0] + size[0], y], color)
                        .build();
                    y += grid;
                }
            }
            draw.channels_split(3, |channels| {
                channels.set_current(1);
                for graph in canvas_graphs(doc, self.graph) {
                    for node in &graph.nodes {
                        let at = node_position(doc, graph, &node.id);
                        let p = [
                            origin[0] + self.pan[0] + at[0] * self.zoom,
                            origin[1] + self.pan[1] + at[1] * self.zoom,
                        ];
                        let pins = node_sockets_with_assets(
                            doc,
                            graph,
                            node,
                            registry,
                            &self.playback_timelines,
                            &self.playback_effects,
                        );
                        let left: Vec<_> = pins.iter().filter(|p| !p.output).cloned().collect();
                        let right: Vec<_> = pins.iter().filter(|p| p.output).cloned().collect();
                        let title = if matches!(node.kind, NodeKind::Entry) {
                            format!(
                                "{} {}",
                                if graph.override_id.is_some() {
                                    "Event"
                                } else {
                                    "Function"
                                },
                                event_label(&graph.name)
                            )
                        } else {
                            playback_label(node, &self.playback_timelines, &self.playback_effects)
                                .unwrap_or_else(|| node_label(node, doc, registry))
                        };
                        // Callable icon only: a play triangle in an event header
                        // looks like an execution input, although it is decorative.
                        let title = match node.kind {
                            NodeKind::CallParent => {
                                format!("\u{eae9} Parent: {}", event_label(&graph.name))
                            }
                            NodeKind::Call { .. } | NodeKind::CallOn { .. } => {
                                format!("\u{eae9} {title}")
                            }
                            _ => title,
                        };
                        let subtitle = match &node.kind {
                            NodeKind::CallOn { class, .. } => Some(format!(
                                "Target is {}",
                                registry
                                    .classes
                                    .get(class)
                                    .map(|class| class.cpp_name.as_str())
                                    .unwrap_or("Missing class")
                            )),
                            NodeKind::CallParent => {
                                Some(format!("Target is {}", parent_class_label(doc, registry)))
                            }
                            NodeKind::Entry if graph.inherits_event() => Some(format!(
                                "Inherited from {}",
                                parent_class_label(doc, registry)
                            )),
                            NodeKind::Call { .. } => Some(format!("Target is {}", doc.name)),
                            _ => None,
                        };
                        let reroute = matches!(node.kind, NodeKind::Reroute);
                        let row_width = (0..left.len().max(right.len()))
                            .map(|index| {
                                let input = left.get(index);
                                let input_label = input
                                    .map_or(0., |pin| ui.calc_text_size(&pin.label)[0] / self.zoom);
                                let output_label = right
                                    .get(index)
                                    .map_or(0., |pin| ui.calc_text_size(&pin.label)[0] / self.zoom);
                                let literal = input
                                    .and_then(|pin| {
                                        inline_values::width(&inline_values::parts(node, pin))
                                    })
                                    .or_else(|| {
                                        input.and_then(|pin| node.inputs.get(&pin.pin)).and_then(
                                            |input| match input {
                                                Input::Literal { value, .. } => {
                                                    Some(if value.is_boolean() {
                                                        13.
                                                    } else {
                                                        ui.calc_text_size(literal_display(value))[0]
                                                            / self.zoom
                                                            + 8.
                                                    })
                                                }
                                                _ => None,
                                            },
                                        )
                                    });
                                socket_row_width(input_label, literal, output_label)
                            })
                            .fold(0., f32::max);
                        let width = if reroute {
                            40. * self.zoom
                        } else {
                            (ui.calc_text_size(&title)[0] / self.zoom + 28.)
                                .clamp(126., 300.)
                                .max(row_width)
                                .max(
                                    subtitle
                                        .as_ref()
                                        .map_or(0., |s| ui.calc_text_size(s)[0] / self.zoom + 24.),
                                )
                                * self.zoom
                        };
                        let header = 20. * self.zoom;
                        let first_pin = if subtitle.is_some() { 45. } else { 33. };
                        let height = if reroute {
                            24. * self.zoom
                        } else {
                            (first_pin
                                + 9.
                                + left.len().max(right.len()).saturating_sub(1) as f32 * 22.)
                                * self.zoom
                        };
                        let max = [p[0] + width, p[1] + height];
                        boxes.push((node.id.clone(), p, max));
                        #[cfg(test)]
                        CONTROLS.with(|c| {
                            c.borrow_mut().insert(
                                format!("bp-node-header:{}", node.id),
                                [p[0] + 35. * self.zoom, p[1] + 10. * self.zoom],
                            );
                        });
                        if let Some(comment) = doc.layout.comments.get(&node.id) {
                            draw.add_text(
                                [p[0], p[1] - 18. * self.zoom],
                                [0.98, 0.87, 0.54, 1.],
                                comment,
                            );
                        }
                        if !reroute {
                            draw.add_rect(
                                [p[0] + 3., p[1] + 4.],
                                [max[0] + 3., max[1] + 4.],
                                [0., 0., 0., 0.32],
                            )
                            .rounding(4. * self.zoom)
                            .filled(true)
                            .build();
                            draw.add_rect(p, max, [0.105, 0.115, 0.11, 1.])
                                .rounding(4. * self.zoom)
                                .filled(true)
                                .build();
                        }
                        if !reroute {
                            let color = node_header(node, graph, doc, registry);
                            let top = [
                                (color[0] + 0.10).min(1.),
                                (color[1] + 0.10).min(1.),
                                (color[2] + 0.10).min(1.),
                                1.,
                            ];
                            draw.add_rect_filled_multicolor(
                                [p[0] + 1., p[1] + 1.],
                                [max[0] - 1., p[1] + header],
                                top,
                                top,
                                color,
                                color,
                            );
                            draw.with_clip_rect_intersect(
                                [p[0] + 5. * self.zoom, p[1]],
                                [max[0] - 5. * self.zoom, p[1] + header],
                                || {
                                    draw.add_text(
                                        [p[0] + 7. * self.zoom, p[1] + 3. * self.zoom],
                                        [0.94, 0.94, 0.94, 1.],
                                        &title,
                                    );
                                },
                            );
                            if let Some(subtitle) = subtitle {
                                draw.add_text(
                                    [p[0] + 12. * self.zoom, p[1] + header + 2. * self.zoom],
                                    [0.55, 0.64, 0.63, 1.],
                                    subtitle,
                                );
                            }
                        }
                        let active = self.debug_node.as_ref() == Some(&node.id);
                        let selected = self.selected.contains(&node.id);
                        let border = if active {
                            [1., 0.45, 0.08, 1.]
                        } else if selected {
                            [1., 0.70, 0.18, 1.]
                        } else {
                            [0.045, 0.045, 0.045, 1.]
                        };
                        if reroute {
                            let center = [p[0] + 20. * self.zoom, p[1] + 12. * self.zoom];
                            let color = pins
                                .first()
                                .map_or([0.8, 0.8, 0.8, 1.], |pin| socket_color(&pin.ty));
                            draw.add_line(
                                [p[0] + 4. * self.zoom, center[1]],
                                [max[0] - 4. * self.zoom, center[1]],
                                color,
                            )
                            .thickness(2.)
                            .build();
                            draw.add_circle(center, 7. * self.zoom, border)
                                .filled(true)
                                .build();
                            draw.add_circle(center, 4.5 * self.zoom, color)
                                .filled(true)
                                .build();
                            if inside(mouse, p, max) && self.wire.is_none() {
                                ui.set_mouse_cursor(Some(imgui::MouseCursor::ResizeAll));
                            }
                        } else {
                            draw.add_rect(p, max, border)
                                .rounding(4. * self.zoom)
                                .thickness(if active {
                                    3.
                                } else if selected {
                                    2.
                                } else {
                                    1.
                                })
                                .build();
                        }
                        for (output, pins) in [(false, left), (true, right)] {
                            for (index, pin) in pins.into_iter().enumerate() {
                                let point = [
                                    if output {
                                        max[0] - if reroute { 4. } else { 10. } * self.zoom
                                    } else {
                                        p[0] + if reroute { 4. } else { 10. } * self.zoom
                                    },
                                    p[1] + if reroute {
                                        12. * self.zoom
                                    } else {
                                        (first_pin + index as f32 * 22.) * self.zoom
                                    },
                                ];
                                let connected = socket_connected(graph, node, &pin);
                                if !reroute {
                                    paint_socket(&draw, point, &pin.ty, connected, self.zoom);
                                }
                                if !reroute {
                                    let text = pin.label.clone();
                                    let text_width = ui.calc_text_size(&text)[0];
                                    draw.add_text(
                                        [
                                            if output {
                                                point[0] - text_width - 10. * self.zoom
                                            } else {
                                                point[0] + 10. * self.zoom
                                            },
                                            point[1] - 6.5 * self.zoom,
                                        ],
                                        [0.88, 0.88, 0.88, 1.],
                                        text,
                                    );
                                    if !output {
                                        let mut x = point[0] + text_width + 17. * self.zoom;
                                        for (axis, value) in inline_values::parts(node, &pin) {
                                            if let Some(axis) = axis {
                                                draw.add_text(
                                                    [x, point[1] - 6.5 * self.zoom],
                                                    [0.7, 0.7, 0.7, 1.],
                                                    ["X", "Y", "Z"][axis],
                                                );
                                                x += 11. * self.zoom;
                                            }
                                            let a = [x, point[1] - 7. * self.zoom];
                                            let b = [x + 35. * self.zoom, a[1] + 15. * self.zoom];
                                            draw.add_rect(a, b, [0.24, 0.25, 0.24, 1.])
                                                .filled(true)
                                                .rounding(2.)
                                                .build();
                                            draw.add_rect(a, b, [0.48, 0.49, 0.48, 1.])
                                                .rounding(2.)
                                                .build();
                                            draw.with_clip_rect_intersect(a, b, || {
                                                draw.add_text(
                                                    [a[0] + 2., a[1] + 1.],
                                                    [0.93, 0.93, 0.93, 1.],
                                                    value.to_string(),
                                                );
                                            });
                                            if inside(mouse, a, b) {
                                                ui.set_mouse_cursor(Some(
                                                    imgui::MouseCursor::TextInput,
                                                ));
                                            }
                                            #[cfg(test)]
                                            CONTROLS.with(|c| {
                                                c.borrow_mut().insert(
                                                    format!(
                                                        "bp-number:{}:{}:{axis:?}",
                                                        pin.node, pin.pin
                                                    ),
                                                    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5],
                                                );
                                            });
                                            numeric_inputs.push(inline_values::Field {
                                                socket: pin.clone(),
                                                axis,
                                                value,
                                                min: a,
                                                max: b,
                                            });
                                            x = b[0] + 4. * self.zoom;
                                        }
                                    }
                                    if !output
                                        && inline_values::parts(node, &pin).is_empty()
                                        && let Some(Input::Literal { value, .. }) =
                                            node.inputs.get(&pin.pin)
                                    {
                                        let a = [
                                            point[0] + text_width + 17. * self.zoom,
                                            point[1] - 7. * self.zoom,
                                        ];
                                        if let Some(checked) = value.as_bool() {
                                            let b =
                                                [a[0] + 13. * self.zoom, a[1] + 13. * self.zoom];
                                            if b[0] < max[0] - 16. * self.zoom {
                                                draw.add_rect(a, b, [0.055, 0.065, 0.07, 1.])
                                                    .filled(true)
                                                    .build();
                                                draw.add_rect(a, b, [0.49, 0.53, 0.53, 1.])
                                                    .rounding(1.)
                                                    .build();
                                                if checked {
                                                    draw.add_line(
                                                        [
                                                            a[0] + 2. * self.zoom,
                                                            a[1] + 6. * self.zoom,
                                                        ],
                                                        [
                                                            a[0] + 5. * self.zoom,
                                                            a[1] + 9. * self.zoom,
                                                        ],
                                                        [0.61, 0.85, 0.98, 1.],
                                                    )
                                                    .thickness(1.6 * self.zoom)
                                                    .build();
                                                    draw.add_line(
                                                        [
                                                            a[0] + 5. * self.zoom,
                                                            a[1] + 9. * self.zoom,
                                                        ],
                                                        [
                                                            a[0] + 11. * self.zoom,
                                                            a[1] + 3. * self.zoom,
                                                        ],
                                                        [0.61, 0.85, 0.98, 1.],
                                                    )
                                                    .thickness(1.6 * self.zoom)
                                                    .build();
                                                }
                                                boolean_inputs.push((
                                                    node.id.clone(),
                                                    pin.pin.clone(),
                                                    a,
                                                    b,
                                                ));
                                                #[cfg(test)]
                                                CONTROLS.with(|controls| {
                                                    controls.borrow_mut().insert(
                                                        format!("bp-bool:{}:{}", node.id, pin.pin),
                                                        [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5],
                                                    );
                                                });
                                            }
                                        } else {
                                            let constant = literal_display(value);
                                            let w =
                                                ui.calc_text_size(&constant)[0] + 8. * self.zoom;
                                            if a[0] + w < max[0] - 16. * self.zoom {
                                                draw.add_rect(
                                                    a,
                                                    [a[0] + w, a[1] + 15. * self.zoom],
                                                    [0.35, 0.36, 0.35, 1.],
                                                )
                                                .rounding(1.)
                                                .build();
                                                draw.add_text(
                                                    [a[0] + 4. * self.zoom, a[1] + 1.],
                                                    [0.90, 0.90, 0.90, 1.],
                                                    constant,
                                                );
                                            }
                                        }
                                    }
                                }
                                #[cfg(test)]
                                SOCKETS.with(|sockets| {
                                    sockets.borrow_mut().insert(
                                        format!("{}:{}:{}", pin.node, pin.pin, pin.output),
                                        point,
                                    );
                                });
                                sockets.push((pin, point));
                            }
                        }
                        if matches!(node.kind, NodeKind::Entry) && graph.inherits_event() {
                            draw.add_rect(p, max, [0.20, 0.20, 0.20, 0.45])
                                .filled(true)
                                .rounding(4. * self.zoom)
                                .build();
                        }
                    }
                }
                channels.set_current(0);
                for graph in canvas_graphs(doc, self.graph) {
                    for node in &graph.nodes {
                        for (pin, input) in &node.inputs {
                            if let Some((source, source_pin)) =
                                input_connection(&graph.entry, input)
                                && let Some(wire) =
                                    paint_wire(&draw, &sockets, source, source_pin, &node.id, pin)
                            {
                                wires.push(wire);
                            }
                        }
                        for (pin, targets) in &node.outputs {
                            for target in targets {
                                if let Some(wire) =
                                    paint_wire(&draw, &sockets, &node.id, pin, target, "exec")
                                {
                                    wires.push(wire);
                                }
                            }
                        }
                    }
                }
                if let Some(wire) = &self.wire
                    && let Some((_, p)) = sockets.iter().find(|(s, _)| {
                        s.node == wire.node && s.pin == wire.pin && s.output == wire.output
                    })
                {
                    // The live wire and accepting socket sit above node bodies.
                    channels.set_current(2);
                    let menu_open = self.action_menu.is_open();
                    let target = hit_socket(&sockets, mouse, self.zoom).filter(|(socket, _)| {
                        !menu_open && self.can_connect(wire, socket, registry)
                    });
                    let endpoint = if menu_open {
                        self.catalog_position.map_or(mouse, |p| {
                            [
                                origin[0] + self.pan[0] + p[0] * self.zoom,
                                origin[1] + self.pan[1] + p[1] * self.zoom,
                            ]
                        })
                    } else {
                        target.map_or(mouse, |(_, point)| *point)
                    };
                    let (a, b) = if wire.output {
                        (*p, endpoint)
                    } else {
                        (endpoint, *p)
                    };
                    let d = ((b[0] - a[0]).abs() * 0.5).max(30.);
                    draw.add_bezier_curve(
                        a,
                        [a[0] + d, a[1]],
                        [b[0] - d, b[1]],
                        b,
                        socket_color(&wire.ty),
                    )
                    .thickness(2.5)
                    .build();
                    if let Some((socket, point)) = target {
                        let mut glow = socket_color(&socket.ty);
                        glow[3] = 0.3;
                        draw.add_circle(*point, 10. * self.zoom, glow)
                            .filled(true)
                            .build();
                        draw.add_circle(*point, 8. * self.zoom, socket_color(&socket.ty))
                            .thickness(2.)
                            .build();
                        paint_socket(&draw, *point, &socket.ty, true, self.zoom);
                        paint_socket(&draw, *p, &wire.ty, true, self.zoom);
                        #[cfg(test)]
                        CONTROLS.with(|c| {
                            c.borrow_mut()
                                .insert("bp-connection-preview".into(), *point);
                        });
                    }
                }
            });
        });
        ui.set_window_font_scale(1.);
        if hovered && ui.is_mouse_clicked(MouseButton::Right) {
            if let Some((socket, _)) = hit_socket(&sockets, mouse, self.zoom) {
                self.select_node_graph(&socket.node);
                self.pin_menu = Some(socket.clone());
                self.wire = None;
                self.wire_dragging = false;
                ui.open_popup("Blueprint Pin");
            } else if let Some((node, _, _)) =
                boxes.iter().rev().find(|(_, a, b)| inside(mouse, *a, *b))
            {
                self.select_node_graph(node);
                self.selected = BTreeSet::from([node.clone()]);
                self.node_menu = Some(node.clone());
                self.wire = None;
                ui.open_popup("Blueprint Node");
            } else {
                self.catalog_position = Some([
                    (mouse[0] - origin[0] - self.pan[0]) / self.zoom,
                    (mouse[1] - origin[1] - self.pan[1]) / self.zoom,
                ]);
                self.action_menu.open(ui);
            }
        }
        self.pin_popup(ui, registry);
        if let Some(_popup) = ui.begin_popup("Blueprint Node") {
            let entry = self
                .node_menu
                .clone()
                .filter(|key| self.current().is_some_and(|graph| graph.entry == *key));
            if let Some(entry) = entry {
                let available = self
                    .asset
                    .as_ref()
                    .zip(self.current())
                    .is_some_and(|(doc, graph)| can_call_parent(doc, graph, registry));
                let _disabled = ui.begin_disabled(!available);
                if button(ui, "Add Call to Parent Function") {
                    self.add_parent_call(&entry, registry);
                    ui.close_current_popup();
                }
            }
            if button(ui, "Copy") {
                self.copy_to_clipboard(ui);
                ui.close_current_popup();
            }
            if button(ui, "Delete") {
                self.delete_selected();
                ui.close_current_popup();
            }
        }
        let canvas_max = [origin[0] + size[0], origin[1] + size[1]];
        numeric_inputs
            .retain(|f| inside(f.min, origin, canvas_max) && inside(f.max, origin, canvas_max));
        let editing_number = self.inline_edit.is_some();
        ui.set_window_font_scale(0.85 * self.zoom);
        self.draw_inline_edit(ui, &numeric_inputs);
        ui.set_window_font_scale(1.);
        if hovered && !editing_number && ui.is_mouse_clicked(MouseButton::Left) {
            if ui.is_mouse_double_clicked(MouseButton::Left)
                && !boxes.iter().any(|(_, a, b)| inside(mouse, *a, *b))
                && let Some(wire) = wires.iter().rev().find(|wire| wire.distance(mouse) <= 6.)
            {
                self.select_node_graph(&wire.from.node);
                self.insert_reroute(
                    wire,
                    [
                        (mouse[0] - origin[0] - self.pan[0]) / self.zoom,
                        (mouse[1] - origin[1] - self.pan[1]) / self.zoom,
                    ],
                );
            } else if let Some(field) = numeric_inputs
                .iter()
                .rev()
                .find(|f| inside(mouse, f.min, f.max))
            {
                self.begin_inline_edit(field);
            } else if let Some((node, pin, _, _)) =
                boolean_inputs.iter().rev().find(|(_, _, a, b)| {
                    mouse[0] >= a[0] && mouse[0] <= b[0] && mouse[1] >= a[1] && mouse[1] <= b[1]
                })
            {
                self.select_node_graph(node);
                if let Some(Input::Literal { value, .. }) = self
                    .asset
                    .as_mut()
                    .and_then(|doc| doc.functions.get_mut(self.graph))
                    .and_then(|graph| graph.nodes.iter_mut().find(|item| item.id == *node))
                    .and_then(|node| node.inputs.get_mut(pin))
                    && let Some(checked) = value.as_bool()
                {
                    *value = Value::Bool(!checked);
                }
                // The enclosing draw transaction records exactly one undo step.
                // Clicking an inline value must not start a node drag or a wire.
                self.drag = None;
                self.drag_before = None;
            } else if let Some((socket, _)) =
                hit_socket(&sockets, mouse, self.zoom).filter(|(socket, _)| {
                    let center_drag = self.wire.is_none()
                        && !ui.io().key_alt
                        && boxes.iter().any(|(key, a, b)| {
                            *key == socket.node
                                && (b[0] - a[0] - 40. * self.zoom).abs() < 0.01
                                && (mouse[0] - (a[0] + b[0]) * 0.5).abs() < 7. * self.zoom
                                && (mouse[1] - (a[1] + b[1]) * 0.5).abs() < 7. * self.zoom
                        });
                    !center_drag
                })
            {
                self.select_node_graph(&socket.node);
                if ui.io().key_alt {
                    self.disconnect(socket);
                    self.wire = None;
                } else if let Some(previous) = self.wire.take() {
                    if let Err(e) = self.connect(&previous, socket, registry) {
                        self.error = Some(e);
                    }
                } else {
                    self.wire = Some(socket.clone());
                    self.wire_dragging = false;
                }
            } else if let Some((node, _, _)) = boxes.iter().rev().find(|(_, a, b)| {
                mouse[0] >= a[0] && mouse[0] <= b[0] && mouse[1] >= a[1] && mouse[1] <= b[1]
            }) {
                self.select_node_graph(node);
                if ui.io().key_ctrl {
                    if !self.selected.insert(node.clone()) {
                        self.selected.remove(node);
                    }
                } else if !self.selected.contains(node) {
                    self.selected = BTreeSet::from([node.clone()]);
                }
                self.drag = Some(node.clone());
                self.details = Details::Node;
                self.drag_before = self.asset.clone();
            } else {
                self.selected.clear();
                self.wire = None;
            }
        }
        if self.wire.is_some() && ui.is_mouse_dragging(MouseButton::Left) {
            self.wire_dragging = true;
        }
        if self.wire_dragging && ui.is_mouse_released(MouseButton::Left) {
            self.wire_dragging = false;
            if hovered {
                if let Some((socket, _)) = hit_socket(&sockets, mouse, self.zoom) {
                    if let Some(previous) = self.wire.take()
                        && let Err(error) = self.connect(&previous, socket, registry)
                    {
                        self.error = Some(error);
                    }
                } else if !boxes.iter().any(|(_, a, b)| inside(mouse, *a, *b)) {
                    self.catalog_position = Some([
                        (mouse[0] - origin[0] - self.pan[0]) / self.zoom,
                        (mouse[1] - origin[1] - self.pan[1]) / self.zoom,
                    ]);
                    self.action_menu.context_sensitive = true;
                    self.action_menu.open(ui);
                } else {
                    self.wire = None;
                }
            } else {
                self.wire = None;
            }
        }
        if self.drag.is_some() && ui.is_mouse_dragging(MouseButton::Left) {
            let doc = self.asset.as_mut().unwrap();
            for key in &self.selected {
                let initial = node_position(doc, &doc.functions[self.graph], key);
                let p = doc.layout.positions.entry(key.clone()).or_insert(initial);
                p[0] += ui.io().mouse_delta[0] / self.zoom;
                p[1] += ui.io().mouse_delta[1] / self.zoom;
            }
        }
        if self.drag.is_some() && !ui.is_mouse_down(MouseButton::Left) {
            self.drag = None;
            if let Some(before) = self.drag_before.take() {
                self.checkpoint(before);
            }
        }
    }

    fn disconnect(&mut self, socket: &Socket) {
        if let Some(graph) = self
            .asset
            .as_mut()
            .and_then(|d| d.functions.get_mut(self.graph))
        {
            if socket.output {
                if socket.ty == SocketType::Exec {
                    if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == socket.node) {
                        node.outputs.remove(&socket.pin);
                    }
                } else {
                    let entry = graph.entry.clone();
                    for node in &mut graph.nodes {
                        node.inputs.retain(|_, input| {
                            input_connection(&entry, input)
                                != Some((socket.node.as_str(), socket.pin.as_str()))
                        });
                    }
                }
            } else if socket.ty == SocketType::Exec {
                for node in &mut graph.nodes {
                    for targets in node.outputs.values_mut() {
                        targets.retain(|n| n != &socket.node);
                    }
                }
            } else if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == socket.node) {
                node.inputs.remove(&socket.pin);
            }
        }
    }
    fn insert_reroute(&mut self, wire: &CanvasWire, point: [f32; 2]) {
        let Some(doc) = self.asset.as_mut() else {
            return;
        };
        let Some(graph) = doc.functions.get_mut(self.graph) else {
            return;
        };
        let key = id();
        let mut knot = Node {
            id: key.clone(),
            kind: NodeKind::Reroute,
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        };
        if wire.from.ty == SocketType::Exec {
            let Some(targets) = graph
                .nodes
                .iter_mut()
                .find(|n| n.id == wire.from.node)
                .and_then(|n| n.outputs.get_mut(&wire.from.pin))
            else {
                return;
            };
            let Some(target) = targets.iter_mut().find(|n| **n == wire.to.node) else {
                return;
            };
            *target = key.clone();
            knot.outputs
                .insert("next".into(), vec![wire.to.node.clone()]);
        } else {
            let Some(input) = graph
                .nodes
                .iter_mut()
                .find(|n| n.id == wire.to.node)
                .and_then(|n| n.inputs.get_mut(&wire.to.pin))
            else {
                return;
            };
            knot.inputs.insert("value".into(), input.clone());
            *input = Input::Link {
                node: key.clone(),
                pin: "value".into(),
            };
        }
        graph.nodes.push(knot);
        doc.layout
            .positions
            .insert(key.clone(), [point[0] - 20., point[1] - 12.]);
        self.selected = BTreeSet::from([key]);
        self.details = Details::Node;
        self.wire = None;
        self.drag = None;
        self.drag_before = None;
        // The enclosing draw transaction owns the entire split as one undo step.
    }
    fn connect(&mut self, a: &Socket, b: &Socket, registry: &Registry) -> Result<(), String> {
        let before = self.asset.clone();
        let selected_graph = self.graph;
        let result = self
            .join_event_island(a, b)
            .and_then(|()| self.connect_scoped(a, b, registry));
        if result.is_err() {
            self.asset = before;
            self.graph = selected_graph;
        }
        result
    }
    fn join_event_island(&mut self, a: &Socket, b: &Socket) -> Result<(), String> {
        let doc = self.asset.as_mut().ok_or("No Blueprint")?;
        let owner = |key: &str| {
            doc.functions
                .iter()
                .position(|g| g.nodes.iter().any(|n| n.id == key))
        };
        let from = owner(&a.node).ok_or("Missing source node")?;
        let to = owner(&b.node).ok_or("Missing target node")?;
        if from == to {
            self.graph = from;
            return Ok(());
        }
        if doc.functions[from].override_id.is_none() || doc.functions[to].override_id.is_none() {
            return Err("Use a function call to connect separate function graphs.".into());
        }
        // Nodes placed in the shared event canvas acquire the event's scope
        // when connected. Existing event chains keep their own parameters.
        for (source, destination, key) in [(to, from, &b.node), (from, to, &a.node)] {
            let graph = &doc.functions[source];
            let mut component = BTreeSet::from([key.clone()]);
            loop {
                let count = component.len();
                for node in &graph.nodes {
                    for peer in node.outputs.values().flatten().map(String::as_str).chain(
                        node.inputs.values().filter_map(|input| {
                            input_connection(&graph.entry, input).map(|(node, _)| node)
                        }),
                    ) {
                        if component.contains(&node.id) || component.contains(peer) {
                            component.insert(node.id.clone());
                            component.insert(peer.to_owned());
                        }
                    }
                }
                if component.len() == count {
                    break;
                }
            }
            if component.contains(&graph.entry) {
                continue;
            }
            let moving: Vec<_> = graph
                .nodes
                .iter()
                .filter(|n| component.contains(&n.id))
                .cloned()
                .collect();
            doc.functions[source]
                .nodes
                .retain(|n| !component.contains(&n.id));
            doc.functions[destination].nodes.extend(moving);
            self.graph = destination;
            return Ok(());
        }
        Err(
            "These nodes belong to different events. Use a function to share logic between events."
                .into(),
        )
    }
    fn connect_scoped(
        &mut self,
        a: &Socket,
        b: &Socket,
        registry: &Registry,
    ) -> Result<(), String> {
        let (from, to) = if a.output { (a, b) } else { (b, a) };
        if !from.output || to.output {
            return Err("Connect an output socket to an input socket.".into());
        }
        let doc = self.asset.as_ref().ok_or("No Blueprint")?;
        let graph = doc.functions.get(self.graph).ok_or("No graph")?;
        if ![from, to]
            .iter()
            .all(|s| graph.nodes.iter().any(|n| n.id == s.node))
        {
            return Err("Events have separate parameters and execution scopes. Use a function to share logic between events.".into());
        }
        if !compatible(&from.ty, &to.ty, registry)
            && !graph
                .nodes
                .iter()
                .any(|n| n.id == to.node && flexible_input(n, &to.pin, &from.ty))
        {
            return Err(
                "Pin types differ. Add an explicit operation; implicit conversion is not allowed."
                    .into(),
            );
        }
        if from.node == to.node {
            return Err("A node cannot connect to itself.".into());
        }
        if from.ty == SocketType::Exec
            && graph
                .nodes
                .iter()
                .any(|node| matches!(node.kind, NodeKind::Reroute) && node.id == to.node)
            && graph.nodes.iter().any(|node| {
                node.inputs.values().any(|input| {
                    input_connection(&graph.entry, input)
                        .is_some_and(|(source, _)| source == to.node)
                })
            })
        {
            return Err(
                "Disconnect the reroute's data wires before using it for execution.".into(),
            );
        }
        for socket in [from, to] {
            if !graph.nodes.iter().any(|n| {
                node_sockets_with_assets(
                    doc,
                    graph,
                    n,
                    registry,
                    &self.playback_timelines,
                    &self.playback_effects,
                )
                .iter()
                .any(|p| {
                    p.node == socket.node
                        && p.pin == socket.pin
                        && p.output == socket.output
                        && p.ty == socket.ty
                })
            }) {
                return Err("Socket changed; reconnect using its current signature.".into());
            }
        }
        if reachable(
            graph,
            &to.node,
            &from.node,
            from.ty == SocketType::Exec,
            &mut BTreeSet::new(),
        ) {
            return Err("Connection creates a cycle. Use a bounded Loop node.".into());
        }
        let graph = &mut self.asset.as_mut().unwrap().functions[self.graph];
        if from.ty == SocketType::Exec {
            if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == to.node)
                && matches!(node.kind, NodeKind::Reroute)
            {
                node.outputs.entry("next".into()).or_default();
            }
            let node = graph.nodes.iter_mut().find(|n| n.id == from.node).unwrap();
            node.outputs.insert(from.pin.clone(), vec![to.node.clone()]);
        } else {
            let node = graph.nodes.iter_mut().find(|n| n.id == to.node).unwrap();
            if matches!(
                node.inputs.get(&to.pin),
                Some(Input::Link { .. } | Input::Parameter { .. })
            ) {
                return Err(
                    "Input is already connected. Alt-click it to disconnect explicitly.".into(),
                );
            }
            node.inputs.insert(
                to.pin.clone(),
                Input::Link {
                    node: from.node.clone(),
                    pin: from.pin.clone(),
                },
            );
        }
        self.error = None;
        Ok(())
    }

    fn catalog(&mut self, ui: &Ui, registry: &Registry) {
        if self.catalog_position.is_some() && !self.action_menu.is_open() {
            self.catalog_position = None;
            self.wire = None;
        }
        let mut menu = std::mem::take(&mut self.action_menu);
        let chosen = menu.draw(
            ui,
            |context| self.catalog_actions(registry, context),
            self.playback_error.as_deref(),
        );
        self.action_menu = menu;
        if let Some(kind) = chosen {
            self.add_catalog_node(kind, self.wire.clone().as_ref());
            if let Some(point) = self.catalog_position.take()
                && let Some(key) = self.selected.iter().next()
                && let Some(doc) = self.asset.as_mut()
            {
                doc.layout.positions.insert(key.clone(), point);
            }
            if let Some(previous) = self.wire.take() {
                let doc = self.asset.as_ref().unwrap();
                let graph = self.current().unwrap();
                let node = graph.nodes.last().unwrap();
                if let Some(socket) = self.accepting_socket(doc, graph, node, &previous, registry)
                    && let Err(error) = self.connect(&previous, &socket, registry)
                {
                    self.error = Some(error);
                }
            }
        }
    }

    fn add_catalog_node(&mut self, kind: NodeKind, wire: Option<&Socket>) {
        let reroute = matches!(kind, NodeKind::Reroute);
        self.add_node(kind);
        // A reroute created upstream needs its type before its first input wire.
        if reroute
            && let Some(wire) = wire.filter(|w| !w.output)
            && let Some(node) = self
                .asset
                .as_mut()
                .and_then(|d| d.functions.get_mut(self.graph))
                .and_then(|g| g.nodes.last_mut())
        {
            match &wire.ty {
                SocketType::Value(ty) => {
                    node.inputs.insert(
                        "value".into(),
                        Input::Literal {
                            value_type: ty.clone(),
                            value: default_value(ty),
                        },
                    );
                }
                SocketType::Exec => {
                    node.outputs.insert("next".into(), vec![]);
                }
            }
        }
    }

    /// The menu and automatic wiring use the same validation as pin hover/drop.
    fn accepting_socket(
        &self,
        doc: &BlueprintAsset,
        graph: &Graph,
        node: &Node,
        wire: &Socket,
        registry: &Registry,
    ) -> Option<Socket> {
        let mut sockets = node_sockets_with_assets(
            doc,
            graph,
            node,
            registry,
            &self.playback_timelines,
            &self.playback_effects,
        );
        // Prefer an exact type over a conversion or a wildcard input.
        sockets.sort_by_key(|s| s.ty != wire.ty);
        sockets
            .into_iter()
            .find(|s| self.can_connect(wire, s, registry))
    }

    fn can_create_connected(&self, kind: &NodeKind, wire: &Socket, registry: &Registry) -> bool {
        let mut probe = BlueprintEditor {
            asset: self.asset.clone(),
            graph: self.graph,
            playback_timelines: self.playback_timelines.clone(),
            playback_effects: self.playback_effects.clone(),
            ..Default::default()
        };
        probe.add_catalog_node(kind.clone(), Some(wire));
        let Some(doc) = probe.asset.as_ref() else {
            return false;
        };
        let Some(graph) = probe.current() else {
            return false;
        };
        let Some(node) = graph.nodes.last() else {
            return false;
        };
        probe
            .accepting_socket(doc, graph, node, wire, registry)
            .is_some()
    }

    fn catalog_actions(
        &self,
        registry: &Registry,
        context_sensitive: bool,
    ) -> Vec<action_menu::Action> {
        let mut candidates = vec![
            ("Flow / Branch".into(), NodeKind::Branch),
            ("Flow / Sequence".into(), NodeKind::Sequence),
            ("Flow / Bounded loop".into(), NodeKind::Loop { count: 10 }),
            ("Flow / Return".into(), NodeKind::Return),
            ("Flow / Delay".into(), NodeKind::Delay),
            (
                "Sequence / Wait for completion".into(),
                NodeKind::WaitPlayback {
                    condition: asset::PlaybackCondition::SequenceComplete,
                },
            ),
            (
                "Effect / Wait for completion".into(),
                NodeKind::WaitPlayback {
                    condition: asset::PlaybackCondition::EffectComplete,
                },
            ),
            ("Inheritance / Call Parent".into(), NodeKind::CallParent),
            ("Logic / Not".into(), NodeKind::Not),
            ("Layout / Reroute".into(), NodeKind::Reroute),
        ];
        for (_, timeline) in &self.playback_timelines {
            for marker in &timeline.markers {
                candidates.push((
                    format!(
                        "Sequence / Subscribe to marker / {} / {}",
                        timeline.name, marker.name
                    ),
                    NodeKind::WaitPlayback {
                        condition: asset::PlaybackCondition::SubscribeMarker {
                            timeline: timeline.id.to_string(),
                            marker: marker.id.to_string(),
                        },
                    },
                ));
                candidates.push((
                    format!(
                        "Sequence / Wait for marker / {} / {}",
                        timeline.name, marker.name
                    ),
                    NodeKind::WaitPlayback {
                        condition: asset::PlaybackCondition::Marker {
                            timeline: timeline.id.to_string(),
                            marker: marker.id.to_string(),
                        },
                    },
                ));
            }
        }
        for (path, timeline) in &self.playback_timelines {
            if path.to_string_lossy().ends_with(".timeline.json") {
                candidates.push((
                    format!("Sequence / Play asset / {}", timeline.name),
                    NodeKind::Builtin {
                        operation: asset::Builtin::PlayTimelineAsset {
                            asset: timeline.id.to_string(),
                        },
                    },
                ));
            }
        }
        for (_, effect) in &self.playback_effects {
            candidates.push((
                format!("Effect / Spawn asset / {}", effect.name),
                NodeKind::Builtin {
                    operation: asset::Builtin::SpawnParticleEffect {
                        asset: effect.id.to_string(),
                    },
                },
            ));
        }
        for mut ty in authoring_types(registry) {
            if let schema::Type::ClassRef { base } = &mut ty {
                *base = self.asset.as_ref().unwrap().parent.clone();
            }
            candidates.push((
                format!("Values / {} literal", ty.label()),
                NodeKind::Literal {
                    value: default_value(&ty),
                    value_type: ty,
                },
            ));
        }
        for (name, op) in [
            ("Add", asset::BinaryOp::Add),
            ("Subtract", asset::BinaryOp::Subtract),
            ("Multiply", asset::BinaryOp::Multiply),
            ("Divide", asset::BinaryOp::Divide),
            ("Equal", asset::BinaryOp::Equal),
            ("Not equal", asset::BinaryOp::NotEqual),
            ("Less", asset::BinaryOp::Less),
            ("Less or equal", asset::BinaryOp::LessEqual),
            ("Greater", asset::BinaryOp::Greater),
            ("Greater or equal", asset::BinaryOp::GreaterEqual),
            ("And", asset::BinaryOp::And),
            ("Or", asset::BinaryOp::Or),
        ] {
            candidates.push((format!("Operators / {name}"), NodeKind::Binary { op }));
        }
        let doc = self.asset.as_ref().unwrap();
        for length in [2, 3] {
            candidates.push((
                format!("Vector / Make Vector{length}"),
                NodeKind::MakeVector { length },
            ));
            for index in 0..length {
                candidates.push((
                    format!("Vector / Vector{length} {}", ["X", "Y", "Z"][index]),
                    NodeKind::VectorComponent { length, index },
                ));
            }
        }
        for (name, operation) in [
            ("Actor / Self", asset::Builtin::SelfObject),
            ("Actor / Is valid", asset::Builtin::IsValid),
            ("Transform 3D / Get position", asset::Builtin::GetPosition),
            ("World2D / Get position 2d", asset::Builtin::GetPosition2D),
            ("World2D / Set position 2d", asset::Builtin::SetPosition2D),
            ("World2D / Get rotation 2d", asset::Builtin::GetRotation2D),
            ("World2D / Set rotation 2d", asset::Builtin::SetRotation2D),
            ("World2D / Get scale 2d", asset::Builtin::GetScale2D),
            ("World2D / Set scale 2d", asset::Builtin::SetScale2D),
            ("UI / Get rect position", asset::Builtin::GetRectPosition),
            ("UI / Set rect position", asset::Builtin::SetRectPosition),
            ("UI / Get rect size", asset::Builtin::GetRectSize),
            ("UI / Set rect size", asset::Builtin::SetRectSize),
            ("Transform / Get rotation", asset::Builtin::GetRotation),
            ("Transform / Get scale", asset::Builtin::GetScale),
            ("Transform / Get transform", asset::Builtin::GetTransform),
            ("Transform / Make transform", asset::Builtin::MakeTransform),
            ("Transform / Set position", asset::Builtin::SetPosition),
            ("Transform / Set rotation", asset::Builtin::SetRotation),
            ("Transform / Set scale", asset::Builtin::SetScale),
            ("Input / Button held", asset::Builtin::InputHeld),
            ("Input / Button pressed", asset::Builtin::InputPressed),
            ("Input / Button released", asset::Builtin::InputReleased),
            ("Scene / Request scene", asset::Builtin::RequestScene),
            ("Actor / Set active", asset::Builtin::SetActive),
            ("Actor / Destroy", asset::Builtin::DestroyActor),
            ("Audio / Play", asset::Builtin::PlayAudio),
            ("Audio / Stop", asset::Builtin::StopAudio),
            ("Audio / Set clip", asset::Builtin::SetAudioClip),
            ("Material / Set texture", asset::Builtin::SetTexture),
            (
                "Sequence / Play component",
                asset::Builtin::PlaySequenceComponent,
            ),
            ("Sequence / Stop", asset::Builtin::StopSequence),
            ("Sequence / Pause", asset::Builtin::PauseSequence),
            ("Sequence / Resume", asset::Builtin::ResumeSequence),
            (
                "Effect / Play component",
                asset::Builtin::PlayEffectComponent,
            ),
            ("Effect / Stop", asset::Builtin::StopEffect),
            (
                "Effect / Burst enabled emitter layers",
                asset::Builtin::BurstEffect,
            ),
            ("Effect / Pause", asset::Builtin::PauseEffect),
            ("Effect / Resume", asset::Builtin::ResumeEffect),
            ("Effect / Sequence handle", asset::Builtin::EffectSequence),
        ] {
            candidates.push((name.into(), NodeKind::Builtin { operation }));
        }
        for operation in registry.operations.values() {
            candidates.push((
                format!(
                    "Gameplay API / {} / {}",
                    operation.category,
                    display_port(&operation.name)
                ),
                NodeKind::Operation {
                    operation: operation.id.clone(),
                },
            ));
        }
        for class in registry.classes.values() {
            candidates.push((
                format!("Spawn / Class reference ({})", class.cpp_name),
                NodeKind::Builtin {
                    operation: asset::Builtin::SpawnClass {
                        base: class.id.clone(),
                    },
                },
            ));
            for function in target_functions(registry, &class.id) {
                candidates.push((
                    format!(
                        "Functions / {} / {}",
                        class.cpp_name,
                        function_label(&function)
                    ),
                    NodeKind::CallOn {
                        class: class.id.clone(),
                        function: function.id,
                    },
                ));
            }
            candidates.push((
                format!("Actor / Cast to {}", class.cpp_name),
                NodeKind::Builtin {
                    operation: asset::Builtin::Cast {
                        class: class.id.clone(),
                    },
                },
            ));
            candidates.push((
                format!("Actor / Is A {}", class.cpp_name),
                NodeKind::Builtin {
                    operation: asset::Builtin::IsA {
                        class: class.id.clone(),
                    },
                },
            ));
            if class.backend.id == "native" && !class.abstract_class {
                candidates.push((
                    format!("Spawn / {}", class.cpp_name),
                    NodeKind::Builtin {
                        operation: asset::Builtin::Spawn {
                            class: class.id.clone(),
                        },
                    },
                ));
            }
        }
        for property in properties(doc, registry) {
            if property.value_type == schema::Type::Fixed && property.editable {
                candidates.push((
                    format!("Timeline / {}", property.name),
                    NodeKind::Timeline {
                        keys: vec![[0., 0.], [1., 1.]],
                        looping: false,
                        member: property.id.clone(),
                    },
                ));
            }
            candidates.push((
                format!("Get / {}", property.name),
                NodeKind::GetVariable {
                    member: property.id.clone(),
                },
            ));
            if property.editable {
                candidates.push((
                    format!("Set / {}", property.name),
                    NodeKind::SetVariable {
                        member: property.id,
                    },
                ));
            }
        }
        for variable in &doc.variables {
            if variable.value_type == schema::Type::Fixed && variable.editable {
                candidates.push((
                    format!("Timeline / {}", variable.name),
                    NodeKind::Timeline {
                        keys: vec![[0., 0.], [1., 1.]],
                        looping: false,
                        member: variable.id.clone(),
                    },
                ));
            }
            candidates.push((
                format!("Get / {}", variable.name),
                NodeKind::GetVariable {
                    member: variable.id.clone(),
                },
            ));
            candidates.push((
                format!("Set / {}", variable.name),
                NodeKind::SetVariable {
                    member: variable.id.clone(),
                },
            ));
        }
        for function in functions(doc, registry) {
            if function.callable && function.access == "public" {
                candidates.push((
                    format!("Functions / Self / {}", function_label(&function)),
                    NodeKind::Call {
                        function: function.id,
                    },
                ));
            }
        }
        for graph in doc
            .functions
            .iter()
            .filter(|graph| graph.override_id.is_none())
        {
            candidates.push((
                format!("Functions / {} / {}", doc.name, graph.name),
                NodeKind::Call {
                    function: graph.id.clone(),
                },
            ));
        }
        if let Some(graph) = self.current() {
            for node in &graph.nodes {
                if matches!(node.kind, NodeKind::Timeline { .. }) {
                    candidates.push((
                        format!("Timeline / Stop {}", node.id),
                        NodeKind::StopTimeline {
                            node: node.id.clone(),
                        },
                    ));
                }
            }
        }
        let mut actions = Vec::new();
        for (label, kind) in candidates {
            let dummy = Node {
                id: "catalog".into(),
                kind: kind.clone(),
                inputs: BTreeMap::new(),
                outputs: BTreeMap::new(),
            };
            let sockets = self
                .current()
                .map(|graph| {
                    node_sockets_with_assets(
                        self.asset.as_ref().unwrap(),
                        graph,
                        &dummy,
                        registry,
                        &self.playback_timelines,
                        &self.playback_effects,
                    )
                })
                .unwrap_or_default();
            if context_sensitive
                && let Some(wire) = &self.wire
                && !self.can_create_connected(&kind, wire, registry)
            {
                continue;
            }
            let pure = !sockets.iter().any(|socket| socket.ty == SocketType::Exec);
            let color = if pure {
                [0.44, 0.72, 0.36, 1.]
            } else {
                [0.36, 0.65, 0.85, 1.]
            };
            // Use the existing licensed Codicons font; no external artwork is bundled.
            let (icon, color) = match &kind {
                NodeKind::Literal { value_type, .. } => (
                    "\u{eb5d}",
                    socket_color(&SocketType::Value(value_type.clone())),
                ),
                NodeKind::GetVariable { .. } | NodeKind::SetVariable { .. } => (
                    "\u{ea88}",
                    sockets
                        .iter()
                        .find(|s| s.ty != SocketType::Exec)
                        .map_or(color, |s| socket_color(&s.ty)),
                ),
                NodeKind::Binary { .. } | NodeKind::Not => ("\u{eb64}", color),
                NodeKind::Branch | NodeKind::Sequence | NodeKind::Loop { .. } => {
                    ("\u{ea68}", [0.8, 0.8, 0.8, 1.])
                }
                NodeKind::Delay
                | NodeKind::WaitPlayback { .. }
                | NodeKind::Timeline { .. }
                | NodeKind::StopTimeline { .. } => ("\u{ea82}", color),
                NodeKind::Reroute | NodeKind::Return => ("\u{ead0}", [0.8, 0.8, 0.8, 1.]),
                _ => ("\u{e900}", color),
            };
            let tooltip = label.split("##").next().unwrap_or(&label).to_string();
            let (path, leaf) = tooltip.rsplit_once(" / ").unwrap_or(("", &tooltip));
            let leaf = if matches!(kind, NodeKind::Call { .. } | NodeKind::CallOn { .. }) {
                leaf.split('(').next().unwrap_or(leaf).trim()
            } else {
                leaf
            };
            let display = match &kind {
                NodeKind::Literal {
                    value_type: schema::Type::ClassRef { base },
                    ..
                } => format!(
                    "{} Class Reference",
                    registry
                        .classes
                        .get(base)
                        .map_or("Blueprint", |c| c.cpp_name.as_str())
                ),
                _ => display_port(leaf),
            };
            actions.push(action_menu::Action {
                path: path.into(),
                label: display,
                tooltip,
                kind,
                icon,
                color,
            });
        }
        actions
    }

    fn node_inspector(&mut self, ui: &Ui, registry: &Registry) {
        let Some(key) = self.selected.iter().next().cloned() else {
            ui.text_disabled("Select a node on the canvas.");
            return;
        };
        if button(ui, "Copy") {
            self.copy_to_clipboard(ui);
        }
        ui.same_line();
        if button(ui, "Duplicate") {
            self.copy_selected();
            self.paste();
        }
        if button(ui, "Delete selected") {
            self.delete_selected();
            return;
        }
        let doc = self.asset.as_ref().unwrap();
        let Some(graph) = doc.functions.get(self.graph) else {
            return;
        };
        let Some(node) = graph.nodes.iter().find(|n| n.id == key) else {
            return;
        };
        let sockets = node_sockets_with_assets(
            doc,
            graph,
            node,
            registry,
            &self.playback_timelines,
            &self.playback_effects,
        );
        let parameters = graph.parameters.clone();
        let parameter_labels: BTreeMap<_, _> = parameters
            .iter()
            .map(|p| (p.name.clone(), event_parameter_label(graph, &p.name)))
            .collect();
        let timelines: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Timeline { .. }))
            .map(|n| n.id.clone())
            .collect();
        let mut fixed_members: Vec<_> = properties(doc, registry)
            .into_iter()
            .filter(|p| p.value_type == schema::Type::Fixed && p.editable)
            .map(|p| (p.id, p.name))
            .collect();
        fixed_members.extend(
            doc.variables
                .iter()
                .filter(|v| v.value_type == schema::Type::Fixed && v.editable)
                .map(|v| (v.id.clone(), v.name.clone())),
        );
        let mut members: Vec<_> = properties(doc, registry)
            .into_iter()
            .map(|p| (p.id, p.name, p.editable))
            .collect();
        members.extend(
            doc.variables
                .iter()
                .map(|v| (v.id.clone(), v.name.clone(), v.editable)),
        );
        let mut calls: Vec<_> = functions(doc, registry)
            .into_iter()
            .filter(|f| f.callable && f.access == "public")
            .map(|f| (f.id, f.name))
            .collect();
        calls.extend(doc.functions.iter().map(|g| (g.id.clone(), g.name.clone())));
        self.comment = doc.layout.comments.get(&key).cloned().unwrap_or_default();
        if ui.input_text("Comment", &mut self.comment).build() {
            self.asset
                .as_mut()
                .unwrap()
                .layout
                .comments
                .insert(key.clone(), self.comment.clone());
        }
        let doc = self.asset.as_mut().unwrap();
        let node = doc.functions[self.graph]
            .nodes
            .iter_mut()
            .find(|n| n.id == key)
            .unwrap();
        ui.text_disabled(format!("Node: {}", node.id));
        match &mut node.kind {
            NodeKind::Timeline {
                keys,
                looping,
                member,
            } => {
                let current = fixed_members
                    .iter()
                    .find(|(id, _)| id == member)
                    .map(|(_, name)| name.as_str())
                    .unwrap_or("Missing Fixed member");
                if let Some(_combo) = ui.begin_combo("Animated member", current) {
                    for (id, name) in &fixed_members {
                        if ui.selectable(name) {
                            *member = id.clone();
                        }
                    }
                }
                ui.checkbox("Looping", looping);
                timeline_curve(ui, keys, &mut self.timeline_key);
                let mut remove = None;
                for (index, key) in keys.iter_mut().enumerate() {
                    let _id = ui.push_id_usize(index);
                    crate::gui::Drag::new("Time (seconds)")
                        .speed(0.1)
                        .build(ui, &mut key[0]);
                    crate::gui::Drag::new("Value (Q12)")
                        .speed(0.1)
                        .build(ui, &mut key[1]);
                    if ui.small_button("Remove key") {
                        remove = Some(index);
                    }
                }
                if let Some(index) = remove
                    && keys.len() > 2
                {
                    keys.remove(index);
                    self.timeline_key = None;
                }
                if keys.len() < 16 && button(ui, "Add key") {
                    let last = keys.last().copied().unwrap_or([0., 0.]);
                    keys.push([last[0] + 1., last[1]]);
                }
                ui.text_wrapped("Drag curve keys or edit their time/value. First time must be zero; times must increase. Timeline updates this instance's Fixed member in simulation time.");
            }
            NodeKind::StopTimeline { node: target } => {
                if let Some(_combo) = ui.begin_combo("Timeline node", target.clone()) {
                    for id in &timelines {
                        if ui.selectable(id) {
                            *target = id.clone();
                        }
                    }
                }
            }
            NodeKind::Builtin {
                operation: asset::Builtin::PlayTimelineAsset { asset },
            } => {
                let label = self
                    .playback_timelines
                    .iter()
                    .find(|(_, value)| value.id.to_string() == *asset)
                    .map_or("Missing asset (preserved)", |(_, value)| {
                        value.name.as_str()
                    });
                if let Some(_combo) = ui.begin_combo("Timeline asset", label) {
                    for (path, value) in &self.playback_timelines {
                        if path.to_string_lossy().ends_with(".timeline.json")
                            && ui.selectable(format!("{}##{}", value.name, value.id))
                        {
                            *asset = value.id.to_string();
                        }
                    }
                }
                ui.text_wrapped("Binding pins use persistent slot IDs. Replacing the asset preserves existing connections; repair any stale bindings before compiling.");
            }
            NodeKind::Builtin {
                operation: asset::Builtin::SpawnParticleEffect { asset },
            } => {
                let label = self
                    .playback_effects
                    .iter()
                    .find(|(_, value)| value.id.to_string() == *asset)
                    .map_or("Missing asset (preserved)", |(_, value)| {
                        value.name.as_str()
                    });
                if let Some(_combo) = ui.begin_combo("ParticleEffect asset", label) {
                    for (_, value) in &self.playback_effects {
                        if ui.selectable(format!("{}##{}", value.name, value.id)) {
                            *asset = value.id.to_string();
                        }
                    }
                }
                ui.text_wrapped("A null owner creates a detached effect. External bindings retain typed scene handles; the effect pool supplies internal layers. Zero seed uses the asset seed.");
            }
            NodeKind::WaitPlayback { condition } => {
                if let asset::PlaybackCondition::Marker { timeline, marker }
                | asset::PlaybackCondition::SubscribeMarker { timeline, marker } = condition
                {
                    let label = self
                        .playback_timelines
                        .iter()
                        .find(|(_, asset)| asset.id.to_string() == *timeline)
                        .and_then(|(_, asset)| {
                            asset
                                .markers
                                .iter()
                                .find(|m| m.id.to_string() == *marker)
                                .map(|m| format!("{} / {}", asset.name, m.name))
                        })
                        .unwrap_or_else(|| "Missing marker (reference preserved)".into());
                    if let Some(_combo) = ui.begin_combo("Marker", label) {
                        for (_, asset) in &self.playback_timelines {
                            for entry in &asset.markers {
                                if ui.selectable(format!(
                                    "{} / {}##{}",
                                    asset.name, entry.name, entry.id
                                )) {
                                    *timeline = asset.id.to_string();
                                    *marker = entry.id.to_string();
                                }
                            }
                        }
                    }
                }
                if matches!(condition, asset::PlaybackCondition::SubscribeMarker { .. }) {
                    ui.text_wrapped("Next continues immediately. A captured listener frame uses the existing continuation table. Reached runs once per future crossing, at most once per simulation tick; its branch can suspend. Completed or Cancelled runs after pending crossings. Return from a listener branch ends it. Registering this node again replaces its listener.");
                } else {
                    ui.text_wrapped("Suspends this event in its existing continuation slot. Reached runs once for the first marker crossing. Completed runs when playback ends; Cancelled runs if playback is stopped, unavailable, or the wait pool is full.");
                }
                ui.text_wrapped("Destroying this behaviour or replacing the scene cancels its frame without running gameplay.");
            }
            NodeKind::GetVariable { member } | NodeKind::SetVariable { member } => {
                let current = members
                    .iter()
                    .find(|(id, _, _)| id == member)
                    .map(|(_, name, _)| name.as_str())
                    .unwrap_or("Missing member (preserved)");
                if let Some(_combo) = ui.begin_combo("Member", current) {
                    for (id, name, _) in &members {
                        if ui.selectable(name) {
                            *member = id.clone();
                        }
                    }
                }
                ui.text_wrapped("Changing the member preserves all wires. Compile to diagnose incompatible connections.");
            }
            NodeKind::Call { function } => {
                let current = calls
                    .iter()
                    .find(|(id, _)| id == function)
                    .map(|(_, name)| name.as_str())
                    .unwrap_or("Missing function (preserved)");
                if let Some(_combo) = ui.begin_combo("Function", current) {
                    for (id, name) in &calls {
                        if ui.selectable(name) {
                            *function = id.clone();
                        }
                    }
                }
                ui.text_wrapped(
                    "Existing argument IDs and wires remain intact after changing the target.",
                );
            }
            NodeKind::CallOn { class, function } => {
                let label = registry
                    .classes
                    .get(class)
                    .map(|class| class.cpp_name.as_str())
                    .unwrap_or("Missing class (preserved)");
                if let Some(_combo) = ui.begin_combo("Target class", label) {
                    for candidate in registry.classes.values() {
                        if !target_functions(registry, &candidate.id).is_empty()
                            && ui.selectable(&candidate.cpp_name)
                        {
                            *class = candidate.id.clone();
                        }
                    }
                }
                let current = crate::blueprint_ir::call_on_function(registry, class, function);
                let label = current
                    .as_ref()
                    .map(|function| display_port(&function.name))
                    .unwrap_or_else(|_| "Missing function (preserved)".into());
                if let Some(_combo) = ui.begin_combo("Function", label) {
                    for candidate in target_functions(registry, class) {
                        if ui.selectable(function_label(&candidate)) {
                            *function = candidate.id;
                        }
                    }
                }
                if let Err(error) = current {
                    ui.text_colored([1., 0.4, 0.3, 1.], error);
                }
                ui.text_wrapped("Connect a compatible entity to Target. Retargeting preserves existing arguments and wires; compile to diagnose incompatible connections.");
            }
            NodeKind::Literal { value_type, value } => {
                edit_value(ui, "Literal", value, value_type, registry);
            }
            NodeKind::Loop { count } => {
                let mut n = *count as i32;
                if crate::gui::Drag::new("Iterations")
                    .speed(1.)
                    .build(ui, &mut n)
                {
                    *count = n.clamp(1, 1024) as u32;
                }
            }
            _ => {}
        }
        for socket in sockets.into_iter().filter(|p| !p.output) {
            let SocketType::Value(ty) = socket.ty else {
                continue;
            };
            let _id = ui.push_id(&socket.pin);
            ui.text(format!("{} : {}", socket.label, ty.label()));
            match node.inputs.get_mut(&socket.pin) {
                Some(Input::Link { node: source, pin }) => {
                    ui.text_disabled(format!("Connected: {source}.{pin}"));
                    if ui.is_item_hovered() {
                        ui.tooltip_text(asset::link_id(source, pin, &key, &socket.pin));
                    }
                }
                Some(Input::Parameter { name }) => ui.text_disabled(format!(
                    "Parameter: {}",
                    parameter_labels
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| display_port(name))
                )),
                Some(Input::Literal { value_type, value }) => {
                    edit_value(ui, "##input", value, value_type, registry);
                }
                None => {
                    if !matches!(ty, schema::Type::Record { .. } | schema::Type::Void)
                        && ui.small_button("Use literal")
                    {
                        node.inputs.insert(
                            socket.pin.clone(),
                            Input::Literal {
                                value: default_value(&ty),
                                value_type: ty.clone(),
                            },
                        );
                    }
                }
            }
            if let Some(_combo) = ui.begin_combo("Parameter", "Choose...") {
                for parameter in &parameters {
                    if parameter.value_type == ty
                        && ui.selectable(&parameter_labels[&parameter.name])
                    {
                        node.inputs.insert(
                            socket.pin.clone(),
                            Input::Parameter {
                                name: parameter.name.clone(),
                            },
                        );
                    }
                }
            }
            if node.inputs.contains_key(&socket.pin) && ui.small_button("Disconnect / clear") {
                node.inputs.remove(&socket.pin);
            }
        }
        ui.text_disabled("Alt-click a socket to remove its connections.");
    }
}

fn target_functions(registry: &Registry, target: &str) -> Vec<schema::Function> {
    let Some(class) = registry.classes.get(target) else {
        return vec![];
    };
    let mut functions = BTreeMap::new();
    for inherited in registry.ancestry(&class.cpp_name) {
        for function in &inherited.functions {
            if let Ok(resolved) =
                crate::blueprint_ir::call_on_function(registry, target, &function.id)
            {
                functions.insert(resolved.id.clone(), resolved);
            }
        }
    }
    let mut functions: Vec<_> = functions.into_values().collect();
    functions.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    functions
}
fn functions(doc: &BlueprintAsset, registry: &Registry) -> Vec<schema::Function> {
    let mut list = BTreeMap::new();
    if let Some(parent) = registry.classes.get(&doc.parent) {
        for class in registry.ancestry(&parent.cpp_name) {
            for function in &class.functions {
                for inherited in &function.overrides {
                    list.remove(inherited);
                }
                list.insert(function.id.clone(), function.clone());
            }
        }
    }
    let mut functions: Vec<_> = list.into_values().collect();
    functions.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    functions
}
fn function_by_id<'a>(
    doc: &BlueprintAsset,
    registry: &'a Registry,
    id: &str,
) -> Option<&'a schema::Function> {
    let parent = registry.classes.get(&doc.parent)?;
    registry
        .ancestry(&parent.cpp_name)
        .into_iter()
        .rev()
        .flat_map(|c| c.functions.iter())
        .find(|f| f.id == id)
}
fn function_label(function: &schema::Function) -> String {
    format!(
        "{}({}) -> {}##{}",
        function.name,
        function
            .parameters
            .iter()
            .map(|p| p.value_type.label())
            .collect::<Vec<_>>()
            .join(", "),
        function.returns.label(),
        function.id
    )
}
fn node_position(doc: &BlueprintAsset, graph: &Graph, key: &str) -> [f32; 2] {
    doc.layout.positions.get(key).copied().unwrap_or_else(|| {
        let index = graph
            .nodes
            .iter()
            .position(|n| n.id == key)
            .unwrap_or_default();
        [(index % 4) as f32 * 290., (index / 4) as f32 * 240.]
    })
}
fn canvas_graphs(doc: &BlueprintAsset, index: usize) -> Vec<&Graph> {
    if doc
        .functions
        .get(index)
        .is_some_and(|g| g.override_id.is_some())
    {
        doc.functions
            .iter()
            .filter(|g| g.override_id.is_some())
            .collect()
    } else {
        doc.functions.get(index).into_iter().collect()
    }
}
fn event_label(name: &str) -> String {
    match name {
        "start" => "On Start".into(),
        "update" => "On Update".into(),
        "on_trigger" => "On Trigger".into(),
        _ => display_port(name),
    }
}
fn event_pin_label(graph: &Graph, node: &Node, pin: &str) -> String {
    if matches!(node.kind, NodeKind::Entry | NodeKind::CallParent) {
        return event_parameter_label(graph, pin);
    }
    if let NodeKind::Builtin { operation } = &node.kind {
        use asset::Builtin::*;
        if pin == "value" {
            // The serialized port stays stable; the visible label describes the
            // actual value. This match deliberately covers every system adapter.
            return match operation {
                GetPosition | SetPosition | GetPosition2D | SetPosition2D | GetRectPosition
                | SetRectPosition => "Position",
                GetRotation | SetRotation | GetRotation2D | SetRotation2D => "Rotation",
                GetScale | SetScale | GetScale2D | SetScale2D => "Scale",
                GetRectSize | SetRectSize => "Size",
                GetTransform | MakeTransform => "Transform",
                SelfObject => "Self",
                GetOwner => "Owner",
                Spawn { .. } | SpawnClass { .. } => "Spawned Object",
                SpawnActor { .. } => "Spawned Actor",
                Cast { .. } => "Cast Result",
                IsA { .. } => "Is Matching Class",
                IsValid => "Is Valid",
                InputHeld => "Is Held",
                InputPressed => "Was Pressed",
                InputReleased => "Was Released",
                RequestScene => "Request Accepted",
                PlaySequenceComponent | PlayTimelineAsset { .. } | EffectSequence => {
                    "Sequence Playback"
                }
                PlayEffectComponent | SpawnParticleEffect { .. } => "Effect Playback",
                StopSequence | PauseSequence | ResumeSequence | StopEffect | PauseEffect
                | ResumeEffect | BurstEffect => "Succeeded",
                SetActive | DestroyActor | PlayAudio | StopAudio | SetTexture | SetAudioClip => {
                    "Result"
                }
            }
            .into();
        }
        if pin == "port" {
            return "Controller Port".into();
        }
        if pin == "index" && matches!(operation, RequestScene) {
            return "Scene Index".into();
        }
        if pin == "seed" {
            return "Random Seed".into();
        }
    }
    display_port(pin)
}
fn event_parameter_label(graph: &Graph, pin: &str) -> String {
    if graph.override_id.is_some()
        && let Some(index) = graph.parameters.iter().position(|p| p.name == pin)
    {
        let label = match (graph.name.as_str(), index) {
            ("tick", 0) => Some("Delta Seconds"),
            ("end_play", 0) => Some("End Play Reason"),
            ("start" | "update", 0) => Some("Transform"),
            ("update", 1) => Some("Delta Seconds"),
            ("on_trigger", 0) => Some("Other Actor"),
            ("on_trigger", 1) => Some("Phase"),
            _ => None,
        };
        if let Some(label) = label {
            return label.into();
        }
    }
    display_port(pin)
}
fn parent_class_label(doc: &BlueprintAsset, registry: &Registry) -> String {
    registry
        .classes
        .get(&doc.parent)
        .map(|c| c.cpp_name.trim_start_matches("epok::").to_owned())
        .unwrap_or_else(|| "Missing parent class".into())
}
fn can_call_parent(doc: &BlueprintAsset, graph: &Graph, registry: &Registry) -> bool {
    graph
        .override_id
        .as_ref()
        .and_then(|id| function_by_id(doc, registry, id))
        .is_some_and(|f| !f.abstract_method && !f.final_method && f.access != "private")
}
fn exec_reroute(graph: &Graph, node: &Node) -> bool {
    matches!(node.kind, NodeKind::Reroute)
        && (node.outputs.contains_key("next")
            || graph
                .nodes
                .iter()
                .any(|n| n.outputs.values().flatten().any(|id| *id == node.id)))
}
fn properties(doc: &BlueprintAsset, registry: &Registry) -> Vec<schema::Property> {
    registry
        .classes
        .get(&doc.parent)
        .map(|parent| {
            registry
                .properties(&parent.cpp_name)
                .into_iter()
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}
fn member_type(doc: &BlueprintAsset, registry: &Registry, member: &str) -> Option<schema::Type> {
    doc.variables
        .iter()
        .find(|v| v.id == member)
        .map(|v| v.value_type.clone())
        .or_else(|| {
            properties(doc, registry)
                .iter()
                .find(|p| p.id == member)
                .map(|p| p.value_type.clone())
        })
}
/// Unbound generic sockets take their type from the first wire, not a coercion.
fn flexible_input(node: &Node, pin: &str, ty: &SocketType) -> bool {
    if !node.inputs.is_empty() {
        return false;
    }
    if matches!(node.kind, NodeKind::Reroute) {
        return pin == "value" && node.outputs.is_empty();
    }
    let SocketType::Value(ty) = ty else {
        return false;
    };
    match &node.kind {
        NodeKind::Reroute => pin == "value",
        NodeKind::Binary { op } if pin == "a" || pin == "b" => match op {
            asset::BinaryOp::And | asset::BinaryOp::Or => *ty == schema::Type::Bool,
            asset::BinaryOp::Equal | asset::BinaryOp::NotEqual => matches!(
                ty,
                schema::Type::Bool
                    | schema::Type::Fixed
                    | schema::Type::Int32
                    | schema::Type::UInt32
            ),
            _ => matches!(
                ty,
                schema::Type::Fixed | schema::Type::Int32 | schema::Type::UInt32
            ),
        },
        _ => false,
    }
}
fn compatible(actual: &SocketType, expected: &SocketType, registry: &Registry) -> bool {
    match (actual, expected) {
        (SocketType::Value(a), SocketType::Value(b)) => {
            crate::blueprint_ir::assignable(a, b, registry)
        }
        _ => actual == expected,
    }
}
fn input_type(
    doc: &BlueprintAsset,
    graph: &Graph,
    input: &Input,
    registry: &Registry,
    depth: usize,
) -> Option<schema::Type> {
    if depth > 64 {
        return None;
    }
    match input {
        Input::Literal { value_type, .. } => Some(value_type.clone()),
        Input::Parameter { name } => graph
            .parameters
            .iter()
            .find(|p| p.name == *name)
            .map(|p| p.value_type.clone()),
        Input::Link { node, pin } => {
            let node = graph.nodes.iter().find(|n| n.id == *node)?;
            let (root, path) = pin.split_once('.').unwrap_or((pin, ""));
            let ty = if matches!(node.kind, NodeKind::Entry) {
                graph
                    .parameters
                    .iter()
                    .find(|p| p.name == root)
                    .map(|p| p.value_type.clone())?
            } else {
                if root != "value" {
                    return None;
                }
                output_type(doc, graph, node, registry, depth + 1)?
            };
            ty.member_type(path)
        }
    }
}
fn output_type(
    doc: &BlueprintAsset,
    graph: &Graph,
    node: &Node,
    registry: &Registry,
    depth: usize,
) -> Option<schema::Type> {
    match &node.kind {
        NodeKind::Literal { value_type, .. } => Some(value_type.clone()),
        NodeKind::MakeVector { length } => Some(schema::Type::Vector { length: *length }),
        NodeKind::VectorComponent { .. } => Some(schema::Type::Fixed),
        NodeKind::GetVariable { member } => member_type(doc, registry, member),
        NodeKind::Reroute if exec_reroute(graph, node) => None,
        NodeKind::Reroute => node
            .inputs
            .get("value")
            .and_then(|i| input_type(doc, graph, i, registry, depth + 1))
            .or(Some(schema::Type::Fixed)),
        NodeKind::Not => Some(schema::Type::Bool),
        NodeKind::Binary { op } => {
            let tag = serde_json::to_string(op).unwrap_or_default();
            if tag.contains("equal")
                || tag.contains("less")
                || tag.contains("greater")
                || tag.contains("and")
                || tag.contains("or")
            {
                Some(schema::Type::Bool)
            } else {
                node.inputs
                    .get("a")
                    .or_else(|| node.inputs.get("b"))
                    .and_then(|i| input_type(doc, graph, i, registry, depth + 1))
                    .or(Some(schema::Type::Fixed))
            }
        }
        NodeKind::Call { function } => function_by_id(doc, registry, function)
            .map(|f| f.returns.clone())
            .or_else(|| {
                doc.functions
                    .iter()
                    .find(|g| g.id == *function)
                    .map(|g| g.returns.clone())
            }),
        NodeKind::CallParent => Some(graph.returns.clone()),
        NodeKind::CallOn { class, function } => {
            crate::blueprint_ir::call_on_function(registry, class, function)
                .ok()
                .map(|function| function.returns)
        }
        NodeKind::Builtin { operation } => {
            use asset::Builtin;
            let family = registry
                .classes
                .get(&doc.parent)
                .map(|p| crate::blueprint_workflow::parent_family(registry, p));
            Some(match operation {
                Builtin::SelfObject if family == Some(schema::ClassFamily::Component) => {
                    schema::Type::ComponentRef {
                        class: Some(doc.id.clone()),
                    }
                }
                Builtin::SelfObject => schema::Type::ActorRef {
                    class: Some(doc.id.clone()),
                },
                Builtin::GetOwner => {
                    let inherited = registry.classes.get(&doc.parent).and_then(|p| {
                        registry
                            .ancestry(&p.cpp_name)
                            .into_iter()
                            .rev()
                            .filter_map(|c| c.component.as_ref())
                            .find(|c| !c.owners.is_empty())
                    });
                    let contract = doc
                        .component
                        .as_ref()
                        .filter(|c| !c.owners.is_empty())
                        .or(inherited);
                    let class = contract
                        .filter(|c| c.owners.len() == 1)
                        .and_then(|c| c.owners.first())
                        .and_then(|d| match d {
                            schema::Domain::World3D => Some(crate::object_model::ACTOR3D_ID.into()),
                            schema::Domain::World2D => Some(crate::object_model::ACTOR2D_ID.into()),
                            schema::Domain::UI => Some(crate::object_model::UI_ACTOR_ID.into()),
                            _ => None,
                        });
                    schema::Type::ActorRef { class }
                }
                Builtin::Cast { class } => {
                    let kind = registry
                        .classes
                        .get(class)
                        .map(|p| crate::blueprint_workflow::parent_family(registry, p));
                    match kind {
                        Some(schema::ClassFamily::Actor) => schema::Type::ActorRef {
                            class: Some(class.clone()),
                        },
                        Some(schema::ClassFamily::Component) => schema::Type::ComponentRef {
                            class: Some(class.clone()),
                        },
                        _ => schema::Type::ObjectRef {
                            class: Some(class.clone()),
                        },
                    }
                }
                _ => crate::blueprint_ir::builtin_signature(operation).1,
            })
        }
        NodeKind::Operation { operation } => registry
            .operation(operation)
            .and_then(|operation| operation.outputs.first())
            .map(|output| output.value_type.clone()),
        _ => None,
    }
}
#[cfg(test)]
fn node_sockets(
    doc: &BlueprintAsset,
    graph: &Graph,
    node: &Node,
    registry: &Registry,
) -> Vec<Socket> {
    node_sockets_with_assets(doc, graph, node, registry, &[], &[])
}
fn node_sockets_with_assets(
    doc: &BlueprintAsset,
    graph: &Graph,
    node: &Node,
    registry: &Registry,
    timelines: &[(PathBuf, crate::timeline::TimelineAsset)],
    effects: &[(PathBuf, crate::particle_effect::ParticleEffect)],
) -> Vec<Socket> {
    let mut pins = vec![];
    let binding_slots = if let NodeKind::Builtin { operation } = &node.kind {
        crate::blueprint_playback::slots(operation, timelines, effects)
            .ok()
            .flatten()
    } else {
        None
    };
    let operation_labels = if let NodeKind::Operation { operation } = &node.kind {
        registry
            .operation(operation)
            .map(|operation| {
                operation
                    .parameters
                    .iter()
                    .map(|parameter| (parameter.id.clone(), display_port(&parameter.name)))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };
    let mut push = |pin: &str, output: bool, ty: SocketType| {
        pins.push(Socket {
            node: node.id.clone(),
            pin: pin.into(),
            output,
            ty,
            label: operation_labels
                .iter()
                .find(|(id, _)| id == pin)
                .map(|(_, label)| label.clone())
                .or_else(|| {
                    binding_slots
                        .and_then(|slots| {
                            slots
                                .iter()
                                .find(|slot| crate::blueprint_playback::pin(slot) == pin)
                        })
                        .map(|slot| {
                            format!(
                                "{} ({})",
                                slot.name,
                                if slot.required {
                                    "required"
                                } else {
                                    "optional"
                                }
                            )
                        })
                })
                .unwrap_or_else(|| event_pin_label(graph, node, pin)),
        })
    };
    let pure = matches!(
        node.kind,
        NodeKind::Literal { .. }
            | NodeKind::GetVariable { .. }
            | NodeKind::Binary { .. }
            | NodeKind::Not
            | NodeKind::MakeVector { .. }
            | NodeKind::VectorComponent { .. }
    ) || (matches!(node.kind, NodeKind::Reroute) && !exec_reroute(graph, node))
        || matches!(&node.kind,NodeKind::Call{function} if function_by_id(doc,registry,function).is_some_and(|f|f.pure))
        || matches!(&node.kind,NodeKind::CallOn{class,function} if crate::blueprint_ir::call_on_function(registry,class,function).is_ok_and(|f|f.pure))
        || matches!(&node.kind,NodeKind::Builtin{operation} if crate::blueprint_ir::builtin_signature(operation).2);
    let pure = pure
        || matches!(&node.kind,NodeKind::Operation{operation} if registry.operation(operation).is_some_and(|operation| matches!(operation.effect, schema::OperationEffect::PureValue | schema::OperationEffect::StateRead)));
    if !pure && !matches!(node.kind, NodeKind::Entry) {
        push("exec", false, SocketType::Exec);
    }
    if !pure && !matches!(node.kind, NodeKind::Return) {
        match node.kind {
            NodeKind::Sequence => {
                for pin in asset::sequence_outputs(node) {
                    push(&pin, true, SocketType::Exec);
                }
            }
            NodeKind::Branch => {
                push("true", true, SocketType::Exec);
                push("false", true, SocketType::Exec);
            }
            NodeKind::Loop { .. } => {
                push("body", true, SocketType::Exec);
                push("next", true, SocketType::Exec);
            }
            NodeKind::Timeline { .. } => {
                push("updated", true, SocketType::Exec);
                push("finished", true, SocketType::Exec);
                push("next", true, SocketType::Exec);
            }
            NodeKind::WaitPlayback { ref condition } => {
                if matches!(
                    condition,
                    asset::PlaybackCondition::Marker { .. }
                        | asset::PlaybackCondition::SubscribeMarker { .. }
                ) {
                    push("reached", true, SocketType::Exec);
                }
                push("completed", true, SocketType::Exec);
                push("cancelled", true, SocketType::Exec);
                push("next", true, SocketType::Exec);
            }
            _ => push("next", true, SocketType::Exec),
        }
    }
    match &node.kind {
        NodeKind::MakeVector { length } => {
            for axis in ["x", "y", "z"].into_iter().take(*length) {
                push(axis, false, SocketType::Value(schema::Type::Fixed));
            }
        }
        NodeKind::VectorComponent { length, .. } => push(
            "value",
            false,
            SocketType::Value(schema::Type::Vector { length: *length }),
        ),
        NodeKind::Entry => {
            for p in &graph.parameters {
                push(&p.name, true, SocketType::Value(p.value_type.clone()));
            }
        }
        NodeKind::SetVariable { member } => {
            if let Some(ty) = member_type(doc, registry, member) {
                push("value", false, SocketType::Value(ty));
            }
        }
        NodeKind::Call { function } => {
            let params = function_by_id(doc, registry, function)
                .map(|f| f.parameters.clone())
                .or_else(|| {
                    doc.functions
                        .iter()
                        .find(|g| g.id == *function)
                        .map(|g| g.parameters.clone())
                })
                .unwrap_or_default();
            for p in params {
                push(&p.name, false, SocketType::Value(p.value_type));
            }
        }
        NodeKind::CallParent => {
            for p in &graph.parameters {
                push(&p.name, false, SocketType::Value(p.value_type.clone()));
            }
        }
        NodeKind::CallOn { class, function } => {
            push(
                "__target",
                false,
                SocketType::Value(schema::Type::ObjectRef {
                    class: Some(class.clone()),
                }),
            );
            if let Ok(function) = crate::blueprint_ir::call_on_function(registry, class, function) {
                for parameter in function.parameters {
                    push(
                        &parameter.name,
                        false,
                        SocketType::Value(parameter.value_type),
                    );
                }
            }
        }
        NodeKind::Builtin { operation } => {
            for (name, ty) in crate::blueprint_ir::builtin_signature(operation).0 {
                push(&name, false, SocketType::Value(ty));
            }
            if let Some(slots) = binding_slots {
                for slot in crate::blueprint_playback::external(slots) {
                    push(
                        &crate::blueprint_playback::pin(slot),
                        false,
                        SocketType::Value(slot.target.clone()),
                    );
                }
            }
        }
        NodeKind::Operation { operation } => {
            if let Some(operation) = registry.operation(operation) {
                match &operation.receiver {
                    schema::ReceiverKind::Instance { class } => push(
                        "__target",
                        false,
                        SocketType::Value(schema::Type::ObjectRef {
                            class: Some(class.clone()),
                        }),
                    ),
                    schema::ReceiverKind::Value { cpp_name } => push(
                        "__target",
                        false,
                        SocketType::Value(schema::Type::Record {
                            cpp_name: cpp_name.clone(),
                            fields: vec![],
                        }),
                    ),
                    schema::ReceiverKind::Service { .. } => {}
                }
                for parameter in &operation.parameters {
                    push(
                        &parameter.id,
                        false,
                        SocketType::Value(parameter.value_type.clone()),
                    );
                }
            }
        }
        NodeKind::Binary { op } => {
            let tag = serde_json::to_string(op).unwrap_or_default();
            let ty = if tag.contains("and") || tag.contains("or") {
                schema::Type::Bool
            } else {
                node.inputs
                    .get("a")
                    .or_else(|| node.inputs.get("b"))
                    .and_then(|i| input_type(doc, graph, i, registry, 0))
                    .unwrap_or(schema::Type::Fixed)
            };
            push("a", false, SocketType::Value(ty.clone()));
            push("b", false, SocketType::Value(ty));
        }
        NodeKind::Reroute if !exec_reroute(graph, node) => push(
            "value",
            false,
            SocketType::Value(
                output_type(doc, graph, node, registry, 0).unwrap_or(schema::Type::Fixed),
            ),
        ),
        NodeKind::Delay => push("seconds", false, SocketType::Value(schema::Type::Fixed)),
        NodeKind::WaitPlayback { condition } => push(
            "playback",
            false,
            SocketType::Value(
                if matches!(condition, asset::PlaybackCondition::EffectComplete) {
                    schema::Type::EffectHandle
                } else {
                    schema::Type::SequenceHandle
                },
            ),
        ),
        NodeKind::Not => push("value", false, SocketType::Value(schema::Type::Bool)),
        NodeKind::Branch => push("condition", false, SocketType::Value(schema::Type::Bool)),
        NodeKind::Return if graph.returns != schema::Type::Void => {
            push("value", false, SocketType::Value(graph.returns.clone()));
        }
        _ => {}
    }
    if let Some(ty) = output_type(doc, graph, node, registry, 0)
        && ty != schema::Type::Void
    {
        push("value", true, SocketType::Value(ty));
    }
    split_pins::expand(doc, node, pins)
}
fn literal_display(value: &Value) -> String {
    let text = value.to_string();
    if text.len() > 12 {
        format!("{}...", text.chars().take(9).collect::<String>())
    } else {
        text
    }
}
/// Unscaled row extent: both pin/label insets, the literal's label gap, and
/// separation before a same-row output. The title cap cannot shrink this width.
fn socket_row_width(input_label: f32, literal: Option<f32>, output_label: f32) -> f32 {
    input_label + output_label + 62. + literal.map_or(0., |width| width + 7.)
}
fn node_label(node: &Node, doc: &BlueprintAsset, registry: &Registry) -> String {
    match &node.kind {
        NodeKind::Entry => "Event / Entry".into(),
        NodeKind::Literal { value_type, .. } => format!("{} literal", value_type.label()),
        NodeKind::MakeVector { length } => format!("Make Vector{length}"),
        NodeKind::VectorComponent { length, index } => format!(
            "Vector{length} {}",
            ["X", "Y", "Z"].get(*index).unwrap_or(&"Invalid axis")
        ),
        NodeKind::GetVariable { member } | NodeKind::SetVariable { member } => {
            let name = doc
                .variables
                .iter()
                .find(|v| v.id == *member)
                .map(|v| v.name.clone())
                .or_else(|| {
                    properties(doc, registry)
                        .iter()
                        .find(|p| p.id == *member)
                        .map(|p| p.name.clone())
                })
                .unwrap_or_else(|| "Missing member".into());
            format!(
                "{} {}",
                if matches!(node.kind, NodeKind::GetVariable { .. }) {
                    "Get"
                } else {
                    "Set"
                },
                display_port(&name)
            )
        }
        NodeKind::Call { function } => function_by_id(doc, registry, function)
            .map(|f| display_port(&f.name))
            .or_else(|| {
                doc.functions
                    .iter()
                    .find(|g| g.id == *function)
                    .map(|g| display_port(&g.name))
            })
            .unwrap_or_else(|| "Missing function".into()),
        NodeKind::CallParent => doc
            .functions
            .iter()
            .find(|g| g.nodes.iter().any(|n| n.id == node.id))
            .map(|g| format!("Parent: {}", event_label(&g.name)))
            .unwrap_or_else(|| "Call Parent".into()),
        NodeKind::CallOn { class, function } => {
            crate::blueprint_ir::call_on_function(registry, class, function)
                .map(|function| display_port(&function.name))
                .unwrap_or_else(|_| "Missing target function".into())
        }
        NodeKind::Binary { op } => format!("{op:?}"),
        NodeKind::Reroute => "Reroute".into(),
        NodeKind::Not => "Not".into(),
        NodeKind::Branch => "Branch".into(),
        NodeKind::Sequence => "Sequence".into(),
        NodeKind::Loop { count } => format!("Loop ({count})"),
        NodeKind::Return => "Return".into(),
        NodeKind::Delay => "Delay (simulation seconds)".into(),
        NodeKind::WaitPlayback { condition } => match condition {
            asset::PlaybackCondition::Marker { .. } => "Wait for marker".into(),
            asset::PlaybackCondition::SubscribeMarker { .. } => "Subscribe to marker".into(),
            asset::PlaybackCondition::SequenceComplete => "Wait for sequence completion".into(),
            asset::PlaybackCondition::EffectComplete => "Wait for effect completion".into(),
        },
        NodeKind::Timeline { .. } => "Timeline (Q12 curve)".into(),
        NodeKind::StopTimeline { .. } => "Stop Timeline".into(),
        NodeKind::Builtin { operation } => match operation {
            asset::Builtin::PlayTimelineAsset { .. } => "Play Timeline Asset".into(),
            asset::Builtin::SpawnParticleEffect { .. } => "Spawn Particle Effect".into(),
            asset::Builtin::Spawn { class }
            | asset::Builtin::IsA { class }
            | asset::Builtin::Cast { class }
            | asset::Builtin::SpawnClass { base: class } => format!(
                "{} {}",
                if matches!(operation, asset::Builtin::SpawnClass { .. }) {
                    "Spawn Class"
                } else if matches!(operation, asset::Builtin::Spawn { .. }) {
                    "Spawn"
                } else if matches!(operation, asset::Builtin::Cast { .. }) {
                    "Cast to"
                } else {
                    "Is A"
                },
                registry
                    .classes
                    .get(class)
                    .map(|c| c.cpp_name.as_str())
                    .unwrap_or("Missing class")
            ),
            _ => display_port(&format!("{operation:?}")),
        },
        NodeKind::Operation { operation } => registry
            .operation(operation)
            .map(|operation| display_port(&operation.name))
            .unwrap_or_else(|| "Missing gameplay operation".into()),
    }
}
fn playback_label(
    node: &Node,
    timelines: &[(PathBuf, crate::timeline::TimelineAsset)],
    effects: &[(PathBuf, crate::particle_effect::ParticleEffect)],
) -> Option<String> {
    match &node.kind {
        NodeKind::Builtin {
            operation: asset::Builtin::PlayTimelineAsset { asset },
        } => Some(format!(
            "Play {}",
            timelines
                .iter()
                .find(|(_, value)| value.id.to_string() == *asset)
                .map_or("missing TimelineAsset", |(_, value)| value.name.as_str())
        )),
        NodeKind::Builtin {
            operation: asset::Builtin::SpawnParticleEffect { asset },
        } => Some(format!(
            "Spawn {}",
            effects
                .iter()
                .find(|(_, value)| value.id.to_string() == *asset)
                .map_or("missing ParticleEffect", |(_, value)| value.name.as_str())
        )),
        NodeKind::WaitPlayback {
            condition:
                condition @ (asset::PlaybackCondition::Marker { timeline, marker }
                | asset::PlaybackCondition::SubscribeMarker { timeline, marker }),
        } => Some(format!(
            "{} {}",
            if matches!(condition, asset::PlaybackCondition::SubscribeMarker { .. }) {
                "Subscribe to"
            } else {
                "Wait for"
            },
            timelines
                .iter()
                .find(|(_, value)| value.id.to_string() == *timeline)
                .and_then(|(_, value)| value
                    .markers
                    .iter()
                    .find(|value| value.id.to_string() == *marker))
                .map_or("missing marker", |value| value.name.as_str())
        )),
        _ => None,
    }
}
fn socket_color(ty: &SocketType) -> [f32; 4] {
    match ty {
        SocketType::Exec => [0.95, 0.95, 0.95, 1.],
        SocketType::Value(schema::Type::Bool) => [0.65, 0.12, 0.16, 1.],
        SocketType::Value(schema::Type::Fixed) => [0.42, 0.87, 0.22, 1.],
        SocketType::Value(schema::Type::Int32 | schema::Type::UInt32) => [0.2, 0.8, 0.9, 1.],
        SocketType::Value(
            schema::Type::ObjectRef { .. }
            | schema::Type::AssetRef { .. }
            | schema::Type::Record { .. },
        ) => [0., 0.70, 0.90, 1.],
        SocketType::Value(schema::Type::Vector { .. }) => [0.95, 0.64, 0.15, 1.],
        _ => [0.65, 0.3, 0.85, 1.],
    }
}
fn display_port(port: &str) -> String {
    if port == "__target" {
        return "Target".into();
    }
    if matches!(port, "exec" | "next") {
        return String::new();
    }
    let mut result = String::new();
    let mut upper = true;
    let chars: Vec<_> = port.chars().collect();
    for (index, c) in chars.iter().copied().enumerate() {
        if c == '_' {
            result.push(' ');
            upper = true;
        } else if upper {
            result.extend(c.to_uppercase());
            upper = false;
        } else {
            if c.is_uppercase()
                && index > 0
                && (chars[index - 1].is_lowercase()
                    || (chars[index - 1].is_uppercase()
                        && chars.get(index + 1).is_some_and(|next| next.is_lowercase())))
            {
                result.push(' ');
            }
            result.push(c);
        }
    }
    result
}
fn node_header(node: &Node, graph: &Graph, doc: &BlueprintAsset, registry: &Registry) -> [f32; 4] {
    match &node.kind {
        NodeKind::Entry if graph.override_id.is_some() => [0.39, 0.10, 0.08, 1.],
        NodeKind::Entry | NodeKind::Return => [0.26, 0.14, 0.31, 1.],
        NodeKind::GetVariable { .. }
        | NodeKind::Literal { .. }
        | NodeKind::Binary { .. }
        | NodeKind::MakeVector { .. }
        | NodeKind::VectorComponent { .. }
        | NodeKind::Not => [0.24, 0.34, 0.18, 1.],
        NodeKind::Call { function }
            if function_by_id(doc, registry, function).is_some_and(|f| f.pure) =>
        {
            [0.24, 0.34, 0.18, 1.]
        }
        NodeKind::CallOn { class, function }
            if crate::blueprint_ir::call_on_function(registry, class, function)
                .is_ok_and(|function| function.pure) =>
        {
            [0.24, 0.34, 0.18, 1.]
        }
        NodeKind::Builtin {
            operation: asset::Builtin::IsA { .. } | asset::Builtin::Cast { .. },
        } => [0.13, 0.31, 0.29, 1.],
        NodeKind::Builtin { operation } if crate::blueprint_ir::builtin_signature(operation).2 => {
            [0.24, 0.34, 0.18, 1.]
        }
        NodeKind::Operation { operation }
            if registry.operation(operation).is_some_and(|operation| {
                matches!(
                    operation.effect,
                    schema::OperationEffect::PureValue | schema::OperationEffect::StateRead
                )
            }) =>
        {
            [0.24, 0.34, 0.18, 1.]
        }
        NodeKind::Call { .. }
        | NodeKind::CallOn { .. }
        | NodeKind::CallParent
        | NodeKind::SetVariable { .. }
        | NodeKind::Operation { .. }
        | NodeKind::Builtin { .. } => [0.15, 0.27, 0.34, 1.],
        NodeKind::Timeline { .. } | NodeKind::StopTimeline { .. } | NodeKind::Delay => {
            [0.35, 0.27, 0.13, 1.]
        }
        _ => [0.26, 0.27, 0.25, 1.],
    }
}
fn input_connection<'a>(entry: &'a str, input: &'a Input) -> Option<(&'a str, &'a str)> {
    match input {
        Input::Link { node, pin } => Some((node, pin)),
        Input::Parameter { name } => Some((entry, name)),
        Input::Literal { .. } => None,
    }
}
fn socket_connected(graph: &Graph, node: &Node, socket: &Socket) -> bool {
    match (&socket.ty, socket.output) {
        (SocketType::Exec, true) => node
            .outputs
            .get(&socket.pin)
            .is_some_and(|targets| !targets.is_empty()),
        (SocketType::Exec, false) => graph
            .nodes
            .iter()
            .any(|n| n.outputs.values().flatten().any(|id| id == &node.id)),
        (_, true) => graph.nodes.iter().any(|n| {
            n.inputs.values().any(|input| {
                input_connection(&graph.entry, input)
                    == Some((node.id.as_str(), socket.pin.as_str()))
            })
        }),
        (_, false) => node
            .inputs
            .get(&socket.pin)
            .is_some_and(|input| input_connection(&graph.entry, input).is_some()),
    }
}
fn paint_socket(
    draw: &imgui::DrawListMut<'_>,
    p: [f32; 2],
    ty: &SocketType,
    connected: bool,
    zoom: f32,
) {
    let color = socket_color(ty);
    let radius = 4.5 * zoom;
    if *ty == SocketType::Exec {
        let a = [p[0] - radius * 0.65, p[1] - radius];
        let b = [p[0] + radius, p[1]];
        let c = [p[0] - radius * 0.65, p[1] + radius];
        draw.add_triangle(a, b, c, color)
            .filled(connected)
            .thickness(1.4)
            .build();
    } else {
        draw.add_circle(p, radius, color)
            .filled(connected)
            .thickness(1.4)
            .build();
    }
}
fn timeline_curve(ui: &Ui, keys: &mut [[f64; 2]], drag: &mut Option<usize>) {
    let origin = ui.cursor_screen_pos();
    let size = [ui.content_region_avail()[0].max(80.), 120.];
    ui.invisible_button("timeline-curve", size);
    let hovered = ui.is_item_hovered();
    if keys.is_empty() || keys.iter().flatten().any(|v| !v.is_finite()) {
        return;
    }
    let duration = keys.last().unwrap()[0].max(0.001);
    let minimum = keys.iter().map(|v| v[1]).fold(f64::INFINITY, f64::min);
    let maximum = keys.iter().map(|v| v[1]).fold(f64::NEG_INFINITY, f64::max);
    let range = (maximum - minimum).max(1.);
    let to_screen = |key: [f64; 2]| {
        [
            origin[0] + 8. + (key[0] / duration) as f32 * (size[0] - 16.),
            origin[1] + size[1] - 8. - ((key[1] - minimum) / range) as f32 * (size[1] - 16.),
        ]
    };
    let draw = ui.get_window_draw_list();
    draw.add_rect(
        origin,
        [origin[0] + size[0], origin[1] + size[1]],
        [0.07, 0.09, 0.12, 1.],
    )
    .filled(true)
    .build();
    for segment in keys.windows(2) {
        draw.add_line(
            to_screen(segment[0]),
            to_screen(segment[1]),
            [0.4, 0.95, 0.4, 1.],
        )
        .thickness(2.)
        .build();
    }
    for (index, key) in keys.iter().enumerate() {
        draw.add_circle(
            to_screen(*key),
            5.,
            if *drag == Some(index) {
                [1., 0.7, 0.2, 1.]
            } else {
                [0.8, 1., 0.8, 1.]
            },
        )
        .filled(true)
        .build();
    }
    if hovered && ui.is_mouse_clicked(MouseButton::Left) {
        let mouse = ui.io().mouse_pos;
        *drag = keys.iter().position(|k| {
            let p = to_screen(*k);
            (p[0] - mouse[0]).abs() < 9. && (p[1] - mouse[1]).abs() < 9.
        });
    }
    if !ui.is_mouse_down(MouseButton::Left) {
        *drag = None;
    }
    if let Some(index) = *drag
        && index < keys.len()
        && ui.is_mouse_dragging(MouseButton::Left)
    {
        let mouse = ui.io().mouse_pos;
        let time = f64::from((mouse[0] - origin[0] - 8.) / (size[0] - 16.)) * duration;
        if index > 0 {
            let lower = keys[index - 1][0] + 1. / 4096.;
            let upper = keys
                .get(index + 1)
                .map(|k| k[0] - 1. / 4096.)
                .unwrap_or(duration * 2.);
            if lower <= upper {
                keys[index][0] = time.clamp(lower, upper);
            }
        }
        keys[index][1] = (minimum
            + f64::from((origin[1] + size[1] - 8. - mouse[1]) / (size[1] - 16.)) * range)
            .clamp(-524288., 524287.);
    }
}
struct CanvasWire {
    from: Socket,
    to: Socket,
    points: [[f32; 2]; 4],
}
impl CanvasWire {
    fn point(&self, t: f32) -> [f32; 2] {
        let u = 1. - t;
        std::array::from_fn(|axis| {
            u * u * u * self.points[0][axis]
                + 3. * u * u * t * self.points[1][axis]
                + 3. * u * t * t * self.points[2][axis]
                + t * t * t * self.points[3][axis]
        })
    }
    fn distance(&self, mouse: [f32; 2]) -> f32 {
        let mut distance = f32::INFINITY;
        let mut a = self.points[0];
        for step in 1..=64 {
            let b = self.point(step as f32 / 64.);
            let delta = [b[0] - a[0], b[1] - a[1]];
            let length = delta[0] * delta[0] + delta[1] * delta[1];
            let t = (((mouse[0] - a[0]) * delta[0] + (mouse[1] - a[1]) * delta[1])
                / length.max(0.0001))
            .clamp(0., 1.);
            distance = distance
                .min((mouse[0] - a[0] - t * delta[0]).hypot(mouse[1] - a[1] - t * delta[1]));
            a = b;
        }
        distance
    }
}
fn inside(point: [f32; 2], a: [f32; 2], b: [f32; 2]) -> bool {
    point[0] >= a[0] && point[0] <= b[0] && point[1] >= a[1] && point[1] <= b[1]
}
fn paint_wire(
    draw: &imgui::DrawListMut<'_>,
    sockets: &[(Socket, [f32; 2])],
    source: &str,
    pin: &str,
    target: &str,
    input: &str,
) -> Option<CanvasWire> {
    let a = sockets
        .iter()
        .find(|(s, _)| s.node == source && s.pin == pin && s.output);
    let b = sockets
        .iter()
        .find(|(s, _)| s.node == target && s.pin == input && !s.output);
    if let (Some((s, a)), Some((target, b))) = (a, b) {
        let d = ((b[0] - a[0]).abs() * 0.5).max(30.);
        draw.add_bezier_curve(
            *a,
            [a[0] + d, a[1]],
            [b[0] - d, b[1]],
            *b,
            socket_color(&s.ty),
        )
        .thickness(2.)
        .build();
        return Some(CanvasWire {
            from: s.clone(),
            to: target.clone(),
            points: [*a, [a[0] + d, a[1]], [b[0] - d, b[1]], *b],
        });
    }
    None
}
fn reachable(
    graph: &Graph,
    from: &str,
    target: &str,
    exec: bool,
    seen: &mut BTreeSet<String>,
) -> bool {
    if from == target {
        return true;
    }
    if !seen.insert(from.into()) {
        return false;
    }
    if exec {
        graph.nodes.iter().find(|n| n.id == from).is_some_and(|n| {
            n.outputs
                .values()
                .flatten()
                .any(|next| reachable(graph, next, target, exec, seen))
        })
    } else {
        graph
            .nodes
            .iter()
            .filter(|n| {
                n.inputs
                    .values()
                    .any(|i| matches!(i,Input::Link{node,..}if node==from))
            })
            .any(|n| reachable(graph, &n.id, target, exec, seen))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod events {
        include!("blueprint_event_tests.rs");
    }
    mod reroutes {
        include!("blueprint_reroute_tests.rs");
    }
    mod split_pin_tests {
        include!("blueprint_split_pin_tests.rs");
    }
    #[test]
    #[ignore = "Owns a real ImGui context; run explicitly and serially"]
    fn graph_canvas_creates_connects_drags_and_undoes_using_imgui_events() {
        let mut context = crate::gui::tests::imgui_context();
        context.set_ini_filename(None);
        context.io_mut().display_size = [1280., 850.];
        context.io_mut().delta_time = 1. / 60.;
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        let mut editor = editor();
        editor.open = true;
        let registry = Registry::new();
        let frame = |ctx: &mut imgui::Context, e: &mut BlueprintEditor| {
            e.draw(ctx.frame(), &registry);
            ctx.render();
        };
        let click = |ctx: &mut imgui::Context, e: &mut BlueprintEditor, p: [f32; 2]| {
            ctx.io_mut().add_mouse_pos_event(p);
            frame(ctx, e);
            ctx.io_mut().add_mouse_button_event(MouseButton::Left, true);
            frame(ctx, e);
            ctx.io_mut()
                .add_mouse_button_event(MouseButton::Left, false);
            frame(ctx, e);
        };
        frame(&mut context, &mut editor);
        let add = CONTROLS.with(|c| c.borrow()["Add node..."]);
        click(&mut context, &mut editor, add);
        frame(&mut context, &mut editor);
        let flow = CONTROLS.with(|c| c.borrow()["bp-action:Flow"]);
        click(&mut context, &mut editor, flow);
        frame(&mut context, &mut editor);
        let branch = CONTROLS.with(|c| c.borrow()["Flow / Branch"]);
        click(&mut context, &mut editor, branch);
        assert_eq!(
            editor.current().unwrap().nodes.len(),
            2,
            "Catalog creates a production node"
        );
        let entry = editor.current().unwrap().entry.clone();
        let branch = editor.current().unwrap().nodes[1].id.clone();
        editor
            .asset
            .as_mut()
            .unwrap()
            .layout
            .positions
            .insert(branch.clone(), [320., 100.]);
        frame(&mut context, &mut editor);
        let a = SOCKETS.with(|c| c.borrow()[&format!("{entry}:next:true")]);
        let b = SOCKETS.with(|c| c.borrow()[&format!("{branch}:exec:false")]);
        click(&mut context, &mut editor, a);
        click(&mut context, &mut editor, b);
        assert_eq!(
            editor.current().unwrap().nodes[0].outputs["next"],
            vec![branch.clone()]
        );
        let header = [b[0] + 50., b[1] - 30.];
        click(&mut context, &mut editor, header);
        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, true);
        frame(&mut context, &mut editor);
        context
            .io_mut()
            .add_mouse_pos_event([header[0] + 45., header[1] + 35.]);
        frame(&mut context, &mut editor);
        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, false);
        frame(&mut context, &mut editor);
        assert_ne!(
            editor.asset.as_ref().unwrap().layout.positions[&branch],
            [320., 100.]
        );
        let undo = CONTROLS.with(|c| c.borrow()["Undo"]);
        click(&mut context, &mut editor, undo);
        assert_eq!(
            editor.asset.as_ref().unwrap().layout.positions[&branch],
            [320., 100.]
        );
        assert!(editor.current().unwrap().nodes[0].outputs["next"].contains(&branch));
        editor.asset.as_mut().unwrap().functions[0].nodes[1]
            .inputs
            .insert(
                "condition".into(),
                Input::Literal {
                    value_type: schema::Type::Bool,
                    value: Value::Bool(true),
                },
            );
        frame(&mut context, &mut editor);
        let checkbox = CONTROLS.with(|c| c.borrow()[&format!("bp-bool:{branch}:condition")]);
        let before_undo = editor.undo.len();
        click(&mut context, &mut editor, checkbox);
        assert!(
            matches!(&editor.current().unwrap().nodes[1].inputs["condition"],Input::Literal {value,..} if value==&Value::Bool(false))
        );
        assert_eq!(
            editor.undo.len(),
            before_undo + 1,
            "Inline checkbox records one transaction"
        );
        assert_eq!(
            editor.asset.as_ref().unwrap().layout.positions[&branch],
            [320., 100.],
            "Checkbox does not drag the node"
        );
        click(&mut context, &mut editor, undo);
        assert!(
            matches!(&editor.current().unwrap().nodes[1].inputs["condition"],Input::Literal {value,..} if value==&Value::Bool(true))
        );
        let class_settings = CONTROLS.with(|c| c.borrow()["Class Settings"]);
        click(&mut context, &mut editor, class_settings);
        assert!(
            editor.details == Details::Class,
            "Toolbar opens contextual class Details"
        );
        let add_member = CONTROLS.with(|c| c.borrow()["\u{ea60} Add##member"]);
        click(&mut context, &mut editor, add_member);
        let variable = CONTROLS.with(|c| c.borrow()["Variable"]);
        click(&mut context, &mut editor, variable);
        assert!(editor.details == Details::CreateVariable);
        let create = CONTROLS.with(|c| c.borrow()["Add variable"]);
        click(&mut context, &mut editor, create);
        assert_eq!(
            editor.asset.as_ref().unwrap().variables.len(),
            1,
            "My Blueprint Add creates a real declaration"
        );
        assert!(editor.details == Details::Variable);
        verify_panel_navigation(&mut context);
    }
    /// Shares the existing serialized ImGui context instead of constructing a
    /// second context that could race the editor's other integration fixtures.
    fn verify_panel_navigation(context: &mut imgui::Context) {
        let mut editor = editor();
        editor.open = true;
        let mut template = crate::blueprint_templates::Template::root("Root");
        let root = template.actors[0].entity.id;
        template.actors[0].entity.canvas = Some(crate::hud::Canvas { enabled: false });
        let child = template.add_child(root, "Target Child");
        let other = template.add_child(root, "Other Child");
        let original = template
            .actors
            .iter_mut()
            .find(|item| item.entity.id == child)
            .unwrap();
        original.entity.image = Some(crate::hud::Image {
            enabled: false,
            ..Default::default()
        });
        original.entity.rect = Some(crate::hud::RectTransform {
            position: [37., 19.],
            ..Default::default()
        });
        editor.asset.as_mut().unwrap().template = template;
        editor
            .asset
            .as_mut()
            .unwrap()
            .variables
            .push(asset::Variable {
                timeline_animatable: false,
                id: "local-other".into(),
                name: "unrelated".into(),
                value_type: schema::Type::Bool,
                default: Value::Bool(false),
                editable: true,
            });
        let source = schema::Location {
            file: "Parent.hpp".into(),
            line: 1,
            column: 1,
        };
        let mut registry = Registry::new();
        registry.classes.insert(
            "parent".into(),
            schema::Class {
                family: None,
                domain: None,
                placement: Default::default(),
                component: None,
                default_components: vec![],
                explicit_abstract: false,
                id: "parent".into(),
                provider: schema::native_provider(),
                backend: schema::native_backend(),
                cpp_name: "Parent".into(),
                parent: None,
                abstract_class: false,
                final_class: false,
                timeline_component: None,
                blueprintable: true,
                properties: vec![schema::Property {
                    id: "inherited-health".into(),
                    name: "health".into(),
                    value_type: schema::Type::Fixed,
                    default: json!(100),
                    editable: true,
                    timeline: None,
                    source: source.clone(),
                }],
                functions: vec![],
                source,
            },
        );
        let frame = |ctx: &mut imgui::Context, editor: &mut BlueprintEditor| {
            CONTROLS.with(|controls| controls.borrow_mut().clear());
            editor.draw(ctx.frame(), &registry);
            ctx.render()
                .draw_lists()
                .flat_map(|list| {
                    list.commands().filter_map(|command| match command {
                        imgui::DrawCmd::Elements { count, cmd_params } if count > 0 => {
                            Some(cmd_params.clip_rect)
                        }
                        _ => None,
                    })
                })
                .collect::<Vec<_>>()
        };
        let click = |ctx: &mut imgui::Context, editor: &mut BlueprintEditor, label: &str| {
            let point = CONTROLS.with(|controls| {
                let controls=controls.borrow();
                *controls.get(label).unwrap_or_else(||panic!("Missing panel control {label:?}; rendered controls={:?}; editor error={:?}; selected template={}, source nodes={}",controls.keys().collect::<Vec<_>>(),editor.error,editor.asset.as_ref().unwrap().template.actors.len(),editor.current().map_or(0,|graph|graph.nodes.len())))
            });
            ctx.io_mut().add_mouse_pos_event(point);
            frame(ctx, editor);
            ctx.io_mut().add_mouse_button_event(MouseButton::Left, true);
            frame(ctx, editor);
            ctx.io_mut()
                .add_mouse_button_event(MouseButton::Left, false);
            frame(ctx, editor);
        };
        // New child windows can need an initial layout frame before their
        // content is considered visible. Stabilize the new document, not clicks.
        frame(context, &mut editor);
        frame(context, &mut editor);
        assert!(
            editor.error.is_none(),
            "Panel fixture must resolve before interaction: {:?}",
            editor.error
        );
        click(context, &mut editor, &format!("bp-component:{root}"));
        let before = bytes(editor.asset.as_ref().unwrap());
        let history = editor.undo.len();
        click(context, &mut editor, "\u{ea60} Add##component");
        click(context, &mut editor, "Canvas");
        assert_eq!(
            bytes(editor.asset.as_ref().unwrap()),
            before,
            "Readding root Canvas preserves its disabled state and child Rect position"
        );
        assert_eq!(
            editor.undo.len(),
            history,
            "No-op component addition creates no undo noise"
        );
        click(context, &mut editor, &format!("bp-component:{child}"));
        assert!(
            editor.details == Details::Template,
            "Selecting a child routes component Details"
        );
        click(context, &mut editor, "\u{ea60} Add##component");
        click(context, &mut editor, "HUD Image");
        assert_eq!(
            bytes(editor.asset.as_ref().unwrap()),
            before,
            "Readding child HUD Image preserves enabled=false and RectTransform values"
        );
        click(context, &mut editor, "\u{ea60} Add##component");
        click(context, &mut editor, "Light");
        assert!(
            editor
                .asset
                .as_ref()
                .unwrap()
                .template
                .actors
                .iter()
                .find(|item| item.entity.id == child)
                .unwrap()
                .entity
                .light
                .is_some(),
            "Add targets selected child, not root"
        );
        assert_eq!(editor.undo.len(), history + 1);
        click(context, &mut editor, "Undo");
        assert_eq!(
            bytes(editor.asset.as_ref().unwrap()),
            before,
            "Undo restores original components and values exactly"
        );
        click(context, &mut editor, "bp-component-search");
        for c in "target".chars() {
            context.io_mut().add_input_character(c);
        }
        frame(context, &mut editor);
        CONTROLS.with(|controls| {
            let controls = controls.borrow();
            assert!(
                controls.contains_key(&format!("bp-component:{root}")),
                "Search preserves ancestor rows"
            );
            assert!(controls.contains_key(&format!("bp-component:{child}")));
            assert!(
                !controls.contains_key(&format!("bp-component:{other}")),
                "Search hides unrelated siblings"
            );
        });
        click(context, &mut editor, "bp-member-search");
        for c in "health".chars() {
            context.io_mut().add_input_character(c);
        }
        frame(context, &mut editor);
        CONTROLS.with(|controls| {
            let controls = controls.borrow();
            assert!(
                controls.contains_key("bp-member:inherited-health"),
                "Member search includes inherited properties"
            );
            assert!(!controls.contains_key("bp-member:local-other"));
        });
        click(context, &mut editor, "bp-member:inherited-health");
        assert!(editor.details == Details::Defaults);
        assert_eq!(editor.detail_member, "inherited-health");
        assert!(
            editor.asset.as_ref().unwrap().defaults.is_empty(),
            "Navigation must not materialize inherited overrides"
        );
        editor.add_node(NodeKind::Return);
        let node = editor.selected.iter().next().unwrap().clone();
        editor
            .asset
            .as_mut()
            .unwrap()
            .layout
            .positions
            .insert(node, [5000., 100.]);
        let clips = frame(context, &mut editor);
        assert!(
            clips
                .iter()
                .all(|clip| clip[2] <= context.io().display_size[0]
                    && clip[3] <= context.io().display_size[1]),
            "Off-canvas node titles must not create unclipped draw commands: {clips:?}"
        );
    }
    fn editor() -> BlueprintEditor {
        let mut e = BlueprintEditor {
            asset: Some(BlueprintAsset::new("BP_Test".into(), "parent".into())),
            ..BlueprintEditor::default()
        };
        e.add_graph("test".into(), None);
        e
    }
    fn rotation_fixture() -> (PathBuf, Registry, BlueprintEditor) {
        let root = std::env::temp_dir().join(format!("epok-bp-diagnostics-{}", id()));
        std::fs::create_dir_all(root.join("assets/scripts")).unwrap();
        let header = root.join("assets/scripts/Parent.hpp");
        std::fs::write(&header, "// reflected fixture\n").unwrap();
        let parent: schema::Class = serde_json::from_value(json!({
            "id": "parent", "cpp_name": "Parent", "parent": crate::object_model::ACTOR3D_ID,
            "abstract_class": false, "final_class": false, "blueprintable": true,
            "properties": [], "functions": [],
            "source": {"file": header, "line": 1, "column": 1}
        }))
        .unwrap();
        let mut registry = crate::actor_document::tests::registry();
        registry.classes.insert("parent".into(), parent);
        let mut e = editor();
        e.path = Some(root.join("assets/Blueprints/BP_Test.epokbp"));
        e.add_node(NodeKind::Builtin {
            operation: asset::Builtin::GetRotation,
        });
        e.add_node(NodeKind::Return);
        e.add_node(NodeKind::Builtin {
            operation: asset::Builtin::SelfObject,
        });
        let graph = &mut e.asset.as_mut().unwrap().functions[0];
        graph.returns = schema::Type::Fixed;
        let rotation = graph.nodes[1].id.clone();
        let exit = graph.nodes[2].id.clone();
        graph.nodes[0].outputs.insert("next".into(), vec![exit]);
        graph.nodes[2].inputs.insert(
            "value".into(),
            Input::Link {
                node: rotation,
                pin: "value.x".into(),
            },
        );
        (root, registry, e)
    }
    #[test]
    fn compile_diagnostics_locate_missing_target_and_clear_after_repair() {
        let (root, registry, mut e) = rotation_fixture();
        e.save().unwrap();
        assert!(!e.compile(&root, &registry));
        assert!(!e.compile_stale);
        assert!(
            e.diagnostics[0].contains("BP_Test / Test / Get Rotation"),
            "{:?}",
            e.diagnostics
        );
        assert!(e.diagnostics[0].contains("Target"));
        e.focus_diagnostic(0);
        let graph = e.current().unwrap();
        assert!(e.selected.contains(&graph.nodes[1].id));
        let before = e.asset.clone().unwrap();
        let graph = &mut e.asset.as_mut().unwrap().functions[0];
        let receiver = graph.nodes[3].id.clone();
        graph.nodes[1].inputs.insert(
            "target".into(),
            Input::Link {
                node: receiver,
                pin: "value".into(),
            },
        );
        e.checkpoint(before);
        assert!(e.compile_stale);
        // Compiling uses the unsaved canvas, even though the saved asset is still broken.
        assert!(e.compile(&root, &registry), "{:?}", e.diagnostics);
        assert!(e.diagnostic_nodes.is_empty());
        assert!(e.generated.contains("epok::bp::api::rotation"));
        e.undo();
        assert!(e.compile_stale);
        assert!(!e.compile(&root, &registry));
        e.redo();
        assert!(e.compile_stale);
        e.save().unwrap();
        assert!(e.compile(&root, &registry), "{:?}", e.diagnostics);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "Owns a real ImGui context; run explicitly and serially"]
    fn compile_feedback_and_build_options_use_the_blueprint_window() {
        let (root, registry, mut e) = rotation_fixture();
        e.open = true;
        e.save().unwrap();
        assert!(!e.compile(&root, &registry));
        let mut context = crate::gui::tests::imgui_context();
        context.set_ini_filename(None);
        context.io_mut().display_size = [1280., 850.];
        context.io_mut().delta_time = 1. / 60.;
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        let mut options_drawn = false;
        let mut frame = |ctx: &mut imgui::Context, e: &mut BlueprintEditor| {
            e.draw_with_options(ctx.frame(), &registry, |ui| {
                options_drawn = true;
                ui.text("Instrument Blueprint Debugger");
                assert!(ui.is_window_focused_with_flags(
                    imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS
                ));
            });
            ctx.render();
        };
        frame(&mut context, &mut e);
        for control in ["Show error", "Build Options"] {
            let point = CONTROLS.with(|c| c.borrow()[control]);
            assert!(
                point[1] > 0. && point[1] < 850.,
                "{control} must be visible"
            );
            context.io_mut().add_mouse_pos_event(point);
            frame(&mut context, &mut e);
            context
                .io_mut()
                .add_mouse_button_event(MouseButton::Left, true);
            frame(&mut context, &mut e);
            context
                .io_mut()
                .add_mouse_button_event(MouseButton::Left, false);
            frame(&mut context, &mut e);
        }
        assert!(options_drawn);
        assert!(e.selected.contains(&e.current().unwrap().nodes[1].id));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn visual_labels_humanize_identifiers_without_changing_source_names() {
        assert_eq!(display_port("enable_input"), "Enable Input");
        assert_eq!(display_port("SelfObject"), "Self Object");
        assert_eq!(display_port("SetAudioClip"), "Set Audio Clip");
        assert_eq!(display_port("HTTPServer"), "HTTP Server");
        assert_eq!(display_port("next"), "");
        assert_eq!(display_port("__target"), "Target");
    }
    #[test]
    fn inline_literals_reserve_space_before_same_row_outputs() {
        for chip in [13., 24., 86.] {
            let width = socket_row_width(40., Some(chip), 32.).max(126.);
            let literal_end = 40. + 27. + chip;
            let output_start = width - 32. - 20.;
            assert!(
                output_start - literal_end >= 12.,
                "Literal and output need visible separation"
            );
            assert_eq!(
                socket_row_width(40., Some(chip), 32.) - socket_row_width(40., None, 32.),
                chip + 7.
            );
        }
        assert!(
            socket_row_width(200., Some(100.), 150.) > 300.,
            "Long pin rows must not be collapsed by the header cap"
        );
        assert_eq!(literal_display(&json!(1.0)), "1.0");
        assert!(literal_display(&json!("long literal contents")).ends_with("..."));
    }
    #[test]
    fn vector_construction_and_component_sockets_are_typed_pure_values() {
        let editor = editor();
        let doc = editor.asset.as_ref().unwrap();
        let graph = &doc.functions[0];
        let registry = Registry::new();
        for length in [2, 3] {
            let make = Node {
                id: "make".into(),
                kind: NodeKind::MakeVector { length },
                inputs: BTreeMap::new(),
                outputs: BTreeMap::new(),
            };
            let pins = node_sockets(doc, graph, &make, &registry);
            assert_eq!(pins.iter().filter(|pin| !pin.output).count(), length);
            assert!(pins.iter().all(|pin| pin.ty != SocketType::Exec));
            assert_eq!(
                output_type(doc, graph, &make, &registry, 0),
                Some(schema::Type::Vector { length })
            );
            let component = Node {
                id: "component".into(),
                kind: NodeKind::VectorComponent { length, index: 0 },
                inputs: BTreeMap::new(),
                outputs: BTreeMap::new(),
            };
            let pins = node_sockets(doc, graph, &component, &registry);
            assert!(
                pins.iter().any(|pin| !pin.output
                    && pin.ty == SocketType::Value(schema::Type::Vector { length }))
            );
            assert_eq!(
                output_type(doc, graph, &component, &registry, 0),
                Some(schema::Type::Fixed)
            );
        }
    }
    #[test]
    fn captured_parameters_are_visible_connections_and_disconnect_with_undo() {
        let mut e = editor();
        e.asset.as_mut().unwrap().functions[0]
            .parameters
            .push(schema::Parameter {
                name: "power".into(),
                value_type: schema::Type::Fixed,
                direction: schema::Direction::Value,
            });
        e.add_node(NodeKind::Delay);
        e.asset.as_mut().unwrap().functions[0].nodes[1]
            .inputs
            .insert(
                "seconds".into(),
                Input::Parameter {
                    name: "power".into(),
                },
            );
        let doc = e.asset.as_ref().unwrap();
        let graph = &doc.functions[0];
        let registry = Registry::new();
        let output = node_sockets(doc, graph, &graph.nodes[0], &registry)
            .into_iter()
            .find(|pin| pin.pin == "power")
            .unwrap();
        let input = node_sockets(doc, graph, &graph.nodes[1], &registry)
            .into_iter()
            .find(|pin| pin.pin == "seconds")
            .unwrap();
        assert!(socket_connected(graph, &graph.nodes[0], &output));
        assert!(socket_connected(graph, &graph.nodes[1], &input));
        let before = doc.clone();
        e.disconnect(&output);
        e.checkpoint(before.clone());
        assert!(e.current().unwrap().nodes[1].inputs.is_empty());
        e.undo();
        assert_eq!(bytes(e.asset.as_ref().unwrap()), bytes(&before));
    }
    #[test]
    fn playback_asset_pins_follow_slot_ids_and_subscription_edits_are_undoable() {
        use uuid::Uuid;
        let mut e = editor();
        let registry = Registry::new();
        let mut timeline = crate::timeline::TimelineAsset::new("Spell".into());
        let slot = crate::timeline::Slot {
            id: Uuid::new_v4(),
            name: "Caster".into(),
            required: true,
            target: schema::Type::ObjectRef {
                class: Some("caster".into()),
            },
            extra: Default::default(),
        };
        let mut optional = slot.clone();
        optional.id = Uuid::new_v4();
        optional.name = "Target".into();
        optional.required = false;
        timeline.slots = vec![slot.clone(), optional];
        e.add_node(NodeKind::Builtin {
            operation: asset::Builtin::PlayTimelineAsset {
                asset: timeline.id.to_string(),
            },
        });
        let pins = |e: &BlueprintEditor, source: &crate::timeline::TimelineAsset| {
            let doc = e.asset.as_ref().unwrap();
            let graph = &doc.functions[0];
            node_sockets_with_assets(
                doc,
                graph,
                &graph.nodes[1],
                &registry,
                &[(PathBuf::from("Spell.timeline.json"), source.clone())],
                &[],
            )
        };
        let first = pins(&e, &timeline);
        let binding = first
            .iter()
            .find(|pin| pin.pin == crate::blueprint_playback::pin(&slot))
            .unwrap();
        assert_eq!(binding.label, "Caster (required)");
        assert_eq!(binding.ty, SocketType::Value(slot.target.clone()));
        timeline.slots[0].name = "Renamed caster".into();
        timeline.slots.reverse();
        let reordered = pins(&e, &timeline);
        assert_eq!(
            first.iter().map(|pin| &pin.pin).collect::<Vec<_>>(),
            reordered.iter().map(|pin| &pin.pin).collect::<Vec<_>>()
        );
        assert!(
            reordered
                .iter()
                .any(|pin| pin.pin == binding.pin && pin.label == "Renamed caster (required)")
        );
        let before = e.asset.clone().unwrap();
        e.add_node(NodeKind::WaitPlayback {
            condition: asset::PlaybackCondition::SubscribeMarker {
                timeline: timeline.id.to_string(),
                marker: Uuid::new_v4().to_string(),
            },
        });
        let after = e.asset.clone().unwrap();
        e.checkpoint(before.clone());
        e.undo();
        assert_eq!(bytes(e.asset.as_ref().unwrap()), bytes(&before));
        e.redo();
        assert_eq!(bytes(e.asset.as_ref().unwrap()), bytes(&after));
        let graph = &after.functions[0];
        let sockets = node_sockets(&after, graph, &graph.nodes[2], &registry);
        assert!(sockets.iter().any(|pin| pin.pin == "reached" && pin.output));
        assert!(sockets.iter().any(|pin| pin.pin == "playback"
            && pin.ty == SocketType::Value(schema::Type::SequenceHandle)));
    }
    #[test]
    fn foreign_calls_use_compiler_signatures_and_typed_receiver_pins() {
        let mut registry = Registry::new();
        let source = schema::Location {
            file: "Target.hpp".into(),
            line: 1,
            column: 1,
        };
        let function = schema::Function {
            id: "target-function".into(),
            name: "read_health".into(),
            parameters: vec![schema::Parameter {
                name: "amount".into(),
                value_type: schema::Type::Fixed,
                direction: schema::Direction::Value,
            }],
            returns: schema::Type::Int32,
            callable: true,
            timeline: None,
            event: false,
            pure: false,
            resource_demands: vec![],
            abstract_method: false,
            final_method: false,
            access: "public".into(),
            overrides: vec![],
            source: source.clone(),
        };
        registry.classes.insert(
            "target".into(),
            schema::Class {
                family: None,
                domain: None,
                placement: Default::default(),
                component: None,
                default_components: vec![],
                explicit_abstract: false,
                id: "target".into(),
                provider: schema::native_provider(),
                backend: schema::native_backend(),
                cpp_name: "Target".into(),
                parent: None,
                abstract_class: false,
                final_class: false,
                timeline_component: None,
                blueprintable: true,
                properties: vec![],
                functions: vec![function],
                source,
            },
        );
        let editor = editor();
        let doc = editor.asset.as_ref().unwrap();
        let graph = &doc.functions[0];
        let node = Node {
            id: "call".into(),
            kind: NodeKind::CallOn {
                class: "target".into(),
                function: "target-function".into(),
            },
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        };
        for pure in [false, true] {
            registry.classes.get_mut("target").unwrap().functions[0].pure = pure;
            let pins = node_sockets(doc, graph, &node, &registry);
            assert_eq!(pins.iter().any(|pin| pin.ty == SocketType::Exec), !pure);
            assert!(pins.iter().any(|pin| pin.pin == "__target"
                && pin.ty
                    == SocketType::Value(schema::Type::ObjectRef {
                        class: Some("target".into())
                    })
                && !pin.output));
            assert!(
                pins.iter()
                    .any(|pin| pin.pin == "amount"
                        && pin.ty == SocketType::Value(schema::Type::Fixed))
            );
            assert!(pins.iter().any(|pin| pin.pin == "value"
                && pin.output
                && pin.ty == SocketType::Value(schema::Type::Int32)));
            assert_eq!(target_functions(&registry, "target").len(), 1);
            assert_eq!(node_label(&node, doc, &registry), "Read Health");
        }
        registry.classes.get_mut("target").unwrap().functions[0].access = "private".into();
        assert!(target_functions(&registry, "target").is_empty());
        assert_eq!(node_label(&node, doc, &registry), "Missing target function");
    }
    #[test]
    fn event_entries_have_no_inputs_and_execution_rewiring_replaces_the_target() {
        let mut e = editor();
        let registry = Registry::new();
        e.add_node(NodeKind::Sequence);
        e.add_node(NodeKind::Sequence);
        let doc = e.asset.as_ref().unwrap();
        let graph = e.current().unwrap();
        let entry = node_sockets(doc, graph, &graph.nodes[0], &registry);
        assert!(entry.iter().all(|s| s.output));
        let source = entry
            .iter()
            .find(|s| s.ty == SocketType::Exec)
            .unwrap()
            .clone();
        let a = node_sockets(doc, graph, &graph.nodes[1], &registry)
            .into_iter()
            .find(|s| !s.output && s.ty == SocketType::Exec)
            .unwrap();
        let b = node_sockets(doc, graph, &graph.nodes[2], &registry)
            .into_iter()
            .find(|s| !s.output && s.ty == SocketType::Exec)
            .unwrap();
        e.connect(&source, &a, &registry).unwrap();
        let before = e.asset.clone().unwrap();
        e.connect(&source, &b, &registry).unwrap();
        e.checkpoint(before);
        assert_eq!(e.current().unwrap().nodes[0].outputs["next"], vec![b.node]);
        e.undo();
        assert_eq!(e.current().unwrap().nodes[0].outputs["next"], vec![a.node]);
        let mut input = source.clone();
        input.output = false;
        input.pin = "exec".into();
        let output = node_sockets(
            e.asset.as_ref().unwrap(),
            e.current().unwrap(),
            &e.current().unwrap().nodes[2],
            &registry,
        )
        .into_iter()
        .find(|s| s.output)
        .unwrap();
        assert!(e.connect(&output, &input, &registry).is_err());
    }
    #[test]
    fn copy_remaps_internal_links_and_excludes_entry() {
        let mut e = editor();
        e.add_node(NodeKind::Sequence);
        let a = e.current().unwrap().nodes[1].id.clone();
        e.add_node(NodeKind::Return);
        let b = e.current().unwrap().nodes[2].id.clone();
        e.asset.as_mut().unwrap().functions[0].nodes[1]
            .outputs
            .insert("next".into(), vec![b.clone()]);
        e.selected = BTreeSet::from([a, b]);
        e.copy_selected();
        e.paste();
        let g = e.current().unwrap();
        assert_eq!(g.nodes.len(), 5);
        assert_eq!(g.nodes[3].outputs["next"], vec![g.nodes[4].id.clone()]);
        assert_ne!(g.nodes[3].id, g.nodes[1].id);
    }
    #[test]
    fn deletion_explicitly_removes_links_and_undo_restores_them() {
        let mut e = editor();
        e.add_node(NodeKind::Return);
        let key = e.current().unwrap().nodes[1].id.clone();
        e.asset.as_mut().unwrap().functions[0].nodes[0]
            .outputs
            .insert("next".into(), vec![key.clone()]);
        let before = e.asset.clone().unwrap();
        e.selected = BTreeSet::from([key]);
        e.delete_selected();
        e.checkpoint(before);
        assert!(e.current().unwrap().nodes[0].outputs["next"].is_empty());
        e.undo();
        assert_eq!(e.current().unwrap().nodes.len(), 2);
        e.redo();
        assert_eq!(e.current().unwrap().nodes.len(), 1);
    }
    #[test]
    fn connections_reject_type_mismatch_and_cycles() {
        let mut e = editor();
        e.add_node(NodeKind::Sequence);
        e.add_node(NodeKind::Sequence);
        let nodes = e.current().unwrap().nodes.clone();
        let s = |index: usize, output, pin: &str| Socket {
            label: display_port(pin),
            node: nodes[index].id.clone(),
            pin: pin.into(),
            output,
            ty: SocketType::Exec,
        };
        let registry = Registry::new();
        e.connect(&s(1, true, "then_0"), &s(2, false, "exec"), &registry)
            .unwrap();
        assert!(
            e.connect(&s(2, true, "then_0"), &s(1, false, "exec"), &registry)
                .is_err()
        );
        let mut wrong = s(2, false, "exec");
        wrong.ty = SocketType::Value(schema::Type::Bool);
        assert!(e.connect(&s(1, true, "then_0"), &wrong, &registry).is_err());
    }
    #[test]
    fn unbound_numeric_and_reroute_sockets_infer_without_coercion() {
        let mut e = editor();
        let registry = Registry::new();
        e.add_node(NodeKind::Literal {
            value_type: schema::Type::Int32,
            value: json!(3),
        });
        e.add_node(NodeKind::Binary {
            op: asset::BinaryOp::Add,
        });
        let doc = e.asset.as_ref().unwrap();
        let graph = e.current().unwrap();
        let from = node_sockets(doc, graph, &graph.nodes[1], &registry)
            .into_iter()
            .find(|s| s.output)
            .unwrap();
        let to = node_sockets(doc, graph, &graph.nodes[2], &registry)
            .into_iter()
            .find(|s| s.pin == "a")
            .unwrap();
        e.connect(&from, &to, &registry).unwrap();
        let doc = e.asset.as_ref().unwrap();
        let graph = e.current().unwrap();
        let sockets = node_sockets(doc, graph, &graph.nodes[2], &registry);
        assert!(
            sockets
                .iter()
                .all(|s| s.ty == SocketType::Value(schema::Type::Int32))
        );
    }
    #[test]
    fn playback_picker_keeps_independent_choices_and_rejects_ambiguous_ids() {
        let root = std::env::temp_dir().join(format!("epok-playback-picker-{}", id()));
        std::fs::create_dir_all(root.join("assets/Timelines")).unwrap();
        let source = crate::timeline::TimelineAsset::new("Gate".into());
        let path = root.join("assets/Timelines/Gate.timeline.json");
        std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
        let broken = root.join("assets/Timelines/Broken.timeline.json");
        std::fs::write(&broken, b"{").unwrap();
        let mut e = editor();
        e.add_node(NodeKind::Builtin {
            operation: asset::Builtin::PlayTimelineAsset {
                asset: source.id.to_string(),
            },
        });
        let original = bytes(e.asset.as_ref().unwrap());
        e.compile_requested = false;
        e.refresh_playback_sources(&root);
        assert!(e.compile_requested);
        assert_eq!(e.playback_timelines.len(), 1);
        assert_eq!(e.playback_timelines[0].1.id, source.id);
        assert!(
            e.playback_error
                .as_ref()
                .unwrap()
                .contains("Broken.timeline.json")
        );
        e.compile_requested = false;
        e.refresh_playback_sources(&root);
        assert!(
            !e.compile_requested,
            "Unchanged catalog errors do not recompile the canvas repeatedly"
        );
        std::fs::write(&broken, serde_json::to_vec(&source).unwrap()).unwrap();
        e.refresh_playback_sources(&root);
        assert!(e.playback_timelines.is_empty());
        assert!(e.compile_requested && !e.compile_valid);
        assert!(
            e.playback_error
                .as_ref()
                .unwrap()
                .contains(&source.id.to_string())
        );
        std::fs::remove_file(&broken).unwrap();
        e.refresh_playback_sources(&root);
        assert_eq!(e.playback_timelines.len(), 1);
        assert!(e.playback_error.is_none());
        assert_eq!(
            bytes(e.asset.as_ref().unwrap()),
            original,
            "Catalog refresh never rewrites authored nodes"
        );
    }
    #[test]
    fn save_refuses_external_edits_and_revert_is_undoable() {
        let dir = std::env::temp_dir().join(format!("epok-bp-editor-{}", id()));
        let path = dir.join("Test.epokbp");
        let mut e = editor();
        e.path = Some(path.clone());
        e.save().unwrap();
        let original = std::fs::read(&path).unwrap();
        let mut changed = e.asset.clone().unwrap();
        changed.name = "External".into();
        std::fs::write(&path, bytes(&changed)).unwrap();
        assert!(e.save().is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes(&changed));
        e.revert().unwrap();
        assert_eq!(e.asset.as_ref().unwrap().name, "External");
        e.undo();
        assert_eq!(bytes(e.asset.as_ref().unwrap()), original);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
