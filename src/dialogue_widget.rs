//! Editable project widget starters. Each is one ordinary source class and its
//! composition, authored through the common class factories and UI designer.
use crate::{dialogue_asset::Presentation, workspace::GameplayFlavor};
use std::path::Path;

pub fn create(
    root: &Path,
    name: &str,
    flavor: GameplayFlavor,
    presentation: Presentation,
    display: [u16; 2],
) -> Result<String, String> {
    // Validate defaults before publishing a class. All live layout remains in
    // the resulting ordinary controls; no runtime preset dispatch is installed.
    crate::dialogue_starter::widget_preset(presentation, display)?;
    let source = match flavor {
        GameplayFlavor::Cpp => {
            crate::scripts::create_in(root, name, "", "epok::DialoguePresenter", false)?;
            root.join(format!("assets/scripts/{name}.hpp"))
        }
        GameplayFlavor::Blueprint => {
            let catalog = crate::scripts::catalog(root)?;
            let registry = crate::blueprint::native_registry(root, &catalog)?;
            crate::blueprint_workflow::create(
                root,
                &registry,
                name,
                "",
                crate::dialogue_layout::PRESENTER,
                false,
            )?
        }
        GameplayFlavor::Lua => {
            crate::lua_asset::create_in(root, name, "", "epok::DialoguePresenter")?
        }
    };
    let mut companion = None;
    let mut resources = crate::helper_install::Transaction::new(root);
    let result = (|| {
        let catalog = crate::scripts::catalog(root)?;
        let registry = crate::blueprint::native_registry(root, &catalog)?;
        let class = registry
            .named(name)
            .ok_or("Created widget class is unavailable.")?;
        let model = registry.model().map_err(|errors| {
            errors
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        let mut scene = crate::scene::Scene {
            display_size: display,
            ..Default::default()
        };
        scene.actors.clear();
        // Construct the root directly: normal placement would also create the
        // generic default composition before the chosen starter is authored.
        scene
            .actors
            .push(crate::class_composition::actor(&model, &class.id, name)?);
        crate::game_helpers::compose_dialogue_widget_in(
            &mut scene,
            root,
            &registry,
            &model,
            0,
            presentation,
            &mut resources,
        )?;
        let path = root.join(format!("assets/Classes/{}.epokclass", class.id));
        if flavor != GameplayFlavor::Blueprint && !path.exists() {
            companion = Some(path);
        }
        crate::class_designer::capture(root, &registry, &class.id, &scene, 0)?;
        Ok(class.id.clone())
    })();
    if result.is_err() {
        // These paths were exclusively created by the successful source factory.
        // Never remove an existing class when creation itself failed.
        let _ = std::fs::remove_file(&source);
        if flavor == GameplayFlavor::Cpp {
            let _ = std::fs::remove_file(source.with_extension("cpp"));
        }
        if let Some(path) = companion {
            let _ = std::fs::remove_file(path);
        }
        if flavor == GameplayFlavor::Lua {
            crate::lua_identity::removed(root, &[crate::assets::path_string(root, &source)])?;
        }
    } else {
        resources.commit();
    }
    result
}

pub fn create_in_editor(
    e: &mut crate::editor::Editor,
    flavor: GameplayFlavor,
    presentation: Presentation,
) -> Result<(), String> {
    if e.playing {
        return Err("Stop Play before creating a widget class.".into());
    }
    let stem = format!(
        "{}Dialogue{}",
        match flavor {
            GameplayFlavor::Cpp => "",
            GameplayFlavor::Blueprint => "BP_",
            GameplayFlavor::Lua => "Lua",
        },
        presentation.label().replace(' ', "")
    );
    let catalog = crate::scripts::catalog(&e.root)?;
    let registry = crate::blueprint::native_registry(&e.root, &catalog)?;
    let name = (0..10000)
        .map(|n| {
            if n == 0 {
                stem.clone()
            } else {
                format!("{stem}{n}")
            }
        })
        .find(|name| {
            !registry
                .classes
                .values()
                .any(|c| c.cpp_name.eq_ignore_ascii_case(name))
        })
        .ok_or("Choose another widget class name.")?;
    let id = create(&e.root, &name, flavor, presentation, e.scene.display_size)?;
    e.assets.refresh();
    e.refresh_scripts();
    crate::class_designer::open(e, &id);
    if let Some(window) = e.class_designer.as_mut().filter(|w| w.class == id) {
        window.sample = 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialogue_widget_failed_resource_install_rolls_back_each_provider_class() {
        for flavor in GameplayFlavor::ALL {
            let root = crate::workspace::tests::temp("widget-install-rollback");
            crate::workspace::create(&root, "Widgets", crate::workspace::Template::Basic).unwrap();
            let ui = root.join("assets/GameHelpers/UI");
            std::fs::create_dir_all(&ui).unwrap();
            let conflict = ui.join("portrait-frame.epokasset");
            std::fs::write(&conflict, b"existing user asset").unwrap();
            assert!(
                create(
                    &root,
                    "RejectedWidget",
                    flavor,
                    Presentation::PortraitConversation,
                    [320, 240]
                )
                .is_err()
            );
            assert_eq!(std::fs::read(&conflict).unwrap(), b"existing user asset");
            assert!(!ui.join("dialogue-frame.epokasset").exists());
            assert!(!ui.join("dialogue-frame.png").exists());
            for path in [
                "assets/scripts/RejectedWidget.hpp",
                "assets/scripts/RejectedWidget.cpp",
                "assets/scripts/RejectedWidget.lua",
                "assets/Blueprints/RejectedWidget.epokbp",
            ] {
                assert!(!root.join(path).exists(), "{flavor:?}: {path}");
            }
            assert!(
                !crate::scripts::catalog(&root)
                    .unwrap()
                    .iter()
                    .any(|class| class.name == "RejectedWidget")
            );
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    fn extend_with_project_properties(root: &Path, name: &str, flavor: GameplayFlavor) {
        match flavor {
            GameplayFlavor::Cpp => {
                let path = root.join(format!("assets/scripts/{name}.hpp"));
                let mut fields = String::from("public:\n");
                for index in 0..3 {
                    fields.push_str(&format!(" EPOK_PROPERTY(EditAnywhere, Id=\"{}\") uint32_t project_value{index}={index};\n", uuid::Uuid::new_v4()));
                }
                let source = std::fs::read_to_string(&path).unwrap();
                assert!(source.contains("public:"));
                std::fs::write(path, source.replacen("public:", &fields, 1)).unwrap();
            }
            GameplayFlavor::Blueprint => {
                let path = root.join(format!("assets/Blueprints/{name}.epokbp"));
                let mut asset = crate::blueprint_asset::load(&path).unwrap();
                for index in 0..3 {
                    asset.variables.push(crate::blueprint_asset::Variable {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: format!("project_value{index}"),
                        value_type: crate::reflection_schema::Type::UInt32,
                        default: serde_json::json!(index),
                        editable: true,
                        timeline_animatable: false,
                    });
                }
                std::fs::write(path, crate::document::to_vec(&asset).unwrap()).unwrap();
            }
            GameplayFlavor::Lua => {
                let path = root.join(format!("assets/scripts/{name}.lua"));
                let mut fields = String::new();
                for index in 0..3 {
                    fields.push_str(&format!(
                        "---@id {}\n{name}.project_value{index} = epok.UInt32({index})\n",
                        uuid::Uuid::new_v4()
                    ));
                }
                let source = std::fs::read_to_string(&path).unwrap();
                let marker = "\n-- Lifecycle events";
                assert!(source.contains(marker));
                std::fs::write(path, source.replacen(marker, &(fields + marker), 1)).unwrap();
            }
        }
    }
    #[test]
    fn dialogue_widget_starters_are_normal_classes_in_all_providers_and_resolutions() {
        let root = crate::workspace::tests::temp("dialogue-widget-starters");
        crate::workspace::create(&root, "Widget Starters", crate::workspace::Template::Basic)
            .unwrap();
        for flavor in GameplayFlavor::ALL {
            for presentation in Presentation::ALL {
                let name = format!(
                    "Widget{}{}",
                    flavor.key(),
                    presentation.label().replace(' ', "")
                );
                let id = create(&root, &name, flavor, presentation, [320, 240]).unwrap();
                extend_with_project_properties(&root, &name, flavor);
                let catalog = crate::scripts::catalog(&root).unwrap();
                let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
                let model = registry.model().unwrap();
                let class = &registry.classes[&id];
                assert_eq!(class.provider.id, flavor.key());
                assert_eq!(
                    registry
                        .properties(&class.cpp_name)
                        .iter()
                        .find(|p| p.name == "default_timing")
                        .unwrap()
                        .default,
                    serde_json::json!({"reveal_ms":30,"auto_advance_ms":0})
                );
                assert_eq!(
                    registry.properties(&class.cpp_name).len(),
                    16,
                    "The native base must leave three project property slots in every provider"
                );
                assert!(model.is_a(&id, crate::dialogue_layout::PRESENTER));
                let window = crate::class_designer::Window::load(&root, &registry, &id).unwrap();
                assert!(!window.dirty());
                for display in [[320, 240], [640, 480]] {
                    let mut scene = crate::scene::Scene {
                        display_size: display,
                        ..Default::default()
                    };
                    scene.actors.clear();
                    let index =
                        crate::editor::place_class(&mut scene, &root, &registry, &model, &id)
                            .unwrap();
                    assert_eq!(
                        scene.actors.len(),
                        if presentation == Presentation::SpeechBubble {
                            13
                        } else {
                            12
                        }
                    );
                    if presentation == Presentation::SpeechBubble {
                        assert!(
                            scene.actors[index]
                                .properties
                                .get("anchor")
                                .is_some_and(|v| !v.is_null())
                        );
                    }
                    let bindings = crate::dialogue_layout::bindings(&scene, &registry, index);
                    assert_eq!(bindings.len(), 11);
                    let profile = crate::dialogue_layout::Profile::read(&scene, &registry, index)
                        .unwrap()
                        .unwrap();
                    assert_eq!(profile.choices.len(), 4);
                    let boxes = crate::hud::layouts(&scene);
                    for control in bindings.values() {
                        if !scene.actors[*control].active {
                            continue;
                        }
                        let r = boxes[*control].unwrap().rect;
                        assert!(
                            r[0] >= 0.
                                && r[1] >= 0.
                                && r[0] + r[2] <= f32::from(display[0])
                                && r[1] + r[3] <= f32::from(display[1]),
                            "{name} {display:?} {r:?}"
                        );
                    }
                    assert_eq!(
                        scene.actors[bindings["portrait"]].active,
                        presentation == Presentation::PortraitConversation
                    );
                    for state in 1..=4 {
                        let mut rendered = scene.clone();
                        crate::dialogue_presenter::sample(&mut rendered, &registry, state);
                        let (_, stats) = crate::hud_native::compile(&rendered);
                        assert_eq!(stats[4], 0, "{name} / {display:?} / sample {state}");
                        assert_eq!(stats[1] > 0, state != 3);
                        if state == 2 {
                            let mut unfocused = rendered.clone();
                            let canvas = unfocused.actors[index].canvas.as_mut().unwrap();
                            canvas.focused = -1;
                            canvas.initial_focus = None;
                            let (_, plain) = crate::hud_native::compile(&unfocused);
                            assert_eq!(stats[0], plain[0] + 4, "{name} focus outline");
                            assert_eq!(stats[1], plain[1]);
                        }
                        for (original, preview) in scene.actors.iter().zip(&rendered.actors) {
                            assert_eq!(original.rect, preview.rect);
                            assert_eq!(
                                original.text.as_ref().map(|t| t.color),
                                preview.text.as_ref().map(|t| t.color)
                            );
                        }
                    }
                    // Ordinary instance edits survive class refresh.
                    let label = bindings["body"];
                    let before = scene.actors[label].clone();
                    scene.actors[label].text.as_mut().unwrap().color = [0.2, 0.6, 0.8];
                    scene.sync_actor_components();
                    crate::blueprint_templates::record_overrides(&before, &mut scene.actors[label]);
                    let files = crate::blueprint_asset::load_all(&root).unwrap();
                    crate::class_composition::refresh_instances(
                        &root, &mut scene, &files, &registry,
                    )
                    .unwrap();
                    assert_eq!(
                        scene.actors[label].text.as_ref().unwrap().color,
                        [0.2, 0.6, 0.8]
                    );
                }
                assert!(create(&root, &name, flavor, presentation, [320, 240]).is_err());
                assert!(
                    !crate::class_designer::Window::load(&root, &registry, &id)
                        .unwrap()
                        .dirty()
                );
            }
        }
        let catalog = crate::scripts::catalog(&root).unwrap();
        let index = crate::assets::scan(&root, &mut Default::default());
        let cooked = crate::blueprint_spawn::prepare(&root, &catalog, &index).unwrap();
        assert_eq!(cooked.len(), 12);
        assert_eq!(
            cooked.iter().filter(|t| t.scene.actors.len() == 12).count(),
            9
        );
        assert_eq!(
            cooked.iter().filter(|t| t.scene.actors.len() == 13).count(),
            3
        );
        // A custom derived widget is allowed to own an empty hierarchy. Normal
        // placement must never silently install starter controls into it.
        crate::scripts::create_in(&root, "CustomWidget", "", "epok::DialoguePresenter", false)
            .unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
        let id = registry.named("CustomWidget").unwrap().id.clone();
        let mut window = crate::class_designer::Window::load(&root, &registry, &id).unwrap();
        window.save(&root, &registry).unwrap();
        let mut scene = crate::scene::Scene::default();
        scene.actors.clear();
        let index = crate::editor::place_class(
            &mut scene,
            &root,
            &registry,
            &registry.model().unwrap(),
            &id,
        )
        .unwrap();
        assert_eq!(scene.actors.len(), 1);
        assert!(crate::dialogue_layout::bindings(&scene, &registry, index).is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}
