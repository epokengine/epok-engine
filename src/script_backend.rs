//! Host-side extension contracts. No VM or dynamic plugin loader is linked.
//! Native code generation uses the same backend boundary exercised by test providers.
use crate::{reflection_schema as schema, scene::ClassDefaults, scripts::Script};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct Capabilities {
    pub create: bool,
    pub derive_backends: BTreeSet<String>,
    pub attach: bool,
    pub invoke: bool,
}

/// Produced on the host, never by executing a target script in the Inspector.
#[allow(dead_code)] // Reserved provider API; Native preparation is currently in project staging.
pub trait AuthoringProvider {
    fn identity(&self) -> schema::Extension;
    fn capabilities(&self) -> Capabilities;
    fn declarations(&self) -> Result<Vec<schema::Class>, String>;
    fn prepare(&self, class: &schema::Class) -> Result<Artifacts, String>;
}

/// Resources need not be C++ sources. A provider owns its packaging and build rules.
#[derive(Default)]
pub struct Artifacts {
    pub files: BTreeMap<PathBuf, Vec<u8>>,
    pub native_sources: Vec<PathBuf>,
    pub dependencies: BTreeSet<PathBuf>,
    pub runtime_capabilities: BTreeSet<String>,
    /// Native emitted path -> exact original source key and byte signature.
    pub native_inputs: BTreeMap<PathBuf, (String, String)>,
    pub native_set: Option<String>,
    /// Exact Blueprint compiler membership, including the empty project case.
    pub blueprint_set: Option<String>,
    pub blueprint_footprints: crate::blueprint_dependencies::Footprints,
}

pub(crate) fn cpp_without_comments(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut output = bytes.to_vec();
    let mut index = 0;
    let mut quote = None;
    while index < bytes.len() {
        if let Some(delimiter) = quote {
            if bytes[index] == b'\\' {
                index = (index + 2).min(bytes.len());
                continue;
            }
            if bytes[index] == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        if matches!(bytes[index], b'\'' | b'"') {
            quote = Some(bytes[index]);
            index += 1;
            continue;
        }
        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'/') {
            let start = index;
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            output[start..index].fill(b' ');
            continue;
        }
        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
            let start = index;
            index += 2;
            while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
            for byte in &mut output[start..index] {
                if *byte != b'\n' {
                    *byte = b' ';
                }
            }
            continue;
        }
        index += 1;
    }
    String::from_utf8(output).expect("comment masking preserves UTF-8 bytes")
}

fn cpp_without_literals(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut output = bytes.to_vec();
    let mut index = 0;
    while index < bytes.len() {
        if !matches!(bytes[index], b'\'' | b'"') {
            index += 1;
            continue;
        }
        let delimiter = bytes[index];
        output[index] = b' ';
        index += 1;
        while index < bytes.len() {
            let escaped = bytes[index] == b'\\';
            let end = bytes[index] == delimiter;
            if output[index] != b'\n' {
                output[index] = b' ';
            }
            index += 1;
            if escaped && index < bytes.len() {
                if output[index] != b'\n' {
                    output[index] = b' ';
                }
                index += 1;
            } else if end {
                break;
            }
        }
    }
    String::from_utf8(output).expect("literal masking preserves UTF-8 bytes")
}

fn identifier_calls(source: &str, name: &str) -> bool {
    let bytes = source.as_bytes();
    let mut offset = 0;
    while let Some(found) = source[offset..].find(name) {
        let start = offset + found;
        let end = start + name.len();
        let boundary =
            start == 0 || !bytes[start - 1].is_ascii_alphanumeric() && bytes[start - 1] != b'_';
        let mut next = end;
        while bytes.get(next).is_some_and(u8::is_ascii_whitespace) {
            next += 1;
        }
        if boundary && bytes.get(next) == Some(&b'(') {
            return true;
        }
        offset = end;
    }
    false
}

pub(crate) fn native_code(source: &str) -> String {
    cpp_without_literals(&cpp_without_comments(source))
}
// All supported direct emitter spellings and generic runtime class creation
// retain particles. A dynamic class ID cannot prove the instantiated type.
fn native_particle_demand(source: &str) -> bool {
    native_particle_demand_for(source, true)
}

fn native_particle_demand_for(source: &str, authored: bool) -> bool {
    let comments = cpp_without_comments(source);
    let code = cpp_without_literals(&comments);
    // This sealed SDK constructor only copies values; it creates no actor or
    // emitter. Keep aliases/foreign namespaces conservative. Generated provider
    // bindings use this exact qualified spelling.
    let mut dynamic_code = code.clone();
    let constructor = "epok::ProjectileLibrary::spawn";
    for (start, _) in code.match_indices(constructor) {
        let end = start + constructor.len();
        let bytes = code.as_bytes();
        let before = start == 0
            || bytes[start - 1].is_ascii_whitespace()
            || matches!(
                bytes[start - 1],
                b'(' | b'&' | b'=' | b',' | b';' | b'{' | b'}'
            );
        let after = bytes.get(end).is_none_or(|byte| {
            byte.is_ascii_whitespace() || matches!(byte, b'(' | b';' | b',' | b')' | b']' | b'}')
        });
        if before && after {
            dynamic_code.replace_range(start..end, &" ".repeat(constructor.len()));
        }
    }
    let macros = comments.lines().any(|line| {
        line.trim_start()
            .strip_prefix('#')
            .is_some_and(|directive| {
                let directive = directive.trim_start();
                directive.starts_with("define") || directive.starts_with("undef")
            })
            || line.trim_end().ends_with(char::from(92))
    });
    let qualified = dynamic_code
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    (authored && macros)
        || qualified.contains("::spawn")
        || qualified.contains("::acquire")
        || code.contains("Particle")
        || code.contains("particle_emitter")
        || code.contains("particle_stats")
        || code.contains("play_effect")
        || code.contains("##")
        || [
            "spawn_actor",
            "spawn_batch",
            "spawn_class",
            "find_object_class",
            "object_construct",
            "object_classes",
        ]
        .into_iter()
        .any(|name| dynamic_code.contains(name))
        || ["spawn", "acquire"].into_iter().any(|name| {
            dynamic_code.match_indices(name).any(|(start, _)| {
                let end = start + name.len();
                let bytes = dynamic_code.as_bytes();
                let identifier =
                    |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 128;
                let bounded = (start == 0 || !identifier(bytes[start - 1]))
                    && bytes.get(end).is_none_or(|byte| !identifier(*byte));
                if !bounded {
                    return false;
                }
                // acquire is always conservative. A sealed SDK spawn function
                // cannot be an assignment LHS, array or object member; these
                // are ordinary project variables (e.g. the player's spawn[3]).
                name == "acquire"
                    || !matches!(
                        dynamic_code[end..].trim_start().as_bytes().first().copied(),
                        Some(b'[' | b'.' | b'=')
                    )
            })
        })
        || [
            "1d067605-c408-40b8-b2c2-718b8cf0c601",
            "1d067605-c408-40b8-b2c2-718b8cf0c603",
        ]
        .into_iter()
        .any(|id| comments.contains(id))
}

// This intentionally small allowlist contains shipped SDK headers whose own
// implementation does not create particles implicitly, plus C/C++ standard
// headers. Any other include must be in the observed native source closure;
// external/unknown or macro-generated includes keep the pool conservatively.
fn native_particle_includes_known(source: &str, observed: impl Fn(&str) -> bool) -> bool {
    let source = cpp_without_comments(source);
    if source
        .lines()
        .any(|line| line.trim_end().ends_with(char::from(92)))
    {
        return false;
    }
    for line in source.lines() {
        let Some(directive) = line.trim_start().strip_prefix('#') else {
            continue;
        };
        let directive = directive.trim_start();
        let Some(rest) = directive.strip_prefix("include") else {
            continue;
        };
        let rest = rest.trim_start();
        let (open, close) = match rest.as_bytes().first() {
            Some(b'"') => ('"', '"'),
            Some(b'<') => ('<', '>'),
            _ => return false,
        };
        let Some((header, _)) = rest.strip_prefix(open).and_then(|s| s.split_once(close)) else {
            return false;
        };
        let trusted = matches!(
            header,
            "epok.hpp"
                | "scene.hh"
                | "gameplay_api.hpp"
                | "blueprint_api.hpp"
                | "object_model.hpp"
                | "camera_rig.hpp"
                | "character3d.hpp"
                | "character2d.hpp"
                | "world2d.hpp"
                | "projectile3d.hpp"
                | "pocketstation_async.hpp"
                | "pocketstation_apps.hpp"
                | "retained.hpp"
                | "common/hardware/counters.h"
                | "stdint.h"
                | "stddef.h"
                | "stdbool.h"
                | "string.h"
                | "stdlib.h"
                | "math.h"
                | "limits.h"
                | "cstdint"
                | "cstddef"
                | "cstring"
                | "cstdlib"
                | "cmath"
                | "limits"
                | "type_traits"
                | "array"
                | "algorithm"
                | "utility"
                | "initializer_list"
        );
        if !observed(header) && !trusted {
            return false;
        }
    }
    true
}

pub(crate) fn particle_pool_required(
    scenes: &[crate::scene::Scene],
    needs_effects: bool,
    artifacts: &Artifacts,
) -> bool {
    artifacts.native_set.is_none()
        || needs_effects
        || artifacts.runtime_capabilities.contains("particles")
        || artifacts.runtime_capabilities.contains("effect")
        || artifacts.runtime_capabilities.iter().any(|capability| {
            !matches!(
                capability.as_str(),
                "native"
                    | "blueprint"
                    | "lua"
                    | "lua-vm"
                    | "rumble"
                    | "playback_wait"
                    | "audio"
                    | "body2d"
                    | "camera"
                    | "camera2d"
                    | "canvas"
                    | "canvas2d"
                    | "collider"
                    | "collider2d"
                    | "dialogue"
                    | "focus"
                    | "gameplay-control"
                    | "health"
                    | "image"
                    | "interaction"
                    | "layout"
                    | "light"
                    | "line2d"
                    | "mesh"
                    | "mesh-streaming"
                    | "navigation2d"
                    | "notifier2d"
                    | "palette"
                    | "parallax2d"
                    | "particles2d"
                    | "path2d"
                    | "physics2d"
                    | "polygon2d"
                    | "progress"
                    | "remote2d"
                    | "shadow"
                    | "skeletal-bone-query"
                    | "skeletal-vertex-query"
                    | "skeleton2d"
                    | "sprite"
                    | "sprite2d"
                    | "text"
                    | "tilemap2d"
                    | "timeline"
                    | "viewport2d"
                    | "world-gameplay"
                    | "world2d"
                    | "ui-anchor"
                    | "text-reveal"
                    | "camera-rig"
                    | "character-animation"
                    | "character-commands"
                    | "character-input"
            )
        })
        || scenes.iter().any(|scene| {
            scene
                .actors
                .iter()
                .any(|actor| actor.particle_emitter.is_some() || actor.particle_effect.is_some())
        })
        || artifacts.files.iter().any(|(path, bytes)| {
            if !path.extension().is_some_and(|extension| {
                ["cpp", "cc", "hpp", "hh", "h", "inl"]
                    .iter()
                    .any(|name| extension == *name)
            }) {
                return false;
            }
            let authored = artifacts.native_inputs.contains_key(path);
            // Only compiler-owned generated paths may contain trusted numeric
            // configuration macros. Native/generated path collisions are
            // rejected by prepare_all; native inputs take precedence here.
            if !authored && !path.starts_with("scripts/generated") {
                return true;
            }
            native_particle_demand_for(&String::from_utf8_lossy(bytes), authored)
        })
}

pub(crate) fn native_requirements(source: &str) -> Result<BTreeSet<String>, String> {
    let mut requirements = BTreeSet::new();
    let without_comments = cpp_without_comments(source);
    let code = cpp_without_literals(&without_comments);
    if native_particle_demand(source) {
        requirements.insert("particles".into());
    }
    if [
        "LocomotionClips",
        "LocomotionTuning",
        "LocomotionSelection",
        "LocomotionSample",
        "LocomotionState",
        "LocomotionStatus",
    ]
    .into_iter()
    .chain(
        crate::game_helpers::ANIMATION_CLASSES
            .iter()
            .map(|(name, _)| *name),
    )
    .any(|name| code.contains(name))
    {
        requirements.insert("character-animation".into());
    }
    // These spellings are deliberately conservative. Native source cannot be
    // proven unreachable on the host, so any recognized call retains its
    // corresponding cooker data/service implementation.
    for (name, capability) in [
        ("sample_vertex", "skeletal-vertex-query"),
        ("sample_vertices", "skeletal-vertex-query"),
        ("skeletal_sample_vertex", "skeletal-vertex-query"),
        ("skeletal_sample_vertices", "skeletal-vertex-query"),
        ("sample_bone", "skeletal-bone-query"),
        ("skeletal_sample_bone", "skeletal-bone-query"),
        ("geometry_state", "mesh-streaming"),
        ("request_geometry", "mesh-streaming"),
        ("sample_geometry_vertex", "mesh-streaming"),
        ("mesh_geometry_state", "mesh-streaming"),
        ("request_mesh_geometry", "mesh-streaming"),
        ("sample_mesh_vertex", "mesh-streaming"),
        ("PlaybackLibrary::play_sequence", "timeline"),
        ("PlaybackLibrary::play_effect", "effect"),
        ("rumble", "rumble"),
        ("stop_rumble", "rumble"),
        ("rumble_supported", "rumble"),
        ("set_rumble_enabled", "rumble"),
        ("rumble_snapshot", "rumble"),
        ("rumble.play", "rumble"),
    ] {
        if identifier_calls(&code, name) {
            requirements.insert(capability.to_string());
        }
    }
    if code.contains("GameplayInputLibrary") || code.contains("gameplay_controls") {
        requirements.insert("gameplay-control".into());
    }
    if [
        "CharacterCommandResult3D",
        "CharacterCommandResult2D",
        "CharacterCommandStatus",
    ]
    .into_iter()
    .chain(
        crate::game_helpers::COMMAND_SOURCE_CLASSES
            .iter()
            .map(|(name, _)| *name),
    )
    .any(|name| code.contains(name))
    {
        requirements.insert("character-commands".into());
    }
    if crate::game_helpers::CHARACTER_INPUT_CLASSES
        .iter()
        .any(|(name, _)| code.contains(name))
    {
        requirements.extend(
            [
                "character-input",
                "character-commands",
                "gameplay-control",
                "camera-rig",
            ]
            .map(str::to_owned),
        );
    }
    if ["CameraRigSample", "SpringArmSample"]
        .into_iter()
        .chain(
            crate::game_helpers::CAMERA_CLASSES
                .iter()
                .map(|(name, _)| *name),
        )
        .any(|name| code.contains(name))
    {
        requirements.insert("camera-rig".into());
    }
    if code.contains("visible_glyphs") || identifier_calls(&code, "show_all_text") {
        requirements.insert("text-reveal".into());
    }
    if code.contains("DialogueSessionLibrary")
        || crate::game_helpers::DIALOGUE_CLASSES
            .iter()
            .any(|(name, _)| code.contains(name))
    {
        requirements.insert("dialogue".into());
    }
    if ["HelperResult", "DoorState"]
        .into_iter()
        .chain(
            crate::game_helpers::WORLD_GAMEPLAY_CLASSES
                .iter()
                .map(|(name, _)| *name),
        )
        .any(|name| code.contains(name))
    {
        requirements.insert("world-gameplay".into());
    }
    // Opaque native libraries can declare requirements without teaching this
    // scanner their call syntax. The macro expands to no target code.
    let macro_name = "EPOK_NATIVE_REQUIREMENTS";
    let mut offset = 0;
    while let Some(found) = code[offset..].find(macro_name) {
        let start = offset + found;
        let end = start + macro_name.len();
        let bytes = code.as_bytes();
        let boundary =
            start == 0 || !bytes[start - 1].is_ascii_alphanumeric() && bytes[start - 1] != b'_';
        let mut open = end;
        while bytes.get(open).is_some_and(u8::is_ascii_whitespace) {
            open += 1;
        }
        if !boundary || bytes.get(open) != Some(&b'(') {
            offset = end;
            continue;
        }
        let close = code[open + 1..]
            .find(')')
            .map(|value| open + 1 + value)
            .ok_or("Unterminated EPOK_NATIVE_REQUIREMENTS declaration")?;
        let body = &without_comments[open + 1..close];
        for value in body.split(',') {
            let value = value.trim().trim_matches('"');
            if value.is_empty()
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            {
                return Err(format!("Invalid native capability requirement '{value}'"));
            }
            requirements.insert(value.to_string());
        }
        offset = close + 1;
    }
    Ok(requirements)
}

pub fn prepare_native(root: &Path) -> Result<Artifacts, String> {
    let mut artifacts = Artifacts::default();
    let files = crate::staging_files::native_files(root)?;
    let observed = files
        .iter()
        .filter_map(|path| std::fs::canonicalize(path).ok())
        .collect::<BTreeSet<_>>();
    artifacts.native_set = Some(crate::staging_files::native_set(root, &files)?);
    let mut requirement_sources = BTreeMap::<String, Vec<String>>::new();
    for path in files {
        let relative = Path::new("scripts").join(
            path.strip_prefix(root.join("assets/scripts"))
                .map_err(|e| e.to_string())?,
        );
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&bytes);
        let mut requirements = native_requirements(&text)?;
        if !native_particle_includes_known(&text, |header| {
            [
                path.parent().unwrap_or(root).join(header),
                root.join("assets/scripts").join(header),
            ]
            .iter()
            .any(|include| {
                std::fs::canonicalize(include).is_ok_and(|resolved| observed.contains(&resolved))
            })
        }) {
            requirements.insert("particles".into());
        }
        for requirement in requirements {
            artifacts.runtime_capabilities.insert(requirement.clone());
            requirement_sources
                .entry(requirement)
                .or_default()
                .push(crate::staging_files::native_key(root, &path)?);
        }
        artifacts.native_inputs.insert(
            relative.clone(),
            (
                crate::staging_files::native_key(root, &path)?,
                crate::assets::hash(&bytes),
            ),
        );
        artifacts.files.insert(relative.clone(), bytes);
        artifacts.dependencies.insert(path.clone());
        if path.extension().is_some_and(|e| e == "cpp" || e == "cc") {
            artifacts.native_sources.push(relative);
        }
    }
    artifacts.runtime_capabilities.insert("native".into());
    artifacts.files.insert(
        PathBuf::from("scripts/generated/native-requirements.json"),
        crate::document::to_vec(&serde_json::json!({
            "version": 1,
            "requirements": requirement_sources,
        }))
        .map_err(|error| error.to_string())?,
    );
    Ok(artifacts)
}
pub fn prepare_all(root: &Path, native: &[Script]) -> Result<Artifacts, String> {
    let mut artifacts = prepare_native(root)?;
    artifacts.blueprint_set = Some(crate::blueprint_dependencies::source_set(&[]));
    // Lua first: its generated classes are ordinary catalog entries, so the
    // Blueprint compiler sees them as eligible parents.
    let mut catalog = native.to_vec();
    if let Some(compiled) = crate::scripts::compile_lua(root, native)? {
        for (path, bytes) in compiled.artifacts.files {
            if artifacts.files.insert(path.clone(), bytes).is_some() {
                return Err(format!(
                    "Lua artifact collides with native source: {}",
                    path.display()
                ));
            }
        }
        artifacts
            .native_sources
            .extend(compiled.artifacts.native_sources);
        artifacts
            .dependencies
            .extend(compiled.artifacts.dependencies);
        artifacts
            .runtime_capabilities
            .extend(compiled.artifacts.runtime_capabilities);
        catalog.extend(compiled.scripts);
        catalog.sort_by(|a, b| a.name.cmp(&b.name));
    }
    let native = &catalog;
    if let Some(compiled) = crate::scripts::compile_blueprints(root, native)? {
        artifacts.blueprint_set = compiled.artifacts.blueprint_set;
        artifacts.blueprint_footprints = compiled.footprints;
        for (path, bytes) in compiled.artifacts.files {
            if artifacts.files.insert(path.clone(), bytes).is_some() {
                return Err(format!(
                    "Blueprint artifact collides with native source: {}",
                    path.display()
                ));
            }
        }
        artifacts
            .native_sources
            .extend(compiled.artifacts.native_sources);
        artifacts
            .dependencies
            .extend(compiled.artifacts.dependencies);
        artifacts
            .runtime_capabilities
            .extend(compiled.artifacts.runtime_capabilities);
    }
    Ok(artifacts)
}

pub fn blueprint_provider() -> schema::Extension {
    schema::Extension {
        id: "blueprint".into(),
        version: 1,
    }
}
pub fn lua_provider() -> schema::Extension {
    schema::Extension {
        id: "lua".into(),
        version: 1,
    }
}
impl Artifacts {
    pub fn stage(&self, root: &Path, destination: &Path) -> Result<(), String> {
        for path in self.files.keys().chain(&self.native_sources) {
            if path.as_os_str().is_empty()
                || path
                    .components()
                    .any(|c| !matches!(c, Component::Normal(_)))
            {
                return Err("Provider artifact paths must remain relative to the export.".into());
            }
        }
        for source in &self.native_sources {
            if !self.files.contains_key(source) {
                return Err(format!("Missing provider source {}", source.display()));
            }
        }
        let mut dependencies = BTreeSet::new();
        for path in &self.dependencies {
            let relative = if path.is_absolute() {
                match path.strip_prefix(root) {
                    Ok(relative) => relative.to_owned(),
                    Err(_) => {
                        let source = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
                        let project = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
                        source.strip_prefix(project).map_err(|_| format!(
                            "Script dependency {} is outside the project; portable manifests require project-relative sources",
                            path.display()
                        ))?.to_owned()
                    }
                }
            } else {
                path.clone()
            };
            if relative.as_os_str().is_empty()
                || relative
                    .components()
                    .any(|c| !matches!(c, Component::Normal(_)))
            {
                return Err(format!(
                    "Script dependency {} must remain relative to the project",
                    path.display()
                ));
            }
            dependencies.insert(relative.to_string_lossy().replace('\\', "/"));
        }
        // Generated v1 manifests carried host paths. Regeneration emits v2;
        // authoring inputs remain unchanged and are not needed by standalone make.
        let manifest = crate::document::to_vec(&serde_json::json!({
            "version": 2,
            "file_base": "manifest",
            "dependency_base": "project",
            "files": self.files.keys().map(|p| p.to_string_lossy().replace('\\', "/")).collect::<Vec<_>>(),
            "native_sources": self.native_sources.iter().map(|p| p.to_string_lossy().replace('\\', "/")).collect::<Vec<_>>(),
            "dependencies": dependencies,
            "runtime_capabilities": self.runtime_capabilities,
        }))
        .map_err(|e| e.to_string())?;
        for (path, bytes) in &self.files {
            crate::project::write_changed(&destination.join(path), bytes)?;
        }
        crate::project::write_changed(&destination.join("Scripts.epokmanifest"), &manifest)?;
        Ok(())
    }
}

pub fn capabilities(provider: &schema::Extension) -> Result<Capabilities, String> {
    match (provider.id.as_str(), provider.version) {
        ("cpp" | "blueprint" | "lua", 1) => Ok(Capabilities {
            create: true,
            derive_backends: BTreeSet::from(["native".into()]),
            attach: true,
            invoke: true,
        }),
        ("legacy-cpp", 1) => Ok(Capabilities {
            create: false,
            derive_backends: BTreeSet::new(),
            attach: true,
            invoke: false,
        }),
        _ => Err(format!(
            "Authoring provider {} v{} is unavailable. Its serialized values are preserved; enable a compatible provider or explicitly migrate the component.",
            provider.id, provider.version
        )),
    }
}
pub fn can_derive(author: &schema::Extension, parent: &schema::Class) -> bool {
    derivation_supported(
        author,
        &parent.provider,
        &parent.backend,
        parent.blueprintable,
        parent.final_class,
    )
}

pub fn derivation_supported(
    author: &schema::Extension,
    provider: &schema::Extension,
    backend: &schema::Extension,
    blueprintable: bool,
    final_class: bool,
) -> bool {
    capabilities(author).is_ok_and(|c| c.create && c.derive_backends.contains(&backend.id))
        // A Lua class is a real native subclass in every execution mode, so it
        // is an eligible parent for every author. Lua itself derives from C++
        // or Lua only; Blueprint parents wait for a joint declaration order.
        && (*provider == schema::native_provider()
            || *provider == lua_provider()
            || (*author == blueprint_provider() && *provider == blueprint_provider()))
        && *backend == schema::native_backend()
        && blueprintable
        && !final_class
}
pub fn validate_binding(binding: &ClassDefaults) -> Result<(), String> {
    if !capabilities(&binding.provider)?.attach {
        return Err("This provider cannot attach instances.".into());
    }
    backend(&binding.backend)?;
    Ok(())
}

#[allow(dead_code)] // Reserved backend API like AuthoringProvider; only the tests use it today.
pub trait ExecutionBackend {
    fn declaration(&self, class: &str, instance: &str) -> String;
    fn reset(&self, class: &str, instance: &str) -> String;
}
pub struct Native;
impl ExecutionBackend for Native {
    fn declaration(&self, class: &str, instance: &str) -> String {
        format!("inline {class} {instance};\n")
    }
    fn reset(&self, class: &str, instance: &str) -> String {
        format!("{instance}={class}{{}};\n")
    }
}
#[allow(dead_code)] // Part of the reserved ExecutionBackend API.
pub fn backend(id: &schema::Extension) -> Result<&'static dyn ExecutionBackend, String> {
    if *id == schema::native_backend() {
        Ok(&Native)
    } else {
        Err(format!(
            "Execution backend {} v{} is unavailable; no C++ instance or reset was generated. Preserve this component until its backend is available.",
            id.id, id.version
        ))
    }
}
#[allow(dead_code)] // Part of the reserved ExecutionBackend API.
pub fn resolve<'a>(binding: &ClassDefaults, catalog: &'a [Script]) -> Result<&'a Script, String> {
    validate_binding(binding)?;
    let script = if let Some(id) = &binding.class_id {
        catalog.iter().find(|s| {
            s.classes
                .first()
                .map(|c| c.id.clone())
                .unwrap_or_else(|| format!("legacy:{}", s.name))
                == *id
        })
    } else {
        catalog.iter().find(|s| s.name == binding.name)
    }
    .ok_or_else(|| {
        format!(
            "Missing class {} ({:?}); the component is preserved for explicit migration.",
            binding.name, binding.class_id
        )
    })?;
    for (name, id) in &binding.member_ids {
        if binding.properties.contains_key(name)
            && !script.properties.iter().any(|p| {
                p.name == *name && (p.id == *id || id == &format!("legacy:{}:{name}", script.name))
            })
        {
            return Err(format!(
                "Property {name} ({id}) changed identity or name; its stored value is preserved. Migrate the member explicitly."
            ));
        }
    }
    Ok(script)
}

/// Serializable, checked call boundary for future interpreted destinations.
/// Mutable references/opaque records require a backend-owned adapter, never a cast.
#[allow(dead_code)]
pub fn validate_invocation(
    provider: &schema::Extension,
    function: &schema::Function,
    args: &[serde_json::Value],
) -> Result<(), String> {
    if !capabilities(provider)?.invoke {
        return Err("Provider does not support typed invocation.".into());
    }
    if !function.callable || function.access != "public" || args.len() != function.parameters.len()
    {
        return Err("Invocation visibility or arity mismatch.".into());
    }
    for (parameter, value) in function.parameters.iter().zip(args) {
        if parameter.direction != schema::Direction::Value
            || !crate::script_values::valid(value, &parameter.value_type)
        {
            return Err(format!(
                "Argument {} requires its declared type and an explicit reference adapter.",
                parameter.name
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locomotion_types_select_animation_without_input_or_query_services() {
        for name in [
            "LocomotionClips",
            "LocomotionTuning",
            "LocomotionSelection",
            "LocomotionSample",
            "LocomotionState",
            "LocomotionStatus",
        ]
        .into_iter()
        .chain(
            crate::game_helpers::ANIMATION_CLASSES
                .iter()
                .map(|(name, _)| *name),
        ) {
            assert_eq!(
                native_requirements(&format!("using Animation=epok::{name};")).unwrap(),
                BTreeSet::from(["character-animation".into()])
            );
            assert!(
                native_requirements(&format!("// {name}\nconst char* ignored=\"{name}\";"))
                    .unwrap()
                    .is_empty()
            );
        }
    }
    #[test]
    fn character_command_native_types_select_only_their_required_services() {
        for name in [
            "CharacterCommandResult3D",
            "CharacterCommandResult2D",
            "CharacterCommandStatus",
        ]
        .into_iter()
        .chain(
            crate::game_helpers::COMMAND_SOURCE_CLASSES
                .iter()
                .map(|(name, _)| *name),
        ) {
            assert_eq!(
                native_requirements(&format!("using Source = epok::{name};")).unwrap(),
                BTreeSet::from(["character-commands".into()])
            );
            assert!(
                native_requirements(&format!("// epok::{name}\nconst char* ignored=\"{name}\";"))
                    .unwrap()
                    .is_empty()
            );
        }
        for (name, _) in crate::game_helpers::CHARACTER_INPUT_CLASSES {
            assert_eq!(
                native_requirements(&format!("using Source = epok::{name};")).unwrap(),
                BTreeSet::from(
                    [
                        "character-input",
                        "character-commands",
                        "gameplay-control",
                        "camera-rig"
                    ]
                    .map(str::to_owned)
                )
            );
        }
    }
    #[test]
    fn camera_native_types_select_optional_header_without_comment_or_string_matches() {
        for name in ["CameraRigSample", "SpringArmSample"].into_iter().chain(
            crate::game_helpers::CAMERA_CLASSES
                .iter()
                .map(|(name, _)| *name),
        ) {
            let source = format!("using Camera = epok::{name};");
            assert!(native_requirements(&source).unwrap().contains("camera-rig"));
            for ignored in [
                format!("// {source}"),
                format!("const char* name=\"{name}\";"),
            ] {
                assert!(native_requirements(&ignored).unwrap().is_empty());
            }
        }
    }
    #[test]
    fn particle_pool_demand_preserves_native_dynamic_and_opaque_uses() {
        for source in [
            "value.particle_emitter.enabled=true;",
            "using E=epok::ParticleEmitter;",
            "epok::ParticleEmitterComponent* emitter;",
            "auto spawn=&epok::Level::spawn_actor;",
            "auto type=epok::find_object_class(runtime_id);",
            "auto factory=&epok::ObjectRegistry::acquire;",
            "epok::bp::api::spawn_class(base, input, parent);",
            "auto effect=&epok::PlaybackLibrary::play_effect;",
            "#define FIELD(a,b) a ## b",
            "#define TAKE acquire\nTAKE(type);",
            "auto factory=& acquire;",
            "auto factory=acquire;",
            "using namespace epok::bp::api; auto factory=spawn;",
            "install(spawn);",
            "constexpr auto id=\"1d067605-c408-40b8-b2c2-718b8cf0c601\";",
        ] {
            assert!(
                native_requirements(source).unwrap().contains("particles"),
                "{source}"
            );
        }
        for source in [
            "// actor.particle_emitter.enabled=true;",
            "const char* label=\"ParticleEmitterComponent\";",
            "epok::Fixed spawn[3]{};",
            "auto* rules=registry->resolve<HealthRules>(policy);",
            "auto& spawn=encounters[index];spawn.health=health;",
            "return epok::ProjectileLibrary::spawn(origin,velocity,lifetime,0);",
            "auto constructor=&epok::ProjectileLibrary::spawn;",
        ] {
            assert!(
                !native_requirements(source).unwrap().contains("particles"),
                "{source}"
            );
        }
        assert!(native_particle_demand(
            "foreign::epok::ProjectileLibrary::spawn();"
        ));
        assert!(native_particle_demand(
            "epok::ProjectileLibrary::spawn_other();"
        ));
        assert!(native_particle_includes_known(
            "#include \"epok.hpp\"\n#include \"Observed.hpp\"",
            |name| name == "Observed.hpp"
        ));
        for source in [
            "#include \"Opaque.hpp\"",
            "#include EXTERNAL_HEADER",
            "#include_next <epok.hpp>",
        ] {
            assert!(
                !native_particle_includes_known(source, |_| false),
                "{source}"
            );
        }
        assert!(native_particle_includes_known(
            "// #include \"Opaque.hpp\"",
            |_| false
        ));
    }

    #[test]
    fn particle_pool_demand_includes_every_bank_templates_and_generated_provider_calls() {
        let scene = crate::scene::Scene::default();
        let mut artifacts = Artifacts::default();
        assert!(particle_pool_required(
            std::slice::from_ref(&scene),
            false,
            &artifacts
        ));
        artifacts.native_set = Some("observed-test-native-closure".into());
        assert!(!particle_pool_required(
            std::slice::from_ref(&scene),
            false,
            &artifacts
        ));
        assert!(particle_pool_required(
            std::slice::from_ref(&scene),
            true,
            &artifacts
        ));
        for capability in ["particles", "effect", "opaque-extension"] {
            artifacts.runtime_capabilities.insert(capability.into());
            assert!(particle_pool_required(
                std::slice::from_ref(&scene),
                false,
                &artifacts
            ));
            artifacts.runtime_capabilities.clear();
        }
        let mut emitter_bank = scene.clone();
        let mut emitter = crate::scene::Actor::cube("Emitter".into());
        emitter.particle_emitter = Some(Default::default());
        emitter_bank.actors.push(emitter);
        assert!(particle_pool_required(
            &[scene.clone(), emitter_bank],
            false,
            &artifacts
        ));
        artifacts.files.insert(
            "scripts/generated/lua/lua_bindings.cpp".into(),
            b"self.resolve<epok::ParticleEmitterComponent>(id)->play();".to_vec(),
        );
        assert!(particle_pool_required(&[scene], false, &artifacts));
    }

    #[test]
    fn particle_pool_demand_distinguishes_compiler_configuration_from_authored_macros() {
        let mut artifacts = Artifacts {
            native_set: Some("observed".into()),
            ..Default::default()
        };
        let path = PathBuf::from("scripts/generated/lua/lua_value_config.hpp");
        artifacts.files.insert(
            path.clone(),
            b"#define EPOK_LUA_REQUIRED_VALUE_WORDS 10".to_vec(),
        );
        assert!(!particle_pool_required(&[], false, &artifacts));
        artifacts
            .native_inputs
            .insert(path, ("native-file".into(), "hash".into()));
        assert!(particle_pool_required(&[], false, &artifacts));
        artifacts
            .files
            .insert("scripts/unproven.hpp".into(), b"void opaque();".to_vec());
        artifacts.native_inputs.clear();
        assert!(particle_pool_required(&[], false, &artifacts));
    }

    #[test]
    fn dialogue_text_reveal_native_calls_select_capabilities_without_comment_matches() {
        for source in [
            "text.visible_glyphs = 3;",
            "text.set_visible_glyphs (3);",
            "text.show_all_text ();",
        ] {
            assert!(native_requirements(source).unwrap().contains("text-reveal"));
        }
        for source in [
            "epok::DialogueSessionLibrary::snapshot();",
            "using Speaker=epok::DialogueInteractionComponent;",
            "EPOK_NATIVE_REQUIREMENTS(\"dialogue\")",
        ] {
            assert!(native_requirements(source).unwrap().contains("dialogue"));
        }
        for source in [
            "// text.visible_glyphs = 3; DialogueSessionLibrary::snapshot();",
            "const char* label = \"DialogueInteractionComponent visible_glyphs\";",
            "/* text.show_all_text(); */",
        ] {
            assert!(native_requirements(source).unwrap().is_empty());
        }
    }
    #[test]
    fn native_requirement_observation_is_lexical_and_supports_explicit_metadata() {
        let requirements = native_requirements(
            r#"
            // mesh.sample_bone(0);
            const char* ignored = "sample_vertices(";
            mesh.sample_vertex (4);
            mesh.request_geometry();
            epok::PlaybackLibrary::play_effect(component);
            EPOK_NATIVE_REQUIREMENTS("skeletal-bone-query", "custom-2d")
            "#,
        )
        .unwrap();
        assert_eq!(
            requirements,
            BTreeSet::from([
                "custom-2d".into(),
                "effect".into(),
                "particles".into(),
                "mesh-streaming".into(),
                "skeletal-bone-query".into(),
                "skeletal-vertex-query".into(),
            ])
        );
        assert!(
            native_requirements("/* sample_vertex(0); */")
                .unwrap()
                .is_empty()
        );
        assert!(
            native_requirements("EPOK_NATIVE_REQUIREMENTS(\"Bad Name\")")
                .unwrap_err()
                .contains("Invalid native capability")
        );
        assert!(
            native_requirements("EPOK_NATIVE_REQUIREMENTS(\"broken\"")
                .unwrap_err()
                .contains("Unterminated")
        );
    }

    #[test]
    fn gameplay_control_native_aliases_and_spaced_calls_select_the_runtime() {
        for source in [
            "using Controls=epok::GameplayInputLibrary; Controls::sample(action,0);",
            "epok::GameplayInputLibrary :: mask(action);",
            "epok::gameplay_controls ().begin_frame();",
        ] {
            assert!(
                native_requirements(source)
                    .unwrap()
                    .contains("gameplay-control")
            );
        }
        for source in [
            "// GameplayInputLibrary::sample(action,0);",
            "const char* text=\"GameplayInputLibrary\";",
        ] {
            assert!(native_requirements(source).unwrap().is_empty());
        }
    }

    #[test]
    fn script_manifests_are_portable_and_regenerate_v1_without_changing_sources() {
        let fixture = crate::workspace::tests::temp("portable-script-manifest");
        let mut outputs = vec![];
        for name in ["Original Project", "Copied Project"] {
            let root = fixture.join(name);
            let source = root.join("assets/scripts/Nested/Probe.cpp");
            std::fs::create_dir_all(source.parent().unwrap()).unwrap();
            let bytes = b"int probe=7;\n";
            std::fs::write(&source, bytes).unwrap();
            let artifacts = prepare_native(&root).unwrap();
            assert!(artifacts.dependencies.contains(&source));
            let destination = root.join("exports/test");
            std::fs::create_dir_all(&destination).unwrap();
            let manifest = destination.join("Scripts.epokmanifest");
            std::fs::write(&manifest, b"version: 1\ndependencies: [old-host-path]\n").unwrap();
            artifacts.stage(&root, &destination).unwrap();
            let generated = std::fs::read(&manifest).unwrap();
            let value: serde_json::Value = crate::document::from_slice(&generated).unwrap();
            assert_eq!(value["version"], 2);
            assert_eq!(value["file_base"], "manifest");
            assert_eq!(value["dependency_base"], "project");
            assert_eq!(
                value["dependencies"],
                serde_json::json!(["assets/scripts/Nested/Probe.cpp"])
            );
            assert_eq!(
                value["native_sources"],
                serde_json::json!(["scripts/Nested/Probe.cpp"])
            );
            assert_eq!(std::fs::read(&source).unwrap(), bytes);
            outputs.push(generated);
        }
        assert_eq!(outputs[0], outputs[1]);
        let root = fixture.join("Rejected");
        std::fs::create_dir_all(&root).unwrap();
        let manifest = root.join("Scripts.epokmanifest");
        let previous = b"previous generated artifact";
        std::fs::write(&manifest, previous).unwrap();
        let mut artifacts = Artifacts {
            files: BTreeMap::from([("Probe.cpp".into(), b"new output".to_vec())]),
            dependencies: BTreeSet::from(["../outside.cpp".into()]),
            ..Default::default()
        };
        assert!(artifacts.stage(&root, &root).is_err());
        assert_eq!(std::fs::read(manifest).unwrap(), previous);
        assert!(!root.join("Probe.cpp").exists());
        let external = fixture.join("External.cpp");
        std::fs::write(&external, b"int external=1;").unwrap();
        artifacts.dependencies = BTreeSet::from([external]);
        assert!(
            artifacts
                .stage(&root, &root)
                .unwrap_err()
                .contains("outside the project")
        );
        assert!(!root.join("Probe.cpp").exists());
    }
    struct Fake;
    impl AuthoringProvider for Fake {
        fn identity(&self) -> schema::Extension {
            schema::Extension {
                id: "fake-author".into(),
                version: 1,
            }
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                create: true,
                derive_backends: BTreeSet::from(["fake-vm".into()]),
                attach: true,
                invoke: true,
            }
        }
        fn declarations(&self) -> Result<Vec<schema::Class>, String> {
            Ok(vec![schema::Class {
                family: None,
                domain: None,
                placement: Default::default(),
                component: None,
                default_components: vec![],
                explicit_abstract: false,
                default_overrides: Default::default(),
                id: "fake:enemy".into(),
                provider: self.identity(),
                backend: schema::Extension {
                    id: "fake-vm".into(),
                    version: 1,
                },
                cpp_name: "FakeEnemy".into(),
                parent: None,
                abstract_class: false,
                final_class: false,
                timeline_component: None,
                blueprintable: true,
                properties: vec![schema::Property {
                    id: "fake:health".into(),
                    name: "health".into(),
                    value_type: schema::Type::Int32,
                    default: serde_json::json!(100),
                    editable: true,
                    timeline: None,
                    source: schema::Location {
                        file: "enemy.fake".into(),
                        line: 1,
                        column: 1,
                    },
                }],
                functions: vec![],
                source: schema::Location {
                    file: "enemy.fake".into(),
                    line: 1,
                    column: 1,
                },
            }])
        }
        fn prepare(&self, class: &schema::Class) -> Result<Artifacts, String> {
            if class.provider != self.identity() {
                return Err("Provider identity mismatch".into());
            }
            Ok(Artifacts {
                files: BTreeMap::from([("resources/enemy.bytecode".into(), vec![1, 2, 3])]),
                dependencies: BTreeSet::from(["enemy.fake".into()]),
                runtime_capabilities: BTreeSet::from(["fake-vm".into()]),
                ..Default::default()
            })
        }
    }
    struct FakeVm;
    impl ExecutionBackend for FakeVm {
        fn declaration(&self, _: &str, instance: &str) -> String {
            format!("FakeHandle {instance};")
        }
        fn reset(&self, _: &str, instance: &str) -> String {
            format!("cancel_tasks({instance}); release({instance}); {instance}=new_environment();")
        }
    }
    #[test]
    fn non_cpp_provider_artifacts_and_lifecycle_are_backend_owned() {
        let provider = Fake;
        let class = provider.declarations().unwrap().remove(0);
        assert!(provider.capabilities().derive_backends.contains("fake-vm"));
        assert!(!can_derive(&schema::native_provider(), &class));
        let artifacts = provider.prepare(&class).unwrap();
        assert!(artifacts.native_sources.is_empty());
        assert!(artifacts.dependencies.contains(Path::new("enemy.fake")));
        assert!(artifacts.runtime_capabilities.contains("fake-vm"));
        let dir = crate::workspace::tests::temp("fake-provider");
        artifacts.stage(&dir, &dir).unwrap();
        assert_eq!(
            std::fs::read(dir.join("resources/enemy.bytecode")).unwrap(),
            vec![1, 2, 3]
        );
        let reset = FakeVm.reset("FakeEnemy", "enemy");
        assert!(reset.contains("cancel_tasks(enemy); release(enemy)"));
        assert!(!reset.contains("FakeEnemy{}"));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn unavailable_instances_roundtrip_without_reinterpretation() {
        let binding = ClassDefaults {
            name: "FakeEnemy".into(),
            provider: Fake.identity(),
            backend: schema::Extension {
                id: "fake-vm".into(),
                version: 1,
            },
            class_id: Some("fake:enemy".into()),
            properties: BTreeMap::from([("health".into(), serde_json::json!(5))]),
            overrides: BTreeSet::from(["health".into()]),
            ..Default::default()
        };
        let bytes = serde_json::to_vec(&binding).unwrap();
        let loaded: ClassDefaults = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(loaded, binding);
        assert!(resolve(&loaded, &[]).unwrap_err().contains("unavailable"));
        assert_eq!(serde_json::to_vec(&loaded).unwrap(), bytes);
        assert!(
            backend(&loaded.backend)
                .err()
                .unwrap()
                .contains("no C++ instance")
        );
    }
    #[test]
    fn invocation_checks_types_arity_visibility_and_reference_policy() {
        let function = schema::Function {
            id: "damage".into(),
            name: "damage".into(),
            parameters: vec![schema::Parameter {
                name: "amount".into(),
                value_type: schema::Type::Fixed,
                direction: schema::Direction::Value,
            }],
            returns: schema::Type::Void,
            callable: true,
            timeline: None,
            event: false,
            pure: false,
            resource_demands: vec![],
            abstract_method: false,
            final_method: false,
            access: "public".into(),
            overrides: vec![],
            source: schema::Location {
                file: "Enemy.hpp".into(),
                line: 1,
                column: 1,
            },
        };
        assert!(
            validate_invocation(
                &schema::native_provider(),
                &function,
                &[serde_json::json!(2.5)]
            )
            .is_ok()
        );
        for arguments in [
            vec![],
            vec![serde_json::json!(true)],
            vec![serde_json::json!(524288)],
        ] {
            assert!(
                validate_invocation(&schema::native_provider(), &function, &arguments).is_err()
            );
        }
        let mut reference = function.clone();
        reference.parameters[0].direction = schema::Direction::MutableReference;
        assert!(
            validate_invocation(
                &schema::native_provider(),
                &reference,
                &[serde_json::json!(2.5)]
            )
            .unwrap_err()
            .contains("adapter")
        );
        assert!(
            validate_invocation(&Fake.identity(), &function, &[serde_json::json!(2.5)])
                .unwrap_err()
                .contains("unavailable")
        );
    }

    fn parent(
        provider: schema::Extension,
        blueprintable: bool,
        final_class: bool,
    ) -> schema::Class {
        schema::Class {
            family: None,
            domain: None,
            placement: Default::default(),
            component: None,
            default_components: vec![],
            explicit_abstract: false,
            default_overrides: Default::default(),
            id: "lua-test:parent".into(),
            provider,
            backend: schema::native_backend(),
            cpp_name: "EnemyBase".into(),
            parent: None,
            abstract_class: false,
            final_class,
            timeline_component: None,
            blueprintable,
            properties: vec![],
            functions: vec![],
            source: schema::Location {
                file: "EnemyBase.hpp".into(),
                line: 1,
                column: 1,
            },
        }
    }
    #[test]
    fn lua_provider_creates_classes_and_derives_across_authoring_providers() {
        let lua = lua_provider();
        let lua_capabilities = capabilities(&lua).unwrap();
        assert!(lua_capabilities.create && lua_capabilities.attach && lua_capabilities.invoke);
        assert!(lua_capabilities.derive_backends.contains("native"));
        // A Lua class is a native subclass in every execution mode, so it is an
        // eligible parent everywhere; Lua itself derives from C++ or Lua only.
        assert!(can_derive(
            &lua,
            &parent(schema::native_provider(), true, false)
        ));
        assert!(!can_derive(
            &lua,
            &parent(blueprint_provider(), true, false)
        ));
        assert!(can_derive(&lua, &parent(lua_provider(), true, false)));
        assert!(can_derive(
            &blueprint_provider(),
            &parent(lua_provider(), true, false)
        ));
        assert!(can_derive(
            &schema::native_provider(),
            &parent(lua_provider(), true, false)
        ));
        assert!(!can_derive(
            &lua,
            &parent(schema::native_provider(), true, true)
        ));
        assert!(!can_derive(
            &lua,
            &parent(schema::native_provider(), false, false)
        ));
        let binding = crate::scene::ClassDefaults {
            provider: lua,
            backend: schema::native_backend(),
            ..Default::default()
        };
        validate_binding(&binding).unwrap();
        // The unavailable-provider diagnostic no longer claims Lua is missing.
        let message = capabilities(&schema::Extension {
            id: "unknown".into(),
            version: 9,
        })
        .unwrap_err();
        assert!(!message.contains("Lua"));
        assert!(message.contains("unavailable"));
    }
}
