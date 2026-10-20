//! Project source buffers and conflict-aware persistence for the integrated editor.
use crate::{assets, pocketstation};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone)]
enum Target {
    File(PathBuf),
    Pocket {
        asset: PathBuf,
        app: Uuid,
        name: String,
    },
}

#[derive(Clone, Serialize)]
pub struct Entry {
    pub uri: String,
    pub path: String,
    pub language: String,
}

#[derive(Serialize)]
pub struct Document {
    pub uri: String,
    pub path: String,
    pub language: String,
    pub text: String,
    pub version: u64,
    pub readonly: bool,
    #[serde(rename = "savedText")]
    saved: String,
    #[serde(skip)]
    revision: String,
}

#[derive(Default)]
pub struct Workspace {
    targets: BTreeMap<String, Target>,
    pub documents: BTreeMap<String, Document>,
}

pub fn file_uri(path: &Path) -> Result<String, String> {
    let uri = url::Url::from_file_path(crate::code_editor::editor_path(path))
        .map(String::from)
        .map_err(|_| "Source path must be absolute".to_string())?;
    // Monaco/VS Code serialize drive letters in lowercase and encode the
    // colon. Use that spelling for model keys and cross-process diagnostics.
    #[cfg(windows)]
    {
        let mut uri = uri;
        if uri.starts_with("file:///") && uri.as_bytes().get(9) == Some(&b':') {
            let drive = char::from(uri.as_bytes()[8]).to_ascii_lowercase();
            uri.replace_range(8..10, &format!("{drive}%3A"));
        }
        Ok(uri)
    }
    #[cfg(not(windows))]
    {
        Ok(uri)
    }
}

pub fn uri_path(uri: &str) -> Result<PathBuf, String> {
    url::Url::parse(uri)
        .map_err(|e| e.to_string())?
        .to_file_path()
        .map_err(|_| "Expected a local file URI".into())
}

fn language(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "lua" => Some("lua"),
        "plua" => Some("pocketlua"),
        "cpp" | "cc" | "cxx" | "hpp" | "h" | "hh" | "hxx" | "inl" => Some("cpp"),
        "s" | "asm" => Some("arm"),
        _ => None,
    }
}

fn read_source(path: &Path) -> Result<String, String> {
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > MAX_SOURCE_BYTES as u64 {
        return Err("Source file exceeds the 4 MiB editor limit".into());
    }
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

impl Workspace {
    pub fn pocket_app(&self, uri: &str) -> Option<Uuid> {
        match self.targets.get(uri) {
            Some(Target::Pocket { app, .. }) => Some(*app),
            _ => None,
        }
    }
    pub fn dirty(&self) -> bool {
        self.documents.values().any(|d| d.text != d.saved)
    }

    pub fn tree(&mut self, root: &Path, index: &assets::Index) -> Result<Vec<Entry>, String> {
        let canonical = fs::canonicalize(root).map_err(|e| e.to_string())?;
        let scripts = canonical.join("assets/scripts");
        let mut entries = Vec::new();
        fn walk(
            workspace: &mut Workspace,
            root: &Path,
            path: &Path,
            out: &mut Vec<Entry>,
        ) -> Result<(), String> {
            if !path.exists() {
                return Ok(());
            }
            for item in fs::read_dir(path).map_err(|e| e.to_string())? {
                let item = item.map_err(|e| e.to_string())?;
                if item.file_type().map_err(|e| e.to_string())?.is_symlink() {
                    continue;
                }
                let path = fs::canonicalize(item.path()).map_err(|e| e.to_string())?;
                if !path.starts_with(root) {
                    continue;
                }
                if path.is_dir() {
                    walk(workspace, root, &path, out)?;
                } else if let Some(language) = language(&path) {
                    if out.len() >= 10000 {
                        return Err("Script tree exceeds 10,000 files".into());
                    }
                    let uri = file_uri(&path)?;
                    workspace
                        .targets
                        .insert(uri.clone(), Target::File(path.clone()));
                    out.push(Entry {
                        uri,
                        path: assets::path_string(root, &path),
                        language: language.into(),
                    });
                }
            }
            Ok(())
        }
        walk(self, &canonical, &scripts, &mut entries)?;
        for records in index.assets.values() {
            if records.len() != 1 {
                continue;
            }
            let record = &records[0];
            if record.meta.kind != assets::Kind::PocketStationApp {
                continue;
            }
            let app = pocketstation::document(record)?;
            for file in &app.files {
                let mirror = pocket_mirror(&canonical, record.meta.id, &file.name)?;
                let uri = file_uri(&mirror)?;
                self.targets.insert(
                    uri.clone(),
                    Target::Pocket {
                        asset: record.path.clone(),
                        app: record.meta.id,
                        name: file.name.clone(),
                    },
                );
                entries.push(Entry {
                    uri,
                    path: format!("PocketStation/{}/{}", app.title, file.name),
                    language: language(Path::new(&file.name)).unwrap_or("arm").into(),
                });
            }
        }
        entries.sort_by_cached_key(|entry| entry.path.to_lowercase());
        Ok(entries)
    }

    pub fn open(&mut self, root: &Path, uri: &str) -> Result<&Document, String> {
        let normalized = file_uri(&uri_path(uri)?)?;
        let uri = normalized.as_str();
        if !self.documents.contains_key(uri) {
            let mut target = self
                .targets
                .get(uri)
                .cloned()
                .unwrap_or(Target::File(uri_path(uri)?));
            if let Target::File(path) = &mut target {
                *path = fs::canonicalize(&*path).map_err(|e| e.to_string())?;
            }
            let (path, text, revision, readonly) = match &target {
                Target::File(path) => {
                    let canonical = fs::canonicalize(path).map_err(|e| e.to_string())?;
                    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
                    let home = fs::canonicalize(crate::workspace::editor_home())
                        .map_err(|e| e.to_string())?;
                    let config = crate::project::Config::load(&root)?;
                    let mut sdk_roots = vec![
                        home.join("runtime"),
                        home.join("third_party"),
                        home.join(".tools/mips"),
                    ];
                    if !config.nugget.is_empty() {
                        sdk_roots.push(PathBuf::from(&config.nugget));
                    }
                    if !config.toolchain_bin.is_empty()
                        && let Some(parent) = Path::new(&config.toolchain_bin).parent()
                    {
                        sdk_roots.push(parent.to_path_buf());
                    }
                    let sdk = sdk_roots
                        .iter()
                        .filter_map(|p| fs::canonicalize(p).ok())
                        .any(|p| canonical.starts_with(p));
                    if !canonical.starts_with(&root) && !sdk {
                        return Err(
                            "The definition is outside the project and SDK source roots".into()
                        );
                    }
                    if language(&canonical).is_none() {
                        return Err("Select a Lua, C++ or PocketStation source file".into());
                    }
                    let text = read_source(&canonical)?;
                    let readonly = !canonical.starts_with(root.join("assets/scripts"));
                    let revision = assets::hash(text.as_bytes());
                    (
                        assets::path_string(&root, &canonical),
                        text,
                        revision,
                        readonly,
                    )
                }
                Target::Pocket { asset, app, name } => {
                    let record = current_record(asset)?;
                    if record.meta.id != *app {
                        return Err("PocketStation app was replaced".into());
                    }
                    let doc = pocketstation::document(&record)?;
                    let source = doc
                        .files
                        .iter()
                        .find(|f| &f.name == name)
                        .ok_or("PocketStation source no longer exists")?;
                    let mirror = pocket_mirror(root, *app, name)?;
                    crate::project::write_changed(&mirror, source.text.as_bytes())?;
                    (
                        format!("PocketStation/{}/{name}", doc.title),
                        source.text.clone(),
                        record.revision,
                        false,
                    )
                }
            };
            let language = language(Path::new(&path)).unwrap_or("arm").into();
            self.targets.insert(uri.into(), target);
            self.documents.insert(
                uri.into(),
                Document {
                    uri: uri.into(),
                    path,
                    language,
                    saved: text.clone(),
                    text,
                    revision,
                    version: 1,
                    readonly,
                },
            );
        }
        Ok(&self.documents[uri])
    }

    pub fn change(&mut self, uri: &str, text: String, version: u64) -> Result<(), String> {
        let doc = self
            .documents
            .get_mut(uri)
            .ok_or("Source buffer is not open")?;
        if doc.readonly {
            return Err("SDK and generated source is read-only".into());
        }
        if text.len() > MAX_SOURCE_BYTES {
            return Err("Source exceeds the 4 MiB editor limit".into());
        }
        if version > doc.version {
            doc.text = text;
            doc.version = version;
        }
        Ok(())
    }

    /// Returns the PocketStation app to refresh in its device/sprite inspector.
    pub fn save(&mut self, uri: &str) -> Result<Option<Uuid>, String> {
        let doc = self
            .documents
            .get_mut(uri)
            .ok_or("Source buffer is not open")?;
        if doc.readonly || doc.text == doc.saved {
            return Ok(None);
        }
        let target = self
            .targets
            .get(uri)
            .ok_or("Source target is unavailable")?;
        let app = match target {
            Target::File(path) => {
                let canonical = fs::canonicalize(path).map_err(|e| e.to_string())?;
                if &canonical != path {
                    return Err("Source path changed; reopen it before saving".into());
                }
                assets::atomic_write(path, doc.text.as_bytes(), Some(&doc.revision))?;
                doc.revision = assets::hash(doc.text.as_bytes());
                None
            }
            Target::Pocket { asset, app, name } => {
                let record = current_record(asset)?;
                if record.meta.id != *app {
                    return Err("PocketStation asset was replaced".into());
                }
                let mut latest = pocketstation::document(&record)?;
                let source = latest
                    .files
                    .iter_mut()
                    .find(|f| &f.name == name)
                    .ok_or("PocketStation source was removed")?;
                if source.text != doc.saved {
                    return Err("PocketStation source changed outside this buffer. Reopen it to compare before saving.".into());
                }
                source.text = doc.text.clone();
                doc.revision = pocketstation::save(&record, &latest)?;
                crate::project::write_changed(&uri_path(uri)?, doc.text.as_bytes())?;
                Some(*app)
            }
        };
        doc.saved = doc.text.clone();
        Ok(app)
    }

    pub fn close(&mut self, uri: &str, discard: bool) -> Result<(), String> {
        if self.documents.get(uri).is_some_and(|d| d.text != d.saved) && !discard {
            return Err("Save or discard source changes before closing".into());
        }
        self.documents.remove(uri);
        Ok(())
    }

    pub fn add_pocket_assembly(
        &mut self,
        root: &Path,
        uri: &str,
    ) -> Result<(Uuid, String), String> {
        let Target::Pocket { asset, app, .. } = self
            .targets
            .get(uri)
            .cloned()
            .ok_or("Select a PocketStation source tab first")?
        else {
            return Err("Select a PocketStation source tab first".into());
        };
        let record = current_record(&asset)?;
        if record.meta.id != app {
            return Err("PocketStation asset was replaced".into());
        }
        let mut document = pocketstation::document(&record)?;
        let mut number = document.files.len();
        let name = loop {
            let candidate = format!("routines{number}.s");
            if !document.files.iter().any(|f| f.name == candidate) {
                break candidate;
            }
            number += 1;
        };
        let routine = format!("double_{number}");
        document.files.push(pocketstation::SourceFile {
            name: name.clone(),
            text: format!("; Thumb functions callable from Pocket Lua.\n    .thumb\n    .global {routine}\n    .thumb_func\n{routine}:\n    add r0, r0, r0\n    bx lr\n"),
        });
        pocketstation::save(&record, &document)?;
        let uri = file_uri(&pocket_mirror(root, app, &name)?)?;
        self.targets
            .insert(uri.clone(), Target::Pocket { asset, app, name });
        self.open(root, &uri)?;
        Ok((app, uri))
    }

    pub fn discard(&mut self) {
        self.documents.clear();
    }
}

fn current_record(path: &Path) -> Result<assets::Record, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let package = assets::Package::load(path)?;
    Ok(assets::Record {
        path: path.into(),
        meta: package.meta,
        revision: assets::hash(&bytes),
    })
}

fn pocket_mirror(root: &Path, id: Uuid, name: &str) -> Result<PathBuf, String> {
    let path = Path::new(name);
    if path.components().count() != 1 || path.file_name().and_then(|s| s.to_str()) != Some(name) {
        return Err("PocketStation source names must be single filenames".into());
    }
    Ok(root
        .join(".epok/code/pocket")
        .join(id.to_string())
        .join(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    fn windows_model_uris_match_monaco_and_preserve_unicode_paths() {
        let source = Path::new(r"C:\Epok Projects\日本語\Guard.lua");
        let uri = file_uri(source).unwrap();
        assert_eq!(
            uri,
            "file:///c%3A/Epok%20Projects/%E6%97%A5%E6%9C%AC%E8%AA%9E/Guard.lua"
        );
        assert_eq!(
            uri_path(&uri).unwrap(),
            Path::new(r"c:\Epok Projects\日本語\Guard.lua")
        );
        let network = Path::new(r"\\server\games\Guard.lua");
        assert_eq!(uri_path(&file_uri(network).unwrap()).unwrap(), network);
    }
    #[test]
    fn source_save_preserves_external_edits_and_dirty_buffers() {
        let root = crate::workspace::tests::temp("code-workspace-conflict");
        fs::create_dir_all(root.join("assets/scripts")).unwrap();
        let source = root.join("assets/scripts/Guard.lua");
        fs::write(&source, "-- initial\n").unwrap();
        let mut workspace = Workspace::default();
        let entry = workspace
            .tree(&root, &Default::default())
            .unwrap()
            .remove(0);
        workspace.open(&root, &entry.uri).unwrap();
        workspace
            .change(&entry.uri, "-- buffer\n".into(), 2)
            .unwrap();
        fs::write(&source, "-- external\n").unwrap();
        assert!(workspace.save(&entry.uri).is_err());
        assert!(workspace.dirty());
        assert!(workspace.close(&entry.uri, false).is_err());
        assert_eq!(fs::read_to_string(&source).unwrap(), "-- external\n");
        workspace.close(&entry.uri, true).unwrap();
        workspace.open(&root, &entry.uri).unwrap();
        workspace
            .change(&entry.uri, "-- saved\n".into(), 2)
            .unwrap();
        workspace.save(&entry.uri).unwrap();
        assert_eq!(fs::read_to_string(&source).unwrap(), "-- saved\n");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pocket_source_save_merges_other_asset_fields_and_detects_code_conflicts() {
        let root = crate::workspace::tests::temp("code-pocket-buffer");
        fs::create_dir_all(&root).unwrap();
        let app =
            pocketstation::PocketApp::template(pocketstation::Template::BlankLua, "Pocket Code");
        let id = pocketstation::create(&root, "assets/PocketStation/Test.epokasset", &app).unwrap();
        let mut index = assets::scan(&root, &mut Default::default());
        let mut workspace = Workspace::default();
        let entry = workspace
            .tree(&root, &index)
            .unwrap()
            .into_iter()
            .find(|e| e.language == "pocketlua")
            .unwrap();
        workspace.open(&root, &entry.uri).unwrap();
        let initial = workspace.documents[&entry.uri].text.clone();
        workspace
            .change(&entry.uri, format!("{initial}\n-- edited in Monaco\n"), 2)
            .unwrap();
        let record = index.resolve(id).unwrap();
        let mut other = pocketstation::document(record).unwrap();
        other.title = "Changed title".into();
        pocketstation::save(record, &other).unwrap();
        workspace.save(&entry.uri).unwrap();
        index = assets::scan(&root, &mut Default::default());
        let saved = pocketstation::document(index.resolve(id).unwrap()).unwrap();
        assert_eq!(saved.title, "Changed title");
        assert!(saved.files[0].text.ends_with("-- edited in Monaco\n"));
        workspace
            .change(&entry.uri, "-- unsaved Monaco edit\n".into(), 3)
            .unwrap();
        let record = index.resolve(id).unwrap();
        let mut external = pocketstation::document(record).unwrap();
        external.files[0].text = "-- external code\n".into();
        pocketstation::save(record, &external).unwrap();
        assert!(workspace.save(&entry.uri).is_err());
        assert!(workspace.dirty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn adding_pocket_assembly_keeps_the_app_buildable_and_the_main_source_unchanged() {
        let root = crate::workspace::tests::temp("code-pocket-assembly");
        fs::create_dir_all(&root).unwrap();
        let app = pocketstation::PocketApp::template(
            pocketstation::Template::BlankLua,
            "Assembly companion",
        );
        let id = pocketstation::create(&root, "assets/PocketStation/Test.epokasset", &app).unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let mut workspace = Workspace::default();
        let entry = workspace.tree(&root, &index).unwrap().remove(0);
        workspace.open(&root, &entry.uri).unwrap();
        let (_, uri) = workspace.add_pocket_assembly(&root, &entry.uri).unwrap();
        assert_eq!(workspace.documents[&uri].language, "arm");
        let (_, second) = workspace.add_pocket_assembly(&root, &entry.uri).unwrap();
        assert_ne!(uri, second);
        let record = current_record(&index.resolve(id).unwrap().path).unwrap();
        let saved = pocketstation::document(&record).unwrap();
        assert_eq!(saved.files[0], app.files[0]);
        assert_eq!(saved.files.len(), app.files.len() + 2);
        pocketstation::build_app(&saved)
            .expect("Assembly functions remain linked through the real PocketLua compiler");
        fs::remove_dir_all(root).unwrap();
    }
}
