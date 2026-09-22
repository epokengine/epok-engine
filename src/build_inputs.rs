//! Native build provenance from the existing Make rules and GCC dependency files.
//! Host-only records; no compiler metadata is shipped in runtime tables.
use crate::{artifact_dependencies as dag, project::Config, staging_files::Files};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

const REPORT: &str = "BuildInputs.epokcache";
pub const REPORT_PREFIX: &str = "EPOK_NATIVE_INPUT:";
const STAMP: &str = "BuildInputs.stamp";
const INCREMENTAL_STATE: &str = "BuildInputs.incremental.epokcache";
const CONFIGURATION: &str = "native-build:configuration";
const ENVIRONMENT: &[&str] = &[
    "PATH",
    "MAKEFLAGS",
    "MFLAGS",
    "GNUMAKEFLAGS",
    "MAKEOVERRIDES",
    "MAKESHELL",
    "SHELL",
    "COMSPEC",
    "CPPFLAGS",
    "CXXFLAGS",
    "CFLAGS",
    "LDFLAGS",
    "GCC_EXEC_PREFIX",
    "COMPILER_PATH",
    "LIBRARY_PATH",
    "CPATH",
    "C_INCLUDE_PATH",
    "CPLUS_INCLUDE_PATH",
    "EPOK_RUNTIME_OPT",
    "EPOK_SCRIPT_OPT",
    "EPOK_VALIDATE_GTE",
    "EPOK_PROFILE_DETAIL",
];

fn configuration(config: &Config) -> String {
    crate::scene_dependencies::hash((
        &config.make,
        &config.toolchain_bin,
        &config.nugget,
        ENVIRONMENT
            .iter()
            .map(|key| {
                (
                    *key,
                    std::env::var_os(key).map(|v| v.to_string_lossy().into_owned()),
                )
            })
            .collect::<Vec<_>>(),
    ))
}

pub fn observe(root: &Path) -> Result<(), String> {
    if !dag::Graph::load(root)?.nodes.contains_key(CONFIGURATION) {
        return Ok(());
    }
    let current = Config::load(root).map(|config| configuration(&config));
    dag::transaction(root, |graph| match current {
        Ok(signature) => graph.publish(CONFIGURATION, signature, Default::default()),
        Err(error) => graph.invalidate(CONFIGURATION, &error),
    })?;
    Ok(())
}

pub struct Invocation {
    make: PathBuf,
    directory: PathBuf,
    search: OsString,
    nugget: OsString,
}
impl Invocation {
    pub fn new(root: &Path, build: &Path, config: &Config) -> Result<Self, String> {
        let mut search = Vec::new();
        if !config.toolchain_bin.is_empty() {
            search.push(Config::path(root, &config.toolchain_bin));
        }
        search.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        let search = std::env::join_paths(search).map_err(|e| e.to_string())?;
        let make = crate::dependencies::find_in_path(
            &Config::executable(root, &config.make).to_string_lossy(),
            Some(&search),
        )
        .ok_or("Configured Make executable is unavailable")?;
        Ok(Self {
            make,
            directory: crate::native::build_directory(build)?,
            search,
            nugget: format!(
                "NUGGET_DIR={}",
                Config::path(root, &config.nugget)
                    .to_string_lossy()
                    .replace('\\', "/")
            )
            .into(),
        })
    }
    pub fn command(&self, arguments: &[OsString]) -> Command {
        let mut command = Command::new(&self.make);
        // Dependency certification launches many independent preprocessor jobs.
        // Keeping Make at two workers made every scene-only Play rescan PsyQo
        // almost serially, even when the certified SDK archive was reusable.
        let jobs = std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(2)
            .clamp(1, 16);
        command
            .current_dir(&self.directory)
            .env("PATH", &self.search)
            .args([
                OsString::from(format!("-j{jobs}")),
                OsString::from("BUILD=Release"),
                self.nugget.clone(),
            ])
            .args(arguments);
        command
    }
    fn tool(&self, name: &str) -> Result<PathBuf, String> {
        crate::dependencies::find_in_path(name, Some(&self.search)).ok_or_else(|| {
            format!("Native build tool is unavailable or is not a single executable: {name}")
        })
    }
    fn query(&self, compiler: &Path, argument: &str) -> Result<String, String> {
        let mut command = Command::new(compiler);
        command
            .current_dir(&self.directory)
            .env("PATH", &self.search)
            .arg(argument);
        crate::pipeline::quiet(&mut command);
        let output = command
            .output()
            .map_err(|e| format!("Native compiler query: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "Native compiler query failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        String::from_utf8(output.stdout)
            .map(|value| value.trim().to_owned())
            .map_err(|e| e.to_string())
    }
}

#[derive(Default)]
struct Report {
    tools: BTreeMap<String, String>,
    values: BTreeMap<String, String>,
    files: BTreeSet<PathBuf>,
    dependencies: BTreeSet<PathBuf>,
    libraries: BTreeSet<String>,
}
impl Report {
    fn read(build: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(build.join(REPORT))
            .map_err(|e| format!("Native build input report: {e}"))?;
        Self::parse(build, &text)
    }
    fn parse(build: &Path, text: &str) -> Result<Self, String> {
        let mut lines = text.lines();
        if lines.next() != Some("epok-build-inputs-v1") {
            return Err("Unsupported native build input report".into());
        }
        let mut out = Self::default();
        for line in lines {
            let (kind, value) = line
                .split_once(':')
                .ok_or("Invalid native build input record")?;
            match kind {
                "tool" | "value" => {
                    let (key, value) = value
                        .split_once(':')
                        .ok_or("Invalid native build input value")?;
                    let map = if kind == "tool" {
                        &mut out.tools
                    } else {
                        &mut out.values
                    };
                    if map.insert(key.into(), value.into()).is_some() {
                        return Err("Duplicate native build input value".into());
                    }
                }
                "file" => {
                    out.files.insert(build.join(value));
                }
                "dependency" => {
                    out.dependencies.insert(build.join(value));
                }
                "library" => {
                    out.libraries.insert(value.into());
                }
                _ => return Err(format!("Unknown native build input record: {kind}")),
            }
        }
        if out.tools.len() != 5 || out.files.is_empty() || out.dependencies.is_empty() {
            return Err("Incomplete native build input report".into());
        }
        Ok(out)
    }
}

/// Parse GCC -M output, including continuations, escaped spaces/#/$ and Windows
/// drive prefixes. Only the prerequisites after the rule separator are inputs.
fn prerequisites(text: &str) -> Result<Vec<String>, String> {
    let text = text.replace("\\\r\n", " ").replace("\\\n", " ");
    let separator = text
        .char_indices()
        .find_map(|(index, c)| {
            (c == ':'
                && text[index + 1..]
                    .chars()
                    .next()
                    .is_some_and(char::is_whitespace))
            .then_some(index)
        })
        .ok_or("GCC dependency output has no rule separator")?;
    let mut words = Vec::new();
    let mut word = String::new();
    let mut chars = text[separator + 1..].chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\'
            && chars
                .peek()
                .is_some_and(|next| next.is_whitespace() || *next == '#' || *next == '\\')
        {
            word.push(chars.next().unwrap());
        } else if c == '$' && chars.peek() == Some(&'$') {
            chars.next();
            word.push('$');
        } else if c.is_whitespace() {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
        } else if c == '#' {
            break;
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    if words.is_empty() {
        return Err("GCC dependency output has no prerequisites".into());
    }
    Ok(words)
}

struct Snapshot {
    inputs: Files,
    recipe: String,
    /// Inputs already represented in Make's object dependency graph. Ordinary
    /// edits to these files can use Make's per-translation-unit invalidation.
    object_inputs: BTreeSet<String>,
    /// Libraries are Make inputs to the link, but not to every object.
    link_inputs: BTreeSet<String>,
    paths: BTreeMap<String, PathBuf>,
}
impl Snapshot {
    fn capture(
        root: &Path,
        build: &Path,
        config: &Config,
        invocation: &Invocation,
    ) -> Result<Self, String> {
        let report = Report::read(build)?;
        Self::from_report(root, build, config, invocation, report)
    }
    fn from_report(
        root: &Path,
        build: &Path,
        config: &Config,
        invocation: &Invocation,
        report: Report,
    ) -> Result<Self, String> {
        let recipe = crate::scene_dependencies::hash((&report.tools, &report.values));
        let mut paths = report.files;
        let libraries = report
            .libraries
            .iter()
            .map(|value| build.join(value))
            .collect::<BTreeSet<_>>();
        paths.extend(libraries.iter().cloned());
        let mut object_paths = BTreeSet::new();
        for dependency in report.dependencies {
            let bytes = std::fs::read_to_string(&dependency)
                .map_err(|e| format!("GCC dependency {}: {e}", dependency.display()))?;
            let prerequisites = prerequisites(&bytes)?
                .iter()
                .map(|value| build.join(value))
                .collect::<BTreeSet<_>>();
            object_paths.extend(prerequisites.iter().cloned());
            paths.extend(prerequisites);
        }
        paths.insert(invocation.make.clone());
        for (name, tool) in &report.tools {
            let resolved = invocation.tool(tool).or_else(|error| {
                // Native Windows Make can report its default sh.exe while
                // executing through COMSPEC when that default shell is absent.
                if cfg!(windows) && name == "SHELL" && tool == "sh.exe" {
                    let shell = std::env::var("COMSPEC").map_err(|_| error)?;
                    invocation.tool(&shell)
                } else {
                    Err(error)
                }
            })?;
            paths.insert(resolved);
        }
        let compiler = invocation.tool(
            report
                .tools
                .get("CC")
                .ok_or("Missing compiler in native input report")?,
        )?;
        for name in ["cc1", "cc1plus", "collect2", "as", "ld", "ar"] {
            let program = invocation.query(&compiler, &format!("-print-prog-name={name}"))?;
            paths.insert(invocation.tool(&program)?);
        }
        let mut inputs = Files::from([(CONFIGURATION.into(), configuration(config))]);
        let mut input_paths = BTreeMap::new();
        let mut object_inputs = BTreeSet::new();
        let mut link_inputs = BTreeSet::new();
        for path in paths {
            let key = crate::native_metadata::file_key(root, &path)?;
            let bytes = std::fs::read(&path)
                .map_err(|e| format!("Native build input {}: {e}", path.display()))?;
            let hash = crate::assets::hash(&bytes);
            if inputs.get(&key).is_some_and(|old| old != &hash) {
                return Err("Native build input changed while reading its aliases".into());
            }
            if object_paths.contains(&path) {
                object_inputs.insert(key.clone());
            }
            if libraries.contains(&path) {
                link_inputs.insert(key.clone());
            }
            input_paths.insert(key.clone(), path);
            inputs.insert(key, hash);
        }
        Ok(Self {
            inputs,
            recipe,
            object_inputs,
            link_inputs,
            paths: input_paths,
        })
    }
    fn signature(&self) -> String {
        crate::scene_dependencies::hash((&self.inputs, &self.recipe))
    }
    fn include_sdk(&mut self, sdk: &Snapshot) -> Result<(), String> {
        for (key, value) in &sdk.inputs {
            if self.inputs.get(key).is_some_and(|old| old != value) {
                return Err("SDK and application consumed conflicting native inputs".into());
            }
            self.inputs.insert(key.clone(), value.clone());
        }
        self.recipe = crate::scene_dependencies::hash((&self.recipe, &sdk.recipe));
        self.paths.extend(sdk.paths.clone());
        Ok(())
    }
}

const SDK_HELPER: &str = ".epok-build-inputs.dep";
const SDK_HELPER_MARKER: &[u8] = b"# Epok generated SDK build-input adapter.\n";
const SDK_ARCHIVE: &str = "sdk/libpsyqo.a";
const SDK_CACHE_VERSION: u32 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
struct SdkCacheEntry {
    version: u32,
    snapshot: String,
    archive: String,
}

fn sdk_cache(snapshot: &str) -> (PathBuf, PathBuf) {
    let directory = crate::workspace::user_data().join("BuildCache/PsyQo");
    (
        directory.join(format!("{snapshot}.json")),
        directory.join(format!("{snapshot}.a")),
    )
}

fn cached_sdk(snapshot: &str) -> Option<Vec<u8>> {
    let (record, archive) = sdk_cache(snapshot);
    let entry: SdkCacheEntry = serde_json::from_slice(&std::fs::read(record).ok()?).ok()?;
    let bytes = std::fs::read(archive).ok()?;
    (entry.version == SDK_CACHE_VERSION
        && entry.snapshot == snapshot
        && entry.archive == crate::assets::hash(&bytes)
        && bytes.starts_with(b"!<arch>\n"))
    .then_some(bytes)
}

fn store_cached_sdk(snapshot: &str, bytes: &[u8]) {
    let (record, archive) = sdk_cache(snapshot);
    let Some(directory) = record.parent() else {
        return;
    };
    if std::fs::create_dir_all(directory).is_err() {
        return;
    }
    if crate::project::write_changed(&archive, bytes).is_err() {
        return;
    }
    let entry = SdkCacheEntry {
        version: SDK_CACHE_VERSION,
        snapshot: snapshot.into(),
        archive: crate::assets::hash(bytes),
    };
    if let Ok(bytes) = serde_json::to_vec(&entry) {
        let _ = crate::project::write_changed(&record, &bytes);
    }
}

fn make_path(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    if let Some(unc) = value.strip_prefix("//?/UNC/") {
        format!("//{unc}")
    } else {
        value.strip_prefix("//?/").unwrap_or(&value).to_owned()
    }
}

struct Sdk {
    directory: PathBuf,
    arguments: Vec<OsString>,
    snapshot: Snapshot,
    archive_hash: String,
    preparation: &'static str,
}

impl Sdk {
    fn lock(directory: &Path) -> Result<std::fs::File, String> {
        let identity = crate::assets::hash(directory.to_string_lossy().as_bytes());
        let path = std::env::temp_dir().join(format!("epok-sdk-{identity}.lock"));
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| format!("SDK build lock: {e}"))?;
        file.try_lock()
            .map_err(|e| format!("SDK build is busy or cannot be locked; retry the build: {e}"))?;
        Ok(file)
    }

    fn capture(
        &self,
        root: &Path,
        build: &Path,
        config: &Config,
        invocation: &Invocation,
        fresh_dependencies: bool,
        run: &mut impl FnMut(&[OsString]) -> Result<String, String>,
    ) -> Result<Snapshot, String> {
        let mut arguments = self.arguments.clone();
        arguments.extend(query_arguments(fresh_dependencies));
        let text = run(&arguments)?;
        crate::project::write_changed(&build.join("SdkBuildInputs.epokcache"), text.as_bytes())?;
        Snapshot::from_report(
            root,
            &self.directory,
            config,
            invocation,
            Report::parse(&self.directory, &text)?,
        )
    }

    fn prepare(
        root: &Path,
        build: &Path,
        config: &Config,
        invocation: &Invocation,
        run: &mut impl FnMut(&[OsString]) -> Result<String, String>,
    ) -> Result<Self, String> {
        let report = Report::read(build)?;
        let directory = report
            .values
            .get("PSYQODIR")
            .ok_or("Missing PsyQo SDK build directory")?;
        let directory = std::fs::canonicalize(build.join(directory))
            .map_err(|e| format!("SDK directory: {e}"))?;
        let expected = std::fs::canonicalize(Config::path(root, &config.nugget).join("psyqo"))
            .map_err(|e| e.to_string())?;
        if directory != expected {
            return Err("Make selected an SDK outside the configured Nugget directory".into());
        }
        let _lock = Self::lock(&directory)?;
        let helper = directory.join(SDK_HELPER);
        let previous_helper = match std::fs::read(&helper) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(format!("SDK build adapter: {error}")),
        };
        if previous_helper
            .as_ref()
            .is_some_and(|bytes| !bytes.starts_with(SDK_HELPER_MARKER))
        {
            return Err(format!(
                "SDK capture would overwrite an unrelated file: {}",
                helper.display()
            ));
        }
        let mut bytes = SDK_HELPER_MARKER.to_vec();
        bytes.extend(std::fs::read(build.join("build-inputs.mk")).map_err(|e| e.to_string())?);
        if previous_helper.as_deref() != Some(&bytes) {
            crate::assets::atomic_write(
                &helper,
                &bytes,
                previous_helper
                    .as_deref()
                    .map(crate::assets::hash)
                    .as_deref(),
            )?;
        }
        let mut arguments = vec![
            "-C".into(),
            make_path(&directory).into(),
            "-f".into(),
            "Makefile".into(),
            "-f".into(),
            SDK_HELPER.into(),
        ];
        // Match psyqo.mk's recursive invocation, including command-line variable
        // precedence over the SDK's own += defaults.
        for name in ["CPPFLAGS_Release", "LDFLAGS_Release"] {
            arguments.push(
                format!(
                    "{name}={}",
                    report.values.get(name).ok_or("Missing SDK build options")?
                )
                .into(),
            );
        }
        let mut sdk = Self {
            directory,
            arguments,
            snapshot: Snapshot {
                inputs: Files::new(),
                recipe: String::new(),
                object_inputs: BTreeSet::new(),
                link_inputs: BTreeSet::new(),
                paths: BTreeMap::new(),
            },
            archive_hash: String::new(),
            preparation: "reused project certificate",
        };
        let key = format!(
            "native-sdk:{}",
            crate::playback_staging::target(root, build)?
        );
        let before = dag::Graph::load(root)?;
        dag::transaction(root, |graph| {
            graph.invalidate(&key, "SDK input capture has not completed")
        })?;
        // A normal Make query still discovers source membership and refreshes
        // missing/outdated .dep files. Hashing that report against the previous
        // certificate catches timestamp-preserving changes to every known input.
        // Only a changed/stale SDK needs the expensive forced transitive scan.
        sdk.snapshot = sdk.capture(root, build, config, invocation, false, run)?;
        let archive = build.join(SDK_ARCHIVE);
        let signature = |snapshot: &Snapshot, bytes: &[u8]| {
            crate::scene_dependencies::hash((snapshot.signature(), crate::assets::hash(bytes)))
        };
        let previous_archive = match std::fs::read(&archive) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(format!("SDK build archive: {error}")),
        };
        let reusable = previous_archive.as_ref().is_some_and(|bytes| {
            before.nodes.get(&key).is_some_and(|node| {
                node.stale.is_empty()
                    && node.signature.as_ref() == Some(&signature(&sdk.snapshot, bytes))
            })
        });
        let certified_signature;
        if !reusable {
            let snapshot_signature = sdk.snapshot.signature();
            // A shared certificate can initialize a new/fresh project build,
            // but must not hide a failed local certification. A stale local
            // node deliberately requires one clean compile in this invocation.
            let shared_reusable = before
                .nodes
                .get(&key)
                .is_none_or(|node| node.stale.is_empty());
            if let Some(bytes) = shared_reusable
                .then(|| cached_sdk(&snapshot_signature))
                .flatten()
            {
                std::fs::create_dir_all(archive.parent().unwrap()).map_err(|e| e.to_string())?;
                if previous_archive.as_deref() != Some(&bytes) {
                    crate::assets::atomic_write(
                        &archive,
                        &bytes,
                        previous_archive
                            .as_deref()
                            .map(crate::assets::hash)
                            .as_deref(),
                    )?;
                }
                certified_signature = signature(&sdk.snapshot, &bytes);
                sdk.archive_hash = crate::assets::hash(&bytes);
                sdk.preparation = "reused shared certificate";
            } else {
                sdk.snapshot = sdk.capture(root, build, config, invocation, true, run)?;
                std::fs::create_dir_all(archive.parent().unwrap()).map_err(|e| e.to_string())?;
                let temporary =
                    archive.with_file_name(format!("libpsyqo-{}.a", uuid::Uuid::new_v4()));
                let output = make_path(&temporary);
                if output.contains(['"', '$', '`', '\n', '\r']) {
                    return Err(
                        "SDK archive output path contains unsupported shell characters".into(),
                    );
                }
                let mut arguments = sdk.arguments.clone();
                let object_directory = format!(".epok-sdk-objects-{}", uuid::Uuid::new_v4());
                std::fs::create_dir(sdk.directory.join(&object_directory))
                    .map_err(|e| format!("SDK object directory: {e}"))?;
                arguments.extend([
                    "-B".into(),
                    "epok-sdk-archive".into(),
                    format!("EPOK_SDK_ARCHIVE_OUTPUT={output}").into(),
                    format!("EPOK_SDK_OBJECT_DIRECTORY={object_directory}").into(),
                ]);
                run(&arguments)?;
                let current = sdk.capture(root, build, config, invocation, true, run)?;
                if current.signature() != sdk.snapshot.signature() {
                    return Err(
                        "SDK inputs changed during archive compilation; rebuild before Play".into(),
                    );
                }
                let bytes =
                    std::fs::read(&temporary).map_err(|e| format!("Missing SDK archive: {e}"))?;
                if !bytes.starts_with(b"!<arch>\n") {
                    return Err("SDK build did not produce an archive".into());
                }
                // Always construct a new archive: ar rcs on an old archive retains
                // members whose source files have been removed from the Make list.
                if previous_archive.as_deref() != Some(&bytes) {
                    crate::assets::atomic_write(
                        &archive,
                        &bytes,
                        previous_archive
                            .as_deref()
                            .map(crate::assets::hash)
                            .as_deref(),
                    )?;
                }
                certified_signature = signature(&sdk.snapshot, &bytes);
                sdk.archive_hash = crate::assets::hash(&bytes);
                sdk.preparation = "compiled and certified";
                store_cached_sdk(&sdk.snapshot.signature(), &bytes);
                // This unique archive was created by this invocation in its own
                // output directory. Failed invocations retain it for diagnosis.
                let _ = std::fs::remove_file(&temporary);
            }
        } else {
            let bytes = previous_archive.as_ref().unwrap();
            certified_signature = signature(&sdk.snapshot, bytes);
            sdk.archive_hash = crate::assets::hash(bytes);
        }
        dag::transaction(root, |graph| {
            for (key, value) in &sdk.snapshot.inputs {
                graph.publish(key, value.clone(), Default::default());
            }
            graph.publish(
                &key,
                certified_signature,
                sdk.snapshot.inputs.keys().cloned().collect(),
            );
        })?;
        Ok(sdk)
    }
}

pub struct Prepared {
    snapshot: Snapshot,
    sdk: Sdk,
    key: String,
    rebuild: bool,
    force_all: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct IncrementalState {
    version: u32,
    signature: String,
    recipe: String,
    inputs: Files,
}

impl IncrementalState {
    fn load(build: &Path) -> Option<Self> {
        let state: Self =
            serde_json::from_slice(&std::fs::read(build.join(INCREMENTAL_STATE)).ok()?).ok()?;
        (state.version == 1).then_some(state)
    }

    fn save(build: &Path, snapshot: &Snapshot) -> Result<(), String> {
        let state = Self {
            version: 1,
            signature: snapshot.signature(),
            recipe: snapshot.recipe.clone(),
            inputs: snapshot.inputs.clone(),
        };
        crate::project::write_changed(
            &build.join(INCREMENTAL_STATE),
            &serde_json::to_vec(&state).map_err(|e| e.to_string())?,
        )
    }

    fn make_can_rebuild(&self, snapshot: &Snapshot, executable: &Path) -> bool {
        if self.recipe != snapshot.recipe {
            return false;
        }
        let Ok(output_time) = executable.metadata().and_then(|value| value.modified()) else {
            return false;
        };
        let keys = self
            .inputs
            .keys()
            .chain(snapshot.inputs.keys())
            .collect::<BTreeSet<_>>();
        let changed = keys
            .into_iter()
            .filter(|key| self.inputs.get(*key) != snapshot.inputs.get(*key));
        let mut any = false;
        for key in changed {
            any = true;
            if !snapshot.object_inputs.contains(key) && !snapshot.link_inputs.contains(key) {
                return false;
            }
            let Some(path) = snapshot.paths.get(key) else {
                return false;
            };
            let Ok(input_time) = path.metadata().and_then(|value| value.modified()) else {
                return false;
            };
            if input_time <= output_time {
                return false;
            }
        }
        any
    }
}
fn query_arguments(fresh_dependencies: bool) -> Vec<OsString> {
    fresh_dependencies
        .then_some("-B")
        .into_iter()
        .chain(["dep", "epok-build-inputs"])
        .map(Into::into)
        .collect()
}
fn application_arguments(arguments: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    std::iter::once(format!("EPOK_CERTIFIED_SDK={SDK_ARCHIVE}").into())
        .chain(arguments)
        .collect()
}

fn query(
    build: &Path,
    fresh_dependencies: bool,
    run: &mut impl FnMut(&[OsString]) -> Result<String, String>,
) -> Result<(), String> {
    let report = run(&query_arguments(fresh_dependencies))?;
    // Replace any previous report only with this invocation's captured output.
    // An empty/malformed result must fail validation, never reuse old records.
    crate::project::write_changed(&build.join(REPORT), report.as_bytes())
}

pub fn prepare(
    root: &Path,
    build: &Path,
    config: &Config,
    invocation: &Invocation,
    run: &mut impl FnMut(&[OsString]) -> Result<String, String>,
) -> Result<Prepared, String> {
    let target = crate::playback_staging::target(root, build)?;
    let key = format!("native-build:{target}");
    let executable = format!("executable:{target}/epok.ps-exe");
    let before = dag::Graph::load(root)?;
    dag::transaction(root, |graph| {
        graph.invalidate(&executable, "Native build preparation has not completed")
    })?;
    // This first report is used only to locate/configure the SDK. Avoid forcing
    // the application's transitive dependency pass until the certified archive
    // is in place and the final application inputs can be captured once.
    query(build, false, run)?;
    let sdk = Sdk::prepare(root, build, config, invocation, run)?;
    // Libraries may invoke their own build. Capture the application inputs only
    // after those artifacts exist, with a fresh compiler-reported include closure.
    let mut run_application =
        |arguments: &[OsString]| run(&application_arguments(arguments.iter().cloned()));
    query(build, true, &mut run_application)?;
    let mut snapshot = Snapshot::capture(root, build, config, invocation)?;
    let archive_key = crate::native_metadata::file_key(root, &build.join(SDK_ARCHIVE))?;
    if snapshot.inputs.get(&archive_key) != Some(&sdk.archive_hash) {
        return Err("The application SDK archive changed after certification".into());
    }
    snapshot.include_sdk(&sdk.snapshot)?;
    let signature = snapshot.signature();
    let rebuild =
        before.nodes.get(&key).is_none_or(|node| {
            node.signature.as_ref() != Some(&signature) || !node.stale.is_empty()
        }) || before
            .nodes
            .get(&executable)
            .is_none_or(|node| !node.stale.is_empty() || node.signature.is_none());
    let force_all = rebuild
        && !IncrementalState::load(build)
            .is_some_and(|state| state.make_can_rebuild(&snapshot, &build.join("epok.ps-exe")));
    crate::project::write_changed(&build.join(STAMP), signature.as_bytes())?;
    dag::transaction(root, |graph| {
        for (key, signature) in &snapshot.inputs {
            graph.publish(key, signature.clone(), Default::default());
        }
        let mut dependencies = snapshot.inputs.keys().cloned().collect::<BTreeSet<_>>();
        dependencies.insert(format!("native-sdk:{target}"));
        graph.publish(&key, signature.clone(), dependencies);
    })?;
    Ok(Prepared {
        snapshot,
        sdk,
        key,
        rebuild,
        force_all,
    })
}
impl Prepared {
    pub fn requires_rebuild(&self) -> bool {
        self.rebuild
    }
    pub fn sdk_preparation(&self) -> &'static str {
        self.sdk.preparation
    }
    pub fn forces_full_recompile(&self) -> bool {
        self.force_all
    }
    pub fn record_reused(&self, build: &Path) -> Result<(), String> {
        IncrementalState::save(build, &self.snapshot)
    }
    pub fn arguments(&self) -> Vec<OsString> {
        let mut arguments = vec!["all".into(), format!("EPOK_INPUT_STAMP={STAMP}").into()];
        if self.force_all {
            arguments.extend(["-W".into(), STAMP.into()]);
        }
        application_arguments(arguments)
    }
    pub fn verify(
        &self,
        root: &Path,
        build: &Path,
        config: &Config,
        invocation: &Invocation,
        run: &mut impl FnMut(&[OsString]) -> Result<String, String>,
    ) -> Result<(), String> {
        let result: Result<(), String> = (|| {
            let _lock = Sdk::lock(&self.sdk.directory)?;
            let sdk = self
                .sdk
                .capture(root, build, config, invocation, true, run)?;
            let mut run_application =
                |arguments: &[OsString]| run(&application_arguments(arguments.iter().cloned()));
            query(build, true, &mut run_application)?;
            let mut current = Snapshot::capture(root, build, config, invocation)?;
            current.include_sdk(&sdk)?;
            if current.signature() != self.snapshot.signature() {
                return Err(
                    "Native build inputs changed during compilation; no executable launched".into(),
                );
            }
            IncrementalState::save(build, &self.snapshot)
        })();
        if let Err(error) = &result {
            crate::native_metadata::observe(root)?;
            dag::transaction(root, |graph| graph.invalidate(&self.key, error))?;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::staging_files::BuildTicket;

    #[test]
    fn incremental_state_uses_make_only_for_newer_known_inputs() {
        let root =
            std::env::temp_dir().join(format!("epok-incremental-policy-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let executable = root.join("epok.ps-exe");
        let input = root.join("scene.hh");
        std::fs::write(&executable, b"exe").unwrap();
        std::fs::write(&input, b"new").unwrap();
        let output_time = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(100);
        let input_time = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(200);
        std::fs::File::options()
            .write(true)
            .open(&executable)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(output_time))
            .unwrap();
        std::fs::File::options()
            .write(true)
            .open(&input)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(input_time))
            .unwrap();
        let key = "input".to_owned();
        let state = IncrementalState {
            version: 1,
            signature: "old".into(),
            recipe: "recipe".into(),
            inputs: Files::from([(key.clone(), "old".into())]),
        };
        let mut snapshot = Snapshot {
            inputs: Files::from([(key.clone(), "new".into())]),
            recipe: "recipe".into(),
            object_inputs: BTreeSet::from([key.clone()]),
            link_inputs: BTreeSet::new(),
            paths: BTreeMap::from([(key.clone(), input.clone())]),
        };
        assert!(state.make_can_rebuild(&snapshot, &executable));

        std::fs::File::options()
            .write(true)
            .open(&input)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(output_time))
            .unwrap();
        assert!(
            !state.make_can_rebuild(&snapshot, &executable),
            "timestamp-preserving edits require the conservative full rebuild"
        );
        snapshot.object_inputs.clear();
        snapshot.link_inputs.insert(key.clone());
        std::fs::File::options()
            .write(true)
            .open(&input)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(input_time))
            .unwrap();
        assert!(state.make_can_rebuild(&snapshot, &executable));
        snapshot.link_inputs.clear();
        assert!(
            !state.make_can_rebuild(&snapshot, &executable),
            "untracked compiler/configuration inputs must force every object"
        );
        snapshot.object_inputs.insert(key);
        snapshot.recipe = "changed recipe".into();
        assert!(!state.make_can_rebuild(&snapshot, &executable));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn gcc_dependency_paths_preserve_spaces_hashes_dollars_and_drive_prefixes() {
        assert_eq!(
            prerequisites(
                "main.o: main.cpp C:/Program\\ Files/a\\#b.hh \\\n dir/c$$d.hh # ignored comment\n"
            )
            .unwrap(),
            ["main.cpp", "C:/Program Files/a#b.hh", "dir/c$d.hh"]
        );
        assert_eq!(
            prerequisites("C:/build/main.o: C:/source/main.cpp\r\n").unwrap(),
            ["C:/source/main.cpp"]
        );
        assert!(prerequisites("no dependency rule").is_err());
        assert!(prerequisites("main.o: \n").is_err());
    }

    #[test]
    #[ignore = "requires configured Make/MIPS tools; mutates only an owned miniature SDK fixture"]
    fn sdk_archive_tracks_real_assembly_inputs_source_membership_and_failed_rebuilds() {
        let fixture =
            std::env::temp_dir().join(format!("epok-sdk-inputs-{}", uuid::Uuid::new_v4()));
        let root = fixture.join("Game With Spaces");
        let build = root.join(".epok/build");
        let sdk_root = fixture.join("SDK");
        let sdk_dir = sdk_root.join("psyqo");
        std::fs::create_dir_all(&build).unwrap();
        std::fs::create_dir_all(&sdk_dir).unwrap();
        let mut config = Config::load(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let original_sdk = Config::path(Path::new(env!("CARGO_MANIFEST_DIR")), &config.nugget);
        for file in ["common.mk", "ps-exe.ld", "nooverlay.ld"] {
            std::fs::copy(original_sdk.join(file), sdk_root.join(file)).unwrap();
        }
        config.nugget = make_path(&sdk_root);
        std::fs::write(
            build.join("build-inputs.mk"),
            include_bytes!("../runtime/build-inputs.mk"),
        )
        .unwrap();
        std::fs::write(build.join("App.cpp"), "int app() { return 1; }\n").unwrap();
        std::fs::write(build.join("Makefile"), format!("TARGET=app\nTYPE=ps-exe\nSRCS=App.cpp\nPSYQODIR:={}/psyqo/\ninclude {}/common.mk\nLIBRARIES += $(PSYQODIR)libpsyqo.a\ninclude build-inputs.mk\n", make_path(&sdk_root), make_path(&sdk_root))).unwrap();
        std::fs::write(sdk_dir.join("Makefile"), "TARGET=psyqo\nTYPE=library\nSRCS=$(wildcard *.cpp) $(wildcard *.s)\nCPPFLAGS += -I.\ninclude ../common.mk\n").unwrap();
        let header = sdk_dir.join("Value.inc");
        let original_header = "#define EPOK_TEST_VALUE 1\n";
        std::fs::write(&header, original_header).unwrap();
        let source = sdk_dir.join("Value.cpp");
        let original_source = "#include \"Value.inc\"\n#if __has_include(\"Shadow.inc\")\n#include \"Shadow.inc\"\n#endif\nextern \"C\" int sdk_value() { return EPOK_TEST_VALUE; }\n";
        std::fs::write(&source, original_source).unwrap();
        std::fs::write(
            sdk_dir.join("Removed.cpp"),
            "extern \"C\" int sdk_removed() { return 7; }\n",
        )
        .unwrap();
        std::fs::write(
            sdk_dir.join("Assembly.inc"),
            ".equ EPOK_ASSEMBLY_VALUE, 3\n",
        )
        .unwrap();
        std::fs::write(sdk_dir.join("Data.bin"), [1u8, 2, 3, 4]).unwrap();
        std::fs::write(
            sdk_dir.join("Assembly.s"),
            r#".include "Assembly.inc"
.section .text.sdk_assembly,"ax"
.globl sdk_assembly
sdk_assembly:
.set noreorder
jr $ra
li $v0, EPOK_ASSEMBLY_VALUE
.section .rodata.sdk_blob,"a"
.globl sdk_blob
sdk_blob:
.incbin "Data.bin"
"#,
        )
        .unwrap();
        let shared = sdk_dir.join("libpsyqo.a");
        std::fs::write(
            &shared,
            b"Pre-existing shared SDK archive must remain untouched",
        )
        .unwrap();
        let shared_before = std::fs::read(&shared).unwrap();
        let shared_object = sdk_dir.join("Value.o");
        std::fs::write(
            &shared_object,
            b"Existing shared SDK object must remain untouched",
        )
        .unwrap();
        let object_before = std::fs::read(&shared_object).unwrap();
        let invocation = Invocation::new(&root, &build, &config).unwrap();
        let builds = std::cell::Cell::new(0u32);
        let mutate_after_archive = std::cell::Cell::new(false);
        let mut run = |arguments: &[OsString]| -> Result<String, String> {
            let mut command = invocation.command(arguments);
            crate::pipeline::quiet(&mut command);
            let output = command.output().map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err(format!(
                    "{}\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            if arguments.iter().any(|arg| arg == "epok-sdk-archive") {
                builds.set(builds.get() + 1);
                if mutate_after_archive.replace(false) {
                    std::fs::write(&header, "#define EPOK_TEST_VALUE 9\n").unwrap();
                }
            }
            Ok(String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| line.strip_prefix(REPORT_PREFIX))
                .map(|line| format!("{line}\n"))
                .collect())
        };
        query(&build, true, &mut run).unwrap();
        let sdk = Sdk::prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        assert_eq!(builds.get(), 1);
        for file in [
            "Value.inc",
            "Assembly.inc",
            "Data.bin",
            "Value.cpp",
            "Removed.cpp",
        ] {
            assert!(
                sdk.snapshot.inputs.contains_key(
                    &crate::native_metadata::file_key(&root, &sdk_dir.join(file)).unwrap()
                ),
                "Missing SDK input: {file}"
            );
        }
        let archive = build.join(SDK_ARCHIVE);
        let first = std::fs::read(&archive).unwrap();
        std::fs::remove_file(&archive).unwrap();
        dag::transaction(&root, |graph| {
            graph.nodes.remove("native-sdk:.epok/build");
        })
        .unwrap();
        let shared_sdk = Sdk::prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        assert_eq!(shared_sdk.preparation, "reused shared certificate");
        assert_eq!(builds.get(), 1, "a new build must reuse the shared SDK");
        assert_eq!(std::fs::read(&archive).unwrap(), first);
        let time = std::fs::metadata(&archive).unwrap().modified().unwrap();
        Sdk::prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        assert_eq!(
            builds.get(),
            1,
            "Unchanged certified SDK must reuse its private archive"
        );
        assert_eq!(
            std::fs::metadata(&archive).unwrap().modified().unwrap(),
            time
        );
        let header_time = std::fs::metadata(&header).unwrap().modified().unwrap();
        std::fs::write(&header, "#define EPOK_TEST_VALUE 2\n").unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .open(&header)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(header_time))
            .unwrap();
        Sdk::prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        assert_eq!(builds.get(), 2);
        let second = std::fs::read(&archive).unwrap();
        assert_ne!(
            first, second,
            "Preserved-timestamp SDK changes must rebuild object code"
        );
        std::fs::rename(
            sdk_dir.join("Removed.cpp"),
            sdk_dir.join("Removed.cpp.disabled"),
        )
        .unwrap();
        Sdk::prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        let mut command =
            std::process::Command::new(invocation.tool("mipsel-none-elf-ar").unwrap());
        crate::pipeline::quiet(&mut command);
        let members = command.arg("t").arg(&archive).output().unwrap();
        assert!(members.status.success());
        assert!(
            !String::from_utf8_lossy(&members.stdout).contains("Removed.o"),
            "Fresh archives must remove obsolete members"
        );
        let valid = std::fs::read(&archive).unwrap();
        std::fs::write(&source, format!("{original_source}\ninvalid C++ source;\n")).unwrap();
        assert!(Sdk::prepare(&root, &build, &config, &invocation, &mut run).is_err());
        assert_eq!(std::fs::read(&archive).unwrap(), valid);
        assert!(
            !dag::Graph::load(&root).unwrap().nodes["native-sdk:.epok/build"]
                .stale
                .is_empty()
        );
        std::fs::write(&source, original_source).unwrap();
        mutate_after_archive.set(true);
        let error = Sdk::prepare(&root, &build, &config, &invocation, &mut run)
            .err()
            .unwrap();
        assert!(error.contains("SDK inputs changed"), "{error}");
        assert_eq!(std::fs::read(&archive).unwrap(), valid);
        Sdk::prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        assert_ne!(std::fs::read(&archive).unwrap(), valid);
        let prepared = prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        std::fs::write(
            sdk_dir.join("Shadow.inc"),
            "#undef EPOK_TEST_VALUE\n#define EPOK_TEST_VALUE 11\n",
        )
        .unwrap();
        assert!(
            prepared
                .verify(&root, &build, &config, &invocation, &mut run)
                .is_err()
        );
        assert!(
            !dag::Graph::load(&root).unwrap().nodes["native-build:.epok/build"]
                .stale
                .is_empty()
        );
        assert_eq!(std::fs::read(&shared).unwrap(), shared_before);
        assert_eq!(std::fs::read(&shared_object).unwrap(), object_before);
        let lock = Sdk::lock(&std::fs::canonicalize(&sdk_dir).unwrap()).unwrap();
        assert!(Sdk::lock(&std::fs::canonicalize(&sdk_dir).unwrap()).is_err());
        drop(lock);
        assert!(Sdk::lock(&std::fs::canonicalize(&sdk_dir).unwrap()).is_ok());
        println!("Verified SDK input fixture: {}", fixture.display());
    }

    #[test]
    #[ignore = "requires configured Make/MIPS tools; compiles an owned fixture without launching the emulator"]
    fn native_make_inputs_detect_timestamp_preserving_edits_and_new_include_resolution() {
        let fixture =
            std::env::temp_dir().join(format!("epok-build-inputs-{}", uuid::Uuid::new_v4()));
        let root = fixture.join("Game With Spaces");
        std::fs::create_dir_all(root.join("assets/scripts")).unwrap();
        let external = fixture.join("External Headers");
        std::fs::create_dir_all(&external).unwrap();
        let header = external.join("Value #1.inc");
        let first_header = "#if __has_include(\"Shadow.inc\")\n#include \"Shadow.inc\"\n#else\n#define EPOK_TEST_VALUE 1\n#endif\n";
        std::fs::write(&header, first_header).unwrap();
        let class_id = "069ca18c-c618-4af8-b8fb-c34fa044da7a";
        std::fs::write(root.join("assets/scripts/Probe.hpp"), format!("#pragma once\n#include \"epok.hpp\"\nclass EPOK_CLASS(Blueprintable,Id=\"{class_id}\") Probe:public epok::Actor3D {{ public: EPOK_PROPERTY(EditAnywhere,Id=\"e7cded42-0b8a-4347-8a70-0caef7c101dd\") int32_t value=0; void tick(epok::Fixed) override; }};\n")).unwrap();
        std::fs::write(root.join("assets/scripts/Probe.cpp"), format!("#include \"Probe.hpp\"\n#include \"{}\"\nvoid Probe::tick(epok::Fixed) {{ value=EPOK_TEST_VALUE; }}\n", header.to_string_lossy().replace('\\', "/"))).unwrap();
        let mut scene = crate::scene::Scene::default();
        scene.actors[0].set_class_defaults(
            &(crate::scene::ClassDefaults {
                name: "Probe".into(),
                class_id: Some(class_id.into()),
                ..Default::default()
            }),
        );
        let build = root.join(".epok/build");
        crate::project::stage_into(&root, &scene, &build).unwrap();
        let mut patches = Files::new();
        let main_path = build.join("main.cpp");
        let main = format!(
            "#ifdef __OPTIMIZE_SIZE__\n#include \"SizeOnly.inc\"\n#else\n#include \"SpeedOnly.inc\"\n#endif\n{}",
            std::fs::read_to_string(&main_path).unwrap()
        );
        for (path, bytes) in [
            ("main.cpp", main.as_bytes()),
            ("SizeOnly.inc", b"// Size-optimized include\n".as_slice()),
            ("SpeedOnly.inc", b"// Speed-optimized include\n".as_slice()),
        ] {
            crate::project::write_changed(&build.join(path), bytes).unwrap();
            patches.insert(path.into(), crate::assets::hash(bytes));
        }
        crate::staging_files::patch(&root, &build, patches, "release".into()).unwrap();
        let config = Config::load(&root).unwrap();
        let invocation = Invocation::new(&root, &build, &config).unwrap();
        let mut run = |arguments: &[OsString]| -> Result<String, String> {
            let mut command = invocation.command(arguments);
            crate::pipeline::quiet(&mut command);
            let output = command.output().map_err(|e| e.to_string())?;
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter_map(|line| line.strip_prefix(REPORT_PREFIX))
                    .map(|line| format!("{line}\n"))
                    .collect())
            } else {
                Err(format!(
                    "{}\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        };
        let compile =
            |prepared: &Prepared, run: &mut dyn FnMut(&[OsString]) -> Result<String, String>| {
                run(&prepared.arguments()).unwrap();
                let bytes = std::fs::read(build.join("epok.ps-exe")).unwrap();
                assert!(bytes.starts_with(b"PS-X EXE"));
                bytes
            };
        let initial = prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        assert!(initial.rebuild);
        assert!(initial.force_all);
        assert!(
            initial
                .snapshot
                .inputs
                .keys()
                .any(|key| key.ends_with("SpeedOnly.inc"))
        );
        assert!(
            !initial
                .snapshot
                .inputs
                .keys()
                .any(|key| key.ends_with("SizeOnly.inc")),
            "main.dep must use main.o's target-specific -O2 flags, not the global -Os flags"
        );
        assert!(
            initial
                .snapshot
                .inputs
                .keys()
                .any(|key| key.ends_with("libpsyqo.a"))
        );
        assert!(
            initial
                .snapshot
                .inputs
                .keys()
                .any(|key| key.ends_with("libgcc.a"))
        );
        assert!(
            initial
                .snapshot
                .inputs
                .keys()
                .any(|key| key.ends_with("ps-exe.ld"))
        );
        let ticket = BuildTicket::begin(&root, &build).unwrap();
        let original = compile(&initial, &mut run);
        initial
            .verify(&root, &build, &config, &invocation, &mut run)
            .unwrap();
        ticket.complete(&root, &build, &original).unwrap();
        let pending = prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        assert!(
            !pending.rebuild,
            "Unchanged validated inputs must permit Make reuse"
        );
        assert!(!pending.force_all);
        let ticket = BuildTicket::begin(&root, &build).unwrap();
        compile(&pending, &mut run);
        let time = std::fs::metadata(&header).unwrap().modified().unwrap();
        std::fs::write(&header, first_header.replace("VALUE 1", "VALUE 2")).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .open(&header)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(time))
            .unwrap();
        assert!(
            pending
                .verify(&root, &build, &config, &invocation, &mut run)
                .is_err()
        );
        assert!(ticket.complete(&root, &build, &original).is_err());
        let changed = prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        assert!(changed.rebuild);
        assert!(
            changed.force_all,
            "timestamp-preserving edits require a full application rebuild"
        );
        let ticket = BuildTicket::begin(&root, &build).unwrap();
        let updated = compile(&changed, &mut run);
        assert_ne!(
            updated, original,
            "A timestamp-preserving header edit must recompile the application"
        );
        changed
            .verify(&root, &build, &config, &invocation, &mut run)
            .unwrap();
        ticket.complete(&root, &build, &updated).unwrap();

        let pending = prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        let ticket = BuildTicket::begin(&root, &build).unwrap();
        compile(&pending, &mut run);
        let shadow = external.join("Shadow.inc");
        // No recorded source bytes change: __has_include selects a newly created
        // file. A post-build dependency pass must still reject the old closure.
        std::fs::write(&shadow, b"#define EPOK_TEST_VALUE 3\n").unwrap();
        assert!(
            pending
                .verify(&root, &build, &config, &invocation, &mut run)
                .is_err()
        );
        assert!(ticket.complete(&root, &build, &updated).is_err());
        let changed = prepare(&root, &build, &config, &invocation, &mut run).unwrap();
        assert!(changed.rebuild);
        assert!(
            !changed.force_all,
            "a newly discovered, newer header can use Make's translation-unit dependency"
        );
        assert!(
            changed
                .snapshot
                .inputs
                .contains_key(&crate::native_metadata::file_key(&root, &shadow).unwrap())
        );
        let ticket = BuildTicket::begin(&root, &build).unwrap();
        let updated_again = compile(&changed, &mut run);
        assert_ne!(updated_again, updated);
        changed
            .verify(&root, &build, &config, &invocation, &mut run)
            .unwrap();
        ticket.complete(&root, &build, &updated_again).unwrap();
        println!("Verified native Make input fixture: {}", root.display());
    }
}
