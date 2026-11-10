//! Demand loading for SDK helpers. Unused helpers have no runtime factories or pools.
use crate::blueprint::Registry;
use std::collections::BTreeSet;
use std::path::Path;

pub const INTERACTION_CLASSES: &[(&str, &str)] = &[
    (
        "InteractableComponent",
        "310e882c-2353-55bc-bba3-f3d54277f671",
    ),
    (
        "InteractorComponent",
        "ebe82fcd-e9db-5de5-89a7-ed42f3a6d29a",
    ),
    (
        "InteractionSensor3DComponent",
        "ce7a9a84-40f2-500d-9b51-03de04fbfb8a",
    ),
    (
        "InteractionSensor2DComponent",
        "eaa0415b-21d7-5659-87b8-d0929c8756fc",
    ),
    ("TriggerBox3D", "177ad606-c610-5ca6-ba99-a6126b85595d"),
    ("TriggerArea2D", "1d128dde-89a7-516e-bc39-2b4c2eada9c5"),
    (
        "InteractionPromptComponent",
        "b7e56800-3c6f-5e1d-a165-848c122b51c5",
    ),
    ("InteractionPrompt", "b68bfec7-d1e2-5a99-8afd-14536a1196c3"),
];
pub const DIALOGUE_CLASSES: &[(&str, &str)] = &[
    (
        "DialogueInteractionComponent",
        "aa343ddf-5098-5b32-b993-d8b9b619d455",
    ),
    ("DialoguePresenter", "d40dc688-f7c0-50a3-9af8-7beaf1e3eea3"),
    (
        "DialogueRoleBindingsComponent",
        "bdd1ba5d-d32f-57b8-83f3-7b2f6cd37060",
    ),
    ("DialogueNPC3D", "3a5fa04e-d6e9-59f7-80b1-666f5eaf94af"),
    ("DialogueNPC2D", "9c3b4da8-a056-5016-b106-c58e1e057fb5"),
];
pub const UI_CLASSES: &[(&str, &str)] = &[(
    "ScreenAnchorComponent",
    "066be6a2-ec4e-50fd-a0bd-3dae2a7b8d73",
)];
pub const WORLD_GAMEPLAY_CLASSES: &[(&str, &str)] = &[
    (
        "GameplayFlagComponent",
        "397c4a8e-a044-59b5-ab51-a54d214f1789",
    ),
    ("PickupComponent", "117a8e4b-4df8-53da-b42e-58be2f3cae4f"),
    ("DoorComponent", "fd68c79f-403a-55f2-8f8f-2ef861166923"),
    ("SwitchComponent", "94fdf04c-4d40-5962-a6cc-2626e2bf1b1e"),
    ("ExaminableObject3D", "be73a9b8-baa1-5790-be17-f3dd20ac2d50"),
    ("ExaminableObject2D", "21c4d386-6ea9-548f-920e-3a5696820cd5"),
    ("Pickup3D", "2fa741a6-d51d-5b6b-ba1f-4e7b6df58a9e"),
    ("Pickup2D", "f118743d-70a1-5644-bd82-e5b816c80257"),
    ("Door3D", "ee1d25d5-a166-5827-9a6d-30221e91f6c6"),
    ("Door2D", "f7eb1ccc-0ac9-5c08-a210-7cea6912279b"),
    ("Switch3D", "ce9fc943-b001-5eff-aad4-3dbf93721250"),
    ("Switch2D", "1c070d4c-13a4-52af-9428-548f026c3e9c"),
    (
        "ProjectilePool3DComponent",
        "99c8c724-d8ee-5697-9d7f-927bfb9adc7e",
    ),
    (
        "PocketTransactionComponent",
        "abd3923b-cc40-589c-b755-5d607835a22f",
    ),
];
pub const CHARACTER_CLASSES: &[(&str, &str)] = &[
    (
        "CharacterMotor3DComponent",
        "34a2d5d2-b0bb-5faf-8dc4-7f34f07f9c95",
    ),
    ("Character3D", "a9264178-1960-5ec3-8407-2ef64bbdd9dd"),
    ("Character2D", "059ce53e-b93c-5f7c-9446-af075cf9898b"),
    (
        "CharacterMotor2DComponent",
        "705deefe-c507-5369-bcaf-c1b3d53f9aa9",
    ),
];
pub const CAMERA_CLASSES: &[(&str, &str)] = &[
    ("CameraRig3D", "c95b4c92-f2c9-5c5d-985a-ff4cc0f6c8c0"),
    ("FirstPersonCamera", "121b95af-7e66-50df-bcc1-7b61da65ae7b"),
    ("ThirdPersonCamera", "d43dbec0-82ca-5088-8b34-b7380fd4e1d4"),
    ("TopDownCamera", "1c40f100-3758-5838-b19a-f08d5744e2be"),
    ("IsometricCamera", "1c06764a-ac44-5a70-a9cf-ed827a4cf9ba"),
    ("SpringArmComponent", "37538bc0-b257-52af-be93-b344301667b1"),
    (
        "CameraOrbitInputComponent",
        "ad1162b7-8041-5495-8558-991f319cc5c8",
    ),
];
pub const COMMAND_SOURCE_CLASSES: &[(&str, &str)] = &[
    (
        "CharacterCommandSource3DComponent",
        "319e0e29-420a-5a7c-a115-d6eb2e6459ff",
    ),
    (
        "CharacterCommandSource2DComponent",
        "f0fcc71d-d59a-5292-b3b7-e0d1f2881d6d",
    ),
    (
        "ScriptedCommand3DComponent",
        "76c91654-0c63-5474-b44f-08019e831fa9",
    ),
    (
        "ScriptedCommand2DComponent",
        "1eb12c94-7e09-55fd-801f-1955f5e5cc2a",
    ),
    ("MoveTo3DComponent", "3393c8d2-55a0-5e23-999c-5160f5a1014c"),
    ("MoveTo2DComponent", "8c499172-ee35-5727-bc12-7f77b8a65165"),
    ("NPC3D", "308ddb5e-632e-5266-901f-a5ce234e481c"),
    ("NPC2D", "c25d7cb9-0c02-5130-8735-1a4864928ff5"),
];
pub const CHARACTER_INPUT_CLASSES: &[(&str, &str)] = &[
    (
        "LocalPlayer3DInputComponent",
        "73925989-7e27-5509-a493-9e6c1b71939d",
    ),
    (
        "LocalPlayer2DInputComponent",
        "26900842-e053-5f2c-9057-250242310406",
    ),
    ("Player3D", "e3832599-5fa1-5d20-9714-1d32934f63ef"),
    ("Player2D", "6551d454-ed4c-579f-9a85-95cae7eb3a19"),
];
pub const ANIMATION_CLASSES: &[(&str, &str)] = &[
    (
        "SkeletalLocomotionComponent",
        "af114d97-d97c-5aca-8c0e-c0bf179273d8",
    ),
    (
        "SpriteLocomotion2DComponent",
        "a55c1719-65fb-52f1-addf-7b397da706fd",
    ),
];
pub const QUERY_CLASSES: &[(&str, &str)] = &[(
    "SpatialQueryContextComponent",
    "382ad26d-3bc1-53e4-94a2-c1e18e6b25e9",
)];
pub fn optional(id: &str) -> bool {
    INTERACTION_CLASSES
        .iter()
        .chain(DIALOGUE_CLASSES.iter())
        .chain(UI_CLASSES.iter())
        .chain(WORLD_GAMEPLAY_CLASSES.iter())
        .chain(CHARACTER_CLASSES.iter())
        .chain(CAMERA_CLASSES.iter())
        .chain(COMMAND_SOURCE_CLASSES.iter())
        .chain(CHARACTER_INPUT_CLASSES.iter())
        .chain(ANIMATION_CLASSES.iter())
        .chain(QUERY_CLASSES.iter())
        .any(|(_, key)| *key == id)
}

fn sources(path: &Path, result: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    if !path.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            continue;
        }
        let path = entry.path();
        if kind.is_dir() {
            sources(&path, result)?;
        } else if path.extension().is_some_and(|e| {
            ["cpp", "cc", "hpp", "h", "lua", "epokbp"]
                .iter()
                .any(|s| e == *s)
        }) {
            result.push(path);
        }
    }
    Ok(())
}
pub fn source_demands(root: &Path, registry: &Registry) -> Result<BTreeSet<String>, String> {
    let mut files = Vec::new();
    sources(&root.join("assets"), &mut files)?;
    let mut result = BTreeSet::new();
    for path in files {
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let raw = String::from_utf8_lossy(&bytes).into_owned();
        let blueprint = path.extension().is_some_and(|s| s == "epokbp");
        let lua = path.extension().is_some_and(|s| s == "lua");
        if !blueprint && !path.starts_with(root.join("assets/scripts")) {
            continue;
        }
        let source = if blueprint {
            raw.clone()
        } else {
            crate::script_backend::native_code(&raw)
        };
        let literals = if !blueprint && !lua {
            crate::script_backend::cpp_without_comments(&raw)
        } else {
            String::new()
        };
        let identifiers: BTreeSet<String> = if lua {
            crate::lua_frontend::helper_reference_names(&path, &raw).map_err(|e| e.to_string())?
        } else {
            source
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .map(str::to_owned)
                .collect()
        };
        if !blueprint
            && !lua
            && crate::script_backend::native_requirements(&raw)?.contains("interaction")
        {
            result.extend(INTERACTION_CLASSES.iter().map(|(_, id)| (*id).to_owned()));
        }
        for (name, id) in INTERACTION_CLASSES
            .iter()
            .chain(DIALOGUE_CLASSES.iter())
            .chain(UI_CLASSES.iter())
            .chain(WORLD_GAMEPLAY_CLASSES.iter())
            .chain(CHARACTER_CLASSES.iter())
            .chain(CAMERA_CLASSES.iter())
            .chain(COMMAND_SOURCE_CLASSES.iter())
            .chain(CHARACTER_INPUT_CLASSES.iter())
            .chain(ANIMATION_CLASSES.iter())
            .chain(QUERY_CLASSES.iter())
        {
            let member = blueprint
                && registry
                    .classes
                    .get(*id)
                    .is_some_and(|c| c.functions.iter().any(|f| raw.contains(&f.id)));
            let literal = if blueprint {
                raw.contains(id)
            } else {
                !lua && literals.contains(id)
            };
            let compact =
                !lua && source.contains(&crate::blueprint_refs::compact_id(id).to_string());
            if identifiers.contains(*name) || member || literal || compact {
                result.insert((*id).to_owned());
            }
        }
    }
    Ok(result)
}

pub struct Retention {
    pub descriptors: BTreeSet<String>,
    pub factories: BTreeSet<String>,
}

/// Class selections are creation demands, including selections inside values.
/// Object/asset UUIDs are not class references and must not retain a factory.
pub fn referenced_classes(
    value_type: &crate::reflection_schema::Type,
    value: &serde_json::Value,
    result: &mut BTreeSet<String>,
) {
    use crate::reflection_schema::Type;
    match value_type {
        Type::ClassRef { .. } => {
            if let Some(id) = value.as_str() {
                result.insert(id.into());
            }
        }
        Type::Record { fields, .. } => {
            for field in fields {
                if let Some(value) = value.get(&field.name) {
                    referenced_classes(&field.value_type, value, result);
                }
            }
        }
        _ => {}
    }
}

pub fn retained(
    registry: &Registry,
    model: &crate::object_model::Model,
    capacities: &std::collections::BTreeMap<String, usize>,
) -> Retention {
    let mut keep: BTreeSet<String> = model
        .iter()
        .filter(|c| !optional(&c.id))
        .map(|c| c.id.clone())
        .collect();
    keep.extend(capacities.keys().cloned());
    keep.extend(registry.demanded_classes.iter().cloned());
    // Embedded controls need descriptors and callbacks when adopted. Retain their
    // normal creation surface too, without promoting every ancestor to a factory.
    loop {
        let before = keep.len();
        for key in keep.clone() {
            if let Some(class) = model.class(&key) {
                let mut ancestor = Some(class);
                while let Some(owner) = ancestor {
                    keep.extend(owner.default_components.iter().filter_map(|component| {
                        model.class(&component.class).map(|class| class.id.clone())
                    }));
                    ancestor = owner.parent.as_ref().and_then(|id| model.class(id));
                }
                let mut selected = BTreeSet::new();
                for property in registry.properties(&class.cpp_name) {
                    referenced_classes(&property.value_type, &property.default, &mut selected);
                }
                keep.extend(selected.into_iter().filter(|id| model.class(id).is_some()));
            }
        }
        if keep.len() == before {
            break;
        }
    }
    let factories = keep.clone();
    loop {
        let before = keep.len();
        for key in keep.clone() {
            let Some(class) = model.class(&key) else {
                continue;
            };
            if let Some(parent) = &class.parent {
                keep.insert(parent.clone());
            }
            for component in &class.default_components {
                if let Some(class) = model.class(&component.class) {
                    keep.insert(class.id.clone());
                }
            }
        }
        if keep.len() == before {
            return Retention {
                descriptors: keep,
                factories,
            };
        }
    }
}

/// Materialize the shared UI using the existing actor/component authoring path.
/// The caller owns the placement transaction; custom template bindings survive.
pub fn compose_dialogue_presenter(
    scene: &mut crate::scene::Scene,
    root: &Path,
    registry: &Registry,
    model: &crate::object_model::Model,
    index: usize,
) -> Result<(), String> {
    compose_dialogue_widget(
        scene,
        root,
        registry,
        model,
        index,
        crate::dialogue_asset::Presentation::ClassicBox,
    )
}
pub fn compose_dialogue_widget(
    scene: &mut crate::scene::Scene,
    root: &Path,
    registry: &Registry,
    model: &crate::object_model::Model,
    index: usize,
    presentation: crate::dialogue_asset::Presentation,
) -> Result<(), String> {
    let mut transaction = crate::helper_install::Transaction::new(root);
    let mut candidate = scene.clone();
    compose_dialogue_widget_in(
        &mut candidate,
        root,
        registry,
        model,
        index,
        presentation,
        &mut transaction,
    )?;
    candidate.validate_with_model(Some(model))?;
    *scene = candidate;
    transaction.commit();
    Ok(())
}
pub fn compose_dialogue_widget_in(
    scene: &mut crate::scene::Scene,
    root: &Path,
    registry: &Registry,
    model: &crate::object_model::Model,
    index: usize,
    presentation: crate::dialogue_asset::Presentation,
    transaction: &mut crate::helper_install::Transaction,
) -> Result<(), String> {
    compose_dialogue_widget_using(scene, root, registry, model, index, presentation, || {
        crate::dialogue_starter::skin_in(root, presentation, transaction)
    })
}

/// Resolve the same control composition without installing project resources.
/// Opaque texture identities describe pending resources, never runtime assets.
pub(crate) fn preview_dialogue_presenter(
    scene: &mut crate::scene::Scene,
    root: &Path,
    registry: &Registry,
    model: &crate::object_model::Model,
    index: usize,
) -> Result<(), String> {
    compose_dialogue_widget_using(
        scene,
        root,
        registry,
        model,
        index,
        crate::dialogue_asset::Presentation::ClassicBox,
        || {
            let texture = uuid::Uuid::new_v4();
            Ok(crate::dialogue_starter::Skin {
                panel: Some(texture),
                choices: texture,
            })
        },
    )
}

fn compose_dialogue_widget_using(
    scene: &mut crate::scene::Scene,
    root: &Path,
    registry: &Registry,
    model: &crate::object_model::Model,
    index: usize,
    presentation: crate::dialogue_asset::Presentation,
    skin: impl FnOnce() -> Result<crate::dialogue_starter::Skin, String>,
) -> Result<(), String> {
    let fields = [
        "panel",
        "portrait",
        "speaker_name",
        "body",
        "prompt",
        "continuation",
        "choice_panel",
        "choice0",
        "choice1",
        "choice2",
        "choice3",
    ];
    if fields.iter().any(|field| {
        scene.actors[index]
            .properties
            .get(*field)
            .is_some_and(|v| !v.is_null())
    }) {
        return Ok(());
    }
    let bubble = presentation == crate::dialogue_asset::Presentation::SpeechBubble;
    if scene.actors.len() + fields.len() + usize::from(bubble) > 64 {
        return Err("Dialogue widget controls exceed the 64-actor level capacity".into());
    }
    let defaults = crate::dialogue_starter::widget_preset(presentation, scene.display_size)?;
    let skin = skin()?;
    let layout = &defaults.layout;
    let mut parent = index;
    if bubble {
        parent = crate::editor::place_class(
            scene,
            root,
            registry,
            model,
            crate::object_model::UI_ACTOR_ID,
        )?;
        let container = &mut scene.actors[parent];
        container.name = "Bubble Content".into();
        container.parent = Some(index);
        container.rect = Some(crate::hud::RectTransform {
            anchor_min: [0.5, 0.],
            anchor_max: [0.5, 0.],
            pivot: [0.5, 0.],
            position: [0., f32::from(layout.panel[1])],
            size: [f32::from(layout.panel[2]), f32::from(layout.panel[3])],
            ..Default::default()
        });
        crate::actor_components::sync(container);
        let content = container
            .components
            .iter()
            .find(|c| {
                c.class.class_id.as_deref()
                    == Some(crate::object_model::RECT_TRANSFORM_COMPONENT_ID)
            })
            .ok_or("Bubble content needs a RectTransform component")?
            .id;
        let anchor_id = uuid::Uuid::new_v4();
        let mut anchor = crate::actor_document::ComponentInstance::new(
            anchor_id,
            crate::actor_document::ClassReference::new(
                "epok::ScreenAnchorComponent",
                UI_CLASSES[0].1,
            ),
            "Screen Anchor",
        );
        anchor
            .properties
            .insert("content".into(), serde_json::json!(content));
        scene.actors[index].components.push(anchor);
        scene.actors[index]
            .properties
            .insert("anchor".into(), serde_json::json!(anchor_id));
        scene.actors[index].overrides.insert("anchor".into());
    }
    for field in fields {
        let child = crate::editor::place_class(
            scene,
            root,
            registry,
            model,
            crate::object_model::UI_ACTOR_ID,
        )?;
        let popup = field == "choice_panel" || field.starts_with("choice");
        let field_parent = if bubble && popup { index } else { parent };
        let image = matches!(field, "panel" | "portrait" | "choice_panel");
        let (name, id) = if image {
            (
                "epok::ImageComponent",
                crate::object_model::IMAGE_COMPONENT_ID,
            )
        } else {
            (
                "epok::TextComponent",
                crate::object_model::TEXT_COMPONENT_ID,
            )
        };
        let component = uuid::Uuid::new_v4();
        let actor = &mut scene.actors[child];
        actor.name = format!("Dialogue {field}");
        actor.parent = Some(field_parent);
        actor
            .components
            .push(crate::actor_document::ComponentInstance::new(
                component,
                crate::actor_document::ClassReference::new(name, id),
                field,
            ));
        actor.refresh_components();
        let rect = match field {
            "panel" => layout.panel,
            "choice_panel" => layout.choice_panel,
            "body" => layout.body,
            "prompt" => defaults.prompt,
            "continuation" => defaults.continuation,
            "portrait" => layout.portrait,
            "speaker_name" => layout.name,
            _ => {
                let row = field
                    .strip_prefix("choice")
                    .and_then(|v| v.parse::<u16>().ok())
                    .unwrap();
                let mut r = layout.choices;
                r[3] /= 4;
                r[1] += (3 - row) * r[3];
                r
            }
        };
        actor.rect = Some(crate::hud::RectTransform {
            anchor_min: [0.; 2],
            anchor_max: [0.; 2],
            pivot: [0.; 2],
            position: [f32::from(rect[0]), f32::from(rect[1])],
            size: [f32::from(rect[2]), f32::from(rect[3])],
            ..Default::default()
        });
        let r = actor.rect.as_mut().unwrap();
        if bubble && !popup {
            r.position[0] -= f32::from(layout.panel[0]);
            r.position[1] -= f32::from(layout.panel[1]);
        } else if popup {
            let center = bubble || presentation == crate::dialogue_asset::Presentation::Subtitles;
            r.anchor_min[0] = if center { 0.5 } else { 1. };
            r.anchor_max[0] = r.anchor_min[0];
            r.position[0] -= f32::from(scene.display_size[0]) * r.anchor_min[0];
        } else if field == "continuation" {
            r.anchor_min[0] = 1.;
            r.anchor_max[0] = 1.;
            r.position[0] -= f32::from(scene.display_size[0]);
        } else if field != "portrait" {
            r.anchor_max[0] = 1.;
            r.size[0] -= f32::from(scene.display_size[0]);
        }
        if field == "portrait" {
            actor.active =
                presentation == crate::dialogue_asset::Presentation::PortraitConversation;
        }
        if field == "continuation" {
            actor.active = presentation != crate::dialogue_asset::Presentation::Subtitles;
        }
        if field == "speaker_name" && presentation == crate::dialogue_asset::Presentation::Subtitles
        {
            actor.active = false;
        }
        if let Some(text) = actor.text.as_mut() {
            text.enabled = false;
            text.text.clear();
            if field == "continuation" {
                text.text = ">".into();
            }
            text.color = if bubble && field.starts_with("choice") {
                [1.; 3]
            } else {
                layout.source.text_color
            };
            if presentation == crate::dialogue_asset::Presentation::Subtitles {
                text.align = crate::hud::TextAlign::Center;
            }
        }
        if let Some(image) = actor.image.as_mut() {
            image.enabled = false;
            image.color = if field == "portrait" {
                [1.; 3]
            } else {
                layout.source.panel_color
            };
            if field == "panel" {
                image.borders = layout.source.panel_borders;
                image.texture = skin.panel;
            } else if field == "choice_panel" {
                image.color = [1.; 3];
                image.borders = [4; 4];
                image.texture = Some(skin.choices);
            }
        }
        if field.starts_with("choice") && field != "choice_panel" {
            actor.focusable = Some(crate::hud::Focusable {
                highlight: [1., 0.9, 0.3],
                outline: true,
                ..Default::default()
            });
        }
        crate::actor_components::sync(actor);
        scene.actors[index]
            .properties
            .insert(field.into(), serde_json::json!(component));
        scene.actors[index].overrides.insert(field.into());
    }
    scene.actors[index].canvas = Some(Default::default());
    scene.actors[index].properties.insert(
        "default_timing".into(),
        serde_json::json!({"reveal_ms": if presentation == crate::dialogue_asset::Presentation::Subtitles {
                0
            } else {
                30
            }, "auto_advance_ms":0}),
    );
    scene.actors[index]
        .overrides
        .insert("default_timing".into());
    if bubble {
        scene.actors[index].canvas.as_mut().unwrap().screen_overlay = true;
    }
    scene.actors[index].rect = Some(crate::hud::RectTransform {
        anchor_min: [0.; 2],
        anchor_max: [1.; 2],
        pivot: [0.; 2],
        position: [0.; 2],
        size: [0.; 2],
        ..Default::default()
    });
    crate::actor_components::sync(&mut scene.actors[index]);
    scene.sync_actor_components();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projectile_pool_exposes_typed_bounded_records_without_unused_factories() {
        use crate::reflection_schema::Type;
        let root = crate::workspace::tests::temp("projectile-pool-contract");
        crate::workspace::create(&root, "Projectile Pool", crate::workspace::Template::Basic)
            .unwrap();
        let mut registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let id = "99c8c724-d8ee-5697-9d7f-927bfb9adc7e";
        let class = &registry.classes[id];
        assert_eq!(class.cpp_name, "epok::ProjectilePool3DComponent");
        assert!(class.blueprintable && !class.final_class && optional(id));
        for slot in 0..8 {
            let property = class
                .properties
                .iter()
                .find(|p| p.name == format!("visual{slot}"))
                .unwrap();
            assert!(
                matches!(&property.value_type, Type::ActorRef { class: Some(name) } if name == "epok::Actor3D")
            );
        }
        let context = class
            .properties
            .iter()
            .find(|p| p.name == "query_context")
            .unwrap();
        assert!(
            matches!(&context.value_type, Type::ComponentRef { class: Some(name) } if name == "epok::SpatialQueryContextComponent")
        );
        for name in [
            "binding",
            "active_count",
            "bind",
            "launch",
            "step",
            "result",
            "stop",
            "reset",
            "snapshot",
        ] {
            assert!(class.functions.iter().any(|f| f.name == name), "{name}");
        }
        assert!(!class.functions.iter().any(|f| f.name == "step_shared"));
        for function in &class.functions {
            assert!(function.returns.wire_words() <= 10, "{}", function.name);
            assert!(
                function
                    .parameters
                    .iter()
                    .all(|p| p.value_type.wire_words() <= 10),
                "{}",
                function.name
            );
        }
        for (name, words) in [
            ("ProjectileHandle", 2),
            ("ProjectileSpawn3D", 8),
            ("ProjectileTraceFilter", 3),
            ("ProjectileSpriteGrid", 7),
            ("ProjectileStep", 8),
            ("ProjectileBatch", 4),
            ("ProjectileLaunchResult", 4),
            ("ProjectileSnapshot", 10),
        ] {
            let value = registry
                .value_types
                .values()
                .find(|v| v.cpp_name == format!("epok::{name}"))
                .unwrap();
            assert_eq!(value.value_type.wire_words(), words, "{name}");
        }
        assert!(
            !registry
                .value_types
                .values()
                .any(|v| v.cpp_name == "epok::ProjectileStep4")
        );
        let library = registry
            .function_libraries
            .values()
            .find(|l| l.cpp_name == "epok::ProjectileLibrary")
            .unwrap();
        assert_eq!(library.operations.len(), 2);
        for operation in &library.operations {
            assert!(
                operation
                    .resource_demands
                    .iter()
                    .any(|s| s == "world-gameplay")
            );
            assert!(
                operation
                    .outputs
                    .iter()
                    .all(|p| p.value_type.wire_words() <= 10)
            );
        }
        let grid = registry
            .default_value("epok::ProjectilePool3DComponent", "sprite_grid")
            .unwrap();
        assert_eq!(grid["enabled"], false);
        assert_eq!(grid["frames"], 4);
        assert_eq!(grid["ticks_per_frame"], 2);
        let device_id = "abd3923b-cc40-589c-b755-5d607835a22f";
        let device = &registry.classes[device_id];
        assert!(device.blueprintable && !device.final_class && optional(device_id));
        for name in [
            "clear_request",
            "set_request_byte",
            "begin",
            "poll",
            "snapshot",
            "reply_byte",
            "cancel",
            "last_rejection",
        ] {
            assert!(device.functions.iter().any(|f| f.name == name), "{name}");
        }
        for (name, words) in [("PocketTransferHandle", 2), ("PocketTransferSnapshot", 6)] {
            let value = registry
                .value_types
                .values()
                .find(|v| v.cpp_name == format!("epok::{name}"))
                .unwrap();
            assert_eq!(value.value_type.wire_words(), words);
        }
        let card = registry
            .function_libraries
            .values()
            .find(|l| l.cpp_name == "epok::MemoryCardLibrary")
            .unwrap();
        for name in [
            "clear_staged_metadata",
            "set_staged_name_byte",
            "set_staged_title_byte",
            "read_staged_name",
            "write_staged_named",
        ] {
            assert!(card.operations.iter().any(|op| op.name == name), "{name}");
        }
        let empty = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(!empty.contains("ObjectPool<epok::ProjectilePool3DComponent"));
        assert!(!empty.contains("ObjectPool<epok::PocketTransactionComponent"));
        assert!(!empty.contains("EPOK_WORLD_GAMEPLAY_USED"));
        registry.demanded_classes.insert(id.into());
        let demanded =
            crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(demanded.contains("ObjectPool<epok::ProjectilePool3DComponent"));
        assert!(demanded.contains("EPOK_WORLD_GAMEPLAY_USED"));
        assert!(!demanded.contains("EPOK_INTERACTION_USED"));
        assert!(!demanded.contains("EPOK_DIALOGUE_USED"));
        assert!(
            crate::project::runtime_sources()
                .iter()
                .any(|(path, _)| *path == "projectile3d.hpp")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn world_gameplay_helpers_share_native_contracts_presets_and_optional_factories() {
        let root = crate::workspace::tests::temp("world-helper-contract");
        crate::workspace::create(&root, "World Helpers", crate::workspace::Template::Basic)
            .unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let model = registry.model().unwrap();
        for (name, id) in WORLD_GAMEPLAY_CLASSES {
            let class = &registry.classes[*id];
            assert_eq!(class.cpp_name, format!("epok::{name}"));
            assert!(class.blueprintable && !class.final_class);
            assert!(optional(id));
            assert_eq!(
                crate::game_helper_catalog::group(model.class(id).unwrap()),
                Some("World Gameplay")
            );
        }
        for name in ["PickupComponent", "SwitchComponent"] {
            let class = registry.named(&format!("epok::{name}")).unwrap();
            for name in ["can_interact", "interaction_started"] {
                assert!(
                    class
                        .functions
                        .iter()
                        .any(|f| f.name == name && f.final_method),
                    "{}: {name}",
                    class.cpp_name
                );
            }
        }
        let empty = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(!empty.contains("EPOK_WORLD_GAMEPLAY_USED"));
        for (name, _) in WORLD_GAMEPLAY_CLASSES {
            assert!(!empty.contains(&format!("ObjectPool<epok::{name}")));
        }
        for two in [false, true] {
            let mut scene = crate::scene::Scene {
                actors: vec![],
                ..Default::default()
            };
            for stem in [
                "ExaminableObject",
                "Pickup",
                "Door",
                "Switch",
                "DialogueNPC",
            ] {
                let name = format!("epok::{stem}{}D", if two { 2 } else { 3 });
                let class = registry.named(&name).unwrap();
                let index =
                    crate::editor::place_class(&mut scene, &root, &registry, &model, &class.id)
                        .unwrap();
                assert_eq!(
                    scene.actors[index].components.len(),
                    if stem == "Door" { 3 } else { 2 },
                    "{name}"
                );
            }
            scene.validate_with_model(Some(&model)).unwrap();
            let table = crate::blueprint_spawn::object_class_table_for_scenes(
                &registry,
                std::slice::from_ref(&scene),
            )
            .unwrap();
            assert!(
                table.contains("EPOK_WORLD_GAMEPLAY_USED")
                    && table.contains("EPOK_INTERACTION_USED")
            );
            let header = crate::project::scene_header_with_registry(
                &scene,
                &[],
                &scene,
                true,
                &scene,
                &registry,
            )
            .unwrap();
            assert!(
                header.contains("epok::DoorComponent") && header.contains("epok::PickupComponent")
            );
        }
        assert!(
            crate::project::runtime_sources()
                .iter()
                .any(|(path, _)| *path == "world_gameplay.hpp")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn world_gameplay_native_parents_are_derivable_in_all_three_providers() {
        let root = crate::workspace::tests::temp("world-helper-providers");
        crate::workspace::create(&root, "World Helpers", crate::workspace::Template::Basic)
            .unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        for (parent, id) in WORLD_GAMEPLAY_CLASSES.iter().take(4) {
            let native = format!("epok::{parent}");
            crate::scripts::create_in(&root, &format!("Cpp{parent}"), "", &native, false).unwrap();
            crate::blueprint_workflow::create(
                &root,
                &registry,
                &format!("BP{parent}"),
                "",
                id,
                false,
            )
            .unwrap();
            crate::lua_asset::create_in(&root, &format!("Lua{parent}"), "", &native).unwrap();
        }
        let registry = crate::scripts::declaration_registry(&root).unwrap();
        registry.model().unwrap();
        for (parent, id) in WORLD_GAMEPLAY_CLASSES.iter().take(4) {
            for prefix in ["Cpp", "BP", "Lua"] {
                let class = registry.named(&format!("{prefix}{parent}")).unwrap();
                assert_eq!(class.parent.as_deref(), Some(*id));
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn locomotion_drivers_use_editable_configuration_and_optional_factories() {
        let root = crate::workspace::tests::temp("locomotion-contract");
        crate::workspace::create(&root, "Locomotion", crate::workspace::Template::Basic).unwrap();
        let mut registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        for (name, words) in [
            ("LocomotionClips", 6),
            ("LocomotionTuning", 7),
            ("LocomotionSelection", 4),
            ("LocomotionSample", 5),
        ] {
            assert_eq!(
                registry
                    .value_types
                    .values()
                    .find(|v| v.cpp_name == format!("epok::{name}"))
                    .unwrap()
                    .value_type
                    .wire_words(),
                words,
                "{name}"
            );
        }
        let empty = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(!empty.contains("EPOK_CHARACTER_ANIMATION_USED"));
        for (name, id) in ANIMATION_CLASSES {
            let class = registry.classes.get(*id).unwrap();
            assert_eq!(class.cpp_name, format!("epok::{name}"));
            assert_eq!(
                registry.properties(&class.cpp_name).len(),
                if name.starts_with("Sprite") { 9 } else { 7 }
            );
            for method in [
                "select_animation",
                "animation_applied",
                "sample",
                "reset",
                "set_enabled",
                "update",
            ] {
                assert!(
                    class.functions.iter().any(|f| f.name == method),
                    "{name}::{method}"
                );
            }
            assert_eq!(
                registry.default_value(&class.cpp_name, "clips").unwrap()["land"],
                -1
            );
            registry.demanded_classes.insert((*id).into());
        }
        let header = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(header.contains("EPOK_CHARACTER_ANIMATION_USED"));
        assert!(!header.contains("EPOK_CHARACTER_INPUT_USED"));
        assert!(!header.contains("EPOK_INTERACTION_USED"));
        let staged = root.join("animation-sdk");
        std::fs::create_dir_all(&staged).unwrap();
        crate::project::stage_runtime(&staged).unwrap();
        assert_eq!(
            std::fs::read(staged.join("character_animation.hpp")).unwrap(),
            include_bytes!("../runtime/character_animation.hpp")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn command_producers_share_reflection_composition_and_optional_capabilities() {
        let root = crate::workspace::tests::temp("character-command-contract");
        crate::workspace::create(&root, "Commands", crate::workspace::Template::Basic).unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        for (name, id) in COMMAND_SOURCE_CLASSES.iter().chain(CHARACTER_INPUT_CLASSES) {
            assert_eq!(
                registry.classes.get(*id).unwrap().cpp_name,
                format!("epok::{name}")
            );
        }
        for (name, words) in [
            ("CharacterCommandResult3D", 6),
            ("CharacterCommandResult2D", 4),
        ] {
            assert_eq!(
                registry
                    .value_types
                    .values()
                    .find(|v| v.cpp_name == format!("epok::{name}"))
                    .unwrap()
                    .value_type
                    .wire_words(),
                words
            );
        }
        for name in [
            "CharacterCommandSource3DComponent",
            "CharacterCommandSource2DComponent",
        ] {
            let class = registry.named(&format!("epok::{name}")).unwrap();
            for method in [
                "calculate_command",
                "command_submitted",
                "update",
                "sample",
                "release_command",
                "set_enabled",
            ] {
                assert!(
                    class.functions.iter().any(|f| f.name == method),
                    "{name}::{method}"
                );
            }
        }
        let empty = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(!empty.contains("EPOK_CHARACTER_COMMANDS_USED"));
        assert!(!empty.contains("EPOK_CHARACTER_INPUT_USED"));
        let model = registry.model().unwrap();
        for (name, count, input) in [
            ("Player3D", 4, true),
            ("Player2D", 5, true),
            ("NPC3D", 4, false),
            ("NPC2D", 5, false),
        ] {
            let class = registry.named(&format!("epok::{name}")).unwrap();
            let mut scene = crate::scene::Scene::default();
            scene.actors.clear();
            let index = crate::editor::place_class(&mut scene, &root, &registry, &model, &class.id)
                .unwrap();
            assert_eq!(scene.actors[index].components.len(), count);
            scene.validate().unwrap();
            let header =
                crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[scene]).unwrap();
            assert!(header.contains("EPOK_CHARACTER_COMMANDS_USED"));
            assert_eq!(header.contains("EPOK_CHARACTER_INPUT_USED"), input);
            assert!(!header.contains("EPOK_DIALOGUE_USED"));
            assert!(!header.contains("EPOK_INTERACTION_USED"));
        }
        for name in ["ScriptedCommand3DComponent", "ScriptedCommand2DComponent"] {
            let defaults = registry
                .default_value(&format!("epok::{name}"), "desired")
                .unwrap();
            assert_eq!(defaults["jump"], false);
        }
        let declaration = |property: &str| {
            format!(
                "#pragma once\n#include \"epok.hpp\"\nclass EPOK_CLASS(Blueprintable) ConfiguredSource:public epok::ScriptedCommand3DComponent{{public:EPOK_PROPERTY(EditAnywhere) {property};}};"
            )
        };
        let path = root.join("assets/scripts/ConfiguredSource.hpp");
        std::fs::write(
            &path,
            declaration("epok::CharacterCommand3D route{{1.0,-2.0},90.0,true,false}"),
        )
        .unwrap();
        let configured = crate::blueprint::native_registry(&root, &[]).unwrap();
        let value = configured
            .default_value("ConfiguredSource", "route")
            .unwrap();
        assert_eq!(value["velocity"]["y"], -2.0);
        assert_eq!(value["heading"], 90.0);
        assert_eq!(value["turn"], true);
        for property in [
            "epok::CharacterCommand3D route{}",
            "epok::CharacterCommand3D route{{1.0,2.0},90.0,true}",
            "epok::CharacterMotionSample3D route{}",
            "epok::CharacterCommand3D route{{epok::MathLibrary::abs(1.0),2.0},90.0,true,false}",
        ] {
            std::fs::write(&path, declaration(property)).unwrap();
            assert!(
                crate::blueprint::native_registry(&root, &[]).is_err(),
                "{property}"
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn character_classes_compose_native_bodies_and_keep_unused_pools_out() {
        let root = crate::workspace::tests::temp("character-class-contract");
        crate::workspace::create(&root, "Characters", crate::workspace::Template::Basic).unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let model = registry.model().unwrap();
        let empty = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        for name in ["Character3D", "Character2D", "CharacterMotor2DComponent"] {
            assert!(!empty.contains(&format!("ObjectPool<epok::{name}")));
        }
        for (name, count) in [("epok::Character3D", 3), ("epok::Character2D", 4)] {
            let mut scene = crate::scene::Scene::default();
            scene.actors.clear();
            let class = registry.named(name).unwrap();
            let index = crate::editor::place_class(&mut scene, &root, &registry, &model, &class.id)
                .unwrap();
            assert_eq!(scene.actors.len(), 1);
            assert_eq!(scene.actors[index].components.len(), count);
            if name.ends_with("3D") {
                assert!(scene.actors[index].collider.as_ref().unwrap().enabled);
            } else {
                for id in [
                    "aad9ca37-7c63-4ca9-8a23-5f4ff49e35fe",
                    "8a092515-6374-4792-985c-9ff9a967a82d",
                ] {
                    assert!(
                        scene.actors[index]
                            .components
                            .iter()
                            .any(|c| c.class.class_id.as_deref() == Some(id))
                    );
                }
            }
            scene.validate().unwrap();
            let header =
                crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[scene]).unwrap();
            assert!(header.contains(&format!("ObjectPool<{name}")));
            assert!(header.contains("epok::TickGroupOf<"));
        }
        for (name, words) in [
            ("epok::CharacterCommand2D", 3),
            ("epok::CharacterMotionSample2D", 10),
        ] {
            assert_eq!(
                registry
                    .value_types
                    .values()
                    .find(|v| v.cpp_name == name)
                    .unwrap()
                    .value_type
                    .wire_words(),
                words
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn camera_presets_have_inspectable_defaults_and_optional_runtime_pools() {
        let root = crate::workspace::tests::temp("camera-preset-contract");
        crate::workspace::create(&root, "Camera Presets", crate::workspace::Template::Basic)
            .unwrap();
        let mut registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        for (name, id) in CAMERA_CLASSES {
            assert_eq!(
                registry.classes.get(*id).unwrap().cpp_name,
                format!("epok::{name}")
            );
        }
        for (name, words) in [("CameraRigSample", 9), ("SpringArmSample", 6)] {
            let value = registry
                .value_types
                .values()
                .find(|v| v.cpp_name == format!("epok::{name}"))
                .unwrap();
            assert_eq!(value.value_type.wire_words(), words);
        }
        let model = registry.model().unwrap();
        for (name, distance, pitch, yaw, components) in [
            ("FirstPersonCamera", 0.0, 0.0, 0.0, 2),
            ("ThirdPersonCamera", 5.4, 18.0, 0.0, 3),
            ("TopDownCamera", 10.0, 90.0, 0.0, 2),
            ("IsometricCamera", 8.0, 35.264, 45.0, 2),
        ] {
            let name = format!("epok::{name}");
            let properties = registry.properties(&name);
            assert_eq!(properties.len(), 13);
            for (property, expected) in [("distance", distance), ("pitch", pitch), ("yaw", yaw)] {
                assert_eq!(
                    registry.default_value(&name, property).unwrap().as_f64(),
                    Some(expected)
                );
                assert_eq!(
                    properties
                        .iter()
                        .find(|p| p.name == property)
                        .unwrap()
                        .default
                        .as_f64(),
                    Some(expected)
                );
            }
            let class = registry.named(&name).unwrap();
            let mut scene = crate::scene::Scene::default();
            scene.actors.clear();
            let index = crate::editor::place_class(&mut scene, &root, &registry, &model, &class.id)
                .unwrap();
            assert_eq!(scene.actors.len(), 1);
            assert_eq!(scene.actors[index].components.len(), components);
            assert_eq!(scene.actors[index].kind, "Camera");
        }
        let empty = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(!empty.contains("ObjectPool<epok::FirstPersonCamera"));
        assert!(!empty.contains("ObjectPool<epok::SpringArmComponent"));
        assert!(!empty.contains("EPOK_CAMERA_INPUT_USED"));
        assert!(!empty.contains("EPOK_CAMERA_RIG_USED"));
        registry.demanded_classes.insert(CAMERA_CLASSES[2].1.into());
        let third = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(third.contains("ObjectPool<epok::ThirdPersonCamera"));
        assert!(third.contains("ObjectPool<epok::SpringArmComponent"));
        assert!(third.contains("EPOK_CAMERA_RIG_USED"));
        assert!(!third.contains("EPOK_CAMERA_INPUT_USED"));
        registry.demanded_classes.insert(CAMERA_CLASSES[6].1.into());
        assert!(
            crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[])
                .unwrap()
                .contains("EPOK_CAMERA_INPUT_USED")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn nested_class_selections_retain_factories_without_confusing_instance_or_asset_ids() {
        use crate::reflection_schema::{RecordField, Type};
        let config = Type::Record {
            cpp_name: "FactoryConfig".into(),
            fields: vec![
                RecordField {
                    name: "selected".into(),
                    value_type: Type::ClassRef {
                        base: crate::object_model::ACTOR3D_ID.into(),
                    },
                },
                RecordField {
                    name: "nested".into(),
                    value_type: Type::Record {
                        cpp_name: "NestedFactoryConfig".into(),
                        fields: vec![
                            RecordField {
                                name: "selected".into(),
                                value_type: Type::ClassRef {
                                    base: crate::object_model::ACTOR3D_ID.into(),
                                },
                            },
                            RecordField {
                                name: "target".into(),
                                value_type: Type::ActorRef { class: None },
                            },
                            RecordField {
                                name: "texture".into(),
                                value_type: Type::AssetRef {
                                    kind: "Texture".into(),
                                },
                            },
                        ],
                    },
                },
            ],
        };
        let value = serde_json::json!({"selected": CAMERA_CLASSES[0].1, "nested": {
            "selected": CAMERA_CLASSES[2].1, "target": CAMERA_CLASSES[3].1, "texture": CAMERA_CLASSES[4].1
        }});
        let mut selected = BTreeSet::new();
        referenced_classes(&config, &value, &mut selected);
        assert_eq!(
            selected,
            [CAMERA_CLASSES[0].1.into(), CAMERA_CLASSES[2].1.into()]
                .into_iter()
                .collect()
        );
        selected.clear();
        referenced_classes(
            &config,
            &serde_json::json!({"selected": null}),
            &mut selected,
        );
        assert!(selected.is_empty());
    }
    #[test]
    fn inherited_helper_metadata_does_not_reserve_an_unused_ancestor_factory() {
        let root = crate::workspace::tests::temp("helper-ancestor-factory");
        crate::workspace::create(&root, "Helper Ancestors", crate::workspace::Template::Basic)
            .unwrap();
        crate::scripts::create_in(&root, "ProjectCamera", "", "epok::ThirdPersonCamera", false)
            .unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let mut registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let model = registry.model().unwrap();
        let derived = registry.named("ProjectCamera").unwrap().id.clone();
        let mut scene = crate::scene::Scene::default();
        scene.actors.clear();
        crate::editor::place_class(&mut scene, &root, &registry, &model, &derived).unwrap();
        let table =
            crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[scene.clone()])
                .unwrap();
        let ancestor_id = crate::blueprint_refs::compact_id(CAMERA_CLASSES[0].1);
        let ancestor = table
            .lines()
            .find(|line| line.starts_with(&format!("{{UINT64_C({ancestor_id}),")))
            .unwrap();
        assert!(ancestor.contains("nullptr,nullptr,0,0,nullptr,nullptr"));
        assert!(!table.contains("SharedObjectPool<epok::CameraRig3D"));
        assert!(table.contains("SharedObjectPool<ProjectCamera,5>"));
        assert!(table.contains("&self.camera"));
        assert!(table.contains("&self.boom"));
        // An Inspector ClassRef selection creates a native factory even when the
        // selected helper appears nowhere in the project's source text.
        let mut property = registry.named("epok::CameraRig3D").unwrap().properties[0].clone();
        property.id = uuid::Uuid::new_v4().to_string();
        property.name = "spawn_camera".into();
        property.value_type = crate::reflection_schema::Type::ClassRef {
            base: crate::object_model::ACTOR3D_ID.into(),
        };
        property.default = serde_json::Value::Null;
        registry
            .classes
            .get_mut(&derived)
            .unwrap()
            .properties
            .push(property);
        scene.actors[0]
            .properties
            .insert("spawn_camera".into(), CAMERA_CLASSES[0].1.into());
        let selected =
            crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[scene.clone()])
                .unwrap();
        assert!(selected.contains("SharedObjectPool<epok::CameraRig3D,4>"));
        scene.actors[0]
            .properties
            .insert("spawn_camera".into(), serde_json::Value::Null);
        let cleared =
            crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[scene.clone()])
                .unwrap();
        assert!(!cleared.contains("SharedObjectPool<epok::CameraRig3D"));
        registry
            .classes
            .get_mut(&derived)
            .unwrap()
            .properties
            .last_mut()
            .unwrap()
            .default = CAMERA_CLASSES[0].1.into();
        let default =
            crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(default.contains("SharedObjectPool<epok::CameraRig3D,4>"));
        registry
            .classes
            .get_mut(&derived)
            .unwrap()
            .properties
            .last_mut()
            .unwrap()
            .default = serde_json::Value::Null;
        // Naming the native class explicitly must still allow its direct creation.
        registry.demanded_classes.insert(CAMERA_CLASSES[0].1.into());
        let direct =
            crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[scene]).unwrap();
        assert!(direct.contains("SharedObjectPool<epok::CameraRig3D,4>"));
        std::fs::write(
            root.join("assets/scripts/Factory.lua"),
            r#"
local Factory = epok.Actor3D:extend()
-- TopDownCamera is only a comment, not a creation dependency.
function Factory:begin_play()
    local camera = epok.spawn("epok::CameraRig3D")
end
return Factory
"#,
        )
        .unwrap();
        let demands = source_demands(&root, &registry).unwrap();
        assert!(demands.contains(CAMERA_CLASSES[0].1));
        assert!(!demands.contains(CAMERA_CLASSES[3].1));
        // Native code can construct the same compact class identity from its
        // stable UUID rather than mentioning the helper's C++ type name.
        std::fs::write(
            root.join("assets/scripts/Factory.cpp"),
            format!("#include \"epok.hpp\"\n// Unused class: {}\nconstexpr auto factory_type = epok::detail::compact_class_id(\"{}\");\n", CAMERA_CLASSES[3].1, CAMERA_CLASSES[4].1),
        ).unwrap();
        let demands = source_demands(&root, &registry).unwrap();
        assert!(demands.contains(CAMERA_CLASSES[4].1));
        assert!(!demands.contains(CAMERA_CLASSES[3].1));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn native_constructor_defaults_are_constant_and_inherit_without_shadowing() {
        let root = crate::workspace::tests::temp("native-constructor-defaults");
        crate::workspace::create(&root, "Native Defaults", crate::workspace::Template::Basic)
            .unwrap();
        let path = root.join("assets/scripts/Defaults.hpp");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let prefix = "#pragma once\n#include \"epok.hpp\"\nclass EPOK_CLASS(Blueprintable) DefaultBase: public epok::Actor3D { public: EPOK_PROPERTY(EditAnywhere) epok::Fixed speed=2.0; EPOK_PROPERTY(EditAnywhere) bool enabled=true; EPOK_PROPERTY(EditAnywhere) uint32_t count=0; int private_state=0; static double roll(){return 3.0;} };\n";
        let source = format!(
            "{prefix}class EPOK_CLASS(Blueprintable) DefaultChild: public DefaultBase {{public: DefaultChild(){{speed=7.5; /* Constant defaults */ enabled=false;count=3u;}}}};\nclass EPOK_CLASS(Blueprintable) DefaultLeaf:public DefaultChild {{public: DefaultLeaf(){{speed=-1.25;}}}};"
        );
        std::fs::write(&path, source).unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        assert_eq!(registry.properties("DefaultLeaf").len(), 3);
        let scripts = crate::scripts::native_catalog(&root).unwrap();
        let leaf = scripts.iter().find(|s| s.name == "DefaultLeaf").unwrap();
        assert_eq!(
            leaf.properties
                .iter()
                .find(|p| p.name == "speed")
                .unwrap()
                .default
                .as_f64(),
            Some(-1.25)
        );
        assert_eq!(
            registry
                .default_value("DefaultBase", "speed")
                .unwrap()
                .as_f64(),
            Some(2.0)
        );
        assert_eq!(
            registry
                .default_value("DefaultChild", "speed")
                .unwrap()
                .as_f64(),
            Some(7.5)
        );
        assert_eq!(
            registry
                .default_value("DefaultLeaf", "speed")
                .unwrap()
                .as_f64(),
            Some(-1.25)
        );
        assert_eq!(
            registry
                .default_value("DefaultLeaf", "count")
                .unwrap()
                .as_u64(),
            Some(3)
        );
        assert_eq!(
            registry
                .default_value("DefaultLeaf", "enabled")
                .unwrap()
                .as_bool(),
            Some(false)
        );
        let base = registry.properties("DefaultBase");
        for property in registry.properties("DefaultLeaf") {
            assert_eq!(
                base.iter().find(|p| p.name == property.name).unwrap().id,
                property.id
            );
        }
        for constructor in [
            "DefaultChild(int count){speed=2.0;}",
            "DefaultChild():DefaultBase(){speed=2.0;}",
            "DefaultChild(){speed=roll();}",
            "DefaultChild(){speed+=1.0;}",
            "DefaultChild(){private_state=1;}",
            "DefaultChild(){speed=1.0;speed=2.0;}",
            "DefaultChild(){speed=1;}",
        ] {
            std::fs::write(&path, format!("{prefix}class EPOK_CLASS(Blueprintable) DefaultChild:public DefaultBase{{public:{constructor}}};")).unwrap();
            let error = crate::blueprint::native_registry(&root, &[])
                .err()
                .unwrap_or_else(|| panic!("Unexpectedly accepted {constructor}"));
            assert!(
                error.contains("constructor") || error.contains("Constructor"),
                "{constructor}: {error}"
            );
        }
        for replacement in ["false", "(private_state=1,false)"] {
            std::fs::write(&path, format!("{prefix}\n#define true {replacement}\nclass EPOK_CLASS(Blueprintable) DefaultChild:public DefaultBase{{public:DefaultChild(){{enabled=true;}}}};")).unwrap();
            let error = crate::blueprint::native_registry(&root, &[])
                .err()
                .unwrap_or_else(|| panic!("Unexpectedly accepted macro {replacement}"));
            assert!(
                error.contains("constructor") || error.contains("Constructor"),
                "{replacement}: {error}"
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn character_motor_uses_shared_reflected_records_without_an_unused_pool() {
        let root = crate::workspace::tests::temp("character-motor-contract");
        crate::workspace::create(&root, "Character Motor", crate::workspace::Template::Basic)
            .unwrap();
        let mut registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let id = "34a2d5d2-b0bb-5faf-8dc4-7f34f07f9c95";
        let class = registry.classes.get(id).unwrap();
        assert_eq!(class.cpp_name, "epok::CharacterMotor3DComponent");
        assert_eq!(class.properties.len(), 10);
        for name in [
            "submit",
            "stop",
            "motion",
            "reset_motion",
            "simulate",
            "simulate_shared",
            "set_facing",
            "clear_velocity",
        ] {
            assert!(class.functions.iter().any(|f| f.name == name), "{name}");
        }
        for (name, words) in [("CharacterCommand3D", 5), ("CharacterMotionSample3D", 13)] {
            let declaration = registry
                .value_types
                .values()
                .find(|v| v.cpp_name == format!("epok::{name}"))
                .unwrap();
            assert_eq!(declaration.value_type.wire_words(), words, "{name}");
        }
        let empty = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(!empty.contains("ObjectPool<epok::CharacterMotor3DComponent"));
        registry.demanded_classes.insert(id.into());
        let demanded =
            crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(demanded.contains("ObjectPool<epok::CharacterMotor3DComponent"));
        let staged = root.join("motor-sdk");
        std::fs::create_dir_all(&staged).unwrap();
        crate::project::stage_runtime(&staged).unwrap();
        assert_eq!(
            std::fs::read(staged.join("character3d.hpp")).unwrap(),
            include_bytes!("../runtime/character3d.hpp")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn screen_anchor_is_an_independent_optional_ui_component() {
        let root = crate::workspace::tests::temp("ui-anchor-contract");
        crate::workspace::create(&root, "UI Anchor", crate::workspace::Template::Basic).unwrap();
        let mut registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let class = registry.classes.get(UI_CLASSES[0].1).unwrap();
        assert_eq!(class.cpp_name, "epok::ScreenAnchorComponent");
        for name in ["valid_content", "refresh", "set_target", "status"] {
            assert!(class.functions.iter().any(|f| f.name == name));
        }
        let empty = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(!empty.contains("EPOK_UI_ANCHOR_USED"));
        assert!(!empty.contains("ObjectPool<epok::ScreenAnchorComponent"));
        registry.demanded_classes.insert(UI_CLASSES[0].1.into());
        let header = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(header.contains("EPOK_UI_ANCHOR_USED"));
        assert!(header.contains("ObjectPool<epok::ScreenAnchorComponent"));
        assert!(!header.contains("EPOK_INTERACTION_USED"));
        assert!(!header.contains("EPOK_DIALOGUE_USED"));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_session_facade_uses_registered_records_and_bounded_wire_values() {
        let root = crate::workspace::tests::temp("dialogue-session-api");
        crate::workspace::create(
            &root,
            "Dialogue Session API",
            crate::workspace::Template::Basic,
        )
        .unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let library = registry
            .function_libraries
            .values()
            .find(|l| l.cpp_name == "epok::DialogueSessionLibrary")
            .unwrap();
        assert_eq!(library.operations.len(), 8);
        for operation in &library.operations {
            assert!(operation.resource_demands.iter().any(|s| s == "dialogue"));
            assert!(
                operation
                    .parameters
                    .iter()
                    .all(|p| p.value_type.wire_words() <= 32)
            );
            assert!(
                operation
                    .outputs
                    .iter()
                    .all(|p| p.value_type.wire_words() <= 32)
            );
        }
        for (name, words) in [
            ("DialogueHandle", 1),
            ("DialogueStartResult", 2),
            ("DialogueSnapshot", 16),
        ] {
            let declaration = registry
                .value_types
                .values()
                .find(|v| v.cpp_name == format!("epok::{name}"))
                .unwrap();
            assert_eq!(declaration.value_type.wire_words(), words, "{name}");
        }
        let snapshot = library
            .operations
            .iter()
            .find(|o| o.name == "snapshot")
            .unwrap();
        assert_eq!(snapshot.outputs[0].value_type.wire_words(), 16);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_helpers_place_shared_presenter_and_preserve_component_bindings() {
        let root = crate::workspace::tests::temp("dialogue-presenter-placement");
        crate::workspace::create(
            &root,
            "Dialogue Presenter",
            crate::workspace::Template::Basic,
        )
        .unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let model = registry.model().unwrap();
        let mut scene = crate::scene::Scene::default();
        scene.actors.clear();
        let empty = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(
            !empty.contains("EPOK_DIALOGUE_USED") && !empty.contains("ObjectPool<epok::Dialogue")
        );
        let index =
            crate::editor::place_class(&mut scene, &root, &registry, &model, DIALOGUE_CLASSES[1].1)
                .unwrap();
        assert_eq!(scene.actors.len(), 12);
        assert_eq!(
            scene.actors[index].components.len(),
            3,
            "root, canvas and audio"
        );
        for field in [
            "panel",
            "choice_panel",
            "body",
            "prompt",
            "continuation",
            "speaker_name",
            "portrait",
            "choice0",
            "choice1",
            "choice2",
            "choice3",
        ] {
            let id = scene.actors[index].properties[field].as_str().unwrap();
            assert!(
                scene.actors.iter().skip(1).any(|a| a.parent == Some(index)
                    && a.components.iter().any(|c| c.id.to_string() == id)),
                "{field}"
            );
        }
        scene.validate().unwrap();
        let original = scene.actors[index].id;
        let original_body = scene.actors[index].properties["body"].clone();
        let duplicate = scene.duplicate_actor_branch(original).unwrap();
        let copied = scene.actors.iter_mut().find(|a| a.id == duplicate).unwrap();
        copied.name = "Alternate presenter".into();
        let copied_body = copied.properties["body"].clone();
        assert_ne!(original_body, copied_body);
        assert!(
            scene
                .actors
                .iter()
                .filter(|a| a.logical_parent == Some(duplicate))
                .any(|a| a
                    .components
                    .iter()
                    .any(|c| serde_json::json!(c.id) == copied_body))
        );
        let map = root.join("assets/scenes/DialoguePresenters.epokmap");
        std::fs::write(&map, crate::document::to_vec(&scene).unwrap()).unwrap();
        let scene_from_disk = crate::scene::Scene::load(&map).unwrap();
        scene_from_disk.validate().unwrap();
        for (owner, body) in [(original, original_body), (duplicate, copied_body)] {
            let placed = scene_from_disk
                .actors
                .iter()
                .find(|a| a.id == owner)
                .unwrap();
            assert_eq!(placed.properties["body"], body);
            assert!(
                scene_from_disk
                    .actors
                    .iter()
                    .any(|a| a.logical_parent == Some(owner)
                        && a.components.iter().any(|c| serde_json::json!(c.id) == body))
            );
        }
        let header = crate::blueprint_spawn::object_class_table_for_scenes(
            &registry,
            std::slice::from_ref(&scene),
        )
        .unwrap();
        assert!(
            header.contains("EPOK_DIALOGUE_USED")
                && header.contains("ObjectPool<epok::DialoguePresenter")
        );
        while scene.actors.len() < 56 {
            crate::editor::place_class(
                &mut scene,
                &root,
                &registry,
                &model,
                crate::object_model::UI_ACTOR_ID,
            )
            .unwrap();
        }
        let before = scene.actors.len();
        assert!(
            crate::editor::place_class(&mut scene, &root, &registry, &model, DIALOGUE_CLASSES[1].1)
                .unwrap_err()
                .contains("64-actor")
        );
        assert_eq!(
            scene.actors.len(),
            before,
            "failed placement is transactional"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn interaction_native_contract_and_demand_loading() {
        let root = crate::workspace::tests::temp("interaction-contract");
        crate::workspace::create(
            &root,
            "Interaction Contract",
            crate::workspace::Template::Basic,
        )
        .unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let model = registry.model().unwrap();
        assert!(!crate::world2d::project_uses_world2d(
            std::iter::empty(),
            &registry
        ));
        for (name, id) in INTERACTION_CLASSES {
            assert_eq!(model.class(id).unwrap().cpp_name, format!("epok::{name}"));
        }
        let empty = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(!empty.contains("EPOK_INTERACTION_USED"));
        assert!(!empty.contains("ObjectPool<epok::InteractableComponent"));
        let target = registry
            .classes
            .get("310e882c-2353-55bc-bba3-f3d54277f671")
            .unwrap();
        assert!(
            target
                .functions
                .iter()
                .any(|f| f.name == "can_interact" && f.pure && f.event)
        );
        let sensor = registry
            .classes
            .get("ce7a9a84-40f2-500d-9b51-03de04fbfb8a")
            .unwrap();
        assert!(sensor.properties.iter().any(|p| p.name=="target"&&matches!(&p.value_type,crate::reflection_schema::Type::ComponentRef{class:Some(class)} if class=="epok::InteractableComponent")));
        // Use the real actor picker path, then duplicate and reparent a linked
        // target/sensor branch before saving and cooking its typed references.
        let mut scene = crate::scene::Scene::default();
        scene.actors.clear();
        let target_index = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            crate::object_model::ACTOR3D_ID,
        )
        .unwrap();
        let target_id = scene.actors[target_index].id;
        let mut endpoint = crate::actor_document::ComponentInstance::new(
            uuid::Uuid::new_v4(),
            crate::actor_document::ClassReference::new(
                "epok::InteractableComponent",
                "310e882c-2353-55bc-bba3-f3d54277f671",
            ),
            "Examine",
        );
        endpoint
            .properties
            .insert("prompt_key".into(), serde_json::json!(0));
        endpoint.overrides.insert("prompt_key".into());
        let endpoint_id = endpoint.id;
        scene.actors[target_index].components.push(endpoint);
        let sensor_index = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            "177ad606-c610-5ca6-ba99-a6126b85595d",
        )
        .unwrap();
        scene.actors[sensor_index].logical_parent = Some(target_id);
        let sensor = scene.actors[sensor_index]
            .components
            .iter_mut()
            .find(|c| c.class.class_id.as_deref() == Some("ce7a9a84-40f2-500d-9b51-03de04fbfb8a"))
            .unwrap();
        sensor
            .properties
            .insert("target".into(), serde_json::json!(endpoint_id));
        sensor.overrides.insert("target".into());
        let duplicate = scene.duplicate_actor_branch(target_id).unwrap();
        let copy_endpoint = scene
            .actors
            .iter()
            .find(|a| a.id == duplicate)
            .unwrap()
            .components
            .iter()
            .find(|c| c.class.class_id.as_deref() == Some("310e882c-2353-55bc-bba3-f3d54277f671"))
            .unwrap()
            .id;
        let copy_sensor = scene
            .actors
            .iter_mut()
            .find(|a| a.logical_parent == Some(duplicate))
            .unwrap();
        copy_sensor.name = "Renamed sensor".into();
        assert_eq!(
            copy_sensor
                .components
                .iter()
                .find(|c| c.properties.contains_key("target"))
                .unwrap()
                .properties["target"],
            serde_json::json!(copy_endpoint)
        );
        assert_ne!(copy_endpoint, endpoint_id);
        // Reparenting changes order, not the endpoint identity.
        copy_sensor.logical_parent = Some(target_id);
        let prompt_index = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &model,
            "b68bfec7-d1e2-5a99-8afd-14536a1196c3",
        )
        .unwrap();
        assert!(scene.actors[prompt_index].text.is_some());
        let map = root.join("assets/scenes/Interaction.epokmap");
        std::fs::write(&map, crate::document::to_vec(&scene).unwrap()).unwrap();
        let mut scene = crate::scene::Scene::load(&map).unwrap();
        let empty_index = crate::assets::scan(&root, &mut Default::default());
        assert!(
            crate::localization::resolve_runtime(&mut scene, &empty_index, &registry)
                .unwrap_err()
                .contains("prompt_key 0")
        );
        crate::localization::create(
            &root,
            "assets/Prompts.epokasset",
            &crate::localization::StringTable {
                languages: vec!["English".into(), "Spanish".into()],
                entries: vec![crate::localization::Entry {
                    key: "interaction.examine".into(),
                    values: vec!["Examine".into(), "Examinar".into()],
                }],
                ..Default::default()
            },
        )
        .unwrap();
        let index = crate::assets::scan(&root, &mut Default::default());
        crate::localization::resolve_runtime(&mut scene, &index, &registry).unwrap();
        assert_eq!(
            scene
                .strings
                .as_ref()
                .unwrap()
                .lookup("interaction.examine", 1),
            Some("Examinar")
        );
        let header = crate::project::scene_header_with_registry(
            &scene,
            &[],
            &scene,
            true,
            &scene,
            &registry,
        )
        .unwrap();
        assert!(header.contains("localization_keys=1,localization_languages=2"));
        assert!(header.contains("->target") && header.contains("component_id("));
        scene.hud_budget.glyphs = 1;
        assert!(
            crate::localization::resolve_runtime(&mut scene, &index, &registry)
                .unwrap_err()
                .contains("HUD")
        );
        let scripts = root.join("assets/scripts");
        std::fs::write(
            scripts.join("Demand.hpp"),
            "// TriggerBox3D\nconst char* label=\"InteractorComponent\";",
        )
        .unwrap();
        std::fs::write(
            scripts.join("demand.lua"),
            "-- InteractableComponent\nlocal text = [[ TriggerArea2D ]]\n",
        )
        .unwrap();
        assert!(source_demands(&root, &registry).unwrap().is_empty());
        std::fs::write(
            scripts.join("Demand.hpp"),
            "using Box = epok::TriggerBox3D;\n",
        )
        .unwrap();
        let mut registry = registry;
        registry.demanded_classes = source_demands(&root, &registry).unwrap();
        let linked = crate::blueprint_spawn::object_class_table_for_scenes(&registry, &[]).unwrap();
        assert!(linked.contains("EPOK_INTERACTION_USED"));
        assert!(linked.contains("ObjectPool<epok::TriggerBox3D"));
        assert!(linked.contains("ObjectPool<epok::InteractionSensor3DComponent"));
        assert!(!linked.contains("ObjectPool<epok::TriggerArea2D"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
