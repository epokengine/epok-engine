//! The domain the Scene window authors, plus the bounded scene-edit history.
//!
//! `SceneViewMode` replaces the old `Editor.scene_2d: bool` (design.md section
//! 8). The boolean survives as a compatibility accessor because the render,
//! screenshot and HUD-preview paths only ever asked "is the Canvas editor
//! showing?", and that question still has a single answer: `UI`.
use crate::reflection_schema::Domain;
use serde::{Deserialize, Serialize};

/// Which authoring domain the Scene window and the Hierarchy show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneViewMode {
    #[default]
    #[serde(rename = "3d")]
    ThreeD,
    #[serde(rename = "2d")]
    TwoD,
    #[serde(rename = "ui")]
    UI,
}

impl SceneViewMode {
    pub const ALL: [SceneViewMode; 3] = [Self::ThreeD, Self::TwoD, Self::UI];
    /// The short label shown in the Scene window selector.
    pub fn label(self) -> &'static str {
        match self {
            Self::ThreeD => "3D",
            Self::TwoD => "2D",
            Self::UI => "UI",
        }
    }
    /// The wire value used by `editor_view` / `editor_state`.
    pub fn key(self) -> &'static str {
        match self {
            Self::ThreeD => "3d",
            Self::TwoD => "2d",
            Self::UI => "ui",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.key() == value)
    }
    /// The class domain whose actors this mode shows.
    pub fn domain(self) -> Domain {
        match self {
            Self::ThreeD => Domain::World3D,
            Self::TwoD => Domain::World2D,
            Self::UI => Domain::UI,
        }
    }
    /// Legacy compatibility: the Canvas/HUD editor is the `UI` mode.
    pub fn from_scene_2d(scene_2d: bool) -> Self {
        if scene_2d { Self::UI } else { Self::ThreeD }
    }
    pub fn scene_2d(self) -> bool {
        self == Self::UI
    }
}

/// Screen pixels covered by one 2D world unit at zoom 1.
///
/// This is `epok::pixels_per_unit_2d` from `runtime/world2d.hpp`; the editor and
/// the runtime must agree or an author would place content against a different
/// density than the game shows.
pub const PIXELS_PER_UNIT_2D: f32 = 32.;

/// The Scene window's orthographic 2D camera.
///
/// It mirrors the runtime projection of `runtime/world2d.hpp`:
///
/// ```text
/// screen = viewport_centre + (world - centre) * zoom * PIXELS_PER_UNIT_2D
/// ```
///
/// with **world Y up and screen Y down**, which is the only sign difference. The
/// editor camera has no rotation - rotating the authoring view would make the
/// handles ambiguous - so the runtime's `R(-camera.rotation)` term is identity
/// here. `centre` is the world position drawn at the middle of the viewport.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneView2D {
    pub centre: [f32; 2],
    pub zoom: f32,
}
impl Default for SceneView2D {
    fn default() -> Self {
        Self {
            centre: [0., 0.],
            zoom: 1.,
        }
    }
}
impl SceneView2D {
    /// Below this the grid stops being readable; above it a world unit no longer
    /// fits the viewport. Both bounds are editor ergonomics, not document rules.
    pub const MIN_ZOOM: f32 = 0.05;
    pub const MAX_ZOOM: f32 = 16.;
    /// Screen pixels per world unit at the current zoom.
    pub fn scale(&self) -> f32 {
        self.zoom * PIXELS_PER_UNIT_2D
    }
    pub fn world_to_screen(&self, world: [f32; 2], origin: [f32; 2], size: [f32; 2]) -> [f32; 2] {
        let k = self.scale();
        [
            origin[0] + size[0] * 0.5 + (world[0] - self.centre[0]) * k,
            origin[1] + size[1] * 0.5 - (world[1] - self.centre[1]) * k,
        ]
    }
    pub fn screen_to_world(&self, screen: [f32; 2], origin: [f32; 2], size: [f32; 2]) -> [f32; 2] {
        let k = self.scale();
        [
            self.centre[0] + (screen[0] - origin[0] - size[0] * 0.5) / k,
            self.centre[1] - (screen[1] - origin[1] - size[1] * 0.5) / k,
        ]
    }
    /// Drags the world with the cursor: the point under the mouse stays under it.
    pub fn pan_pixels(&mut self, delta: [f32; 2]) {
        let k = self.scale();
        self.centre[0] -= delta[0] / k;
        self.centre[1] += delta[1] / k;
    }
    /// Wheel zoom anchored on the cursor, so the world point under the mouse
    /// does not move. `wheel` is ImGui's notch count.
    pub fn zoom_at(&mut self, wheel: f32, screen: [f32; 2], origin: [f32; 2], size: [f32; 2]) {
        let anchor = self.screen_to_world(screen, origin, size);
        self.zoom = (self.zoom * 1.2_f32.powf(wheel)).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        let after = self.screen_to_world(screen, origin, size);
        self.centre[0] += anchor[0] - after[0];
        self.centre[1] += anchor[1] - after[1];
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// The authored 2D transform of one `epok::SceneComponent2D`, as the Scene
/// window reads it out of `ComponentInstance::properties`.
///
/// `runtime/object_model.hpp` declares the member as a single `Transform2D
/// transform`, so a document may carry either the flattened override keys
/// (`position`, `rotation`, `scale`, `draw_order`) or one nested `transform`
/// object. Both are read; a key that is absent or malformed falls back to the
/// runtime default rather than hiding the actor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    /// World units, Y up. `runtime/world2d.hpp` clamps to ±8192.
    pub position: [f32; 2],
    /// Degrees, counter-clockwise (+X towards +Y).
    pub rotation: f32,
    pub scale: [f32; 2],
    pub draw_order: i32,
}
impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: [0., 0.],
            rotation: 0.,
            scale: [1., 1.],
            draw_order: 0,
        }
    }
}

fn number(value: Option<&serde_json::Value>) -> Option<f32> {
    value?.as_f64().map(|v| v as f32)
}
fn pair(value: Option<&serde_json::Value>) -> Option<[f32; 2]> {
    let items = value?.as_array()?;
    Some([number(items.first())?, number(items.get(1))?])
}

/// Reads a 2D transform out of one component's authored properties.
pub fn read_transform_2d(
    properties: &std::collections::BTreeMap<String, serde_json::Value>,
) -> Transform2D {
    let nested = properties.get("transform").and_then(|v| v.as_object());
    let get = |key: &str| -> Option<&serde_json::Value> {
        properties
            .get(key)
            .or_else(|| nested.and_then(|object| object.get(key)))
    };
    let default = Transform2D::default();
    Transform2D {
        position: pair(get("position")).unwrap_or(default.position),
        rotation: number(get("rotation")).unwrap_or(default.rotation),
        scale: pair(get("scale")).unwrap_or(default.scale),
        draw_order: get("draw_order")
            .and_then(serde_json::Value::as_i64)
            .map(|v| v as i32)
            .unwrap_or(default.draw_order),
    }
}

/// Writes a dragged position back, keeping the shape the document already uses:
/// a nested `transform` object stays nested, anything else writes the flat key.
/// The key is always marked as an override, because a drag is an explicit edit
/// even when it lands exactly on the class default.
pub fn write_position_2d(
    properties: &mut std::collections::BTreeMap<String, serde_json::Value>,
    overrides: &mut std::collections::BTreeSet<String>,
    position: [f32; 2],
) {
    let value = serde_json::json!([position[0], position[1]]);
    if !properties.contains_key("position")
        && let Some(object) = properties
            .get_mut("transform")
            .and_then(|v| v.as_object_mut())
    {
        object.insert("position".into(), value);
        overrides.insert("transform".into());
        return;
    }
    properties.insert("position".into(), value);
    overrides.insert("position".into());
}

/// A bounded undo/redo stack over whole snapshots.
///
/// The editor had no general undo: only the component-attachment stack
/// (`Editor::undo_attachment`). This keeps the previous value of the document
/// as a baseline and pushes it when an edit is recorded, so a caller that
/// mutates first and reports afterwards — which is what `Editor::changed()`
/// does — still produces a correct stack.
pub struct History<T> {
    limit: usize,
    baseline: T,
    undo: Vec<T>,
    redo: Vec<T>,
    /// The gesture currently being coalesced. While it is set, a
    /// [`History::record_coalesced`] call with the same key folds into the step
    /// the gesture already pushed instead of pushing another one.
    run: Option<&'static str>,
}

impl<T: Clone> History<T> {
    pub fn new(baseline: T, limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            baseline,
            undo: Vec::new(),
            redo: Vec::new(),
            run: None,
        }
    }
    /// Forgets every step and adopts `current` as the new starting point. Used
    /// when a different document is loaded: its history is not this one's.
    pub fn reset(&mut self, current: &T) {
        self.baseline = current.clone();
        self.undo.clear();
        self.redo.clear();
        self.run = None;
    }
    /// Records that the document just moved from the baseline to `current`.
    pub fn record(&mut self, current: &T) {
        self.run = None;
        self.undo
            .push(std::mem::replace(&mut self.baseline, current.clone()));
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
        self.redo.clear();
    }
    /// Records a continuous gesture as a single step.
    ///
    /// A drag reports every frame. The first call of a run pushes one step, like
    /// [`History::record`]; every further call carrying the same `key` only moves
    /// the baseline forward, so undoing once returns to the value the gesture
    /// started from rather than to the previous frame. Any discrete edit, a
    /// different key, an undo/redo or [`History::end_run`] ends the run.
    pub fn record_coalesced(&mut self, current: &T, key: &'static str) {
        if self.run == Some(key) {
            self.baseline = current.clone();
            self.redo.clear();
            return;
        }
        self.record(current);
        self.run = Some(key);
    }
    /// Ends the open coalescing run, so the next gesture with the same key
    /// records its own step. Releasing the mouse calls this.
    pub fn end_run(&mut self) {
        self.run = None;
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    #[cfg(test)]
    pub fn depth(&self) -> (usize, usize) {
        (self.undo.len(), self.redo.len())
    }
    /// Returns the document to restore, having recorded `current` as the redo step.
    pub fn undo(&mut self, current: &T) -> Option<T> {
        self.run = None;
        let previous = self.undo.pop()?;
        self.redo.push(current.clone());
        self.baseline = previous.clone();
        Some(previous)
    }
    pub fn redo(&mut self, current: &T) -> Option<T> {
        self.run = None;
        let next = self.redo.pop()?;
        self.undo.push(current.clone());
        self.baseline = next.clone();
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_modes_round_trip_their_wire_values_and_the_legacy_boolean() {
        for mode in SceneViewMode::ALL {
            assert_eq!(SceneViewMode::parse(mode.key()), Some(mode));
            assert_eq!(
                serde_json::to_value(mode).unwrap(),
                serde_json::Value::String(mode.key().into())
            );
        }
        assert_eq!(SceneViewMode::default(), SceneViewMode::ThreeD);
        assert_eq!(SceneViewMode::parse("hud"), None);
        assert_eq!(SceneViewMode::from_scene_2d(true), SceneViewMode::UI);
        assert_eq!(SceneViewMode::from_scene_2d(false), SceneViewMode::ThreeD);
        assert!(SceneViewMode::UI.scene_2d());
        assert!(!SceneViewMode::TwoD.scene_2d());
        assert!(!SceneViewMode::ThreeD.scene_2d());
        assert_eq!(SceneViewMode::TwoD.domain(), Domain::World2D);
    }

    #[test]
    fn history_is_bounded_and_redo_survives_until_the_next_edit() {
        let mut history = History::new(0, 3);
        assert!(!history.can_undo() && !history.can_redo());
        for value in 1..=5 {
            history.record(&value);
        }
        // Bounded: only the three most recent steps remain.
        assert_eq!(history.depth(), (3, 0));
        assert_eq!(history.undo(&5), Some(4));
        assert_eq!(history.undo(&4), Some(3));
        assert_eq!(history.depth(), (1, 2));
        assert_eq!(history.redo(&3), Some(4));
        assert_eq!(history.redo(&4), Some(5));
        assert_eq!(history.redo(&5), None);
        // A new edit after an undo drops the redo branch.
        assert_eq!(history.undo(&5), Some(4));
        history.record(&9);
        assert!(!history.can_redo());
        assert_eq!(history.undo(&9), Some(4));
    }

    #[test]
    fn a_coalesced_run_is_one_step_and_a_different_key_breaks_it() {
        let mut history = History::new(0, 32);
        for value in 1..=50 {
            history.record_coalesced(&value, "gizmo-drag");
        }
        // Fifty frames of one drag are a single undo step back to the start.
        assert_eq!(history.depth(), (1, 0));
        assert_eq!(history.undo(&50), Some(0));
        assert!(!history.can_undo());
        // A different key starts its own run.
        history.redo(&0);
        history.record_coalesced(&60, "gizmo-drag");
        history.record_coalesced(&61, "blueprint-sync");
        history.record_coalesced(&62, "blueprint-sync");
        assert_eq!(history.depth(), (3, 0));
        assert_eq!(history.undo(&62), Some(60));
        // A discrete edit also ends the run, and the run does not resume.
        let mut history = History::new(0, 32);
        history.record_coalesced(&1, "drag");
        history.record(&2);
        history.record_coalesced(&3, "drag");
        history.record_coalesced(&4, "drag");
        assert_eq!(history.depth(), (3, 0));
        // end_run splits one gesture from the next sharing its key.
        let mut history = History::new(0, 32);
        history.record_coalesced(&1, "drag");
        history.end_run();
        history.record_coalesced(&2, "drag");
        assert_eq!(history.depth(), (2, 0));
    }

    #[test]
    fn a_2d_transform_reads_flat_or_nested_keys_and_defaults_to_identity() {
        use std::collections::{BTreeMap, BTreeSet};
        assert_eq!(read_transform_2d(&BTreeMap::new()), Transform2D::default());
        let flat: BTreeMap<String, serde_json::Value> = serde_json::from_value(serde_json::json!({
            "position": [2.0, -3.5], "rotation": 90.0, "scale": [2.0, 0.5], "draw_order": -4
        }))
        .unwrap();
        let read = read_transform_2d(&flat);
        assert_eq!(read.position, [2., -3.5]);
        assert_eq!(read.rotation, 90.);
        assert_eq!(read.scale, [2., 0.5]);
        assert_eq!(read.draw_order, -4);
        let nested: BTreeMap<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({"transform": {"position": [1.0, 1.0]}}))
                .unwrap();
        assert_eq!(read_transform_2d(&nested).position, [1., 1.]);
        // Identity for everything the nested object does not carry.
        assert_eq!(read_transform_2d(&nested).scale, [1., 1.]);
        // A malformed value never hides the actor.
        let broken: BTreeMap<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({"position": "nowhere", "scale": [1.0]}))
                .unwrap();
        assert_eq!(read_transform_2d(&broken), Transform2D::default());
        // Writing keeps the document's existing shape and always marks an override.
        let mut properties = nested.clone();
        let mut overrides = BTreeSet::new();
        write_position_2d(&mut properties, &mut overrides, [5., 6.]);
        assert!(!properties.contains_key("position"));
        assert_eq!(read_transform_2d(&properties).position, [5., 6.]);
        assert!(overrides.contains("transform"));
        let mut properties = BTreeMap::new();
        let mut overrides = BTreeSet::new();
        write_position_2d(&mut properties, &mut overrides, [0., 0.]);
        assert_eq!(properties["position"], serde_json::json!([0.0, 0.0]));
        assert!(overrides.contains("position"));
    }

    #[test]
    fn the_2d_camera_matches_the_runtime_projection() {
        let origin = [100., 50.];
        let size = [640., 480.];
        let view = SceneView2D::default();
        // The centre of the viewport is the camera centre; world Y is up.
        assert_eq!(view.world_to_screen([0., 0.], origin, size), [420., 290.]);
        assert_eq!(
            view.world_to_screen([1., 1.], origin, size),
            [420. + PIXELS_PER_UNIT_2D, 290. - PIXELS_PER_UNIT_2D]
        );
        // Round trip through both directions, panned and zoomed.
        let mut view = SceneView2D {
            centre: [3.5, -2.25],
            zoom: 2.,
        };
        for point in [[0., 0.], [12.5, -7.], [-40., 33.]] {
            let screen = view.world_to_screen(point, origin, size);
            let back = view.screen_to_world(screen, origin, size);
            assert!((back[0] - point[0]).abs() < 1e-3 && (back[1] - point[1]).abs() < 1e-3);
        }
        // Panning drags the world with the cursor.
        let before = view.world_to_screen([1., 1.], origin, size);
        view.pan_pixels([10., -6.]);
        let after = view.world_to_screen([1., 1.], origin, size);
        assert!((after[0] - before[0] - 10.).abs() < 1e-3);
        assert!((after[1] - before[1] + 6.).abs() < 1e-3);
        // Wheel zoom keeps the world point under the cursor in place.
        let cursor = [180., 120.];
        let anchor = view.screen_to_world(cursor, origin, size);
        view.zoom_at(3., cursor, origin, size);
        let held = view.screen_to_world(cursor, origin, size);
        assert!((held[0] - anchor[0]).abs() < 1e-3 && (held[1] - anchor[1]).abs() < 1e-3);
        // Zoom is clamped rather than allowed to collapse or explode.
        view.zoom_at(-100., cursor, origin, size);
        assert_eq!(view.zoom, SceneView2D::MIN_ZOOM);
        view.zoom_at(100., cursor, origin, size);
        assert_eq!(view.zoom, SceneView2D::MAX_ZOOM);
    }

    #[test]
    fn reset_forgets_the_history_of_the_previous_document() {
        let mut history = History::new("a", 8);
        history.record(&"b");
        history.reset(&"c");
        assert!(!history.can_undo() && !history.can_redo());
        history.record(&"d");
        assert_eq!(history.undo(&"d"), Some("c"));
    }
}
