use crate::{
    editor::Editor,
    settings::{Preferences, Rendering},
    workspace::Manifest,
};
use imgui::{Condition, StyleColor as C, StyleVar as V};

#[derive(Default)]
pub struct State {
    #[cfg(test)]
    apply_button: [f32; 2],
    pub project: Option<Manifest>,
    pub preferences: Option<Preferences>,
    pub project_open: bool,
    pub preferences_open: bool,
    project_search: String,
    scene_paths: String,
    preferences_search: String,
    project_page: usize,
    preferences_page: usize,
    pub controls: crate::controls_ui::State,
    pub message: String,
    capture: Option<(usize, crate::controls::Control)>,
}
impl State {
    pub fn cancel_capture(&mut self) {
        self.capture = None;
    }
    pub fn capture_binding(&mut self, binding: crate::controls::Binding) {
        if !self.project_open {
            self.cancel_capture();
            return;
        }
        // Pointer motion while choosing a button must not become its binding.
        if let Some((_, control)) = self.capture {
            let axis = matches!(
                binding,
                crate::controls::Binding::MouseAxis { .. }
                    | crate::controls::Binding::GamepadAxis { .. }
            );
            if matches!(control, crate::controls::Control::Axis(_)) && !axis {
                return;
            }
            if matches!(control, crate::controls::Control::Button(_))
                && matches!(binding, crate::controls::Binding::MouseAxis { .. })
            {
                return;
            }
        }
        let Some((pad, control)) = self.capture.take() else {
            return;
        };
        let Some(project) = self.project.as_mut() else {
            return;
        };
        let Some(profile) = project
            .controls
            .pads
            .get(pad)
            .and_then(|slot| slot.profile)
            .and_then(|id| {
                project
                    .controls
                    .profiles
                    .iter_mut()
                    .find(|profile| profile.id == id)
            })
        else {
            return;
        };
        profile.set(control, binding.clone());
        self.message = format!(
            "{} mapped to {}. Apply to save.",
            control.label(),
            binding.label()
        );
    }
}
pub fn open_project(e: &mut Editor) {
    match crate::workspace::read_manifest(&e.root) {
        Ok(value) => {
            e.settings.scene_paths = crate::scene_bank::read(&e.root)
                .map(|r| r.scenes.join("\n"))
                .unwrap_or_default();
            e.settings.project = Some(value);
            e.settings.project_open = true;
            e.settings.project_page = 2;
            e.settings.message.clear();
            e.settings.cancel_capture();
        }
        Err(error) => e.log(error),
    }
}
pub fn open_controls(e: &mut Editor) {
    open_project(e);
    e.settings.project_page = 5;
    e.settings.project_search.clear();
}

#[cfg(test)]
mod controls_capture_tests {
    use super::*;
    use crate::controls::{Axis, Binding, Button, Control, MouseAxis};

    #[test]
    fn capture_ignores_pointer_motion_for_buttons_and_buttons_for_axes() {
        let project: Manifest = serde_json::from_value(serde_json::json!({
            "format_version": 1, "editor_version": "0.3.0", "name": "Input capture",
            "startup_scene": "assets/scenes/Main.epokmap", "auto_build": false
        }))
        .unwrap();
        let mut state = State {
            project: Some(project),
            project_open: true,
            ..Default::default()
        };
        let cross = Control::Button(Button::Cross);
        state.capture = Some((0, cross));
        state.capture_binding(Binding::MouseAxis { axis: MouseAxis::X });
        assert_eq!(state.capture, Some((0, cross)));
        let key = Binding::Keyboard {
            key: "Space".into(),
        };
        state.capture_binding(key.clone());
        assert_eq!(
            state
                .project
                .as_ref()
                .unwrap()
                .controls
                .profile(0)
                .unwrap()
                .binding(cross),
            Some(&key)
        );
        assert!(state.capture.is_none());
        let axis = Control::Axis(Axis::LeftX);
        state.capture = Some((0, axis));
        state.capture_binding(key.clone());
        assert_eq!(state.capture, Some((0, axis)));
        state.project_open = false;
        state.capture_binding(Binding::MouseAxis { axis: MouseAxis::Y });
        assert!(state.capture.is_none());
        assert!(
            state
                .project
                .as_ref()
                .unwrap()
                .controls
                .profile(0)
                .unwrap()
                .binding(axis)
                .is_none()
        );
        assert!(Button::ALL.iter().all(|b| b.label().is_ascii()));
    }
}
/// `cpp_name` of every class a scene Blueprint may derive from, in name order.
fn scene_script_parents(e: &crate::editor::Editor) -> Vec<String> {
    let Some(model) = e.object_model() else {
        return Vec::new();
    };
    let mut names = model
        .scene_script_parents()
        .map(|c| c.cpp_name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names
}
pub fn open_preferences(e: &mut Editor) {
    e.settings.preferences = Some(e.preferences.clone());
    e.settings.preferences_open = true;
    e.settings.message.clear();
}
pub fn open_mcp(e: &mut Editor) {
    open_preferences(e);
    e.settings.preferences_page = 2;
}
pub fn open_dependencies(e: &mut Editor) {
    open_preferences(e);
    e.settings.preferences_page = 3;
    e.settings.preferences_search.clear();
}
fn mcp_page(ui: &imgui::Ui, p: &mut Preferences, e: &Editor) {
    ui.text("General  >  Integrations / MCP");
    ui.text_wrapped("Connect an external client to this editor. Access includes scenes, assets, scripts, builds and viewport screenshots.");
    section(ui, "Local MCP Server", || {
        row(
            ui,
            "Enable MCP Server",
            "Off by default. Listens on this computer while a project is open.",
            || {
                if ui.checkbox("##mcp-enable", &mut p.mcp.enabled) {
                    p.mcp.prepare();
                }
            },
        );
        row(
            ui,
            "Port",
            "Apply to restart the connection on a different port.",
            || {
                let mut port = i32::from(p.mcp.port);
                if crate::gui::Drag::new("##mcp-port")
                    .speed(1.)
                    .build(ui, &mut port)
                {
                    p.mcp.port = port.clamp(1024, 65535) as u16;
                }
            },
        );
        row(
            ui,
            "Status",
            "Current server status. Apply pending changes to update it.",
            || {
                ui.text_wrapped(if e.mcp.status.is_empty() {
                    "Disabled"
                } else {
                    &e.mcp.status
                });
            },
        );
        row(
            ui,
            "Endpoint",
            "Streamable HTTP endpoint for MCP clients.",
            || {
                ui.text_wrapped(p.mcp.endpoint());
            },
        );
        row(
            ui,
            "Access Key",
            "Regenerating the key disconnects existing clients after Apply.",
            || {
                ui.text_disabled(if p.mcp.token.is_empty() {
                    "Generated when enabled"
                } else {
                    "Private key configured"
                });
                if ui.button("Copy Key") {
                    ui.set_clipboard_text(&p.mcp.token);
                }
                ui.same_line();
                if ui.button("Regenerate") {
                    p.mcp.rotate_token();
                }
            },
        );
    });
    ui.spacing();
    let _disabled = ui.begin_disabled(p.mcp.token.is_empty());
    if ui.button("Copy HTTP Client Config") {
        ui.set_clipboard_text(serde_json::to_string_pretty(&serde_json::json!({"mcpServers":{"epok":{"url":p.mcp.endpoint(),"headers":{"Authorization":format!("Bearer {}",p.mcp.token)}}}})).unwrap());
    }
    if ui.button("Copy Stdio Client Config")
        && let Ok(executable) = std::env::current_exe()
    {
        ui.set_clipboard_text(serde_json::to_string_pretty(&serde_json::json!({"mcpServers":{"epok":{"command":executable,"args":["--mcp-stdio"]}}})).unwrap());
    }
    ui.spacing();
    ui.text_wrapped("Apply to start or stop the server. Use the copied configuration in an MCP client that supports Streamable HTTP. The access key grants editing access to the open game project.");
}
fn row(ui: &imgui::Ui, label: &str, tip: &str, control: impl FnOnce()) {
    ui.table_next_row();
    ui.table_next_column();
    ui.align_text_to_frame_padding();
    ui.text(label);
    if ui.is_item_hovered() {
        ui.tooltip_text(tip);
    }
    ui.table_next_column();
    ui.set_next_item_width(-12.);
    control();
}
fn section(ui: &imgui::Ui, title: &str, body: impl FnOnce()) {
    ui.spacing();
    if crate::gui::heading(ui, title)
        && let Some(_table) = ui.begin_table_with_flags(
            format!("rows-{title}"),
            2,
            imgui::TableFlags::SIZING_STRETCH_PROP
                | imgui::TableFlags::ROW_BG
                | imgui::TableFlags::BORDERS_INNER_H,
        )
    {
        ui.table_setup_column_with(imgui::TableColumnSetup {
            name: "Property",
            init_width_or_weight: 0.42,
            ..Default::default()
        });
        ui.table_setup_column_with(imgui::TableColumnSetup {
            name: "Value",
            init_width_or_weight: 0.58,
            ..Default::default()
        });
        body();
    }
}
fn matches(query: &str, text: &str) -> bool {
    query.trim().is_empty() || text.to_lowercase().contains(&query.trim().to_lowercase())
}
pub fn windows(ui: &imgui::Ui, e: &mut Editor) {
    e.dependencies.poll();
    if e.dependencies.warning_ui(ui) {
        open_dependencies(e);
    }
    let mut state = std::mem::take(&mut e.settings);
    let _vars = [
        V::WindowPadding([16., 14.]),
        V::FramePadding([8., 7.]),
        V::ItemSpacing([8., 8.]),
        V::WindowRounding(4.),
        V::FrameRounding(2.),
        V::CellPadding([12., 8.]),
    ]
    .map(|v| ui.push_style_var(v));
    for preferences in [false, true] {
        let open = if preferences {
            state.preferences_open
        } else {
            state.project_open
        };
        if !open {
            continue;
        }
        let mut open = true;
        let title = if preferences {
            "Editor Preferences"
        } else {
            "Project Settings"
        };
        let screen = ui.io().display_size;
        let previous_page = if preferences {
            state.preferences_page
        } else {
            state.project_page
        };
        ui.window(title).opened(&mut open).position([screen[0]*0.5,screen[1]*0.5],Condition::Appearing).position_pivot([0.5,0.5])
            .size([(screen[0]-60.).min(1180.),(screen[1]-80.).min(780.)],Condition::FirstUseEver).size_constraints([760.,460.],[1800.,1200.]).build(||{
            ui.text_disabled(if preferences{"EPOK  /  LOCAL EDITOR"}else{"EPOK  /  GAME PROJECT"});
            ui.separator();
            let available=ui.content_region_avail();
            let height=(available[1]-60.).max(200.);
            ui.child_window(format!("nav-{title}")).size([215.,height]).build(||{
                ui.text_disabled(if preferences{"GENERAL"}else{"PROJECT"});
                let pages: &[(&str,&str)]=if preferences{&[("Viewports","grid camera speed navigation"),("Play","game integer scale filter emulator serial NOTPSXSerial nops COM fast"),("Integrations / MCP","server integrations mcp connection port key"),("Dependencies","tools paths install repair make MIPS Nugget PCSX psxavenc mkpsxiso libclang")]}else{&[("Description","name project scripting lua execution native VM bytecode source interpreter"),("Maps & Build","startup scene build compilation asset report generate play target content data transition fade loading text image"),("Rendering","resolution display video NTSC interlaced progressive pixels dithering RGB555 banding retained packets visibility geometry static movement position interpolation smoothing camera experimental FPS performance"),("Streaming","geometry pool pages memory CD disc music XA triangle budget preload nearby prefetch experimental FPS performance"),("Debug","HUD overlay FPS CPU GTE GPU DMA SPU audio bars runtime performance"),("Controls","controller pad multitap keyboard mouse gamepad analog profile mapping input") ]};
                let page=if preferences{&mut state.preferences_page}else{&mut state.project_page};
                let query=if preferences{&state.preferences_search}else{&state.project_search};
                for (i,(label,keywords)) in pages.iter().enumerate(){
                    if !preferences && i==2 {ui.spacing();ui.text_disabled("ENGINE");}
                    let active=*page==i && query.is_empty();
                    let _selected=ui.push_style_color(C::Header,[0.075,0.28,0.48,1.]);
                    if matches(query,&format!("{label} {keywords}")) && ui.selectable_config(label).selected(active).size([0.,30.]).build(){*page=i;}
                }
                ui.spacing();ui.separator();ui.text_wrapped(if preferences && state.preferences_page==3 {"Tool paths are saved locally. A project's Local.epokconfig takes priority over installation settings."}else if preferences{"Preferences are saved for this user and shared across projects."}else{"These settings travel with your project and are used by native builds."});
            });
            ui.same_line();
            ui.child_window(format!("content-{title}")).size([0.,height]).build(||{
                if previous_page != if preferences { state.preferences_page } else { state.project_page } { ui.set_scroll_y(0.); state.capture=None; }
                let query=if preferences{&mut state.preferences_search}else{&mut state.project_search};
                ui.set_next_item_width(-1.);ui.input_text("##search",query).hint("Search settings...").build();
                ui.spacing();
                let q=query.clone();
                let keywords=if preferences{"Viewports grid camera speed navigation Play game integer scale filter emulator serial NOTPSXSerial nops COM fast Integrations MCP server connection port key Dependencies tools paths install repair make MIPS Nugget PCSX psxavenc mkpsxiso libclang"}else{"Description name project scripting lua execution native VM bytecode source interpreter Maps Build startup scene build compilation asset report generate play target content data transition fade loading text image Rendering resolution display video NTSC interlaced progressive pixels dithering RGB555 banding retained packets visibility geometry static movement position interpolation smoothing camera Streaming pool pages memory CD disc music XA triangle budget preload nearby prefetch experimental FPS performance Debug HUD overlay FPS CPU GTE GPU DMA SPU audio bars runtime Controls controller pad multitap keyboard mouse gamepad analog profile mapping input"};
                if !matches(&q,keywords){ui.text_disabled("No settings match your search.");}
                if preferences {
                    if (state.preferences_page==3 && q.is_empty()) || (!q.is_empty() && matches(&q,"Dependencies tools paths install repair make MIPS Nugget PCSX psxavenc mkpsxiso libclang")) { e.dependencies.page(ui, e.job.is_some() || e.assets.busy || e.bake_job.is_some()); }
                    let p=state.preferences.as_mut().unwrap();
                    if (state.preferences_page==2 && q.is_empty()) || (!q.is_empty() && matches(&q,"Integrations MCP server connection port key")) { mcp_page(ui,p,e); }
                    if (state.preferences_page==0 && q.is_empty()) || (!q.is_empty() && matches(&q,"Viewports grid camera speed navigation")) {
                        ui.text("General  >  Viewports");
                        section(ui,"Viewport Options",||{
                            row(ui,"Show Grid","Default for scenes without a saved template view.",||{ui.checkbox("##grid",&mut p.show_grid);});
                            row(ui,"Camera Speed","Default free-flight navigation speed.",||{crate::gui::Drag::new("##speed").speed(0.1).range(0.01,1000.).build(ui,&mut p.fly_speed);});
                        });
                    }
                    if (state.preferences_page==1 && q.is_empty()) || (!q.is_empty() && matches(&q,"Play game integer scale filter emulator serial NOTPSXSerial nops COM fast")) {
                        ui.text("General  >  Play");
                        section(ui,"Game View",||{row(ui,"Scaling","Fit fills the width or height while keeping 4:3. Stretch fills both axes. Integer uses whole scale steps when possible. This affects display only; it does not rebuild the game.",||{
                            if let Some(_combo)=ui.begin_combo("##game-scale-setting",p.game_scale.label()) {
                                for mode in crate::settings::GameScale::ALL {if ui.selectable_config(mode.label()).selected(p.game_scale==mode).build(){p.game_scale=mode;}}
                            }
                        });});
                        section(ui,"Emulator Window",||{row(ui,"Smooth Image","Linear filtering in the external debugger window. Applies on next Play.",||{ui.checkbox("##linear",&mut p.emulator_linear_filter);});});
                        ui.text_wrapped("Game view uses point filtering. Leave Smooth Image off for crisp pixels in the emulator window.");
                        section(ui,"PSX via serial",||{
                            ui.text_wrapped("Epok keeps serial tools inside its installation and remembers your adapter. Start Unirom, then Play to build and send the program.");
                            if ui.button("PSX connection...") { crate::serial_ui::open(e); }
                        });
                    }
                } else {
                    let m=state.project.as_mut().unwrap();
                    if (state.project_page==0 && q.is_empty()) || (!q.is_empty() && matches(&q,"Description name project scripting lua execution native VM bytecode source interpreter")) {
                        ui.text("Project  >  Description");
                        section(ui,"About",||{row(ui,"Project Name","Name shown in the Hub.",||{ui.input_text("##name",&mut m.name).build();});});
                        section(ui,"Audio",||{row(ui,"Project Default SoundBank","Used by raw MIDI preview and MusicSequences without an explicit bank. No OS instruments are substituted.",||{
                            crate::asset_ui::bank_selector(ui,"##default-sound-bank",&mut m.default_sound_bank,&e.assets.index,"None (MIDI needs a bank)");
                        });});
                        section(ui,"Scripting",||{row(ui,"Default Scene Blueprint Parent","Proposed in Map Settings when a map's scene Blueprint is created. Existing maps are never changed.",||{
                            let parents=scene_script_parents(e);
                            let current=m.default_scene_script_parent.clone().unwrap_or_else(||"None (epok::SceneScriptActor)".into());
                            ui.set_next_item_width(-1.);
                            if let Some(_combo)=ui.begin_combo("##default-scene-script-parent",&current){
                                if ui.selectable("None (epok::SceneScriptActor)"){m.default_scene_script_parent=None;}
                                for name in &parents {
                                    if ui.selectable_config(name).selected(Some(name)==m.default_scene_script_parent.as_ref()).build(){m.default_scene_script_parent=Some(name.clone());}
                                }
                            }
                            if parents.is_empty(){ui.text_disabled("No SceneScriptActor class is available in this project.");}
                        });
                        row(ui,"Lua Execution","How this project's Lua scripts run. One mode per build; the same scripts are valid in all of them. Changing it rebuilds script artifacts and relinks.",||{
                            ui.set_next_item_width(-1.);
                            if let Some(_combo)=ui.begin_combo("##lua-execution",m.lua_execution.label()){
                                for mode in crate::settings::LuaExecution::ALL {
                                    if ui.selectable_config(mode.label()).selected(m.lua_execution==mode).build(){m.lua_execution=mode;}
                                    if ui.is_item_hovered(){ui.tooltip_text(mode.describe());}
                                }
                            }
                        });
                        ui.text_wrapped(if m.lua_execution.is_vm(){"The same scripts run through the PsyQo Lua interpreter, which is linked into the game and reserves a static memory budget."}else{m.lua_execution.describe()});
                        row(ui,"Lua Language Profile","The source-language and value ABI contract. Existing projects stay on v1 until changed explicitly.",||{
                            ui.set_next_item_width(-1.);
                            if let Some(_combo)=ui.begin_combo("##lua-profile",m.lua_profile.label()){
                                for profile in crate::settings::LuaProfile::ALL {
                                    if ui.selectable_config(profile.label()).selected(m.lua_profile==profile).build(){m.lua_profile=profile;}
                                    if ui.is_item_hovered(){ui.tooltip_text(profile.describe());}
                                }
                            }
                        });
                        ui.text_wrapped(m.lua_profile.describe());
                        });
                    }
                    if (state.project_page==1 && q.is_empty()) || (!q.is_empty() && matches(&q,"Maps Build startup scene build compilation asset report generate play target content data transition fade loading text image")) {
                        ui.text("Project  >  Maps & Build");
                        section(ui,"Default Maps",||{row(ui,"Startup Scene","Scene loaded the next time this project opens.",||{ui.input_text("##startup",&mut m.startup_scene).build();});});
                        ui.text_wrapped("Build generates the current Play configuration. Play builds pending changes and launches it. Editing never starts compilation automatically.");
                        section(ui,"Build Reports",||{row(ui,"Generate Asset Report","Generate the detailed asset and memory report after Build/Play. File sizes are always recorded.",||{ui.checkbox("##asset-report",&mut m.build.generate_asset_report);});});
                        ui.spacing();
                        ui.text("Play Profile");
                        crate::play_ui::controls(ui,&mut m.play, &e.root, ((ui.content_region_avail()[0]-24.)/4.).min(220.));
                        section(ui,"Scene Transitions",||{
                            row(ui,"Fade Out (ms)","Fade picture, music and sound effects to silence before releasing the outgoing scene.",||{let mut ms=i32::from(m.transition.fade_out_ms);if crate::gui::Drag::new("##fade-out").speed(1.).build(ui, &mut ms){m.transition.fade_out_ms=ms.clamp(0,10000) as u16;}});
                            row(ui,"Fade In (ms)","Reveal the ready scene and restore its authored audio levels.",||{let mut ms=i32::from(m.transition.fade_in_ms);if crate::gui::Drag::new("##fade-in").speed(1.).build(ui, &mut ms){m.transition.fade_in_ms=ms.clamp(0,10000) as u16;}});
                            row(ui,"Loading Text","Displayed at the lower right, inside the TV safe margin. Up to 95 printable ASCII characters.",||{ui.input_text("##loading-text",&mut m.transition.text).build();});
                            row(ui,"Loading Image","Optional resident texture, at most 64 x 64 pixels with even width.",||{
                                let label=m.transition.image.and_then(|id|e.assets.index.resolve(id).ok()).map(|r|r.path.file_stem().unwrap_or_default().to_string_lossy().into_owned()).unwrap_or_else(||"None".into());
                                if let Some(_combo)=ui.begin_combo("##loading-image",label){
                                    if ui.selectable("None"){m.transition.image=None;}
                                    for record in e.assets.index.assets.values().flatten().filter(|r|r.meta.kind==crate::assets::Kind::Texture){
                                        if ui.selectable(format!("{}##{}",record.path.file_stem().unwrap_or_default().to_string_lossy(),record.meta.id)){m.transition.image=Some(record.meta.id);}
                                    }
                                }
                            });
                        });
                        section(ui,"Scene Banks",||{
                            ui.text_wrapped("Additional scenes, one assets/scenes/*.epokmap path per line. Startup is bank 0; scripts may request a bank by name or index.");
                            ui.input_text_multiline("##scene-banks",&mut state.scene_paths,[-1.,100.]).build();
                        });
                    }
                    if (state.project_page==2 && q.is_empty()) || (!q.is_empty() && matches(&q,"Rendering resolution display video NTSC interlaced progressive pixels retained packets geometry static visibility experimental FPS performance movement position interpolation smoothing camera dithering RGB555 banding")) {
                        ui.text("Engine  >  Rendering");
                        ui.text_wrapped("Configure the native PlayStation output. Changes apply to the next build and standalone export.");
                        section(ui,"Default Display",||{
                            row(ui,"Video Standard","The current runtime targets NTSC at a fixed 1/60 simulation step.",||{ui.text("NTSC");});
                            row(ui,"Resolution","640 x 480 is the maximum NTSC mode. Lower modes reduce framebuffer work.",||{
                                let labels:Vec<_>=Rendering::MODES.iter().map(|&(width,height)|Rendering{width,height,..Default::default()}.label()).collect();
                                let mut index=Rendering::MODES.iter().position(|&(w,h)|w==m.rendering.width && h==m.rendering.height).unwrap_or(9);
                                if ui.combo_simple_string("##resolution",&mut index,&labels){let (width,height)=Rendering::MODES[index];m.rendering=Rendering{width,height,..m.rendering};}
                            });
                            row(ui,"Scan Mode","480-line modes are interlaced; 240-line modes are progressive.",||{ui.text(if m.rendering.height==480{"Interlaced"}else{"Progressive"});});
                            row(ui,"Output Pixels","Pixels rendered per frame, excluding overdraw.",||{ui.text(format!("{}",u32::from(m.rendering.width)*u32::from(m.rendering.height)));});
                            row(ui,"Display Aspect Ratio","Native pixels are presented at the intended TV aspect ratio.",||{ui.text("4:3");});
                        });
                        ui.spacing();ui.text_colored([0.5,0.72,0.94,1.],"240-line modes use progressive output.");
                        ui.text_wrapped("Higher resolution uses more GPU fill work. Interlaced output can flicker or show motion artifacts on physical displays. HUD coordinates use the selected resolution.");
                        section(ui,"Movement",||{
                            row(ui,"Position Interpolation","Smooth entity and camera translations between 60 Hz simulation steps. Physics and running speed are unchanged. Adds up to one simulation tick of visual latency and a small fixed RAM/CPU cost. Rotation, skeletal poses and particles are not interpolated. Changes apply on the next build.",||{
                                ui.checkbox("##motion-interpolation",&mut m.rendering.motion_interpolation);
                            });
                        });
                        section(ui,"Geometry",||{
                            row(ui,"Sprite Triangle Budget","Fixed, double-buffered sprite/VFX packet pool. Lower values save RAM; excess triangles are counted as dropped. 64..2048, applies on next build. Includes particles, not mesh triangles or HUD.",||{
                                let mut value=i32::from(m.rendering.sprite_triangle_budget);
                                if ui.input_int("##sprite-triangle-budget",&mut value).build(){m.rendering.sprite_triangle_budget=value.clamp(64,2048) as u16;}
                            });
                            row(ui,"3D Dithering","Reduce RGB555 color banding in shaded 3D geometry using the PS1 GPU's ordered dithering. Adds a fine pixel pattern. HUD and text remain undithered. Applies on the next native build.",||{
                                ui.checkbox("##dither-3d",&mut m.rendering.dither_3d);
                            });
                            row(ui,"Retained Packets","Reuse prepared geometry between frames to reduce CPU work. Uses additional RAM. Lighting and material changes refresh the cached data automatically.",||{
                                let mut on=m.rendering.retained_geometry;
                                if ui.checkbox("##retained-geometry",&mut on){m.rendering.retained_geometry=on;}
                                ui.same_line();ui.text(if on{"On (default)"}else{"Off: per-frame packets"});
                            });
                            row(ui,"Precomputed Visibility","Experimental. May lower FPS because mask queries add CPU work. No performance improvement is guaranteed. Compare native frame median and p95 with this option off before enabling it. Camera movement and rotation remain unrestricted.",||{
                                ui.checkbox("##precomputed-visibility",&mut m.rendering.precomputed_visibility);
                                ui.same_line();ui.text_colored([1.,0.72,0.3,1.],"Experimental - may lower FPS");
                            });
                        });
                        section(ui,"Current Scene HUD Budget",||{
                            ui.text_wrapped("HUD budgets belong to one map. Edit them in Map Settings: click the map root in the Hierarchy.");
                            if ui.button("Open Map Settings"){e.map_settings=true;}
                        });
                    }
                    if (state.project_page==4 && q.is_empty()) || (!q.is_empty() && matches(&q,"Debug HUD overlay FPS CPU GTE GPU DMA SPU audio bars runtime performance")) {
                        ui.text("Engine  >  Debug");
                        ui.text_wrapped("Runtime overlay, drawn above the game in the bottom-left corner. Each toggle applies independently on the next build, including exports. All disabled by default.");
                        section(ui,"Runtime Counters",||{
                            row(ui,"FPS","Rendered frames per second, averaged over at least 30 NTSC vblanks. Measured on the PSX, not the editor or host PC.",||{ui.checkbox("##debug-fps",&mut m.debug.fps);});
                            row(ui,"CPU Frame Time","CPU: time inside the game frame, including waits but excluding the final flip/vblank wait. Full bar = one 16.7 ms NTSC budget; red = budget reached or exceeded.",||{ui.checkbox("##debug-cpu",&mut m.debug.cpu);});
                            row(ui,"Geometry / GTE Time","GTE: mesh vertex preparation and GTE projection time. Includes CPU work; excludes lighting and software projection. Not hardware GTE utilization. Full bar = 16.7 ms.",||{ui.checkbox("##debug-gte",&mut m.debug.gte);});
                            row(ui,"GPU Command DMA","GPU: observed command-transfer time overlapping the CPU frame. Not rasterizer utilization; transfers completed before the frame starts are not counted. Full bar = 16.7 ms.",||{ui.checkbox("##debug-gpu",&mut m.debug.gpu);});
                            row(ui,"SPU Sample Memory","SPU: resident sound-effect data plus the reserved 4 KiB area, out of 512 KiB of sound RAM. Not audio processor load or XA streaming-buffer fill.",||{ui.checkbox("##debug-spu",&mut m.debug.spu_ram);});
                        });
                        ui.text_wrapped("Fixed packets, no dynamic allocation and no additional font texture. All five counters together use less than 2 KiB of packet/state RAM, plus code and a few GPU primitives. With every toggle off, the overlay is compiled out.");
                    }
                    if (state.project_page==3 && q.is_empty()) || (!q.is_empty() && matches(&q,"Streaming geometry pool pages memory CD disc music XA triangle budget preload nearby prefetch experimental FPS performance")) {
                        ui.text("Engine  >  Streaming");
                        ui.text_wrapped("Control how much geometry stays in memory. Changes apply to the next build.");
                        ui.text_colored([1.,0.72,0.3,1.],"Experimental");
                        ui.text_wrapped("Geometry streaming and preloading may lower FPS and cause frame stalls. Keep streaming off unless native performance comparisons meet your frame-time and dropped-step requirements. Development and validation are ongoing.");
                        section(ui,"Geometry Loading",||{
                            row(ui,"Geometry Streaming","Experimental. Loads immutable editable mesh data from disc. May lower FPS and cause frame stalls. Requires a disc image build. Disabled by default.",||{
                                ui.checkbox("##streaming-geometry",&mut m.rendering.streaming_geometry);
                                ui.same_line();ui.text_colored([1.,0.72,0.3,1.],"Experimental");
                            });
                            row(ui,"Preload Nearby Geometry","Experimental. Requests a candidate page while a memory slot and the disc are available. Extra work may lower FPS; reduced waiting is not guaranteed. Inactive while Geometry Streaming is off.",||{
                                let _disabled=ui.begin_disabled(!m.rendering.streaming_geometry);
                                ui.checkbox("##streaming-prefetch",&mut m.rendering.streaming_prefetch);
                                ui.same_line();ui.text_colored([1.,0.72,0.3,1.],"Experimental");
                            });
                            row(ui,"Memory Pool Pages","Each page reserves 64 KiB. More pages retain more geometry and reduce repeat disc reads. Choose 2 to 8 pages.",||{
                                let _disabled=ui.begin_disabled(!m.rendering.streaming_geometry);
                                let mut count=i32::from(m.rendering.streaming_pool_pages);
                                if crate::gui::Drag::new("##streaming-pool-pages").speed(1.).build(ui, &mut count){m.rendering.streaming_pool_pages=count.clamp(2,8) as u8;}
                            });
                            row(ui,"Geometry Memory Budget","RAM reserved for loaded geometry pages when streaming is enabled. Other scene resources use additional memory.",||{
                                ui.text(if m.rendering.streaming_geometry {format!("{} KiB",u32::from(m.rendering.streaming_pool_pages)*64)} else {"Disabled".into()});
                            });
                            row(ui,"Per-frame Triangle Budget","Maximum rendered triangles when streaming is active, including resident meshes and triangles created by clipping. Lower values save RAM; geometry beyond the budget is omitted and reported as dropped triangles.",||{
                                let _disabled=ui.begin_disabled(!m.rendering.streaming_geometry);
                                let mut triangles=i32::from(m.rendering.streaming_triangle_budget);
                                if crate::gui::Drag::new("##streaming-triangle-budget").speed(1.).build(ui, &mut triangles){m.rendering.streaming_triangle_budget=triangles.clamp(512,8192) as u16;}
                            });
                            row(ui,"Disc Music","Geometry reads and XA music share the disc drive. Active XA music pauses during a geometry read and restarts afterward.",||{
                                ui.text_wrapped("Pause and restart during geometry reads");
                            });
                        });
                    }
                    if (state.project_page==5 && q.is_empty()) || (!q.is_empty() && matches(&q,"Controls controller pad multitap keyboard mouse gamepad analog profile mapping input")) {
                        crate::controls_ui::page(ui,&mut m.controls,&mut state.controls,&mut state.capture);
                    }
                }
            });
            ui.separator();
            let dependency_page=preferences && ((state.preferences_page==3 && state.preferences_search.is_empty()) || (!state.preferences_search.is_empty() && matches(&state.preferences_search,"Dependencies tools paths install repair make MIPS Nugget PCSX psxavenc mkpsxiso libclang")));
            // The dependency page adds Install / Repair to the footer row. Reserve
            // its width too, or Apply is laid out past the window and cannot be clicked.
            let buttons=if dependency_page {430.} else {270.};
            ui.child_window(format!("message-{title}")).size([(ui.content_region_avail()[0]-buttons).max(150.),44.]).build(||{ui.text_wrapped(&state.message);});
            ui.same_line();
            let reset_disabled=ui.begin_disabled(dependency_page);
            if ui.button(if preferences {"Reset Defaults"} else if state.project_page==5 {"Reset Controls"} else if state.project_page==4 {"Reset Debug"} else if state.project_page==3 {"Reset Streaming"} else {"Reset Rendering"}) {
                if preferences {state.preferences=Some(Preferences::default());}else if let Some(m)=state.project.as_mut(){
                    let defaults=Rendering::default();
                    if state.project_page==5 {m.controls=Default::default();state.capture=None;}
                    else if state.project_page==4 {m.debug=Default::default();}
                    else if state.project_page==3 {m.rendering.streaming_geometry=defaults.streaming_geometry;m.rendering.streaming_pool_pages=defaults.streaming_pool_pages;m.rendering.streaming_triangle_budget=defaults.streaming_triangle_budget;m.rendering.streaming_prefetch=defaults.streaming_prefetch;}
                    else {m.rendering=Rendering{streaming_geometry:m.rendering.streaming_geometry,streaming_pool_pages:m.rendering.streaming_pool_pages,streaming_triangle_budget:m.rendering.streaming_triangle_budget,streaming_prefetch:m.rendering.streaming_prefetch,..defaults};}
                }
                state.message="Defaults restored in this window. Apply to save.".into();
            }
            drop(reset_disabled);
            ui.same_line();
            if dependency_page {
                e.dependencies
                    .install_all_control(ui, e.job.is_some() || e.assets.busy || e.bake_job.is_some());
                ui.same_line();
            }
            let _blue=ui.push_style_color(C::Button,[0.035,0.35,0.65,1.]);
            let _disabled=ui.begin_disabled((!preferences || dependency_page) && (e.job.is_some() || e.dependencies.busy()));
            if ui.button_with_size("Apply",[90.,0.]) {
                let result=if dependency_page {e.dependencies.save().and_then(|()|e.apply_preferences(state.preferences.as_ref().unwrap().clone()))}else if preferences {e.apply_preferences(state.preferences.as_ref().unwrap().clone())}else{
                    let registry=crate::scene_bank::Registry{scenes:state.scene_paths.lines().map(str::trim).filter(|s|!s.is_empty()).map(str::to_string).collect()};
                    e.apply_project_configuration(state.project.as_ref().unwrap().clone(),Some(registry))
                };
                state.message=match result{Ok(())=>{if preferences{state.preferences=Some(e.preferences.clone());}"Settings saved.".into()},Err(error)=>error};
            }
            #[cfg(test)] { let a=ui.item_rect_min();let b=ui.item_rect_max();state.apply_button=[(a[0]+b[0])*0.5,(a[1]+b[1])*0.5]; }
            if !preferences && e.job.is_some() && ui.is_item_hovered(){ui.tooltip_text("Stop Play or wait for the build before applying settings.");}
        });
        if preferences {
            state.preferences_open = open;
        } else {
            state.project_open = open;
            if !open {
                state.cancel_capture();
            }
        }
    }
    e.settings = state;
}

#[cfg(test)]
pub fn verify_interactions(context: &mut imgui::Context) {
    let root = crate::workspace::editor_home()
        .join(".epok")
        .join(format!("settings-ui-{}", uuid::Uuid::new_v4()));
    drop(
        crate::workspace::create(&root, "Settings UI", crate::workspace::Template::Basic).unwrap(),
    );
    let mut editor = Editor::new(root.clone());
    editor.dependencies.warning = false;
    open_project(&mut editor);
    editor.settings.project.as_mut().unwrap().rendering = Rendering {
        width: 320,
        height: 240,
        ..Default::default()
    };
    let frame = |context: &mut imgui::Context, editor: &mut Editor| {
        windows(context.frame(), editor);
        context.render();
    };
    frame(context, &mut editor);
    frame(context, &mut editor);
    context
        .io_mut()
        .add_mouse_pos_event(editor.settings.apply_button);
    frame(context, &mut editor);
    context
        .io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, true);
    frame(context, &mut editor);
    context
        .io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, false);
    frame(context, &mut editor);
    assert_eq!(
        crate::workspace::read_manifest(&root).unwrap().rendering,
        Rendering {
            width: 320,
            height: 240,
            ..Default::default()
        }
    );
    assert_eq!(editor.settings.message, "Settings saved.");
    editor.settings.project_page = 3;
    {
        let rendering = &mut editor.settings.project.as_mut().unwrap().rendering;
        rendering.precomputed_visibility = true;
        rendering.motion_interpolation = false;
        rendering.streaming_geometry = true;
        rendering.streaming_pool_pages = 6;
    }
    frame(context, &mut editor);
    context
        .io_mut()
        .add_mouse_pos_event(editor.settings.apply_button);
    frame(context, &mut editor);
    context
        .io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, true);
    frame(context, &mut editor);
    context
        .io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, false);
    frame(context, &mut editor);
    let persisted = crate::workspace::read_manifest(&root).unwrap().rendering;
    assert!(persisted.streaming_geometry && persisted.precomputed_visibility);
    assert!(!persisted.motion_interpolation);
    assert_eq!(persisted.streaming_pool_pages, 6);
    editor.settings.project_search = "streaming".into();
    frame(context, &mut editor);
    editor.settings.project_search = "visibility".into();
    frame(context, &mut editor);
    editor.settings.project_search = "resolution".into();
    frame(context, &mut editor);
    editor.settings.project_search = "no such setting".into();
    frame(context, &mut editor);
    editor.settings.project_open = false;
    open_preferences(&mut editor);
    frame(context, &mut editor);
    editor.settings.preferences_page = 1;
    frame(context, &mut editor);
    open_mcp(&mut editor);
    frame(context, &mut editor);
    assert!(!editor.settings.preferences.as_ref().unwrap().mcp.enabled);
    // Use a project override so the actual Apply button cannot change machine tools.
    let config = crate::project::Config {
        psxavenc: "missing/old.exe".into(),
        web_port: 9123,
        ..Default::default()
    };
    std::fs::write(
        root.join("Local.epokconfig"),
        crate::document::to_vec(&config).unwrap(),
    )
    .unwrap();
    editor.dependencies = crate::dependencies::State::new(&root);
    editor.dependencies.warning = false;
    open_dependencies(&mut editor);
    editor.dependencies.draft.psxavenc = "replacement/encoder.exe".into();
    frame(context, &mut editor);
    frame(context, &mut editor);
    context
        .io_mut()
        .add_mouse_pos_event(editor.settings.apply_button);
    frame(context, &mut editor);
    context
        .io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, true);
    frame(context, &mut editor);
    context
        .io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, false);
    frame(context, &mut editor);
    let saved = crate::project::Config::load(&root).unwrap();
    assert_eq!(
        std::path::Path::new(&saved.psxavenc),
        root.join("replacement/encoder.exe")
    );
    assert_eq!(saved.web_port, 9123);
}
