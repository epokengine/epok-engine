//! Build-only PlayStation executable packages for PCDrv-compatible loaders.
//! The ZIP holds the certified executable and the host files it opens through
//! PCDrv, selected from build metadata; it is never a copy of `.epok/build`.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub const EXECUTABLE: &str = "epok.ps-exe";
pub const MANIFEST: &str = "epok-package.json";
pub const README: &str = "README.txt";
const FORMAT: &str = "epok-ps-exe-package";
const FORMAT_VERSION: u32 = 1;
const DEBUG_SYMBOLS: [&str; 2] = ["epok.elf", "epok.map"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub content: crate::play::Content,
    pub selected_scenes: Vec<String>,
    pub initial_scene: Option<String>,
    pub destination: PathBuf,
    pub debug_symbols: bool,
    /// Compile the Epok debug overlay in, hidden until Select toggles it.
    pub debug_overlay: bool,
}
impl Options {
    /// Whole Game into `exports/<Project>-ps-exe.zip`, without debug symbols.
    pub fn for_project(root: &Path) -> Result<Self, String> {
        let saved = crate::play::Profile::load(root)?;
        Ok(Self {
            content: crate::play::Content::WholeGame,
            selected_scenes: saved.selected_scenes,
            initial_scene: saved.initial_scene,
            destination: crate::code_editor::editor_path(&default_destination(
                root,
                &crate::workspace::read_manifest(root)?.name,
            )),
            debug_symbols: false,
            debug_overlay: false,
        })
    }
    /// The hardware build used for every package. It lives only in memory:
    /// the project's saved Play profile and serial preferences are untouched.
    pub fn profile(&self) -> crate::play::Profile {
        let mut profile = crate::play::Profile {
            runtime: crate::play::Runtime::PlayStation,
            // Serial keeps the loader's resident handler installed after PsyQo
            // initialization, which PCDrv on a physical console relies on.
            target: crate::play::Target::Serial,
            content: self.content,
            data: crate::play::DataSource::Host,
            selected_scenes: self.selected_scenes.clone(),
            initial_scene: self.initial_scene.clone(),
            analog_controller: false,
        };
        profile.normalize();
        profile
    }
    pub fn validate(&self, root: &Path) -> Result<(), String> {
        let name = self
            .destination
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !name.to_ascii_lowercase().ends_with(".zip") || name.len() <= 4 {
            return Err("Choose a destination file ending in .zip.".into());
        }
        if self.destination.is_dir() {
            return Err(format!(
                "{} is a folder. Choose a .zip file name.",
                self.destination.display()
            ));
        }
        let builds = absolute(&root.join(".epok"));
        if absolute(&self.destination).starts_with(&builds) {
            return Err("Choose a destination outside the project's .epok folder; builds there are replaced without notice.".into());
        }
        self.profile().validate_build(root)
    }
    /// Command-line form of the dialog: `--output`, `--content`, `--scene`,
    /// repeated `--selected-scene`, `--initial-scene`, `--debug-symbols` and
    /// `--debug-overlay`.
    pub fn from_args(root: &Path, args: &[String]) -> Result<Self, String> {
        let value = |flag: &str| {
            args.windows(2)
                .find(|pair| pair[0] == flag)
                .map(|pair| pair[1].clone())
        };
        let mut options = Self::for_project(root)?;
        if let Some(content) = value("--content") {
            options.content = match content.as_str() {
                "whole-game" => crate::play::Content::WholeGame,
                "current-scene" => crate::play::Content::CurrentScene,
                "selected-scenes" => crate::play::Content::SelectedScenes,
                other => {
                    return Err(format!(
                        "Unknown --content {other}. Use whole-game, current-scene or selected-scenes."
                    ));
                }
            };
        }
        let selected = args
            .windows(2)
            .filter(|pair| pair[0] == "--selected-scene")
            .map(|pair| pair[1].replace('\\', "/"))
            .collect::<Vec<_>>();
        if !selected.is_empty() {
            options.content = crate::play::Content::SelectedScenes;
            options.selected_scenes = selected;
            options.initial_scene = None;
        }
        if let Some(initial) = value("--initial-scene") {
            let initial = initial.replace('\\', "/");
            if !options.selected_scenes.contains(&initial) {
                return Err(format!(
                    "--initial-scene {initial} must also be one of the selected scenes."
                ));
            }
            options.initial_scene = Some(initial);
        }
        if let Some(output) = value("--output") {
            options.destination = PathBuf::from(output);
        }
        options.debug_symbols = args.iter().any(|a| a == "--debug-symbols");
        options.debug_overlay = args.iter().any(|a| a == "--debug-overlay");
        Ok(options)
    }
}

/// `<root>/exports/<Project>-ps-exe.zip`, with the name reduced to portable characters.
pub fn default_destination(root: &Path, project: &str) -> PathBuf {
    let stem = project
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ' ' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let stem = stem.trim_matches([' ', '.']);
    root.join("exports").join(format!(
        "{}-ps-exe.zip",
        if stem.is_empty() { "Epok" } else { stem }
    ))
}

fn absolute(path: &Path) -> PathBuf {
    // The destination may not exist yet; canonicalize its nearest existing parent.
    let mut existing = path;
    let mut tail = Vec::new();
    while !existing.exists() {
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                tail.push(name.to_owned());
                existing = parent;
            }
            _ => return path.to_owned(),
        }
    }
    let mut resolved = fs::canonicalize(existing).unwrap_or_else(|_| existing.to_owned());
    resolved.extend(tail.into_iter().rev());
    resolved
}

fn relative(root: &Path, path: &Path) -> String {
    let resolved = fs::canonicalize(path).unwrap_or_else(|_| path.to_owned());
    let base = fs::canonicalize(root).unwrap_or_else(|_| root.to_owned());
    resolved
        .strip_prefix(&base)
        .or_else(|_| path.strip_prefix(root))
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Everything the worker needs, resolved on the caller's thread from saved files.
pub struct Request {
    pub input: crate::scene_dependencies::Input,
    pub options: Options,
    pub profile: crate::play::Profile,
    pub project: String,
    pub initial_scene: String,
    pub scenes: Vec<String>,
}
impl Request {
    /// The editor and the command line both resolve saved project files here,
    /// so an equivalent export produces the same runtime payload.
    pub fn new(root: &Path, current: &Path, options: Options) -> Result<Self, String> {
        options.validate(root)?;
        let profile = options.profile();
        let mut input = crate::play::saved_input(root, current, profile.clone())?;
        input.reject_disc_media = true;
        input.debug_overlay = options.debug_overlay;
        let initial = match profile.content {
            crate::play::Content::CurrentScene => current.to_owned(),
            crate::play::Content::WholeGame => crate::workspace::startup_scene(root)?,
            crate::play::Content::SelectedScenes => root.join(
                profile
                    .initial_scene
                    .as_ref()
                    .ok_or("Selected Scenes has no initial scene.")?,
            ),
        };
        let initial_scene = relative(root, &initial);
        let mut scenes = match profile.content {
            crate::play::Content::CurrentScene => vec![initial_scene.clone()],
            crate::play::Content::SelectedScenes => profile.selected_scenes.clone(),
            crate::play::Content::WholeGame => {
                let mut scenes = crate::scene_bank::read(root)?.scenes;
                scenes.push(initial_scene.clone());
                scenes.extend(crate::scene_bank::available(root)?);
                scenes
            }
        };
        scenes.sort_by_key(|path| path.to_ascii_lowercase());
        scenes.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        Ok(Self {
            input,
            options,
            profile,
            project: crate::workspace::read_manifest(root)?.name,
            initial_scene,
            scenes,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Role {
    Executable,
    Pcdrv,
    Debug,
}
#[derive(Clone, Debug, Serialize)]
struct FileRecord {
    path: String,
    bytes: u64,
    sha256: String,
    role: Role,
}
#[derive(Serialize)]
struct Display {
    width: u32,
    height: u32,
    interlaced: bool,
    video_mode: &'static str,
}
#[derive(Serialize)]
struct Loader {
    format: &'static str,
    pcdrv_required: bool,
    pcdrv_files: Vec<String>,
    pcdrv_access: &'static str,
    resident_handler_preserved: bool,
    cd_required: bool,
}
/// The Epok debug overlay compiled into the program, if any.
#[derive(Clone, Serialize)]
struct DebugOverlay {
    panels: Vec<&'static str>,
    /// Shows or hides the overlay; `None` when it is always visible.
    toggle_button: Option<&'static str>,
    initially_visible: bool,
}
#[derive(Serialize)]
struct Manifest<'a> {
    format: &'static str,
    format_version: u32,
    project: &'a str,
    epok_version: &'static str,
    executable: &'static str,
    pcdrv_root: &'static str,
    content: crate::play::Content,
    initial_scene: &'a str,
    scenes: &'a [String],
    lua_execution: &'static str,
    display: Display,
    debug_overlay: Option<DebugOverlay>,
    loader: Loader,
    files: &'a [FileRecord],
    /// Content knowingly left out of this package. Version 1 never omits any.
    omitted_content: Vec<String>,
}

/// One file entering the archive and the hash it must have.
struct Planned {
    path: String,
    source: PathBuf,
    expected: String,
    role: Role,
}

/// Read a staged header the build certified, rejecting any later change.
fn certified_text(built: &crate::pipeline::Built, name: &str) -> Result<String, String> {
    let expected = built
        .certified
        .get(name)
        .ok_or_else(|| format!("The certified build does not include {name}."))?;
    let bytes = fs::read(built.build.join(name)).map_err(|e| format!("{name}: {e}"))?;
    if crate::assets::hash(&bytes) != *expected {
        return Err(format!(
            "{name} changed after the build was certified. Export again."
        ));
    }
    String::from_utf8(bytes).map_err(|e| format!("{name}: {e}"))
}
fn define(header: &str, name: &str) -> Option<i64> {
    header.lines().find_map(|line| {
        let mut words = line.split_whitespace();
        (words.next() == Some("#define") && words.next() == Some(name))
            .then(|| words.next()?.parse().ok())
            .flatten()
    })
}
fn constant(header: &str, name: &str) -> Option<String> {
    let start = header.find(&format!(" {name} = "))? + name.len() + 4;
    let end = header[start..].find(';')? + start;
    Some(header[start..end].trim().to_owned())
}

/// Lua's MIT notice as carried by the pinned psxlua headers, for VM builds.
fn lua_notice(header: &str) -> Option<String> {
    let lines = header.lines().collect::<Vec<_>>();
    // The notice is the trailing comment block, not the LUA_COPYRIGHT macro.
    let start = lines
        .iter()
        .position(|l| l.trim_start().starts_with("* Copyright (C)"))?;
    let end = start
        + lines[start..]
            .iter()
            .position(|l| l.contains("SOFTWARE."))?;
    Some(
        lines[start..=end]
            .iter()
            .map(|line| {
                line.trim_start()
                    .trim_start_matches('*')
                    .strip_prefix(' ')
                    .unwrap_or(line.trim_start().trim_start_matches('*'))
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
}

/// The runtime opens host files by these exact names from the PCDrv root.
fn pcdrv_name(path: &str) -> Result<(), String> {
    let valid = !path.is_empty()
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
        && path
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || matches!(b, b'_' | b'.'));
    if !valid {
        return Err(format!(
            "Runtime file {path} does not have a portable PCDrv name."
        ));
    }
    if path != crate::streaming::ARCHIVE {
        return Err(format!(
            "Runtime file {path} cannot be served by PCDrv in this export version. Only streamed geometry ({}) is supported.",
            crate::streaming::ARCHIVE
        ));
    }
    Ok(())
}

fn plan(
    request: &Request,
    built: &crate::pipeline::Built,
) -> Result<(Vec<Planned>, Facts), String> {
    if built.summary.profile != request.profile || built.summary.debug {
        return Err("The completed build does not match the export request.".into());
    }
    if built.disc.is_some() {
        return Err("A PS-EXE package cannot contain a disc image.".into());
    }
    let recorded = |path: &str| {
        built
            .summary
            .output_hash(path)
            .map(str::to_owned)
            .ok_or_else(|| format!("The validated build did not record {path}."))
    };
    // The external file list is read from the snapshot the summary hashed.
    let inputs = fs::read(built.build.join("memory-inputs.json")).map_err(|e| e.to_string())?;
    if crate::assets::hash(&inputs) != recorded("memory-inputs.json")? {
        return Err("Build metadata changed after validation. Export again.".into());
    }
    let data = certified_text(built, "data-config.hh")?;
    if define(&data, "EPOK_HOST_DATA") != Some(1) || define(&data, "EPOK_SERIAL_DEBUG") != Some(1) {
        return Err("The build was not configured for PCDrv data with the resident loader handler. Export again.".into());
    }
    let display = certified_text(built, "display.hh")?;
    let dimension = |name| {
        constant(&display, name)
            .and_then(|value| value.parse().ok())
            .ok_or_else(|| format!("display.hh does not define {name}."))
    };
    let lua_mode = define(&certified_text(built, "lua-config.hh")?, "EPOK_LUA_MODE")
        .ok_or("lua-config.hh does not define EPOK_LUA_MODE.")?;
    let debug = certified_text(built, "debug-hud.hh")?;
    let toggle = define(&debug, "EPOK_DEBUG_TOGGLE") == Some(1);
    if toggle != request.options.debug_overlay {
        return Err(
            "The build's debug overlay does not match the export request. Export again.".into(),
        );
    }
    let panels = [
        ("EPOK_DEBUG_FPS", "fps"),
        ("EPOK_DEBUG_CPU", "cpu"),
        ("EPOK_DEBUG_GTE", "gte"),
        ("EPOK_DEBUG_GPU", "gpu"),
        ("EPOK_DEBUG_SPU", "spu"),
    ]
    .into_iter()
    .filter(|(name, _)| define(&debug, name) == Some(1))
    .map(|(_, panel)| panel)
    .collect::<Vec<_>>();
    let facts = Facts {
        width: dimension("display_width")?,
        height: dimension("display_height")?,
        interlaced: constant(&display, "display_interlaced").as_deref() == Some("true"),
        lua_mode,
        overlay: (!panels.is_empty()).then(|| DebugOverlay {
            panels,
            toggle_button: toggle.then_some("select"),
            initially_visible: !toggle,
        }),
    };
    let mut files = vec![Planned {
        path: EXECUTABLE.into(),
        source: built.exe.clone(),
        expected: recorded(EXECUTABLE)?,
        role: Role::Executable,
    }];
    for path in crate::memory::external_file_paths(&built.build)? {
        pcdrv_name(&path)?;
        let expected = recorded(&path)?;
        if built.certified.get(&path) != Some(&expected) {
            return Err(format!(
                "{path} does not match the file certified with the executable. Export again."
            ));
        }
        files.push(Planned {
            source: built.build.join(&path),
            path,
            expected,
            role: Role::Pcdrv,
        });
    }
    if request.options.debug_symbols {
        for name in DEBUG_SYMBOLS {
            files.push(Planned {
                path: format!("debug/{name}"),
                source: built.build.join(name),
                expected: recorded(name)?,
                role: Role::Debug,
            });
        }
    }
    Ok((files, facts))
}

struct Facts {
    width: u32,
    height: u32,
    interlaced: bool,
    lua_mode: i64,
    overlay: Option<DebugOverlay>,
}

fn readme(request: &Request, facts: &Facts, files: &[FileRecord]) -> String {
    let size = crate::memory_ui::size;
    let pcdrv = files
        .iter()
        .filter(|f| f.role == Role::Pcdrv)
        .collect::<Vec<_>>();
    let mut text = format!(
        "{} - PlayStation executable package\r\nCreated with Epok Engine {}.\r\n\r\n",
        request.project,
        env!("CARGO_PKG_VERSION")
    );
    text.push_str("CONTENTS\r\n\r\n");
    for file in files {
        let purpose = match file.role {
            Role::Executable => "PS-X EXE program",
            Role::Pcdrv => "Streamed geometry, read through PCDrv while the game runs",
            Role::Debug => "Debug symbols; not needed to run the game",
        };
        text.push_str(&format!(
            "  {:<20} {purpose} ({})\r\n  {:<20} SHA-256 {}\r\n",
            file.path,
            size(file.bytes),
            "",
            file.sha256
        ));
    }
    text.push_str(&format!(
        "  {MANIFEST:<20} Machine-readable description of this package\r\n  \
         {:<20} Epok runtime license\r\n  \
         THIRD_PARTY_NOTICES.md\r\n  \
         {:<20} Notices for the SDK components linked into the program\r\n  \
         {:<20} License texts referred to by those notices\r\n\r\n",
        "LICENSE", "", "licenses/"
    ));
    text.push_str(&format!(
        "Startup scene: {}\r\nDisplay: {} x {} {}, NTSC timing\r\n\r\n",
        request.initial_scene,
        facts.width,
        facts.height,
        if facts.interlaced {
            "interlaced"
        } else {
            "progressive"
        }
    ));
    text.push_str("HOW TO RUN\r\n\r\n");
    if pcdrv.is_empty() {
        text.push_str(
            "This program is fully resident: it reads no files through PCDrv at run time.\r\n\r\n\
             1. Extract this ZIP.\r\n\
             2. Upload and run epok.ps-exe with a PS-X EXE loader or a compatible emulator.\r\n\
             3. No CD needs to be mounted.\r\n\r\n",
        );
    } else {
        let names = pcdrv.iter().map(|f| f.path.as_str()).collect::<Vec<_>>();
        text.push_str(&format!(
            "This program reads {} through PCDrv while it runs.\r\n\r\n\
             1. Extract this ZIP into an empty folder.\r\n\
             2. Set the PCDrv server's root folder to that extracted folder. The game opens\r\n   \
             {} by exact name from the root; keep it beside epok.ps-exe and do not\r\n   \
             rename it.\r\n\
             3. Upload and run epok.ps-exe with a loader that provides PCDrv host file access,\r\n   \
             such as Unirom's resident debug handler with a PCDrv-serving host.\r\n\
             4. Keep the PCDrv connection active for the whole session.\r\n\
             5. No CD needs to be mounted.\r\n\r\n",
            names.join(", "),
            names.join(", ")
        ));
    }
    text.push_str("Controls are those of the game itself. ");
    text.push_str(match &facts.overlay {
        Some(overlay) if overlay.toggle_button.is_some() => {
            "Press Select to show or hide the Epok\r\n\
             debug overlay (FPS, CPU, GTE, GPU and SPU meters, lower left); it starts hidden.\r\n\
             The game also receives each Select press.\r\n\r\n"
        }
        Some(_) => {
            "The project's Epok debug overlay is always\r\nvisible in the lower left.\r\n\r\n"
        }
        None => "The package adds no loader controls.\r\n\r\n",
    });
    text.push_str("NOTES\r\n\r\n");
    text.push_str(
        "- The program keeps the loader's resident interrupt handler installed after\r\n  \
         PsyQo initializes, as Unirom-style serial loaders require. If the BIOS\r\n  \
         interrupt queues are invalid at startup, it stops with an error message.\r\n",
    );
    if !pcdrv.is_empty() {
        text.push_str(
            "- PCDrv reads are synchronous: the game waits while a geometry page is read.\r\n  \
             Loading time depends on the host link and differs from CD playback.\r\n\
             - If the file cannot be opened, has the wrong size, or a page fails its\r\n  \
             integrity check, streaming stops for the session and the failure is\r\n  \
             counted in the streaming statistics. Pages are not retried.\r\n",
        );
    }
    text.push_str(
        "- XA music and video are not included in PS-EXE packages; they need the\r\n  \
         PlayStation CD decoder. This package contains no omitted content.\r\n\
         - Compatibility with a particular loader or remote console service is confirmed\r\n  \
         only by running this package there.\r\n\r\n",
    );
    text.push_str(
        "LICENSES\r\n\r\n\
         LICENSE and THIRD_PARTY_NOTICES.md cover the Epok runtime and the SDK components\r\n\
         linked into epok.ps-exe; the texts they refer to are in licenses/. They do not\r\n\
         license the game's own scripts or assets. Keep these notices with every copy.\r\n",
    );
    if facts.lua_mode != 0 {
        text.push_str("This build links the Lua VM; its notice is in licenses/Lua.txt.\r\n");
    }
    text
}

/// Package a build this worker just validated. The archive is written beside
/// its destination and replaces it only after its contents are verified.
pub fn package(
    request: &Request,
    built: &crate::pipeline::Built,
    log: &mut dyn FnMut(String),
    cancelled: &dyn Fn() -> bool,
) -> Result<PathBuf, String> {
    let (planned, facts) = plan(request, built)?;
    let destination = absolute(&request.options.destination);
    let directory = destination
        .parent()
        .ok_or("The destination has no folder.")?
        .to_owned();
    fs::create_dir_all(&directory).map_err(|e| format!("{}: {e}", directory.display()))?;
    let name = destination
        .file_name()
        .ok_or("The destination has no file name.")?
        .to_string_lossy()
        .into_owned();
    let partial = directory.join(format!(".{name}.{}.partial", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut entries = Vec::new();
        let nugget = Path::new(&built.config.nugget);
        let records = write(
            &partial,
            request,
            &facts,
            &planned,
            nugget,
            &mut entries,
            log,
            cancelled,
        )?;
        log("Verifying the package...".into());
        verify(&partial, &entries, cancelled)?;
        if cancelled() {
            return Err("Export cancelled.".to_owned());
        }
        fs::rename(&partial, &destination)
            .map_err(|e| format!("Cannot replace {}: {e}", destination.display()))?;
        let runtime = records
            .iter()
            .filter(|f| f.role != Role::Debug)
            .map(|f| f.path.as_str())
            .collect::<Vec<_>>();
        log(format!("Runtime files: {}", runtime.join(", ")));
        Ok(crate::code_editor::editor_path(&destination))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&partial);
    }
    result
}

fn options() -> zip::write::SimpleFileOptions {
    zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644)
}

#[allow(clippy::too_many_arguments)]
fn write(
    partial: &Path,
    request: &Request,
    facts: &Facts,
    planned: &[Planned],
    nugget: &Path,
    entries: &mut Vec<(String, String)>,
    log: &mut dyn FnMut(String),
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<FileRecord>, String> {
    let file = fs::File::create_new(partial).map_err(|e| format!("{}: {e}", partial.display()))?;
    let mut zip = zip::ZipWriter::new(file);
    let mut records = Vec::new();
    for item in planned {
        if cancelled() {
            return Err("Export cancelled.".into());
        }
        log(format!("Adding {}", item.path));
        let mut source = fs::File::open(&item.source).map_err(|e| format!("{}: {e}", item.path))?;
        zip.start_file(item.path.as_str(), options())
            .map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes = 0u64;
        let mut header = Vec::new();
        loop {
            let n = source
                .read(&mut buffer)
                .map_err(|e| format!("{}: {e}", item.path))?;
            if n == 0 {
                break;
            }
            if header.len() < 8 {
                header.extend_from_slice(&buffer[..n.min(8 - header.len())]);
            }
            hasher.update(&buffer[..n]);
            zip.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
            bytes += n as u64;
            if cancelled() {
                return Err("Export cancelled.".into());
            }
        }
        let sha256 = format!("{:x}", hasher.finalize());
        if sha256 != item.expected {
            return Err(format!(
                "{} changed after the build was validated; the package was not published. Export again.",
                item.path
            ));
        }
        if item.role == Role::Executable && header != b"PS-X EXE" {
            return Err("The validated executable is not a PS-X EXE.".into());
        }
        entries.push((item.path.clone(), sha256.clone()));
        records.push(FileRecord {
            path: item.path.clone(),
            bytes,
            sha256,
            role: item.role,
        });
    }
    let pcdrv_files = records
        .iter()
        .filter(|f| f.role == Role::Pcdrv)
        .map(|f| f.path.clone())
        .collect::<Vec<_>>();
    let manifest = Manifest {
        format: FORMAT,
        format_version: FORMAT_VERSION,
        project: &request.project,
        epok_version: env!("CARGO_PKG_VERSION"),
        executable: EXECUTABLE,
        pcdrv_root: ".",
        content: request.profile.content,
        initial_scene: &request.initial_scene,
        scenes: &request.scenes,
        lua_execution: match facts.lua_mode {
            0 => "native_cpp",
            1 => "vm_bytecode",
            _ => "vm_source",
        },
        display: Display {
            width: facts.width,
            height: facts.height,
            interlaced: facts.interlaced,
            video_mode: "ntsc",
        },
        debug_overlay: facts.overlay.clone(),
        loader: Loader {
            format: "ps-x-exe",
            pcdrv_required: !pcdrv_files.is_empty(),
            pcdrv_files,
            pcdrv_access: "read-only, synchronous, relative to the PCDrv root",
            resident_handler_preserved: true,
            cd_required: false,
        },
        files: &records,
        omitted_content: Vec::new(),
    };
    let mut documents = vec![
        (
            MANIFEST.to_owned(),
            serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())? + "\n",
        ),
        (README.to_owned(), readme(request, facts, &records)),
    ];
    documents.extend(
        crate::export::RUNTIME_NOTICES
            .iter()
            .map(|(path, text)| ((*path).to_owned(), (*text).to_owned())),
    );
    if facts.lua_mode != 0 {
        let header = nugget.join("third_party/psxlua/src/lua.h");
        let notice = fs::read_to_string(&header)
            .ok()
            .as_deref()
            .and_then(lua_notice)
            .ok_or_else(|| {
                format!(
                    "Cannot read the Lua license notice from {}. Run setup to restore the SDK.",
                    header.display()
                )
            })?;
        documents.push(("licenses/Lua.txt".into(), notice));
    }
    for (path, text) in documents {
        zip.start_file(path.as_str(), options())
            .map_err(|e| e.to_string())?;
        zip.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        entries.push((path, crate::assets::hash(text.as_bytes())));
    }
    let file = zip.finish().map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    Ok(records)
}

/// Reopen the written archive: exactly the planned entries, each with the bytes hashed while writing.
fn verify(
    partial: &Path,
    entries: &[(String, String)],
    cancelled: &dyn Fn() -> bool,
) -> Result<(), String> {
    let file = fs::File::open(partial).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Package check: {e}"))?;
    if archive.len() != entries.len() {
        return Err("Package check: unexpected archive entries.".into());
    }
    for (index, (path, expected)) in entries.iter().enumerate() {
        if cancelled() {
            return Err("Export cancelled.".into());
        }
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("Package check: {e}"))?;
        if entry.name() != path
            || entry.is_dir()
            || Path::new(entry.name())
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(format!("Package check: unexpected entry {}.", entry.name()));
        }
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 64 * 1024];
        loop {
            let n = entry
                .read(&mut buffer)
                .map_err(|e| format!("Package check {path}: {e}"))?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        if format!("{:x}", hasher.finalize()) != *expected {
            return Err(format!("Package check: {path} does not match its source."));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    const NOTICES: [&str; 9] = [
        "LICENSE",
        "THIRD_PARTY_NOTICES.md",
        "licenses/nugget-MIT.txt",
        "licenses/EASTL.txt",
        "licenses/EASTL-third-party.txt",
        "licenses/EABase.txt",
        "licenses/psx-font.txt",
        "licenses/GCC-GPL-3.txt",
        "licenses/GCC-Runtime-Exception-3.1.txt",
    ];

    /// A validated build as the pipeline returns it, beside stale outputs of
    /// earlier configurations that must never be packaged.
    fn fixture(label: &str, geometry: bool) -> (PathBuf, Request, crate::pipeline::Built) {
        let root = crate::workspace::tests::temp(label);
        let build = root.join(".epok/build");
        fs::create_dir_all(build.join("music")).unwrap();
        fs::write(build.join(EXECUTABLE), b"PS-X EXE fixture program").unwrap();
        fs::write(build.join("epok.elf"), b"ELF fixture").unwrap();
        fs::write(build.join("epok.map"), b"map fixture").unwrap();
        for stale in ["epok.cue", "epok.bin", "music/M0000000.XA", "notes.txt"] {
            fs::write(build.join(stale), b"stale").unwrap();
        }
        let mut files = Vec::new();
        let mut certified = crate::staging_files::Files::new();
        if geometry {
            let pages = vec![7u8; 65536];
            fs::write(build.join("GEOMETRY.BIN"), &pages).unwrap();
            files.push(crate::memory::Node::leaf("GEOMETRY.BIN", 65536, "external"));
            certified.insert("GEOMETRY.BIN".into(), crate::assets::hash(&pages));
        } else {
            // A stale archive from an earlier streaming build is not listed.
            fs::write(build.join("GEOMETRY.BIN"), b"stale").unwrap();
        }
        fs::write(
            build.join("memory-inputs.json"),
            serde_json::to_vec(&serde_json::json!({"hints":[],"scenes":[],"spu":[],"files":files}))
                .unwrap(),
        )
        .unwrap();
        for (name, text) in [
            (
                "data-config.hh",
                "#pragma once\n#define EPOK_HOST_DATA 1\n#define EPOK_SERIAL_DEBUG 1\n",
            ),
            (
                "display.hh",
                "namespace epok {\ninline constexpr int display_width = 320;\ninline constexpr int display_height = 240;\ninline constexpr bool display_interlaced = false;\n}\n",
            ),
            ("lua-config.hh", "#pragma once\n#define EPOK_LUA_MODE 0\n"),
            ("debug-hud.hh", "#pragma once\n#define EPOK_DEBUG_FPS 0\n"),
        ] {
            fs::write(build.join(name), text).unwrap();
            certified.insert(name.into(), crate::assets::hash(text.as_bytes()));
        }
        let options = Options {
            content: crate::play::Content::WholeGame,
            selected_scenes: Vec::new(),
            initial_scene: None,
            destination: root.join("exports/Fixture-ps-exe.zip"),
            debug_symbols: false,
            debug_overlay: false,
        };
        let profile = options.profile();
        let summary = crate::build_report::Summary::capture(
            &build,
            profile.clone(),
            false,
            Some("request".into()),
            None,
            false,
        )
        .unwrap();
        let request = Request {
            input: crate::scene::Scene::default().into(),
            options,
            profile,
            project: "Fixture".into(),
            initial_scene: "assets/scenes/Main.epokmap".into(),
            scenes: vec!["assets/scenes/Main.epokmap".into()],
        };
        let built = crate::pipeline::Built {
            config: crate::project::Config::default(),
            exe: build.join(EXECUTABLE),
            build,
            disc: None,
            summary,
            certified,
        };
        (root, request, built)
    }
    fn export(request: &Request, built: &crate::pipeline::Built) -> Result<PathBuf, String> {
        package(request, built, &mut |_| {}, &|| false)
    }
    fn entries(path: &Path) -> BTreeSet<String> {
        let archive = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
        archive.file_names().map(str::to_owned).collect()
    }
    fn read(path: &Path, name: &str) -> Vec<u8> {
        let mut archive = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
        let mut bytes = Vec::new();
        archive
            .by_name(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    }

    #[test]
    fn streamed_package_holds_only_certified_runtime_files_at_the_pcdrv_root() {
        let (root, request, built) = fixture("ps-exe-streamed", true);
        let archive = export(&request, &built).unwrap();
        let mut expected = BTreeSet::from([
            EXECUTABLE.to_owned(),
            "GEOMETRY.BIN".to_owned(),
            MANIFEST.to_owned(),
            README.to_owned(),
        ]);
        expected.extend(NOTICES.map(str::to_owned));
        assert_eq!(entries(&archive), expected);
        assert_eq!(read(&archive, "GEOMETRY.BIN"), vec![7u8; 65536]);
        let manifest: serde_json::Value =
            serde_json::from_slice(&read(&archive, MANIFEST)).unwrap();
        assert_eq!(manifest["format"], FORMAT);
        assert_eq!(manifest["pcdrv_root"], ".");
        assert_eq!(manifest["content"], "whole_game");
        assert_eq!(manifest["loader"]["pcdrv_required"], true);
        assert_eq!(
            manifest["loader"]["pcdrv_files"],
            serde_json::json!(["GEOMETRY.BIN"])
        );
        assert_eq!(manifest["display"]["width"], 320);
        assert_eq!(manifest["omitted_content"], serde_json::json!([]));
        let geometry = &manifest["files"][1];
        assert_eq!(geometry["path"], "GEOMETRY.BIN");
        assert_eq!(geometry["role"], "pcdrv");
        assert_eq!(geometry["sha256"], built.certified["GEOMETRY.BIN"]);
        let readme = String::from_utf8(read(&archive, README)).unwrap();
        assert!(readme.contains("PCDrv server's root folder"));
        assert!(readme.contains("reads GEOMETRY.BIN through PCDrv"));
        assert!(!readme.contains("Lua.txt"));
        // Packaging an unchanged build again reproduces the archive exactly.
        let first = fs::read(&archive).unwrap();
        export(&request, &built).unwrap();
        assert_eq!(fs::read(&archive).unwrap(), first);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resident_package_needs_no_pcdrv_and_debug_symbols_are_separate() {
        let (root, mut request, built) = fixture("ps-exe-resident", false);
        request.options.debug_symbols = true;
        let archive = export(&request, &built).unwrap();
        let names = entries(&archive);
        assert!(!names.contains("GEOMETRY.BIN"), "Stale archives stay out");
        assert!(names.contains("debug/epok.elf") && names.contains("debug/epok.map"));
        assert!(
            !names
                .iter()
                .any(|n| n.ends_with(".cue") || n.ends_with(".XA"))
        );
        let manifest: serde_json::Value =
            serde_json::from_slice(&read(&archive, MANIFEST)).unwrap();
        assert_eq!(manifest["loader"]["pcdrv_required"], false);
        assert_eq!(manifest["files"][1]["role"], "debug");
        let readme = String::from_utf8(read(&archive, README)).unwrap();
        assert!(readme.contains("fully resident: it reads no files through PCDrv"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn debug_overlay_is_described_from_the_certified_build() {
        let (root, mut request, mut built) = fixture("ps-exe-overlay", true);
        let manifest = |archive: &Path| -> serde_json::Value {
            serde_json::from_slice(&read(archive, MANIFEST)).unwrap()
        };
        let archive = export(&request, &built).unwrap();
        assert!(manifest(&archive)["debug_overlay"].is_null());
        let readme = String::from_utf8(read(&archive, README)).unwrap();
        assert!(readme.contains("The package adds no loader controls."));
        let mut header = crate::settings::DebugHud::ALL.header();
        header.push_str("#define EPOK_DEBUG_TOGGLE 1\n");
        fs::write(built.build.join("debug-hud.hh"), &header).unwrap();
        built.certified.insert(
            "debug-hud.hh".into(),
            crate::assets::hash(header.as_bytes()),
        );
        // The request and the certified build must agree on the overlay.
        assert!(
            export(&request, &built)
                .unwrap_err()
                .contains("debug overlay")
        );
        request.options.debug_overlay = true;
        let archive = export(&request, &built).unwrap();
        assert_eq!(
            manifest(&archive)["debug_overlay"],
            serde_json::json!({"panels":["fps","cpu","gte","gpu","spu"],
                "toggle_button":"select","initially_visible":false})
        );
        let readme = String::from_utf8(read(&archive, README)).unwrap();
        assert!(readme.contains("Press Select to show or hide the Epok"));
        assert!(readme.contains("it starts hidden"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn changed_outputs_are_never_published_over_a_previous_package() {
        let (root, request, built) = fixture("ps-exe-changed", true);
        let archive = export(&request, &built).unwrap();
        let previous = fs::read(&archive).unwrap();
        let directory = archive.parent().unwrap().to_owned();
        let no_partials = || {
            fs::read_dir(&directory).unwrap().all(|entry| {
                !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".partial")
            })
        };
        fs::write(built.build.join("GEOMETRY.BIN"), vec![8u8; 65536]).unwrap();
        let error = export(&request, &built).unwrap_err();
        assert!(error.contains("GEOMETRY.BIN"), "{error}");
        assert_eq!(fs::read(&archive).unwrap(), previous);
        assert!(no_partials());
        fs::write(built.build.join("GEOMETRY.BIN"), vec![7u8; 65536]).unwrap();
        fs::write(&built.exe, b"PS-X EXE tampered program").unwrap();
        assert!(export(&request, &built).unwrap_err().contains(EXECUTABLE));
        assert_eq!(fs::read(&archive).unwrap(), previous);
        // A display header edited after certification is rejected as well.
        fs::write(&built.exe, b"PS-X EXE fixture program").unwrap();
        fs::write(built.build.join("display.hh"), "display_width = 640;").unwrap();
        assert!(export(&request, &built).unwrap_err().contains("display.hh"));
        assert!(no_partials());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cancellation_leaves_no_archive_and_preserves_the_previous_one() {
        let (root, request, built) = fixture("ps-exe-cancel", true);
        let calls = std::cell::Cell::new(0);
        let cancel_later = || {
            calls.set(calls.get() + 1);
            calls.get() > 2
        };
        let error = package(&request, &built, &mut |_| {}, &cancel_later).unwrap_err();
        assert_eq!(error, "Export cancelled.");
        assert!(!request.options.destination.exists());
        let archive = export(&request, &built).unwrap();
        let previous = fs::read(&archive).unwrap();
        calls.set(0);
        package(&request, &built, &mut |_| {}, &cancel_later).unwrap_err();
        assert_eq!(fs::read(&archive).unwrap(), previous);
        assert_eq!(fs::read_dir(archive.parent().unwrap()).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn builds_without_the_hardware_configuration_are_refused() {
        let (root, request, mut built) = fixture("ps-exe-profile", true);
        let mut other = request.profile.clone();
        other.target = crate::play::Target::Embedded;
        built.summary.profile = other;
        assert!(
            export(&request, &built)
                .unwrap_err()
                .contains("does not match")
        );
        built.summary.profile = request.profile.clone();
        let text = "#define EPOK_HOST_DATA 1\n#define EPOK_SERIAL_DEBUG 0\n";
        fs::write(built.build.join("data-config.hh"), text).unwrap();
        built.certified.insert(
            "data-config.hh".into(),
            crate::assets::hash(text.as_bytes()),
        );
        assert!(
            export(&request, &built)
                .unwrap_err()
                .contains("resident loader handler")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pcdrv_files_keep_the_exact_runtime_name() {
        assert!(pcdrv_name("GEOMETRY.BIN").is_ok());
        for name in [
            "geometry.bin",
            "../GEOMETRY.BIN",
            "music/M0000000.XA",
            "",
            "C:\\GEOMETRY.BIN",
        ] {
            assert!(pcdrv_name(name).is_err(), "{name}");
        }
    }

    #[test]
    fn options_use_an_in_memory_hardware_profile_and_cli_scope() {
        let root = crate::workspace::tests::temp("ps-exe-options");
        crate::workspace::create(&root, "Café Run!", crate::workspace::Template::Basic).unwrap();
        let saved = fs::read(crate::workspace::manifest_path(&root).unwrap()).unwrap();
        let options = Options::for_project(&root).unwrap();
        assert_eq!(
            options.destination,
            root.join("exports/Caf_ Run_-ps-exe.zip")
        );
        let profile = options.profile();
        assert_eq!(profile.runtime, crate::play::Runtime::PlayStation);
        assert_eq!(profile.target, crate::play::Target::Serial);
        assert_eq!(profile.data, crate::play::DataSource::Host);
        assert_eq!(profile.content, crate::play::Content::WholeGame);
        assert!(!profile.analog_controller);
        let args = |list: &[&str]| list.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let parsed = Options::from_args(
            &root,
            &args(&[
                "--selected-scene",
                "assets/scenes/A.epokmap",
                "--selected-scene",
                "assets/scenes/B.epokmap",
                "--initial-scene",
                "assets/scenes/B.epokmap",
                "--output",
                "out.zip",
                "--debug-symbols",
                "--debug-overlay",
            ]),
        )
        .unwrap();
        assert!(parsed.debug_overlay && !options.debug_overlay);
        assert_eq!(parsed.content, crate::play::Content::SelectedScenes);
        assert_eq!(parsed.selected_scenes.len(), 2);
        assert_eq!(
            parsed.profile().initial_scene.as_deref(),
            Some("assets/scenes/B.epokmap")
        );
        assert_eq!(parsed.destination, PathBuf::from("out.zip"));
        assert!(parsed.debug_symbols);
        assert!(Options::from_args(&root, &args(&["--content", "everything"])).is_err());
        assert!(
            Options::from_args(
                &root,
                &args(&["--initial-scene", "assets/scenes/C.epokmap"])
            )
            .is_err()
        );
        let mut invalid = options.clone();
        invalid.destination = root.join("package.tar");
        assert!(invalid.validate(&root).is_err());
        invalid.destination = root.join(".epok/build/package.zip");
        assert!(invalid.validate(&root).unwrap_err().contains(".epok"));
        assert!(options.validate(&root).is_ok());
        let startup = crate::workspace::startup_scene(&root).unwrap();
        let request = Request::new(&root, &startup, options).unwrap();
        assert!(request.input.reject_disc_media && !request.input.debug_overlay);
        assert_eq!(request.input.play.as_ref(), Some(&request.profile));
        assert!(request.scenes.contains(&request.initial_scene));
        assert_eq!(
            fs::read(crate::workspace::manifest_path(&root).unwrap()).unwrap(),
            saved,
            "The saved Play profile is not changed"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lua_notice_is_read_from_the_sdk_header() {
        let header = "#define LUA_COPYRIGHT LUA_RELEASE \"  Copyright (C) 1994-2015 Lua.org, PUC-Rio\"\n#endif\n/******************************************************************************\n* Copyright (C) 1994-2015 Lua.org, PUC-Rio.\n*\n* Permission is hereby granted, free of charge.\n*\n* THE SOFTWARE IS PROVIDED \"AS IS\". IN NO EVENT SHALL THE AUTHORS BE LIABLE\n* FOR ANY CLAIM ARISING FROM THE SOFTWARE.\n******************************************************************************/\n";
        let notice = lua_notice(header).unwrap();
        assert!(notice.starts_with("Copyright (C) 1994-2015 Lua.org, PUC-Rio.\n\nPermission"));
        assert!(notice.ends_with("FROM THE SOFTWARE.\n"));
        assert!(!notice.contains('*'));
        assert!(lua_notice("no notice").is_none());
        // Setup initializes the pinned psxlua headers that VM builds link.
        let pinned = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("third_party/nugget/third_party/psxlua/src/lua.h");
        if let Ok(header) = fs::read_to_string(pinned) {
            let notice = lua_notice(&header).unwrap();
            assert!(notice.starts_with("Copyright (C) 1994-2015 Lua.org, PUC-Rio."));
            assert!(notice.contains("Permission is hereby granted, free of charge"));
            assert!(notice.ends_with("OTHER DEALINGS IN THE SOFTWARE.\n"));
            assert!(!notice.contains("#define"), "Only the license comment");
        }
    }
}
