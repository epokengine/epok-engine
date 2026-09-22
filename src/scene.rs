pub use crate::actor_document::ActorInstance as Actor;
use crate::transform::Matrix;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::{fs, path::Path};
fn default_fov() -> f32 {
    90.
}
pub const DEFAULT_CAMERA_SKY_COLOR: [f32; 3] = [33. / 255., 40. / 255., 52. / 255.];
fn default_camera_sky_color() -> [f32; 3] {
    DEFAULT_CAMERA_SKY_COLOR
}
fn default_active() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Material {
    /// Normalized texture cycles per second, split at wrap seams before rendering.
    pub uv_scroll: [f32; 2],
    pub color: [f32; 3],
    pub unlit: bool,
    pub texture: Option<uuid::Uuid>,
    pub blend: crate::texture::BlendMode,
    pub depth_bias: i16,
}
impl Default for Material {
    fn default() -> Self {
        Self {
            uv_scroll: [0.; 2],
            color: [1.; 3],
            unlit: false,
            texture: None,
            blend: Default::default(),
            depth_bias: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ClassDefaults {
    pub name: String,
    pub provider: crate::reflection_schema::Extension,
    pub backend: crate::reflection_schema::Extension,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_id: Option<String>,
    /// Stored alongside readable names so renamed members can be migrated explicitly.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub member_ids: BTreeMap<String, String>,
    /// Only explicit instance overrides are stored. Values missing from this map use
    /// the (possibly inherited) class default at build time.
    #[serde(default)]
    pub properties: BTreeMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub overrides: BTreeSet<String>,
}
impl Default for ClassDefaults {
    fn default() -> Self {
        Self {
            name: String::new(),
            provider: crate::reflection_schema::native_provider(),
            backend: crate::reflection_schema::native_backend(),
            class_id: None,
            member_ids: BTreeMap::new(),
            properties: BTreeMap::new(),
            overrides: BTreeSet::new(),
        }
    }
}
impl<'de> Deserialize<'de> for ClassDefaults {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            name: String,
            #[serde(default = "crate::reflection_schema::native_provider")]
            provider: crate::reflection_schema::Extension,
            #[serde(default = "crate::reflection_schema::native_backend")]
            backend: crate::reflection_schema::Extension,
            #[serde(default)]
            class_id: Option<String>,
            #[serde(default)]
            member_ids: BTreeMap<String, String>,
            #[serde(default)]
            properties: BTreeMap<String, serde_json::Value>,
            #[serde(default)]
            overrides: Option<BTreeSet<String>>,
        }
        let raw = Raw::deserialize(deserializer)?;
        // Legacy .epokmap scenes have no explicitness field. Every persisted
        // legacy value is therefore retained as an override; equality is never used.
        let mut overrides = raw.overrides.unwrap_or_default();
        overrides.extend(raw.properties.keys().cloned());
        Ok(Self {
            name: raw.name,
            provider: raw.provider,
            backend: raw.backend,
            class_id: raw.class_id,
            member_ids: raw.member_ids,
            properties: raw.properties,
            overrides,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BuiltinData {
    /// Persistent authoring identity; runtime references are resolved to fresh handles at cook/load.
    #[serde(default)]
    pub id: uuid::Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blueprint_instance: Option<crate::blueprint_templates::Instance>,
    #[serde(default = "default_fov")]
    pub camera_fov: f32,
    /// Flat clear colour used whenever this camera is active. It deliberately
    /// starts at the runtime's historic blue-grey clear so old projects render
    /// exactly as they did before this camera setting existed.
    #[serde(default = "default_camera_sky_color")]
    pub camera_sky_color: [f32; 3],
    #[serde(default = "default_active")]
    pub active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite: Option<crate::sprites::Sprite>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite_animator: Option<crate::sprites::SpriteAnimator>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub particle_emitter: Option<crate::particles::Emitter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeline: Option<crate::timeline_scene::Component>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub particle_effect: Option<crate::particle_effect_scene::Component>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collider: Option<crate::collision::Collider>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub palette_animator: Option<crate::palette::Animator>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skeletal_mesh: Option<crate::skeletal::Component>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editable_mesh: Option<crate::mesh::Component>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain: Option<crate::terrain::Component>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<crate::audio::AudioSource>,
    pub name: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<usize>,
    #[serde(default)]
    pub material: Material,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canvas: Option<crate::hud::Canvas>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rect: Option<crate::hud::RectTransform>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<crate::hud::Image>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<crate::hud::Text>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<crate::hud::ProgressBar>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_element: Option<crate::hud::LayoutElement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_container: Option<crate::hud::LayoutContainer>,
    #[serde(default)]
    pub lighting: crate::lighting::MeshLighting,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light: Option<crate::lighting::Light>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob_shadow: Option<crate::shadows::BlobShadow>,
}

impl BuiltinData {
    pub fn cube(name: String) -> Self {
        Self {
            name,
            id: uuid::Uuid::new_v4(),
            blueprint_instance: None,
            active: true,
            camera_fov: 90.,
            camera_sky_color: default_camera_sky_color(),
            sprite: None,
            sprite_animator: None,
            particle_emitter: None,
            timeline: None,
            particle_effect: None,
            collider: None,
            palette_animator: None,
            editable_mesh: None,
            terrain: None,
            skeletal_mesh: None,
            audio: None,
            kind: "Mesh".into(),
            parent: None,
            material: Material::default(),
            position: [0., 0.5, 0.],
            rotation: [0.; 3],
            scale: [1.; 3],
            canvas: None,
            rect: None,
            image: None,
            text: None,
            progress: None,
            layout_element: None,
            layout_container: None,
            lighting: Default::default(),
            light: None,
            blob_shadow: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(from = "SceneDocument", into = "SceneDocument")]
pub struct Scene {
    #[serde(default)]
    pub fog: crate::effects::Fog,
    #[serde(default)]
    pub hud_budget: crate::hud::Budget,
    #[serde(skip)]
    pub textures: BTreeMap<uuid::Uuid, std::sync::Arc<crate::texture::Data>>,
    /// Project output size used by HUD authoring; never serialized into a scene.
    #[serde(skip, default = "legacy_display_size")]
    pub display_size: [u16; 2],
    pub version: u32,
    pub name: String,
    pub actors: Vec<Actor>,
    #[serde(default)]
    pub environment: crate::lighting::Settings,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bake: Option<crate::lighting::Bake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation: Option<crate::navigation::Bake>,
    /// The map's own Blueprint, one `SceneScriptActor` subclass per scene.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_script: Option<crate::actor_document::SceneScript>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SceneDocument {
    version: u32,
    name: String,
    actors: Vec<Actor>,
    #[serde(default)]
    fog: crate::effects::Fog,
    #[serde(default)]
    hud_budget: crate::hud::Budget,
    #[serde(default)]
    environment: crate::lighting::Settings,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bake: Option<crate::lighting::Bake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    navigation: Option<crate::navigation::Bake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scene_script: Option<crate::actor_document::SceneScript>,
}
impl From<Scene> for SceneDocument {
    fn from(mut scene: Scene) -> Self {
        scene.sync_actor_components();
        Self {
            version: scene.version,
            name: scene.name,
            actors: scene.actors,
            fog: scene.fog,
            hud_budget: scene.hud_budget,
            environment: scene.environment,
            bake: scene.bake,
            navigation: scene.navigation,
            scene_script: scene.scene_script,
        }
    }
}
impl From<SceneDocument> for Scene {
    fn from(doc: SceneDocument) -> Self {
        let mut scene = Self {
            version: doc.version,
            name: doc.name,
            actors: doc.actors,
            fog: doc.fog,
            hud_budget: doc.hud_budget,
            environment: doc.environment,
            bake: doc.bake,
            navigation: doc.navigation,
            scene_script: doc.scene_script,
            textures: Default::default(),
            display_size: legacy_display_size(),
        };
        scene.refresh_actor_hierarchy();
        // Older caches used only positional color arrays. Bind those positions
        // once when loading, before edits can add, remove or reorder actors.
        if let Some(bake) = &mut scene.bake
            && bake.actor_ids.is_empty()
            && bake.colors.len() == scene.actors.len()
        {
            bake.actor_ids = scene.actors.iter().map(|actor| actor.id).collect();
        }
        scene
    }
}
impl PartialEq for Scene {
    fn eq(&self, other: &Self) -> bool {
        serde_json::to_value(self).ok() == serde_json::to_value(other).ok()
    }
}

fn legacy_display_size() -> [u16; 2] {
    [320, 240]
}

impl Default for Scene {
    fn default() -> Self {
        let mut camera = Actor::cube("Main Camera".into());
        camera.kind = "Camera".into();
        camera.position = [0., 3., -6.];
        let mut floor = Actor::cube("Ground".into());
        floor.position = [0., -0.15, 0.];
        floor.scale = [7., 0.2, 7.];
        let mut second = Actor::cube("Cube.001".into());
        second.position = [2., 0.5, 1.];
        Self {
            hud_budget: Default::default(),
            fog: Default::default(),
            environment: Default::default(),
            bake: None,
            navigation: None,
            display_size: legacy_display_size(),
            textures: Default::default(),
            version: crate::actor_document::SCENE_VERSION,
            name: "SampleScene".into(),
            actors: vec![camera, Actor::cube("Cube".into()), second, floor],
            scene_script: None,
        }
    }
}

impl Scene {
    pub fn is_active(&self, index: usize) -> bool {
        let mut current = Some(index);
        for _ in 0..=self.actors.len() {
            let Some(i) = current else {
                return true;
            };
            let Some(e) = self.actors.get(i) else {
                return false;
            };
            if !e.active {
                return false;
            }
            current = e.parent;
        }
        false
    }
    pub fn is_descendant(&self, child: usize, ancestor: usize) -> bool {
        let mut current = Some(child);
        for _ in 0..=self.actors.len() {
            let Some(i) = current else {
                return false;
            };
            if i == ancestor {
                return true;
            }
            current = self.actors.get(i).and_then(|e| e.parent);
        }
        false
    }
    pub fn world_matrix(&self, index: usize) -> Matrix {
        let e = &self.actors[index];
        let local = Matrix::trs(e.position, e.rotation, e.scale);
        self.spatial_parent(index)
            .map_or(local, |p| self.world_matrix(p).compose(local))
    }
    pub fn spatial_parent(&self, index: usize) -> Option<usize> {
        self.actors[index]
            .attach
            .as_ref()
            .and_then(|a| self.actor_index(a.actor))
    }
    pub fn parent_matrix(&self, index: usize) -> Matrix {
        self.spatial_parent(index)
            .map_or(Matrix::IDENTITY, |p| self.world_matrix(p))
    }
    pub fn reparent(
        &mut self,
        index: usize,
        parent: Option<usize>,
        keep_world: bool,
    ) -> Result<(), String> {
        if index >= self.actors.len()
            || parent.is_some_and(|p| p >= self.actors.len() || self.is_descendant(p, index))
        {
            return Err("Cannot parent an object to itself or one of its descendants".into());
        }
        if self.actors[index].parent == parent {
            return Ok(());
        }
        let original = self.actors[index].clone();
        let mut next = original.clone();
        let spatial_parent = parent.filter(|p| {
            crate::actor_components::domain(&next)
                == crate::actor_components::domain(&self.actors[*p])
        });
        if keep_world && next.rect.is_some() {
            // One layout pass answers for the element and its new parent. A rect
            // the pass never placed has no world rectangle to preserve — it is
            // hidden under a container that assigns its rect wherever it lands —
            // so the move keeps the authored values instead of failing.
            let boxes = crate::hud::layouts(self);
            if let Some(old) = boxes.get(index).copied().flatten()
                && let Some(r) = next.rect.as_mut()
            {
                let p = spatial_parent
                    .and_then(|p| boxes.get(p).copied().flatten())
                    .unwrap_or([
                        0.,
                        0.,
                        self.display_size[0] as f32,
                        self.display_size[1] as f32,
                    ]);
                for i in 0..2 {
                    r.size[i] = old[i + 2] - p[i + 2] * (r.anchor_max[i] - r.anchor_min[i]);
                    r.position[i] = old[i]
                        - p[i]
                        - p[i + 2]
                            * (r.anchor_min[i] + (r.anchor_max[i] - r.anchor_min[i]) * r.pivot[i])
                        + old[i + 2] * r.pivot[i];
                }
            }
        } else if keep_world {
            let inverse =
                spatial_parent.map_or(Ok(Matrix::IDENTITY), |p| self.world_matrix(p).inverse())?;
            inverse
                .compose(self.world_matrix(index))
                .apply_trs(&mut next)?;
        }
        next.parent = parent;
        next.logical_parent = parent.map(|index| self.actors[index].id);
        next.attach = spatial_parent.map(|p| crate::actor_document::Attachment {
            actor: self.actors[p].id,
            component: None,
        });
        self.actors[index] = next;
        if let Err(error) = self.validate() {
            self.actors[index] = original;
            return Err(error);
        }
        Ok(())
    }
    pub fn duplicate_branch(&mut self, index: usize) -> Result<usize, String> {
        self.sync_actor_components();
        let id = self.actors.get(index).ok_or("No actor selected")?.id;
        let copy = self.duplicate_actor_branch(id)?;
        self.actor_index(copy)
            .ok_or("Duplicated actor is missing".into())
    }
    pub fn delete_branch(&mut self, index: usize) {
        self.sync_actor_components();
        if let Some(actor) = self.actors.get(index) {
            self.delete_actor_branch(actor.id);
        }
    }
    pub fn duplicate_identities(&mut self) {
        self.sync_actor_components();
        let map = crate::actor_document::fresh_identities(&self.actors);
        for actor in &mut self.actors {
            crate::actor_document::remap_actor(actor, &map);
        }
        if let Some(script) = &mut self.scene_script {
            crate::actor_document::remap_scene_script(script, &map, true);
        }
        self.refresh_actor_hierarchy();
    }
    /// Gives a map that has no scene Blueprint the default one, in memory.
    ///
    /// Every map owns exactly one `SceneScriptActor`, so the editor does not ask
    /// an author to create it. The default is not an edit: it never marks the
    /// document dirty and it never rewrites the file on load. It is written the
    /// next time the author saves the map, like any other part of the document.
    ///
    /// `default_parent` is the project's `default_scene_script_parent` when it
    /// resolves to a `SceneScriptActor` subclass; otherwise the native
    /// `epok::SceneScriptActor` is used. Returns whether one was created.
    pub fn ensure_scene_script(
        &mut self,
        default_parent: Option<&crate::actor_document::ClassReference>,
    ) -> bool {
        if self.scene_script.is_some() {
            return false;
        }
        let parent = default_parent.cloned().unwrap_or_else(|| {
            crate::actor_document::ClassReference::new(
                "epok::SceneScriptActor",
                crate::object_model::SCENE_SCRIPT_ACTOR_ID,
            )
        });
        let class_id = parent
            .class_id
            .clone()
            .unwrap_or_else(|| crate::object_model::SCENE_SCRIPT_ACTOR_ID.to_owned());
        self.scene_script = Some(crate::actor_document::SceneScript {
            blueprint: crate::blueprint_asset::BlueprintAsset::new(
                format!("{}_SceneScript", self.name),
                class_id,
            ),
            parent,
        });
        true
    }
    /// A copy of this map under a new name, with fresh identities everywhere.
    ///
    /// The embedded Blueprint gets a new asset id and every `ActorRef`/`ObjectRef`
    /// literal inside it is rewritten to the duplicated actor, component or entity,
    /// so the copy refers to itself and never back to the original. Asset UUIDs
    /// (textures, meshes, sounds) are not remapped: they name shared content.
    pub fn duplicate_document(&self, new_name: &str) -> Self {
        let mut copy = self.clone();
        copy.name = new_name.to_owned();
        copy.duplicate_identities();
        if let Some(script) = &mut copy.scene_script {
            script.blueprint.name = format!("{new_name}_SceneScript");
        }
        copy
    }
    /// Position of one authored actor, by identity.
    pub fn actor_index(&self, id: uuid::Uuid) -> Option<usize> {
        self.actors.iter().position(|actor| actor.id == id)
    }
    /// `id` plus every actor whose `logical_parent` chain reaches it, in
    /// document order. A parent cycle cannot extend the set: an actor already in
    /// it is never visited twice.
    pub fn actor_branch(&self, id: uuid::Uuid) -> Vec<uuid::Uuid> {
        let mut branch = BTreeSet::from([id]);
        // The chain is walked per actor rather than recursively, so a document
        // whose parents point forwards is covered in one pass per generation.
        loop {
            let before = branch.len();
            for actor in &self.actors {
                if actor
                    .logical_parent
                    .is_some_and(|parent| branch.contains(&parent))
                {
                    branch.insert(actor.id);
                }
            }
            if branch.len() == before {
                break;
            }
        }
        self.actors
            .iter()
            .map(|actor| actor.id)
            .filter(|id| branch.contains(id))
            .collect()
    }
    /// A name no actor and no entity of this map already uses, suffixed like
    /// [`Self::duplicate_branch`] does for actors.
    pub fn unique_actor_name(&self, base: &str) -> String {
        let mut name = base.to_owned();
        let mut suffix = 1;
        while self.actors.iter().any(|a| a.name == name)
            || self.actors.iter().any(|e| e.name == name)
        {
            name = format!("{base}.{suffix:03}");
            suffix += 1;
        }
        name
    }
    /// Removes one actor and its whole logical branch, returning the removed
    /// identities. Attachments and logical parents that pointed into the branch
    /// are cleared rather than left dangling, including attachments that named a
    /// component of a removed actor.
    pub fn delete_actor_branch(&mut self, id: uuid::Uuid) -> Vec<uuid::Uuid> {
        let branch = self.actor_branch(id);
        if branch.is_empty() {
            return branch;
        }
        let removed: BTreeSet<_> = branch.iter().copied().collect();
        let components: BTreeSet<_> = self
            .actors
            .iter()
            .filter(|actor| removed.contains(&actor.id))
            .flat_map(|actor| actor.components.iter().map(|c| c.id))
            .collect();
        self.actors.retain(|actor| !removed.contains(&actor.id));
        for actor in &mut self.actors {
            if actor.logical_parent.is_some_and(|p| removed.contains(&p)) {
                actor.logical_parent = None;
            }
            if actor.attach.as_ref().is_some_and(|at| {
                removed.contains(&at.actor) || at.component.is_some_and(|c| components.contains(&c))
            }) {
                actor.attach = None;
            }
        }
        self.refresh_actor_hierarchy();
        branch
    }
    /// Copies one actor and its whole logical branch with fresh identities,
    /// appended in document order. The copied root keeps the original's logical
    /// parent; references that leave the branch keep their target.
    pub fn duplicate_actor_branch(&mut self, id: uuid::Uuid) -> Result<uuid::Uuid, String> {
        let branch = self.actor_branch(id);
        if branch.is_empty() {
            return Err("No actor selected".into());
        }
        let mut copies: Vec<_> = self
            .actors
            .iter()
            .filter(|actor| branch.contains(&actor.id))
            .cloned()
            .collect();
        let map = crate::actor_document::fresh_identities(&copies);
        for actor in &mut copies {
            crate::actor_document::remap_actor(actor, &map);
        }
        let root = *map.get(&id).expect("the branch contains its own root");
        for mut actor in copies {
            actor.name = self.unique_actor_name(&actor.name);
            self.actors.push(actor);
        }
        self.refresh_actor_hierarchy();
        Ok(root)
    }
    /// Structural validation only. Rules that need the reflected class model
    /// live in [`Self::validate_with_model`]; every existing caller keeps the
    /// offline behaviour it had before the actor model existed.
    pub fn validate(&self) -> Result<(), String> {
        self.validate_with_model(None)
    }
    /// Full validation. `model` enables the class-aware half of the actor
    /// document rules (`actor_document::validate`); without it only the
    /// structural half runs.
    pub fn validate_with_model(
        &self,
        model: Option<&crate::object_model::Model>,
    ) -> Result<(), String> {
        if self.version > crate::actor_document::SCENE_VERSION {
            return Err(format!(
                "This scene was written by a newer editor (document version {}, this editor supports up to {}). Update the editor rather than opening it: the document is never loaded as empty.",
                self.version,
                crate::actor_document::SCENE_VERSION
            ));
        }
        if self.version != crate::actor_document::SCENE_VERSION {
            return Err("This map uses the retired Entity/Behaviour format; recreate the project with the Actor templates.".into());
        }
        if self.actors.len() > 512 {
            return Err("Editor limit: 512 objects per scene".into());
        }
        let mut entity_ids = BTreeSet::new();
        for entity in &self.actors {
            if entity.id.is_nil() {
                if self.version >= 3 {
                    return Err("Scene v3 requires a persistent UUID for every entity.".into());
                }
            } else if !entity_ids.insert(entity.id) {
                return Err(format!(
                    "Duplicate entity UUID {}. Duplicate actors through the editor to assign new identities.",
                    entity.id
                ));
            }
        }
        for (index, e) in self.actors.iter().enumerate() {
            if let Some(m) = &e.editable_mesh
                && (m.asset.is_nil()
                    || m.materials.iter().any(|(id, mat)| {
                        id.is_nil()
                            || mat
                                .color
                                .iter()
                                .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
                    }))
            {
                return Err("Invalid EditableMesh reference or material override".into());
            }
            let mut parent = e.parent;
            let mut depth = 0;
            while let Some(p) = parent {
                if p >= self.actors.len() || p == index || depth >= 32 {
                    return Err(
                        "Invalid hierarchy: missing parent, cycle, or depth above 32".into(),
                    );
                }
                parent = self.actors[p].parent;
                depth += 1;
            }
            if e.material
                .color
                .iter()
                .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
            {
                return Err("Material colors must be between 0 and 1".into());
            }
            crate::texture::validate_material(&e.material)?;
            if let Some(m) = &e.editable_mesh {
                for mat in m.materials.values() {
                    crate::texture::validate_material(mat)?;
                }
            }
            if !["Mesh", "Camera", "Empty"].contains(&e.kind.as_str()) {
                return Err("Unknown object type".into());
            }
            if e.name.trim().is_empty() || e.name.len() > 128 {
                return Err("Object names must contain 1–128 bytes".into());
            }
            if !e.camera_fov.is_finite() || !(25. ..=120.).contains(&e.camera_fov) {
                return Err("Camera horizontal FOV must be 25..120 degrees".into());
            }
            if !e
                .camera_sky_color
                .iter()
                .all(|channel| channel.is_finite() && (0. ..=1.).contains(channel))
            {
                return Err("Camera sky color channels must be between 0.0 and 1.0".into());
            }
            if e.position
                .iter()
                .chain(&e.rotation)
                .chain(&e.scale)
                .any(|v| !v.is_finite() || v.abs() > 10000.)
            {
                return Err("Transforms must be finite and within ±10000".into());
            }
            if e.scale.iter().any(|v| *v <= 0.) {
                return Err("Scale must be positive".into());
            }
        }
        for e in &self.actors {
            if let Some(b) = &e.blob_shadow
                && (!b.radius.is_finite()
                    || !(0.01..=8.).contains(&b.radius)
                    || !b.strength.is_finite()
                    || !(0. ..=1.).contains(&b.strength)
                    || !b.distance.is_finite()
                    || !(0.01..=32.).contains(&b.distance))
            {
                return Err("Invalid Blob Shadow".into());
            }
        }
        crate::skeletal::validate(self)?;
        crate::sprites::validate(self)?;
        crate::effects::validate(self)?;
        crate::particles::validate(self)?;
        crate::collision::validate(self)?;
        crate::palette::validate(self)?;
        crate::lighting::validate(self)?;
        crate::audio::validate(self)?;
        crate::hud::validate(self)?;
        crate::actor_document::validate_message(self, model)
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let mut scene = Self::load_unresolved(path)?;
        if scene.actors.iter().any(|e| {
            e.editable_mesh.is_some()
                || e.terrain.is_some()
                || e.skeletal_mesh.is_some()
                || e.material.texture.is_some()
                || e.sprite.is_some()
                || e.particle_emitter.is_some()
                || e.image.as_ref().is_some_and(|i| i.texture.is_some())
        }) && let Some(root) = path.ancestors().find(|p| p.join("assets").is_dir())
        {
            let index = crate::assets::scan(root, &mut Default::default());
            let _ = crate::mesh::resolve(&mut scene, &index);
            let _ = crate::terrain::resolve(&mut scene, &index);
            let _ = crate::skeletal::resolve(&mut scene, &index);
            let _ = crate::texture::resolve(&mut scene, &index);
        }
        Ok(scene)
    }
    pub fn load_unresolved(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        let document: serde_json::Value = crate::document::from_slice(&bytes)?;
        if document["version"]
            .as_u64()
            .is_none_or(|version| version < u64::from(crate::actor_document::SCENE_VERSION))
        {
            return Err("This map uses the retired Entity/Behaviour format; recreate the project with the Actor templates.".into());
        }
        let mut scene: Self = crate::document::from_slice(&bytes).map_err(|e| e.to_string())?;
        let indices: BTreeMap<_, _> = scene
            .actors
            .iter()
            .enumerate()
            .map(|(index, actor)| (actor.id, index))
            .collect();
        for actor in &mut scene.actors {
            actor.parent = actor
                .logical_parent
                .and_then(|id| indices.get(&id).copied());
        }
        scene.validate()?;
        // Deterministic in-memory migration keeps references stable across repeated reads
        // and complete-folder relocation. Only Save publishes the assigned identities.
        scene.upgrade_entity_ids();
        Ok(scene)
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let mut document = self.clone();
        document.upgrade_entity_ids();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let temp = path.with_extension("epok.tmp");
        fs::write(
            &temp,
            crate::document::to_vec(&document).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        fs::rename(temp, path).map_err(|e| e.to_string())
    }
    pub fn upgrade_entity_ids(&mut self) {
        self.sync_actor_components();
    }
    pub fn refresh_actor_hierarchy(&mut self) {
        let indices: BTreeMap<_, _> = self
            .actors
            .iter()
            .enumerate()
            .map(|(i, a)| (a.id, i))
            .collect();
        for actor in &mut self.actors {
            actor.data.parent = actor
                .logical_parent
                .and_then(|id| indices.get(&id).copied());
            actor.data.id = actor.id;
            actor.data.name = actor.name.clone();
            actor.data.active = actor.active;
        }
    }
    pub fn sync_actor_components(&mut self) {
        let ids: Vec<_> = self.actors.iter().map(|actor| actor.id).collect();
        let domains: Vec<_> = self
            .actors
            .iter()
            .map(crate::actor_components::domain)
            .collect();
        for actor in &mut self.actors {
            if actor.logical_parent.is_none() && actor.parent.is_some() && actor.attach.is_none() {
                actor.attach = actor
                    .parent
                    .filter(|i| domains.get(*i) == Some(&crate::actor_components::domain(actor)))
                    .and_then(|i| ids.get(i))
                    .map(|id| crate::actor_document::Attachment {
                        actor: *id,
                        component: None,
                    });
            }
            actor.logical_parent = actor
                .parent
                .and_then(|index| ids.get(index).copied())
                .or(actor.logical_parent);
            crate::actor_components::sync(actor);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn near(a: Matrix, b: Matrix) {
        for r in 0..3 {
            for c in 0..4 {
                assert!((a.0[r][c] - b.0[r][c]).abs() < 0.002, "{a:?} != {b:?}");
            }
        }
    }
    #[test]
    fn nested_transform_inherits_rotation_scale_and_translation() {
        let mut s = Scene::default();
        s.actors[1].position = [3., 1., 0.];
        s.actors[1].rotation = [0., 0., 90.];
        s.actors[1].scale = [2.; 3];
        s.actors[2].parent = Some(1);
        s.sync_actor_components();
        s.actors[2].position = [1., 0., 0.];
        s.actors[2].scale = [1., 2., 1.];
        let p = s.world_matrix(2).point([1., 0., 0.]);
        for (a, b) in p.into_iter().zip([3., 5., 0.]) {
            assert!((a - b).abs() < 0.0001);
        }
        s.actors[3].parent = Some(2);
        s.sync_actor_components();
        s.actors[3].position = [0., 1., 0.];
        let p = s.world_matrix(3).point([0.; 3]);
        for (a, b) in p.into_iter().zip([-1., 3., 0.]) {
            assert!((a - b).abs() < 0.0001);
        }
        s.actors[1].position[0] += 5.;
        assert!((s.world_matrix(3).point([0.; 3])[0] - 4.).abs() < 0.0001);
    }
    #[test]
    fn reparent_and_unparent_preserve_world_and_reject_cycles_atomically() {
        let mut s = Scene::default();
        s.actors[3].position = [2., 1., -3.];
        s.actors[3].rotation = [23., 41., -12.];
        s.actors[3].scale = [2.; 3];
        s.actors[1].rotation = [18., -24., 71.];
        let before = s.world_matrix(1);
        s.reparent(1, Some(3), true).unwrap();
        near(before, s.world_matrix(1));
        let valid = s.clone();
        assert!(s.reparent(3, Some(1), true).is_err());
        assert_eq!(s, valid);
        assert!(s.reparent(1, Some(99), false).is_err());
        assert_eq!(s, valid);
        s.reparent(1, None, true).unwrap();
        near(before, s.world_matrix(1));
        let local = s.actors[1].clone();
        s.reparent(1, Some(3), false).unwrap();
        assert_eq!(s.actors[1].position, local.position);
        assert_eq!(s.actors[1].rotation, local.rotation);
    }
    #[test]
    fn shear_is_rendered_but_not_silently_discarded_when_unparenting() {
        let mut s = Scene::default();
        s.actors[1].scale = [2., 1., 1.];
        s.actors[2].rotation = [0., 0., 45.];
        s.reparent(2, Some(1), false).unwrap();
        let m = s.world_matrix(2);
        let dot = (0..3).map(|r| m.0[r][0] * m.0[r][1]).sum::<f32>();
        assert!(dot.abs() > 0.5);
        let before = s.clone();
        assert!(
            s.reparent(2, None, true)
                .unwrap_err()
                .contains("non-uniform")
        );
        assert_eq!(s, before);
        s.reparent(2, None, false).unwrap();
    }
    #[test]
    fn branch_duplicate_and_delete_remap_unsorted_parent_references() {
        let mut s = Scene::default();
        s.actors[1].parent = Some(3);
        s.actors[2].parent = Some(1);
        s.actors[1].material.color = [1., 0., 0.];
        let copy = s.duplicate_branch(3).unwrap();
        assert_eq!(copy, 6);
        assert_eq!(s.actors[4].parent, Some(6));
        assert_eq!(s.actors[5].parent, Some(4));
        assert_eq!(s.actors[4].material, s.actors[1].material);
        s.actors[4].material.color = [0., 1., 0.];
        assert_eq!(s.actors[1].material.color, [1., 0., 0.]);
        s.delete_branch(3);
        s.validate().unwrap();
        assert_eq!(s.actors.len(), 4);
        assert_eq!(s.actors[1].parent, Some(3));
        assert_eq!(s.actors[2].parent, Some(1));
    }
    #[test]
    fn old_scenes_load_and_new_material_and_parent_roundtrip() {
        let mut value = serde_json::to_value(Scene::default()).unwrap();
        for e in value["actors"].as_array_mut().unwrap() {
            e.as_object_mut().unwrap().remove("material");
        }
        let mut s: Scene = serde_json::from_value(value).unwrap();
        s.validate().unwrap();
        assert_eq!(s.actors[1].material, Material::default());
        s.actors[2].parent = Some(1);
        s.actors[2].material = Material {
            color: [0.25, 0.5, 1.],
            unlit: true,
            ..Default::default()
        };
        assert_eq!(
            crate::lighting::modulate([255; 3], s.actors[2].material.color),
            [64, 128, 255]
        );
        let decoded: Scene = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(decoded, s);
        s.actors[2].material.color[0] = f32::NAN;
        assert!(s.validate().is_err());
    }
    #[test]
    fn camera_sky_color_round_trips_and_rejects_invalid_channels() {
        let mut scene = Scene::default();
        assert_eq!(scene.actors[0].camera_sky_color, DEFAULT_CAMERA_SKY_COLOR);
        scene.actors[0].camera_sky_color = [0.15, 0.35, 0.8];
        let restored: Scene =
            serde_json::from_value(serde_json::to_value(&scene).unwrap()).unwrap();
        assert_eq!(restored.actors[0].camera_sky_color, [0.15, 0.35, 0.8]);
        scene.actors[0].camera_sky_color[1] = 1.01;
        assert!(scene.validate().unwrap_err().contains("sky color"));
    }
    #[test]
    fn scene_roundtrip_preserves_transforms() {
        let mut scene = Scene::default();
        scene.actors[1].position = [-42., 0.25, 123.];
        let decoded: Scene = serde_json::from_str(&serde_json::to_string(&scene).unwrap()).unwrap();
        decoded.validate().unwrap();
        assert_eq!(decoded, scene);
    }
    #[test]
    fn previous_scene_formats_are_rejected_without_rewriting_the_file() {
        let root = crate::workspace::tests::temp("retired-scene");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("Old.epokmap");
        for version in 1..crate::actor_document::SCENE_VERSION {
            let bytes =
                serde_json::to_vec(&serde_json::json!({"version":version,"entities":[]})).unwrap();
            fs::write(&path, &bytes).unwrap();
            assert!(Scene::load(&path).unwrap_err().contains("recreate"));
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn reject_invalid_scene_before_replacing_editor_state() {
        let mut scene = Scene {
            version: 99,
            ..Scene::default()
        };
        assert!(scene.validate().is_err());
        scene.version = 1;
        scene.actors[1].scale[0] = 0.;
        assert!(scene.validate().is_err());
        scene.actors[1].scale[0] = f32::NAN;
        assert!(scene.validate().is_err());
    }

    #[test]
    fn every_supported_scene_version_loads_and_a_newer_document_is_refused() {
        let mut scene = Scene::default();
        for version in [crate::actor_document::SCENE_VERSION] {
            scene.version = version;
            scene.validate().unwrap();
        }
        scene.version = crate::actor_document::SCENE_VERSION + 1;
        let error = scene.validate().unwrap_err();
        assert!(error.contains("newer editor"), "{error}");
        scene.version = 0;
        assert!(scene.validate().unwrap_err().contains("recreate"));
    }

    #[test]
    fn actor_content_survives_a_document_round_trip_and_raises_the_version() {
        let root = crate::workspace::tests::temp("scene-actors");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("Actors.epokmap");
        let mut scene = Scene::default();
        let actor = crate::actor_document::ActorInstance::new(
            uuid::Uuid::new_v4(),
            crate::actor_document::ClassReference::new(
                "epok::Actor3D",
                crate::object_model::ACTOR3D_ID,
            ),
            "Hero",
        );
        scene.actors.push(actor);
        assert_eq!(scene.version, crate::actor_document::SCENE_VERSION);
        scene.save(&path).unwrap();
        let loaded = Scene::load(&path).unwrap();
        assert_eq!(loaded.version, crate::actor_document::SCENE_VERSION);
        assert_eq!(loaded.actors, scene.actors);
        let bytes = fs::read(&path).unwrap();
        assert_eq!(Scene::load(&path).unwrap(), loaded);
        assert_eq!(fs::read(&path).unwrap(), bytes, "load never rewrites bytes");
        fs::remove_dir_all(root).unwrap();
    }

    /// Every map owns exactly one scene Blueprint, so the editor creates the default
    /// rather than asking. Creating it is not an edit: the document on disk is
    /// untouched until the author saves the map, and then it is written like any
    /// other part of the document.
    #[test]
    fn a_map_without_a_scene_blueprint_gets_the_default_in_memory_and_saves_it() {
        let root = crate::workspace::tests::temp("scene-script-default");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("Legacy.epokmap");
        Scene::default().save(&path).unwrap();
        let bytes = fs::read(&path).unwrap();

        let mut scene = Scene::load_unresolved(&path).unwrap();
        assert!(scene.scene_script.is_none());
        assert!(scene.ensure_scene_script(None));
        assert!(!scene.ensure_scene_script(None), "one per map, never two");
        let script = scene.scene_script.as_ref().unwrap();
        assert_eq!(
            script.parent.class_id.as_deref(),
            Some(crate::object_model::SCENE_SCRIPT_ACTOR_ID),
            "without a resolvable project default the native class is used"
        );
        assert_eq!(script.parent.name, "epok::SceneScriptActor");
        assert_eq!(
            script.blueprint.parent,
            script.parent.class_id.clone().unwrap()
        );
        assert_eq!(script.blueprint.name, format!("{}_SceneScript", scene.name));
        assert!(
            script.blueprint.functions.is_empty(),
            "the graph starts empty"
        );
        assert_eq!(
            scene.version,
            crate::actor_document::SCENE_VERSION,
            "the default is not a document edit"
        );
        assert_eq!(
            fs::read(&path).unwrap(),
            bytes,
            "and never rewrites the file"
        );

        // A project default that resolves is used verbatim.
        let mut custom = Scene::load_unresolved(&path).unwrap();
        let mode = uuid::Uuid::from_u128(70).to_string();
        custom.ensure_scene_script(Some(&crate::actor_document::ClassReference::new(
            "GameMode", &mode,
        )));
        let custom = custom.scene_script.unwrap();
        assert_eq!(custom.parent.name, "GameMode");
        assert_eq!(custom.blueprint.parent, mode);

        // Only an explicit save publishes it, and the document becomes version 5.
        scene.save(&path).unwrap();
        let reloaded = Scene::load_unresolved(&path).unwrap();
        assert_eq!(reloaded.version, crate::actor_document::SCENE_VERSION);
        assert_eq!(reloaded.scene_script, scene.scene_script);
        fs::remove_dir_all(root).unwrap();
    }

    /// A duplicated map must refer to itself: a copy that kept the original's class
    /// identity would collide with it in the compiler, and one that kept its actor
    /// UUIDs would drive the original's actors. Shared asset ids are not identities
    /// of this document and survive untouched.
    #[test]
    fn duplicating_a_map_remaps_the_scene_blueprint_and_its_map_scoped_references() {
        let mut scene = Scene {
            name: "Town".into(),
            ..Scene::default()
        };
        let actor = uuid::Uuid::new_v4();
        scene.actors.push(crate::actor_document::ActorInstance::new(
            actor,
            crate::actor_document::ClassReference::new(
                "epok::Actor3D",
                crate::object_model::ACTOR3D_ID,
            ),
            "Hero",
        ));
        assert!(scene.ensure_scene_script(None));
        let entity = scene.actors[0].id;
        let texture = uuid::Uuid::new_v4();
        let blueprint = &mut scene.scene_script.as_mut().unwrap().blueprint;
        let class = blueprint.id.clone();
        blueprint.variables.push(crate::blueprint_asset::Variable {
            id: uuid::Uuid::new_v4().to_string(),
            name: "hero".into(),
            value_type: crate::reflection_schema::Type::ActorRef { class: None },
            default: serde_json::json!(actor.to_string()),
            editable: true,
            timeline_animatable: false,
        });
        blueprint
            .defaults
            .insert("banner".into(), serde_json::json!(texture.to_string()));

        let copy = scene.duplicate_document("Town_Copy");
        assert_eq!(copy.name, "Town_Copy");
        let script = copy.scene_script.as_ref().unwrap();
        assert_ne!(script.blueprint.id, class, "the copy is a different class");
        assert_eq!(script.blueprint.name, "Town_Copy_SceneScript");
        assert_ne!(copy.actors[0].id, actor);
        assert_ne!(copy.actors[0].id, entity);
        assert_eq!(
            script.blueprint.variables[0].default,
            serde_json::json!(copy.actors.last().unwrap().id.to_string()),
            "the copy's reference names the copy's actor"
        );
        assert_eq!(
            script.blueprint.defaults["banner"],
            serde_json::json!(texture.to_string()),
            "an asset id is not an identity of this document"
        );
    }

    #[test]
    fn save_replaces_existing_file_and_invalid_data_preserves_it() {
        let directory =
            std::env::temp_dir().join(format!("epok-persistence-{}", std::process::id()));
        let path = directory.join("scene.json");
        let mut scene = Scene::default();
        scene.save(&path).unwrap();
        scene.actors[1].position[0] = 7.5;
        scene.save(&path).unwrap();
        assert_eq!(Scene::load(&path).unwrap(), scene);
        scene.actors[1].scale[0] = -1.;
        assert!(scene.save(&path).is_err());
        assert_eq!(Scene::load(&path).unwrap().actors[1].position[0], 7.5);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
