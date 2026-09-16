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

fn cpp_without_comments(source: &str) -> String {
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

fn native_requirements(source: &str) -> Result<BTreeSet<String>, String> {
    let mut requirements = BTreeSet::new();
    let without_comments = cpp_without_comments(source);
    let code = cpp_without_literals(&without_comments);
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
    ] {
        if identifier_calls(&code, name) {
            requirements.insert(capability.to_string());
        }
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
    artifacts.native_set = Some(crate::staging_files::native_set(root, &files)?);
    let mut requirement_sources = BTreeMap::<String, Vec<String>>::new();
    for path in files {
        let relative = Path::new("scripts").join(
            path.strip_prefix(root.join("assets/scripts"))
                .map_err(|e| e.to_string())?,
        );
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&bytes);
        for requirement in native_requirements(&text)? {
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
    capabilities(author).is_ok_and(|c| c.create && c.derive_backends.contains(&parent.backend.id))
        // A Lua class is a real native subclass in every execution mode, so it
        // is an eligible parent for every author. Lua itself derives from C++
        // or Lua only; Blueprint parents wait for a joint declaration order.
        && (parent.provider == schema::native_provider()
            || parent.provider == lua_provider()
            || (*author == blueprint_provider() && parent.provider == blueprint_provider()))
        && parent.backend == schema::native_backend()
        && parent.blueprintable
        && !parent.final_class
}
pub fn validate_binding(binding: &ClassDefaults) -> Result<(), String> {
    if !capabilities(&binding.provider)?.attach {
        return Err("This provider cannot attach instances.".into());
    }
    backend(&binding.backend)?;
    Ok(())
}

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
