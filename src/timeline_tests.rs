use super::*;
use crate::{
    reflection_schema as schema, timeline_compile as cook, timeline_editor::TimelineEditor,
};

fn fixture() -> (TimelineAsset, Registry, crate::scene::Scene) {
    let class = Uuid::new_v4().to_string();
    let property = Uuid::new_v4().to_string();
    let source = schema::Location {
        file: "assets/scripts/Spell.hpp".into(),
        line: 4,
        column: 1,
    };
    let mut registry = Registry::new();
    registry.classes.insert(
        class.clone(),
        schema::Class {
            family: None,
            domain: None,
            placement: Default::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            id: class.clone(),
            provider: schema::native_provider(),
            backend: schema::native_backend(),
            cpp_name: "Spell".into(),
            parent: None,
            abstract_class: false,
            final_class: false,
            timeline_component: None,
            blueprintable: true,
            properties: vec![schema::Property {
                id: property.clone(),
                name: "progress".into(),
                value_type: Type::Fixed,
                default: serde_json::json!(0),
                editable: true,
                timeline: Some(schema::TimelineProperty::FixedLinearAbsolute),
                source: source.clone(),
            }],
            functions: vec![],
            source,
        },
    );
    let mut a = TimelineAsset::new("Cast".into());
    let slot = Uuid::new_v4();
    a.slots.push(Slot {
        id: slot,
        name: "Caster".into(),
        target: Type::ObjectRef {
            class: Some(class.clone()),
        },
        required: true,
        extra: Default::default(),
    });
    a.tracks.push(Track {
        sections: vec![],
        id: Uuid::new_v4(),
        name: "Charge".into(),
        slot,
        property,
        value_type: Type::Fixed,
        priority: 0,
        blend: Blend::Absolute,
        restore: Restore::LeaveFinal,
        interpolation: Interpolation::Linear,
        keys: vec![
            Key {
                id: Uuid::new_v4(),
                tick: 0,
                value: serde_json::json!(-10),
                extra: Default::default(),
            },
            Key {
                id: Uuid::new_v4(),
                tick: 4096,
                value: serde_json::json!(10),
                extra: Default::default(),
            },
        ],
        extra: Default::default(),
    });
    a.markers.push(Marker {
        id: Uuid::new_v4(),
        name: "Impact".into(),
        tick: 2048,
        extra: Default::default(),
    });
    let mut scene = crate::scene::Scene::default();
    scene.actors[0].set_class_defaults(
        &(crate::scene::ClassDefaults {
            name: "Spell".into(),
            class_id: Some(class),
            ..Default::default()
        }),
    );
    (a, registry, scene)
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("epok-timeline-test-{}", Uuid::new_v4()));
        fs::create_dir_all(p.join("assets/Timelines")).unwrap();
        Self(p)
    }
}

#[test]
fn typed_tracks_cook_all_lanes_and_reject_discrete_interpolation() {
    let (mut a, mut r, _) = fixture();
    for (ty, first, last, expected) in [
        (
            Type::Vector { length: 3 },
            serde_json::json!([-1, 2, 4]),
            serde_json::json!([1, 4, 0]),
            [0, 12288, 8192, 0],
        ),
        (
            Type::Int32,
            serde_json::json!(-200),
            serde_json::json!(100),
            [-50, 0, 0, 0],
        ),
        (
            Type::UInt32,
            serde_json::json!(0),
            serde_json::json!(u32::MAX),
            [i32::MAX, 0, 0, 0],
        ),
    ] {
        let p = &mut r.classes.values_mut().next().unwrap().properties[0];
        p.value_type = ty.clone();
        p.timeline = schema::TimelineProperty::for_type(&ty);
        let t = &mut a.tracks[0];
        t.value_type = ty;
        t.keys[0].value = first;
        t.keys[1].value = last;
        let c = cook::compile(&a, &r).unwrap();
        assert_eq!(c.sample_values(2048)[0].1, expected);
        assert!(c.tables().contains("inline constexpr Curve"));
    }
    let ty = Type::Bool;
    let p = &mut r.classes.values_mut().next().unwrap().properties[0];
    p.value_type = ty.clone();
    p.timeline = schema::TimelineProperty::for_type(&ty);
    let t = &mut a.tracks[0];
    t.value_type = ty;
    t.keys[0].value = serde_json::json!(false);
    t.keys[1].value = serde_json::json!(true);
    assert!(cook::compile(&a, &r).is_err());
    a.tracks[0].interpolation = Interpolation::Step;
    let c = cook::compile(&a, &r).unwrap();
    assert_eq!(c.sample(4095)[0].1, 0);
    assert_eq!(c.sample(4096)[0].1, 1);
    a.tracks[0].blend = Blend::Additive;
    assert!(cook::compile(&a, &r).is_err());
}

#[test]
fn v3_sections_map_independent_source_time_and_are_silent_outside_their_range() {
    let (mut asset, registry, _) = fixture();
    let track = &mut asset.tracks[0];
    track.make_section(asset.duration_ticks);
    let section = &mut track.sections[0];
    section.start_tick = 1024;
    section.end_tick = 3072;
    section.source_offset_tick = 0;
    section.rate_numerator = 1;
    section.rate_denominator = 1;
    assert!(asset.validate(&registry).is_empty());
    let compiled = cook::compile(&asset, &registry).unwrap();
    assert!(compiled.sample_values(0).is_empty());
    assert_eq!(compiled.sample_values(1024)[0].1[0], -40960);
    assert_eq!(compiled.sample_values(2048)[0].1[0], -20480);
    assert!(compiled.sample_values(3072).is_empty());
    assert!(
        crate::timeline_runtime::header(&compiled, &registry)
            .unwrap()
            .contains("1024,3072,0,1,1")
    );
}

#[test]
fn authoring_and_cooked_curves_accept_the_full_256_key_profile() {
    let (mut asset, registry, _) = fixture();
    asset.tracks[0].keys = (0..256)
        .map(|i| Key {
            id: Uuid::new_v4(),
            tick: i * 16,
            value: serde_json::json!(i),
            extra: Default::default(),
        })
        .collect();
    let compiled = cook::compile(&asset, &registry).unwrap();
    assert_eq!(compiled.tracks[0].channels[0].len(), 256);
    assert_eq!(compiled.sample_values(2040)[0].1[0], 127 * 4096 + 2048);
    asset.tracks[0].keys.push(Key {
        id: Uuid::new_v4(),
        tick: 4095,
        value: serde_json::json!(256),
        extra: Default::default(),
    });
    assert!(
        asset
            .validate(&registry)
            .iter()
            .any(|d| d.message.contains("2–256"))
    );
}

fn add_event(a: &mut TimelineAsset, r: &mut Registry) {
    let c = r.classes.values_mut().next().unwrap();
    let id = Uuid::new_v4().to_string();
    c.functions.push(schema::Function {
        id: id.clone(),
        name: "Impact".into(),
        parameters: vec![schema::Parameter {
            name: "power".into(),
            value_type: Type::Fixed,
            direction: schema::Direction::Value,
        }],
        returns: Type::Void,
        callable: true,
        timeline: Some(schema::TimelineCall::CrossingEvent),
        event: false,
        pure: false,
        abstract_method: false,
        final_method: false,
        access: "public".into(),
        overrides: vec![],
        source: c.source.clone(),
    });
    a.events.push(EventTrack {
        id: Uuid::new_v4(),
        name: "Impact".into(),
        slot: a.slots[0].id,
        function: id,
        keys: vec![EventKey {
            id: Uuid::new_v4(),
            tick: 2048,
            arguments: BTreeMap::from([(
                "power".into(),
                Argument::Literal {
                    value_type: Type::Fixed,
                    value: serde_json::json!(1.5),
                },
            )]),
            extra: Default::default(),
        }],
        extra: Default::default(),
    });
}

#[test]
fn event_exposure_ids_signatures_arguments_and_merged_crossings_are_cooked() {
    let (mut a, mut r, _) = fixture();
    add_event(&mut a, &mut r);
    let c = cook::compile(&a, &r).unwrap();
    assert_eq!(c.events[0].arguments[0].lanes, [6144, 0, 0, 0]);
    assert!(c.tables().contains("inline constexpr Argument"));
    assert!(c.tables().contains("inline constexpr Signal"));
    let original = c.signature;
    a.events[0].name = "Renamed".into();
    assert_eq!(cook::compile(&a, &r).unwrap().signature, original);
    r.classes.values_mut().next().unwrap().functions[0].name = "ApplyImpact".into();
    let renamed = cook::compile(&a, &r).unwrap();
    assert_ne!(renamed.signature, original);
    assert_eq!(renamed.events[0].method, "ApplyImpact");
    r.classes.values_mut().next().unwrap().functions[0].timeline = None;
    assert!(cook::compile(&a, &r).is_err()); // Blueprint Callable alone is insufficient.
    r.classes.values_mut().next().unwrap().functions[0].timeline =
        Some(schema::TimelineCall::CrossingEvent);
    let key = &mut a.events[0].keys[0];
    key.arguments.insert(
        "power".into(),
        Argument::Literal {
            value_type: Type::Int32,
            value: serde_json::json!(1),
        },
    );
    assert!(cook::compile(&a, &r).is_err());
    a.events[0].function = "ApplyImpact".into();
    assert!(
        a.validate(&r)
            .iter()
            .any(|d| d.message.contains("no name fallback"))
    );
}

#[test]
fn event_entity_arguments_share_blueprint_assignability_and_slot_reorder_is_stable() {
    let (mut a, mut r, _) = fixture();
    add_event(&mut a, &mut r);
    let mut slot = a.slots[0].clone();
    slot.id = Uuid::new_v4();
    slot.name = "Target".into();
    a.slots.push(slot);
    r.classes.values_mut().next().unwrap().functions[0].parameters[0].value_type =
        Type::ObjectRef { class: None };
    a.events[0].keys[0].arguments.insert(
        "power".into(),
        Argument::Slot {
            slot: a.slots[1].id,
        },
    );
    let c = cook::compile(&a, &r).unwrap();
    a.slots.reverse();
    let reordered = cook::compile(&a, &r).unwrap();
    assert_eq!(c.signature, reordered.signature);
    assert_eq!(c.tables(), reordered.tables());
    assert_eq!(c.events[0].arguments[0].slot, Some(a.slots[0].id));
    r.classes.values_mut().next().unwrap().functions[0].parameters[0].value_type = Type::Record {
        cpp_name: "epok::Transform".into(),
        fields: vec![],
    };
    assert!(cook::compile(&a, &r).is_err());
}

#[test]
fn event_resource_dependencies_use_the_existing_import_index_and_fail_closed() {
    let (mut a, mut r, _) = fixture();
    add_event(&mut a, &mut r);
    let id = Uuid::new_v4();
    let ty = Type::AssetRef {
        kind: "Texture".into(),
    };
    r.classes.values_mut().next().unwrap().functions[0].parameters[0].value_type = ty.clone();
    a.events[0].keys[0].arguments.insert(
        "power".into(),
        Argument::Literal {
            value_type: ty,
            value: serde_json::json!(id),
        },
    );
    let mut index = crate::assets::Index::default();
    assert!(cook::compile_index(&a, &r, &index).is_err());
    let record = crate::assets::Record {
        path: "assets/Texture.epokasset".into(),
        revision: "test".into(),
        meta: crate::assets::Metadata {
            version: 1,
            id,
            kind: crate::assets::Kind::Texture,
            importer_version: 1,
            source: "texture.png".into(),
            source_hash: "a".into(),
            settings: Default::default(),
            extra: Default::default(),
        },
    };
    index.assets.insert(id, vec![record]);
    let original = cook::compile_index(&a, &r, &index).unwrap();
    assert_eq!(original.events[0].arguments[0].resource, Some(id));
    index.assets.get_mut(&id).unwrap()[0].meta.source_hash = "b".into();
    assert_ne!(
        original.signature,
        cook::compile_index(&a, &r, &index).unwrap().signature
    );
    index.assets.get_mut(&id).unwrap()[0].meta.kind = crate::assets::Kind::AudioClip;
    assert!(cook::compile_index(&a, &r, &index).is_err());
}

#[test]
fn v1_migration_is_in_memory_only_and_does_not_grant_blueprint_permissions() {
    let (a, _, _) = fixture();
    let temp = Temp::new();
    let mut json = serde_json::to_value(&a).unwrap();
    json["version"] = serde_json::json!(1);
    json.as_object_mut().unwrap().remove("events");
    let path = temp.0.join("assets/Timelines/Legacy.timeline.json");
    let raw = serde_json::to_vec(&json).unwrap();
    fs::write(&path, &raw).unwrap();
    let migrated = load(&path).unwrap();
    assert_eq!(migrated.version, VERSION);
    assert_eq!(migrated.id, a.id);
    assert!(migrated.events.is_empty());
    assert_eq!(fs::read(&path).unwrap(), raw);
    let mut b = crate::blueprint_asset::BlueprintAsset::new("Legacy".into(), "base".into());
    b.version = 1;
    let path = temp.0.join("Legacy.blueprint.json");
    let raw = serde_json::to_vec(&b).unwrap();
    fs::write(&path, &raw).unwrap();
    assert!(
        crate::blueprint_asset::load(&path)
            .unwrap_err()
            .contains("unsupported Blueprint version")
    );
    assert_eq!(fs::read(&path).unwrap(), raw);
    let variable:crate::blueprint_asset::Variable=serde_json::from_value(serde_json::json!({"id":"old-variable","name":"Power","value_type":{"kind":"fixed"},"default":0})).unwrap();
    assert!(!variable.timeline_animatable);
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn roundtrip_reorder_rename_preserves_id_and_semantics() {
    let (mut a, registry, _) = fixture();
    let semantic = a.semantic_hash();
    let original = a.clone();
    a.name = "Renamed".into();
    a.slots[0].name = "Hero".into();
    a.tracks[0].name = "Glow".into();
    a.markers[0].name = "Hit".into();
    a.tracks[0].keys.reverse();
    a.layout.insert(a.tracks[0].id, [90., 20.]);
    let roundtrip: TimelineAsset =
        serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
    assert_eq!(roundtrip.id, original.id);
    assert_eq!(roundtrip.tracks[0].id, original.tracks[0].id);
    assert_eq!(
        roundtrip.tracks[0].keys[0].id,
        original.tracks[0].keys[1].id
    );
    assert_eq!(roundtrip.semantic_hash(), semantic);
    assert!(roundtrip.validate(&registry).is_empty());
    assert_eq!(cook::compile(&a, &registry).unwrap().sample(2048)[0].1, 0);
}
#[test]
fn required_optional_bindings_use_existing_typed_entity_identity() {
    let (mut a, registry, mut scene) = fixture();
    let id = scene.actors[0].id;
    let bindings = Bindings::from([(a.slots[0].id, Some(id))]);
    assert!(a.validate_bindings(&bindings, &scene, &registry).is_empty());
    scene.actors.swap(0, 1);
    assert!(a.validate_bindings(&bindings, &scene, &registry).is_empty());
    scene.actors[1].active = false;
    assert!(!a.validate_bindings(&bindings, &scene, &registry)[0].required);
    scene.actors[1].class = crate::actor_document::ClassReference::new(
        "epok::Actor3D",
        crate::object_model::ACTOR3D_ID,
    );
    scene.actors[1].properties.clear();
    scene.actors[1].overrides.clear();
    assert!(a.validate_bindings(&bindings, &scene, &registry)[0].required);
    a.slots[0].required = false;
    scene.actors.remove(1);
    let errors = a.validate_bindings(&bindings, &scene, &registry);
    assert!(!errors[0].required);
    assert!(errors[0].message.contains("missing"));
    assert_eq!(bindings.values().next(), Some(&Some(id)));
    let serialized = serde_json::to_string(&bindings).unwrap();
    assert!(serialized.contains(&id.to_string()));
    assert!(!serialized.contains("generation"));
}
#[test]
fn component_requirements_inherit_validate_and_change_cooked_dependencies() {
    use schema::TimelineComponentRequirement as Component;
    let (mut asset, mut registry, mut scene) = fixture();
    let child_id = registry.classes.keys().next().unwrap().clone();
    let mut parent = registry.classes[&child_id].clone();
    parent.id = Uuid::new_v4().to_string();
    parent.cpp_name = "ComponentAdapter".into();
    parent.properties.clear();
    parent.timeline_component = Some(Component::Camera);
    let parent_id = parent.id.clone();
    registry.classes.get_mut(&child_id).unwrap().parent = Some(parent_id.clone());
    registry.classes.insert(parent_id.clone(), parent);
    scene.actors[0].kind = "Empty".into();
    let bindings = Bindings::from([(asset.slots[0].id, Some(scene.actors[0].id))]);
    let missing = asset.validate_bindings(&bindings, &scene, &registry);
    assert!(
        missing
            .iter()
            .any(|d| d.required && d.message.contains("Camera"))
    );
    asset.slots[0].required = false;
    assert!(
        asset
            .validate_bindings(&bindings, &scene, &registry)
            .iter()
            .all(|d| !d.required)
    );
    asset.slots[0].required = true;
    scene.actors[0].kind = "Camera".into();
    assert!(
        asset
            .validate_bindings(&bindings, &scene, &registry)
            .is_empty()
    );
    let camera = cook::compile(&asset, &registry).unwrap();
    let header = crate::timeline_runtime::header(&camera, &registry).unwrap();
    assert!(header.contains("&&target.data()->camera"));
    assert!(header.contains("timeline_sync("));
    registry
        .classes
        .get_mut(&parent_id)
        .unwrap()
        .timeline_component = Some(Component::Light);
    let light = cook::compile(&asset, &registry).unwrap();
    assert_ne!(camera.signature, light.signature);
    assert!(
        crate::timeline_runtime::header(&light, &registry)
            .unwrap()
            .contains("&&target.data()->light.enabled")
    );
    scene.actors[0].light = Some(Default::default());
    assert!(
        asset
            .validate_bindings(&bindings, &scene, &registry)
            .is_empty()
    );
    scene.actors[0].light.as_mut().unwrap().enabled = false;
    assert!(
        asset
            .validate_bindings(&bindings, &scene, &registry)
            .iter()
            .any(|d| d.required)
    );

    // Two independent requirements survive native/Blueprint inheritance.
    registry
        .classes
        .get_mut(&parent_id)
        .unwrap()
        .timeline_component = Some(Component::RectTransform);
    registry
        .classes
        .get_mut(&child_id)
        .unwrap()
        .timeline_component = Some(Component::Text);
    scene.actors[0].text = Some(Default::default());
    assert!(
        asset
            .validate_bindings(&bindings, &scene, &registry)
            .iter()
            .any(|d| d.message.contains("RectTransform"))
    );
    scene.actors[0].rect = Some(Default::default());
    assert!(
        asset
            .validate_bindings(&bindings, &scene, &registry)
            .is_empty()
    );
    let header =
        crate::timeline_runtime::header(&cook::compile(&asset, &registry).unwrap(), &registry)
            .unwrap();
    assert!(
        header
            .contains("&&target.data()->rect.enabled&&target.data()&&target.data()->text.enabled")
    );

    // Older manifests retain authoring identity and grant no new capability.
    let original = &registry.classes[&child_id];
    let mut legacy = serde_json::to_value(original).unwrap();
    legacy.as_object_mut().unwrap().remove("timeline_component");
    let migrated: schema::Class = serde_json::from_value(legacy.clone()).unwrap();
    assert_eq!(migrated.id, original.id);
    assert_eq!(migrated.properties[0].id, original.properties[0].id);
    assert_eq!(migrated.timeline_component, None);
    legacy["timeline_component"] = serde_json::json!("UnknownComponent");
    assert!(serde_json::from_value::<schema::Class>(legacy).is_err());
}
#[test]
fn installing_component_adapters_preserves_project_owned_source() {
    let temp = Temp::new();
    let path = crate::timeline_adapters::install(&temp.0).unwrap();
    let original = fs::read(&path).unwrap();
    assert_eq!(crate::timeline_adapters::install(&temp.0).unwrap(), path);
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::write(&path, b"// Project-owned changes\n").unwrap();
    assert!(
        crate::timeline_adapters::install(&temp.0)
            .unwrap_err()
            .contains("preserved")
    );
    assert_eq!(fs::read(&path).unwrap(), b"// Project-owned changes\n");
}
#[test]
fn permission_types_conflicts_and_orphans_are_not_rebound() {
    let (mut a, mut registry, scene) = fixture();
    let class = registry.classes.values_mut().next().unwrap();
    class.properties[0].timeline = None;
    assert!(!a.validate(&registry).is_empty());
    registry.classes.values_mut().next().unwrap().properties[0].timeline =
        Some(schema::TimelineProperty::FixedLinearAbsolute);
    a.tracks[0].value_type = Type::Int32;
    assert!(!a.validate(&registry).is_empty());
    a.tracks[0].value_type = Type::Fixed;
    let mut copy = a.tracks[0].clone();
    copy.id = Uuid::new_v4();
    for k in &mut copy.keys {
        k.id = Uuid::new_v4();
    }
    a.tracks.push(copy);
    assert!(
        a.validate(&registry)
            .iter()
            .any(|d| d.message.contains("conflict"))
    );
    let mut slot = a.slots[0].clone();
    slot.id = Uuid::new_v4();
    a.tracks[1].slot = slot.id;
    a.slots.push(slot);
    assert!(a.validate(&registry).is_empty());
    let b = a
        .slots
        .iter()
        .map(|s| (s.id, Some(scene.actors[0].id)))
        .collect();
    assert!(
        a.validate_bindings(&b, &scene, &registry)
            .iter()
            .any(|d| d.message.contains("alias"))
    );
    a.tracks[0].property = "progress".into();
    assert!(
        a.validate(&registry)
            .iter()
            .any(|d| d.message.contains("no name fallback"))
    );
}
#[test]
fn limits_times_quantization_and_unknown_versions_are_explicit() {
    let (mut a, r, _) = fixture();
    a.tracks[0].keys[1].tick = 0;
    assert!(!a.validate(&r).is_empty());
    a.tracks[0].keys[1].tick = 4096;
    a.tracks[0].interpolation = Interpolation::Smoothstep;
    assert!(a.validate(&r).iter().any(|d| d.message.contains("profile")));
    assert_eq!(quantize(&serde_json::json!(-0.5 / 4096.)).unwrap(), -1);
    assert_eq!(
        quantize(&serde_json::json!(f64::from(i32::MIN) / 4096.)).unwrap(),
        i32::MIN
    );
    assert!(quantize(&serde_json::json!(524288)).is_err());
    assert!(quantize(&serde_json::json!(true)).is_err());
    a.version = VERSION + 1;
    let temp = Temp::new();
    let path = temp.0.join("assets/Timelines/Future.timeline.json");
    let data = serde_json::to_vec(&a).unwrap();
    fs::write(&path, &data).unwrap();
    assert!(load(&path).unwrap_err().contains("preserved"));
    assert_eq!(fs::read(path).unwrap(), data);
}
#[test]
fn dependency_signatures_follow_ancestry_but_ignore_unrelated_classes() {
    let (a, mut r, _) = fixture();
    let original = cook::compile(&a, &r).unwrap().signature;
    let mut unrelated = r.classes.values().next().unwrap().clone();
    unrelated.id = Uuid::new_v4().to_string();
    unrelated.cpp_name = "Other".into();
    r.classes.insert(unrelated.id.clone(), unrelated);
    assert_eq!(cook::compile(&a, &r).unwrap().signature, original);
    let class_id = match &a.slots[0].target {
        Type::ObjectRef { class: Some(id) } => id.clone(),
        _ => unreachable!(),
    };
    r.classes.get_mut(&class_id).unwrap().properties[0].name = "renamed_progress".into();
    let changed = cook::compile(&a, &r).unwrap();
    assert_ne!(changed.signature, original);
    assert_eq!(changed.tracks[0].field, "renamed_progress");
    assert_eq!(changed.tracks[0].property, a.tracks[0].property);
    let mut derived = r.classes[&class_id].clone();
    derived.id = Uuid::new_v4().to_string();
    derived.cpp_name = "Child".into();
    derived.parent = Some(class_id.clone());
    derived.properties.clear();
    let mut child_asset = a.clone();
    child_asset.slots[0].target = Type::ObjectRef {
        class: Some(derived.id.clone()),
    };
    r.classes.insert(derived.id.clone(), derived);
    let old = cook::compile(&child_asset, &r).unwrap().signature;
    r.classes.get_mut(&class_id).unwrap().properties[0].default = serde_json::json!(3);
    assert_eq!(cook::compile(&child_asset, &r).unwrap().signature, old);
    r.classes.get_mut(&class_id).unwrap().properties[0].name = "inherited_charge".into();
    assert_ne!(cook::compile(&child_asset, &r).unwrap().signature, old);
}
#[test]
fn invalid_refresh_retains_only_explicitly_stale_preview_and_recovers() {
    let (mut a, r, _) = fixture();
    let temp = Temp::new();
    let first = cook::refresh(&temp.0, &a, &r).unwrap();
    assert!(!first.stale);
    let property = a.tracks[0].property.clone();
    a.tracks[0].property = Uuid::new_v4().to_string();
    let stale = cook::refresh(&temp.0, &a, &r).unwrap();
    assert!(stale.stale);
    assert!(!stale.diagnostics.is_empty());
    assert_eq!(
        first.compiled.unwrap().signature,
        stale.compiled.unwrap().signature
    );
    assert!(cook::compile(&a, &r).is_err());
    a.tracks[0].property = property;
    assert!(!cook::refresh(&temp.0, &a, &r).unwrap().stale);
}
#[test]
fn reflected_changes_invalidate_only_consumed_cached_timelines_and_survive_relocation() {
    let (a, mut registry, _) = fixture();
    let (b, mut other, _) = fixture();
    other.classes.values_mut().next().unwrap().cpp_name = "OtherSpell".into();
    registry.classes.extend(other.classes);
    let temp = Temp::new();
    assert!(!cook::refresh(&temp.0, &a, &registry).unwrap().stale);
    assert!(!cook::refresh(&temp.0, &b, &registry).unwrap().stale);
    let path_a = temp.0.join(format!(".epok/timelines/{}.json", a.id));
    let path_b = temp.0.join(format!(".epok/timelines/{}.json", b.id));
    let unrelated = fs::read(&path_b).unwrap();
    let graph_before = fs::read(temp.0.join(".epok/ArtifactDependencies.json")).unwrap();
    cook::observe_reflection(&temp.0, &registry).unwrap();
    assert_eq!(
        fs::read(temp.0.join(".epok/ArtifactDependencies.json")).unwrap(),
        graph_before
    );
    let class = a.slots[0].class_id().unwrap().to_string();
    registry.classes.get_mut(&class).unwrap().properties[0].name = "charge".into();
    cook::observe_reflection(&temp.0, &registry).unwrap();
    let cache: cook::PreviewCache = serde_json::from_slice(&fs::read(&path_a).unwrap()).unwrap();
    assert!(cache.stale && cache.compiled.is_some());
    assert!(cache.diagnostics[0].message.contains(&a.tracks[0].property));
    assert_eq!(fs::read(&path_b).unwrap(), unrelated);
    assert!(!cook::refresh(&temp.0, &a, &registry).unwrap().stale);

    // Provenance contains stable IDs, no absolute project paths. Move the actual
    // cache directory before removing the property and observe with the new root.
    let moved = Temp::new();
    fs::rename(temp.0.join(".epok"), moved.0.join(".epok")).unwrap();
    // Existing projects only have PreviewCache footprints. Regenerate graph
    // provenance from those without treating the old code as freshly cooked.
    fs::remove_file(moved.0.join(".epok/ArtifactDependencies.json")).unwrap();
    registry.classes.get_mut(&class).unwrap().properties.clear();
    cook::observe_reflection(&moved.0, &registry).unwrap();
    let stale = cook::refresh(&moved.0, &a, &registry).unwrap();
    assert!(stale.stale && stale.compiled.is_some());
    assert_eq!(
        fs::read(moved.0.join(format!(".epok/timelines/{}.json", b.id))).unwrap(),
        unrelated
    );
}
#[test]
fn source_changes_and_deletions_mark_only_their_cached_output_stale() {
    let temp = Temp::new();
    let registry = Registry::new();
    let mut a = TimelineAsset::new("Cast".into());
    let b = TimelineAsset::new("Door".into());
    let source = temp.0.join("assets/Timelines/Cast.timeline.json");
    fs::write(&source, serde_json::to_vec(&a).unwrap()).unwrap();
    fs::write(
        temp.0.join("assets/Timelines/Door.timeline.json"),
        serde_json::to_vec(&b).unwrap(),
    )
    .unwrap();
    cook::compile_project(&temp.0, &registry).unwrap();
    let output_a = temp.0.join(format!(".epok/timelines/{}.json", a.id));
    let output_b = temp.0.join(format!(".epok/timelines/{}.json", b.id));
    let unchanged = fs::read(&output_b).unwrap();
    a.duration_ticks += 68;
    fs::write(&source, serde_json::to_vec(&a).unwrap()).unwrap();
    cook::observe_sources(&temp.0).unwrap();
    let cache: cook::PreviewCache = serde_json::from_slice(&fs::read(&output_a).unwrap()).unwrap();
    assert!(cache.stale && cache.compiled.is_some());
    assert_eq!(fs::read(&output_b).unwrap(), unchanged);
    assert!(!cook::refresh(&temp.0, &a, &registry).unwrap().stale);
    fs::remove_file(&source).unwrap();
    cook::observe_sources(&temp.0).unwrap();
    let cache: cook::PreviewCache = serde_json::from_slice(&fs::read(&output_a).unwrap()).unwrap();
    assert!(cache.stale && cache.diagnostics[0].message.contains("removed"));
    fs::write(&source, serde_json::to_vec(&a).unwrap()).unwrap();
    cook::observe_sources(&temp.0).unwrap();
    let cache: cook::PreviewCache = serde_json::from_slice(&fs::read(&output_a).unwrap()).unwrap();
    assert!(
        cache.stale,
        "Restoring a source cannot certify a former artifact"
    );
    assert_eq!(fs::read(&output_b).unwrap(), unchanged);
    assert!(!cook::refresh(&temp.0, &a, &registry).unwrap().stale);
    fs::write(&source, b"{ broken timeline").unwrap();
    assert!(cook::compile_project(&temp.0, &registry).is_err());
    let cache: cook::PreviewCache = serde_json::from_slice(&fs::read(&output_a).unwrap()).unwrap();
    assert!(
        cache.stale && cache.compiled.is_some(),
        "CLI parse failures must also invalidate the previous preview"
    );
    assert_eq!(fs::read(&output_b).unwrap(), unchanged);
}
#[test]
fn malformed_effects_and_uuid_collisions_do_not_hide_independent_timeline_sources() {
    let temp = Temp::new();
    fs::create_dir_all(temp.0.join("assets/Effects")).unwrap();
    let registry = Registry::new();
    let door = TimelineAsset::new("Door".into());
    let independent = TimelineAsset::new("Independent".into());
    let mut effect = crate::particle_effect::ParticleEffect::new("Fire".into());
    effect.timeline.markers.push(Marker {
        id: Uuid::new_v4(),
        name: "Impact".into(),
        tick: 1,
        extra: Default::default(),
    });
    for asset in [&door, &independent] {
        fs::write(
            temp.0
                .join(format!("assets/Timelines/{}.timeline.json", asset.name)),
            serde_json::to_vec(asset).unwrap(),
        )
        .unwrap();
    }
    let path = temp.0.join("assets/Effects/Fire.particle-effect.json");
    let valid = serde_json::to_vec(&effect).unwrap();
    fs::write(&path, &valid).unwrap();
    for asset in [&door, &independent, &effect.timeline] {
        assert!(!cook::refresh(&temp.0, asset, &registry).unwrap().stale);
    }
    cook::observe_sources(&temp.0).unwrap();
    let cache_path = |id| temp.0.join(format!(".epok/timelines/{id}.json"));
    let door_bytes = fs::read(cache_path(door.id)).unwrap();
    let independent_bytes = fs::read(cache_path(independent.id)).unwrap();
    fs::write(&path, b"{ broken effect").unwrap();
    let observed = cook::observe_sources(&temp.0).unwrap();
    assert!(observed.contains(&door.id) && !observed.contains(&effect.timeline.id));
    let cache: cook::PreviewCache =
        serde_json::from_slice(&fs::read(cache_path(effect.timeline.id)).unwrap()).unwrap();
    assert!(cache.stale && cache.compiled.is_some());
    assert!(crate::particle_effect::load_all(&temp.0).is_err());
    assert_eq!(fs::read(cache_path(door.id)).unwrap(), door_bytes);
    fs::write(&path, &valid).unwrap();
    cook::observe_sources(&temp.0).unwrap();
    let cache: cook::PreviewCache =
        serde_json::from_slice(&fs::read(cache_path(effect.timeline.id)).unwrap()).unwrap();
    assert!(cache.stale, "Restored sources need fresh validation");
    cook::refresh(&temp.0, &effect.timeline, &registry).unwrap();
    let duplicate = temp.0.join("assets/Effects/Duplicate.particle-effect.json");
    fs::write(&duplicate, &valid).unwrap();
    let observed = cook::observe_sources(&temp.0).unwrap();
    assert!(observed.contains(&door.id) && !observed.contains(&effect.timeline.id));
    assert!(crate::particle_effect::load_all(&temp.0).is_err());
    fs::remove_file(duplicate).unwrap();
    effect.timeline.id = door.id;
    fs::write(&path, serde_json::to_vec(&effect).unwrap()).unwrap();
    let observed = cook::observe_sources(&temp.0).unwrap();
    assert!(!observed.contains(&door.id) && observed.contains(&independent.id));
    let cache: cook::PreviewCache =
        serde_json::from_slice(&fs::read(cache_path(door.id)).unwrap()).unwrap();
    assert!(cache.stale);
    assert_eq!(
        fs::read(cache_path(independent.id)).unwrap(),
        independent_bytes
    );
}
#[test]
fn playback_watch_uses_selected_stage_edges_and_survives_preview_publication() {
    let temp = Temp::new();
    let mut used = TimelineAsset::new("Used".into());
    let mut other = TimelineAsset::new("Other".into());
    let path = temp.0.join("assets/Timelines/Used.timeline.json");
    let other_path = temp.0.join("assets/Timelines/Other.timeline.json");
    fs::write(&path, serde_json::to_vec(&used).unwrap()).unwrap();
    fs::write(&other_path, serde_json::to_vec(&other).unwrap()).unwrap();
    let registry = Registry::new();
    cook::refresh(&temp.0, &used, &registry).unwrap();
    cook::refresh(&temp.0, &other, &registry).unwrap();
    crate::artifact_dependencies::transaction(&temp.0, |graph| {
        graph.publish(
            "stage:.epok/build",
            "release".into(),
            [format!("cooked-timeline:{}", used.id)]
                .into_iter()
                .collect(),
        );
        graph.publish(
            "stage:.epok/build-blueprint-debug",
            "debug".into(),
            [format!("cooked-timeline:{}", other.id)]
                .into_iter()
                .collect(),
        );
    })
    .unwrap();
    let mut watch = cook::SourceWatch::default();
    assert!(!watch.poll(&temp.0, ".epok/build").unwrap().1);
    other.duration_ticks += 68;
    fs::write(&other_path, serde_json::to_vec(&other).unwrap()).unwrap();
    assert!(!watch.poll(&temp.0, ".epok/build").unwrap().1);
    fs::write(&other_path, b"{").unwrap();
    assert!(!watch.poll(&temp.0, ".epok/build").unwrap().1);
    used.name = "Renamed".into();
    fs::write(&path, serde_json::to_vec(&used).unwrap()).unwrap();
    assert!(!watch.poll(&temp.0, ".epok/build").unwrap().1);
    used.duration_ticks += 68;
    fs::write(&path, serde_json::to_vec(&used).unwrap()).unwrap();
    // A preview can publish the source first. Watch history still detects it.
    cook::refresh(&temp.0, &used, &registry).unwrap();
    assert!(watch.poll(&temp.0, ".epok/build").unwrap().1);
    assert!(!watch.poll(&temp.0, ".epok/build").unwrap().1);
    fs::write(&path, b"{").unwrap();
    assert!(watch.poll(&temp.0, ".epok/build").unwrap().1);
    assert!(!watch.poll(&temp.0, ".epok/build").unwrap().1);
    fs::write(&path, serde_json::to_vec(&used).unwrap()).unwrap();
    assert!(watch.poll(&temp.0, ".epok/build").unwrap().1);
    fs::write(&other_path, serde_json::to_vec(&other).unwrap()).unwrap();
    assert!(
        watch
            .poll(&temp.0, ".epok/build-blueprint-debug")
            .unwrap()
            .1
    );
}
#[test]
fn required_scene_assets_ignore_unrelated_errors_and_reject_missing_or_ambiguous_sources() {
    let temp = Temp::new();
    let (asset, mut registry, mut scene) = fixture();
    let mut layer_class = registry.classes.values().next().unwrap().clone();
    layer_class.id = crate::particle_effect::LAYER_CLASS_ID.into();
    layer_class.cpp_name = "epok::EffectLayer".into();
    layer_class.properties.clear();
    registry.classes.insert(layer_class.id.clone(), layer_class);
    scene.actors[0].timeline = Some(crate::timeline_scene::Component {
        asset: Some(asset.id),
        bindings: BTreeMap::from([(asset.slots[0].id, Some(scene.actors[0].id))]),
        ..Default::default()
    });
    let path = temp.0.join("assets/Timelines/Cast.timeline.json");
    let bytes = serde_json::to_vec(&asset).unwrap();
    fs::write(&path, &bytes).unwrap();
    fs::write(temp.0.join("assets/Timelines/Broken.timeline.json"), b"{").unwrap();
    fs::create_dir_all(temp.0.join("assets/Effects")).unwrap();
    fs::write(
        temp.0.join("assets/Effects/Broken.particle-effect.json"),
        b"{",
    )
    .unwrap();
    let mut effect = crate::particle_effect::ParticleEffect::new("Selected".into());
    effect
        .add_layer(
            "Glow".into(),
            crate::particle_effect::LayerContent::Sprite {
                sprite: Default::default(),
                frames: 1,
                columns: 1,
                frame_ticks: 68,
            },
        )
        .unwrap();
    let effect_path = temp.0.join("assets/Effects/Selected.particle-effect.json");
    let effect_bytes = serde_json::to_vec(&effect).unwrap();
    fs::write(&effect_path, &effect_bytes).unwrap();
    let prepare = || {
        crate::timeline_scene::prepare(
            &temp.0,
            std::slice::from_ref(&scene),
            &registry,
            &BTreeSet::new(),
        )
    };
    assert_eq!(prepare().unwrap()[0].source.id, asset.id);
    let prepare_effect = || {
        crate::particle_effect_scene::prepare(&temp.0, &[], &registry, &BTreeSet::from([effect.id]))
    };
    assert_eq!(prepare_effect().unwrap()[0].source.id, effect.id);
    assert!(load_all(&temp.0).is_err());
    assert!(crate::particle_effect::load_all(&temp.0).is_err());
    let independent_cache = temp
        .0
        .join(format!(".epok/timelines/{}.json", effect.timeline.id));
    let independent_bytes = fs::read(&independent_cache).unwrap();
    fs::write(&path, b"{").unwrap();
    assert!(prepare().err().unwrap().contains(&asset.id.to_string()));
    assert_eq!(fs::read(&independent_cache).unwrap(), independent_bytes);
    fs::write(&path, &bytes).unwrap();
    fs::write(&effect_path, b"{").unwrap();
    assert!(
        prepare_effect()
            .err()
            .unwrap()
            .contains(&effect.id.to_string())
    );
    assert_eq!(prepare().unwrap()[0].source.id, asset.id);
    fs::write(&effect_path, &effect_bytes).unwrap();
    let duplicate = temp.0.join("assets/Effects/Duplicate.particle-effect.json");
    fs::write(&duplicate, &effect_bytes).unwrap();
    assert!(
        prepare_effect()
            .err()
            .unwrap()
            .contains(&effect.id.to_string())
    );
    assert_eq!(prepare().unwrap()[0].source.id, asset.id);
    assert!(load_referenced(&temp.0, &BTreeSet::from([effect.timeline.id]), true).is_err());
    fs::remove_file(&duplicate).unwrap();
    let mut collision = effect.clone();
    collision.timeline.id = asset.id;
    fs::write(&effect_path, serde_json::to_vec(&collision).unwrap()).unwrap();
    assert!(prepare().err().unwrap().contains(&asset.id.to_string()));
    assert!(prepare_effect().is_err());
}
#[test]
fn staged_headers_follow_reflected_inputs_and_rebuild_each_destination_independently() {
    use crate::{artifact_dependencies::Graph, playback_staging::Batch};
    let temp = Temp::new();
    let (a, mut registry, _) = fixture();
    let b = TimelineAsset::new("Independent".into());
    let build = temp.0.join(".epok/build");
    let export = temp.0.join("exports/retained");
    let output = |target: &str, id| format!("generated-playback:{target}/timelines/{id}.hh");
    let stage = |destination: &Path, registry: &Registry, assets: &[&TimelineAsset]| {
        let mut batch = Batch::new(&temp.0, destination).unwrap();
        for source in assets {
            let compiled = cook::compile(source, registry).unwrap();
            let header = crate::timeline_runtime::header(&compiled, registry).unwrap();
            batch.timeline(&compiled, header.as_bytes()).unwrap();
        }
        batch.publish(&temp.0).unwrap();
    };
    stage(&build, &registry, &[&a, &b]);
    stage(&export, &registry, &[&a, &b]);
    let before = Graph::load(&temp.0).unwrap();
    let independent = before.nodes[&output(".epok/build", b.id)].clone();
    let class = registry.classes.values_mut().next().unwrap();
    class.properties[0].name = "renamed_progress".into();
    cook::observe_reflection(&temp.0, &registry).unwrap();
    let changed = Graph::load(&temp.0).unwrap();
    for target in [".epok/build", "exports/retained"] {
        let id = output(target, a.id);
        assert!(!changed.nodes[&id].stale.is_empty());
        assert_eq!(changed.nodes[&id].signature, before.nodes[&id].signature);
        assert!(
            !changed.nodes[&format!("stage-playback:{target}")]
                .stale
                .is_empty()
        );
    }
    assert_eq!(changed.nodes[&output(".epok/build", b.id)], independent);
    stage(&build, &registry, &[&a, &b]);
    let rebuilt = Graph::load(&temp.0).unwrap();
    assert!(rebuilt.nodes[&output(".epok/build", a.id)].stale.is_empty());
    assert!(
        !rebuilt.nodes[&output("exports/retained", a.id)]
            .stale
            .is_empty()
    );
    stage(&export, &registry, &[&a, &b]);
    crate::playback_staging::invalidate(&temp.0, &build, "Required binding is broken").unwrap();
    let failed = Graph::load(&temp.0).unwrap();
    assert!(!failed.nodes[&output(".epok/build", a.id)].stale.is_empty());
    assert!(
        failed.nodes[&output("exports/retained", a.id)]
            .stale
            .is_empty()
    );
    // Successful removal retains navigation provenance, but excludes the old
    // header from the fresh target. No generated file becomes a source of truth.
    stage(&build, &registry, &[&b]);
    let removed = Graph::load(&temp.0).unwrap();
    assert!(!removed.nodes[&output(".epok/build", a.id)].stale.is_empty());
    assert_eq!(
        removed.nodes["stage-playback:.epok/build"].dependencies,
        BTreeSet::from([output(".epok/build", b.id)])
    );
    assert!(removed.nodes["stage-playback:.epok/build"].stale.is_empty());
    assert!(
        !serde_json::to_string(&removed)
            .unwrap()
            .contains(&temp.0.to_string_lossy().replace('\\', "\\\\"))
    );
}

#[test]
fn audio_bank_tracks_timeline_clip_selection_without_visual_or_timing_dependencies() {
    use crate::{artifact_dependencies::Graph, playback_staging::Batch};
    let temp = Temp::new();
    fs::write(
        temp.0.join("assets/tone.wav"),
        crate::audio_import::test_wav(),
    )
    .unwrap();
    let clip = crate::assets::commit(
        crate::assets::prepare(
            &temp.0,
            "assets/tone.wav",
            "assets/tone.epokasset",
            Default::default(),
            None,
            false,
        )
        .unwrap(),
    )
    .unwrap();
    let index = crate::assets::scan(&temp.0, &mut Default::default());
    let (mut asset, mut registry, _) = fixture();
    add_event(&mut asset, &mut registry);
    let audio_type = Type::AssetRef {
        kind: "AudioClip".into(),
    };
    registry.classes.values_mut().next().unwrap().functions[0].parameters[0].value_type =
        audio_type.clone();
    let mut event = asset.events.pop().unwrap();
    event.keys[0].arguments.insert(
        "power".into(),
        Argument::Literal {
            value_type: audio_type,
            value: serde_json::json!(clip),
        },
    );
    let path = temp.0.join("assets/Timelines/Sound.timeline.json");
    let save =
        |asset: &TimelineAsset| fs::write(&path, serde_json::to_vec(asset).unwrap()).unwrap();
    let bank = |target: &str| format!("generated-resource:{target}/audio-bank.hh");
    let projection = format!("timeline-audio:{}", asset.id);
    let stage = |target: &str, asset: &TimelineAsset| {
        let destination = temp.0.join(target);
        let mut batch = Batch::new(&temp.0, &destination).unwrap();
        let compiled = cook::compile_index(asset, &registry, &index).unwrap();
        let header = crate::timeline_runtime::header(&compiled, &registry).unwrap();
        batch.timeline(&compiled, header.as_bytes()).unwrap();
        batch.timeline_audio_source(asset).unwrap();
        let extra = resources(asset)
            .map(|(ty, value)| (ty.clone(), value.clone()))
            .collect::<Vec<_>>();
        let selected =
            crate::blueprint_refs::resources(&[], &[], &registry, &index, &extra).unwrap();
        batch
            .resources(crate::audio::stage(&temp.0, &selected, &destination, &index).unwrap())
            .unwrap();
        batch.audio_bank_inputs();
        batch.publish(&temp.0).unwrap();
        let graph = Graph::load(&temp.0).unwrap();
        let output = &graph.nodes[&bank(target)];
        assert!(output.stale.is_empty());
        assert_eq!(
            output.signature,
            Some(crate::assets::hash(
                &fs::read(destination.join("audio-bank.hh")).unwrap()
            ))
        );
        assert!(output.dependencies.contains(&projection));
        assert!(
            !output
                .dependencies
                .iter()
                .any(|key| key.starts_with("generated-playback:"))
        );
        graph
    };
    save(&asset);
    let empty = stage(".epok/build", &asset);
    assert!(
        !empty.nodes[&bank(".epok/build")]
            .dependencies
            .contains(&format!("asset:{clip}"))
    );
    // An empty projection must detect adding the first clip, not just edits to
    // previously selected resources. Both targets own independent certificates.
    stage("exports/audio", &asset);
    asset.events.push(event);
    save(&asset);
    cook::observe_sources(&temp.0).unwrap();
    let changed = Graph::load(&temp.0).unwrap();
    for target in [".epok/build", "exports/audio"] {
        assert!(changed.nodes[&bank(target)].stale.contains_key(&projection));
    }
    let selected = stage(".epok/build", &asset);
    assert!(
        selected.nodes[&bank(".epok/build")]
            .dependencies
            .contains(&format!("asset:{clip}"))
    );
    assert!(!selected.nodes[&bank("exports/audio")].stale.is_empty());
    stage("exports/audio", &asset);
    // Curves, marker positions and event times alter playback, not the bank.
    asset.tracks[0].keys[0].value = serde_json::json!(-5);
    asset.markers[0].tick += 100;
    asset.events[0].keys[0].tick += 200;
    save(&asset);
    cook::observe_sources(&temp.0).unwrap();
    let visual = Graph::load(&temp.0).unwrap();
    assert_eq!(
        visual.nodes[&bank(".epok/build")],
        selected.nodes[&bank(".epok/build")]
    );
    assert!(
        !visual.nodes[&format!("generated-playback:.epok/build/timelines/{}.hh", asset.id)]
            .stale
            .is_empty()
    );
    let rebuilt = stage(".epok/build", &asset);
    assert_eq!(
        rebuilt.nodes[&bank(".epok/build")],
        selected.nodes[&bank(".epok/build")]
    );
    let mut duplicate = asset.events[0].keys[0].clone();
    duplicate.id = Uuid::new_v4();
    duplicate.tick += 100;
    duplicate.arguments.insert(
        "power".into(),
        Argument::Literal {
            value_type: Type::AssetRef {
                kind: "AudioClip".into(),
            },
            value: serde_json::json!(clip.to_string().to_uppercase()),
        },
    );
    asset.events[0].keys.push(duplicate);
    save(&asset);
    cook::observe_sources(&temp.0).unwrap();
    assert_eq!(
        Graph::load(&temp.0).unwrap().nodes[&bank(".epok/build")],
        selected.nodes[&bank(".epok/build")]
    );
    asset.events.clear();
    save(&asset);
    cook::observe_sources(&temp.0).unwrap();
    assert!(
        Graph::load(&temp.0).unwrap().nodes[&bank(".epok/build")]
            .stale
            .contains_key(&projection)
    );
    let removed = stage(".epok/build", &asset);
    assert!(
        !removed.nodes[&bank(".epok/build")]
            .dependencies
            .contains(&format!("asset:{clip}"))
    );
    assert_eq!(
        removed.nodes[&bank(".epok/build")].signature,
        empty.nodes[&bank(".epok/build")].signature
    );
    for ambiguous in [false, true] {
        let duplicate_path = temp.0.join("assets/Timelines/Duplicate.timeline.json");
        if ambiguous {
            fs::copy(&path, &duplicate_path).unwrap();
        } else {
            fs::write(&path, b"{").unwrap();
        }
        cook::observe_sources(&temp.0).unwrap();
        assert!(
            Graph::load(&temp.0).unwrap().nodes[&bank(".epok/build")]
                .stale
                .contains_key(&projection)
        );
        if ambiguous {
            fs::remove_file(&duplicate_path).unwrap();
        } else {
            save(&asset);
        }
        cook::observe_sources(&temp.0).unwrap();
        assert!(
            !Graph::load(&temp.0).unwrap().nodes[&bank(".epok/build")]
                .stale
                .is_empty()
        );
        stage(".epok/build", &asset);
    }
}

#[test]
fn staging_rejects_mixed_playback_snapshots_before_publishing() {
    let temp = Temp::new();
    let mut a = TimelineAsset::new("Cast".into());
    let registry = Registry::new();
    let old = cook::compile(&a, &registry).unwrap();
    a.duration_ticks += 68;
    let changed = cook::compile(&a, &registry).unwrap();
    let mut batch =
        crate::playback_staging::Batch::new(&temp.0, &temp.0.join(".epok/build")).unwrap();
    batch.timeline(&old, old.tables().as_bytes()).unwrap();
    assert!(
        batch
            .timeline(&changed, changed.tables().as_bytes())
            .unwrap_err()
            .contains("changed during staging")
    );
    assert!(!temp.0.join(".epok/ArtifactDependencies.json").exists());
    // A watcher observation after capture must survive an older worker's
    // publication attempt. The failed batch must not publish any output.
    let mut older =
        crate::playback_staging::Batch::new(&temp.0, &temp.0.join(".epok/build")).unwrap();
    older.timeline(&old, old.tables().as_bytes()).unwrap();
    crate::artifact_dependencies::transaction(&temp.0, |graph| {
        graph.publish(
            &format!("timeline:{}", a.id),
            a.semantic_hash(),
            BTreeSet::new(),
        );
    })
    .unwrap();
    assert!(
        older
            .publish(&temp.0)
            .unwrap_err()
            .contains("changed during staging")
    );
    let graph = crate::artifact_dependencies::Graph::load(&temp.0).unwrap();
    assert_eq!(
        graph.nodes[&format!("timeline:{}", a.id)].signature,
        Some(a.semantic_hash())
    );
    assert!(
        !graph
            .nodes
            .keys()
            .any(|id| id.starts_with("generated-playback:"))
    );
}

#[test]
fn editing_undo_redo_save_preserves_unknown_data_and_external_writes() {
    let (mut a, _, _) = fixture();
    let temp = Temp::new();
    a.extra.insert("future".into(), serde_json::json!({"x":7}));
    a.tracks[0].keys[0]
        .extra
        .insert("hint".into(), serde_json::json!("kept"));
    let path = temp.0.join("assets/Timelines/Cast.timeline.json");
    fs::write(&path, serde_json::to_vec_pretty(&a).unwrap()).unwrap();
    let mut editor = TimelineEditor::default();
    editor.open(&path).unwrap();
    editor.edit(|a| a.name = "New".into());
    assert!(editor.dirty());
    editor.undo();
    assert!(!editor.dirty());
    editor.redo();
    assert!(editor.dirty());
    editor.save().unwrap();
    let restored = load(&path).unwrap();
    assert_eq!(restored.extra, a.extra);
    assert_eq!(restored.tracks[0].keys[0].extra, a.tracks[0].keys[0].extra);
    editor.edit(|a| a.name = "Unsaved".into());
    let mut external = restored.clone();
    external.name = "External".into();
    let external = serde_json::to_vec_pretty(&external).unwrap();
    fs::write(&path, &external).unwrap();
    assert!(editor.save().is_err());
    assert_eq!(fs::read(path).unwrap(), external);
    assert!(editor.dirty());
}
#[test]
fn shared_scanner_discovers_source_assets_and_duplicate_ids_fail() {
    let temp = Temp::new();
    let path = create(&temp.0, "Cast").unwrap();
    assert_eq!(load_all(&temp.0).unwrap().len(), 1);
    assert!(
        crate::assets::scan(&temp.0, &mut Default::default())
            .native
            .contains(&path)
    );
    assert!(create(&temp.0, "Cast").is_err());
    let copy = path.with_file_name("Copy.timeline.json");
    fs::copy(path, copy).unwrap();
    assert!(load_all(&temp.0).unwrap_err().contains("Duplicate"));
}

#[test]
fn playback_discovery_reads_nested_documents_and_observes_same_stamp_edits() {
    let temp = Temp::new();
    let timeline_path = create(&temp.0, "Cast").unwrap();
    let effect_path = crate::particle_effect::create(&temp.0, "Before").unwrap();
    let nested = temp.0.join("assets/Spells/Nested");
    fs::create_dir_all(&nested).unwrap();
    let moved = nested.join("Before.particle-effect.json");
    fs::rename(&effect_path, &moved).unwrap();
    // Unrelated media and malformed packages are not playback documents.
    fs::write(nested.join("music.wav"), b"unrelated media").unwrap();
    fs::write(nested.join("broken.epokasset"), b"invalid package").unwrap();
    let first = inspect_playback(&temp.0).unwrap();
    assert!(first.errors.is_empty());
    assert_eq!(first.timelines.len(), 1);
    assert_eq!(first.effects[0].0, moved);
    let before = fs::metadata(&moved).unwrap();
    let mut effect = first.effects[0].1.clone();
    effect.name = "After!".into();
    let bytes = serde_json::to_vec_pretty(&effect).unwrap();
    assert_eq!(bytes.len() as u64, before.len());
    fs::write(&moved, bytes).unwrap();
    fs::File::options()
        .write(true)
        .open(&moved)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(before.modified().unwrap()))
        .unwrap();
    let refreshed = inspect_playback(&temp.0).unwrap();
    assert_eq!(refreshed.effects[0].1.name, "After!");
    fs::remove_file(timeline_path).unwrap();
    fs::write(nested.join("Broken.timeline.json"), b"{").unwrap();
    let broken = inspect_playback(&temp.0).unwrap();
    assert!(broken.timelines.is_empty());
    assert_eq!(broken.effects.len(), 1);
    assert_eq!(broken.errors.len(), 1);
    assert!(broken.errors[0].contains("Broken.timeline.json"));
}
