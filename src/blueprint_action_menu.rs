//! Blueprint action picker. Independent ImGui implementation of the measured
//! Reference interaction/layout: 400x400 content, 5px border padding, category tree.
use super::{NodeKind, record_control};
use imgui::{Key, StyleColor, StyleVar, Ui};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const POPUP: &str = "Blueprint node catalog";

#[derive(Clone, Copy)]
pub(crate) struct Fonts {
    regular: imgui::FontId,
    bold: imgui::FontId,
    title: imgui::FontId,
}

impl Fonts {
    pub(crate) fn load(context: &mut imgui::Context) -> Self {
        let regular = context.fonts().add_font(&[
            imgui::FontSource::TtfData {
                data: include_bytes!("../resources/editor/Roboto-Regular.ttf"),
                size_pixels: 12.,
                config: None,
            },
            imgui::FontSource::TtfData {
                data: include_bytes!("../resources/editor/codicon.ttf"),
                size_pixels: 16.,
                config: Some(imgui::FontConfig {
                    glyph_ranges: imgui::FontGlyphRanges::from_slice(&[0xea60, 0xedff, 0]),
                    glyph_min_advance_x: 16.,
                    glyph_offset: [0., 4.],
                    ..Default::default()
                }),
            },
            imgui::FontSource::TtfData {
                data: include_bytes!("../resources/editor/blueprint-function-icon.ttf"),
                size_pixels: 16.,
                config: Some(imgui::FontConfig {
                    glyph_ranges: imgui::FontGlyphRanges::from_slice(&[0xe900, 0xe900, 0]),
                    glyph_min_advance_x: 16.,
                    glyph_offset: [0., 4.],
                    ..Default::default()
                }),
            },
        ]);
        let bold = context.fonts().add_font(&[imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/Roboto-Bold.ttf"),
            size_pixels: 12.,
            config: None,
        }]);
        let title = context.fonts().add_font(&[imgui::FontSource::TtfData {
            data: include_bytes!("../resources/editor/Roboto-Regular.ttf"),
            size_pixels: 16.,
            config: None,
        }]);
        Self {
            regular,
            bold,
            title,
        }
    }
}

pub(super) struct Action {
    pub path: String,
    pub label: String,
    pub tooltip: String,
    pub kind: NodeKind,
    pub icon: &'static str,
    pub color: [f32; 4],
}

pub(super) struct State {
    pub fonts: Option<Fonts>,
    pub query: String,
    pub context_sensitive: bool,
    expanded: BTreeSet<String>,
    selected: String,
    focus_search: bool,
    open: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            fonts: None,
            query: String::new(),
            context_sensitive: true,
            expanded: BTreeSet::new(),
            selected: String::new(),
            focus_search: false,
            open: false,
        }
    }
}

#[derive(Default)]
struct Category {
    children: BTreeMap<String, Category>,
    actions: Vec<usize>,
}

struct Row {
    key: String,
    label: String,
    depth: usize,
    action: Option<usize>,
    open: bool,
}

impl State {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self, ui: &Ui) {
        self.open = true;
        self.query.clear();
        self.expanded.clear();
        self.selected.clear();
        self.focus_search = true;
        ui.open_popup(POPUP);
    }

    fn rows(&self, actions: &[Action]) -> Vec<Row> {
        let mut root = Category::default();
        let query = self.query.to_lowercase();
        for (index, action) in actions.iter().enumerate() {
            let searchable =
                format!("{} {} {}", action.path, action.label, action.tooltip).to_lowercase();
            if !query
                .split_whitespace()
                .all(|word| searchable.contains(word))
            {
                continue;
            }
            let mut category = &mut root;
            for part in action.path.split(" / ").filter(|s| !s.is_empty()) {
                category = category.children.entry(part.into()).or_default();
            }
            category.actions.push(index);
        }
        let mut rows = Vec::new();
        self.flatten(&root, "", 0, actions, &mut rows);
        rows
    }

    fn flatten(
        &self,
        category: &Category,
        path: &str,
        depth: usize,
        actions: &[Action],
        rows: &mut Vec<Row>,
    ) {
        for (label, child) in &category.children {
            let key = if path.is_empty() {
                label.clone()
            } else {
                format!("{path} / {label}")
            };
            let open = !self.query.trim().is_empty() || self.expanded.contains(&key);
            rows.push(Row {
                key: key.clone(),
                label: label.clone(),
                depth,
                action: None,
                open,
            });
            if open {
                self.flatten(child, &key, depth + 1, actions, rows);
            }
        }
        let mut leaves = category.actions.clone();
        leaves.sort_by_key(|&i| actions[i].label.to_lowercase());
        for index in leaves {
            rows.push(Row {
                key: format!("action:{index}"),
                label: actions[index].label.clone(),
                depth,
                action: Some(index),
                open: false,
            });
        }
    }

    /// The action provider sees the context toggle in the same frame.
    pub fn draw(
        &mut self,
        ui: &Ui,
        provider: impl FnOnce(bool) -> Vec<Action>,
        error: Option<&str>,
    ) -> Option<NodeKind> {
        let _vars = [
            ui.push_style_var(StyleVar::WindowPadding([5., 5.])),
            ui.push_style_var(StyleVar::PopupRounding(0.)),
            ui.push_style_var(StyleVar::PopupBorderSize(1.)),
            ui.push_style_var(StyleVar::ChildBorderSize(0.)),
            ui.push_style_var(StyleVar::ItemSpacing([0., 0.])),
            ui.push_style_var(StyleVar::FramePadding([3., 2.])),
            ui.push_style_var(StyleVar::FrameRounding(8.)),
            ui.push_style_var(StyleVar::ScrollbarSize(9.)),
        ];
        let _colors = [
            ui.push_style_color(
                StyleColor::PopupBg,
                [25. / 255., 25. / 255., 25. / 255., 1.],
            ),
            ui.push_style_color(
                StyleColor::ChildBg,
                [25. / 255., 25. / 255., 25. / 255., 1.],
            ),
            ui.push_style_color(StyleColor::Border, [0.25, 0.25, 0.25, 1.]),
            ui.push_style_color(
                StyleColor::FrameBg,
                [14. / 255., 14. / 255., 14. / 255., 1.],
            ),
            ui.push_style_color(StyleColor::Header, [0.10, 0.30, 0.52, 1.]),
            ui.push_style_color(StyleColor::HeaderHovered, [0.16, 0.26, 0.36, 1.]),
            ui.push_style_color(StyleColor::HeaderActive, [0.10, 0.30, 0.52, 1.]),
            ui.push_style_color(StyleColor::Text, [0.78, 0.78, 0.78, 1.]),
            ui.push_style_color(StyleColor::CheckMark, [0.12, 0.55, 0.85, 1.]),
        ];
        let Some(_popup) = ui.begin_popup(POPUP) else {
            self.open = false;
            return None;
        };
        let _font = self.fonts.map(|fonts| ui.push_font(fonts.regular));
        let mut chosen = None;
        let size = ui.io().display_size.map(|n| 400_f32.min((n - 30.).max(1.)));
        ui.child_window("bp-action-body")
            .size(size)
            .scroll_bar(false)
            .build(|| {
                ui.set_window_font_scale(if self.fonts.is_some() { 1. } else { 0.8 });
                let top = ui.cursor_screen_pos();
                ui.get_window_draw_list()
                    .add_rect(
                        top,
                        [top[0] + size[0], top[1] + 24.],
                        [55. / 255., 55. / 255., 55. / 255., 1.],
                    )
                    .filled(true)
                    .build();
                ui.set_cursor_screen_pos([top[0] + 3., top[1] + 4.]);
                {
                    let _title = self.fonts.map(|fonts| ui.push_font(fonts.title));
                    ui.text(if self.context_sensitive {
                        "All Actions for this Blueprint"
                    } else {
                        "All Possible Actions"
                    });
                }
                ui.set_cursor_screen_pos([top[0] + size[0] - 119., top[1] + 2.]);
                let context_changed = {
                    let _rounding = ui.push_style_var(StyleVar::FrameRounding(0.));
                    ui.checkbox("Context Sensitive", &mut self.context_sensitive)
                };
                record_control(ui, "bp-action-context");
                ui.set_cursor_screen_pos([top[0], top[1] + 27.]);
                if self.focus_search {
                    ui.set_keyboard_focus_here();
                    self.focus_search = false;
                }
                let _width = ui.push_item_width(size[0]);
                let mut changed = {
                    let _padding = ui.push_style_var(StyleVar::FramePadding([20., 3.]));
                    ui.input_text("##bp-action-search", &mut self.query)
                        .hint("Search")
                        .build()
                };
                record_control(ui, "bp-action-search");
                let editing = ui.is_item_active();
                ui.get_window_draw_list().add_text(
                    [top[0] + 2., top[1] + 28.],
                    [0.6, 0.6, 0.6, 1.],
                    "\u{ea6d}",
                );
                if !self.query.is_empty() {
                    ui.set_cursor_screen_pos([top[0] + size[0] - 20., top[1] + 27.]);
                    let _button = ui.push_style_color(StyleColor::Button, [0., 0., 0., 0.]);
                    let _border = ui.push_style_var(StyleVar::FrameBorderSize(0.));
                    if ui.small_button("\u{ea76}##clear-action-search") {
                        self.query.clear();
                        self.focus_search = true;
                        changed = true;
                    }
                    record_control(ui, "bp-action-clear");
                }
                let actions = provider(self.context_sensitive);
                let rows = self.rows(&actions);
                if changed || context_changed || !rows.iter().any(|row| row.key == self.selected) {
                    self.selected = if self.query.is_empty() {
                        String::new()
                    } else {
                        rows.iter()
                            .find(|r| r.action.is_some())
                            .map(|r| r.key.clone())
                            .unwrap_or_default()
                    };
                }
                let mut scroll = changed || context_changed;
                let current = rows.iter().position(|r| r.key == self.selected);
                let next = if ui.is_key_pressed(Key::DownArrow) {
                    Some(current.map_or(0, |i| (i + 1).min(rows.len().saturating_sub(1))))
                } else if ui.is_key_pressed(Key::UpArrow) {
                    Some(current.map_or(rows.len().saturating_sub(1), |i| i.saturating_sub(1)))
                } else {
                    None
                };
                if let Some(row) = next.and_then(|i| rows.get(i)) {
                    self.selected = row.key.clone();
                    scroll = true;
                }
                let enter = ui.is_key_pressed(Key::Enter) || ui.is_key_pressed(Key::KeypadEnter);
                ui.set_cursor_screen_pos([top[0], top[1] + 49.]);
                ui.child_window("bp-action-tree")
                    .size([size[0], (size[1] - 49.).max(1.)])
                    .build(|| {
                        if let Some(error) = error {
                            ui.text_wrapped(format!("Playback marker catalog: {error}"));
                        }
                        if rows.is_empty() {
                            ui.text_disabled("No matching actions");
                        }
                        for row in &rows {
                            let start = ui.cursor_screen_pos();
                            let height = if row.action.is_some() { 24. } else { 18. };
                            let selected = row.key == self.selected;
                            let clicked = ui
                                .selectable_config(format!("##{}", row.key))
                                .close_popups(false)
                                .selected(selected)
                                .size([ui.content_region_avail()[0], height])
                                .build();
                            record_control(ui, &format!("bp-action:{}", row.key));
                            if let Some(index) = row.action {
                                record_control(ui, &actions[index].tooltip);
                            }
                            if selected && scroll {
                                ui.set_scroll_here_y();
                            }
                            let (icon, color) = row
                                .action
                                .map(|i| (actions[i].icon, actions[i].color))
                                .unwrap_or((
                                    if row.open { "\u{eab4}" } else { "\u{eab6}" },
                                    [0.55, 0.55, 0.55, 1.],
                                ));
                            let x = start[0] + row.depth as f32 * 14.;
                            let draw = ui.get_window_draw_list();
                            if row.action.is_some() {
                                draw.add_text([x, start[1] + 4.], color, icon);
                            } else {
                                // Native tree disclosure triangles, matching Slate's compact arrows.
                                let y = start[1] + 6.;
                                if row.open {
                                    draw.add_triangle(
                                        [x + 2., y],
                                        [x + 10., y],
                                        [x + 6., y + 5.],
                                        color,
                                    )
                                    .filled(true)
                                    .build();
                                } else {
                                    draw.add_triangle(
                                        [x + 4., y - 1.],
                                        [x + 9., y + 3.],
                                        [x + 4., y + 7.],
                                        color,
                                    )
                                    .filled(true)
                                    .build();
                                }
                            }
                            let _bold = if row.action.is_none() {
                                self.fonts.map(|fonts| ui.push_font(fonts.bold))
                            } else {
                                None
                            };
                            draw.add_text(
                                [
                                    x + if row.action.is_some() { 19. } else { 13. },
                                    start[1] + (height - 12.) * 0.5,
                                ],
                                [0.78, 0.78, 0.78, 1.],
                                &row.label,
                            );
                            if ui.is_item_hovered()
                                && let Some(i) = row.action
                            {
                                ui.tooltip_text(&actions[i].tooltip);
                            }
                            if clicked || (selected && enter) {
                                self.selected = row.key.clone();
                                if let Some(i) = row.action {
                                    chosen = Some(actions[i].kind.clone());
                                } else if !self.expanded.remove(&row.key) {
                                    self.expanded.insert(row.key.clone());
                                }
                            }
                            if selected && !editing && row.action.is_none() {
                                if ui.is_key_pressed(Key::RightArrow) {
                                    self.expanded.insert(row.key.clone());
                                }
                                if ui.is_key_pressed(Key::LeftArrow) {
                                    self.expanded.remove(&row.key);
                                }
                            }
                        }
                    });
            });
        if chosen.is_some() || ui.is_key_pressed(Key::Escape) {
            self.open = false;
            ui.close_current_popup();
        }
        chosen
    }
}

#[cfg(test)]
#[path = "blueprint_action_menu_tests.rs"]
mod tests;
