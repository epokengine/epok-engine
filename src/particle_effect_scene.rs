//! Effect components reuse scene UUID bindings and the shared timeline cooker.
use crate::{
    blueprint::Registry,
    particle_effect::{self, LayerContent, ParticleEffect},
    reflection_schema::Type,
    scene::Scene,
    timeline, timeline_compile,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Component {
    pub version: u32,
    pub asset: Option<Uuid>,
    pub bindings: timeline::Bindings,
    pub enabled: bool,
    pub play_on_start: bool,
    pub seed: u32,
    /// Layer UUID -> reflected property UUID -> existing typed literal format.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub layer_overrides: BTreeMap<Uuid, BTreeMap<String, timeline::Argument>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
impl Default for Component {
    fn default() -> Self {
        Self {
            version: 2,
            asset: None,
            bindings: Default::default(),
            enabled: true,
            play_on_start: true,
            seed: 0,
            layer_overrides: Default::default(),
            extra: Default::default(),
        }
    }
}
impl Component {
    pub fn migrate(&mut self) {
        if self.version == 1 && self.layer_overrides.is_empty() && self.extra.is_empty() {
            self.version = 2;
        }
    }
}

fn overrides(
    component: &Component,
    effect: &ParticleEffect,
    registry: &Registry,
) -> Result<String, String> {
    if component.layer_overrides.len() > 8 {
        return Err("Effect overrides exceed eight layers".into());
    }
    let mut layers = effect.layers.iter().collect::<Vec<_>>();
    layers.sort_by_key(|l| l.id);
    let class = registry
        .classes
        .get(particle_effect::LAYER_CLASS_ID)
        .ok_or("Missing reflected EffectLayer class")?;
    let properties = registry.properties(&class.cpp_name);
    let mut result = String::new();
    for (id, values) in &component.layer_overrides {
        let index = layers.iter().position(|l| l.id == *id).ok_or_else(|| {
            format!(
                "Effect {}: orphaned layer override {id}; preserved for repair",
                effect.id
            )
        })?;
        if values.len() > 8 {
            return Err(format!("Layer {id}: too many overridden properties"));
        }
        result += &format!("case {index}:{{");
        for (property, literal) in values {
            let p=properties.iter().find(|p|p.id==*property && p.timeline.is_some()).ok_or_else(||format!("Layer {id}: missing or non-animatable property {property}; override preserved"))?;
            let timeline::Argument::Literal { value_type, value } = literal else {
                return Err(format!(
                    "Layer {id}/{property}: expected a typed literal override"
                ));
            };
            if value_type != &p.value_type {
                return Err(format!(
                    "Layer {id}/{property}: override type changed; migrate the preserved value"
                ));
            }
            timeline::pack(value, value_type).map_err(|e| format!("Layer {id}/{property}: {e}"))?;
            result +=
                &crate::script_values::assignment(&format!("layer.{}", p.name), value, value_type)?;
        }
        result += "break;}";
    }
    Ok(result)
}
pub struct Prepared {
    pub source: ParticleEffect,
    pub compiled: timeline_compile::Compiled,
    /// Typed initializer bodies validated against the authoritative full manifest.
    pub initializers: BTreeMap<String, String>,
}
fn override_signature(component: &Component) -> String {
    crate::assets::hash(
        &serde_json::to_vec(&component.layer_overrides).expect("Serializable overrides"),
    )
}
fn inherited_override(
    layer: &particle_effect::Layer,
    property: &crate::reflection_schema::Property,
) -> timeline::Argument {
    let emitter = match &layer.content {
        LayerContent::Emitter { emitter } => Some(emitter.as_ref()),
        _ => None,
    };
    let value = match property.id.as_str() {
        "af33a70b-5855-4430-8cba-bc2b38119e86" => {
            serde_json::json!(layer.enabled && emitter.is_none_or(|e| e.enabled))
        }
        "37e43711-dba0-4bca-8499-952a73450d44" => {
            serde_json::json!(emitter.is_none_or(|e| e.play_on_start))
        }
        "a8ab3325-89dd-416f-a946-082292c297c6" => serde_json::json!(layer.position),
        "286d0bc0-1d5e-4b47-974d-b87119cb63df" => {
            emitter.map_or_else(|| property.default.clone(), |e| serde_json::json!(e.rate))
        }
        "41c3e62b-b9df-4df6-86b8-638e7851c4f3" => emitter.map_or_else(
            || property.default.clone(),
            |e| serde_json::json!(e.velocity),
        ),
        _ => property.default.clone(),
    };
    timeline::Argument::Literal {
        value_type: property.value_type.clone(),
        value,
    }
}
#[derive(Default)]
pub struct Inspector {
    assets: Vec<(std::path::PathBuf, ParticleEffect)>,
    checked: Option<std::time::Instant>,
    error: Option<String>,
}
pub fn inspector(
    ui: &imgui::Ui,
    editor: &mut crate::editor::Editor,
    entity: &mut crate::scene::Entity,
) {
    let Some(component) = &mut entity.particle_effect else {
        return;
    };
    if !ui.collapsing_header(
        "Particle Effect Component",
        imgui::TreeNodeFlags::DEFAULT_OPEN,
    ) {
        return;
    }
    if editor
        .effect_inspector
        .checked
        .is_none_or(|t| t.elapsed().as_secs_f32() >= 1.)
    {
        editor.effect_inspector.checked = Some(std::time::Instant::now());
        match timeline::inspect_playback(&editor.root) {
            Ok(catalog) => {
                editor.effect_inspector.assets = catalog.effects;
                editor.effect_inspector.error =
                    (!catalog.errors.is_empty()).then(|| catalog.errors.join("\n"));
            }
            Err(error) => {
                editor.effect_inspector.assets.clear();
                editor.effect_inspector.error = Some(error);
            }
        }
    }
    if let Some(error) = &editor.effect_inspector.error {
        ui.text_wrapped(format!("Effect catalog has errors: {error}"));
    }
    ui.checkbox("Enabled##effect", &mut component.enabled);
    ui.checkbox("Play on start##effect", &mut component.play_on_start);
    crate::gui::Drag::new("Seed override").build(ui, &mut component.seed);
    ui.text_disabled("Zero uses the asset seed.");
    let preview = editor
        .effect_inspector
        .assets
        .iter()
        .find(|(_, a)| Some(a.id) == component.asset)
        .map_or_else(
            || {
                component
                    .asset
                    .map_or("None".into(), |id| format!("Missing asset {id}"))
            },
            |(_, a)| a.name.clone(),
        );
    if let Some(_combo) = ui.begin_combo("Effect asset", preview) {
        if ui.selectable("None") {
            component.asset = None;
        }
        for (_, asset) in &editor.effect_inspector.assets {
            if ui.selectable(format!("{}##{}", asset.name, asset.id)) {
                component.asset = Some(asset.id);
            }
        }
    }
    let mut open = None;
    if let Some((path, asset)) = editor
        .effect_inspector
        .assets
        .iter()
        .find(|(_, a)| Some(a.id) == component.asset)
    {
        if ui.small_button("Open Particle Effect") {
            open = Some(path.clone());
        }
        if ui.collapsing_header("Layer overrides", imgui::TreeNodeFlags::empty()) {
            ui.text_disabled("Starting values for this instance. Timeline tracks take precedence.");
            if let Some(class) = editor
                .class_registry
                .classes
                .get(particle_effect::LAYER_CLASS_ID)
            {
                let properties = editor.class_registry.properties(&class.cpp_name);
                for layer in &asset.layers {
                    let _layer = ui.push_id(layer.id.to_string());
                    if !ui.collapsing_header(&layer.name, imgui::TreeNodeFlags::empty()) {
                        continue;
                    }
                    for property in properties.iter().filter(|p| p.timeline.is_some()) {
                        let _property = ui.push_id(&property.id);
                        let mut enabled = component
                            .layer_overrides
                            .get(&layer.id)
                            .is_some_and(|v| v.contains_key(&property.id));
                        if ui.checkbox(format!("Override {}", property.name), &mut enabled) {
                            component.version = 2;
                            let values = component.layer_overrides.entry(layer.id).or_default();
                            if enabled {
                                values.insert(
                                    property.id.clone(),
                                    inherited_override(layer, property),
                                );
                            } else {
                                values.remove(&property.id);
                            }
                        }
                        if let Some(literal) = component
                            .layer_overrides
                            .get_mut(&layer.id)
                            .and_then(|v| v.get_mut(&property.id))
                        {
                            if let timeline::Argument::Literal { value_type, value } = literal {
                                crate::script_values::inspector(
                                    ui,
                                    &property.name,
                                    value,
                                    value_type,
                                );
                                if value_type != &property.value_type {
                                    ui.text_colored(
                                        [1., 0.5, 0.3, 1.],
                                        "Type changed. Reset to migrate this override.",
                                    );
                                }
                            }
                            if ui.small_button("Reset override to current schema") {
                                *literal = inherited_override(layer, property);
                            }
                        }
                    }
                }
            }
            component
                .layer_overrides
                .retain(|_, values| !values.is_empty());
            if let Err(error) = overrides(component, asset, &editor.class_registry) {
                ui.text_colored([1., 0.5, 0.3, 1.], error);
                if ui.small_button("Clear preserved layer overrides") {
                    component.layer_overrides.clear();
                }
            }
        }
        for slot in asset
            .timeline
            .slots
            .iter()
            .filter(|s| matches!(s.target, Type::EntityRef { .. }))
        {
            let _id = ui.push_id(slot.id.to_string());
            let mut value = component
                .bindings
                .get(&slot.id)
                .copied()
                .flatten()
                .map_or(serde_json::Value::Null, |id| serde_json::json!(id));
            let label = format!(
                "{} ({})",
                slot.name,
                if slot.required {
                    "required"
                } else {
                    "optional"
                }
            );
            if crate::blueprint_refs::inspector(
                ui,
                &label,
                &mut value,
                &slot.target,
                &editor.scene,
                &editor.class_registry,
                &editor.assets.index,
            ) {
                component.bindings.insert(
                    slot.id,
                    value.as_str().and_then(|id| Uuid::parse_str(id).ok()),
                );
            }
        }
        let mut external = asset.timeline.clone();
        external
            .slots
            .retain(|s| matches!(s.target, Type::EntityRef { .. }));
        for diagnostic in
            external.validate_bindings(&component.bindings, &editor.scene, &editor.class_registry)
        {
            ui.text_wrapped(diagnostic.to_string());
        }
    }
    if ui.small_button("Remove Particle Effect Component") {
        entity.particle_effect = None;
    }
    if let Some(path) = open
        && let Err(error) = editor.timeline_editor.open(&path)
    {
        editor.log(error);
    }
    ui.separator();
}
pub fn remap(
    component: &mut Component,
    identities: &BTreeMap<Uuid, Uuid>,
    strict: bool,
) -> Result<(), String> {
    for target in component.bindings.values_mut() {
        let Some(id) = *target else { continue };
        if let Some(replacement) = identities.get(&id) {
            *target = Some(*replacement);
        } else if strict {
            return Err(format!(
                "Effect template binding {id} points outside its subtree; clear or replace it first"
            ));
        }
    }
    Ok(())
}
pub fn prepare(
    root: &Path,
    scenes: &[Scene],
    registry: &Registry,
    direct: &BTreeSet<Uuid>,
) -> Result<Vec<Prepared>, String> {
    let mut required = direct.clone();
    for scene in scenes {
        let mut count = 0;
        for entity in &scene.entities {
            let Some(component) = &entity.particle_effect else {
                continue;
            };
            count += 1;
            if ![1, 2].contains(&component.version)
                || (component.version == 1 && !component.layer_overrides.is_empty())
                || !component.extra.is_empty()
            {
                return Err(format!(
                    "Scene {} / {}: unsupported ParticleEffectComponent schema; source preserved",
                    scene.name, entity.id
                ));
            }
            required.insert(component.asset.ok_or_else(|| {
                format!(
                    "Scene {} / {}: ParticleEffectComponent has no asset UUID",
                    scene.name, entity.id
                )
            })?);
        }
        if count > 8 {
            return Err(format!(
                "Scene {} exceeds eight ParticleEffectComponents",
                scene.name
            ));
        }
    }
    if required.is_empty() {
        return Ok(vec![]);
    }
    let mut result = vec![];
    for (_, source) in particle_effect::load_referenced(root, &required).inspect_err(|error| {
        let _ = timeline_compile::invalidate_catalog(root, error);
    })? {
        if !required.remove(&source.id) {
            continue;
        }
        let errors = source.validate(registry);
        if !errors.is_empty() {
            let error = errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n");
            timeline_compile::invalidate(root, source.timeline.id, &error)?;
            return Err(error);
        }
        // Internal slots are filled by the effect pool. Only explicitly typed
        // external EntityRefs go through scene authoring binding resolution.
        let mut external = source.timeline.clone();
        let mut initializers = BTreeMap::new();
        external
            .slots
            .retain(|s| matches!(s.target, Type::EntityRef { .. }));
        for scene in scenes {
            for entity in &scene.entities {
                let Some(component) = &entity.particle_effect else {
                    continue;
                };
                if component.asset != Some(source.id) {
                    continue;
                }
                let initializer = overrides(component, &source, registry)
                    .map_err(|e| format!("Scene {} / {}: {e}", scene.name, entity.id))?;
                initializers.insert(override_signature(component), initializer);
                if component.bindings.keys().any(|id| {
                    source
                        .timeline
                        .slots
                        .iter()
                        .any(|s| s.id == *id && matches!(s.target, Type::EffectLayerRef { .. }))
                }) {
                    return Err(format!(
                        "Scene {} / {}: internal effect layers cannot be overridden by scene entity bindings",
                        scene.name, entity.id
                    ));
                }
                let errors = external
                    .validate_bindings(&component.bindings, scene, registry)
                    .into_iter()
                    .filter(|d| d.required)
                    .map(|d| format!("Scene {} / {}: {d}", scene.name, entity.id))
                    .collect::<Vec<_>>();
                if !errors.is_empty() {
                    return Err(errors.join("\n"));
                }
            }
        }
        let preview = timeline_compile::refresh(root, &source.timeline, registry)?;
        if preview.stale {
            return Err(preview
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"));
        }
        result.push(Prepared {
            source,
            initializers,
            compiled: preview
                .compiled
                .ok_or("Effect cook has no valid embedded timeline")?,
        });
    }
    if !required.is_empty() {
        timeline_compile::invalidate_catalog(root, "Required ParticleEffect source is missing")?;
        return Err(format!(
            "Missing ParticleEffect UUIDs: {}",
            required
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    result.sort_by_key(|p| p.source.id);
    crate::timeline_runtime::validate_ids(result.iter().map(|p| &p.compiled), registry)?;
    Ok(result)
}
pub fn resources(effects: &[Prepared]) -> Vec<(Type, serde_json::Value)> {
    effects
        .iter()
        .flat_map(|effect| effect.source.layers.iter())
        .filter_map(|layer| {
            let sprite = match &layer.content {
                LayerContent::Sprite { sprite, .. } => sprite,
                LayerContent::Emitter { emitter } => &emitter.sprite,
            };
            sprite.texture.map(|id| {
                (
                    Type::AssetRef {
                        kind: "Texture".into(),
                    },
                    serde_json::json!(id),
                )
            })
        })
        .collect()
}
pub fn header(effect: &Prepared, registry: &Registry, resources: &Scene) -> Result<String, String> {
    let ids = crate::texture::ids(resources);
    let mut out = crate::timeline_runtime::header(&effect.compiled, registry)?;
    out += "#include \"particle_effect_runtime.hpp\"\n";
    out += &format!(
        "namespace epok::effects::cooked::asset_{} {{\ninline constexpr LayerDefinition layers[]={{\n",
        effect.source.id.simple()
    );
    let mut layers = effect.source.layers.iter().collect::<Vec<_>>();
    layers.sort_by_key(|layer| layer.id);
    for layer in layers {
        let slot = effect
            .compiled
            .slots
            .iter()
            .position(|s| s.id == layer.slot)
            .ok_or("Missing compiled effect layer slot")?;
        let (sprite, frames, columns) = match &layer.content {
            LayerContent::Sprite {
                sprite,
                frames,
                columns,
                ..
            } => (sprite, *frames, *columns),
            LayerContent::Emitter { emitter } => (
                &emitter.sprite,
                emitter.frames,
                emitter.frame_columns.min(emitter.frames),
            ),
        };
        crate::texture::validate_region(sprite.texture, sprite.region, resources)?;
        if frames > 1 {
            let [x, y, width, height] = sprite.region;
            let end_x = u32::from(x) + u32::from(columns - 1) * u32::from(width);
            let end_y = u32::from(y) + u32::from((frames - 1) / columns) * u32::from(height);
            if end_x + u32::from(width) > 256
                || end_y + u32::from(height) > 256
                || width == 0
                || height == 0
            {
                return Err(format!(
                    "Effect {} / layer {}: flipbook exceeds a 256x256 atlas",
                    effect.source.id, layer.id
                ));
            }
            crate::texture::validate_region(
                sprite.texture,
                [end_x as u16, end_y as u16, width, height],
                resources,
            )?;
        }
        let texture = match sprite.texture {
            Some(id) => ids.iter().position(|v| *v == id).ok_or_else(|| {
                format!(
                    "Effect {} / layer {}: unresolved texture {id}",
                    effect.source.id, layer.id
                )
            })? as i32,
            None => -1,
        };
        out += &format!(
            "[]() constexpr {{LayerDefinition value;value.slot={slot};value.initial.enabled={};",
            layer.enabled
        );
        for (axis, position) in layer.position.iter().enumerate() {
            out += &format!(
                "value.initial.position[{axis}]=Fixed({},Fixed::RAW);",
                timeline::quantize(&serde_json::json!(position))?
            );
        }
        match &layer.content {
            LayerContent::Sprite {
                sprite,
                frames,
                columns,
                frame_ticks,
            } => {
                out += &format!(
                    "value.initial.sprite={};value.frames={frames};value.columns={columns};value.frame_ticks={frame_ticks};",
                    crate::sprites::cpp_sprite(sprite, texture)
                );
            }
            LayerContent::Emitter { emitter } => {
                out += &crate::particles::cpp_emitter(emitter, "value.initial.emitter", texture);
                out += &format!(
                    "value.emitter=true;value.initial.enabled={};value.initial.playing={};value.initial.rate=value.initial.emitter.rate;for(unsigned i=0;i<3;++i)value.initial.velocity[i]=value.initial.emitter.velocity[i];",
                    layer.enabled && emitter.enabled,
                    emitter.play_on_start
                );
            }
        }
        out += "return value;}(),\n";
    }
    out += &format!(
        "}};\ninline constexpr Asset asset{{UINT64_C({}),&epok::timeline::cooked::asset_{}::asset,layers,{},{}u}};\n}}\n",
        crate::blueprint_refs::compact_id(&effect.source.id.to_string()),
        effect.compiled.asset.simple(),
        effect.source.layers.len(),
        effect.source.seed
    );
    Ok(out)
}
pub fn setup(
    scene: &Scene,
    effects: &[Prepared],
    registry: &Registry,
    template: bool,
) -> Result<String, String> {
    let mut out = String::new();
    if !template {
        out += "epok::effects::component_count=0;\n";
    }
    for (owner, entity) in scene.entities.iter().enumerate() {
        let Some(component) = &entity.particle_effect else {
            continue;
        };
        let effect = effects
            .iter()
            .find(|p| Some(p.source.id) == component.asset)
            .ok_or("Missing validated component effect")?;
        let owner = if template {
            format!("handles[{owner}]")
        } else {
            format!("epok::handle(&objects[{owner}])")
        };
        out += &format!(
            "{{auto* component=epok::effects::configure({owner},epok::effects::cooked::asset_{}::asset,{},{},{}u);",
            effect.source.id.simple(),
            component.enabled,
            component.play_on_start,
            component.seed
        );
        if template {
            out += "if(!component)return false;";
        } else {
            out += "if(component){";
        }
        if !component.layer_overrides.is_empty() {
            out += "component->initialize=[](epok::EffectLayer& layer,uint16_t index){using epok::Fixed;switch(index){";
            out += effect
                .initializers
                .get(&override_signature(component))
                .ok_or(
                    "Effect component overrides were not validated; rebuild from fresh reflection",
                )?;
            out += "default:break;}};";
        }
        for (i, slot) in effect.compiled.slots.iter().enumerate() {
            if !matches!(slot.target, Type::EntityRef { .. }) {
                continue;
            }
            let value = component
                .bindings
                .get(&slot.id)
                .copied()
                .flatten()
                .map_or(serde_json::Value::Null, |id| serde_json::json!(id));
            let mut assignment = match crate::blueprint_refs::assignment(
                &format!("component->bindings[{i}]"),
                &value,
                &slot.target,
                scene,
                registry,
            ) {
                Ok(code) => code,
                Err(error) if slot.required => return Err(error),
                Err(_) => format!("component->bindings[{i}]={{}};\n"),
            };
            if template {
                for index in 0..scene.entities.len() {
                    assignment = assignment.replace(
                        &format!("epok::handle(&objects[{index}])"),
                        &format!("handles[{index}]"),
                    );
                }
            }
            out += &assignment;
        }
        if !template {
            out += "}";
        }
        out += "}\n";
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn component_migration_preserves_explicit_typed_layer_identity() {
        let mut old: Component =
            serde_json::from_value(serde_json::json!({"version":1,"seed":91})).unwrap();
        old.migrate();
        assert_eq!(old.version, 2);
        assert_eq!(old.seed, 91);
        let layer = Uuid::new_v4();
        let property = Uuid::new_v4().to_string();
        old.layer_overrides.insert(
            layer,
            BTreeMap::from([(
                property.clone(),
                timeline::Argument::Literal {
                    value_type: Type::Fixed,
                    value: serde_json::json!(0.75),
                },
            )]),
        );
        let bytes = serde_json::to_vec(&old).unwrap();
        let mut loaded: Component = serde_json::from_slice(&bytes).unwrap();
        remap(
            &mut loaded,
            &BTreeMap::from([(layer, Uuid::new_v4())]),
            true,
        )
        .unwrap();
        assert_eq!(
            loaded, old,
            "Layer/property IDs belong to the asset, not the scene UUID remapper"
        );
        loaded.version = 1;
        loaded.migrate();
        assert_eq!(
            loaded.version, 1,
            "An invalid v1 override is not silently migrated"
        );
    }
    #[test]
    fn component_roundtrip_and_template_remapping_keep_authoring_identity() {
        let mut scene = Scene::default();
        let slot = Uuid::new_v4();
        let target = scene.entities[1].id;
        let asset = Uuid::new_v4();
        scene.entities[0].particle_effect = Some(Component {
            asset: Some(asset),
            bindings: BTreeMap::from([(slot, Some(target))]),
            seed: 73,
            ..Default::default()
        });
        scene.upgrade_entity_ids();
        assert_eq!(scene.version, 4);
        let mut loaded: Scene =
            serde_json::from_slice(&serde_json::to_vec(&scene).unwrap()).unwrap();
        loaded.entities.swap(1, 2);
        let component = loaded.entities[0].particle_effect.as_mut().unwrap();
        assert_eq!(component.bindings[&slot], Some(target));
        let replacement = Uuid::new_v4();
        remap(component, &BTreeMap::from([(target, replacement)]), true).unwrap();
        assert_eq!(component.bindings[&slot], Some(replacement));
        assert_eq!(component.asset, Some(asset));
        assert_eq!(component.seed, 73);
        assert!(remap(component, &BTreeMap::new(), true).is_err());
        let bytes = serde_json::to_string(component).unwrap();
        assert!(!bytes.contains("generation") && !bytes.contains("index"));
    }
}
