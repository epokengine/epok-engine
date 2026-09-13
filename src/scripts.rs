//! Native script catalog and legacy metadata adapter.
use crate::{reflection_schema as schema, script_values};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub default: Value,
    #[serde(default)]
    pub value_type: schema::Type,
    #[serde(default)]
    pub id: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Script {
    pub name: String,
    #[serde(default)]
    pub parent: Option<String>,
    pub properties: Vec<Property>,
    #[serde(skip)]
    pub header: PathBuf,
    #[serde(skip)]
    pub classes: Vec<schema::Class>,
}
impl Script {
    pub fn header_path(&self) -> String {
        if self.header.as_os_str().is_empty() {
            format!("{}.hpp", self.name)
        } else {
            self.header.to_string_lossy().replace('\\', "/")
        }
    }
    pub fn instantiable(&self) -> bool {
        self.classes.first().is_none_or(|c| !c.abstract_class)
    }
}
pub fn identifier(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(),Some(c) if c.is_ascii_alphabetic())
        && s.len() <= 64 && !s.contains("__")
        && chars.all(|c|c.is_ascii_alphanumeric() || c == '_')
        && !("alignas alignof and and_eq asm atomic_cancel atomic_commit atomic_noexcept auto bitand bitor bool break case catch char char8_t char16_t char32_t class compl concept const consteval constexpr constinit const_cast continue co_await co_return co_yield decltype default delete do double dynamic_cast else enum explicit export extern false float for friend goto if inline int long mutable namespace new noexcept not not_eq nullptr operator or or_eq private protected public reflexpr register reinterpret_cast requires return short signed sizeof static static_assert static_cast struct switch synchronized template this thread_local throw true try typedef typeid typename union unsigned using virtual void volatile wchar_t while xor xor_eq final override".split_whitespace().any(|word|word == s))
}
pub fn class_identifier(s: &str) -> bool {
    !s.is_empty() && s.split("::").all(identifier)
}

pub fn catalog(root: &Path) -> Result<Vec<Script>, String> {
    let result = catalog_inner(root);
    crate::scene_dependencies::observe_catalog(root, result.as_deref().map_err(String::as_str))?;
    result
}
fn catalog_inner(root: &Path) -> Result<Vec<Script>, String> {
    let mut scripts = native_catalog(root)?;
    if let Some(compiled) = compile_blueprints(root, &scripts)? {
        scripts.extend(compiled.scripts);
        scripts.sort_by(|a, b| a.name.cmp(&b.name));
    }
    Ok(scripts)
}

/// Current authoring types for dependency observation, using the compiler's
/// declaration pass without executable compilation or cached artifact fallback.
pub fn declaration_registry(root: &Path) -> Result<crate::blueprint::Registry, String> {
    let scripts = native_catalog(root)?;
    let files = crate::blueprint_asset::load_all(root)?;
    if files.is_empty() {
        return Ok(crate::blueprint::legacy_registry(root, &scripts));
    }
    let native = crate::blueprint::native_registry(root, &scripts)?;
    crate::blueprint_compile::declaration_registry(&native, &files).map_err(|errors| {
        errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    })
}

pub fn compile_blueprints(
    root: &Path,
    native: &[Script],
) -> Result<Option<crate::blueprint_compile::Compilation>, String> {
    let assets = crate::blueprint_asset::load_all(root).inspect_err(|error| {
        let _ = crate::blueprint_dependencies::invalidate_all(root, error);
    })?;
    if assets.is_empty() {
        crate::blueprint_dependencies::observe_sources(root, &assets, None)?;
        return Ok(None);
    }
    let registry = crate::blueprint::native_registry(root, native);
    crate::blueprint_dependencies::observe_sources(root, &assets, registry.as_ref().ok())?;
    let registry = registry.inspect_err(|error| {
        let _ = crate::blueprint_dependencies::invalidate(root, &assets, error);
    })?;
    let compiled =
        crate::blueprint_compile::compile(root, &registry, &assets).map_err(|errors| {
            let message = errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n");
            let affected = assets
                .iter()
                .filter(|file| errors.iter().any(|error| error.asset == file.path))
                .cloned()
                .collect::<Vec<_>>();
            let _ = crate::blueprint_dependencies::invalidate(root, &affected, &message);
            message
        })?;
    crate::blueprint_dependencies::record(root, &compiled)?;
    Ok(Some(compiled))
}

pub fn metadata_files(files: &[PathBuf]) -> impl Iterator<Item = &PathBuf> {
    files
        .iter()
        .filter(|p| p.to_string_lossy().ends_with(".epokscript"))
}

pub fn native_catalog(root: &Path) -> Result<Vec<Script>, String> {
    let files = crate::reflection::script_files(root)?;
    crate::native_metadata::catalog_files(root, &files)?;
    let mut result = Vec::new();
    let mut names = BTreeSet::new();
    for path in metadata_files(&files) {
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        let mut script: Script =
            crate::document::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        crate::native_metadata::file(root, path, crate::assets::hash(&bytes))?;
        if !identifier(&script.name)
            || path.file_name().unwrap().to_string_lossy() != format!("{}.epokscript", script.name)
            || !names.insert(script.name.to_ascii_lowercase())
        {
            return Err(format!(
                "Invalid or duplicate script name: {}",
                path.display()
            ));
        }
        let dir = path.parent().unwrap();
        for ext in ["hpp", "cpp"] {
            if !dir.join(format!("{}.{ext}", script.name)).is_file() {
                return Err(format!("Missing {}.{ext}", script.name));
            }
        }
        script.header = dir
            .join(format!("{}.hpp", script.name))
            .strip_prefix(root.join("assets/scripts"))
            .map_err(|e| e.to_string())?
            .into();
        validate_properties(&script)?;
        result.push(script);
    }
    // Text detection only schedules semantic extraction. Clang determines all declarations.
    let annotated = files
        .iter()
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e == "hpp" || e == "h" || e == "hh")
        })
        .any(|path| fs::read_to_string(path).is_ok_and(|text| text.contains("EPOK_CLASS(")));
    if annotated {
        let manifest = crate::reflection::discover(root)?;
        let classes = manifest
            .classes
            .iter()
            .map(|c| (c.id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let canonical_root = fs::canonicalize(root).map_err(|e| e.to_string())?;
        let runtime = canonical_root.join(".epok/reflection/runtime");
        for class in manifest.classes.iter().filter(|c| {
            c.cpp_name != "epok::Behaviour" && c.id != crate::particle_effect::LAYER_CLASS_ID
        }) {
            let header = fs::canonicalize(&class.source.file).map_err(|e| e.to_string())?;
            // SDK declarations belong to the native registry, not the list of
            // project-authored scripts. Keep them available in ancestry chains.
            if header.starts_with(&runtime) {
                continue;
            }
            if !class_identifier(&class.cpp_name) {
                return Err(format!("Unsupported class name {}", class.cpp_name));
            }
            let mut chain = vec![class.clone()];
            let mut seen = BTreeSet::from([class.id.clone()]);
            let mut parent = class.parent.as_ref();
            while let Some(id) = parent {
                if !seen.insert(id.clone()) {
                    return Err("Class inheritance cycle".into());
                }
                let base = classes
                    .get(id)
                    .ok_or_else(|| format!("{}: missing reflected parent {id}", class.cpp_name))?;
                chain.push(base.clone());
                parent = base.parent.as_ref();
            }
            if chain
                .last()
                .is_none_or(|base| base.cpp_name != "epok::Behaviour" && base.id != crate::object_model::OBJECT_ID)
            {
                return Err(format!(
                    "{} is not derived from epok::Behaviour or epok::Object",
                    class.cpp_name
                ));
            }
            let mut properties = Vec::new();
            for base in chain.iter().rev() {
                for p in &base.properties {
                    properties.push(Property {
                        name: p.name.clone(),
                        default: p.default.clone(),
                        value_type: p.value_type.clone(),
                        id: p.id.clone(),
                    });
                }
            }
            let header = header
                .strip_prefix(canonical_root.join("assets/scripts"))
                .map_err(|_| {
                    format!(
                        "Reflected class {} must be authored inside assets/scripts",
                        class.cpp_name
                    )
                })?
                .to_path_buf();
            let script = Script {
                name: class.cpp_name.clone(),
                parent: chain.get(1).map(|p| p.cpp_name.clone()),
                properties,
                header,
                classes: chain,
            };
            validate_properties(&script)?;
            // The old wizard briefly produced annotations and duplicated JSON. Keep
            // that descriptor readable, but semantic types/signatures are authoritative.
            if let Some(existing) = result.iter_mut().find(|s| s.name == script.name) {
                *existing = script;
            } else {
                if !names.insert(script.name.to_ascii_lowercase()) {
                    return Err(format!("Case-insensitive class collision: {}", script.name));
                }
                result.push(script);
            }
        }
    }
    // Only legacy assets use their manual parent metadata.
    let direct = result.clone();
    for script in result.iter_mut().filter(|s| s.classes.is_empty()) {
        let mut parent = script.parent.clone();
        let mut seen = BTreeSet::from([script.name.clone()]);
        let mut inherited = Vec::new();
        while let Some(name) = parent {
            if name == "Behaviour" {
                break;
            }
            if !seen.insert(name.clone()) {
                return Err("Legacy inheritance cycle".into());
            }
            let base = direct
                .iter()
                .find(|s| s.name == name)
                .ok_or_else(|| format!("Missing parent {name}"))?;
            inherited.extend(base.properties.clone());
            parent = base.parent.clone();
        }
        inherited.extend(script.properties.clone());
        script.properties = inherited;
        validate_properties(script)?;
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}
fn validate_properties(script: &Script) -> Result<(), String> {
    if script.properties.len() > 16 {
        return Err(format!("{} exceeds 16 exposed properties", script.name));
    }
    let mut names = BTreeSet::new();
    for p in &script.properties {
        if !identifier(&p.name)
            || !names.insert(&p.name)
            || !script_values::valid(&p.default, &p.value_type)
        {
            return Err(format!(
                "{}: invalid, duplicated, or shadowed property {}",
                script.name, p.name
            ));
        }
    }
    Ok(())
}

/// Source creation publishes with create_new and rolls back only files this call owns.
pub fn create_named(root: &Path, name: &str, parent: &str) -> Result<(), String> {
    create_in(root, name, "", parent, false)
}

/// Folder components are portable source-directory identifiers relative to assets/scripts.
/// Concrete validation happens before publication is committed when attachment is requested.
/// Reflected runtime bases a new C++ class may derive from directly. These are declared
/// in `runtime/object_model.hpp`, so they never appear in the `assets/scripts` catalog;
/// the extractor is what validates the result, this list only decides the header text.
fn runtime_base(parent: &str) -> Option<String> {
    const BASES: &[&str] = &[
        "Actor",
        "Actor3D",
        "Actor2D",
        "UIActor",
        "SceneScriptActor",
        "ActorComponent",
        "SceneComponent3D",
        "SceneComponent2D",
        "UIComponent",
        "RectTransformComponent",
        "AudioComponent",
    ];
    let short = parent.strip_prefix("epok::").unwrap_or(parent);
    BASES.contains(&short).then(|| format!("epok::{short}"))
}

pub fn create_in(
    root: &Path,
    name: &str,
    folder: &str,
    parent: &str,
    concrete: bool,
) -> Result<(), String> {
    if !identifier(name) {
        return Err("Script class names must be valid, non-reserved C++ identifiers.".into());
    }
    crate::workspace::validate_name(name)?;
    let catalog = catalog(root)?;
    if catalog.iter().any(|s| s.name.eq_ignore_ascii_case(name)) {
        return Err(format!("A class named {name} already exists"));
    }
    let (include, base, implementation) = if parent == "Behaviour" || parent == "epok::Behaviour" {
        (
            "epok.hpp".into(),
            "epok::Behaviour".into(),
            "\npublic:\n    void update(epok::Transform&, epok::Fixed) override {}\n".into(),
        )
    } else if let Some(base) = runtime_base(parent) {
        // Actor and Component bases live in the runtime headers, not in assets/scripts,
        // and their lifecycle events (begin_play/tick/end_play/on_enable/on_disable) all
        // have defaults, so the generated class needs no body to be concrete.
        ("epok.hpp".into(), base, String::new())
    } else {
        let script = catalog
            .iter()
            .find(|s| s.name == parent)
            .ok_or("Unknown parent")?;
        let metadata = script
            .classes
            .first()
            .ok_or("Legacy parent must be annotated and reflected before it can be selected")?;
        if !crate::script_backend::can_derive(&schema::native_provider(), metadata) {
            return Err(format!("{parent} is not an eligible parent"));
        }
        (script.header_path(), parent.to_string(), String::new())
    };
    let relative = folder.replace('\\', "/");
    if !relative.is_empty() && !relative.split('/').all(identifier) {
        return Err(
            "Use relative folder components such as Enemies/Bosses (letters, digits, underscores)."
                .into(),
        );
    }
    let scripts_root = fs::canonicalize(root.join("assets/scripts")).map_err(|e| e.to_string())?;
    let canonical_root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    if !scripts_root.starts_with(&canonical_root) {
        return Err("Script folders must remain inside the project.".into());
    }
    let mut dir = scripts_root.clone();
    let mut owned_dirs = Vec::new();
    // Record only directories exclusively created here; rollback never removes user contents.
    for part in relative.split('/').filter(|p| !p.is_empty()) {
        dir.push(part);
        match fs::create_dir(&dir) {
            Ok(()) => owned_dirs.push(dir.clone()),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && dir.is_dir() => {}
            Err(e) => {
                for path in owned_dirs.iter().rev() {
                    let _ = fs::remove_dir(path);
                }
                return Err(e.to_string());
            }
        }
        if !fs::canonicalize(&dir)
            .map_err(|e| e.to_string())?
            .starts_with(&scripts_root)
        {
            for path in owned_dirs.iter().rev() {
                let _ = fs::remove_dir(path);
            }
            return Err("Script folder links must remain inside assets/scripts.".into());
        }
    }
    let id = uuid::Uuid::new_v4();
    let header = format!(
        "#pragma once\n#include \"{include}\"\n\nclass EPOK_CLASS(Blueprintable, Id=\"{id}\") {name} : public {base} {{{implementation}}};\n"
    );
    let source = format!("#include \"{name}.hpp\"\n");
    let mut owned = Vec::new();
    let result = (|| {
        for (ext, content) in [("hpp", header), ("cpp", source)] {
            let path = dir.join(format!("{name}.{ext}"));
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            owned.push(path);
            file.write_all(content.as_bytes())
                .map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
        }
        // Validate the entire transaction semantically before it can be attached.
        catalog_after_creation(root, name, concrete)?;
        Ok(())
    })();
    if result.is_err() {
        for path in owned {
            let _ = fs::remove_file(path);
        }
        for path in owned_dirs.iter().rev() {
            let _ = fs::remove_dir(path);
        }
    }
    result
}
fn catalog_after_creation(root: &Path, name: &str, concrete: bool) -> Result<(), String> {
    if let Some(script) = catalog(root)?.iter().find(|s| s.name == name) {
        if concrete && !script.instantiable() {
            Err(format!(
                "{name} inherits pure virtual events. Use Create, implement those events, then attach the concrete class."
            ))
        } else {
            Ok(())
        }
    } else {
        Err(format!(
            "{name}: generated annotation did not produce a reflected class"
        ))
    }
}
pub fn source(root: &Path, name: &str) -> PathBuf {
    catalog(root)
        .ok()
        .and_then(|scripts| scripts.into_iter().find(|s| s.name == name))
        .map(|s| {
            root.join("assets/scripts")
                .join(s.header_path())
                .with_extension("cpp")
        })
        .unwrap_or_else(|| root.join("assets/scripts").join(format!("{name}.cpp")))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Requires pinned libclang/MIPS SDK and cargo build --bins"]
    fn reflected_catalog_keeps_legacy_and_actor_scripts_without_sdk_entries() {
        let root = crate::workspace::tests::temp("mixed-script-catalog");
        crate::workspace::create(&root, "Mixed scripts", crate::workspace::Template::Basic).unwrap();
        create_in(&root, "LegacyController", "", "Behaviour", false).unwrap();
        create_in(&root, "HeroActor", "", "Actor3D", false).unwrap();
        create_in(&root, "HeroAudio", "", "AudioComponent", false).unwrap();
        let scripts = native_catalog(&root).unwrap();
        for name in ["LegacyController", "HeroActor", "HeroAudio"] {
            assert!(scripts.iter().any(|s| s.name == name), "missing {name}");
        }
        assert!(scripts.iter().all(|s| !s.name.starts_with("epok::")));
        let actor = scripts.iter().find(|s| s.name == "HeroActor").unwrap();
        assert_eq!(actor.classes.last().unwrap().id, crate::object_model::OBJECT_ID);
        let registry = crate::blueprint::native_registry(&root, &scripts).unwrap();
        assert!(registry.named("epok::Object").is_some());
        assert!(registry.named("epok::Behaviour").is_some());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejects_cpp_and_path_injection() {
        for bad in [
            "../Foo",
            "A\"",
            "A;",
            "class",
            "9foo",
            "_reserved",
            "co_await",
            "int",
            "A__B",
            "Enemy::Boss",
        ] {
            assert!(!identifier(bad), "{bad}");
        }
        assert!(identifier("PlayerController"));
        assert!(class_identifier("game::Enemy"));
    }
    #[test]
    fn legacy_values_stay_typed_fixed() {
        let property: Property =
            serde_json::from_value(serde_json::json!({"name":"speed","default":90.0})).unwrap();
        assert_eq!(property.value_type, schema::Type::Fixed);
        assert!(script_values::valid(
            &property.default,
            &property.value_type
        ));
    }
}
