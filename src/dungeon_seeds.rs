//! The seed bench: generates the layout of many seeds of one Dungeon Generator without
//! touching the map, measures each against the console's budget (actors, quads,
//! collision boxes, navigation nodes, 2D tiles) and how many attempts its recipe took,
//! and lists the seeds that fit, to pick one or export them. It runs a slice of seeds
//! each frame so the editor stays responsive.
use crate::dungeon::{self, Generator};
use crate::dungeon_editor::Budget;
use crate::editor::Editor;
use imgui::Ui;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub const ICON: &str = "\u{f522}";
/// The console's limits a seed is measured against.
pub const ACTORS: usize = 64;
pub const NAVIGATION: usize = 512;
const GREEN: [f32; 4] = [0.45, 0.85, 0.55, 1.];
const AMBER: [f32; 4] = [1., 0.75, 0.3, 1.];
const RED: [f32; 4] = [1., 0.45, 0.4, 1.];

/// One seed's result.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub seed: u32,
    /// The layout or budget could not be made; the reason.
    pub error: Option<String>,
    /// Recipe attempts; 0 for rooms-and-corridors layouts.
    pub attempts: u32,
    pub rooms: usize,
    pub budget: Budget,
    /// The limits the seed goes over.
    pub over: Vec<&'static str>,
}
impl Sample {
    pub fn fits(&self) -> bool {
        self.error.is_none() && self.over.is_empty()
    }
}

/// Measures one seed of `g` in `scene`.
pub fn sample(
    scene: &crate::scene::Scene,
    index: &crate::assets::Index,
    g: &Generator,
    seed: u32,
) -> Sample {
    let mut g = g.clone();
    g.seed = seed;
    let two_d = crate::dungeon_bake::is_2d(scene, g.root);
    let mut result = Sample {
        seed,
        error: None,
        attempts: 0,
        rooms: 0,
        budget: Budget::default(),
        over: Vec::new(),
    };
    let measured = dungeon::layout(&g, index).and_then(|mut layout| {
        if two_d {
            layout.without_handmade_content();
        }
        let (style, _) = crate::dungeon_bake::style_of(index, &g, two_d)?;
        let budget = crate::dungeon_editor::budget(scene, index, &g, &layout, &style)?;
        Ok((layout, budget))
    });
    match measured {
        Ok((layout, budget)) => {
            result.attempts = layout.attempts;
            result.rooms = layout.rooms.len();
            if budget.actors > ACTORS {
                result.over.push("actors");
            }
            if two_d {
                if budget.tiles > crate::tilemap::MAX_CELLS {
                    result.over.push("tiles");
                }
            } else {
                if budget.quads > crate::dungeon_bake::SCENE_QUADS {
                    result.over.push("quads");
                }
                if budget.boxes > crate::collision::MAX_COMPOUND_BOXES {
                    result.over.push("collision");
                }
                if g.navigation && budget.navigation.is_none_or(|n| n > NAVIGATION) {
                    result.over.push("navigation");
                }
            }
            result.budget = budget;
        }
        Err(error) => result.error = Some(error),
    }
    result
}

/// Counts of `values` in `buckets` equal ranges from 0 to `top`, the last one holding
/// everything above.
pub fn histogram(values: impl Iterator<Item = usize>, top: usize, buckets: usize) -> Vec<f32> {
    let mut counts = vec![0f32; buckets.max(1)];
    let width = top.div_ceil(buckets.max(1)).max(1);
    for v in values {
        counts[(v / width).min(buckets - 1)] += 1.;
    }
    counts
}

/// The seeds that fit, as CSV with what each one measured.
pub fn csv(samples: &[Sample]) -> String {
    let mut out =
        String::from("seed,rooms,attempts,actors,quads,collision_boxes,navigation_nodes,tiles\n");
    for s in samples.iter().filter(|s| s.fits()) {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            s.seed,
            s.rooms,
            s.attempts,
            s.budget.actors,
            s.budget.quads,
            s.budget.boxes,
            s.budget.navigation.map_or(String::new(), |n| n.to_string()),
            s.budget.tiles
        ));
    }
    out
}

/// A one-line account of a run.
pub fn summary(samples: &[Sample]) -> String {
    let fits = samples.iter().filter(|s| s.fits()).count();
    let failed = samples.iter().filter(|s| s.error.is_some()).count();
    let over = samples.len() - fits - failed;
    let mut reasons: Vec<(&str, usize)> = Vec::new();
    for s in samples {
        for r in &s.over {
            match reasons.iter_mut().find(|(n, _)| n == r) {
                Some((_, c)) => *c += 1,
                None => reasons.push((r, 1)),
            }
        }
    }
    let mut text = format!(
        "{fits} of {} seeds fit the console · {over} over budget · {failed} failed",
        samples.len()
    );
    if !reasons.is_empty() {
        text.push_str(&format!(
            " (over: {})",
            reasons
                .iter()
                .map(|(n, c)| format!("{n} {c}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    text
}

/// Writes the seeds that fit to `exports/dungeon-seeds/` and returns the path, relative
/// to the project.
pub fn export(root: &std::path::Path, name: &str, samples: &[Sample]) -> Result<String, String> {
    let first = samples.first().map_or(0, |s| s.seed);
    let file: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let relative = format!("exports/dungeon-seeds/{file}-{first}-{}.csv", samples.len());
    let path = root.join(&relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, csv(samples)).map_err(|e| e.to_string())?;
    Ok(relative)
}

/// Runs a whole bench at once, for the command line and screenshots.
pub fn run(
    scene: &crate::scene::Scene,
    index: &crate::assets::Index,
    g: &Generator,
    from: u32,
    count: u32,
) -> Vec<Sample> {
    (0..count)
        .map(|i| sample(scene, index, g, from.wrapping_add(i)))
        .collect()
}

pub struct Window {
    pub generator: Uuid,
    pub from: u32,
    pub count: u32,
    pub samples: Vec<Sample>,
    running: bool,
    status: String,
    /// The table lists the seeds that fit instead of those that do not.
    show_fitting: bool,
}

/// Opens the bench on generator `id`, starting at its seed.
pub fn open(e: &mut Editor, id: Uuid) {
    let from = e
        .scene
        .dungeons
        .iter()
        .find(|g| g.id == id)
        .map_or(1, |g| g.seed);
    if e.dungeon_seeds.as_ref().is_some_and(|w| w.generator == id) {
        return;
    }
    e.dungeon_seeds = Some(Window {
        generator: id,
        from,
        count: 200,
        samples: Vec::new(),
        running: false,
        status: String::new(),
        show_fitting: false,
    });
}

/// Opens the bench and runs it to the end before the next frame.
pub fn open_and_run(e: &mut Editor, id: Uuid, count: u32) {
    open(e, id);
    let Some(g) = e.scene.dungeons.iter().find(|g| g.id == id).cloned() else {
        return;
    };
    if let Some(w) = e.dungeon_seeds.as_mut() {
        w.count = count;
        w.samples = run(&e.scene, &e.assets.index, &g, w.from, count);
    }
}

pub fn window(ui: &Ui, e: &mut Editor) {
    let Some(mut w) = e.dungeon_seeds.take() else {
        return;
    };
    let Some(g) = e
        .scene
        .dungeons
        .iter()
        .find(|g| g.id == w.generator)
        .cloned()
    else {
        return;
    };
    let name = e
        .scene
        .actor_index(g.root)
        .map_or_else(|| "Dungeon".into(), |i| e.scene.actors[i].name.clone());
    // A slice of seeds per frame.
    if w.running {
        let started = Instant::now();
        while w.samples.len() < w.count as usize && started.elapsed() < Duration::from_millis(12) {
            let seed = w.from.wrapping_add(w.samples.len() as u32);
            w.samples.push(sample(&e.scene, &e.assets.index, &g, seed));
        }
        if w.samples.len() >= w.count as usize {
            w.running = false;
        }
    }
    let mut open = true;
    let mut use_seed = None;
    ui.window(format!("{ICON} Seed bench: {name}###dungeon-seed-bench"))
        .opened(&mut open)
        .size([560., 620.], imgui::Condition::FirstUseEver)
        .position([420., 120.], imgui::Condition::FirstUseEver)
        .build(|| {
            let _font = crate::inspector_theme::scope(ui, e.inspector_font);
            body(ui, e, &mut w, &g, &name, &mut use_seed);
        });
    if let Some(seed) = use_seed {
        crate::dungeon_editor::use_seed(e, w.generator, seed);
    }
    if open {
        e.dungeon_seeds = Some(w);
    }
}

fn body(
    ui: &Ui,
    e: &mut Editor,
    w: &mut Window,
    g: &Generator,
    name: &str,
    use_seed: &mut Option<u32>,
) {
    crate::gui::muted(
        ui,
        "Generates the layout of each seed without touching the map and measures it against the console's budget, with this generator's settings, recipe and style.",
    );
    ui.disabled(w.running, || {
        let mut from = w.from as i32;
        if crate::gui::Drag::new(crate::gui::field(ui, "From seed"))
            .range(0, i32::MAX)
            .build(ui, &mut from)
        {
            w.from = from.max(0) as u32;
            w.samples.clear();
        }
        let mut count = w.count as i32;
        if crate::gui::Drag::new(crate::gui::field(ui, "Seeds"))
            .range(10, 5000)
            .build(ui, &mut count)
        {
            w.count = count.clamp(10, 5000) as u32;
            w.samples.clear();
        }
    });
    if w.running {
        if ui.button("Stop") {
            w.running = false;
        }
    } else if ui.button(format!("{ICON} Run")) {
        w.samples.clear();
        w.running = true;
        w.status.clear();
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Measure every seed from the first one; a few hundred take seconds.");
    }
    ui.same_line();
    let done = w.samples.len() as f32 / w.count.max(1) as f32;
    imgui::ProgressBar::new(done)
        .overlay_text(format!("{} / {}", w.samples.len(), w.count))
        .build(ui);
    if w.samples.is_empty() {
        return;
    }
    let text = summary(&w.samples);
    let fits = w.samples.iter().filter(|s| s.fits()).count();
    ui.text_colored(
        if fits == w.samples.len() {
            GREEN
        } else {
            AMBER
        },
        &text,
    );
    let width = ui.content_region_avail()[0];
    // A histogram from 0 to the larger of the limit and the largest value, with the
    // limit marked when values go past it.
    let graph = |label: &str, values: Vec<usize>, limit: usize| {
        let most = values.iter().copied().max().unwrap_or(0);
        let top = limit.max(most).max(1);
        crate::gui::muted(ui, label);
        ui.plot_histogram(
            format!("##{label}"),
            &histogram(values.into_iter(), top, 32),
        )
        .graph_size([width, 54.])
        .scale_min(0.)
        .overlay_text(format!("most {most} of {limit}"))
        .build();
        if most > limit {
            let (lo, hi) = (ui.item_rect_min(), ui.item_rect_max());
            let x = lo[0] + (hi[0] - lo[0]) * limit as f32 / top as f32;
            ui.get_window_draw_list()
                .add_line([x, lo[1]], [x, hi[1]], RED)
                .thickness(2.)
                .build();
        }
    };
    let measured: Vec<&Sample> = w.samples.iter().filter(|s| s.error.is_none()).collect();
    let two_d = crate::dungeon_bake::is_2d(&e.scene, g.root);
    if g.source == dungeon::Source::Recipe {
        graph(
            "Recipe attempts: 1 is the first try; at 64 the recipe gives up",
            measured.iter().map(|s| s.attempts as usize).collect(),
            crate::dungeon_recipe::ATTEMPTS as usize,
        );
    }
    if two_d {
        graph(
            "Tiles (the red line is the limit)",
            measured.iter().map(|s| s.budget.tiles).collect(),
            crate::tilemap::MAX_CELLS,
        );
    } else {
        graph(
            "Quads (the red line is the limit)",
            measured.iter().map(|s| s.budget.quads).collect(),
            crate::dungeon_bake::SCENE_QUADS,
        );
    }
    graph(
        "Actors",
        measured.iter().map(|s| s.budget.actors).collect(),
        ACTORS,
    );
    ui.spacing();
    if ui.button("Export seeds that fit") {
        w.status = match export(&e.root, &format!("{}-{name}", e.scene.name), &w.samples) {
            Ok(path) => format!("Written to {path}"),
            Err(error) => format!("Not written: {error}"),
        };
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("A CSV of the seeds that fit, with what each measured, in the project's exports folder.");
    }
    ui.same_line();
    if ui.button("Copy seeds that fit") {
        let seeds: Vec<String> = w
            .samples
            .iter()
            .filter(|s| s.fits())
            .map(|s| s.seed.to_string())
            .collect();
        ui.set_clipboard_text(seeds.join(", "));
        w.status = format!("{} seeds copied", seeds.len());
    }
    if !w.status.is_empty() {
        crate::gui::muted(ui, &w.status);
    }
    ui.spacing();
    let fitting = w.samples.iter().filter(|s| s.fits()).count();
    let problems = w.samples.len() - fitting;
    if problems == 0 {
        w.show_fitting = true;
    }
    if ui.radio_button_bool(format!("Do not fit ({problems})"), !w.show_fitting) {
        w.show_fitting = false;
    }
    ui.same_line();
    if ui.radio_button_bool(format!("Fit ({fitting})"), w.show_fitting) {
        w.show_fitting = true;
    }
    let rows: Vec<&Sample> = w
        .samples
        .iter()
        .filter(|s| s.fits() == w.show_fitting)
        .collect();
    if let Some(_table) = ui.begin_table_with_flags(
        "##seed-rows",
        5,
        imgui::TableFlags::ROW_BG
            | imgui::TableFlags::SCROLL_Y
            | imgui::TableFlags::BORDERS_INNER_H,
    ) {
        for (label, width) in [
            ("Seed", 70.),
            ("Rooms", 50.),
            ("Quads", 60.),
            ("Actors", 50.),
        ] {
            ui.table_setup_column_with(imgui::TableColumnSetup {
                name: label,
                flags: imgui::TableColumnFlags::WIDTH_FIXED,
                init_width_or_weight: width,
                user_id: imgui::Id::default(),
            });
        }
        ui.table_setup_column("");
        ui.table_headers_row();
        for s in rows {
            ui.table_next_row();
            ui.table_next_column();
            ui.text(s.seed.to_string());
            ui.table_next_column();
            ui.text(s.rooms.to_string());
            ui.table_next_column();
            let quads = if two_d {
                s.budget.tiles
            } else {
                s.budget.quads
            };
            ui.text(quads.to_string());
            ui.table_next_column();
            ui.text(s.budget.actors.to_string());
            ui.table_next_column();
            if let Some(error) = &s.error {
                ui.text_colored(RED, error);
            } else if !s.over.is_empty() {
                ui.text_colored(AMBER, format!("over: {}", s.over.join(", ")));
            }
            ui.same_line();
            if ui.small_button(format!("Use##seed-{}", s.seed)) {
                *use_seed = Some(s.seed);
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Set this seed on the generator and generate (Ctrl+Z undoes it).");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histograms_put_the_overflow_in_the_last_bucket() {
        let counts = histogram([0, 1, 9, 10, 99, 500].into_iter(), 100, 10);
        assert_eq!(counts.len(), 10);
        assert_eq!(counts[0], 3.);
        assert_eq!(counts[1], 1.);
        assert_eq!(counts[9], 2.);
    }

    #[test]
    fn summaries_and_csv_count_what_fits() {
        let fit = Sample {
            seed: 3,
            error: None,
            attempts: 2,
            rooms: 12,
            budget: Budget {
                actors: 20,
                quads: 2000,
                boxes: 300,
                navigation: Some(400),
                tiles: 0,
            },
            over: Vec::new(),
        };
        let mut heavy = fit.clone();
        heavy.seed = 4;
        heavy.over = vec!["quads"];
        let mut broken = fit.clone();
        broken.seed = 5;
        broken.error = Some("no room".into());
        let samples = [fit, heavy, broken];
        assert_eq!(
            summary(&samples),
            "1 of 3 seeds fit the console · 1 over budget · 1 failed (over: quads 1)"
        );
        let text = csv(&samples);
        assert_eq!(text.lines().count(), 2);
        assert!(text.ends_with("3,12,2,20,2000,300,400,0\n"));
    }
}
