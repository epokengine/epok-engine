//! Read-only navigation of the compiler's published artifact graph.
use crate::artifact_dependencies::{Graph, Node};
use imgui::{Condition, Ui};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct State {
    pub open: bool,
    graph: Graph,
    error: Option<String>,
    checked: Option<Instant>,
    selected: Option<String>,
    history: VecDeque<String>,
    search: String,
    stale_only: bool,
}

fn status(node: Option<&Node>) -> &'static str {
    match node {
        Some(node) if !node.stale.is_empty() => "Stale",
        Some(node) if node.signature.is_some() => "Recorded",
        _ => "Missing",
    }
}

/// Breadth-first traversal keeps one shortest explanation per consumer and
/// terminates even when the retained diagnostic graph contains a broken cycle.
fn consumer_paths(graph: &Graph, selected: &str) -> BTreeMap<String, Vec<String>> {
    let mut reverse: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (key, node) in &graph.nodes {
        for input in &node.dependencies {
            reverse.entry(input).or_default().push(key);
        }
    }
    let mut paths = BTreeMap::new();
    let mut seen = BTreeSet::from([selected.to_owned()]);
    let mut queue = VecDeque::from([(selected.to_owned(), vec![selected.to_owned()])]);
    while let Some((key, path)) = queue.pop_front() {
        for consumer in reverse.get(key.as_str()).into_iter().flatten() {
            if seen.insert((*consumer).to_owned()) {
                let mut next = path.clone();
                next.push((*consumer).to_owned());
                paths.insert((*consumer).to_owned(), next.clone());
                queue.push_back(((*consumer).to_owned(), next));
            }
        }
    }
    paths
}

impl State {
    fn refresh(&mut self, root: &Path) {
        self.checked = Some(Instant::now());
        match Graph::load(root) {
            Ok(graph) => {
                self.graph = graph;
                self.error = None;
                if self.selected.is_none() {
                    self.selected = self
                        .graph
                        .nodes
                        .iter()
                        .find(|(_, n)| !n.stale.is_empty())
                        .or_else(|| self.graph.nodes.first_key_value())
                        .map(|(key, _)| key.clone());
                }
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn select(&mut self, key: String) {
        if self.selected.as_ref() != Some(&key)
            && let Some(previous) = self.selected.replace(key)
        {
            if self.history.len() == 64 {
                self.history.pop_front();
            }
            self.history.push_back(previous);
        }
    }

    pub fn draw(&mut self, ui: &Ui, root: &Path) {
        if !self.open {
            self.checked = None;
            return;
        }
        if self
            .checked
            .is_none_or(|t| t.elapsed() >= Duration::from_secs(1))
        {
            self.refresh(root);
        }
        let mut open = self.open;
        let display = ui.io().display_size;
        ui.window("Artifact Dependencies")
            .opened(&mut open)
            .size(
                [
                    (display[0] - 40.).clamp(620., 1050.),
                    (display[1] - 100.).clamp(380., 680.),
                ],
                Condition::FirstUseEver,
            )
            .position(
                [display[0] * 0.5, display[1] * 0.5],
                Condition::FirstUseEver,
            )
            .position_pivot([0.5, 0.5])
            .size_constraints([620., 380.], [2200., 1600.])
            .build(|| self.body(ui, root));
        self.open = open;
    }

    fn body(&mut self, ui: &Ui, root: &Path) {
        if ui.button("Refresh") {
            self.refresh(root);
        }
        ui.same_line();
        ui.checkbox("Stale only", &mut self.stale_only);
        ui.same_line();
        ui.set_next_item_width(-1.);
        ui.input_text("##artifact-search", &mut self.search)
            .hint("Search IDs, paths or stale reasons")
            .build();
        let stale = self
            .graph
            .nodes
            .values()
            .filter(|n| !n.stale.is_empty())
            .count();
        crate::gui::muted(
            ui,
            format!(
                "{} recorded nodes | {stale} stale | Updates every second while open",
                self.graph.nodes.len()
            ),
        );
        if let Some(error) = &self.error {
            ui.text_colored(
                [1., 0.55, 0.3, 1.],
                "Snapshot unavailable. Displaying the last read snapshot for navigation only.",
            );
            ui.text_wrapped(error);
        }
        crate::gui::muted(
            ui,
            "Recorded means no known invalidation in this snapshot. Build and Export still validate current sources.",
        );
        ui.separator();
        let query = self.search.to_lowercase();
        let keys = self
            .graph
            .nodes
            .iter()
            .filter(|(key, n)| {
                (!self.stale_only || !n.stale.is_empty())
                    && (query.is_empty()
                        || key.to_lowercase().contains(&query)
                        || n.stale.iter().any(|(source, reason)| {
                            source.to_lowercase().contains(&query)
                                || reason.to_lowercase().contains(&query)
                        }))
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        let mut navigate = None;
        let width = ui.content_region_avail()[0];
        ui.child_window("artifact-list").size([(width * 0.37).max(200.), 0.]).border(true).build(|| {
            ui.text(format!("Results ({})", keys.len()));
            let mut clipper = imgui::ListClipper::new(keys.len() as i32).begin(ui);
            while clipper.step() {
                for i in clipper.display_start()..clipper.display_end() {
                    let key = &keys[i as usize];
                    let node = self.graph.nodes.get(key);
                    if link(ui, key, node, self.selected.as_ref() == Some(key)) {
                        navigate = Some(key.clone());
                    }
                }
            }
            if keys.is_empty() {
                crate::gui::muted(ui, "No matching records. Validate or build an asset to record its dependencies.");
            }
        });
        ui.same_line();
        ui.child_window("artifact-detail").size([0., 0.]).border(true).build(|| {
            if !self.history.is_empty() && ui.small_button("Back") {
                self.selected = self.history.pop_back();
            }
            let Some(key) = self.selected.as_ref() else { return; };
            ui.text_wrapped(key);
            let node = self.graph.nodes.get(key);
            ui.text(format!("Status: {}", status(node)));
            if ui.small_button("Copy ID") {
                ui.set_clipboard_text(key);
            }
            if let Some(node) = node {
                if !node.stale.is_empty() {
                    ui.separator();
                    ui.text_colored([1., 0.65, 0.3, 1.], "Stale: regenerate before use");
                    for (source, reason) in &node.stale {
                        let _id = ui.push_id(format!("reason:{source}"));
                        ui.text_wrapped(reason);
                        if link(ui, source, self.graph.nodes.get(source), false) {
                            navigate = Some(source.clone());
                        }
                    }
                }
                if let Some(signature) = &node.signature {
                    ui.separator();
                    crate::gui::muted(ui, "Last recorded content signature (retained when stale)");
                    ui.text_wrapped(signature);
                }
                ui.separator();
                ui.text(format!("Inputs -> this node ({})", node.dependencies.len()));
                for input in &node.dependencies {
                    let _id = ui.push_id("input");
                    if link(ui, input, self.graph.nodes.get(input), false) {
                        navigate = Some(input.clone());
                    }
                }
            } else {
                crate::gui::muted(ui, "This dependency has no recorded node. Retained references are available below.");
            }
            let paths = consumer_paths(&self.graph, key);
            ui.separator();
            ui.text(format!("This node -> direct consumers ({})", paths.values().filter(|p| p.len() == 2).count()));
            for (consumer, path) in paths.iter().filter(|(_, p)| p.len() == 2) {
                let _id = ui.push_id("direct");
                if link(ui, consumer, self.graph.nodes.get(consumer), false) {
                    navigate = Some(consumer.clone());
                }
                if ui.is_item_hovered() { ui.tooltip_text(path.join("\n-> ")); }
            }
            ui.separator();
            ui.text(format!("Further consumers ({})", paths.values().filter(|p| p.len() > 2).count()));
            crate::gui::muted(ui, "Hover a consumer to see one shortest dependency path.");
            for (consumer, path) in paths.iter().filter(|(_, p)| p.len() > 2) {
                let _id = ui.push_id("transitive");
                if link(ui, consumer, self.graph.nodes.get(consumer), false) {
                    navigate = Some(consumer.clone());
                }
                if ui.is_item_hovered() { ui.tooltip_text(path.join("\n-> ")); }
            }
        });
        if let Some(key) = navigate {
            self.select(key);
        }
    }
}

fn link(ui: &Ui, key: &str, node: Option<&Node>, selected: bool) -> bool {
    let _id = ui.push_id(key);
    let state = status(node);
    let text = format!("[{state}] {key}");
    // The full identity remains visible in the tooltip and selected detail pane.
    let clicked = ui.selectable_config(&text).selected(selected).build();
    #[cfg(test)]
    CONTROLS.with(|controls| {
        let a = ui.item_rect_min();
        let b = ui.item_rect_max();
        controls
            .borrow_mut()
            .insert(key.to_owned(), [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]);
    });
    if ui.is_item_hovered() {
        ui.tooltip_text(&text);
    }
    clicked
}

#[cfg(test)]
thread_local! {
    static CONTROLS: std::cell::RefCell<BTreeMap<String, [f32; 2]>> = Default::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_missing_inputs_and_cycles_have_finite_consumer_paths() {
        let mut graph = Graph::default();
        for (key, inputs) in [
            ("timeline", vec!["missing", "effect"]),
            ("effect", vec!["timeline"]),
            ("scene", vec!["effect"]),
            ("unrelated", vec![]),
        ] {
            graph.publish(
                key,
                "1".into(),
                inputs.into_iter().map(str::to_owned).collect(),
            );
        }
        let paths = consumer_paths(&graph, "missing");
        assert_eq!(paths["scene"], ["missing", "timeline", "effect", "scene"]);
        assert_eq!(paths.len(), 3);
        assert!(!consumer_paths(&graph, "effect").contains_key("effect"));
        assert_eq!(status(graph.nodes.get("timeline")), "Stale");
        assert_eq!(status(graph.nodes.get("missing")), "Missing");
    }

    #[test]
    #[ignore = "Owns an ImGui context; run explicitly and serially"]
    fn dependency_navigation_uses_real_clicks_and_never_certifies_a_failed_snapshot() {
        let root =
            std::env::temp_dir().join(format!("epok-dependency-ui-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join(".epok")).unwrap();
        crate::artifact_dependencies::transaction(&root, |graph| {
            graph.publish(
                "effect:fire",
                "1".into(),
                BTreeSet::from(["timeline:missing".into()]),
            );
            graph.publish(
                "scene:combat",
                "2".into(),
                BTreeSet::from(["effect:fire".into()]),
            );
        })
        .unwrap();
        let file = root.join(".epok/ArtifactDependencies.json");
        let bytes = std::fs::read(&file).unwrap();
        let mut state = State {
            open: true,
            ..Default::default()
        };
        let mut ctx = crate::gui::tests::imgui_context();
        ctx.set_ini_filename(None);
        ctx.io_mut().display_size = [1280., 900.];
        ctx.io_mut().delta_time = 1. / 60.;
        ctx.fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        ctx.fonts().build_rgba32_texture();
        let frame = |ctx: &mut imgui::Context, state: &mut State| {
            state.draw(ctx.frame(), &root);
            ctx.render();
        };
        let click = |ctx: &mut imgui::Context, state: &mut State, key: &str| {
            let point = CONTROLS.with(|controls| controls.borrow()[key]);
            ctx.io_mut().add_mouse_pos_event(point);
            frame(ctx, state);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, true);
            frame(ctx, state);
            ctx.io_mut()
                .add_mouse_button_event(imgui::MouseButton::Left, false);
            frame(ctx, state);
            // Selection is committed after drawing; let the next frame lay out
            // the newly selected node before clicking one of its consumers.
            frame(ctx, state);
        };
        frame(&mut ctx, &mut state);
        frame(&mut ctx, &mut state);
        assert_eq!(state.selected.as_deref(), Some("effect:fire"));
        click(&mut ctx, &mut state, "timeline:missing");
        assert_eq!(state.selected.as_deref(), Some("timeline:missing"));
        click(&mut ctx, &mut state, "scene:combat");
        assert_eq!(state.selected.as_deref(), Some("scene:combat"));
        assert_eq!(std::fs::read(&file).unwrap(), bytes);
        let previous = state.graph.clone();
        std::fs::write(&file, b"invalid snapshot").unwrap();
        state.refresh(&root);
        frame(&mut ctx, &mut state);
        assert!(state.error.is_some());
        assert_eq!(state.graph, previous);
        assert_eq!(std::fs::read(&file).unwrap(), b"invalid snapshot");
        std::fs::write(&file, &bytes).unwrap();
        state.refresh(&root);
        assert!(state.error.is_none());
        assert_eq!(state.selected.as_deref(), Some("scene:combat"));
    }
}
