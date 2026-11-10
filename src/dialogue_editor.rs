//! Conversation authoring over the existing versioned asset and compiler.
use crate::{
    assets::{self, Kind},
    dialogue_asset as da,
    editor::Editor,
};
use imgui::Ui;
use uuid::Uuid;

pub struct Window {
    pub asset: Uuid,
    record: assets::Record,
    document: da::DialogueAsset,
    saved: da::DialogueAsset,
    undo: Vec<da::DialogueAsset>,
    redo: Vec<da::DialogueAsset>,
    selected: Uuid,
    error: Option<String>,
    graph: crate::dialogue_graph::State,
}
impl Window {
    fn load(record: &assets::Record) -> Result<Self, String> {
        let before = std::fs::read(&record.path).map_err(|e| e.to_string())?;
        let package = assets::Package::load(&record.path)?;
        let after = std::fs::read(&record.path).map_err(|e| e.to_string())?;
        if before != after {
            return Err("Dialogue changed while opening; try again.".into());
        }
        if package.meta.id != record.meta.id || package.meta.kind != Kind::Dialogue {
            return Err(
                "Dialogue asset was replaced; select the replacement before opening.".into(),
            );
        }
        let document = da::DialogueAsset::parse(&package.source)?;
        Ok(Self {
            asset: record.meta.id,
            record: assets::Record {
                path: record.path.clone(),
                meta: package.meta,
                revision: assets::hash(&after),
            },
            selected: document.entry,
            saved: document.clone(),
            document,
            undo: vec![],
            redo: vec![],
            error: None,
            graph: Default::default(),
        })
    }
    pub fn dirty(&self) -> bool {
        self.document != self.saved
    }
    fn checkpoint(&mut self, before: da::DialogueAsset) {
        if before == self.document {
            return;
        }
        self.undo.push(before);
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.error = None;
        self.reconcile_selection();
    }
    fn reconcile_selection(&mut self) {
        if !self.document.nodes.iter().any(|n| n.id == self.selected) {
            self.selected = self.document.entry;
        }
    }
    fn undo(&mut self) {
        if let Some(document) = self.undo.pop() {
            self.redo
                .push(std::mem::replace(&mut self.document, document));
            self.reconcile_selection();
            self.error = None;
        }
    }
    fn redo(&mut self) {
        if let Some(document) = self.redo.pop() {
            self.undo
                .push(std::mem::replace(&mut self.document, document));
            self.reconcile_selection();
            self.error = None;
        }
    }
    fn save(
        &mut self,
        index: &assets::Index,
        registry: &crate::blueprint::Registry,
    ) -> Result<(), String> {
        crate::dialogue_compile::compile(self.asset, &self.document, registry, index)?;
        self.record.revision = da::save(&self.record, &self.document, index)?;
        self.saved = self.document.clone();
        self.error = None;
        Ok(())
    }
    fn remove_selected(&mut self) -> Result<(), String> {
        if self.selected == self.document.entry {
            return Err("Choose another entry node before removing this one.".into());
        }
        if self
            .document
            .nodes
            .iter()
            .any(|n| n.id != self.selected && n.kind.destinations().contains(&self.selected))
        {
            return Err("Reconnect incoming links before removing this node.".into());
        }
        self.document.nodes.retain(|n| n.id != self.selected);
        self.reconcile_selection();
        Ok(())
    }
    fn duplicate_selected(&mut self) -> Result<(), String> {
        if self.document.nodes.len() >= da::MAX_NODES {
            return Err("Conversation has reached its node capacity.".into());
        }
        let mut node = self
            .document
            .nodes
            .iter()
            .find(|n| n.id == self.selected)
            .cloned()
            .ok_or("Select a node to duplicate.")?;
        let original = node.id;
        node.id = Uuid::new_v4();
        let remap = |next: &mut Uuid| {
            if *next == original {
                *next = node.id;
            }
        };
        match &mut node.kind {
            da::NodeKind::Line { next, .. } | da::NodeKind::Event { next, .. } => remap(next),
            da::NodeKind::Branch { yes, no, .. } => {
                remap(yes);
                remap(no);
            }
            da::NodeKind::Choice { options, .. } => {
                for option in options {
                    option.id = Uuid::new_v4();
                    remap(&mut option.next);
                }
            }
            da::NodeKind::End => {}
        }
        self.selected = node.id;
        self.graph.focus(node.id);
        self.document.nodes.push(node);
        Ok(())
    }
    fn remove_participant(&mut self, id: Uuid) -> Result<(), String> {
        let speaker=self.document.nodes.iter().any(|n|matches!(&n.kind,da::NodeKind::Line{speaker:Some(role),..}|da::NodeKind::Choice{speaker:Some(role),..} if *role==id));
        if speaker || self.document.bindings.iter().any(|b| b.participant == id) {
            return Err(
                "Replace this participant's speaker and callback references before removing it."
                    .into(),
            );
        }
        self.document.participants.retain(|p| p.id != id);
        Ok(())
    }
    fn remove_binding(&mut self, id: Uuid) -> Result<(), String> {
        if self.document.nodes.iter().any(|n| match &n.kind {
            da::NodeKind::Branch { condition, .. } => *condition == id,
            da::NodeKind::Event { action, .. } => *action == id,
            da::NodeKind::Choice { options, .. } => options.iter().any(|o| o.condition == Some(id)),
            _ => false,
        }) {
            return Err("Replace this callback's node references before removing it.".into());
        }
        self.document.bindings.retain(|b| b.id != id);
        Ok(())
    }
}
pub fn open(e: &mut Editor, id: Uuid) {
    if let Some(window) = &mut e.dialogue {
        if window.asset == id {
            return;
        }
        if window.dirty() {
            window.error =
                Some("Save or Revert this conversation before opening another one.".into());
            return;
        }
    }
    match e.assets.index.resolve(id).and_then(Window::load) {
        Ok(window) => e.dialogue = Some(window),
        Err(error) => e.last_error = Some(error),
    }
}
pub fn create(e: &mut Editor) -> Result<Uuid, String> {
    if e.dialogue.as_ref().is_some_and(Window::dirty) {
        return Err("Save or Revert the open conversation first.".into());
    }
    let mut serial = 1;
    let path = loop {
        let suffix = if serial == 1 {
            String::new()
        } else {
            serial.to_string()
        };
        let path = format!("assets/Dialogue/Conversation{suffix}.epokasset");
        if !e.root.join(&path).exists() {
            break path;
        }
        serial += 1;
    };
    let id = da::create(&e.root, &path, &da::DialogueAsset::default())?;
    e.assets.index = assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    open(e, id);
    Ok(id)
}
pub fn save_all(e: &mut Editor) -> Result<(), String> {
    let Some(mut window) = e.dialogue.take() else {
        return Ok(());
    };
    let result = if window.dirty() {
        let index = assets::scan(&e.root, &mut Default::default());
        window.save(&index, &e.class_registry)
    } else {
        Ok(())
    };
    if let Err(error) = &result {
        window.error = Some(error.clone());
    } else {
        e.assets.index = assets::scan(&e.root, &mut Default::default());
        e.assets.refresh();
    }
    e.dialogue = Some(window);
    result
}
pub fn window(ui: &Ui, e: &mut Editor) {
    let Some(mut window) = e.dialogue.take() else {
        return;
    };
    let mut opened = true;
    let name = window
        .record
        .path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    ui.window(format!(
        "Dialogue: {name}{}###dialogue-editor",
        if window.dirty() { " *" } else { "" }
    ))
    .opened(&mut opened)
    .size([980., 640.], imgui::Condition::FirstUseEver)
    .build(|| body(ui, e, &mut window));
    if !opened && window.dirty() {
        window.error = Some("Save or Revert changes before closing this conversation.".into());
        opened = true;
    }
    if opened {
        e.dialogue = Some(window);
    }
}
fn body(ui: &Ui, e: &mut Editor, w: &mut Window) {
    if ui.button("Save") {
        let index = assets::scan(&e.root, &mut Default::default());
        match w.save(&index, &e.class_registry) {
            Ok(()) => {
                e.assets.index = assets::scan(&e.root, &mut Default::default());
                e.assets.refresh();
            }
            Err(error) => w.error = Some(error),
        }
    }
    ui.same_line();
    if ui.button("Revert") {
        match Window::load(&w.record) {
            Ok(window) => *w = window,
            Err(error) => w.error = Some(error),
        }
        return;
    }
    ui.same_line();
    if ui.button("Undo") {
        w.undo();
        return;
    }
    ui.same_line();
    if ui.button("Redo") {
        w.redo();
        return;
    }
    ui.same_line();
    if ui.button("Validate") {
        match crate::dialogue_compile::compile(
            w.asset,
            &w.document,
            &e.class_registry,
            &e.assets.index,
        ) {
            Ok(_) => {
                w.error = Some("Conversation references and callback signatures are valid.".into())
            }
            Err(error) => {
                if let Some(node) = w
                    .document
                    .nodes
                    .iter()
                    .find(|n| error.contains(&n.id.to_string()))
                {
                    w.selected = node.id;
                    w.graph.focus(node.id);
                }
                w.error = Some(error);
            }
        }
    }
    ui.separator();
    let before = w.document.clone();
    let labels = w
        .document
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            (
                n.id,
                format!(
                    "{} {}{}",
                    i + 1,
                    kind_name(&n.kind),
                    if n.id == w.document.entry {
                        " (entry)"
                    } else {
                        ""
                    }
                ),
            )
        })
        .collect::<Vec<_>>();
    let graph_width = (ui.content_region_avail()[0] * 0.56).max(320.);
    ui.child_window("conversation-nodes")
        .size([graph_width, 0.])
        .build(|| {
            id_picker(ui, "Find node", &mut w.selected, &labels);
            if ui.button("Frame selected") {
                w.graph.focus(w.selected);
            }
            ui.same_line();
            if ui.button("Frame all") {
                w.graph.frame_all();
            }
            ui.same_line();
            if ui.button("Arrange") {
                w.graph.arrange(&w.document);
                w.graph.focus(w.document.entry);
            }
            ui.text_disabled("Drag an output to link. Middle mouse pans; wheel zooms.");
            for name in ["Line", "Choice", "Branch", "Event", "End"] {
                if ui.small_button(name) && w.document.nodes.len() < da::MAX_NODES {
                    let id = Uuid::new_v4();
                    let next = w
                        .document
                        .nodes
                        .iter()
                        .find(|n| matches!(n.kind, da::NodeKind::End))
                        .map(|n| n.id)
                        .unwrap_or(w.document.entry);
                    let text = default_text(&e.assets.index);
                    let binding = |kind| {
                        w.document
                            .bindings
                            .iter()
                            .find(|b| b.kind == kind)
                            .map(|b| b.id)
                            .unwrap_or(Uuid::nil())
                    };
                    let kind = match name {
                        "Line" => da::NodeKind::Line {
                            speaker: None,
                            text,
                            next,
                            policy: None,
                        },
                        "Choice" => da::NodeKind::Choice {
                            speaker: None,
                            prompt: None,
                            options: vec![da::ChoiceOption {
                                id: Uuid::new_v4(),
                                text,
                                next,
                                condition: None,
                            }],
                        },
                        "Branch" => da::NodeKind::Branch {
                            condition: binding(da::BindingKind::Condition),
                            yes: next,
                            no: next,
                        },
                        "Event" => da::NodeKind::Event {
                            action: binding(da::BindingKind::Action),
                            next,
                        },
                        _ => da::NodeKind::End,
                    };
                    w.document.nodes.push(da::Node { id, kind });
                    w.selected = id;
                    w.graph.focus(id);
                }
                if name != "End" {
                    ui.same_line();
                }
            }
            ui.separator();
            if let Some(link) = w.graph.draw(ui, &w.document, &mut w.selected)
                && let Err(error) = crate::dialogue_graph::connect(&mut w.document, link)
            {
                w.error = Some(error);
            }
        });
    ui.same_line();
    ui.child_window("conversation-details")
        .size([0., 0.])
        .build(|| {
            if ui.collapsing_header("Selected node", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                if ui.button("Set as entry") {
                    w.document.entry = w.selected;
                }
                ui.same_line();
                if ui.button("Remove node")
                    && let Err(error) = w.remove_selected()
                {
                    w.error = Some(error);
                }
                ui.same_line();
                if ui.button("Duplicate node")
                    && let Err(error) = w.duplicate_selected()
                {
                    w.error = Some(error);
                }
                let participants = w
                    .document
                    .participants
                    .iter()
                    .map(|p| (p.id, p.label.clone()))
                    .collect::<Vec<_>>();
                let conditions = w
                    .document
                    .bindings
                    .iter()
                    .filter(|b| b.kind == da::BindingKind::Condition)
                    .map(|b| (b.id, b.label.clone()))
                    .collect::<Vec<_>>();
                let actions = w
                    .document
                    .bindings
                    .iter()
                    .filter(|b| b.kind == da::BindingKind::Action)
                    .map(|b| (b.id, b.label.clone()))
                    .collect::<Vec<_>>();
                if let Some(node) = w.document.nodes.iter_mut().find(|n| n.id == w.selected) {
                    match &mut node.kind {
                        da::NodeKind::Line {
                            speaker,
                            text,
                            next,
                            policy,
                        } => {
                            optional_id(ui, "Speaker", speaker, &participants);
                            text_key(ui, "Text", text, &e.assets.index);
                            id_picker(ui, "Next", next, &labels);
                            let mut custom = policy.is_some();
                            if ui.checkbox("Custom timing", &mut custom) {
                                *policy = custom.then(da::LinePolicy::default);
                            }
                            if let Some(policy) = policy {
                                line_policy(ui, policy);
                            }
                        }
                        da::NodeKind::Choice {
                            speaker,
                            prompt,
                            options,
                        } => {
                            optional_id(ui, "Speaker", speaker, &participants);
                            let mut present = prompt.is_some();
                            if ui.checkbox("Prompt", &mut present) {
                                *prompt = present.then(|| default_text(&e.assets.index));
                            }
                            if let Some(prompt) = prompt {
                                text_key(ui, "Prompt text", prompt, &e.assets.index);
                            }
                            let mut remove = None;
                            for (i, option) in options.iter_mut().enumerate() {
                                let _scope = ui.push_id(option.id.to_string());
                                ui.separator();
                                ui.text(format!("Choice {}", i + 1));
                                text_key(ui, "Label", &mut option.text, &e.assets.index);
                                id_picker(ui, "Next", &mut option.next, &labels);
                                optional_id(ui, "Condition", &mut option.condition, &conditions);
                                if ui.small_button("Remove choice") {
                                    remove = Some(i);
                                }
                            }
                            if let Some(i) = remove {
                                options.remove(i);
                            }
                            if options.len() < da::MAX_OPTIONS && ui.button("Add choice") {
                                options.push(da::ChoiceOption {
                                    id: Uuid::new_v4(),
                                    text: default_text(&e.assets.index),
                                    next: w.document.entry,
                                    condition: None,
                                });
                            }
                        }
                        da::NodeKind::Branch { condition, yes, no } => {
                            id_picker(ui, "Condition", condition, &conditions);
                            id_picker(ui, "True", yes, &labels);
                            id_picker(ui, "False", no, &labels);
                        }
                        da::NodeKind::Event { action, next } => {
                            id_picker(ui, "Action", action, &actions);
                            id_picker(ui, "Next", next, &labels);
                        }
                        da::NodeKind::End => {
                            ui.text("Completes the conversation and releases controls.")
                        }
                    }
                }
            }
            if ui.collapsing_header("Participants", imgui::TreeNodeFlags::empty()) {
                let mut remove = None;
                for participant in &mut w.document.participants {
                    let _scope = ui.push_id(participant.id.to_string());
                    ui.input_text("Role", &mut participant.label).build();
                    let mut named = participant.display_name.is_some();
                    if ui.checkbox("Display name", &mut named) {
                        participant.display_name = named.then(|| default_text(&e.assets.index));
                    }
                    if let Some(name) = &mut participant.display_name {
                        text_key(ui, "Name", name, &e.assets.index);
                    }
                    asset_picker(
                        ui,
                        "Portrait",
                        &mut participant.portrait,
                        &e.assets.index,
                        Kind::Texture,
                    );
                    ui.text_disabled(format!(
                        "Role ID: {}",
                        crate::dialogue_compile::compact(participant.id)
                    ));
                    if ui.small_button("Remove participant") {
                        remove = Some(participant.id);
                    }
                }
                if let Some(id) = remove
                    && let Err(error) = w.remove_participant(id)
                {
                    w.error = Some(error);
                }
                if w.document.participants.len() < da::MAX_PARTICIPANTS
                    && ui.button("Add participant")
                {
                    w.document.participants.push(da::Participant {
                        id: Uuid::new_v4(),
                        label: format!("Speaker {}", w.document.participants.len() + 1),
                        display_name: None,
                        portrait: None,
                    });
                }
            }
            if ui.collapsing_header("Conditions and actions", imgui::TreeNodeFlags::empty()) {
                let participants = w
                    .document
                    .participants
                    .iter()
                    .map(|p| (p.id, p.label.clone()))
                    .collect::<Vec<_>>();
                let mut remove = None;
                for binding in &mut w.document.bindings {
                    let _scope = ui.push_id(binding.id.to_string());
                    ui.separator();
                    ui.input_text("Label", &mut binding.label).build();
                    id_picker(ui, "Participant", &mut binding.participant, &participants);
                    ui.text(format!("{:?}", binding.kind));
                    callback_picker(ui, binding, &e.class_registry);
                    if ui.small_button("Remove callback") {
                        remove = Some(binding.id);
                    }
                }
                if let Some(id) = remove
                    && let Err(error) = w.remove_binding(id)
                {
                    w.error = Some(error);
                }
                for kind in [da::BindingKind::Condition, da::BindingKind::Action] {
                    if w.document.bindings.len() < da::MAX_BINDINGS
                        && ui.button(format!("Add {kind:?}"))
                    {
                        w.document.bindings.push(da::Binding {
                            id: Uuid::new_v4(),
                            label: format!("{kind:?}"),
                            kind,
                            participant: w
                                .document
                                .participants
                                .first()
                                .map(|p| p.id)
                                .unwrap_or(Uuid::nil()),
                            class: String::new(),
                            member: String::new(),
                        });
                    }
                    ui.same_line();
                }
                ui.new_line();
            }
            if let Some(error) = &w.error {
                ui.separator();
                ui.text_wrapped(error);
            } else if let Err(error) = w.document.validate() {
                ui.separator();
                ui.text_wrapped(error);
            }
        });
    w.checkpoint(before);
}
fn kind_name(kind: &da::NodeKind) -> &'static str {
    match kind {
        da::NodeKind::Line { .. } => "Line",
        da::NodeKind::Choice { .. } => "Choice",
        da::NodeKind::Branch { .. } => "Branch",
        da::NodeKind::Event { .. } => "Event",
        da::NodeKind::End => "End",
    }
}
fn id_picker(ui: &Ui, label: &str, value: &mut Uuid, choices: &[(Uuid, String)]) {
    let selected = choices
        .iter()
        .find(|(id, _)| id == value)
        .map(|(_, label)| label.as_str())
        .unwrap_or("Select...");
    if let Some(_combo) = ui.begin_combo(label, selected) {
        for (id, name) in choices {
            if ui
                .selectable_config(format!("{name}##{id}"))
                .selected(id == value)
                .build()
            {
                *value = *id;
            }
        }
    }
}
fn optional_id(ui: &Ui, label: &str, value: &mut Option<Uuid>, choices: &[(Uuid, String)]) {
    let mut all = vec![(Uuid::nil(), "None".into())];
    all.extend_from_slice(choices);
    let mut selected = value.unwrap_or(Uuid::nil());
    id_picker(ui, label, &mut selected, &all);
    *value = (!selected.is_nil()).then_some(selected);
}
pub(crate) fn asset_picker(
    ui: &Ui,
    label: &str,
    value: &mut Option<Uuid>,
    index: &assets::Index,
    kind: Kind,
) {
    let choices = index
        .usable()
        .filter(|r| r.meta.kind == kind)
        .map(|r| {
            (
                r.meta.id,
                r.path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            )
        })
        .collect::<Vec<_>>();
    optional_id(ui, label, value, &choices);
}
fn default_text(index: &assets::Index) -> da::TextKey {
    index
        .usable()
        .filter(|r| r.meta.kind == Kind::StringTable)
        .find_map(|r| {
            crate::localization::document(r).ok().and_then(|t| {
                t.entries.first().map(|entry| da::TextKey {
                    table: r.meta.id,
                    key: entry.key.clone(),
                })
            })
        })
        .unwrap_or(da::TextKey {
            table: Uuid::nil(),
            key: String::new(),
        })
}
fn text_key(ui: &Ui, label: &str, text: &mut da::TextKey, index: &assets::Index) {
    let _scope = ui.push_id(label);
    let choices = index
        .usable()
        .filter(|r| r.meta.kind == Kind::StringTable)
        .map(|r| {
            (
                r.meta.id,
                r.path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            )
        })
        .collect::<Vec<_>>();
    let previous = text.table;
    id_picker(ui, "String Table", &mut text.table, &choices);
    if let Ok(record) = index.resolve(text.table)
        && let Ok(table) = crate::localization::document(record)
    {
        if previous != text.table {
            text.key = table
                .entries
                .first()
                .map(|e| e.key.clone())
                .unwrap_or_default();
        }
        if let Some(_combo) = ui.begin_combo(
            label,
            if text.key.is_empty() {
                "Select key..."
            } else {
                &text.key
            },
        ) {
            for entry in &table.entries {
                if ui
                    .selectable_config(&entry.key)
                    .selected(text.key == entry.key)
                    .build()
                {
                    text.key = entry.key.clone();
                }
            }
        }
        if let Some(entry) = table.entries.iter().find(|entry| entry.key == text.key) {
            ui.text_wrapped(entry.values.first().map(|s| s.as_str()).unwrap_or(""));
        }
    } else {
        ui.text_disabled("Create a String Table, then select a text key.");
    }
}
pub(crate) fn line_policy(ui: &Ui, policy: &mut da::LinePolicy) {
    let mut reveal = i32::from(policy.milliseconds_per_glyph);
    if ui.input_int("Milliseconds per glyph", &mut reveal).build() {
        policy.milliseconds_per_glyph = reveal.clamp(0, 1000) as u16;
    }
    crate::blueprint_editor::record_control(ui, "line-policy:reveal");
    let mut automatic = policy.auto_advance_ms.is_some();
    if ui.checkbox("Automatic advance", &mut automatic) {
        policy.auto_advance_ms = automatic.then_some(1000);
    }
    crate::blueprint_editor::record_control(ui, "line-policy:automatic");
    if let Some(delay) = &mut policy.auto_advance_ms {
        let mut value = *delay as i32;
        if ui
            .input_int("Advance after milliseconds", &mut value)
            .build()
        {
            *delay = value.clamp(1, 3_600_000) as u32;
        }
        crate::blueprint_editor::record_control(ui, "line-policy:advance");
    }
}
fn callback_picker(ui: &Ui, binding: &mut da::Binding, registry: &crate::blueprint::Registry) {
    let selected = registry
        .classes
        .get(&binding.class)
        .map(|class| {
            format!(
                "{} / {}",
                class.cpp_name,
                registry
                    .ancestry(&class.cpp_name)
                    .iter()
                    .flat_map(|c| c.functions.iter())
                    .find(|f| f.id == binding.member)
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| "Missing member".into())
            )
        })
        .unwrap_or_else(|| "Select callback...".into());
    if let Some(_combo) = ui.begin_combo("Callback", selected) {
        let model = registry.model().ok();
        for class in registry
            .classes
            .values()
            .filter(|c| c.backend == crate::reflection_schema::native_backend())
        {
            if !model
                .as_ref()
                .and_then(|m| m.class(&class.id))
                .is_some_and(|c| {
                    matches!(
                        c.family,
                        crate::reflection_schema::ClassFamily::Actor
                            | crate::reflection_schema::ClassFamily::Component
                    )
                })
            {
                continue;
            }
            for function in registry
                .ancestry(&class.cpp_name)
                .iter()
                .flat_map(|c| c.functions.iter())
            {
                if crate::dialogue_compile::callback_signature(binding.kind, function)
                    && ui
                        .selectable_config(format!(
                            "{} / {}##{}{}",
                            class.cpp_name, function.name, class.id, function.id
                        ))
                        .selected(binding.class == class.id && binding.member == function.id)
                        .build()
                {
                    binding.class = class.id.clone();
                    binding.member = function.id.clone();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialogue_editor_history_preserves_ids_dirty_state_and_external_changes() {
        let root = crate::workspace::tests::temp("dialogue-editor-history");
        crate::workspace::create(&root, "Dialogue Editor", crate::workspace::Template::Basic)
            .unwrap();
        let id = da::create(
            &root,
            "assets/dialogue.epokasset",
            &da::DialogueAsset::default(),
        )
        .unwrap();
        let mut index = assets::scan(&root, &mut Default::default());
        let record = index.resolve(id).unwrap().clone();
        let mut window = Window::load(&record).unwrap();
        let entry = window.document.entry;
        let extra = Uuid::new_v4();
        let before = window.document.clone();
        window.document.nodes.push(da::Node {
            id: extra,
            kind: da::NodeKind::End,
        });
        window.selected = extra;
        window.checkpoint(before);
        assert!(window.dirty());
        window.undo();
        assert!(!window.dirty() && window.document.entry == entry && window.selected == entry);
        window.redo();
        assert_eq!(window.document.nodes[1].id, extra);
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        window.save(&index, &registry).unwrap();
        assert!(!window.dirty());
        window.undo();
        assert!(window.dirty());
        window.redo();
        assert!(!window.dirty());
        let mut external = Window::load(&record).unwrap();
        external.document.entry = extra;
        external.save(&index, &registry).unwrap();
        let before = window.document.clone();
        window.document.nodes.push(da::Node {
            id: Uuid::new_v4(),
            kind: da::NodeKind::End,
        });
        window.checkpoint(before);
        let draft = window.document.clone();
        assert!(window.save(&index, &registry).is_err());
        assert_eq!(window.document, draft);
        assert!(window.dirty());
        index = assets::scan(&root, &mut Default::default());
        assert_eq!(
            da::document(index.resolve(id).unwrap()).unwrap().entry,
            extra
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_editor_removal_requires_reconnecting_incoming_links() {
        let root = crate::workspace::tests::temp("dialogue-editor-remove");
        crate::workspace::create(&root, "Dialogue Remove", crate::workspace::Template::Basic)
            .unwrap();
        let mut document = da::DialogueAsset::default();
        let end = Uuid::new_v4();
        document.nodes[0].kind = da::NodeKind::Branch {
            condition: Uuid::new_v4(),
            yes: end,
            no: end,
        };
        // The draft may temporarily be invalid, while structural operations
        // still protect entry and incoming links without changing identities.
        let record = assets::Record {
            path: root.join("draft"),
            meta: assets::Metadata {
                version: 1,
                id: Uuid::new_v4(),
                kind: Kind::Dialogue,
                importer_version: 1,
                source: String::new(),
                source_hash: String::new(),
                settings: crate::import_settings::Settings::Authored,
                extra: Default::default(),
            },
            revision: String::new(),
        };
        document.nodes.push(da::Node {
            id: end,
            kind: da::NodeKind::End,
        });
        let mut window = Window {
            asset: record.meta.id,
            selected: document.entry,
            saved: document.clone(),
            document,
            record,
            undo: vec![],
            redo: vec![],
            error: None,
            graph: Default::default(),
        };
        assert!(window.remove_selected().is_err());
        window.selected = end;
        assert!(window.remove_selected().is_err());
        window.document.nodes[0].kind = da::NodeKind::End;
        window.remove_selected().unwrap();
        assert_eq!(window.document.nodes.len(), 1);
        assert_eq!(window.selected, window.document.entry);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_editor_duplicates_local_edges_and_protects_role_callback_references() {
        let root = crate::workspace::tests::temp("dialogue-editor-duplicate");
        crate::workspace::create(&root, "Duplicate", crate::workspace::Template::Basic).unwrap();
        let id = da::create(
            &root,
            "assets/dialogue.epokasset",
            &da::DialogueAsset::default(),
        )
        .unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let mut window = Window::load(index.resolve(id).unwrap()).unwrap();
        let source = window.selected;
        let end = Uuid::new_v4();
        let role = Uuid::new_v4();
        let binding = Uuid::new_v4();
        let option = Uuid::new_v4();
        let table = Uuid::new_v4();
        window.document.participants.push(da::Participant {
            id: role,
            label: "Speaker".into(),
            display_name: None,
            portrait: None,
        });
        window.document.bindings.push(da::Binding {
            id: binding,
            label: "Allowed".into(),
            participant: role,
            kind: da::BindingKind::Condition,
            class: "class".into(),
            member: "member".into(),
        });
        window.document.nodes[0].kind = da::NodeKind::Choice {
            speaker: Some(role),
            prompt: None,
            options: vec![da::ChoiceOption {
                id: option,
                text: da::TextKey {
                    table,
                    key: "choice".into(),
                },
                next: source,
                condition: Some(binding),
            }],
        };
        window.document.nodes.push(da::Node {
            id: end,
            kind: da::NodeKind::End,
        });
        let before = window.document.clone();
        window.duplicate_selected().unwrap();
        let copied = window.selected;
        window.checkpoint(before.clone());
        assert_ne!(copied, source);
        assert_eq!(window.document.entry, source);
        let da::NodeKind::Choice {
            speaker, options, ..
        } = &window.document.nodes[2].kind
        else {
            panic!()
        };
        assert_eq!(*speaker, Some(role));
        assert_ne!(options[0].id, option);
        assert_eq!(options[0].next, copied);
        assert_eq!(options[0].text.table, table);
        assert_eq!(options[0].condition, Some(binding));
        window.undo();
        assert_eq!(window.document, before);
        window.redo();
        assert_eq!(window.document.nodes[2].id, copied);
        assert!(window.remove_participant(role).is_err());
        assert!(window.remove_binding(binding).is_err());
        for node in &mut window.document.nodes {
            if let da::NodeKind::Choice {
                speaker, options, ..
            } = &mut node.kind
            {
                *speaker = None;
                for option in options {
                    option.condition = None;
                }
            }
        }
        assert!(window.remove_participant(role).is_err());
        window.remove_binding(binding).unwrap();
        window.remove_participant(role).unwrap();
        assert!(window.document.participants.is_empty() && window.document.bindings.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}
