use crate::{
    assets,
    editor::Editor,
    mesh::{self, Document},
    scene::{Actor, Scene},
    transform::Matrix,
};
use imgui::{Condition, Ui};
use std::{collections::BTreeSet, sync::Arc};
use uuid::Uuid;
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Faces,
    Edges,
    Vertices,
}

/// A mesh element under the cursor or a click, in the current selection mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Face(Uuid),
    Edge(crate::mesh_ops::Edge),
    Vertex(u32),
}

/// What a left drag in the Scene view does while editing geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DragSelect {
    /// A rectangle selects what it encloses when the button is released.
    #[default]
    Box,
    /// Every element the cursor passes over is selected as it goes.
    Paint,
}

/// How a selection gesture combines with the current selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectOp {
    Replace,
    Toggle,
    Add,
    Remove,
}
impl SelectOp {
    /// A click: Ctrl toggles the element, Shift adds it, otherwise it alone is
    /// selected.
    pub fn click(shift: bool, ctrl: bool) -> Self {
        if ctrl {
            Self::Toggle
        } else if shift {
            Self::Add
        } else {
            Self::Replace
        }
    }
    /// A drag: Shift adds, Ctrl removes, otherwise it replaces the selection.
    pub fn drag(shift: bool, ctrl: bool) -> Self {
        if ctrl {
            Self::Remove
        } else if shift {
            Self::Add
        } else {
            Self::Replace
        }
    }
}

/// The left-button gesture in the Scene view while editing geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Gesture {
    #[default]
    Idle,
    /// Pressed on the Scene view: still a click until the cursor moves.
    Pressed { mouse: [f32; 2], pixel: [f32; 2] },
    /// A box from where the drag started (screen and Scene pixels).
    Box {
        mouse: [f32; 2],
        pixel: [f32; 2],
        op: SelectOp,
    },
    /// Painting; `pixel` is where the cursor was on the previous frame.
    Paint { pixel: [f32; 2], op: SelectOp },
    /// Escape dropped the drag; the rest of the press does nothing.
    Cancelled,
}

/// How far the cursor moves before a press becomes a drag; the Scene view's
/// click gesture uses the same distance.
const DRAG_SLOP: f32 = 3.;

fn button(ui: &Ui, label: &str) -> bool {
    let pressed = ui.button(label);
    #[cfg(test)]
    BUTTONS.with(|buttons| {
        let a = ui.item_rect_min();
        let b = ui.item_rect_max();
        buttons
            .borrow_mut()
            .insert(label.to_owned(), [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]);
    });
    pressed
}
#[cfg(test)]
thread_local! {static BUTTONS: std::cell::RefCell<std::collections::BTreeMap<String,[f32;2]>> = const {std::cell::RefCell::new(std::collections::BTreeMap::new())};}

/// Mesh revision, limit bits, hidden and locked group counts, isolated group.
type SubdivideKey = (u64, u32, usize, usize, Option<Uuid>);

#[derive(Clone)]
struct History {
    doc: Document,
    scene: Option<Scene>,
    guard: Option<String>,
}
pub struct State {
    pub open: bool,
    pub record: Option<assets::Record>,
    pub doc: Option<Document>,
    pub target: Option<usize>,
    pub selected: BTreeSet<Uuid>,
    pub vertices: BTreeSet<u32>,
    pub mode: Mode,
    pub edges: BTreeSet<crate::mesh_ops::Edge>,
    pub hidden: BTreeSet<Uuid>,
    pub locked: BTreeSet<Uuid>,
    pub isolate: Option<Uuid>,
    pub revision: u64,
    /// The element under the cursor in the Scene view, for hover feedback.
    pub hover: Option<Hit>,
    /// The mesh as it was when this editing session began, for Revert.
    pub session_start: Option<Document>,
    /// Darken everything but the edited mesh while editing it.
    pub dim_others: bool,
    /// The Geometry panel's category and Model tool, kept between sessions.
    pub category: Category,
    pub model_tool: ModelTool,
    /// Dock the panel where the Hierarchy is, until it reports it is there.
    dock_pending: bool,
    /// The panel sits in the Hierarchy's dock node, so the Hierarchy is hidden.
    pub docked: bool,
    /// What a left drag in the Scene view does: a box or paint selection.
    pub drag_select: DragSelect,
    /// A box also selects elements the mesh hides from the camera.
    pub select_occluded: bool,
    gesture: Gesture,
    /// The `revision` the Scene view last redrew for.
    drawn_revision: u64,
    group: Option<Uuid>,
    slot: Option<Uuid>,
    name: String,
    path: String,
    shape: usize,
    origin: [f32; 3],
    size: [f32; 3],
    steps: i32,
    grid: f32,
    offset: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
    distance: f32,
    /// Texture correction: longest edge, in local units, the split leaves.
    max_edge: f32,
    /// What that split would do, cached until the mesh, selection or limit changes.
    subdivide_preview: Option<(SubdivideKey, crate::mesh_subdivide::Plan)>,
    brush: crate::brush::Brush,
    undo: Vec<History>,
    redo: Vec<History>,
    pub error: Option<String>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            record: None,
            doc: None,
            target: None,
            selected: BTreeSet::new(),
            vertices: BTreeSet::new(),
            mode: Mode::Faces,
            edges: BTreeSet::new(),
            hidden: BTreeSet::new(),
            locked: BTreeSet::new(),
            isolate: None,
            revision: 0,
            hover: None,
            session_start: None,
            dim_others: true,
            category: Category::default(),
            model_tool: ModelTool::default(),
            dock_pending: false,
            docked: false,
            drag_select: DragSelect::default(),
            select_occluded: false,
            gesture: Gesture::Idle,
            drawn_revision: 0,
            group: None,
            slot: None,
            name: "New group".into(),
            path: "assets/Meshes/Blockout.epokasset".into(),
            shape: 0,
            origin: [0.; 3],
            size: [1.; 3],
            steps: 4,
            grid: 0.25,
            offset: [0.; 3],
            rotation: [0.; 3],
            scale: [1.; 3],
            distance: 0.25,
            max_edge: 2.,
            subdivide_preview: None,
            brush: crate::brush::Brush::default(),
            undo: vec![],
            redo: vec![],
            error: None,
        }
    }
}
impl State {
    pub fn visible(&self, doc: &Document, group: Uuid) -> bool {
        !self.hidden.iter().any(|g| doc.contains_group(group, *g))
            && self.isolate.is_none_or(|g| doc.contains_group(group, g))
    }
    fn unlocked(&self, doc: &Document, group: Uuid) -> bool {
        !self.locked.iter().any(|g| doc.contains_group(group, *g))
    }
}
fn scene_hash(scene: &Scene) -> String {
    assets::hash(&serde_json::to_vec(scene).unwrap())
}
pub fn open(e: &mut Editor, record: assets::Record, target: Option<usize>) {
    e.mesh_editor.open = false;
    preview(e);
    let target = target.or_else(|| {
        e.scene.actors.iter().position(|v| {
            v.editable_mesh
                .as_ref()
                .is_some_and(|m| m.asset == record.meta.id)
        })
    });
    match mesh::document(&record) {
        Ok(doc) => {
            // The renderer caches geometry by revision: never reuse a number.
            let revision = e.mesh_editor.revision.wrapping_add(1);
            let dim_others = e.mesh_editor.dim_others;
            let (category, model_tool) = (e.mesh_editor.category, e.mesh_editor.model_tool);
            let (drag_select, select_occluded) =
                (e.mesh_editor.drag_select, e.mesh_editor.select_occluded);
            e.mesh_editor = State {
                open: true,
                group: doc.groups.first().map(|g| g.id),
                slot: doc.materials.first().map(|s| s.id),
                path: assets::path_string(&e.root, &record.path),
                record: Some(record),
                session_start: Some(doc.clone()),
                doc: Some(doc),
                target,
                revision,
                dim_others,
                category,
                model_tool,
                drag_select,
                select_occluded,
                dock_pending: true,
                ..Default::default()
            };
            preview(e);
        }
        Err(error) => e.log(error),
    }
}
/// End the geometry editing session (Done, Esc, closing the panel, Play,
/// selecting another actor). Edits are already saved; the edited actor stays
/// selected, as it was before entering.
pub fn close(e: &mut Editor) {
    if !e.mesh_editor.open {
        return;
    }
    e.mesh_editor.open = false;
    e.mesh_editor.docked = false;
    e.mesh_editor.hover = None;
    e.mesh_editor.gesture = Gesture::Idle;
    e.mesh_editor.session_start = None;
    let target = target_actor(e);
    preview(e);
    if let Some(id) = target {
        e.select_actor(Some(id));
    }
}
/// The actor whose geometry is being edited, when the session has one.
pub fn target_actor(e: &Editor) -> Option<Uuid> {
    let record = e.mesh_editor.record.as_ref()?;
    let actor = e.scene.actors.get(e.mesh_editor.target?)?;
    actor
        .editable_mesh
        .as_ref()
        .is_some_and(|m| m.asset == record.meta.id)
        .then_some(actor.id)
}
/// Whether the mesh differs from how it was when this session began.
pub fn changed_in_session(e: &Editor) -> bool {
    e.mesh_editor
        .session_start
        .as_ref()
        .is_some_and(|start| e.mesh_editor.doc.as_ref() != Some(start))
}
/// Restore the mesh to how it was when this session began, as one Undo step.
pub fn revert(e: &mut Editor) -> Result<(), String> {
    let start = e
        .mesh_editor
        .session_start
        .clone()
        .ok_or("No geometry editing session")?;
    e.mesh_editor.selected.clear();
    e.mesh_editor.edges.clear();
    e.mesh_editor.vertices.clear();
    edit(e, |doc| {
        *doc = start;
        Ok(())
    })
}
pub fn preview(e: &mut Editor) {
    if let (Some(record), Some(doc)) = (&e.mesh_editor.record, &e.mesh_editor.doc) {
        if e.assets.index.resolve(record.meta.id).is_err() {
            e.mesh_editor.revision = e.mesh_editor.revision.wrapping_add(1);
            e.view_dirty = true;
            return;
        }
        let mut shown = doc.clone();
        if e.mesh_editor.open {
            shown.faces.retain(|f| e.mesh_editor.visible(doc, f.group));
        }
        let shown = Arc::new(shown);
        for entity in &mut e.scene.actors {
            if let Some(m) = &mut entity.editable_mesh
                && m.asset == record.meta.id
            {
                m.document = Some(shown.clone());
                m.error = None;
            }
        }
    }
    e.mesh_editor.revision = e.mesh_editor.revision.wrapping_add(1);
    e.view_dirty = true;
}
pub fn synchronize(e: &mut Editor) {
    let Some(old) = e.mesh_editor.record.clone() else {
        return;
    };
    match e.assets.index.resolve(old.meta.id).cloned() {
        Ok(record) => {
            if record.revision != old.revision {
                // A background scan can complete after our own save. Never reload its stale result.
                if std::fs::read(&record.path)
                    .is_ok_and(|bytes| assets::hash(&bytes) == record.revision)
                {
                    match mesh::document(&record) {
                        Ok(doc) => {
                            e.mesh_editor.doc = Some(doc);
                            e.mesh_editor.undo.clear();
                            e.mesh_editor.redo.clear();
                            e.mesh_editor.vertices.clear();
                            e.mesh_editor.edges.clear();
                            e.mesh_editor.selected.clear();
                            e.mesh_editor.record = Some(record);
                            e.mesh_editor.error = Some("External geometry changes loaded. Geometry undo history was reset.".into());
                        }
                        Err(error) => e.mesh_editor.error = Some(error),
                    }
                }
            } else {
                e.mesh_editor.record.as_mut().unwrap().path = record.path;
            }
        }
        Err(error) => e.mesh_editor.error = Some(error),
    }
    preview(e);
}
fn publish(e: &mut Editor, doc: Document, scene: Option<Scene>) -> Result<(), String> {
    let record = e.mesh_editor.record.as_ref().ok_or("No mesh open")?.clone();
    let revision = mesh::save(&record, &doc)?;
    let mut record = record;
    record.revision = revision;
    record.meta.source_hash = assets::hash(&serde_json::to_vec(&doc).unwrap());
    e.mesh_editor
        .selected
        .retain(|id| doc.faces.iter().any(|f| f.id == *id));
    e.mesh_editor
        .vertices
        .retain(|v| (*v as usize) < doc.vertices.len());
    // Topology edits may compact/reindex vertices. Face UUID selections remain stable.
    if e.mesh_editor
        .doc
        .as_ref()
        .is_some_and(|old| old.faces != doc.faces)
    {
        e.mesh_editor.vertices.clear();
        e.mesh_editor.edges.clear();
    }
    if !doc.groups.iter().any(|g| Some(g.id) == e.mesh_editor.group) {
        e.mesh_editor.group = doc.groups.first().map(|g| g.id);
    }
    if !doc
        .materials
        .iter()
        .any(|s| Some(s.id) == e.mesh_editor.slot)
    {
        e.mesh_editor.slot = doc.materials.first().map(|s| s.id);
    }
    e.mesh_editor.record = Some(record);
    e.mesh_editor.doc = Some(doc);
    if let Some(scene) = scene {
        e.scene = scene;
    }
    e.assets.refresh();
    preview(e);
    e.changed();
    Ok(())
}
pub fn edit(
    e: &mut Editor,
    change: impl FnOnce(&mut Document) -> Result<(), String>,
) -> Result<(), String> {
    if e.playing {
        return Err("Stop Play before editing geometry".into());
    }
    let before = e.mesh_editor.doc.clone().ok_or("No mesh open")?;
    let mut after = before.clone();
    change(&mut after)?;
    after.validate()?;
    if before == after {
        return Ok(());
    }
    publish(e, after, None)?;
    e.mesh_editor.undo.push(History {
        doc: before,
        scene: None,
        guard: None,
    });
    if e.mesh_editor.undo.len() > 32 {
        e.mesh_editor.undo.remove(0);
    }
    e.mesh_editor.redo.clear();
    Ok(())
}
pub fn undo(e: &mut Editor, redo: bool) -> Result<(), String> {
    if e.playing {
        return Err("Stop Play before editing geometry".into());
    }
    let entry = if redo {
        e.mesh_editor.redo.last()
    } else {
        e.mesh_editor.undo.last()
    }
    .cloned()
    .ok_or("No geometry operation to undo/redo")?;
    if entry
        .guard
        .as_ref()
        .is_some_and(|hash| *hash != scene_hash(&e.scene))
    {
        return Err("The scene changed after extraction. Undo later scene changes before restoring that extraction.".into());
    }
    let current = History {
        doc: e.mesh_editor.doc.clone().unwrap(),
        scene: entry.scene.as_ref().map(|_| e.scene.clone()),
        guard: entry.scene.as_ref().map(scene_hash),
    };
    publish(e, entry.doc, entry.scene)?;
    if redo {
        e.mesh_editor.redo.pop();
        e.mesh_editor.undo.push(current);
    } else {
        e.mesh_editor.undo.pop();
        e.mesh_editor.redo.push(current);
    }
    Ok(())
}
fn fresh_path(_e: &Editor) -> String {
    format!("assets/Meshes/Blockout-{}.epokasset", Uuid::new_v4())
}
pub fn create_component(e: &mut Editor, entity: &mut Actor) -> Result<(), String> {
    let mut doc = Document::default();
    doc.primitive(
        "Box",
        [0.; 3],
        [1.; 3],
        1,
        doc.groups[0].id,
        doc.materials[0].id,
    );
    let id = mesh::create(&e.root, &fresh_path(e), &doc)?;
    let mut c = mesh::Component::new(id);
    c.document = Some(Arc::new(doc));
    entity.editable_mesh = Some(c);
    entity.material = Default::default();
    entity.kind = "Mesh".into();
    entity.lighting.static_geometry = true;
    entity.lighting.receive = crate::lighting::Receive::Baked;
    e.assets.refresh();
    Ok(())
}
pub fn allocate_actor_data(e: &mut Editor) {
    allocate_actor_data_under(e, None);
}
/// Creates a blockout actor under `parent` (`None` is the map root) and opens
/// its mesh for editing, returning the new actor.
pub fn allocate_actor_data_under(e: &mut Editor, parent: Option<uuid::Uuid>) -> Option<uuid::Uuid> {
    if e.playing {
        return None;
    }
    if e.scene.actors.len() >= 512 {
        e.log("Scene entity limit: 512");
        return None;
    }
    let mut entity = Actor::cube("Blockout".into());
    entity.position = [0.; 3];
    let created = entity.id;
    match create_component(e, &mut entity) {
        Ok(()) => {
            e.scene.actors.push(entity);
            e.parent_new_actor(created, parent);
            e.selected = Some(e.scene.actors.len() - 1);
            e.changed();
            e.assets.index = assets::scan(&e.root, &mut Default::default());
            let id = e
                .scene
                .actors
                .last()
                .unwrap()
                .editable_mesh
                .as_ref()
                .unwrap()
                .asset;
            if let Ok(r) = e.assets.index.resolve(id) {
                open(e, r.clone(), e.selected);
            }
            Some(created)
        }
        Err(error) => {
            e.log(error);
            None
        }
    }
}
pub fn extract(e: &mut Editor) -> Result<(), String> {
    if e.playing {
        return Err("Stop Play before editing geometry".into());
    }
    let index = e
        .mesh_editor
        .target
        .ok_or("Open the component on a scene entity to extract geometry")?;
    if e.scene
        .actors
        .get(index)
        .and_then(|v| v.editable_mesh.as_ref())
        .is_none_or(|m| Some(m.asset) != e.mesh_editor.record.as_ref().map(|r| r.meta.id))
    {
        return Err("The edited entity changed. Reopen its EditableMesh component.".into());
    }
    let asset = e.mesh_editor.record.as_ref().unwrap().meta.id;
    let current_scene = format!("{} / ", assets::path_string(&e.root, &e.scene_path()));
    if assets::dependencies(&e.root, asset)?
        .iter()
        .any(|usage| !usage.starts_with(&current_scene))
    {
        return Err(
            "Make an independent copy before extracting an asset used by other saved scenes".into(),
        );
    }
    if e.scene
        .actors
        .iter()
        .filter(|v| v.editable_mesh.as_ref().is_some_and(|m| m.asset == asset))
        .count()
        > 1
    {
        return Err("Make an independent copy before extracting from a shared asset".into());
    }
    let selected = &e.mesh_editor.selected;
    if selected.is_empty() {
        return Err("Select faces to extract".into());
    }
    let before = e.mesh_editor.doc.clone().unwrap();
    let extracted = before.subset(selected);
    let mut after = before.clone();
    after.faces.retain(|f| !selected.contains(&f.id));
    after.compact();
    let path = fresh_path(e);
    let mut scene = e.scene.clone();
    let original_scene = scene.clone();
    let id = mesh::create(&e.root, &path, &extracted)?;
    let mut child = Actor::cube("Extracted geometry".into());
    child.position = [0.; 3];
    child.parent = Some(index);
    child.lighting = scene.actors[index].lighting.clone();
    child.material = scene.actors[index].material.clone();
    child.editable_mesh = Some(mesh::Component {
        asset: id,
        materials: scene.actors[index]
            .editable_mesh
            .as_ref()
            .unwrap()
            .materials
            .clone(),
        document: Some(Arc::new(extracted)),
        error: None,
    });
    scene.actors.push(child);
    let result = scene
        .validate()
        .and_then(|_| publish(e, after, Some(scene)));
    if result.is_err() {
        let _ = std::fs::remove_file(assets::inside(&e.root, &path)?);
    }
    result?;
    e.mesh_editor.undo.push(History {
        doc: before,
        scene: Some(original_scene),
        guard: Some(scene_hash(&e.scene)),
    });
    if e.mesh_editor.undo.len() > 32 {
        e.mesh_editor.undo.remove(0);
    }
    e.mesh_editor.redo.clear();
    e.mesh_editor.selected.clear();
    Ok(())
}
/// Switch this instance's mesh reference without modifying a shared mesh asset.
/// No asset reference denotes the immutable cube supplied by the engine.
pub(crate) fn assign_mesh(
    entity: &mut Actor,
    index: &assets::Index,
    asset: Option<Uuid>,
) -> Result<(), String> {
    let (editable, skeletal) = match asset {
        None => (None, None),
        Some(id) => {
            let record = index.resolve(id)?;
            match record.meta.kind {
                assets::Kind::EditableMesh => {
                    let mut component = mesh::Component::new(id);
                    component.document = Some(Arc::new(mesh::document(record)?));
                    (Some(component), None)
                }
                assets::Kind::SkeletalMesh => {
                    let mut component = crate::skeletal::Component::new(id);
                    component.model = Some(Arc::new(crate::skeletal::Model::load(index, id)?));
                    (None, Some(component))
                }
                _ => return Err("Select a mesh asset from the Project browser.".into()),
            }
        }
    };
    entity.kind = "Mesh".into();
    entity.editable_mesh = editable;
    entity.skeletal_mesh = skeletal;
    if entity.skeletal_mesh.is_some() {
        entity.lighting.static_geometry = false;
        entity.lighting.receive = crate::lighting::Receive::Realtime;
    }
    Ok(())
}

/// The actor's mesh: the read-only engine cube or a project mesh asset. Epok has
/// one mesh component, so this is the first row of whichever renderer section is
/// showing rather than a section of its own.
pub fn selector(ui: &Ui, e: &mut Editor, entity: &mut Actor) {
    let current = entity
        .editable_mesh
        .as_ref()
        .map(|mesh| mesh.asset)
        .or(entity.skeletal_mesh.as_ref().map(|mesh| mesh.asset));
    let label = current
        .map(|id| {
            e.assets
                .index
                .resolve(id)
                .map(|record| assets::path_string(&e.root, &record.path))
                .unwrap_or_else(|_| format!("Missing / conflicting {id}"))
        })
        .unwrap_or_else(|| "Engine / Cube".into());
    let mut chosen = None;
    let combo = ui.begin_combo(crate::gui::field(ui, "Mesh"), &label);
    #[cfg(test)]
    crate::gui::record_script_control(ui, "Mesh");
    if let Some(_combo) = combo {
        ui.text_disabled("Engine");
        if ui
            .selectable_config("Cube (read-only)")
            .selected(current.is_none())
            .build()
        {
            chosen = Some(None);
        }
        #[cfg(test)]
        crate::gui::record_script_control(ui, "Engine / Cube");
        ui.separator();
        ui.text_disabled("Project");
        let mut records: Vec<_> = e
            .assets
            .index
            .usable()
            .filter(|record| {
                matches!(
                    record.meta.kind,
                    assets::Kind::EditableMesh | assets::Kind::SkeletalMesh
                )
            })
            .collect();
        records.sort_by_key(|record| &record.path);
        for record in records {
            let path = assets::path_string(&e.root, &record.path);
            if ui
                .selectable_config(&path)
                .selected(current == Some(record.meta.id))
                .build()
            {
                chosen = Some(Some(record.meta.id));
            }
            #[cfg(test)]
            crate::gui::record_script_control(ui, &path);
        }
    }
    if let Some(chosen) = chosen
        && chosen != current
    {
        match assign_mesh(entity, &e.assets.index, chosen) {
            Ok(()) => e.mesh_editor.open = false,
            Err(error) => e.log(error),
        }
    }
    if entity.editable_mesh.is_none() && entity.skeletal_mesh.is_none() {
        ui.text_disabled("Engine mesh (read-only)");
    }
}

pub fn component(ui: &Ui, e: &mut Editor, entity: &mut Actor) {
    if entity.editable_mesh.is_none() {
        return;
    }
    ui.separator();
    let mut remove = false;
    let expanded = crate::gui::section(ui, "Mesh Renderer", || {
        remove = ui.menu_item("Remove Editable Mesh");
    });
    if remove {
        entity.editable_mesh = None;
        entity.kind = "Empty".into();
        return;
    }
    if !expanded {
        return;
    }
    selector(ui, e, entity);
    let Some(m) = entity.editable_mesh.as_mut() else {
        return;
    };
    if let Some(error) = &m.error {
        ui.text_wrapped(error);
    }
    if button(ui, "Edit geometry...") {
        match e.assets.index.resolve(m.asset).cloned() {
            Ok(r) => open(e, r, e.selected),
            Err(error) => e.log(error),
        }
    }
    if button(ui, "Make independent copy") {
        let result = (|| {
            let r = e.assets.index.resolve(m.asset)?.clone();
            let path = assets::inside(&e.root, &fresh_path(e))?;
            let id = assets::duplicate(&r, &path)?;
            m.asset = id;
            m.document = Some(Arc::new(mesh::document(&r)?));
            m.error = None;
            e.mesh_editor.open = false;
            preview(e);
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            e.log(error);
        } else {
            e.assets.refresh();
        }
    }
    if let Some(doc) = m.document.clone() {
        crate::gui::muted(ui, "Material overrides apply only to this entity.");
        for slot in &doc.materials {
            let _id = ui.push_id(slot.id.to_string());
            let mut mat = m.materials.get(&slot.id).unwrap_or(&slot.material).clone();
            ui.text(&slot.name);
            let changed = ui.color_edit3(crate::gui::field(ui, "Color"), &mut mat.color)
                | ui.checkbox("Unlit", &mut mat.unlit)
                | crate::texture::picker(ui, &e.assets.index, &mut mat);
            if changed {
                m.materials.insert(slot.id, mat);
            }
            if m.materials.contains_key(&slot.id) && ui.small_button("Use asset default") {
                m.materials.remove(&slot.id);
            }
        }
        if m.materials
            .keys()
            .any(|id| !doc.materials.iter().any(|s| s.id == *id))
        {
            ui.text_wrapped("Overrides for removed material slots are retained for undo/recovery.");
        }
    }
}
/// The edited actor and its mesh, when a geometry editing session owns the
/// Scene view.
fn session(e: &Editor) -> Option<(usize, &Document)> {
    if !e.mesh_editor.open {
        return None;
    }
    let index = e.mesh_editor.target?;
    let doc = e.mesh_editor.doc.as_ref()?;
    e.scene
        .actors
        .get(index)
        .and_then(|v| v.editable_mesh.as_ref())
        .is_some_and(|m| Some(m.asset) == e.mesh_editor.record.as_ref().map(|r| r.meta.id))
        .then_some((index, doc))
}
/// The eye ray through Scene `pixel` in the mesh's local space. The point at
/// depth `d` (as `viewport::project` reports it) is `origin + direction * d`.
fn ray(e: &Editor, inverse: &Matrix, pixel: [f32; 2]) -> ([f32; 3], [f32; 3]) {
    let near = e.view.unproject(pixel, 0.);
    let far = e.view.unproject(pixel, 1.);
    (inverse.point(near), inverse.vector(crate::lighting::sub(far, near)))
}
/// How much nearer than a point a face may be and still not hide it, so the
/// faces that meet at an element never hide it themselves.
const OCCLUSION_TOLERANCE: f32 = 0.03;
/// Faces of the edited mesh that can hide others from the camera, with their
/// bounds on screen, so each test only casts against faces covering its pixel.
struct Occluders<'a> {
    doc: &'a Document,
    inverse: Matrix,
    /// Per face; `None` for faces in hidden groups, which hide nothing.
    bounds: Vec<Option<[f32; 4]>>,
}
impl<'a> Occluders<'a> {
    fn new(e: &Editor, doc: &'a Document, world: &Matrix, projected: &[[f32; 3]]) -> Option<Self> {
        let inverse = world.inverse().ok()?;
        let bounds = doc
            .faces
            .iter()
            .map(|f| {
                e.mesh_editor.visible(doc, f.group).then(|| {
                    let corners = f.vertices.map(|v| projected[v as usize]);
                    if corners.iter().any(|p| p[2] <= 1.) {
                        // Too close to the camera to project reliably.
                        return [f32::NEG_INFINITY, f32::NEG_INFINITY, f32::INFINITY, f32::INFINITY];
                    }
                    corners.iter().fold(
                        [f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY],
                        |b, p| [b[0].min(p[0]), b[1].min(p[1]), b[2].max(p[0]), b[3].max(p[1])],
                    )
                })
            })
            .collect();
        Some(Self { doc, inverse, bounds })
    }
    /// Whether no face of the mesh is in front of `point` (Scene pixel, depth).
    fn front(&self, e: &Editor, point: [f32; 3]) -> bool {
        let pixel = [point[0], point[1]];
        let (origin, direction) = ray(e, &self.inverse, pixel);
        mesh::pick_indexed(self.doc, origin, direction, |i, _| {
            self.bounds[i].is_some_and(|b| {
                (b[0]..=b[2]).contains(&pixel[0]) && (b[1]..=b[3]).contains(&pixel[1])
            })
        })
        .is_none_or(|(_, depth)| depth >= point[2] - OCCLUSION_TOLERANCE)
    }
}
/// The element of the edited mesh at `pixel` in the current mode: `None` when
/// no geometry editing session owns the Scene view, `Some(None)` on a miss.
pub fn hit_test(e: &Editor, pixel: [f32; 2]) -> Option<Option<Hit>> {
    let (index, doc) = session(e)?;
    let inverse = e.scene.world_matrix(index).inverse().ok()?;
    let usable = |f: &mesh::Face| e.mesh_editor.visible(doc, f.group) && e.mesh_editor.unlocked(doc, f.group);
    let hit = match e.mesh_editor.mode {
        Mode::Edges => {
            let world = e.scene.world_matrix(index);
            doc.faces
                .iter()
                .filter(|f| usable(f))
                .flat_map(crate::mesh_ops::edges)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .filter_map(|edge| {
                    let a = crate::viewport::project(&e.view, world.point(doc.vertices[edge[0] as usize]));
                    let b = crate::viewport::project(&e.view, world.point(doc.vertices[edge[1] as usize]));
                    if a[2] <= 1. || b[2] <= 1. {
                        return None;
                    }
                    let d = [b[0] - a[0], b[1] - a[1]];
                    let len = d[0] * d[0] + d[1] * d[1];
                    if len < 0.001 {
                        return None;
                    }
                    let t = (((pixel[0] - a[0]) * d[0] + (pixel[1] - a[1]) * d[1]) / len).clamp(0., 1.);
                    let distance = (pixel[0] - a[0] - d[0] * t).hypot(pixel[1] - a[1] - d[1] * t);
                    // Only edges near the cursor pay for the occlusion test.
                    if distance > 10. {
                        return None;
                    }
                    let depth = 1. / ((1. - t) / a[2] + t / b[2]);
                    let projected = [a[0] + d[0] * t, a[1] + d[1] * t];
                    let (origin, direction) = ray(e, &inverse, projected);
                    let front = mesh::pick(doc, origin, direction, |f| {
                        e.mesh_editor.visible(doc, f.group)
                    })
                    .is_none_or(|(_, z)| z >= depth - OCCLUSION_TOLERANCE);
                    front.then_some((edge, distance, depth))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1).then(a.2.total_cmp(&b.2)))
                .map(|v| Hit::Edge(v.0))
        }
        Mode::Vertices => {
            let world = e.scene.world_matrix(index);
            doc.faces
                .iter()
                .filter(|f| usable(f))
                .flat_map(|f| f.vertices)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .filter_map(|i| {
                    let p = crate::viewport::project(&e.view, world.point(doc.vertices[i as usize]));
                    let d = (p[0] - pixel[0]).hypot(p[1] - pixel[1]);
                    (p[2] > 1. && d <= 10.).then_some((i, d, p[2]))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1).then(a.2.total_cmp(&b.2)))
                .map(|v| Hit::Vertex(v.0))
        }
        Mode::Faces => {
            let a = e.view.unproject(pixel, 1.);
            let b = e.view.unproject(pixel, 2.);
            mesh::pick(
                doc,
                inverse.point(a),
                inverse.vector(crate::lighting::sub(b, a)),
                usable,
            )
            .map(|(id, _)| Hit::Face(id))
        }
    };
    Some(hit)
}
/// The elements of the current mode inside the Scene pixel rectangle `a`..`b`:
/// faces by their centre, edges by both ends, vertices by position. Unless
/// `occluded`, only elements the mesh does not hide from the camera count.
fn box_hits(e: &Editor, a: [f32; 2], b: [f32; 2], occluded: bool) -> Vec<Hit> {
    let Some((index, doc)) = session(e) else {
        return vec![];
    };
    let min = [a[0].min(b[0]), a[1].min(b[1])];
    let max = [a[0].max(b[0]), a[1].max(b[1])];
    let inside = |p: [f32; 3]| {
        p[2] > 1. && (min[0]..=max[0]).contains(&p[0]) && (min[1]..=max[1]).contains(&p[1])
    };
    let world = e.scene.world_matrix(index);
    let project = |p: [f32; 3]| crate::viewport::project(&e.view, world.point(p));
    let projected: Vec<[f32; 3]> = doc.vertices.iter().map(|v| project(*v)).collect();
    let Some(occluders) = Occluders::new(e, doc, &world, &projected) else {
        return vec![];
    };
    let shown = |p: [f32; 3]| occluded || occluders.front(e, p);
    let usable: Vec<&mesh::Face> = doc
        .faces
        .iter()
        .filter(|f| e.mesh_editor.visible(doc, f.group) && e.mesh_editor.unlocked(doc, f.group))
        .collect();
    match e.mesh_editor.mode {
        Mode::Faces => usable
            .iter()
            .filter_map(|f| {
                let corners = if f.vertices[2] == f.vertices[3] { 3 } else { 4 };
                let points = doc.points(f);
                let center: [f32; 3] = std::array::from_fn(|c| {
                    points[..corners].iter().map(|p| p[c]).sum::<f32>() / corners as f32
                });
                let p = project(center);
                (inside(p) && shown(p)).then_some(Hit::Face(f.id))
            })
            .collect(),
        Mode::Edges => usable
            .iter()
            .copied()
            .flat_map(crate::mesh_ops::edges)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|edge| {
                let [a, b] = edge.map(|v| doc.vertices[v as usize]);
                inside(projected[edge[0] as usize])
                    && inside(projected[edge[1] as usize])
                    && shown(project(std::array::from_fn(|c| (a[c] + b[c]) * 0.5)))
            })
            .map(Hit::Edge)
            .collect(),
        Mode::Vertices => usable
            .iter()
            .flat_map(|f| f.vertices)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|&v| inside(projected[v as usize]) && shown(projected[v as usize]))
            .map(Hit::Vertex)
            .collect(),
    }
}
/// Combine `hits` with the selection of the current mode. Replace clears it
/// first, so a plain click on nothing empties it.
fn apply(e: &mut Editor, hits: &[Hit], op: SelectOp) {
    fn combine<T: Ord + Clone>(set: &mut BTreeSet<T>, items: impl Iterator<Item = T>, op: SelectOp) -> bool {
        let before = set.clone();
        if op == SelectOp::Replace {
            set.clear();
        }
        for item in items {
            match op {
                SelectOp::Replace | SelectOp::Add => {
                    set.insert(item);
                }
                SelectOp::Remove => {
                    set.remove(&item);
                }
                SelectOp::Toggle => {
                    if !set.remove(&item) {
                        set.insert(item);
                    }
                }
            }
        }
        *set != before
    }
    let state = &mut e.mesh_editor;
    let changed = match state.mode {
        Mode::Faces => combine(
            &mut state.selected,
            hits.iter().filter_map(|h| match h {
                Hit::Face(id) => Some(*id),
                _ => None,
            }),
            op,
        ),
        Mode::Edges => combine(
            &mut state.edges,
            hits.iter().filter_map(|h| match h {
                Hit::Edge(edge) => Some(*edge),
                _ => None,
            }),
            op,
        ),
        Mode::Vertices => combine(
            &mut state.vertices,
            hits.iter().filter_map(|h| match h {
                Hit::Vertex(v) => Some(*v),
                _ => None,
            }),
            op,
        ),
    };
    if changed {
        state.revision += 1;
    }
    e.view_dirty = true;
}
/// A click in the Scene view while editing geometry: the element under it,
/// combined with the selection by `op`. Clicks never leave the edited mesh; a
/// plain click on nothing clears the selection. `false` when no editing
/// session owns the click.
pub fn pick(e: &mut Editor, pixel: [f32; 2], op: SelectOp) -> bool {
    let Some(hit) = hit_test(e, pixel) else {
        return false;
    };
    apply(e, hit.as_slice(), op);
    true
}
/// Track the element under the cursor (`None` when the cursor is elsewhere).
/// Hover feedback is drawn as an overlay, so the scene is not rebuilt.
pub fn hover(e: &mut Editor, pixel: Option<[f32; 2]>) {
    let hit = pixel.and_then(|pixel| hit_test(e, pixel).flatten());
    if e.mesh_editor.hover != hit {
        e.mesh_editor.hover = hit;
        e.view_dirty = true;
    }
}
/// Select (`op` Add) or deselect (Remove) what the cursor crossed between
/// `from` and `to`, sampling the path so a quick stroke skips nothing.
fn paint(e: &mut Editor, from: [f32; 2], to: [f32; 2], op: SelectOp, include_start: bool) {
    let length = (to[0] - from[0]).hypot(to[1] - from[1]);
    let steps = (length / 2.).ceil().clamp(1., 256.) as usize;
    let mut hits = Vec::new();
    for k in usize::from(!include_start)..=steps {
        let t = k as f32 / steps as f32;
        let p = [from[0] + (to[0] - from[0]) * t, from[1] + (to[1] - from[1]) * t];
        if let Some(Some(hit)) = hit_test(e, p)
            && !hits.contains(&hit)
        {
            hits.push(hit);
        }
    }
    apply(e, &hits, op);
}
/// The Scene view draws the selection (the overlay, and the edited mesh's
/// edges and vertices), and every change to it moves `revision`. Redraw the
/// view whenever that happens, whoever made the change: a Select tile, a group
/// or material, an operation. Called once per frame, after the panels.
pub fn redraw_on_change(e: &mut Editor) {
    if e.mesh_editor.revision != e.mesh_editor.drawn_revision {
        e.mesh_editor.drawn_revision = e.mesh_editor.revision;
        e.view_dirty = true;
    }
}
/// Whether a selection box is being dragged; hover feedback pauses meanwhile.
pub fn boxing(e: &Editor) -> bool {
    matches!(e.mesh_editor.gesture, Gesture::Box { .. })
}
/// The selection box on screen, between where the drag started and the cursor.
fn draw_box(ui: &Ui, a: [f32; 2], b: [f32; 2]) {
    let min = [a[0].min(b[0]), a[1].min(b[1])];
    let max = [a[0].max(b[0]), a[1].max(b[1])];
    let list = ui.get_window_draw_list();
    list.add_rect(min, max, [0.3, 0.76, 1., 0.12]).filled(true).build();
    list.add_rect(min, max, [0.3, 0.76, 1., 0.9]).build();
}
/// Left drags in the Scene view while editing geometry. A box selects what it
/// encloses when released; paint selects what the cursor passes over. Shift
/// adds and Ctrl removes. `view` is the Scene image on screen: the cursor is
/// kept inside it, and `to_pixel` maps it to Scene pixels. `true` while a drag
/// owns the press, so releasing it is not also a click.
pub fn drag(
    ui: &Ui,
    e: &mut Editor,
    view: [[f32; 2]; 2],
    to_pixel: impl Fn([f32; 2]) -> [f32; 2],
    eligible: bool,
) -> bool {
    if !e.mesh_editor.open || e.playing {
        e.mesh_editor.gesture = Gesture::Idle;
        return false;
    }
    let mouse: [f32; 2] = std::array::from_fn(|i| ui.io().mouse_pos[i].clamp(view[0][i], view[1][i]));
    let pixel = to_pixel(mouse);
    let down = ui.is_mouse_down(imgui::MouseButton::Left);
    match e.mesh_editor.gesture {
        Gesture::Idle => {
            if eligible && ui.is_mouse_clicked(imgui::MouseButton::Left) {
                e.mesh_editor.gesture = Gesture::Pressed { mouse, pixel };
            }
            false
        }
        Gesture::Pressed { mouse: start, pixel: from } => {
            if !down {
                e.mesh_editor.gesture = Gesture::Idle;
                return false;
            }
            if (mouse[0] - start[0]).hypot(mouse[1] - start[1]) <= DRAG_SLOP {
                return false;
            }
            let op = SelectOp::drag(ui.io().key_shift, ui.io().key_ctrl);
            e.mesh_editor.gesture = match e.mesh_editor.drag_select {
                DragSelect::Box => {
                    draw_box(ui, start, mouse);
                    Gesture::Box {
                        mouse: start,
                        pixel: from,
                        op,
                    }
                }
                DragSelect::Paint => {
                    // A plain stroke starts a new selection, then adds to it.
                    let op = if op == SelectOp::Replace {
                        apply(e, &[], SelectOp::Replace);
                        SelectOp::Add
                    } else {
                        op
                    };
                    paint(e, from, pixel, op, true);
                    Gesture::Paint { pixel, op }
                }
            };
            true
        }
        Gesture::Box {
            mouse: start,
            pixel: from,
            op,
        } => {
            if down {
                draw_box(ui, start, mouse);
            } else {
                let hits = box_hits(e, from, pixel, e.mesh_editor.select_occluded);
                apply(e, &hits, op);
                e.mesh_editor.gesture = Gesture::Idle;
            }
            true
        }
        Gesture::Paint { pixel: last, op } => {
            if down {
                paint(e, last, pixel, op, false);
                e.mesh_editor.gesture = Gesture::Paint { pixel, op };
            } else {
                e.mesh_editor.gesture = Gesture::Idle;
            }
            true
        }
        Gesture::Cancelled => {
            if !down {
                e.mesh_editor.gesture = Gesture::Idle;
            }
            true
        }
    }
}
fn report(e: &mut Editor, result: Result<(), String>) {
    e.mesh_editor.error = result.err();
}
fn add_instance(e: &mut Editor) -> Result<(), String> {
    if e.scene.actors.len() >= 512 {
        return Err("Scene entity limit: 512".into());
    }
    let record = e.mesh_editor.record.as_ref().ok_or("No mesh open")?;
    let doc = mesh::document(record)?;
    let mut entity = Actor::cube("Blockout".into());
    entity.position = [0.; 3];
    entity.material = Default::default();
    let mut component = mesh::Component::new(record.meta.id);
    component.document = Some(Arc::new(doc));
    entity.editable_mesh = Some(component);
    entity.lighting.static_geometry = true;
    entity.lighting.receive = crate::lighting::Receive::Baked;
    e.scene.actors.push(entity);
    e.mesh_editor.target = Some(e.scene.actors.len() - 1);
    e.selected = e.mesh_editor.target;
    preview(e);
    e.changed();
    Ok(())
}
fn prune_selection(e: &mut Editor) {
    let Some(doc) = &e.mesh_editor.doc else {
        return;
    };
    let faces = doc
        .faces
        .iter()
        .filter(|f| e.mesh_editor.visible(doc, f.group) && e.mesh_editor.unlocked(doc, f.group))
        .map(|f| f.id)
        .collect::<BTreeSet<_>>();
    let forbidden = doc
        .faces
        .iter()
        .filter(|f| !faces.contains(&f.id))
        .flat_map(|f| f.vertices)
        .collect::<BTreeSet<_>>();
    let previous = (e.mesh_editor.selected.len(), e.mesh_editor.vertices.len());
    e.mesh_editor.selected.retain(|id| faces.contains(id));
    e.mesh_editor.vertices.retain(|id| !forbidden.contains(id));
    let edges = doc
        .faces
        .iter()
        .filter(|f| faces.contains(&f.id))
        .flat_map(crate::mesh_ops::edges)
        .collect::<BTreeSet<_>>();
    e.mesh_editor
        .edges
        .retain(|edge| edges.contains(edge) && !edge.iter().any(|v| forbidden.contains(v)));
    if previous != (e.mesh_editor.selected.len(), e.mesh_editor.vertices.len()) {
        e.mesh_editor.revision += 1;
    }
}
pub fn shortcuts(ui: &Ui, e: &mut Editor) {
    // Escape returns to object editing, as in other editors' element modes.
    // An open popup (search, menus, dialogs) keeps Escape for itself.
    let popup = unsafe {
        imgui::sys::igIsPopupOpen_ID(
            0,
            (imgui::sys::ImGuiPopupFlags_AnyPopupId | imgui::sys::ImGuiPopupFlags_AnyPopupLevel)
                as i32,
        )
    };
    if e.mesh_editor.open
        && !e.scene_navigation
        && !popup
        && ui.is_key_pressed_no_repeat(imgui::Key::Escape)
    {
        // A drag in progress is dropped first; the next Escape leaves the mode.
        if ui.is_mouse_down(imgui::MouseButton::Left)
            && !matches!(e.mesh_editor.gesture, Gesture::Idle | Gesture::Cancelled)
        {
            e.mesh_editor.gesture = Gesture::Cancelled;
        } else {
            close(e);
        }
        return;
    }
    if e.mesh_editor.open && !e.playing && !e.scene_navigation && ui.io().key_ctrl {
        let redo = ui.is_key_pressed(imgui::Key::Y)
            || (ui.io().key_shift && ui.is_key_pressed(imgui::Key::Z));
        if redo || ui.is_key_pressed(imgui::Key::Z) {
            let result = undo(e, redo);
            report(e, result);
        }
    }
}
pub fn geometry_shortcuts(ui: &Ui, e: &mut Editor) {
    if !e.mesh_editor.open
        || e.playing
        || ui.io().want_text_input
        || ui.io().key_ctrl
        || ui.io().key_alt
        || e.scene_navigation
    {
        return;
    }
    for (key, mode) in [
        (imgui::Key::Alpha1, Mode::Faces),
        (imgui::Key::Alpha2, Mode::Edges),
        (imgui::Key::Alpha3, Mode::Vertices),
    ] {
        if ui.is_key_pressed(key) {
            set_mode(e, mode);
        }
    }
    if ui.is_key_pressed(imgui::Key::Q) {
        let result = push_selection(e, false);
        report(e, result);
    } else if ui.is_key_pressed(imgui::Key::E) {
        let result = push_selection(e, true);
        report(e, result);
    }
}
fn set_mode(e: &mut Editor, mode: Mode) {
    if e.mesh_editor.mode != mode {
        e.mesh_editor.mode = mode;
        e.mesh_editor.selected.clear();
        e.mesh_editor.vertices.clear();
        e.mesh_editor.edges.clear();
        e.mesh_editor.revision += 1;
        e.view_dirty = true;
    }
}
fn push_selection(e: &mut Editor, outward: bool) -> Result<(), String> {
    prune_selection(e);
    let distance = e.mesh_editor.distance.abs();
    match e.mesh_editor.mode {
        Mode::Faces => {
            let selected = e.mesh_editor.selected.clone();
            if selected.is_empty() {
                return Err("Select one or more faces".into());
            }
            edit(e, |d| {
                d.extrude(&selected, if outward { distance } else { -distance })
            })
        }
        Mode::Edges if !outward => {
            let edges = e.mesh_editor.edges.clone();
            let doc = e.mesh_editor.doc.as_ref().ok_or("No mesh open")?;
            let protected = doc
                .faces
                .iter()
                .filter(|f| {
                    !e.mesh_editor.visible(doc, f.group) || !e.mesh_editor.unlocked(doc, f.group)
                })
                .map(|f| (f.clone(), doc.points(f)))
                .collect::<Vec<_>>();
            edit(e, |d| {
                crate::mesh_ops::bevel(d, &edges, distance)?;
                if protected.iter().any(|(old, points)| {
                    d.faces.iter().find(|f| f.id == old.id).is_none_or(|f| {
                        d.points(f) != *points || f.material != old.material || f.group != old.group
                    })
                }) {
                    return Err("Bevel would change hidden or locked faces".into());
                }
                Ok(())
            })
        }
        Mode::Edges => Err("Q bevels the selected edge; E extrudes faces in Face mode (1)".into()),
        Mode::Vertices => Err("Switch to Faces (1) to extrude, or Edges (2) to bevel".into()),
    }
}
/// Tool categories of the Geometry panel, in palette order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Category {
    Create,
    #[default]
    Select,
    Transform,
    Model,
    Sculpt,
    Texture,
    Groups,
    Materials,
    Asset,
}
const CATEGORIES: [(Category, &str); 9] = [
    (Category::Create, "Create"),
    (Category::Select, "Select"),
    (Category::Transform, "Transform"),
    (Category::Model, "Model"),
    (Category::Sculpt, "Sculpt"),
    (Category::Texture, "Texture"),
    (Category::Groups, "Groups"),
    (Category::Materials, "Materials"),
    (Category::Asset, "Asset"),
];
/// Tools of the Model category: each shows its settings and an action button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ModelTool {
    #[default]
    Extrude,
    Push,
    Subdivide,
    Flip,
    Delete,
    Extract,
}
// Palette glyphs (Font Awesome; listed in platform::EDITOR_FA_GLYPHS).
const ICON_UNDO: &str = "\u{f2ea}";
const ICON_REDO: &str = "\u{f2f9}";
const ICON_REVERT: &str = "\u{f1da}";
const ICON_DONE: &str = "\u{f00c}";
const TILE: [f32; 2] = [76., 58.];

/// A palette tile: an icon over a short label, highlighted when `active`.
/// `id` is its tooltip and the name scripted tests click it by.
fn tile(ui: &Ui, glyph: &str, label: &str, id: &str, active: bool) -> bool {
    let p = ui.cursor_screen_pos();
    let pressed = ui.invisible_button(format!("##tile-{id}"), TILE);
    #[cfg(test)]
    BUTTONS.with(|buttons| {
        buttons
            .borrow_mut()
            .insert(id.to_owned(), [p[0] + TILE[0] * 0.5, p[1] + TILE[1] * 0.5]);
    });
    let hovered = ui.is_item_hovered();
    let draw = ui.get_window_draw_list();
    let b = [p[0] + TILE[0], p[1] + TILE[1]];
    let (fill, border) = if active {
        ([0.13, 0.30, 0.52, 1.], ACCENT)
    } else if hovered {
        (crate::gui::gray(56), crate::gui::gray(88))
    } else {
        (crate::gui::gray(40), crate::gui::gray(56))
    };
    draw.add_rect(p, b, fill).filled(true).rounding(5.).build();
    draw.add_rect(p, b, border).rounding(5.).build();
    let icon = ui.calc_text_size(glyph);
    draw.add_text(
        [p[0] + (TILE[0] - icon[0]) * 0.5, p[1] + 11.],
        if active { [1., 1., 1., 1.] } else { crate::gui::gray(225) },
        glyph,
    );
    let text = ui.calc_text_size(label);
    draw.add_text(
        [p[0] + ((TILE[0] - text[0]) * 0.5).max(4.), b[1] - text[1] - 8.],
        if active { [0.85, 0.92, 1., 1.] } else { crate::gui::gray(195) },
        label,
    );
    if hovered {
        ui.tooltip_text(id);
    }
    pressed
}
/// Lay `tiles` out in rows that fit the panel; the index of a clicked tile.
fn tile_grid(ui: &Ui, tiles: &[(&str, &str, &str, bool)]) -> Option<usize> {
    let width = ui.content_region_avail()[0];
    let spacing = 6.;
    let per_row = (((width + spacing) / (TILE[0] + spacing)).floor() as usize).max(1);
    let mut clicked = None;
    let _spacing = ui.push_style_var(imgui::StyleVar::ItemSpacing([spacing, spacing]));
    for (i, (glyph, label, id, active)) in tiles.iter().enumerate() {
        if i % per_row != 0 {
            ui.same_line();
        }
        if tile(ui, glyph, label, id, *active) {
            clicked = Some(i);
        }
    }
    clicked
}
/// A small icon button for the panel header; `id` names it in tooltips and tests.
fn icon_button(ui: &Ui, glyph: &str, id: &str, tip: &str) -> bool {
    let pressed = ui.button(format!("{glyph}##{id}"));
    #[cfg(test)]
    BUTTONS.with(|buttons| {
        let a = ui.item_rect_min();
        let b = ui.item_rect_max();
        buttons
            .borrow_mut()
            .insert(id.to_owned(), [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]);
    });
    if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
        ui.tooltip_text(tip);
    }
    pressed
}

pub fn window(ui: &Ui, e: &mut Editor) {
    if !e.mesh_editor.open {
        return;
    }
    if e.playing {
        // Play runs the saved geometry; editing resumes from the Inspector.
        close(e);
        return;
    }
    prune_selection(e);
    // Take the Hierarchy's place: while the mode lasts the Hierarchy is hidden,
    // so nothing else can be chosen until the mode ends. A floating Hierarchy
    // has no dock node; then the panel floats and the Hierarchy stays.
    // Take the Hierarchy's place: while the mode lasts the Hierarchy is hidden,
    // so nothing else can be chosen until the mode ends. The request repeats
    // until the panel is really in that node (a freshly built layout docks a
    // frame late). A floating Hierarchy has no node: the panel floats instead.
    if e.mesh_editor.dock_pending && e.hierarchy_dock != 0 {
        unsafe {
            imgui::sys::igSetNextWindowDockID(e.hierarchy_dock, imgui::sys::ImGuiCond_Always as i32);
            imgui::sys::igSetNextWindowFocus();
        }
    }
    let mut open = true;
    let _padding = ui.push_style_var(imgui::StyleVar::WindowPadding([10., 8.]));
    ui.window("\u{f304} Geometry###Blockout")
        .opened(&mut open)
        .position([20., 80.], Condition::FirstUseEver)
        .size([340., (ui.io().display_size[1] - 110.).min(720.)], Condition::FirstUseEver)
        .size_constraints([260., 360.], [1400., 1600.])
        .build(|| {
            let dock = unsafe { imgui::sys::igGetWindowDockID() };
            e.mesh_editor.docked = dock != 0 && dock == e.hierarchy_dock;
            if e.mesh_editor.docked {
                e.mesh_editor.dock_pending = false;
            }
            let Some(doc) = e.mesh_editor.doc.clone() else {
                return;
            };
            let Some(record) = e.mesh_editor.record.clone() else {
                return;
            };
            if header(ui, e) {
                return;
            }
            categories(ui, e);
            ui.separator();
            let footer = ui.frame_height_with_spacing() + 6.;
            ui.child_window("##geometry-tools")
                .size([0., -footer])
                .build(|| tools(ui, e, &doc, &record));
            ui.separator();
            panel_footer(ui, e, &doc, &record);
        });
    if !open {
        close(e);
    }
}
/// The longest prefix of `text` that fits `width`, with "..." when cut.
fn fit(ui: &Ui, text: &str, width: f32) -> String {
    if ui.calc_text_size(text)[0] <= width {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    for n in (0..chars.len()).rev() {
        let candidate = format!("{}...", chars[..n].iter().collect::<String>());
        if ui.calc_text_size(&candidate)[0] <= width {
            return candidate;
        }
    }
    String::new()
}
/// Name, then the element mode on the left and history and session commands
/// on the right. `true` when Done ended the session.
fn header(ui: &Ui, e: &mut Editor) -> bool {
    if let Some(error) = &e.mesh_editor.error {
        let _color = ui.push_style_color(imgui::StyleColor::Text, [1., 0.5, 0.3, 1.]);
        ui.text_wrapped(error);
    }
    ui.align_text_to_frame_padding();
    ui.text_colored(ACCENT, BANNER_ICON);
    ui.same_line();
    let name = banner_name(e);
    ui.text(fit(ui, &name, ui.content_region_avail()[0]));
    if ui.is_item_hovered()
        && let Some(record) = &e.mesh_editor.record
    {
        ui.tooltip_text(format!("{name}\n{}", assets::path_string(&e.root, &record.path)));
    }
    for (i, (mode, label, tip)) in MODES.into_iter().enumerate() {
        if i > 0 {
            ui.same_line_with_spacing(0., 2.);
        }
        let active = (e.mesh_editor.mode == mode)
            .then(|| ui.push_style_color(imgui::StyleColor::Button, [0.16, 0.36, 0.62, 1.]));
        if button(ui, label) {
            set_mode(e, mode);
        }
        drop(active);
        if ui.is_item_hovered() {
            ui.tooltip_text(tip);
        }
    }
    // Four icon buttons, right-aligned; they wrap below on a narrow panel.
    let icons = 4. * (ui.frame_height() + 10.) + 3. * 4.;
    let right = ui.window_content_region_max()[0] - icons;
    ui.same_line();
    if ui.cursor_pos()[0] + 4. > right {
        ui.new_line();
    }
    let start = ui.cursor_pos();
    ui.set_cursor_pos([right.max(start[0]), start[1]]);
    if icon_button(ui, ICON_UNDO, "Undo", "Undo geometry (Ctrl+Z)") {
        let result = undo(e, false);
        report(e, result);
    }
    ui.same_line_with_spacing(0., 4.);
    if icon_button(ui, ICON_REDO, "Redo", "Redo geometry (Ctrl+Y)") {
        let result = undo(e, true);
        report(e, result);
    }
    ui.same_line_with_spacing(0., 4.);
    let changed = changed_in_session(e);
    ui.disabled(!changed, || {
        if icon_button(
            ui,
            ICON_REVERT,
            "Revert session",
            "Revert: restore the mesh as it was when editing started (one Undo step)",
        ) {
            let result = revert(e);
            report(e, result);
        }
    });
    ui.same_line_with_spacing(0., 4.);
    let done = {
        let _accent = ui.push_style_color(imgui::StyleColor::Button, [0.16, 0.36, 0.62, 1.]);
        icon_button(
            ui,
            ICON_DONE,
            "Done editing",
            "Done: leave geometry editing (Esc); changes are already saved",
        )
    };
    if done {
        close(e);
        return true;
    }
    false
}
/// Category chips, wrapped to the panel width.
fn categories(ui: &Ui, e: &mut Editor) {
    let _vars = [
        ui.push_style_var(imgui::StyleVar::FrameRounding(10.)),
        ui.push_style_var(imgui::StyleVar::FramePadding([9., 3.])),
        ui.push_style_var(imgui::StyleVar::ItemSpacing([5., 5.])),
    ];
    let start = ui.cursor_pos()[0];
    let limit = ui.content_region_avail()[0] + start;
    for (i, (category, label)) in CATEGORIES.into_iter().enumerate() {
        let width = ui.calc_text_size(label)[0] + 18.;
        if i > 0 {
            ui.same_line();
            if ui.cursor_pos()[0] + width > limit {
                ui.new_line();
            }
        }
        let active = e.mesh_editor.category == category;
        let colors = active.then(|| {
            [
                ui.push_style_color(imgui::StyleColor::Button, [0.16, 0.36, 0.62, 1.]),
                ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.20, 0.42, 0.70, 1.]),
            ]
        });
        if button(ui, label) {
            e.mesh_editor.category = category;
        }
        drop(colors);
    }
}
/// What is selected, and how many scene instances share this mesh.
fn panel_footer(ui: &Ui, e: &Editor, _doc: &Document, record: &assets::Record) {
    let state = &e.mesh_editor;
    ui.text(format!(
        "Selected: {} faces / {} edges / {} vertices",
        state.selected.len(),
        state.edges.len(),
        state.vertices.len()
    ));
    let users = e
        .scene
        .actors
        .iter()
        .filter(|v| v.editable_mesh.as_ref().is_some_and(|m| m.asset == record.meta.id))
        .count();
    if users > 1 {
        ui.same_line();
        ui.text_disabled(format!("| shared by {users} instances"));
    }
}
/// The active category's tools.
fn tools(ui: &Ui, e: &mut Editor, doc: &Document, record: &assets::Record) {
    match e.mesh_editor.category {
        Category::Create => create_section(ui, e, doc),
        Category::Select => select_section(ui, e, doc),
        Category::Transform => transform_section(ui, e, doc),
        Category::Model => model_section(ui, e, doc),
        Category::Sculpt => sculpt_controls(ui, e, doc),
        Category::Texture => texture_correction(ui, e, doc),
        Category::Groups => groups_panel(ui, e, doc),
        Category::Materials => materials_panel(ui, e, doc),
        Category::Asset => asset_section(ui, e, doc, record),
    }
}
fn asset_section(ui: &Ui, e: &mut Editor, doc: &Document, record: &assets::Record) {
    ui.input_text(crate::gui::field(ui, "Asset path"), &mut e.mesh_editor.path)
        .build();
    if button(ui, "Move / Rename asset") {
        match assets::move_asset(&e.root, record, &e.mesh_editor.path) {
            Ok(()) => {
                e.mesh_editor.record.as_mut().unwrap().path =
                    assets::inside(&e.root, &e.mesh_editor.path).unwrap();
                e.assets.refresh();
            }
            Err(error) => e.mesh_editor.error = Some(error),
        }
    }
    let users = e
        .scene
        .actors
        .iter()
        .filter(|v| v.editable_mesh.as_ref().is_some_and(|m| m.asset == record.meta.id))
        .count();
    ui.text_wrapped(format!(
        "Shared asset: {users} instance(s) in this scene. Edits save automatically."
    ));
    if users == 0 && button(ui, "Add instance to scene") {
        let result = add_instance(e);
        report(e, result);
    }
    if button(ui, "Reload external changes") {
        open_asset_reload(e, record);
    }
    ui.separator();
    ui.text_wrapped(format!(
        "{} authored faces / {} stored vertices / {} groups / {} materials",
        doc.faces.len(),
        doc.vertices.len(),
        doc.groups.len(),
        doc.materials.len()
    ));
    let mut entity = Actor::cube("Budget".into());
    let mut component = mesh::Component::new(record.meta.id);
    component.document = Some(Arc::new(doc.clone()));
    entity.editable_mesh = Some(component);
    let quads = crate::lighting::quads(&entity);
    if let Ok(chunks) = crate::mesh_compile::chunks(&quads) {
        let triangles: usize = quads
            .iter()
            .map(|q| if q.points[2] == q.points[3] { 1 } else { 2 })
            .sum();
        ui.text_wrapped(format!(
            "{} compiled triangles / {} spatial chunks / {} indexed vertices",
            triangles,
            chunks.len(),
            chunks.iter().map(|c| c.vertices.len()).sum::<usize>()
        ));
    }
    ui.text_wrapped("PSX: chunk culling, shared positions, backface rejection and clipping. Scene build budget: 7000 base triangles plus clipping reserve. Measure performance in Play.");
}

const BANNER_HEIGHT: f32 = 34.;
const ACCENT: [f32; 4] = [0.36, 0.66, 1., 1.];
const MODES: [(Mode, &str, &str); 3] = [
    (Mode::Faces, "Faces", "Select faces (1)"),
    (Mode::Edges, "Edges", "Select edges (2)"),
    (Mode::Vertices, "Vertices", "Select vertices (3)"),
];
/// Pen glyph (listed in platform::EDITOR_FA_GLYPHS) and the mode's name.
const BANNER_ICON: &str = "\u{f304}";
const BANNER_TITLE: &str = "Editing geometry";
/// The edited mesh and, when it has one in this scene, its actor.
fn banner_name(e: &Editor) -> String {
    let mesh = e.mesh_editor.record.as_ref().map_or_else(String::new, |r| {
        let name = r.path.file_name().unwrap_or_default().to_string_lossy();
        crate::project_browser::display_file_name(&name).to_string()
    });
    // Generated meshes are named by UUID; keep the banner compact.
    let short = |s: &str| {
        if s.chars().count() > 28 {
            format!("{}...", s.chars().take(25).collect::<String>())
        } else {
            s.to_string()
        }
    };
    match target_actor(e).and_then(|id| e.scene.actors.iter().find(|a| a.id == id)) {
        Some(actor) => format!("{} ({})", short(&mesh), short(&actor.name)),
        None => short(&mesh),
    }
}
fn selection_count(e: &Editor) -> String {
    let state = &e.mesh_editor;
    let (n, one, many) = match state.mode {
        Mode::Faces => (state.selected.len(), "face", "faces"),
        Mode::Edges => (state.edges.len(), "edge", "edges"),
        Mode::Vertices => (state.vertices.len(), "vertex", "vertices"),
    };
    match n {
        0 => format!("No {many} selected"),
        1 => format!("1 {one}"),
        n => format!("{n} {many}"),
    }
}
/// How the banner fits the Scene view: its size, what text it keeps, and
/// whether its controls wrap onto a second row in a narrow panel.
#[derive(Clone, Copy, Debug, PartialEq)]
struct BannerLayout {
    size: [f32; 2],
    title: bool,
    name: bool,
    two_rows: bool,
}
fn banner_layout(ui: &Ui, e: &Editor, available: f32) -> BannerLayout {
    const SPACING: f32 = 8.;
    const PADDING: f32 = 26.;
    let text = |t: &str| ui.calc_text_size(t)[0];
    let button = |t: &str| text(t) + 16.;
    let icon = text(BANNER_ICON) + SPACING;
    let identity = MODES.iter().map(|(_, label, _)| button(label) + SPACING).sum::<f32>()
        + text("No vertices selected");
    let actions = ui.frame_height()
        + 4.
        + text("Dim others")
        + SPACING
        + button("Revert")
        + SPACING
        + button("Done (Esc)");
    let title = text(BANNER_TITLE) + SPACING;
    let name = text(&banner_name(e)) + SPACING;
    let limit = (available - 24.).max(160.);
    let row = BANNER_HEIGHT;
    let one = |width: f32, title: bool, name: bool| BannerLayout {
        size: [width + PADDING, row],
        title,
        name,
        two_rows: false,
    };
    let core = icon + identity + SPACING + actions;
    if core + title + name + PADDING <= limit {
        one(core + title + name, true, true)
    } else if core + title + PADDING <= limit {
        one(core + title, true, false)
    } else if core + PADDING <= limit {
        one(core, false, false)
    } else {
        BannerLayout {
            size: [((icon + identity).max(actions) + PADDING).min(limit), row * 2. - 6.],
            title: false,
            name: false,
            two_rows: true,
        }
    }
}
/// Where the geometry-editing banner sits over the Scene image: centred at the
/// bottom, where tool confirmations live in other editors, clear of the tool
/// column and orientation cube above. `None` when no session is open.
pub fn banner_rect(
    ui: &Ui,
    e: &Editor,
    origin: [f32; 2],
    available: [f32; 2],
) -> Option<[[f32; 2]; 2]> {
    if !e.mesh_editor.open {
        return None;
    }
    let size = banner_layout(ui, e, available[0]).size;
    let x = origin[0] + ((available[0] - size[0]) * 0.5).max(8.);
    let y = origin[1] + (available[1] - size[1] - 10.).max(130.);
    Some([[x, y], [x + size[0], y + size[1]]])
}
/// The Scene view's mode banner: what is being edited, the element mode, the
/// selection, dimming, Revert and Done. It is the visible edge of the mode.
pub fn banner(ui: &Ui, e: &mut Editor, origin: [f32; 2], available: [f32; 2]) {
    let Some([min, max]) = banner_rect(ui, e, origin, available) else {
        return;
    };
    let layout = banner_layout(ui, e, available[0]);
    let name = banner_name(e);
    let count = selection_count(e);
    ui.set_cursor_screen_pos(min);
    let _colors = [
        ui.push_style_color(imgui::StyleColor::ChildBg, [0.08, 0.12, 0.18, 0.94]),
        ui.push_style_color(imgui::StyleColor::Border, [0.25, 0.52, 0.86, 1.]),
    ];
    let _vars = [
        ui.push_style_var(imgui::StyleVar::WindowPadding([12., 6.])),
        ui.push_style_var(imgui::StyleVar::ChildRounding(5.)),
        ui.push_style_var(imgui::StyleVar::ChildBorderSize(1.)),
        ui.push_style_var(imgui::StyleVar::ItemSpacing([8., 6.])),
        ui.push_style_var(imgui::StyleVar::FramePadding([8., 3.])),
    ];
    ui.child_window("##mesh-edit-banner")
        .size([max[0] - min[0], max[1] - min[1]])
        .border(true)
        .scroll_bar(false)
        .scrollable(false)
        .build(|| {
            ui.align_text_to_frame_padding();
            ui.text_colored(ACCENT, BANNER_ICON);
            if ui.is_item_hovered() {
                ui.tooltip_text(format!("{BANNER_TITLE}: {name}"));
            }
            if layout.title {
                ui.same_line();
                ui.text_colored(ACCENT, BANNER_TITLE);
            }
            if layout.name {
                ui.same_line();
                ui.text_disabled(&name);
            }
            for (mode, label, tip) in MODES {
                ui.same_line();
                let current = e.mesh_editor.mode == mode;
                let active = current
                    .then(|| ui.push_style_color(imgui::StyleColor::Button, [0.16, 0.36, 0.62, 1.]));
                if button(ui, label) {
                    set_mode(e, mode);
                }
                drop(active);
                if ui.is_item_hovered() {
                    ui.tooltip_text(tip);
                }
            }
            ui.same_line();
            ui.text(&count);
            if !layout.two_rows {
                ui.same_line();
            }
            if ui.checkbox("Dim others", &mut e.mesh_editor.dim_others) {
                e.mesh_editor.revision += 1;
                e.view_dirty = true;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Darken the rest of the scene while editing this mesh");
            }
            ui.same_line();
            let changed = changed_in_session(e);
            ui.disabled(!changed || e.playing, || {
                if button(ui, "Revert") {
                    let result = revert(e);
                    report(e, result);
                }
            });
            if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
                ui.tooltip_text("Restore the mesh as it was when editing started (one Undo step)");
            }
            ui.same_line();
            if button(ui, "Done (Esc)") {
                close(e);
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Leave geometry editing; changes are already saved");
            }
        });
}
fn open_asset_reload(e: &mut Editor, record: &assets::Record) {
    let index = assets::scan(&e.root, &mut Default::default());
    match index.resolve(record.meta.id).cloned() {
        Ok(r) => open(e, r, e.mesh_editor.target),
        Err(error) => e.mesh_editor.error = Some(error),
    }
}
/// Shapes added at the origin below, snapped to the grid.
fn create_section(ui: &Ui, e: &mut Editor, doc: &Document) {
    let shapes = ["Box", "Plane", "Ramp", "Stairs"];
    let glyphs = ["\u{f1b2}", "\u{f0c8}", "\u{e098}", "\u{e289}"];
    crate::gui::muted(ui, "Add a shape to this mesh");
    let tiles = shapes
        .iter()
        .zip(glyphs)
        .enumerate()
        .map(|(i, (label, glyph))| (glyph, *label, *label, e.mesh_editor.shape == i))
        .collect::<Vec<_>>();
    if let Some(i) = tile_grid(ui, &tiles) {
        e.mesh_editor.shape = i;
    }
    ui.spacing();
    crate::gui::Drag::new(crate::gui::field(ui, "Origin"))
        .speed(0.02)
        .build_array(ui, &mut e.mesh_editor.origin);
    crate::gui::Drag::new(crate::gui::field(ui, "Size"))
        .speed(0.02)
        .build_array(ui, &mut e.mesh_editor.size);
    if shapes[e.mesh_editor.shape] == "Stairs" {
        crate::gui::Drag::new(crate::gui::field(ui, "Steps"))
            .range(1, 32)
            .build(ui, &mut e.mesh_editor.steps);
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Grid"))
        .range(0.001, 4.)
        .speed(0.01)
        .build(ui, &mut e.mesh_editor.grid);
    if button(ui, "Add shape") {
        let shape = shapes[e.mesh_editor.shape];
        let grid = e.mesh_editor.grid.max(0.001);
        let origin = e.mesh_editor.origin.map(|v| (v / grid).round() * grid);
        let size = e.mesh_editor.size;
        let steps = e.mesh_editor.steps as usize;
        let group = e.mesh_editor.group.unwrap_or(doc.groups[0].id);
        let slot = e.mesh_editor.slot.unwrap_or(doc.materials[0].id);
        let result = edit(e, |doc| {
            if size.iter().any(|v| !v.is_finite() || *v <= 0.) {
                return Err("Shape size must be positive".into());
            }
            doc.primitive(shape, origin, size, steps, group, slot);
            Ok(())
        });
        report(e, result);
    }
}
/// Selection helpers; each tile acts at once. Clicking in the Scene selects too.
fn select_section(ui: &Ui, e: &mut Editor, doc: &Document) {
    crate::gui::muted(
        ui,
        "Click or drag in the Scene to select. Shift adds, Ctrl removes; Ctrl-click toggles.",
    );
    let tiles = [
        ("\u{f00a}", "All", "Select all visible", false),
        ("\u{f00d}", "Clear", "Clear selection", false),
        ("\u{f5fd}", "Coplanar", "Select coplanar", false),
        ("\u{f0c1}", "Connected", "Select connected", false),
    ];
    match tile_grid(ui, &tiles) {
        Some(0) => {
            e.mesh_editor.selected = doc
                .faces
                .iter()
                .filter(|f| e.mesh_editor.visible(doc, f.group) && e.mesh_editor.unlocked(doc, f.group))
                .map(|f| f.id)
                .collect();
            if e.mesh_editor.mode == Mode::Edges {
                e.mesh_editor.edges = doc
                    .faces
                    .iter()
                    .filter(|f| e.mesh_editor.selected.contains(&f.id))
                    .flat_map(crate::mesh_ops::edges)
                    .collect();
                e.mesh_editor.selected.clear();
            } else if e.mesh_editor.mode == Mode::Vertices {
                e.mesh_editor.vertices = doc.selected_vertices(&e.mesh_editor.selected);
                e.mesh_editor.selected.clear();
            }
            e.mesh_editor.revision += 1;
        }
        Some(1) => {
            e.mesh_editor.selected.clear();
            e.mesh_editor.vertices.clear();
            e.mesh_editor.edges.clear();
            e.mesh_editor.revision += 1;
        }
        Some(2) => {
            if let Some(f) = doc
                .faces
                .iter()
                .find(|f| e.mesh_editor.selected.contains(&f.id))
            {
                let p = doc.points(f);
                let n = mesh::face_normal(p);
                e.mesh_editor.selected = doc
                    .faces
                    .iter()
                    .filter(|f| e.mesh_editor.visible(doc, f.group) && e.mesh_editor.unlocked(doc, f.group))
                    .filter(|f| {
                        let q = doc.points(f);
                        crate::lighting::dot(n, mesh::face_normal(q)) > 0.999
                            && q.iter().all(|q| {
                                crate::lighting::dot(n, crate::lighting::sub(*q, p[0])).abs() < 0.001
                            })
                    })
                    .map(|f| f.id)
                    .collect();
                e.mesh_editor.revision += 1;
            }
        }
        Some(3) => {
            let mut selected = e.mesh_editor.selected.clone();
            loop {
                let points = doc
                    .faces
                    .iter()
                    .filter(|f| selected.contains(&f.id))
                    .flat_map(|f| doc.points(f))
                    .collect::<Vec<_>>();
                let before = selected.len();
                for f in &doc.faces {
                    if e.mesh_editor.visible(doc, f.group)
                        && e.mesh_editor.unlocked(doc, f.group)
                        && doc.points(f).iter().any(|p| points.contains(p))
                    {
                        selected.insert(f.id);
                    }
                }
                if before == selected.len() {
                    break;
                }
            }
            e.mesh_editor.selected = selected;
            e.mesh_editor.revision += 1;
        }
        _ => {}
    }
    ui.spacing();
    ui.align_text_to_frame_padding();
    ui.text("Drag");
    for (mode, label, tip) in [
        (
            DragSelect::Box,
            "Box",
            "Drag a rectangle: what it encloses is selected when you release",
        ),
        (
            DragSelect::Paint,
            "Paint",
            "Drag across the mesh: each element the cursor passes over is selected",
        ),
    ] {
        ui.same_line();
        ui.radio_button(label, &mut e.mesh_editor.drag_select, mode);
        if ui.is_item_hovered() {
            ui.tooltip_text(tip);
        }
    }
    ui.disabled(e.mesh_editor.drag_select != DragSelect::Box, || {
        ui.checkbox("Select through", &mut e.mesh_editor.select_occluded);
        if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
            ui.tooltip_text(
                "The box also selects elements that other faces of this mesh hide from the camera",
            );
        }
    });
    ui.spacing();
    ui.text_wrapped("Select by material in Materials and by group in Groups. Ctrl+Z / Ctrl+Y undo and redo geometry.");
}
/// Numeric move, rotation and scale of the selection about its center.
fn transform_section(ui: &Ui, e: &mut Editor, doc: &Document) {
    crate::gui::muted(ui, "Transform the selection about its center");
    crate::gui::Drag::new(crate::gui::field(ui, "Move"))
        .speed(0.02)
        .build_array(ui, &mut e.mesh_editor.offset);
    crate::gui::Drag::new(crate::gui::field(ui, "Rotate degrees"))
        .speed(0.02)
        .build_array(ui, &mut e.mesh_editor.rotation);
    crate::gui::Drag::new(crate::gui::field(ui, "Scale selection"))
        .speed(0.02)
        .build_array(ui, &mut e.mesh_editor.scale);
    if button(ui, "Apply selection transform") {
        let vertices = if e.mesh_editor.mode == Mode::Vertices {
            e.mesh_editor.vertices.clone()
        } else if e.mesh_editor.mode == Mode::Edges {
            e.mesh_editor.edges.iter().flatten().copied().collect()
        } else {
            doc.selected_vertices(&e.mesh_editor.selected)
        };
        let offset = e.mesh_editor.offset;
        let forbidden = doc
            .faces
            .iter()
            .filter(|f| {
                !e.mesh_editor.visible(doc, f.group) || !e.mesh_editor.unlocked(doc, f.group)
            })
            .flat_map(|f| f.vertices)
            .collect::<BTreeSet<_>>();
        let rotation = e.mesh_editor.rotation;
        let scale = e.mesh_editor.scale;
        let grid = e.mesh_editor.grid.max(0.001);
        let result = edit(e, |doc| {
            if vertices.is_empty() {
                return Err("Select faces or vertices".into());
            }
            if vertices.iter().any(|v| forbidden.contains(v)) {
                return Err("The selection shares vertices with hidden or locked faces".into());
            }
            let center = std::array::from_fn(|c| {
                vertices
                    .iter()
                    .map(|v| doc.vertices[*v as usize][c])
                    .sum::<f32>()
                    / vertices.len() as f32
            });
            let transform = crate::transform::Matrix::trs(offset, rotation, scale);
            for i in vertices {
                let v = transform.point(crate::lighting::sub(doc.vertices[i as usize], center));
                doc.vertices[i as usize] =
                    std::array::from_fn(|c| ((v[c] + center[c]) / grid).round() * grid);
            }
            Ok(())
        });
        report(e, result);
    }
    ui.text_wrapped("Results snap to the Create grid. Shared corners move with every face that uses them.");
}
/// Topology tools: pick one, then apply it to the selection.
fn model_section(ui: &Ui, e: &mut Editor, doc: &Document) {
    let _ = doc;
    let edges = e.mesh_editor.mode == Mode::Edges;
    let current = e.mesh_editor.model_tool;
    let tools = [
        (ModelTool::Extrude, "\u{f062}", "Extrude", "Extrude faces"),
        (
            ModelTool::Push,
            if edges { "\u{f5cb}" } else { "\u{f063}" },
            if edges { "Bevel" } else { "Push in" },
            "Push faces in or bevel an edge",
        ),
        (ModelTool::Subdivide, "\u{f00a}", "Subdivide", "Subdivide selected faces"),
        (ModelTool::Flip, "\u{f0ec}", "Flip", "Flip face winding"),
        (ModelTool::Delete, "\u{f1f8}", "Delete", "Delete faces"),
        (ModelTool::Extract, "\u{f24d}", "Extract", "Extract to a child actor"),
    ];
    let tiles = tools
        .iter()
        .map(|(tool, glyph, label, id)| (*glyph, *label, *id, *tool == current))
        .collect::<Vec<_>>();
    if let Some(i) = tile_grid(ui, &tiles) {
        e.mesh_editor.model_tool = tools[i].0;
    }
    ui.spacing();
    ui.separator();
    match e.mesh_editor.model_tool {
        ModelTool::Extrude => {
            crate::gui::Drag::new(crate::gui::field(ui, "Extrusion distance"))
                .speed(0.1)
                .build(ui, &mut e.mesh_editor.distance);
            if button(ui, "Extrude (E)") {
                let result = push_selection(e, true);
                report(e, result);
            }
            ui.text_wrapped("Moves each selected face out along its normal and joins it with new side faces. With the Scene focused, E does the same.");
        }
        ModelTool::Push => {
            crate::gui::Drag::new(crate::gui::field(ui, "Extrusion distance"))
                .speed(0.1)
                .build(ui, &mut e.mesh_editor.distance);
            if button(
                ui,
                if edges {
                    "Bevel / ramp (Q)"
                } else {
                    "Push inward (Q)"
                },
            ) {
                let result = push_selection(e, false);
                report(e, result);
            }
            ui.text_wrapped("Faces: pushes them inward. Edges (2): cuts the corner into a sloped face; needs a convex closed solid and two quad faces. With the Scene focused, Q does the same.");
        }
        ModelTool::Subdivide => {
            if button(ui, "Subdivide faces") {
                let selected = e.mesh_editor.selected.clone();
                let result = edit(e, |d| {
                    d.split(&selected);
                    Ok(())
                });
                report(e, result);
            }
            ui.text_wrapped("Splits each selected face into four (three for a triangle). To split by edge length without cracks, use Texture.");
        }
        ModelTool::Flip => {
            if button(ui, "Flip faces") {
                let selected = e.mesh_editor.selected.clone();
                let result = edit(e, |d| {
                    for f in &mut d.faces {
                        if selected.contains(&f.id) {
                            if f.vertices[2] == f.vertices[3] {
                                f.vertices = [f.vertices[0], f.vertices[2], f.vertices[1], f.vertices[1]];
                                f.uv = [f.uv[0], f.uv[2], f.uv[1], f.uv[1]];
                                f.vertex_colors = f.vertex_colors.map(|c| [c[0], c[2], c[1], c[1]]);
                            } else {
                                f.vertices = [f.vertices[0], f.vertices[3], f.vertices[2], f.vertices[1]];
                                f.uv = [f.uv[0], f.uv[3], f.uv[2], f.uv[1]];
                                f.vertex_colors = f.vertex_colors.map(|c| [c[0], c[3], c[2], c[1]]);
                            }
                        }
                    }
                    Ok(())
                });
                report(e, result);
            }
            ui.text_wrapped("Reverses the winding of the selected faces. The PSX renderer rejects backfaces, so flip faces that should be seen from the other side.");
        }
        ModelTool::Delete => {
            if button(ui, "Delete selected faces") {
                let selected = e.mesh_editor.selected.clone();
                let result = edit(e, |d| {
                    d.faces.retain(|f| !selected.contains(&f.id));
                    d.compact();
                    Ok(())
                });
                e.mesh_editor.vertices.clear();
                report(e, result);
            }
            ui.text_wrapped("Removes the selected faces and any vertex left unused.");
        }
        ModelTool::Extract => {
            if button(ui, "Extract to child entity") {
                let result = extract(e);
                report(e, result);
            }
            ui.text_wrapped("Moves the selected faces into a new mesh on a child actor.");
        }
    }
}
/// Split faces whose edges exceed a length, along whole edge loops so no crack
/// opens. Smaller faces bend textures less under the PSX's affine mapping; the
/// build derives chunks, pages and budgets from the result as from any edit.
fn texture_correction(ui: &Ui, e: &mut Editor, doc: &Document) {
    crate::gui::muted(ui, "Split large faces so textures bend less");
    crate::gui::Drag::new(crate::gui::field(ui, "Max edge length"))
        .speed(0.05)
        .range(crate::mesh_subdivide::MIN_EDGE, 64.)
        .display_format("%.2f")
        .build(ui, &mut e.mesh_editor.max_edge);
    if ui.is_item_hovered() {
        ui.tooltip_text("Local units of this mesh; the actor's scale multiplies them.
Smaller faces bend textures less on PlayStation, at the cost of more triangles.");
    }
    let max_edge = e
        .mesh_editor
        .max_edge
        .clamp(crate::mesh_subdivide::MIN_EDGE, 64.);
    let state = &e.mesh_editor;
    let key = (
        state.revision,
        max_edge.to_bits(),
        state.hidden.len(),
        state.locked.len(),
        state.isolate,
    );
    let plan = match state.subdivide_preview {
        Some((cached, plan)) if cached == key => plan,
        _ => {
            let plan = crate::mesh_subdivide::plan(
                doc,
                &state.selected,
                &protected_faces(state, doc),
                max_edge,
            );
            e.mesh_editor.subdivide_preview = Some((key, plan));
            plan
        }
    };
    let scope = if e.mesh_editor.selected.is_empty() {
        "the whole mesh"
    } else {
        "the selected faces"
    };
    if plan.split == 0 {
        ui.text_disabled(format!(
            "No edge of {scope} is longer than {max_edge:.2} units."
        ));
    } else {
        ui.text_wrapped(format!(
            "Splits {} faces of {scope}: {} -> {} faces (about +{} PSX triangles).",
            plan.split,
            plan.faces_before,
            plan.faces_after,
            plan.added_faces() * 2
        ));
        if plan.protected > 0 {
            ui.text_colored(
                [0.95, 0.62, 0.35, 1.],
                format!(
                    "Reaches {} hidden or locked faces: show and unlock them first.",
                    plan.protected
                ),
            );
        }
    }
    if button(ui, "Subdivide large faces") {
        let scope = e.mesh_editor.selected.clone();
        let protected = protected_faces(&e.mesh_editor, doc);
        let result = edit(e, |d| {
            crate::mesh_subdivide::apply(d, &scope, &protected, max_edge).map(|_| ())
        });
        report(e, result);
    }
    ui.text_wrapped("A selection extends along its edge loops so neighbouring faces stay joined. After splitting a mesh that a camera track covers, bake visibility again; check the triangle budget in Project Settings.");
}
/// Faces in hidden or locked groups, which no bulk operation may change.
fn protected_faces(state: &State, doc: &Document) -> BTreeSet<Uuid> {
    doc.faces
        .iter()
        .filter(|f| !state.visible(doc, f.group) || !state.unlocked(doc, f.group))
        .map(|f| f.id)
        .collect()
}
fn groups_panel(ui: &Ui, e: &mut Editor, doc: &Document) {
    ui.input_text(crate::gui::field(ui, "Name"), &mut e.mesh_editor.name)
        .build();
    let root = button(ui, "New root group");
    ui.same_line();
    let child = button(ui, "New subgroup");
    if root || child {
        let name = e.mesh_editor.name.clone();
        let parent = if child { e.mesh_editor.group } else { None };
        let id = Uuid::new_v4();
        let result = edit(e, |d| {
            d.groups.push(mesh::Group { id, name, parent });
            Ok(())
        });
        if result.is_ok() {
            e.mesh_editor.group = Some(id);
        }
        report(e, result);
    }
    for g in &doc.groups {
        let _id = ui.push_id(g.id.to_string());
        let mut depth = 0;
        let mut p = g.parent;
        while let Some(id) = p {
            depth += 1;
            p = doc
                .groups
                .iter()
                .find(|g| g.id == id)
                .and_then(|g| g.parent);
        }
        if ui
            .selectable_config(format!("{}{}", "  ".repeat(depth), g.name))
            .selected(e.mesh_editor.group == Some(g.id))
            .build()
        {
            e.mesh_editor.group = Some(g.id);
            e.mesh_editor.name = g.name.clone();
        }
    }
    let Some(group) = e.mesh_editor.group else {
        return;
    };
    if button(ui, "Select group faces") {
        set_mode(e, Mode::Faces);
        e.mesh_editor.selected = doc
            .faces
            .iter()
            .filter(|f| {
                doc.contains_group(f.group, group)
                    && e.mesh_editor.visible(doc, f.group)
                    && e.mesh_editor.unlocked(doc, f.group)
            })
            .map(|f| f.id)
            .collect();
        e.mesh_editor.revision += 1;
    }
    crate::gui::inline(ui, "Delete empty group");
    if button(ui, "Delete empty group") {
        let result = edit(e, |d| {
            if d.groups.len() == 1
                || d.faces.iter().any(|f| f.group == group)
                || d.groups.iter().any(|g| g.parent == Some(group))
            {
                return Err(
                    "Only empty groups without subgroups can be deleted; keep at least one group"
                        .into(),
                );
            }
            d.groups.retain(|g| g.id != group);
            Ok(())
        });
        report(e, result);
    }
    if button(ui, "Rename group") {
        let name = e.mesh_editor.name.clone();
        let result = edit(e, |d| {
            d.groups
                .iter_mut()
                .find(|g| g.id == group)
                .ok_or("Group removed")?
                .name = name;
            Ok(())
        });
        report(e, result);
    }
    if let Some(_combo) = ui.begin_combo(
        crate::gui::field(ui, "Parent"),
        doc.groups
            .iter()
            .find(|g| g.id == group)
            .and_then(|g| g.parent)
            .and_then(|id| doc.groups.iter().find(|g| g.id == id))
            .map_or("Scene-independent root", |g| g.name.as_str()),
    ) {
        let mut request = None;
        if ui.selectable("None") {
            request = Some(None);
        }
        for g in &doc.groups {
            if !doc.contains_group(g.id, group) && ui.selectable(&g.name) {
                request = Some(Some(g.id));
            }
        }
        if let Some(parent) = request {
            let result = edit(e, |d| {
                d.groups
                    .iter_mut()
                    .find(|g| g.id == group)
                    .ok_or("Group removed")?
                    .parent = parent;
                Ok(())
            });
            report(e, result);
        }
    }
    if button(ui, "Move selected faces into group") {
        let selected = e.mesh_editor.selected.clone();
        let result = edit(e, |d| {
            for f in &mut d.faces {
                if selected.contains(&f.id) {
                    f.group = group;
                }
            }
            Ok(())
        });
        report(e, result);
    }
    let mut hidden = e.mesh_editor.hidden.contains(&group);
    let mut locked = e.mesh_editor.locked.contains(&group);
    let mut isolate = e.mesh_editor.isolate == Some(group);
    if ui.checkbox("Hide in editor", &mut hidden) {
        if hidden {
            e.mesh_editor.hidden.insert(group);
        } else {
            e.mesh_editor.hidden.remove(&group);
        }
        preview(e);
    }
    crate::gui::inline(ui, "Lock selection");
    if ui.checkbox("Lock selection", &mut locked) {
        if locked {
            e.mesh_editor.locked.insert(group);
            e.mesh_editor.selected.retain(|id| {
                doc.faces
                    .iter()
                    .find(|f| f.id == *id)
                    .is_none_or(|f| !doc.contains_group(f.group, group))
            });
        } else {
            e.mesh_editor.locked.remove(&group);
        }
    }
    if ui.checkbox("Isolate in editor", &mut isolate) {
        e.mesh_editor.isolate = isolate.then_some(group);
        preview(e);
    }
}
fn materials_panel(ui: &Ui, e: &mut Editor, doc: &Document) {
    ui.input_text(crate::gui::field(ui, "Slot name"), &mut e.mesh_editor.name)
        .build();
    if button(ui, "New material slot") {
        let id = Uuid::new_v4();
        let name = e.mesh_editor.name.clone();
        let result = edit(e, |d| {
            d.materials.push(mesh::Slot {
                id,
                name,
                material: Default::default(),
            });
            Ok(())
        });
        if result.is_ok() {
            e.mesh_editor.slot = Some(id);
        }
        report(e, result);
    }
    for slot in &doc.materials {
        if ui
            .selectable_config(&slot.name)
            .selected(e.mesh_editor.slot == Some(slot.id))
            .build()
        {
            e.mesh_editor.slot = Some(slot.id);
            e.mesh_editor.name = slot.name.clone();
        }
    }
    let Some(id) = e.mesh_editor.slot else {
        return;
    };
    let Some(slot) = doc.materials.iter().find(|s| s.id == id) else {
        return;
    };
    let mut mat = slot.material.clone();
    if ui.color_edit3(crate::gui::field(ui, "Default color"), &mut mat.color)
        | ui.checkbox("Default unlit", &mut mat.unlit)
    {
        let result = edit(e, |d| {
            d.materials
                .iter_mut()
                .find(|s| s.id == id)
                .unwrap()
                .material = mat;
            Ok(())
        });
        report(e, result);
    }
    if button(ui, "Rename slot") {
        let name = e.mesh_editor.name.clone();
        let result = edit(e, |d| {
            d.materials.iter_mut().find(|s| s.id == id).unwrap().name = name;
            Ok(())
        });
        report(e, result);
    }
    crate::gui::inline(ui, "Move slot up");
    if button(ui, "Move slot up") {
        let result = edit(e, |d| {
            let i = d.materials.iter().position(|s| s.id == id).unwrap();
            if i > 0 {
                d.materials.swap(i, i - 1);
            }
            Ok(())
        });
        report(e, result);
    }
    if button(ui, "Assign to selected faces") {
        let selected = e.mesh_editor.selected.clone();
        let result = edit(e, |d| {
            for f in &mut d.faces {
                if selected.contains(&f.id) {
                    f.material = id;
                }
            }
            Ok(())
        });
        report(e, result);
    }
    if button(ui, "Assign to current group")
        && let Some(group) = e.mesh_editor.group
    {
        let eligible = doc
            .faces
            .iter()
            .filter(|f| e.mesh_editor.visible(doc, f.group) && e.mesh_editor.unlocked(doc, f.group))
            .map(|f| f.id)
            .collect::<BTreeSet<_>>();
        let result = edit(e, |d| {
            let selected = d
                .faces
                .iter()
                .filter(|f| d.contains_group(f.group, group) && eligible.contains(&f.id))
                .map(|f| f.id)
                .collect::<BTreeSet<_>>();
            for f in &mut d.faces {
                if selected.contains(&f.id) {
                    f.material = id;
                }
            }
            Ok(())
        });
        report(e, result);
    }
    if button(ui, "Select faces using material") {
        select_material_faces(e, doc, id);
    }
}
/// Select every visible, unlocked face using material slot `slot`.
fn select_material_faces(e: &mut Editor, doc: &Document, slot: Uuid) {
    set_mode(e, Mode::Faces);
    e.mesh_editor.selected = doc
        .faces
        .iter()
        .filter(|f| {
            f.material == slot
                && e.mesh_editor.visible(doc, f.group)
                && e.mesh_editor.unlocked(doc, f.group)
        })
        .map(|f| f.id)
        .collect();
    e.mesh_editor.revision += 1;
    e.view_dirty = true;
}
/// Enter geometry editing for the actor named `actor`, optionally selecting
/// the faces of material slot `material`, and frame it (screenshots).
pub fn edit_actor_named(e: &mut Editor, actor: &str, material: Option<&str>) -> Result<(), String> {
    let index = e
        .scene
        .actors
        .iter()
        .position(|a| a.name == actor)
        .ok_or_else(|| format!("No actor named {actor}"))?;
    let asset = e.scene.actors[index]
        .editable_mesh
        .as_ref()
        .map(|m| m.asset)
        .ok_or_else(|| format!("{actor} has no editable mesh"))?;
    let record = e.assets.index.resolve(asset)?.clone();
    e.select_actor(Some(e.scene.actors[index].id));
    e.action("frame-selected");
    open(e, record, Some(index));
    if let Some(material) = material {
        let doc = e.mesh_editor.doc.clone().ok_or("No mesh open")?;
        let slot = doc
            .materials
            .iter()
            .find(|m| m.name == material)
            .ok_or_else(|| format!("No material slot named {material}"))?
            .id;
        select_material_faces(e, &doc, slot);
    }
    Ok(())
}

#[cfg(test)]
pub fn verify_interactions(context: &mut imgui::Context) {
    let root = std::env::temp_dir().join(format!("epok-blockout-ui-{}", Uuid::new_v4()));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    let mut e = Editor::new(root.clone());
    e.auto_build = false;
    allocate_actor_data(&mut e);
    fn frame(context: &mut imgui::Context, e: &mut Editor, panel: usize) {
        let ui = context.frame();
        ui.window("Blockout interaction")
            .position([10., 10.], Condition::Always)
            .size([700., 800.], Condition::Always)
            .build(|| {
                let doc = e.mesh_editor.doc.clone().unwrap();
                match panel {
                    0 => {
                        let record = e.mesh_editor.record.clone().unwrap();
                        tools(ui, e, &doc, &record)
                    }
                    1 => groups_panel(ui, e, &doc),
                    _ => materials_panel(ui, e, &doc),
                }
                geometry_shortcuts(ui, e);
            });
        context.render();
    }
    fn click(context: &mut imgui::Context, e: &mut Editor, panel: usize, label: &str) {
        frame(context, e, panel);
        frame(context, e, panel);
        let p = BUTTONS.with(|buttons| buttons.borrow()[label]);
        context.io_mut().add_mouse_pos_event(p);
        frame(context, e, panel);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(context, e, panel);
        context
            .io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(context, e, panel);
        assert!(
            e.mesh_editor.error.is_none(),
            "{}: {:?}",
            label,
            e.mesh_editor.error
        );
    }
    e.mesh_editor.origin = [3., 0., 0.];
    e.mesh_editor.category = Category::Create;
    click(context, &mut e, 0, "Add shape");
    assert_eq!(e.mesh_editor.doc.as_ref().unwrap().faces.len(), 12);
    e.mesh_editor.category = Category::Select;
    click(context, &mut e, 0, "Select all visible");
    assert_eq!(e.mesh_editor.selected.len(), 12);
    e.mesh_editor.name = "Walls".into();
    click(context, &mut e, 1, "New root group");
    let group = e.mesh_editor.group.unwrap();
    assert_eq!(
        e.mesh_editor.selected.len(),
        12,
        "Changing destination group must preserve face selection"
    );
    click(context, &mut e, 1, "Move selected faces into group");
    assert!(
        e.mesh_editor
            .doc
            .as_ref()
            .unwrap()
            .faces
            .iter()
            .all(|f| f.group == group)
    );
    e.mesh_editor.name = "Brick".into();
    click(context, &mut e, 2, "New material slot");
    let slot = e.mesh_editor.slot.unwrap();
    click(context, &mut e, 2, "Assign to selected faces");
    assert!(
        e.mesh_editor
            .doc
            .as_ref()
            .unwrap()
            .faces
            .iter()
            .all(|f| f.material == slot)
    );
    e.mesh_editor.category = Category::Model;
    e.mesh_editor.model_tool = ModelTool::Subdivide;
    click(context, &mut e, 0, "Subdivide faces");
    assert_eq!(e.mesh_editor.doc.as_ref().unwrap().faces.len(), 48);
    undo(&mut e, false).unwrap();
    assert_eq!(e.mesh_editor.doc.as_ref().unwrap().faces.len(), 12);
    // Texture correction: with nothing selected the whole mesh is split, every
    // unit edge in two, and one Undo restores it.
    let before = e.mesh_editor.doc.clone().unwrap();
    e.mesh_editor.selected.clear();
    e.mesh_editor.max_edge = 0.5;
    e.mesh_editor.category = Category::Texture;
    click(context, &mut e, 0, "Subdivide large faces");
    let split = e.mesh_editor.doc.clone().unwrap();
    assert_eq!(split.faces.len(), 48, "two unit boxes, every face cut 2 x 2");
    split.validate().unwrap();
    frame(context, &mut e, 0);
    assert!(
        e.mesh_editor
            .subdivide_preview
            .is_some_and(|(_, plan)| plan.split == 0),
        "the preview follows the split mesh"
    );
    undo(&mut e, false).unwrap();
    assert_eq!(e.mesh_editor.doc.as_ref(), Some(&before));
    e.mesh_editor.selected.clear();
    let pixel = crate::viewport::project(&e.view, [0., 0., 0.]);
    assert!(pick(&mut e, [pixel[0], pixel[1]], SelectOp::Replace));
    assert_eq!(e.mesh_editor.selected.len(), 1);
    let original = e.mesh_editor.doc.clone().unwrap();
    context.io_mut().add_key_event(imgui::Key::E, true);
    frame(context, &mut e, 0);
    context.io_mut().add_key_event(imgui::Key::E, false);
    frame(context, &mut e, 0);
    assert_eq!(
        e.mesh_editor.doc.as_ref().unwrap().faces.len(),
        original.faces.len() + 4
    );
    undo(&mut e, false).unwrap();
    assert_eq!(e.mesh_editor.doc.as_ref(), Some(&original));
    let id = *e.mesh_editor.selected.first().unwrap();
    let original_face = original.faces.iter().find(|f| f.id == id).unwrap();
    let points = original.points(original_face);
    let normal = mesh::face_normal(points);
    context.io_mut().add_key_event(imgui::Key::Q, true);
    frame(context, &mut e, 0);
    context.io_mut().add_key_event(imgui::Key::Q, false);
    frame(context, &mut e, 0);
    let after = e.mesh_editor.doc.as_ref().unwrap();
    let face = after.faces.iter().find(|f| f.id == id).unwrap();
    assert!(
        (crate::lighting::dot(
            crate::lighting::sub(after.points(face)[0], points[0]),
            normal
        ) + 0.25)
            .abs()
            < 0.001
    );
    undo(&mut e, false).unwrap();
    e.scene_navigation = true;
    context.io_mut().add_key_event(imgui::Key::E, true);
    frame(context, &mut e, 0);
    context.io_mut().add_key_event(imgui::Key::E, false);
    frame(context, &mut e, 0);
    assert_eq!(
        e.mesh_editor.doc.as_ref(),
        Some(&original),
        "Camera QE must not edit geometry"
    );
    e.scene_navigation = false;
    context.io_mut().add_key_event(imgui::Key::Alpha2, true);
    frame(context, &mut e, 0);
    context.io_mut().add_key_event(imgui::Key::Alpha2, false);
    frame(context, &mut e, 0);
    assert!(e.mesh_editor.mode == Mode::Edges);
    let pixel = crate::viewport::project(&e.view, [0., 0.5, -0.5]);
    pick(&mut e, [pixel[0], pixel[1]], SelectOp::Replace);
    assert_eq!(
        e.mesh_editor.edges.len(),
        1,
        "Visible authored edges must be selectable"
    );
    context.io_mut().add_key_event(imgui::Key::Q, true);
    frame(context, &mut e, 0);
    context.io_mut().add_key_event(imgui::Key::Q, false);
    frame(context, &mut e, 0);
    assert!(e.mesh_editor.error.is_none(), "{:?}", e.mesh_editor.error);
    assert!(e.mesh_editor.doc.as_ref().unwrap().faces.iter().any(|f| {
        let n = mesh::face_normal(e.mesh_editor.doc.as_ref().unwrap().points(f));
        n[1] > 0.6 && n[2] < -0.6
    }));
    undo(&mut e, false).unwrap();
    assert_eq!(e.mesh_editor.doc.as_ref(), Some(&original));
    assert_eq!(
        mesh::document(e.mesh_editor.record.as_ref().unwrap()).unwrap(),
        e.mesh_editor.doc.clone().unwrap()
    );
    drop(e);
    std::fs::remove_dir_all(root).unwrap();
}

/// Height brush for blockout geometry, centred on the selection.
///
/// It is the terrain brush applied to authored vertices: same falloff, same
/// modes, same arithmetic. A quad the displacement bends is split into two
/// triangles, because a blockout face has to stay planar.
fn sculpt_controls(ui: &Ui, e: &mut Editor, doc: &Document) {
    ui.separator();
    crate::gui::muted(ui, "Sculpt brush");
    let mut mode = crate::brush::Mode::SCULPT
        .iter()
        .position(|m| *m == e.mesh_editor.brush.mode)
        .unwrap_or(0);
    let labels: Vec<&str> = crate::brush::Mode::SCULPT
        .iter()
        .map(|m| m.label())
        .collect();
    if ui.combo_simple_string(crate::gui::field(ui, "Sculpt"), &mut mode, &labels) {
        e.mesh_editor.brush.mode = crate::brush::Mode::SCULPT[mode];
    }
    let mut falloff = crate::brush::Falloff::ALL
        .iter()
        .position(|f| *f == e.mesh_editor.brush.falloff)
        .unwrap_or(0);
    let falloffs: Vec<&str> = crate::brush::Falloff::ALL
        .iter()
        .map(|f| f.label())
        .collect();
    if ui.combo_simple_string(
        crate::gui::field(ui, "Sculpt falloff"),
        &mut falloff,
        &falloffs,
    ) {
        e.mesh_editor.brush.falloff = crate::brush::Falloff::ALL[falloff];
    }
    crate::gui::Drag::new(crate::gui::field(ui, "Sculpt radius"))
        .speed(0.05)
        .range(crate::brush::MIN_RADIUS, crate::brush::MAX_RADIUS)
        .build(ui, &mut e.mesh_editor.brush.radius);
    crate::gui::Drag::new(crate::gui::field(ui, "Sculpt strength"))
        .speed(0.01)
        .range(0., crate::brush::MAX_STRENGTH)
        .build(ui, &mut e.mesh_editor.brush.strength);
    let selection = selected_vertices(e, doc);
    let center = brush_center(doc, &selection);
    if button(ui, "Sculpt selection") {
        let brush = e.mesh_editor.brush;
        let result = edit(e, move |doc| {
            crate::mesh_ops::sculpt(doc, &brush, center, &selection)
        });
        report(e, result);
    }
    ui.text_wrapped(
        "Displaces vertices along Y under the brush, centred on the selection or on the whole mesh when nothing is selected. Bent quads become triangle pairs. Use the Terrain tool for a heightmap you can paint and stroke in the viewport.",
    );
}

/// Vertices the sculpt applies to: the explicit vertex selection, or the
/// vertices of the selected faces, or everything when nothing is selected.
fn selected_vertices(e: &Editor, doc: &Document) -> BTreeSet<u32> {
    if !e.mesh_editor.vertices.is_empty() {
        return e.mesh_editor.vertices.clone();
    }
    if !e.mesh_editor.selected.is_empty() {
        return doc.selected_vertices(&e.mesh_editor.selected);
    }
    BTreeSet::new()
}

fn brush_center(doc: &Document, selection: &BTreeSet<u32>) -> [f32; 3] {
    let points: Vec<[f32; 3]> = if selection.is_empty() {
        doc.vertices.clone()
    } else {
        selection
            .iter()
            .filter_map(|v| doc.vertices.get(*v as usize).copied())
            .collect()
    };
    if points.is_empty() {
        return [0.; 3];
    }
    std::array::from_fn(|c| points.iter().map(|p| p[c]).sum::<f32>() / points.len() as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inspector_mesh_selection_preserves_shared_assets_and_instance_identity() {
        let root = std::env::temp_dir().join(format!("epok-mesh-picker-{}", Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets/Meshes")).unwrap();
        let path = root.join("assets/Meshes/Ramp.epokasset");
        let asset = mesh::create(
            &root,
            "assets/Meshes/Ramp.epokasset",
            &mesh::tests::shape("Ramp"),
        )
        .unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let mut first = Actor::cube("First".into());
        let mut second = Actor::cube("Second".into());
        let identity = (first.id, first.class.clone());
        assign_mesh(&mut first, &index, Some(asset)).unwrap();
        assign_mesh(&mut second, &index, Some(asset)).unwrap();
        assert_eq!((first.id, first.class.clone()), identity);
        let before = first.clone();
        assert!(assign_mesh(&mut first, &index, Some(Uuid::new_v4())).is_err());
        assert_eq!(first, before);
        let shared = second.clone();
        assign_mesh(&mut first, &index, None).unwrap();
        assert_eq!(first.kind, "Mesh");
        assert!(first.editable_mesh.is_none() && first.skeletal_mesh.is_none());
        assert_eq!(second, shared);
        assert_eq!(
            std::fs::read(&path).unwrap(),
            bytes,
            "Selecting a mesh must never edit its asset"
        );
        let scene = Scene {
            actors: vec![first, second],
            ..Default::default()
        };
        let restored: Scene =
            serde_json::from_str(&serde_json::to_string(&scene).unwrap()).unwrap();
        assert!(restored.actors[0].editable_mesh.is_none());
        assert_eq!(
            restored.actors[1].editable_mesh.as_ref().unwrap().asset,
            asset
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn autosave_undo_shared_edit_extraction_and_guard() {
        let root = std::env::temp_dir().join(format!("epok-blockout-{}", Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let mut e = Editor::new(root.clone());
        e.auto_build = false;
        allocate_actor_data(&mut e);
        assert!(e.mesh_editor.open);
        let target = e.mesh_editor.target.unwrap();
        let original = e.mesh_editor.doc.clone().unwrap();
        let mut shared = e.scene.actors[target].clone();
        shared.id = uuid::Uuid::new_v4();
        e.scene.actors.push(shared);
        edit(&mut e, |d| {
            d.groups[0].name = "Building".into();
            Ok(())
        })
        .unwrap();
        assert_eq!(
            e.scene
                .actors
                .last()
                .unwrap()
                .editable_mesh
                .as_ref()
                .unwrap()
                .document
                .as_ref()
                .unwrap()
                .groups[0]
                .name,
            "Building"
        );
        undo(&mut e, false).unwrap();
        assert_eq!(e.mesh_editor.doc.as_ref().unwrap(), &original);
        undo(&mut e, true).unwrap();
        e.mesh_editor.selected.insert(original.faces[0].id);
        assert!(extract(&mut e).unwrap_err().contains("independent"));
        e.scene.actors.pop();
        let other_scene = root.join("assets/scenes/Other.epokmap");
        e.scene.save(&other_scene).unwrap();
        assert!(extract(&mut e).unwrap_err().contains("other saved scenes"));
        std::fs::remove_file(other_scene).unwrap();
        let count = e.scene.actors.len();
        extract(&mut e).unwrap();
        assert_eq!(e.scene.actors.len(), count + 1);
        assert_eq!(e.scene.actors.last().unwrap().parent, Some(target));
        assert_eq!(e.mesh_editor.doc.as_ref().unwrap().faces.len(), 5);
        undo(&mut e, false).unwrap();
        assert_eq!(e.scene.actors.len(), count);
        assert_eq!(e.mesh_editor.doc.as_ref().unwrap().faces.len(), 6);
        undo(&mut e, true).unwrap();
        e.scene.actors[target].name = "Changed after extraction".into();
        assert!(undo(&mut e, false).unwrap_err().contains("scene changed"));
        let persisted = mesh::document(e.mesh_editor.record.as_ref().unwrap()).unwrap();
        assert_eq!(persisted, e.mesh_editor.doc.clone().unwrap());
        let group = persisted.groups[0].id;
        e.mesh_editor.hidden.insert(group);
        preview(&mut e);
        assert!(
            e.scene.actors[target]
                .editable_mesh
                .as_ref()
                .unwrap()
                .document
                .as_ref()
                .unwrap()
                .faces
                .is_empty()
        );
        assert_eq!(
            mesh::document(e.mesh_editor.record.as_ref().unwrap()).unwrap(),
            persisted
        );
        e.mesh_editor.open = false;
        preview(&mut e);
        assert_eq!(
            e.scene.actors[target]
                .editable_mesh
                .as_ref()
                .unwrap()
                .document
                .as_deref(),
            Some(&persisted)
        );
        e.playing = true;
        assert!(undo(&mut e, false).is_err());
        assert!(edit(&mut e, |_| Ok(())).is_err());
        e.playing = false;
        let mut external = persisted.clone();
        external.groups[0].name = "External change".into();
        mesh::save(e.mesh_editor.record.as_ref().unwrap(), &external).unwrap();
        e.assets.index = assets::scan(&root, &mut Default::default());
        synchronize(&mut e);
        assert_eq!(e.mesh_editor.doc.as_ref(), Some(&external));
        assert!(e.mesh_editor.undo.is_empty());
        drop(e);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod session_tests {
    use super::*;

    fn editor() -> (std::path::PathBuf, Editor) {
        let root = std::env::temp_dir().join(format!("epok-mesh-session-{}", Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let mut e = Editor::new(root.clone());
        e.auto_build = false;
        allocate_actor_data(&mut e);
        assert!(e.mesh_editor.open, "creating a blockout opens its geometry");
        (root, e)
    }

    #[test]
    fn a_session_reverts_tracks_hover_and_ends_when_another_actor_is_chosen() {
        let (root, mut e) = editor();
        let target = target_actor(&e).expect("the blockout is the edited actor");
        let start = e.mesh_editor.doc.clone().unwrap();
        assert!(!changed_in_session(&e));
        edit(&mut e, |d| {
            crate::mesh_subdivide::apply(d, &BTreeSet::new(), &BTreeSet::new(), 0.5).map(|_| ())
        })
        .unwrap();
        assert!(changed_in_session(&e));
        revert(&mut e).unwrap();
        assert_eq!(e.mesh_editor.doc.as_ref(), Some(&start), "Revert restores the start");
        undo(&mut e, false).unwrap();
        assert!(changed_in_session(&e), "Revert is one Undo step");

        // The face under the cursor is tracked for hover feedback.
        let pixel = crate::viewport::project(&e.view, [0., 0., 0.]);
        hover(&mut e, Some([pixel[0], pixel[1]]));
        assert!(matches!(e.mesh_editor.hover, Some(Hit::Face(_))), "{:?}", e.mesh_editor.hover);
        hover(&mut e, None);
        assert_eq!(e.mesh_editor.hover, None);

        // Choosing another actor leaves geometry editing for that actor.
        let mut other = Actor::cube("Other".into());
        other.position = [5., 0., 0.];
        let other_id = other.id;
        e.scene.actors.push(other);
        let revision = e.mesh_editor.revision;
        e.select_actor(Some(other_id));
        assert!(!e.mesh_editor.open);
        assert_eq!(e.selected_actor, Some(other_id));
        assert!(e.mesh_editor.revision > revision, "the Scene view redraws without the mode");

        // Done (here: closing) keeps the edited actor selected.
        let record = e.mesh_editor.record.clone().unwrap();
        let index = e.scene.actor_index(target);
        open(&mut e, record, index);
        assert!(e.mesh_editor.open && e.mesh_editor.session_start.is_some());
        close(&mut e);
        assert!(!e.mesh_editor.open);
        assert_eq!(e.selected_actor, Some(target));
        drop(e);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn the_geometry_panel_switches_categories_tools_and_ends_with_done() {
        let (root, mut e) = editor();
        let mut ctx = crate::gui::tests::imgui_context();
        crate::gui::configure_input(ctx.io_mut());
        ctx.io_mut().display_size = [1280., 800.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        ctx.fonts().build_rgba32_texture();
        let frame = |ctx: &mut imgui::Context, e: &mut Editor| {
            BUTTONS.with(|b| b.borrow_mut().clear());
            let ui = ctx.frame();
            window(ui, e);
            // As at the end of `gui::draw`.
            redraw_on_change(e);
            ctx.render();
        };
        let click = |ctx: &mut imgui::Context, e: &mut Editor, label: &str| {
            frame(ctx, e);
            frame(ctx, e);
            let p = BUTTONS.with(|b| {
                b.borrow().get(label).copied().unwrap_or_else(|| {
                    panic!("{label} not drawn; drawn: {:?}", b.borrow().keys().collect::<Vec<_>>())
                })
            });
            ctx.io_mut().add_mouse_pos_event(p);
            frame(ctx, e);
            ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(ctx, e);
            ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(ctx, e);
        };
        assert_eq!(e.mesh_editor.category, Category::Select, "selection first");
        // A Select tile redraws the Scene view at once, not when the cursor
        // next moves over it.
        frame(&mut ctx, &mut e);
        e.view_dirty = false;
        click(&mut ctx, &mut e, "Select all visible");
        assert_eq!(e.mesh_editor.selected.len(), 6);
        assert!(e.view_dirty, "the new selection is drawn in the same frame");
        click(&mut ctx, &mut e, "Model");
        assert_eq!(e.mesh_editor.category, Category::Model);
        assert!(BUTTONS.with(|b| !b.borrow().contains_key("Flip faces")));
        click(&mut ctx, &mut e, "Flip face winding");
        assert_eq!(e.mesh_editor.model_tool, ModelTool::Flip, "tiles pick the tool");
        frame(&mut ctx, &mut e);
        assert!(
            BUTTONS.with(|b| b.borrow().contains_key("Flip faces")),
            "the chosen tool shows its action"
        );
        click(&mut ctx, &mut e, "Edges");
        assert!(e.mesh_editor.mode == Mode::Edges, "the header switches the element mode");
        click(&mut ctx, &mut e, "Done editing");
        assert!(!e.mesh_editor.open, "Done leaves geometry editing");
        drop(ctx);
        drop(e);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn the_scene_banner_shows_the_mode_and_done_or_escape_leave_it() {
        let (root, mut e) = editor();
        let mut ctx = crate::gui::tests::imgui_context();
        crate::gui::configure_input(ctx.io_mut());
        ctx.io_mut().display_size = [1280., 720.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        ctx.fonts().build_rgba32_texture();
        let frame = |ctx: &mut imgui::Context, e: &mut Editor| {
            let ui = ctx.frame();
            ui.window("Scene banner")
                .position([0., 0.], Condition::Always)
                .size([1280., 720.], Condition::Always)
                .build(|| banner(ui, e, [0., 0.], [1280., 700.]));
            shortcuts(ui, e);
            ctx.render();
        };
        let click = |ctx: &mut imgui::Context, e: &mut Editor, label: &str| {
            frame(ctx, e);
            frame(ctx, e);
            let p = BUTTONS.with(|b| b.borrow()[label]);
            ctx.io_mut().add_mouse_pos_event(p);
            frame(ctx, e);
            ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(ctx, e);
            ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(ctx, e);
        };
        let rect = banner_rect(ctx.frame(), &e, [0., 0.], [1280., 700.]).unwrap();
        ctx.render();
        assert!(rect[0][1] > 600., "at the bottom of the Scene view: {rect:?}");
        assert!(rect[1][0] <= 1280. - 8., "inside the Scene view: {rect:?}");
        // Narrow panels drop the name, then the title, then wrap to two rows,
        // always keeping Done reachable.
        let ui = ctx.frame();
        let wide = banner_layout(ui, &e, 1280.);
        let medium = banner_layout(ui, &e, 640.);
        let narrow = banner_layout(ui, &e, 380.);
        ctx.render();
        assert!(wide.title && !wide.two_rows);
        assert!(!medium.name && medium.size[0] <= 640. - 24.);
        assert!(narrow.two_rows && narrow.size[0] <= 380. - 24. + 0.5, "{narrow:?}");
        click(&mut ctx, &mut e, "Edges");
        assert!(e.mesh_editor.mode == Mode::Edges, "mode buttons switch the element mode");
        click(&mut ctx, &mut e, "Done (Esc)");
        assert!(!e.mesh_editor.open, "Done leaves geometry editing");
        let (record, target) = (e.mesh_editor.record.clone().unwrap(), e.mesh_editor.target);
        open(&mut e, record, target);
        frame(&mut ctx, &mut e);
        ctx.io_mut().add_key_event(imgui::Key::Escape, true);
        frame(&mut ctx, &mut e);
        ctx.io_mut().add_key_event(imgui::Key::Escape, false);
        frame(&mut ctx, &mut e);
        assert!(!e.mesh_editor.open, "Escape leaves geometry editing");
        drop(ctx);
        drop(e);
        let _ = std::fs::remove_dir_all(root);
    }

    /// Release a box over `a`..`b` (`through`: Select through) with `op`.
    fn boxed(e: &mut Editor, a: [f32; 2], b: [f32; 2], through: bool, op: SelectOp) {
        let hits = box_hits(e, a, b, through);
        apply(e, &hits, op);
    }

    /// A face, its centre in Scene pixels, and whether it faces the camera.
    type ScreenFace = (Uuid, [f32; 2], bool);

    /// The blockout box seen from the default view: a Scene pixel rectangle
    /// around all of it, and its faces with their centres on screen, the
    /// faces turned toward the camera first.
    fn blockout_on_screen(e: &Editor) -> ([f32; 2], [f32; 2], Vec<ScreenFace>) {
        let (index, doc) = session(e).expect("the blockout is being edited");
        assert_eq!(doc.faces.len(), 6, "the blockout starts as a box");
        let world = e.scene.world_matrix(index);
        let project = |p: [f32; 3]| crate::viewport::project(&e.view, world.point(p));
        let (mut min, mut max) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
        for p in doc.vertices.iter().map(|v| project(*v)) {
            for c in 0..2 {
                min[c] = min[c].min(p[c] - 6.);
                max[c] = max[c].max(p[c] + 6.);
            }
        }
        let eye = e.view.unproject([480., 300.], 0.);
        let mut faces: Vec<_> = doc
            .faces
            .iter()
            .map(|f| {
                let points = doc.points(f);
                let world_points = points.map(|p| world.point(p));
                let toward = crate::lighting::sub(eye, world_points[0]);
                let facing = crate::lighting::dot(mesh::face_normal(world_points), toward) > 0.;
                let center: [f32; 3] = std::array::from_fn(|c| points.iter().map(|p| p[c]).sum::<f32>() / 4.);
                let p = project(center);
                (f.id, [p[0], p[1]], facing)
            })
            .collect();
        faces.sort_by_key(|f| !f.2);
        (min, max, faces)
    }

    #[test]
    fn a_box_selects_what_it_encloses_and_only_what_the_mesh_shows() {
        let (root, mut e) = editor();
        let (min, max, faces) = blockout_on_screen(&e);
        let facing: BTreeSet<Uuid> = faces.iter().filter(|f| f.2).map(|f| f.0).collect();
        assert_eq!(facing.len(), 3, "the default view looks at a corner of the box");

        let hits = box_hits(&e, min, max, false);
        apply(&mut e, &hits, SelectOp::Replace);
        assert_eq!(e.mesh_editor.selected, facing, "only the faces the camera sees");
        boxed(&mut e, min, max, true, SelectOp::Replace);
        assert_eq!(e.mesh_editor.selected.len(), 6, "Select through takes hidden faces too");
        boxed(&mut e, min, max, false, SelectOp::Remove);
        assert_eq!(e.mesh_editor.selected.len(), 3, "Ctrl removes what the box covers");
        assert!(e.mesh_editor.selected.is_disjoint(&facing));
        boxed(&mut e, min, max, false, SelectOp::Add);
        assert_eq!(e.mesh_editor.selected.len(), 6, "Shift adds to the selection");
        boxed(&mut e, [0., 0.], [3., 3.], false, SelectOp::Replace);
        assert!(e.mesh_editor.selected.is_empty(), "an empty box clears the selection");

        // A face counts by its centre: a box around one centre takes that face.
        let (one, center, _) = faces[0];
        let near = [center[0] - 2., center[1] - 2.];
        let far = [center[0] + 2., center[1] + 2.];
        boxed(&mut e, near, far, false, SelectOp::Replace);
        assert_eq!(e.mesh_editor.selected, BTreeSet::from([one]));

        set_mode(&mut e, Mode::Edges);
        boxed(&mut e, min, max, false, SelectOp::Replace);
        assert_eq!(e.mesh_editor.edges.len(), 9, "the edges of the three visible faces");
        boxed(&mut e, min, max, true, SelectOp::Replace);
        assert_eq!(e.mesh_editor.edges.len(), 12);
        set_mode(&mut e, Mode::Vertices);
        boxed(&mut e, min, max, false, SelectOp::Replace);
        assert_eq!(e.mesh_editor.vertices.len(), 7, "the far corner is hidden");
        boxed(&mut e, min, max, true, SelectOp::Replace);
        assert_eq!(e.mesh_editor.vertices.len(), 8);

        // Faces in hidden groups are never selected.
        set_mode(&mut e, Mode::Faces);
        let groups: Vec<Uuid> = e.mesh_editor.doc.as_ref().unwrap().faces.iter().map(|f| f.group).collect();
        e.mesh_editor.hidden.extend(groups);
        boxed(&mut e, min, max, true, SelectOp::Replace);
        assert!(e.mesh_editor.selected.is_empty(), "every face of the box is hidden");
        drop(e);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn painting_selects_what_the_cursor_crosses() {
        let (root, mut e) = editor();
        let (_, _, faces) = blockout_on_screen(&e);
        let facing: BTreeSet<Uuid> = faces.iter().filter(|f| f.2).map(|f| f.0).collect();
        let (a, from, _) = faces[0];
        let (b, to, _) = faces[1];
        paint(&mut e, from, to, SelectOp::Add, true);
        let selected = &e.mesh_editor.selected;
        assert!(selected.contains(&a) && selected.contains(&b), "both ends of the stroke");
        assert!(selected.is_subset(&facing), "only faces the cursor actually crossed");
        paint(&mut e, to, to, SelectOp::Remove, true);
        assert!(!e.mesh_editor.selected.contains(&b), "Ctrl paints the selection away");
        assert!(e.mesh_editor.selected.contains(&a));
        drop(e);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn left_drags_box_or_paint_and_escape_cancels_them() {
        let (root, mut e) = editor();
        let (min, max, faces) = blockout_on_screen(&e);
        let facing: BTreeSet<Uuid> = faces.iter().filter(|f| f.2).map(|f| f.0).collect();
        let mut ctx = crate::gui::tests::imgui_context();
        crate::gui::configure_input(ctx.io_mut());
        ctx.io_mut().display_size = [1280., 800.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        ctx.fonts().build_rgba32_texture();
        // The Scene image fills 960 x 600 at the origin, one pixel per Scene pixel.
        let frame = |ctx: &mut imgui::Context, e: &mut Editor| -> (bool, usize) {
            let ui = ctx.frame();
            let mut owned = false;
            ui.window("Scene drag")
                .position([0., 0.], Condition::Always)
                .size([1280., 800.], Condition::Always)
                .build(|| owned = drag(ui, e, [[0., 0.], [960., 600.]], |mouse| mouse, true));
            shortcuts(ui, e);
            let rects = ctx
                .render()
                .draw_lists()
                .map(|list| list.vtx_buffer().len())
                .sum();
            (owned, rects)
        };
        let mouse = |ctx: &mut imgui::Context, e: &mut Editor, at: [f32; 2]| {
            ctx.io_mut().add_mouse_pos_event(at);
            frame(ctx, e)
        };
        let button = |ctx: &mut imgui::Context, e: &mut Editor, down: bool| {
            ctx.io_mut().add_mouse_button_event(imgui::MouseButton::Left, down);
            frame(ctx, e)
        };

        // Box: press beside the mesh, drag across it, release.
        mouse(&mut ctx, &mut e, min);
        let (idle_owned, idle_vertices) = button(&mut ctx, &mut e, true);
        assert!(!idle_owned, "a press is still a click");
        let (owned, boxed_vertices) = mouse(&mut ctx, &mut e, max);
        assert!(owned && boxing(&e), "moving turns the press into a box");
        assert!(boxed_vertices > idle_vertices, "the box is drawn over the Scene view");
        assert!(button(&mut ctx, &mut e, false).0, "the release is not also a click");
        assert_eq!(e.mesh_editor.selected, facing);
        assert!(!boxing(&e));

        // Escape drops a box without leaving geometry editing.
        apply(&mut e, &[], SelectOp::Replace);
        mouse(&mut ctx, &mut e, min);
        button(&mut ctx, &mut e, true);
        mouse(&mut ctx, &mut e, max);
        ctx.io_mut().add_key_event(imgui::Key::Escape, true);
        frame(&mut ctx, &mut e);
        ctx.io_mut().add_key_event(imgui::Key::Escape, false);
        frame(&mut ctx, &mut e);
        assert!(e.mesh_editor.open, "Escape cancels the drag first");
        assert!(button(&mut ctx, &mut e, false).0);
        assert!(e.mesh_editor.selected.is_empty(), "a cancelled box selects nothing");

        // Shift adds a box to the selection.
        let (one, center, _) = faces[0];
        apply(&mut e, &[Hit::Face(one)], SelectOp::Replace);
        let hidden = faces.iter().find(|f| !f.2).unwrap().0;
        e.mesh_editor.select_occluded = true;
        ctx.io_mut().add_key_event(imgui::Key::ModShift, true);
        mouse(&mut ctx, &mut e, min);
        button(&mut ctx, &mut e, true);
        mouse(&mut ctx, &mut e, max);
        button(&mut ctx, &mut e, false);
        ctx.io_mut().add_key_event(imgui::Key::ModShift, false);
        frame(&mut ctx, &mut e);
        assert_eq!(e.mesh_editor.selected.len(), 6, "Select through reaches {hidden}");
        e.mesh_editor.select_occluded = false;

        // Moving less than the drag distance keeps a click a click.
        mouse(&mut ctx, &mut e, center);
        button(&mut ctx, &mut e, true);
        assert!(!mouse(&mut ctx, &mut e, [center[0] + 2., center[1]]).0);
        assert!(!button(&mut ctx, &mut e, false).0);
        assert_eq!(e.mesh_editor.selected.len(), 6, "clicks are the Scene view's to handle");

        // Paint: a plain stroke starts a new selection along its path.
        e.mesh_editor.drag_select = DragSelect::Paint;
        let (other, end, _) = faces[1];
        mouse(&mut ctx, &mut e, center);
        button(&mut ctx, &mut e, true);
        assert!(mouse(&mut ctx, &mut e, end).0, "painting owns the drag");
        assert!(!boxing(&e), "paint draws no box");
        button(&mut ctx, &mut e, false);
        let selected = &e.mesh_editor.selected;
        assert!(selected.contains(&one) && selected.contains(&other));
        assert!(selected.is_subset(&facing), "the old selection was replaced");
        drop(ctx);
        drop(e);
        let _ = std::fs::remove_dir_all(root);
    }
}
