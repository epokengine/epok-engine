//! Shared dependency discovery and the editor's local tool configuration UI.
use crate::project::Config;
#[cfg(any(windows, target_os = "linux", target_os = "macos"))]
use std::process::Stdio;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command},
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tool {
    Make,
    Compiler,
    Sdk,
    Emulator,
    Audio,
    Disc,
    Clang,
}
const TOOLS: [Tool; 7] = [
    Tool::Make,
    Tool::Compiler,
    Tool::Sdk,
    Tool::Emulator,
    Tool::Audio,
    Tool::Disc,
    Tool::Clang,
];
#[cfg(not(target_os = "macos"))]
fn stage_bundled(config: &mut Config, home: &Path) {
    for tool in TOOLS {
        *tool.value(config) = tool.bundled(home);
    }
}

fn log_tail(path: &Path) -> String {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = fs::File::open(path) else {
        return String::new();
    };
    let size = file.metadata().map_or(0, |m| m.len());
    let _ = file.seek(SeekFrom::Start(size.saturating_sub(4096)));
    let mut bytes = Vec::new();
    let _ = file.take(4096).read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

impl Tool {
    fn browse(self) -> Result<Option<String>, String> {
        let dialog = if matches!(self, Self::Compiler | Self::Sdk | Self::Clang) {
            "$dialog=New-Object System.Windows.Forms.FolderBrowserDialog; $dialog.Description='Select dependency directory'; if($dialog.ShowDialog() -eq 'OK'){[Console]::Write($dialog.SelectedPath)}; $dialog.Dispose()"
        } else {
            "$dialog=New-Object System.Windows.Forms.OpenFileDialog; $dialog.Title='Select dependency executable'; $dialog.Filter='Executables (*.exe)|*.exe|All files (*.*)|*.*'; $dialog.CheckFileExists=$true; if($dialog.ShowDialog() -eq 'OK'){[Console]::Write($dialog.FileName)}; $dialog.Dispose()"
        };
        let mut command = Command::new("powershell.exe");
        command.args(["-NoProfile", "-STA", "-Command"])
            .arg(format!("Add-Type -AssemblyName System.Windows.Forms; [Console]::OutputEncoding=[System.Text.UTF8Encoding]::new(); {dialog}"));
        crate::pipeline::quiet(&mut command);
        let output = command.output().map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err("Could not open the file browser. Enter the path directly.".into());
        }
        let path = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        Ok((!path.is_empty()).then_some(path))
    }
    fn label(self) -> &'static str {
        match self {
            Self::Make => "GNU Make",
            Self::Compiler => "MIPS toolchain",
            Self::Sdk => "Nugget SDK",
            Self::Emulator => "PCSX-Redux",
            Self::Audio => "psxavenc",
            Self::Disc => "mkpsxiso",
            Self::Clang => "libclang 18.1.1",
        }
    }
    fn purpose(self) -> &'static str {
        match self {
            Self::Make | Self::Compiler | Self::Sdk => "Required for native builds",
            Self::Emulator => "Required for Play",
            Self::Audio => "Required for XA music conversion",
            Self::Disc => "Required for disc export, music and geometry streaming",
            Self::Clang => "Required for C++ script reflection (library directory)",
        }
    }
    fn value(self, c: &mut Config) -> &mut String {
        match self {
            Self::Make => &mut c.make,
            Self::Compiler => &mut c.toolchain_bin,
            Self::Sdk => &mut c.nugget,
            Self::Emulator => &mut c.emulator,
            Self::Audio => &mut c.psxavenc,
            Self::Disc => &mut c.mkpsxiso,
            Self::Clang => &mut c.libclang,
        }
    }
    fn bundled(self, home: &Path) -> String {
        #[cfg(target_os = "linux")]
        let path = match self {
            Self::Make => return "make".into(),
            Self::Compiler => ".tools/linux/mips/bin",
            Self::Sdk => "third_party/nugget",
            Self::Emulator => ".tools/linux/redux/PCSX-Redux-HEAD-x86_64.AppImage",
            Self::Audio => ".tools/linux/psxavenc/bin/psxavenc",
            Self::Disc => ".tools/linux/mkpsxiso/mkpsxiso-2.30-Linux/bin/mkpsxiso",
            Self::Clang => ".tools/linux/libclang/clang/native",
        };
        #[cfg(not(target_os = "linux"))]
        let path = match self {
            Self::Make => ".tools/mips/bin/make.exe",
            Self::Compiler => ".tools/mips/bin",
            Self::Sdk => "third_party/nugget",
            Self::Emulator => ".tools/redux/pcsx-redux.exe",
            Self::Audio => ".tools/psxavenc/bin/psxavenc.exe",
            Self::Disc => ".tools/mkpsxiso/mkpsxiso-2.30-win64/mkpsxiso.exe",
            Self::Clang => ".tools/libclang/libclang-18.1.1.data/platlib/clang/native",
        };
        home.join(path).to_string_lossy().into()
    }
}

fn executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Resolve explicit paths and PATH names without launching untrusted executables.
pub fn find_executable(value: &str) -> Option<PathBuf> {
    find_in_path(value, std::env::var_os("PATH").as_deref())
}
pub(crate) fn find_in_path(value: &str, search: Option<&std::ffi::OsStr>) -> Option<PathBuf> {
    if value.trim().is_empty() {
        return None;
    }
    let path = Path::new(value);
    if path.is_absolute() || value.contains(['/', '\\']) {
        return executable_file(path).then(|| path.into());
    }
    for directory in search.into_iter().flat_map(std::env::split_paths) {
        if !directory.is_absolute() {
            continue;
        }
        let candidate = directory.join(value);
        if executable_file(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        if path.extension().is_none() {
            let candidate = directory.join(format!("{value}.exe"));
            if executable_file(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn compiler_name() -> &'static str {
    if cfg!(windows) {
        "mipsel-none-elf-g++.exe"
    } else {
        "mipsel-none-elf-g++"
    }
}
pub fn clang_library_name() -> &'static str {
    if cfg!(windows) {
        "libclang.dll"
    } else if cfg!(target_os = "macos") {
        "libclang.dylib"
    } else {
        "libclang.so"
    }
}
fn inspect(tool: Tool, config: &Config) -> Result<String, String> {
    let mut c = config.clone();
    let value = tool.value(&mut c).clone();
    let missing = |path: &Path| format!("Missing: {}", path.display());
    match tool {
        Tool::Sdk => {
            for file in [
                "common.mk",
                "psyqo/psyqo.mk",
                "third_party/EASTL/include/EASTL/array.h",
                "third_party/EABase/include/Common/EABase/eabase.h",
            ] {
                let path = Path::new(&value).join(file);
                if !path.is_file() {
                    return Err(missing(&path));
                }
            }
            Ok(value)
        }
        Tool::Clang => {
            let path = Path::new(&value).join(clang_library_name());
            if path.is_file() {
                Ok(path.display().to_string())
            } else {
                Err(missing(&path))
            }
        }
        Tool::Compiler => {
            for name in [
                compiler_name(),
                if cfg!(windows) {
                    "mipsel-none-elf-objcopy.exe"
                } else {
                    "mipsel-none-elf-objcopy"
                },
            ] {
                let path = if value.is_empty() {
                    name.into()
                } else {
                    Path::new(&value).join(name).to_string_lossy().into_owned()
                };
                if find_executable(&path).is_none() {
                    return Err(format!("Missing: {path}"));
                }
            }
            Ok(if value.is_empty() {
                "Located on PATH".into()
            } else {
                value
            })
        }
        _ => find_executable(&value)
            .map(|p| p.display().to_string())
            .ok_or_else(|| format!("Missing: {value}")),
    }
}

struct Install {
    child: Child,
    log: PathBuf,
    total_checks: usize,
    _lock: fs::File,
}
impl Drop for Install {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            return;
        }
        #[cfg(windows)]
        {
            let mut command = Command::new("taskkill.exe");
            command
                .args(["/PID", &self.child.id().to_string(), "/T", "/F"])
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            crate::pipeline::quiet(&mut command);
            let _ = command.status();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct State {
    #[cfg(test)]
    warning_button: [f32; 2],
    owner: PathBuf,
    pub draft: Config,
    report: Vec<Result<String, String>>,
    error: Option<String>,
    pub warning: bool,
    pub open: bool,
    installer: Option<Install>,
    pub message: String,
}
impl State {
    pub fn new(root: &Path) -> Self {
        let loaded = Config::editable(root);
        let error = loaded.as_ref().err().cloned();
        let (owner, draft) = loaded.unwrap_or_else(|_| (Config::owner(root), Config::default()));
        let mut state = Self {
            #[cfg(test)]
            warning_button: [0.; 2],
            owner,
            draft,
            report: vec![],
            error,
            warning: false,
            open: false,
            installer: None,
            message: String::new(),
        };
        state.check();
        state.warning = state.error.is_some() || state.report.iter().any(Result::is_err);
        state
    }
    pub fn busy(&self) -> bool {
        self.installer.is_some()
    }
    pub fn progress(&self) -> String {
        self.installer.as_ref().map_or_else(
            || self.message.clone(),
            |i| {
                let ready = self.report.iter().filter(|result| result.is_ok()).count();
                format!(
                    "{}\nChecks ready: {ready}/{}\nLive output:\n{}\nLog: {}",
                    self.message,
                    i.total_checks,
                    log_tail(&i.log),
                    i.log.display()
                )
            },
        )
    }
    fn check(&mut self) {
        let mut resolved = self.draft.clone();
        resolved.resolve_paths(&self.owner);
        self.report = TOOLS.iter().map(|tool| inspect(*tool, &resolved)).collect();
    }
    pub fn save(&mut self) -> Result<(), String> {
        if self.busy() {
            return Err("Wait for dependency installation to finish.".into());
        }
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        let path = self.owner.join("Local.epokconfig");
        // Keep a recoverable copy of the machine configuration, including unrelated fields.
        let backup = self.owner.join(".epok/dependencies/Local.epokconfig.bak");
        if path.is_file() && !backup.exists() {
            fs::create_dir_all(backup.parent().unwrap()).map_err(|e| e.to_string())?;
            fs::copy(&path, &backup).map_err(|e| e.to_string())?;
        }
        let (_, mut current) = Config::editable(&self.owner)?;
        for tool in TOOLS {
            *tool.value(&mut current) = tool.value(&mut self.draft).clone();
        }
        crate::project::write_changed(&path, &crate::document::to_vec(&current)?)?;
        self.draft = current;
        self.check();
        self.warning = false;
        self.message = "Dependency paths saved. Changes apply to the next build or import.".into();
        Ok(())
    }
    fn install_all(&mut self) -> Result<(), String> {
        if self.busy() {
            return Err("An installation is already running.".into());
        }
        let missing = self.report.iter().filter(|result| result.is_err()).count();
        if missing == 0 {
            return Err("All dependency checks already pass.".into());
        }
        #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
        {
            let home = crate::workspace::editor_home();
            #[cfg(windows)]
            let (program, script, arguments): (&str, PathBuf, Vec<String>) = (
                "powershell.exe",
                home.join("tools/setup.ps1"),
                vec![
                    "-NoProfile".into(),
                    "-NonInteractive".into(),
                    "-ExecutionPolicy".into(),
                    "Bypass".into(),
                    "-File".into(),
                ],
            );
            #[cfg(target_os = "linux")]
            let (program, script, arguments): (&str, PathBuf, Vec<String>) =
                ("bash", home.join("tools/setup-linux.sh"), vec![]);
            #[cfg(target_os = "macos")]
            let (program, script, arguments): (&str, PathBuf, Vec<String>) =
                ("bash", home.join("tools/setup-macos.sh"), vec![]);
            if !script.is_file() {
                return Err(format!(
                    "Missing dependency setup script: {}",
                    script.display()
                ));
            }
            let folder = home.join(".epok/dependencies");
            fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
            let lock = fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(folder.join("install.lock"))
                .map_err(|e| e.to_string())?;
            lock.try_lock().map_err(|e| {
                format!("Another dependency installer is using this installation: {e}")
            })?;
            let log = folder.join(format!("install-all-{}.log", uuid::Uuid::new_v4()));
            let file = fs::File::create(&log).map_err(|e| e.to_string())?;
            let mut command = Command::new(program);
            command
                .args(arguments)
                .arg(script)
                .args(if cfg!(windows) {
                    vec!["-Repair"]
                } else {
                    // Linux and macOS setup are host-wide. On macOS, Homebrew
                    // owns the compiler and several tools share build inputs.
                    vec!["--repair"]
                })
                .current_dir(&home)
                .stdin(Stdio::null())
                .stdout(file.try_clone().map_err(|e| e.to_string())?)
                .stderr(file);
            crate::pipeline::quiet(&mut command);
            let child = command.spawn().map_err(|e| e.to_string())?;
            self.installer = Some(Install {
                child,
                log,
                total_checks: TOOLS.len(),
                _lock: lock,
            });
            self.message = format!(
                "Installing or repairing {missing} missing dependencies. Editing is locked until installation finishes."
            );
            Ok(())
        }
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        {
            Err("Automatic package downloads are not available for this host.".into())
        }
    }
    pub fn poll(&mut self) {
        let Some(install) = &mut self.installer else {
            return;
        };
        let result = install.child.try_wait();
        if matches!(result, Ok(None)) {
            // Files may appear before setup has finished. Rechecking makes the
            // live counter reflect actual completed dependency checks.
            self.check();
            return;
        }
        let install = self.installer.take().unwrap();
        self.message = match result {
            Ok(Some(status)) if status.success() => {
                #[cfg(target_os = "macos")]
                {
                    // macOS setup writes one coherent Local.epokconfig for its
                    // shared Homebrew/local toolchain. Reload it instead of
                    // overwriting it with the pre-install editor draft.
                    match Config::editable(&self.owner) {
                        Ok((owner, draft)) => {
                            self.owner = owner;
                            self.draft = draft;
                            self.check();
                            format!(
                                "macOS setup completed and its saved paths were reloaded. Log: {}",
                                install.log.display()
                            )
                        }
                        Err(error) => format!(
                            "Setup completed, but could not reload its configuration: {error}"
                        ),
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    stage_bundled(&mut self.draft, &crate::workspace::editor_home());
                    self.check();
                    format!(
                        "All bundled dependencies are installed. Apply to use the new paths. Log: {}",
                        install.log.display()
                    )
                }
            }
            Ok(Some(status)) => format!(
                "Installation failed ({status}).\n{}\nLog: {}",
                log_tail(&install.log).trim(),
                install.log.display()
            ),
            Err(error) => format!("Installer failed: {error}. Log: {}", install.log.display()),
            Ok(None) => unreachable!(),
        };
        // Keep the dependency window available after a fast setup failure so
        // the final error and its log path do not vanish with the busy modal.
        self.open = true;
    }
    /// Non-modal warning: missing optional tools must not prevent scene editing.
    pub fn warning_ui(&mut self, ui: &imgui::Ui) -> bool {
        if !self.warning {
            return false;
        }
        let mut configure = false;
        let mut open = true;
        let screen = ui.io().display_size;
        ui.window("Missing editor dependencies").opened(&mut open)
            .position([screen[0]*0.5,screen[1]*0.5],imgui::Condition::Appearing).position_pivot([0.5,0.5])
            .size([640., 440.], imgui::Condition::FirstUseEver).size_constraints([480.,300.],[1000.,800.]).build(|| {
                ui.child_window("missing-tool-list").size([0.,-48.]).build(|| {
                ui.text_colored([1.,0.72,0.3,1.], "Some editor features need additional tools.");
                ui.text_wrapped("You can keep editing. Configure missing tools before using the affected features.");
                if let Some(error) = &self.error { ui.text_wrapped(error); }
                for (tool, result) in TOOLS.iter().zip(&self.report) {
                    if result.is_err() { ui.text_wrapped(format!("{}: {}", tool.label(), tool.purpose())); }
                }
                });
                if ui.button("Open Dependencies") { configure = true; }
                #[cfg(test)] {
                    let a = ui.item_rect_min(); let b = ui.item_rect_max();
                    self.warning_button = [(a[0]+b[0])*0.5, (a[1]+b[1])*0.5];
                }
                ui.same_line();
                if ui.button("Later") { self.warning = false; }
            });
        self.warning &= open && !configure;
        configure
    }
    pub fn page(&mut self, ui: &imgui::Ui, blocked: bool) {
        ui.text("General  >  Dependencies");
        ui.text_wrapped(format!(
            "Configuration: {}",
            self.owner.join("Local.epokconfig").display()
        ));
        ui.text_wrapped("Paths are local to this installation, or this project when it has a Local.epokconfig. Relative paths use the folder above. Executable names use PATH. Apply saves without restarting.");
        ui.text_disabled("Checks locate files; tool compatibility is validated when building.");
        if !self.message.is_empty() {
            ui.text_wrapped(&self.message);
        }
        if let Some(error) = &self.error {
            ui.text_wrapped(error);
        }
        let mut reload = false;
        {
            let _disabled = ui.begin_disabled(self.busy() || blocked);
            if ui.button("Recheck paths") {
                self.check();
            }
            ui.same_line();
            if ui.button("Reload saved paths") {
                reload = true;
            }
            if (cfg!(windows) || cfg!(target_os = "linux")) && ui.button("Use bundled paths") {
                for tool in TOOLS {
                    *tool.value(&mut self.draft) = tool.bundled(&crate::workspace::editor_home());
                }
                self.check();
                self.message = "Bundled paths selected. Apply to save.".into();
            }
        }
        if reload {
            *self = Self::new(&self.owner);
            self.warning = false;
        }
        let disabled = blocked || self.busy() || self.error.is_some();
        for (index, tool) in TOOLS.iter().copied().enumerate() {
            let _id = ui.push_id(tool.label());
            ui.separator();
            ui.text(tool.label());
            ui.same_line();
            let found = self.report[index].is_ok();
            ui.text_colored(
                if found {
                    [0.4, 0.85, 0.55, 1.]
                } else {
                    [1., 0.72, 0.3, 1.]
                },
                if found { "Located" } else { "Missing" },
            );
            ui.text_disabled(tool.purpose());
            let _disabled = ui.begin_disabled(disabled);
            ui.set_next_item_width(-1.);
            if ui
                .input_text("##path", tool.value(&mut self.draft))
                .hint(match tool {
                    Tool::Compiler => "Empty = PATH",
                    Tool::Clang => "Empty = bundled library",
                    _ => "Executable or directory path",
                })
                .build()
            {
                self.check();
            }
            if let Err(error) = &self.report[index] {
                ui.text_wrapped(error);
            }
            if cfg!(windows) {
                if ui.small_button("Browse...") {
                    match tool.browse() {
                        Ok(Some(path)) => {
                            *tool.value(&mut self.draft) = path;
                            self.check();
                        }
                        Ok(None) => {}
                        Err(error) => self.message = error,
                    }
                }
                ui.same_line();
                if ui.small_button("Use bundled path") {
                    *tool.value(&mut self.draft) = tool.bundled(&crate::workspace::editor_home());
                    self.check();
                }
            } else if cfg!(target_os = "linux") && ui.small_button("Use bundled path") {
                *tool.value(&mut self.draft) = tool.bundled(&crate::workspace::editor_home());
                self.check();
            }
        }
        ui.separator();
        if cfg!(windows) || cfg!(target_os = "linux") {
            ui.text_wrapped("Install / Repair runs the complete verified setup once, so package installations cannot overlap. Custom paths are kept until you Apply. Nugget uses the pinned SDK revision.");
        } else if cfg!(target_os = "macos") {
            ui.text_wrapped("Install / Repair runs the complete macOS setup once, with live output below. It repairs the shared Homebrew and local tools together, and downloads PCSX-Redux into this Epok installation when needed.");
        } else {
            ui.text_wrapped(
                "Automatic package downloads are currently available on Windows, macOS and Linux.",
            );
        }
        if blocked {
            ui.text_wrapped("Stop Play or wait for the build before changing dependencies.");
        }
        if let Some(install) = &self.installer {
            let ready = self.report.iter().filter(|result| result.is_ok()).count();
            ui.text_wrapped(format!(
                "Installing… {ready}/{} checks ready. Live log: {}",
                install.total_checks,
                install.log.display()
            ));
            if let Ok(mut file) = fs::File::open(&install.log) {
                use std::io::{Read, Seek, SeekFrom};
                let size = file.metadata().map_or(0, |m| m.len());
                let _ = file.seek(SeekFrom::Start(size.saturating_sub(4096)));
                let mut bytes = Vec::new();
                let _ = file.take(4096).read_to_end(&mut bytes);
                let log = String::from_utf8_lossy(&bytes);
                let lines: Vec<_> = log.lines().rev().take(12).collect();
                for line in lines.iter().rev() {
                    ui.text_wrapped(line);
                }
            }
        }
    }
    pub fn install_all_control(&mut self, ui: &imgui::Ui, blocked: bool) {
        let missing = self.report.iter().filter(|result| result.is_err()).count();
        let _disabled = ui.begin_disabled(blocked || self.busy() || missing == 0);
        if ui.button(format!("Install / Repair {missing} missing"))
            && let Err(error) = self.install_all()
        {
            self.message = error;
        }
    }
    pub fn hub_window(&mut self, ui: &imgui::Ui) {
        self.poll();
        if self.warning_ui(ui) {
            self.open = true;
        }
        if !self.open {
            return;
        }
        let mut open = true;
        ui.window("Editor Dependencies")
            .opened(&mut open)
            .size([820., 680.], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.child_window("dependency-settings")
                    .size([0., -45.])
                    .build(|| self.page(ui, false));
                self.install_all_control(ui, false);
                ui.same_line();
                let _disabled = ui.begin_disabled(self.busy());
                if ui.button("Apply") {
                    self.message = self
                        .save()
                        .map_or_else(|e| e, |()| "Dependency paths saved.".into());
                }
            });
        self.open = open;
    }
}

#[cfg(test)]
pub fn verify_interactions(context: &mut imgui::Context) {
    let root = std::env::temp_dir().join(format!("epok-dependency-ui-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("Local.epokconfig"),
        "psxavenc: missing/encoder.exe",
    )
    .unwrap();
    let mut state = State::new(&root);
    assert!(state.warning);
    let frame = |context: &mut imgui::Context, state: &mut State| {
        state.hub_window(context.frame());
        context.render();
    };
    frame(context, &mut state);
    frame(context, &mut state);
    context.io_mut().add_mouse_pos_event(state.warning_button);
    frame(context, &mut state);
    context
        .io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, true);
    frame(context, &mut state);
    context
        .io_mut()
        .add_mouse_button_event(imgui::MouseButton::Left, false);
    frame(context, &mut state);
    assert!(state.open && !state.warning);
    assert_eq!(
        fs::read_to_string(root.join("Local.epokconfig")).unwrap(),
        "psxavenc: missing/encoder.exe"
    );
    fs::remove_dir_all(root).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn installing_all_updates_bundled_paths_together() {
        let home = std::env::temp_dir().join("epok-package-test");
        let mut config = Config {
            make: "old/make.exe".into(),
            toolchain_bin: "old/bin".into(),
            emulator: "custom/redux.exe".into(),
            ..Default::default()
        };
        stage_bundled(&mut config, &home);
        assert_eq!(config.make, Tool::Make.bundled(&home));
        assert_eq!(config.toolchain_bin, Tool::Compiler.bundled(&home));
        assert_eq!(config.emulator, Tool::Emulator.bundled(&home));
    }
    #[test]
    fn explicit_missing_path_does_not_fall_back_to_path() {
        let folder = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&folder).unwrap();
        let executable = folder.join(if cfg!(windows) { "tool.exe" } else { "tool" });
        fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
        let search = std::env::join_paths([&folder]).unwrap();
        assert_eq!(find_in_path("tool", Some(&search)), Some(executable));
        assert!(
            find_in_path(folder.join("old/tool.exe").to_str().unwrap(), Some(&search)).is_none()
        );
        assert!(find_in_path("", Some(&search)).is_none());
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn local_paths_round_trip_preserving_other_configuration() {
        let folder = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&folder).unwrap();
        let path = folder.join("Local.epokconfig");
        let original = Config {
            web_port: 9123,
            code: "custom-code".into(),
            psxavenc: "old/encoder.exe".into(),
            ..Default::default()
        };
        fs::write(&path, crate::document::to_vec(&original).unwrap()).unwrap();
        let mut state = State::new(&folder);
        assert!(state.warning);
        state.draft.psxavenc = "new/encoder.exe".into();
        state.save().unwrap();
        let loaded = Config::load(&folder).unwrap();
        assert_eq!(loaded.web_port, 9123);
        assert_eq!(loaded.code, "custom-code");
        assert_eq!(Path::new(&loaded.psxavenc), folder.join("new/encoder.exe"));
        let backup: Config = crate::document::from_slice(
            &fs::read(folder.join(".epok/dependencies/Local.epokconfig.bak")).unwrap(),
        )
        .unwrap();
        assert_eq!(backup.psxavenc, original.psxavenc);
        fs::write(&path, "unknown_field: broken").unwrap();
        let mut state = State::new(&folder);
        assert!(state.save().is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "unknown_field: broken");
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn incomplete_sdk_is_reported() {
        let folder = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("common.mk"), "").unwrap();
        let config = Config {
            nugget: folder.display().to_string(),
            ..Default::default()
        };
        assert!(
            inspect(Tool::Sdk, &config)
                .unwrap_err()
                .contains("psyqo.mk")
        );
        fs::remove_dir_all(folder).unwrap();
    }
}
