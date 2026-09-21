use crate::{editor::Editor, gui, scene_gpu};
struct Clipboard(arboard::Clipboard);
impl imgui::ClipboardBackend for Clipboard {
    fn get(&mut self) -> Option<String> {
        self.0.get_text().ok()
    }
    fn set(&mut self, value: &str) {
        let _ = self.0.set_text(value);
    }
}
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{CursorGrabMode, WindowAttributes},
};

pub fn run(
    request: Option<crate::loading::Request>,
    screenshot: Option<PathBuf>,
    profile: bool,
    startup_error: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut session: Option<Editor> = None;
    let mut loading = request.map(|request| {
        crate::loading::Loading::new(
            request,
            crate::workspace::user_data().join("RecentProjects.epokprefs"),
        )
    });
    let startup_started = Instant::now();
    let mut startup_previous = startup_started;
    let startup_profile = std::env::var_os("EPOK_PROFILE_STARTUP").is_some();
    let mut startup_mark = move |stage: &str| {
        if startup_profile {
            let now = Instant::now();
            eprintln!(
                "[startup] {stage}: {:.1} ms (total {:.1} ms)",
                now.duration_since(startup_previous).as_secs_f64() * 1000.,
                now.duration_since(startup_started).as_secs_f64() * 1000.
            );
            startup_previous = now;
        }
    };
    let mut hub = crate::hub::Hub::new(startup_error);
    startup_mark("hub and dependency discovery");
    let args = std::env::args().collect::<Vec<_>>();
    let content_capture = args.iter().any(|arg| arg == "--screenshot-content-browser");
    let minimum_size = if content_capture || args.iter().any(|arg| arg == "--sequencer-layout") {
        [640, 300]
    } else {
        [1024, 720]
    };
    let initial_size = args
        .windows(2)
        .find(|v| v[0] == "--window-size")
        .map(|v| {
            let (w, h) = v[1]
                .split_once('x')
                .ok_or("Use --window-size WIDTHxHEIGHT")?;
            let w = w.parse::<u32>().map_err(|_| "Invalid window width")?;
            let h = h.parse::<u32>().map_err(|_| "Invalid window height")?;
            if !(minimum_size[0]..=7680).contains(&w) || !(minimum_size[1]..=4320).contains(&h) {
                return Err("Window size must be between 1024x720 and 7680x4320");
            }
            Ok([w, h])
        })
        .transpose()?
        .unwrap_or([1440, 900]);
    let event_loop = EventLoop::new()?;
    let (brand_pixels, brand_width, brand_height) = crate::branding::pixels()?;
    let app_icon = winit::window::Icon::from_rgba(brand_pixels.clone(), brand_width, brand_height)?;
    let window_attributes = WindowAttributes::default()
        .with_title("Epok Engine | Projects")
        .with_visible(loading.is_none())
        .with_window_icon(Some(app_icon.clone()))
        .with_inner_size(LogicalSize::new(initial_size[0], initial_size[1]))
        .with_min_inner_size(LogicalSize::new(minimum_size[0], minimum_size[1]));
    #[cfg(windows)]
    let window_attributes = {
        use winit::platform::windows::WindowAttributesExtWindows;
        window_attributes.with_taskbar_icon(Some(app_icon))
    };
    #[allow(deprecated)]
    let window = Arc::new(event_loop.create_window(window_attributes)?);
    let mut splash_window = loading
        .as_ref()
        .map(|_| crate::loading::WindowPresentation::enter(&window));
    startup_mark("window and icon");
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: if cfg!(windows) {
            wgpu::Backends::DX12
        } else {
            wgpu::Backends::PRIMARY
        },
        ..Default::default()
    });
    let surface = instance.create_surface(window.clone())?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }))?;
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
    let size = window.inner_size();
    let mut config = surface
        .get_default_config(&adapter, size.width, size.height)
        .ok_or("Surface configuration unavailable")?;
    config.format = surface
        .get_capabilities(&adapter)
        .formats
        .into_iter()
        .find(|f| f.is_srgb())
        .ok_or("No sRGB surface format")?;
    config.usage |= wgpu::TextureUsages::COPY_SRC;
    config.desired_maximum_frame_latency = 1;
    surface.configure(&device, &config);
    startup_mark("graphics device and surface");
    let mut imgui = imgui::Context::create();
    gui::configure_input(imgui.io_mut());
    if let Ok(clipboard) = arboard::Clipboard::new() {
        imgui.set_clipboard_backend(Clipboard(clipboard));
    }
    let mut initial_layout = true;
    configure_project_layout(
        &mut imgui,
        session.as_ref(),
        screenshot.is_some(),
        &mut initial_layout,
    );
    if let Some(editor) = &session {
        window.set_title(&format!("Epok Engine | {}", editor.project_name()));
    }
    imgui.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;
    let mut platform = imgui_winit_support::WinitPlatform::new(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Rounded,
    );
    let font = std::fs::read("C:/Windows/Fonts/segoeui.ttf").ok();
    if let Some(font) = &font {
        imgui.fonts().add_font(&[
            imgui::FontSource::TtfData {
                data: font,
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
            imgui::FontSource::TtfData {
                data: include_bytes!("../resources/editor/fa-solid-900.ttf"),
                size_pixels: 14.,
                config: Some(imgui::FontConfig {
                    glyph_ranges: imgui::FontGlyphRanges::from_slice(EDITOR_FA_GLYPHS),
                    ..Default::default()
                }),
            },
        ]);
    } else {
        imgui.fonts().add_font(&[
            imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    size_pixels: 15.,
                    ..Default::default()
                }),
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
            imgui::FontSource::TtfData {
                data: include_bytes!("../resources/editor/fa-solid-900.ttf"),
                size_pixels: 14.,
                config: Some(imgui::FontConfig {
                    glyph_ranges: imgui::FontGlyphRanges::from_slice(EDITOR_FA_GLYPHS),
                    ..Default::default()
                }),
            },
        ]);
    }
    let sequencer_font = imgui.fonts().add_font(&[
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/Roboto-Regular.ttf"),
            size_pixels: 13.,
            config: None,
        },
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/codicon.ttf"),
            size_pixels: 17.,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(&[0xea60, 0xedff, 0]),
                glyph_min_advance_x: 17.,
                ..Default::default()
            }),
        },
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/fa-solid-900.ttf"),
            size_pixels: 16.,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(&[0xf000, 0xf8ff, 0]),
                glyph_min_advance_x: 16.,
                ..Default::default()
            }),
        },
    ]);
    // The property editor draws at a smaller, proportional size than the rest of
    // the editor so a narrow panel still fits a label and its value on one row.
    let inspector_font = imgui.fonts().add_font(&[
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/Roboto-Regular.ttf"),
            size_pixels: 13.,
            config: None,
        },
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/codicon.ttf"),
            size_pixels: 14.,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(&[0xea60, 0xedff, 0]),
                glyph_min_advance_x: 14.,
                ..Default::default()
            }),
        },
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/fa-solid-900.ttf"),
            size_pixels: 12.,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(&[0xf000, 0xf8ff, 0]),
                glyph_min_advance_x: 13.,
                ..Default::default()
            }),
        },
    ]);
    gui::theme(imgui.style_mut());
    let asset_font = imgui.fonts().add_font(&[
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/codicon.ttf"),
            size_pixels: 38.,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(&[0xea60, 0xedff, 0]),
                ..Default::default()
            }),
        },
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/fa-solid-900.ttf"),
            size_pixels: 38.,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(
                    crate::project_browser::ASSET_ICON_RANGES,
                ),
                ..Default::default()
            }),
        },
    ]);
    hub.load_fonts(&mut imgui);
    let browser_font = imgui.fonts().add_font(&[
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/Roboto-Regular.ttf"),
            size_pixels: 15.,
            config: None,
        },
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/fa-solid-900.ttf"),
            size_pixels: 13.,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(&[
                    0xf002, 0xf002, 0xf013, 0xf013, 0xf067, 0xf067, 0xf07b, 0xf07b, 0xf0b0, 0xf0b0,
                    0xf0c7, 0xf0c7, 0xf105, 0xf105, 0xf107, 0xf107, 0xf2d2, 0xf2d2, 0xf359, 0xf35a,
                    0xf56f, 0xf56f, 0,
                ]),
                ..Default::default()
            }),
        },
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/fa-solid-900.ttf"),
            size_pixels: 13.,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(
                    crate::project_browser::ASSET_ICON_RANGES,
                ),
                ..Default::default()
            }),
        },
        imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/codicon.ttf"),
            size_pixels: 13.,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(&[0xea60, 0xedff, 0]),
                ..Default::default()
            }),
        },
    ]);
    let action_menu_fonts = crate::blueprint_editor::ActionMenuFonts::load(&mut imgui);
    startup_mark("UI setup and font loading");
    let mut renderer = imgui_wgpu::Renderer::new(
        &mut imgui,
        &device,
        &queue,
        imgui_wgpu::RendererConfig {
            texture_format: config.format,
            ..Default::default()
        },
    );
    startup_mark("UI renderer and font atlas");
    let brand_texture = imgui_wgpu::Texture::new(
        &device,
        &renderer,
        imgui_wgpu::TextureConfig {
            size: wgpu::Extent3d {
                width: brand_width,
                height: brand_height,
                depth_or_array_layers: 1,
            },
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        },
    );
    brand_texture.write(&queue, &brand_pixels, brand_width, brand_height);
    let brand_texture_id = renderer.textures.insert(brand_texture);
    hub.logo = Some(brand_texture_id);
    let (lockup_pixels, lockup_width, lockup_height) = crate::branding::lockup_pixels()?;
    let lockup_texture = imgui_wgpu::Texture::new(
        &device,
        &renderer,
        imgui_wgpu::TextureConfig {
            size: wgpu::Extent3d {
                width: lockup_width,
                height: lockup_height,
                depth_or_array_layers: 1,
            },
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        },
    );
    lockup_texture.write(&queue, &lockup_pixels, lockup_width, lockup_height);
    let lockup_texture_id = renderer.textures.insert(lockup_texture);
    hub.lockup = Some(lockup_texture_id);
    let (splash_pixels, splash_width, splash_height) = crate::branding::splash_pixels()?;
    let splash_texture = imgui_wgpu::Texture::new(
        &device,
        &renderer,
        imgui_wgpu::TextureConfig {
            size: wgpu::Extent3d {
                width: splash_width,
                height: splash_height,
                depth_or_array_layers: 1,
            },
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            ..Default::default()
        },
    );
    splash_texture.write(&queue, &splash_pixels, splash_width, splash_height);
    let splash_texture_id = renderer.textures.insert(splash_texture);
    let texture = imgui_wgpu::Texture::new(
        &device,
        &renderer,
        imgui_wgpu::TextureConfig {
            size: wgpu::Extent3d {
                width: 960,
                height: 600,
                depth_or_array_layers: 1,
            },
            format: Some(scene_gpu::FORMAT),
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            ..Default::default()
        },
    );
    let scene_target = texture.texture().create_view(&Default::default());
    let texture_id = renderer.textures.insert(texture);
    startup_mark("branding textures and scene target");
    let mut scene_renderer = scene_gpu::SceneGpu::new(&device, &queue);
    startup_mark("scene renderer");
    let effect_texture = imgui_wgpu::Texture::new(
        &device,
        &renderer,
        imgui_wgpu::TextureConfig {
            size: scene_gpu::SIZE,
            format: Some(scene_gpu::FORMAT),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            ..Default::default()
        },
    );
    let effect_target = effect_texture.texture().create_view(&Default::default());
    let effect_texture_id = renderer.textures.insert(effect_texture);
    let mut effect_renderer = scene_gpu::SceneGpu::new(&device, &queue);
    let mut asset_renderer: Option<(scene_gpu::SceneGpu, imgui::TextureId, wgpu::TextureView)> =
        None;
    startup_mark("effect renderer");
    let mut hud_size = [320, 240];
    let hud_texture = imgui_wgpu::Texture::new(
        &device,
        &renderer,
        imgui_wgpu::TextureConfig {
            size: wgpu::Extent3d {
                width: 320,
                height: 240,
                depth_or_array_layers: 1,
            },
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            sampler_desc: wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let hud_texture_id = renderer.textures.insert(hud_texture);

    let native_game_texture = imgui_wgpu::Texture::new(
        &device,
        &renderer,
        imgui_wgpu::TextureConfig {
            size: scene_gpu::NATIVE_PLAY_SIZE,
            format: Some(scene_gpu::FORMAT),
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            sampler_desc: wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let mut native_game_target = native_game_texture
        .texture()
        .create_view(&Default::default());
    let native_game_texture_id = renderer.textures.insert(native_game_texture);
    let mut native_game_renderer =
        scene_gpu::SceneGpu::new_with_size(&device, &queue, scene_gpu::NATIVE_PLAY_SIZE);
    let mut native_game_size = [
        scene_gpu::NATIVE_PLAY_SIZE.width,
        scene_gpu::NATIVE_PLAY_SIZE.height,
    ];
    let native_hud_texture = imgui_wgpu::Texture::new(
        &device,
        &renderer,
        imgui_wgpu::TextureConfig {
            size: wgpu::Extent3d {
                width: 320,
                height: 240,
                depth_or_array_layers: 1,
            },
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            sampler_desc: wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let native_hud_texture_id = renderer.textures.insert(native_hud_texture);
    let mut native_hud_size = [320, 240];
    let mut native_sequence = 0;

    let mut game_texture = None;
    let mut game_sequence = 0;
    let capture_game = args
        .iter()
        .any(|arg| arg == "--screenshot-game" || arg == "--screenshot-native-game");
    let capture_loading = args.iter().any(|arg| arg == "--screenshot-loading");
    let mut project_frames = 0;
    let started = Instant::now();
    let mut project_started = started;
    let frame_period = Duration::from_micros(8333);
    let mut next_frame = Instant::now();
    let mut captured = false;
    let mut frame_intervals = Vec::new();
    let mut frame_costs = Vec::new();
    let mut previous_frame = Instant::now();
    let mut frame_number = 0;
    let mut look_captured = false;
    let mut raw_motion = [0_f32; 2];
    let mut startup_first_frame = true;
    startup_mark("remaining setup");
    let mut look_restore = winit::dpi::PhysicalPosition::new(0_f64, 0_f64);
    // A hidden Win32 window may not receive paint events. Show the compact
    // window after GPU setup so the event loop can present its first splash.
    window.set_visible(true);
    #[allow(deprecated)]
    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::WaitUntil(next_frame));
        if !look_captured || !matches!(&event,Event::WindowEvent{event:WindowEvent::CursorMoved{..}|WindowEvent::CursorLeft{..},..}) {
            platform.handle_event(imgui.io_mut(), &window, &event);
        }
        match event {
            Event::WindowEvent { event: WindowEvent::DroppedFile(path), .. } => {
                if let Some(editor) = session.as_mut().filter(|e| !e.critical_busy()) {
                    crate::project_browser::external_drop(editor, path);
                }
            }
            Event::WindowEvent{event:WindowEvent::CursorMoved{position,..},..} if !look_captured => {look_restore=position;}
            Event::DeviceEvent {event:DeviceEvent::MouseMotion{delta},..} if look_captured => {
                raw_motion[0]+=delta.0 as f32;raw_motion[1]+=delta.1 as f32;
            }
            Event::AboutToWait => {
                if Instant::now()>=next_frame {
                    window.request_redraw();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if session.as_ref().is_some_and(|e| e.critical_busy()) || (session.is_none() && hub.dependencies.busy()) {
                    window.request_redraw();
                    return;
                }
                if let Some(editor) = session.as_mut().filter(|e| e.has_unsaved_changes()) {
                    editor.close_requested = true;
                } else {
                    target.exit();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if size.width > 0 && size.height > 0 {
                    config.width = size.width;
                    config.height = size.height;
                    surface.configure(&device, &config);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let frame_start=Instant::now();
                let mut frame_stage = frame_start;
                let mut frame_mark = |stage: &str| {
                    if startup_profile {
                        let now = Instant::now();
                        let ms = now.duration_since(frame_stage).as_secs_f64()*1000.;
                        if ms > 50. { eprintln!("[frame] {stage}: {ms:.1} ms"); }
                        frame_stage = now;
                    }
                };
                next_frame=frame_start+frame_period;
                imgui.io_mut().update_delta_time(frame_start.duration_since(previous_frame));
                let interval=frame_start.duration_since(previous_frame).as_secs_f64()*1000.;
                previous_frame=frame_start;
                if let Some(editor) = session.as_mut() {
                editor.blueprint_editor.set_action_menu_fonts(action_menu_fonts);
                if profile {editor.view.yaw+=0.008;editor.view_dirty=true;}
                editor.tick();
                crate::mcp::tick(editor);
                frame_mark("editor and MCP tick");
                if let Some(game) = &editor.game_frame {
                    if game_sequence != game.sequence {
                        let size = [game.width, game.height];
                        if game_texture
                            .as_ref()
                            .is_none_or(|(_, previous)| *previous != size)
                        {
                            if let Some((id, _)) = game_texture.take() {
                                renderer.textures.remove(id);
                            }
                            let texture = imgui_wgpu::Texture::new(
                                &device,
                                &renderer,
                                imgui_wgpu::TextureConfig {
                                    size: wgpu::Extent3d {
                                        width: size[0],
                                        height: size[1],
                                        depth_or_array_layers: 1,
                                    },
                                    format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                                    sampler_desc: wgpu::SamplerDescriptor {
                                        mag_filter: wgpu::FilterMode::Nearest,
                                        min_filter: wgpu::FilterMode::Nearest,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                            );
                            game_texture = Some((renderer.textures.insert(texture), size));
                        }
                        renderer
                            .textures
                            .get(game_texture.unwrap().0)
                            .unwrap()
                            .write(&queue, &game.rgba, game.width, game.height);
                        game_sequence = game.sequence;
                    }
                } else {
                    game_sequence = 0;
                }
                if let Some(native) = &editor.native_frame {
                    if native_game_size != native.hud_size {
                        let size = wgpu::Extent3d {
                            width: native.hud_size[0],
                            height: native.hud_size[1],
                            depth_or_array_layers: 1,
                        };
                        let texture = imgui_wgpu::Texture::new(
                            &device,
                            &renderer,
                            imgui_wgpu::TextureConfig {
                                size,
                                format: Some(scene_gpu::FORMAT),
                                usage: wgpu::TextureUsages::TEXTURE_BINDING
                                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                                    | wgpu::TextureUsages::COPY_SRC,
                                sampler_desc: wgpu::SamplerDescriptor {
                                    mag_filter: wgpu::FilterMode::Nearest,
                                    min_filter: wgpu::FilterMode::Nearest,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                        );
                        native_game_target = texture.texture().create_view(&Default::default());
                        renderer.textures.replace(native_game_texture_id, texture);
                        native_game_renderer = scene_gpu::SceneGpu::new_with_size(
                            &device,
                            &queue,
                            size,
                        );
                        native_game_size = native.hud_size;
                    }
                    if native_sequence != native.number || native_hud_size != native.hud_size {
                        if native_hud_size != native.hud_size {
                            let texture = imgui_wgpu::Texture::new(
                                &device,
                                &renderer,
                                imgui_wgpu::TextureConfig {
                                    size: wgpu::Extent3d {
                                        width: native.hud_size[0],
                                        height: native.hud_size[1],
                                        depth_or_array_layers: 1,
                                    },
                                    format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                                    sampler_desc: wgpu::SamplerDescriptor {
                                        mag_filter: wgpu::FilterMode::Nearest,
                                        min_filter: wgpu::FilterMode::Nearest,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                            );
                            renderer.textures.replace(native_hud_texture_id, texture);
                            native_hud_size = native.hud_size;
                        }
                        renderer
                            .textures
                            .get(native_hud_texture_id)
                            .unwrap()
                            .write(
                                &queue,
                                &native.hud_rgba,
                                native.hud_size[0],
                                native.hud_size[1],
                            );
                        native_sequence = native.number;
                    }
                } else {
                    native_sequence = 0;
                }
                }
                let frame = match surface.get_current_texture() {
                    Ok(f) => f,
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        surface.configure(&device, &config);
                        return;
                    }
                    Err(wgpu::SurfaceError::Timeout) => return,
                    Err(e) => {
                        eprintln!("GPU surface: {e}");
                        target.exit();
                        return;
                    }
                };
                let view = frame.texture.create_view(&Default::default());
                if let Err(e)=platform.prepare_frame(imgui.io_mut(),&window){eprintln!("{e}");}
                if let Some(editor) = session.as_mut() {
                    editor.project_browser.previews.upload(&device, &queue, &mut renderer);
                    editor.asset_inspector.media.upload(&device, &queue, &mut renderer);
                    editor.asset_inspector.visible = false;
                    editor.asset_inspector.texture = asset_renderer.as_ref().map(|(_,id,_)|*id);
                }
                let ui = imgui.frame();
                let mut opened = None;
                let native_visible = session
                    .as_ref()
                    .is_some_and(|editor| editor.native_frame.is_some());
                let visible_game = if native_visible {
                    Some(native_game_texture_id)
                } else {
                    game_texture.map(|(id, _)| id)
                };
                let visible_overlay = native_visible.then_some(native_hud_texture_id);
                if let Some(editor) = session.as_mut() {
                editor.raw_look=look_captured.then_some(std::mem::take(&mut raw_motion));
                editor.project_browser.font = Some(browser_font);
                editor.timeline_editor.font = Some(sequencer_font);
                editor.inspector_font = Some(inspector_font);
                gui::draw(
                    ui,
                    editor,
                    [texture_id,hud_texture_id,brand_texture_id],
                    visible_game,
                    visible_overlay,
                    asset_font,
                    [960., 600.],
                    &mut initial_layout,
                );
                if !editor.critical_busy() { crate::hub::close_project_dialog(ui, editor); }
                } else if let Some(loading) = &loading {
                    loading.draw(ui, lockup_texture_id, splash_texture_id);
                } else { opened = hub.draw(ui); }
                platform.prepare_render(ui, &window);
                frame_mark("UI draw");
                let capture_look=session.as_ref().is_some_and(|e|e.scene_look && !e.scene_2d() && !e.hub_requested && !e.close_requested && !e.should_close && !e.return_to_hub);
                update_look_cursor(look_captured, capture_look, |visible| window.set_cursor_visible(visible));
                if capture_look != look_captured {
                    if capture_look {
                        let _=window.set_cursor_grab(CursorGrabMode::Locked).or_else(|_|window.set_cursor_grab(CursorGrabMode::Confined));
                    } else {
                        let _=window.set_cursor_grab(CursorGrabMode::None);
                        let _=window.set_cursor_position(look_restore);
                    }
                    look_captured=capture_look;raw_motion=[0.;2];
                }
                let mut encoder = device.create_command_encoder(&Default::default());
                if let Some(frame) = session.as_ref().and_then(|editor| editor.native_frame.as_ref()) {
                    native_game_renderer.render_game(
                        &device,
                        &queue,
                        &mut encoder,
                        &native_game_target,
                        frame,
                    );
                }
                if let Some(editor) = session.as_mut().filter(|e| e.asset_inspector.visible && e.asset_inspector.details.as_ref().is_some_and(|d|d.scene.is_some())) {
                    if asset_renderer.is_none() {
                        let texture = imgui_wgpu::Texture::new(&device,&renderer,imgui_wgpu::TextureConfig {
                            size: scene_gpu::SIZE, format: Some(scene_gpu::FORMAT),
                            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, ..Default::default()
                        });
                        let target = texture.texture().create_view(&Default::default());
                        let id = renderer.textures.insert(texture);
                        asset_renderer = Some((scene_gpu::SceneGpu::new(&device,&queue),id,target));
                        editor.asset_inspector.model_dirty = true;
                    }
                    let (gpu,_,target) = asset_renderer.as_mut().unwrap();
                    if editor.asset_inspector.model_dirty || editor.asset_inspector.details.as_ref().and_then(|d|d.scene.as_ref()).is_some_and(scene_gpu::SceneGpu::animated) {
                        gpu.render_asset(&device,&queue,&mut encoder,target,&editor.asset_inspector);
                        editor.asset_inspector.model_dirty = false;
                    }
                }
                if let Some(editor) = session.as_mut() {
                    editor.timeline_editor.effect_preview.texture = Some(effect_texture_id);
                    if editor.timeline_editor.open {
                        effect_renderer.render_effect(&device, &queue, &mut encoder, &effect_target, &editor.timeline_editor.effect_preview);
                    }
                }
                // Apply UI input first: camera and transforms reach the texture in this frame.
                if let Some(editor) = session.as_mut().filter(|e| e.view_dirty || e.timeline_editor.open || scene_gpu::SceneGpu::animated(&e.scene)) {
                    if editor.scene_2d() {
                        let preview_scene=editor.hud_simulation.session.as_ref().map_or(&editor.scene,|s|&s.scene);
                        let size=preview_scene.display_size.map(u32::from);
                        if size != hud_size {
                            let texture=imgui_wgpu::Texture::new(&device,&renderer,imgui_wgpu::TextureConfig {
                                size:wgpu::Extent3d{width:size[0],height:size[1],depth_or_array_layers:1},
                                format:Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                                sampler_desc:wgpu::SamplerDescriptor{mag_filter:wgpu::FilterMode::Nearest,min_filter:wgpu::FilterMode::Nearest,..Default::default()},
                                ..Default::default()
                            });
                            renderer.textures.replace(hud_texture_id,texture);hud_size=size;
                        }
                        let phase=scene_renderer.preview_time(&editor.scene,editor.view.phase);
                        let pixels=editor.hud_simulation.pixels.clone().unwrap_or_else(||crate::hud::render_at(preview_scene,phase));
                        renderer.textures.get(hud_texture_id).unwrap().write(&queue,&pixels,size[0],size[1]);
                    } else {
                        scene_renderer.render(&device, &queue, &mut encoder, &scene_target, editor);
                        editor.navigation_preview_status=scene_renderer.navigation_status();
                    }
                    editor.view_dirty = false;
                }
                frame_mark("scene render");
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Epok UI"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.12,
                                    g: 0.13,
                                    b: 0.15,
                                    a: 1.,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    imgui.render();
                    normalize_imgui_indices();
                    use imgui::internal::RawCast;
                    // A frame that produced no draw lists leaves CmdLists null, and
                    // imgui-rs builds a slice from it unconditionally, which trips
                    // the std pointer precondition and aborts. Skipping the empty
                    // frame costs nothing: the pass has already cleared the target.
                    let raw = unsafe { imgui::sys::igGetDrawData() };
                    if !raw.is_null() && unsafe { (*raw).CmdListsCount } > 0 {
                        // No ImGui frame mutation occurs while the renderer borrows this data.
                        let draw_data = unsafe { imgui::DrawData::from_raw(&*raw) };
                        if let Err(e) = renderer.render(draw_data, &queue, &device, &mut pass) {
                            eprintln!("UI renderer: {e}");
                            target.exit();
                        }
                    }
                }
                queue.submit(Some(encoder.finish()));
                if let Some(editor) = session.as_mut() {
                    for request in std::mem::take(&mut editor.mcp.screenshots) {
                        if !request.active() { continue; }
                        let result = match request.args["target"].as_str() {
                            Some("editor") => capture_png(&device, &queue, &frame.texture),
                            Some("scene") if !editor.scene_2d() => capture_png(&device, &queue, renderer.textures.get(texture_id).unwrap().texture()),
                            Some("scene") | Some("hud") => {
                                let preview_scene=editor.hud_simulation.session.as_ref().map_or(&editor.scene,|s|&s.scene);
                                let size=preview_scene.display_size.map(u32::from);
                                let pixels=editor.hud_simulation.pixels.clone().unwrap_or_else(||crate::hud::render_at(preview_scene,scene_renderer.preview_time(preview_scene,editor.view.phase)));
                                crate::mcp::png(size[0],size[1],&pixels)
                            },
                            Some("game") if editor.native_frame.is_some() => capture_png(&device, &queue, renderer.textures.get(native_game_texture_id).unwrap().texture()),
                            Some("game") => editor.game_frame.as_ref().ok_or("No game frame available. Start Play first.".to_string()).and_then(|f| crate::mcp::png(f.width, f.height, &f.rgba)),
                            _ => Err("Choose editor, scene, hud or game".into()),
                        };
                        crate::mcp::screenshot_reply(request, result);
                    }
                }
                if session.is_some() { project_frames += 1; }
                if !captured
                    && ((capture_loading && loading.is_some()) ||
                        (!capture_loading && loading.is_none() && (session.is_none() || project_frames > 10)
                            && if session.is_some() { project_started.elapsed() > Duration::from_secs(2) } else { started.elapsed() > Duration::from_secs(2) }))
                    && (!capture_game
                        || session.as_ref().and_then(|e| e.game_frame.as_ref()).is_some_and(|f| f.sequence > 90)
                        || session.as_ref().and_then(|e| e.native_frame.as_ref()).is_some_and(|f| f.number > 90)
                        || project_started.elapsed() > Duration::from_secs(25))
                    && (!std::env::args().any(|a|a=="--screenshot-native-hud")
                        || session.as_ref().and_then(|e|e.hud_simulation.session.as_ref()).and_then(|s|s.frame.as_ref()).is_some()
                        || session.as_ref().is_some_and(|e|e.hud_simulation.error.is_some())
                        || project_started.elapsed()>Duration::from_secs(30))
                    && let Some(path) = &screenshot
                {
                    match capture(&device, &queue, &frame.texture, path) {
                        Ok(()) => println!("Screenshot: {}", path.display()),
                        Err(e) => eprintln!("Screenshot failed: {e}"),
                    };
                    let _ = std::fs::write(path.with_extension("log"), session.as_ref().map_or_else(String::new, |e| e.logs.join("\n")));
                    captured = true;
                    target.exit();
                }
                frame.present();
                if startup_first_frame {
                    startup_mark("first frame presented");
                    startup_first_frame = false;
                }
                if profile && let Some(editor) = &session {
                    if frame_number>=30{frame_intervals.push(interval);frame_costs.push(frame_start.elapsed().as_secs_f64()*1000.);}
                    frame_number+=1;
                    if frame_number==210 {
                        frame_intervals.sort_by(f64::total_cmp);frame_costs.sort_by(f64::total_cmp);
                        let summary=format!("Editor orbit (180 frames): frame interval median {:.2} ms / p95 {:.2} ms; update+draw+present median {:.2} ms / p95 {:.2} ms",frame_intervals[90],frame_intervals[171],frame_costs[90],frame_costs[171]);
                        println!("{summary}");let _=std::fs::write(editor.root.join("artifacts/scene-performance.txt"),summary);
                        target.exit();
                    }
                }
                if session.as_ref().is_some_and(|e| e.should_close) { target.exit(); }
                if session.as_ref().is_some_and(|e| e.return_to_hub) {
                    save_layout(&mut imgui, session.as_ref(), screenshot.is_some());
                    if let Some(editor) = session.as_mut() { editor.project_browser.previews.clear(&mut renderer); editor.asset_inspector.media.clear(&mut renderer); }
                    if let Some((_,id,_)) = asset_renderer.take() { renderer.textures.remove(id); }
                    session = None; // Drop joins the build worker and stops the owned emulator before unlocking.
                    imgui.set_ini_filename(None);
                    hub.refresh();
                    window.set_title("Epok Engine | Projects");
                    if let Some((id, _)) = game_texture.take() { renderer.textures.remove(id); }
                    game_sequence = 0;
                }
                if let Some(active) = loading.as_mut() {
                    active.start();
                    if let Some(result) = active.poll() {
                        let result = result.and_then(|ready| {
                            let mut editor = Editor::open_prepared(ready.project)?;
                            if let Some(warning) = ready.warning { editor.log(warning); }
                            crate::prepare_editor(&mut editor);
                            Ok(editor)
                        });
                        loading = None;
                        if let Some(presentation) = splash_window.take() {
                            presentation.restore(&window, minimum_size);
                        }
                        match result {
                            Ok(editor) => {
                                window.set_title(&format!("Epok Engine | {}", editor.project_name()));
                                session = Some(editor);
                                project_frames = 0;
                                project_started = Instant::now();
                                configure_project_layout(&mut imgui, session.as_ref(), screenshot.is_some(), &mut initial_layout);
                            }
                            Err(error) => { hub.refresh(); hub.error = Some(error); window.set_title("Epok Engine | Projects"); }
                        }
                    }
                }
                if let Some(request) = opened {
                    splash_window = Some(crate::loading::WindowPresentation::enter(&window));
                    loading = Some(crate::loading::Loading::new(request,
                        crate::workspace::user_data().join("RecentProjects.epokprefs")));
                    window.set_title("Epok Engine | Loading project");
                }
            }
            Event::LoopExiting => {
                save_layout(&mut imgui, session.as_ref(), screenshot.is_some());
                if let Some(editor) = session.as_mut() { editor.project_browser.previews.clear(&mut renderer); editor.asset_inspector.media.clear(&mut renderer); }
                session = None;
            }
            Event::WindowEvent {
                event: WindowEvent::Focused(false),
                ..
            } => {
                if let Some(editor) = session.as_mut() { editor.game_capture = false; editor.set_buttons(0);editor.scene_navigation=false;editor.scene_look=false;editor.raw_look=None; }
                look_captured=false;raw_motion=[0.;2];let _=window.set_cursor_grab(CursorGrabMode::None);window.set_cursor_visible(true);
            }
            _ => {}
        }
    })?;
    Ok(())
}
fn update_look_cursor(previous: bool, captured: bool, set_visible: impl FnOnce(bool)) {
    // ImGui caches its cursor choice. Hiding the OS cursor outside the backend
    // must be explicitly undone when RMB navigation ends, even over another tab.
    if previous || captured {
        set_visible(!captured);
    }
}

#[test]
fn camera_capture_restores_cursor_without_a_backend_shape_change() {
    let mut visible = true;
    for (previous, captured, expected) in [
        (false, true, false),
        (true, true, false),
        (true, false, true),
        (false, false, true),
    ] {
        update_look_cursor(previous, captured, |value| visible = value);
        assert_eq!(visible, expected);
    }
}

/// imgui-wgpu 0.25 consumes indices sequentially instead of using Cmd::IdxOffset.
/// ImGui moves modal dimming commands to the front without moving their indices.
/// Normalize only those lists, after Render and before the backend uploads them.
fn normalize_imgui_indices() {
    // SAFETY: called on the ImGui thread after Render, with no outstanding draw-list
    // references. Buffers keep their allocation and length; only index order changes.
    unsafe {
        let data = imgui::sys::igGetDrawData();
        if data.is_null() {
            return;
        }
        for n in 0..(*data).CmdListsCount as usize {
            let list = &mut **(*data).CmdLists.add(n);
            let mut expected = 0;
            let mut ordered = true;
            for n in 0..list.CmdBuffer.Size as usize {
                let cmd = &*list.CmdBuffer.Data.add(n);
                if cmd.UserCallback.is_some() {
                    continue;
                }
                ordered &= cmd.IdxOffset as usize == expected;
                expected += cmd.ElemCount as usize;
            }
            if ordered {
                continue;
            }
            assert_eq!(expected, list.IdxBuffer.Size as usize);
            let mut indices = Vec::with_capacity(expected);
            for n in 0..list.CmdBuffer.Size as usize {
                let cmd = &*list.CmdBuffer.Data.add(n);
                if cmd.UserCallback.is_some() {
                    continue;
                }
                let start = cmd.IdxOffset as usize;
                let count = cmd.ElemCount as usize;
                assert!(start + count <= expected);
                assert_eq!(cmd.VtxOffset, 0, "Backend does not support vertex offsets");
                indices.extend_from_slice(std::slice::from_raw_parts(
                    list.IdxBuffer.Data.add(start),
                    count,
                ));
            }
            std::ptr::copy_nonoverlapping(indices.as_ptr(), list.IdxBuffer.Data, indices.len());
            let mut offset = 0;
            for n in 0..list.CmdBuffer.Size as usize {
                let cmd = &mut *list.CmdBuffer.Data.add(n);
                if cmd.UserCallback.is_some() {
                    continue;
                }
                cmd.IdxOffset = offset;
                offset += cmd.ElemCount;
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn verify_modal_indices(context: &mut imgui::Context) {
    use imgui::internal::RawCast;
    for frame in 0..4 {
        let ui = context.frame();
        ui.window("Modal backdrop regression")
            .size([600., 400.], imgui::Condition::Always)
            .build(|| {
                ui.text("The backdrop must dim behind the modal.");
                ui.button("Underlying control");
            });
        if frame == 0 {
            ui.open_popup("Modal regression");
        }
        ui.modal_popup_config("Modal regression")
            .always_auto_resize(true)
            .build(|| {
                ui.text("Modal foreground must remain legible.");
                ui.button("Modal control");
            });
        context.render();
    }
    // Snapshot the intended geometry using each command's actual offsets.
    let signature = || {
        let data = unsafe { imgui::DrawData::from_raw(&*imgui::sys::igGetDrawData()) };
        data.draw_lists()
            .flat_map(|list| {
                list.commands()
                    .filter_map(|cmd| {
                        if let imgui::DrawCmd::Elements { count, cmd_params } = cmd {
                            Some(
                                list.idx_buffer()
                                    [cmd_params.idx_offset..cmd_params.idx_offset + count]
                                    .iter()
                                    .map(|i| {
                                        let v =
                                            list.vtx_buffer()[*i as usize + cmd_params.vtx_offset];
                                        (v.pos, v.col)
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    let reordered = || {
        let data = unsafe { imgui::DrawData::from_raw(&*imgui::sys::igGetDrawData()) };
        data.draw_lists().any(|list| {
            let mut offset = 0;
            list.commands().any(|cmd| {
                if let imgui::DrawCmd::Elements { count, cmd_params } = cmd {
                    let differs = cmd_params.idx_offset != offset;
                    offset += count;
                    differs
                } else {
                    false
                }
            })
        })
    };
    assert!(
        reordered(),
        "Exercise a real modal with reordered dimming indices"
    );
    let before = signature();
    normalize_imgui_indices();
    assert!(!reordered(), "Backend must receive sequential indices");
    assert_eq!(
        before,
        signature(),
        "Every command must retain its intended geometry"
    );
    normalize_imgui_indices();
    assert_eq!(before, signature(), "Normalization must be idempotent");
}
fn save_layout(imgui: &mut imgui::Context, editor: Option<&Editor>, screenshot: bool) {
    if !screenshot && let Some(editor) = editor {
        let mut ini = String::new();
        imgui.save_ini_settings(&mut ini);
        let _ = std::fs::write(editor.root.join("UserSettings/editor-layout-v2.ini"), ini);
    }
}
fn configure_project_layout(
    imgui: &mut imgui::Context,
    editor: Option<&Editor>,
    screenshot: bool,
    initial: &mut bool,
) {
    imgui.set_ini_filename(None);
    // Forget the previous project's dock/window settings before loading this project's layout.
    unsafe {
        imgui::sys::igClearIniSettings();
    }
    *initial = true;
    if !screenshot && let Some(editor) = editor {
        let ini = editor.root.join("UserSettings/editor-layout-v2.ini");
        if let Ok(contents) = std::fs::read_to_string(&ini)
            && usable_project_layout(&contents)
        {
            imgui.load_ini_settings(&contents);
            *initial = false;
        }
        imgui.set_ini_filename(Some(ini));
    }
}
fn usable_project_layout(contents: &str) -> bool {
    [
        "Scene",
        "Game",
        "Hierarchy",
        "Inspector",
        "Project",
        "Console",
    ]
    .iter()
    .all(|name| {
        contents
            .split(&format!("[Window][###{name}]"))
            .nth(1)
            .map(|section| section.split("[Window]").next().unwrap_or_default())
            .is_some_and(|section| {
                section.lines().any(|line| {
                    line.strip_prefix("Size=")
                        .and_then(|size| size.split_once(','))
                        .is_some_and(|(width, height)| {
                            [width, height].iter().all(|value| {
                                value
                                    .parse::<f32>()
                                    .is_ok_and(|size| size.is_finite() && size >= 32.)
                            })
                        })
                })
            })
    })
}

#[test]
fn incomplete_or_zero_sized_layouts_restore_default_panels() {
    let valid = [
        "Scene",
        "Game",
        "Hierarchy",
        "Inspector",
        "Project",
        "Console",
    ]
    .map(|name| format!("[Window][###{name}]\nPos=0,0\nSize=640,480\nCollapsed=0\n\n"))
    .join("");
    assert!(usable_project_layout(&valid));
    assert!(!usable_project_layout(""));
    assert!(!usable_project_layout(
        &valid.replace("[Window][###Scene]", "[Window][OldScene]")
    ));
    assert!(!usable_project_layout(&valid.replacen(
        "Size=640,480",
        "Size=0,0",
        1
    )));
    assert!(!usable_project_layout(&valid.replacen(
        "Size=640,480",
        "Size=NaN,480",
        1
    )));
}

fn capture_bytes(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let config = texture.size();
    let padded = (config.width * 4).div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("UI screenshot"),
        size: u64::from(padded) * u64::from(config.height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(config.height),
            },
        },
        wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));
    let (tx, rx) = std::sync::mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::PollType::Wait)?;
    rx.recv()??;
    let mapped = buffer.slice(..).get_mapped_range();
    let mut pixels = Vec::with_capacity((config.width * config.height * 4) as usize);
    for row in mapped.chunks_exact(padded as usize) {
        pixels.extend_from_slice(&row[..config.width as usize * 4]);
    }
    if matches!(
        texture.format(),
        wgpu::TextureFormat::Bgra8UnormSrgb | wgpu::TextureFormat::Bgra8Unorm
    ) {
        for p in pixels.chunks_exact_mut(4) {
            p.swap(0, 2);
        }
    }
    drop(mapped);
    buffer.unmap();
    Ok(crate::mcp::png(config.width, config.height, &pixels)?)
}
#[test]
#[ignore = "requires a graphics adapter"]
fn gpu_scene_and_gizmo_projection_agree_at_different_orbit_distances() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: if cfg!(windows) {
            wgpu::Backends::DX12
        } else {
            wgpu::Backends::PRIMARY
        },
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Gizmo projection regression"),
        size: wgpu::Extent3d {
            width: 960,
            height: 600,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: scene_gpu::FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target = texture.create_view(&Default::default());
    let mut renderer = scene_gpu::SceneGpu::new(&device, &queue);
    let mut editor = crate::editor::Editor::new(std::env::temp_dir().join("epok-projection-test"));
    let mut cube = crate::scene::Actor::cube("Projection marker".into());
    cube.position = [0.4, 0.5, 0.2];
    cube.scale = [0.1; 3];
    cube.material.color = [1., 0., 0.];
    cube.material.unlit = true;
    editor.scene.actors = vec![cube];
    editor.selected = None;
    editor.grid = false;
    for distance in [4., 12., 30.] {
        for pitch in [-0.4, 0.55] {
            editor.view.distance = distance;
            editor.view.pitch = pitch;
            let mut encoder = device.create_command_encoder(&Default::default());
            renderer.render(&device, &queue, &mut encoder, &target, &editor);
            queue.submit(Some(encoder.finish()));
            let png = capture_bytes(&device, &queue, &texture).unwrap();
            let mut reader = png::Decoder::new(std::io::Cursor::new(png))
                .read_info()
                .unwrap();
            let mut pixels = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut pixels).unwrap();
            let mut sum = [0.; 2];
            let mut count = 0.;
            for (index, color) in pixels[..info.buffer_size()].chunks_exact(4).enumerate() {
                if color[0] > 200 && color[1] < 50 && color[2] < 50 {
                    sum[0] += (index % 960) as f32 + 0.5;
                    sum[1] += (index / 960) as f32 + 0.5;
                    count += 1.;
                }
            }
            assert!(
                count > 0.,
                "GPU marker must be visible at distance {distance}"
            );
            let projected =
                crate::viewport::project(&editor.view, editor.scene.world_matrix(0).point([0.; 3]));
            for axis in 0..2 {
                assert!(
                    (sum[axis] / count - projected[axis]).abs() < 1.5,
                    "GPU marker and gizmo disagree at distance {distance}, pitch {pitch}: rendered {sum:?}/{count}, projected {projected:?}"
                );
            }
            assert_eq!(
                crate::picking::pick(&editor.scene, &editor.view, [projected[0], projected[1]]),
                Some(0)
            );
        }
    }
}

fn capture_png(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> Result<Vec<u8>, String> {
    capture_bytes(device, queue, texture).map_err(|e| e.to_string())
}
fn capture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = capture_bytes(device, queue, texture)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Font Awesome codepoints rasterized into the main UI font. The face is the
/// full free set, so nothing has to be re-subset to add one: list it here and
/// use it. Ranges are inclusive pairs, ascending, zero-terminated.
const EDITOR_FA_GLYPHS: &[u32] = &[
    0x25a0, 0x25a0, // square: constant falloff
    0x25d0, 0x25d0, // circle-half-stroke: smooth falloff
    0xf0aa, 0xf0ab, // circle-arrow-up / down: raise, lower
    0xf140, 0xf140, // bullseye: set height
    0xf1fc, 0xf1fc, // paintbrush: paint tile
    0xf1fe, 0xf1fe, // chart-area: linear falloff
    0xf256, 0xf256, // hand: select tool
    0xf2ea, 0xf2ea, // rotate-left: undo
    0xf2f9, 0xf2f9, // rotate-right: redo
    0xf522, 0xf522, // dice: noise
    0xf547, 0xf547, // ruler-horizontal: flatten
    0xf6fc, 0xf6fc, // mountain: sharp falloff
    0xf773, 0xf773, // water: smooth
    0,
];
