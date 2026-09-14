use crate::{
    assets,
    editor::Editor,
    mesh::{self, Document},
    scene::{Actor, Scene},
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
            e.mesh_editor = State {
                open: true,
                group: doc.groups.first().map(|g| g.id),
                slot: doc.materials.first().map(|s| s.id),
                path: assets::path_string(&e.root, &record.path),
                record: Some(record),
                doc: Some(doc),
                target,
                ..Default::default()
            };
            preview(e);
        }
        Err(error) => e.log(error),
    }
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
    if e.playing {
        return;
    }
    if e.scene.actors.len() >= 512 {
        e.log("Scene entity limit: 512");
        return;
    }
    let mut entity = Actor::cube("Blockout".into());
    entity.position = [0.; 3];
    match create_component(e, &mut entity) {
        Ok(()) => {
            e.scene.actors.push(entity);
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
        }
        Err(error) => e.log(error),
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

pub fn filter(ui: &Ui, e: &mut Editor, entity: &mut Actor) {
    if !crate::gui::heading(ui, "Mesh Filter") {
        return;
    }
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
    ui.separator();
}

pub fn component(ui: &Ui, e: &mut Editor, entity: &mut Actor) {
    let Some(m) = &mut entity.editable_mesh else {
        return;
    };
    ui.separator();
    if !crate::gui::heading(ui, "Mesh Renderer") {
        return;
    }
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
    if button(ui, "Remove Editable Mesh") {
        entity.editable_mesh = None;
        entity.kind = "Empty".into();
    }
}
pub fn pick(e: &mut Editor, pixel: [f32; 2], extend: bool) -> bool {
    if !e.mesh_editor.open {
        return false;
    }
    let Some(index) = e.mesh_editor.target else {
        return false;
    };
    let Some(doc) = e.mesh_editor.doc.clone() else {
        return false;
    };
    if e.scene
        .actors
        .get(index)
        .and_then(|v| v.editable_mesh.as_ref())
        .is_none_or(|m| Some(m.asset) != e.mesh_editor.record.as_ref().map(|r| r.meta.id))
    {
        return false;
    }
    let Ok(inverse) = e.scene.world_matrix(index).inverse() else {
        return false;
    };
    if e.mesh_editor.mode == Mode::Edges {
        let world = e.scene.world_matrix(index);
        let candidates = doc
            .faces
            .iter()
            .filter(|f| {
                e.mesh_editor.visible(&doc, f.group) && e.mesh_editor.unlocked(&doc, f.group)
            })
            .flat_map(crate::mesh_ops::edges)
            .collect::<BTreeSet<_>>();
        let hit = candidates
            .into_iter()
            .filter_map(|edge| {
                let a =
                    crate::viewport::project(&e.view, world.point(doc.vertices[edge[0] as usize]));
                let b =
                    crate::viewport::project(&e.view, world.point(doc.vertices[edge[1] as usize]));
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
                let depth = 1. / ((1. - t) / a[2] + t / b[2]);
                let projected = [a[0] + d[0] * t, a[1] + d[1] * t];
                let origin = inverse.point(e.view.unproject(projected, 0.));
                let direction = inverse.vector(crate::lighting::sub(
                    e.view.unproject(projected, 1.),
                    e.view.unproject(projected, 0.),
                ));
                let front = mesh::pick(&doc, origin, direction, |f| {
                    e.mesh_editor.visible(&doc, f.group)
                })
                .is_none_or(|(_, z)| z >= depth - 0.03);
                (distance <= 10. && front).then_some((edge, distance, depth))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.2.total_cmp(&b.2)))
            .map(|v| v.0);
        if !extend {
            e.mesh_editor.edges.clear();
        }
        if let Some(edge) = hit
            && !e.mesh_editor.edges.insert(edge)
        {
            e.mesh_editor.edges.remove(&edge);
        }
    } else if e.mesh_editor.mode == Mode::Vertices {
        let world = e.scene.world_matrix(index);
        let candidates = doc
            .faces
            .iter()
            .filter(|f| {
                e.mesh_editor.visible(&doc, f.group) && e.mesh_editor.unlocked(&doc, f.group)
            })
            .flat_map(|f| f.vertices)
            .collect::<BTreeSet<_>>();
        let hit = candidates
            .into_iter()
            .filter_map(|i| {
                let p = crate::viewport::project(&e.view, world.point(doc.vertices[i as usize]));
                let d = (p[0] - pixel[0]).hypot(p[1] - pixel[1]);
                (p[2] > 1. && d <= 10.).then_some((i, d, p[2]))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.2.total_cmp(&b.2)))
            .map(|v| v.0);
        if !extend {
            e.mesh_editor.vertices.clear();
        }
        if let Some(id) = hit
            && !e.mesh_editor.vertices.insert(id)
        {
            e.mesh_editor.vertices.remove(&id);
        }
    } else {
        let a = e.view.unproject(pixel, 1.);
        let b = e.view.unproject(pixel, 2.);
        let hit = mesh::pick(
            &doc,
            inverse.point(a),
            inverse.vector(crate::lighting::sub(b, a)),
            |f| e.mesh_editor.visible(&doc, f.group) && e.mesh_editor.unlocked(&doc, f.group),
        );
        if !extend {
            e.mesh_editor.selected.clear();
        }
        if let Some((id, _)) = hit
            && !e.mesh_editor.selected.insert(id)
        {
            e.mesh_editor.selected.remove(&id);
        }
    }
    e.mesh_editor.revision += 1;
    e.view_dirty = true;
    true
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
pub fn window(ui: &Ui, e: &mut Editor) {
    if !e.mesh_editor.open {
        return;
    }
    prune_selection(e);
    let mut open = true;
    ui.window("Blockout")
        .opened(&mut open)
        .position([20., 80.], Condition::FirstUseEver)
        .size([500., (ui.io().display_size[1] - 110.).min(720.)], Condition::FirstUseEver)
        .size_constraints([380., 420.], [1400., 1200.])
        .build(|| {
            if let Some(error) = &e.mesh_editor.error {
                let _color = ui.push_style_color(imgui::StyleColor::Text, [1.,0.5,0.3,1.]);
                ui.text_wrapped(error);
            }
            let Some(doc) = e.mesh_editor.doc.clone() else {return;};
            let Some(record) = e.mesh_editor.record.clone() else {return;};
            ui.text_wrapped(assets::path_string(&e.root, &record.path));
            let users = e.scene.actors.iter().filter(|v|v.editable_mesh.as_ref().is_some_and(|m|m.asset==record.meta.id)).count();
            ui.text_wrapped(format!("Shared asset: {users} instance(s) in this scene. Edits save automatically. Use Undo/Redo to revert geometry."));
            ui.disabled(e.playing, || {
                if button(ui, "Undo") {let result=undo(e,false);report(e,result);}
                crate::gui::inline(ui, "Redo");
                if button(ui, "Redo") {let result=undo(e,true);report(e,result);}
                crate::gui::inline(ui, "Reload external changes");
                if button(ui, "Reload external changes") {open_asset_reload(e,&record);}
                let mut mode=match e.mesh_editor.mode {Mode::Faces=>0,Mode::Edges=>1,Mode::Vertices=>2};
                if ui.combo_simple_string(crate::gui::field(ui, "Selection"),&mut mode,&["Faces (1)","Edges (2)","Vertices (3)"]) {set_mode(e,[Mode::Faces,Mode::Edges,Mode::Vertices][mode]);}
                if users==0 && button(ui, "Add instance to scene") {let result=add_instance(e);report(e,result);}
                ui.text_wrapped("Click Scene to select; Ctrl-click adds/removes. Ctrl+Z / Ctrl+Y undo and redo geometry.");
                if let Some(_tabs)=ui.tab_bar("Blockout tabs") {
                    if let Some(_tab)=ui.tab_item("Build / Edit") {edit_panel(ui,e,&doc);}
                    if let Some(_tab)=ui.tab_item("Groups") {groups_panel(ui,e,&doc);}
                    if let Some(_tab)=ui.tab_item("Materials") {materials_panel(ui,e,&doc);}
                    if let Some(_tab)=ui.tab_item("Asset / Budget") {
                        ui.input_text(crate::gui::field(ui, "Asset path"),&mut e.mesh_editor.path).build();
                        if button(ui, "Move / Rename asset") {
                            match assets::move_asset(&e.root,&record,&e.mesh_editor.path) {
                                Ok(())=> {
                                    e.mesh_editor.record.as_mut().unwrap().path=assets::inside(&e.root,&e.mesh_editor.path).unwrap();
                                    e.assets.refresh();
                                }
                                Err(error)=>e.mesh_editor.error=Some(error),
                            }
                        }
                        ui.text_wrapped(format!("{} authored faces / {} stored vertices / {} groups / {} materials", doc.faces.len(),doc.vertices.len(),doc.groups.len(),doc.materials.len()));
                        let mut entity=Actor::cube("Budget".into());
                        let mut component=mesh::Component::new(record.meta.id);
                        component.document=Some(Arc::new(doc.clone()));entity.editable_mesh=Some(component);
                        let quads=crate::lighting::quads(&entity);
                        if let Ok(chunks)=crate::mesh_compile::chunks(&quads) {
                            ui.text_wrapped(format!("{} compiled triangles / {} spatial chunks / {} indexed vertices",quads.len()*2,chunks.len(),chunks.iter().map(|c|c.vertices.len()).sum::<usize>()));
                        }
                        ui.text_wrapped("PSX: chunk culling, shared positions, backface rejection and clipping. Scene build budget: 7000 base triangles plus clipping reserve. Measure performance in Play.");
                        ui.text_wrapped("Materials support color and Unlit. UV coordinates are stored; texture import/rendering is a separate feature.");
                    }
                }
            });
        });
    if !open {
        e.mesh_editor.open = false;
        preview(e);
    }
}
fn open_asset_reload(e: &mut Editor, record: &assets::Record) {
    let index = assets::scan(&e.root, &mut Default::default());
    match index.resolve(record.meta.id).cloned() {
        Ok(r) => open(e, r, e.mesh_editor.target),
        Err(error) => e.mesh_editor.error = Some(error),
    }
}
fn edit_panel(ui: &Ui, e: &mut Editor, doc: &Document) {
    let shapes = ["Box", "Plane", "Ramp", "Stairs"];
    ui.combo_simple_string(
        crate::gui::field(ui, "Shape"),
        &mut e.mesh_editor.shape,
        &shapes,
    );
    crate::gui::Drag::new(crate::gui::field(ui, "Origin"))
        .speed(0.02)
        .build_array(ui, &mut e.mesh_editor.origin);
    crate::gui::Drag::new(crate::gui::field(ui, "Size"))
        .speed(0.02)
        .build_array(ui, &mut e.mesh_editor.size);
    crate::gui::Drag::new(crate::gui::field(ui, "Steps"))
        .range(1, 32)
        .build(ui, &mut e.mesh_editor.steps);
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
    ui.separator();
    ui.text(format!(
        "Selected: {} faces / {} vertices / {} edges",
        e.mesh_editor.selected.len(),
        e.mesh_editor.vertices.len(),
        e.mesh_editor.edges.len()
    ));
    if button(ui, "Select all visible") {
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
    crate::gui::inline(ui, "Clear selection");
    if button(ui, "Clear selection") {
        e.mesh_editor.selected.clear();
        e.mesh_editor.vertices.clear();
        e.mesh_editor.edges.clear();
        e.mesh_editor.revision += 1;
    }
    if button(ui, "Select coplanar")
        && let Some(f) = doc
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
    crate::gui::inline(ui, "Select connected");
    if button(ui, "Select connected") {
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
    crate::gui::Drag::new(crate::gui::field(ui, "Extrusion distance"))
        .speed(0.1)
        .build(ui, &mut e.mesh_editor.distance);
    if button(ui, "Extrude (E)") {
        let result = push_selection(e, true);
        report(e, result);
    }
    ui.same_line();
    if button(
        ui,
        if e.mesh_editor.mode == Mode::Edges {
            "Bevel / ramp (Q)"
        } else {
            "Push inward (Q)"
        },
    ) {
        let result = push_selection(e, false);
        report(e, result);
    }
    ui.text_wrapped("RMB + WASD/QE flies the camera. Without RMB: E extrudes faces; Q pushes faces inward or bevels edges. Bevel needs a convex closed solid and two quad faces.");
    if button(ui, "Subdivide faces") {
        let selected = e.mesh_editor.selected.clone();
        let result = edit(e, |d| {
            d.split(&selected);
            Ok(())
        });
        report(e, result);
    }
    crate::gui::inline(ui, "Flip faces");
    if button(ui, "Flip faces") {
        let selected = e.mesh_editor.selected.clone();
        let result = edit(e, |d| {
            for f in &mut d.faces {
                if selected.contains(&f.id) {
                    if f.vertices[2] == f.vertices[3] {
                        f.vertices = [f.vertices[0], f.vertices[2], f.vertices[1], f.vertices[1]];
                        f.uv = [f.uv[0], f.uv[2], f.uv[1], f.uv[1]];
                    } else {
                        f.vertices = [f.vertices[0], f.vertices[3], f.vertices[2], f.vertices[1]];
                        f.uv = [f.uv[0], f.uv[3], f.uv[2], f.uv[1]];
                    }
                }
            }
            Ok(())
        });
        report(e, result);
    }
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
    crate::gui::inline(ui, "Extract to child entity");
    if button(ui, "Extract to child entity") {
        let result = extract(e);
        report(e, result);
    }
    ui.text_wrapped(
        "Extrusion acts on individual faces. Coplanar convex quads and triangles are supported.",
    );
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
        set_mode(e, Mode::Faces);
        e.mesh_editor.selected = doc
            .faces
            .iter()
            .filter(|f| {
                f.material == id
                    && e.mesh_editor.visible(doc, f.group)
                    && e.mesh_editor.unlocked(doc, f.group)
            })
            .map(|f| f.id)
            .collect();
        e.mesh_editor.revision += 1;
    }
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
                    0 => edit_panel(ui, e, &doc),
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
    click(context, &mut e, 0, "Add shape");
    assert_eq!(e.mesh_editor.doc.as_ref().unwrap().faces.len(), 12);
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
    click(context, &mut e, 0, "Subdivide faces");
    assert_eq!(e.mesh_editor.doc.as_ref().unwrap().faces.len(), 48);
    undo(&mut e, false).unwrap();
    assert_eq!(e.mesh_editor.doc.as_ref().unwrap().faces.len(), 12);
    e.mesh_editor.selected.clear();
    let pixel = crate::viewport::project(&e.view, [0., 0., 0.]);
    assert!(pick(&mut e, [pixel[0], pixel[1]], false));
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
    pick(&mut e, [pixel[0], pixel[1]], false);
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
        let mut scene = Scene::default();
        scene.actors = vec![first, second];
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
