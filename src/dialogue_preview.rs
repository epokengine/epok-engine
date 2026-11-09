//! Isolated preview of real conversations through the native session and widget
//! binder. No project class, condition or action implementation executes here.
use crate::{
    assets::Index,
    blueprint::Registry,
    dialogue_asset as asset, dialogue_compile as graph, dialogue_layout,
    hud_native::{self, Command},
    scene::Scene,
};
use serde::Serialize;
use std::{collections::BTreeMap, ffi::CString};
use uuid::Uuid;

pub const CONFIRM: u32 = 1 << 14;
pub const CANCEL: u32 = 1 << 13;
pub const UP: u32 = 1 << 4;
pub const DOWN: u32 = 1 << 6;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Frame {
    pub buttons: u32,
    pub microseconds: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub phase: u32,
    pub result: u32,
    pub start_result: u32,
    pub node: u32,
    pub revealed: u32,
    pub glyphs: u32,
    pub page: u32,
    pub pages: u32,
    pub choices: u32,
    pub choice_page: u32,
    pub selected: u32,
    pub trace_count: u32,
}
impl Snapshot {
    pub fn result_name(self) -> &'static str {
        [
            "Accepted",
            "Busy",
            "Invalid owner",
            "Invalid binding",
            "Invalid data",
            "Missing presenter",
            "Capture pool full",
            "No session",
            "Completed",
            "Cancelled",
            "Source unavailable",
            "Presenter unavailable",
            "Level unloaded",
            "Transition budget exhausted",
            "No choices",
            "Capture lost",
            "Generation exhausted",
            "Unavailable",
            "Out of range",
            "Domain mismatch",
        ]
        .get(self.result as usize)
        .copied()
        .unwrap_or("Invalid result")
    }
    pub fn phase_name(self) -> &'static str {
        [
            "Idle",
            "Revealing",
            "Waiting",
            "Selecting",
            "Executing",
            "Completed",
            "Cancelled",
            "Failed",
        ]
        .get(self.phase as usize)
        .copied()
        .unwrap_or("Invalid")
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Trace {
    pub kind: u32,
    pub id: u32,
    pub node: u32,
}
#[derive(Default, Clone, Serialize)]
pub struct State {
    epoch: u64,
    pub asset: Option<Uuid>,
    pub style: Option<Uuid>,
    pub entry: Option<Uuid>,
    pub participant: Option<Uuid>,
    pub conditions: BTreeMap<Uuid, bool>,
    pub frames: Vec<Frame>,
    #[serde(skip)]
    pub snapshot: Option<Snapshot>,
    #[serde(skip)]
    pub trace: Vec<Trace>,
    #[serde(skip)]
    pub condition_labels: Vec<(Uuid, String)>,
    #[serde(skip)]
    pub node_labels: Vec<(Uuid, String)>,
    #[serde(skip)]
    pub participant_labels: Vec<(Uuid, String)>,
    #[serde(skip)]
    pub node_speakers: BTreeMap<Uuid, Option<Uuid>>,
    #[serde(skip)]
    pub trace_labels: BTreeMap<u32, String>,
    #[serde(skip)]
    pub languages: Vec<String>,
    #[serde(skip)]
    context: Option<String>,
}
impl State {
    pub fn restart(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.frames.clear();
        self.snapshot = None;
        self.trace.clear();
    }
    pub fn select(&mut self, asset: Option<Uuid>) {
        if self.asset != asset {
            let style = self.style;
            *self = Self {
                asset,
                style,
                ..Self::default()
            };
        }
    }
    pub fn press(&mut self, button: u32) {
        self.frames.extend([
            Frame {
                buttons: button,
                microseconds: 16_667,
            },
            Frame {
                buttons: 0,
                microseconds: 16_667,
            },
        ]);
    }
    pub fn step(&mut self, microseconds: u32) {
        self.frames.push(Frame {
            buttons: 0,
            microseconds,
        });
    }
    pub fn input(&mut self, sequence: &str) -> Result<(), String> {
        let mut staged = Self::default();
        for value in sequence.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            match value {
                "confirm" => staged.press(CONFIRM),
                "cancel" => staged.press(CANCEL),
                "up" => staged.press(UP),
                "down" => staged.press(DOWN),
                _ => {
                    let ms = value.strip_prefix("step:").and_then(|s| s.parse::<u32>().ok())
                        .and_then(|ms| ms.checked_mul(1000)).ok_or_else(|| format!("Invalid preview input '{value}'; use confirm, up, down, cancel or step:MILLISECONDS"))?;
                    staged.step(ms);
                }
            }
        }
        self.frames.extend(staged.frames);
        Ok(())
    }
}

#[cfg(test)]
#[path = "dialogue_preview_tests.rs"]
pub(crate) mod tests;

#[repr(C)]
struct Page {
    text: *const std::ffi::c_char,
    glyphs: u32,
}
#[repr(C)]
struct Text {
    first: u32,
    count: u32,
}
#[repr(C)]
struct Role {
    id: u32,
    name: i32,
    portrait: i32,
}
#[repr(C)]
struct Binding {
    id: u32,
    role: u32,
    kind: u32,
    value: u32,
}
#[repr(C)]
struct OptionRow {
    id: u32,
    text: i32,
    next: i32,
    binding: i32,
}
#[repr(C)]
struct Node {
    id: u32,
    kind: u32,
    speaker: i32,
    text: i32,
    next: i32,
    otherwise: i32,
    binding: i32,
    first_option: u32,
    option_count: u32,
    reveal_ms: u32,
    auto_ms: u32,
}
#[repr(C)]
struct Graph {
    roles: *const Role,
    role_count: u32,
    bindings: *const Binding,
    binding_count: u32,
    pages: *const Page,
    page_count: u32,
    texts: *const Text,
    text_count: u32,
    options: *const OptionRow,
    option_count: u32,
    nodes: *const Node,
    node_count: u32,
    entry: u32,
}
#[repr(C)]
struct Request {
    graph: Graph,
    nodes: *const hud_native::Node,
    count: u32,
    root: u32,
    components: *const u32,
    logical_parents: *const i32,
    controls: [i32; 11],
    frames: *const Frame,
    frame_count: u32,
    reveal_ms: u32,
    auto_ms: u32,
    dimensions: *const i32,
    texture_count: u32,
    width: i32,
    height: i32,
    budget: *const u32,
    fonts: *const i32,
    font_words: u32,
}
unsafe extern "C" {
    fn epok_dialogue_preview(
        input: *const Request,
        output: *mut Command,
        capacity: u32,
        stats: *mut u32,
        snapshot: *mut Snapshot,
        trace: *mut Trace,
        trace_capacity: u32,
        error: *mut std::ffi::c_char,
        error_capacity: u32,
    ) -> u32;
}
pub struct Rendered {
    pub pixels: Vec<u8>,
    #[cfg(test)]
    pub commands: Vec<Command>,
    pub stats: [u32; 6],
    pub texture_count: usize,
    pub font_count: usize,
}

pub fn render(
    scene: &mut Scene,
    registry: &Registry,
    index: &Index,
    class: &str,
    state: &mut State,
) -> Result<Rendered, String> {
    state.snapshot = None;
    state.trace.clear();
    let id = state.asset.ok_or("Select a conversation to preview")?;
    let document = asset::document(index.resolve(id)?)?;
    let compiled = graph::compile(id, &document, registry, index)?;
    state.node_speakers = compiled
        .nodes
        .iter()
        .map(|node| {
            let role = match node.kind {
                graph::NodeKind::Line { speaker, .. } | graph::NodeKind::Choice { speaker, .. } => {
                    speaker
                }
                _ => None,
            };
            (
                node.source,
                role.map(|i| compiled.roles[usize::from(i)].source),
            )
        })
        .collect();
    state.participant_labels = document
        .participants
        .iter()
        .filter(|p| state.node_speakers.values().any(|s| *s == Some(p.id)))
        .map(|p| (p.id, p.label.clone()))
        .collect();
    if let Some(participant) = state.participant
        && state.entry.is_none()
    {
        state.entry = compiled
            .nodes
            .iter()
            .find(|n| state.node_speakers[&n.source] == Some(participant))
            .map(|n| n.source);
        if state.entry.is_none() {
            return Err("This preview participant has no line or choice node".into());
        }
    }
    state.condition_labels = compiled
        .bindings
        .iter()
        .filter(|b| b.kind == asset::BindingKind::Condition)
        .map(|b| {
            (
                b.source,
                document
                    .bindings
                    .iter()
                    .find(|v| v.id == b.source)
                    .unwrap()
                    .label
                    .clone(),
            )
        })
        .collect();
    state.node_labels = compiled
        .nodes
        .iter()
        .map(|node| {
            let kind = match node.kind {
                graph::NodeKind::Line { .. } => "Line",
                graph::NodeKind::Choice { .. } => "Choice",
                graph::NodeKind::Branch { .. } => "Branch",
                graph::NodeKind::Event { .. } => "Event",
                graph::NodeKind::End => "End",
            };
            (
                node.source,
                format!("{kind} {}", &node.source.to_string()[..8]),
            )
        })
        .collect();
    state.trace_labels = compiled
        .bindings
        .iter()
        .map(|b| {
            (
                b.id,
                document
                    .bindings
                    .iter()
                    .find(|v| v.id == b.source)
                    .unwrap()
                    .label
                    .clone(),
            )
        })
        .chain(
            compiled
                .nodes
                .iter()
                .zip(&state.node_labels)
                .map(|(n, (_, label))| (n.id, label.clone())),
        )
        .collect();
    let root = scene
        .actors
        .iter()
        .position(|a| a.class.class_id.as_deref() == Some(class))
        .ok_or("Preview widget root is missing")?;
    let profile = dialogue_layout::Profile::read(scene, registry, root)?
        .ok_or("The selected class has no dialogue presentation bindings")?;
    let locales = crate::dialogue_cook::Locales::prepare(
        index,
        &compiled.texts.iter().map(|t| t.table).collect(),
    )?;
    state.languages = locales.names.clone();
    let language = scene.preview_language;
    if language >= locales.names.len() {
        return Err("Preview language is missing from the conversation's string tables".into());
    }
    let cooked = profile.texts(&compiled, &locales, &scene.fonts)?;
    let mut style = state
        .style
        .map(|id| asset::style_document(index.resolve(id)?))
        .transpose()?
        .unwrap_or_default();
    if state.style.is_none() {
        let actor = &scene.actors[root];
        let timing =
            dialogue_layout::property(registry, &actor.class, &actor.properties, "default_timing");
        let value = |name| {
            timing
                .and_then(|t| t.get(name))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0)
        };
        let reveal = value("reveal_ms");
        let automatic = value("auto_advance_ms");
        if reveal > 1000 || automatic > 3_600_000 {
            return Err("Widget default timing exceeds supported limits".into());
        }
        style.line_policy = asset::LinePolicy {
            milliseconds_per_glyph: reveal as u16,
            auto_advance_ms: (automatic != 0).then_some(automatic as u32),
        };
    }
    let page_signature = cooked
        .iter()
        .map(|t| {
            (
                &t.offsets,
                t.pages
                    .iter()
                    .map(|p| (&p.text, p.glyphs))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let context = crate::scene_dependencies::hash((
        &scene,
        &document,
        &style,
        &profile,
        &locales.dependencies,
        language,
        page_signature,
        &state.conditions,
        state.entry,
    ));
    if state.context.as_ref().is_some_and(|old| old != &context) {
        state.restart();
    }
    state.context = Some(context);
    for (id, label) in &state.condition_labels {
        if !state.conditions.contains_key(id) {
            return Err(format!("Choose a preview value for condition '{label}'"));
        }
    }
    let mut texture_ids = crate::texture::ids(scene)
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    for portrait in compiled.roles.iter().filter_map(|r| r.portrait) {
        texture_ids.insert(portrait);
        if let std::collections::btree_map::Entry::Vacant(entry) = scene.textures.entry(portrait) {
            entry.insert(crate::texture::load_asset(index, portrait)?);
        }
    }
    let texture_ids = texture_ids.into_iter().collect::<Vec<_>>();
    let dimensions = texture_ids
        .iter()
        .flat_map(|id| {
            scene
                .textures
                .get(id)
                .map_or([0, 0], |t| [i32::from(t.width), i32::from(t.height)])
        })
        .collect::<Vec<_>>();
    let font_ids = crate::hud::font_ids(scene);
    let fonts = hud_native::font_table(scene, &font_ids);
    let wire_nodes = hud_native::nodes(scene, &texture_ids, &font_ids);
    let component_masks = scene
        .actors
        .iter()
        .map(|a| {
            u32::from(a.text.is_some())
                | (u32::from(a.image.is_some()) << 1)
                | (u32::from(a.focusable.is_some()) << 2)
        })
        .collect::<Vec<_>>();
    let logical_parents = scene
        .actors
        .iter()
        .map(|a| {
            a.logical_parent
                .and_then(|id| scene.actors.iter().position(|p| p.id == id))
                .map_or(-1, |i| i as i32)
        })
        .collect::<Vec<_>>();
    let control_bindings = dialogue_layout::bindings(scene, registry, root);
    let controls =
        dialogue_layout::FIELDS.map(|name| control_bindings.get(name).map_or(-1, |&i| i as i32));
    let mut strings = vec![];
    let mut page_glyphs = vec![];
    let mut texts = vec![];
    for text in &cooked {
        let start = usize::from(text.offsets[language]);
        let end = usize::from(text.offsets[language + 1]);
        texts.push(Text {
            first: strings.len() as u32,
            count: (end - start) as u32,
        });
        for page in &text.pages[start..end] {
            strings.push(
                CString::new(page.text.as_bytes())
                    .map_err(|_| "Dialogue preview text contains a NUL byte")?,
            );
            page_glyphs.push(u32::from(page.glyphs));
        }
    }
    let pages = strings
        .iter()
        .zip(page_glyphs)
        .map(|(s, glyphs)| Page {
            text: s.as_ptr(),
            glyphs,
        })
        .collect::<Vec<_>>();
    let roles = compiled
        .roles
        .iter()
        .map(|r| Role {
            id: r.id,
            name: r.display_name.map_or(-1, i32::from),
            portrait: r
                .portrait
                .and_then(|id| texture_ids.iter().position(|v| *v == id))
                .map_or(-1, |i| i as i32),
        })
        .collect::<Vec<_>>();
    let bindings = compiled
        .bindings
        .iter()
        .map(|b| Binding {
            id: b.id,
            role: u32::from(b.role),
            kind: u32::from(b.kind == asset::BindingKind::Action),
            value: u32::from(state.conditions.get(&b.source).copied().unwrap_or(false)),
        })
        .collect::<Vec<_>>();
    let mut options = vec![];
    let nodes = compiled
        .nodes
        .iter()
        .map(|node| {
            let mut out = Node {
                id: node.id,
                kind: 4,
                speaker: -1,
                text: -1,
                next: -1,
                otherwise: -1,
                binding: -1,
                first_option: options.len() as u32,
                option_count: 0,
                reveal_ms: 65535,
                auto_ms: u32::MAX,
            };
            match &node.kind {
                graph::NodeKind::Line {
                    speaker,
                    text,
                    next,
                    policy,
                } => {
                    out.kind = 0;
                    out.speaker = speaker.map_or(-1, i32::from);
                    out.text = i32::from(*text);
                    out.next = i32::from(*next);
                    if let Some(policy) = policy {
                        out.reveal_ms = u32::from(policy.milliseconds_per_glyph);
                        out.auto_ms = policy.auto_advance_ms.unwrap_or(0);
                    }
                }
                graph::NodeKind::Choice {
                    speaker,
                    prompt,
                    options: rows,
                } => {
                    out.kind = 1;
                    out.speaker = speaker.map_or(-1, i32::from);
                    out.text = prompt.map_or(-1, i32::from);
                    out.option_count = rows.len() as u32;
                    options.extend(rows.iter().map(|o| OptionRow {
                        id: o.id,
                        text: i32::from(o.text),
                        next: i32::from(o.next),
                        binding: o.condition.map_or(-1, i32::from),
                    }));
                }
                graph::NodeKind::Branch { condition, yes, no } => {
                    out.kind = 2;
                    out.binding = i32::from(*condition);
                    out.next = i32::from(*yes);
                    out.otherwise = i32::from(*no);
                }
                graph::NodeKind::Event { action, next } => {
                    out.kind = 3;
                    out.binding = i32::from(*action);
                    out.next = i32::from(*next);
                }
                graph::NodeKind::End => {}
            }
            out
        })
        .collect::<Vec<_>>();
    let entry = state
        .entry
        .map(|id| {
            compiled
                .nodes
                .iter()
                .position(|n| n.source == id)
                .ok_or("Preview start node no longer exists")
        })
        .transpose()?
        .unwrap_or(usize::from(compiled.entry));
    let b = &scene.hud_budget;
    let budget = [
        b.layouts as u32,
        b.rectangles as u32,
        b.texts as u32,
        b.glyphs as u32,
        b.rotated as u32,
    ];
    let request = Request {
        graph: Graph {
            roles: roles.as_ptr(),
            role_count: roles.len() as u32,
            bindings: bindings.as_ptr(),
            binding_count: bindings.len() as u32,
            pages: pages.as_ptr(),
            page_count: pages.len() as u32,
            texts: texts.as_ptr(),
            text_count: texts.len() as u32,
            options: options.as_ptr(),
            option_count: options.len() as u32,
            nodes: nodes.as_ptr(),
            node_count: nodes.len() as u32,
            entry: entry as u32,
        },
        nodes: wire_nodes.as_ptr(),
        count: wire_nodes.len() as u32,
        root: root as u32,
        components: component_masks.as_ptr(),
        logical_parents: logical_parents.as_ptr(),
        controls,
        frames: state.frames.as_ptr(),
        frame_count: state.frames.len() as u32,
        reveal_ms: u32::from(style.line_policy.milliseconds_per_glyph),
        auto_ms: style.line_policy.auto_advance_ms.unwrap_or(0),
        dimensions: dimensions.as_ptr(),
        texture_count: texture_ids.len() as u32,
        width: i32::from(scene.display_size[0]),
        height: i32::from(scene.display_size[1]),
        budget: budget.as_ptr(),
        fonts: fonts.as_ptr(),
        font_words: fonts.len() as u32,
    };
    let mut commands = vec![Command::default(); b.rectangles + b.glyphs + b.rotated];
    let mut stats = [0; 6];
    let mut snapshot = Snapshot::default();
    let mut trace = vec![Trace::default(); 64 * (state.frames.len() + 1)];
    let mut error = [0; 1024];
    let count = unsafe {
        epok_dialogue_preview(
            &request,
            commands.as_mut_ptr(),
            commands.len() as u32,
            stats.as_mut_ptr(),
            &mut snapshot,
            trace.as_mut_ptr(),
            trace.len() as u32,
            error.as_mut_ptr(),
            error.len() as u32,
        )
    };
    if error[0] != 0 {
        return Err(unsafe { std::ffi::CStr::from_ptr(error.as_ptr()) }
            .to_string_lossy()
            .into_owned());
    }
    commands.truncate(count as usize);
    trace.truncate(snapshot.trace_count as usize);
    state.snapshot = Some(snapshot);
    state.trace = trace;
    let pixels = hud_native::render_with_textures(scene, &commands, &texture_ids);
    Ok(Rendered {
        pixels,
        #[cfg(test)]
        commands,
        stats,
        texture_count: texture_ids.len(),
        font_count: font_ids.len(),
    })
}
