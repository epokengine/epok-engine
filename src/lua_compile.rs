//! Orchestration for the Lua authoring provider: declaration extraction,
//! registry publication, body lowering and backend dispatch.
//!
//! The execution mode selects method bodies and packaging only. Declarations,
//! identities, layouts and the generated physical type are mode-independent, so
//! switching modes recompiles code without migrating a single serialized value.
use crate::{
    blueprint::Registry,
    lua_aot::{self, BodyMode},
    lua_asset::{self, Diagnostic, LuaFile},
    reflection_schema as schema,
    script_backend::Artifacts,
    script_ir::{self, ClassIr},
    scripts::{Property, Script},
    settings::LuaExecution,
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub struct Compilation {
    pub scripts: Vec<Script>,
    pub registry: Registry,
    pub artifacts: Artifacts,
    /// Published for the VM backend and the editor; the staging path consumes
    /// `scripts` and `artifacts` only.
    #[allow(dead_code)]
    pub classes: Vec<(schema::Class, ClassIr)>,
    pub footprints: crate::lua_dependencies::Footprints,
}

#[allow(dead_code)] // Provider identity is asserted by tests and consumed by the editor.
pub fn provider() -> schema::Extension {
    lua_asset::provider()
}

const BLUEPRINT_PARENT: &str =
    "Lua classes derive from C++ or Lua classes; Blueprint parents are not supported yet";

fn fail(file: &Path, span: script_ir::Span, message: impl Into<String>) -> Vec<Diagnostic> {
    vec![Diagnostic::new(file, span, message)]
}

/// Every persistent identity already claimed by the project. A Lua class id may
/// not collide with a reflected id, a member id or a Blueprint asset id.
fn claimed(root: &Path, native: &Registry) -> Result<BTreeSet<String>, String> {
    let mut identities = native
        .classes
        .values()
        .flat_map(|c| {
            std::iter::once(c.id.clone())
                .chain(c.properties.iter().map(|p| p.id.clone()))
                .chain(c.functions.iter().map(|f| f.id.clone()))
        })
        .collect::<BTreeSet<_>>();
    for file in crate::blueprint_asset::load_all(root)? {
        identities.insert(file.asset.id.clone());
        identities.extend(file.asset.variables.iter().map(|v| v.id.clone()));
    }
    Ok(identities)
}

/// Blueprint classes are compiled after Lua so a Blueprint may derive from a
/// Lua class. That ordering is also why the reverse is not available yet: the
/// Blueprint type does not exist when a Lua parent must be resolved.
fn blueprint_names(root: &Path) -> BTreeSet<String> {
    crate::blueprint_asset::load_all(root)
        .map(|files| files.into_iter().map(|f| f.asset.name).collect())
        .unwrap_or_default()
}

struct Declared<'a> {
    declaration: lua_asset::Declaration,
    file: &'a LuaFile,
}

/// Topological order by `extends`, with one diagnostic per unresolved root.
fn declaration_order<'a>(
    native: &Registry,
    files: &'a [LuaFile],
    blueprints: &BTreeSet<String>,
) -> Result<Vec<Declared<'a>>, Vec<Diagnostic>> {
    let mut declarations = vec![];
    let mut errors = vec![];
    for file in files {
        match lua_asset::extract(file) {
            Ok(declaration) => declarations.push(Declared { declaration, file }),
            Err(error) => errors.push(error),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut names = native
        .classes
        .values()
        .map(|c| c.cpp_name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut ids = BTreeSet::new();
    for entry in &declarations {
        let d = &entry.declaration;
        if !names.insert(d.name.to_ascii_lowercase()) {
            errors.push(Diagnostic::new(
                &d.file,
                d.span,
                format!("A class named {} already exists", d.name),
            ));
        }
        if !ids.insert(d.id.clone()) {
            errors.push(Diagnostic::new(&d.file, d.span, "Duplicate class UUID"));
        }
        if blueprints.contains(&d.extends)
            || native.named(&d.extends).is_some_and(|parent| {
                parent.provider == crate::script_backend::blueprint_provider()
            })
        {
            errors.push(Diagnostic::new(&d.file, d.span, BLUEPRINT_PARENT));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let declared = declarations
        .iter()
        .map(|e| e.declaration.name.clone())
        .collect::<BTreeSet<_>>();
    let mut known = native
        .classes
        .values()
        .map(|c| c.cpp_name.clone())
        .collect::<BTreeSet<_>>();
    let mut remaining = declarations.into_iter().collect::<Vec<_>>();
    let mut ordered = vec![];
    while !remaining.is_empty() {
        let (ready, waiting): (Vec<_>, Vec<_>) = remaining
            .into_iter()
            .partition(|e| known.contains(&e.declaration.extends));
        if ready.is_empty() {
            return Err(waiting
                .iter()
                .map(|e| {
                    Diagnostic::new(
                        &e.declaration.file,
                        e.declaration.span,
                        if declared.contains(&e.declaration.extends) {
                            format!("Inheritance cycle through {}", e.declaration.extends)
                        } else {
                            format!("Unknown parent {}", e.declaration.extends)
                        },
                    )
                })
                .collect());
        }
        for entry in ready {
            known.insert(entry.declaration.name.clone());
            ordered.push(entry);
        }
        remaining = waiting;
    }
    Ok(ordered)
}

fn register(
    native: &Registry,
    order: &[Declared<'_>],
    claimed: &BTreeSet<String>,
) -> Result<Registry, Vec<Diagnostic>> {
    let mut registry = native.clone();
    let mut seen = claimed.clone();
    for entry in order {
        let d = &entry.declaration;
        for id in std::iter::once(&d.id)
            .chain(d.properties.iter().map(|p| &p.id))
            .chain(d.functions.iter().map(|f| &f.id))
        {
            if !uuid::Uuid::parse_str(id).is_ok_and(|u| !u.is_nil() && u.to_string() == *id)
                || !seen.insert(id.clone())
            {
                return Err(fail(
                    &d.file,
                    d.span,
                    format!("Duplicate, nil, or noncanonical persistent UUID {id}"),
                ));
            }
        }
        let class = lua_asset::declarations(d, entry.file, &registry).map_err(|e| vec![e])?;
        registry.classes.insert(class.id.clone(), class);
    }
    registry.normalize_functions();
    Ok(registry)
}

/// Current Lua declarations resolved through the compiler's identity and
/// inheritance checks, without lowering a single body. Observers use this to
/// select dependencies; it never certifies executable code.
pub fn declaration_registry(
    native: &Registry,
    files: &[LuaFile],
) -> Result<Registry, Vec<Diagnostic>> {
    let blueprints = BTreeSet::new();
    let order = declaration_order(native, files, &blueprints)?;
    register(native, &order, &BTreeSet::new())
}

pub fn compile(
    root: &Path,
    native: &Registry,
    files: &[LuaFile],
    mode: LuaExecution,
) -> Result<Compilation, Vec<Diagnostic>> {
    let anywhere = files.first().map(|f| f.path.clone()).unwrap_or_default();
    let top = script_ir::Span { line: 1, column: 1 };
    let order = declaration_order(native, files, &blueprint_names(root))?;
    let claimed = claimed(root, native).map_err(|e| fail(&anywhere, top, e))?;
    let registry = register(native, &order, &claimed)?;
    registry.model().map_err(|d| {
        fail(
            &anywhere,
            top,
            d.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    })?;

    let mut classes = vec![];
    let mut errors = vec![];
    for entry in &order {
        let d = &entry.declaration;
        let class = registry.classes[&d.id].clone();
        let chunk = match crate::lua_frontend::parse(entry.file) {
            Ok(chunk) => chunk,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        match crate::lua_frontend::lower_class(d, &chunk, &class, &registry) {
            Ok(ir) => match script_ir::validate(&ir) {
                Ok(()) => classes.push((class, ir)),
                Err(message) => errors.push(Diagnostic::new(&d.file, d.span, message)),
            },
            Err(mut diagnostics) => errors.append(&mut diagnostics),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let mut artifacts = Artifacts::default();
    artifacts.runtime_capabilities.insert("lua".into());
    for file in files {
        artifacts.dependencies.insert(file.path.clone());
    }
    // A VM mode never silently degrades to AOT: a packaging failure is the
    // compilation's failure, reported with its real cause.
    let package = if mode.is_vm() {
        artifacts.runtime_capabilities.insert("lua-vm".into());
        Some(crate::lua_vm::package(root, mode, &classes, &registry)?)
    } else {
        None
    };

    let mut scripts = vec![];
    for (index, (class, ir)) in classes.iter().enumerate() {
        let body = match package {
            None => BodyMode::Native,
            Some(_) => BodyMode::Vm {
                class_index: index as u32,
            },
        };
        let symbol = package
            .as_ref()
            .and_then(|p| p.chunk_symbols.get(&class.id));
        let text = lua_aot::emit_class(class, ir, &registry, body, symbol.map(String::as_str))
            .map_err(|e| fail(&class.source.file, top, e))?;
        artifacts.files.insert(
            PathBuf::from(format!("scripts/generated/lua/{}.hpp", class.id)),
            text.into_bytes(),
        );
        let parent = registry
            .named(&ir.parent_cpp_name)
            .expect("Published parent")
            .clone();
        let mut chain = vec![class.clone()];
        chain.extend(registry.ancestry(&parent.cpp_name).into_iter().cloned());
        scripts.push(Script {
            name: class.cpp_name.clone(),
            parent: Some(parent.cpp_name.clone()),
            properties: registry
                .properties(&class.cpp_name)
                .into_iter()
                .map(|p| Property {
                    name: p.name.clone(),
                    default: p.default.clone(),
                    value_type: p.value_type.clone(),
                    id: p.id.clone(),
                })
                .collect(),
            header: PathBuf::from(format!("generated/lua/{}.hpp", class.id)),
            classes: chain,
        });
    }
    if let Some(package) = package {
        let bindings = lua_aot::emit_bindings(&classes, &registry, &package.chunk_symbols)
            .map_err(|e| fail(&anywhere, top, e))?;
        let path = PathBuf::from("scripts/generated/lua/lua_bindings.cpp");
        artifacts.files.insert(path.clone(), bindings.into_bytes());
        artifacts.native_sources.push(path);
        for (path, bytes) in package.files {
            if artifacts.files.insert(path.clone(), bytes).is_some() {
                return Err(fail(
                    &anywhere,
                    top,
                    format!(
                        "Lua VM artifact collides with generated code: {}",
                        path.display()
                    ),
                ));
            }
        }
        artifacts.native_sources.extend(package.native_sources);
        artifacts.dependencies.extend(package.dependencies);
    }
    scripts.sort_by(|a, b| a.name.cmp(&b.name));

    let sources = order
        .iter()
        .map(|e| (e.declaration.id.clone(), e.file))
        .collect::<Vec<_>>();
    let footprints = crate::lua_dependencies::capture(&sources, &registry, &artifacts, mode)
        .map_err(|e| fail(&anywhere, top, e))?;
    Ok(Compilation {
        scripts,
        registry,
        artifacts,
        classes,
        footprints,
    })
}

/// Diagnostics joined the way every caller reports them.
pub fn report(errors: &[Diagnostic]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_asset::{LuaFile, tests::HEADER};

    fn temp(label: &str) -> PathBuf {
        let root = crate::workspace::tests::temp(label);
        std::fs::create_dir_all(&root).unwrap();
        root
    }
    fn errors(result: Result<Compilation, Vec<Diagnostic>>) -> String {
        match result {
            Ok(_) => panic!("Compilation was expected to fail"),
            Err(diagnostics) => report(&diagnostics),
        }
    }
    fn enemy_logic(body: &str) -> LuaFile {
        LuaFile {
            path: "assets/scripts/EnemyLogic.lua".into(),
            source: format!("{}\n{body}\nreturn EnemyLogic\n", HEADER.trim_start()),
        }
    }

    #[test]
    fn lua_compile_publishes_classes_scripts_and_generated_headers() {
        let root = temp("lua-compile-native");
        let files = [enemy_logic(
            "function EnemyLogic:damage(amount)\n    self.health = self.health - amount\nend\n",
        )];
        let compiled = compile(
            &root,
            &crate::lua_asset::tests::registry(),
            &files,
            LuaExecution::NativeCpp,
        )
        .unwrap();
        let script = &compiled.scripts[0];
        assert_eq!(script.name, "EnemyLogic");
        assert_eq!(script.parent.as_deref(), Some("epok::ActorComponent"));
        assert_eq!(
            script.header_path(),
            "generated/lua/956f4946-0c61-42f8-899e-2db063b42420.hpp"
        );
        // Inherited storage is flattened into the catalog entry exactly as a
        // Blueprint's is, so the Inspector sees one authoritative layout.
        assert!(script.properties.iter().any(|p| p.name == "armour"));
        assert!(script.properties.iter().any(|p| p.name == "health"));
        assert_eq!(script.classes[0].provider, provider());
        assert!(compiled.artifacts.files.contains_key(Path::new(&format!(
            "scripts/generated/lua/{}.hpp",
            script.classes[0].id
        ))));
        assert!(compiled.artifacts.runtime_capabilities.contains("lua"));
        assert!(!compiled.artifacts.runtime_capabilities.contains("lua-vm"));
        assert!(
            compiled
                .artifacts
                .dependencies
                .contains(Path::new("assets/scripts/EnemyLogic.lua"))
        );
        assert!(
            compiled
                .footprints
                .contains_key("generated-lua:956f4946-0c61-42f8-899e-2db063b42420")
        );
        let (_, inputs) =
            &compiled.footprints["generated-lua:956f4946-0c61-42f8-899e-2db063b42420"];
        assert_eq!(inputs["lua-mode"], LuaExecution::NativeCpp.signature());
        assert_ne!(inputs["lua-mode"], LuaExecution::VmSource.signature());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lua_compile_reports_one_profile_diagnostic_in_every_execution_mode() {
        let root = temp("lua-compile-modes");
        let files = [enemy_logic(
            "function EnemyLogic:damage(amount)\n    while true do end\nend\n",
        )];
        let registry = crate::lua_asset::tests::registry();
        let mut reported = vec![];
        for mode in LuaExecution::ALL {
            reported.push(errors(compile(&root, &registry, &files, mode)));
        }
        assert!(
            reported[0].contains(crate::lua_frontend::profile::WHILE),
            "{reported:?}"
        );
        assert_eq!(reported[0], reported[1]);
        assert_eq!(reported[1], reported[2]);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lua_bodies_cannot_read_or_write_asset_and_class_references() {
        let root = temp("lua-reference-values");
        let mut registry = crate::lua_asset::tests::registry();
        registry
            .classes
            .get_mut(crate::object_model::ACTOR_COMPONENT_ID)
            .unwrap()
            .properties
            .push(crate::reflection_schema::Property {
                id: "6a7b8c9d-0e1f-4a2b-8c3d-4e5f6a7b8c9d".into(),
                name: "mesh".into(),
                value_type: crate::reflection_schema::Type::AssetRef {
                    kind: "mesh".into(),
                },
                default: serde_json::Value::Null,
                editable: true,
                timeline: None,
                source: crate::reflection_schema::Location {
                    file: "object_model.hpp".into(),
                    line: 1,
                    column: 1,
                },
            });
        let files = [enemy_logic(
            "function EnemyLogic:damage(amount)\n    self.mesh = self.mesh\nend\n",
        )];
        let message = errors(compile(&root, &registry, &files, LuaExecution::NativeCpp));
        assert!(
            message.contains(crate::lua_frontend::profile::REFERENCE_VALUE),
            "{message}"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A VM build shares the shell but never keeps a lowered native body, and a
    /// packaging failure is the compilation's failure: nothing degrades to AOT.
    #[test]
    fn lua_compile_vm_modes_emit_trampolines_and_bindings_without_native_fallback() {
        let root = temp("lua-compile-vm");
        let files = [enemy_logic(
            "function EnemyLogic:damage(amount)\n    self.health = self.health - amount\nend\n",
        )];
        let registry = crate::lua_asset::tests::registry();
        let bindings = Path::new("scripts/generated/lua/lua_bindings.cpp");
        let header = Path::new("scripts/generated/lua/956f4946-0c61-42f8-899e-2db063b42420.hpp");
        let native = compile(&root, &registry, &files, LuaExecution::NativeCpp).unwrap();
        assert!(!native.artifacts.files.contains_key(bindings));
        for mode in [LuaExecution::VmBytecode, LuaExecution::VmSource] {
            let compiled = match compile(&root, &registry, &files, mode) {
                Ok(compiled) => compiled,
                // A host without the pinned cooker reports its real cause and
                // still refuses to produce native bodies.
                Err(diagnostics) => {
                    let message = report(&diagnostics);
                    assert!(message.contains("Lua VM"), "{message}");
                    continue;
                }
            };
            assert!(compiled.artifacts.runtime_capabilities.contains("lua-vm"));
            assert!(
                compiled
                    .artifacts
                    .native_sources
                    .contains(&bindings.to_path_buf())
            );
            let text = String::from_utf8(compiled.artifacts.files[header].clone()).unwrap();
            assert!(
                text.contains("epok::lua::Frame f(*this, kEpokLuaClass_0"),
                "{text}"
            );
            assert!(!text.contains("epok::bp::sub"), "{text}");
            let (_, inputs) =
                &compiled.footprints["generated-lua:956f4946-0c61-42f8-899e-2db063b42420"];
            assert_eq!(inputs["lua-mode"], mode.signature());
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lua_classes_are_blueprint_parents_but_blueprint_classes_are_not_lua_parents() {
        let root = temp("lua-blueprint-parents");
        let files = [enemy_logic("")];
        let native = crate::lua_asset::tests::registry();
        let registry = declaration_registry(&native, &files).unwrap();
        assert!(
            registry
                .blueprint_parents()
                .any(|c| c.cpp_name == "EnemyLogic")
        );
        assert!(registry.lua_parents().any(|c| c.cpp_name == "EnemyLogic"));
        // The Blueprint compiler resolves the Lua class by id and derives from it.
        let mut asset = crate::blueprint_asset::BlueprintAsset::new(
            "Boss".into(),
            "956f4946-0c61-42f8-899e-2db063b42420".into(),
        );
        asset.id = "4e2a1c6f-8b3d-4e5a-9c7f-0d1e2a3b4c5d".into();
        let blueprint =
            crate::blueprint_asset::AssetFile::file("assets/blueprints/Boss.epokbp".into(), asset);
        let resolved =
            crate::blueprint_compile::declaration_registry(&registry, &[blueprint]).unwrap();
        assert_eq!(
            resolved.classes["4e2a1c6f-8b3d-4e5a-9c7f-0d1e2a3b4c5d"].parent,
            Some("956f4946-0c61-42f8-899e-2db063b42420".into())
        );

        // The reverse is not available yet: Lua compiles first, so a Blueprint
        // type does not exist when a Lua parent must be resolved.
        let lua = LuaFile {
            path: "assets/scripts/Minion.lua".into(),
            source: "local Minion = epok.class {\n    profile = 1,\n    id = \"7a1b2c3d-4e5f-4a6b-8c9d-0e1f2a3b4c5d\",\n    name = \"Minion\",\n    extends = \"Boss\",\n    properties = {},\n    functions = {}\n}\nreturn Minion\n".into(),
        };
        let message = errors(compile(&root, &resolved, &[lua], LuaExecution::NativeCpp));
        assert!(message.contains(BLUEPRINT_PARENT), "{message}");
        std::fs::remove_dir_all(root).unwrap();
    }

    const SENTINEL: &str = r#"local Sentinel = epok.class {
    profile = 1,
    id = "1c0d3f57-9a2b-4e6d-8f31-5b7c9d0e2a41",
    name = "Sentinel",
    extends = "epok::ActorComponent",
    properties = {
        health = { id = "2d1e4a68-0b3c-4f7e-9a42-6c8d0e1f3b52",
            type = "Fixed", default = 100, editable = true },
        ticks = { id = "3e2f5b79-1c4d-4a8f-8b53-7d9e1f2a4c63",
            type = "Int32", default = 0, editable = true },
        ready = { id = "4f306c8a-2d5e-4b90-9c64-8e0f2a3b5d74",
            type = "Bool", default = true, editable = true }
    },
    functions = {
        damage = { id = "50417d9b-3e6f-4ca1-8d75-9f102b4c6e85", callable = true,
            parameters = { { name = "amount", type = "Fixed" } }, returns = "void" },
        absorb = { id = "61528eac-4f70-4db2-9e86-0a213c5d7f96", callable = true,
            parameters = { { name = "amount", type = "Fixed" } }, returns = "Fixed" }
    }
}

function Sentinel:damage(amount)
    if self.ready then
        self.health = self.health - amount
    else
        self.health = self.health + amount
    end
end

function Sentinel:absorb(amount)
    self:damage(amount)
    for i = 1, 3 do
        self.ticks = self.ticks + i
    end
    return self.health
end

function Sentinel:tick(delta_seconds)
    self.health = self.health + delta_seconds
    self.rotation.y = self.rotation.y + delta_seconds
end

function Sentinel:begin_play()
    self.ticks = 0
    self.ready = true
end

return Sentinel
"#;

    fn lua_project(label: &str) -> (PathBuf, crate::scene::Scene) {
        let root = crate::workspace::tests::temp(label);
        crate::workspace::create(&root, "Lua Build", crate::workspace::Template::Basic).unwrap();
        std::fs::write(root.join("assets/scripts/Sentinel.lua"), SENTINEL).unwrap();
        let scene =
            crate::scene::Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
        (root, scene)
    }
    fn select(root: &Path, mode: LuaExecution) {
        let mut manifest = crate::workspace::read_manifest(root).unwrap();
        manifest.lua_execution = mode;
        crate::workspace::save_manifest(root, &manifest).unwrap();
    }

    /// A mode change replaces every generated body and leaves nothing of the
    /// previous mode behind; returning reproduces the original bytes exactly.
    #[test]
    #[ignore = "requires the pinned host extractor; stages a disposable project without building"]
    fn lua_mode_change_replaces_generated_bodies_in_both_directions() {
        let (root, scene) = lua_project("lua-mode-switch");
        let build = root.join(".epok/build");
        let header = build.join("scripts/generated/lua/1c0d3f57-9a2b-4e6d-8f31-5b7c9d0e2a41.hpp");
        let bindings = build.join("scripts/generated/lua/lua_bindings.cpp");

        select(&root, LuaExecution::NativeCpp);
        crate::project::stage_into(&root, &scene, &build).unwrap();
        let native = std::fs::read_to_string(&header).unwrap();
        assert!(
            native.contains("epok::bp::sub(this->health, epok_p0)"),
            "{native}"
        );
        assert!(!native.contains("epok::lua::Frame"));
        assert!(!bindings.exists());

        select(&root, LuaExecution::VmSource);
        match crate::project::stage_into(&root, &scene, &build) {
            Ok(_) => {
                let vm = std::fs::read_to_string(&header).unwrap();
                assert!(vm.contains("epok::lua::Frame"), "{vm}");
                // No lowered native arithmetic may survive into a VM build.
                assert!(!vm.contains("epok::bp::sub"), "{vm}");
                assert!(bindings.is_file());
            }
            Err(message) => {
                // A packaging failure is reported with its real cause and must
                // not leave the previous mode's generated code runnable.
                assert!(message.contains("Lua"), "{message}");
                assert!(!header.exists(), "stale native header survived: {message}");
            }
        }

        select(&root, LuaExecution::NativeCpp);
        crate::project::stage_into(&root, &scene, &build).unwrap();
        assert_eq!(std::fs::read_to_string(&header).unwrap(), native);
        assert!(
            !bindings.exists(),
            "a VM binding survived the return to native"
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// The real proof: the generated C++ compiles for MIPS, the executable
    /// links, and a native build contains no interpreter at all.
    #[test]
    #[ignore = "requires pinned MIPS tools and the host extractor; builds without launching the emulator"]
    fn lua_project_builds_for_mips_and_links_no_interpreter_in_native_mode() {
        let (root, scene) = lua_project("lua-mips-build");
        select(&root, LuaExecution::NativeCpp);
        // A reflected user C++ base, then a Lua subclass of it: this is the path
        // through the extractor that a runtime base alone would not exercise.
        crate::scripts::create_in(&root, "EnemyBase", "", "ActorComponent", false).unwrap();
        let created = crate::lua_asset::create_in(&root, "Guard", "Enemies", "EnemyBase").unwrap();
        assert!(created.ends_with("Enemies/Guard.lua"));
        // Qualified parent dispatch has to compile and link like any other call.
        let guard = std::fs::read_to_string(&created).unwrap().replace(
            "function Guard:begin_play()\nend",
            "function Guard:begin_play()\n    epok.super(Guard, self):begin_play()\nend",
        );
        assert!(guard.contains("epok.super"));
        std::fs::write(&created, &guard).unwrap();
        let catalog = crate::scripts::catalog(&root).unwrap();
        for name in ["EnemyBase", "Sentinel", "Guard"] {
            assert!(catalog.iter().any(|s| s.name == name), "missing {name}");
        }

        let path = crate::workspace::startup_scene(&root).unwrap();
        let profile = crate::play::Profile::default();
        let input = crate::play::input(&root, path, scene, profile, false).unwrap();
        let job = crate::pipeline::Job::start_with_debug(root.clone(), input, false, false);
        let started = std::time::Instant::now();
        loop {
            assert!(started.elapsed().as_secs() < 600, "Build timed out");
            match job.events.recv_timeout(std::time::Duration::from_secs(2)) {
                Ok(crate::pipeline::Event::Log(line)) => eprintln!("[build] {line}"),
                Ok(crate::pipeline::Event::Finished(result)) => {
                    result.unwrap();
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("Build worker disconnected")
                }
                _ => {}
            }
        }
        let build = root.join(".epok/build");
        assert!(
            std::fs::read(build.join("epok.ps-exe"))
                .unwrap()
                .starts_with(b"PS-X EXE")
        );

        let config = crate::project::Config::load(&root).unwrap();
        let tool = |name: &str| {
            if config.toolchain_bin.is_empty() {
                PathBuf::from(name)
            } else {
                crate::project::Config::path(&root, &config.toolchain_bin).join(name)
            }
        };
        let symbols = std::process::Command::new(tool("mipsel-none-elf-nm"))
            .arg(build.join("epok.elf"))
            .output()
            .unwrap();
        assert!(symbols.status.success());
        let symbols = String::from_utf8_lossy(&symbols.stdout);
        let interpreter = symbols
            .lines()
            .filter(|line| {
                line.split_whitespace().last().is_some_and(|name| {
                    name.starts_with("lua_")
                        || name.starts_with("luaL_")
                        || name.starts_with("luaU_")
                })
            })
            .collect::<Vec<_>>();
        assert!(
            interpreter.is_empty(),
            "Native mode linked an interpreter: {interpreter:?}"
        );
        assert!(
            symbols.contains("Sentinel") && symbols.contains("Guard"),
            "generated classes are absent"
        );
        let size = std::process::Command::new(tool("mipsel-none-elf-size"))
            .arg(build.join("epok.elf"))
            .output()
            .unwrap();
        eprintln!("{}", String::from_utf8_lossy(&size.stdout));
        std::fs::remove_dir_all(&root).unwrap();
    }
}
