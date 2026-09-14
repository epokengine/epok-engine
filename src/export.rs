#[cfg(test)]
use crate::scene::Scene;
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn export_project(
    root: &Path,
    input: &crate::scene_dependencies::Input,
) -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let destination = root.join("exports").join(format!("psyqo-{stamp}"));
    if destination.exists() {
        return Err("Export directory already exists".into());
    }
    crate::project::stage_with_origin(root, &input.scene, &destination, &input.origin)?;
    let documents = crate::staging_files::Capture::begin(&destination)?;
    for (path, contents) in [
        ("docs/hud.md", include_str!("../docs/hud.md")),
        ("docs/streaming.md", include_str!("../docs/streaming.md")),
        ("docs/settings.md", include_str!("../docs/settings.md")),
        (
            "docs/performance.md",
            include_str!("../docs/performance.md"),
        ),
        (
            "docs/runtime-services.md",
            include_str!("../docs/runtime-services.md"),
        ),
        (
            "docs/input-collision.md",
            include_str!("../docs/input-collision.md"),
        ),
        (
            "docs/memory-card.md",
            include_str!("../docs/memory-card.md"),
        ),
        (
            "docs/sprites-particles.md",
            include_str!("../docs/sprites-particles.md"),
        ),
        ("docs/textures.md", include_str!("../docs/textures.md")),
        ("docs/timelines.md", include_str!("../docs/timelines.md")),
        ("docs/blueprints.md", include_str!("../docs/blueprints.md")),
        (
            "docs/camera-resources.md",
            include_str!("../docs/camera-resources.md"),
        ),
        (
            "docs/static-mesh-import.md",
            include_str!("../docs/static-mesh-import.md"),
        ),
        (
            "docs/environment-effects.md",
            include_str!("../docs/environment-effects.md"),
        ),
        (
            "docs/palette-animation.md",
            include_str!("../docs/palette-animation.md"),
        ),
        ("README.md", include_str!("../runtime/README.md")),
        ("LICENSE", include_str!("../LICENSE")),
        (
            "THIRD_PARTY_NOTICES.md",
            include_str!("../runtime/THIRD_PARTY_NOTICES.md"),
        ),
        (
            "licenses/nugget-MIT.txt",
            include_str!("../runtime/licenses/nugget-MIT.txt"),
        ),
        (
            "licenses/EASTL.txt",
            include_str!("../runtime/licenses/EASTL.txt"),
        ),
        (
            "licenses/EASTL-third-party.txt",
            include_str!("../runtime/licenses/EASTL-third-party.txt"),
        ),
        (
            "licenses/EABase.txt",
            include_str!("../runtime/licenses/EABase.txt"),
        ),
        (
            "licenses/psx-font.txt",
            include_str!("../runtime/licenses/psx-font.txt"),
        ),
        (
            "licenses/GCC-GPL-3.txt",
            include_str!("../runtime/licenses/GCC-GPL-3.txt"),
        ),
        (
            "licenses/GCC-Runtime-Exception-3.1.txt",
            include_str!("../runtime/licenses/GCC-Runtime-Exception-3.1.txt"),
        ),
    ] {
        crate::project::write_changed(&destination.join(path), contents.as_bytes())?;
    }
    crate::staging_files::publish_export(root, &destination, documents.finish())?;
    Ok(destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn exported_project_carries_runtime_sources_and_notices() {
        let sample = crate::workspace::tests::temp("actor-export");
        let _project =
            crate::workspace::create(&sample, "Export", crate::workspace::Template::Sample)
                .unwrap();
        let root = sample.as_path();
        let scene = Scene::load(&crate::workspace::startup_scene(root).unwrap()).unwrap();
        let destination = export_project(root, &scene.into()).unwrap();
        for file in [
            "main.cpp",
            "epok.hpp",
            "scene.hh",
            "sources.mk",
            "scene.epokmap",
            "scripts/Spinner.cpp",
            "Scripts.epokmanifest",
            "docs/timelines.md",
            "docs/blueprints.md",
            "licenses/nugget-MIT.txt",
            "licenses/EASTL.txt",
            "licenses/EASTL-third-party.txt",
            "licenses/EABase.txt",
            "licenses/psx-font.txt",
            "licenses/GCC-GPL-3.txt",
            "licenses/GCC-Runtime-Exception-3.1.txt",
        ] {
            assert!(
                destination.join(file).is_file(),
                "Missing export file: {file}"
            );
        }
        let readme = fs::read_to_string(destination.join("README.md")).unwrap();
        assert!(readme.contains("make BUILD=Release NUGGET_DIR=third_party/nugget"));
        let notices = fs::read_to_string(destination.join("THIRD_PARTY_NOTICES.md")).unwrap();
        assert!(notices.contains("Zingot Games"));
        assert!(notices.contains("user-authored game scripts or assets"));
        assert!(
            fs::read_to_string(destination.join("LICENSE"))
                .unwrap()
                .starts_with("MIT License")
        );
        assert!(!destination.join("resources/editor").exists());
        // Keep the generated export available for a standalone MIPS build check.
        println!("Verified export: {}", destination.display());
    }
}
