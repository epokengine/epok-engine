use super::super::{BlueprintEditor, CONTROLS, Registry, Socket, SocketType, asset, schema};
use super::*;

fn editor() -> BlueprintEditor {
    let mut editor = BlueprintEditor {
        asset: Some(asset::BlueprintAsset::new(
            "BP_Spell".into(),
            "parent".into(),
        )),
        ..Default::default()
    };
    editor.add_graph("cast_spell".into(), None);
    editor
}

#[test]
fn category_search_preserves_ancestors_and_manual_expansion() {
    let editor = editor();
    let actions = editor.catalog_actions(&Registry::new(), true);
    let mut menu = State::default();
    let rows = menu.rows(&actions);
    assert!(rows.iter().all(|r| r.action.is_none() && r.depth == 0));
    assert!(rows.windows(2).all(|pair| pair[0].label <= pair[1].label));
    menu.expanded.insert("Flow".into());
    assert!(menu.rows(&actions).iter().any(|r| r.label == "Branch"));
    menu.query = "spell CAST".into();
    let rows = menu.rows(&actions);
    assert_eq!(
        rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(),
        ["Functions", "BP_Spell", "Cast Spell"]
    );
    menu.query = "not-a-node".into();
    assert!(menu.rows(&actions).is_empty());
    menu.query.clear();
    assert!(menu.rows(&actions).iter().any(|r| r.label == "Branch"));
}

#[test]
fn context_toggle_filters_real_socket_types_and_keeps_literal_labels() {
    let mut editor = editor();
    let registry = Registry::new();
    editor.add_node(NodeKind::Literal {
        value_type: schema::Type::Bool,
        value: serde_json::json!(true),
    });
    let all = editor.catalog_actions(&registry, false);
    assert!(all.iter().any(|a| a.label.contains("(Q12)")));
    editor.wire = Some(Socket {
        node: editor.current().unwrap().nodes.last().unwrap().id.clone(),
        pin: "value".into(),
        output: true,
        ty: SocketType::Value(schema::Type::Bool),
        label: "Value".into(),
    });
    let filtered = editor.catalog_actions(&registry, true);
    assert!(filtered.iter().any(|a| matches!(a.kind, NodeKind::Branch)));
    assert!(
        !filtered
            .iter()
            .any(|a| matches!(a.kind, NodeKind::Literal { .. }))
    );
    assert_eq!(all.len(), editor.catalog_actions(&registry, false).len());
    assert!(filtered.len() < all.len());
    let not = all
        .iter()
        .find(|a| matches!(a.kind, NodeKind::Not))
        .unwrap();
    let delay = all
        .iter()
        .find(|a| matches!(a.kind, NodeKind::Delay))
        .unwrap();
    assert_ne!(not.icon, delay.icon);
    assert_ne!(not.color, delay.color);
}

#[test]
#[ignore = "Owns a real ImGui context; run explicitly and serially"]
fn action_menu_mouse_keyboard_bounds_and_gpu_capture() {
    let mut context = crate::gui::tests::imgui_context();
    context.set_ini_filename(None);
    context.io_mut().display_size = [672., 504.];
    context.io_mut().delta_time = 1. / 60.;
    let font = std::fs::read("C:/Windows/Fonts/segoeui.ttf").unwrap();
    context.fonts().add_font(&[
        imgui::FontSource::TtfData {
            data: &font,
            size_pixels: 15.,
            config: None,
        },
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/codicon.ttf"),
            size_pixels: 16.,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(&[0xea60, 0xedff, 0]),
                glyph_min_advance_x: 16.,
                ..Default::default()
            }),
        },
    ]);
    let menu_fonts = Fonts::load(&mut context);
    context.fonts().build_rgba32_texture();
    crate::gui::theme(context.style_mut());
    let mut editor = editor();
    let registry = Registry::new();
    let mut menu = State {
        fonts: Some(menu_fonts),
        ..Default::default()
    };
    let mut chosen = None;
    let mut open = true;
    let frame = |ctx: &mut imgui::Context,
                 menu: &mut State,
                 editor: &BlueprintEditor,
                 open: &mut bool,
                 chosen: &mut Option<NodeKind>| {
        CONTROLS.with(|c| c.borrow_mut().clear());
        let ui = ctx.frame();
        ui.window("Graph")
            .position([0., 0.], imgui::Condition::Always)
            .size(ui.io().display_size, imgui::Condition::Always)
            .build(|| {
                if *open {
                    menu.open(ui);
                    *open = false;
                }
                if let Some(kind) = menu.draw(
                    ui,
                    |sensitive| editor.catalog_actions(&registry, sensitive),
                    None,
                ) {
                    *chosen = Some(kind);
                }
            });
    };
    macro_rules! tick {
        () => {{
            frame(&mut context, &mut menu, &editor, &mut open, &mut chosen);
            context.render();
        }};
    }
    macro_rules! click {
        ($label:expr) => {{
            let point = CONTROLS.with(|c| {
                *c.borrow()
                    .get($label)
                    .unwrap_or_else(|| panic!("Missing {}", $label))
            });
            context.io_mut().add_mouse_pos_event(point);
            tick!();
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            tick!();
            context
                .io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            tick!();
        }};
    }
    tick!();
    tick!();
    tick!();
    assert!(CONTROLS.with(|c| c.borrow().contains_key("bp-action:Flow")));
    assert!(!CONTROLS.with(|c| c.borrow().contains_key("Flow / Branch")));
    click!("bp-action:Flow");
    tick!();
    assert!(
        chosen.is_none(),
        "Expanding categories must not close the popup or spawn nodes"
    );
    assert!(CONTROLS.with(|c| c.borrow().contains_key("Flow / Branch")));
    if let Some(directory) = std::env::var_os("EPOK_BP_ACTION_CAPTURE") {
        capture(
            &mut context,
            &mut menu,
            &editor,
            &mut open,
            &mut chosen,
            &frame,
            std::path::Path::new(&directory).join("blueprint-action-menu.png"),
        );
    }
    click!("Flow / Branch");
    tick!();
    assert!(matches!(chosen.take(), Some(NodeKind::Branch)));
    assert!(CONTROLS.with(|c| c.borrow().is_empty()));
    open = true;
    tick!();
    tick!();
    tick!();
    // Search takes focus on opening; no preliminary click in the field.
    for c in "get position".chars() {
        context.io_mut().add_input_character(c);
    }
    tick!();
    tick!();
    assert_eq!(menu.query, "get position");
    assert!(CONTROLS.with(|c| c.borrow().contains_key("Transform / Get position")));
    context.io_mut().add_key_event(Key::Enter, true);
    tick!();
    context.io_mut().add_key_event(Key::Enter, false);
    tick!();
    assert!(matches!(
        chosen.take(),
        Some(NodeKind::Builtin {
            operation: asset::Builtin::GetPosition
        })
    ));
    // Add many actions: only the tree scrolls and keyboard selection remains reachable.
    for index in 0..100 {
        editor.add_graph(format!("spell_{index:03}"), None);
    }
    open = true;
    tick!();
    tick!();
    tick!();
    for c in "spell".chars() {
        context.io_mut().add_input_character(c);
    }
    tick!();
    tick!();
    for _ in 0..60 {
        context.io_mut().add_key_event(Key::DownArrow, true);
        tick!();
        context.io_mut().add_key_event(Key::DownArrow, false);
        tick!();
    }
    if let Some(directory) = std::env::var_os("EPOK_BP_ACTION_CAPTURE") {
        capture(
            &mut context,
            &mut menu,
            &editor,
            &mut open,
            &mut chosen,
            &frame,
            std::path::Path::new(&directory).join("blueprint-action-search.png"),
        );
    }
    let selected = menu.selected.clone();
    assert!(selected.starts_with("action:"));
    CONTROLS.with(|c| {
        let controls = c.borrow();
        let position = controls[&format!("bp-action:{selected}")];
        assert!(
            (0. ..504.).contains(&position[1]),
            "Keyboard selection must scroll into view"
        );
        assert!(
            controls["bp-action-search"][1] < position[1],
            "Search stays above the scrolling tree"
        );
    });
    context.io_mut().add_key_event(Key::Escape, true);
    tick!();
    context.io_mut().add_key_event(Key::Escape, false);
    tick!();
    assert!(chosen.is_none());
    assert!(CONTROLS.with(|c| c.borrow().is_empty()));
}

/// Render the actual ImGui draw data through the production WGPU backend.
#[allow(clippy::too_many_arguments)]
fn capture(
    context: &mut imgui::Context,
    menu: &mut State,
    editor: &BlueprintEditor,
    open: &mut bool,
    chosen: &mut Option<NodeKind>,
    frame: &impl Fn(&mut imgui::Context, &mut State, &BlueprintEditor, &mut bool, &mut Option<NodeKind>),
    path: std::path::PathBuf,
) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut renderer = imgui_wgpu::Renderer::new(
        context,
        &device,
        &queue,
        imgui_wgpu::RendererConfig {
            texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            ..Default::default()
        },
    );
    let [width, height] = context.io().display_size.map(|x| x as u32);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Blueprint action menu visual test"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    frame(context, menu, editor, open, chosen);
    let data = context.render();
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        renderer.render(data, &queue, &device, &mut pass).unwrap();
    }
    let padded = (width * 4).div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(padded * height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        texture.size(),
    );
    queue.submit(Some(encoder.finish()));
    let (tx, rx) = std::sync::mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        tx.send(r).unwrap();
    });
    device.poll(wgpu::PollType::Wait).unwrap();
    rx.recv().unwrap().unwrap();
    let mapped = buffer.slice(..).get_mapped_range();
    let pixels: Vec<_> = mapped
        .chunks_exact(padded as usize)
        .flat_map(|row| row[..width as usize * 4].iter().copied())
        .collect();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, crate::mcp::png(width, height, &pixels).unwrap()).unwrap();
}
