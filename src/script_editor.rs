//! Integrated Monaco workspace. UI state belongs to the project; the WebView
//! belongs to the platform thread and is laid out inside the Code dock window.
use crate::{editor::Editor, script_lsp, script_tools, script_workspace};
use serde_json::{Value, json};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, SyncSender},
    },
};

static WEB: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources/code-editor");

pub struct State {
    pub open: bool,
    pub region: Option<[f32; 4]>,
    pub region_owner: u32,
    pub pocket_app: Option<uuid::Uuid>,
    pub pocket_requested: Option<(uuid::Uuid, Option<String>, Option<usize>)>,
    pub workspace: script_workspace::Workspace,
    focus: bool,
    web_focus: bool,
    pub setup_requested: bool,
    ready: bool,
    config: Option<script_tools::Config>,
    service: Option<script_lsp::Service>,
    input: SyncSender<String>,
    changes: Arc<Mutex<BTreeMap<String, (String, u64)>>>,
    incoming: Receiver<String>,
    async_events: Receiver<Value>,
    async_sender: SyncSender<Value>,
    outbox: Vec<Value>,
    pending_open: Option<(String, usize)>,
    installing: bool,
    registry_revision: u64,
    pub error: Option<String>,
}
impl Default for State {
    fn default() -> Self {
        let (input, incoming) = mpsc::sync_channel(128);
        let (async_sender, async_events) = mpsc::sync_channel(128);
        Self {
            open: false,
            region: None,
            region_owner: 0,
            pocket_app: None,
            pocket_requested: None,
            workspace: Default::default(),
            focus: false,
            web_focus: false,
            setup_requested: false,
            ready: false,
            config: None,
            service: None,
            input,
            changes: Default::default(),
            incoming,
            async_events,
            async_sender,
            outbox: Vec::new(),
            pending_open: None,
            installing: false,
            registry_revision: 0,
            error: None,
        }
    }
}
impl State {
    pub fn dirty(&self) -> bool {
        self.workspace.dirty() || self.changes.lock().is_ok_and(|c| !c.is_empty())
    }
    pub fn discard(&mut self) {
        self.workspace.discard();
        if let Ok(mut changes) = self.changes.lock() {
            changes.clear();
        }
    }
    fn flush_changes(&mut self) -> Result<(), String> {
        let changes = std::mem::take(
            &mut *self
                .changes
                .lock()
                .map_err(|_| "Source edit queue is unavailable")?,
        );
        for (uri, (text, version)) in changes {
            self.workspace.change(&uri, text, version)?;
        }
        Ok(())
    }
}

pub fn open(editor: &mut Editor, path: &Path, line: Option<usize>) -> Result<(), String> {
    editor.script_editor.open = true;
    editor.script_editor.focus = true;
    editor.script_editor.web_focus = true;
    editor.script_editor.error = None;
    if path.is_file() {
        editor
            .script_editor
            .workspace
            .tree(&editor.root, &editor.assets.index)?;
        let uri = script_workspace::file_uri(&fs::canonicalize(path).map_err(|e| e.to_string())?)?;
        editor.script_editor.workspace.open(&editor.root, &uri)?;
        editor.script_editor.pending_open = Some((uri, line.unwrap_or(1).max(1)));
    }
    Ok(())
}
pub fn open_pocket(
    editor: &mut Editor,
    id: uuid::Uuid,
    name: Option<&str>,
    line: Option<usize>,
) -> Result<(), String> {
    let entries = editor
        .script_editor
        .workspace
        .tree(&editor.root, &editor.assets.index)?;
    let needle = format!("/{id}/");
    let entry = entries
        .iter()
        .find(|e| {
            e.uri.contains(&needle)
                && name.is_none_or(|name| {
                    e.uri.ends_with(&format!(
                        "/{}",
                        url::form_urlencoded::byte_serialize(name.as_bytes()).collect::<String>()
                    )) || script_workspace::uri_path(&e.uri)
                        .is_ok_and(|p| p.file_name().is_some_and(|f| f == name))
                })
        })
        .ok_or("PocketStation source is unavailable")?;
    editor
        .script_editor
        .workspace
        .open(&editor.root, &entry.uri)?;
    editor.script_editor.pending_open = Some((entry.uri.clone(), line.unwrap_or(1).max(1)));
    editor.script_editor.web_focus = true;
    Ok(())
}

pub fn draw(ui: &imgui::Ui, editor: &mut Editor) {
    let state = &mut editor.script_editor;
    state.region = None;
    if !state.open {
        return;
    }
    let mut opened = true;
    let focus = std::mem::take(&mut state.focus);
    let _padding = ui.push_style_var(imgui::StyleVar::WindowPadding([0., 0.]));
    ui.window("Code###CodeEditor").opened(&mut opened)
        .size([1000.,680.],imgui::Condition::FirstUseEver).focused(focus)
        .flags(imgui::WindowFlags::NO_SCROLLBAR|imgui::WindowFlags::NO_SCROLL_WITH_MOUSE)
        .build(||{
            if let Some(error)=&state.error {ui.text_wrapped(error);ui.text_wrapped("File buffers are retained. Reopen the Code window after repairing WebView2 / WebKit.");return;}
            let position=ui.cursor_screen_pos();let size=ui.content_region_avail();
            if size[0]>10. && size[1]>10. {state.region=Some([position[0],position[1],size[0],size[1]]);}
            state.region_owner=unsafe{(*(*imgui::sys::igGetCurrentWindow()).RootWindow).ID};
            ui.dummy(size);
        });
    state.open = opened;
}

fn refresh_pocket(editor: &mut Editor, id: uuid::Uuid) {
    editor.assets.index = crate::assets::scan(&editor.root, &mut Default::default());
    if editor.pocketstation.app == Some(id)
        && let Ok(record) = editor.assets.index.resolve(id)
        && let Ok(doc) = crate::pocketstation::document(record)
        && let Some(current) = &mut editor.pocketstation.doc
    {
        current.files = doc.files;
    }
    editor.assets.refresh();
}
fn save_one(editor: &mut Editor, state: &mut State, uri: &str) -> Result<(), String> {
    crate::pocketstation_ui::store(editor);
    if let Some(id) = state.workspace.save(uri)? {
        refresh_pocket(editor, id);
    }
    let doc = state
        .workspace
        .documents
        .get(uri)
        .ok_or("Source buffer is unavailable")?;
    state
        .outbox
        .push(json!({"event":"saved","uri":uri,"text":doc.text,"version":doc.version}));
    Ok(())
}
pub fn save_all(editor: &mut Editor) -> Result<(), String> {
    let mut state = std::mem::take(&mut editor.script_editor);
    let result = (|| {
        state.flush_changes()?;
        for uri in state
            .workspace
            .documents
            .keys()
            .cloned()
            .collect::<Vec<_>>()
        {
            save_one(editor, &mut state, &uri)?;
        }
        Ok(())
    })();
    editor.script_editor = state;
    result
}

pub fn tick(editor: &mut Editor) {
    if let Some((id, name, line)) = editor.script_editor.pocket_requested.take()
        && let Err(error) = open_pocket(editor, id, name.as_deref(), line)
    {
        editor.pocketstation.error = Some(error);
    }
    let mut state = std::mem::take(&mut editor.script_editor);
    if let Err(error) = state.flush_changes() {
        state.outbox.push(json!({"event":"error","message":error}));
    }
    if state.ready && state.registry_revision != editor.registry_revision {
        state.registry_revision = editor.registry_revision;
        if let Err(error) = prepare_workspaces(editor) {
            state.outbox.push(json!({"event":"error","message":error}));
        }
        if let Some(service) = &state.service {
            for (name, path) in [
                ("lua", editor.root.join(".epok/lua/epok.d.lua")),
                (
                    "cpp",
                    editor.root.join(".epok/code/cpp/compile_commands.json"),
                ),
            ] {
                if let Ok(uri) = script_workspace::file_uri(&path) {
                    let _=service.send(name,json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[{"uri":uri,"type":2}]}}));
                }
            }
        }
        state.outbox.push(json!({"event":"tree_changed"}));
    }
    if let Some(service) = &state.service {
        for event in service.events.try_iter().take(128) {
            state.outbox.push(event);
        }
    }
    for event in state.async_events.try_iter().take(64) {
        if event["event"] == "installed" {
            state.installing = false;
            if let Some(path) = event["path"].as_str() {
                let config = state.config.get_or_insert_with(script_tools::Config::load);
                config.set(event["tool"].as_str().unwrap_or_default(), path.into());
                if let Err(error) = config.save() {
                    state.outbox.push(json!({"event":"error","message":error}));
                }
            }
        }
        state.outbox.push(event);
    }
    for _ in 0..64 {
        let Ok(message) = state.incoming.try_recv() else {
            break;
        };
        let request = match serde_json::from_str::<Value>(&message) {
            Ok(value) => value,
            Err(error) => {
                editor.log(format!("Code editor bridge: {error}"));
                continue;
            }
        };
        let id = request.get("id").cloned();
        match handle(editor, &mut state, &request) {
            Ok(Some(result)) => {
                if let Some(id) = id {
                    state.outbox.push(json!({"id":id,"result":result}));
                }
            }
            Ok(None) => {}
            Err(error) => {
                if let Some(id) = id {
                    state.outbox.push(json!({"id":id,"error":error}));
                } else {
                    state.outbox.push(json!({"event":"error","message":error}));
                }
            }
        }
    }
    if state.ready && std::mem::take(&mut state.setup_requested) {
        state.outbox.push(json!({"event":"setup"}));
    }
    if state.ready
        && let Some((uri, line)) = state.pending_open.take()
    {
        state
            .outbox
            .push(json!({"event":"open","uri":uri,"line":line}));
    }
    editor.script_editor = state;
}
fn text<'a>(params: &'a Value, key: &str) -> Result<&'a str, String> {
    params[key].as_str().ok_or_else(|| format!("Missing {key}"))
}
fn handle(
    editor: &mut Editor,
    state: &mut State,
    request: &Value,
) -> Result<Option<Value>, String> {
    let params = &request["params"];
    let result = match request["method"]
        .as_str()
        .ok_or("Bridge method is missing")?
    {
        "close_application" => {
            if state.dirty() || editor.has_unsaved_changes() {
                return Err("Save or discard changes before closing the application".into());
            }
            editor.should_close = true;
            json!(true)
        }
        "run" => {
            let uri = text(params, "uri")?;
            let pocket = state.workspace.pocket_app(uri);
            for uri in state
                .workspace
                .documents
                .keys()
                .cloned()
                .collect::<Vec<_>>()
            {
                save_one(editor, state, &uri)?;
            }
            if let Some(id) = pocket {
                if editor.pocketstation.app != Some(id) {
                    crate::pocketstation_ui::open_app(editor, id);
                }
                crate::pocketstation_ui::build_and_run(editor);
                json!({"message":editor.pocketstation.status,"error":editor.pocketstation.error})
            } else {
                editor.action("play");
                json!({"message":"Play requested"})
            }
        }
        "new_class" => {
            let provider = text(params, "provider")?;
            match provider {
                "lua" => editor.action("new-lua"),
                "cpp" => editor.action("new-script"),
                _ => return Err("Choose a Lua or C++ class".into()),
            }
            json!(true)
        }
        "new_pocket_assembly" => {
            crate::pocketstation_ui::store(editor);
            let (id, uri) = state
                .workspace
                .add_pocket_assembly(&editor.root, text(params, "uri")?)?;
            refresh_pocket(editor, id);
            let tree = state.workspace.tree(&editor.root, &editor.assets.index)?;
            json!({"document":state.workspace.documents[&uri],"tree":tree})
        }
        "open_external" => {
            let doc = state
                .workspace
                .documents
                .get(text(params, "uri")?)
                .ok_or("Select an open source file")?;
            if doc.path.starts_with("PocketStation/") {
                return Err(
                    "PocketStation source is embedded in its asset; edit it in Monaco.".into(),
                );
            }
            editor.open_external_code(
                &script_workspace::uri_path(&doc.uri)?,
                Some(params["line"].as_u64().unwrap_or(1) as usize),
            );
            json!(true)
        }
        "init" => {
            state.ready = true;
            let config = state.config.get_or_insert_with(script_tools::Config::load);
            config.detect();
            prepare_workspaces(editor)?;
            let tree = state.workspace.tree(&editor.root, &editor.assets.index)?;
            state.ready = true;
            json!({"project":editor.project_name(),"root":script_workspace::file_uri(&editor.root)?,"rootPath":crate::code_editor::editor_path(&editor.root),"pocketRoot":script_workspace::file_uri(&editor.root.join(".epok/code/pocket"))?,"luaLibrary":crate::code_editor::editor_path(&editor.root.join(".epok/lua")),"tree":tree,"documents":state.workspace.documents.values().collect::<Vec<_>>(),"config":config,"version":env!("CARGO_PKG_VERSION")})
        }
        "tree" => serde_json::to_value(state.workspace.tree(&editor.root, &editor.assets.index)?)
            .map_err(|e| e.to_string())?,
        "open" => serde_json::to_value(state.workspace.open(&editor.root, text(params, "uri")?)?)
            .map_err(|e| e.to_string())?,
        "change" => {
            state.workspace.change(
                text(params, "uri")?,
                text(params, "text")?.into(),
                params["version"].as_u64().ok_or("Missing buffer version")?,
            )?;
            return Ok(None);
        }
        "save" => {
            save_one(editor, state, text(params, "uri")?)?;
            json!(true)
        }
        "save_all" => {
            for uri in state
                .workspace
                .documents
                .keys()
                .cloned()
                .collect::<Vec<_>>()
            {
                save_one(editor, state, &uri)?;
            }
            json!(true)
        }
        "close" => {
            state.workspace.close(
                text(params, "uri")?,
                params["discard"].as_bool().unwrap_or(false),
            )?;
            json!(true)
        }
        "config" => json!(state.config.get_or_insert_with(script_tools::Config::load)),
        "set_config" => {
            let config: script_tools::Config =
                serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
            config.save()?;
            state.config = Some(config);
            state.service = None;
            json!(true)
        }
        "detect" => {
            let config = state.config.get_or_insert_with(script_tools::Config::load);
            config.detect();
            config.save()?;
            json!(config)
        }
        "install" => {
            if state.installing {
                return Err("A language tool is already being installed".into());
            }
            let name = text(params, "tool")?.to_string();
            if !["lua_ls", "clangd", "copilot"].contains(&name.as_str()) {
                return Err("Unknown tool".into());
            }
            state.installing = true;
            let events = state.async_sender.clone();
            std::thread::spawn(move || {
                let result = script_tools::install(&name, |message| {
                    let _ = events.send(json!({"event":"install_progress","message":message}));
                });
                let event = match result {
                    Ok(path) => json!({"event":"installed","tool":name,"path":path}),
                    Err(error) => json!({"event":"installed","tool":name,"error":error}),
                };
                let _ = events.send(event);
            });
            json!(true)
        }
        "server_start" => {
            let name = text(params, "server")?;
            prepare_workspaces(editor)?;
            let spec = state
                .config
                .get_or_insert_with(script_tools::Config::load)
                .spec(name, &editor.root)?;
            state.service.get_or_insert_with(Default::default).start(
                request["id"].clone(),
                name,
                spec,
            )?;
            return Ok(None);
        }
        "server_send" => {
            let name = text(params, "server")?;
            if !["lua", "pocket", "cpp", "copilot", "copilot_chat"].contains(&name) {
                return Err("Unknown language service".into());
            }
            state
                .service
                .as_ref()
                .ok_or("Language services have not started")?
                .send(name, params["message"].clone())?;
            return Ok(None);
        }
        "open_url" => {
            open_auth_url(text(params, "url")?)?;
            json!(true)
        }
        _ => return Err("Unknown code editor operation".into()),
    };
    Ok(Some(result))
}

fn prepare_workspaces(editor: &Editor) -> Result<(), String> {
    crate::lua_api_stub::write(&editor.root, &editor.class_registry)?;
    let pocket = editor.root.join(".epok/code/pocket");
    crate::project::write_changed(
        &pocket.join("api/pocket.d.lua"),
        epok_pocketstation::plua::codegen::language_definitions().as_bytes(),
    )?;
    let config = json!({"runtime.version":"Lua 5.4","files.associations":{"*.plua":"lua"},"workspace.library":["api"],"workspace.checkThirdParty":false,"diagnostics.disable":["lowercase-global"]});
    crate::project::write_changed(
        &pocket.join(".luarc.json"),
        serde_json::to_string_pretty(&config)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    let request_path = editor.root.join(".epok/reflection/Reflection.epokrequest");
    let arguments: Vec<String> = fs::read(request_path)
        .ok()
        .and_then(|bytes| {
            crate::document::from_slice::<crate::reflection_schema::Request>(&bytes).ok()
        })
        .map(|r| r.arguments)
        .unwrap_or_else(|| {
            vec![
                "-x".into(),
                "c++".into(),
                "-std=c++20".into(),
                "-DEPOK_REFLECTION".into(),
                format!(
                    "-I{}",
                    crate::code_editor::editor_path(
                        &crate::workspace::editor_home().join("runtime")
                    )
                    .display()
                ),
                format!(
                    "-I{}",
                    crate::code_editor::editor_path(&editor.root.join("assets/scripts")).display()
                ),
            ]
        });
    let mut workspace = script_workspace::Workspace::default();
    let entries = workspace.tree(&editor.root, &editor.assets.index)?;
    let commands=entries.iter().filter(|e|e.language=="cpp").map(|e|{
        let path=script_workspace::uri_path(&e.uri)?;
        Ok(json!({"directory":crate::code_editor::editor_path(&editor.root),"file":path,"arguments":std::iter::once("clang++".into()).chain(arguments.clone()).chain([crate::code_editor::editor_path(&path).to_string_lossy().into_owned()]).collect::<Vec<_>>()}))
    }).collect::<Result<Vec<_>,String>>()?;
    crate::project::write_changed(
        &editor.root.join(".epok/code/cpp/compile_commands.json"),
        serde_json::to_vec_pretty(&commands)
            .map_err(|e| e.to_string())?
            .as_slice(),
    )?;
    Ok(())
}

fn open_auth_url(value: &str) -> Result<(), String> {
    let url = url::Url::parse(value).map_err(|e| e.to_string())?;
    if url.scheme() != "https"
        || url.host_str() != Some("github.com")
        || !(url.path().starts_with("/login/") || url.path().starts_with("/settings/copilot"))
    {
        return Err(
            "Only GitHub's HTTPS sign-in / Copilot settings pages may be opened here".into(),
        );
    }
    #[cfg(windows)]
    let mut command = {
        let mut c = std::process::Command::new("rundll32.exe");
        c.args(["url.dll,FileProtocolHandler", value]);
        c
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut c = std::process::Command::new("open");
        c.arg(value);
        c
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(value);
        c
    };
    crate::pipeline::quiet(&mut command);
    command.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

/// Main-thread-only native view. Project buffers survive hide/show and native
/// view creation errors. Navigations cannot turn the local bridge into a web API.
#[derive(Default)]
pub struct Native {
    view: Option<wry::WebView>,
    context: Option<wry::WebContext>,
    root: Option<PathBuf>,
    last_region: Option<[f32; 4]>,
    last_visible: Option<bool>,
}
impl Native {
    pub fn pump(&self) {
        #[cfg(target_os = "linux")]
        if self.context.is_some() {
            for _ in 0..64 {
                if !gtk::events_pending() {
                    break;
                }
                gtk::main_iteration_do(false);
            }
        }
    }

    pub fn sync(
        &mut self,
        window: &winit::window::Window,
        editor: Option<&mut Editor>,
        scale: [f32; 2],
        popup: bool,
    ) {
        let Some(editor) = editor else {
            self.view = None;
            self.context = None;
            self.root = None;
            self.last_region = None;
            self.last_visible = None;
            return;
        };
        if self.root.as_ref().is_some_and(|root| root != &editor.root) {
            self.view = None;
            self.context = None;
            self.last_region = None;
            self.last_visible = None;
        }
        let blocked = popup
            || editor.settings.project_open
            || editor.settings.preferences_open
            || editor.close_requested
            || editor.critical_busy();
        let state = &mut editor.script_editor;
        if self.view.is_none() && state.region.is_some() {
            #[cfg(target_os = "linux")]
            if let Err(error) = gtk::init() {
                state.error = Some(format!("Could not initialize GTK for Monaco: {error}"));
                return;
            }
            let input = state.input.clone();
            let changes = state.changes.clone();
            let bridge_errors = state.async_sender.clone();
            let context_path = crate::workspace::user_data().join("CodeWebView");
            self.context = Some(wry::WebContext::new(Some(context_path)));
            let result = wry::WebViewBuilder::new_with_web_context(self.context.as_mut().unwrap())
                .with_custom_protocol("epok-code".into(), |_, request| {
                    asset_response(request.uri().path())
                })
                .with_ipc_handler(move |request| {
                    if request.body().len() <= script_workspace::MAX_SOURCE_BYTES*6+65536 {
                        let body=request.into_body();
                        if let Ok(value)=serde_json::from_str::<Value>(&body) && value["method"]=="change" {
                            let params=&value["params"];
                            if let (Some(uri), Some(text), Some(version)) =
                                (params["uri"].as_str(), params["text"].as_str(), params["version"].as_u64())
                                && text.len() <= script_workspace::MAX_SOURCE_BYTES
                                && let Ok(mut changes) = changes.lock()
                            {
                                changes.insert(uri.into(), (text.into(), version));
                            }
                        }else if let Err(error)=input.try_send(body){
                            let body=match error{mpsc::TrySendError::Full(body)|mpsc::TrySendError::Disconnected(body)=>body};let id=serde_json::from_str::<Value>(&body).ok().and_then(|v|v.get("id").cloned());
                            let _=bridge_errors.try_send(json!({"id":id,"error":"The editor is busy. Retry this operation."}));
                        }
                    }
                })
                .with_navigation_handler(|url| {
                    url.starts_with("epok-code://localhost/")
                        || url.starts_with("http://epok-code.localhost/")
                })
                .with_background_color((30, 30, 30, 255))
                .with_visible(false)
                .with_url("epok-code://localhost/index.html")
                .build_as_child(window);
            match result {
                Ok(view) => {
                    self.view = Some(view);
                    self.root = Some(editor.root.clone());
                    state.ready = false;
                    self.last_visible = None;
                }
                Err(error) => {
                    state.error = Some(format!("Could not create Monaco's WebView: {error}"));
                    return;
                }
            }
        }
        if let Some(view) = &self.view {
            let visible = state
                .region
                .is_some_and(|region| unobstructed(state.region_owner, region))
                && !blocked;
            if self.last_visible != Some(visible) {
                if std::env::var_os("EPOK_CODE_VIEW_DEBUG").is_some() {
                    eprintln!(
                        "Code view visible={visible}, blocked={blocked}, region={:?}, owner={}",
                        state.region, state.region_owner
                    );
                }
                if self.last_visible == Some(true) && !visible {
                    let _ = view.focus_parent();
                }
                let _ = view.set_visible(visible);
                self.last_visible = Some(visible);
            }
            if visible && state.ready && state.web_focus {
                let _ = view.focus();
                state.web_focus = false;
            }
            if let Some(region) = state.region.map(|r| {
                [
                    r[0] * scale[0],
                    r[1] * scale[1],
                    r[2] * scale[0],
                    r[3] * scale[1],
                ]
            }) && self.last_region != Some(region)
            {
                let _ = view.set_bounds(wry::Rect {
                    position: wry::dpi::PhysicalPosition::new(region[0] as i32, region[1] as i32)
                        .into(),
                    size: wry::dpi::PhysicalSize::new(region[2] as u32, region[3] as u32).into(),
                });
                self.last_region = Some(region);
            }
            if state.ready {
                for event in state.outbox.drain(..) {
                    if let Ok(json) = serde_json::to_string(&event) {
                        let _ = view.evaluate_script(&format!("window.epokReceive?.({json})"));
                    }
                }
            }
        }
    }
}
fn unobstructed(owner: u32, region: [f32; 4]) -> bool {
    // Native child views cannot be clipped by ImGui draw commands. Hide the
    // view while another floating ImGui window covers it, preserving stacking.
    unsafe {
        let context = imgui::sys::igGetCurrentContext();
        if context.is_null() {
            return false;
        }
        let windows = &(*context).Windows;
        let pointers = std::slice::from_raw_parts(windows.Data, windows.Size as usize);
        let owner_dock_root = pointers
            .iter()
            .find(|pointer| (***pointer).ID == owner)
            .map(|pointer| (**pointer).RootWindowDockTree);
        let mut seen = false;
        for pointer in pointers {
            let window = &**pointer;
            if window.ID == owner {
                seen = true;
                continue;
            }
            if !seen
                || !window.Active
                || window.Hidden
                || window.Collapsed
                || window.IsFallbackWindow
                || (*window.RootWindow).ID == owner
                || window.RootWindow != *pointer
                || owner_dock_root == Some(*pointer)
            {
                continue;
            }
            let rect = window.OuterRectClipped;
            if rect.Min.x < region[0] + region[2]
                && rect.Max.x > region[0]
                && rect.Min.y < region[1] + region[3]
                && rect.Max.y > region[1]
            {
                if std::env::var_os("EPOK_CODE_VIEW_DEBUG").is_some() {
                    static LAST: std::sync::atomic::AtomicU32 =
                        std::sync::atomic::AtomicU32::new(0);
                    if LAST.swap(window.ID, std::sync::atomic::Ordering::Relaxed) != window.ID {
                        eprintln!(
                            "Code view occluded by {:?} ({})",
                            std::ffi::CStr::from_ptr(window.Name),
                            window.ID
                        );
                    }
                }
                return false;
            }
        }
        seen
    }
}

fn asset_response(path: &str) -> wry::http::Response<Cow<'static, [u8]>> {
    let file = WEB.get_file(path.trim_start_matches('/'));
    let mime = if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "text/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else {
        "text/plain"
    };
    wry::http::Response::builder().status(if file.is_some(){200}else{404}).header("Content-Type",mime)
        .header("Content-Security-Policy","default-src 'self'; script-src 'self'; worker-src 'self' blob:; style-src 'self' 'unsafe-inline'; font-src 'self' data:; img-src 'self' data:; connect-src 'none'; object-src 'none'; base-uri 'none'; frame-src 'none'")
        .body(Cow::Borrowed(file.map_or(b"Not found".as_slice(),|f|f.contents()))).unwrap()
}
