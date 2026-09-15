//! Native Sequencer view and document commands. Time is always Q12 seconds;
//! frames are a display/snap grid, never a second playback clock.
use crate::{
    blueprint::Registry,
    timeline::{self, TimelineAsset},
};
use imgui::{MouseButton, Ui};
use std::collections::BTreeSet;
use uuid::Uuid;

const BG: [f32; 4] = [0.115, 0.112, 0.112, 1.];
const ROW: [f32; 4] = [0.19, 0.185, 0.185, 1.];
const TEXT: [f32; 4] = [0.72, 0.70, 0.70, 1.];
const MUTED: [f32; 4] = [0.43, 0.42, 0.42, 1.];
const ORANGE: [f32; 4] = [0.80, 0.45, 0.32, 1.];
const TEAL: [f32; 4] = [0.23, 0.39, 0.38, 1.];
const ROW_HEIGHT: f32 = 25.;

#[derive(Clone)]
struct CopiedKey {
    track: Uuid,
    channel: Option<Uuid>,
    time: i32,
    key: timeline::Key,
}
struct SectionDrag {
    id: Uuid,
    mode: u8,
    anchor: f32,
    original: crate::timeline_section::Section,
}
#[derive(Default)]
pub struct Actions {
    pub save: bool,
    pub undo: bool,
    pub redo: bool,
    pub validate: bool,
    pub details: bool,
}

pub struct View {
    pub playing: bool,
    pub reverse: bool,
    pub snapping: bool,
    pub fps: i32,
    pub selected: BTreeSet<Uuid>,
    pub track: Option<Uuid>,
    pub gesture: bool,
    pub status: String,
    pub preview_scene: bool,
    lane: Option<usize>,
    search: String,
    collapsed: BTreeSet<Uuid>,
    start: f64,
    span: f64,
    fraction: f64,
    tree_width: f32,
    scroll: f32,
    curves: bool,
    sections_open: bool,
    drag: Option<(f32, Vec<(Uuid, i32)>)>,
    section_drag: Option<SectionDrag>,
    box_select: Option<([f32; 2], BTreeSet<Uuid>)>,
    clipboard: Vec<CopiedKey>,
}
impl Default for View {
    fn default() -> Self {
        Self {
            playing: false,
            reverse: false,
            snapping: true,
            fps: 30,
            selected: BTreeSet::new(),
            track: None,
            gesture: false,
            status: String::new(),
            preview_scene: true,
            lane: None,
            search: String::new(),
            collapsed: BTreeSet::new(),
            start: 0.,
            span: 0.,
            fraction: 0.,
            tree_width: 400.,
            scroll: 0.,
            curves: false,
            sections_open: false,
            drag: None,
            section_drag: None,
            box_select: None,
            clipboard: Vec::new(),
        }
    }
}

/// Integer round trip keeps 30 fps keys on a stable grid in a 4096 Hz clock.
pub fn frame_tick(frame: i64, fps: i32) -> i32 {
    ((frame * 4096 + i64::from(fps) / 2) / i64::from(fps)).clamp(0, i64::from(i32::MAX)) as i32
}
fn tick_frame(tick: i32, fps: i32) -> i64 {
    (i64::from(tick) * i64::from(fps) + 2048) / 4096
}

impl View {
    pub fn fit(&mut self, duration: i32) {
        self.span = f64::from(duration.max(1)) * 1.18;
        self.start = -self.span * 0.04;
    }
    fn snapped(&self, tick: f64, duration: i32) -> i32 {
        let tick = tick.round().clamp(0., f64::from(duration.max(0))) as i32;
        if self.snapping {
            frame_tick(tick_frame(tick, self.fps), self.fps).min(duration)
        } else {
            tick
        }
    }
    fn snapped_to_asset(
        &self,
        asset: &TimelineAsset,
        tick: f64,
        tolerance: f64,
        ignore_selection: bool,
    ) -> i32 {
        let mut result = self.snapped(tick, asset.duration_ticks);
        if !self.snapping {
            return result;
        }
        let mut distance = (f64::from(result) - tick).abs();
        let mut consider = |id: Uuid, candidate: i32| {
            if ignore_selection && self.selected.contains(&id) {
                return;
            }
            let candidate_distance = (f64::from(candidate) - tick).abs();
            if candidate_distance <= tolerance && candidate_distance < distance {
                result = candidate;
                distance = candidate_distance;
            }
        };
        for track in &asset.tracks {
            for key in &track.keys {
                consider(key.id, key.tick);
            }
            for section in &track.sections {
                consider(section.id, section.start_tick);
                consider(section.id, section.end_tick);
                for channel in &section.channels {
                    for key in &channel.keys {
                        consider(key.id, section.sequence_tick(key.tick));
                    }
                }
            }
        }
        for track in &asset.events {
            for key in &track.keys {
                consider(key.id, key.tick);
            }
        }
        for marker in &asset.markers {
            consider(marker.id, marker.tick);
        }
        result.clamp(0, asset.duration_ticks)
    }
    pub fn advance(&mut self, asset: &TimelineAsset, tick: &mut i32, seconds: f64) {
        if !self.playing || seconds <= 0. || !seconds.is_finite() {
            return;
        }
        self.fraction += seconds.min(0.25) * 4096.;
        let delta = self.fraction.floor() as i64;
        self.fraction -= delta as f64;
        let end = i64::from(asset.duration_ticks.max(1));
        let next = i64::from(*tick) + if self.reverse { -delta } else { delta };
        if next > end || next < 0 {
            if asset.loop_mode == timeline::LoopMode::Repeat {
                *tick = next.rem_euclid(end) as i32;
            } else {
                *tick = next.clamp(0, end) as i32;
                self.playing = false;
            }
        } else {
            *tick = next as i32;
        }
    }
    fn toggle_play(&mut self, duration: i32, tick: &mut i32) {
        if !self.playing {
            if self.reverse && *tick <= 0 {
                *tick = duration;
            } else if !self.reverse && *tick >= duration {
                *tick = 0;
            }
            self.fraction = 0.;
        }
        self.playing = !self.playing;
    }
    pub fn copy(&mut self, asset: &TimelineAsset) {
        self.clipboard = asset
            .tracks
            .iter()
            .flat_map(|t| {
                t.keys
                    .iter()
                    .filter(|k| self.selected.contains(&k.id))
                    .map(|key| CopiedKey {
                        track: t.id,
                        channel: None,
                        time: key.tick,
                        key: key.clone(),
                    })
            })
            .collect();
        for t in &asset.tracks {
            for s in &t.sections {
                for c in &s.channels {
                    for key in &c.keys {
                        if self.selected.contains(&key.id) {
                            self.clipboard.push(CopiedKey {
                                track: t.id,
                                channel: Some(c.id),
                                time: s.sequence_tick(key.tick),
                                key: key.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
    pub fn paste(&mut self, asset: &mut TimelineAsset, tick: i32) {
        let Some(first) = self.clipboard.iter().map(|k| k.time).min() else {
            return;
        };
        // Validate the complete edit before applying it: no partial paste or
        // silent key loss when any destination collides or exceeds the profile.
        let mut candidate = asset.clone();
        let mut selected = BTreeSet::new();
        for source in &self.clipboard {
            let Some(track) = candidate.tracks.iter_mut().find(|t| t.id == source.track) else {
                return;
            };
            let time = i64::from(tick) + i64::from(source.time) - i64::from(first);
            if time < 0 || time > i64::from(asset.duration_ticks) {
                return;
            }
            let (keys, time) = if let Some(channel) = source.channel {
                let Some(section) = track
                    .sections
                    .iter_mut()
                    .find(|s| s.channels.iter().any(|c| c.id == channel))
                else {
                    return;
                };
                if time < i64::from(section.start_tick) || time > i64::from(section.end_tick) {
                    return;
                }
                let time = section.source_tick(time as i32);
                (
                    &mut section
                        .channels
                        .iter_mut()
                        .find(|c| c.id == channel)
                        .unwrap()
                        .keys,
                    time,
                )
            } else {
                (&mut track.keys, time as i32)
            };
            if keys.len() >= timeline::KEY_LIMIT || keys.iter().any(|k| k.tick == time) {
                return;
            }
            let mut key = source.key.clone();
            key.id = Uuid::new_v4();
            key.tick = time;
            selected.insert(key.id);
            keys.push(key);
            keys.sort_by_key(|k| k.tick);
        }
        *asset = candidate;
        self.selected = selected;
    }
    pub fn delete(&mut self, asset: &mut TimelineAsset) {
        // A typed curve needs its endpoints. Keep it valid while deleting a selection.
        for t in &mut asset.tracks {
            if t.keys
                .iter()
                .filter(|k| !self.selected.contains(&k.id))
                .count()
                >= 2
            {
                t.keys.retain(|k| !self.selected.contains(&k.id));
            }
            for section in &mut t.sections {
                for channel in &mut section.channels {
                    if channel.keys.iter().any(|k| !self.selected.contains(&k.id)) {
                        channel.keys.retain(|k| !self.selected.contains(&k.id));
                    }
                }
            }
        }
        for t in &mut asset.events {
            t.keys.retain(|k| !self.selected.contains(&k.id));
        }
        asset.markers.retain(|k| !self.selected.contains(&k.id));
        self.selected.clear();
    }
    pub fn move_keys(
        &self,
        asset: &mut TimelineAsset,
        originals: &[(Uuid, i32)],
        delta: i32,
    ) -> bool {
        let time = |id: Uuid, current: i32| {
            originals
                .iter()
                .find(|k| k.0 == id)
                .map_or(current, |k| k.1.saturating_add(delta))
        };
        let valid = |keys: Vec<(Uuid, i32)>| {
            let times = keys.iter().map(|(id, t)| time(*id, *t)).collect::<Vec<_>>();
            times.iter().all(|t| (0..=asset.duration_ticks).contains(t))
                && times.iter().copied().collect::<BTreeSet<_>>().len() == times.len()
        };
        if !asset
            .tracks
            .iter()
            .all(|t| valid(t.keys.iter().map(|k| (k.id, k.tick)).collect()))
            || !asset
                .events
                .iter()
                .all(|t| valid(t.keys.iter().map(|k| (k.id, k.tick)).collect()))
            || asset
                .markers
                .iter()
                .any(|k| !(0..=asset.duration_ticks).contains(&time(k.id, k.tick)))
        {
            return false;
        }
        // Section channel edits use source time. Validate all affected channels
        // before writing any of them, including rational-rate collision checks.
        let mut changes = Vec::new();
        for t in &asset.tracks {
            for s in &t.sections {
                for c in &s.channels {
                    let mut times = BTreeSet::new();
                    for k in &c.keys {
                        let next =
                            originals
                                .iter()
                                .find(|v| v.0 == k.id)
                                .map_or(i64::from(k.tick), |v| {
                                    i64::from(v.1)
                                        + i64::from(delta) * i64::from(s.rate_numerator)
                                            / i64::from(s.rate_denominator.max(1))
                                });
                        if !(0..=i64::from(i32::MAX)).contains(&next) || !times.insert(next) {
                            return false;
                        }
                        changes.push((k.id, next as i32));
                    }
                }
            }
        }
        for t in &mut asset.tracks {
            for k in &mut t.keys {
                k.tick = time(k.id, k.tick);
            }
            t.keys.sort_by_key(|k| k.tick);
        }
        for t in &mut asset.tracks {
            for s in &mut t.sections {
                for c in &mut s.channels {
                    for k in &mut c.keys {
                        if let Some((_, next)) = changes.iter().find(|v| v.0 == k.id) {
                            k.tick = *next;
                        }
                    }
                    c.keys.sort_by_key(|k| k.tick);
                }
            }
        }
        for t in &mut asset.events {
            for k in &mut t.keys {
                k.tick = time(k.id, k.tick);
            }
            t.keys.sort_by_key(|k| k.tick);
        }
        for k in &mut asset.markers {
            k.tick = time(k.id, k.tick);
        }
        true
    }
    fn add_key(&mut self, asset: &mut TimelineAsset, tick: i32) {
        let Some(t) = asset.tracks.iter_mut().find(|t| Some(t.id) == self.track) else {
            return;
        };
        if !t.sections.is_empty() {
            let Some(section) = t
                .sections
                .iter_mut()
                .find(|s| tick >= s.start_tick && tick <= s.end_tick)
            else {
                return;
            };
            let source = section.source_tick(tick);
            let scalar = crate::timeline_section::scalar_type(&t.value_type);
            self.selected.clear();
            for channel in &mut section.channels {
                if self
                    .lane
                    .is_some_and(|lane| lane != usize::from(channel.lane))
                {
                    continue;
                }
                if channel.keys.len() >= timeline::KEY_LIMIT
                    || channel.keys.iter().any(|k| k.tick == source)
                {
                    continue;
                }
                let mut curve = channel
                    .keys
                    .iter()
                    .filter_map(|k| {
                        timeline::pack(&k.value, scalar)
                            .ok()
                            .map(|v| (k.tick, v[0]))
                    })
                    .collect::<Vec<_>>();
                curve.sort_by_key(|k| k.0);
                let raw = crate::timeline_curve::sample_mode(
                    &curve,
                    source,
                    channel.interpolation as u8,
                    matches!(scalar, crate::reflection_schema::Type::UInt32),
                );
                let value = match scalar {
                    crate::reflection_schema::Type::Fixed => serde_json::json!(raw as f64 / 4096.),
                    crate::reflection_schema::Type::Bool => serde_json::json!(raw != 0),
                    crate::reflection_schema::Type::UInt32 => serde_json::json!(raw as u32),
                    _ => serde_json::json!(raw),
                };
                let id = Uuid::new_v4();
                channel.keys.push(timeline::Key {
                    id,
                    tick: source,
                    value,
                    extra: Default::default(),
                });
                channel.keys.sort_by_key(|k| k.tick);
                self.selected.insert(id);
            }
            return;
        }
        if t.keys.len() >= timeline::KEY_LIMIT || t.keys.iter().any(|k| k.tick == tick) {
            return;
        }
        let value = sample_authoring(t, tick);
        let id = Uuid::new_v4();
        t.keys.push(timeline::Key {
            id,
            tick,
            value,
            extra: Default::default(),
        });
        t.keys.sort_by_key(|k| k.tick);
        self.selected = BTreeSet::from([id]);
    }

    pub fn draw(
        &mut self,
        ui: &Ui,
        asset: &mut TimelineAsset,
        registry: &Registry,
        tick: &mut i32,
        scene: &crate::scene::Scene,
        bindings: &mut timeline::Bindings,
    ) -> Actions {
        if self.span <= 0. {
            self.fit(asset.duration_ticks);
            self.track = asset
                .tracks
                .iter()
                .find(|t| t.name.to_lowercase().contains("transform"))
                .or_else(|| asset.tracks.first())
                .map(|t| t.id);
        }
        self.advance(asset, tick, f64::from(ui.io().delta_time));
        let mut actions = Actions::default();
        let _colors = ui.push_style_color(imgui::StyleColor::Text, TEXT);
        let _button = ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.]);
        let _hover = ui.push_style_color(imgui::StyleColor::ButtonHovered, [0.29, 0.28, 0.28, 1.]);
        let _border = ui.push_style_var(imgui::StyleVar::FrameBorderSize(0.));
        let _rounding = ui.push_style_var(imgui::StyleVar::FrameRounding(1.));
        let _space = ui.push_style_var(imgui::StyleVar::ItemSpacing([4., 4.]));
        let _padding = ui.push_style_var(imgui::StyleVar::FramePadding([5., 3.]));
        let origin = ui.cursor_screen_pos();
        let size = ui.content_region_avail();
        let width = size[0].max(120.);
        ui.get_window_draw_list()
            .add_rect(origin, [origin[0] + width, origin[1] + size[1]], BG)
            .filled(true)
            .build();
        ui.set_cursor_screen_pos([origin[0] + 6., origin[1] + 5.]);
        actions.save = tool(ui, "\u{f0c7}", "Save", "Save asset · Ctrl+S");
        ui.same_line();
        actions.undo = tool(ui, "\u{f0e2}", "Undo", "Undo · Ctrl+Z");
        ui.same_line();
        actions.redo = tool(ui, "\u{f01e}", "Redo", "Redo · Ctrl+Y");
        ui.same_line();
        separator(ui);
        if tool(ui, "\u{f049}", "Start", "Go to start") {
            *tick = 0;
            self.playing = false;
        }
        ui.same_line();
        if tool(ui, "\u{f048}", "Previous frame", "Previous frame") {
            *tick = frame_tick(tick_frame(*tick, self.fps) - 1, self.fps);
            self.playing = false;
        }
        ui.same_line();
        if tool(
            ui,
            if self.playing { "\u{f04c}" } else { "\u{f04b}" },
            "Play",
            "Play / pause · Space",
        ) {
            self.toggle_play(asset.duration_ticks, tick);
        }
        ui.same_line();
        if tool(ui, "\u{f051}", "Next frame", "Next frame") {
            *tick = frame_tick(tick_frame(*tick, self.fps) + 1, self.fps).min(asset.duration_ticks);
            self.playing = false;
        }
        ui.same_line();
        if tool(ui, "\u{f050}", "End", "Go to end") {
            *tick = asset.duration_ticks;
            self.playing = false;
        }
        ui.same_line();
        separator(ui);
        if tool(ui, "\u{f04a}", "Reverse", "Toggle reverse playback") {
            self.reverse = !self.reverse;
        }
        ui.same_line();
        if tool(ui, "\u{f363}", "Loop", "Toggle loop playback") {
            asset.loop_mode = if asset.loop_mode == timeline::LoopMode::Repeat {
                timeline::LoopMode::Once
            } else {
                timeline::LoopMode::Repeat
            };
        }
        ui.same_line();
        separator(ui);
        if tool(ui, "\u{f219}", "Add key", "Add key at playhead · Enter") {
            self.add_key(asset, *tick);
        }
        ui.same_line();
        if tool(ui, "\u{f024}", "Add marker", "Add marker at playhead")
            && asset.markers.len() < timeline::MARKER_LIMIT
        {
            asset.markers.push(timeline::Marker {
                id: Uuid::new_v4(),
                name: "Marker".into(),
                tick: *tick,
                extra: Default::default(),
            });
        }
        ui.same_line();
        separator(ui);
        {
            let _active = ui.push_style_color(
                imgui::StyleColor::Button,
                if self.snapping {
                    [0.13, 0.43, 0.71, 1.]
                } else {
                    [0., 0., 0., 0.]
                },
            );
            if tool(ui, "\u{f076}", "Snapping", "Snap to display frames · S") {
                self.snapping = !self.snapping;
            }
        }
        ui.same_line();
        ui.set_next_item_width(80.);
        if let Some(_combo) = ui.begin_combo("##Frame rate", format!("{} fps", self.fps)) {
            for fps in [24, 25, 30, 50, 60] {
                if ui.selectable(format!("{fps} fps")) {
                    self.fps = fps;
                }
            }
        }
        ui.same_line();
        separator(ui);
        if tool(ui, "\u{f201}", "Curve editor", "Show curves / keys") {
            self.curves = !self.curves;
        }
        ui.same_line();
        if tool(ui, "\u{f065}", "Fit", "Fit sequence · F") {
            self.fit(asset.duration_ticks);
        }
        ui.same_line();
        if tool(
            ui,
            "\u{f06e}",
            "Scene preview",
            "Toggle scene preview / restore authored scene",
        ) {
            self.preview_scene = !self.preview_scene;
            if !self.preview_scene {
                self.playing = false;
            }
        }
        ui.same_line();
        actions.details = tool(
            ui,
            "\u{f1de}",
            "Details",
            "Open sequence properties and bindings",
        );
        ui.same_line();
        actions.validate = tool(ui, "\u{f00c}", "Validate", "Validate and cook preview");
        ui.set_cursor_screen_pos([origin[0] + 8., origin[1] + 46.]);
        let status_width = (width * 0.65).max(150.);
        ui.get_window_draw_list().with_clip_rect(
            [origin[0], origin[1] + 42.],
            [origin[0] + status_width, origin[1] + 67.],
            || {
                ui.text_disabled(&self.status);
            },
        );
        if ui.is_item_hovered() {
            ui.tooltip_text(&self.status);
        }
        let crumb = format!("\u{f07c}   {}   \u{eab6}   {}", "Sequences", asset.name);
        let cw = ui.calc_text_size(&crumb)[0];
        ui.set_cursor_screen_pos([
            origin[0] + (width - cw - 12.).max(self.tree_width),
            origin[1] + 46.,
        ]);
        ui.text(&crumb);
        let header = origin[1] + 70.;
        self.tree_width = self.tree_width.clamp(170., (width * 0.55).max(170.));
        ui.set_cursor_screen_pos([origin[0] + 3., header]);
        if ui.button("+ Track") {
            ui.open_popup("Sequencer add track");
        }
        if let Some(_popup) = ui.begin_popup("Sequencer add track") {
            self.add_menu(ui, asset, registry);
        }
        ui.same_line();
        ui.set_next_item_width((self.tree_width - 151.).max(30.));
        ui.input_text("##Search Tracks", &mut self.search)
            .hint("Search Tracks")
            .build();
        ui.same_line();
        ui.text_colored(ORANGE, format!("{:04}", tick_frame(*tick, self.fps)));
        let ruler_y = header;
        let top = header + 31.;
        let bottom = (origin[1] + size[1] - 26.).max(top + 20.);
        let left = origin[0] + self.tree_width;
        let right = origin[0] + width;
        let px = (right - left).max(1.) / self.span as f32;
        let x = |t: i32| left + (f64::from(t) - self.start) as f32 * px;
        let draw = ui.get_window_draw_list();
        draw.add_rect([left, ruler_y], [right, bottom], [0.085, 0.08, 0.08, 1.])
            .filled(true)
            .build();
        let frame_pixels = 4096. / self.fps as f32 * px;
        let step = [1, 2, 5, 10, 15, 30, 60, 120, 300, 600, 1200, 3600, 18000]
            .into_iter()
            .find(|v| *v as f32 * frame_pixels >= 54.)
            .unwrap_or(18000);
        let first = ((self.start * self.fps as f64 / 4096.).floor() as i64).div_euclid(step) * step;
        draw.with_clip_rect([left, ruler_y], [right, bottom], || {
            let mut frame = first;
            while (frame as f64 * 4096. / self.fps as f64) < self.start + self.span {
                let xx = left + (frame as f64 * 4096. / self.fps as f64 - self.start) as f32 * px;
                draw.add_line([xx, ruler_y + 23.], [xx, bottom], [0.17, 0.16, 0.16, 1.])
                    .build();
                draw.add_line([xx, ruler_y + 17.], [xx, ruler_y + 30.], MUTED)
                    .build();
                draw.add_text([xx + 5., ruler_y + 16.], MUTED, format!("{frame:04}"));
                frame += step;
            }
        });
        let rows = self.rows(asset);
        self.scroll = self.scroll.clamp(
            0.,
            (rows.len() as f32 * ROW_HEIGHT - (bottom - top)).max(0.),
        );
        let mouse = ui.io().mouse_pos;
        let mut hovered_key = None;
        let mut hovered_row = None;
        let mut hovered_section = None;
        let mut points = Vec::new();
        draw.with_clip_rect([origin[0], top], [right, bottom], || {
            for (i, row) in rows.iter().enumerate() {
                let y = top + i as f32 * ROW_HEIGHT - self.scroll;
                if y + ROW_HEIGHT < top || y > bottom {
                    continue;
                }
                let chosen = self.track == Some(row.id) && self.lane == row.lane;
                draw.add_rect(
                    [origin[0], y],
                    [left, y + ROW_HEIGHT],
                    if chosen { [0.24, 0.27, 0.27, 1.] } else { ROW },
                )
                .filled(true)
                .build();
                draw.add_rect(
                    [left, y],
                    [right, y + ROW_HEIGHT],
                    if i % 2 == 0 {
                        [0.15, 0.145, 0.145, 1.]
                    } else {
                        [0.17, 0.165, 0.165, 1.]
                    },
                )
                .filled(true)
                .build();
                draw.add_line(
                    [origin[0], y + ROW_HEIGHT],
                    [right, y + ROW_HEIGHT],
                    [0.1, 0.095, 0.095, 1.],
                )
                .build();
                if row.group || row.expandable {
                    draw.add_text(
                        [origin[0] + if row.group { 8. } else { 27. }, y + 5.],
                        MUTED,
                        if self.collapsed.contains(&row.id) {
                            "\u{eab6}"
                        } else {
                            "\u{eab4}"
                        },
                    );
                    if row.group {
                        draw.add_text([origin[0] + 24., y + 5.], TEXT, "\u{f03d}");
                    }
                }
                let indent = if row.group {
                    44.
                } else if row.lane.is_some() {
                    64.
                } else {
                    46.
                };
                draw.with_clip_rect(
                    [origin[0] + indent, y],
                    [left - 165., y + ROW_HEIGHT],
                    || {
                        draw.add_text([origin[0] + indent, y + 5.], TEXT, &row.name);
                    },
                );
                if inside(mouse, [origin[0], y], [right, y + ROW_HEIGHT]) {
                    hovered_row = Some((row.id, row.group, row.lane, row.expandable));
                }
                for &(id, first, last) in &row.sections {
                    let a = x(first).max(left);
                    let b = x(last).min(right);
                    if b > a {
                        draw.add_rect(
                            [a, y + 2.],
                            [b, y + ROW_HEIGHT - 2.],
                            if chosen {
                                TEAL
                            } else {
                                [0.25, 0.25, 0.24, 0.85]
                            },
                        )
                        .filled(true)
                        .build();
                    }
                    if inside(mouse, [a, y + 2.], [b, y + ROW_HEIGHT - 2.]) {
                        let mode = if (mouse[0] - x(first)).abs() < 7. {
                            1
                        } else if (mouse[0] - x(last)).abs() < 7. {
                            2
                        } else {
                            0
                        };
                        hovered_section = Some((id, mode));
                    }
                }
                for &(id, t) in &row.keys {
                    let p = [x(t), y + ROW_HEIGHT / 2.];
                    if p[0] < left || p[0] > right {
                        continue;
                    }
                    points.push((id, p));
                    if row.group {
                        draw.add_line([p[0], y + 5.], [p[0], y + ROW_HEIGHT - 5.], TEXT)
                            .build();
                    } else {
                        draw.add_circle(p, 5.5, [0.12, 0.12, 0.12, 1.])
                            .filled(true)
                            .build();
                        draw.add_circle(
                            p,
                            if self.selected.contains(&id) {
                                4.5
                            } else {
                                3.8
                            },
                            if self.selected.contains(&id) {
                                [1., 0.70, 0.38, 1.]
                            } else {
                                ORANGE
                            },
                        )
                        .filled(true)
                        .build();
                        if (p[0] - mouse[0]).abs() < 7. && (p[1] - mouse[1]).abs() < 9. {
                            hovered_key = Some((id, t, row.id, row.lane));
                        }
                    }
                }
            }
            if rows.is_empty() {
                draw.add_text(
                    [origin[0] + 14., top + 18.],
                    MUTED,
                    "Add a binding and an animatable property to begin.",
                );
            }
        });
        draw.with_clip_rect([left, ruler_y], [right, bottom], || {
            for bound in [0, asset.duration_ticks] {
                let xx = x(bound);
                draw.add_line(
                    [xx, ruler_y],
                    [xx, bottom],
                    if bound == 0 {
                        [0.28, 0.43, 0.40, 1.]
                    } else {
                        [0.54, 0.25, 0.18, 1.]
                    },
                )
                .thickness(2.)
                .build();
            }
            let xx = x(*tick);
            draw.add_rect([xx - 6., ruler_y], [xx + 6., ruler_y + 19.], ORANGE)
                .filled(true)
                .build();
            draw.add_triangle(
                [xx - 6., ruler_y + 19.],
                [xx + 6., ruler_y + 19.],
                [xx, ruler_y + 25.],
                ORANGE,
            )
            .filled(true)
            .build();
            draw.add_line([xx, ruler_y + 23.], [xx, bottom], [0.75, 0.71, 0.66, 0.75])
                .build();
            draw.add_text(
                [xx + 10., ruler_y],
                TEXT,
                format!("{:04}", tick_frame(*tick, self.fps)),
            );
        });
        draw.add_line(
            [left - 1., header],
            [left - 1., bottom],
            [0.09, 0.09, 0.09, 1.],
        )
        .thickness(3.)
        .build();
        // Real ImGui controls keep text editing, focus and keyboard navigation
        // independent of the canvas's pointer gestures.
        for (i, row) in rows.iter().enumerate() {
            let y = top + i as f32 * ROW_HEIGHT - self.scroll;
            if y < top || y + ROW_HEIGHT > bottom {
                continue;
            }
            let _id = ui.push_id(format!("{}:{:?}", row.id, row.lane));
            if row.group {
                let slot = asset.slots.iter().find(|s| s.id == row.id).unwrap();
                let choices = crate::timeline_scene_preview::candidates(slot, scene, registry);
                let current = bindings.get(&slot.id).copied().flatten();
                let name = choices
                    .iter()
                    .find(|c| Some(c.0) == current)
                    .map_or("Bind target...", |c| c.1.as_str());
                ui.set_cursor_screen_pos([left - 157., y + 1.]);
                ui.set_next_item_width(150.);
                if let Some(_combo) = ui.begin_combo("##Binding", name) {
                    if ui.selectable("None") {
                        bindings.insert(slot.id, None);
                    }
                    for (id, name) in &choices {
                        if ui
                            .selectable_config(format!("{name}##{id}"))
                            .selected(Some(*id) == current)
                            .build()
                        {
                            bindings.insert(slot.id, Some(*id));
                        }
                    }
                }
                continue;
            }
            let Some(track) = asset.tracks.iter_mut().find(|t| t.id == row.id) else {
                continue;
            };
            let sampled = sample_track(track, *tick, asset.duration_ticks);
            if row.expandable {
                if let Some(value) = sampled {
                    let text = value
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| format!("{:.2}", v.as_f64().unwrap_or(0.)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    draw.with_clip_rect([left - 157., y], [left - 5., y + ROW_HEIGHT], || {
                        draw.add_text([left - 157., y + 5.], MUTED, text);
                    });
                }
                continue;
            }
            ui.set_cursor_screen_pos([left - 157., y + 1.]);
            ui.set_next_item_width(74.);
            if let Some(mut value) =
                sampled.map(|v| row.lane.map_or_else(|| v.clone(), |lane| v[lane].clone()))
            {
                let ty = crate::timeline_section::scalar_type(&track.value_type);
                if crate::script_values::inspector(ui, "##Value", &mut value, ty) {
                    self.playing = false;
                    self.track = Some(row.id);
                    self.lane = row.lane;
                    set_value(track, *tick, asset.duration_ticks, row.lane, value);
                }
                #[cfg(test)]
                crate::timeline_editor::record_control(
                    ui,
                    &format!("Value:{}:{:?}", row.id, row.lane),
                );
            } else {
                ui.text_disabled("—");
            }
            for (offset, label, action) in [(79., "<", -1), (54., "+", 0), (29., ">", 1)] {
                ui.set_cursor_screen_pos([left - offset, y + 1.]);
                if ui.button_with_size(label, [23., ROW_HEIGHT - 2.]) {
                    self.playing = false;
                    self.track = Some(row.id);
                    self.lane = row.lane;
                    if action == 0 {
                        self.add_key(asset, *tick);
                    } else if let Some(time) = adjacent_key(&row.keys, *tick, action > 0) {
                        *tick = time;
                    }
                }
                #[cfg(test)]
                crate::timeline_editor::record_control(
                    ui,
                    &format!("Key:{action}:{}:{:?}", row.id, row.lane),
                );
                if ui.is_item_hovered() {
                    ui.tooltip_text(match action {
                        -1 => "Previous key",
                        0 => "Add key at playhead",
                        _ => "Next key",
                    });
                }
            }
        }
        if let Some((id, t, _, _)) = hovered_key {
            ui.tooltip_text(format!(
                "Frame {} · {:.4} s\n{}",
                tick_frame(t, self.fps),
                t as f64 / 4096.,
                id
            ));
        }
        let canvas_hover = inside(mouse, [left, ruler_y], [right, bottom])
            && ui.is_window_hovered_with_flags(imgui::WindowHoveredFlags::ROOT_AND_CHILD_WINDOWS)
            && !ui.is_any_item_active();
        if canvas_hover && ui.io().mouse_wheel != 0. {
            if ui.io().key_ctrl {
                let at = self.start + f64::from((mouse[0] - left) / px);
                let ratio = f64::from((mouse[0] - left) / (right - left));
                self.span = (self.span * 0.8f64.powf(f64::from(ui.io().mouse_wheel)))
                    .clamp(4096. / self.fps as f64 * 5., i32::MAX as f64);
                self.start = at - ratio * self.span;
            } else if ui.io().key_shift {
                self.start -= f64::from(ui.io().mouse_wheel) * self.span * 0.1;
            } else {
                self.scroll -= ui.io().mouse_wheel * ROW_HEIGHT * 3.;
            }
        }
        if canvas_hover && ui.is_mouse_dragging(MouseButton::Middle) {
            self.start -= f64::from(ui.io().mouse_delta[0] / px);
        }
        if canvas_hover && ui.is_mouse_clicked(MouseButton::Left) {
            self.playing = false;
            if mouse[1] < top {
                *tick = self.snapped_to_asset(
                    asset,
                    self.start + f64::from((mouse[0] - left) / px),
                    f64::from(8. / px),
                    false,
                );
            } else if let Some((id, _, track, lane)) = hovered_key {
                self.track = Some(track);
                self.lane = lane;
                if ui.io().key_ctrl {
                    if !self.selected.insert(id) {
                        self.selected.remove(&id);
                    }
                } else if !self.selected.contains(&id) {
                    self.selected = BTreeSet::from([id]);
                }
                self.drag = Some((
                    mouse[0],
                    source_keys(asset)
                        .into_iter()
                        .filter(|k| self.selected.contains(&k.0))
                        .collect(),
                ));
            } else if let Some((id, mode)) = hovered_section
                && (mode != 0 || ui.io().key_alt)
                && let Some(original) = asset
                    .tracks
                    .iter()
                    .flat_map(|t| &t.sections)
                    .find(|s| s.id == id)
            {
                self.section_drag = Some(SectionDrag {
                    id,
                    mode,
                    anchor: mouse[0],
                    original: original.clone(),
                });
            } else {
                let old = if ui.io().key_ctrl {
                    self.selected.clone()
                } else {
                    BTreeSet::new()
                };
                self.selected = old.clone();
                self.box_select = Some((mouse, old));
                if let Some((id, false, lane, _)) = hovered_row {
                    self.track = Some(id);
                    self.lane = lane;
                }
                if ui.is_mouse_double_clicked(MouseButton::Left) {
                    let time = self.snapped_to_asset(
                        asset,
                        self.start + f64::from((mouse[0] - left) / px),
                        f64::from(8. / px),
                        false,
                    );
                    self.add_key(asset, time);
                    self.box_select = None;
                }
            }
        }
        if ui.is_mouse_down(MouseButton::Left)
            && let Some((anchor, originals)) = &self.drag
        {
            let raw = ((mouse[0] - anchor) / px).round() as i32;
            let delta = if self.snapping {
                let first = originals.iter().map(|k| k.1).min().unwrap_or(0);
                self.snapped_to_asset(
                    asset,
                    f64::from(first) + f64::from(raw),
                    f64::from(8. / px),
                    true,
                ) - first
            } else {
                raw
            };
            self.move_keys(asset, originals, delta);
        }
        if ui.is_mouse_down(MouseButton::Left)
            && let Some(drag) = &self.section_drag
        {
            let raw = ((mouse[0] - drag.anchor) / px).round() as i32;
            let anchor = if drag.mode == 2 {
                drag.original.end_tick
            } else {
                drag.original.start_tick
            };
            let delta = self.snapped_to_asset(
                asset,
                f64::from(anchor) + f64::from(raw),
                f64::from(8. / px),
                false,
            ) - anchor;
            let mut candidate = asset.clone();
            if let Some(track) = candidate
                .tracks
                .iter_mut()
                .find(|t| t.sections.iter().any(|s| s.id == drag.id))
            {
                let s = track.sections.iter_mut().find(|s| s.id == drag.id).unwrap();
                *s = drag.original.clone();
                match drag.mode {
                    1 => {
                        s.start_tick += delta;
                        s.source_offset_tick = s.source_offset_tick.saturating_add(
                            (i64::from(delta) * i64::from(s.rate_numerator)
                                / i64::from(s.rate_denominator.max(1)))
                                as i32,
                        );
                    }
                    2 => s.end_tick += delta,
                    _ => {
                        s.start_tick += delta;
                        s.end_tick = s.end_tick.saturating_add(delta);
                    }
                }
                if crate::timeline_section::validate(track, candidate.duration_ticks).is_empty() {
                    *asset = candidate;
                }
            }
        }
        if ui.is_mouse_down(MouseButton::Left)
            && let Some((anchor, old)) = &self.box_select
        {
            let a = [anchor[0].min(mouse[0]), anchor[1].min(mouse[1])];
            let b = [anchor[0].max(mouse[0]), anchor[1].max(mouse[1])];
            self.selected = old.clone();
            self.selected.extend(
                points
                    .iter()
                    .filter(|(_, p)| inside(*p, a, b))
                    .map(|(id, _)| *id),
            );
            draw.add_rect(a, b, [0.22, 0.52, 0.75, 0.2])
                .filled(true)
                .build();
            draw.add_rect(a, b, [0.35, 0.62, 0.8, 0.8]).build();
        }
        if inside(mouse, [left, ruler_y], [right, top])
            && ui.is_mouse_dragging(MouseButton::Left)
            && self.drag.is_none()
            && self.box_select.is_none()
        {
            *tick = self.snapped_to_asset(
                asset,
                self.start + f64::from((mouse[0] - left) / px),
                f64::from(8. / px),
                false,
            );
        }
        if inside(mouse, [origin[0], top], [left, bottom])
            && ui.is_mouse_clicked(MouseButton::Left)
            && ui.is_window_hovered()
            && !ui.is_any_item_hovered()
            && !ui.is_any_item_active()
        {
            if let Some((id, group, lane, expandable)) = hovered_row {
                if (group || expandable) && mouse[0] < origin[0] + 44. {
                    if !self.collapsed.insert(id) {
                        self.collapsed.remove(&id);
                    }
                } else {
                    self.track = Some(id);
                    self.lane = lane;
                }
            }
        }
        if !ui.is_mouse_down(MouseButton::Left) {
            self.drag = None;
            self.section_drag = None;
            self.box_select = None;
        }
        self.gesture = self.drag.is_some() || self.section_drag.is_some();
        self.gesture |= ui.is_any_item_active();
        if canvas_hover && ui.is_mouse_clicked(MouseButton::Right) {
            ui.open_popup("Sequencer key menu");
        }
        if let Some(_popup) = ui.begin_popup("Sequencer key menu") {
            if ui.menu_item("Add key at playhead") {
                self.add_key(asset, *tick);
            }
            if ui.menu_item("Copy keys") {
                self.copy(asset);
            }
            if ui.menu_item("Paste keys") {
                self.paste(asset, *tick);
            }
            if ui.menu_item("Delete keys") {
                self.delete(asset);
            }
            ui.separator();
            if let Some(track) = asset.tracks.iter_mut().find(|t| Some(t.id) == self.track) {
                if ui.menu_item("Edit sections and channels") {
                    track.make_section(asset.duration_ticks);
                    self.sections_open = true;
                }
                for mode in [
                    timeline::Interpolation::Linear,
                    timeline::Interpolation::Step,
                    timeline::Interpolation::Smoothstep,
                    timeline::Interpolation::EaseIn,
                    timeline::Interpolation::EaseOut,
                ] {
                    if ui.menu_item(format!("{mode:?}")) {
                        track.interpolation = mode;
                    }
                }
            }
        }
        let focused =
            ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS);
        if focused && !ui.io().want_text_input && !ui.is_any_item_active() {
            if ui.io().key_ctrl || ui.io().key_super {
                if ui.is_key_pressed(imgui::Key::C) {
                    self.copy(asset);
                }
                if ui.is_key_pressed(imgui::Key::V) {
                    self.paste(asset, *tick);
                }
                actions.save |= ui.is_key_pressed(imgui::Key::S);
                actions.undo |= ui.is_key_pressed(imgui::Key::Z) && !ui.io().key_shift;
                actions.redo |= ui.is_key_pressed(imgui::Key::Y)
                    || (ui.is_key_pressed(imgui::Key::Z) && ui.io().key_shift);
            } else {
                if ui.is_key_pressed(imgui::Key::Space) {
                    self.toggle_play(asset.duration_ticks, tick);
                }
                if ui.is_key_pressed(imgui::Key::F) {
                    self.fit(asset.duration_ticks);
                }
                if ui.is_key_pressed(imgui::Key::S) {
                    self.snapping = !self.snapping;
                }
                if ui.is_key_pressed(imgui::Key::Delete) || ui.is_key_pressed(imgui::Key::Backspace)
                {
                    self.delete(asset);
                }
                if ui.is_key_pressed(imgui::Key::Enter) {
                    self.add_key(asset, *tick);
                }
                if ui.is_key_pressed(imgui::Key::LeftArrow) {
                    *tick = frame_tick(tick_frame(*tick, self.fps) - 1, self.fps);
                    self.playing = false;
                }
                if ui.is_key_pressed(imgui::Key::RightArrow) {
                    *tick = frame_tick(tick_frame(*tick, self.fps) + 1, self.fps)
                        .min(asset.duration_ticks);
                    self.playing = false;
                }
            }
        }
        ui.set_cursor_screen_pos([origin[0] + 9., bottom + 6.]);
        ui.text_disabled(format!(
            "{} tracks   ·   {} selected",
            asset.tracks.len() + asset.events.len(),
            self.selected.len()
        ));
        ui.set_cursor_screen_pos([left + 8., bottom + 6.]);
        ui.text_disabled(format!(
            "{:.3} s / {:.3} s     Ctrl+wheel zoom  ·  Middle drag pan",
            *tick as f64 / 4096.,
            asset.duration_ticks as f64 / 4096.
        ));
        if self.curves {
            self.curve_window(ui, asset, *tick);
        }
        if self.sections_open {
            self.section_window(ui, asset);
        }
        actions
    }

    fn rows(&self, asset: &TimelineAsset) -> Vec<Row> {
        let mut rows = Vec::new();
        let query = self.search.to_lowercase();
        for slot in &asset.slots {
            let tracks = asset
                .tracks
                .iter()
                .filter(|t| {
                    t.slot == slot.id
                        && (query.is_empty()
                            || t.name.to_lowercase().contains(&query)
                            || slot.name.to_lowercase().contains(&query))
                })
                .collect::<Vec<_>>();
            let events = asset
                .events
                .iter()
                .filter(|t| {
                    t.slot == slot.id
                        && (query.is_empty()
                            || t.name.to_lowercase().contains(&query)
                            || slot.name.to_lowercase().contains(&query))
                })
                .collect::<Vec<_>>();
            if !query.is_empty() && tracks.is_empty() && events.is_empty() {
                continue;
            }
            let keys = tracks
                .iter()
                .flat_map(|t| track_keys(t))
                .chain(
                    events
                        .iter()
                        .flat_map(|t| t.keys.iter().map(|k| (k.id, k.tick))),
                )
                .collect();
            rows.push(Row {
                id: slot.id,
                name: slot.name.clone(),
                group: true,
                lane: None,
                expandable: false,
                keys,
                sections: vec![],
            });
            if self.collapsed.contains(&slot.id) {
                continue;
            }
            for t in tracks {
                let vector = matches!(t.value_type, crate::reflection_schema::Type::Vector { .. });
                rows.push(Row {
                    id: t.id,
                    name: t.name.clone(),
                    group: false,
                    lane: None,
                    expandable: vector,
                    keys: track_keys(t),
                    sections: if t.sections.is_empty() {
                        vec![(t.id, 0, asset.duration_ticks)]
                    } else {
                        t.sections
                            .iter()
                            .map(|s| (s.id, s.start_tick, s.end_tick))
                            .collect()
                    },
                });
                if vector && !self.collapsed.contains(&t.id) {
                    for lane in 0..timeline::channels(&t.value_type) {
                        rows.push(Row {
                            id: t.id,
                            name: ["X", "Y", "Z"][lane].into(),
                            group: false,
                            lane: Some(lane),
                            expandable: false,
                            keys: if t.sections.is_empty() {
                                track_keys(t)
                            } else {
                                t.sections
                                    .iter()
                                    .flat_map(|s| {
                                        s.channels
                                            .iter()
                                            .filter(move |c| usize::from(c.lane) == lane)
                                            .flat_map(move |c| {
                                                c.keys
                                                    .iter()
                                                    .map(move |k| (k.id, s.sequence_tick(k.tick)))
                                                    .filter(move |(_, tick)| {
                                                        *tick >= s.start_tick && *tick <= s.end_tick
                                                    })
                                            })
                                    })
                                    .collect()
                            },
                            sections: t
                                .ranges(asset.duration_ticks)
                                .into_iter()
                                .map(|(a, b)| (t.id, a, b))
                                .collect(),
                        });
                    }
                }
            }
            for t in events {
                rows.push(Row {
                    id: t.id,
                    name: format!("\u{f0e7}  {}", t.name),
                    group: false,
                    lane: None,
                    expandable: false,
                    keys: t.keys.iter().map(|k| (k.id, k.tick)).collect(),
                    sections: vec![],
                });
            }
        }
        if !asset.markers.is_empty() {
            rows.push(Row {
                id: asset.id,
                name: "Markers".into(),
                group: false,
                lane: None,
                expandable: false,
                keys: asset.markers.iter().map(|k| (k.id, k.tick)).collect(),
                sections: vec![],
            });
        }
        rows
    }
    fn add_menu(&mut self, ui: &Ui, asset: &mut TimelineAsset, registry: &Registry) {
        if let Some(_menu) = ui.begin_menu("New binding") {
            for class in registry
                .classes
                .values()
                .filter(|c| c.id != crate::particle_effect::LAYER_CLASS_ID)
            {
                if ui.menu_item(&class.cpp_name) && asset.slots.len() < timeline::SLOT_LIMIT {
                    asset.slots.push(timeline::Slot {
                        id: Uuid::new_v4(),
                        name: class.cpp_name.clone(),
                        target: crate::reflection_schema::Type::ObjectRef {
                            class: Some(class.id.clone()),
                        },
                        required: true,
                        extra: Default::default(),
                    });
                }
            }
        }
        ui.separator();
        for slot in &asset.slots {
            if let Some(_menu) = ui.begin_menu(format!("{}##{}", slot.name, slot.id))
                && let Some(class) = slot.class_id().and_then(|id| registry.classes.get(id))
            {
                for p in registry
                    .properties(&class.cpp_name)
                    .into_iter()
                    .filter(|p| p.timeline.is_some())
                {
                    if ui.menu_item(&p.name)
                        && asset.tracks.len() + asset.events.len() < timeline::TRACK_LIMIT
                    {
                        let id = Uuid::new_v4();
                        asset.tracks.push(timeline::Track {
                            sections: vec![],
                            id,
                            name: p.name.clone(),
                            slot: slot.id,
                            property: p.id.clone(),
                            value_type: p.value_type.clone(),
                            priority: 0,
                            blend: timeline::Blend::Absolute,
                            restore: timeline::Restore::LeaveFinal,
                            interpolation: if matches!(
                                p.value_type,
                                crate::reflection_schema::Type::Bool
                                    | crate::reflection_schema::Type::Enum { .. }
                            ) {
                                timeline::Interpolation::Step
                            } else {
                                timeline::Interpolation::Linear
                            },
                            keys: vec![
                                timeline::Key {
                                    id: Uuid::new_v4(),
                                    tick: 0,
                                    value: p.default.clone(),
                                    extra: Default::default(),
                                },
                                timeline::Key {
                                    id: Uuid::new_v4(),
                                    tick: asset.duration_ticks,
                                    value: p.default.clone(),
                                    extra: Default::default(),
                                },
                            ],
                            extra: Default::default(),
                        });
                        self.track = Some(id);
                    }
                }
                ui.separator();
                for f in registry
                    .ancestry(&class.cpp_name)
                    .into_iter()
                    .rev()
                    .flat_map(|c| &c.functions)
                    .filter(|f| f.timeline.is_some())
                {
                    if ui.menu_item(format!("Event: {}", f.name))
                        && asset.tracks.len() + asset.events.len() < timeline::TRACK_LIMIT
                    {
                        asset.events.push(timeline::EventTrack {
                            id: Uuid::new_v4(),
                            name: f.name.clone(),
                            slot: slot.id,
                            function: f.id.clone(),
                            keys: vec![],
                            extra: Default::default(),
                        });
                    }
                }
            }
        }
    }
    fn curve_window(&mut self, ui: &Ui, asset: &mut TimelineAsset, tick: i32) {
        let mut open = true;
        ui.window("Curve Editor###SequencerCurves")
            .opened(&mut open)
            .size([640., 320.], imgui::Condition::FirstUseEver)
            .build(|| {
                let Some(t) = asset.tracks.iter_mut().find(|t| Some(t.id) == self.track) else {
                    ui.text_disabled("Select a property track in Sequencer.");
                    return;
                };
                ui.text(&t.name);
                ui.same_line();
                if let Some(_combo) =
                    ui.begin_combo("Interpolation", format!("{:?}", t.interpolation))
                {
                    for mode in [
                        timeline::Interpolation::Linear,
                        timeline::Interpolation::Step,
                        timeline::Interpolation::Smoothstep,
                        timeline::Interpolation::EaseIn,
                        timeline::Interpolation::EaseOut,
                    ] {
                        if ui.selectable(format!("{mode:?}")) {
                            t.interpolation = mode;
                        }
                    }
                }
                let origin = ui.cursor_screen_pos();
                let size = [ui.content_region_avail()[0], 150.];
                let draw = ui.get_window_draw_list();
                draw.add_rect(origin, [origin[0] + size[0], origin[1] + size[1]], BG)
                    .filled(true)
                    .build();
                let mut keys = t
                    .keys
                    .iter()
                    .filter_map(|k| {
                        timeline::pack(&k.value, &t.value_type)
                            .ok()
                            .map(|v| (k.tick, v))
                    })
                    .collect::<Vec<_>>();
                keys.sort_by_key(|k| k.0);
                for lane in 0..timeline::channels(&t.value_type) {
                    let curve = keys
                        .iter()
                        .map(|(time, v)| (*time, v[lane]))
                        .collect::<Vec<_>>();
                    let min = curve.iter().map(|k| k.1).min().unwrap_or(0) as f64;
                    let max = curve.iter().map(|k| k.1).max().unwrap_or(1) as f64;
                    let point = |time: i32, value: i32| {
                        [
                            origin[0] + time as f32 / asset.duration_ticks.max(1) as f32 * size[0],
                            origin[1] + size[1]
                                - 12.
                                - ((value as f64 - min) / (max - min).max(1.)) as f32
                                    * (size[1] - 24.),
                        ]
                    };
                    let color = [
                        [0.86, 0.36, 0.3, 1.],
                        [0.4, 0.73, 0.4, 1.],
                        [0.36, 0.57, 0.88, 1.],
                    ][lane.min(2)];
                    let points = (0..=256)
                        .map(|i| {
                            let time = (i64::from(asset.duration_ticks) * i / 256) as i32;
                            point(
                                time,
                                crate::timeline_curve::sample_mode(
                                    &curve,
                                    time,
                                    t.interpolation as u8,
                                    matches!(t.value_type, crate::reflection_schema::Type::UInt32),
                                ),
                            )
                        })
                        .collect::<Vec<_>>();
                    draw.add_polyline(points, color).thickness(1.5).build();
                    for key in &curve {
                        draw.add_circle(point(key.0, key.1), 3., color)
                            .filled(true)
                            .build();
                    }
                }
                let x = origin[0] + tick as f32 / asset.duration_ticks.max(1) as f32 * size[0];
                draw.add_line([x, origin[1]], [x, origin[1] + size[1]], TEXT)
                    .build();
                ui.dummy(size);
                for key in &mut t.keys {
                    if self.selected.contains(&key.id) {
                        let _id = ui.push_id(key.id.to_string());
                        ui.text(format!("Frame {}", tick_frame(key.tick, self.fps)));
                        ui.same_line();
                        crate::script_values::inspector(ui, "Value", &mut key.value, &t.value_type);
                    }
                }
            });
        self.curves = open;
    }
    fn section_window(&mut self, ui: &Ui, asset: &mut TimelineAsset) {
        let mut open = true;
        ui.window("Sections and Channels###SequencerSections")
            .opened(&mut open)
            .size([520., 540.], imgui::Condition::FirstUseEver)
            .build(|| {
                let Some(track) = asset.tracks.iter_mut().find(|t| Some(t.id) == self.track) else {
                    ui.text_disabled("Select a property track.");
                    return;
                };
                ui.text(&track.name);
                ui.text_disabled("Drag section edges to trim. Alt+drag the body to move.");
                let scalar = crate::timeline_section::scalar_type(&track.value_type).clone();
                let mut remove = None;
                let mut duplicate = None;
                for (i, section) in track.sections.iter_mut().enumerate() {
                    let _id = ui.push_id(section.id.to_string());
                    if !ui.collapsing_header(
                        format!("Section {}", i + 1),
                        imgui::TreeNodeFlags::DEFAULT_OPEN,
                    ) {
                        continue;
                    }
                    let before = section.clone();
                    let mut start = tick_frame(section.start_tick, self.fps) as i32;
                    let mut end = tick_frame(section.end_tick, self.fps) as i32;
                    if crate::gui::Drag::new("Start frame")
                        .speed(1.)
                        .build(ui, &mut start)
                    {
                        section.start_tick = frame_tick(i64::from(start), self.fps);
                    }
                    if crate::gui::Drag::new("End frame")
                        .speed(1.)
                        .build(ui, &mut end)
                    {
                        section.end_tick =
                            frame_tick(i64::from(end), self.fps).min(asset.duration_ticks);
                    }
                    crate::gui::Drag::new("Source offset (ticks)")
                        .speed(1.)
                        .range(0, i32::MAX)
                        .build(ui, &mut section.source_offset_tick);
                    crate::gui::Drag::new("Speed numerator")
                        .speed(1.)
                        .range(1, 1024)
                        .build(ui, &mut section.rate_numerator);
                    crate::gui::Drag::new("Speed denominator")
                        .speed(1.)
                        .range(1, 1024)
                        .build(ui, &mut section.rate_denominator);
                    if section.end_tick <= section.start_tick {
                        section.start_tick = before.start_tick;
                        section.end_tick = before.end_tick;
                    }
                    for channel in &mut section.channels {
                        let _channel = ui.push_id(channel.id.to_string());
                        ui.separator();
                        ui.text(format!(
                            "Channel {}",
                            ["X", "Y", "Z", "W"][usize::from(channel.lane).min(3)]
                        ));
                        if let Some(_combo) =
                            ui.begin_combo("Interpolation", format!("{:?}", channel.interpolation))
                        {
                            for mode in [
                                timeline::Interpolation::Linear,
                                timeline::Interpolation::Step,
                                timeline::Interpolation::Smoothstep,
                                timeline::Interpolation::EaseIn,
                                timeline::Interpolation::EaseOut,
                            ] {
                                if ui.selectable(format!("{mode:?}")) {
                                    channel.interpolation = mode;
                                }
                            }
                        }
                        let mut delete = None;
                        for (k, key) in channel.keys.iter_mut().enumerate() {
                            let _key = ui.push_id(key.id.to_string());
                            ui.set_next_item_width(100.);
                            crate::gui::Drag::new("Tick")
                                .speed(1.)
                                .range(0, i32::MAX)
                                .build(ui, &mut key.tick);
                            ui.same_line();
                            ui.set_next_item_width(140.);
                            crate::script_values::inspector(ui, "Value", &mut key.value, &scalar);
                            ui.same_line();
                            if ui.small_button("Remove") {
                                delete = Some(k);
                            }
                        }
                        if let Some(k) = delete
                            && channel.keys.len() > 1
                        {
                            channel.keys.remove(k);
                        }
                        channel.keys.sort_by_key(|k| k.tick);
                    }
                    if ui.small_button("Duplicate after this section") {
                        duplicate = Some(i);
                    }
                    ui.same_line();
                    if ui.small_button("Remove section") {
                        remove = Some(i);
                    }
                }
                if let Some(i) = duplicate {
                    let mut section = track.sections[i].clone();
                    let duration = section.end_tick - section.start_tick;
                    section.start_tick = section.end_tick;
                    section.end_tick = section
                        .end_tick
                        .saturating_add(duration)
                        .min(asset.duration_ticks);
                    if section.end_tick > section.start_tick
                        && !track.sections.iter().any(|s| {
                            s.start_tick < section.end_tick && section.start_tick < s.end_tick
                        })
                    {
                        section.id = Uuid::new_v4();
                        for c in &mut section.channels {
                            c.id = Uuid::new_v4();
                            for k in &mut c.keys {
                                k.id = Uuid::new_v4();
                            }
                        }
                        track.sections.push(section);
                    }
                }
                if let Some(i) = remove
                    && track.sections.len() > 1
                {
                    track.sections.remove(i);
                }
                for (_, error) in crate::timeline_section::validate(track, asset.duration_ticks) {
                    ui.text_colored(ORANGE, error);
                }
            });
        self.sections_open = open;
    }
}
struct Row {
    id: Uuid,
    name: String,
    group: bool,
    lane: Option<usize>,
    expandable: bool,
    keys: Vec<(Uuid, i32)>,
    sections: Vec<(Uuid, i32, i32)>,
}
fn track_keys(track: &timeline::Track) -> Vec<(Uuid, i32)> {
    track
        .keys
        .iter()
        .map(|k| (k.id, k.tick))
        .chain(track.sections.iter().flat_map(|s| {
            s.channels.iter().flat_map(move |c| {
                c.keys
                    .iter()
                    .map(move |k| (k.id, s.sequence_tick(k.tick)))
                    .filter(move |(_, t)| *t >= s.start_tick && *t <= s.end_tick)
            })
        }))
        .collect()
}
fn source_keys(asset: &TimelineAsset) -> Vec<(Uuid, i32)> {
    asset
        .tracks
        .iter()
        .flat_map(|t| {
            t.keys.iter().chain(
                t.sections
                    .iter()
                    .flat_map(|s| s.channels.iter().flat_map(|c| &c.keys)),
            )
        })
        .map(|k| (k.id, k.tick))
        .chain(
            asset
                .events
                .iter()
                .flat_map(|t| t.keys.iter().map(|k| (k.id, k.tick))),
        )
        .chain(asset.markers.iter().map(|k| (k.id, k.tick)))
        .collect()
}
fn inside(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> bool {
    p[0] >= a[0] && p[0] < b[0] && p[1] >= a[1] && p[1] < b[1]
}
fn separator(ui: &Ui) {
    let p = ui.cursor_screen_pos();
    ui.get_window_draw_list()
        .add_line(
            [p[0] + 3., p[1]],
            [p[0] + 3., p[1] + 25.],
            [0.08, 0.08, 0.08, 1.],
        )
        .build();
    ui.dummy([9., 25.]);
    ui.same_line();
}
fn tool(ui: &Ui, glyph: &str, id: &str, tip: &str) -> bool {
    let _id = ui.push_id(id);
    let result = ui.button_with_size(glyph, [29., 28.]);
    #[cfg(test)]
    crate::timeline_editor::record_control(ui, id);
    if ui.is_item_hovered() {
        ui.tooltip_text(tip);
    }
    result
}
fn sample_authoring(track: &timeline::Track, tick: i32) -> serde_json::Value {
    let mut keys = track
        .keys
        .iter()
        .filter_map(|k| {
            timeline::pack(&k.value, &track.value_type)
                .ok()
                .map(|v| (k.tick, v))
        })
        .collect::<Vec<_>>();
    keys.sort_by_key(|k| k.0);
    let lane = |i: usize| {
        crate::timeline_curve::sample_mode(
            &keys.iter().map(|(t, v)| (*t, v[i])).collect::<Vec<_>>(),
            tick,
            track.interpolation as u8,
            matches!(track.value_type, crate::reflection_schema::Type::UInt32),
        )
    };
    match &track.value_type {
        crate::reflection_schema::Type::Fixed => serde_json::json!(lane(0) as f64 / 4096.),
        crate::reflection_schema::Type::Vector { .. } => serde_json::json!(
            (0..timeline::channels(&track.value_type))
                .map(|i| lane(i) as f64 / 4096.)
                .collect::<Vec<_>>()
        ),
        crate::reflection_schema::Type::Bool => serde_json::json!(lane(0) != 0),
        crate::reflection_schema::Type::UInt32 => serde_json::json!(lane(0) as u32),
        _ => serde_json::json!(lane(0)),
    }
}

fn adjacent_key(keys: &[(Uuid, i32)], tick: i32, next: bool) -> Option<i32> {
    if next {
        keys.iter().map(|k| k.1).filter(|t| *t > tick).min()
    } else {
        keys.iter().map(|k| k.1).filter(|t| *t < tick).max()
    }
}

fn sample_track(track: &timeline::Track, tick: i32, duration: i32) -> Option<serde_json::Value> {
    if track.sections.is_empty() {
        return Some(sample_authoring(track, tick));
    }
    let section = track.sections.iter().find(|s| {
        tick >= s.start_tick && (tick < s.end_tick || tick == duration && tick == s.end_tick)
    })?;
    let scalar = crate::timeline_section::scalar_type(&track.value_type);
    let mut values = vec![serde_json::Value::Null; timeline::channels(&track.value_type)];
    for channel in &section.channels {
        let mut keys = channel
            .keys
            .iter()
            .map(|k| timeline::pack(&k.value, scalar).map(|v| (k.tick, v[0])))
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        keys.sort_by_key(|k| k.0);
        let raw = crate::timeline_curve::sample_mode(
            &keys,
            section.source_tick(tick),
            channel.interpolation as u8,
            matches!(scalar, crate::reflection_schema::Type::UInt32),
        );
        let value = match scalar {
            crate::reflection_schema::Type::Fixed => serde_json::json!(raw as f64 / 4096.),
            crate::reflection_schema::Type::Bool => serde_json::json!(raw != 0),
            crate::reflection_schema::Type::UInt32 => serde_json::json!(raw as u32),
            _ => serde_json::json!(raw),
        };
        *values.get_mut(usize::from(channel.lane))? = value;
    }
    if matches!(
        track.value_type,
        crate::reflection_schema::Type::Vector { .. }
    ) {
        Some(serde_json::json!(values))
    } else {
        values.into_iter().next()
    }
}

/// Editing a displayed value records a key at the playhead. A legacy vector key
/// keeps its other lanes; section channels keep independent key times and IDs.
fn set_value(
    track: &mut timeline::Track,
    tick: i32,
    duration: i32,
    lane: Option<usize>,
    value: serde_json::Value,
) -> bool {
    let Some(mut sampled) = sample_track(track, tick, duration) else {
        return false;
    };
    let (keys, time, value) = if track.sections.is_empty() {
        if let Some(lane) = lane {
            sampled[lane] = value;
        } else {
            sampled = value;
        }
        if timeline::pack(&sampled, &track.value_type).is_err() {
            return false;
        }
        (&mut track.keys, tick, sampled)
    } else {
        let scalar = crate::timeline_section::scalar_type(&track.value_type);
        if timeline::pack(&value, scalar).is_err() {
            return false;
        }
        let section = track
            .sections
            .iter_mut()
            .find(|s| {
                tick >= s.start_tick
                    && (tick < s.end_tick || tick == duration && tick == s.end_tick)
            })
            .unwrap();
        let time = section.source_tick(tick);
        let Some(channel) = section
            .channels
            .iter_mut()
            .find(|c| usize::from(c.lane) == lane.unwrap_or(0))
        else {
            return false;
        };
        (&mut channel.keys, time, value)
    };
    if let Some(key) = keys.iter_mut().find(|k| k.tick == time) {
        key.value = value;
    } else {
        if keys.len() >= timeline::KEY_LIMIT {
            return false;
        }
        keys.push(timeline::Key {
            id: Uuid::new_v4(),
            tick: time,
            value,
            extra: Default::default(),
        });
        keys.sort_by_key(|k| k.tick);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset() -> TimelineAsset {
        let mut asset = TimelineAsset::new("Edit".into());
        asset.tracks.push(timeline::Track {
            sections: vec![],
            id: Uuid::new_v4(),
            name: "Value".into(),
            slot: Uuid::new_v4(),
            property: Uuid::new_v4().to_string(),
            value_type: crate::reflection_schema::Type::Fixed,
            priority: 0,
            blend: timeline::Blend::Absolute,
            restore: timeline::Restore::LeaveFinal,
            interpolation: timeline::Interpolation::Linear,
            keys: [0, 100]
                .into_iter()
                .map(|tick| timeline::Key {
                    id: Uuid::new_v4(),
                    tick,
                    value: serde_json::json!(tick),
                    extra: Default::default(),
                })
                .collect(),
            extra: Default::default(),
        });
        asset
    }

    #[test]
    fn display_frame_round_trip_has_no_30_fps_drift() {
        for frame in 0..10_000 {
            assert_eq!(tick_frame(frame_tick(frame, 30), 30), frame);
        }
    }

    #[test]
    fn track_rows_expose_vector_lanes_and_edits_preserve_other_values() {
        let (mut asset, _, _) = crate::timeline_scene_preview::tests::fixture();
        let view = View::default();
        let rows = view.rows(&asset);
        assert_eq!(rows.iter().filter(|r| r.lane.is_some()).count(), 6);
        let track = &mut asset.tracks[0];
        assert!(set_value(track, 2048, 4096, Some(0), serde_json::json!(7)));
        assert_eq!(track.keys[1].value, serde_json::json!([7, 4.0, -4.0]));
        let id = track.keys[1].id;
        assert!(set_value(track, 2048, 4096, Some(1), serde_json::json!(8)));
        assert_eq!(track.keys[1].id, id);
        assert_eq!(track.keys[1].value, serde_json::json!([7.0, 8, -4.0]));
        assert_eq!(adjacent_key(&track_keys(track), 2048, false), Some(0));
        assert_eq!(adjacent_key(&track_keys(track), 2048, true), Some(4096));
        track.make_section(4096);
        track.sections[0].start_tick = 1024;
        track.sections[0].rate_numerator = 2;
        assert!(sample_track(track, 0, 4096).is_none());
        let untouched = track.sections[0].channels[1].clone();
        assert!(set_value(track, 1536, 4096, Some(0), serde_json::json!(9)));
        assert_eq!(track.sections[0].channels[1], untouched);
        assert!(
            track.sections[0].channels[0]
                .keys
                .iter()
                .any(|k| k.tick == 1024 && k.value == serde_json::json!(9))
        );
    }

    #[test]
    fn multikey_edit_is_atomic_and_clipboard_remaps_ids() {
        let mut asset = asset();
        let mut view = View::default();
        let originals = asset.tracks[0]
            .keys
            .iter()
            .map(|key| (key.id, key.tick))
            .collect::<Vec<_>>();
        view.selected = originals.iter().map(|key| key.0).collect();
        assert!(view.move_keys(&mut asset, &originals, 50));
        assert_eq!(
            asset.tracks[0]
                .keys
                .iter()
                .map(|k| k.tick)
                .collect::<Vec<_>>(),
            [50, 150]
        );

        let first = asset.tracks[0].keys[0].id;
        assert!(!view.move_keys(&mut asset, &[(first, 50)], 100));
        assert_eq!(
            asset.tracks[0]
                .keys
                .iter()
                .map(|k| k.tick)
                .collect::<Vec<_>>(),
            [50, 150]
        );

        view.copy(&asset);
        let old_ids = view.selected.clone();
        view.paste(&mut asset, 300);
        assert_eq!(
            asset.tracks[0]
                .keys
                .iter()
                .map(|k| k.tick)
                .collect::<Vec<_>>(),
            [50, 150, 300, 400]
        );
        assert!(view.selected.is_disjoint(&old_ids));
    }
}
