use crate::{scene::Scene, scripts};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub make: String,
    pub toolchain_bin: String,
    pub nugget: String,
    pub emulator: String,
    pub psxavenc: String,
    pub mkpsxiso: String,
    pub libclang: String,
    pub code: String,
    pub web_port: u16,
    pub auto_build: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            make: "make".into(),
            toolchain_bin: String::new(),
            nugget: "third_party/nugget".into(),
            emulator: "pcsx-redux".into(),
            psxavenc: if cfg!(windows) {
                ".tools/psxavenc/bin/psxavenc.exe".into()
            } else {
                "psxavenc".into()
            },
            mkpsxiso: if cfg!(windows) {
                ".tools/mkpsxiso/mkpsxiso-2.30-win64/mkpsxiso.exe".into()
            } else {
                "mkpsxiso".into()
            },
            code: String::new(),
            libclang: String::new(),
            web_port: 8077,
            auto_build: false,
        }
    }
}
impl Config {
    pub fn owner(root: &Path) -> PathBuf {
        if root.join("Local.epokconfig").exists() {
            root.into()
        } else {
            crate::workspace::editor_home()
        }
    }
    pub fn editable(root: &Path) -> Result<(PathBuf, Self), String> {
        let owner = Self::owner(root);
        let config = Self::from_directory(&owner)?;
        Ok((owner, config))
    }
    pub fn load(root: &Path) -> Result<Self, String> {
        // Machine overrides are untracked and resolved relative to their owner.
        let (owner, mut config) = Self::editable(root)?;
        config.resolve_paths(&owner);
        if let Some(manifest) = crate::workspace::optional_manifest(root)? {
            config.auto_build = manifest.auto_build;
        }
        Ok(config)
    }
    fn from_directory(root: &Path) -> Result<Self, String> {
        let local = root.join("Local.epokconfig");
        let path = if local.exists() {
            local
        } else {
            root.join("Editor.epokconfig")
        };
        if !path.exists() {
            return Ok(Self::default());
        }
        crate::document::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Configuration {}: {e}", path.display()))
    }
    pub fn resolve_paths(&mut self, root: &Path) {
        self.make = Self::executable(root, &self.make).to_string_lossy().into();
        self.emulator = Self::executable(root, &self.emulator)
            .to_string_lossy()
            .into();
        self.psxavenc = Self::executable(root, &self.psxavenc)
            .to_string_lossy()
            .into();
        self.mkpsxiso = Self::executable(root, &self.mkpsxiso)
            .to_string_lossy()
            .into();
        self.nugget = Self::path(root, &self.nugget).to_string_lossy().into();
        self.libclang = if self.libclang.is_empty() {
            crate::workspace::editor_home()
                .join(".tools/libclang/libclang-18.1.1.data/platlib/clang/native")
        } else {
            Self::path(root, &self.libclang)
        }
        .to_string_lossy()
        .into();
        if !self.toolchain_bin.is_empty() {
            self.toolchain_bin = Self::path(root, &self.toolchain_bin)
                .to_string_lossy()
                .into();
        }
        if !self.code.is_empty() {
            self.code = Self::executable(root, &self.code).to_string_lossy().into();
        }
    }
    pub fn path(root: &Path, value: &str) -> PathBuf {
        let p = PathBuf::from(value);
        if p.is_absolute() { p } else { root.join(p) }
    }
    pub fn executable(root: &Path, value: &str) -> PathBuf {
        if value.contains(['/', '\\']) {
            Self::path(root, value)
        } else {
            PathBuf::from(value)
        }
    }
}
pub fn write_changed(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return crate::staging_files::written(path, bytes);
    }
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    fs::write(path, bytes).map_err(|e| e.to_string())?;
    crate::staging_files::written(path, bytes)
}
#[cfg(test)]
pub fn scene_header(scene: &Scene, catalog: &[scripts::Script]) -> Result<String, String> {
    scene_header_with_resources(scene, catalog, scene)
}
#[cfg(test)]
pub fn scene_header_with_resources(
    scene: &Scene,
    catalog: &[scripts::Script],
    resources: &Scene,
) -> Result<String, String> {
    scene_header_body_with_layout(scene, catalog, resources, true, scene)
}
#[cfg(test)]
pub fn scene_header_body_with_layout(
    scene: &Scene,
    catalog: &[scripts::Script],
    resources: &Scene,
    emit_resources: bool,
    layout_scene: &Scene,
) -> Result<String, String> {
    let mut registry = crate::actor_document::tests::registry();
    registry
        .classes
        .extend(crate::blueprint::registry_from_catalog(Path::new(""), catalog).classes);
    scene_header_with_registry(
        scene,
        catalog,
        resources,
        emit_resources,
        layout_scene,
        &registry,
    )
}
pub fn scene_header_with_registry(
    scene: &Scene,
    catalog: &[scripts::Script],
    resources: &Scene,
    emit_resources: bool,
    layout_scene: &Scene,
    class_registry: &crate::blueprint::Registry,
) -> Result<String, String> {
    scene_header_with_registry_for(
        scene,
        catalog,
        resources,
        emit_resources,
        layout_scene,
        class_registry,
        crate::skeletal_compile::QueryDemand::ALL,
    )
}
pub fn scene_header_with_registry_for(
    scene: &Scene,
    catalog: &[scripts::Script],
    resources: &Scene,
    emit_resources: bool,
    layout_scene: &Scene,
    class_registry: &crate::blueprint::Registry,
    skeletal_queries: crate::skeletal_compile::QueryDemand,
) -> Result<String, String> {
    scene.validate()?;
    let mut prepared = scene.clone();
    prepared.sync_actor_components();
    if !crate::lighting::valid_bake(&prepared) {
        prepared.bake = Some(crate::lighting::bake(&prepared)?);
    }
    let scene = &prepared;
    let mut text = String::from(
        "// Generated. Edit the scene and scripts, not this file.\n#pragma once\n#include \"epok.hpp\"\n#include \"actor_tables.hpp\"\n#include <array>\n",
    );
    for script in catalog {
        text.push_str(&format!("#include \"scripts/{}\"\n", script.header_path()));
    }
    text.push_str("namespace epok {\n");
    let (navigation_table, navigation_init) = crate::navigation::cpp(scene)?;
    text.push_str(&navigation_table);
    if emit_resources {
        text.push_str(&crate::texture::header(resources)?);
    }
    let texture_ids = crate::texture::ids(resources);
    let (sprite_tables, sprite_init) = crate::sprites::generate(scene, &texture_ids);
    text.push_str(&sprite_tables);
    let mut topologies = std::collections::BTreeMap::new();
    for e in scene
        .actors
        .iter()
        .filter(|e| e.kind == "Mesh" && e.editable_mesh.is_none() && e.skeletal_mesh.is_none())
    {
        let key = (e.lighting.subdivisions, crate::lighting::tiled(e));
        if topologies.contains_key(&key) {
            continue;
        }
        let id = topologies.len();
        topologies.insert(key, id);
        let mut vertices = Vec::<[i16; 3]>::new();
        let mut quads = Vec::new();
        for q in crate::lighting::quads(e) {
            let indices = q.points.map(|p| {
                let p = p.map(|c| (c * 4096.).round() as i16);
                if let Some(i) = vertices.iter().position(|v| *v == p) {
                    i
                } else {
                    vertices.push(p);
                    vertices.len() - 1
                }
            });
            quads.push((indices, q.face, q.uv));
        }
        if vertices.len() > 512 {
            return Err("Indexed mesh exceeds 512 cached vertices".into());
        }
        text.push_str(&format!(
            "inline constexpr int16_t mesh_vertices_{id}[{}][3]={{{}}};\n",
            vertices.len(),
            vertices
                .iter()
                .map(|p| format!("{{{},{},{}}}", p[0], p[1], p[2]))
                .collect::<Vec<_>>()
                .join(",")
        ));
        text.push_str(&format!(
            "inline constexpr MeshQuad mesh_quads_{id}[{}]={{{}}};\n",
            quads.len(),
            quads
                .iter()
                .map(|(v, f, uv)| format!(
                    "{{{{{},{},{},{}}},{f},{{}},{{{{255,255,255}},false}},0,{}}}",
                    v[0],
                    v[1],
                    v[2],
                    v[3],
                    crate::texture::uv_cpp(*uv)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ));
        text.push_str(&format!("inline constexpr MeshGeometry mesh_{id}={{mesh_vertices_{id},{},mesh_quads_{id},{}}};\n",vertices.len(),quads.len()));
    }
    // Page geometry follows this bank's VRAM layout, the same one its texture
    // descriptors are generated from.
    let layout = crate::texture::layout(layout_scene)?;
    let pages = |id: uuid::Uuid| {
        let placement = layout.iter().find(|(v, _)| *v == id)?;
        let t = scene.textures.get(&id)?;
        Some((t.width, t.height, placement.1.y))
    };
    for (index, e) in scene
        .actors
        .iter()
        .enumerate()
        .filter(|(_, e)| e.kind == "Mesh" && e.editable_mesh.is_some())
    {
        text.push_str(&crate::mesh_compile::header_with_pages(e, index, &pages)?);
    }
    let mut skeletal_assets = std::collections::BTreeMap::new();
    for (i, e) in scene.actors.iter().enumerate() {
        if let Some(c) = &e.skeletal_mesh
            && let std::collections::btree_map::Entry::Vacant(entry) =
                skeletal_assets.entry(c.asset)
        {
            entry.insert(i);
            text.push_str(&crate::skeletal_compile::header_with_pages_for(
                e,
                i,
                &pages,
                skeletal_queries,
            )?);
        }
    }
    text.push_str("}\n");
    text.push_str(&format!(
        "namespace epok {{\ninline std::array<ActorData, {}> objects = {{{{\n",
        scene.actors.len() + 32
    ));
    let fixed = |v: f32| format!("Fixed({}, Fixed::RAW)", (v as f64 * 4096.).round() as i32);
    for (index, e) in scene.actors.iter().enumerate() {
        if e.position.iter().any(|v| v.abs() > 128.) || e.scale.iter().any(|v| *v > 64.) {
            return Err(format!(
                "{}: initial PSX renderer supports positions ±128 and scales up to 64",
                e.name
            ));
        }
        let world = scene.world_matrix(index);
        if world
            .0
            .iter()
            .any(|row| row[3].abs() > 128. || row[..3].iter().any(|v| v.abs() > 64.))
        {
            return Err(format!(
                "{}: inherited world transform exceeds PSX positions ±128 or basis ±64",
                e.name
            ));
        }
        if e.scale.iter().any(|s| *s < 1. / 4096.) {
            return Err(format!("{}: scale is too small for PSX Q12", e.name));
        }
        let vector = |v: &[f32; 3]| v.iter().map(|v| fixed(*v)).collect::<Vec<_>>().join(",");
        // EditableMesh owns its material slots; the legacy cube renderer's tint is unrelated.
        let material = if e.editable_mesh.is_some() || e.skeletal_mesh.is_some() {
            crate::scene::Material::default()
        } else {
            e.material.clone()
        };
        text.push_str(&format!(
            "{{{}, {}, {}, {}, {{{{{}}},{{{}}},{{{}}}}}, {{{{{}}}, {}}}}},\n",
            e.kind == "Camera",
            e.kind == "Mesh",
            tiled(e),
            e.parent.map_or(-1, |p| p as i32),
            vector(&e.position),
            vector(&e.rotation),
            vector(&e.scale),
            material
                .color
                .iter()
                .map(|c| ((c * 255.).round() as u8).to_string())
                .collect::<Vec<_>>()
                .join(","),
            material.unlit
        ));
    }
    text.push_str("}};\n");
    text.push_str(&format!(
        "inline constexpr size_t authored_count={};\ninline size_t object_count=authored_count;\n",
        scene.actors.len()
    ));
    text.push_str(&format!(
        "inline constexpr size_t render_capacity={};\n",
        scene
            .actors
            .iter()
            .map(crate::lighting::quad_count)
            .sum::<usize>()
            * 2
            + 32 * 12
            + 513
    ));
    for (i, colors) in scene.bake.as_ref().unwrap().colors.iter().enumerate() {
        if !colors.is_empty() {
            text.push_str(&format!(
                "inline constexpr uint8_t baked_{i}[{}][3]={{{}}};\n",
                colors.len(),
                colors
                    .iter()
                    .map(|c| format!("{{{},{},{}}}", c[0], c[1], c[2]))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
    }
    text.push_str(
        r#"
inline void initialize_components(){
"#,
    );
    let color = |c: &[f32; 3]| {
        c.iter()
            .map(|v| ((v * 255.).round() as u8).to_string())
            .collect::<Vec<_>>()
            .join(",")
    };
    let vec2 = |c: &[f32; 2]| c.iter().map(|v| fixed(*v)).collect::<Vec<_>>().join(",");
    let string = |s: &str| {
        format!(
            "\"{}\"",
            s.bytes().map(|b| format!("\\{b:03o}")).collect::<String>()
        )
    };
    text.push_str(&format!(
        "lighting_environment=LightingEnvironment{{{{{}}},{}}};\n",
        scene
            .environment
            .ambient
            .iter()
            .map(|v| fixed(*v))
            .collect::<Vec<_>>()
            .join(","),
        scene.environment.point_lights
    ));
    let clips = crate::audio::clip_ids(resources);
    for (i, e) in scene.actors.iter().enumerate() {
        if let Some(audio) = &e.audio {
            let clip = audio.clip.map_or(-1, |id| {
                clips.iter().position(|candidate| *candidate == id).unwrap() as i32
            });
            text.push_str(&format!("objects[{i}].audio.enabled=true;objects[{i}].audio.clip={clip};objects[{i}].audio.volume={};objects[{i}].audio.pitch={};objects[{i}].audio.play_on_start={};objects[{i}].audio.priority={};\n", fixed(audio.volume),fixed(audio.pitch),audio.play_on_start,audio.priority));
        }
        if e.kind == "Mesh" {
            if let Some(c) = &e.skeletal_mesh {
                let model = c.model.as_ref().ok_or("Unresolved skeletal model")?;
                let clip = c
                    .clip
                    .and_then(|id| model.clips.iter().position(|(key, _)| *key == id))
                    .map_or(-1, |v| v as i32);
                let asset = skeletal_assets[&c.asset];
                text.push_str(&format!("objects[{i}].animator=Animator{{true,&skin_{asset},{clip},0,{},{}}};objects[{i}].geometry=&skin_geometry_{asset};\n",c.play_on_start,c.looping));
            } else if e.editable_mesh.is_some() {
                text.push_str(&format!("objects[{i}].geometry=&editable_{i}_0;\n"));
            } else {
                let id = topologies[&(e.lighting.subdivisions, crate::lighting::tiled(e))];
                text.push_str(&format!("objects[{i}].geometry=&mesh_{id};\n"));
            }
        }
        text.push_str(&format!("objects[{i}].set_name({});\n", string(&e.name)));
        text.push_str(&format!("objects[{i}].active={};\n", e.active));
        if e.kind == "Camera" {
            text.push_str(&format!(
                "objects[{i}].camera_settings={{true,{},{{{}}}}};\n",
                fixed(e.camera_fov),
                color(&e.camera_sky_color)
            ));
        }
        if e.editable_mesh.is_none() && e.skeletal_mesh.is_none() {
            text.push_str(&format!(
                "objects[{i}].material={};\n",
                crate::texture::material_cpp(&e.material)
            ));
        }
        text.push_str(&format!(
            "objects[{i}].lighting=MeshLighting{{true,ReceiveLighting::{:?},{},{},{},{}}};\n",
            e.lighting.receive,
            e.lighting.static_geometry,
            e.lighting.cast_shadows,
            e.lighting.subdivisions,
            e.lighting.background_pass
        ));
        if crate::lighting::baked(e) {
            text.push_str(&format!(
                "objects[{i}].baked_colors=baked_{i};objects[{i}].baked_color_count={};\n",
                scene.bake.as_ref().unwrap().colors[i].len()
            ));
        }
        if let Some(l) = &e.light {
            text.push_str(&format!(
                "objects[{i}].light=Light{{{},LightType::{:?},LightMode::{:?},{{{}}},{},{},{}}};\n",
                l.enabled,
                l.kind,
                l.mode,
                l.color
                    .iter()
                    .map(|c| ((c * 255.).round() as u8).to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                fixed(l.intensity),
                fixed(l.range),
                l.priority
            ));
        }
        if let Some(b) = &e.blob_shadow {
            text.push_str(&format!(
                "objects[{i}].blob_shadow=BlobShadow{{{},{},{},{}}};\n",
                b.enabled,
                fixed(b.radius),
                fixed(b.strength),
                fixed(b.distance)
            ));
        }
        if let Some(c) = &e.canvas {
            text.push_str(&format!("objects[{i}].canvas.enabled={};\n", c.enabled));
        }
        if let Some(r) = &e.rect {
            text.push_str(&format!(
                "objects[{i}].rect=RectTransform{{true,{{{}}},{{{}}},{{{}}},{{{}}},{{{}}}}};\n",
                vec2(&r.anchor_min),
                vec2(&r.anchor_max),
                vec2(&r.pivot),
                vec2(&r.position),
                vec2(&r.size)
            ));
        }
        if let Some(c) = &e.image {
            text.push_str(&format!(
                "objects[{i}].image=Image{{{},{{{}}},{},{{{}}},{{{}}}}};\n",
                c.enabled,
                color(&c.color),
                c.texture.map(crate::texture::symbol).unwrap_or("-1".into()),
                c.region.map(|v| v.to_string()).join(","),
                c.borders.map(|v| v.to_string()).join(",")
            ));
        }
        if let Some(c) = &e.text {
            text.push_str(&format!(
                "objects[{i}].text=Text{{{},{{{}}}}};objects[{i}].text.set_text({});objects[{i}].text.wrap={};\n",
                c.enabled,
                color(&c.color),
                string(&c.text),c.wrap
            ));
        }
        if let Some(c) = &e.progress {
            text.push_str(&format!(
                "objects[{i}].progress=ProgressBar{{{},{},{{{}}},{{{}}}}};\n",
                c.enabled,
                fixed(c.value),
                color(&c.color),
                color(&c.background)
            ));
        }
    }

    text.push_str(&sprite_init);
    text.push_str(&crate::particles::generate(scene, &texture_ids));
    text.push_str(&crate::collision::cpp_setup(scene));
    text.push_str(&navigation_init);
    text.push_str(&crate::palette::cpp_setup(scene));
    text.push_str(&crate::effects::cpp_setup(scene));
    text.push_str("}\n");

    let mut order: Vec<_> = (0..scene.actors.len()).collect();
    order.sort_by_key(|i| {
        let mut depth = 0;
        let mut p = scene.spatial_parent(*i);
        while let Some(index) = p {
            depth += 1;
            p = scene.spatial_parent(index);
        }
        depth
    });
    text.push_str(&format!(
        "inline constexpr std::array<size_t, {}> transform_order = {{{{{}}}}};\n",
        order.len(),
        order
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",")
    ));
    text.push_str(&format!(
        "inline constexpr size_t terrain_count = {};\n",
        scene.actors.iter().filter(|e| tiled(e)).count()
    ));
    text.push_str("inline void initialize_scripts() {initialize_components();}\n");
    text.push_str(&actor_table_with_registry(scene, class_registry, &clips)?);
    text.push_str("#ifdef EPOK_EDITOR_PREVIEW\ninline void initialize_editor_preview() {initialize_components();}\n#endif\n}\n");
    Ok(text)
}
fn tiled(e: &crate::scene::Actor) -> bool {
    crate::lighting::tiled(e)
}
#[cfg(test)]
pub fn stage(root: &Path, scene: &Scene) -> Result<PathBuf, String> {
    stage_into(root, scene, &root.join(".epok/build"))
}
pub fn refresh_linked_scene(root: &Path, scene: &mut Scene) -> Result<(), String> {
    if scene
        .actors
        .iter()
        .any(|entity| entity.blueprint_instance.is_some())
    {
        let catalog = crate::scripts::catalog(root)?;
        let registry = crate::blueprint::registry_from_catalog(root, &catalog);
        crate::blueprint_templates::refresh_instances(
            scene,
            &crate::blueprint_asset::load_all(root)?,
            &registry,
        )?;
    }
    Ok(())
}
#[cfg(test)]
pub fn stage_into(root: &Path, scene: &Scene, build: &Path) -> Result<PathBuf, String> {
    stage_with_origin(
        root,
        scene,
        build,
        &crate::scene_dependencies::Origin::Anonymous(crate::scene_dependencies::signature(scene)),
    )
}
pub fn stage_with_origin(
    root: &Path,
    scene: &Scene,
    build: &Path,
    origin: &crate::scene_dependencies::Origin,
) -> Result<PathBuf, String> {
    stage_sources(root, scene, build, origin, scene, None)
}

pub fn stage_prepared(
    root: &Path,
    input: &crate::scene_dependencies::Input,
    prepared: &Scene,
    build: &Path,
) -> Result<PathBuf, String> {
    stage_sources(
        root,
        prepared,
        build,
        &input.origin,
        &input.scene,
        Some(input),
    )
}

fn stage_sources(
    root: &Path,
    scene: &Scene,
    build: &Path,
    origin: &crate::scene_dependencies::Origin,
    authored: &Scene,
    input: Option<&crate::scene_dependencies::Input>,
) -> Result<PathBuf, String> {
    let mut playback = crate::playback_staging::Batch::new(root, build)?;
    playback.scene_source(root, origin)?;
    stage_with_playback(root, scene, build, playback, (origin, authored), input).map_err(|error| {
        match crate::playback_staging::invalidate(root, build, &error) {
            Ok(()) => error,
            Err(provenance) => format!("{error}\n{provenance}"),
        }
    })
}
fn stage_with_playback(
    root: &Path,
    scene: &Scene,
    build: &Path,
    mut playback: crate::playback_staging::Batch,
    authored: (&crate::scene_dependencies::Origin, &Scene),
    input: Option<&crate::scene_dependencies::Input>,
) -> Result<PathBuf, String> {
    let written = crate::staging_files::Capture::begin(build)?;
    let mut resolved = scene.clone();
    // Parent template defaults may replace a removed resource. Refresh the
    // linked data before resolving the obsolete serialized instance resources.
    refresh_linked_scene(root, &mut resolved)?;
    let rendering = crate::settings::rendering(root)?;
    let debug = crate::settings::debug_hud(root)?;
    playback.scene_input(
        "scene-debug-settings".into(),
        crate::scene_dependencies::hash(debug),
    )?;
    write_changed(&build.join("debug-hud.hh"), debug.header().as_bytes())?;
    let lua = crate::settings::lua_execution(root)?;
    let lua_profile = crate::settings::lua_profile(root)?;
    playback.scene_input(
        "scene-lua-settings".into(),
        crate::scene_dependencies::hash((lua, lua_profile, lua_profile.version())),
    )?;
    let mut lua_config = lua.header();
    lua_config.push_str(&format!(
        "#define EPOK_LUA_PROFILE {}\n#define EPOK_LUA_ABI_VERSION {}\n",
        lua_profile.version(),
        lua_profile.version()
    ));
    write_changed(&build.join("lua-config.hh"), lua_config.as_bytes())?;
    let profile = input.and_then(|i| i.play.as_ref());
    if let Some(signature) = input.and_then(|i| i.play_settings_signature.as_ref()) {
        playback.scene_input("scene-play-settings".into(), signature.clone())?;
    }
    let transition = crate::workspace::optional_manifest(root)?
        .map(|m| m.transition)
        .unwrap_or_default();
    transition.validate()?;
    playback.scene_input(
        "scene-transition-settings".into(),
        crate::scene_dependencies::hash(&transition),
    )?;
    write_changed(&build.join("transition-config.hh"), format!("#pragma once\n#define EPOK_TRANSITIONS 1\n#define EPOK_FADE_OUT_MS {}\n#define EPOK_FADE_IN_MS {}\n#define EPOK_LOADING_TEXT {}\n", transition.fade_out_ms, transition.fade_in_ms, serde_json::to_string(&transition.text).unwrap()).as_bytes())?;
    write_changed(
        &build.join("data-config.hh"),
        format!(
            "#pragma once\n#define EPOK_HOST_DATA {}\n#define EPOK_SERIAL_DEBUG {}\n",
            u8::from(profile.is_some_and(|p| p.data == crate::play::DataSource::Host)),
            u8::from(profile.is_some_and(|p| p.target == crate::play::Target::Serial))
        )
        .as_bytes(),
    )?;
    playback.scene_input(
        "scene-render-settings".into(),
        crate::scene_dependencies::hash(rendering),
    )?;
    resolved.display_size = [rendering.width, rendering.height];
    let display = rendering.header()?;
    write_changed(&build.join("display.hh"), display.as_bytes())?;
    playback.resources(vec![crate::playback_staging::ResourceOutput {
        path: "display.hh".into(),
        signature: crate::assets::hash(display.as_bytes()),
        inputs: std::collections::BTreeMap::from([(
            "display-settings".into(),
            crate::assets::hash(display.as_bytes()),
        )]),
    }])?;
    if resolved.actors.iter().any(|e| e.editable_mesh.is_some()) {
        crate::mesh::resolve(
            &mut resolved,
            &crate::assets::scan(root, &mut Default::default()),
        )?;
    }
    crate::skeletal::resolve(
        &mut resolved,
        &crate::assets::scan(root, &mut Default::default()),
    )?;
    crate::texture::resolve(
        &mut resolved,
        &crate::assets::scan(root, &mut Default::default()),
    )?;
    let scene = &resolved;
    // A failed script compilation must never leave generated code from the
    // previous mode or revision behind in the staged build.
    let catalog = scripts::catalog(root).inspect_err(|_| {
        let _ = fs::remove_dir_all(build.join("scripts/generated/lua"));
    })?;
    // Resource selection must see engine components too, not only classes
    // inherited by project scripts. Otherwise their numeric properties are
    // conservatively mistaken for unresolved audio inputs during staging.
    let blueprint_registry = crate::blueprint::native_registry(root, &catalog)?;
    // Template refresh can replace resource defaults. Observe the submitted
    // document with the same types used to select the actual cooked resources.
    playback.scene_audio_source(root, authored.0, authored.1, &blueprint_registry)?;
    let loaded = crate::scene_bank::load_selected(root, scene, authored.0, input)?;
    for (source, bank) in loaded.sources.iter().zip(loaded.scenes.iter().skip(1)) {
        playback.scene_source(root, source)?;
        playback.scene_audio_source(root, source, bank, &blueprint_registry)?;
    }
    if profile.is_none_or(|p| p.content == crate::play::Content::WholeGame) {
        playback.scene_input(
            "scene-registry".into(),
            crate::scene_dependencies::hash(crate::scene_bank::read(root)?),
        )?;
    }
    if profile.is_some_and(|p| p.content == crate::play::Content::WholeGame) {
        playback.scene_input(
            "scene-inventory".into(),
            crate::scene_dependencies::hash(crate::scene_bank::available(root)?),
        )?;
    }
    playback.scene_catalog(root, &catalog)?;
    let mut banks = loaded.scenes;
    let asset_index = crate::assets::scan(root, &mut Default::default());
    playback.resources(vec![transition.stage_image(root, build, &asset_index)?])?;
    let lua_files = crate::lua_asset::load_all(root)?;
    playback.scene_input(
        "lua-sources".into(),
        crate::lua_dependencies::source_set(&lua_files),
    )?;
    playback.scene_input("lua-mode".into(), lua.signature())?;
    let blueprint_files = crate::blueprint_asset::load_all(root)?;
    playback.scene_input(
        "blueprint-sources".into(),
        crate::blueprint_dependencies::source_set(&blueprint_files),
    )?;
    for file in &blueprint_files {
        playback.blueprint_audio_source(file, Some(&blueprint_registry))?;
        playback.scene_input(
            format!("blueprint:{}", file.asset.id),
            file.asset.semantic_hash(),
        )?;
    }
    let templates = crate::blueprint_spawn::prepare(root, &catalog, &asset_index)?;
    let mut hud_budget = scene.hud_budget.clone();
    for bank in &mut banks {
        crate::blueprint_templates::refresh_instances(bank, &blueprint_files, &blueprint_registry)?;
        bank.display_size = scene.display_size;
        crate::mesh::resolve(bank, &asset_index)?;
        crate::skeletal::resolve(bank, &asset_index)?;
        crate::texture::resolve(bank, &asset_index)?;
        bank.validate()?;
        if !rendering.streaming_geometry
            && bank
                .actors
                .iter()
                .map(crate::lighting::quad_count)
                .sum::<usize>()
                > 3500
        {
            return Err(format!(
                "Scene {} exceeds 7000 resident triangles. Enable Engine > Streaming > Geometry Streaming for editable meshes, or reduce geometry/subdivisions.",
                bank.name
            ));
        }
        if profile.is_some_and(|p| p.target == crate::play::Target::Serial) {
            crate::audio::validate_assets_with_music(bank, &asset_index, false)?;
        } else {
            crate::audio::validate_assets(bank, &asset_index)?;
        }
        hud_budget.layouts = hud_budget.layouts.max(bank.hud_budget.layouts);
        hud_budget.rectangles = hud_budget.rectangles.max(bank.hud_budget.rectangles);
        hud_budget.texts = hud_budget.texts.max(bank.hud_budget.texts);
        hud_budget.glyphs = hud_budget.glyphs.max(bank.hud_budget.glyphs);
    }
    let mut timeline_scenes = banks.clone();
    timeline_scenes.extend(templates.iter().map(|t| t.scene.clone()));
    let (direct_timelines, direct_effects) =
        crate::blueprint_playback::references(&blueprint_files)?;
    let timelines = crate::timeline_scene::prepare(
        root,
        &timeline_scenes,
        &blueprint_registry,
        &direct_timelines,
    )?;
    let needs_native_classes = timeline_scenes
        .iter()
        .any(|scene| scene.scene_script.is_some() || !scene.actors.is_empty());
    let effect_registry = if needs_native_classes
        || !direct_effects.is_empty()
        || timeline_scenes
            .iter()
            .any(|scene| scene.actors.iter().any(|e| e.particle_effect.is_some()))
    {
        crate::blueprint::native_registry(root, &catalog)?
    } else {
        blueprint_registry.clone()
    };
    // The headless path refreshes the same catalog the editor does, so it keeps
    // the generated Lua definitions current. They are editor tooling: a failed
    // write is reported and the build continues.
    if let Err(error) = crate::lua_api_stub::write(root, &effect_registry) {
        eprintln!("Lua API definitions: {error}");
    }
    let effects = crate::particle_effect_scene::prepare(
        root,
        &timeline_scenes,
        &effect_registry,
        &direct_effects,
    )?;
    crate::timeline_runtime::validate_ids(
        timelines
            .iter()
            .map(|p| &p.compiled)
            .chain(effects.iter().map(|p| &p.compiled)),
        &effect_registry,
    )?;
    let stream = if rendering.streaming_geometry {
        Some(crate::streaming::compile_with_budget(
            &banks,
            rendering.streaming_pool_pages as usize,
            usize::from(rendering.streaming_triangle_budget),
        )?)
    } else {
        None
    };
    if let Some(bundle) = &stream {
        bundle.write(build)?;
    } else {
        crate::streaming::clear(build)?;
    }
    let mut resource_scenes = banks.clone();
    for template in &templates {
        resource_scenes.push(template.scene.clone());
        hud_budget.layouts = hud_budget.layouts.max(template.scene.hud_budget.layouts);
        hud_budget.rectangles = hud_budget
            .rectangles
            .max(template.scene.hud_budget.rectangles);
        hud_budget.texts = hud_budget.texts.max(template.scene.hud_budget.texts);
        hud_budget.glyphs = hud_budget.glyphs.max(template.scene.hud_budget.glyphs);
    }
    let playback_sources = || {
        timelines
            .iter()
            .map(|t| &t.source)
            .chain(effects.iter().map(|e| &e.source.timeline))
    };
    for source in playback_sources() {
        playback.timeline_audio_source(source)?;
    }
    let mut extra_resources = playback_sources()
        .flat_map(crate::timeline::resources)
        .map(|(ty, value)| (ty.clone(), value.clone()))
        .collect::<Vec<_>>();
    extra_resources.extend(crate::particle_effect_scene::resources(&effects));
    let referenced = crate::blueprint_refs::resources(
        &blueprint_files,
        &resource_scenes,
        &blueprint_registry,
        &asset_index,
        &extra_resources,
    )?;
    // Scene actor records may refer directly to SDK-native classes. Those classes are
    // deliberately absent from the project script catalog, so use the authoritative
    // reflection registry for both validation and the cooked runtime class table.
    let class_registry = if needs_native_classes {
        &effect_registry
    } else {
        &blueprint_registry
    };
    let mut artifacts = crate::script_backend::prepare_all(root, &scripts::native_catalog(root)?)?;
    let skeletal_queries = crate::skeletal_compile::QueryDemand {
        vertices: artifacts
            .runtime_capabilities
            .contains("skeletal-vertex-query"),
        bones: artifacts
            .runtime_capabilities
            .contains("skeletal-bone-query"),
    };
    let generated = crate::scene_bank::header_with_templates_for(
        &banks,
        &catalog,
        stream.as_ref(),
        &templates,
        Some(&referenced),
        &timelines,
        &effects,
        class_registry,
        skeletal_queries,
    )?;
    resource_scenes.push(referenced);
    let audio_outputs = if profile.is_some_and(|p| p.target == crate::play::Target::Serial) {
        crate::audio::stage_with_music(
            root,
            &crate::scene_bank::resources(&resource_scenes),
            build,
            &asset_index,
            false,
        )?
    } else {
        crate::audio::stage(
            root,
            &crate::scene_bank::resources(&resource_scenes),
            build,
            &asset_index,
        )?
    };
    if let Some(profile) = profile {
        if profile.data == crate::play::DataSource::Executable && build.join("disc.xml").is_file() {
            return Err("This build requires external geometry or XA music. Choose CD on demand / PC on demand, or disable Geometry Streaming and use resident sound effects.".into());
        }
        if profile.data == crate::play::DataSource::Host
            && fs::read_to_string(build.join("audio-bank.hh")).is_ok_and(|s| s.contains(".XA;1"))
        {
            return Err("XA music requires the physical CD decoder and cannot be served by PCDrv. Use CD on demand for XA music.".into());
        }
    }
    crate::hud::stage(build, &hud_budget)?;
    stage_runtime(build)?;
    for timeline in &timelines {
        let header = crate::timeline_runtime::header(&timeline.compiled, &blueprint_registry)?;
        playback.timeline(&timeline.compiled, header.as_bytes())?;
        write_changed(
            &build.join(format!("timelines/{}.hh", timeline.compiled.asset)),
            header.as_bytes(),
        )?;
    }
    let shared_resources = crate::scene_bank::resources(&resource_scenes);
    crate::memory::stage(
        root,
        build,
        &banks,
        &shared_resources,
        &asset_index,
        !templates.is_empty() || resource_scenes.last().is_some_and(|s| !s.actors.is_empty()),
        &catalog,
        &audio_outputs,
    )?;
    playback.resources(audio_outputs)?;
    playback.scene_resources(&shared_resources, &asset_index)?;
    for effect in &effects {
        let header =
            crate::particle_effect_scene::header(effect, &effect_registry, &shared_resources)?;
        playback.effect(effect, &shared_resources, &asset_index, header.as_bytes())?;
        write_changed(
            &build.join(format!("effects/{}.hh", effect.source.id)),
            header.as_bytes(),
        )?;
    }
    // Override properties may have no track in the embedded timeline, but their
    // typed assignments in scene.hh still consume explicit reflection contracts.
    for scene in &timeline_scenes {
        for component in scene
            .actors
            .iter()
            .filter_map(|e| e.particle_effect.as_ref())
        {
            for properties in component.layer_overrides.values() {
                for property in properties.keys() {
                    let key = format!(
                        "property:{}:{property}",
                        crate::particle_effect::LAYER_CLASS_ID
                    );
                    let signature =
                        crate::timeline_compile::reflection_dependency(&key, &effect_registry)
                            .ok_or_else(|| {
                                format!("Missing effect override reflection dependency {key}")
                            })?;
                    playback.scene_input(key, signature)?;
                }
            }
        }
    }
    if !direct_timelines.is_empty() || !direct_effects.is_empty() {
        let path = PathBuf::from("scripts/generated/playback_calls.cpp");
        let source = format!(
            "// Generated typed asset plays. Definitions follow the complete cooked scene types.\n#include \"scene.hh\"\n{}",
            crate::blueprint_playback::definitions(&timelines, &effects)
        );
        artifacts.files.insert(path.clone(), source.into_bytes());
        artifacts.native_sources.push(path);
    }
    // Generated Lua output is mode-scoped. A class that was removed, renamed or
    // rebuilt in another execution mode must not leave its previous artifact
    // behind: an AOT header or a VM bindings source has to disappear with it.
    let generated_lua = build.join("scripts/generated/lua");
    if generated_lua.is_dir() {
        for entry in fs::read_dir(&generated_lua).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            let stale = path
                .strip_prefix(build)
                .is_ok_and(|relative| !artifacts.files.contains_key(relative));
            if stale && path.is_file() {
                fs::remove_file(&path).map_err(|e| e.to_string())?;
            }
        }
    }
    artifacts.stage(root, build)?;
    playback.scripts(&artifacts)?;
    for (key, signature) in written.inputs() {
        playback.scene_input(key, signature)?;
    }
    let native_sources = artifacts
        .native_sources
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect::<Vec<_>>();
    // An explicit source list ensures removed scripts are not linked from the cache.
    let needs_effects = !effects.is_empty() || artifacts.runtime_capabilities.contains("effect");
    let needs_timelines = !timelines.is_empty()
        || artifacts.runtime_capabilities.contains("timeline")
        || needs_effects;
    let blueprint_flags = if needs_effects {
        "CPPFLAGS += -DEPOK_BLUEPRINTS -DEPOK_TIMELINES -DEPOK_EFFECTS\n"
    } else if needs_timelines {
        "CPPFLAGS += -DEPOK_BLUEPRINTS -DEPOK_TIMELINES\n"
    } else if artifacts.runtime_capabilities.contains("blueprint") {
        "CPPFLAGS += -DEPOK_BLUEPRINTS\n"
    } else {
        ""
    };
    let playback_flags = if artifacts.runtime_capabilities.contains("playback_wait") {
        "CPPFLAGS += -DEPOK_PLAYBACK_WAITS\n"
    } else {
        ""
    };
    let sources = format!(
        "SCRIPT_SRCS := {}\n{blueprint_flags}{playback_flags}{}{}",
        native_sources.join(" "),
        lua.cppflags(),
        lua.libraries()
    );
    write_changed(&build.join("sources.mk"), sources.as_bytes())?;
    write_changed(&build.join("scene.hh"), generated.as_bytes())?;
    playback.scene_header(generated.as_bytes());
    write_changed(
        &build.join("scene.epokmap"),
        &crate::document::to_vec(scene).map_err(|e| e.to_string())?,
    )?;
    playback.audio_bank_inputs();
    playback.files(written.finish())?;
    playback.publish(root)?;
    Ok(build.to_path_buf())
}
pub fn fingerprint(root: &Path) -> Result<u64, String> {
    fingerprint_inputs(root, true)
}
/// Settings have their own dependency nodes. They do not change C++/Blueprint
/// declarations and must not trigger an editor script/scene refresh.
pub fn source_fingerprint(root: &Path) -> Result<u64, String> {
    fingerprint_inputs(root, false)
}

/// The same embedded runtime snapshot is used by reflection, staging, and export.
pub fn stage_runtime(build: &Path) -> Result<(), String> {
    // Snapshot inputs so an edit during compilation is picked up by the next build.
    for (file, contents) in runtime_sources() {
        write_changed(&build.join(file), contents)?;
    }
    Ok(())
}
pub fn runtime_sources() -> &'static [(&'static str, &'static [u8])] {
    static SOURCES: &[(&str, &[u8])] = &[
        ("navigation.hpp", include_bytes!("../runtime/navigation.hpp").as_slice()),
        ("navigation_components.hpp", include_bytes!("../runtime/navigation_components.hpp").as_slice()),
        (
            "gameplay_api.hpp",
            include_bytes!("../runtime/gameplay_api.hpp").as_slice(),
        ),
        (
            "object_model.hpp",
            include_bytes!("../runtime/object_model.hpp").as_slice(),
        ),
        (
            "world2d.hpp",
            include_bytes!("../runtime/world2d.hpp").as_slice(),
        ),
        (
            "actor_blueprint.hpp",
            include_bytes!("../runtime/actor_blueprint.hpp").as_slice(),
        ),
        (
            "actor_tables.hpp",
            include_bytes!("../runtime/actor_tables.hpp").as_slice(),
        ),
        (
            "hud_core.hpp",
            include_bytes!("../runtime/hud_core.hpp").as_slice(),
        ),
        (
            "serial_kernel.hpp",
            include_bytes!("../runtime/serial_kernel.hpp").as_slice(),
        ),
        (
            "serial_debug.hpp",
            include_bytes!("../runtime/serial_debug.hpp").as_slice(),
        ),
        (
            "debug_hud.hpp",
            include_bytes!("../runtime/debug_hud.hpp").as_slice(),
        ),
        (
            "frame_clear.hpp",
            include_bytes!("../runtime/frame_clear.hpp").as_slice(),
        ),
        (
            "transition.hpp",
            include_bytes!("../runtime/transition.hpp").as_slice(),
        ),
        (
            "loading_renderer.hpp",
            include_bytes!("../runtime/loading_renderer.hpp").as_slice(),
        ),
        (
            "playback_types.hpp",
            include_bytes!("../runtime/playback_types.hpp").as_slice(),
        ),
        (
            "blueprint_playback_service.hpp",
            include_bytes!("../runtime/blueprint_playback_service.hpp").as_slice(),
        ),
        (
            "particle_effect_runtime.hpp",
            include_bytes!("../runtime/particle_effect_runtime.hpp").as_slice(),
        ),
        (
            "particle_effect_service.hpp",
            include_bytes!("../runtime/particle_effect_service.hpp").as_slice(),
        ),
        (
            "effect_types.hpp",
            include_bytes!("../runtime/effect_types.hpp").as_slice(),
        ),
        (
            "timeline_service.hpp",
            include_bytes!("../runtime/timeline_service.hpp").as_slice(),
        ),
        (
            "timeline_runtime.hpp",
            include_bytes!("../runtime/timeline_runtime.hpp").as_slice(),
        ),
        (
            "timeline.hpp",
            include_bytes!("../runtime/timeline.hpp").as_slice(),
        ),
        (
            "blueprint_debug.hpp",
            include_bytes!("../runtime/blueprint_debug.hpp").as_slice(),
        ),
        (
            "blueprint_template.hpp",
            include_bytes!("../runtime/blueprint_template.hpp").as_slice(),
        ),
        (
            "blueprint_api.hpp",
            include_bytes!("../runtime/blueprint_api.hpp").as_slice(),
        ),
        (
            "blueprint_spawn.hpp",
            include_bytes!("../runtime/blueprint_spawn.hpp").as_slice(),
        ),
        (
            "blueprint_runtime.hpp",
            include_bytes!("../runtime/blueprint_runtime.hpp").as_slice(),
        ),
        ("main.cpp", include_bytes!("../runtime/main.cpp").as_slice()),
        (
            "visibility.hpp",
            include_bytes!("../runtime/visibility.hpp").as_slice(),
        ),
        (
            "streaming.hpp",
            include_bytes!("../runtime/streaming.hpp").as_slice(),
        ),
        (
            "streaming_pool.hpp",
            include_bytes!("../runtime/streaming_pool.hpp").as_slice(),
        ),
        (
            "effects.hpp",
            include_bytes!("../runtime/effects.hpp").as_slice(),
        ),
        (
            "input.hpp",
            include_bytes!("../runtime/input.hpp").as_slice(),
        ),
        (
            "memory_card.hpp",
            include_bytes!("../runtime/memory_card.hpp").as_slice(),
        ),
        (
            "memory_card_backend.hpp",
            include_bytes!("../runtime/memory_card_backend.hpp").as_slice(),
        ),
        (
            "utility.hpp",
            include_bytes!("../runtime/utility.hpp").as_slice(),
        ),
        ("time.hpp", include_bytes!("../runtime/time.hpp").as_slice()),
        (
            "collision.hpp",
            include_bytes!("../runtime/collision.hpp").as_slice(),
        ),
        (
            "palette.hpp",
            include_bytes!("../runtime/palette.hpp").as_slice(),
        ),
        (
            "palette_types.hpp",
            include_bytes!("../runtime/palette_types.hpp").as_slice(),
        ),
        (
            "lifecycle.hpp",
            include_bytes!("../runtime/lifecycle.hpp").as_slice(),
        ),
        (
            "scene_service.hpp",
            include_bytes!("../runtime/scene_service.hpp").as_slice(),
        ),
        (
            "texture.hpp",
            include_bytes!("../runtime/texture.hpp").as_slice(),
        ),
        (
            "sprites.hpp",
            include_bytes!("../runtime/sprites.hpp").as_slice(),
        ),
        (
            "sprite_math.hpp",
            include_bytes!("../runtime/sprite_math.hpp").as_slice(),
        ),
        (
            "sprite_types.hpp",
            include_bytes!("../runtime/sprite_types.hpp").as_slice(),
        ),
        (
            "particles.hpp",
            include_bytes!("../runtime/particles.hpp").as_slice(),
        ),
        (
            "particle_types.hpp",
            include_bytes!("../runtime/particle_types.hpp").as_slice(),
        ),
        (
            "texture_types.hpp",
            include_bytes!("../runtime/texture_types.hpp").as_slice(),
        ),
        (
            "resources.hpp",
            include_bytes!("../runtime/resources.hpp").as_slice(),
        ),
        (
            "music.hpp",
            include_bytes!("../runtime/music.hpp").as_slice(),
        ),
        (
            "audio.hpp",
            include_bytes!("../runtime/audio.hpp").as_slice(),
        ),
        (
            "spu_transfer.hpp",
            include_bytes!("../runtime/spu_transfer.hpp").as_slice(),
        ),
        (
            "sequence_kernel.hpp",
            include_bytes!("../runtime/sequence_kernel.hpp").as_slice(),
        ),
        (
            "sequence_data.hpp",
            include_bytes!("../runtime/sequence_data.hpp").as_slice(),
        ),
        (
            "native_music_data.hpp",
            include_bytes!("../runtime/native_music_data.hpp").as_slice(),
        ),
        (
            "native_music_service.hpp",
            include_bytes!("../runtime/native_music_service.hpp").as_slice(),
        ),
        (
            "native_music_runtime.hpp",
            include_bytes!("../runtime/native_music_runtime.hpp").as_slice(),
        ),
        (
            "sequence_tables.hpp",
            include_bytes!("../runtime/sequence_tables.hpp").as_slice(),
        ),
        (
            "sequence_lock.hpp",
            include_bytes!("../runtime/sequence_lock.hpp").as_slice(),
        ),
        (
            "sequence_service.hpp",
            include_bytes!("../runtime/sequence_service.hpp").as_slice(),
        ),
        (
            "sequence_instrument_service.hpp",
            include_bytes!("../runtime/sequence_instrument_service.hpp").as_slice(),
        ),
        (
            "instrument_bank.hpp",
            include_bytes!("../runtime/instrument_bank.hpp").as_slice(),
        ),
        (
            "instrument_allocator.hpp",
            include_bytes!("../runtime/instrument_allocator.hpp").as_slice(),
        ),
        (
            "instrument_synth.hpp",
            include_bytes!("../runtime/instrument_synth.hpp").as_slice(),
        ),
        (
            "instrument_preparation.hpp",
            include_bytes!("../runtime/instrument_preparation.hpp").as_slice(),
        ),
        (
            "instrument_reverb.hpp",
            include_bytes!("../runtime/instrument_reverb.hpp").as_slice(),
        ),
        (
            "sequence_clock.hpp",
            include_bytes!("../runtime/sequence_clock.hpp").as_slice(),
        ),
        ("epok.hpp", include_bytes!("../runtime/epok.hpp").as_slice()),
        (
            "affine.hpp",
            include_bytes!("../runtime/affine.hpp").as_slice(),
        ),
        (
            "transform_cache.hpp",
            include_bytes!("../runtime/transform_cache.hpp").as_slice(),
        ),
        (
            "motion_interpolation.hpp",
            include_bytes!("../runtime/motion_interpolation.hpp").as_slice(),
        ),
        (
            "frustum.hpp",
            include_bytes!("../runtime/frustum.hpp").as_slice(),
        ),
        (
            "gte_geometry.hpp",
            include_bytes!("../runtime/gte_geometry.hpp").as_slice(),
        ),
        (
            "polygon.hpp",
            include_bytes!("../runtime/polygon.hpp").as_slice(),
        ),
        (
            "retained.hpp",
            include_bytes!("../runtime/retained.hpp").as_slice(),
        ),
        ("hud.hpp", include_bytes!("../runtime/hud.hpp").as_slice()),
        (
            "skeletal.hpp",
            include_bytes!("../runtime/skeletal.hpp").as_slice(),
        ),
        (
            "lighting.hpp",
            include_bytes!("../runtime/lighting.hpp").as_slice(),
        ),
        (
            "shadows.hpp",
            include_bytes!("../runtime/shadows.hpp").as_slice(),
        ),
        (
            "lua_runtime.hpp",
            include_bytes!("../runtime/lua_runtime.hpp").as_slice(),
        ),
        ("lua.mk", include_bytes!("../runtime/lua.mk").as_slice()),
        ("Makefile", include_bytes!("../runtime/Makefile").as_slice()),
        (
            "build-inputs.mk",
            include_bytes!("../runtime/build-inputs.mk").as_slice(),
        ),
        (
            "build.ps1",
            include_bytes!("../runtime/build.ps1").as_slice(),
        ),
        ("build.sh", include_bytes!("../runtime/build.sh").as_slice()),
    ];
    SOURCES
}

fn fingerprint_inputs(root: &Path, include_settings: bool) -> Result<u64, String> {
    let mut files = crate::reflection::script_files(root)?;
    // Canvas layout is authoring-only. Moving a node must not restart a PSX build.
    files.retain(|path| !path.to_string_lossy().ends_with(".epokbp"));
    if include_settings && let Ok(manifest) = crate::workspace::manifest_path(root) {
        files.push(manifest);
    }
    for name in [crate::scene_bank::REGISTRY, "Local.epokconfig"] {
        if name == crate::scene_bank::REGISTRY && !include_settings {
            continue;
        }
        let p = root.join(name);
        if p.exists() {
            files.push(p);
        }
    }
    let home = crate::workspace::editor_home();
    for name in ["Editor.epokconfig", "Local.epokconfig"] {
        let path = home.join(name);
        if path.exists() {
            files.push(path);
        }
    }
    files.sort();
    let mut hash = DefaultHasher::new();
    for asset in crate::blueprint_asset::load_all(root)? {
        asset.path.hash(&mut hash);
        asset.asset.semantic_hash().hash(&mut hash);
    }
    // Playback assets have their own source observation history and use the
    // selected stage's existing dependency edges to decide whether to restart.
    for path in files {
        path.hash(&mut hash);
        fs::read(path).map_err(|e| e.to_string())?.hash(&mut hash);
    }
    Ok(hash.finish())
}
/// Cooked `ActorTable` for one scene bank (design.md section 7).
///
/// Emitted inside the bank's namespace, after the legacy tables, so the shape of the
/// existing generated text is unchanged. A bank with no actors and no scene script still
/// emits the empty table: `main.cpp` and `load_bank_N()` then link against exactly the
/// same symbols whatever the map contains.
///
/// Nothing here is persisted: `properties`/`overrides` come from the document, every
/// index is derived from the order of `scene.actors`, and class identities come from the
/// resolved `object_model::Model`.
fn actor_table_with_registry(
    scene: &Scene,
    registry: &crate::blueprint::Registry,
    clips: &[uuid::Uuid],
) -> Result<String, String> {
    if scene.actors.is_empty() && scene.scene_script.is_none() {
        return Ok(
            "inline constexpr ActorTable actor_table={nullptr,0,UINT64_C(0)};\ninline constexpr uint64_t scene_script_class=UINT64_C(0);\ninline constexpr size_t actor_registry_slots=0;\n"
                .into(),
        );
    }
    let model = registry.model().map_err(|diagnostics| {
        format!(
            "{}: cooking actors needs a resolved class model\n{}",
            scene.name,
            diagnostics
                .iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("\n")
        )
    })?;
    // Parents before children: Level::spawn_batch can only bind a logical parent whose
    // ObjectId already exists, so the runtime loader walks the table in this order.
    let mut order: Vec<usize> = (0..scene.actors.len()).collect();
    order.sort_by_key(|i| {
        let mut depth = 0;
        let mut parent = scene.actors[*i].logical_parent;
        while let Some(id) = parent {
            depth += 1;
            if depth > 64 {
                break;
            }
            parent = scene
                .actors
                .iter()
                .find(|actor| actor.id == id)
                .and_then(|actor| actor.logical_parent);
        }
        depth
    });
    let slot = |id: uuid::Uuid| order.iter().position(|i| scene.actors[*i].id == id);
    let mut components_text = String::new();
    let mut applies = String::new();
    let mut rows = Vec::new();
    let mut registry_slots = 0usize;
    for (index, source) in order.iter().enumerate() {
        let actor = &scene.actors[*source];
        let class = actor.class.resolve(&model).ok_or_else(|| {
            format!(
                "{}: actor {} uses unknown class {}",
                scene.name, actor.name, actor.class.name
            )
        })?;
        registry_slots += 1 + actor.components.len();
        let mut setters = String::new();
        setters.push_str(&property_setters(
            &format!("registry.resolve<{}>(actor.id())", class.cpp_name),
            "self",
            &actor.properties,
            &actor.overrides,
            &class.cpp_name,
            &registry,
            scene,
            &format!("{}: actor {}", scene.name, actor.name),
        )?);
        let mut records = Vec::new();
        for (position, component) in actor.components.iter().enumerate() {
            let component_class = component.class.resolve(&model).ok_or_else(|| {
                format!(
                    "{}: actor {} component {} uses unknown class {}",
                    scene.name, actor.name, component.name, component.class.name
                )
            })?;
            let data_slot = *source as i32;
            let default_index = component
                .default_id
                .as_ref()
                .map(|id| {
                    class
                        .default_components
                        .iter()
                        .position(|c| &c.id == id)
                        .map(|i| i as i16)
                        .ok_or_else(|| {
                            format!(
                                "{}: native default component {id} no longer exists",
                                actor.name
                            )
                        })
                })
                .transpose()?
                .unwrap_or(-1);
            let attach_parent = component
                .attach_parent
                .and_then(|id| actor.components.iter().position(|c| c.id == id))
                .map_or(-1, |i| i as i32);
            records.push(format!(
                "{{UINT64_C({}),{},{},{attach_parent},{data_slot},{default_index}}}",
                crate::blueprint_refs::compact_id(&component_class.id),
                serde_json::to_string(&component.name).unwrap(),
                component.root
            ));
            if component_class.id == crate::object_model::SCENE_COMPONENT2D_ID {
                let p = &component.properties;
                let position_value = p
                    .get("position")
                    .cloned()
                    .unwrap_or(serde_json::json!([0, 0]));
                let scale = p.get("scale").cloned().unwrap_or(serde_json::json!([1, 1]));
                let rotation = p.get("rotation").cloned().unwrap_or(serde_json::json!(0));
                let ty = crate::reflection_schema::Type::Vector { length: 2 };
                let body = crate::script_values::assignment(
                    "component->transform.position",
                    &position_value,
                    &ty,
                )? + &crate::script_values::assignment(
                    "component->transform.scale",
                    &scale,
                    &ty,
                )? + &crate::script_values::assignment(
                    "component->transform.rotation",
                    &rotation,
                    &crate::reflection_schema::Type::Fixed,
                )?;
                setters += &format!(
                    "if(auto* component=registry.resolve<epok::SceneComponent2D>(components[{position_index}])){{{body}}}\n",
                    position_index = position
                );
            }
            if component_class.id == crate::object_model::AUDIO_COMPONENT_ID {
                let audio: crate::audio::AudioSource = component
                    .properties
                    .get("audio")
                    .map(|v| serde_json::from_value(v.clone()))
                    .transpose()
                    .map_err(|e| e.to_string())?
                    .unwrap_or_default();
                let clip = audio
                    .clip
                    .map(|id| {
                        clips
                            .iter()
                            .position(|v| *v == id)
                            .map(|i| i as i32)
                            .ok_or_else(|| {
                                format!(
                                    "{}: Audio clip {id} is missing from the resource bank",
                                    actor.name
                                )
                            })
                    })
                    .transpose()?
                    .unwrap_or(-1);
                let fixed = |v: f32| (v as f64 * 4096.).round() as i32;
                setters += &format!(
                    "if(auto* component=registry.resolve<epok::AudioComponent>(components[{position}])){{if(auto* source=component->source){{source->enabled=true;source->clip={clip};source->volume=epok::Fixed({},epok::Fixed::RAW);source->pitch=epok::Fixed({},epok::Fixed::RAW);source->play_on_start={};source->priority={};}}}}\n",
                    fixed(audio.volume),
                    fixed(audio.pitch),
                    audio.play_on_start,
                    audio.priority
                );
            }
            if !crate::actor_components::native(&component_class.id) {
                setters.push_str(&property_setters(
                    &format!(
                        "registry.resolve<{}>(components[{position}])",
                        component_class.cpp_name
                    ),
                    "component",
                    &component.properties,
                    &component.overrides,
                    &component_class.cpp_name,
                    &registry,
                    scene,
                    &format!(
                        "{}: actor {} component {}",
                        scene.name, actor.name, component.name
                    ),
                )?);
            }
        }
        let components_symbol = if records.is_empty() {
            "nullptr".to_string()
        } else {
            components_text.push_str(&format!(
                "inline constexpr ActorComponentRecord actor_components_{index}[]={{{}}};\n",
                records.join(",")
            ));
            format!("actor_components_{index}")
        };
        let apply = if setters.is_empty() {
            "nullptr".to_string()
        } else {
            applies.push_str(&format!(
                "inline void actor_apply_{index}(ObjectRegistry& registry,Actor& actor,const ObjectId* components,const ObjectId* actors){{\n(void)registry;(void)actor;(void)components;(void)actors;\n{setters}}}\n"
            ));
            format!("&actor_apply_{index}")
        };
        let attach_actor = actor
            .attach
            .as_ref()
            .and_then(|attach| slot(attach.actor))
            .map_or(-1, |i| i as i32);
        let attach_component = actor
            .attach
            .as_ref()
            .filter(|_| attach_actor >= 0)
            .and_then(|attach| attach.component)
            .and_then(|id| {
                scene.actors[order[attach_actor as usize]]
                    .components
                    .iter()
                    .position(|c| c.id == id)
            })
            .map_or(-1, |i| i as i32);
        rows.push(format!(
            "{{UINT64_C({}),{},{},{},{attach_actor},{attach_component},{components_symbol},{},{apply}}}",
            crate::blueprint_refs::compact_id(&class.id),
            serde_json::to_string(&actor.name).unwrap(),
            actor.active,
            actor
                .logical_parent
                .and_then(slot)
                .map_or(-1, |i| i as i32),
            actor.components.len()
        ));
    }
    let script_class = match &scene.scene_script {
        Some(script) => {
            let class = script.parent.resolve(&model).ok_or_else(|| {
                format!(
                    "{}: scene script parent {} is not a reflected class",
                    scene.name, script.parent.name
                )
            })?;
            registry_slots += 2;
            crate::blueprint_refs::compact_id(&class.id)
        }
        // Every bank still gets one SceneScriptActor; the loader falls back to the base.
        None => {
            registry_slots += 2;
            0
        }
    };
    let (reference_text, reference_fields) = scene_reference_table(scene, &model, &order)?;
    let mut text = applies;
    text.push_str(&components_text);
    text.push_str(&reference_text);
    if rows.is_empty() {
        text.push_str(&format!(
            "inline constexpr ActorTable actor_table={{nullptr,0,UINT64_C({script_class}){reference_fields}}};\n"
        ));
    } else {
        text.push_str(&format!(
            "inline constexpr ActorRecord actor_records[]={{{}}};\ninline constexpr ActorTable actor_table={{actor_records,{},UINT64_C({script_class}){reference_fields}}};\n",
            rows.join(","),
            rows.len()
        ));
    }
    text.push_str(&format!(
        "inline constexpr uint64_t scene_script_class=UINT64_C({script_class});\ninline constexpr size_t actor_registry_slots={registry_slots};\n"
    ));
    Ok(text)
}

/// Cooked `scene_reference` table for one bank, plus the trailing `ActorTable` fields
/// that point at it.
///
/// P6 lowers every persisted `ActorRef`/`ComponentRef`/`ObjectRef` default of a map's own
/// Blueprint to the null identity and resolves the authored UUID against the map's scope.
/// This turns each resolution into a *table index* — never a UUID and never a name — and
/// emits the generated writer that assigns it, because typed member access needs the C++
/// class. `SceneLevel` binds the rows before the scene script's `begin_play`.
///
/// A map with no map-scoped reference emits nothing at all, so the generated text of every
/// existing project is byte identical.
fn scene_reference_table(
    scene: &Scene,
    model: &crate::object_model::Model,
    order: &[usize],
) -> Result<(String, String), String> {
    let references = crate::blueprint_compile::map_scene_references(Path::new(&scene.name), scene)?;
    if references.is_empty() {
        return Ok((String::new(), String::new()));
    }
    let script = scene.scene_script.as_ref().ok_or_else(|| {
        format!(
            "{}: map-scoped references without a scene script",
            scene.name
        )
    })?;
    let class = model.class(&script.blueprint.id).ok_or_else(|| {
        format!(
            "{}: the scene Blueprint class {} is not in the compiled catalog; its map-scoped references cannot be bound",
            scene.name, script.blueprint.id
        )
    })?;
    let actor_slot = |id: uuid::Uuid| order.iter().position(|i| scene.actors[*i].id == id);
    let mut rows = Vec::new();
    let mut body = String::new();
    for reference in &references {
        let name = reference
            .member
            .strip_prefix("property:")
            .unwrap_or(&reference.member);
        let variable = script
            .blueprint
            .variables
            .iter()
            .find(|variable| variable.name == name)
            .ok_or_else(|| {
                format!(
                    "{}: the scene Blueprint declares no member {name} for its map-scoped reference",
                    scene.name
                )
            })?;
        let member = crate::blueprint_refs::compact_id(&variable.id);
        let missing = || {
            format!(
                "{}: {name} references {}, which is not in this map",
                scene.name, reference.target
            )
        };
        let (kind, actor, component, value) = match reference.kind {
            crate::blueprint_compile::SceneRefKind::Actor => {
                let index = actor_slot(reference.target).ok_or_else(missing)?;
                ("Actor", index as i32, -1, "target")
            }
            crate::blueprint_compile::SceneRefKind::Component => {
                let (actor, position) = scene
                    .actors
                    .iter()
                    .find_map(|actor| {
                        actor
                            .components
                            .iter()
                            .position(|c| c.id == reference.target)
                            .map(|position| (actor.id, position))
                    })
                    .ok_or_else(missing)?;
                let index = actor_slot(actor).ok_or_else(missing)?;
                ("Component", index as i32, position as i32, "target")
            }
        };
        rows.push(format!(
            "{{UINT64_C({}),UINT64_C({member}),SceneRefKind::{kind},{actor},{component}}}",
            crate::blueprint_refs::compact_id(&class.id)
        ));
        body.push_str(&format!(
            "if(member==UINT64_C({member})){{self->{name} = {value};}}\n"
        ));
    }
    let text = format!(
        "inline void scene_reference_bind(ObjectRegistry& registry,Actor& owner,uint64_t member,ObjectId target){{\n(void)registry;(void)owner;(void)member;(void)target;\nif(auto* self=registry.resolve<{}>(owner.id())){{\n{body}}}\n}}\ninline constexpr SceneReferenceRecord scene_references[]={{{}}};\n",
        class.cpp_name,
        rows.join(",")
    );
    Ok((
        text,
        format!(",scene_references,{},&scene_reference_bind", rows.len()),
    ))
}

/// Cooked property overrides for one actor or component, through the same reflected
/// assignment generator the legacy `Binding` setup uses. An authored key the class chain
/// does not declare is a cook diagnostic naming the key, never a silent drop.
#[allow(clippy::too_many_arguments)]
fn property_setters(
    resolve: &str,
    binding: &str,
    properties: &std::collections::BTreeMap<String, serde_json::Value>,
    overrides: &std::collections::BTreeSet<String>,
    class: &str,
    registry: &crate::blueprint::Registry,
    scene: &Scene,
    location: &str,
) -> Result<String, String> {
    let keys: Vec<&String> = if overrides.is_empty() {
        properties.keys().collect()
    } else {
        overrides
            .iter()
            .filter(|k| properties.contains_key(*k))
            .collect()
    };
    if keys.is_empty() {
        return Ok(String::new());
    }
    let reflected = registry.properties(class);
    let mut body = String::new();
    for key in keys {
        let property = reflected
            .iter()
            .find(|p| &p.name == key)
            .ok_or_else(|| format!("{location}: class {class} declares no property {key}"))?;
        body.push_str(&crate::blueprint_refs::assignment(
            &format!("{binding}->{}", property.name),
            &properties[key],
            &property.value_type,
            scene,
            registry,
        )?);
    }
    Ok(format!("if(auto* {binding}={resolve}){{\n{body}}}\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::ClassDefaults;
    #[test]
    fn camera_sky_color_exports_to_the_runtime_clear_setting() {
        let mut scene = Scene::default();
        scene.actors[0].camera_sky_color = [0.25, 0.5, 1.];
        let header = scene_header(&scene, &[]).unwrap();
        assert!(
            header.contains(
                "objects[0].camera_settings={true,Fixed(368640, Fixed::RAW),{64,128,255}};"
            )
        );
    }
    #[test]
    fn background_pass_only_exports_for_selected_actor() {
        let mut scene = Scene::default();
        scene.actors[3].lighting.background_pass = true;
        let header = scene_header(&scene, &[]).unwrap();
        assert!(header.contains(
            "objects[3].lighting=MeshLighting{true,ReceiveLighting::Realtime,false,true,1,true};"
        ));
        assert!(header.contains(
            "objects[1].lighting=MeshLighting{true,ReceiveLighting::Realtime,false,true,1,false};"
        ));
    }
    #[test]
    fn lua_execution_mode_rewrites_sources_and_restores_on_return() {
        use crate::settings::LuaExecution;
        let root = crate::workspace::tests::temp("lua-execution-sources");
        let project =
            crate::workspace::create(&root, "Lua Modes", crate::workspace::Template::Basic)
                .unwrap();
        let mut manifest = project.manifest.clone();
        drop(project);
        let scene =
            crate::scene::Scene::load(&crate::workspace::startup_scene(&root).unwrap()).unwrap();
        let build = root.join(".epok/build");
        let mut emitted = Vec::new();
        for mode in [
            LuaExecution::NativeCpp,
            LuaExecution::VmBytecode,
            LuaExecution::VmSource,
            LuaExecution::NativeCpp,
        ] {
            manifest.lua_execution = mode;
            crate::workspace::save_manifest(&root, &manifest).unwrap();
            stage_into(&root, &scene, &build).unwrap();
            let sources = std::fs::read_to_string(build.join("sources.mk")).unwrap();
            assert!(sources.contains(mode.cppflags()), "{mode:?}: {sources}");
            assert_eq!(
                std::fs::read_to_string(build.join("lua-config.hh")).unwrap(),
                format!(
                    "{}#define EPOK_LUA_PROFILE {}\n#define EPOK_LUA_ABI_VERSION {}\n",
                    mode.header(),
                    manifest.lua_profile.version(),
                    manifest.lua_profile.version()
                )
            );
            // Only the selected mode's runtime is linked; an AOT build links no
            // interpreter and the two VM packagings never share an archive.
            for other in LuaExecution::ALL.into_iter().filter(|o| *o != mode) {
                for line in other
                    .libraries()
                    .lines()
                    .filter(|l| l.starts_with("LIBRARIES +="))
                {
                    assert!(!sources.contains(line), "{mode:?} leaked {line}");
                }
            }
            assert_eq!(sources.contains("EPOK_LUA_VM"), mode.is_vm());
            let linked = sources
                .lines()
                .filter(|l| l.starts_with("LIBRARIES +="))
                .collect::<Vec<_>>();
            assert_eq!(
                linked.iter().any(|l| l.contains("lua/liblua-epok")),
                mode.is_vm(),
                "{mode:?}: {sources}"
            );
            assert!(
                linked.iter().all(|l| !l.contains("libpsyqo-lua")),
                "{mode:?} must not link the wrapper library: {sources}"
            );
            emitted.push(sources);
        }
        // Returning to the original mode restores the original recipe byte for
        // byte, so a round trip cannot leave a stale relink pending.
        assert_eq!(emitted[0], emitted[3]);
        assert_ne!(emitted[0], emitted[1]);
        assert_ne!(emitted[1], emitted[2]);
        std::fs::remove_dir_all(&root).unwrap();
    }
    #[test]
    #[ignore = "requires the configured pinned host extractor and PSX SDK; does not launch the emulator"]
    fn linked_refresh_precedes_obsolete_asset_resolution_and_preserves_overrides() {
        use crate::blueprint_templates as templates;
        let root = std::env::temp_dir().join(format!(
            "epok-linked-resource-order-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(root.join("assets/blueprints")).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(root.clone());
        let mut registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let parent = registry.named("epok::Behaviour").unwrap().clone();
        // Use the real wizard: Behaviour has abstract events, so a bare asset
        // would remain abstract and obscure the resource-order regression.
        let asset_path = crate::blueprint_workflow::create(
            &root,
            &registry,
            "ResourceOwner",
            "",
            &parent.id,
            true,
        )
        .unwrap();
        let mut asset = crate::blueprint_asset::load(&asset_path).unwrap();
        asset.template = templates::Template::root("Linked Root");
        let removed_texture = uuid::Uuid::new_v4();
        asset.template.actors[0].entity.material.texture = Some(removed_texture);
        asset.template.actors[0].entity.audio = Some(crate::audio::AudioSource {
            clip: Some(uuid::Uuid::new_v4()),
            ..Default::default()
        });
        let class = crate::reflection_schema::Class {
            id: asset.id.clone(),
            cpp_name: asset.name.clone(),
            parent: Some(parent.id.clone()),
            provider: crate::blueprint_compile::provider(),
            abstract_class: false,
            ..parent
        };
        registry.classes.insert(class.id.clone(), class.clone());
        let binding = ClassDefaults {
            name: class.cpp_name,
            class_id: Some(class.id),
            provider: class.provider,
            backend: class.backend,
            ..Default::default()
        };
        let mut saved = Scene::default();
        let placed = templates::place(
            &mut saved,
            &templates::resolve(&[&asset.template]).unwrap(),
            binding,
            &registry,
            None,
        )
        .unwrap();
        let before = saved.actors[placed.root].clone();
        saved.actors[placed.root].position = [9., 2., 1.];
        templates::record_overrides(&before, &mut saved.actors[placed.root]);
        let identities = saved
            .actors
            .iter()
            .map(|entity| entity.id)
            .collect::<Vec<_>>();
        let mut stale = saved.clone();
        assert!(
            crate::texture::resolve(&mut stale, &crate::assets::Index::default())
                .unwrap_err()
                .contains(&removed_texture.to_string())
        );
        // The parent removes the inherited asset, but the serialized instance
        // still contains its old UUID. A deliberate position edit must survive.
        asset.template.actors[0].entity.material.texture = None;
        asset.template.actors[0].entity.audio = None;
        asset.template.actors[0].entity.position = [3., 4., 5.];
        write_changed(&asset_path, &crate::document::to_vec(&asset).unwrap()).unwrap();
        let mut refreshed = saved.clone();
        refresh_linked_scene(&root, &mut refreshed).unwrap();
        crate::texture::resolve(&mut refreshed, &crate::assets::Index::default()).unwrap();
        assert_eq!(refreshed.actors[placed.root].material.texture, None);
        assert_eq!(refreshed.actors[placed.root].position, [9., 2., 1.]);
        assert!(
            refreshed.actors[placed.root]
                .blueprint_instance
                .as_ref()
                .unwrap()
                .overrides
                .contains_key("uq.entity.position.v1")
        );
        assert_eq!(
            refreshed
                .actors
                .iter()
                .map(|entity| entity.id)
                .collect::<Vec<_>>(),
            identities
        );
        // Exercise the actual staging entry point with stale serialized data,
        // not only a hand-ordered sequence of refresh and resolver calls.
        stage_into(&root, &saved, &root.join(".epok/build")).unwrap();
        assert_eq!(
            saved.actors[placed.root].material.texture,
            Some(removed_texture)
        );
        assert_eq!(saved.actors[placed.root].position, [9., 2., 1.]);
        // The Play pipeline prepares linked instances before staging. Its
        // resource-selection provenance must still describe the saved input.
        let path = root.join("assets/scenes/Main.epokmap");
        saved.save(&path).unwrap();
        let input = crate::scene_dependencies::Input::load(&path).unwrap();
        let authored_audio = crate::audio::selection_signature(&input.scene, None);
        assert_ne!(
            authored_audio,
            crate::audio::selection_signature(&refreshed, None)
        );
        let build = root.join(".epok/build");
        stage_prepared(&root, &input, &refreshed, &build).unwrap();
        crate::staging_files::patch(&root, &build, Default::default(), "release".into()).unwrap();
        crate::staging_files::BuildTicket::begin(&root, &build)
            .unwrap()
            .complete(&root, &build, b"refreshed template resources")
            .unwrap();
        let graph = crate::artifact_dependencies::Graph::load(&root).unwrap();
        assert_eq!(
            graph.nodes["audio-selection:scene-file:assets/scenes/Main.epokmap"].signature,
            Some(authored_audio)
        );
        assert!(graph.nodes["stage:.epok/build"].stale.is_empty());
    }
    #[test]
    fn native_export_keeps_local_transforms_materials_and_parent_first_order() {
        let mut s = Scene::default();
        s.actors[3].kind = "Empty".into();
        s.actors[3].scale = [1.; 3];
        s.actors[1].parent = Some(3);
        s.actors[2].parent = Some(1);
        s.actors[2].material.color = [1., 0., 0.];
        s.actors[2].material.unlit = true;
        let header = scene_header(&s, &[]).unwrap();
        assert!(header.contains("transform_order = {{0,3,1,2}}"));
        assert!(header.contains("{false, true, false, 3,"));
        assert!(header.contains("{{255,0,0}, true}"));
        s.actors[3].scale = [64.; 3];
        s.actors[1].scale = [2.; 3];
        assert!(scene_header(&s, &[]).is_err());
    }
    #[test]
    fn missing_script_fails_before_build() {
        let mut s = Scene::default();
        s.actors[1].set_class_defaults(
            &(ClassDefaults {
                name: "Missing".into(),
                ..Default::default()
            }),
        );
        assert!(
            scene_header(&s, &[])
                .unwrap_err()
                .contains("unknown class Missing")
        );
    }
    #[test]
    fn properties_generate_independent_native_instances() {
        let mut scene = Scene::default();
        let mut class = crate::actor_document::tests::class(
            "spinner",
            "Spinner",
            Some(crate::object_model::ACTOR3D_ID),
        );
        class.properties.push(crate::reflection_schema::Property {
            id: "speed".into(),
            name: "speed".into(),
            value_type: crate::reflection_schema::Type::Fixed,
            default: serde_json::json!(90),
            editable: true,
            timeline: None,
            source: class.source.clone(),
        });
        for i in [1, 2] {
            scene.actors[i].set_class_defaults(&ClassDefaults {
                name: "Spinner".into(),
                class_id: Some("spinner".into()),
                properties: [("speed".into(), serde_json::json!(i))].into(),
                overrides: ["speed".into()].into(),
                ..Default::default()
            });
        }
        let catalog = vec![scripts::Script {
            name: "Spinner".into(),
            classes: vec![class],
            ..Default::default()
        }];
        let out = scene_header(&scene, &catalog).unwrap();
        assert!(out.contains("self->speed = Fixed(4096"), "{out}");
        assert!(out.contains("self->speed = Fixed(8192"), "{out}");
    }
}
