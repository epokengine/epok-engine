//! Reusable effect source data. Its timeline is the shared TimelineAsset type.
use crate::{
    blueprint::Registry,
    particles::Emitter,
    sprites::Sprite,
    timeline::{self, TimelineAsset},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use uuid::Uuid;

pub const VERSION: u32 = 1;
pub const LAYER_LIMIT: usize = 8;
pub const LAYER_CLASS_ID: &str = "27550d6d-5bba-4618-9d9e-33b9605bfb6c";
type Extra = BTreeMap<String, serde_json::Value>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParticleEffect {
    pub version: u32,
    pub id: Uuid,
    pub name: String,
    pub seed: u32,
    pub layers: Vec<Layer>,
    pub timeline: TimelineAsset,
    #[serde(flatten)]
    pub extra: Extra,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: Uuid,
    pub name: String,
    /// Persistent slot UUID in the embedded TimelineAsset, not a runtime index.
    pub slot: Uuid,
    pub enabled: bool,
    pub position: [f32; 3],
    pub content: LayerContent,
    #[serde(flatten)]
    pub extra: Extra,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LayerContent {
    Sprite {
        sprite: Sprite,
        frames: u16,
        columns: u16,
        frame_ticks: i32,
    },
    Emitter {
        emitter: Box<Emitter>,
    },
}
impl ParticleEffect {
    pub fn new(name: String) -> Self {
        Self {
            version: VERSION,
            id: Uuid::new_v4(),
            timeline: TimelineAsset::new(name.clone()),
            name,
            seed: 1,
            layers: vec![],
            extra: Extra::new(),
        }
    }
    pub fn add_layer(&mut self, name: String, content: LayerContent) -> Result<Uuid, String> {
        if self.layers.len() >= LAYER_LIMIT || self.timeline.slots.len() >= timeline::SLOT_LIMIT {
            return Err("ParticleEffect exceeds eight layers or timeline binding slots".into());
        }
        let id = Uuid::new_v4();
        let slot = Uuid::new_v4();
        self.timeline.slots.push(timeline::Slot {
            id: slot,
            name: name.clone(),
            target: crate::reflection_schema::Type::EffectLayerRef {
                class: LAYER_CLASS_ID.into(),
            },
            required: true,
            extra: Extra::new(),
        });
        self.layers.push(Layer {
            id,
            slot,
            name,
            enabled: true,
            position: [0.; 3],
            content,
            extra: Extra::new(),
        });
        Ok(id)
    }
    pub fn semantic_hash(&self) -> String {
        let mut source = self.clone();
        source.name.clear();
        source.layers.sort_by_key(|l| l.id);
        for layer in &mut source.layers {
            layer.name.clear();
        }
        // Timeline's own canonicalization excludes names/layout and orders keys.
        let timeline = source.timeline.semantic_hash();
        let mut value = serde_json::to_value(&source).expect("serializable effect");
        value["timeline"] = serde_json::json!(timeline);
        crate::assets::hash(&serde_json::to_vec(&value).expect("serializable effect"))
    }
    pub fn validate(&self, registry: &Registry) -> Vec<timeline::Diagnostic> {
        let mut errors = self.timeline.validate(registry);
        let mut report = |item, message: String| {
            errors.push(timeline::Diagnostic {
                asset: self.id,
                item,
                required: true,
                message,
            })
        };
        if self.version != VERSION || !self.extra.is_empty() {
            report(
                self.id,
                "Unsupported ParticleEffect schema; source preserved".into(),
            );
        }
        if self.layers.is_empty() || self.layers.len() > LAYER_LIMIT {
            report(
                self.id,
                "ParticleEffect requires one to eight layers".into(),
            );
        }
        let mut ids = self.timeline.identities().collect::<BTreeSet<_>>();
        if self.id.is_nil() || !ids.insert(self.id) {
            report(
                self.id,
                "Effect and embedded timeline require distinct persistent UUIDs".into(),
            );
        }
        let mut slots = BTreeSet::new();
        for layer in &self.layers {
            if layer.id.is_nil() || !ids.insert(layer.id) {
                report(layer.id, "Missing or duplicate layer UUID".into());
            }
            if !layer.extra.is_empty() {
                report(
                    layer.id,
                    "Unsupported layer fields preserved; migrate before cooking".into(),
                );
            }
            if !slots.insert(layer.slot) || !self.timeline.slots.iter().any(|s| s.id == layer.slot
                && matches!(&s.target, crate::reflection_schema::Type::EffectLayerRef {class} if class == LAYER_CLASS_ID)) {
                report(layer.id, "Layer requires a unique, typed embedded timeline slot UUID".into());
            }
            if layer
                .position
                .iter()
                .any(|v| timeline::quantize(&serde_json::json!(v)).is_err())
            {
                report(layer.id, "Layer position exceeds Q12 storage".into());
            }
            let result = match &layer.content {
                LayerContent::Emitter {emitter} => crate::particles::validate_emitter(emitter),
                LayerContent::Sprite {sprite, frames, columns, frame_ticks} => {
                    crate::sprites::validate_sprite(sprite).and_then(|_| {
                        if *frames == 0 || *frames > 256 || *columns == 0 || columns > frames || *frame_ticks <= 0 {
                            Err("Sprite layer requires 1–256 frames, valid columns and positive Q12 frame duration".into())
                        } else {Ok(())}
                    })
                }
            };
            if let Err(error) = result {
                report(layer.id, error);
            }
        }
        for slot in &self.timeline.slots {
            if matches!(
                slot.target,
                crate::reflection_schema::Type::EffectLayerRef { .. }
            ) && !slots.contains(&slot.id)
            {
                report(
                    slot.id,
                    "Internal timeline slot has no effect layer; source preserved".into(),
                );
            }
        }
        errors
    }
}
pub fn load(path: &Path) -> Result<ParticleEffect, String> {
    let bytes = crate::assets::read_bounded(path)?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("ParticleEffect source exceeds 2 MiB".into());
    }
    let mut effect: ParticleEffect = serde_json::from_slice(&bytes)
        .map_err(|e| format!("ParticleEffect source preserved: {e}"))?;
    // Existing Sprite/Emitter payloads predate flattened orphan storage. Refuse
    // to rewrite future fields that their serializers cannot preserve.
    fn check_keys(
        raw: &serde_json::Value,
        retained: &serde_json::Value,
        path: &str,
    ) -> Result<(), String> {
        match (raw, retained) {
            (serde_json::Value::Object(raw), serde_json::Value::Object(retained)) => {
                for (key, value) in raw {
                    let field = format!("{path}.{key}");
                    let Some(saved) = retained.get(key) else {
                        return Err(format!(
                            "Unsupported effect field {field}; original source preserved"
                        ));
                    };
                    check_keys(value, saved, &field)?;
                }
            }
            (serde_json::Value::Array(raw), serde_json::Value::Array(retained)) => {
                for (i, (value, saved)) in raw.iter().zip(retained).enumerate() {
                    check_keys(value, saved, &format!("{path}[{i}]"))?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    check_keys(
        &serde_json::from_slice(&bytes).map_err(|e| e.to_string())?,
        &serde_json::to_value(&effect).map_err(|e| e.to_string())?,
        "effect",
    )?;
    if effect.version != VERSION {
        return Err(format!(
            "ParticleEffect version {} is unsupported; source preserved",
            effect.version
        ));
    }
    // Apply the same TimelineAsset v1 migration as standalone timelines, in
    // memory only. There is no effect-specific timeline representation.
    effect.timeline.migrate()?;
    Ok(effect)
}
pub fn load_all(root: &Path) -> Result<Vec<(std::path::PathBuf, ParticleEffect)>, String> {
    let catalog = inspect_all(root)?;
    if !catalog.errors.is_empty() {
        return Err(catalog.errors.join("\n"));
    }
    let mut ids = timeline::load_all(root)?
        .into_iter()
        .map(|(_, asset)| asset.id)
        .collect::<BTreeSet<_>>();
    for (_, effect) in &catalog.entries {
        if !ids.insert(effect.id) || !ids.insert(effect.timeline.id) {
            return Err(format!(
                "Duplicate ParticleEffect/embedded TimelineAsset UUID {}",
                effect.id
            ));
        }
    }
    Ok(catalog.entries)
}
pub fn inspect_all(root: &Path) -> Result<timeline::Catalog<ParticleEffect>, String> {
    let (paths, errors) = crate::assets::source_paths(root, ".particle-effect.json");
    if paths.len() > timeline::ASSET_LIMIT {
        return Err("Project exceeds 32 ParticleEffects".into());
    }
    let mut result = timeline::Catalog {
        entries: Vec::new(),
        errors,
    };
    for path in paths {
        match load(&path) {
            Ok(effect) => result.entries.push((path, effect)),
            Err(error) => result.errors.push(format!("{}: {error}", path.display())),
        }
    }
    Ok(result)
}
pub fn load_referenced(
    root: &Path,
    required: &BTreeSet<Uuid>,
) -> Result<Vec<(std::path::PathBuf, ParticleEffect)>, String> {
    let timelines = timeline::inspect_all(root)?;
    let effects = inspect_all(root)?;
    let counts = timeline::catalog_id_counts(&timelines, &effects);
    let mut result = Vec::new();
    for (path, effect) in effects.entries {
        if required.contains(&effect.id) {
            if counts[&effect.id] != 1 || counts[&effect.timeline.id] != 1 {
                return Err(format!(
                    "Ambiguous ParticleEffect/TimelineAsset UUID {}/{}",
                    effect.id, effect.timeline.id
                ));
            }
            result.push((path, effect));
        }
    }
    Ok(result)
}
pub fn create(root: &Path, name: &str) -> Result<std::path::PathBuf, String> {
    create_from_preset(root, name, None)
}
pub fn create_from_preset(
    root: &Path,
    name: &str,
    preset: Option<crate::particle_effect_editor::Preset>,
) -> Result<std::path::PathBuf, String> {
    if load_all(root)?.len() >= timeline::ASSET_LIMIT {
        return Err("Project exceeds 32 ParticleEffects".into());
    }
    crate::workspace::validate_name(name)?;
    if !crate::scripts::identifier(name) {
        return Err("Use a portable identifier for the effect filename".into());
    }
    let path = crate::assets::inside(root, &format!("assets/Effects/{name}.particle-effect.json"))?;
    std::fs::create_dir_all(path.parent().ok_or("Missing effect directory")?)
        .map_err(|e| e.to_string())?;
    let mut effect = ParticleEffect::new(name.into());
    if let Some(preset) = preset {
        crate::particle_effect_editor::add_preset(&mut effect, preset)?;
    } else {
        effect.add_layer(
            "Emitter".into(),
            LayerContent::Emitter {
                emitter: Box::default(),
            },
        )?;
    }
    crate::assets::atomic_write(
        &path,
        &serde_json::to_vec_pretty(&effect).map_err(|e| e.to_string())?,
        None,
    )?;
    Ok(path)
}
/// Host source validation exposes the shared embedded timeline cook. Builds and
/// Play also validate scene bindings and resource layout against current sources.
pub fn validate_project(root: &Path) -> Result<serde_json::Value, String> {
    let registry =
        crate::scripts::catalog(root).and_then(|c| crate::blueprint::native_registry(root, &c))?;
    let mut output = vec![];
    for (path, effect) in load_all(root).inspect_err(|error| {
        let _ = crate::timeline_compile::invalidate_catalog(root, error);
    })? {
        let errors = effect.validate(&registry);
        if !errors.is_empty() {
            let error = errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n");
            crate::timeline_compile::invalidate(root, effect.timeline.id, &error)?;
            return Err(error);
        }
        let preview = crate::timeline_compile::refresh(root, &effect.timeline, &registry)?;
        if preview.stale {
            return Err(preview
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"));
        }
        let timeline = preview
            .compiled
            .ok_or("Embedded timeline has no valid compiled preview")?;
        output.push(serde_json::json!({"asset":effect.id,"path":path,"signature":effect.semantic_hash(),"layers":effect.layers.len(),"timeline":timeline}));
    }
    Ok(serde_json::json!(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_timeline_and_layers_preserve_identity_and_particle_limits() {
        let mut effect = ParticleEffect::new("Spell".into());
        for i in 0..8 {
            effect
                .add_layer(
                    format!("Layer {i}"),
                    LayerContent::Emitter {
                        emitter: Box::default(),
                    },
                )
                .unwrap();
        }
        assert!(
            effect
                .add_layer(
                    "Overflow".into(),
                    LayerContent::Emitter {
                        emitter: Box::default()
                    }
                )
                .is_err()
        );
        let signature = effect.semantic_hash();
        let mut reordered: ParticleEffect =
            serde_json::from_slice(&serde_json::to_vec(&effect).unwrap()).unwrap();
        reordered.layers.reverse();
        reordered.timeline.slots.reverse();
        reordered.name = "Renamed".into();
        for layer in &mut reordered.layers {
            layer.name = "Label".into();
        }
        assert_eq!(signature, reordered.semantic_hash());
        for layer in &reordered.layers {
            assert!(
                effect
                    .layers
                    .iter()
                    .any(|l| l.id == layer.id && l.slot == layer.slot)
            );
        }
        if let LayerContent::Emitter { emitter } = &mut reordered.layers[0].content {
            emitter.max_particles = 129;
        }
        assert!(
            reordered
                .validate(&Registry::new())
                .iter()
                .any(|d| d.item == reordered.layers[0].id)
        );
        assert_eq!(crate::particles::GLOBAL_LIMIT, 256);
        assert_eq!(crate::particles::EMITTER_LIMIT, 64);
    }
}
