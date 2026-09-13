//! Prelinked scene banks share immutable resource tables and one reusable runtime
//! object pool. No filesystem/CD reads are needed for a transition.
use crate::{scene::Scene, scripts::Script};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path},
};
pub const REGISTRY: &str = "ProjectSettings/Maps.epoksettings";

/// Fixed bounds of the Object/Actor/Component runtime, mirroring the constants in
/// `runtime/object_model.hpp`. The cook refuses a map that would exceed one instead of
/// emitting a capacity the runtime arrays cannot honour.
const LEVEL_ACTOR_CAPACITY: usize = 64;
const ACTOR_COMPONENT_CAPACITY: usize = 8;
/// Largest `EPOK_OBJECT_REGISTRY_CAPACITY` the bounds above can ever produce: every actor
/// of a full level carrying a full component set, plus the 32 dynamic slots.
const OBJECT_REGISTRY_MAX: usize = LEVEL_ACTOR_CAPACITY * (1 + ACTOR_COMPONENT_CAPACITY) + 32;
/// Saved maps available to the Play menu. Do not follow directory links outside the project.
pub fn available(root: &Path) -> Result<Vec<String>, String> {
    fn visit(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                visit(root, &entry.path(), out)?;
            } else if kind.is_file() && entry.path().extension().is_some_and(|ext| ext == "epokmap")
            {
                out.push(
                    entry
                        .path()
                        .strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    if root.join("assets/scenes").is_dir() {
        visit(root, &root.join("assets/scenes"), &mut out)?;
    }
    out.sort();
    Ok(out)
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Registry {
    pub scenes: Vec<String>,
}
pub fn read(root: &Path) -> Result<Registry, String> {
    let path = root.join(REGISTRY);
    if !path.exists() {
        return Ok(Registry::default());
    }
    let registry: Registry =
        crate::document::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Scene registry: {e}"))?;
    registry.validate(root)?;
    Ok(registry)
}
impl Registry {
    pub fn validate(&self, root: &Path) -> Result<(), String> {
        let registry = self;
        if registry.scenes.len() > 15 {
            return Err(
                "Register at most 15 additional scenes (16 banks including startup)".into(),
            );
        }
        let mut unique = std::collections::BTreeSet::new();
        for entry in &registry.scenes {
            let p = Path::new(entry);
            if p.components().any(|c| !matches!(c, Component::Normal(_)))
                || !p.starts_with("assets/scenes")
                || !entry.ends_with(".epokmap")
                || !unique.insert(entry.to_lowercase())
            {
                return Err(
                    "Scene registry requires unique relative assets/scenes/*.epokmap paths".into(),
                );
            }
            let resolved =
                fs::canonicalize(root.join(p)).map_err(|e| format!("Scene {entry}: {e}"))?;
            let assets = fs::canonicalize(root.join("assets/scenes")).map_err(|e| e.to_string())?;
            if !resolved.starts_with(assets) {
                return Err("Registered scenes must remain inside assets/scenes".into());
            }
        }
        Ok(())
    }
    pub fn save(&self, root: &Path) -> Result<(), String> {
        self.validate(root)?;
        crate::project::write_changed(
            &root.join(REGISTRY),
            &crate::document::to_vec(self).map_err(|e| e.to_string())?,
        )
    }
}
pub struct Loaded {
    pub scenes: Vec<Scene>,
    pub sources: Vec<crate::scene_dependencies::Origin>,
}
#[cfg(test)]
pub fn load(
    root: &Path,
    startup: &Scene,
    origin: &crate::scene_dependencies::Origin,
) -> Result<Loaded, String> {
    load_selected(root, startup, origin, None)
}
pub fn load_selected(
    root: &Path,
    startup: &Scene,
    origin: &crate::scene_dependencies::Origin,
    input: Option<&crate::scene_dependencies::Input>,
) -> Result<Loaded, String> {
    let mut scenes = vec![startup.clone()];
    let profile = input.and_then(|i| i.play.as_ref());
    let registry = match profile.map(|p| p.content) {
        Some(crate::play::Content::CurrentScene) => Registry::default(),
        Some(crate::play::Content::SelectedScenes) => {
            let profile = profile.unwrap();
            profile.validate_build(root)?;
            Registry {
                scenes: profile.selected_scenes.clone(),
            }
        }
        Some(crate::play::Content::WholeGame) => {
            let mut registry = read(root)?;
            for path in available(root)? {
                if !registry
                    .scenes
                    .iter()
                    .any(|entry| entry.eq_ignore_ascii_case(&path))
                {
                    registry.scenes.push(path);
                }
            }
            registry
        }
        None => read(root)?,
    };
    let active_path = match origin {
        crate::scene_dependencies::Origin::Saved(path, _)
        | crate::scene_dependencies::Origin::Editor(path, _) => fs::canonicalize(path).ok(),
        crate::scene_dependencies::Origin::Anonymous(_) => None,
    };
    let mut sources = vec![];
    for p in &registry.scenes {
        // Play starts from the open document, which may itself be registered.
        // Keep that snapshot (including unsaved edits), rather than loading its
        // disk version again. Names alone cannot establish document identity.
        if active_path.as_ref().is_some_and(|active| {
            fs::canonicalize(root.join(p)).is_ok_and(|registered| registered == *active)
        }) {
            continue;
        }
        let input = if let Some((path, scene)) = input
            .and_then(|i| i.editor_override.as_ref())
            .filter(|(path, _)| {
                fs::canonicalize(path).ok().is_some_and(|path| {
                    fs::canonicalize(root.join(p)).is_ok_and(|registered| registered == path)
                })
            }) {
            crate::scene_dependencies::Input::editor(path.clone(), scene.clone())
        } else {
            crate::scene_dependencies::Input::load(&root.join(p))?
        };
        let scene = input.scene;
        if scenes.iter().any(|s| s.name == scene.name) {
            return Err(format!(
                "Scene bank name {} is duplicated across different maps; give each map a unique name",
                scene.name
            ));
        }
        scenes.push(scene);
        sources.push(input.origin);
    }
    if scenes.len() > 16 {
        return Err("The runtime supports at most 16 scenes per build. Use Selected Scenes to choose a smaller set.".into());
    }
    Ok(Loaded { scenes, sources })
}
pub fn resources(scenes: &[Scene]) -> Scene {
    let mut out = scenes[0].clone();
    out.entities.clear();
    out.textures.clear();
    for s in scenes {
        out.entities.extend(s.entities.clone());
        out.textures.extend(s.textures.clone());
    }
    for e in &mut out.entities {
        if let Some(a) = &mut e.audio {
            a.play_on_start = false;
        }
    }
    out
}
fn number(header: &str, key: &str) -> Result<usize, String> {
    let part = header
        .split_once(key)
        .ok_or_else(|| format!("Missing generated {key}"))?
        .1;
    part.split(';')
        .next()
        .unwrap()
        .trim()
        .parse()
        .map_err(|_| format!("Invalid generated {key}"))
}
#[cfg(test)]
pub fn header(scenes: &[Scene], catalog: &[Script]) -> Result<String, String> {
    header_with_streaming(scenes, catalog, None)
}
#[cfg(test)]
pub fn header_with_streaming(
    scenes: &[Scene],
    catalog: &[Script],
    streaming: Option<&crate::streaming::Bundle>,
) -> Result<String, String> {
    let registry = crate::blueprint::legacy_registry(Path::new(""), catalog);
    header_with_templates(
        scenes,
        catalog,
        streaming,
        &[],
        None,
        &[],
        &[],
        &registry,
    )
}
pub fn header_with_templates(
    scenes: &[Scene],
    catalog: &[Script],
    streaming: Option<&crate::streaming::Bundle>,
    templates: &[crate::blueprint_spawn::CookedTemplate],
    referenced: Option<&Scene>,
    timelines: &[crate::timeline_scene::Prepared],
    effects: &[crate::particle_effect_scene::Prepared],
    class_registry: &crate::blueprint::Registry,
) -> Result<String, String> {
    if scenes.is_empty() || scenes.len() > 16 {
        return Err("Scene registry needs 1..16 scenes".into());
    }
    let mut all_resources = scenes.to_vec();
    all_resources.extend(templates.iter().map(|template| template.scene.clone()));
    all_resources.extend(referenced.cloned());
    let shared = resources(&all_resources);
    let global_layout =
        !templates.is_empty() || referenced.is_some_and(|scene| !scene.entities.is_empty());
    let ids = crate::texture::ids(&shared);
    let mut textures = String::new();
    for (i, id) in ids.iter().enumerate() {
        let t = shared
            .textures
            .get(id)
            .ok_or_else(|| format!("Unresolved shared texture {id}"))?;
        textures += &format!(
            "inline constexpr int {}={i};\n",
            crate::texture::symbol(*id)
        );
        for (name, values) in [("pixels", &t.words), ("palette", &t.palette)] {
            textures += &format!(
                "alignas(4) inline constexpr uint16_t texture_{i}_{name}[]={{{}}};\n",
                values
                    .iter()
                    .map(u16::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
    }
    textures += &format!(
        "inline constexpr size_t texture_count={};\ninline const Texture* texture_assets=nullptr;\n",
        ids.len()
    );
    textures += &format!(
        "inline constexpr size_t resident_texture_bytes={};\n",
        ids.iter()
            .map(|id| {
                let t = &shared.textures[id];
                (t.words.len() + t.palette.len()) * 2
            })
            .sum::<usize>()
    );
    for (bank, s) in scenes.iter().enumerate() {
        // Validate each working bank independently; unused bank textures do not
        // occupy VRAM. Shared source pixels exist once in executable memory.
        // Dynamic prototypes use one fixed resident texture layout in every
        // bank. Reject overflow explicitly instead of spawning invisible assets.
        let layout_scene = if global_layout { &shared } else { s };
        crate::texture::header(layout_scene)?;
        let layout = crate::texture::layout(layout_scene)?;
        let descriptors = ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                if let Some((_, p)) = layout.iter().find(|(v, _)| v == id) {
                    let t = &shared.textures[id];
                    format!(
                        "{{{},{},{},{},{},640,{},texture_{i}_pixels,texture_{i}_palette}}",
                        t.width,
                        t.height,
                        p.x,
                        p.y,
                        t.width.div_ceil(4) * 2,
                        p.clut_y
                    )
                } else {
                    "{}".into()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        textures += &format!(
            "inline constexpr Texture bank_textures_{bank}[]={{{}}};\n",
            if descriptors.is_empty() {
                "{}"
            } else {
                &descriptors
            }
        );
    }
    let capacity = scenes.iter().map(|s| s.entities.len() + 32).max().unwrap();
    let mut headers = Vec::new();
    let mut render_capacity = 0;
    for (bank, s) in scenes.iter().enumerate() {
        let mut h = crate::project::scene_header_with_registry(
            s,
            catalog,
            &shared,
            false,
            if global_layout { &shared } else { s },
            class_registry,
        )?;
        if let Some(streaming) = streaming {
            h = streaming.rewrite(bank, &h)?;
        }
        render_capacity =
            render_capacity.max(number(&h, "inline constexpr size_t render_capacity=")?);
        headers.push(h);
    }
    let dynamic_quads = templates
        .iter()
        .flat_map(|template| template.scene.entities.iter())
        .map(crate::lighting::quad_count)
        .max()
        .unwrap_or(6)
        .max(6);
    render_capacity = render_capacity.saturating_add(32 * (dynamic_quads - 6) * 2);
    let streaming_active = streaming.is_some_and(|bundle| bundle.page_count() > 0);
    let retained_quad_capacity = if streaming_active {
        render_capacity = render_capacity.min(streaming.unwrap().triangle_budget());
        // Retention also serves streamed meshes, but the configured frame
        // budget bounds its records independently of the total world size.
        scenes
            .iter()
            .map(|scene| {
                scene
                    .entities
                    .iter()
                    .map(crate::lighting::quad_count)
                    .sum::<usize>()
                    + 32 * dynamic_quads
            })
            .max()
            .unwrap()
            .min(render_capacity / 2)
            .max(1)
    } else {
        (render_capacity / 2).max(1)
    };
    // Slot table for the Object/Actor/Component runtime: the worst bank's actors and
    // components plus headroom for runtime spawns, the Level and the scene script.
    //
    // Every bound below is a compile-time constant of runtime/object_model.hpp. Exceeding
    // one is a cook diagnostic naming the map and the bound, never a silent increase: the
    // runtime arrays are fixed-size, so a larger table would be dropped at load time
    // (`ObjectStats::rejected`) rather than honoured.
    let object_registry_capacity = {
        let mut capacity = 32;
        for scene in scenes {
            if scene.actors.len() > LEVEL_ACTOR_CAPACITY {
                return Err(format!(
                    "{}: {} actors exceed the runtime level capacity of {LEVEL_ACTOR_CAPACITY} (epok::level_actor_capacity); split the map or raise the runtime constant",
                    scene.name,
                    scene.actors.len()
                ));
            }
            if let Some(actor) = scene
                .actors
                .iter()
                .find(|actor| actor.components.len() > ACTOR_COMPONENT_CAPACITY)
            {
                return Err(format!(
                    "{}: actor {} has {} components, above the runtime limit of {ACTOR_COMPONENT_CAPACITY} per actor (epok::actor_component_capacity)",
                    scene.name,
                    actor.name,
                    actor.components.len()
                ));
            }
            let slots = scene.actors.len()
                + scene
                    .actors
                    .iter()
                    .map(|actor| actor.components.len())
                    .sum::<usize>()
                + 32;
            capacity = capacity.max(slots);
        }
        if capacity > OBJECT_REGISTRY_MAX {
            return Err(format!(
                "the cooked object registry would need {capacity} slots, above the {OBJECT_REGISTRY_MAX} EPOK_OBJECT_REGISTRY_CAPACITY supports"
            ));
        }
        capacity
    };
    let mut out = format!(
        "// Generated prelinked scene banks.\n#pragma once\n#include \"epok.hpp\"\n#define EPOK_OBJECT_REGISTRY_CAPACITY {object_registry_capacity}\n#include \"actor_tables.hpp\"\n#include <array>\n"
    );
    for s in catalog {
        out += &format!("#include \"scripts/{}\"\n", s.header_path());
    }
    out += &crate::blueprint_spawn::header_with_templates(
        catalog,
        templates,
        !timelines.is_empty() || !effects.is_empty(),
        class_registry,
    )?;
    if !timelines.is_empty() {
        out += "#include \"timeline_service.hpp\"\n";
        for timeline in timelines {
            out += &format!("#include \"timelines/{}.hh\"\n", timeline.compiled.asset);
        }
    }
    if !effects.is_empty() {
        out += "#include \"particle_effect_service.hpp\"\n";
        for effect in effects {
            out += &format!("#include \"effects/{}.hh\"\n", effect.source.id);
        }
    }
    out += "namespace epok {\n";
    out += &format!("inline constexpr size_t retained_quad_capacity={retained_quad_capacity};\n");
    out += &streaming.map_or_else(
        || crate::streaming::Bundle::default().globals(),
        |bundle| bundle.globals(),
    );
    out += "}\n";
    out += &format!(
        "namespace epok {{\ntemplate<class T> struct TableView {{T* data=nullptr;size_t count=0;T* begin() const{{return data;}}T* end() const{{return data+count;}}size_t size() const{{return count;}}}};\ninline std::array<Entity,{capacity}> objects;\ninline size_t object_count=0,authored_count=0;\ninline constexpr size_t render_capacity={render_capacity};\ninline TableView<Binding> bindings;\ninline TableView<const size_t> transform_order;\n{textures}\n}}\n"
    );
    out += &crate::blueprint_spawn::prototypes(catalog, templates, &shared, timelines, effects)?;
    if !timelines.is_empty()
        || !effects.is_empty()
        || catalog.iter().any(|script| {
            script
                .classes
                .iter()
                .any(|class| class.provider.id == "blueprint")
        })
    {
        out += &crate::blueprint_refs::asset_table(&shared)?;
    }
    for (i, (s, h)) in scenes.iter().zip(headers).enumerate() {
        let mut h = h
            .lines()
            .filter(|l| !l.starts_with("#include") && !l.starts_with("#pragma"))
            .collect::<Vec<_>>()
            .join("\n");
        h = h.replace("namespace epok {", &format!("namespace epok::scene_{i} {{"));
        h = h.replace(
            &format!(
                "inline std::array<Entity, {}> objects",
                s.entities.len() + 32
            ),
            &format!(
                "inline const std::array<Entity, {}> initial_objects",
                s.entities.len()
            ),
        );
        // Scene-local capacities are metadata; all actual component writes target
        // the enclosing namespace's shared pool.
        h = h.replace("inline size_t object_count=authored_count;", "");
        out += &h;
        out.push('\n');
        out += &format!(
            "namespace epok {{inline void load_bank_{i}(){{\nobject_count=authored_count=scene_{i}::authored_count;\nfor(size_t j=0;j<objects.size();++j){{auto generation=objects[j].generation+1;if(!generation)generation=1;objects[j]=j<object_count?scene_{i}::initial_objects[j]:Entity{{}};objects[j].generation=generation;objects[j].alive=j<object_count;}}\nbindings={{scene_{i}::bindings.data(),scene_{i}::bindings.size()}};transform_order={{scene_{i}::transform_order.data(),scene_{i}::transform_order.size()}};\n"
        );
        for (entity, e) in s.entities.iter().enumerate() {
            if let Some(b) = &e.script {
                let class = crate::script_backend::resolve(b, catalog)?;
                out += &crate::script_backend::backend(&b.backend)?
                    .reset(&class.name, &format!("scene_{i}::behaviour_{entity}"));
            }
        }
        if !timelines.is_empty() {
            out += &crate::timeline_scene::setup(
                s,
                timelines,
                &crate::blueprint::legacy_registry(std::path::Path::new(""), catalog),
            )?;
        }
        if !effects.is_empty() {
            out += &crate::particle_effect_scene::setup(
                s,
                effects,
                &crate::blueprint::legacy_registry(std::path::Path::new(""), catalog),
                false,
            )?;
        }
        // Actors, their components and the bank's single SceneScriptActor come up
        // before the legacy Behaviour bindings, so a scene script observes both halves.
        out += &format!(
            "load_actor_bank(scene_{i}::actor_table,objects.data(),object_count);\nactivate_texture_bank(bank_textures_{i});\nscene_{i}::initialize_scripts();\n"
        );
        if !timelines.is_empty() {
            out += "epok::timeline::start_components();\n";
        }
        out += "}}\n";
    }
    out += "namespace epok {\n";
    for (i, _) in scenes.iter().enumerate() {
        out += &format!(
            "inline constexpr uint64_t scene_script_class_{i}=scene_{i}::scene_script_class;\n"
        );
    }
    out += "struct SceneBank {const char* name;void(*load)();};\ninline const SceneBank scene_banks[]={\n";
    for (i, s) in scenes.iter().enumerate() {
        out += &format!(
            "{{{},load_bank_{i}}},\n",
            serde_json::to_string(&s.name).unwrap()
        );
    }
    out += "};\ninline constexpr size_t scene_bank_count=sizeof(scene_banks)/sizeof(scene_banks[0]);\ninline void initialize_scripts(){load_bank_0();}\n}\n";
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_dependencies::{Input, Origin};

    fn registered_maps() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("epok-scene-bank-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("assets/scenes")).unwrap();
        fs::create_dir_all(root.join("ProjectSettings")).unwrap();
        for name in ["Title", "ForestClearing", "ForestBattle"] {
            Scene {
                name: name.into(),
                ..Default::default()
            }
            .save(&root.join(format!("assets/scenes/{name}.epokmap")))
            .unwrap();
        }
        Registry {
            scenes: vec![
                "assets/scenes/ForestClearing.epokmap".into(),
                "assets/scenes/ForestBattle.epokmap".into(),
            ],
        }
        .save(&root)
        .unwrap();
        root
    }

    #[test]
    fn playing_registered_map_keeps_editor_snapshot_once_and_other_sources_aligned() {
        let root = registered_maps();
        let path = root.join("assets/scenes/ForestClearing.epokmap");
        let disk = fs::read(&path).unwrap();
        let registry = fs::read(root.join(REGISTRY)).unwrap();
        let mut scene = Scene::load_unresolved(&path).unwrap();
        scene.entities[0].position[0] += 7.;
        let input = Input::editor(path.clone(), scene);
        let loaded = load(&root, &input.scene, &input.origin).unwrap();
        assert_eq!(
            loaded
                .scenes
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            ["ForestClearing", "ForestBattle"]
        );
        assert_eq!(
            crate::scene_dependencies::signature(&loaded.scenes[0]),
            crate::scene_dependencies::signature(&input.scene)
        );
        assert_eq!(loaded.sources.len(), 1);
        assert!(
            matches!(&loaded.sources[0], Origin::Saved(p, _) if p.ends_with("ForestBattle.epokmap"))
        );
        // Identity survives an unsaved rename and path normalization.
        let mut renamed = input.scene.clone();
        renamed.name = "EditedForest".into();
        let origin = Origin::Editor(
            root.join("assets/scenes/../scenes/ForestClearing.epokmap"),
            String::new(),
        );
        assert_eq!(load(&root, &renamed, &origin).unwrap().scenes.len(), 2);
        assert_eq!(fs::read(&path).unwrap(), disk);
        assert_eq!(fs::read(root.join(REGISTRY)).unwrap(), registry);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn saved_registered_startup_is_included_once_and_normal_startup_keeps_all_banks() {
        let root = registered_maps();
        for (name, count) in [("Title", 3), ("ForestClearing", 2), ("ForestBattle", 2)] {
            let input = Input::load(&root.join(format!("assets/scenes/{name}.epokmap"))).unwrap();
            let loaded = load(&root, &input.scene, &input.origin).unwrap();
            assert_eq!(loaded.scenes.len(), count);
            assert_eq!(loaded.scenes[0].name, name);
            assert_eq!(loaded.sources.len(), count - 1);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn different_maps_with_same_name_are_still_rejected() {
        let root = registered_maps();
        let title = root.join("assets/scenes/Title.epokmap");
        let mut input = Input::load(&title).unwrap();
        input.scene.name = "ForestClearing".into();
        assert!(
            load(&root, &input.scene, &input.origin)
                .err()
                .unwrap()
                .contains("different maps")
        );
        let input = Input::load(&root.join("assets/scenes/ForestClearing.epokmap")).unwrap();
        let anonymous = Origin::Anonymous(crate::scene_dependencies::signature(&input.scene));
        assert!(load(&root, &input.scene, &anonymous).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "Builds a real PSX project; set EPOK_SCENE_TEST_PROJECT and EPOK_SCENE_TEST_MAP"]
    fn build_registered_map_from_editor_input() {
        let root = std::path::PathBuf::from(std::env::var("EPOK_SCENE_TEST_PROJECT").unwrap());
        let path = root.join(std::env::var("EPOK_SCENE_TEST_MAP").unwrap());
        let input = Input::load(&path).unwrap();
        let name = input.scene.name.clone();
        let job = crate::pipeline::Job::start_with_target(
            root.clone(),
            Input::editor(path, input.scene),
            false,
            false,
            false,
        );
        let mut built = false;
        loop {
            match job
                .events
                .recv_timeout(std::time::Duration::from_secs(180))
                .unwrap()
            {
                crate::pipeline::Event::Log(line) => println!("{line}"),
                crate::pipeline::Event::Built(path) => {
                    assert!(path.exists());
                    built = true;
                }
                crate::pipeline::Event::Finished(result) => {
                    result.unwrap();
                    break;
                }
                _ => {}
            }
        }
        assert!(built);
        let header = fs::read_to_string(root.join(".epok/build/scene.hh")).unwrap();
        assert_eq!(
            header.matches(&format!("{{\"{name}\",load_bank_")).count(),
            1
        );
    }

    #[test]
    fn bank_headers_share_pool_resources_and_reset_scripts() {
        let a = Scene::default();
        let mut b = a.clone();
        b.name = "Second".into();
        b.entities.pop();
        let h = header(&[a, b], &[]).unwrap();
        assert_eq!(h.matches("std::array<Entity,36> objects;").count(), 1);
        assert!(h.contains("scene_1::initial_objects[j]"));
        assert_eq!(
            h.matches("inline constexpr size_t texture_count=").count(),
            1
        );
        assert!(h.contains("objects[j].generation=generation"));
    }

    // ---- cooked actor tables (P10) --------------------------------------------------

    /// Reflected classes in the shape `Model::from_registry` resolves: the native bases
    /// of design.md section 2 plus one Blueprint actor class carrying a property.
    fn object_model_catalog() -> Vec<Script> {
        use crate::{object_model as om, reflection_schema as schema};
        fn class(id: &str, cpp_name: &str, parent: Option<&str>) -> schema::Class {
            schema::Class {
                family: None,
                domain: None,
                placement: Default::default(),
                component: None,
                default_components: vec![],
                explicit_abstract: false,
                id: id.into(),
                provider: schema::native_provider(),
                backend: schema::native_backend(),
                cpp_name: cpp_name.into(),
                parent: parent.map(str::to_owned),
                abstract_class: false,
                final_class: false,
                timeline_component: None,
                blueprintable: true,
                properties: vec![],
                functions: vec![],
                source: schema::Location {
                    file: std::path::PathBuf::from("runtime/object_model.hpp"),
                    line: 0,
                    column: 0,
                },
            }
        }
        let mut actor = class(om::ACTOR_ID, "epok::Actor", None);
        actor.family = Some(schema::ClassFamily::Actor);
        actor.abstract_class = true;
        let mut actor3d = class(om::ACTOR3D_ID, "epok::Actor3D", Some(om::ACTOR_ID));
        actor3d.domain = Some(schema::Domain::World3D);
        actor3d.placement = schema::Placement {
            placeable: true,
            spawnable: true,
            scene_managed: false,
        };
        let mut script_actor = class(
            om::SCENE_SCRIPT_ACTOR_ID,
            "epok::SceneScriptActor",
            Some(om::ACTOR_ID),
        );
        script_actor.placement = schema::Placement {
            placeable: false,
            spawnable: false,
            scene_managed: true,
        };
        let mut component = class(om::ACTOR_COMPONENT_ID, "epok::ActorComponent", None);
        component.family = Some(schema::ClassFamily::Component);
        component.abstract_class = true;
        let mut root = class(
            om::SCENE_COMPONENT3D_ID,
            "epok::SceneComponent3D",
            Some(om::ACTOR_COMPONENT_ID),
        );
        root.domain = Some(schema::Domain::World3D);
        root.component = Some(schema::ComponentContract {
            owners: [schema::Domain::World3D].into_iter().collect(),
            requires: vec![],
            excludes: vec![],
            cardinality: schema::Cardinality::Single,
            can_root: true,
            capabilities: Default::default(),
        });
        let mut audio = class(
            om::AUDIO_COMPONENT_ID,
            "epok::AudioComponent",
            Some(om::ACTOR_COMPONENT_ID),
        );
        audio.component = Some(schema::ComponentContract {
            owners: [
                schema::Domain::World3D,
                schema::Domain::World2D,
                schema::Domain::UI,
            ]
            .into_iter()
            .collect(),
            requires: vec![],
            excludes: vec![],
            cardinality: schema::Cardinality::Multiple,
            can_root: false,
            capabilities: ["audio".to_string()].into_iter().collect(),
        });
        let mut hero = class(HERO_ID, "BP_Hero", Some(om::ACTOR3D_ID));
        hero.properties = vec![schema::Property {
            id: "speed".into(),
            name: "speed".into(),
            value_type: schema::Type::Fixed,
            default: serde_json::json!(0.0),
            editable: true,
            timeline: None,
            source: hero.source.clone(),
        }];
        vec![Script {
            name: "epok::Actor".into(),
            parent: None,
            properties: vec![],
            header: std::path::PathBuf::from("object_model.hpp"),
            classes: vec![actor, actor3d, script_actor, component, root, audio, hero],
        }]
    }

    const HERO_ID: &str = "6b9dfe10-3f2d-4a41-9f7e-2b0f8c5a1d01";

    fn actor_scene() -> Scene {
        use crate::actor_document::{
            ActorInstance, ClassReference, ComponentInstance, SceneScript,
        };
        use crate::object_model as om;
        let mut scene = Scene {
            name: "Actors".into(),
            ..Default::default()
        };
        let hero_id = uuid::Uuid::parse_str("7d0f1c02-9f3a-4c0e-9d3a-6b1c2f7d4a11").unwrap();
        let child_id = uuid::Uuid::parse_str("7d0f1c02-9f3a-4c0e-9d3a-6b1c2f7d4a12").unwrap();
        let root_id = uuid::Uuid::parse_str("1a2b3c4d-0000-4000-8000-000000000001").unwrap();
        let audio_id = uuid::Uuid::parse_str("1a2b3c4d-0000-4000-8000-000000000002").unwrap();
        let child_root = uuid::Uuid::parse_str("1a2b3c4d-0000-4000-8000-000000000003").unwrap();

        let mut hero = ActorInstance::new(hero_id, ClassReference::new("BP_Hero", HERO_ID), "Hero");
        hero.legacy_entity = Some(scene.entities[0].id);
        hero.components = vec![
            {
                let mut c = ComponentInstance::new(
                    root_id,
                    ClassReference::new("epok::SceneComponent3D", om::SCENE_COMPONENT3D_ID),
                    "Root",
                );
                c.root = true;
                c
            },
            ComponentInstance::new(
                audio_id,
                ClassReference::new("epok::AudioComponent", om::AUDIO_COMPONENT_ID),
                "Footsteps",
            ),
        ];
        hero.properties
            .insert("speed".into(), serde_json::json!(2.5));
        hero.overrides.insert("speed".into());

        // Emitted second: a logical child whose root attaches to the hero's root.
        let mut child = ActorInstance::new(
            child_id,
            ClassReference::new("epok::Actor3D", om::ACTOR3D_ID),
            "Marker",
        );
        child.logical_parent = Some(hero_id);
        child.attach = Some(crate::actor_document::Attachment {
            actor: hero_id,
            component: Some(root_id),
        });
        child.active = false;
        child.components = vec![{
            let mut c = ComponentInstance::new(
                child_root,
                ClassReference::new("epok::SceneComponent3D", om::SCENE_COMPONENT3D_ID),
                "Root",
            );
            c.root = true;
            c
        }];

        scene.actors = vec![hero, child];
        scene.scene_script = Some(SceneScript {
            parent: ClassReference::new("epok::SceneScriptActor", om::SCENE_SCRIPT_ACTOR_ID),
            blueprint: crate::blueprint_asset::BlueprintAsset::new(
                "MapScript".into(),
                om::SCENE_SCRIPT_ACTOR_ID.into(),
            ),
        });
        scene
    }

    #[test]
    fn actor_banks_emit_records_overrides_scene_script_and_registry_capacity() {
        let catalog = object_model_catalog();
        let scene = actor_scene();
        let text = header(std::slice::from_ref(&scene), &catalog).unwrap();
        let compact = crate::blueprint_refs::compact_id;

        // Capacity policy: actors + components + 32 dynamic slots, before any bank body.
        assert!(
            text.contains(
                "#define EPOK_OBJECT_REGISTRY_CAPACITY 37\n#include \"actor_tables.hpp\""
            )
        );
        // Component records: the root carries the legacy entity slot behind it, the
        // audio component does not; attach_parent is a component index of the same actor.
        assert!(text.contains(&format!(
            "inline constexpr ActorComponentRecord actor_components_0[]={{{{UINT64_C({}),\"Root\",true,-1,0}},{{UINT64_C({}),\"Footsteps\",false,-1,-1}}}};",
            compact(crate::object_model::SCENE_COMPONENT3D_ID),
            compact(crate::object_model::AUDIO_COMPONENT_ID)
        )));
        // Actor records: class, name, active, logical parent index, attach indices.
        assert!(text.contains(&format!(
            "{{UINT64_C({}),\"Hero\",true,-1,-1,-1,actor_components_0,2,&actor_apply_0}}",
            compact(HERO_ID)
        )));
        assert!(text.contains(&format!(
            "{{UINT64_C({}),\"Marker\",false,0,0,0,actor_components_1,1,nullptr}}",
            compact(crate::object_model::ACTOR3D_ID)
        )));
        assert!(
            text.contains("inline constexpr ActorTable actor_table={actor_records,2,UINT64_C(")
        );
        // Overrides go through the reflected assignment generator, not ad-hoc text.
        assert!(text.contains("inline void actor_apply_0(ObjectRegistry& registry,Actor& actor,const ObjectId* components){"));
        assert!(text.contains("if(auto* self=registry.resolve<BP_Hero>(actor.id())){"));
        assert!(text.contains("self->speed = Fixed(10240, Fixed::RAW);"));
        // The scene script class is cooked per bank and reachable from epok.
        let script = compact(crate::object_model::SCENE_SCRIPT_ACTOR_ID);
        assert!(text.contains(&format!(
            "inline constexpr uint64_t scene_script_class=UINT64_C({script});"
        )));
        assert!(text.contains(
            "inline constexpr uint64_t scene_script_class_0=scene_0::scene_script_class;"
        ));
        // The bank loader brings the actor half up before the legacy bindings.
        let load = text
            .split("load_actor_bank(scene_0::actor_table,objects.data(),object_count);")
            .nth(1)
            .unwrap();
        assert!(load.contains("scene_0::initialize_scripts();"));
        // Deterministic: re-cooking the same document is byte identical.
        assert_eq!(text, header(&[scene], &catalog).unwrap());
    }

    #[test]
    fn a_scene_without_actors_emits_the_empty_table_and_unchanged_legacy_text() {
        let a = Scene::default();
        let mut b = a.clone();
        b.name = "Second".into();
        b.entities.pop();
        let text = header(&[a, b], &[]).unwrap();
        // Legacy byte shape is untouched.
        assert_eq!(text.matches("std::array<Entity,36> objects;").count(), 1);
        assert!(text.contains("scene_1::initial_objects[j]"));
        // Empty tables, still one per bank, so main.cpp always links.
        assert_eq!(
            text.matches("inline constexpr ActorTable actor_table={nullptr,0,UINT64_C(0)};")
                .count(),
            2
        );
        assert_eq!(text.matches("actor_registry_slots=0;").count(), 2);
        assert!(text.contains("#define EPOK_OBJECT_REGISTRY_CAPACITY 32\n"));
        assert!(!text.contains("actor_records"));
        // The object class table exists even with no reflected object-model class.
        assert!(text.contains("inline const size_t object_class_count=0;"));

        // Byte neutrality: a project with no actors, no scene script and no object-model
        // class differs from the pre-initiative shape by exactly these additive lines.
        // Removing them must leave nothing of the initiative behind, so a future phase
        // cannot quietly add a line to every existing project's generated header.
        let mut rest = text.clone();
        let additive = [
            "#define EPOK_OBJECT_REGISTRY_CAPACITY 32\n",
            "#include \"actor_tables.hpp\"\n",
            "inline const ClassDescriptor object_classes[] = {\n{},\n};\n",
            "inline const size_t object_class_count=0;\n",
            "inline constexpr ActorTable actor_table={nullptr,0,UINT64_C(0)};\n",
            "inline constexpr ActorTable actor_table={nullptr,0,UINT64_C(0)};\n",
            "inline constexpr uint64_t scene_script_class=UINT64_C(0);\n",
            "inline constexpr uint64_t scene_script_class=UINT64_C(0);\n",
            "inline constexpr size_t actor_registry_slots=0;\n",
            "inline constexpr size_t actor_registry_slots=0;\n",
            "inline constexpr uint64_t scene_script_class_0=scene_0::scene_script_class;\n",
            "inline constexpr uint64_t scene_script_class_1=scene_1::scene_script_class;\n",
            "load_actor_bank(scene_0::actor_table,objects.data(),object_count);\n",
            "load_actor_bank(scene_1::actor_table,objects.data(),object_count);\n",
        ];
        for line in additive {
            let at = rest
                .find(line)
                .unwrap_or_else(|| panic!("additive line missing: {line:?}"));
            rest.replace_range(at..at + line.len(), "");
        }
        for token in [
            "actor_tables",
            "ActorTable",
            "ActorRecord",
            "ActorComponentRecord",
            "SceneReference",
            "scene_reference",
            "scene_script",
            "actor_registry_slots",
            "object_class",
            "ClassDescriptor",
            "EPOK_OBJECT_REGISTRY_CAPACITY",
            "load_actor_bank",
            "ObjectPool",
        ] {
            assert!(
                !rest.contains(token),
                "the actor initiative leaked {token} into a project that has no actors"
            );
        }
    }

    const MAP_SCRIPT_ID: &str = "6b9dfe10-3f2d-4a41-9f7e-2b0f8c5a1d02";
    const HERO_REF_ID: &str = "6b9dfe10-3f2d-4a41-9f7e-2b0f8c5a1d03";
    const SLOT_REF_ID: &str = "6b9dfe10-3f2d-4a41-9f7e-2b0f8c5a1d04";

    /// The map's own Blueprint, with one `ActorRef` and one `EntityRef` typed in. P6
    /// lowers both defaults to the null identity; the resolution is what P10/P11 cook.
    fn scene_with_map_scoped_references() -> (Scene, Vec<Script>) {
        use crate::{blueprint_asset as asset, reflection_schema as schema};
        let mut scene = actor_scene();
        let hero = scene.actors[0].id;
        let entity = scene.entities[0].id;
        let script = scene.scene_script.as_mut().unwrap();
        script.blueprint.id = MAP_SCRIPT_ID.into();
        let variable = |id: &str, name: &str, value_type, default| asset::Variable {
            id: id.into(),
            name: name.into(),
            value_type,
            default,
            editable: true,
            timeline_animatable: false,
        };
        script.blueprint.variables = vec![
            variable(
                HERO_REF_ID,
                "hero",
                schema::Type::ActorRef { class: None },
                serde_json::json!(hero.to_string()),
            ),
            variable(
                SLOT_REF_ID,
                "marker_slot",
                schema::Type::EntityRef { class: None },
                serde_json::json!(entity.to_string()),
            ),
        ];
        // The generated scene-script class, as the Blueprint compiler publishes it.
        let mut catalog = object_model_catalog();
        let mut generated = catalog[0]
            .classes
            .iter()
            .find(|class| class.id == crate::object_model::SCENE_SCRIPT_ACTOR_ID)
            .cloned()
            .unwrap();
        generated.id = MAP_SCRIPT_ID.into();
        generated.cpp_name = "Actors_SceneScript".into();
        generated.parent = Some(crate::object_model::SCENE_SCRIPT_ACTOR_ID.into());
        generated.placement = Default::default();
        catalog[0].classes.push(generated);
        (scene, catalog)
    }

    /// The reference table names table indices, never UUIDs and never names, and carries
    /// a generated writer because typed member access needs the C++ class.
    #[test]
    fn map_scoped_references_cook_into_a_scene_reference_table_of_indices() {
        let (scene, catalog) = scene_with_map_scoped_references();
        let text = header(std::slice::from_ref(&scene), &catalog).unwrap();
        let compact = crate::blueprint_refs::compact_id;
        assert!(text.contains(&format!(
            "inline constexpr SceneReferenceRecord scene_references[]={{{{UINT64_C({}),UINT64_C({}),SceneRefKind::Actor,0,-1,-1}},{{UINT64_C({}),UINT64_C({}),SceneRefKind::Entity,-1,-1,0}}}};",
            compact(MAP_SCRIPT_ID),
            compact(HERO_REF_ID),
            compact(MAP_SCRIPT_ID),
            compact(SLOT_REF_ID)
        )));
        assert!(text.contains("inline void scene_reference_bind(ObjectRegistry& registry,Actor& owner,uint64_t member,ObjectId target,EntityHandle entity){"));
        assert!(text.contains("if(auto* self=registry.resolve<Actors_SceneScript>(owner.id())){"));
        assert!(text.contains(&format!(
            "if(member==UINT64_C({})){{self->hero = target;}}",
            compact(HERO_REF_ID)
        )));
        assert!(text.contains(&format!(
            "if(member==UINT64_C({})){{self->marker_slot = entity;}}",
            compact(SLOT_REF_ID)
        )));
        // The ActorTable points at the rows; no UUID reaches the generated text.
        assert!(text.contains(",scene_references,2,&scene_reference_bind};"));
        assert!(!text.contains(&scene.actors[0].id.to_string()));
        // Deterministic: re-cooking the same document is byte identical.
        assert_eq!(text, header(&[scene], &catalog).unwrap());
    }

    /// Capacity is a diagnostic, not a silent increase: the runtime arrays are fixed.
    #[test]
    fn exceeding_a_runtime_capacity_is_a_cook_diagnostic_naming_the_bound() {
        use crate::actor_document::{ActorInstance, ClassReference, ComponentInstance};
        use crate::object_model as om;
        let catalog = object_model_catalog();
        let actor = |n: usize| {
            ActorInstance::new(
                uuid::Uuid::from_u128(0x4000_0000_0000_0000_0000_0000_0000_0000 + n as u128),
                ClassReference::new("epok::Actor3D", om::ACTOR3D_ID),
                &format!("A{n}"),
            )
        };
        let mut scene = actor_scene();
        scene.actors = (0..LEVEL_ACTOR_CAPACITY + 1).map(actor).collect();
        let error = header(&[scene], &catalog).unwrap_err();
        assert!(error.contains("65 actors"), "{error}");
        assert!(
            error.contains(&LEVEL_ACTOR_CAPACITY.to_string()) && error.contains("Actors"),
            "{error}"
        );

        let mut scene = actor_scene();
        scene.actors = vec![actor(0)];
        scene.actors[0].components = (0..ACTOR_COMPONENT_CAPACITY + 1)
            .map(|n| {
                ComponentInstance::new(
                    uuid::Uuid::from_u128(0x5000_0000_0000_0000_0000_0000_0000_0000 + n as u128),
                    ClassReference::new("epok::AudioComponent", om::AUDIO_COMPONENT_ID),
                    &format!("C{n}"),
                )
            })
            .collect();
        let error = header(&[scene], &catalog).unwrap_err();
        assert!(error.contains("9 components"), "{error}");
        assert!(error.contains(&ACTOR_COMPONENT_CAPACITY.to_string()), "{error}");

        // Inside the bounds the capacity is emitted, never rounded up.
        let mut scene = actor_scene();
        scene.actors.truncate(1);
        scene.actors[0].components.truncate(1);
        assert!(
            header(&[scene], &catalog)
                .unwrap()
                .contains("#define EPOK_OBJECT_REGISTRY_CAPACITY 34\n")
        );
    }

    /// Typed object pools stay inside the 64 KiB image budget; the cook emits the
    /// static_assert that fails the target build rather than growing a pool silently.
    #[test]
    fn object_pools_carry_the_sixty_four_kibibyte_cook_assert() {
        let text = header(&[actor_scene()], &object_model_catalog()).unwrap();
        assert!(
            text.contains(
                "static_assert((epok::ObjectPool<BP_Hero,4>::storage_bytes+epok::ObjectPool<epok::Actor3D,4>::storage_bytes+epok::ObjectPool<epok::AudioComponent,4>::storage_bytes+epok::ObjectPool<epok::SceneComponent3D,4>::storage_bytes+epok::ObjectPool<epok::SceneScriptActor,4>::storage_bytes) <= 65536, \"Object pools exceed the 64 KiB cook limit\");"
            ),
            "{}",
            text.lines()
                .find(|line| line.contains("64 KiB"))
                .unwrap_or_default()
        );
    }

    #[test]
    fn an_unknown_override_key_is_a_cook_diagnostic_naming_the_key() {
        let catalog = object_model_catalog();
        let mut scene = actor_scene();
        scene.actors[0]
            .properties
            .insert("stamina".into(), serde_json::json!(1.0));
        scene.actors[0].overrides.insert("stamina".into());
        let error = header(&[scene], &catalog).unwrap_err();
        assert!(error.contains("stamina"), "{error}");
        assert!(error.contains("BP_Hero"), "{error}");
    }

    #[test]
    fn generated_actor_tables_reference_no_absolute_worktree_path() {
        let text = header(&[actor_scene()], &object_model_catalog()).unwrap();
        let root = env!("CARGO_MANIFEST_DIR");
        assert!(!text.contains(root), "generated text leaked {root}");
        for line in text.lines().filter(|l| l.starts_with("#include")) {
            assert!(
                !line.contains(':') && !line.contains(".."),
                "non-portable include: {line}"
            );
        }
    }

    #[test]
    fn an_override_or_scene_script_change_changes_the_cooked_text_and_nothing_else_does() {
        let catalog = object_model_catalog();
        let base = actor_scene();
        let table = |scene: &Scene| {
            let text = header(std::slice::from_ref(scene), &catalog).unwrap();
            text.split("inline void actor_apply_0")
                .nth(1)
                .unwrap()
                .split("namespace epok {inline void load_bank_0")
                .next()
                .unwrap()
                .to_string()
        };
        let before = table(&base);

        // An actor property override is cooked into the emitted setters.
        let mut changed = base.clone();
        changed.actors[0]
            .properties
            .insert("speed".into(), serde_json::json!(4.0));
        assert_ne!(table(&changed), before);

        // So is the scene script parent.
        let mut script = base.clone();
        script.scene_script.as_mut().unwrap().parent =
            crate::actor_document::ClassReference::new("BP_Hero", HERO_ID);
        assert_ne!(
            header(&[script], &catalog).unwrap(),
            header(std::slice::from_ref(&base), &catalog).unwrap()
        );

        // An unrelated legacy edit leaves the actor half byte identical.
        let mut unrelated = base.clone();
        unrelated.entities[0].material.color = [0.1, 0.2, 0.3];
        assert_eq!(table(&unrelated), before);
        assert_ne!(
            header(&[unrelated], &catalog).unwrap(),
            header(&[base], &catalog).unwrap()
        );
    }
}
