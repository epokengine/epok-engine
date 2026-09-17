//! Original TimelineAsset documents. UUIDs are authoring identities, never runtime handles.
pub use crate::reflection_schema::{Blend, Interpolation};
use crate::{blueprint::Registry, reflection_schema::Type};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub const VERSION: u32 = 3;
pub const ASSET_LIMIT: usize = 32;
pub const SLOT_LIMIT: usize = 8;
pub const TRACK_LIMIT: usize = 16;
pub const KEY_LIMIT: usize = 256;
pub const MARKER_LIMIT: usize = 64;
pub const EVENT_LIMIT: usize = 64;
type Extra = BTreeMap<String, serde_json::Value>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimelineAsset {
    pub version: u32,
    pub id: Uuid,
    pub name: String,
    /// Raw Q12 seconds; the timebase is fixed at 4096 ticks per second in v1.
    pub duration_ticks: i32,
    pub timebase: Timebase,
    pub loop_mode: LoopMode,
    pub slots: Vec<Slot>,
    pub tracks: Vec<Track>,
    pub markers: Vec<Marker>,
    #[serde(default)]
    pub events: Vec<EventTrack>,
    #[serde(default)]
    pub layout: BTreeMap<Uuid, [f32; 2]>,
    #[serde(flatten)]
    pub extra: Extra,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Timebase {
    Q12Seconds,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopMode {
    Once,
    Repeat,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Restore {
    LeaveFinal,
    RestoreInitial,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    pub id: Uuid,
    pub name: String,
    /// Reuses the Blueprint reference type and class ancestry contract.
    pub target: Type,
    pub required: bool,
    #[serde(flatten)]
    pub extra: Extra,
}
impl Slot {
    pub fn class_id(&self) -> Option<&str> {
        match &self.target {
            Type::ObjectRef { class: Some(class) }
            | Type::ActorRef { class: Some(class) }
            | Type::ComponentRef { class: Some(class) }
            | Type::EffectLayerRef { class } => Some(class),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub name: String,
    pub slot: Uuid,
    pub property: String,
    pub value_type: Type,
    pub priority: i16,
    pub blend: Blend,
    pub restore: Restore,
    pub interpolation: Interpolation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<crate::timeline_section::Section>,
    #[serde(default)]
    pub keys: Vec<Key>,
    #[serde(flatten)]
    pub extra: Extra,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Key {
    pub id: Uuid,
    pub tick: i32,
    /// Typed authoring value; Fixed numbers are quantized once during cooking.
    pub value: serde_json::Value,
    #[serde(flatten)]
    pub extra: Extra,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub id: Uuid,
    pub name: String,
    pub tick: i32,
    #[serde(flatten)]
    pub extra: Extra,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventTrack {
    pub id: Uuid,
    pub name: String,
    pub slot: Uuid,
    pub function: String,
    pub keys: Vec<EventKey>,
    #[serde(flatten)]
    pub extra: Extra,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventKey {
    pub id: Uuid,
    pub tick: i32,
    pub arguments: BTreeMap<String, Argument>,
    #[serde(flatten)]
    pub extra: Extra,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Argument {
    Literal {
        value_type: Type,
        value: serde_json::Value,
    },
    Slot {
        slot: Uuid,
    },
}
/// Typed event literals consumed by the existing shared resource cooker.
/// Selection projections use this same traversal; resolution and validation
/// remain with Blueprint resources and the timeline compiler.
pub fn resources(asset: &TimelineAsset) -> impl Iterator<Item = (&Type, &serde_json::Value)> {
    asset
        .events
        .iter()
        .flat_map(|track| &track.keys)
        .flat_map(|key| key.arguments.values())
        .filter_map(|argument| match argument {
            Argument::Literal { value_type, value }
                if matches!(value_type, Type::AssetRef { .. }) =>
            {
                Some((value_type, value))
            }
            _ => None,
        })
}
/// Four raw 32-bit lanes. Unsigned values use their exact bits; vectors use Q12.
/// No host-float value survives cooking.
pub fn pack(value: &serde_json::Value, ty: &Type) -> Result<[i32; 4], String> {
    if !crate::script_values::valid(value, ty) {
        return Err(format!("Value does not match {}", ty.label()));
    }
    let mut lanes = [0; 4];
    match ty {
        Type::Fixed => lanes[0] = quantize(value)?,
        Type::Bool => lanes[0] = i32::from(value.as_bool().unwrap()),
        Type::Int32 | Type::Enum { .. } => {
            lanes[0] = i32::try_from(value.as_i64().unwrap())
                .map_err(|_| "Enum exceeds signed 32-bit storage")?
        }
        Type::UInt32 => lanes[0] = value.as_u64().unwrap() as u32 as i32,
        Type::Vector { length: 2..=3 } => {
            for (i, v) in value.as_array().unwrap().iter().enumerate() {
                lanes[i] = quantize(v)?;
            }
        }
        _ => return Err("No bounded timeline value adapter for this type".into()),
    }
    Ok(lanes)
}
pub fn channels(ty: &Type) -> usize {
    match ty {
        Type::Vector { length } => *length,
        _ => 1,
    }
}
/// This is the serialized binding table for scene/effect components. Only
/// authoring UUIDs enter the file; existing ObjectRef cooking resolves them.
pub type Bindings = BTreeMap<Uuid, Option<Uuid>>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub asset: Uuid,
    pub item: Uuid,
    pub required: bool,
    pub message: String,
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Timeline {} / {}: {}",
            self.asset, self.item, self.message
        )
    }
}
fn validate_components(
    class: &str,
    entity: &crate::scene::Actor,
    registry: &Registry,
) -> Result<(), String> {
    use crate::reflection_schema::TimelineComponentRequirement as Component;
    let class = registry
        .classes
        .get(class)
        .ok_or("Missing timeline target class")?;
    for requirement in registry
        .ancestry(&class.cpp_name)
        .iter()
        .filter_map(|class| class.timeline_component)
    {
        let available = match requirement {
            Component::Camera => entity.kind == "Camera",
            Component::AudioSource => entity.audio.is_some(),
            Component::Light => entity
                .light
                .as_ref()
                .is_some_and(|component| component.enabled),
            Component::PaletteAnimator => entity
                .palette_animator
                .as_ref()
                .is_some_and(|component| component.enabled),
            Component::ParticleEmitter => entity
                .particle_emitter
                .as_ref()
                .is_some_and(|component| component.enabled),
            Component::RectTransform => entity.rect.is_some(),
            Component::Text => entity
                .text
                .as_ref()
                .is_some_and(|component| component.enabled),
            Component::Image => entity
                .image
                .as_ref()
                .is_some_and(|component| component.enabled),
            Component::ProgressBar => entity
                .progress
                .as_ref()
                .is_some_and(|component| component.enabled),
        };
        if !available {
            return Err(format!(
                "Timeline adapter {} requires an enabled {requirement:?} component on {}",
                class.cpp_name, entity.name
            ));
        }
    }
    Ok(())
}
impl TimelineAsset {
    pub fn identities(&self) -> impl Iterator<Item = Uuid> + '_ {
        std::iter::once(self.id)
            .chain(self.slots.iter().map(|x| x.id))
            .chain(self.tracks.iter().map(|x| x.id))
            .chain(self.tracks.iter().flat_map(|t| t.keys.iter().map(|k| k.id)))
            .chain(
                self.tracks
                    .iter()
                    .flat_map(|t| t.sections.iter())
                    .flat_map(|s| {
                        std::iter::once(s.id).chain(
                            s.channels.iter().flat_map(|c| {
                                std::iter::once(c.id).chain(c.keys.iter().map(|k| k.id))
                            }),
                        )
                    }),
            )
            .chain(self.markers.iter().map(|x| x.id))
            .chain(self.events.iter().map(|e| e.id))
            .chain(self.events.iter().flat_map(|e| e.keys.iter().map(|k| k.id)))
    }
    pub fn migrate(&mut self) -> Result<(), String> {
        if ![1, 2, VERSION].contains(&self.version) {
            return Err(format!(
                "Unsupported timeline version {}; original preserved",
                self.version
            ));
        }
        self.version = VERSION;
        Ok(())
    }
    pub fn new(name: String) -> Self {
        Self {
            version: VERSION,
            id: Uuid::new_v4(),
            name,
            duration_ticks: 4096,
            timebase: Timebase::Q12Seconds,
            loop_mode: LoopMode::Once,
            slots: vec![],
            tracks: vec![],
            markers: vec![],
            events: vec![],
            layout: BTreeMap::new(),
            extra: BTreeMap::new(),
        }
    }
    pub fn semantic_hash(&self) -> String {
        let mut a = self.clone();
        a.layout.clear();
        a.name.clear();
        a.slots.sort_by_key(|s| s.id);
        a.tracks.sort_by_key(|t| t.id);
        a.markers.sort_by_key(|m| m.id);
        a.events.sort_by_key(|e| e.id);
        for event in &mut a.events {
            event.name.clear();
            event.keys.sort_by_key(|k| (k.tick, k.id));
        }
        for s in &mut a.slots {
            s.name.clear();
        }
        for t in &mut a.tracks {
            t.name.clear();
            t.keys.sort_by_key(|k| (k.tick, k.id));
            t.sections.sort_by_key(|s| (s.start_tick, s.id));
            for section in &mut t.sections {
                section.channels.sort_by_key(|c| c.lane);
                for channel in &mut section.channels {
                    channel.keys.sort_by_key(|k| (k.tick, k.id));
                }
            }
        }
        for m in &mut a.markers {
            m.name.clear();
        }
        crate::assets::hash(&serde_json::to_vec(&a).expect("serializable timeline"))
    }
    pub fn validate(&self, registry: &Registry) -> Vec<Diagnostic> {
        let mut errors = Vec::new();
        let mut report = |item, message: String| {
            errors.push(Diagnostic {
                asset: self.id,
                item,
                required: true,
                message,
            })
        };
        if self.version != VERSION {
            report(
                self.id,
                format!("Unsupported version {}; source preserved", self.version),
            );
        }
        for (id, extra) in std::iter::once((self.id, &self.extra))
            .chain(self.slots.iter().map(|s| (s.id, &s.extra)))
            .chain(self.tracks.iter().map(|t| (t.id, &t.extra)))
            .chain(
                self.tracks
                    .iter()
                    .flat_map(|t| t.keys.iter().map(|k| (k.id, &k.extra))),
            )
            .chain(
                self.tracks
                    .iter()
                    .flat_map(|t| t.sections.iter())
                    .flat_map(|s| s.channels.iter())
                    .flat_map(|c| c.keys.iter().map(|k| (k.id, &k.extra))),
            )
            .chain(
                self.tracks
                    .iter()
                    .flat_map(|t| t.sections.iter())
                    .flat_map(|s| {
                        std::iter::once((s.id, &s.extra))
                            .chain(s.channels.iter().map(|c| (c.id, &c.extra)))
                    }),
            )
            .chain(self.markers.iter().map(|m| (m.id, &m.extra)))
        {
            if !extra.is_empty() {
                report(
                    id,
                    "Unsupported fields preserved; migrate them before cooking".into(),
                );
            }
        }
        if self.duration_ticks <= 0 {
            report(self.id, "Duration must be positive raw Q12 ticks".into());
        }
        if self.slots.len() > SLOT_LIMIT
            || self.tracks.len() + self.events.len() > TRACK_LIMIT
            || self.markers.len() > MARKER_LIMIT
            || self.events.iter().map(|e| e.keys.len()).sum::<usize>() > EVENT_LIMIT
        {
            report(
                self.id,
                "Asset exceeds 8 slots, 16 tracks or 64 markers".into(),
            );
        }
        let mut identities = BTreeSet::new();
        for id in self.identities() {
            if id.is_nil() || !identities.insert(id) {
                report(id, "Missing or duplicate persistent UUID".into());
            }
        }
        for slot in &self.slots {
            match &slot.target {
                Type::ObjectRef { class: Some(class) }|Type::ActorRef {class:Some(class)}|Type::ComponentRef {class:Some(class)} if class != crate::particle_effect::LAYER_CLASS_ID && registry.classes.contains_key(class) => {}
                Type::EffectLayerRef { class } if class == crate::particle_effect::LAYER_CLASS_ID && registry.classes.contains_key(class) => {}
                _ => report(
                    slot.id,
                    "Target requires a registered ObjectRef class or the reflected EffectLayerRef class ID".into(),
                ),
            }
        }
        for track in &self.tracks {
            let property = self
                .slots
                .iter()
                .find(|s| s.id == track.slot)
                .and_then(|s| s.class_id().and_then(|id| registry.classes.get(id)))
                .and_then(|class| {
                    registry
                        .properties(&class.cpp_name)
                        .into_iter()
                        .find(|p| p.id == track.property)
                });
            match property {
                None => report(
                    track.id,
                    format!(
                        "Missing slot/class/property ID {}; no name fallback",
                        track.property
                    ),
                ),
                Some(p) => {
                    if p.timeline
                        .as_ref()
                        .is_none_or(|profile| !profile.allows(track.interpolation, track.blend))
                        || !p.editable
                        || track.value_type != p.value_type
                        || crate::reflection_schema::TimelineProperty::for_type(&p.value_type)
                            .is_none_or(|profile| !profile.allows(track.interpolation, track.blend))
                    {
                        report(
                            track.id,
                            format!(
                                "Property {} must explicitly expose the matching type, interpolation and blend profile ({}:{})",
                                p.id,
                                p.source.file.display(),
                                p.source.line
                            ),
                        );
                    }
                }
            }
            for (item, error) in crate::timeline_section::validate(track, self.duration_ticks) {
                report(item, error);
            }
            if track.sections.is_empty() && !(2..=KEY_LIMIT).contains(&track.keys.len()) {
                report(track.id, "A curve requires 2–256 keys".into());
            }
            let mut keys = track.keys.iter().collect::<Vec<_>>();
            keys.sort_by_key(|k| (k.tick, k.id));
            for (i, key) in keys.iter().enumerate() {
                if key.tick < 0
                    || key.tick > self.duration_ticks
                    || (i > 0 && keys[i - 1].tick == key.tick)
                {
                    report(
                        key.id,
                        "Keys need unique times inside the asset duration".into(),
                    );
                }
                if let Err(error) = pack(&key.value, &track.value_type) {
                    report(key.id, error);
                }
            }
        }
        if self
            .tracks
            .iter()
            .map(|t| t.sections.len().max(1))
            .sum::<usize>()
            > TRACK_LIMIT
        {
            report(
                self.id,
                "Cooked property sections exceed the 16-entry PSX profile".into(),
            );
        }
        for (i, track) in self.tracks.iter().enumerate() {
            for other in &self.tracks[..i] {
                if track.slot == other.slot
                    && track.property == other.property
                    && track.restore != other.restore
                {
                    report(
                        track.id,
                        format!(
                            "Tracks sharing a property must agree on restoration (track {})",
                            other.id
                        ),
                    );
                }
                if track.slot == other.slot
                    && track.property == other.property
                    && track.priority == other.priority
                    && track.blend == Blend::Absolute
                    && other.blend == Blend::Absolute
                    && track.ranges(self.duration_ticks).iter().any(|a| {
                        other
                            .ranges(self.duration_ticks)
                            .iter()
                            .any(|b| a.0 < b.1 && b.0 < a.1)
                    })
                {
                    // Tracks hold endpoint values outside their keys, so they overlap
                    // for the entire sequence, including before their first key.
                    report(
                        track.id,
                        format!(
                            "Absolute write conflict with track {} at priority {}",
                            other.id, track.priority
                        ),
                    );
                }
            }
        }
        for marker in &self.markers {
            if marker.tick < 0 || marker.tick > self.duration_ticks {
                report(marker.id, "Marker is outside the asset duration".into());
            }
        }
        for event in &self.events {
            if event.keys.is_empty() {
                report(event.id, "Event tracks require at least one key".into());
            }
            if !event.extra.is_empty() {
                report(event.id, "Unsupported event fields preserved".into());
            }
            let function = self
                .slots
                .iter()
                .find(|s| s.id == event.slot)
                .and_then(|s| s.class_id().and_then(|id| registry.classes.get(id)))
                .and_then(|class| {
                    registry
                        .ancestry(&class.cpp_name)
                        .into_iter()
                        .rev()
                        .flat_map(|c| c.functions.iter())
                        .find(|f| f.id == event.function || f.overrides.contains(&event.function))
                });
            let Some(function) = function else {
                report(
                    event.id,
                    format!(
                        "Missing timeline function ID {}; no name fallback",
                        event.function
                    ),
                );
                continue;
            };
            if function.timeline.is_none()
                || function.access != "public"
                || function.returns != Type::Void
                || function.parameters.len() > 4
            {
                report(event.id,"Function requires explicit timeline exposure, public void signature and at most four arguments".into());
            }
            for key in &event.keys {
                if key.tick < 0 || key.tick > self.duration_ticks || !key.extra.is_empty() {
                    report(
                        key.id,
                        "Invalid event timestamp or unsupported fields preserved".into(),
                    );
                }
                if key.arguments.len() != function.parameters.len() {
                    report(
                        key.id,
                        "Event arguments differ from the reflected signature".into(),
                    );
                }
                for parameter in &function.parameters {
                    if parameter.direction == crate::reflection_schema::Direction::MutableReference
                    {
                        report(
                            key.id,
                            "Timeline arguments cannot be mutable references".into(),
                        );
                    }
                    let valid = match key.arguments.get(&parameter.name) {
                        Some(Argument::Literal { value_type, value }) => {
                            value_type == &parameter.value_type
                                && (pack(value, value_type).is_ok()
                                    || matches!(value_type, Type::AssetRef { kind } if matches!(kind.as_str(), "Texture" | "AudioClip" | "PlayableAudio" | "MusicSequence"))
                                        && crate::script_values::valid(value, value_type))
                        }
                        Some(Argument::Slot { slot }) => {
                            self.slots.iter().find(|s| s.id == *slot).is_some_and(|s| {
                                crate::blueprint_ir::assignable(
                                    &s.target,
                                    &parameter.value_type,
                                    registry,
                                )
                            })
                        }
                        _ => false,
                    };
                    if !valid {
                        report(
                            key.id,
                            format!("Missing/incompatible event argument {}", parameter.name),
                        );
                    }
                }
            }
        }
        errors
    }
    pub fn validate_bindings(
        &self,
        bindings: &Bindings,
        scene: &crate::scene::Scene,
        registry: &Registry,
    ) -> Vec<Diagnostic> {
        let mut errors = vec![];
        for slot in &self.slots {
            let value = bindings
                .get(&slot.id)
                .copied()
                .flatten()
                .map_or(serde_json::Value::Null, |id| {
                    serde_json::json!(id.to_string())
                });
            let result = if value.is_null() {
                Err("Binding is empty".into())
            } else {
                crate::blueprint_refs::assignment(
                    "timeline_target",
                    &value,
                    &slot.target,
                    scene,
                    registry,
                )
                .and_then(|_| {
                    if let Some(entity) = bindings
                        .get(&slot.id)
                        .copied()
                        .flatten()
                        .and_then(|id| scene.actors.iter().find(|entity| entity.id == id))
                        && let Some(class) = slot.class_id()
                    {
                        validate_components(class, entity, registry)?;
                    }
                    Ok(())
                })
            };
            if let Err(message) = result {
                errors.push(Diagnostic {
                    asset: self.id,
                    item: slot.id,
                    required: slot.required,
                    message,
                });
            } else if let Some(id) = bindings.get(&slot.id).copied().flatten()
                && let Some(index) = scene.actors.iter().position(|e| e.id == id)
                && !scene.is_active(index)
            {
                errors.push(Diagnostic {
                    asset: self.id,
                    item: slot.id,
                    required: false,
                    message: "Bound entity is inactive; runtime work will be skipped".into(),
                });
            }
        }
        for id in bindings
            .keys()
            .filter(|id| !self.slots.iter().any(|s| s.id == **id))
        {
            errors.push(Diagnostic {
                asset: self.id,
                item: *id,
                required: false,
                message: "Orphaned binding preserved for repair".into(),
            });
        }
        for (i, track) in self.tracks.iter().enumerate() {
            let target = bindings.get(&track.slot).copied().flatten();
            for other in &self.tracks[..i] {
                if track.slot != other.slot
                    && target.is_some()
                    && target == bindings.get(&other.slot).copied().flatten()
                    && track.property == other.property
                    && track.restore != other.restore
                {
                    errors.push(Diagnostic {
                        asset: self.id,
                        item: track.id,
                        required: true,
                        message: format!(
                            "Aliased tracks must agree on restoration (track {})",
                            other.id
                        ),
                    });
                }
                if track.slot != other.slot
                    && target.is_some()
                    && target == bindings.get(&other.slot).copied().flatten()
                    && track.property == other.property
                    && track.priority == other.priority
                    && track.blend == Blend::Absolute
                    && other.blend == Blend::Absolute
                    && track.ranges(self.duration_ticks).iter().any(|a| {
                        other
                            .ranges(self.duration_ticks)
                            .iter()
                            .any(|b| a.0 < b.1 && b.0 < a.1)
                    })
                {
                    errors.push(Diagnostic {
                        asset: self.id,
                        item: track.id,
                        required: true,
                        message: format!(
                            "Bindings alias the same entity and conflict with track {}",
                            other.id
                        ),
                    });
                }
            }
        }
        errors
    }
}

#[cfg(test)]
#[path = "timeline_tests.rs"]
mod tests;
pub fn quantize(value: &serde_json::Value) -> Result<i32, String> {
    let v = value.as_f64().ok_or("Expected a Fixed number")?;
    let raw = (v * 4096.).round();
    if !v.is_finite() || !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&raw) {
        return Err("Fixed value is out of range".into());
    }
    Ok(raw as i32)
}
pub fn load(path: &Path) -> Result<TimelineAsset, String> {
    let bytes = crate::assets::read_bounded(path)?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("Timeline source exceeds 2 MiB".into());
    }
    let mut asset: TimelineAsset =
        serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    asset.migrate()?;
    Ok(asset)
}
pub fn load_all(root: &Path) -> Result<Vec<(PathBuf, TimelineAsset)>, String> {
    let catalog = inspect_all(root)?;
    if !catalog.errors.is_empty() {
        return Err(catalog.errors.join("\n"));
    }
    let mut seen = BTreeSet::new();
    for (_, asset) in &catalog.entries {
        if !seen.insert(asset.id) {
            return Err(format!("Duplicate TimelineAsset UUID {}", asset.id));
        }
    }
    Ok(catalog.entries)
}

/// Partial source observations use the same typed loader as strict cooking.
/// A broken document must not hide independent, successfully loaded identities.
pub struct Catalog<T> {
    pub entries: Vec<(PathBuf, T)>,
    pub errors: Vec<String>,
}
pub fn inspect_all(root: &Path) -> Result<Catalog<TimelineAsset>, String> {
    let (paths, errors) = crate::assets::source_paths(root, ".timeline.json");
    if paths.len() > ASSET_LIMIT {
        return Err("Project exceeds 32 authored timelines".into());
    }
    let mut result = Catalog {
        entries: Vec::new(),
        errors,
    };
    for path in paths {
        match load(&path) {
            Ok(asset) => result.entries.push((path, asset)),
            Err(error) => result.errors.push(format!("{}: {error}", path.display())),
        }
    }
    Ok(result)
}
pub fn catalog_id_counts(
    timelines: &Catalog<TimelineAsset>,
    effects: &Catalog<crate::particle_effect::ParticleEffect>,
) -> BTreeMap<Uuid, usize> {
    let mut counts = BTreeMap::new();
    for (_, asset) in &timelines.entries {
        *counts.entry(asset.id).or_default() += 1;
    }
    for (_, effect) in &effects.entries {
        *counts.entry(effect.id).or_default() += 1;
        *counts.entry(effect.timeline.id).or_default() += 1;
    }
    counts
}
/// A transient view of the existing typed source loaders, shared by pickers and
/// observers. Ambiguous identities cannot be selected; errors remain visible.
pub struct PlaybackSources {
    pub timelines: Vec<(PathBuf, TimelineAsset)>,
    pub effects: Vec<(PathBuf, crate::particle_effect::ParticleEffect)>,
    pub errors: Vec<String>,
}
pub fn inspect_playback(root: &Path) -> Result<PlaybackSources, String> {
    let timelines = inspect_all(root)?;
    let effects = crate::particle_effect::inspect_all(root)?;
    let counts = catalog_id_counts(&timelines, &effects);
    let mut result = PlaybackSources {
        timelines: vec![],
        effects: vec![],
        errors: timelines.errors.into_iter().chain(effects.errors).collect(),
    };
    for (path, asset) in timelines.entries {
        if counts[&asset.id] == 1 {
            result.timelines.push((path, asset));
        } else {
            result.errors.push(format!(
                "{}: ambiguous TimelineAsset UUID {}",
                path.display(),
                asset.id
            ));
        }
    }
    for (path, effect) in effects.entries {
        if counts[&effect.id] == 1 && counts[&effect.timeline.id] == 1 {
            result.effects.push((path, effect));
        } else {
            result.errors.push(format!(
                "{}: ambiguous ParticleEffect/TimelineAsset UUID {}/{}",
                path.display(),
                effect.id,
                effect.timeline.id
            ));
        }
    }
    Ok(result)
}
/// Return current unambiguous candidates. Callers diagnose absent references at
/// their own component/node location; unrelated malformed files are not inputs.
pub fn load_referenced(
    root: &Path,
    required: &BTreeSet<Uuid>,
    embedded: bool,
) -> Result<Vec<(PathBuf, TimelineAsset)>, String> {
    let timelines = inspect_all(root)?;
    let effects = crate::particle_effect::inspect_all(root)?;
    let counts = catalog_id_counts(&timelines, &effects);
    let mut result = Vec::new();
    for (path, asset) in timelines.entries {
        if required.contains(&asset.id) {
            if counts[&asset.id] != 1 {
                return Err(format!("Duplicate TimelineAsset UUID {}", asset.id));
            }
            result.push((path, asset));
        }
    }
    if embedded {
        for (path, effect) in effects.entries {
            if required.contains(&effect.timeline.id) {
                if counts[&effect.id] != 1 || counts[&effect.timeline.id] != 1 {
                    return Err(format!(
                        "Ambiguous ParticleEffect/TimelineAsset UUID {}/{}",
                        effect.id, effect.timeline.id
                    ));
                }
                result.push((path, effect.timeline));
            }
        }
    }
    Ok(result)
}
pub fn create(root: &Path, name: &str) -> Result<PathBuf, String> {
    if load_all(root)?.len() >= ASSET_LIMIT {
        return Err("Project exceeds 32 authored timelines".into());
    }
    crate::workspace::validate_name(name)?;
    if !crate::scripts::identifier(name) {
        return Err("Use a portable identifier for the timeline filename".into());
    }
    let path = crate::assets::inside(root, &format!("assets/Timelines/{name}.timeline.json"))?;
    fs::create_dir_all(path.parent().ok_or("Missing asset directory")?)
        .map_err(|e| e.to_string())?;
    let asset = TimelineAsset::new(name.into());
    crate::assets::atomic_write(
        &path,
        &serde_json::to_vec_pretty(&asset).map_err(|e| e.to_string())?,
        None,
    )?;
    Ok(path)
}
