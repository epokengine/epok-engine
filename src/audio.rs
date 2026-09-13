use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AudioSource {
    pub clip: Option<Uuid>,
    pub volume: f32,
    pub pitch: f32,
    pub play_on_start: bool,
    pub priority: u8,
}
impl Default for AudioSource {
    fn default() -> Self {
        Self {
            clip: None,
            volume: 1.,
            pitch: 1.,
            play_on_start: true,
            priority: 128,
        }
    }
}
pub fn validate(scene: &crate::scene::Scene) -> Result<(), String> {
    for entity in &scene.entities {
        if let Some(source) = &entity.audio
            && (!source.volume.is_finite()
                || !(0. ..=1.).contains(&source.volume)
                || !source.pitch.is_finite()
                || !(0.25..=4.).contains(&source.pitch)
                || source.clip.is_some_and(|id| id.is_nil()))
        {
            return Err(format!("{}: invalid AudioSource settings", entity.name));
        }
    }
    Ok(())
}
pub fn clip_ids(scene: &crate::scene::Scene) -> Vec<Uuid> {
    scene
        .entities
        .iter()
        .filter_map(|e| e.audio.as_ref().and_then(|a| a.clip))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
/// Authored inputs that can contribute clips to the shared bank. Spatial and
/// AudioSource playback controls do not change resource selection. Script values
/// use the same bound class and inherited properties as resource cooking.
/// Unknown classes/members remain conservative, including during source errors.
pub fn selection_signature(
    scene: &crate::scene::Scene,
    registry: Option<&crate::blueprint::Registry>,
) -> String {
    let bindings = scene
        .entities
        .iter()
        .filter(|entity| entity.script.is_some() || entity.blueprint_instance.is_some())
        .map(|entity| {
            (
                entity.id,
                (
                    entity.script.as_ref().map(|binding| {
                        let mut selected = binding.clone();
                        if let Some(registry) = registry
                            && let Some(class) = registry.bound(binding)
                        {
                            let properties = registry.properties(&class.cpp_name);
                            let keep = |name: &str| {
                                properties
                                    .iter()
                                    .find(|property| property.name == name)
                                    .is_none_or(|property| is_clip_type(&property.value_type))
                            };
                            selected.properties.retain(|name, _| keep(name));
                            selected.member_ids.retain(|name, _| keep(name));
                            selected.overrides.retain(|name| keep(name));
                        }
                        selected
                    }),
                    entity.blueprint_instance.as_ref().map(|instance| {
                        crate::blueprint_templates::instance_audio_signature(instance, registry)
                    }),
                ),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let timelines = scene
        .entities
        .iter()
        .filter_map(|entity| {
            entity
                .timeline
                .as_ref()
                .and_then(|component| component.asset)
        })
        .collect::<std::collections::BTreeSet<_>>();
    let effects = scene
        .entities
        .iter()
        .filter_map(|entity| {
            entity
                .particle_effect
                .as_ref()
                .and_then(|component| component.asset)
        })
        .collect::<std::collections::BTreeSet<_>>();
    crate::scene_dependencies::hash((clip_ids(scene), bindings, timelines, effects))
}
/// Resource selection, including an empty set, independent of event timing,
/// curves, labels and layer rendering. Preserve invalid values for observation;
/// only the existing typed cooker may validate or resolve a resource reference.
pub fn timeline_selection_signature(asset: &crate::timeline::TimelineAsset) -> String {
    reference_selection_signature(crate::timeline::resources(asset))
}
/// Graph logic does not determine which literal clips are cooked. Resolved
/// declarations identify AudioClip defaults through the existing registry.
/// Missing/invalid declarations preserve raw values for conservative observation.
pub fn blueprint_selection_signature(
    file: &crate::blueprint_asset::AssetFile,
    registry: Option<&crate::blueprint::Registry>,
) -> String {
    let asset = &file.asset;
    // Keep declaration identity even for null defaults: changing its name/type
    // changes which authored instance values the existing resolver reads.
    let mut variables = asset
        .variables
        .iter()
        .filter(|variable| is_clip_type(&variable.value_type))
        .map(|variable| {
            serde_json::to_string(&(&variable.id, &variable.name, &variable.default))
                .expect("Serializable audio declaration")
        })
        .collect::<Vec<_>>();
    variables.sort();
    let properties = registry.and_then(|registry| {
        registry
            .classes
            .get(&asset.id)
            .map(|class| registry.properties(&class.cpp_name))
    });
    let defaults = asset
        .defaults
        .iter()
        .filter(|(id, _)| {
            properties.as_ref().is_none_or(|properties| {
                properties
                    .iter()
                    .find(|property| &property.id == *id)
                    .is_none_or(|property| is_clip_type(&property.value_type))
            })
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    crate::scene_dependencies::hash((
        reference_selection_signature(crate::blueprint_refs::graph_resources(asset)),
        crate::blueprint_playback::references(std::slice::from_ref(file)),
        &asset.parent,
        defaults,
        variables,
        crate::blueprint_templates::template_audio_signature(&asset.template, registry),
    ))
}
fn is_clip_type(ty: &crate::reflection_schema::Type) -> bool {
    matches!(ty, crate::reflection_schema::Type::AssetRef { kind } if matches!(kind.as_str(), "AudioClip" | "PlayableAudio" | "MusicSequence"))
}
/// Use the same registry and inherited-property resolution as resource cooking.
/// An empty/default-null declaration still affects scene override selection.
pub fn catalog_selection_signature(
    root: &std::path::Path,
    catalog: &[crate::scripts::Script],
) -> String {
    let registry = crate::blueprint::legacy_registry(root, catalog);
    registry_selection_signature(&registry)
}
pub fn registry_selection_signature(registry: &crate::blueprint::Registry) -> String {
    let classes = registry
        .classes
        .values()
        .filter_map(|class| {
            let properties = registry
                .properties(&class.cpp_name)
                .into_iter()
                .filter(|property| is_clip_type(&property.value_type))
                .map(|property| (&property.id, &property.name, &property.default))
                .collect::<Vec<_>>();
            (!properties.is_empty()).then_some((&class.id, &class.cpp_name, properties))
        })
        .collect::<Vec<_>>();
    crate::scene_dependencies::hash(classes)
}
fn reference_selection_signature<'a>(
    values: impl Iterator<Item = (&'a crate::reflection_schema::Type, &'a serde_json::Value)>,
) -> String {
    let clips = values
        .filter(|(ty, value)| is_clip_type(ty) && !value.is_null())
        .map(|(_, value)| {
            serde_json::from_value::<Uuid>(value.clone())
                .map(|id| id.to_string())
                .unwrap_or_else(|_| value.to_string())
        })
        .collect::<std::collections::BTreeSet<_>>();
    crate::scene_dependencies::hash(clips)
}
fn validate_music_sources(
    scene: &crate::scene::Scene,
    music: &std::collections::BTreeSet<Uuid>,
) -> Result<(), String> {
    let mut autoplay = 0;
    for entity in &scene.entities {
        if let Some(source) = &entity.audio
            && source.clip.is_some_and(|id| music.contains(&id))
        {
            if source.pitch != 1. {
                return Err(format!("{}: XA music requires pitch 1.0", entity.name));
            }
            if source.play_on_start {
                autoplay += 1;
            }
        }
    }
    if autoplay > 1 {
        return Err(format!(
            "{}: only one BGM AudioSource can play on start. Start other tracks from scripts.",
            scene.name
        ));
    }
    Ok(())
}
/// Validate each authored bank before the shared resource view clears autoplay.
pub fn validate_assets(
    scene: &crate::scene::Scene,
    index: &crate::assets::Index,
) -> Result<(), String> {
    validate_assets_with_music(scene, index, true)
}
pub fn validate_assets_with_music(
    scene: &crate::scene::Scene,
    index: &crate::assets::Index,
    include_music: bool,
) -> Result<(), String> {
    validate(scene)?;
    let mut music = std::collections::BTreeSet::new();
    for id in clip_ids(scene) {
        let record = index.resolve(id)?;
        if !matches!(record.meta.kind, crate::assets::Kind::AudioClip | crate::assets::Kind::MusicSequence) {
            return Err("AudioSource must reference playable audio (AudioClip or MusicSequence)".into());
        }
        if include_music && record.meta.kind == crate::assets::Kind::AudioClip && record.meta.settings.audio()?.is_streamed() {
            music.insert(id);
        }
    }
    validate_music_sources(scene, &music)
}
pub fn stage(
    root: &std::path::Path,
    scene: &crate::scene::Scene,
    build: &std::path::Path,
    index: &crate::assets::Index,
) -> Result<Vec<crate::playback_staging::ResourceOutput>, String> {
    stage_with_music(root, scene, build, index, true)
}
pub fn stage_with_music(
    root: &std::path::Path,
    scene: &crate::scene::Scene,
    build: &std::path::Path,
    index: &crate::assets::Index,
    include_music: bool,
) -> Result<Vec<crate::playback_staging::ResourceOutput>, String> {
    validate_assets_with_music(scene, index, include_music)?;
    let sequences = crate::psx_sequence::stage(root, scene, build, index)?;
    let mut bank = String::from(
        "// Generated from UUID asset references.\n#pragma once\n#include \"audio.hpp\"\nnamespace epok {\n",
    );
    let mut descriptors = Vec::new();
    if !sequences.descriptors.is_empty() {
        bank = String::from("// Generated from UUID asset references.\n#pragma once\n#define EPOK_HAS_SEQUENCES 1\n#include \"sequence_data.hpp\"\n#include \"audio.hpp\"\nnamespace epok {\n");
        bank.push_str(&sequences.declarations);
    }
    let mut size = sequences.spu_bytes;
    let mut music_files = Vec::new();
    let mut outputs = sequences.outputs;
    let mut bank_inputs = sequences.inputs;
    for (i, id) in clip_ids(scene).iter().enumerate() {
        if let Some(descriptor) = sequences.descriptors.get(id) {
            descriptors.push(descriptor.clone());
            continue;
        }
        let record = index.resolve(*id)?;
        let package = crate::assets::Package::load(&record.path)?;
        if package.meta.id != *id
            || crate::assets::cache_key(&package.meta) != crate::assets::cache_key(&record.meta)
        {
            return Err(format!(
                "AudioClip {id} changed after resource selection; retry staging"
            ));
        }
        if package.meta.kind != crate::assets::Kind::AudioClip {
            return Err("AudioSource must reference an AudioClip".into());
        }
        bank_inputs.insert(
            format!("asset:{id}"),
            crate::assets::cache_key(&package.meta),
        );
        // Preserve clip indices used by scenes, Blueprints and timelines. A
        // disabled slot cannot start CD playback or allocate an SPU voice.
        if !include_music && package.meta.settings.audio()?.is_streamed() {
            descriptors.push("{nullptr,0,0,false}".into());
            continue;
        }
        let mut bytes = crate::assets::derived(root, &package)?;
        let cook_key = crate::assets::cook_key(root, &package.meta)?;
        bank_inputs.insert(format!("audio-cook:{id}"), cook_key.clone());
        if package.meta.settings.audio()?.is_streamed() {
            let name = format!("M{i:07}.XA");
            crate::project::write_changed(&build.join("music").join(&name), &bytes)?;
            outputs.push(payload(
                format!("music/{name}"),
                &bytes,
                &package.meta,
                cook_key,
            ));
            descriptors.push(format!(
                "{{nullptr,0,{},{},\"{};1\"}}",
                package.meta.settings.audio()?.rate(),
                package.meta.settings.audio()?.looping,
                name
            ));
            music_files.push(name);
            continue;
        }

        bytes.resize(bytes.len().div_ceil(64) * 64, 0); // DMA transfers complete 64-byte blocks.
        size += bytes.len();
        if size > crate::audio_import::SPU_BUDGET {
            return Err(format!(
                "Audio bank uses {size} bytes; PSX budget is {}. Shorten clips or reduce import sample rates.",
                crate::audio_import::SPU_BUDGET
            ));
        }
        crate::project::write_changed(&build.join("audio").join(format!("{id}.adpcm")), &bytes)?;
        outputs.push(payload(
            format!("audio/{id}.adpcm"),
            &bytes,
            &package.meta,
            cook_key,
        ));
        bank.push_str(&format!(
            "alignas(4) inline const uint8_t audio_data_{i}[] = {{\n"
        ));
        for chunk in bytes.chunks(32) {
            bank.push_str(
                &chunk
                    .iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
            bank.push_str(",\n");
        }
        bank.push_str("};\n");
        descriptors.push(format!(
            "{{audio_data_{i},{},{},{}}}",
            bytes.len(),
            package.meta.settings.audio()?.rate(),
            package.meta.settings.audio()?.looping
        ));
    }
    bank.push_str(&format!("inline const AudioClip audio_clips[] = {{{}}};\ninline constexpr size_t audio_clip_count = {};\n}}\n",if descriptors.is_empty(){"{nullptr,0,0,false}".into()}else{descriptors.join(",")},descriptors.len()));
    crate::disc::manifest(build, &music_files)?;
    crate::project::write_changed(&build.join("audio-bank.hh"), bank.as_bytes())?;
    outputs.push(crate::playback_staging::ResourceOutput {
        path: "audio-bank.hh".into(),
        signature: crate::assets::hash(bank.as_bytes()),
        inputs: bank_inputs,
    });
    Ok(outputs)
}

fn payload(
    path: String,
    bytes: &[u8],
    meta: &crate::assets::Metadata,
    cook_key: String,
) -> crate::playback_staging::ResourceOutput {
    crate::playback_staging::ResourceOutput {
        path,
        signature: crate::assets::hash(bytes),
        // Capture the loaded package used by derived(), not a subsequent scan.
        inputs: std::collections::BTreeMap::from([
            (format!("asset:{}", meta.id), crate::assets::cache_key(meta)),
            (format!("audio-cook:{}", meta.id), cook_key),
        ]),
    }
}

#[cfg(test)]
mod bank_tests {
    use super::*;
    #[test]
    fn audio_script_selection_is_typed_for_scenes_templates_and_instance_overrides() {
        use crate::{
            blueprint_templates as templates,
            scene::{Entity, Scene, ScriptBinding},
            scripts::{Property, Script},
        };
        use serde_json::json;
        let root = crate::workspace::tests::temp("script-audio-selection");
        let registry = crate::blueprint::legacy_registry(
            &root,
            &[
                Script {
                    name: "Base".into(),
                    properties: vec![
                        Property {
                            name: "sound".into(),
                            default: json!(null),
                            value_type: crate::reflection_schema::Type::AssetRef {
                                kind: "AudioClip".into(),
                            },
                            id: "sound".into(),
                        },
                        Property {
                            name: "health".into(),
                            default: json!(50),
                            value_type: crate::reflection_schema::Type::Fixed,
                            id: "health".into(),
                        },
                    ],
                    ..Default::default()
                },
                Script {
                    name: "Child".into(),
                    parent: Some("Base".into()),
                    ..Default::default()
                },
            ],
        );
        let mut binding = ScriptBinding {
            name: "Child".into(),
            class_id: Some("legacy:Child".into()),
            properties: [("health".into(), json!(50)), ("sound".into(), json!(null))].into(),
            member_ids: [
                ("health".into(), "legacy:Base:health".into()),
                ("sound".into(), "legacy:Base:sound".into()),
            ]
            .into(),
            overrides: ["health".into(), "sound".into()].into(),
            ..Default::default()
        };
        let entity_id = Uuid::new_v4();
        // Exercise all three authoring paths with the same resolved property set.
        let signature = |binding: &ScriptBinding, registry| {
            let mut entity = Entity::cube("Target".into());
            entity.id = entity_id;
            entity.script = Some(binding.clone());
            let scene = Scene {
                entities: vec![entity.clone()],
                ..Default::default()
            };
            let template = templates::Template {
                entities: vec![templates::TemplateEntity {
                    entity,
                    parent: None,
                }],
                ..Default::default()
            };
            let instance = templates::Instance {
                class: "legacy:Child".into(),
                parent: None,
                template_entity: entity_id,
                instance: entity_id,
                overrides: [("uq.component.script.v1".into(), json!(binding))].into(),
            };
            (
                selection_signature(&scene, registry),
                templates::template_audio_signature(&template, registry),
                templates::instance_audio_signature(&instance, registry),
            )
        };
        let original = signature(&binding, Some(&registry));
        binding.properties.insert("health".into(), json!(90));
        binding.overrides.remove("health");
        binding.member_ids.remove("health");
        assert_eq!(signature(&binding, Some(&registry)), original);
        binding
            .properties
            .insert("sound".into(), json!(Uuid::new_v4()));
        let changed = signature(&binding, Some(&registry));
        assert_ne!(changed.0, original.0);
        assert_ne!(changed.1, original.1);
        assert_ne!(changed.2, original.2);
        binding.properties.insert("sound".into(), json!(null));
        assert_eq!(signature(&binding, Some(&registry)), original);
        binding.properties.insert("unknown".into(), json!(1));
        assert_ne!(signature(&binding, Some(&registry)), original);
        let unresolved = signature(&binding, None);
        binding.properties.insert("health".into(), json!(100));
        assert_ne!(signature(&binding, None), unresolved);
        // Persistent identity wins over the old display name; a missing class
        // cannot borrow a similarly named class's types to discard authored data.
        binding.class_id = Some("missing".into());
        let missing = signature(&binding, Some(&registry));
        binding.properties.insert("health".into(), json!(110));
        assert_ne!(signature(&binding, Some(&registry)), missing);
    }
    #[test]
    fn catalog_audio_uses_inherited_fields_and_retains_failure_until_rebuild() {
        use crate::{
            artifact_dependencies::Graph,
            scripts::{Property, Script},
        };
        use serde_json::json;
        let root = crate::workspace::tests::temp("catalog-audio-selection");
        let clip_type = crate::reflection_schema::Type::AssetRef {
            kind: "AudioClip".into(),
        };
        let mut catalog = vec![
            Script {
                name: "Base".into(),
                properties: vec![Property {
                    name: "sound".into(),
                    default: json!(null),
                    value_type: clip_type,
                    id: "sound".into(),
                }],
                ..Default::default()
            },
            Script {
                name: "Child".into(),
                parent: Some("Base".into()),
                ..Default::default()
            },
        ];
        let signature = catalog_selection_signature(&root, &catalog);
        crate::scene_dependencies::observe_catalog(&root, Ok(&catalog)).unwrap();
        crate::artifact_dependencies::transaction(&root, |graph| {
            graph.publish(
                "test-audio-bank",
                "bank".into(),
                ["audio-catalog".into()].into(),
            );
        })
        .unwrap();
        // Inherited null fields matter: an instance may supply the actual clip.
        catalog[1].parent = None;
        assert_ne!(signature, catalog_selection_signature(&root, &catalog));
        catalog[1].parent = Some("Base".into());
        catalog[0].properties.push(Property {
            name: "health".into(),
            default: json!(50),
            value_type: crate::reflection_schema::Type::Fixed,
            id: "health".into(),
        });
        catalog.reverse();
        assert_eq!(signature, catalog_selection_signature(&root, &catalog));
        crate::scene_dependencies::observe_catalog(&root, Ok(&catalog)).unwrap();
        assert!(
            Graph::load(&root).unwrap().nodes["test-audio-bank"]
                .stale
                .is_empty()
        );
        // A failed catalog never makes retained selection trustworthy. Repair
        // republishes only the source; consumers still need successful staging.
        crate::scene_dependencies::observe_catalog(&root, Err("Invalid native catalog")).unwrap();
        let failed = Graph::load(&root).unwrap();
        assert!(
            failed.nodes["test-audio-bank"]
                .stale
                .contains_key("audio-catalog")
        );
        crate::scene_dependencies::observe_catalog(&root, Ok(&catalog)).unwrap();
        let repaired = Graph::load(&root).unwrap();
        assert!(repaired.nodes["audio-catalog"].stale.is_empty());
        assert!(!repaired.nodes["test-audio-bank"].stale.is_empty());
    }
    #[test]
    fn authored_bank_autoplay_is_checked_before_shared_resource_normalization() {
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let music = [first, second].into_iter().collect();
        let mut scene = crate::scene::Scene::default();
        scene.entities[0].audio = Some(AudioSource {
            clip: Some(first),
            ..Default::default()
        });
        validate_music_sources(&scene, &music).unwrap();
        scene.entities[1].audio = Some(AudioSource {
            clip: Some(second),
            ..Default::default()
        });
        assert!(
            validate_music_sources(&scene, &music)
                .unwrap_err()
                .contains("only one BGM")
        );
        scene.entities[1].audio.as_mut().unwrap().play_on_start = false;
        validate_music_sources(&scene, &music).unwrap();
        scene.entities[1].audio.as_mut().unwrap().pitch = 2.;
        assert!(
            validate_music_sources(&scene, &music)
                .unwrap_err()
                .contains("pitch 1.0")
        );
    }
}
