use crate::{editor::Editor, play::Serial, serial_support::Port};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Scan,
    Prepare,
    Test,
}

#[derive(Default)]
pub struct State {
    pub open: bool,
    pub draft: Option<Serial>,
    pub ports: Vec<Port>,
    pub status: String,
    pub error: Option<String>,
    pub tools_ready: bool,
    pub play_after_prepare: bool,
    pub command_pending: bool,
}

pub fn start(e: &mut Editor, action: Action) {
    if e.job.is_some() || e.dependencies.busy() || e.assets.busy || e.bake_job.is_some() {
        return;
    }
    e.last_error = None;
    e.serial_ui.error = None;
    e.serial_ui.status = "Checking connection...".into();
    e.job_stage = "Preparing PSX connection".into();
    e.job_progress = None;
    e.job = Some(crate::pipeline::Job::serial_setup(
        e.preferences.serial.clone(),
        action,
    ));
}

pub fn open(e: &mut Editor) {
    e.serial_ui.open = true;
    e.serial_ui.draft = Some(e.preferences.serial.clone());
    e.serial_ui.tools_ready = crate::serial_support::tools_installed();
    start(e, Action::Scan);
}

pub fn window(ui: &imgui::Ui, e: &mut Editor) {
    if !e.serial_ui.open {
        return;
    }
    let mut state = std::mem::take(&mut e.serial_ui);
    let mut open = true;
    let mut action = None;
    let mut save = false;
    let mut play = false;
    let mut close = false;
    let screen = ui.io().display_size;
    ui.window("PSX connection").opened(&mut open)
        .position([screen[0]*0.5, screen[1]*0.5], imgui::Condition::Appearing).position_pivot([0.5,0.5])
        .size([640.,500.], imgui::Condition::FirstUseEver).size_constraints([520.,420.],[1000.,900.]).build(|| {
            ui.text("PSX via serial");
            ui.text_disabled(crate::serial_support::host_label());
            if cfg!(target_os="linux") { ui.text_wrapped("Linux serial support is experimental. Mono and device access must be provided by your distribution."); }
            ui.text_wrapped("Connect your adapter and start Unirom on the console. Play enables the resident debug handler, sends your program and keeps the monitor connected. Pause, Continue and Reset PSX are available in Game. Components are managed inside this Epok installation.");
            ui.text(if state.tools_ready { "Serial components installed" } else { "Serial components need installation or repair" });
            ui.text_wrapped(format!("Tools: {}",crate::serial_support::managed_directory().display()));
            ui.separator();
            let disabled = ui.begin_disabled(e.job.is_some() || e.dependencies.busy());
            let draft = state.draft.get_or_insert_with(|| e.preferences.serial.clone());
            ui.text("Adapter");
            let label = state.ports.iter().find(|p| p.path == draft.port).map(|p| p.label.as_str())
                .unwrap_or(if draft.port.is_empty() {"Automatic (one USB adapter)"} else {&draft.port});
            ui.set_next_item_width(-1.);
            if let Some(_combo) = ui.begin_combo("##psx-adapter",label) {
                if ui.selectable_config("Automatic (one USB adapter)").selected(draft.port.is_empty()).build() {
                    draft.port.clear(); draft.device_id=None; save=true;
                }
                for port in &state.ports {
                    if ui.selectable_config(&port.label).selected(port.path==draft.port).build() {
                        draft.port=port.path.clone(); draft.device_id=port.identity.clone(); save=true;
                    }
                }
            }
            if ui.button("Refresh adapters") { action=Some(Action::Scan); }
            ui.same_line();
            if ui.button("Optional Unirom ping") { action=Some(Action::Test); }
            ui.same_line();
            if ui.button(if state.tools_ready {"Repair components"} else {"Download components"}) { action=Some(Action::Prepare); }
            ui.text_disabled("A successful ping is not required to send a program.");
            if state.ports.is_empty() { ui.text_wrapped("No adapters listed yet. Refresh after connecting the cable. Tool preparation works with the console switched off."); }
            ui.spacing();
            if let Some(error)=&state.error { ui.text_colored([1.,0.65,0.3,1.],"Connection needs attention"); ui.text_wrapped(error); }
            else if !state.status.is_empty() { ui.text_wrapped(&state.status); }
            if ui.collapsing_header("Advanced",imgui::TreeNodeFlags::empty()) {
                ui.set_next_item_width(-1.);
                if ui.input_text("##serial-port",&mut draft.port).hint("COM port or /dev device path").build() { draft.device_id=None; }
                ui.checkbox("Fast transfer",&mut draft.fast);
                ui.text_wrapped("Standard speed is the default. Fast mode also depends on the adapter and Unirom; the operating system alone cannot guarantee it.");
                if ui.button("Save connection settings") {save=true;}
            }
            ui.separator();
            if ui.button(if state.tools_ready {"Play on PSX"} else {"Download and Play on PSX"}) { play=true; }
            ui.same_line();
            if ui.button("Close") {close=true;}
            drop(disabled);
        });
    state.open = open && !close;
    if save || action.is_some() || play {
        let mut prefs = e.preferences.clone();
        prefs.serial = state.draft.as_ref().unwrap().clone();
        match prefs.save() {
            Ok(()) => {
                // Saving an adapter must not reset the current viewport's grid or camera speed.
                e.preferences.serial = prefs.serial;
                if let Some(draft) = e.settings.preferences.as_mut() {
                    draft.serial = e.preferences.serial.clone();
                }
                if save {
                    state.error = None;
                    state.status = "Settings saved. Start the Unirom loader, then Play.".into();
                }
            }
            Err(error) => {
                state.error = Some(error);
                action = None;
                play = false;
            }
        }
    }
    e.serial_ui = state;
    if let Some(action) = action {
        start(e, action);
    }
    if play {
        let mut profile = e.play_profile.clone();
        profile.target = crate::play::Target::Serial;
        if let Err(error) = e.set_play_profile(profile) {
            e.serial_ui.error = Some(error);
        } else if !crate::serial_support::tools_installed() {
            e.serial_ui.play_after_prepare = true;
            start(e, Action::Prepare);
        } else {
            e.build(true);
        }
    }
}
