//! Portable imported assets. Packages are authoritative; the index and conversions are disposable.
use crate::audio_import::{self, Settings};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::SystemTime,
};
use uuid::Uuid;

const MAGIC: &[u8; 8] = b"EPOKAS01";
const MAX_METADATA_BYTES: usize = 65536;
pub const MAX_SOUNDFONT_SOURCE: usize = 256 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    pub version: u32,
    pub id: Uuid,
    pub kind: Kind,
    pub importer_version: u32,
    pub source: String,
    pub source_hash: String,
    #[serde(default)]
    pub settings: crate::import_settings::Settings,
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Kind {
    Texture,
    AudioClip,
    MusicSequence,
    SoundBank,
    EditableMesh,
    Terrain,
    ModelSource,
    Skeleton,
    SkeletalMesh,
    AnimationClip,
    Material,
    Font,
}
impl Kind {
    pub fn playable(&self) -> bool {
        matches!(self, Self::AudioClip | Self::MusicSequence)
    }
    /// Runtime references historically called playable handles AudioClip. Keep
    /// those serialized graphs valid; SoundBank sample references remain strict.
    pub fn runtime_reference(name: &str) -> Result<Self, String> {
        if name == "PlayableAudio" {
            return Ok(Self::AudioClip);
        }
        serde_json::from_value(serde_json::Value::String(name.into()))
            .map_err(|_| format!("Unknown asset kind {name}"))
    }
    pub fn accepts_runtime(&self, actual: &Self) -> bool {
        self == actual || (*self == Self::AudioClip && actual.playable())
    }
}
pub struct Package {
    pub meta: Metadata,
    pub source: Vec<u8>,
}
#[derive(Clone, Debug)]
pub struct Record {
    pub path: PathBuf,
    pub meta: Metadata,
    pub revision: String,
}
#[derive(Clone, Debug)]
pub struct Source {
    pub path: String,
    pub hash: String,
}
#[derive(Clone, Debug, Default)]
pub struct Index {
    pub assets: BTreeMap<Uuid, Vec<Record>>,
    pub sources: BTreeMap<String, Source>,
    pub native: Vec<PathBuf>,
    pub problems: Vec<String>,
}
impl Index {
    pub fn fingerprint(&self) -> String {
        hash(
            self.assets
                .iter()
                .map(|(id, records)| {
                    format!(
                        "{id}:{}:{}",
                        records.len(),
                        records
                            .iter()
                            .map(|r| cache_key(&r.meta))
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                })
                .collect::<Vec<_>>()
                .join(";")
                .as_bytes(),
        )
    }
    pub fn resolve(&self, id: Uuid) -> Result<&Record, String> {
        match self.assets.get(&id).map(Vec::as_slice) {
            Some([record]) => Ok(record),
            Some(_) => Err(format!(
                "Conflicting asset UUID {id}: resolve the duplicate files in Imports."
            )),
            None => Err(format!(
                "Missing asset {id}: restore its .epokasset file or replace the reference."
            )),
        }
    }
    pub fn usable(&self) -> impl Iterator<Item = &Record> {
        self.assets
            .values()
            .filter_map(|v| if v.len() == 1 { v.first() } else { None })
    }
    pub fn linked_source(&self, record: &Record) -> Result<Option<&Source>, String> {
        if let Some(parts) = crate::bank_compat::parts(record) {
            // Multipart recovery is explicit: never match a combined hash to
            // one file or silently choose another waveform companion.
            return Ok(parts.first().and_then(|p| self.sources.get(&p.path)));
        }
        if !matches!(
            record.meta.kind,
            Kind::AudioClip
                | Kind::MusicSequence
                | Kind::SoundBank
                | Kind::ModelSource
                | Kind::Texture
                | Kind::Font
        ) {
            return Ok(None);
        }
        if let Some(source) = self.sources.get(&record.meta.source) {
            return Ok(Some(source));
        }
        let matches = self
            .sources
            .values()
            .filter(|s| s.hash == record.meta.source_hash)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Ok(None),
            [one] => Ok(Some(*one)),
            _ => Err("Several source files match. Use Locate source to choose one.".into()),
        }
    }
}
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
/// Import-form routing only. Complete profile validation happens in the source
/// parser; extensions merely nominate files for discovery.
pub fn audio_source_kind(root: &Path, source: &str) -> Option<Kind> {
    let path = inside(root, source).ok()?;
    let mut header = [0u8; 12];
    let mut file = fs::File::open(&path).ok()?;
    file.read_exact(&mut header).ok()?;
    if crate::sf2::has_header(&header) {
        return Some(Kind::SoundBank);
    }
    let bytes = read_bounded(&path).ok()?;
    if crate::bank_compat::has_header(&bytes) {
        Some(Kind::SoundBank)
    } else if bytes.starts_with(b"MThd")
        || bytes.starts_with(b"pQES")
        || crate::sequence::catalog_source(&bytes, None).is_ok()
    {
        Some(Kind::MusicSequence)
    } else {
        None
    }
}
pub fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let mut header = [0u8; 16];
    let read = fs::File::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .read(&mut header)
        .map_err(|e| e.to_string())?;
    if read >= 8 && &header[..8] == MAGIC {
        return read_package(path);
    }
    if crate::sf2::has_header(&header[..read]) {
        return read_soundfont_bounded(path);
    }
    // Historical callers use this for both raw sources and complete legacy packages.
    // Preserve its 64 KiB package-header allowance without raising audio source limits.
    read_bounded_limit(path, audio_import::MAX_SOURCE + MAX_METADATA_BYTES)
}
pub fn read_soundfont_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let bytes = read_bounded_limit(path, MAX_SOUNDFONT_SOURCE)?;
    if !crate::sf2::has_header(&bytes) {
        return Err("Expected a RIFF sfbk SoundFont source".into());
    }
    Ok(bytes)
}
fn read_bounded_limit(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    fs::File::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .take((limit + 1) as u64)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > limit {
        return Err("Asset exceeds the supported file size".into());
    }
    Ok(data)
}
fn is_soundfont_library(meta: &Metadata) -> bool {
    meta.kind == Kind::SoundBank
        && meta
            .settings
            .sound_bank()
            .is_ok_and(|settings| settings.library.is_some())
}
fn source_limit(meta: &Metadata) -> usize {
    if is_soundfont_library(meta) {
        MAX_SOUNDFONT_SOURCE
    } else {
        audio_import::MAX_SOURCE
    }
}
fn read_package(path: &Path) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut header = [0u8; 16];
    file.read_exact(&mut header)
        .map_err(|_| "Invalid Epok asset header")?;
    if &header[..8] != MAGIC {
        return Err("Invalid Epok asset header".into());
    }
    let meta_size = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
    let source_size = u32::from_le_bytes(header[12..16].try_into().unwrap()) as usize;
    if meta_size > MAX_METADATA_BYTES {
        return Err("Incomplete or oversized asset package".into());
    }
    let mut metadata_bytes = vec![0; meta_size];
    file.read_exact(&mut metadata_bytes)
        .map_err(|_| "Incomplete or oversized asset package")?;
    let metadata: Metadata =
        serde_json::from_slice(&metadata_bytes).map_err(|e| format!("Asset metadata: {e}"))?;
    if source_size > source_limit(&metadata) {
        return Err("Incomplete or oversized asset package".into());
    }
    let expected = 16usize
        .checked_add(meta_size)
        .and_then(|n| n.checked_add(source_size))
        .ok_or("Incomplete or oversized asset package")?;
    if file.metadata().map_err(|e| e.to_string())?.len() != expected as u64 {
        return Err("Incomplete or oversized asset package".into());
    }
    let mut bytes = Vec::with_capacity(expected);
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&metadata_bytes);
    bytes.resize(expected, 0);
    file.read_exact(&mut bytes[16 + meta_size..])
        .map_err(|_| "Incomplete or oversized asset package")?;
    Ok(bytes)
}
impl Package {
    pub fn bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut metadata = self.meta.clone();
        // All writes use the typed settings schema; old packages are upgraded on write.
        metadata.version = 2;
        let meta = serde_json::to_vec(&metadata).map_err(|e| e.to_string())?;
        if meta.len() > MAX_METADATA_BYTES {
            return Err("Asset metadata is too large".into());
        }
        if self.source.len() > source_limit(&self.meta) || self.source.len() > u32::MAX as usize {
            return Err("Asset source exceeds the supported file size".into());
        }
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&(meta.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.source.len() as u32).to_le_bytes());
        bytes.extend(meta);
        bytes.extend_from_slice(&self.source);
        Ok(bytes)
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        Self::parse(&read_package(path)?)
    }
    fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 16 || &bytes[..8] != MAGIC {
            return Err("Invalid Epok asset header".into());
        }
        let meta_size = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let source_size = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        if meta_size > MAX_METADATA_BYTES || bytes.len() < 16 + meta_size {
            return Err("Incomplete or oversized asset package".into());
        }
        let meta: Metadata = serde_json::from_slice(&bytes[16..16 + meta_size])
            .map_err(|e| format!("Asset metadata: {e}"))?;
        let expected = 16usize
            .checked_add(meta_size)
            .and_then(|n| n.checked_add(source_size))
            .ok_or("Incomplete or oversized asset package")?;
        if source_size > source_limit(&meta) || expected != bytes.len() {
            return Err("Incomplete or oversized asset package".into());
        }
        let package = Self {
            meta,
            source: bytes[16 + meta_size..].to_vec(),
        };
        package.validate()?;
        Ok(package)
    }
    fn validate(&self) -> Result<(), String> {
        if ![1, 2].contains(&self.meta.version)
            || !(1..=audio_import::IMPORTER_VERSION).contains(&self.meta.importer_version)
            || self.meta.id.is_nil()
            || self.source.len() > source_limit(&self.meta)
        {
            return Err("Unsupported asset/importer version or invalid identity".into());
        }
        if !matches!(
            self.meta.kind,
            Kind::AudioClip | Kind::MusicSequence | Kind::SoundBank
        ) && self.meta.importer_version != 1
        {
            return Err("Unsupported model/mesh importer version".into());
        }
        if matches!(self.meta.kind, Kind::MusicSequence | Kind::SoundBank)
            && self.meta.importer_version > 2
        {
            return Err("Unsupported sequence/bank importer version".into());
        }
        match (&self.meta.kind, &self.meta.settings) {
            (Kind::Texture, crate::import_settings::Settings::Texture) => {}
            (Kind::AudioClip, crate::import_settings::Settings::Audio(s)) => s.validate()?,
            (Kind::MusicSequence, crate::import_settings::Settings::MusicSequence(s)) => {
                s.validate()?
            }
            (Kind::SoundBank, crate::import_settings::Settings::SoundBank(s)) => s.validate()?,
            (
                Kind::EditableMesh,
                crate::import_settings::Settings::Audio(_)
                | crate::import_settings::Settings::Authored,
            ) => {}
            (Kind::Terrain, crate::import_settings::Settings::Authored) => {}
            (Kind::ModelSource, crate::import_settings::Settings::Fbx(s)) => s.validate()?,
            (Kind::Font, crate::import_settings::Settings::Font(s)) => s.validate()?,
            (
                Kind::Skeleton | Kind::SkeletalMesh | Kind::AnimationClip | Kind::Material,
                crate::import_settings::Settings::Derived,
            ) => {}
            _ => return Err("Importer settings do not match asset kind".into()),
        }
        relative(&self.meta.source)?;
        if hash(&self.source) != self.meta.source_hash {
            return Err("Asset source checksum mismatch".into());
        }
        match self.meta.kind {
            Kind::Texture => {
                crate::texture::decode(&self.source)?;
            }
            Kind::AudioClip => crate::audio_decode::probe(&self.source)?,
            Kind::MusicSequence => {
                crate::sequence::decode_source(&self.source, self.meta.settings.sequence()?)?;
            }
            Kind::SoundBank => {
                let settings = self.meta.settings.sound_bank()?;
                if settings.library.is_some() {
                    crate::soundfont_asset::decode(self)?;
                } else if settings.imported.is_some() {
                    crate::bank_compat::decode(self)?;
                } else {
                    let source: crate::sound_bank::Settings =
                        crate::document::from_slice(&self.source).map_err(|e| e.to_string())?;
                    source.validate()?;
                    if source.imported.is_some() || source.library.is_some() {
                        return Err(
                            "Imported SoundBank requires an original source snapshot".into()
                        );
                    }
                }
            }
            Kind::ModelSource => {
                if self.source.is_empty() {
                    return Err("Empty FBX snapshot".into());
                }
            }
            Kind::Skeleton | Kind::SkeletalMesh | Kind::AnimationClip | Kind::Material => {
                if crate::skeletal::Data::parse(&self.source)?.kind() != self.meta.kind {
                    return Err("Asset kind/payload mismatch".into());
                }
            }
            Kind::EditableMesh => {
                crate::mesh::Document::parse(&self.source)?;
            }
            Kind::Terrain => {
                crate::terrain::Document::parse(&self.source)?;
            }
            Kind::Font => {
                crate::font_asset::decode(&self.source, self.meta.settings.font()?)?;
            }
        }
        Ok(())
    }
}
fn relative(value: &str) -> Result<&Path, String> {
    let path = Path::new(value);
    if value.contains('\0')
        || !path.starts_with("assets")
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("Choose a project-relative path inside assets/.".into());
    }
    Ok(path)
}
pub(crate) fn validate_source_path(value: &str) -> Result<(), String> {
    relative(value).map(|_| ())
}
pub fn inside(root: &Path, value: &str) -> Result<PathBuf, String> {
    let path = root.join(relative(value)?);
    let canonical_root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let mut parent = path.as_path();
    while !parent.exists() {
        parent = parent.parent().ok_or("Invalid asset path")?;
    }
    if !fs::canonicalize(parent)
        .map_err(|e| e.to_string())?
        .starts_with(canonical_root)
    {
        return Err("Asset paths cannot follow links outside the project".into());
    }
    Ok(path)
}
pub fn path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[derive(Clone, PartialEq)]
pub struct Stamp {
    size: u64,
    modified: Option<SystemTime>,
}
#[derive(Clone)]
enum Cached {
    Asset(Box<Record>),
    Source(Source),
    Native(PathBuf),
    Error(String),
}
#[derive(Clone, Default)]
pub struct ScanCache(BTreeMap<PathBuf, (Stamp, Cached)>);
fn walk(directory: &Path, files: &mut Vec<PathBuf>, errors: &mut Vec<String>) {
    let entries = match fs::read_dir(directory) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("{}: {e}", directory.display()));
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                errors.push(e.to_string());
                continue;
            }
        };
        let path = entry.path();
        // DirEntry metadata does not follow links and, on Windows, reuses the
        // directory enumeration data instead of opening every asset again.
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                errors.push(e.to_string());
                continue;
            }
        };
        // Junctions and symlinks are deliberately excluded: no cycles or assets outside the root.
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                errors.push(format!("Linked asset path skipped: {}", path.display()));
                continue;
            }
        }
        if metadata.file_type().is_symlink() {
            errors.push(format!("Linked asset path skipped: {}", path.display()));
            continue;
        }
        if metadata.is_dir() {
            walk(&path, files, errors);
        } else if metadata.is_file() {
            files.push(path);
        }
    }
}
/// Discover authored documents without opening or hashing unrelated media.
/// Keep the asset walk's link exclusions and partial traversal diagnostics.
pub(crate) fn source_paths(root: &Path, suffix: &str) -> (Vec<PathBuf>, Vec<String>) {
    let mut paths = Vec::new();
    let mut errors = Vec::new();
    let directory = root.join("assets");
    if directory.is_dir() {
        walk(&directory, &mut paths, &mut errors);
    }
    paths.retain(|path| path.to_string_lossy().ends_with(suffix));
    paths.sort();
    (paths, errors)
}

pub fn scan(root: &Path, cache: &mut ScanCache) -> Index {
    let mut index = Index::default();
    let mut files = Vec::new();
    if root.join("assets").is_dir() {
        walk(&root.join("assets"), &mut files, &mut index.problems);
    }
    let mut seen = BTreeSet::new();
    for path in files {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let name = path.file_name().unwrap().to_string_lossy();
        if ![
            "epokasset",
            "wav",
            "mid",
            "midi",
            "seq",
            "sep",
            "vab",
            "vh",
            "vb",
            "sf2",
            "sf3",
            "mp3",
            "flac",
            "ogg",
            "fbx",
            "png",
            "ttf",
            "otf",
            "cpp",
            "hpp",
        ]
        .contains(&ext.as_str())
            && !name.ends_with(".epokmap")
            && !name.ends_with(".epokbp")
            && !name.ends_with(".timeline.json")
            && !name.ends_with(".particle-effect.json")
        {
            continue;
        }
        seen.insert(path.clone());
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                index.problems.push(e.to_string());
                continue;
            }
        };
        let stamp = Stamp {
            size: metadata.len(),
            modified: metadata.modified().ok(),
        };
        let cached = if let Some((previous, value)) = cache
            .0
            .get(&path)
            .filter(|(s, _)| s == &stamp && ext != "cpp" && ext != "hpp")
        {
            let _ = previous;
            value.clone()
        } else {
            let result = (|| -> Result<Cached, String> {
                if ext == "epokasset" {
                    let bytes = read_package(&path)?;
                    let package = Package::parse(&bytes)?;
                    Ok(Cached::Asset(Box::new(Record {
                        path: path.clone(),
                        meta: package.meta,
                        revision: hash(&bytes),
                    })))
                } else if [
                    "wav", "mp3", "flac", "ogg", "mid", "midi", "seq", "sep", "vab", "vh", "vb",
                    "sf2", "sf3", "fbx", "png", "ttf", "otf",
                ]
                .contains(&ext.as_str())
                {
                    let bytes = if ext == "sf2" || ext == "sf3" {
                        read_soundfont_bounded(&path)?
                    } else {
                        read_bounded(&path)?
                    };
                    Ok(Cached::Source(Source {
                        path: path_string(root, &path),
                        hash: hash(&bytes),
                    }))
                } else if name.ends_with(".epokbp")
                    || name.ends_with(".timeline.json")
                    || name.ends_with(".particle-effect.json")
                {
                    // Broken visual sources stay visible so they can be repaired in the editor.
                    Ok(Cached::Native(path.clone()))
                } else if name.ends_with(".epokmap") {
                    crate::scene::Scene::load_unresolved(&path)?;
                    Ok(Cached::Native(path.clone()))
                } else {
                    // Index source files independently of reflection success so users can
                    // open and repair invalid declarations from the Project browser.
                    let stem = path.file_stem().unwrap().to_string_lossy();
                    let descriptor = path.with_file_name(format!("{stem}.epokscript"));
                    if !descriptor.is_file() {
                        return Ok(Cached::Native(path.clone()));
                    }
                    let script: crate::scripts::Script = crate::document::from_slice(
                        &fs::read(descriptor).map_err(|e| e.to_string())?,
                    )
                    .map_err(|e| e.to_string())?;
                    if script.name != stem || !crate::scripts::identifier(&script.name) {
                        return Err("Invalid script descriptor".into());
                    }
                    Ok(Cached::Native(path.clone()))
                }
            })();
            let value = result
                .unwrap_or_else(|e| Cached::Error(format!("{}: {e}", path_string(root, &path))));
            cache.0.insert(path.clone(), (stamp, value.clone()));
            value
        };
        match cached {
            Cached::Asset(record) => index
                .assets
                .entry(record.meta.id)
                .or_default()
                .push(*record),
            Cached::Source(source) => {
                index.sources.insert(source.path.clone(), source);
            }
            Cached::Native(path) => index.native.push(path),
            Cached::Error(error) => index.problems.push(error),
        }
    }
    cache.0.retain(|path, _| seen.contains(path));
    index.native.sort();
    index
}

pub fn atomic_write(path: &Path, bytes: &[u8], expected: Option<&str>) -> Result<(), String> {
    let parent = path.parent().ok_or("No parent directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = parent.join(format!(".{}.tmp", Uuid::new_v4()));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        file.write_all(bytes).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        match expected {
            Some(revision) => {
                let existing = if path
                    .extension()
                    .is_some_and(|extension| extension == "epokasset")
                {
                    read_package(path)?
                } else {
                    read_bounded(path)?
                };
                if hash(&existing) != revision {
                    return Err(
                        "Asset changed during the operation; retry against the latest version."
                            .into(),
                    );
                }
                fs::rename(&temp, path).map_err(|e| e.to_string())?;
            }
            None => {
                // Publish without replacing any pre-existing destination, even on Unix.
                publish_new(&temp, path).map_err(|e| {
                    format!("Destination already exists or cannot be published: {e}")
                })?;
            }
        }
        Ok(())
    })();
    let _ = fs::remove_file(temp);
    result
}
#[cfg(windows)]
fn publish_new(temp: &Path, path: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    }
    let from = temp
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let to = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    // No REPLACE_EXISTING or cross-volume fallback: the sibling temporary must publish
    // as one rename and never overwrite an existing asset, including on FAT/exFAT.
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 8) } == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
#[cfg(not(windows))]
fn publish_new(temp: &Path, path: &Path) -> std::io::Result<()> {
    fs::hard_link(temp, path)
}
pub fn cache_key(meta: &Metadata) -> String {
    if matches!(meta.kind, Kind::MusicSequence | Kind::SoundBank) {
        return hash(
            format!(
                "portable-audio-v1:{:?}:{}:{}:{}",
                meta.kind,
                meta.importer_version,
                meta.source_hash,
                serde_json::to_string(&meta.settings).unwrap()
            )
            .as_bytes(),
        );
    }
    hash(
        format!(
            "{}-psxavenc-0.3.1:{:?}:{}:{}:{}",
            if meta.kind == Kind::AudioClip {
                audio_import::PSX_PROFILE
            } else {
                "psx-audio-v2"
            },
            meta.kind,
            meta.importer_version,
            meta.source_hash,
            serde_json::to_string(&meta.settings).unwrap()
        )
        .as_bytes(),
    )
}
pub fn derived(root: &Path, package: &Package) -> Result<Vec<u8>, String> {
    let dir = root
        .join(".epok/imported")
        .join(cook_key(root, &package.meta)?);
    let music = package.meta.settings.audio()?.is_streamed();
    let path = dir.join(if music { "audio.xa" } else { "audio.adpcm" });
    let limit = if music {
        crate::music::MAX_BYTES
    } else {
        audio_import::SPU_BUDGET
    };
    let cached = fs::metadata(&path)
        .ok()
        .filter(|m| m.len() <= limit as u64)
        .ok_or("Cache missing")
        .and_then(|_| fs::read(&path).map_err(|_| "Cache unreadable"));
    if let (Ok(bytes), Ok(checksum)) = (cached, fs::read_to_string(dir.join("checksum")))
        && hash(&bytes) == checksum
        && bytes.len() <= limit
        && bytes
            .len()
            .is_multiple_of(if music { crate::music::SECTOR } else { 16 })
        && bytes.len() >= 48
    {
        return Ok(bytes);
    }
    let converted = if music {
        crate::music::convert(root, &package.source, package.meta.settings.audio()?)?
    } else {
        audio_import::convert(&package.source, package.meta.settings.audio()?)?
    };
    let mut target = package.meta.settings.audio()?.target_result();
    target.encoded_bytes = Some(converted.adpcm.len() as u64);
    if !music {
        let aligned = converted.adpcm.len().div_ceil(64) as u64 * 64;
        target.main_ram_bytes = Some(aligned);
        target.spu_ram_bytes = Some(aligned);
    }
    let report=crate::document::to_vec(&serde_json::json!({"source":converted.info,"waveform":converted.waveform,"bytes":converted.adpcm.len(),"target":target})).map_err(|e|e.to_string())?;
    let data = converted.adpcm;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // Each import owns its key. Atomic rename prevents readers observing partial output.
    replace_cache(&path, &data)?;
    replace_cache(&dir.join("checksum"), hash(&data).as_bytes())?;
    replace_cache(&dir.join("Import.epokcache"), &report)?;
    // A small last-cook report lets the UI display measured costs without hashing
    // an encoder executable or starting a conversion on the rendering thread.
    let summary = root.join(".epok/imported").join(cache_key(&package.meta));
    fs::create_dir_all(&summary).map_err(|e| e.to_string())?;
    replace_cache(&summary.join("Import.epokcache"), &report)?;
    Ok(data)
}
/// Target cache identity includes the implementation and the actual selected encoder binary.
/// Authoring/dependency identity stays independent of tool installation and machine paths.
pub fn cook_key(root: &Path, meta: &Metadata) -> Result<String, String> {
    if is_soundfont_library(meta) {
        return Err(crate::soundfont_asset::PLAYBACK_BLOCKER.into());
    }
    if matches!(meta.kind, Kind::MusicSequence | Kind::SoundBank) {
        return crate::psx_sequence::cook_key(root, meta);
    }
    static IMPLEMENTATION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let implementation = IMPLEMENTATION.get_or_init(|| {
        let mut hash = Sha256::new();
        for source in [
            include_bytes!("audio_import.rs").as_slice(),
            include_bytes!("audio_ir.rs").as_slice(),
            include_bytes!("audio_decode.rs").as_slice(),
            include_bytes!("music.rs").as_slice(),
            include_bytes!("../Cargo.lock").as_slice(),
        ] {
            hash.update(source);
        }
        format!("{:x}", hash.finalize())
    });
    let tool = if meta.settings.audio()?.is_streamed() {
        let path = crate::disc::tool(root, "psxavenc")?;
        let path = crate::dependencies::find_executable(&path.to_string_lossy())
            .ok_or("Selected psxavenc executable disappeared")?;
        let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut hash = Sha256::new();
        let mut buffer = [0; 65536];
        loop {
            let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hash.update(&buffer[..n]);
        }
        format!("{:x}", hash.finalize())
    } else {
        "builtin".into()
    };
    Ok(hash(
        format!("psx:{}:{implementation}:{tool}", cache_key(meta)).as_bytes(),
    ))
}
pub(crate) fn replace_cache(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temp = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    fs::write(&temp, bytes).map_err(|e| e.to_string())?;
    let result = fs::rename(&temp, path).map_err(|e| e.to_string());
    let _ = fs::remove_file(temp);
    result
}
pub struct Candidate {
    pub destination: PathBuf,
    pub package: Package,
    pub expected: Option<String>,
    pub source_path: Option<PathBuf>,
    pub source_hash: String,
}
// Recover only a unique unchanged source whose old path disappeared. Publish on the owning
// session, never from a detached scanner. Persist the new path so later edits remain linked.
pub fn source_relinks(root: &Path, index: &Index) -> Vec<Candidate> {
    index
        .usable()
        .filter_map(|record| {
            if crate::bank_compat::parts(record).is_some() {
                return None;
            }
            let source = index.linked_source(record).ok().flatten()?;
            if source.path == record.meta.source || source.hash != record.meta.source_hash {
                return None;
            }
            let mut package = Package::load(&record.path).ok()?;
            package.meta.source = source.path.clone();
            Some(Candidate {
                destination: record.path.clone(),
                package,
                expected: Some(record.revision.clone()),
                source_path: Some(root.join(&source.path)),
                source_hash: source.hash.clone(),
            })
        })
        .collect()
}
pub fn prepare(
    root: &Path,
    source: &str,
    destination: &str,
    settings: Settings,
    existing: Option<&Record>,
    use_snapshot: bool,
) -> Result<Candidate, String> {
    let candidate = prepare_portable(
        root,
        source,
        destination,
        crate::import_settings::Settings::Audio(settings),
        existing,
        use_snapshot,
    )?;
    let s = candidate.package.meta.settings.audio()?;
    crate::audio_ir::DecodedAudioIr::decode(&candidate.package.source)?.with_edits(
        s.trim_start,
        s.trim_end,
        s.normalize,
        s.looping,
    )?;
    Ok(candidate)
}

/// Shared original-snapshot/UUID transaction for sampled and sequenced audio.
pub fn prepare_portable(
    root: &Path,
    source: &str,
    destination: &str,
    settings: crate::import_settings::Settings,
    existing: Option<&Record>,
    use_snapshot: bool,
) -> Result<Candidate, String> {
    let (kind, importer_version) = match &settings {
        crate::import_settings::Settings::Audio(_) => {
            (Kind::AudioClip, audio_import::IMPORTER_VERSION)
        }
        crate::import_settings::Settings::MusicSequence(s) => (
            Kind::MusicSequence,
            if s.source_selection.is_some() { 2 } else { 1 },
        ),
        crate::import_settings::Settings::SoundBank(s) => {
            if s.library.is_some() {
                return Err(
                    "Use soundfont_asset::prepare to import an authoritative SF2/SF3 snapshot"
                        .into(),
                );
            }
            if s.imported.is_some() {
                return Err("Use the Sony SoundBank import transaction to preserve and verify all source parts".into());
            }
            (Kind::SoundBank, 1)
        }
        _ => return Err("Expected sampled audio, MusicSequence or SoundBank settings".into()),
    };
    if existing.is_some_and(|r| r.meta.kind != kind) {
        return Err(
            "Reimport cannot change the asset kind. Import to a new destination instead.".into(),
        );
    }
    let destination = inside(root, destination)?;
    if destination.extension().is_none_or(|e| e != "epokasset") {
        return Err("Asset destination must end in .epokasset".into());
    }
    let old = existing.map(|r| Package::load(&r.path)).transpose()?;
    let source_path = if use_snapshot {
        None
    } else {
        Some(inside(root, source)?)
    };
    let bytes = if let Some(path) = &source_path {
        read_bounded(path)?
    } else {
        old.as_ref()
            .ok_or("No imported source snapshot")?
            .source
            .clone()
    };
    let source_hash = hash(&bytes);
    let package = Package {
        meta: Metadata {
            version: 1,
            id: existing.map_or_else(Uuid::new_v4, |r| r.meta.id),
            kind,
            importer_version,
            source: if use_snapshot {
                old.as_ref().unwrap().meta.source.clone()
            } else {
                source.replace('\\', "/")
            },
            source_hash: source_hash.clone(),
            settings,
            extra: old
                .as_ref()
                .map(|p| p.meta.extra.clone())
                .unwrap_or_default(),
        },
        source: bytes,
    };
    package.validate()?;
    Ok(Candidate {
        destination,
        package,
        expected: existing.map(|r| r.revision.clone()),
        source_path,
        source_hash,
    })
}
pub fn commit(candidate: Candidate) -> Result<Uuid, String> {
    if let Some(path) = &candidate.source_path
        && hash(
            &(if is_soundfont_library(&candidate.package.meta) {
                read_soundfont_bounded(path)?
            } else {
                read_bounded(path)?
            }),
        ) != candidate.source_hash
    {
        return Err("Source changed during import. The previous asset is intact; retry.".into());
    }
    let id = candidate.package.meta.id;
    atomic_write(
        &candidate.destination,
        &candidate.package.bytes()?,
        candidate.expected.as_deref(),
    )?;
    Ok(id)
}
pub fn move_asset(root: &Path, record: &Record, destination: &str) -> Result<(), String> {
    let destination = inside(root, destination)?;
    if destination.extension().is_none_or(|e| e != "epokasset") {
        return Err("Keep the .epokasset extension".into());
    }
    let bytes = read_package(&record.path)?;
    if hash(&bytes) != record.revision {
        return Err("Asset changed; refresh before moving".into());
    }
    atomic_write(&destination, &bytes, None)?;
    // A failure leaves two detectable copies, never destroys the only valid package.
    if hash(&read_package(&record.path)?) != record.revision {
        return Err(
            "Source changed while moving. Both files were retained; resolve the duplicate.".into(),
        );
    }
    fs::remove_file(&record.path)
        .map_err(|e| format!("Move created the destination but could not remove the original: {e}"))
}
pub fn duplicate(record: &Record, destination: &Path) -> Result<Uuid, String> {
    if record.meta.kind == Kind::ModelSource {
        return Err(
            "Import the FBX into a new .imported folder to duplicate a complete model".into(),
        );
    }
    let mut package = Package::load(&record.path)?;
    package.meta.id = Uuid::new_v4();
    atomic_write(destination, &package.bytes()?, None)?;
    Ok(package.meta.id)
}
pub fn make_independent(record: &Record) -> Result<Uuid, String> {
    if matches!(
        record.meta.kind,
        Kind::ModelSource
            | Kind::Skeleton
            | Kind::SkeletalMesh
            | Kind::AnimationClip
            | Kind::Material
    ) {
        return Err("For model assets, remove the conflicting copy or import the FBX into a new folder to create independent identities".into());
    }
    let mut package = Package::load(&record.path)?;
    package.meta.id = Uuid::new_v4();
    atomic_write(&record.path, &package.bytes()?, Some(&record.revision))?;
    Ok(package.meta.id)
}

pub fn dependencies(root: &Path, id: Uuid) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    let mut errors = Vec::new();
    walk(&root.join("assets"), &mut files, &mut errors);
    let mut uses = Vec::new();
    for path in files
        .into_iter()
        .filter(|p| p.to_string_lossy().ends_with(".epokmap"))
    {
        let scene = crate::scene::Scene::load_unresolved(&path)?;
        for entity in &scene.actors {
            if entity.audio.as_ref().is_some_and(|a| a.clip == Some(id))
                || entity.material.texture == Some(id)
                || entity.image.as_ref().is_some_and(|i| i.texture == Some(id))
                || entity
                    .sprite
                    .as_ref()
                    .is_some_and(|i| i.texture == Some(id))
                || entity
                    .particle_emitter
                    .as_ref()
                    .is_some_and(|i| i.sprite.texture == Some(id))
                || entity
                    .editable_mesh
                    .as_ref()
                    .is_some_and(|m| m.materials.values().any(|m| m.texture == Some(id)))
                || entity.editable_mesh.as_ref().is_some_and(|m| m.asset == id)
                || entity
                    .skeletal_mesh
                    .as_ref()
                    .is_some_and(|m| m.asset == id || m.clip == Some(id))
            {
                uses.push(format!("{} / {}", path_string(root, &path), entity.name));
            }
        }
    }
    let index = scan(root, &mut Default::default());
    if !index.problems.is_empty() {
        return Err(index.problems.join("\n"));
    }
    for record in index.usable() {
        let used = match &record.meta.settings {
            crate::import_settings::Settings::MusicSequence(settings) => {
                settings.sound_bank == Some(id)
            }
            crate::import_settings::Settings::SoundBank(settings) => {
                settings.dependencies().contains(&id)
            }
            _ => false,
        };
        if used {
            uses.push(path_string(root, &record.path));
        }
    }
    if crate::workspace::optional_manifest(root)?.is_some_and(|m| m.default_sound_bank == Some(id))
    {
        uses.push("Project Default SoundBank".into());
    }
    for record in index.usable().filter(|r| {
        matches!(
            r.meta.kind,
            Kind::SkeletalMesh
                | Kind::AnimationClip
                | Kind::ModelSource
                | Kind::EditableMesh
                | Kind::Material
        )
    }) {
        let package = Package::load(&record.path)?;
        let used = if package.meta.kind == Kind::EditableMesh {
            crate::mesh::Document::parse(&package.source)?
                .materials
                .iter()
                .any(|s| s.material.texture == Some(id))
        } else if let crate::import_settings::Settings::Fbx(s) = &package.meta.settings {
            s.outputs.values().any(|o| o.id == id)
        } else {
            match crate::skeletal::Data::parse(&package.source)? {
                crate::skeletal::Data::SkeletalMesh(m) => {
                    m.skeleton == id || m.materials.contains(&id) || m.clips.contains(&id)
                }
                crate::skeletal::Data::AnimationClip(c) => c.skeleton == id,
                crate::skeletal::Data::Material(m) => m.texture == Some(id),
                _ => false,
            }
        };
        if used {
            uses.push(path_string(root, &record.path));
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(uses)
}
pub fn trash(root: &Path, record: &Record) -> Result<PathBuf, String> {
    if !dependencies(root, record.meta.id)?.is_empty() {
        return Err(
            "This asset is referenced by saved scenes. Replace or remove those references first."
                .into(),
        );
    }
    let destination = root.join("UserSettings/AssetTrash").join(format!(
        "{}-{}.epokasset",
        record.meta.id,
        Uuid::new_v4()
    ));
    let bytes = read_package(&record.path)?;
    if hash(&bytes) != record.revision {
        return Err("Asset changed; refresh before deleting".into());
    }
    atomic_write(&destination, &bytes, None)?;
    if hash(&read_package(&record.path)?) != record.revision {
        return Err(
            "Asset changed during deletion. The trash copy was retained; refresh before retrying."
                .into(),
        );
    }
    fs::remove_file(&record.path).map_err(|e| e.to_string())?;
    Ok(destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("epok-assets-{}", Uuid::new_v4()));
            fs::create_dir_all(root.join("assets/scenes")).unwrap();
            fs::write(root.join("assets/tone.wav"), audio_import::test_wav()).unwrap();
            Self(root)
        }
        fn import(&self) -> Uuid {
            commit(
                prepare(
                    &self.0,
                    "assets/tone.wav",
                    "assets/tone.epokasset",
                    Settings::default(),
                    None,
                    false,
                )
                .unwrap(),
            )
            .unwrap()
        }
        fn index(&self) -> Index {
            scan(&self.0, &mut Default::default())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            assert!(
                self.0.starts_with(std::env::temp_dir())
                    && self
                        .0
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with("epok-assets-")
            );
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn offline_moves_missing_sources_and_disposable_cache_preserve_scene_references() {
        let f = Fixture::new();
        let id = f.import();
        assert!(!f.0.join(".epok/imported").exists());
        derived(
            &f.0,
            &Package::load(&f.0.join("assets/tone.epokasset")).unwrap(),
        )
        .unwrap();
        let mut scene = crate::scene::Scene::default();
        scene.actors[0].audio = Some(crate::audio::AudioSource {
            clip: Some(id),
            ..Default::default()
        });
        scene.actors[1].audio = scene.actors[0].audio.clone();
        let scene_path = f.0.join("assets/scenes/Main.epokmap");
        scene.save(&scene_path).unwrap();
        let original_scene = fs::read(&scene_path).unwrap();
        fs::create_dir_all(f.0.join("assets/Renamed")).unwrap();
        fs::rename(
            f.0.join("assets/tone.epokasset"),
            f.0.join("assets/Renamed/moved.epokasset"),
        )
        .unwrap();
        fs::remove_file(f.0.join("assets/tone.wav")).unwrap();
        fs::remove_dir_all(f.0.join(".epok")).unwrap();
        let record = f.index().resolve(id).unwrap().clone();
        assert_eq!(record.path, f.0.join("assets/Renamed/moved.epokasset"));
        crate::audio::stage(&f.0, &scene, &f.0.join("build"), &f.index()).unwrap();
        let bank = fs::read_to_string(f.0.join("build/audio-bank.hh")).unwrap();
        assert!(
            bank.contains("audio_clip_count = 1"),
            "Shared references must upload once"
        );
        assert_eq!(fs::read(&scene_path).unwrap(), original_scene);
        let candidate = prepare(
            &f.0,
            "",
            "assets/Renamed/moved.epokasset",
            Settings {
                sample_rate: 11025,
                ..Default::default()
            },
            Some(&record),
            true,
        )
        .unwrap();
        assert_eq!(commit(candidate).unwrap(), id);
        assert_eq!(
            f.index()
                .resolve(id)
                .unwrap()
                .meta
                .settings
                .audio()
                .unwrap()
                .sample_rate,
            11025
        );
        // Removing a package never rewrites the scene's persistent UUID.
        fs::remove_file(&record.path).unwrap();
        assert!(
            crate::audio::stage(&f.0, &scene, &f.0.join("build"), &f.index())
                .unwrap_err()
                .contains("Missing asset")
        );
        assert_eq!(
            crate::scene::Scene::load(&scene_path).unwrap().actors[0]
                .audio
                .as_ref()
                .unwrap()
                .clip,
            Some(id)
        );
    }
    #[test]
    fn certification_rechecks_used_packages_without_an_editor_scan_cache() {
        use crate::{
            artifact_dependencies::Graph,
            staging_files::{BuildTicket, Files},
        };
        let f = Fixture::new();
        let id = f.import();
        let asset_key = format!("asset:{id}");
        let package_path = f.0.join("assets/tone.epokasset");
        let original = fs::read(&package_path).unwrap();
        let mut scene = crate::scene::Scene::default();
        scene.actors[0].audio = Some(crate::audio::AudioSource {
            clip: Some(id),
            ..Default::default()
        });
        let stage = |name: &str, scene: &crate::scene::Scene| {
            let destination = f.0.join(name);
            crate::project::stage_into(&f.0, scene, &destination).unwrap();
            crate::staging_files::patch(&f.0, &destination, Files::new(), "release".into())
                .unwrap();
            destination
        };
        let build = stage(".epok/build", &scene);
        let export = stage("exports/audio", &scene);
        let independent = stage("exports/independent", &crate::scene::Scene::default());
        let pending = BuildTicket::begin(&f.0, &build).unwrap();
        // Raw import sources are not the authoritative package snapshot. An
        // unrelated broken package must not prevent certification either.
        fs::write(f.0.join("assets/tone.wav"), b"unimported edit").unwrap();
        fs::write(f.0.join("assets/unrelated.epokasset"), b"broken").unwrap();
        pending.complete(&f.0, &build, b"current package").unwrap();
        let pending = BuildTicket::begin(&f.0, &build).unwrap();
        let mut package = Package::load(&package_path).unwrap();
        package.meta.settings = crate::import_settings::Settings::Audio(Settings {
            sample_rate: 11025,
            ..Default::default()
        });
        fs::write(&package_path, package.bytes().unwrap()).unwrap();
        assert!(pending.complete(&f.0, &build, b"old import").is_err());
        assert!(crate::staging_files::publish_export(&f.0, &export, Files::new()).is_err());
        crate::staging_files::publish_export(&f.0, &independent, Files::new()).unwrap();
        let graph = Graph::load(&f.0).unwrap();
        assert!(
            graph.nodes["stage:.epok/build"]
                .stale
                .contains_key(&asset_key)
        );
        assert!(graph.nodes["stage:exports/independent"].stale.is_empty());
        fs::write(&package_path, &original).unwrap();
        assert!(BuildTicket::begin(&f.0, &build).is_err());
        stage(".epok/build", &scene);
        let pending = BuildTicket::begin(&f.0, &build).unwrap();
        // Equal-length payload corruption must fail checksum validation even
        // when the package metadata and its recorded cache key did not change.
        let mut corrupt = original.clone();
        *corrupt.last_mut().unwrap() ^= 1;
        fs::write(&package_path, corrupt).unwrap();
        assert!(pending.complete(&f.0, &build, b"corrupt package").is_err());
        fs::write(&package_path, &original).unwrap();
        stage(".epok/build", &scene);
        let duplicate = f.0.join("assets/duplicate.epokasset");
        fs::write(&duplicate, &original).unwrap();
        assert!(BuildTicket::begin(&f.0, &build).is_err());
        fs::remove_file(&duplicate).unwrap();
        assert!(BuildTicket::begin(&f.0, &build).is_err());
        stage(".epok/build", &scene);
        let pending = BuildTicket::begin(&f.0, &build).unwrap();
        fs::remove_file(&package_path).unwrap();
        assert!(pending.complete(&f.0, &build, b"removed package").is_err());
        fs::write(&package_path, &original).unwrap();
        stage(".epok/build", &scene);
        BuildTicket::begin(&f.0, &build)
            .unwrap()
            .complete(&f.0, &build, b"recovered")
            .unwrap();
        assert!(
            !Graph::load(&f.0).unwrap().nodes["stage:exports/audio"]
                .stale
                .is_empty()
        );
        stage("exports/audio", &scene);
        crate::staging_files::publish_export(&f.0, &export, Files::new()).unwrap();
    }

    #[test]
    fn staged_audio_payloads_follow_their_packages_and_not_scene_or_native_edits() {
        use crate::{artifact_dependencies::Graph, scene_dependencies::Input};
        let f = Fixture::new();
        let first = f.import();
        let second = commit(
            prepare(
                &f.0,
                "assets/tone.wav",
                "assets/second.epokasset",
                Settings::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        fs::create_dir_all(f.0.join("assets/scripts")).unwrap();
        let native_path = f.0.join("assets/scripts/Probe.cpp");
        fs::write(&native_path, b"int audio_fixture=1;\n").unwrap();
        let path = f.0.join("assets/scenes/Main.epokmap");
        let mut scene = crate::scene::Scene::default();
        for (entity, id) in scene.actors.iter_mut().zip([first, second]) {
            entity.audio = Some(crate::audio::AudioSource {
                clip: Some(id),
                ..Default::default()
            });
        }
        scene.save(&path).unwrap();
        let saved_bytes = fs::read(&path).unwrap();
        let build = f.0.join(".epok/build");
        let export = f.0.join("exports/audio");
        let stage = |scene: &crate::scene::Scene| {
            crate::project::stage_with_origin(
                &f.0,
                scene,
                &build,
                &Input::editor(path.clone(), scene.clone()).origin,
            )
            .unwrap();
        };
        stage(&scene);
        crate::project::stage_with_origin(
            &f.0,
            &scene,
            &export,
            &Input::load(&path).unwrap().origin,
        )
        .unwrap();
        let key = |target: &str, id: Uuid| format!("generated-resource:{target}/audio/{id}.adpcm");
        let before = Graph::load(&f.0).unwrap();
        for target in [".epok/build", "exports/audio"] {
            let bank = &before.nodes[&format!("generated-resource:{target}/audio-bank.hh")];
            assert!(bank.stale.is_empty());
            assert!(bank.dependencies.contains(&format!("asset:{first}")));
            assert!(bank.dependencies.contains(&format!("asset:{second}")));
            assert!(
                bank.dependencies
                    .iter()
                    .any(|key| key.starts_with("audio-selection:scene-"))
            );
            assert!(
                !bank
                    .dependencies
                    .iter()
                    .any(|key| key.starts_with("scene-file:") || key.starts_with("scene-editor:"))
            );
            assert_eq!(
                bank.signature,
                Some(hash(
                    &fs::read(f.0.join(target).join("audio-bank.hh")).unwrap()
                ))
            );
            for id in [first, second] {
                let node = &before.nodes[&key(target, id)];
                assert!(node.stale.is_empty());
                assert_eq!(
                    node.dependencies,
                    [format!("asset:{id}"), format!("audio-cook:{id}")]
                        .into_iter()
                        .collect()
                );
                assert_eq!(
                    node.signature,
                    Some(hash(
                        &fs::read(f.0.join(target).join(format!("audio/{id}.adpcm"))).unwrap()
                    ))
                );
            }
        }
        fs::write(&native_path, b"int audio_fixture=2;\n").unwrap();
        crate::staging_files::observe_native(&f.0).unwrap();
        scene.actors[1].position[0] += 2.;
        scene.actors[1].audio.as_mut().unwrap().volume = 0.25;
        scene.actors[1].audio.as_mut().unwrap().pitch = 0.5;
        assert!(crate::scene_dependencies::observe(&f.0, &path, &scene).unwrap());
        let unrelated = Graph::load(&f.0).unwrap();
        for target in [".epok/build", "exports/audio"] {
            let bank = format!("generated-resource:{target}/audio-bank.hh");
            assert_eq!(unrelated.nodes[&bank], before.nodes[&bank]);
            for id in [first, second] {
                assert_eq!(
                    unrelated.nodes[&key(target, id)],
                    before.nodes[&key(target, id)]
                );
            }
        }
        // Selecting a different playback asset can add/remove clips even when
        // the scene itself has no new AudioSource. Bindings and controls do not
        // determine which resources its timeline contributes.
        for effect in [false, true] {
            let mut playback_selection = scene.clone();
            if effect {
                playback_selection.actors[0].particle_effect =
                    Some(crate::particle_effect_scene::Component {
                        asset: Some(Uuid::new_v4()),
                        ..Default::default()
                    });
            } else {
                playback_selection.actors[0].timeline = Some(crate::timeline_scene::Component {
                    asset: Some(Uuid::new_v4()),
                    ..Default::default()
                });
            }
            crate::scene_dependencies::observe(&f.0, &path, &playback_selection).unwrap();
            let changed = Graph::load(&f.0).unwrap();
            assert!(
                changed.nodes["generated-resource:.epok/build/audio-bank.hh"]
                    .stale
                    .keys()
                    .any(|key| key.starts_with("audio-selection:scene-editor:"))
            );
            assert_eq!(
                changed.nodes["generated-resource:exports/audio/audio-bank.hh"],
                before.nodes["generated-resource:exports/audio/audio-bank.hh"]
            );
            stage(&scene);
        }
        let selected_index = f.index();
        let mut saved_scene = crate::scene::Scene::load(&path).unwrap();
        saved_scene.actors[1].audio.as_mut().unwrap().volume = 0.5;
        saved_scene.save(&path).unwrap();
        crate::scene_dependencies::observe(&f.0, &path, &scene).unwrap();
        let volume_only = Graph::load(&f.0).unwrap();
        let export_bank = "generated-resource:exports/audio/audio-bank.hh";
        let build_bank = "generated-resource:.epok/build/audio-bank.hh";
        assert_eq!(volume_only.nodes[export_bank], before.nodes[export_bank]);
        saved_scene.actors[1].audio = None;
        saved_scene.save(&path).unwrap();
        crate::scene_dependencies::observe(&f.0, &path, &scene).unwrap();
        let saved_selection = Graph::load(&f.0).unwrap();
        assert!(
            saved_selection.nodes[export_bank]
                .stale
                .keys()
                .any(|key| key.starts_with("audio-selection:scene-file:"))
        );
        assert_eq!(saved_selection.nodes[build_bank], before.nodes[build_bank]);
        fs::write(&path, &saved_bytes).unwrap();
        crate::scene_dependencies::observe(&f.0, &path, &scene).unwrap();
        assert!(
            !Graph::load(&f.0).unwrap().nodes[export_bank]
                .stale
                .is_empty()
        );
        let input = Input::load(&path).unwrap();
        crate::project::stage_with_origin(&f.0, &input.scene, &export, &input.origin).unwrap();
        let first_record = selected_index.resolve(first).unwrap().clone();
        commit(
            prepare(
                &f.0,
                "",
                "assets/tone.epokasset",
                Settings {
                    sample_rate: 11025,
                    ..Default::default()
                },
                Some(&first_record),
                true,
            )
            .unwrap(),
        )
        .unwrap();
        let error = crate::audio::stage(
            &f.0,
            &scene,
            &f.0.join("exports/stale-audio"),
            &selected_index,
        )
        .unwrap_err();
        assert!(error.contains("changed after resource selection"));
        crate::timeline_compile::observe_resources(&f.0, &f.index()).unwrap();
        let changed = Graph::load(&f.0).unwrap();
        for target in [".epok/build", "exports/audio"] {
            assert!(
                changed.nodes[&format!("generated-resource:{target}/audio-bank.hh")]
                    .stale
                    .contains_key(&format!("asset:{first}"))
            );
            assert!(
                changed.nodes[&key(target, first)]
                    .stale
                    .contains_key(&format!("asset:{first}"))
            );
            assert_eq!(
                changed.nodes[&key(target, second)],
                before.nodes[&key(target, second)]
            );
        }
        stage(&scene);
        let rebuilt = Graph::load(&f.0).unwrap();
        assert!(rebuilt.nodes[&key(".epok/build", first)].stale.is_empty());
        assert!(!rebuilt.nodes[&key("exports/audio", first)].stale.is_empty());
        let bank_key = "generated-resource:.epok/build/audio-bank.hh";
        let export_bank = "generated-resource:exports/audio/audio-bank.hh";
        assert!(rebuilt.nodes[bank_key].stale.is_empty());
        let mut changed_selection = scene.clone();
        changed_selection.actors[1].audio = None;
        assert!(crate::scene_dependencies::observe(&f.0, &path, &changed_selection).unwrap());
        let selection = Graph::load(&f.0).unwrap();
        assert!(
            selection.nodes[bank_key]
                .stale
                .keys()
                .any(|key| key.starts_with("audio-selection:scene-editor:"))
        );
        assert_eq!(selection.nodes[export_bank], rebuilt.nodes[export_bank]);
        stage(&scene);
        let old_payload = build.join(format!("audio/{second}.adpcm"));
        let old_bytes = fs::read(&old_payload).unwrap();
        fs::remove_file(f.index().resolve(second).unwrap().path.clone()).unwrap();
        crate::timeline_compile::observe_resources(&f.0, &f.index()).unwrap();
        assert!(crate::project::stage_into(&f.0, &scene, &build).is_err());
        assert_eq!(fs::read(&old_payload).unwrap(), old_bytes);
        scene.actors[1].audio = None;
        stage(&scene);
        let removed = Graph::load(&f.0).unwrap();
        assert!(!removed.nodes[&key(".epok/build", second)].stale.is_empty());
        assert!(
            !removed.nodes["stage:.epok/build"]
                .dependencies
                .contains(&format!("staged-file:.epok/build/audio/{second}.adpcm"))
        );
        assert_eq!(fs::read(&path).unwrap(), saved_bytes);
    }
    #[test]
    fn copied_ids_conflict_until_explicitly_made_independent() {
        let f = Fixture::new();
        let id = f.import();
        fs::copy(
            f.0.join("assets/tone.epokasset"),
            f.0.join("assets/copy.epokasset"),
        )
        .unwrap();
        let index = f.index();
        assert!(index.resolve(id).is_err());
        assert_eq!(index.usable().count(), 0);
        let copy = index.assets[&id]
            .iter()
            .find(|r| r.path.ends_with("copy.epokasset"))
            .unwrap();
        let new_id = make_independent(copy).unwrap();
        assert_ne!(id, new_id);
        let index = f.index();
        assert_eq!(index.usable().count(), 2);
        assert!(index.resolve(id).is_ok());
        let third = duplicate(
            index.resolve(new_id).unwrap(),
            &f.0.join("assets/third.epokasset"),
        )
        .unwrap();
        assert_ne!(third, id);
        assert_ne!(third, new_id);
    }
    #[test]
    fn source_relocation_is_persisted_only_when_unambiguous() {
        let f = Fixture::new();
        let id = f.import();
        fs::rename(f.0.join("assets/tone.wav"), f.0.join("assets/new.wav")).unwrap();
        fs::copy(f.0.join("assets/new.wav"), f.0.join("assets/other.wav")).unwrap();
        let index = f.index();
        assert!(index.linked_source(index.resolve(id).unwrap()).is_err());
        assert!(source_relinks(&f.0, &index).is_empty());
        fs::remove_file(f.0.join("assets/other.wav")).unwrap();
        let candidates = source_relinks(&f.0, &f.index());
        assert_eq!(candidates.len(), 1);
        for candidate in candidates {
            assert_eq!(commit(candidate).unwrap(), id);
        }
        assert_eq!(f.index().resolve(id).unwrap().meta.source, "assets/new.wav");
        let mut changed = audio_import::test_wav();
        changed[44] ^= 1;
        fs::write(f.0.join("assets/new.wav"), &changed).unwrap();
        let index = f.index();
        let record = index.resolve(id).unwrap();
        assert_eq!(
            index.linked_source(record).unwrap().unwrap().path,
            "assets/new.wav"
        );
        assert_ne!(
            index.linked_source(record).unwrap().unwrap().hash,
            record.meta.source_hash
        );
    }
    #[test]
    fn failed_or_stale_imports_never_replace_the_previous_asset() {
        let f = Fixture::new();
        let id = f.import();
        let record = f.index().resolve(id).unwrap().clone();
        let original = fs::read(&record.path).unwrap();
        let candidate = prepare(
            &f.0,
            "assets/tone.wav",
            "assets/tone.epokasset",
            Settings::default(),
            Some(&record),
            false,
        )
        .unwrap();
        fs::write(f.0.join("assets/tone.wav"), b"partial transfer").unwrap();
        assert!(commit(candidate).is_err());
        assert_eq!(fs::read(&record.path).unwrap(), original);
        assert!(
            prepare(
                &f.0,
                "assets/tone.wav",
                "assets/tone.epokasset",
                Settings::default(),
                Some(&record),
                false
            )
            .is_err()
        );
        assert_eq!(fs::read(&record.path).unwrap(), original);
        let candidate = prepare(
            &f.0,
            "",
            "assets/tone.epokasset",
            Settings::default(),
            Some(&record),
            true,
        )
        .unwrap();
        let independent = make_independent(&record).unwrap();
        assert!(commit(candidate).is_err());
        assert_eq!(Package::load(&record.path).unwrap().meta.id, independent);
        assert!(atomic_write(&record.path, b"replacement", None).is_err());
        assert!(inside(&f.0, "assets/../../escape.epokasset").is_err());
    }
    #[test]
    fn corrupted_packages_are_hidden_and_corrupted_conversions_regenerate() {
        let f = Fixture::new();
        let id = f.import();
        let index = f.index();
        let record = index.resolve(id).unwrap();
        let package = Package::load(&record.path).unwrap();
        let original = derived(&f.0, &package).unwrap();
        let cache =
            f.0.join(".epok/imported")
                .join(cook_key(&f.0, &package.meta).unwrap())
                .join("audio.adpcm");
        fs::write(&cache, b"broken").unwrap();
        assert_eq!(derived(&f.0, &package).unwrap(), original);
        let mut bytes = fs::read(&record.path).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        fs::write(&record.path, bytes).unwrap();
        let index = f.index();
        assert!(index.resolve(id).is_err());
        assert!(index.problems.iter().any(|s| s.contains("checksum")));
        assert_eq!(index.sources.len(), 1);
        assert!(index.native.is_empty());
    }
    #[test]
    fn deletion_checks_saved_references_and_keeps_a_recoverable_package() {
        let f = Fixture::new();
        let id = f.import();
        let record = f.index().resolve(id).unwrap().clone();
        let mut scene = crate::scene::Scene::default();
        scene.actors[0].audio = Some(crate::audio::AudioSource {
            clip: Some(id),
            ..Default::default()
        });
        let path = f.0.join("assets/scenes/Main.epokmap");
        scene.save(&path).unwrap();
        assert_eq!(dependencies(&f.0, id).unwrap().len(), 1);
        assert!(trash(&f.0, &record).is_err());
        scene.actors[0].audio = None;
        scene.save(&path).unwrap();
        let deleted = trash(&f.0, &record).unwrap();
        assert!(!record.path.exists());
        assert_eq!(Package::load(&deleted).unwrap().meta.id, id);
        atomic_write(
            &f.0.join("assets/restored.epokasset"),
            &read_bounded(&deleted).unwrap(),
            None,
        )
        .unwrap();
        assert!(f.index().resolve(id).is_ok());
    }
    #[test]
    fn oversized_audio_source_is_rejected_before_decode() {
        let fixture = Fixture::new();
        fs::write(
            fixture.0.join("assets/oversized.wav"),
            vec![0u8; 33 * 1024 * 1024],
        )
        .unwrap();
        assert!(
            prepare(
                &fixture.0,
                "assets/oversized.wav",
                "assets/oversized.epokasset",
                Settings::default(),
                None,
                false,
            )
            .err()
            .unwrap()
            .contains("exceeds the supported file size")
        );
    }
    #[test]
    fn a_font_package_pairs_only_with_font_settings_and_a_readable_face() {
        let package = |settings: crate::import_settings::Settings, source: Vec<u8>| Package {
            meta: Metadata {
                version: 2,
                id: Uuid::new_v4(),
                kind: Kind::Font,
                importer_version: crate::font_asset::IMPORTER_VERSION,
                source: "assets/Title.ttf".into(),
                source_hash: hash(&source),
                settings,
                extra: Default::default(),
            },
            source,
        };
        assert_eq!(
            package(crate::import_settings::Settings::Texture, b"x".into())
                .validate()
                .unwrap_err(),
            "Importer settings do not match asset kind"
        );
        assert!(
            package(
                crate::import_settings::Settings::Font(Default::default()),
                b"not an outline font".into()
            )
            .validate()
            .unwrap_err()
            .contains("TrueType")
        );
        // Font sources relink by path or checksum like the other external-source kinds.
        assert!(matches!(
            Index::default().linked_source(&Record {
                path: PathBuf::from("assets/Title.epokasset"),
                meta: package(
                    crate::import_settings::Settings::Font(Default::default()),
                    Vec::new()
                )
                .meta,
                revision: String::new(),
            }),
            Ok(None)
        ));
    }
}
