use super::*;
use crate::{
    dialogue_asset::{BindingKind, NodeKind, Presentation, TextKey},
    workspace::GameplayFlavor,
};
use std::path::{Path, PathBuf};

pub(crate) struct Fixture {
    pub root: PathBuf,
    pub registry: Registry,
    pub index: Index,
    pub classes: Vec<(String, String, Presentation)>,
    pub conversation: Uuid,
    pub condition: Uuid,
    pub action: Uuid,
    pub role: Uuid,
    pub choice: Uuid,
    pub branch: Uuid,
    pub instant: Uuid,
}
pub(crate) fn fixture(all_styles: bool) -> Fixture {
    let root = crate::workspace::tests::temp("widget-conversation-preview");
    crate::workspace::create(
        &root,
        "Conversation Preview",
        crate::workspace::Template::Basic,
    )
    .unwrap();
    std::fs::write(
        root.join("assets/Preview.ttf"),
        include_bytes!("../resources/editor/Roboto-Regular.ttf"),
    )
    .unwrap();
    let font = crate::assets::commit(
        crate::font_asset::prepare(
            &root,
            "assets/Preview.ttf",
            "assets/PreviewFont.epokasset",
            Default::default(),
            None,
            false,
        )
        .unwrap(),
    )
    .unwrap();
    let mut pixels = vec![];
    {
        let mut encoder = png::Encoder::new(&mut pixels, 4, 4);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[255, 40, 40, 255].repeat(16))
            .unwrap();
    }
    std::fs::write(root.join("assets/Portrait.png"), pixels).unwrap();
    let portrait = crate::assets::commit(
        crate::texture::prepare(
            &root,
            "assets/Portrait.png",
            "assets/Portrait.epokasset",
            None,
            false,
        )
        .unwrap(),
    )
    .unwrap();
    let mut entries = vec![
        crate::localization::Entry {
            key: "greeting".into(),
            values: vec![
                "The traveler returns. ".repeat(20),
                "¡Hola, viajero! ".repeat(24),
            ],
        },
        crate::localization::Entry {
            key: "name".into(),
            values: vec!["Guide".into(), "Guía".into()],
        },
        crate::localization::Entry {
            key: "question".into(),
            values: vec!["Choose".into(), "Elige".into()],
        },
        crate::localization::Entry {
            key: "closed".into(),
            values: vec!["Closed".into(), "Cerrado".into()],
        },
    ];
    for i in 0..6 {
        entries.push(crate::localization::Entry {
            key: format!("option{i}"),
            values: vec![format!("Option {i}"), format!("Opción {i}")],
        });
    }
    let table = crate::localization::create(
        &root,
        "assets/Words.epokasset",
        &crate::localization::StringTable {
            pageable: true,
            languages: vec!["English".into(), "Español".into()],
            entries,
            ..Default::default()
        },
    )
    .unwrap();
    let key = |name: &str| TextKey {
        table,
        key: name.into(),
    };
    let role = Uuid::from_u128(100);
    let condition = Uuid::from_u128(101);
    let action = Uuid::from_u128(102);
    let line = Uuid::from_u128(1);
    let branch = Uuid::from_u128(2);
    let choice = Uuid::from_u128(3);
    let event = Uuid::from_u128(4);
    let closed = Uuid::from_u128(5);
    let end = Uuid::from_u128(6);
    let document = asset::DialogueAsset {
        version: asset::VERSION,
        entry: line,
        participants: vec![asset::Participant {
            id: role,
            label: "Guide".into(),
            display_name: Some(key("name")),
            portrait: Some(portrait),
        }],
        bindings: vec![
            asset::Binding {
                id: condition,
                label: "Available".into(),
                participant: role,
                kind: BindingKind::Condition,
                class: "310e882c-2353-55bc-bba3-f3d54277f671".into(),
                member: "fb9c1c09-be15-5b42-a069-4480e8e389f2".into(),
            },
            asset::Binding {
                id: action,
                label: "Accepted".into(),
                participant: role,
                kind: BindingKind::Action,
                class: "310e882c-2353-55bc-bba3-f3d54277f671".into(),
                member: "e66c2703-4f26-5bfc-b269-2f0c0eefbb8f".into(),
            },
        ],
        nodes: vec![
            asset::Node {
                id: line,
                kind: NodeKind::Line {
                    speaker: Some(role),
                    text: key("greeting"),
                    next: branch,
                    policy: None,
                },
            },
            asset::Node {
                id: branch,
                kind: NodeKind::Branch {
                    condition,
                    yes: choice,
                    no: closed,
                },
            },
            asset::Node {
                id: choice,
                kind: NodeKind::Choice {
                    speaker: Some(role),
                    prompt: Some(key("question")),
                    options: (0..6)
                        .map(|i| asset::ChoiceOption {
                            id: Uuid::from_u128(200 + i),
                            text: key(&format!("option{i}")),
                            next: event,
                            condition: (i == 5).then_some(condition),
                        })
                        .collect(),
                },
            },
            asset::Node {
                id: event,
                kind: NodeKind::Event { action, next: end },
            },
            asset::Node {
                id: closed,
                kind: NodeKind::Line {
                    speaker: Some(role),
                    text: key("closed"),
                    next: end,
                    policy: None,
                },
            },
            asset::Node {
                id: end,
                kind: NodeKind::End,
            },
        ],
    };
    let conversation = asset::create(&root, "assets/Conversation.epokasset", &document).unwrap();
    let instant = asset::create_style(
        &root,
        "assets/Instant.epokasset",
        &asset::DialogueStyle {
            line_policy: asset::LinePolicy {
                milliseconds_per_glyph: 0,
                auto_advance_ms: None,
            },
            ..Default::default()
        },
    )
    .unwrap();
    let mut classes = vec![];
    for flavor in GameplayFlavor::ALL {
        for presentation in if all_styles {
            Presentation::ALL.to_vec()
        } else {
            vec![Presentation::ClassicBox]
        } {
            let name = format!(
                "Preview{}{}",
                flavor.key(),
                presentation.label().replace(' ', "")
            );
            let id = crate::dialogue_widget::create(&root, &name, flavor, presentation, [320, 240])
                .unwrap();
            classes.push((name, id, presentation));
        }
    }
    let catalog = crate::scripts::catalog(&root).unwrap();
    let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
    for (_, class, _) in &classes {
        let members = registry
            .properties(&registry.classes[class].cpp_name)
            .into_iter()
            .filter(|p| matches!(p.name.as_str(), "body" | "prompt"))
            .map(|p| p.id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let edit = |template: &mut crate::blueprint_templates::Template| {
            let targets = template
                .references
                .iter()
                .filter(|r| members.contains(&r.member))
                .map(|r| r.target)
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(targets.len(), 2);
            for item in &mut template.actors {
                if item
                    .entity
                    .components
                    .iter()
                    .any(|c| targets.contains(&c.id))
                {
                    let text = item.entity.text.as_mut().unwrap();
                    text.font = Some(font);
                    text.color = [0.2, 0.8, 0.3];
                    crate::actor_components::sync(&mut item.entity);
                }
            }
        };
        if registry.classes[class].provider.id == "blueprint" {
            let path = &registry.classes[class].source.file;
            let before = std::fs::read(path).unwrap();
            let mut asset = crate::blueprint_asset::load(path).unwrap();
            edit(&mut asset.template);
            crate::assets::atomic_write(
                path,
                &crate::document::to_vec(&asset).unwrap(),
                Some(&crate::assets::hash(&before)),
            )
            .unwrap();
        } else {
            let index = crate::class_composition::load(&root).unwrap();
            let source = &index.sources[class];
            let mut document = source.document.clone();
            edit(&mut document.template);
            crate::class_composition::save(&root, source, &document, &registry).unwrap();
        }
    }
    crate::class_composition::observe_current(&root).unwrap();
    // A project override would replace the body if execution were accidentally
    // delegated to the project's derived class instead of the native preview.
    let header = root.join("assets/scripts/PreviewcppClassicBox.hpp");
    let source = std::fs::read_to_string(&header).unwrap();
    let custom=source.replace("public:","public:\n    void present(epok::DialogueSnapshot) override { if (auto* text = epok::active_object_registry->resolve<epok::TextComponent>(body)) text->get_owner()->data()->text.set_text(\"PROJECT OVERRIDE\"); }\n");
    assert_ne!(source, custom);
    std::fs::write(header, custom).unwrap();
    let catalog = crate::scripts::catalog(&root).unwrap();
    let registry = crate::blueprint::native_registry(&root, &catalog).unwrap();
    let index = crate::assets::scan(&root, &mut Default::default());
    Fixture {
        root,
        registry,
        index,
        classes,
        conversation,
        condition,
        action,
        role,
        choice,
        branch,
        instant,
    }
}
pub(crate) fn widget_scene(
    root: &Path,
    registry: &Registry,
    class: &str,
    display: [u16; 2],
) -> Scene {
    let mut scene = Scene {
        display_size: display,
        ..Default::default()
    };
    scene.actors.clear();
    crate::editor::place_class(
        &mut scene,
        root,
        registry,
        &registry.model().unwrap(),
        class,
    )
    .unwrap();
    scene
}
pub(crate) fn resolve(scene: &mut Scene, index: &Index) {
    crate::texture::resolve(scene, index).unwrap();
    crate::hud::resolve_fonts(scene, index).unwrap();
    crate::localization::resolve(scene, index);
}
fn copy(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == ".epok" {
            continue;
        }
        let kind = entry.file_type().unwrap();
        assert!(!kind.is_symlink());
        let target = destination.join(entry.file_name());
        if kind.is_dir() {
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn real_conversation_preview_uses_native_session_and_binder_in_every_widget_provider() {
    let f = fixture(true);
    let mut reports = vec![];
    let evidence = std::env::var_os("EPOK_WIDGET_CONVERSATION_EVIDENCE_DIR").map(PathBuf::from);
    if let Some(path) = &evidence {
        assert!(!path.exists());
        copy(&f.root, path);
        std::fs::write(path.parent().unwrap().join("fixture.json"),serde_json::to_vec_pretty(&serde_json::json!({"conversation":f.conversation,"condition":f.condition,"action":f.action,"participant":f.role,"choice":f.choice,"branch":f.branch,"instant":f.instant,"classes":f.classes.iter().map(|(name,id,presentation)|serde_json::json!({"name":name,"id":id,"presentation":format!("{presentation:?}")})).collect::<Vec<_>>()})).unwrap()).unwrap();
    }
    for (name, class, presentation) in &f.classes {
        for display in [[320, 240], [640, 480]] {
            for language in 0..2 {
                let mut scene = widget_scene(&f.root, &f.registry, class, display);
                scene.preview_language = language;
                resolve(&mut scene, &f.index);
                let authored = crate::document::to_vec(&scene).unwrap();
                let mut state = State {
                    asset: Some(f.conversation),
                    ..Default::default()
                };
                assert!(
                    render(&mut scene, &f.registry, &f.index, class, &mut state)
                        .err()
                        .unwrap()
                        .contains("Choose a preview value")
                );
                state.conditions.insert(f.condition, true);
                let initial = render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
                let instant = *presentation == Presentation::Subtitles;
                assert_eq!(
                    state.snapshot.unwrap().phase,
                    if instant { 2 } else { 1 },
                    "{name}/{display:?}/{language}"
                );
                if instant {
                    assert!(state.snapshot.unwrap().revealed > 0);
                    assert!(initial.stats[1] > 0);
                } else {
                    assert_eq!(state.snapshot.unwrap().revealed, 0);
                    assert_eq!(
                        initial.stats[1],
                        if language == 0 { 5 } else { 4 },
                        "The native binder, not the project present override, controls the initial state"
                    );
                }
                assert_eq!(initial.stats[4], 0);
                if *presentation == Presentation::PortraitConversation {
                    assert!(
                        initial.stats[3] >= 1,
                        "conversation-only portrait in the authored portrait area"
                    );
                }
                state.step(100_000);
                render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
                if !instant {
                    assert_eq!(state.snapshot.unwrap().revealed, 3);
                }
                if let Some(destination) = &evidence {
                    if !instant {
                        state.press(CONFIRM);
                    }
                    let line =
                        render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
                    let file = std::fs::File::create(
                        destination
                            .parent()
                            .unwrap()
                            .join(format!("{name}-{}-{language}-line.png", display[0])),
                    )
                    .unwrap();
                    let mut encoder =
                        png::Encoder::new(file, u32::from(display[0]), u32::from(display[1]));
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    encoder
                        .write_header()
                        .unwrap()
                        .write_image_data(&line.pixels)
                        .unwrap();
                    assert_eq!(line.stats[4], 0);
                }
                for _ in 0..80 {
                    if state.snapshot.unwrap().phase == 3 {
                        break;
                    }
                    state.press(CONFIRM);
                    render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
                }
                assert_eq!(state.snapshot.unwrap().phase, 3);
                assert_eq!(state.snapshot.unwrap().choices, 6);
                for _ in 0..4 {
                    state.press(DOWN);
                    render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
                }
                assert_eq!(state.snapshot.unwrap().selected, 4);
                assert_eq!(state.snapshot.unwrap().choice_page, 1);
                let output = render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
                assert_eq!(output.stats[4], 0);
                assert!(
                    output
                        .commands
                        .iter()
                        .any(|c| c.0[0] == 2 && c.0[2] >= 0 && c.0[11..14] == [51, 204, 77]),
                    "authored imported font and color survive binding"
                );
                if let Some(destination) = &evidence {
                    let path = destination
                        .parent()
                        .unwrap()
                        .join(format!("{name}-{}-{language}.png", display[0]));
                    let file = std::fs::File::create(&path).unwrap();
                    let mut encoder =
                        png::Encoder::new(file, u32::from(display[0]), u32::from(display[1]));
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    encoder
                        .write_header()
                        .unwrap()
                        .write_image_data(&output.pixels)
                        .unwrap();
                }
                reports.push(serde_json::json!({"class":name,"provider":f.registry.classes[class].provider.id,"presentation":format!("{presentation:?}"),"display":display,"language":language,"snapshot":state.snapshot,"stats":output.stats}));
                state.press(CONFIRM);
                let closed = render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
                assert_eq!(state.snapshot.unwrap().phase, 5);
                assert_eq!(closed.stats[1], 0);
                assert_eq!(
                    state
                        .trace
                        .iter()
                        .filter(|t| t.kind == 1 && t.id == graph::compact(f.action))
                        .count(),
                    1
                );
                assert_eq!(
                    crate::document::to_vec(&scene).unwrap(),
                    authored,
                    "preview must not author bound content or focus"
                );
            }
        }
    }
    if let Some(destination) = &evidence {
        std::fs::write(
            destination.parent().unwrap().join("preview-cases.json"),
            serde_json::to_vec_pretty(&reports).unwrap(),
        )
        .unwrap();
    }
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn preview_conditions_input_edges_locales_timing_and_context_changes_follow_runtime() {
    let f = fixture(false);
    let (_, class, _) = &f.classes[0];
    let mut scene = widget_scene(&f.root, &f.registry, class, [320, 240]);
    resolve(&mut scene, &f.index);
    let mut state = State {
        asset: Some(f.conversation),
        conditions: BTreeMap::from([(f.condition, true)]),
        ..Default::default()
    };
    render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
    assert!(state.snapshot.unwrap().pages > 1);
    state.press(CONFIRM);
    render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
    assert_eq!(state.snapshot.unwrap().phase, 2);
    state.frames.extend(
        [Frame {
            buttons: CONFIRM,
            microseconds: 16_667,
        }; 2],
    );
    render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
    assert_eq!(
        state.snapshot.unwrap().page,
        1,
        "held confirmation advances once"
    );
    state.press(CANCEL);
    let closed = render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
    assert_eq!(state.snapshot.unwrap().phase, 6);
    assert_eq!(closed.stats[1], 0);
    state.entry = Some(f.branch);
    state.conditions.insert(f.condition, false);
    state.restart();
    render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
    assert_eq!(state.snapshot.unwrap().glyphs, 6, "explicit false branch");
    state.press(CONFIRM);
    render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
    scene.preview_language = 1;
    render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
    assert!(state.frames.is_empty());
    assert_eq!(
        state.snapshot.unwrap().glyphs,
        7,
        "locale change restarts with translated content"
    );
    state.style = Some(f.instant);
    render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
    assert_eq!(state.snapshot.unwrap().phase, 2);
    state.entry = Some(f.choice);
    render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
    for _ in 0..3 {
        if state.snapshot.unwrap().phase == 3 {
            break;
        }
        state.press(CONFIRM);
        render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
    }
    assert_eq!(
        state.snapshot.unwrap().choices,
        5,
        "conditional option omitted"
    );
    state.participant = Some(f.role);
    state.entry = None;
    state.restart();
    render(&mut scene, &f.registry, &f.index, class, &mut state).unwrap();
    assert_eq!(state.entry, Some(Uuid::from_u128(1)));
    let frames = state.frames.clone();
    assert!(state.input("confirm,step:invalid").is_err());
    assert_eq!(state.frames, frames);
    let before = crate::scene_dependencies::hash(&state);
    state.restart();
    assert_ne!(
        before,
        crate::scene_dependencies::hash(&state),
        "restart invalidates an initial-state preview too"
    );
    let mut unavailable = asset::document(f.index.resolve(f.conversation).unwrap()).unwrap();
    for node in &mut unavailable.nodes {
        if let NodeKind::Choice { options, .. } = &mut node.kind {
            for option in options {
                option.condition = Some(f.condition);
            }
        }
    }
    let empty = asset::create(&f.root, "assets/Unavailable.epokasset", &unavailable).unwrap();
    let fresh = crate::assets::scan(&f.root, &mut Default::default());
    state = State {
        asset: Some(empty),
        entry: Some(f.choice),
        conditions: BTreeMap::from([(f.condition, false)]),
        ..Default::default()
    };
    let closed = render(&mut scene, &f.registry, &fresh, class, &mut state).unwrap();
    assert_eq!(state.snapshot.unwrap().phase, 7);
    assert_eq!(state.snapshot.unwrap().result_name(), "No choices");
    assert_eq!(closed.stats[1], 0);
    let mut hierarchy = widget_scene(&f.root, &f.registry, class, [320, 240]);
    resolve(&mut hierarchy, &f.index);
    let labels = dialogue_layout::bindings(&hierarchy, &f.registry, 0);
    let panel = labels["panel"];
    let body = labels["body"];
    let panel_id = hierarchy.actors[panel].id;
    hierarchy.actors[panel].active = false;
    hierarchy.actors[body].attach = Some(crate::actor_document::Attachment {
        actor: panel_id,
        component: None,
    });
    hierarchy.refresh_actor_hierarchy();
    state = State {
        asset: Some(f.conversation),
        style: Some(f.instant),
        conditions: BTreeMap::from([(f.condition, true)]),
        ..Default::default()
    };
    render(&mut hierarchy, &f.registry, &f.index, class, &mut state).unwrap();
    assert_eq!(
        state.snapshot.unwrap().phase,
        2,
        "inactive physical parent does not change logical liveness"
    );
    hierarchy.actors[body].logical_parent = Some(panel_id);
    assert!(
        render(&mut hierarchy, &f.registry, &f.index, class, &mut state)
            .err()
            .unwrap()
            .contains("cannot present"),
        "inactive logical parent invalidates the bound body"
    );
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn common_designer_conversation_controls_accept_real_input_without_authoring_content() {
    let f = fixture(false);
    let mut editor = crate::editor::Editor::new(f.root.clone());
    editor.class_registry = f.registry.clone();
    editor.catalog = crate::scripts::catalog(&f.root).unwrap();
    // This harness drives only the class window, so publish the fixture's scan
    // synchronously instead of waiting for the editor's background scan pump.
    editor.assets.index = f.index.clone();
    let mut ctx = crate::gui::tests::imgui_context();
    ctx.io_mut().display_size = [1550., 1100.];
    ctx.io_mut().delta_time = 1. / 60.;
    ctx.fonts().build_rgba32_texture();
    fn frame(ctx: &mut imgui::Context, e: &mut crate::editor::Editor) {
        crate::blueprint_editor::clear_controls();
        let ui = ctx.frame();
        crate::class_designer::window(ui, e);
        ctx.render();
    }
    fn click(ctx: &mut imgui::Context, e: &mut crate::editor::Editor, id: &str) {
        frame(ctx, e);
        let point = crate::blueprint_editor::control_position(id).unwrap_or_else(|| {
            panic!(
                "Missing control {id}; preview error: {:?}",
                e.class_designer.as_ref().unwrap().preview_error
            )
        });
        ctx.io_mut().add_mouse_pos_event(point);
        frame(ctx, e);
        ctx.io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, true);
        frame(ctx, e);
        ctx.io_mut()
            .add_mouse_button_event(imgui::MouseButton::Left, false);
        frame(ctx, e);
        frame(ctx, e);
    }
    for (_, class, _) in &f.classes {
        editor.scene.preview_language = 0;
        let mut window =
            crate::class_designer::Window::load(&f.root, &editor.class_registry, class).unwrap();
        window.sample = 5;
        window.preview_size = Some([320, 240]);
        editor.class_designer = Some(window);
        frame(&mut ctx, &mut editor);
        click(
            &mut ctx,
            &mut editor,
            "asset-field:Conversation##class-preview",
        );
        click(
            &mut ctx,
            &mut editor,
            &format!("reference-choice:{}", f.conversation),
        );
        let field = format!("conversation-preview:condition:{}", f.condition);
        if crate::blueprint_editor::control_position(&field).is_none() {
            click(&mut ctx, &mut editor, "conversation-preview:conditions");
        }
        click(&mut ctx, &mut editor, &field);
        click(
            &mut ctx,
            &mut editor,
            &format!("conversation-preview:condition-value:{}:2", f.condition),
        );
        let w = editor.class_designer.as_ref().unwrap();
        assert!(w.error.is_none(), "{:?}", w.error);
        assert!(w.preview_error.is_none(), "{:?}", w.preview_error);
        assert_eq!(w.conversation.snapshot.unwrap().phase, 1);
        let usage = &w.preview.as_ref().unwrap().usage;
        assert_eq!(
            (usage.actors, usage.components, usage.text_controls),
            (12, 29, 8)
        );
        assert_eq!(
            (usage.text_record_bytes, usage.text_buffer_bytes),
            (528, 512)
        );
        assert_eq!(
            (usage.textures, usage.fonts, usage.builtin_font),
            (2, 1, true),
            "Count the conversation-only portrait and imported font in the actual preview bank"
        );
        let authored = crate::document::to_vec(&w.preview.as_ref().unwrap().scene).unwrap();
        click(&mut ctx, &mut editor, "conversation-preview:step");
        assert_eq!(
            editor
                .class_designer
                .as_ref()
                .unwrap()
                .conversation
                .snapshot
                .unwrap()
                .revealed,
            3
        );
        click(&mut ctx, &mut editor, "conversation-preview:restart");
        assert_eq!(
            editor
                .class_designer
                .as_ref()
                .unwrap()
                .conversation
                .snapshot
                .unwrap()
                .revealed,
            0
        );
        click(&mut ctx, &mut editor, "conversation-preview:restart");
        assert!(
            editor
                .class_designer
                .as_ref()
                .unwrap()
                .conversation
                .snapshot
                .is_some(),
            "restarting initial state must invalidate the cached preview"
        );
        click(&mut ctx, &mut editor, "conversation-preview:language");
        click(&mut ctx, &mut editor, "conversation-preview:language:1");
        assert_eq!(
            editor
                .class_designer
                .as_ref()
                .unwrap()
                .preview
                .as_ref()
                .unwrap()
                .stats[1],
            4
        );
        click(&mut ctx, &mut editor, "conversation-preview:entry");
        click(
            &mut ctx,
            &mut editor,
            &format!("conversation-preview:entry:{}", f.choice),
        );
        for _ in 0..3 {
            if editor
                .class_designer
                .as_ref()
                .unwrap()
                .conversation
                .snapshot
                .unwrap()
                .phase
                == 3
            {
                break;
            }
            click(&mut ctx, &mut editor, "conversation-preview:confirm");
        }
        assert_eq!(
            editor
                .class_designer
                .as_ref()
                .unwrap()
                .conversation
                .snapshot
                .unwrap()
                .choices,
            6
        );
        for _ in 0..4 {
            click(&mut ctx, &mut editor, "conversation-preview:down");
        }
        assert_eq!(
            editor
                .class_designer
                .as_ref()
                .unwrap()
                .conversation
                .snapshot
                .unwrap()
                .choice_page,
            1
        );
        click(&mut ctx, &mut editor, "conversation-preview:confirm");
        let w = editor.class_designer.as_ref().unwrap();
        assert_eq!(w.conversation.snapshot.unwrap().phase, 5);
        assert_eq!(
            w.conversation.trace.iter().filter(|t| t.kind == 1).count(),
            1
        );
        assert!(!w.dirty());
        assert_eq!(
            crate::document::to_vec(&w.preview.as_ref().unwrap().scene).unwrap(),
            authored
        );
        editor
            .class_designer
            .as_mut()
            .unwrap()
            .save(&f.root, &editor.class_registry)
            .unwrap();
        let reopened =
            crate::class_designer::Window::load(&f.root, &editor.class_registry, class).unwrap();
        assert!(!reopened.dirty());
        assert!(
            reopened.conversation.asset.is_none(),
            "preview inputs are not class data"
        );
    }
    drop(ctx);
    std::fs::remove_dir_all(f.root).unwrap();
}
