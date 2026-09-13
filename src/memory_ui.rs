use crate::{
    editor::Editor,
    memory::{Node, Report, Space},
};
use imgui::{StyleColor as C, Ui};

#[derive(Default)]
pub struct State {
    pub open: bool,
    pub pending: bool,
    pub summary: Option<crate::build_report::Summary>,
    pub status_current: bool,
    pub report_only: bool,
    pub building_scene_signature: String,
    pub prompt: Option<Prompt>,
    pub report: Option<Report>,
    pub error: Option<String>,
    pub stale: bool,
    pub scene_signature: String,
    pub profile: crate::play::Profile,
    pub debug: bool,
    pub path: Vec<usize>,
    pub selected: Option<usize>,
    tab: usize,
    bank: usize,
    layout: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prompt { BuildFirst, GenerateMissing { automatic: bool } }

pub fn button(ui: &Ui) -> bool {
    let result = ui.button_with_size("##memory-analyzer", [30., ui.frame_height()]);
    let a = ui.item_rect_min();
    let b = ui.item_rect_max();
    let c = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let draw = ui.get_window_draw_list();
    let color = ui.style_color(C::Text);
    draw.add_circle(c, 7., color)
        .num_segments(24)
        .thickness(1.5)
        .build();
    draw.add_line(c, [c[0], c[1] - 7.], color)
        .thickness(1.5)
        .build();
    draw.add_line(c, [c[0] + 7., c[1]], color)
        .thickness(1.5)
        .build();
    if ui.is_item_hovered() {
        ui.tooltip_text("Asset and memory report\nView the report from the last build.");
    }
    result
}
pub fn size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MiB", bytes as f64 / 1048576.)
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.)
    } else {
        format!("{bytes} B")
    }
}
fn color(ui: &Ui, index: usize) -> [f32; 4] {
    let colors = [
        [0.28, 0.62, 0.81, 1.],
        [0.37, 0.67, 0.46, 1.],
        [0.71, 0.54, 0.79, 1.],
        [0.79, 0.57, 0.29, 1.],
        [0.73, 0.40, 0.42, 1.],
        [0.34, 0.68, 0.65, 1.],
    ];
    let mut c = colors[index % colors.len()];
    let bg = ui.style_color(C::WindowBg);
    for i in 0..3 {
        c[i] = c[i] * 0.68 + bg[i] * 0.32;
    }
    c
}

/// Binary treemap. Partition in byte proportions before adding visual gutters.
fn partition(items: &[(usize, u64)], rect: [f32; 4], output: &mut Vec<(usize, [f32; 4])>) {
    if items.is_empty() {
        return;
    }
    if items.len() == 1 {
        output.push((items[0].0, rect));
        return;
    }
    let total = items.iter().map(|(_, v)| *v).sum::<u64>() as f64;
    if total == 0. {
        return;
    }
    let mut split = 1;
    let mut sum = items[0].1 as f64;
    while split < items.len() - 1
        && (sum + items[split].1 as f64 - total / 2.).abs() < (sum - total / 2.).abs()
    {
        sum += items[split].1 as f64;
        split += 1;
    }
    let ratio = (sum / total) as f32;
    let [x, y, w, h] = rect;
    if w >= h {
        partition(&items[..split], [x, y, w * ratio, h], output);
        partition(
            &items[split..],
            [x + w * ratio, y, w * (1. - ratio), h],
            output,
        );
    } else {
        partition(&items[..split], [x, y, w, h * ratio], output);
        partition(
            &items[split..],
            [x, y + h * ratio, w, h * (1. - ratio)],
            output,
        );
    }
}
fn tooltip(ui: &Ui, node: &Node) {
    ui.tooltip(|| {
        ui.text_wrapped(&node.name);
        ui.text(format!("{} ({} bytes)", size(node.bytes), node.bytes));
        if !node.detail.is_empty() {
            ui.text_wrapped(&node.detail);
        }
        if let Some(asset) = &node.asset {
            ui.text_wrapped(asset);
        }
        if !node.scenes.is_empty() {
            ui.text_wrapped(format!("Used by: {}", node.scenes.join(", ")));
        }
        if !node.children.is_empty() {
            ui.text_disabled("Click to explore");
        }
    });
}
fn plot(ui: &Ui, node: &Node, height: f32, base_color: Option<usize>) -> Option<usize> {
    let origin = ui.cursor_screen_pos();
    let width = ui.content_region_avail()[0].max(1.);
    let mut items = node
        .children
        .iter()
        .enumerate()
        .filter(|(_, n)| n.bytes > 0)
        .map(|(i, n)| (i, n.bytes))
        .collect::<Vec<_>>();
    items.sort_by_key(|(_, v)| std::cmp::Reverse(*v));
    let mut boxes = Vec::new();
    partition(&items, [origin[0], origin[1], width, height], &mut boxes);
    let mut clicked = None;
    for (i, [x, y, w, h]) in boxes {
        if w <= 3. || h <= 3. {
            continue;
        }
        let _id = ui.push_id_usize(i);
        ui.set_cursor_screen_pos([x, y]);
        if ui.invisible_button("memory-block", [w - 2., h - 2.]) {
            clicked = Some(i);
        }
        let draw = ui.get_window_draw_list();
        let n = &node.children[i];
        let fill = if n.name.starts_with("Unassigned") || n.name.starts_with("Unallocated") {
            ui.style_color(C::FrameBg)
        } else {
            color(ui, base_color.unwrap_or(i))
        };
        draw.add_rect([x, y], [x + w - 2., y + h - 2.], fill)
            .filled(true)
            .build();
        if w > 72. && h > ui.text_line_height() * 2. + 12. {
            draw.with_clip_rect([x + 6., y + 4.], [x + w - 6., y + h - 6.], || {
                draw.add_text([x + 7., y + 6.], ui.style_color(C::Text), &n.name);
                draw.add_text(
                    [x + 7., y + 8. + ui.text_line_height()],
                    ui.style_color(C::Text),
                    size(n.bytes),
                );
            });
        }
        if ui.is_item_hovered() {
            tooltip(ui, n);
        }
    }
    ui.set_cursor_screen_pos(origin);
    ui.dummy([width, height]);
    clicked
}
fn space(ui: &Ui, space: &Space, state: &mut State) -> Option<String> {
    if let Some(capacity) = space.capacity {
        ui.text(format!(
            "{} accounted / {}",
            size(space.used),
            size(capacity)
        ));
        ui.same_line();
        if space.used > capacity {
            ui.text_colored(
                [1., 0.4, 0.3, 1.],
                format!("{} over budget", size(space.used - capacity)),
            );
        } else {
            ui.text_disabled(format!("{} unassigned", size(capacity - space.used)));
        }
        imgui::ProgressBar::new((space.used as f64 / capacity as f64) as f32)
            .size([-1., 12.])
            .overlay_text("")
            .build(ui);
    } else {
        ui.text(format!("{} total payload", size(space.used)));
    }
    let mut node = &space.root;
    let mut truncate = None;
    if ui.small_button(&space.root.name) {
        truncate = Some(0);
    }
    for (depth, i) in state.path.iter().enumerate() {
        if let Some(child) = node.children.get(*i) {
            node = child;
            ui.same_line();
            ui.text_disabled("/");
            ui.same_line();
            if ui.small_button(format!("{}##crumb-{depth}", node.name)) {
                truncate = Some(depth + 1);
            }
        } else {
            truncate = Some(depth);
            break;
        }
    }
    if let Some(depth) = truncate {
        state.path.truncate(depth);
        state.selected = None;
        return None;
    }
    let height = (ui.content_region_avail()[1] * 0.48).clamp(130., 245.);
    let mut clicked = plot(ui, node, height, state.path.first().copied());
    if let Some(_table) = ui.begin_table_with_flags(
        "memory-rows",
        3,
        imgui::TableFlags::BORDERS_INNER_H
            | imgui::TableFlags::ROW_BG
            | imgui::TableFlags::SIZING_STRETCH_PROP,
    ) {
        ui.table_setup_column("Resource / allocation");
        ui.table_setup_column("Bytes");
        ui.table_setup_column("Share");
        ui.table_headers_row();
        for (i, n) in node.children.iter().enumerate() {
            ui.table_next_row();
            ui.table_next_column();
            let _id = ui.push_id_usize(i);
            // Hidden IDs preserve uniqueness for aliased/duplicate source labels.
            let mut label = n.name.clone();
            let width = ui.content_region_avail()[0] - 20.;
            if ui.calc_text_size(&label)[0] > width {
                while !label.is_empty() && ui.calc_text_size(format!("{label}..."))[0] > width {
                    label.pop();
                }
                label.push_str("...");
            }
            if ui
                .selectable_config(format!(
                    "{}{label}##memory-row",
                    if n.children.is_empty() { "" } else { "> " }
                ))
                .selected(state.selected == Some(i))
                .build()
            {
                clicked = Some(i);
            }
            if ui.is_item_hovered() {
                tooltip(ui, n);
            }
            ui.table_next_column();
            ui.text(format!("{}", n.bytes));
            ui.table_next_column();
            ui.text(format!(
                "{:.1}%",
                n.bytes as f64 / node.bytes.max(1) as f64 * 100.
            ));
        }
    }
    if let Some(i) = clicked {
        if !node.children[i].children.is_empty() {
            state.path.push(i);
            state.selected = None;
        } else {
            state.selected = Some(i);
        }
    }
    let mut asset = None;
    if let Some(n) = state.selected.and_then(|i| node.children.get(i)) {
        ui.separator();
        ui.text_wrapped(&n.name);
        ui.text_wrapped(&n.detail);
        if !n.scenes.is_empty() {
            ui.text_wrapped(format!("Used by: {}", n.scenes.join(", ")));
        }
        if let Some(path) = &n.asset
            && ui.button("Show asset in Project")
        {
            asset = Some(path.clone());
        }
    } else if let Some(path) = &node.asset
        && ui.button("Show asset in Project")
    {
        asset = Some(path.clone());
    }
    asset
}
fn vram_layout(ui: &Ui, scene: &crate::memory::SceneReport) {
    ui.text(format!(
        "{} accounted / 1 MiB · 1024 x 512 words",
        size(scene.vram.used)
    ));
    let a = ui.cursor_screen_pos();
    let w = ui.content_region_avail()[0];
    let scale = w / 1024.;
    let h = w * 0.5;
    let draw = ui.get_window_draw_list();
    draw.add_rect(a, [a[0] + w, a[1] + h], ui.style_color(C::FrameBg))
        .filled(true)
        .build();
    for (i, r) in scene.rectangles.iter().enumerate() {
        let [x, y, rw, rh] = r.rect;
        let p = [a[0] + f32::from(x) * scale, a[1] + f32::from(y) * scale];
        let q = [p[0] + f32::from(rw) * scale, p[1] + f32::from(rh) * scale];
        draw.add_rect(p, q, color(ui, r.category))
            .filled(true)
            .build();
        if q[0] - p[0] > 110. && q[1] - p[1] > 30. {
            draw.with_clip_rect(p, q, || {
                draw.add_text([p[0] + 5., p[1] + 5.], ui.style_color(C::Text), &r.name)
            });
        }
        let _id = ui.push_id_usize(i);
        ui.set_cursor_screen_pos(p);
        ui.invisible_button(
            "vram-region",
            [(q[0] - p[0]).max(1.), (q[1] - p[1]).max(1.)],
        );
        if ui.is_item_hovered() {
            ui.tooltip_text(format!(
                "{}\n({}, {}) · {} x {} words\n{} bytes",
                r.name,
                x,
                y,
                rw,
                rh,
                u32::from(rw) * u32::from(rh) * 2
            ));
        }
    }
    ui.set_cursor_screen_pos(a);
    ui.dummy([w, h]);
    ui.text_wrapped("Framebuffers | Textures | Palettes | Reserved layout. Unallocated space is subject to the texture allocator's placement constraints.");
}
pub fn window(ui: &Ui, e: &mut Editor) {
    if let Some(prompt) = e.memory.prompt {
        let title = "Asset report required";
        if !crate::busy_ui::popup_open(title) { ui.open_popup(title); }
        let mut accepted = false;
        ui.modal_popup_config(title).always_auto_resize(true).build(|| {
            match prompt {
                Prompt::BuildFirst => ui.text("A verified compilation is required first. Build and generate the report?"),
                Prompt::GenerateMissing { automatic: false } => ui.text("Automatic asset reports were disabled for this build. Generate its report now?"),
                Prompt::GenerateMissing { automatic: true } => ui.text("This build has no available asset report. Generate it now?"),
            }
            if e.memory.stale && prompt != Prompt::BuildFirst {
                ui.text_colored([1.,0.72,0.3,1.], "The report will describe the previous build. Pending changes are not included.");
            }
            if ui.button("OK") { accepted = true; e.memory.prompt = None; ui.close_current_popup(); }
            ui.same_line();
            if ui.button("Cancel") { e.memory.prompt = None; ui.close_current_popup(); }
        });
        if accepted {
            match prompt {
                Prompt::BuildFirst => e.analyze_memory(),
                Prompt::GenerateMissing { .. } => e.generate_memory_report(),
            }
        }
        return;
    }
    let mut state = std::mem::take(&mut e.memory);
    let name = "Memory Analyzer";
    if state.open && !crate::busy_ui::popup_open(name) {
        ui.open_popup(name);
    }
    if !crate::busy_ui::popup_open(name) {
        e.memory = state;
        return;
    }
    state.stale |= state.profile != e.play_profile
        || state.debug != e.blueprint_debug_enabled
        || state.scene_signature != crate::scene_dependencies::signature(&e.scene);
    let [w, h] = ui.io().display_size;
    unsafe {
        imgui::sys::igSetNextWindowPos(
            [w * 0.5, h * 0.5].into(),
            imgui::sys::ImGuiCond_Appearing as i32,
            [0.5, 0.5].into(),
        );
        imgui::sys::igSetNextWindowSize(
            [960_f32.min(w - 32.), 740_f32.min(h - 40.)].into(),
            imgui::sys::ImGuiCond_Appearing as i32,
        );
    }
    let mut open = true;
    let mut close = false;
    let mut rebuild = false;
    let mut asset = None;
    ui.modal_popup_config(name).opened(&mut open).save_settings(false).build(||{
        ui.text_wrapped(format!("{} / {} / {}{}",state.profile.target.label(),crate::play_ui::content_label(state.profile.content),state.profile.data.label(),if state.debug{" / Blueprint debug"}else{""}));
        if state.stale {ui.text_colored([1.,0.72,0.3,1.],"Previous build: sources or Play settings changed. Build again for current information.");}
        if ui.button("Build and update report"){rebuild=true;ui.close_current_popup();}
        ui.same_line();if ui.button("Close"){close=true;ui.close_current_popup();}
        ui.separator();
        if let Some(error)=&state.error {ui.text_colored([1.,0.5,0.4,1.],"Analysis could not complete");ui.text_wrapped(error);ui.text_wrapped("No current memory total is available. Build details are in Console.");return;}
        let Some(report)=state.report.take() else{ui.text("No compiled report available.");return;};
        ui.text_wrapped(format!("Included scenes: {}",report.scenes.iter().map(|s|s.name.as_str()).collect::<Vec<_>>().join(", ")));
        for (i,label) in ["Main RAM","VRAM","Audio / SPU","Files","Scratchpad"].iter().enumerate(){
            if i>0{ui.same_line();}
            if ui.selectable_config(label).selected(state.tab==i).size([ui.calc_text_size(label)[0]+18.,ui.frame_height()]).build(){state.tab=i;state.path.clear();state.selected=None;}
        }
        if state.tab==1 && !report.scenes.is_empty(){
            state.bank=state.bank.min(report.scenes.len()-1);
            ui.set_next_item_width(240.);
            let names=report.scenes.iter().map(|s|s.name.as_str()).collect::<Vec<_>>();
            if ui.combo_simple_string("Active scene",&mut state.bank,&names){state.path.clear();state.selected=None;}
            ui.same_line();ui.checkbox("Physical VRAM layout",&mut state.layout);
        }
        ui.child_window("memory-content").size([0.,-ui.text_line_height_with_spacing()*2.]).build(||{
            if state.tab==1 && state.layout && let Some(scene)=report.scenes.get(state.bank){vram_layout(ui,scene);}
            else {
                let selected=match state.tab{1=>report.scenes.get(state.bank).map(|s|&s.vram).unwrap_or(&report.ram),2=>&report.spu,3=>&report.files,4=>&report.scratchpad,_=>&report.ram};
                asset=space(ui,selected,&mut state);
            }
            if ui.collapsing_header("Accounting details",imgui::TreeNodeFlags::empty()){
                for warning in &report.warnings{ui.bullet_text(warning);}
                ui.text_wrapped(format!("EXE SHA-256: {}",report.executable_hash));
            }
        });
        ui.text_disabled("Build-time allocations. Heap / stack / transition peaks: not measured.");
        if asset.is_some(){close=true;ui.close_current_popup();}
        state.report=Some(report);
    });
    state.open = open && !close && !rebuild;
    e.memory = state;
    if let Some(path) = asset
        && let Ok(path) = crate::assets::inside(&e.root, &path)
    {
        crate::project_browser::State::select_asset(e, &path);
    }
    if rebuild {
        e.analyze_memory();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn treemap_area_is_proportional_and_tiles_stay_inside() {
        let mut tiles = vec![];
        partition(
            &[(0, 1024), (1, 256), (2, 512), (3, 256)],
            [0., 0., 640., 300.],
            &mut tiles,
        );
        let values = [1024., 256., 512., 256.];
        for (i, [x, y, w, h]) in &tiles {
            assert!(*x >= 0. && *y >= 0. && x + w <= 640.01 && y + h <= 300.01);
            assert!((w * h / (640. * 300.) - values[*i] / 2048.).abs() < 0.0001);
        }
        assert!((tiles.iter().map(|(_, r)| r[2] * r[3]).sum::<f32>() - 640. * 300.).abs() < 0.1);
    }
}
