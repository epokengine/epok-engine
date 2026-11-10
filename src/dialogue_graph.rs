//! Graph view state; links always edit the authoritative conversation document.
use crate::dialogue_asset::{DialogueAsset, Node, NodeKind};
use imgui::{MouseButton, Ui};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use uuid::Uuid;

const WIDTH: f32 = 184.;
const HEADER: f32 = 28.;
const ROW: f32 = 22.;
const BACKGROUND: [f32; 4] = [0.075, 0.085, 0.11, 1.];
const GRID: [f32; 4] = [0.14, 0.16, 0.19, 1.];
const LINK: [f32; 4] = [0.56, 0.65, 0.79, 1.];
const ACCENT: [f32; 4] = [0.96, 0.73, 0.32, 1.];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Connection {
    pub source: Uuid,
    pub port: usize,
    pub target: Uuid,
}
pub fn connect(document: &mut DialogueAsset, link: Connection) -> Result<(), String> {
    if !document.nodes.iter().any(|n| n.id == link.target) {
        return Err("The destination node no longer exists.".into());
    }
    let source = document
        .nodes
        .iter_mut()
        .find(|n| n.id == link.source)
        .ok_or("The source node no longer exists.")?;
    let target = match &mut source.kind {
        NodeKind::Line { next, .. } | NodeKind::Event { next, .. } if link.port == 0 => next,
        NodeKind::Choice { options, .. } => {
            &mut options
                .get_mut(link.port)
                .ok_or("The choice no longer exists.")?
                .next
        }
        NodeKind::Branch { yes, .. } if link.port == 0 => yes,
        NodeKind::Branch { no, .. } if link.port == 1 => no,
        _ => return Err("This node has no matching output port.".into()),
    };
    *target = link.target;
    Ok(())
}

#[derive(Clone, Copy)]
enum Drag {
    Node { id: Uuid, offset: [f32; 2] },
    Pan,
    Link { source: Uuid, port: usize },
}
pub struct State {
    positions: BTreeMap<Uuid, [f32; 2]>,
    pan: [f32; 2],
    zoom: f32,
    drag: Option<Drag>,
    focus: Option<Uuid>,
    fit: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            positions: BTreeMap::new(),
            pan: [0.; 2],
            zoom: 1.,
            drag: None,
            focus: None,
            fit: true,
        }
    }
}
impl State {
    pub fn arrange(&mut self, document: &DialogueAsset) {
        self.positions = layout(document);
        self.pan = [0.; 2];
        self.zoom = 1.;
        self.drag = None;
        self.fit = false;
    }
    pub fn focus(&mut self, id: Uuid) {
        self.focus = Some(id);
        self.fit = false;
    }
    pub fn frame_all(&mut self) {
        self.fit = true;
        self.focus = None;
    }
    fn screen(&self, origin: [f32; 2], point: [f32; 2]) -> [f32; 2] {
        [
            origin[0] + (point[0] - self.pan[0]) * self.zoom,
            origin[1] + (point[1] - self.pan[1]) * self.zoom,
        ]
    }
    fn world(&self, origin: [f32; 2], point: [f32; 2]) -> [f32; 2] {
        [
            (point[0] - origin[0]) / self.zoom + self.pan[0],
            (point[1] - origin[1]) / self.zoom + self.pan[1],
        ]
    }
    pub fn draw(
        &mut self,
        ui: &Ui,
        document: &DialogueAsset,
        selected: &mut Uuid,
    ) -> Option<Connection> {
        let identities = document.nodes.iter().map(|n| n.id).collect::<BTreeSet<_>>();
        self.positions.retain(|id, _| identities.contains(id));
        if document
            .nodes
            .iter()
            .any(|n| !self.positions.contains_key(&n.id))
        {
            for (id, position) in layout(document) {
                self.positions.entry(id).or_insert(position);
            }
        }
        let origin = ui.cursor_screen_pos();
        let available = ui.content_region_avail();
        let area = [available[0].max(40.), available[1].max(80.)];
        if self.fit && !self.positions.is_empty() {
            let mut bounds = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
            for node in &document.nodes {
                if let Some(p) = self.positions.get(&node.id) {
                    bounds[0] = bounds[0].min(p[0]);
                    bounds[1] = bounds[1].min(p[1]);
                    bounds[2] = bounds[2].max(p[0] + WIDTH);
                    bounds[3] = bounds[3].max(p[1] + height(node));
                }
            }
            self.zoom = (area[0] / (bounds[2] - bounds[0] + 60.))
                .min(area[1] / (bounds[3] - bounds[1] + 60.))
                .clamp(0.2, 1.8);
            self.pan = [
                (bounds[0] + bounds[2]) * 0.5 - area[0] / (2. * self.zoom),
                (bounds[1] + bounds[3]) * 0.5 - area[1] / (2. * self.zoom),
            ];
            self.fit = false;
        }
        if let Some(id) = self.focus.take()
            && let Some(point) = self.positions.get(&id)
        {
            self.pan = [
                point[0] + WIDTH * 0.5 - area[0] / (2. * self.zoom),
                point[1] + HEADER - area[1] / (2. * self.zoom),
            ];
        }
        ui.invisible_button("##dialogue-canvas", area);
        let hovered = ui.is_item_hovered();
        let mouse = ui.io().mouse_pos;
        if hovered && ui.io().mouse_wheel != 0. {
            let anchor = self.world(origin, mouse);
            self.zoom = (self.zoom * (1. + ui.io().mouse_wheel * 0.12)).clamp(0.2, 1.8);
            self.pan = [
                anchor[0] - (mouse[0] - origin[0]) / self.zoom,
                anchor[1] - (mouse[1] - origin[1]) / self.zoom,
            ];
        }
        if hovered && ui.is_mouse_clicked(MouseButton::Middle) {
            self.drag = Some(Drag::Pan);
        }
        let world = self.world(origin, mouse);
        if hovered && ui.is_mouse_clicked(MouseButton::Left) {
            self.drag = None;
            for node in document.nodes.iter().rev() {
                let Some(point) = self.positions.get(&node.id).copied() else {
                    continue;
                };
                if let Some(port) = (0..port_count(node))
                    .find(|port| near(world, output(point, *port), 9. / self.zoom))
                {
                    *selected = node.id;
                    self.drag = Some(Drag::Link {
                        source: node.id,
                        port,
                    });
                    break;
                }
                if inside(world, point, [WIDTH, height(node)]) {
                    *selected = node.id;
                    self.drag = Some(Drag::Node {
                        id: node.id,
                        offset: [world[0] - point[0], world[1] - point[1]],
                    });
                    break;
                }
            }
        }
        match self.drag {
            Some(Drag::Pan) if ui.is_mouse_down(MouseButton::Middle) => {
                self.pan[0] -= ui.io().mouse_delta[0] / self.zoom;
                self.pan[1] -= ui.io().mouse_delta[1] / self.zoom;
            }
            Some(Drag::Node { id, offset })
                if ui.is_mouse_down(MouseButton::Left)
                    && world.iter().all(|v| v.is_finite() && v.abs() < 1e7) =>
            {
                self.positions
                    .insert(id, [world[0] - offset[0], world[1] - offset[1]]);
            }
            _ => {}
        }
        let mut connection = None;
        if ui.is_mouse_released(MouseButton::Left) {
            if let Some(Drag::Link { source, port }) = self.drag
                && hovered
                && let Some(target) = document.nodes.iter().rev().find(|node| {
                    self.positions.get(&node.id).is_some_and(|point| {
                        near(world, input(*point), 14. / self.zoom)
                            || inside(world, *point, [WIDTH, height(node)])
                    })
                })
            {
                connection = Some(Connection {
                    source,
                    port,
                    target: target.id,
                });
            }
            if !matches!(self.drag, Some(Drag::Pan)) {
                self.drag = None;
            }
        }
        if ui.is_mouse_released(MouseButton::Middle) && matches!(self.drag, Some(Drag::Pan)) {
            self.drag = None;
        }
        let draw = ui.get_window_draw_list();
        let end = [origin[0] + area[0], origin[1] + area[1]];
        ui.set_window_font_scale(self.zoom);
        draw.with_clip_rect_intersect(origin, end, || {
            draw.add_rect(origin, end, BACKGROUND).filled(true).build();
            let step = 32. * self.zoom;
            let mut x = origin[0] - (self.pan[0] * self.zoom).rem_euclid(step);
            while x < end[0] {
                draw.add_line([x, origin[1]], [x, end[1]], GRID).build();
                x += step;
            }
            let mut y = origin[1] - (self.pan[1] * self.zoom).rem_euclid(step);
            while y < end[1] {
                draw.add_line([origin[0], y], [end[0], y], GRID).build();
                y += step;
            }
            let curve = |from: [f32; 2], to: [f32; 2], color| {
                let bend =
                    ((to[0] - from[0]).abs() * 0.45).clamp(35. * self.zoom, 120. * self.zoom);
                if from[0].max(to[0]) + bend < origin[0]
                    || from[0].min(to[0]) - bend > end[0]
                    || from[1].max(to[1]) < origin[1]
                    || from[1].min(to[1]) > end[1]
                {
                    return;
                }
                draw.add_bezier_curve(
                    from,
                    [from[0] + bend, from[1]],
                    [to[0] - bend, to[1]],
                    to,
                    color,
                )
                .thickness(2.)
                .build();
            };
            for node in &document.nodes {
                let Some(point) = self.positions.get(&node.id) else {
                    continue;
                };
                for (port, destination) in node.kind.destinations().iter().enumerate() {
                    if let Some(target) = self.positions.get(destination) {
                        curve(
                            self.screen(origin, output(*point, port)),
                            self.screen(origin, input(*target)),
                            LINK,
                        );
                    }
                }
            }
            if let Some(Drag::Link { source, port }) = self.drag
                && let Some(point) = self.positions.get(&source)
            {
                curve(self.screen(origin, output(*point, port)), mouse, ACCENT);
            }
            for (index, node) in document.nodes.iter().enumerate() {
                let Some(point) = self.positions.get(&node.id) else {
                    continue;
                };
                let a = self.screen(origin, *point);
                let b = self.screen(origin, [point[0] + WIDTH, point[1] + height(node)]);
                if b[0] < origin[0] || a[0] > end[0] || b[1] < origin[1] || a[1] > end[1] {
                    continue;
                }
                draw.add_rect(a, b, [0.13, 0.15, 0.19, 1.])
                    .filled(true)
                    .rounding(5.)
                    .build();
                draw.add_rect(a, [b[0], a[1] + HEADER * self.zoom], color(&node.kind))
                    .filled(true)
                    .rounding(5.)
                    .build();
                draw.add_rect(a, b, if *selected == node.id { ACCENT } else { LINK })
                    .rounding(5.)
                    .thickness(if *selected == node.id { 2. } else { 1. })
                    .build();
                draw.add_text(
                    [a[0] + 8. * self.zoom, a[1] + 6. * self.zoom],
                    [1.; 4],
                    format!(
                        "{} {}{}",
                        index + 1,
                        title(&node.kind),
                        if node.id == document.entry {
                            " / Entry"
                        } else {
                            ""
                        }
                    ),
                );
                draw.add_circle(self.screen(origin, input(*point)), 5. * self.zoom, LINK)
                    .filled(true)
                    .build();
                for (port, name) in ports(node).iter().enumerate() {
                    let p = self.screen(origin, output(*point, port));
                    draw.add_text(
                        [a[0] + 8. * self.zoom, p[1] - 7. * self.zoom],
                        [0.88, 0.9, 0.94, 1.],
                        name,
                    );
                    draw.add_circle(p, 5. * self.zoom, ACCENT)
                        .filled(true)
                        .build();
                }
            }
        });
        ui.set_window_font_scale(1.);
        connection
    }
}
fn title(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Line { .. } => "Line",
        NodeKind::Choice { .. } => "Choice",
        NodeKind::Branch { .. } => "Branch",
        NodeKind::Event { .. } => "Event",
        NodeKind::End => "End",
    }
}
fn color(kind: &NodeKind) -> [f32; 4] {
    match kind {
        NodeKind::Line { .. } => [0.16, 0.33, 0.43, 1.],
        NodeKind::Choice { .. } => [0.28, 0.24, 0.43, 1.],
        NodeKind::Branch { .. } => [0.4, 0.32, 0.13, 1.],
        NodeKind::Event { .. } => [0.38, 0.2, 0.2, 1.],
        NodeKind::End => [0.18, 0.34, 0.26, 1.],
    }
}
fn ports(node: &Node) -> Vec<String> {
    match &node.kind {
        NodeKind::Line { text, .. } => vec![short(&text.key)],
        NodeKind::Choice { options, .. } => options
            .iter()
            .enumerate()
            .map(|(i, o)| format!("{}. {}", i + 1, short(&o.text.key)))
            .collect(),
        NodeKind::Branch { .. } => vec!["True".into(), "False".into()],
        NodeKind::Event { .. } => vec!["Continue".into()],
        NodeKind::End => vec![],
    }
}
fn short(text: &str) -> String {
    let mut chars = text.chars();
    let mut result = chars.by_ref().take(20).collect::<String>();
    if chars.next().is_some() {
        result.push_str("...");
    }
    result
}
fn port_count(node: &Node) -> usize {
    match &node.kind {
        NodeKind::Choice { options, .. } => options.len(),
        NodeKind::Branch { .. } => 2,
        NodeKind::End => 0,
        _ => 1,
    }
}
fn height(node: &Node) -> f32 {
    HEADER + (port_count(node).max(1) as f32) * ROW + 6.
}
fn input(point: [f32; 2]) -> [f32; 2] {
    [point[0], point[1] + HEADER * 0.5]
}
fn output(point: [f32; 2], port: usize) -> [f32; 2] {
    [
        point[0] + WIDTH,
        point[1] + HEADER + (port as f32 + 0.5) * ROW,
    ]
}
fn inside(point: [f32; 2], a: [f32; 2], size: [f32; 2]) -> bool {
    point[0] >= a[0] && point[0] <= a[0] + size[0] && point[1] >= a[1] && point[1] <= a[1] + size[1]
}
fn near(a: [f32; 2], b: [f32; 2], radius: f32) -> bool {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) <= radius * radius
}
fn layout(document: &DialogueAsset) -> BTreeMap<Uuid, [f32; 2]> {
    let nodes = document
        .nodes
        .iter()
        .map(|n| (n.id, n))
        .collect::<BTreeMap<_, _>>();
    let mut columns = BTreeMap::new();
    let mut pending = VecDeque::new();
    if nodes.contains_key(&document.entry) {
        columns.insert(document.entry, 0usize);
        pending.push_back(document.entry);
    }
    while let Some(id) = pending.pop_front() {
        let depth = columns[&id];
        for destination in nodes[&id].kind.destinations() {
            if nodes.contains_key(&destination) && !columns.contains_key(&destination) {
                columns.insert(destination, depth + 1);
                pending.push_back(destination);
            }
        }
    }
    let mut rows = BTreeMap::new();
    let mut result = BTreeMap::new();
    for node in &document.nodes {
        let column = columns.get(&node.id).copied().unwrap_or(0);
        let row = rows.entry(column).or_insert(30.);
        result.insert(node.id, [30. + column as f32 * (WIDTH + 82.), *row]);
        *row += height(node) + 36.;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialogue_graph_mouse_link_updates_the_same_document_and_keeps_option_identity() {
        let mut document = DialogueAsset::default();
        let source = document.entry;
        let target = Uuid::new_v4();
        let option = Uuid::new_v4();
        document.nodes[0].kind = NodeKind::Choice {
            speaker: None,
            prompt: None,
            options: vec![crate::dialogue_asset::ChoiceOption {
                id: option,
                text: crate::dialogue_asset::TextKey {
                    table: Uuid::new_v4(),
                    key: "hello".into(),
                },
                next: source,
                condition: None,
            }],
        };
        document.nodes.push(Node {
            id: target,
            kind: NodeKind::End,
        });
        let mut context = crate::gui::tests::imgui_context();
        context.io_mut().display_size = [800., 600.];
        context.io_mut().delta_time = 0.1;
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        let mut state = State::default();
        state.arrange(&document);
        state.positions.insert(target, [330., 30.]);
        let mut selected = source;
        let mut frame = |context: &mut imgui::Context, state: &mut State| {
            let mut link = None;
            let mut origin = [0.; 2];
            let ui = context.frame();
            ui.window("Graph test")
                .position([0., 0.], imgui::Condition::Always)
                .size([800., 600.], imgui::Condition::Always)
                .build(|| {
                    origin = ui.cursor_screen_pos();
                    link = state.draw(ui, &document, &mut selected);
                });
            context.render();
            (link, origin)
        };
        let (link, origin) = frame(&mut context, &mut state);
        assert!(link.is_none());
        // The canvas origin follows the actual window decoration, not guessed coordinates.
        let from = state.screen(origin, output(state.positions[&source], 0));
        let to = state.screen(origin, input(state.positions[&target]));
        context.io_mut().add_mouse_pos_event(from);
        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, true);
        assert!(frame(&mut context, &mut state).0.is_none());
        context.io_mut().add_mouse_pos_event(to);
        context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, false);
        let link = frame(&mut context, &mut state)
            .0
            .expect("drag must connect a real output to a node");
        assert_eq!(
            link,
            Connection {
                source,
                port: 0,
                target
            }
        );
        connect(&mut document, link).unwrap();
        let NodeKind::Choice { options, .. } = &document.nodes[0].kind else {
            panic!()
        };
        assert_eq!(options[0].id, option);
        assert_eq!(options[0].next, target);
    }
    #[test]
    fn dialogue_graph_layout_and_links_handle_cycles_and_missing_ports() {
        let mut document = DialogueAsset::default();
        let entry = document.entry;
        let end = Uuid::new_v4();
        document.nodes[0].kind = NodeKind::Branch {
            condition: Uuid::new_v4(),
            yes: entry,
            no: end,
        };
        document.nodes.push(Node {
            id: end,
            kind: NodeKind::End,
        });
        let before = document.clone();
        let positions = layout(&document);
        assert_eq!(positions.len(), 2);
        assert!(positions.values().flatten().all(|v| v.is_finite()));
        assert_eq!(document, before);
        assert!(
            connect(
                &mut document,
                Connection {
                    source: end,
                    port: 0,
                    target: entry
                }
            )
            .is_err()
        );
        assert_eq!(document, before);
        connect(
            &mut document,
            Connection {
                source: entry,
                port: 1,
                target: entry,
            },
        )
        .unwrap();
        assert_eq!(document.nodes[0].kind.destinations(), vec![entry, entry]);
    }
}
