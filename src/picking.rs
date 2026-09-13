//! Click-time ray casting against the meshes drawn by Scene; no GPU readback.
use crate::{scene::Scene, viewport::View};

pub fn texture_pixel(mouse: [f32; 2], origin: [f32; 2], factor: f32, uv: [f32; 2]) -> [f32; 2] {
    [
        (mouse[0] - origin[0]) / factor + (1. - uv[0]) * 480.,
        (mouse[1] - origin[1]) / factor + (1. - uv[1]) * 300.,
    ]
}

fn unproject(view: &View, pixel: [f32; 2], depth: f32) -> [f32; 3] {
    view.unproject(pixel, depth)
}

pub fn pick(scene: &Scene, view: &View, pixel: [f32; 2]) -> Option<usize> {
    let start = unproject(view, pixel, 1.);
    let next = unproject(view, pixel, 2.);
    let direction = std::array::from_fn(|i| next[i] - start[i]);
    scene
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| e.kind == "Mesh" || e.light.is_some())
        .filter_map(|(index, e)| {
            if e.kind != "Mesh" {
                let p = crate::viewport::project(view, scene.world_matrix(index).point([0.; 3]));
                return (p[2] > 1. && (p[0] - pixel[0]).hypot(p[1] - pixel[1]) <= 10.)
                    .then_some((index, p[2] - 1.));
            }
            let inverse = scene.world_matrix(index).inverse().ok()?;
            // Transform the ray, not an enclosing world-space box: rotated and
            // sheared cubes are picked exactly, including inherited transforms.
            let origin = inverse.point(start);
            let direction = inverse.vector(direction);
            if let Some(m) = &e.editable_mesh {
                return crate::mesh::pick(m.document.as_deref()?, origin, direction, |_| true)
                    .map(|(_, depth)| (index, depth));
            }
            cube_hit(origin, direction).map(|depth| (index, depth))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
        .map(|(index, _)| index)
}

fn cube_hit(origin: [f32; 3], direction: [f32; 3]) -> Option<f32> {
    let mut enter = f32::NEG_INFINITY;
    let mut exit = f32::INFINITY;
    for axis in 0..3 {
        if direction[axis].abs() < 1e-10 {
            if origin[axis].abs() > 0.5 {
                return None;
            }
        } else {
            let a = (-0.5 - origin[axis]) / direction[axis];
            let b = (0.5 - origin[axis]) / direction[axis];
            enter = enter.max(a.min(b));
            exit = exit.min(a.max(b));
        }
    }
    // If the near plane cuts the cube, its exit surface is the visible hit.
    let hit = if enter >= 0. { enter } else { exit };
    (enter <= exit && (0. ..=19999.).contains(&hit)).then_some(hit)
}

#[derive(Default)]
pub struct ClickGesture {
    press: Option<[f32; 2]>,
}
impl ClickGesture {
    pub fn update(
        &mut self,
        mouse: [f32; 2],
        pressed: bool,
        released: bool,
        down: bool,
        eligible: bool,
    ) -> bool {
        if !eligible {
            self.press = None;
            return false;
        }
        if pressed {
            self.press = Some(mouse);
        }
        if let Some(start) = self.press
            && (mouse[0] - start[0]).hypot(mouse[1] - start[1]) > 3.
        {
            self.press = None;
        }
        if released {
            return self.press.take().is_some();
        }
        if !down {
            self.press = None;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{scene::Entity, viewport};
    fn scene(entities: Vec<Entity>) -> Scene {
        Scene {
            version: 1,
            name: "Picking".into(),
            entities,
            ..Scene::default()
        }
    }
    fn pixel(view: &View, p: [f32; 3]) -> [f32; 2] {
        let p = viewport::project(view, p);
        [p[0], p[1]]
    }
    #[test]
    fn nearest_visible_mesh_wins_not_entity_order() {
        let view = View::default();
        let p = [480., 342.];
        let mut far = Entity::cube("Far".into());
        far.position = unproject(&view, p, 14.);
        let mut near = Entity::cube("Near".into());
        near.position = unproject(&view, p, 8.);
        assert_eq!(
            pick(&scene(vec![far.clone(), near.clone()]), &view, p),
            Some(1)
        );
        assert_eq!(pick(&scene(vec![near, far]), &view, p), Some(0));
    }
    #[test]
    fn inherited_rotation_scale_and_shear_are_pickable_after_orbit_and_zoom() {
        let mut parent = Entity::cube("Parent".into());
        parent.kind = "Empty".into();
        parent.position = [2., 1., 0.];
        parent.rotation = [20., 45., 15.];
        parent.scale = [2., 1., 0.5];
        let mut child = Entity::cube("Child".into());
        child.parent = Some(0);
        child.rotation = [10., 45., 20.];
        let scene = scene(vec![parent, child]);
        for yaw in [-1.2, 0., 0.65, 2.] {
            for zoom in [0.3, 0.85, 3.] {
                let view = View {
                    yaw,
                    zoom,
                    phase: 0.,
                    ..Default::default()
                };
                assert_eq!(
                    pick(
                        &scene,
                        &view,
                        pixel(&view, scene.world_matrix(1).point([0.; 3]))
                    ),
                    Some(1)
                );
            }
        }
    }
    #[test]
    fn misses_background_hidden_objects_and_invisible_entities() {
        let view = View::default();
        let mut cube = Entity::cube("Cube".into());
        assert_eq!(pick(&scene(vec![cube.clone()]), &view, [20., 20.]), None);
        cube.position = unproject(&view, [480., 342.], -3.);
        assert_eq!(pick(&scene(vec![cube.clone()]), &view, [480., 342.]), None);
        cube.position = [0.; 3];
        cube.kind = "Empty".into();
        assert_eq!(pick(&scene(vec![cube]), &view, [480., 342.]), None);
        assert_eq!(cube_hit([0., 0., 0.], [0., 0., 1.]), Some(0.5));
    }
    #[test]
    fn cropped_and_scaled_panel_maps_to_the_rendered_pixel() {
        for size in [[300., 800.], [1400., 400.], [960., 600.]] {
            let factor = (size[0] / 960_f32).max(size[1] / 600.);
            let uv = [size[0] / (960. * factor), size[1] / (600. * factor)];
            let p = [510., 330.];
            let origin = [240., 95.];
            let mouse = [
                origin[0] + (p[0] - (1. - uv[0]) * 480.) * factor,
                origin[1] + (p[1] - (1. - uv[1]) * 300.) * factor,
            ];
            let actual = texture_pixel(mouse, origin, factor, uv);
            for i in 0..2 {
                assert!((actual[i] - p[i]).abs() < 0.001);
            }
        }
    }
    #[test]
    fn clicks_select_but_drags_gizmos_and_alt_orbits_do_not() {
        let mut click = ClickGesture::default();
        assert!(!click.update([10., 10.], true, false, true, true));
        assert!(click.update([10., 10.], false, true, false, true));
        click.update([10., 10.], true, false, true, true);
        click.update([20., 10.], false, false, true, true);
        assert!(!click.update([10., 10.], false, true, false, true));
        click.update([10., 10.], true, false, true, false);
        assert!(!click.update([10., 10.], false, true, false, true));
    }
}
