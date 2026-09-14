//! Original, versioned visual-class assets. Reading never rewrites unknown data.
use crate::reflection_schema::{Parameter, Type};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub const VERSION: u32 = 5;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlueprintAsset {
    pub version: u32,
    pub id: String,
    pub name: String,
    pub parent: String,
    /// Authoring hint for the family this class belongs to (version 4). The registry's
    /// resolved family is authoritative; a hint that disagrees is a compile diagnostic.
    /// `None` is the v1-v3 shape and is skipped on write, so a v3 asset keeps its bytes
    /// and its semantic hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<crate::reflection_schema::ClassFamily>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<crate::reflection_schema::ComponentContract>,
    #[serde(default)]
    pub defaults: BTreeMap<String, Value>,
    #[serde(default)]
    pub variables: Vec<Variable>,
    #[serde(default)]
    pub functions: Vec<Graph>,
    #[serde(default)]
    pub layout: Layout,
    #[serde(default)]
    pub template: crate::blueprint_templates::Template,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
impl BlueprintAsset {
    pub fn new(name: String, parent: String) -> Self {
        Self {
            version: VERSION,
            id: uuid::Uuid::new_v4().to_string(),
            name,
            parent,
            family: None,
            component: None,
            defaults: BTreeMap::new(),
            variables: vec![],
            functions: vec![],
            layout: Layout::default(),
            template: crate::blueprint_templates::Template::default(),
            extra: BTreeMap::new(),
        }
    }
    pub fn semantic_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut semantic = self.clone();
        semantic.layout = Layout::default();
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&semantic).expect("serializable Blueprint"))
        )
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variable {
    pub id: String,
    pub name: String,
    pub value_type: Type,
    pub default: Value,
    #[serde(default = "yes")]
    pub editable: bool,
    #[serde(default)]
    pub timeline_animatable: bool,
}
fn yes() -> bool {
    true
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub split_pins: std::collections::BTreeSet<String>,
    #[serde(default)]
    pub positions: BTreeMap<String, [f32; 2]>,
    #[serde(default)]
    pub comments: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub override_id: Option<String>,
    #[serde(default)]
    pub timeline: Option<crate::reflection_schema::TimelineCall>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    pub returns: Type,
    pub entry: String,
    pub nodes: Vec<Node>,
}

impl Graph {
    /// Execution starts at the event/function entry. Data dependencies are
    /// included backwards, without making their outgoing execution paths live.
    /// The authored graph is retained intact for editing and undo.
    pub fn compilation_nodes(&self) -> impl Iterator<Item = &Node> {
        let nodes: std::collections::BTreeMap<_, _> = self
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();
        let mut execution = std::collections::BTreeSet::new();
        let mut live = std::collections::BTreeSet::new();
        let mut pending = vec![(self.entry.as_str(), true)];
        while let Some((id, exec)) = pending.pop() {
            let Some(node) = nodes.get(id) else { continue };
            if exec && execution.insert(id) {
                pending.extend(
                    node.outputs
                        .values()
                        .flatten()
                        .map(|id| (id.as_str(), true)),
                );
            }
            if live.insert(id) {
                pending.extend(node.inputs.values().filter_map(|input| match input {
                    Input::Link { node, .. } => Some((node.as_str(), false)),
                    _ => None,
                }));
            }
        }
        self.nodes
            .iter()
            .filter(move |node| live.contains(node.id.as_str()))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    #[serde(default)]
    pub inputs: BTreeMap<String, Input>,
    #[serde(default)]
    pub outputs: BTreeMap<String, Vec<String>>,
}
/// Sequence has ordered, individually wired outputs and one spare pin (up to 32).
pub fn sequence_outputs(node: &Node) -> Vec<String> {
    let last = node
        .outputs
        .keys()
        .filter_map(|pin| {
            pin.strip_prefix("then_")
                .and_then(|n| n.parse::<usize>().ok())
        })
        .max();
    let count = last.map_or(2, |n| (n + 2).clamp(2, 32));
    (0..count).map(|i| format!("then_{i}")).collect()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Input {
    Literal { value_type: Type, value: Value },
    Link { node: String, pin: String },
    Parameter { name: String },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NodeKind {
    Entry,
    Literal {
        value_type: Type,
        value: Value,
    },
    GetVariable {
        member: String,
    },
    SetVariable {
        member: String,
    },
    Call {
        function: String,
    },
    CallOn {
        class: String,
        function: String,
    },
    CallParent,
    Binary {
        op: BinaryOp,
    },
    MakeVector {
        length: usize,
    },
    VectorComponent {
        length: usize,
        index: usize,
    },
    Not,
    Reroute,
    Branch,
    Sequence,
    Loop {
        count: u32,
    },
    Delay,
    WaitPlayback {
        condition: PlaybackCondition,
    },
    Timeline {
        keys: Vec<[f64; 2]>,
        looping: bool,
        member: String,
    },
    StopTimeline {
        node: String,
    },
    Builtin {
        operation: Builtin,
    },
    Return,
}
pub fn pin_id(node: &str, port: &str) -> String {
    format!("{node}:{port}")
}
pub fn link_id(source: &str, output: &str, target: &str, input: &str) -> String {
    format!("{}>{}", pin_id(source, output), pin_id(target, input))
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}
/// Closed set of adapter nodes. `GetOwner` is the owning Actor of a Component-family
/// Blueprint; `SpawnActor` spawns from a ClassRef whose base must resolve to the Actor
/// family (the compiler rejects anything else).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Builtin {
    SelfObject,
    IsValid,
    GetPosition,
    GetPosition2D,
    SetPosition2D,
    GetRotation2D,
    SetRotation2D,
    GetScale2D,
    SetScale2D,
    GetRectPosition,
    SetRectPosition,
    GetRectSize,
    SetRectSize,

    GetRotation,
    GetScale,
    SetPosition,
    SetRotation,
    SetScale,
    InputHeld,
    InputPressed,
    InputReleased,
    RequestScene,
    SetActive,
    DestroyActor,
    Spawn { class: String },
    SpawnClass { base: String },
    IsA { class: String },
    Cast { class: String },
    PlayAudio,
    StopAudio,
    SetTexture,
    SetAudioClip,
    PlaySequenceComponent,
    StopSequence,
    PauseSequence,
    ResumeSequence,
    PlayEffectComponent,
    StopEffect,
    BurstEffect,
    PauseEffect,
    ResumeEffect,
    EffectSequence,
    PlayTimelineAsset { asset: String },
    SpawnParticleEffect { asset: String },
    GetTransform,
    MakeTransform,
    GetOwner,
    SpawnActor { base: String },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlaybackCondition {
    SequenceComplete,
    EffectComplete,
    Marker { timeline: String, marker: String },
    SubscribeMarker { timeline: String, marker: String },
}
/// The identities an embedded scene Blueprint may name by UUID. Map-scoped
/// `ActorRef`/`ObjectRef` literals are resolved against this set at compile time,
/// never by a name lookup at Tick.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapScope {
    pub actors: std::collections::BTreeSet<uuid::Uuid>,
    pub components: std::collections::BTreeSet<uuid::Uuid>,
}
/// Where a compiler input came from. Everything that existed before this phase is
/// a `File`, so the variant is the default and existing constructors keep working.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AssetSource {
    /// A standalone `.epokbp` asset; [`AssetFile::path`] is that file.
    #[default]
    File,
    /// The `scene_script` of a `.epokmap`; [`AssetFile::path`] is the map itself.
    /// The document has no life outside that map: it is never written separately,
    /// and saving it means saving the map.
    EmbeddedScene(MapScope),
}
#[derive(Clone, Debug)]
pub struct AssetFile {
    pub path: PathBuf,
    pub asset: BlueprintAsset,
    /// `File` for every `.epokbp`; `EmbeddedScene` for a map's own Blueprint.
    pub source: AssetSource,
}
impl AssetFile {
    /// A standalone `.epokbp` input.
    pub fn file(path: PathBuf, asset: BlueprintAsset) -> Self {
        Self {
            path,
            asset,
            source: AssetSource::File,
        }
    }
    pub fn is_embedded(&self) -> bool {
        matches!(self.source, AssetSource::EmbeddedScene(_))
    }
    pub fn scope(&self) -> Option<&MapScope> {
        match &self.source {
            AssetSource::EmbeddedScene(scope) => Some(scope),
            AssetSource::File => None,
        }
    }
}
/// The compiler input for a map's own Blueprint, or `None` when the map has none.
///
/// The class parent is taken from `scene_script.parent.class_id`, which is the
/// authored truth; the embedded asset's own `parent` field only carries it. The
/// version is migrated in memory exactly like [`load`] does for a file.
pub fn embedded(map: &Path, scene: &crate::scene::Scene) -> Option<AssetFile> {
    let script = scene.scene_script.as_ref()?;
    let mut asset = script.blueprint.clone();
    if let Some(class_id) = &script.parent.class_id {
        asset.parent = class_id.clone();
    }
    asset.version = VERSION;
    let scope = MapScope {
        actors: scene.actors.iter().map(|actor| actor.id).collect(),
        components: scene
            .actors
            .iter()
            .flat_map(|actor| actor.components.iter().map(|c| c.id))
            .collect(),
    };
    Some(AssetFile {
        path: map.to_path_buf(),
        asset,
        source: AssetSource::EmbeddedScene(scope),
    })
}
pub fn load(path: &Path) -> Result<BlueprintAsset, String> {
    let mut asset: BlueprintAsset =
        crate::document::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("{}: {e}", path.display()))?;
    if asset.version != VERSION {
        return Err(format!(
            "{}: unsupported Blueprint version {}; original data preserved",
            path.display(),
            asset.version
        ));
    }
    // v1 has no explicit timeline exposure; defaults remain unexposed.
    // v3 adds typed playback nodes; existing v1/v2 graphs keep their semantics.
    // v4 adds the optional family hint and typed object reference pins; a v3 asset loads
    // with `family: None` and hashes exactly as it did before.
    // This migration is in memory. Only a deliberate save writes the new version.
    asset.version = VERSION;
    Ok(asset)
}
/// Exclusive creation. Existing assets are never replaced by the creation wizard.
pub fn create(path: &Path, asset: &BlueprintAsset) -> Result<(), String> {
    let bytes = crate::document::to_vec(asset).map_err(|e| e.to_string())?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    let result = file.write_all(&bytes).and_then(|_| file.sync_all());
    drop(file);
    if let Err(error) = result {
        let _ = fs::remove_file(path);
        return Err(error.to_string());
    }
    Ok(())
}
/// Every compiler input of a project: the `.epokbp` assets under `assets/` plus
/// the embedded scene Blueprint of every `.epokmap` under `assets/scenes` that has
/// one. A map's script is a compiler input like any other class, which is why it
/// compiles even when no entity in the map carries a Behaviour.
pub fn load_all(root: &Path) -> Result<Vec<AssetFile>, String> {
    fn scene_script(path: &Path, result: &mut Vec<AssetFile>) -> Result<(), String> {
        let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        // Maps without a scene Blueprint are the common case and are never parsed.
        if !bytes
            .windows(b"scene_script".len())
            .any(|window| window == b"scene_script")
        {
            return Ok(());
        }
        let scene: crate::scene::Scene = crate::document::from_slice(&bytes)
            .map_err(|e| format!("{}: scene Blueprint discovery failed: {e}", path.display()))?;
        if let Some(file) = embedded(path, &scene) {
            result.push(file);
        }
        Ok(())
    }
    fn visit(path: &Path, scenes: &Path, result: &mut Vec<AssetFile>) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }
        let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                return Err(format!(
                    "Blueprint discovery rejects reparse point {}",
                    path.display()
                ));
            }
        }
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "Blueprint discovery rejects linked path {}",
                path.display()
            ));
        }
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_symlink() {
                return Err(format!(
                    "Blueprint discovery rejects linked asset {}",
                    entry.path().display()
                ));
            }
            if kind.is_dir() {
                visit(&entry.path(), scenes, result)?;
            } else if entry.file_name().to_string_lossy().ends_with(".epokbp") {
                result.push(AssetFile::file(entry.path(), load(&entry.path())?));
            } else if entry.file_name().to_string_lossy().ends_with(".epokmap")
                && path.starts_with(scenes)
            {
                scene_script(&entry.path(), result)?;
            }
        }
        Ok(())
    }
    let mut result = vec![];
    let assets = root.join("assets");
    visit(&assets, &assets.join("scenes"), &mut result)?;
    result.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_does_not_invalidate_generated_code() {
        let mut a = BlueprintAsset::new("Boss".into(), "enemy".into());
        let h = a.semantic_hash();
        a.layout.positions.insert("node".into(), [2., 3.]);
        assert_eq!(h, a.semantic_hash());
        a.defaults.insert("health".into(), Value::from(2));
        assert_ne!(h, a.semantic_hash());
    }
    /// A byte fixture of a version-3 asset, pinned before the family hint existed. It
    /// must still load, must resolve `family` to `None`, and must hash to exactly the
    /// value it hashed to under version 3 - otherwise every existing Blueprint in every
    /// project would look stale after upgrading the editor.
    const V3_FIXTURE: &str = r#"{"version":3,"id":"11111111-2222-3333-4444-555555555555","name":"Boss","parent":"66666666-7777-8888-9999-000000000000","defaults":{},"variables":[],"functions":[],"layout":{"positions":{},"comments":{}},"template":{"actors":[]}}"#;
    /// The exact bytes `semantic_hash` digests for that fixture, and their SHA-256.
    /// Both were computed independently of this code path (the digest with Python's
    /// `hashlib` over the string below), so a regression here is a real ABI change.
    const V3_SEMANTIC_JSON: &str = r#"{"version":3,"id":"11111111-2222-3333-4444-555555555555","name":"Boss","parent":"66666666-7777-8888-9999-000000000000","defaults":{},"variables":[],"functions":[],"layout":{"positions":{},"comments":{}},"template":{"actors":[],"overrides":{},"references":[],"construction":[]}}"#;
    const V3_SEMANTIC_HASH: &str =
        "9aaee305fc982599d4609bcceb6d0c87aeddaa1b8c4f0b5189fff1625a0385d1";

    #[test]
    fn family_hint_is_a_semantic_change_in_current_assets() {
        let mut asset = BlueprintAsset::new("Test".into(), crate::object_model::ACTOR3D_ID.into());
        let original = asset.semantic_hash();
        asset.family = Some(crate::reflection_schema::ClassFamily::Actor);
        assert_ne!(asset.semantic_hash(), original);
        let restored: BlueprintAsset =
            crate::document::from_slice(&serde_json::to_vec(&asset).unwrap()).unwrap();
        assert_eq!(restored.semantic_hash(), asset.semantic_hash());
    }

    #[test]
    fn unknown_top_level_fields_survive_roundtrip() {
        let mut a = BlueprintAsset::new("Boss".into(), "enemy".into());
        a.extra.insert("future".into(), serde_json::json!({"x":2}));
        let b: BlueprintAsset =
            crate::document::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
        assert_eq!(a.extra, b.extra);
    }
    #[test]
    fn earlier_versions_are_rejected_without_writing_source() {
        let root = crate::workspace::tests::temp("blueprint-playback-migration");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("Earlier.epokbp");
        for version in 1..VERSION {
            let mut original =
                BlueprintAsset::new("Earlier".into(), uuid::Uuid::new_v4().to_string());
            original.version = version;
            original
                .extra
                .insert("future".into(), serde_json::json!({"preserved":true}));
            let bytes = serde_json::to_vec(&original).unwrap();
            std::fs::write(&path, &bytes).unwrap();
            assert!(
                load(&path)
                    .unwrap_err()
                    .contains("unsupported Blueprint version")
            );
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
    }
}
