//! Scrubbing samples a disposable scene. Saving, undo and builds always use authored data.
use crate::{blueprint::Registry, reflection_schema::Type, scene::Scene, timeline};
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Default)]
pub struct Preview {
    pub scene: Option<Scene>,
    pub message: String,
}
impl Preview {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

pub fn candidates(
    slot: &timeline::Slot,
    scene: &Scene,
    registry: &Registry,
) -> Vec<(Uuid, String)> {
    scene
        .actors
        .iter()
        .flat_map(|actor| {
            std::iter::once((actor.id, actor.name.clone())).chain(
                actor.components.iter().map(|component| {
                    (component.id, format!("{} / {}", actor.name, component.name))
                }),
            )
        })
        .filter(|(id, _)| {
            crate::blueprint_refs::assignment(
                "preview",
                &serde_json::json!(id),
                &slot.target,
                scene,
                registry,
            )
            .is_ok()
        })
        .collect()
}

/// Scene-owned UUID bindings take precedence. Reusable assets only auto-bind an
/// unambiguous compatible target; an explicit None or stale binding stays repairable.
pub fn resolve_bindings(
    asset: &timeline::TimelineAsset,
    scene: &Scene,
    registry: &Registry,
    bindings: &mut timeline::Bindings,
) {
    for slot in &asset.slots {
        if bindings.contains_key(&slot.id) {
            continue;
        }
        let instances = scene
            .actors
            .iter()
            .filter_map(|a| a.timeline.as_ref())
            .filter(|c| c.asset == Some(asset.id))
            .collect::<Vec<_>>();
        if !instances.is_empty() {
            let targets = instances
                .iter()
                .map(|c| c.bindings.get(&slot.id).copied().flatten())
                .collect::<BTreeSet<_>>();
            if targets.len() == 1 {
                bindings.insert(slot.id, *targets.first().unwrap());
            }
            continue;
        }
        let choices = candidates(slot, scene, registry);
        let named = choices
            .iter()
            .filter(|(id, _)| {
                scene.actors.iter().any(|a| {
                    (a.id == *id || a.components.iter().any(|c| c.id == *id)) && a.name == slot.name
                })
            })
            .collect::<Vec<_>>();
        if named.len() == 1 {
            bindings.insert(slot.id, Some(named[0].0));
        } else if choices.len() == 1 {
            bindings.insert(slot.id, Some(choices[0].0));
        }
    }
}

#[derive(Clone, Copy)]
enum SpatialProperty {
    Position,
    Rotation,
    Scale,
    FieldOfView,
}
fn spatial_property(
    track: &crate::timeline_ir::CompiledTrack,
    registry: &Registry,
) -> Option<SpatialProperty> {
    // Match the adapter's stable identity, never arbitrary user fields named position.
    let inherits = |id: &str| registry.ancestry(&track.class).iter().any(|c| c.id == id);
    match track.property.as_str() {
        "21635438-8736-5c5a-9089-2a10c37656d6"
            if inherits("313dd749-e96c-5caf-9aa2-47ee3486b814") =>
        {
            Some(SpatialProperty::Position)
        }
        "7aa29452-edc6-5f08-813f-813961e03e21"
            if inherits("313dd749-e96c-5caf-9aa2-47ee3486b814") =>
        {
            Some(SpatialProperty::Rotation)
        }
        "e6a825ce-96e4-5d74-a2d2-13b4e235cdfd"
            if inherits("313dd749-e96c-5caf-9aa2-47ee3486b814") =>
        {
            Some(SpatialProperty::Scale)
        }
        "96829145-5fad-5d63-b350-a4400d855544"
            if inherits("b24d1abd-e72f-5d3d-9f30-85bcb20c3187") =>
        {
            Some(SpatialProperty::FieldOfView)
        }
        _ => None,
    }
}

pub fn evaluate(
    asset: &timeline::TimelineAsset,
    bindings: &timeline::Bindings,
    scene: &Scene,
    registry: &Registry,
    tick: i32,
) -> Preview {
    let compiled = match crate::timeline_compile::compile(asset, registry) {
        Ok(compiled) => compiled,
        Err(errors) => {
            return Preview {
                scene: None,
                message: format!(
                    "Preview unavailable: {}",
                    errors
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
            };
        }
    };
    let errors = asset.validate_bindings(bindings, scene, registry);
    if let Some(error) = errors.iter().find(|e| e.required) {
        let slot = asset
            .slots
            .iter()
            .find(|s| s.id == error.item)
            .map_or("Target", |s| s.name.as_str());
        return Preview {
            scene: None,
            message: format!(
                "Bind {slot} to a compatible scene object: {}",
                error.message
            ),
        };
    }
    let mut result = scene.clone();
    let mut applied = 0;
    let mut unsupported = BTreeSet::new();
    for track in &compiled.tracks {
        if !track.active(tick, compiled.duration_ticks) {
            continue;
        }
        let Some(target) = bindings.get(&track.slot).copied().flatten() else {
            continue;
        };
        let Some(index) = scene
            .actors
            .iter()
            .position(|a| a.id == target || a.components.iter().any(|c| c.id == target))
        else {
            continue;
        };
        if !scene.is_active(index) || errors.iter().any(|e| e.item == track.slot) {
            continue;
        }
        let Some(property) = spatial_property(track, registry) else {
            unsupported.insert(track.field.clone());
            continue;
        };
        let actor = &mut result.actors[index];
        let values: &mut [f32] = match property {
            SpatialProperty::Position => &mut actor.position,
            SpatialProperty::Rotation => &mut actor.rotation,
            SpatialProperty::Scale => &mut actor.scale,
            SpatialProperty::FieldOfView if actor.kind == "Camera" => {
                std::slice::from_mut(&mut actor.camera_fov)
            }
            SpatialProperty::FieldOfView => {
                unsupported.insert("Field of View requires a camera".into());
                continue;
            }
        };
        for (lane, value) in values.iter_mut().enumerate() {
            let raw = crate::timeline_curve::sample_mode(
                &track.channels[lane],
                track.source_tick(tick),
                track.mode(lane) as u8,
                matches!(track.value_type, Type::UInt32),
            );
            let raw = if track.blend == timeline::Blend::Additive {
                ((*value as f64 * 4096.).round() as i32).saturating_add(raw)
            } else {
                raw
            };
            *value = raw as f32 / 4096.;
            if matches!(property, SpatialProperty::FieldOfView) {
                *value = value.clamp(25., 120.);
            }
        }
        applied += 1;
    }
    let message = if !unsupported.is_empty() {
        format!(
            "Scene preview: {applied} tracks · Game required for: {}",
            unsupported.into_iter().collect::<Vec<_>>().join(", ")
        )
    } else if applied > 0 {
        format!(
            "Scene preview: {applied} tracks{}",
            if compiled.events.is_empty() {
                ""
            } else {
                " · Events run in Game"
            }
        )
    } else {
        "No active scene tracks · Choose a binding or move the playhead".into()
    };
    Preview {
        scene: (applied > 0).then_some(result),
        message,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        actor_document::{ClassReference, ComponentInstance},
        reflection_schema as schema,
    };
    use serde_json::json;

    pub fn fixture() -> (timeline::TimelineAsset, Registry, Scene) {
        let mut registry = crate::actor_document::tests::registry();
        let mut transform = registry.named("epok::ActorComponent").unwrap().clone();
        transform.parent = Some(transform.id.clone());
        transform.id = "313dd749-e96c-5caf-9aa2-47ee3486b814".into();
        transform.cpp_name = "TimelineTransform".into();
        transform.abstract_class = false;
        transform.explicit_abstract = false;
        transform.functions.clear();
        transform.properties = [
            ("21635438-8736-5c5a-9089-2a10c37656d6", "position"),
            ("7aa29452-edc6-5f08-813f-813961e03e21", "rotation"),
        ]
        .into_iter()
        .map(|(id, name)| schema::Property {
            id: id.into(),
            name: name.into(),
            value_type: Type::Vector { length: 3 },
            default: json!([0, 0, 0]),
            editable: true,
            timeline: schema::TimelineProperty::for_type(&Type::Vector { length: 3 }),
            source: transform.source.clone(),
        })
        .collect();
        let mut camera = transform.clone();
        camera.parent = Some(transform.id.clone());
        camera.id = "b24d1abd-e72f-5d3d-9f30-85bcb20c3187".into();
        camera.cpp_name = "TimelineCamera".into();
        camera.properties = vec![schema::Property {
            id: "96829145-5fad-5d63-b350-a4400d855544".into(),
            name: "field_of_view".into(),
            value_type: Type::Fixed,
            default: json!(90),
            editable: true,
            timeline: schema::TimelineProperty::for_type(&Type::Fixed),
            source: camera.source.clone(),
        }];
        registry.classes.insert(transform.id.clone(), transform);
        registry.classes.insert(camera.id.clone(), camera.clone());
        let mut scene = Scene::default();
        scene.sync_actor_components();
        scene.actors[0].components.push(ComponentInstance::new(
            Uuid::new_v4(),
            ClassReference::new(&camera.cpp_name, &camera.id),
            "Camera animation",
        ));
        let mut asset = timeline::TimelineAsset::new("Camera preview".into());
        let slot = Uuid::new_v4();
        asset.slots.push(timeline::Slot {
            id: slot,
            name: "Main Camera".into(),
            target: Type::ComponentRef {
                class: Some(camera.id),
            },
            required: true,
            extra: Default::default(),
        });
        for (property, name, ty, start, end) in [
            (
                "21635438-8736-5c5a-9089-2a10c37656d6",
                "Position",
                Type::Vector { length: 3 },
                json!([0, 3, -6]),
                json!([4, 5, -2]),
            ),
            (
                "7aa29452-edc6-5f08-813f-813961e03e21",
                "Rotation",
                Type::Vector { length: 3 },
                json!([0, 0, 0]),
                json!([0, 90, 0]),
            ),
            (
                "96829145-5fad-5d63-b350-a4400d855544",
                "Field of View",
                Type::Fixed,
                json!(90),
                json!(60),
            ),
        ] {
            asset.tracks.push(timeline::Track {
                id: Uuid::new_v4(),
                slot,
                property: property.into(),
                name: name.into(),
                value_type: ty,
                priority: 0,
                blend: timeline::Blend::Absolute,
                restore: timeline::Restore::RestoreInitial,
                interpolation: timeline::Interpolation::Linear,
                sections: vec![],
                keys: vec![
                    timeline::Key {
                        id: Uuid::new_v4(),
                        tick: 0,
                        value: start,
                        extra: Default::default(),
                    },
                    timeline::Key {
                        id: Uuid::new_v4(),
                        tick: 4096,
                        value: end,
                        extra: Default::default(),
                    },
                ],
                extra: Default::default(),
            });
        }
        (asset, registry, scene)
    }

    #[test]
    fn camera_scrubbing_uses_component_binding_and_preserves_authored_scene() {
        let (asset, registry, scene) = fixture();
        let saved = serde_json::to_vec(&scene).unwrap();
        let mut bindings = timeline::Bindings::new();
        resolve_bindings(&asset, &scene, &registry, &mut bindings);
        assert_eq!(
            bindings[&asset.slots[0].id],
            Some(scene.actors[0].components.last().unwrap().id)
        );
        let preview = evaluate(&asset, &bindings, &scene, &registry, 2048);
        let sampled = preview
            .scene
            .unwrap_or_else(|| panic!("{}", preview.message));
        assert_eq!(sampled.actors[0].position, [2., 4., -4.]);
        assert_eq!(sampled.actors[0].rotation, [0., 45., 0.]);
        assert_eq!(sampled.actors[0].camera_fov, 75.);
        assert_eq!(serde_json::to_vec(&scene).unwrap(), saved);
        assert_eq!(
            evaluate(&asset, &bindings, &scene, &registry, 0)
                .scene
                .unwrap()
                .actors[0]
                .position,
            [0., 3., -6.]
        );
    }

    #[test]
    fn sections_honor_gaps_source_rate_and_nonaccumulating_additive_values() {
        let (mut asset, registry, scene) = fixture();
        asset.tracks.truncate(1);
        let track = &mut asset.tracks[0];
        track.blend = timeline::Blend::Additive;
        track.make_section(4096);
        let section = &mut track.sections[0];
        section.start_tick = 1024;
        section.end_tick = 3072;
        section.rate_numerator = 2;
        let mut bindings = timeline::Bindings::new();
        resolve_bindings(&asset, &scene, &registry, &mut bindings);
        assert!(
            evaluate(&asset, &bindings, &scene, &registry, 0)
                .scene
                .is_none()
        );
        let first = evaluate(&asset, &bindings, &scene, &registry, 2048)
            .scene
            .unwrap();
        assert_eq!(first.actors[0].position, [2., 7., -10.]);
        assert_eq!(
            first,
            evaluate(&asset, &bindings, &scene, &registry, 2048)
                .scene
                .unwrap()
        );
        assert!(
            evaluate(&asset, &bindings, &scene, &registry, 3072)
                .scene
                .is_none()
        );
    }

    #[test]
    fn explicit_missing_and_ambiguous_bindings_are_never_guessed() {
        let (asset, registry, mut scene) = fixture();
        let mut second = scene.actors[0].clone();
        second.id = Uuid::new_v4();
        for c in &mut second.components {
            c.id = Uuid::new_v4();
        }
        scene.actors.push(second);
        let mut bindings = timeline::Bindings::new();
        resolve_bindings(&asset, &scene, &registry, &mut bindings);
        assert!(bindings.is_empty());
        assert!(
            evaluate(&asset, &bindings, &scene, &registry, 0)
                .message
                .contains("Bind Main Camera")
        );
        bindings.insert(asset.slots[0].id, None);
        scene.actors.pop();
        resolve_bindings(&asset, &scene, &registry, &mut bindings);
        assert_eq!(bindings[&asset.slots[0].id], None);
        scene.actors[0].timeline = Some(crate::timeline_scene::Component {
            asset: Some(asset.id),
            bindings: std::collections::BTreeMap::from([(asset.slots[0].id, Some(Uuid::new_v4()))]),
            ..Default::default()
        });
        bindings.clear();
        resolve_bindings(&asset, &scene, &registry, &mut bindings);
        assert!(
            evaluate(&asset, &bindings, &scene, &registry, 0)
                .scene
                .is_none()
        );
    }
}
