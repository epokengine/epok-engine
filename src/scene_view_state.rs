//! Authoring camera state belongs to a map, never to its world transforms.
use crate::{scene::Scene, viewport::View};
use std::path::{Path, PathBuf};

fn valid(view: &View) -> bool {
    view.center
        .iter()
        .chain([
            &view.yaw,
            &view.pitch,
            &view.zoom,
            &view.distance,
            &view.fly_speed,
            &view.camera_speed_scalar,
            &view.phase,
        ])
        .all(|v| v.is_finite())
        && (0.05..=8.).contains(&view.zoom)
        && (1.05..=10000.).contains(&view.distance)
        && (0.01..=1000.).contains(&view.fly_speed)
        && (1.0..=128.).contains(&view.camera_speed_scalar)
}

fn preference_path(root: &Path, scene: &Path) -> PathBuf {
    let key = scene
        .strip_prefix(root)
        .unwrap_or(scene)
        .to_string_lossy()
        .replace('\\', "/");
    let key = if cfg!(windows) {
        key.to_lowercase()
    } else {
        key
    };
    root.join("UserSettings/SceneViews")
        .join(format!("{}.epokprefs", crate::assets::hash(key.as_bytes())))
}

pub fn load(root: &Path, scene: &Path, legacy_startup: bool) -> Option<View> {
    let read = |path: PathBuf| -> Option<View> {
        let view = crate::document::from_slice::<View>(&std::fs::read(path).ok()?).ok()?;
        valid(&view).then_some(view)
    };
    read(preference_path(root, scene)).or_else(|| {
        legacy_startup
            .then(|| read(root.join("UserSettings/SceneView.epokprefs")))
            .flatten()
    })
}

pub fn save(root: &Path, scene: &Path, view: &View) -> Result<(), String> {
    if !valid(view) {
        return Err("Invalid Scene View camera".into());
    }
    let path = preference_path(root, scene);
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    crate::project::write_changed(
        &path,
        &crate::document::to_vec(view).map_err(|e| e.to_string())?,
    )
}

/// Walk source geometry, not the PSX subdivision/cooking path. Bounds are cheap
/// and do not depend on the streaming simulation, baked visibility or selection.
pub fn bounds(scene: &Scene, selected: Option<usize>) -> Option<([f32; 3], [f32; 3])> {
    let mut low = [f32::INFINITY; 3];
    let mut high = [f32::NEG_INFINITY; 3];
    for (index, actor) in scene.actors.iter().enumerate() {
        if actor.kind != "Mesh"
            || !scene.is_active(index)
            || selected.is_some_and(|parent| !scene.is_descendant(index, parent))
        {
            continue;
        }
        let world = scene.world_matrix(index);
        let mut extend = |local| {
            let point = world.point(local);
            if point.iter().all(|x| x.is_finite()) {
                for axis in 0..3 {
                    low[axis] = low[axis].min(point[axis]);
                    high[axis] = high[axis].max(point[axis]);
                }
            }
        };
        if let Some(component) = &actor.skeletal_mesh {
            if let Some(model) = &component.model {
                for point in model.points(component.clip, component.time, component.looping) {
                    extend(point);
                }
            }
        } else if let Some(component) = &actor.terrain {
            if let Some(doc) = &component.document {
                let [nx, nz] = doc.cells();
                let [sx, sz] = doc.span();
                for z in 0..=nz {
                    for x in 0..=nx {
                        extend([
                            x as f32 * doc.cell_size() - sx * 0.5,
                            doc.height(x.into(), z.into()),
                            z as f32 * doc.cell_size() - sz * 0.5,
                        ]);
                    }
                }
            }
        } else if let Some(component) = &actor.editable_mesh {
            if let Some(doc) = &component.document {
                for face in &doc.faces {
                    for point in doc.points(face) {
                        extend(point);
                    }
                }
            }
        } else {
            for point in crate::lighting::CORNERS {
                extend(point);
            }
        }
    }
    low.iter().all(|v| v.is_finite()).then_some((low, high))
}

pub fn framed(scene: &Scene, panel: [f32; 2], preferences: &crate::settings::Preferences) -> View {
    let mut view = View {
        fly_speed: preferences.fly_speed,
        camera_speed_scalar: preferences.camera_speed_scalar,
        distance_scaled_speed: preferences.distance_scaled_speed,
        ..Default::default()
    };
    if let Some((low, high)) = bounds(scene, None) {
        view.frame_bounds_in_panel(low, high, panel);
    }
    view
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_far_geometry_without_moving_it_or_including_hidden_meshes() {
        let mut scene = Scene::default();
        scene.actors = vec![crate::scene::Actor::cube("Far mesh".into())];
        scene.actors[0].position = [800., 24., -1200.];
        scene.actors[0].scale = [10., 4., 6.];
        let mut hidden = scene.actors[0].clone();
        hidden.id = uuid::Uuid::new_v4();
        hidden.position = [9000.; 3];
        hidden.active = false;
        scene.actors.push(hidden);
        let before = scene.clone();
        let view = framed(&scene, [960., 600.], &Default::default());
        assert_eq!(view.center, [800., 24., -1200.]);
        assert_eq!(
            bounds(&scene, None),
            Some(([795., 22., -1203.], [805., 26., -1197.]))
        );
        assert_eq!(scene, before);
        assert!(view.distance > 6. && valid(&view));
        scene.actors.clear();
        assert!(bounds(&scene, None).is_none());
        assert_eq!(
            framed(&scene, [960., 600.], &Default::default()).center,
            [0.; 3]
        );
    }

    #[test]
    fn map_preferences_are_independent_and_corruption_is_nonfatal() {
        let root = crate::workspace::tests::temp("map-view-state");
        let a = root.join("assets/scenes/A.epokmap");
        let b = root.join("assets/scenes/B.epokmap");
        let view = View {
            center: [40., 2., -80.],
            lighting: false,
            fly_speed: 20.,
            camera_speed_scalar: 2.5,
            distance_scaled_speed: true,
            ..Default::default()
        };
        save(&root, &a, &view).unwrap();
        assert_eq!(load(&root, &a, false).unwrap().center, view.center);
        assert!(!load(&root, &a, false).unwrap().lighting);
        let restored = load(&root, &a, false).unwrap();
        assert_eq!(restored.fly_speed, 20.);
        assert_eq!(restored.camera_speed_scalar, 2.5);
        assert!(restored.distance_scaled_speed);
        assert!(load(&root, &b, false).is_none());
        std::fs::write(preference_path(&root, &a), b"broken").unwrap();
        assert!(load(&root, &a, false).is_none());
        std::fs::write(
            root.join("UserSettings/SceneView.epokprefs"),
            crate::document::to_vec(&view).unwrap(),
        )
        .unwrap();
        assert!(
            load(&root, &b, false).is_none(),
            "legacy startup view must not leak to other maps"
        );
        assert_eq!(load(&root, &b, true).unwrap().center, view.center);
        let invalid = View {
            distance: f32::NAN,
            ..view
        };
        assert!(save(&root, &b, &invalid).is_err());
        for scalar in [f32::NAN, f32::INFINITY, 0., 129.] {
            assert!(
                save(
                    &root,
                    &b,
                    &View {
                        camera_speed_scalar: scalar,
                        ..view
                    }
                )
                .is_err()
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
