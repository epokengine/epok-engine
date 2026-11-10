//! Generic UI class composition over the existing Actor/component/template model.
//! Preview resolves data and native presentation contracts; project behavior
//! stays in the class's own source or graph and is exercised by explicit Play.
use crate::{
    blueprint::Registry,
    blueprint_editor::template_editor::{self, Document},
    blueprint_templates::{self as templates, Template},
    class_composition as composition,
    editor::Editor,
    scene::{ClassDefaults, Scene},
};
use std::path::{Path, PathBuf};
use uuid::Uuid;

enum Storage {
    Blueprint(PathBuf),
    Source {
        path: PathBuf,
        source: Option<Box<composition::Source>>,
    },
}
pub fn source(root: &Path, class: &crate::reflection_schema::Class) -> Option<PathBuf> {
    if class.provider.id == "blueprint" {
        let source = if class.source.file.is_absolute() {
            class.source.file.clone()
        } else {
            root.join(&class.source.file)
        }
        .canonicalize()
        .ok()?;
        (source.starts_with(root.join("assets").canonicalize().ok()?) && source.is_file())
            .then_some(source)
    } else {
        crate::blueprint_workflow::class_source(root, class)
    }
}
pub struct Preview {
    pub scene: Scene,
    pub pixels: Vec<u8>,
    pub revision: String,
    pub stats: [u32; 6],
    pub(crate) usage: Usage,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Usage {
    pub actors: usize,
    pub components: usize,
    pub max_components: usize,
    pub text_controls: usize,
    pub text_record_bytes: usize,
    pub text_buffer_bytes: usize,
    pub textures: usize,
    pub fonts: usize,
    pub builtin_font: bool,
}
impl Usage {
    fn read(scene: &Scene, textures: usize, fonts: usize) -> Self {
        let [record, buffer] = crate::hud_native::text_storage();
        Self {
            actors: scene.actors.len(),
            components: scene.actors.iter().map(|a| a.components.len()).sum(),
            max_components: scene
                .actors
                .iter()
                .map(|a| a.components.len())
                .max()
                .unwrap_or(0),
            text_controls: scene.actors.iter().filter(|a| a.text.is_some()).count(),
            text_record_bytes: record as usize,
            text_buffer_bytes: buffer as usize,
            textures,
            fonts,
            builtin_font: scene
                .actors
                .iter()
                .any(|a| a.text.as_ref().is_some_and(|t| t.font.is_none())),
        }
    }
}
#[derive(Default)]
pub struct Texture {
    uploaded: Option<(imgui::TextureId, [u32; 2], String)>,
}
impl Texture {
    pub fn upload(
        &mut self,
        window: Option<&mut Window>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut imgui_wgpu::Renderer,
    ) {
        let Some(window) = window else {
            if let Some((id, ..)) = self.uploaded.take() {
                renderer.textures.remove(id);
            }
            return;
        };
        let Some(preview) = &window.preview else {
            return;
        };
        let size = preview.scene.display_size.map(u32::from);
        let create = || {
            imgui_wgpu::Texture::new(
                device,
                renderer,
                imgui_wgpu::TextureConfig {
                    size: wgpu::Extent3d {
                        width: size[0],
                        height: size[1],
                        depth_or_array_layers: 1,
                    },
                    format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                    sampler_desc: wgpu::SamplerDescriptor {
                        mag_filter: wgpu::FilterMode::Nearest,
                        min_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
        };
        match &self.uploaded {
            None => {
                let texture = create();
                let id = renderer.textures.insert(texture);
                self.uploaded = Some((id, size, String::new()));
            }
            Some((id, previous, ..)) if *previous != size => {
                let texture = create();
                renderer.textures.replace(*id, texture);
                self.uploaded = Some((*id, size, String::new()));
            }
            _ => {}
        }
        let (id, _, revision) = self.uploaded.as_mut().unwrap();
        if *revision != preview.revision {
            renderer
                .textures
                .get(*id)
                .unwrap()
                .write(queue, &preview.pixels, size[0], size[1]);
            *revision = preview.revision.clone();
        }
        window.texture = Some(*id);
    }
}
pub struct Window {
    pub class: String,
    name: String,
    template: Template,
    saved: Template,
    storage: Storage,
    undo: Vec<Template>,
    redo: Vec<Template>,
    controls: template_editor::State,
    pub preview: Option<Preview>,
    pub texture: Option<imgui::TextureId>,
    dialogue: bool,
    pub sample: usize,
    pub conversation: crate::dialogue_preview::State,
    pub preview_size: Option<[u16; 2]>,
    pub preview_error: Option<String>,
    drag: Option<Uuid>,
    drag_before: Option<Template>,
    pub error: Option<String>,
}
impl Window {
    pub fn load(root: &Path, registry: &Registry, id: &str) -> Result<Self, String> {
        let class = registry.classes.get(id).ok_or("Class is missing.")?;
        let model = registry.model().map_err(|errors| {
            errors
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        if model.class(id).is_none_or(|c| {
            c.domain != crate::reflection_schema::Domain::UI
                || c.family != crate::reflection_schema::ClassFamily::Actor
        }) {
            return Err("The UI class designer requires a UIActor class.".into());
        }
        if source(root, class).is_none() {
            return Err("Derive a project widget class before editing an SDK class.".into());
        }
        let index = composition::load(root)?;
        index.validate(root, registry)?;
        let (mut template, storage) = if class.provider.id == "blueprint" {
            let path = class.source.file.clone();
            (
                crate::blueprint_asset::load(&path)?.template,
                Storage::Blueprint(path),
            )
        } else if let Some(source) = index.sources.get(id) {
            (
                source.document.template.clone(),
                Storage::Source {
                    path: source.path.clone(),
                    source: Some(Box::new(source.clone())),
                },
            )
        } else {
            (
                Template::default(),
                Storage::Source {
                    path: root.join(format!("assets/Classes/{id}.epokclass")),
                    source: None,
                },
            )
        };
        let saved = template.clone();
        let inherited = parent_layers(root, registry, id)?;
        if template.actors.is_empty() && inherited.iter().all(|t| t.actors.is_empty()) {
            let actor = composition::actor(&model, id, &class.cpp_name)?;
            template.actors.push(templates::TemplateEntity {
                entity: actor,
                parent: None,
            });
        }
        Ok(Self {
            class: id.into(),
            name: class.cpp_name.clone(),
            saved,
            template,
            storage,
            undo: vec![],
            redo: vec![],
            controls: Default::default(),
            preview: None,
            texture: None,
            dialogue: model.is_a(id, crate::dialogue_layout::PRESENTER),
            sample: 0,
            conversation: Default::default(),
            preview_size: None,
            preview_error: None,
            drag: None,
            drag_before: None,
            error: None,
        })
    }
    pub fn dirty(&self) -> bool {
        self.template != self.saved || matches!(&self.storage, Storage::Source { source: None, .. })
    }
    pub fn focus_component(
        &mut self,
        root: &Path,
        registry: &Registry,
        selector: &str,
    ) -> Result<(), String> {
        let scene = self.resolve(root, registry)?;
        let matches = scene
            .actors
            .iter()
            .flat_map(|a| {
                a.components
                    .iter()
                    .filter(move |c| c.id.to_string() == selector || c.name == selector)
                    .map(move |c| (a.id, c.id))
            })
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [(actor, component)] => {
                self.controls.select_component(*actor, *component);
                Ok(())
            }
            [] => Err(format!("No widget component matches {selector}.")),
            _ => Err(format!(
                "Widget component name {selector} is ambiguous; select its UUID."
            )),
        }
    }
    fn checkpoint(&mut self, before: Template) {
        if before != self.template {
            self.undo.push(before);
            if self.undo.len() > 64 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
    }
    fn undo(&mut self) {
        if let Some(previous) = self.undo.pop() {
            self.redo
                .push(std::mem::replace(&mut self.template, previous));
        }
    }
    fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.template, next));
        }
    }
    fn finish_edit(&mut self, before: Template, mouse_down: bool) {
        if self.drag_before.is_none() {
            self.checkpoint(before);
        }
        if !mouse_down && let Some(before) = self.drag_before.take() {
            self.checkpoint(before);
            self.drag = None;
        }
    }
    fn edit<T>(
        &mut self,
        root: &Path,
        registry: &Registry,
        action: impl FnOnce(&mut Template, &Scene) -> Result<T, String>,
    ) -> Result<T, String> {
        let scene = self.resolve(root, registry)?;
        let before = self.template.clone();
        let result = action(&mut self.template, &scene).and_then(|value| {
            self.template
                .references
                .sort_by(|a, b| (a.owner, &a.member).cmp(&(b.owner, &b.member)));
            self.template
                .overrides
                .retain(|_, e| !e.members.is_empty() || e.parent.is_some());
            let resolved = self.resolve(root, registry)?;
            let model = registry.model().map_err(|errors| {
                errors
                    .into_iter()
                    .map(|e| e.message)
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
            for actor in &resolved.actors {
                crate::actor_components::validate_with_model(&model, actor)?;
            }
            Ok(value)
        });
        if result.is_err() {
            self.template = before;
        } else {
            self.error = None;
        }
        result
    }
    fn add_component(
        &mut self,
        root: &Path,
        registry: &Registry,
        actor: Uuid,
        class: &str,
    ) -> Result<Uuid, String> {
        self.edit(root, registry, |template, scene| {
            let original = scene
                .actors
                .iter()
                .find(|a| a.id == actor)
                .ok_or("Actor is missing.")?;
            let model = registry.model().map_err(|errors| {
                errors
                    .into_iter()
                    .map(|e| e.message)
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
            let mut edited = original.clone();
            let id = crate::actor_components::add(&mut edited, &model, class)?;
            template_editor::apply_entity_changes(template, original, &edited)?;
            Ok(id)
        })
    }
    fn remove_component(
        &mut self,
        root: &Path,
        registry: &Registry,
        actor: Uuid,
        component: Uuid,
    ) -> Result<(), String> {
        self.edit(root, registry, |template, scene| {
            let original = scene.actors.iter().find(|a| a.id == actor).ok_or("Actor is missing.")?;
            let existing = original.components.iter().find(|c| c.id == component).ok_or("Component is missing.")?;
            if let Some(reason) = crate::actor_components::component_removal_refusal(existing) { return Err(reason.into()); }
            if !local_component(template, original, component) {
                return Err("Inherited controls and class instances keep their class-defined components; disable or override them instead.".into());
            }
            let mut edited = original.clone();
            edited.components.retain(|c| c.id != component);
            edited.refresh_components();
            if owns_actor(template, actor) {
                template_editor::apply_entity_changes(template, original, &edited)?;
            } else if let Some(edits) = template.overrides.get_mut(&actor) {
                let prefix = format!("/components/{component}");
                edits.members.retain(|path, _| path != &prefix && !path.starts_with(&format!("{prefix}/")));
            }
            template.references.retain(|r| r.owner != component);
            Ok(())
        })
    }
    fn property(
        &mut self,
        root: &Path,
        registry: &Registry,
        actor: Uuid,
        owner: Uuid,
        action: crate::gui::PropertyAction,
    ) -> Result<(), String> {
        self.edit(root, registry, |template, scene| {
            let original = scene
                .actors
                .iter()
                .find(|a| a.id == actor)
                .ok_or("Actor is missing.")?;
            let (class, _, _) =
                property_owner(original, owner).ok_or("Property owner is missing.")?;
            let key = match &action {
                crate::gui::PropertyAction::Set(key, _)
                | crate::gui::PropertyAction::Reset(key)
                | crate::gui::PropertyAction::DiscardOrphan(key) => key,
            };
            let property = registry
                .properties(&class.name)
                .into_iter()
                .find(|p| p.name == *key);
            if let crate::gui::PropertyAction::Set(_, value) = &action {
                let property = property.ok_or("Property is no longer declared.")?;
                if !property.editable {
                    return Err("Property is read-only.".into());
                }
                let reference_target = if is_reference(&property.value_type) {
                    Some(if value.is_null() {
                        Uuid::nil()
                    } else {
                        value
                            .as_str()
                            .and_then(|s| Uuid::parse_str(s).ok())
                            .ok_or("Reference must be a UUID or None.")?
                    })
                } else {
                    None
                };
                if owns_actor(template, actor) {
                    let mut edited = original.clone();
                    if owner == actor {
                        edited.properties.insert(key.clone(), value.clone());
                        edited.overrides.insert(key.clone());
                    } else {
                        let c = edited
                            .components
                            .iter_mut()
                            .find(|c| c.id == owner)
                            .unwrap();
                        c.properties.insert(key.clone(), value.clone());
                        c.overrides.insert(key.clone());
                    }
                    crate::actor_components::sync(&mut edited);
                    template_editor::apply_entity_changes(template, original, &edited)?;
                } else {
                    let path = property_path(actor, owner, key);
                    let members = &mut template.overrides.entry(actor).or_default().members;
                    members.retain(|k, _| !k.starts_with(&format!("{path}/")));
                    members.insert(
                        path,
                        if value.is_null() {
                            templates::literal_null()
                        } else {
                            value.clone()
                        },
                    );
                }
                if let Some(target) = reference_target {
                    template
                        .references
                        .retain(|r| r.owner != owner || r.member != property.id);
                    template.references.push(templates::ObjectReference {
                        owner,
                        member: property.id.clone(),
                        target,
                    });
                }
            } else {
                reset_property(template, original, owner, key);
                if let Some(property) = property {
                    template
                        .references
                        .retain(|r| r.owner != owner || r.member != property.id);
                }
            }
            Ok(())
        })
    }
    fn resolve(&self, root: &Path, registry: &Registry) -> Result<Scene, String> {
        let mut layers = parent_layers(root, registry, &self.class)?;
        layers.push(self.template.clone());
        let sources = composition::sources(root, registry)?;
        let resolved =
            sources.resolve_layers(registry, &self.class, &layers.iter().collect::<Vec<_>>())?;
        let class = &registry.classes[&self.class];
        resolved.scene(
            ClassDefaults {
                name: class.cpp_name.clone(),
                class_id: Some(class.id.clone()),
                provider: class.provider.clone(),
                backend: class.backend.clone(),
                ..Default::default()
            },
            registry,
        )
    }
    pub fn save(&mut self, root: &Path, registry: &Registry) -> Result<(), String> {
        self.resolve(root, registry)?;
        match &mut self.storage {
            Storage::Source { path, source } => {
                let document =
                    composition::Document::new(self.class.clone(), self.template.clone());
                if let Some(previous) = source {
                    previous.revision = composition::save(root, previous, &document, registry)?;
                    previous.document = document;
                } else {
                    composition::create(
                        root,
                        &crate::assets::path_string(root, path),
                        &document,
                        registry,
                    )?;
                    *source = Some(Box::new(composition::Source {
                        path: path.clone(),
                        revision: crate::assets::hash(&crate::assets::read_bounded(path)?),
                        document,
                    }));
                }
            }
            Storage::Blueprint(path) => {
                let bytes = crate::assets::read_bounded(path)?;
                let mut current = crate::blueprint_asset::load(path)?;
                if current.id != self.class || current.template != self.saved {
                    return Err(
                        "Widget composition changed on disk. Revert to reload before saving."
                            .into(),
                    );
                }
                // Preserve behavior edits made by the graph editor while the UI
                // was open; only the captured composition may be replaced.
                current.template = self.template.clone();
                crate::assets::atomic_write(
                    path,
                    &crate::document::to_vec(&current)?,
                    Some(&crate::assets::hash(&bytes)),
                )?;
            }
        }
        composition::observe_current(root)?;
        self.saved = self.template.clone();
        self.error = None;
        Ok(())
    }
    fn update_preview(&mut self, e: &Editor) -> Result<(), String> {
        let mut scene = self.resolve(&e.root, &e.class_registry)?;
        scene.display_size = self.preview_size.unwrap_or(e.scene.display_size);
        scene.preview_language = e.scene.preview_language;
        let signature = self.preview_key(e, &scene);
        if self
            .preview
            .as_ref()
            .is_some_and(|p| p.revision == signature)
        {
            return Ok(());
        }
        crate::texture::resolve(&mut scene, &e.assets.index)?;
        crate::hud::resolve_fonts(&mut scene, &e.assets.index)?;
        crate::localization::resolve(&mut scene, &e.assets.index);
        crate::localization::resolve_runtime(&mut scene, &e.assets.index, &e.class_registry)?;
        // Preview state is a render-only projection. Canvas dragging and the
        // inspectors continue editing authored controls, never sample content.
        let mut rendered = scene.clone();
        let (pixels, stats, textures, fonts) = if self.dialogue && self.sample == 5 {
            let output = crate::dialogue_preview::render(
                &mut rendered,
                &e.class_registry,
                &e.assets.index,
                &self.class,
                &mut self.conversation,
            )?;
            (
                output.pixels,
                output.stats,
                output.texture_count,
                output.font_count,
            )
        } else {
            if self.dialogue && self.sample != 0 {
                crate::dialogue_presenter::sample(&mut rendered, &e.class_registry, self.sample);
            }
            let (commands, stats) = crate::hud_native::compile(&rendered);
            let pixels = crate::hud_native::render(&rendered, &commands, 0., 0);
            (
                pixels,
                stats,
                crate::texture::ids(&rendered).len(),
                crate::hud::font_ids(&rendered).len(),
            )
        };
        let usage = Usage::read(&scene, textures, fonts);
        let signature = self.preview_key(e, &scene);
        self.preview = Some(Preview {
            scene,
            pixels,
            revision: signature,
            stats,
            usage,
        });
        Ok(())
    }
    fn preview_key(&self, e: &Editor, scene: &Scene) -> String {
        crate::scene_dependencies::hash((
            scene,
            e.assets.revision,
            e.registry_revision,
            self.sample,
            scene.preview_language,
            &self.conversation,
        ))
    }
}
fn parent_layers(root: &Path, registry: &Registry, id: &str) -> Result<Vec<Template>, String> {
    let class = registry.classes.get(id).ok_or("Class is missing.")?;
    class.parent.as_ref().map_or(Ok(vec![]), |parent| {
        composition::inherited(root, registry, parent)
    })
}
pub fn open(e: &mut Editor, id: &str) {
    if let Some(window) = &mut e.class_designer {
        if window.class == id {
            return;
        }
        if window.dirty() {
            window.error =
                Some("Save or discard this class composition before opening another class.".into());
            return;
        }
    }
    if e.blueprint_editor.dirty()
        && e.blueprint_editor
            .asset
            .as_ref()
            .is_some_and(|a| a.id == id)
    {
        e.last_error = Some("Save the open Blueprint before editing its UI composition.".into());
        return;
    }
    match Window::load(&e.root, &e.class_registry, id) {
        Ok(window) => {
            if e.blueprint_editor
                .asset
                .as_ref()
                .is_some_and(|a| a.id == id)
            {
                e.blueprint_editor.discard();
            }
            e.class_designer = Some(window)
        }
        Err(error) => e.last_error = Some(error),
    }
}
pub fn capture(
    root: &Path,
    registry: &Registry,
    id: &str,
    scene: &Scene,
    actor: usize,
) -> Result<(), String> {
    let mut window = Window::load(root, registry, id)?;
    if window.saved != Template::default() {
        return Err("This class already has authored composition. Edit it in the UI class designer; capturing a new root would break its placed instances.".into());
    }
    if parent_layers(root, registry, id)?
        .iter()
        .any(|layer| !layer.actors.is_empty())
    {
        return Err("This class inherits a composition. Edit its inherited controls in the UI class designer instead of replacing its root.".into());
    }
    let sources = composition::sources(root, registry)?;
    window.template = templates::capture_with_sources(scene, actor, registry, &sources)?.template;
    window.save(root, registry)
}
pub fn save_all(e: &mut Editor) -> Result<(), String> {
    let Some(mut window) = e.class_designer.take() else {
        return Ok(());
    };
    let changed = window.dirty();
    let result = if changed {
        window.save(&e.root, &e.class_registry)
    } else {
        Ok(())
    };
    if let Err(error) = &result {
        window.error = Some(error.clone());
    }
    e.class_designer = Some(window);
    if result.is_ok() && changed {
        refresh(e);
    }
    result
}
fn refresh(e: &mut Editor) {
    e.assets.index = crate::assets::scan(&e.root, &mut Default::default());
    e.assets.refresh();
    e.refresh_scripts();
}
/// Keep the two editing views on one Blueprint document; opening behavior must
/// never leave another editable snapshot that can overwrite its composition.
pub fn before_behavior(e: &mut Editor, id: &str) -> Result<(), String> {
    if let Some(window) = e.class_designer.as_ref().filter(|w| w.class == id) {
        if window.dirty() {
            return Err(
                "Save or discard the open UI composition before opening its Blueprint behavior."
                    .into(),
            );
        }
        e.class_designer = None;
    }
    Ok(())
}
fn conversation_controls(ui: &imgui::Ui, e: &mut Editor, w: &mut Window) {
    let state = &mut w.conversation;
    let mut selected = state.asset;
    if crate::blueprint_refs::asset_inspector(
        ui,
        "Conversation##class-preview",
        &mut selected,
        "Dialogue",
        &e.assets.index,
        "Select conversation",
    ) {
        state.select(selected);
    }
    if crate::blueprint_refs::asset_inspector(
        ui,
        "Timing##class-preview",
        &mut state.style,
        "DialogueStyle",
        &e.assets.index,
        "Widget timing",
    ) {
        state.restart();
    }
    if !state.languages.is_empty() {
        let label = state
            .languages
            .get(e.scene.preview_language)
            .map_or("Choose language", String::as_str);
        let mut changed = false;
        if let Some(_combo) = ui.begin_combo("Language##class-preview", label) {
            for (i, name) in state.languages.iter().enumerate() {
                if ui.selectable(format!("{name}##preview-language-{i}")) {
                    e.scene.preview_language = i;
                    changed = true;
                }
                crate::blueprint_editor::record_control(
                    ui,
                    &format!("conversation-preview:language:{i}"),
                );
            }
        }
        if changed {
            state.restart();
        }
        crate::blueprint_editor::record_control(ui, "conversation-preview:language");
    }
    let speaker = state
        .participant_labels
        .iter()
        .find(|(id, _)| Some(*id) == state.participant)
        .map_or("All speakers", |(_, name)| name);
    let mut changed = false;
    if let Some(_combo) = ui.begin_combo("Speaker##class-preview", speaker) {
        if ui.selectable("All speakers") {
            state.participant = None;
            state.entry = None;
            changed = true;
        }
        for (id, label) in &state.participant_labels {
            if ui.selectable(format!("{label}##preview-speaker-{id}")) {
                state.participant = Some(*id);
                state.entry = None;
                changed = true;
            }
            crate::blueprint_editor::record_control(
                ui,
                &format!("conversation-preview:speaker:{id}"),
            );
        }
    }
    crate::blueprint_editor::record_control(ui, "conversation-preview:speaker");
    let entry = state
        .node_labels
        .iter()
        .find(|(id, _)| Some(*id) == state.entry)
        .map_or("Conversation entry", |(_, label)| label);
    if let Some(_combo) = ui.begin_combo("Start at##class-preview", entry) {
        if ui.selectable("Conversation entry") {
            state.entry = None;
            state.participant = None;
            changed = true;
        }
        crate::blueprint_editor::record_control(ui, "conversation-preview:entry-default");
        for (id, label) in &state.node_labels {
            if state.participant.is_some()
                && state.node_speakers.get(id).copied().flatten() != state.participant
            {
                continue;
            }
            if ui.selectable(format!("{label}##preview-entry-{id}")) {
                state.entry = Some(*id);
                changed = true;
            }
            crate::blueprint_editor::record_control(
                ui,
                &format!("conversation-preview:entry:{id}"),
            );
        }
    }
    crate::blueprint_editor::record_control(ui, "conversation-preview:entry");
    if !state.condition_labels.is_empty() {
        let tree = ui.tree_node("Condition values##class-preview");
        crate::blueprint_editor::record_control(ui, "conversation-preview:conditions");
        if let Some(_tree) = tree {
            ui.child_window("preview-condition-values")
                .size([0., 110.])
                .build(|| {
                    for (id, label) in &state.condition_labels {
                        let choice = state
                            .conditions
                            .get(id)
                            .map_or(0, |value| if *value { 2 } else { 1 });
                        if let Some(_combo) = ui.begin_combo(
                            format!("{label}##preview-condition-{id}"),
                            ["Choose value", "False", "True"][choice],
                        ) {
                            for (i, name) in
                                ["Choose value", "False", "True"].into_iter().enumerate()
                            {
                                if ui.selectable(format!("{name}##preview-value-{id}-{i}")) {
                                    if i == 0 {
                                        state.conditions.remove(id);
                                    } else {
                                        state.conditions.insert(*id, i == 2);
                                    }
                                    changed = true;
                                }
                                crate::blueprint_editor::record_control(
                                    ui,
                                    &format!("conversation-preview:condition-value:{id}:{i}"),
                                );
                            }
                        }
                        crate::blueprint_editor::record_control(
                            ui,
                            &format!("conversation-preview:condition:{id}"),
                        );
                    }
                });
        }
    }
    if changed {
        state.restart();
    }
    if ui.button("Restart##class-preview") {
        state.restart();
    }
    crate::blueprint_editor::record_control(ui, "conversation-preview:restart");
    ui.same_line();
    ui.disabled(state.snapshot.is_none(), || {
        for (i, (label, button)) in [
            ("Confirm", crate::dialogue_preview::CONFIRM),
            ("Up", crate::dialogue_preview::UP),
            ("Down", crate::dialogue_preview::DOWN),
            ("Cancel", crate::dialogue_preview::CANCEL),
        ]
        .into_iter()
        .enumerate()
        {
            if i != 0 {
                ui.same_line();
            }
            if ui.button(format!("{label}##class-preview")) {
                state.press(button);
            }
            crate::blueprint_editor::record_control(
                ui,
                &format!("conversation-preview:{}", label.to_lowercase()),
            );
        }
        ui.same_line();
        if ui.button("+100 ms##class-preview") {
            state.step(100_000);
        }
        crate::blueprint_editor::record_control(ui, "conversation-preview:step");
    });
    if let Some(snapshot) = state.snapshot {
        let node = state
            .trace_labels
            .get(&snapshot.node)
            .map_or("", String::as_str);
        ui.text_wrapped(format!(
            "{} | {node} | Page {}/{} | Reveal {}/{} | Choice {}/{} | {}",
            snapshot.phase_name(),
            if snapshot.pages == 0 {
                0
            } else {
                snapshot.page + 1
            },
            snapshot.pages,
            snapshot.revealed,
            snapshot.glyphs,
            if snapshot.choices == 0 {
                0
            } else {
                snapshot.selected + 1
            },
            snapshot.choices,
            snapshot.result_name()
        ));
    }
    if !state.trace.is_empty()
        && let Some(_tree) = ui.tree_node("Preview events##class-preview")
    {
        ui.child_window("preview-events").size([0., 90.]).build(|| {
            for event in &state.trace {
                let label = state.trace_labels.get(&event.id).map_or("", String::as_str);
                ui.text(format!(
                    "{}: {label}",
                    if event.kind == 0 { "Entered" } else { "Action" }
                ));
            }
        });
    }
    ui.text_disabled(
        "Conditions use preview values; actions are recorded. Play runs project code.",
    );
}
pub fn window(ui: &imgui::Ui, e: &mut Editor) {
    let Some(mut w) = e.class_designer.take() else {
        return;
    };
    let mut opened = true;
    let mut discard = false;
    ui.window(format!(
        "UI Class: {}{}###class-designer",
        w.name,
        if w.dirty() { " *" } else { "" }
    ))
    .opened(&mut opened)
    .size([1180., 720.], imgui::Condition::FirstUseEver)
    .build(|| {
        if ui.button("Save") {
            match w.save(&e.root, &e.class_registry) {
                Ok(()) => refresh(e),
                Err(error) => w.error = Some(error),
            }
        }
        ui.same_line();
        if ui.button("Revert") {
            match Window::load(&e.root, &e.class_registry, &w.class) {
                Ok(fresh) => w = fresh,
                Err(error) => w.error = Some(error),
            }
        }
        ui.same_line();
        if ui.button("Undo") {
            w.undo();
        }
        ui.same_line();
        if ui.button("Redo") {
            w.redo();
        }
        ui.same_line();
        if ui.button("Behavior") {
            let result = if w.dirty() {
                w.save(&e.root, &e.class_registry)
            } else {
                Ok(())
            };
            if let Err(error) = result {
                w.error = Some(error);
            } else if let Some(class) = e.class_registry.classes.get(&w.class) {
                let binding = ClassDefaults {
                    name: class.cpp_name.clone(),
                    class_id: Some(class.id.clone()),
                    provider: class.provider.clone(),
                    backend: class.backend.clone(),
                    ..Default::default()
                };
                crate::blueprint_workflow::edit_binding(e, &binding);
                if matches!(w.storage, Storage::Blueprint(_)) {
                    discard = true;
                }
            }
        }
        ui.same_line();
        if ui.button("Discard and close") {
            discard = true;
        }
        ui.text_disabled(format!(
            "{} | {}",
            e.class_registry
                .classes
                .get(&w.class)
                .map_or("", |c| c.provider.id.as_str()),
            w.class
        ));
        if w.dialogue {
            ui.set_next_item_width(240.);
            ui.combo_simple_string(
                "Preview state",
                &mut w.sample,
                &[
                    "Authored defaults",
                    "Dialogue line sample",
                    "Choice sample",
                    "Closed sample",
                    "Prompt sample",
                    "Conversation",
                ],
            );
            crate::blueprint_editor::record_control(ui, "conversation-preview:mode");
        }
        ui.same_line();
        ui.set_next_item_width(150.);
        let mut size = match w.preview_size {
            Some([320, 240]) => 1,
            Some([640, 480]) => 2,
            _ => 0,
        };
        if ui.combo_simple_string(
            "Preview size",
            &mut size,
            &["Scene viewport", "320 x 240", "640 x 480"],
        ) {
            w.preview_size = match size {
                1 => Some([320, 240]),
                2 => Some([640, 480]),
                _ => None,
            };
        }
        if let Some(error) = &w.error {
            ui.text_colored([1., 0.4, 0.3, 1.], error);
        }
        let inherited = match parent_layers(&e.root, &e.class_registry, &w.class) {
            Ok(layers) => layers,
            Err(error) => {
                w.error = Some(error);
                return;
            }
        };
        let sources = match composition::sources(&e.root, &e.class_registry) {
            Ok(sources) => sources,
            Err(error) => {
                w.error = Some(error);
                return;
            }
        };
        let before = w.template.clone();
        let avail = ui.content_region_avail();
        let height = (avail[1] - 24.).max(100.);
        ui.child_window("class-hierarchy")
            .size([210., height])
            .border(true)
            .build(|| {
                ui.text("Controls");
                let mut doc = Document {
                    name: &w.name,
                    class: Some(&w.class),
                    template: &mut w.template,
                    inherited: &inherited,
                    sources: Some(&sources),
                    owner: Some(&w.class),
                };
                if let Err(error) =
                    template_editor::class_browser(ui, &mut doc, &e.class_registry, &mut w.controls)
                {
                    w.error = Some(error);
                }
            });
        ui.same_line();
        ui.child_window("class-preview")
            .size([(avail[0] - 550.).max(120.), height])
            .border(true)
            .build(|| {
                if w.dialogue && w.sample == 5 {
                    conversation_controls(ui, e, &mut w);
                }
                if let Err(error) = w.update_preview(e) {
                    w.preview_error = Some(error);
                    w.preview = None;
                } else {
                    w.preview_error = None;
                }
                if let Some(error) = &w.preview_error {
                    ui.text_wrapped(error);
                }
                let _ = canvas(ui, &mut w);
            });
        ui.same_line();
        ui.child_window("class-details")
            .size([0., height])
            .border(true)
            .build(|| {
                ui.text("Properties");
                if component_toolbar(ui, e, &mut w) {
                    return;
                }
                let mut doc = Document {
                    name: &w.name,
                    class: Some(&w.class),
                    template: &mut w.template,
                    inherited: &inherited,
                    sources: Some(&sources),
                    owner: Some(&w.class),
                };
                let builtin = w.controls.component().is_none_or(|id| {
                    w.preview.as_ref().is_some_and(|p| {
                        p.scene.actors.iter().flat_map(|a| &a.components).any(|c| {
                            c.id == id
                                && c.class
                                    .class_id
                                    .as_deref()
                                    .is_some_and(crate::actor_components::native)
                        })
                    })
                });
                if builtin
                    && let Err(error) = template_editor::class_draw(
                        ui,
                        &mut doc,
                        &e.class_registry,
                        &mut w.controls,
                        &crate::hud_editor::Resources {
                            index: &e.assets.index,
                            strings: crate::localization::project_table(&e.assets.index)
                                .as_ref()
                                .map(|(_, table)| table),
                            language: e.scene.preview_language,
                        },
                    )
                {
                    w.error = Some(error);
                }
                reflected_properties(ui, e, &mut w);
            });
        // Continuous canvas drags form one undo step, like the scene editor.
        w.finish_edit(before, ui.is_mouse_down(imgui::MouseButton::Left));
    });
    if !opened && w.dirty() {
        w.error = Some("Save or discard changes before closing this class.".into());
        opened = true;
    }
    let open_strings = w.controls.take_strings_request();
    if opened && !discard {
        e.class_designer = Some(w);
    }
    if open_strings {
        crate::localization_editor::open(e);
    }
}
type PropertyOwner<'a> = (
    &'a crate::actor_document::ClassReference,
    &'a std::collections::BTreeMap<String, serde_json::Value>,
    &'a std::collections::BTreeSet<String>,
);
fn property_owner(actor: &crate::scene::Actor, owner: Uuid) -> Option<PropertyOwner<'_>> {
    if actor.id == owner {
        Some((&actor.class, &actor.properties, &actor.overrides))
    } else {
        actor
            .components
            .iter()
            .find(|c| c.id == owner)
            .map(|c| (&c.class, &c.properties, &c.overrides))
    }
}
fn owns_actor(template: &Template, actor: Uuid) -> bool {
    !template.instances.contains(&actor)
        && template.actors.iter().any(|item| item.entity.id == actor)
}
fn local_component(template: &Template, actor: &crate::scene::Actor, component: Uuid) -> bool {
    if owns_actor(template, actor.id) {
        return true;
    }
    template
        .overrides
        .get(&actor.id)
        .and_then(|o| o.members.get(&format!("/components/{component}")))
        .is_some_and(serde_json::Value::is_object)
}
fn property_path(actor: Uuid, owner: Uuid, key: &str) -> String {
    templates::property_path(actor, owner, key)
}
fn is_reference(ty: &crate::reflection_schema::Type) -> bool {
    matches!(
        ty,
        crate::reflection_schema::Type::ObjectRef { .. }
            | crate::reflection_schema::Type::ActorRef { .. }
            | crate::reflection_schema::Type::ComponentRef { .. }
    )
}
fn authored_properties(
    template: &Template,
    actor: &crate::scene::Actor,
    owner: Uuid,
    registry: &Registry,
) -> std::collections::BTreeSet<String> {
    let Some((class, _, _)) = property_owner(actor, owner) else {
        return Default::default();
    };
    let local = owns_actor(template, actor.id)
        .then(|| {
            template
                .actors
                .iter()
                .find(|i| i.entity.id == actor.id)
                .unwrap()
        })
        .and_then(|item| property_owner(&item.entity, owner));
    registry
        .properties(&class.name)
        .into_iter()
        .filter(|p| {
            if template
                .references
                .iter()
                .any(|r| r.owner == owner && r.member == p.id)
            {
                return true;
            }
            if local.is_some_and(|(_, values, _)| values.contains_key(&p.name)) {
                return true;
            }
            let prefix = property_path(actor.id, owner, &p.name);
            template.overrides.get(&actor.id).is_some_and(|edits| {
                edits.members.iter().any(|(path, value)| {
                    path == &prefix
                        || path.starts_with(&format!("{prefix}/"))
                        || prefix
                            .strip_prefix(&format!("{path}/"))
                            .is_some_and(|tail| value.pointer(&format!("/{tail}")).is_some())
                })
            })
        })
        .map(|p| p.name.clone())
        .collect()
}
fn reset_property(template: &mut Template, actor: &crate::scene::Actor, owner: Uuid, key: &str) {
    if owns_actor(template, actor.id) {
        let local = &mut template
            .actors
            .iter_mut()
            .find(|i| i.entity.id == actor.id)
            .unwrap()
            .entity;
        if owner == actor.id {
            local.properties.remove(key);
            local.overrides.remove(key);
        } else if let Some(c) = local.components.iter_mut().find(|c| c.id == owner) {
            c.properties.remove(key);
            c.overrides.remove(key);
        }
    }
    if let Some(edits) = template.overrides.get_mut(&actor.id) {
        templates::reset_property_members(&mut edits.members, actor.id, owner, key);
    }
    if template
        .overrides
        .get(&actor.id)
        .is_some_and(|e| e.members.is_empty() && e.parent.is_none())
    {
        template.overrides.remove(&actor.id);
    }
}
fn component_toolbar(ui: &imgui::Ui, e: &mut Editor, w: &mut Window) -> bool {
    let Some(actor) = w
        .preview
        .as_ref()
        .and_then(|p| {
            w.controls
                .selected()
                .and_then(|id| p.scene.actors.iter().find(|a| a.id == id))
        })
        .cloned()
    else {
        return false;
    };
    let actor_id = actor.id;
    let mut add = None;
    if ui.button("Add Component##class") {
        ui.open_popup("class-add-component");
    }
    crate::blueprint_editor::record_control(ui, "class-component-add");
    if let Some(_popup) = ui.begin_popup("class-add-component")
        && let Ok(model) = e.class_registry.model()
    {
        for (group, class, label) in crate::actor_components::addable_classes(&model, &actor) {
            if ui.menu_item(format!("{label} ({group})")) {
                add = Some(class.clone());
            }
            crate::blueprint_editor::record_control(ui, &format!("class-add-component:{class}"));
        }
    }
    if let Some(class) = add {
        match w.add_component(&e.root, &e.class_registry, actor_id, &class) {
            Ok(id) => w.controls.select_component(actor_id, id),
            Err(error) => w.error = Some(error),
        }
        return true;
    }
    let owner = w.controls.component().unwrap_or(actor_id);
    let Some((class, _, _)) = property_owner(&actor, owner) else {
        return false;
    };
    if owner != actor_id {
        ui.separator();
        let component = actor.components.iter().find(|c| c.id == owner).unwrap();
        ui.text(&component.name);
        ui.text_disabled(&class.name);
        crate::gui::edit_class_button(ui, e, class);
        let blocked = crate::actor_components::component_removal_refusal(component).is_some()
            || !local_component(&w.template, &actor, owner);
        let disabled = ui.begin_disabled(blocked);
        let remove = ui.button("Remove component");
        crate::blueprint_editor::record_control(ui, "class-component-remove");
        drop(disabled);
        if remove {
            match w.remove_component(&e.root, &e.class_registry, actor_id, owner) {
                Ok(()) => w.controls.select(Some(actor_id)),
                Err(error) => w.error = Some(error),
            }
            return true;
        }
    }
    false
}
fn reflected_properties(ui: &imgui::Ui, e: &mut Editor, w: &mut Window) {
    let Some(preview) = &w.preview else {
        return;
    };
    let Some(actor) = w
        .controls
        .selected()
        .and_then(|id| preview.scene.actors.iter().find(|a| a.id == id))
    else {
        return;
    };
    let actor = actor.clone();
    let actor_id = actor.id;
    let owner = w.controls.component().unwrap_or(actor_id);
    let Some((class, properties, _)) = property_owner(&actor, owner) else {
        w.controls.select(Some(actor_id));
        return;
    };
    let explicit = authored_properties(&w.template, &actor, owner, &e.class_registry);
    let properties = properties
        .iter()
        .filter(|(key, _)| {
            // Built-in projections keep their existing specialized controls; they
            // are authored component data, not orphan reflected C++ properties.
            !class
                .class_id
                .as_deref()
                .is_some_and(crate::actor_components::native)
                || e.class_registry
                    .properties(&class.name)
                    .iter()
                    .any(|p| p.name == **key)
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let context = crate::gui::PropertyContext {
        scene: &preview.scene,
        registry: &e.class_registry,
        assets: &e.assets.index,
    };
    if let Some(action) = crate::gui::reflected_properties(
        ui,
        &context,
        "class-property",
        &class.name,
        &properties,
        &explicit,
    ) && let Err(error) = w.property(&e.root, &e.class_registry, actor_id, owner, action)
    {
        w.error = Some(error);
    }
}
fn canvas(ui: &imgui::Ui, w: &mut Window) -> Option<([f32; 2], f32)> {
    let preview = w.preview.as_ref()?;
    ui.text_disabled(if w.dialogue && w.sample == 5 {
        "Conversation preview"
    } else {
        "Class defaults"
    });
    let [width, height] = preview.scene.display_size.map(f32::from);
    let avail = ui.content_region_avail();
    let scale = (avail[0] / width)
        .min((avail[1] - 66.).max(1.) / height)
        .max(0.01);
    let origin = ui.cursor_screen_pos();
    let size = [width * scale, height * scale];
    if let Some(texture) = w.texture {
        imgui::Image::new(texture, size).build(ui);
    } else {
        ui.invisible_button("class-preview-loading", size);
    }
    let hovered = ui.is_item_hovered();
    let mouse = ui.io().mouse_pos;
    let point = [
        (mouse[0] - origin[0]) / scale,
        height - (mouse[1] - origin[1]) / scale,
    ];
    let boxes = crate::hud::layouts(&preview.scene);
    if hovered && ui.is_mouse_clicked(imgui::MouseButton::Left) {
        let selected = crate::hud::order(&preview.scene)
            .into_iter()
            .rev()
            .find(|i| {
                boxes
                    .get(*i)
                    .copied()
                    .flatten()
                    .is_some_and(|p| contains(&p, point))
            });
        w.controls
            .select(selected.map(|i| preview.scene.actors[i].id));
        w.drag = selected.map(|i| preview.scene.actors[i].id);
        w.drag_before = Some(w.template.clone());
    }
    if ui.is_mouse_dragging(imgui::MouseButton::Left)
        && let Some(id) = w.drag
        && let Some(original) = preview.scene.actors.iter().find(|a| a.id == id)
    {
        let mut actor = original.clone();
        let overlay = actor.canvas.as_ref().is_some_and(|c| c.screen_overlay);
        if let Some(rect) = &mut actor.rect {
            let index = preview
                .scene
                .actors
                .iter()
                .position(|a| a.id == id)
                .unwrap();
            let parent_angle = if overlay {
                0.
            } else {
                preview
                    .scene
                    .spatial_parent(index)
                    .and_then(|i| boxes.get(i).copied().flatten())
                    .as_ref()
                    .map_or(0., crate::hud_editor::placement_angle)
            };
            let raw = ui.io().mouse_delta;
            let delta =
                crate::hud_editor::unrotate([raw[0] / scale, -raw[1] / scale], parent_angle);
            rect.position[0] += delta[0];
            rect.position[1] += delta[1];
            crate::actor_components::sync(&mut actor);
            let mut candidate = preview.scene.clone();
            candidate.actors[index] = actor.clone();
            if candidate.validate().is_err() {
                return Some((origin, scale));
            }
            if let Err(error) =
                template_editor::apply_entity_changes(&mut w.template, original, &actor)
            {
                w.error = Some(error);
            }
        }
    }
    if let Some(selected) = w
        .controls
        .selected()
        .and_then(|id| preview.scene.actors.iter().position(|a| a.id == id))
        && let Some(place) = boxes.get(selected).copied().flatten()
    {
        let [a, b, c, d] = place.corners.map(|p| {
            [
                origin[0] + p[0] * scale,
                origin[1] + (height - p[1]) * scale,
            ]
        });
        let draw = ui.get_window_draw_list();
        for (from, to) in [(a, b), (b, d), (d, c), (c, a)] {
            draw.add_line(from, to, [0.3, 0.8, 1., 1.])
                .thickness(2.)
                .build();
        }
    }
    ui.text_disabled(format!(
        "{} x {} | {} glyphs | {} dropped",
        width as u16, height as u16, preview.stats[1], preview.stats[4]
    ));
    let usage = &preview.usage;
    ui.text_disabled(format!(
        "Class: {} actors | {} components | {} Text controls",
        usage.actors, usage.components, usage.text_controls
    ));
    ui.text_disabled(format!(
        "Text slots: {} B | {} textures | {} fonts{}",
        usage.actors * usage.text_record_bytes,
        usage.textures,
        usage.fonts,
        if usage.builtin_font {
            " + built-in"
        } else {
            ""
        }
    ));
    if ui.is_item_hovered() {
        ui.tooltip(|| {
            ui.text(format!("Busiest actor: {} / {} components", usage.max_components, crate::actor_components::CAPACITY));
            ui.text(format!("Each occupied actor slot reserves {} B for Text, including its {} B UTF-8 buffer.",
                usage.text_record_bytes, usage.text_buffer_bytes));
            ui.text(format!("Authored Text controls use {} B of buffer capacity.", usage.text_controls * usage.text_buffer_bytes));
            ui.text("Slot capacity usage; Build reports the complete linked RAM allocation.");
            ui.text("Resource banks include inactive controls and conversation portraits; font atlases are counted with fonts.");
            let b = &preview.scene.hud_budget;
            ui.text(format!("Frame: rectangles {}/{}, glyphs {}/{}, labels {}/{}, rotated {}/{}; {} images",
                preview.stats[0], b.rectangles, preview.stats[1], b.glyphs,
                preview.stats[2], b.texts, preview.stats[5], b.rotated, preview.stats[3]));
        });
    }
    Some((origin, scale))
}

fn contains(place: &crate::hud::Placement, point: [f32; 2]) -> bool {
    let [a, _, c, d] = place.corners;
    let u = [d[0] - c[0], d[1] - c[1]];
    let v = [a[0] - c[0], a[1] - c[1]];
    let q = [point[0] - c[0], point[1] - c[1]];
    let det = u[0] * v[1] - u[1] * v[0];
    if det.abs() < 1e-5 {
        return false;
    }
    let x = (q[0] * v[1] - q[1] * v[0]) / det;
    let y = (u[0] * q[1] - u[1] * q[0]) / det;
    (0. ..=1.).contains(&x) && (0. ..=1.).contains(&y)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use composition::tests::{BASE, base_template, fixture};
    pub(crate) fn focus_project() -> (PathBuf, Scene, usize) {
        let (root, registry, catalog) = fixture();
        let mut base = base_template(&root, &registry);
        let parent = base.actors[0].entity.id;
        let first = base.actors[1].entity.id;
        base.actors[1].entity.name = "First".into();
        base.actors[1].entity.text.as_mut().unwrap().text = "A".into();
        base.actors[1].entity.focusable = Some(crate::hud::Focusable {
            outline: true,
            ..crate::hud::Focusable::lit()
        });
        crate::actor_components::sync(&mut base.actors[1].entity);
        let mut second = base.actors[1].clone();
        let map = crate::actor_document::fresh_identities(std::slice::from_ref(&second.entity));
        crate::actor_document::remap_actor(&mut second.entity, &map);
        second.entity.name = "Second".into();
        second.entity.text.as_mut().unwrap().text = "B".into();
        second.entity.rect.as_mut().unwrap().position[1] += 40.;
        crate::actor_components::sync(&mut second.entity);
        let second_id = second.entity.id;
        base.actors.push(second);
        composition::create(
            &root,
            "assets/Base.epokclass",
            &composition::Document::new(BASE.into(), base),
            &registry,
        )
        .unwrap();
        let mut editor = Editor::new(root.clone());
        editor.class_registry = registry;
        editor.catalog = catalog;
        let mut ctx = crate::gui::tests::imgui_context();
        ctx.io_mut().display_size = [1100., 900.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts().build_rgba32_texture();
        fn frame(ctx: &mut imgui::Context, e: &Editor, w: &mut Window) {
            crate::blueprint_editor::clear_controls();
            let inherited = parent_layers(&e.root, &e.class_registry, &w.class).unwrap();
            let sources = composition::sources(&e.root, &e.class_registry).unwrap();
            let before = w.template.clone();
            let ui = ctx.frame();
            ui.window("Component focus authoring")
                .position([0., 0.], imgui::Condition::Always)
                .size([1000., 850.], imgui::Condition::Always)
                .build(|| {
                    template_editor::class_draw(
                        ui,
                        &mut Document {
                            name: &w.name,
                            class: Some(&w.class),
                            template: &mut w.template,
                            inherited: &inherited,
                            sources: Some(&sources),
                            owner: Some(&w.class),
                        },
                        &e.class_registry,
                        &mut w.controls,
                        &crate::hud_editor::Resources {
                            index: &e.assets.index,
                            strings: None,
                            language: 0,
                        },
                    )
                    .unwrap();
                    w.finish_edit(before, ui.is_mouse_down(imgui::MouseButton::Left));
                });
            ctx.render();
            assert!(w.error.is_none(), "{:?}", w.error);
            assert!(
                crate::blueprint_editor::control_position("hud-band:Rect Transform").is_none(),
                "Selecting another component must not draw RectTransform fields"
            );
        }
        fn click(ctx: &mut imgui::Context, e: &Editor, w: &mut Window, key: &str) {
            frame(ctx, e, w);
            let point = crate::blueprint_editor::control_position(key)
                .unwrap_or_else(|| panic!("Missing {key}"));
            ctx.io_mut().add_mouse_pos_event(point);
            frame(ctx, e, w);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(ctx, e, w);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(ctx, e, w);
        }
        for name in ["CppWidget", "BP_Widget", "LuaWidget"] {
            let class = editor.class_registry.named(name).unwrap().id.clone();
            let mut w = Window::load(&root, &editor.class_registry, &class).unwrap();
            let scene = w.resolve(&root, &editor.class_registry).unwrap();
            let component = |owner, class| {
                scene
                    .actors
                    .iter()
                    .find(|a| a.id == owner)
                    .unwrap()
                    .components
                    .iter()
                    .find(|c| c.class.class_id.as_deref() == Some(class))
                    .unwrap()
                    .id
            };
            for (actor, part, field, target) in [
                (
                    parent,
                    crate::object_model::CANVAS_COMPONENT_ID,
                    "Initial focus",
                    first,
                ),
                (
                    first,
                    crate::object_model::FOCUSABLE_COMPONENT_ID,
                    "Right",
                    second_id,
                ),
                (
                    second_id,
                    crate::object_model::FOCUSABLE_COMPONENT_ID,
                    "Left",
                    first,
                ),
            ] {
                w.controls.select_component(actor, component(actor, part));
                frame(&mut ctx, &editor, &mut w);
                frame(&mut ctx, &editor, &mut w);
                click(&mut ctx, &editor, &mut w, &format!("focus-field:{field}"));
                click(
                    &mut ctx,
                    &editor,
                    &mut w,
                    &format!("focus-choice:{field}:{target}"),
                );
            }
            w.undo();
            assert!(
                w.resolve(&root, &editor.class_registry)
                    .unwrap()
                    .actors
                    .iter()
                    .find(|a| a.id == second_id)
                    .unwrap()
                    .focusable
                    .as_ref()
                    .unwrap()
                    .navigation
                    .is_none()
            );
            w.redo();
            w.save(&root, &editor.class_registry).unwrap();
            let reopened = Window::load(&root, &editor.class_registry, &class)
                .unwrap()
                .resolve(&root, &editor.class_registry)
                .unwrap();
            assert_eq!(
                reopened.actors[0]
                    .canvas
                    .as_ref()
                    .unwrap()
                    .initial_focus
                    .unwrap()
                    .target,
                Some(first)
            );
            let text = component(first, crate::object_model::TEXT_COMPONENT_ID);
            w.controls.select_component(first, text);
            frame(&mut ctx, &editor, &mut w);
            assert!(crate::blueprint_editor::control_position("hud-font").is_some());
            assert!(crate::blueprint_editor::control_position("focus-field:Right").is_none());
        }
        drop(ctx);
        let classes = ["CppWidget", "BP_Widget", "LuaWidget"].map(|name| {
            crate::blueprint_refs::compact_id(&editor.class_registry.named(name).unwrap().id)
        });
        let probe = r#"#pragma once
#include "WidgetClasses.hpp"
extern "C" {inline uint32_t widget_focus_probe[5]={};}
class EPOK_CLASS(Blueprintable,Id="91c0b342-0754-4ca0-a9ef-9c45d62a5b39") WidgetFocusProbe:public epok::Actor3D {
 bool requested=false;
public:
 void on_frame(uint32_t) override {
  auto* registry=epok::active_object_registry;auto* level=registry->resolve<epok::Level>(level_id());const uint64_t classes[]={CLASSES};
  if(!requested){requested=true;for(auto id:classes)if(const auto* type=epok::find_object_class(id))level->spawn_actor(*type,"Dynamic widget");return;}
  unsigned count=0,mask=0;bool valid=true;epok::ObjectId seen[12];
  for(size_t i=0;i<level->actor_count();++i){auto* actor=registry->resolve<epok::Actor>(level->actor_at(i));if(!actor)continue;
   for(unsigned n=0;n<3;++n)if(actor->class_id()==classes[n]){auto* widget=static_cast<CppWidgetBase*>(actor);auto* text=registry->resolve<epok::TextComponent>(widget->label);auto* first=text?text->get_owner():nullptr;
    const auto a=first?epok::entity_index(first->data()):-1;const auto b=first?first->data()->focusable.neighbors[1]:-1;
    epok::Actor* second=nullptr;if(b>=0)for(size_t j=0;j<level->actor_count();++j){auto* candidate=registry->resolve<epok::Actor>(level->actor_at(j));if(candidate&&epok::entity_index(candidate->data())==b){second=candidate;break;}}
    valid&=a>=0&&second&&actor->data()->canvas.focused==a;
    if(first&&second){valid&=second->data()->focusable.neighbors[0]==a&&first->data()->parent==epok::entity_index(actor->data())&&second->data()->parent==epok::entity_index(actor->data());}
    for(unsigned k=0;k<count;++k)valid&=first&&seen[k]!=first->id();if(count<12)seen[count++]=first?first->id():epok::ObjectId{};else valid=false;mask|=1u<<n;
   }
  }
  data()->transform.rotation[0]=epok::Fixed(int(mask),0);data()->transform.rotation[1]=epok::Fixed(int(count),0);data()->transform.rotation[2]=epok::Fixed(valid?1:0,0);
  widget_focus_probe[0]=mask;widget_focus_probe[1]=count;widget_focus_probe[2]=valid;widget_focus_probe[3]=epok::hud_stats.glyphs;widget_focus_probe[4]=epok::hud_stats.dropped;
 }
};
"#.replace("CLASSES", &classes.map(|id| format!("UINT64_C({id})")).join(","));
        std::fs::write(root.join("assets/scripts/WidgetFocusProbe.hpp"), probe).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.truncate(1);
        for name in ["CppWidget", "BP_Widget", "LuaWidget"] {
            let class = &registry.named(name).unwrap().id;
            let placed =
                crate::editor::place_class(&mut scene, &root, &registry, &model, class).unwrap();
            scene
                .duplicate_actor_branch(scene.actors[placed].id)
                .unwrap();
        }
        let probe = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            "91c0b342-0754-4ca0-a9ef-9c45d62a5b39",
        )
        .unwrap();
        scene
            .save(&crate::workspace::startup_scene(&root).unwrap())
            .unwrap();
        (root, scene, probe)
    }
    pub(crate) fn focus_lifetime_project() -> (PathBuf, Scene, usize) {
        let (root, scene, probe) = focus_project();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let classes = ["CppWidget", "BP_Widget", "LuaWidget"]
            .map(|name| crate::blueprint_refs::compact_id(&registry.named(name).unwrap().id));
        let source = include_str!("../tests/fixtures/native-scenes/WidgetFocusLifetimeProbe.hpp")
            .replace(
                "CLASSES",
                &classes.map(|id| format!("UINT64_C({id})")).join(","),
            );
        std::fs::write(root.join("assets/scripts/WidgetFocusProbe.hpp"), source).unwrap();
        (root, scene, probe)
    }
    #[test]
    fn widget_resource_inspector_edits_and_previews_project_assets_in_all_providers() {
        let (root, registry, catalog) = fixture();
        std::fs::write(
            root.join("assets/Widget.ttf"),
            include_bytes!("../resources/editor/Roboto-Regular.ttf"),
        )
        .unwrap();
        let mut pixels = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut pixels, 2, 2);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&[40, 60, 80, 255].repeat(4))
                .unwrap();
        }
        std::fs::write(root.join("assets/Widget.png"), pixels).unwrap();
        let font = crate::assets::commit(
            crate::font_asset::prepare(
                &root,
                "assets/Widget.ttf",
                "assets/WidgetFont.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let texture = crate::assets::commit(
            crate::texture::prepare(
                &root,
                "assets/Widget.png",
                "assets/WidgetTexture.epokasset",
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let table = crate::localization::StringTable {
            languages: vec!["English".into(), "Español".into()],
            entries: vec![crate::localization::Entry {
                key: "widget.greeting".into(),
                values: vec!["Welcome".into(), "¡Hola!".into()],
            }],
            ..Default::default()
        };
        crate::localization::create(&root, "assets/WidgetStrings.epokasset", &table).unwrap();
        let mut base = base_template(&root, &registry);
        base.actors[1].entity.image = Some(Default::default());
        crate::actor_components::sync(&mut base.actors[1].entity);
        composition::create(
            &root,
            "assets/Base.epokclass",
            &composition::Document::new(BASE.into(), base),
            &registry,
        )
        .unwrap();
        let mut editor = Editor::new(root.clone());
        editor.class_registry = registry;
        editor.catalog = catalog;
        editor.assets.index = crate::assets::scan(&root, &mut Default::default());
        assert_eq!(
            crate::blueprint_refs::asset_choices(&editor.assets.index, "Font"),
            vec![(font.to_string(), "assets/Widget.ttf".into())]
        );
        assert_eq!(
            crate::blueprint_refs::asset_choices(&editor.assets.index, "Texture"),
            vec![(texture.to_string(), "assets/Widget.png".into())]
        );
        let mut conflicting = editor.assets.index.clone();
        conflicting
            .assets
            .get_mut(&font)
            .unwrap()
            .push(editor.assets.index.resolve(font).unwrap().clone());
        assert!(crate::blueprint_refs::asset_choices(&conflicting, "Font").is_empty());
        let mut ctx = crate::gui::tests::imgui_context();
        ctx.io_mut().display_size = [1500., 2400.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts().build_rgba32_texture();
        fn frame(ctx: &mut imgui::Context, e: &Editor, w: &mut Window) {
            let inherited = parent_layers(&e.root, &e.class_registry, &w.class).unwrap();
            let sources = composition::sources(&e.root, &e.class_registry).unwrap();
            let before = w.template.clone();
            let ui = ctx.frame();
            ui.window("Widget resources acceptance")
                .position([0., 0.], imgui::Condition::Always)
                .size([1400., 2300.], imgui::Condition::Always)
                .build(|| {
                    template_editor::class_draw(
                        ui,
                        &mut Document {
                            name: &w.name,
                            class: Some(&w.class),
                            template: &mut w.template,
                            inherited: &inherited,
                            sources: Some(&sources),
                            owner: Some(&w.class),
                        },
                        &e.class_registry,
                        &mut w.controls,
                        &crate::hud_editor::Resources {
                            index: &e.assets.index,
                            strings: crate::localization::project_table(&e.assets.index)
                                .as_ref()
                                .map(|(_, table)| table),
                            language: e.scene.preview_language,
                        },
                    )
                    .unwrap();
                    w.finish_edit(before, ui.is_mouse_down(imgui::MouseButton::Left));
                });
            ctx.render();
            assert!(w.error.is_none(), "{:?}", w.error);
        }
        fn click(ctx: &mut imgui::Context, e: &Editor, w: &mut Window, key: &str) {
            frame(ctx, e, w);
            let point = crate::blueprint_editor::control_position(key)
                .unwrap_or_else(|| panic!("Missing {key}"));
            ctx.io_mut().add_mouse_pos_event(point);
            frame(ctx, e, w);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(ctx, e, w);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(ctx, e, w);
        }
        for name in ["CppWidget", "BP_Widget", "LuaWidget"] {
            let class = editor.class_registry.named(name).unwrap().id.clone();
            let mut window = Window::load(&root, &editor.class_registry, &class).unwrap();
            let scene = window.resolve(&root, &editor.class_registry).unwrap();
            let label = scene.actors.iter().find(|a| a.text.is_some()).unwrap().id;
            window.controls.select(Some(label));
            frame(&mut ctx, &editor, &mut window);
            frame(&mut ctx, &editor, &mut window);
            for (field, id) in [("hud-font", font), ("hud-texture", texture)] {
                click(&mut ctx, &editor, &mut window, field);
                click(
                    &mut ctx,
                    &editor,
                    &mut window,
                    &format!("reference-choice:{id}"),
                );
            }
            click(&mut ctx, &editor, &mut window, "hud-string-key");
            click(&mut ctx, &editor, &mut window, "hud-key:widget.greeting");
            click(&mut ctx, &editor, &mut window, "hud-open-strings");
            assert!(window.controls.take_strings_request());
            assert!(!window.controls.take_strings_request());
            window.undo();
            assert!(
                window
                    .resolve(&root, &editor.class_registry)
                    .unwrap()
                    .actors
                    .iter()
                    .find(|a| a.id == label)
                    .unwrap()
                    .text
                    .as_ref()
                    .unwrap()
                    .key
                    .is_empty()
            );
            window.redo();
            window.save(&root, &editor.class_registry).unwrap();
            let mut reopened = Window::load(&root, &editor.class_registry, &class).unwrap();
            let scene = reopened.resolve(&root, &editor.class_registry).unwrap();
            let control = scene.actors.iter().find(|a| a.id == label).unwrap();
            assert_eq!(control.text.as_ref().unwrap().font, Some(font));
            assert_eq!(control.image.as_ref().unwrap().texture, Some(texture));
            assert_eq!(control.text.as_ref().unwrap().key, "widget.greeting");
            assert_eq!(control.text.as_ref().unwrap().text, "Authored widget");
            editor.scene.preview_language = 0;
            reopened.update_preview(&editor).unwrap();
            let english = reopened.preview.as_ref().unwrap().pixels.clone();
            assert_eq!(reopened.preview.as_ref().unwrap().stats[4], 0);
            assert_eq!(reopened.preview.as_ref().unwrap().stats[1], 7);
            editor.scene.preview_language = 1;
            reopened.update_preview(&editor).unwrap();
            let preview = reopened.preview.as_ref().unwrap();
            assert_eq!(preview.stats[1], 6);
            assert_eq!(preview.stats[4], 0);
            assert_ne!(preview.pixels, english);
            assert_eq!(
                preview
                    .scene
                    .strings
                    .as_ref()
                    .unwrap()
                    .lookup("widget.greeting", 1),
                Some("¡Hola!")
            );
            assert!(!root.join(".epok/native-preview").exists());
        }
        if let Some(destination) = std::env::var_os("EPOK_WIDGET_RESOURCE_EVIDENCE_DIR") {
            fn copy(source: &Path, destination: &Path) {
                std::fs::create_dir_all(destination).unwrap();
                for entry in std::fs::read_dir(source).unwrap() {
                    let entry = entry.unwrap();
                    let kind = entry.file_type().unwrap();
                    assert!(!kind.is_symlink());
                    let target = destination.join(entry.file_name());
                    if kind.is_dir() {
                        copy(&entry.path(), &target);
                    } else {
                        std::fs::copy(entry.path(), target).unwrap();
                    }
                }
            }
            let destination = PathBuf::from(destination);
            assert!(
                !destination.exists(),
                "Evidence must use a fresh project directory"
            );
            for folder in ["assets", "ProjectSettings"] {
                copy(&root.join(folder), &destination.join(folder));
            }
            for entry in std::fs::read_dir(&root).unwrap() {
                let path = entry.unwrap().path();
                if path
                    .extension()
                    .is_some_and(|extension| extension == "epokproject")
                {
                    std::fs::copy(&path, destination.join(path.file_name().unwrap())).unwrap();
                }
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn component_inspector_uses_real_input_in_all_providers() {
        let (root, registry, catalog, ids) = component_fixture();
        let mut editor = Editor::new(root.clone());
        editor.class_registry = registry;
        editor.catalog = catalog;
        let mut ctx = crate::gui::tests::imgui_context();
        crate::gui::configure_input(ctx.io_mut());
        ctx.io_mut().display_size = [1200., 900.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts().build_rgba32_texture();
        fn frame(ctx: &mut imgui::Context, e: &mut Editor, w: &mut Window) {
            w.update_preview(e).unwrap();
            let before = w.template.clone();
            let ui = ctx.frame();
            ui.window("Component authoring acceptance")
                .position([0., 0.], imgui::Condition::Always)
                .size([1100., 860.], imgui::Condition::Always)
                .build(|| {
                    if !component_toolbar(ui, e, w) {
                        reflected_properties(ui, e, w);
                    }
                    w.finish_edit(before, ui.is_mouse_down(imgui::MouseButton::Left));
                });
            ctx.render();
            assert!(w.error.is_none(), "{:?}", w.error);
        }
        fn click(ctx: &mut imgui::Context, e: &mut Editor, w: &mut Window, key: &str) {
            frame(ctx, e, w);
            let point = crate::blueprint_editor::control_position(key)
                .unwrap_or_else(|| panic!("Missing {key}"));
            ctx.io_mut().add_mouse_pos_event(point);
            frame(ctx, e, w);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(ctx, e, w);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(ctx, e, w);
        }
        for (name, part) in ["CppWidget", "BP_Widget", "LuaWidget"].into_iter().zip(ids) {
            let class = editor.class_registry.named(name).unwrap().id.clone();
            let mut w = Window::load(&root, &editor.class_registry, &class).unwrap();
            let scene = w.resolve(&root, &editor.class_registry).unwrap();
            let actor = scene.actors[0].id;
            let target = scene.actors[0].properties["label"]
                .as_str()
                .unwrap()
                .to_owned();
            w.controls.select(Some(actor));
            frame(&mut ctx, &mut editor, &mut w);
            frame(&mut ctx, &mut editor, &mut w);
            let part_name = editor.class_registry.classes[&part].cpp_name.clone();
            click(&mut ctx, &mut editor, &mut w, "class-component-add");
            frame(&mut ctx, &mut editor, &mut w);
            click(
                &mut ctx,
                &mut editor,
                &mut w,
                &format!("class-add-component:{part_name}"),
            );
            let component = w
                .controls
                .component()
                .expect("The added component is selected");
            click(&mut ctx, &mut editor, &mut w, "class-property:value:amount");
            frame(&mut ctx, &mut editor, &mut w);
            assert!(ctx.io().want_text_input);
            for c in "31".chars() {
                ctx.io_mut().add_input_character(c);
            }
            frame(&mut ctx, &mut editor, &mut w);
            ctx.io_mut().add_key_event(imgui::Key::Enter, true);
            frame(&mut ctx, &mut editor, &mut w);
            ctx.io_mut().add_key_event(imgui::Key::Enter, false);
            frame(&mut ctx, &mut editor, &mut w);
            let scene = w.resolve(&root, &editor.class_registry).unwrap();
            assert_eq!(
                property_owner(&scene.actors[0], component).unwrap().1["amount"],
                serde_json::json!(31)
            );
            click(&mut ctx, &mut editor, &mut w, "class-property:reset:amount");
            assert!(
                !property_owner(
                    &w.resolve(&root, &editor.class_registry).unwrap().actors[0],
                    component
                )
                .unwrap()
                .1
                .contains_key("amount")
            );
            click(
                &mut ctx,
                &mut editor,
                &mut w,
                "class-property:value:enabled",
            );
            assert_eq!(
                property_owner(
                    &w.resolve(&root, &editor.class_registry).unwrap().actors[0],
                    component
                )
                .unwrap()
                .1["enabled"],
                serde_json::json!(false)
            );
            click(&mut ctx, &mut editor, &mut w, "class-property:value:target");
            frame(&mut ctx, &mut editor, &mut w);
            click(
                &mut ctx,
                &mut editor,
                &mut w,
                &format!("reference-choice:{target}"),
            );
            assert_eq!(
                property_owner(
                    &w.resolve(&root, &editor.class_registry).unwrap().actors[0],
                    component
                )
                .unwrap()
                .1["target"],
                serde_json::json!(target)
            );
            w.save(&root, &editor.class_registry).unwrap();
            let saved = w.template.clone();
            assert_eq!(
                Window::load(&root, &editor.class_registry, &class)
                    .unwrap()
                    .template,
                saved
            );
            click(&mut ctx, &mut editor, &mut w, "class-component-remove");
            assert!(
                !w.resolve(&root, &editor.class_registry).unwrap().actors[0]
                    .components
                    .iter()
                    .any(|c| c.id == component)
            );
            w.undo();
            assert_eq!(w.template, saved);
            w.redo();
            assert!(!w.template.references.iter().any(|r| r.owner == component));
        }
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }
    const PART: &str = "64d933bf-a037-5d79-8ab5-29032a6aaad1";
    const CPP_PART: &str = "ac38dfc3-a3ec-5e77-92af-6d9b3e148f26";
    const TARGET: &str = "b8d94b3a-2d6a-5265-ab32-e4d54c5b012d";
    const AMOUNT: &str = "a326f9a2-0982-5dc9-b6c5-d5ca2a317b43";
    const ENABLED: &str = "9f041cbe-cf48-5a84-b27e-8cb94b31f928";
    fn component_fixture() -> (PathBuf, Registry, Vec<crate::scripts::Script>, Vec<String>) {
        let (root, _, _) = fixture();
        std::fs::write(root.join("assets/scripts/WidgetParts.hpp"), r#"#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Owners=UI,Cardinality=Multiple,Id="64d933bf-a037-5d79-8ab5-29032a6aaad1") WidgetPartBase:public epok::UIComponent {
public:
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::TextComponent",Id="b8d94b3a-2d6a-5265-ab32-e4d54c5b012d") epok::ObjectId target{};
 EPOK_PROPERTY(EditAnywhere,Id="a326f9a2-0982-5dc9-b6c5-d5ca2a317b43") int32_t amount=7;
 EPOK_PROPERTY(EditAnywhere,Id="9f041cbe-cf48-5a84-b27e-8cb94b31f928") bool enabled=true;
};
class EPOK_CLASS(Blueprintable,Id="ac38dfc3-a3ec-5e77-92af-6d9b3e148f26") CppWidgetPart:public WidgetPartBase {
public:
 void on_frame(uint32_t)override{if(enabled)if(auto* text=epok::active_object_registry->resolve<epok::TextComponent>(target))text->set_number(amount);}
};
"#).unwrap();
        std::fs::write(
            root.join("assets/scripts/LuaWidgetPart.lua"),
            r#"---@class LuaWidgetPart : WidgetPartBase
local LuaWidgetPart = WidgetPartBase:extend()
---@override
function LuaWidgetPart:on_frame(frame_microseconds)
    if self.enabled then
        self.target:set_number(self.amount)
    end
end
return LuaWidgetPart
"#,
        )
        .unwrap();
        let mut bp =
            crate::blueprint_asset::BlueprintAsset::new("BP_WidgetPart".into(), PART.into());
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let frame = registry
            .ancestry("WidgetPartBase")
            .into_iter()
            .flat_map(|c| &c.functions)
            .find(|f| f.name == "on_frame")
            .unwrap();
        let setter = registry
            .ancestry("epok::TextComponent")
            .into_iter()
            .flat_map(|c| &c.functions)
            .find(|f| f.name == "set_number")
            .unwrap();
        let [entry, target, amount, enabled, branch, set, done] =
            std::array::from_fn::<_, 7, _>(|_| Uuid::new_v4().to_string());
        let mut graph = serde_json::json!({"id":Uuid::new_v4(),"name":"on_frame","override_id":frame.id,"parameters":frame.parameters,"returns":frame.returns,"entry":entry,"nodes":[
            {"id":entry,"kind":{"kind":"entry"},"outputs":{"next":[branch]}},
            {"id":target,"kind":{"kind":"get_variable","member":TARGET}},
            {"id":amount,"kind":{"kind":"get_variable","member":AMOUNT}},
            {"id":enabled,"kind":{"kind":"get_variable","member":ENABLED}},
            {"id":branch,"kind":{"kind":"branch"},"inputs":{"condition":{"kind":"link","node":enabled,"pin":"value"}},"outputs":{"true":[set],"false":[done]}},
            {"id":set,"kind":{"kind":"call_on","class":crate::object_model::TEXT_COMPONENT_ID,"function":setter.id},"inputs":{"__target":{"kind":"link","node":target,"pin":"value"}},"outputs":{"next":[done]}},
            {"id":done,"kind":{"kind":"return"}}
        ]});
        graph["nodes"][5]["inputs"][&setter.parameters[0].name] =
            serde_json::json!({"kind":"link","node":amount,"pin":"value"});
        bp.functions.push(serde_json::from_value(graph).unwrap());
        crate::blueprint_asset::create(&root.join("assets/Blueprints/BP_WidgetPart.epokbp"), &bp)
            .unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let ids = vec![
            CPP_PART.into(),
            bp.id,
            registry.named("LuaWidgetPart").unwrap().id.clone(),
        ];
        composition::create(
            &root,
            "assets/Base.epokclass",
            &composition::Document::new(BASE.into(), base_template(&root, &registry)),
            &registry,
        )
        .unwrap();
        (root, registry, catalog, ids)
    }
    pub(crate) fn component_project() -> (PathBuf, Scene, usize) {
        use crate::gui::PropertyAction::Set;
        let (root, registry, _, ids) = component_fixture();
        for (n, (name, part)) in ["CppWidget", "BP_Widget", "LuaWidget"]
            .into_iter()
            .zip(&ids)
            .enumerate()
        {
            let class = registry.named(name).unwrap().id.clone();
            let mut w = Window::load(&root, &registry, &class).unwrap();
            let scene = w.resolve(&root, &registry).unwrap();
            let actor = scene.actors[0].id;
            let target = scene.actors[0].properties["label"].clone();
            let component = w.add_component(&root, &registry, actor, part).unwrap();
            w.property(
                &root,
                &registry,
                actor,
                component,
                Set("target".into(), target),
            )
            .unwrap();
            w.property(
                &root,
                &registry,
                actor,
                component,
                Set("amount".into(), serde_json::json!(101 * (n + 1))),
            )
            .unwrap();
            w.property(
                &root,
                &registry,
                actor,
                component,
                Set("enabled".into(), serde_json::json!(true)),
            )
            .unwrap();
            let scene = w.resolve(&root, &registry).unwrap();
            let label = scene.actors.iter().find(|a| a.text.is_some()).unwrap();
            let mut edited = label.clone();
            let rect = edited.rect.as_mut().unwrap();
            rect.anchor_min = [0.; 2];
            rect.anchor_max = [0.; 2];
            rect.pivot = [0.; 2];
            rect.position = [20. + n as f32 * 180., 40.];
            crate::actor_components::sync(&mut edited);
            template_editor::apply_entity_changes(&mut w.template, label, &edited).unwrap();
            w.save(&root, &registry).unwrap();
        }
        let classes = ids
            .iter()
            .map(|id| format!("UINT64_C({})", crate::blueprint_refs::compact_id(id)))
            .collect::<Vec<_>>()
            .join(",");
        let probe = r#"#pragma once
#include "WidgetParts.hpp"
extern "C" {inline uint32_t widget_component_probe[5]={};}
class EPOK_CLASS(Blueprintable,Id="36b58708-cd72-5b83-87bd-967e3e6b2661") WidgetComponentProbe:public epok::Actor3D {
 static bool equal(const char* a,const char* b){while(*a&&*a==*b){++a;++b;}return *a==*b;}
public:
 void on_frame(uint32_t)override {
  auto* registry=epok::active_object_registry;auto* level=registry->resolve<epok::Level>(level_id());
  const uint64_t classes[]={CLASSES};const char* labels[]={"101","202","303"};
  epok::ObjectId controls[8];unsigned mask=0,count=0;bool valid=true;
  for(size_t i=0;i<level->actor_count();++i){auto* actor=registry->resolve<epok::Actor>(level->actor_at(i));if(!actor)continue;
   for(size_t j=0;j<actor->component_count();++j){auto* component=registry->resolve<epok::ActorComponent>(actor->component_id(j));if(!component)continue;
    for(unsigned n=0;n<3;++n)if(component->class_id()==classes[n]){auto* part=static_cast<WidgetPartBase*>(component);auto* text=registry->resolve<epok::TextComponent>(part->target);auto* owner=text?text->get_owner():nullptr;
     if(owner&&equal(owner->data()->text.value,labels[n])&&part->amount==int32_t(101*(n+1))&&part->enabled)mask|=1u<<n;
     valid&=owner&&owner->data()->parent==epok::entity_index(actor->data());
     for(unsigned k=0;k<count;++k)valid&=controls[k]!=part->target;if(count<8)controls[count++]=part->target;else valid=false;
    }
   }
  }
  data()->transform.rotation[0]=epok::Fixed(int(mask),0);data()->transform.rotation[1]=epok::Fixed(int(count),0);data()->transform.rotation[2]=epok::Fixed(valid?1:0,0);
  widget_component_probe[0]=mask;widget_component_probe[1]=count;widget_component_probe[2]=valid;widget_component_probe[3]=epok::hud_stats.glyphs;widget_component_probe[4]=epok::hud_stats.dropped;
 }
};
"#.replace("CLASSES", &classes);
        std::fs::write(root.join("assets/scripts/WidgetComponentProbe.hpp"), probe).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.truncate(1);
        for name in ["CppWidget", "BP_Widget", "LuaWidget"] {
            let class = &registry.named(name).unwrap().id;
            let placed =
                crate::editor::place_class(&mut scene, &root, &registry, &model, class).unwrap();
            scene
                .duplicate_actor_branch(scene.actors[placed].id)
                .unwrap();
        }
        let driver = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            "36b58708-cd72-5b83-87bd-967e3e6b2661",
        )
        .unwrap();
        scene
            .save(&crate::workspace::startup_scene(&root).unwrap())
            .unwrap();
        (root, scene, driver)
    }
    #[test]
    fn placed_nullable_widget_properties_reset_and_refresh_in_every_provider() {
        use crate::gui::PropertyAction::{Reset, Set};
        use serde_json::{Value, json};
        let (root, registry, _, parts) = component_fixture();
        let model = registry.model().unwrap();
        let mut scene = Scene::default();
        scene.actors.truncate(1);
        let mut widgets = vec![];
        for (name, part) in ["CppWidget", "BP_Widget", "LuaWidget"]
            .into_iter()
            .zip(&parts)
        {
            let class = registry.named(name).unwrap().id.clone();
            let mut window = Window::load(&root, &registry, &class).unwrap();
            let prototype = window.resolve(&root, &registry).unwrap();
            let actor = prototype.actors[0].id;
            let component = window.add_component(&root, &registry, actor, part).unwrap();
            window
                .property(
                    &root,
                    &registry,
                    actor,
                    component,
                    Set(
                        "target".into(),
                        prototype.actors[0].properties["label"].clone(),
                    ),
                )
                .unwrap();
            window
                .property(
                    &root,
                    &registry,
                    actor,
                    component,
                    Set("amount".into(), json!(101)),
                )
                .unwrap();
            window.save(&root, &registry).unwrap();
            let index =
                crate::editor::place_class(&mut scene, &root, &registry, &model, &class).unwrap();
            let placed = &scene.actors[index];
            let control = placed.properties["label"].clone();
            let component = placed
                .components
                .iter()
                .find(|c| c.class.class_id.as_ref() == Some(part))
                .unwrap()
                .id;
            widgets.push((class, placed.id, component, control));
        }
        let path = crate::workspace::startup_scene(&root).unwrap();
        scene.save(&path).unwrap();
        let mut editor =
            crate::editor::Editor::open(crate::workspace::Project::open(&root).unwrap()).unwrap();
        for (_, actor, component, _) in &widgets {
            assert!(
                templates::placement_property_overrides(
                    &editor.scene.actors[editor.scene.actor_index(*actor).unwrap()],
                    *component
                )
                .is_empty()
            );
            editor.edit_actor_property(*actor, *actor, Set("label".into(), Value::Null));
            editor.edit_actor_property(*actor, *component, Set("target".into(), Value::Null));
            assert!(editor.last_error.is_none(), "{:?}", editor.last_error);
            let index = editor.scene.actor_index(*actor).unwrap();
            let placed = &editor.scene.actors[index];
            assert_eq!(placed.properties["label"], Value::Null);
            assert_eq!(
                placed.blueprint_instance.as_ref().unwrap().overrides
                    [&templates::property_path(*actor, *component, "target")],
                templates::literal_null()
            );
            assert!(templates::placement_property_overrides(placed, *component).contains("target"));
        }
        assert!(editor.save());
        let mut loaded = Scene::load_unresolved(&path).unwrap();
        let files = crate::blueprint_asset::load_all(&root).unwrap();
        composition::refresh_instances(&root, &mut loaded, &files, &registry).unwrap();
        let mut copies = vec![];
        for (_, actor, component, _) in &widgets {
            let copy = loaded.duplicate_actor_branch(*actor).unwrap();
            let placed = &loaded.actors[loaded.actor_index(copy).unwrap()];
            let original = &loaded.actors[loaded.actor_index(*actor).unwrap()];
            let part_class = &original
                .components
                .iter()
                .find(|c| c.id == *component)
                .unwrap()
                .class;
            let part = placed
                .components
                .iter()
                .find(|c| c.class == *part_class)
                .unwrap();
            assert_eq!(part.properties["target"], Value::Null);
            assert_eq!(
                placed.blueprint_instance.as_ref().unwrap().overrides
                    [&templates::property_path(copy, part.id, "target")],
                templates::literal_null()
            );
            copies.push((copy, part.id));
        }
        editor.scene = loaded;
        editor.reset_instance_baseline();
        for (class, actor, component, control) in &widgets {
            editor.edit_actor_property(*actor, *actor, Reset("label".into()));
            editor.edit_actor_property(*actor, *component, Reset("target".into()));
            assert!(editor.last_error.is_none(), "{:?}", editor.last_error);
            let placed = &editor.scene.actors[editor.scene.actor_index(*actor).unwrap()];
            assert_eq!(&placed.properties["label"], control);
            assert_eq!(
                &placed
                    .components
                    .iter()
                    .find(|c| c.id == *component)
                    .unwrap()
                    .properties["target"],
                control
            );
            assert!(templates::placement_property_overrides(placed, *component).is_empty());
            let mut window = Window::load(&root, &registry, class).unwrap();
            let prototype = window.resolve(&root, &registry).unwrap();
            let prototype_part = prototype.actors[0]
                .components
                .iter()
                .find(|c| c.name.ends_with("WidgetPart"))
                .unwrap()
                .id;
            window
                .property(
                    &root,
                    &registry,
                    prototype.actors[0].id,
                    prototype_part,
                    Set("amount".into(), json!(202)),
                )
                .unwrap();
            window.save(&root, &registry).unwrap();
        }
        let files = crate::blueprint_asset::load_all(&root).unwrap();
        composition::refresh_instances(&root, &mut editor.scene, &files, &registry).unwrap();
        for ((_, actor, component, control), (copy, copy_part)) in widgets.iter().zip(copies) {
            let placed = &editor.scene.actors[editor.scene.actor_index(*actor).unwrap()];
            let part = placed
                .components
                .iter()
                .find(|c| c.id == *component)
                .unwrap();
            assert_eq!(part.properties["amount"], json!(202));
            assert_eq!(&part.properties["target"], control);
            let duplicated = &editor.scene.actors[editor.scene.actor_index(copy).unwrap()];
            assert_eq!(duplicated.properties["label"], Value::Null);
            let part = duplicated
                .components
                .iter()
                .find(|c| c.id == copy_part)
                .unwrap();
            assert_eq!(part.properties["amount"], json!(202));
            assert_eq!(part.properties["target"], Value::Null);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn component_properties_reset_bindings_removal_and_budgets_use_the_same_class_document_in_all_providers()
     {
        use crate::gui::PropertyAction::{Reset, Set};
        use serde_json::json;
        let (root, registry, _, ids) = component_fixture();
        let mut base = Window::load(&root, &registry, BASE).unwrap();
        let scene = base.resolve(&root, &registry).unwrap();
        let actor = scene.actors[0].id;
        let target = scene.actors[0].properties["label"]
            .as_str()
            .unwrap()
            .to_owned();
        let inherited = base.add_component(&root, &registry, actor, PART).unwrap();
        let initially_empty = base.add_component(&root, &registry, actor, PART).unwrap();
        base.property(
            &root,
            &registry,
            actor,
            inherited,
            Set("amount".into(), json!(7)),
        )
        .unwrap();
        base.property(
            &root,
            &registry,
            actor,
            inherited,
            Set("target".into(), json!(target)),
        )
        .unwrap();
        base.save(&root, &registry).unwrap();
        for (name, part) in ["CppWidget", "BP_Widget", "LuaWidget"].into_iter().zip(ids) {
            let class = registry.named(name).unwrap().id.clone();
            let mut w = Window::load(&root, &registry, &class).unwrap();
            w.property(
                &root,
                &registry,
                actor,
                initially_empty,
                Set("amount".into(), json!(31)),
            )
            .unwrap();
            w.property(
                &root,
                &registry,
                actor,
                initially_empty,
                Reset("amount".into()),
            )
            .unwrap();
            let parent_before = base.template.clone();
            base.property(
                &root,
                &registry,
                actor,
                initially_empty,
                Set("amount".into(), json!(9)),
            )
            .unwrap();
            base.save(&root, &registry).unwrap();
            let propagated = w.resolve(&root, &registry).unwrap();
            assert_eq!(
                property_owner(&propagated.actors[0], initially_empty)
                    .unwrap()
                    .1["amount"],
                json!(9),
                "Reset must permit future parent values to propagate"
            );
            base.template = parent_before;
            base.save(&root, &registry).unwrap();
            let before = w.template.clone();
            assert!(
                w.remove_component(&root, &registry, actor, inherited)
                    .unwrap_err()
                    .contains("Inherited")
            );
            assert_eq!(w.template, before);
            // Choosing the same value still declares an override; resetting it
            // restores the inherited value and its future propagation.
            w.property(
                &root,
                &registry,
                actor,
                inherited,
                Set("amount".into(), json!(7)),
            )
            .unwrap();
            let scene = w.resolve(&root, &registry).unwrap();
            assert!(
                authored_properties(&w.template, &scene.actors[0], inherited, &registry)
                    .contains("amount")
            );
            w.property(&root, &registry, actor, inherited, Reset("amount".into()))
                .unwrap();
            let scene = w.resolve(&root, &registry).unwrap();
            assert!(
                !authored_properties(&w.template, &scene.actors[0], inherited, &registry)
                    .contains("amount")
            );
            assert_eq!(
                property_owner(&scene.actors[0], inherited).unwrap().1["amount"],
                json!(7)
            );
            w.property(
                &root,
                &registry,
                actor,
                inherited,
                Set("target".into(), serde_json::Value::Null),
            )
            .unwrap();
            assert!(
                property_owner(&w.resolve(&root, &registry).unwrap().actors[0], inherited)
                    .unwrap()
                    .1["target"]
                    .is_null()
            );
            w.property(&root, &registry, actor, inherited, Reset("target".into()))
                .unwrap();
            assert_eq!(
                property_owner(&w.resolve(&root, &registry).unwrap().actors[0], inherited)
                    .unwrap()
                    .1["target"],
                json!(target)
            );
            let component = w.add_component(&root, &registry, actor, &part).unwrap();
            w.property(
                &root,
                &registry,
                actor,
                component,
                Set("amount".into(), json!(42)),
            )
            .unwrap();
            w.property(&root, &registry, actor, component, Reset("amount".into()))
                .unwrap();
            assert!(
                !property_owner(&w.resolve(&root, &registry).unwrap().actors[0], component)
                    .unwrap()
                    .1
                    .contains_key("amount")
            );
            let before = w.template.clone();
            assert!(
                w.property(
                    &root,
                    &registry,
                    actor,
                    component,
                    Set("target".into(), json!(scene.actors[0].root().unwrap().id))
                )
                .is_err()
            );
            assert_eq!(
                w.template, before,
                "Wrong-class binding rolls back every table"
            );
            w.property(
                &root,
                &registry,
                actor,
                component,
                Set("target".into(), json!(target)),
            )
            .unwrap();
            w.property(
                &root,
                &registry,
                actor,
                component,
                Set("amount".into(), json!(42)),
            )
            .unwrap();
            let before = w.template.clone();
            w.property(
                &root,
                &registry,
                actor,
                actor,
                Set("label".into(), json!(target)),
            )
            .unwrap();
            let bound = w.template.clone();
            // A control still referenced by either component cannot be removed.
            let scene = w.resolve(&root, &registry).unwrap();
            let label_actor = scene.actors.iter().find(|a| a.text.is_some()).unwrap();
            let label = label_actor
                .components
                .iter()
                .find(|c| {
                    c.class.class_id.as_deref() == Some(crate::object_model::TEXT_COMPONENT_ID)
                })
                .unwrap()
                .id;
            assert!(
                w.remove_component(&root, &registry, label_actor.id, label)
                    .is_err()
            );
            assert_eq!(w.template, bound);
            let local_text = w
                .add_component(
                    &root,
                    &registry,
                    actor,
                    crate::object_model::TEXT_COMPONENT_ID,
                )
                .unwrap();
            w.property(
                &root,
                &registry,
                actor,
                component,
                Set("target".into(), json!(local_text)),
            )
            .unwrap();
            let referenced = w.template.clone();
            assert!(
                w.remove_component(&root, &registry, actor, local_text)
                    .is_err()
            );
            assert_eq!(
                w.template, referenced,
                "Referenced local controls are retained"
            );
            w.property(
                &root,
                &registry,
                actor,
                component,
                Set("target".into(), json!(target)),
            )
            .unwrap();
            w.remove_component(&root, &registry, actor, local_text)
                .unwrap();
            assert_eq!(
                w.template, bound,
                "No stale edits remain after removing a local addition"
            );
            w.checkpoint(before);
            w.undo();
            w.redo();
            w.save(&root, &registry).unwrap();
            let mut w = Window::load(&root, &registry, &class).unwrap();
            assert_eq!(w.template, bound);
            w.remove_component(&root, &registry, actor, component)
                .unwrap();
            assert!(
                !w.resolve(&root, &registry).unwrap().actors[0]
                    .components
                    .iter()
                    .any(|c| c.id == component)
            );
            assert!(!w.template.references.iter().any(|r| r.owner == component));
            while w.resolve(&root, &registry).unwrap().actors[0]
                .components
                .len()
                < 8
            {
                w.add_component(&root, &registry, actor, PART).unwrap();
            }
            let before = w.template.clone();
            assert!(w.add_component(&root, &registry, actor, PART).is_err());
            assert_eq!(w.template, before);
            assert!(
                w.add_component(
                    &root,
                    &registry,
                    actor,
                    crate::object_model::COLLIDER3D_COMPONENT_ID
                )
                .is_err()
            );
            assert_eq!(w.template, before);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn nested_widget_canvas_drag_uses_rotated_bounds_and_one_undo_in_all_providers() {
        let (root, registry, catalog) = fixture();
        composition::create(
            &root,
            "assets/Base.epokclass",
            &composition::Document::new(BASE.into(), base_template(&root, &registry)),
            &registry,
        )
        .unwrap();
        let mut editor = Editor::new(root.clone());
        editor.class_registry = registry;
        editor.catalog = catalog;
        let mut ctx = crate::gui::tests::imgui_context();
        ctx.io_mut().display_size = [900., 700.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts().build_rgba32_texture();
        fn frame(ctx: &mut imgui::Context, editor: &Editor, w: &mut Window) -> ([f32; 2], f32) {
            w.update_preview(editor).unwrap();
            let before = w.template.clone();
            let mut viewport = None;
            let ui = ctx.frame();
            ui.window("Widget canvas acceptance")
                .position([0., 0.], imgui::Condition::Always)
                .size([880., 680.], imgui::Condition::Always)
                .build(|| {
                    viewport = canvas(ui, w);
                    w.finish_edit(before, ui.is_mouse_down(imgui::MouseButton::Left));
                });
            ctx.render();
            viewport.unwrap()
        }
        for name in ["CppWidget", "LuaWidget", "BP_Widget"] {
            let id = editor.class_registry.named(name).unwrap().id.clone();
            let mut w = Window::load(&root, &editor.class_registry, &id).unwrap();
            let parent = w.resolve(&root, &editor.class_registry).unwrap().actors[0].id;
            let site = template_editor::add_class_instance(
                &mut w.template,
                &editor.class_registry.model().unwrap(),
                parent,
                BASE,
                "Nested widget",
            )
            .unwrap();
            let scene = w.resolve(&root, &editor.class_registry).unwrap();
            let original = scene.actors.iter().find(|a| a.id == site).unwrap();
            let mut edited = original.clone();
            let rect = edited.rect.as_mut().unwrap();
            rect.anchor_min = [0.; 2];
            rect.anchor_max = [0.; 2];
            rect.pivot = [0.; 2];
            rect.position = [190., 80.];
            rect.size = [160., 120.];
            rect.rotation = 90.;
            crate::actor_components::sync(&mut edited);
            template_editor::apply_entity_changes(&mut w.template, original, &edited).unwrap();
            let scene = w.resolve(&root, &editor.class_registry).unwrap();
            let child = scene
                .actors
                .iter()
                .find(|a| a.logical_parent == Some(site))
                .unwrap();
            let child_id = child.id;
            let mut edited = child.clone();
            let rect = edited.rect.as_mut().unwrap();
            rect.anchor_min = [0.; 2];
            rect.anchor_max = [0.; 2];
            rect.pivot = [0.; 2];
            rect.position = [20., 10.];
            rect.size = [60., 20.];
            crate::actor_components::sync(&mut edited);
            template_editor::apply_entity_changes(&mut w.template, child, &edited).unwrap();
            w.save(&root, &editor.class_registry).unwrap();
            let before = w.template.clone();
            frame(&mut ctx, &editor, &mut w);
            let (origin, scale) = frame(&mut ctx, &editor, &mut w);
            let preview = w.preview.as_ref().unwrap();
            let index = preview.scene.actor_index(child_id).unwrap();
            let place = crate::hud::layouts(&preview.scene)[index].unwrap();
            let center = [0usize, 1].map(|i| place.corners.iter().map(|p| p[i]).sum::<f32>() / 4.);
            assert!(contains(&place, center));
            // A point in the visible quarter-turned label lies outside its
            // old axis-aligned bounds; selection must follow what is drawn.
            assert!(
                !(center[0] >= place.rect[0]
                    && center[0] <= place.rect[0] + place.rect[2]
                    && center[1] >= place.rect[1]
                    && center[1] <= place.rect[1] + place.rect[3])
            );
            let mouse = [
                origin[0] + center[0] * scale,
                origin[1] + (f32::from(preview.scene.display_size[1]) - center[1]) * scale,
            ];
            ctx.io_mut().add_mouse_pos_event(mouse);
            frame(&mut ctx, &editor, &mut w);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(&mut ctx, &editor, &mut w);
            assert_eq!(w.controls.selected(), Some(child_id));
            for n in 1..=2 {
                ctx.io_mut().add_mouse_pos_event([
                    mouse[0] + 10. * n as f32 * scale,
                    mouse[1] - 5. * n as f32 * scale,
                ]);
                frame(&mut ctx, &editor, &mut w);
            }
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(&mut ctx, &editor, &mut w);
            assert_eq!(w.undo.len(), 1);
            let scene = w.resolve(&root, &editor.class_registry).unwrap();
            let position = scene.actors[scene.actor_index(child_id).unwrap()]
                .rect
                .as_ref()
                .unwrap()
                .position;
            // ImGui quantizes the incoming cursor to physical screen pixels.
            let tolerance = 1. / scale;
            assert!(
                (position[0] - 30.).abs() < tolerance && (position[1] + 10.).abs() < tolerance,
                "{name}: {position:?}"
            );
            assert_eq!(
                w.template.actors.len(),
                1,
                "Nested children remain inherited"
            );
            let moved = w.template.clone();
            w.undo();
            assert_eq!(w.template, before);
            w.redo();
            assert_eq!(w.template, moved);
            w.save(&root, &editor.class_registry).unwrap();
            assert_eq!(
                Window::load(&root, &editor.class_registry, &id)
                    .unwrap()
                    .template,
                moved
            );
        }
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn complete_game_widgets_edit_undo_and_reopen_in_the_common_designer() {
        use crate::workspace::{CreateOptions, GameplayFlavor, Template};
        for flavor in GameplayFlavor::ALL {
            let root = crate::workspace::tests::temp("complete-widget-authoring");
            crate::workspace::create_with_options(
                &root,
                "Widget Authoring",
                CreateOptions::new(Template::DungeonComplete).with_gameplay(flavor),
            )
            .unwrap();
            let catalog = crate::scripts::catalog(&root).unwrap();
            let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
            let mut saved = Vec::new();
            for name in [
                "DungeonGameHUDWidget",
                "DungeonTerminalWidget",
                "DungeonShopWidget",
                "DungeonInventoryWidget",
            ] {
                let class = registry.named(name).unwrap();
                let id = class.id.clone();
                let source = PathBuf::from(&class.source.file);
                let source_before = std::fs::read(&source).unwrap();
                let behavior_before = (flavor == GameplayFlavor::Blueprint).then(|| {
                    let mut value =
                        serde_json::to_value(crate::blueprint_asset::load(&source).unwrap())
                            .unwrap();
                    value.as_object_mut().unwrap().remove("template");
                    value
                });
                let mut window = Window::load(&root, &registry, &id).unwrap();
                let scene = window.resolve(&root, &registry).unwrap();
                let actor_count = scene.actors.len();
                let original = scene
                    .actors
                    .iter()
                    .find(|actor| actor.text.is_some() && actor.parent.is_some())
                    .unwrap();
                let original_position = original.rect.as_ref().unwrap().position;
                let original_parent = original.logical_parent;
                let mut edited = original.clone();
                let position = [original_position[0] + 13., original_position[1] + 7.];
                edited.rect.as_mut().unwrap().position = position;
                crate::actor_components::sync(&mut edited);
                let before = window.template.clone();
                template_editor::apply_entity_changes(&mut window.template, original, &edited)
                    .unwrap();
                assert!(
                    window
                        .template
                        .actors
                        .iter()
                        .all(|item| item.entity.parent.is_none())
                );
                window.checkpoint(before);
                assert!(window.dirty(), "{flavor:?} {name}");
                let preview = window.resolve(&root, &registry).unwrap();
                assert_eq!(
                    preview
                        .actors
                        .iter()
                        .find(|actor| actor.id == original.id)
                        .unwrap()
                        .rect
                        .as_ref()
                        .unwrap()
                        .position,
                    position
                );
                assert_eq!(
                    std::fs::read(&source).unwrap(),
                    source_before,
                    "Preview edits stay unsaved until Save"
                );
                window.undo();
                assert!(!window.dirty());
                let undone = window.resolve(&root, &registry).unwrap();
                assert_eq!(
                    undone
                        .actors
                        .iter()
                        .find(|actor| actor.id == original.id)
                        .unwrap()
                        .rect
                        .as_ref()
                        .unwrap()
                        .position,
                    original_position
                );
                window.redo();
                assert!(window.dirty());
                window.save(&root, &registry).unwrap();
                assert!(!window.dirty());
                saved.push((
                    id,
                    original.id,
                    original_parent,
                    position,
                    actor_count,
                    source,
                    source_before,
                    behavior_before,
                ));
                eprintln!("Edited {flavor:?} {name}");
            }
            // Recompile each provider once after all four layout saves. Every
            // class is still reopened against freshly compiled metadata; the
            // unchanged behavior graphs need no per-control recompilation.
            let catalog = crate::scripts::catalog(&root).unwrap();
            let reloaded_registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
            for (
                id,
                child_id,
                original_parent,
                position,
                actor_count,
                source,
                source_before,
                behavior_before,
            ) in saved
            {
                let reopened = Window::load(&root, &reloaded_registry, &id).unwrap();
                let resolved = reopened.resolve(&root, &reloaded_registry).unwrap();
                assert_eq!(resolved.actors.len(), actor_count);
                let reopened_child = resolved
                    .actors
                    .iter()
                    .find(|actor| actor.id == child_id)
                    .unwrap();
                assert_eq!(reopened_child.logical_parent, original_parent);
                assert_eq!(reopened_child.rect.as_ref().unwrap().position, position);
                if let Some(behavior_before) = behavior_before {
                    let mut after =
                        serde_json::to_value(crate::blueprint_asset::load(&source).unwrap())
                            .unwrap();
                    after.as_object_mut().unwrap().remove("template");
                    assert_eq!(
                        after, behavior_before,
                        "Layout saves must retain Blueprint functions, defaults, variables and graph layout"
                    );
                } else {
                    assert_eq!(
                        std::fs::read(&source).unwrap(),
                        source_before,
                        "Layout saves must retain editable C++/Lua rules"
                    );
                }
            }
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn ui_designer_saves_the_same_cpp_blueprint_and_lua_class_and_preserves_inherited_controls() {
        let (root, registry, _) = fixture();
        composition::create(
            &root,
            "assets/Base.epokclass",
            &composition::Document::new(BASE.into(), base_template(&root, &registry)),
            &registry,
        )
        .unwrap();
        for name in ["CppWidget", "LuaWidget", "BP_Widget"] {
            let id = registry.named(name).unwrap().id.clone();
            let mut window = Window::load(&root, &registry, &id).unwrap();
            let scene = window.resolve(&root, &registry).unwrap();
            let original = scene.actors.iter().find(|a| a.text.is_some()).unwrap();
            let mut edited = original.clone();
            edited.rect.as_mut().unwrap().position = [38., 71.];
            crate::actor_components::sync(&mut edited);
            let before = window.template.clone();
            template_editor::apply_entity_changes(&mut window.template, original, &edited).unwrap();
            window.checkpoint(before);
            assert!(window.dirty());
            assert!(
                window.template.actors.is_empty(),
                "Inherited controls stay inherited"
            );
            window.undo();
            assert!(window.template.overrides.is_empty());
            window.redo();
            window.save(&root, &registry).unwrap();
            assert!(!window.dirty());
            // The normal compiler must see the same inherited native/Lua
            // composition when it reloads a Blueprint with control overrides.
            crate::scripts::catalog(&root).unwrap();
            let reopened = Window::load(&root, &registry, &id).unwrap();
            let scene = reopened.resolve(&root, &registry).unwrap();
            assert_eq!(
                scene.actors.iter().find_map(|a| a.text.as_ref().map(|_| a
                    .rect
                    .as_ref()
                    .unwrap()
                    .position)),
                Some([38., 71.])
            );
            if name == "BP_Widget" {
                assert!(!composition::load(&root).unwrap().sources.contains_key(&id));
                let path = &registry.classes[&id].source.file;
                let mut outside = crate::blueprint_asset::load(path).unwrap();
                outside
                    .layout
                    .positions
                    .insert("behavior-edit".into(), [15., 40.]);
                std::fs::write(path, crate::document::to_vec(&outside).unwrap()).unwrap();
                // Merge graph-side changes without replacing behavior.
                window.save(&root, &registry).unwrap();
                assert_eq!(
                    crate::blueprint_asset::load(path).unwrap().layout.positions["behavior-edit"],
                    [15., 40.]
                );
                outside.template.overrides.clear();
                std::fs::write(path, crate::document::to_vec(&outside).unwrap()).unwrap();
                let bytes = std::fs::read(path).unwrap();
                assert!(window.save(&root, &registry).is_err());
                assert_eq!(std::fs::read(path).unwrap(), bytes);
            }
        }
        assert_eq!(composition::load(&root).unwrap().sources.len(), 3);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn ui_designer_clears_an_inherited_control_binding_explicitly() {
        let (root, registry, _) = fixture();
        composition::create(
            &root,
            "assets/Base.epokclass",
            &composition::Document::new(BASE.into(), base_template(&root, &registry)),
            &registry,
        )
        .unwrap();
        let id = registry.named("LuaWidget").unwrap().id.clone();
        let mut window = Window::load(&root, &registry, &id).unwrap();
        let scene = window.resolve(&root, &registry).unwrap();
        let owner = scene
            .actors
            .iter()
            .find(|a| a.logical_parent.is_none())
            .unwrap();
        window.template.references.push(templates::ObjectReference {
            owner: owner.id,
            member: registry
                .properties("LuaWidget")
                .iter()
                .find(|p| p.name == "label")
                .unwrap()
                .id
                .clone(),
            target: Uuid::nil(),
        });
        window.save(&root, &registry).unwrap();
        let reopened = Window::load(&root, &registry, &id).unwrap();
        let scene = reopened.resolve(&root, &registry).unwrap();
        assert!(
            scene
                .actors
                .iter()
                .find(|a| a.logical_parent.is_none())
                .unwrap()
                .properties["label"]
                .is_null()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn generic_widget_preview_uses_authored_controls_and_runs_no_project_code() {
        let (root, registry, catalog) = fixture();
        let mut template = base_template(&root, &registry);
        let root_id = Uuid::from_u128(1);
        let label_id = Uuid::from_u128(20);
        let identities = std::collections::BTreeMap::from([
            (template.actors[0].entity.id, root_id),
            (template.actors[1].entity.id, label_id),
        ]);
        for item in &mut template.actors {
            crate::actor_document::remap_actor(&mut item.entity, &identities);
            item.parent = item
                .parent
                .map(|id| identities.get(&id).copied().unwrap_or(id));
        }
        for reference in &mut template.references {
            reference.owner = identities
                .get(&reference.owner)
                .copied()
                .unwrap_or(reference.owner);
        }
        let mut background = composition::actor(
            &registry.model().unwrap(),
            crate::object_model::UI_ACTOR_ID,
            "Background",
        )
        .unwrap();
        background.id = Uuid::from_u128(30);
        background.data.id = background.id;
        background.rect = template.actors[1].entity.rect.clone();
        background.image = Some(crate::hud::Image {
            color: [0.; 3],
            ..Default::default()
        });
        background.attach = Some(crate::actor_document::Attachment {
            actor: root_id,
            component: None,
        });
        crate::actor_components::sync(&mut background);
        // The background's UUID sorts after the label, but its authored paint
        // order is before it. Sorting by identity would cover every glyph.
        template.actors.insert(
            1,
            templates::TemplateEntity {
                entity: background,
                parent: Some(root_id),
            },
        );
        composition::create(
            &root,
            "assets/Base.epokclass",
            &composition::Document::new(BASE.into(), template),
            &registry,
        )
        .unwrap();
        let mut editor = Editor::new(root.clone());
        editor.class_registry = registry;
        editor.catalog = catalog;
        let mut window = Window::load(&root, &editor.class_registry, BASE).unwrap();
        window.update_preview(&editor).unwrap();
        let preview = window.preview.as_ref().unwrap();
        assert_eq!(preview.stats[4], 0);
        assert_eq!(preview.stats[1], 14);
        assert!(
            preview
                .pixels
                .chunks_exact(4)
                .any(|p| p[0] > 200 && p[1] > 200 && p[2] > 200),
            "The authored foreground must remain visible over its background."
        );
        assert_eq!(
            preview.pixels.len(),
            usize::from(preview.scene.display_size[0])
                * usize::from(preview.scene.display_size[1])
                * 4
        );
        assert!(!root.join(".epok/native-preview").exists());
        let before = preview.pixels.clone();
        window
            .template
            .actors
            .iter_mut()
            .find(|a| a.entity.text.is_some())
            .unwrap()
            .entity
            .text
            .as_mut()
            .unwrap()
            .text = "Changed".into();
        window.update_preview(&editor).unwrap();
        assert!(
            window.preview.as_ref().unwrap().pixels != before,
            "Changing an on-screen control must change the rendered preview."
        );
        drop(editor);
        std::fs::remove_dir_all(root).unwrap();
    }
}
