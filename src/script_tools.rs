//! Machine-local language service discovery and verified portable installations.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::Duration,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub lua_ls: String,
    pub clangd: String,
    pub copilot: String,
    pub copilot_enabled: bool,
}
impl Config {
    pub fn load() -> Self {
        fs::read(crate::workspace::user_data().join("CodeEditor.epokprefs"))
            .ok()
            .and_then(|b| crate::document::from_slice(&b).ok())
            .unwrap_or_default()
    }
    pub fn save(&self) -> Result<(), String> {
        for name in ["lua_ls", "clangd", "copilot"] {
            let value = self.path(name);
            if !value.is_empty() && crate::dependencies::find_executable(value).is_none() {
                return Err(format!(
                    "{name}: select an existing executable, use Auto-detect, or install the portable package."
                ));
            }
        }
        if self.copilot_enabled && self.copilot.is_empty() {
            return Err("Install or select the Copilot server before enabling suggestions.".into());
        }
        crate::settings::save_document(
            &crate::workspace::user_data().join("CodeEditor.epokprefs"),
            self,
        )
    }
    pub fn detect(&mut self) {
        for name in ["lua_ls", "clangd", "copilot"] {
            let current = self.path(name);
            if !current.is_empty() && Path::new(current).is_file() {
                continue;
            }
            let binary = binary(name);
            let found = find_binary(&tools_root().join(name), binary)
                .or_else(|| crate::dependencies::find_executable(binary))
                .or_else(|| {
                    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
                        .map(PathBuf::from)?;
                    let extensions = home.join(".vscode/extensions");
                    let prefix = match name {
                        "lua_ls" => "sumneko.lua-",
                        "clangd" => "llvm-vs-code-extensions.vscode-clangd-",
                        _ => return None,
                    };
                    fs::read_dir(extensions)
                        .ok()?
                        .filter_map(Result::ok)
                        .filter(|e| e.file_name().to_string_lossy().starts_with(prefix))
                        .find_map(|e| find_binary(&e.path(), binary))
                });
            if let Some(found) = found {
                self.set(name, found.to_string_lossy().into_owned());
            }
        }
    }
    pub fn path(&self, name: &str) -> &str {
        match name {
            "lua_ls" | "lua" | "pocket" => &self.lua_ls,
            "clangd" | "cpp" => &self.clangd,
            "copilot" | "copilot_chat" => &self.copilot,
            _ => "",
        }
    }
    pub fn set(&mut self, name: &str, value: String) {
        match name {
            "lua_ls" => self.lua_ls = value,
            "clangd" => self.clangd = value,
            "copilot" => self.copilot = value,
            _ => {}
        }
    }
    pub fn spec(&self, name: &str, root: &Path) -> Result<crate::script_lsp::Spec, String> {
        let executable =
            crate::dependencies::find_executable(self.path(name)).ok_or_else(|| {
                format!("{name} is not configured. Open Setup to install or select its executable.")
            })?;
        let directory = if name == "pocket" {
            root.join(".epok/code/pocket")
        } else {
            root.to_path_buf()
        };
        fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        let arguments = match name {
            "lua" | "pocket" => vec![],
            "cpp" => vec![
                OsString::from(format!(
                    "--compile-commands-dir={}",
                    crate::code_editor::editor_path(&root.join(".epok/code/cpp")).display()
                )),
                "--header-insertion=never".into(),
            ],
            "copilot" if self.copilot_enabled => vec!["--stdio".into()],
            "copilot_chat" if self.copilot_enabled => vec!["--acp".into()],
            "copilot" | "copilot_chat" => {
                return Err("Enable Copilot in Setup before starting it".into());
            }
            _ => return Err("Unknown language service".into()),
        };
        Ok(crate::script_lsp::Spec {
            executable,
            arguments,
            directory,
            framing: if name == "copilot_chat" {
                crate::script_lsp::Framing::JsonLines
            } else {
                crate::script_lsp::Framing::Lsp
            },
        })
    }
}

pub fn tools_root() -> PathBuf {
    crate::workspace::user_data().join("CodeTools")
}
fn binary(name: &str) -> &'static str {
    match (name, cfg!(windows)) {
        ("lua_ls", true) => "lua-language-server.exe",
        ("lua_ls", false) => "lua-language-server",
        ("clangd", true) => "clangd.exe",
        ("clangd", false) => "clangd",
        (_, true) => "copilot-language-server.exe",
        (_, false) => "copilot-language-server",
    }
}
fn find_binary(root: &Path, name: &str) -> Option<PathBuf> {
    fn visit(root: &Path, name: &str, depth: usize) -> Option<PathBuf> {
        if depth > 7 {
            return None;
        }
        for entry in fs::read_dir(root).ok()?.filter_map(Result::ok) {
            let ty = entry.file_type().ok()?;
            if ty.is_symlink() {
                continue;
            }
            if ty.is_file() && entry.file_name() == name {
                return Some(entry.path());
            }
            if ty.is_dir()
                && let Some(found) = visit(&entry.path(), name, depth + 1)
            {
                return Some(found);
            }
        }
        None
    }
    visit(root, name, 0)
}

struct Package {
    version: &'static str,
    repository: &'static str,
    filename: String,
    digest: &'static str,
}
fn package(name: &str) -> Result<Package, String> {
    let platform = if cfg!(windows) {
        "win32"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    };
    let (version, repository, filename, digest) =
        match (name, platform, arch) {
            ("lua_ls", "win32", "x64") => (
                "3.19.2",
                "LuaLS/lua-language-server",
                "lua-language-server-3.19.2-win32-x64.zip".into(),
                "4a3dc851b3b31e7b75a23f645d1ba4a2137579a55f5fa4edbb17a3e2a6254044",
            ),
            ("lua_ls", "darwin", "arm64") => (
                "3.19.2",
                "LuaLS/lua-language-server",
                "lua-language-server-3.19.2-darwin-arm64.tar.gz".into(),
                "79149584320a23bd5fcd5bc31f47a1975acd8a4560a0e89036bb1016fa61687f",
            ),
            ("lua_ls", "darwin", "x64") => (
                "3.19.2",
                "LuaLS/lua-language-server",
                "lua-language-server-3.19.2-darwin-x64.tar.gz".into(),
                "6f2174ece5ec63bcd9d5b22b2c902445c1cd9045fafe48c62602448ebf4c09cc",
            ),
            ("lua_ls", "linux", "x64") => (
                "3.19.2",
                "LuaLS/lua-language-server",
                "lua-language-server-3.19.2-linux-x64.tar.gz".into(),
                "93ad326b10f6c00911dd8c450fa098332dbd8f9f45d1052f5636a2d54479b947",
            ),
            ("lua_ls", "linux", "arm64") => (
                "3.19.2",
                "LuaLS/lua-language-server",
                "lua-language-server-3.19.2-linux-arm64.tar.gz".into(),
                "17b8e401a874440adeb649e5024c364f2bb9d15156c74a40e56d1ae9a1b2e4f2",
            ),
            ("clangd", "win32", "x64") => (
                "23.1.0",
                "clangd/clangd",
                "clangd-windows-23.1.0.zip".into(),
                "23412a240756a162e7b98a282f36aa2a23a88db5ce16a0cbc4fef7253768c810",
            ),
            ("clangd", "darwin", _) => (
                "23.1.0",
                "clangd/clangd",
                "clangd-mac-23.1.0.zip".into(),
                "1082e6638223b785ca2daf0939f13afcd0bb95c84ee9a4bbaff4745365159253",
            ),
            ("clangd", "linux", "x64") => (
                "23.1.0",
                "clangd/clangd",
                "clangd-linux-23.1.0.zip".into(),
                "e53b1a96196095faedb7642cf64964f7fb9ad4a0c1f00dd2c172a3d9dcbafdfd",
            ),
            ("copilot", "win32", "x64") => (
                "1.556.0",
                "github/copilot-language-server-release",
                "copilot-language-server-win32-x64-1.556.0.zip".into(),
                "7b09b57d008fa79ea5fe67363f8d2df866e928aeb7c47f925d1f404a7821e5be",
            ),
            ("copilot", "win32", "arm64") => (
                "1.556.0",
                "github/copilot-language-server-release",
                "copilot-language-server-win32-arm64-1.556.0.zip".into(),
                "0cff49135fd17f132cea67ff6742e2f680a0bfb3b07eb62d6cbac62b434c39f2",
            ),
            ("copilot", "darwin", "arm64") => (
                "1.556.0",
                "github/copilot-language-server-release",
                "copilot-language-server-darwin-arm64-1.556.0.zip".into(),
                "369bea897e53627a505d41045f58ce937b85d5c26b1c426cf1762690d99dbe22",
            ),
            ("copilot", "darwin", "x64") => (
                "1.556.0",
                "github/copilot-language-server-release",
                "copilot-language-server-darwin-x64-1.556.0.zip".into(),
                "67d0c3e698b2602b70584025a06681049b12bcae5aa63944eab8a5e935325e4a",
            ),
            ("copilot", "linux", "x64") => (
                "1.556.0",
                "github/copilot-language-server-release",
                "copilot-language-server-linux-x64-1.556.0.zip".into(),
                "714087834355debe4bdc973e40d4eebb28ebbd405a5a3a824b6dff59aada1e15",
            ),
            ("copilot", "linux", "arm64") => (
                "1.556.0",
                "github/copilot-language-server-release",
                "copilot-language-server-linux-arm64-1.556.0.zip".into(),
                "6d215b4796a88efafa4f580af66204aaf3e7735e39c5727864b3394cd34a42ad",
            ),
            _ => return Err(
                "No portable package for this host. Select an installed server executable instead."
                    .into(),
            ),
        };
    Ok(Package {
        version,
        repository,
        filename,
        digest,
    })
}

pub fn install(name: &str, progress: impl Fn(String)) -> Result<PathBuf, String> {
    let package = package(name)?;
    let target = tools_root().join(name).join(package.version).join(format!(
        "{}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    if target.join("verified.json").is_file()
        && let Some(exe) = find_binary(&target, binary(name))
    {
        return Ok(exe);
    }
    let parent = target.parent().ok_or("Invalid tool directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(parent.join("install.lock"))
        .map_err(|e| e.to_string())?;
    lock.try_lock()
        .map_err(|_| "This tool is already being installed in another Epok window")?;
    let staging = parent.join(format!("unpack-{}", Uuid::new_v4()));
    fs::create_dir(&staging).map_err(|e| e.to_string())?;
    progress(format!(
        "Downloading {name} {} from its official release…",
        package.version
    ));
    let client = reqwest::blocking::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())?;
    let url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        package.repository, package.version, package.filename
    );
    let mut response = client
        .get(&url)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?;
    let archive = staging.join(&package.filename);
    let mut file = fs::File::create(&archive).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    let mut total = 0usize;
    loop {
        let count = response.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        total += count;
        if total > 200 * 1024 * 1024 {
            return Err("Tool download exceeds expected package limit".into());
        }
        hash.update(&buffer[..count]);
        file.write_all(&buffer[..count])
            .map_err(|e| e.to_string())?;
    }
    drop(file);
    if format!("{:x}", hash.finalize()) != package.digest {
        return Err(
            "Downloaded tool failed SHA-256 verification. Existing tools were preserved.".into(),
        );
    }
    progress(format!("Verified {name}; extracting portable package…"));
    let contents = staging.join("contents");
    fs::create_dir(&contents).map_err(|e| e.to_string())?;
    extract(&archive, &contents)?;
    let executable = find_binary(&contents, binary(name))
        .ok_or("Package does not contain the expected executable")?;
    let relative = executable
        .strip_prefix(&contents)
        .map_err(|e| e.to_string())?
        .to_path_buf();
    crate::settings::save_document(
        &contents.join("verified.json"),
        &serde_json::json!({"url":url,"sha256":package.digest,"version":package.version}),
    )?;
    if target.exists() {
        return Err("An incomplete tool installation exists. Select another executable or repair its directory before retrying.".into());
    }
    fs::rename(&contents, &target).map_err(|e| e.to_string())?;
    // Only this verified download is removed; retain staging diagnostics on failure.
    fs::remove_file(&archive).map_err(|e| e.to_string())?;
    fs::remove_dir(&staging).map_err(|e| e.to_string())?;
    Ok(target.join(relative))
}
use uuid::Uuid;

fn safe_member(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
}
fn extract(archive: &Path, destination: &Path) -> Result<(), String> {
    let file = fs::File::open(archive).map_err(|e| e.to_string())?;
    let mut total = 0u64;
    if archive.extension().is_some_and(|s| s == "zip") {
        let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for n in 0..zip.len() {
            let mut entry = zip.by_index(n).map_err(|e| e.to_string())?;
            let path = entry
                .enclosed_name()
                .filter(|p| safe_member(p))
                .ok_or("Tool archive contains an invalid path")?;
            if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
                return Err("Tool archive links are unsupported".into());
            }
            total += entry.size();
            if total > 1024 * 1024 * 1024 {
                return Err("Expanded tool package exceeds 1 GiB".into());
            }
            let target = destination.join(&path);
            if entry.is_dir() {
                fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            } else {
                fs::create_dir_all(target.parent().ok_or("Invalid archive directory")?)
                    .map_err(|e| e.to_string())?;
                let mut output = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&target)
                    .map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut output).map_err(|e| e.to_string())?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = entry.unix_mode() {
                        fs::set_permissions(&target, fs::Permissions::from_mode(mode & 0o777))
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    } else {
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
        for entry in tar.entries().map_err(|e| e.to_string())? {
            let mut entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path().map_err(|e| e.to_string())?;
            if !safe_member(&path)
                || !(entry.header().entry_type().is_file() || entry.header().entry_type().is_dir())
            {
                return Err("Tool archive contains an invalid entry or link".into());
            }
            total += entry.size();
            if total > 1024 * 1024 * 1024 {
                return Err("Expanded tool package exceeds 1 GiB".into());
            }
            entry.unpack_in(destination).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
