//! Project I/O runs before constructing UI state, with the splash kept responsive.
use crate::{
    editor::PreparedProject,
    workspace::{self, Project},
};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
    time::Instant,
};

pub const SPLASH_SIZE: [u32; 2] = [640, 400];

/// Restore the normal editor/Hub window after either a successful or failed open.
pub struct WindowPresentation {
    size: winit::dpi::PhysicalSize<u32>,
    position: Option<winit::dpi::PhysicalPosition<i32>>,
    maximized: bool,
}
impl WindowPresentation {
    pub fn enter(window: &winit::window::Window) -> Self {
        let maximized = window.is_maximized();
        window.set_maximized(false);
        let previous = Self {
            size: window.inner_size(),
            position: window.outer_position().ok(),
            maximized,
        };
        window.set_min_inner_size(None::<winit::dpi::LogicalSize<u32>>);
        window.set_resizable(false);
        window.set_decorations(false);
        let size = winit::dpi::LogicalSize::new(SPLASH_SIZE[0], SPLASH_SIZE[1]);
        let _ = window.request_inner_size(size);
        if let Some(monitor) = window.current_monitor() {
            let origin = monitor.position();
            let monitor_size = monitor.size();
            let size = size.to_physical::<u32>(window.scale_factor());
            window.set_outer_position(winit::dpi::PhysicalPosition::new(
                origin.x + monitor_size.width.saturating_sub(size.width) as i32 / 2,
                origin.y + monitor_size.height.saturating_sub(size.height) as i32 / 2,
            ));
        }
        previous
    }

    pub fn restore(self, window: &winit::window::Window, minimum: [u32; 2]) {
        window.set_decorations(true);
        window.set_resizable(true);
        window.set_min_inner_size(Some(winit::dpi::LogicalSize::new(minimum[0], minimum[1])));
        let _ = window.request_inner_size(self.size);
        if let Some(position) = self.position {
            window.set_outer_position(position);
        }
        window.set_maximized(self.maximized);
    }
}

pub enum Request {
    Open(PathBuf),
    Created(Box<Project>),
}
impl Request {
    fn label(&self) -> String {
        match self {
            Self::Open(path) => path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            Self::Created(project) => project.manifest.name.clone(),
        }
    }
}

pub struct Ready {
    pub project: PreparedProject,
    pub warning: Option<String>,
}
enum Event {
    Stage(&'static str),
    Ready(Box<Result<Ready, String>>),
}
pub struct Loading {
    request: Option<Request>,
    receiver: Option<Receiver<Event>>,
    pub name: String,
    pub stage: &'static str,
    started: Instant,
    registry: PathBuf,
}
impl Loading {
    pub fn new(request: Request, registry: PathBuf) -> Self {
        Self {
            name: request.label(),
            request: Some(request),
            receiver: None,
            stage: "Opening project",
            started: Instant::now(),
            registry,
        }
    }

    /// Called only after presenting a splash frame, even for very small projects.
    pub fn start(&mut self) {
        let Some(request) = self.request.take() else {
            return;
        };
        let registry = self.registry.clone();
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        let failure = tx.clone();
        if let Err(error) = std::thread::Builder::new()
            .name("epok-project-loading".into())
            .spawn(move || {
                let result = (|| {
                    let opened_at = crate::scene_loading::Message::new("Opening project");
                    let opening = Instant::now();
                    let project = match request {
                        Request::Open(path) => Project::open(&path)?,
                        Request::Created(project) => *project,
                    };
                    let warning = workspace::update_recent(
                        &registry,
                        &project.root,
                        Some(&project.manifest.name),
                    )
                    .err();
                    let opened = crate::scene_loading::Message::new(format!(
                        "Opening project: {:.1} ms",
                        opening.elapsed().as_secs_f64() * 1000.
                    ));
                    let mut project = PreparedProject::load(project, |stage| {
                        let _ = tx.send(Event::Stage(stage));
                    })?;
                    project.messages.splice(0..0, [opened_at, opened]);
                    Ok(Ready { project, warning })
                })();
                let _ = tx.send(Event::Ready(Box::new(result)));
            })
        {
            let _ = failure.send(Event::Ready(Box::new(Err(format!(
                "Cannot start project loader: {error}"
            )))));
        }
    }

    pub fn poll(&mut self) -> Option<Result<Ready, String>> {
        let receiver = self.receiver.as_ref()?;
        loop {
            match receiver.try_recv() {
                Ok(Event::Stage(stage)) => self.stage = stage,
                Ok(Event::Ready(result)) => return Some(*result),
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    return Some(Err("Project loading stopped unexpectedly.".into()));
                }
            }
        }
    }

    pub fn draw(&self, ui: &imgui::Ui, logo: imgui::TextureId, artwork: imgui::TextureId) {
        use imgui::{Condition, StyleColor, StyleVar, WindowFlags};
        let size = ui.io().display_size;
        let _padding = ui.push_style_var(StyleVar::WindowPadding([0., 0.]));
        let _background = ui.push_style_color(StyleColor::WindowBg, [0.075, 0.075, 0.082, 1.]);
        ui.window("Project loading###Splash")
            .position([0., 0.], Condition::Always)
            .size(size, Condition::Always)
            .flags(
                WindowFlags::NO_DECORATION
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SAVED_SETTINGS
                    | WindowFlags::NO_DOCKING
                    | WindowFlags::NO_INPUTS,
            )
            .build(|| {
                let draw = ui.get_window_draw_list();
                let footer = size[1] - 66.;
                draw.add_image(artwork, [0., 0.], [size[0], footer])
                    .uv_min([0., 0.036])
                    .uv_max([1., 0.964])
                    .build();
                // Overlay the original brand asset without regenerating its lettering.
                draw.add_image(logo, [22., footer * 0.27], [310., footer * 0.27 + 144.])
                    .build();
                draw.add_rect([0., footer], size, [0.045, 0.049, 0.057, 1.])
                    .filled(true)
                    .build();
                draw.add_text(
                    [24., footer + 10.],
                    [0.92, 0.93, 0.96, 1.],
                    shorten(ui, &self.name, size[0] - 48.),
                );
                draw.add_text(
                    [24., footer + 34.],
                    [0.59, 0.64, 0.69, 1.],
                    shorten(ui, self.stage, size[0] - 48.),
                );
                let version = concat!("EPOK ", env!("CARGO_PKG_VERSION"));
                draw.add_text(
                    [size[0] - ui.calc_text_size(version)[0] - 24., 18.],
                    [0.60, 0.65, 0.70, 1.],
                    version,
                );
                let y = size[1] - 3.;
                draw.add_rect([0., y], size, [0.14, 0.18, 0.21, 1.])
                    .filled(true)
                    .build();
                // Indeterminate activity, not a fabricated completion percentage.
                let travel = (self.started.elapsed().as_secs_f32() * 1.8).sin() * 0.5 + 0.5;
                let width = size[0] * 0.18;
                let x = travel * (size[0] - width);
                draw.add_rect([x, y], [x + width, size[1]], [0.26, 0.73, 0.86, 1.])
                    .filled(true)
                    .build();
            });
    }
}
fn shorten(ui: &imgui::Ui, text: &str, width: f32) -> String {
    let mut result = text.to_string();
    if ui.calc_text_size(&result)[0] > width {
        while !result.is_empty() && ui.calc_text_size(format!("{result}..."))[0] > width {
            result.pop();
        }
        result.push_str("...");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loader_waits_for_presentation_and_returns_open_errors() {
        let root = std::env::temp_dir().join(format!("epok-missing-{}", uuid::Uuid::new_v4()));
        let mut loading = Loading::new(Request::Open(root.clone()), root.join("recent.epokprefs"));
        assert!(loading.poll().is_none());
        assert!(
            loading.receiver.is_none(),
            "Opening must wait until a splash frame is presented"
        );
        loading.start();
        loading.start(); // A redraw must never start a second project load.
        let deadline = Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if let Some(result) = loading.poll() {
                assert!(
                    result.is_err(),
                    "Invalid projects return to the Hub with an error"
                );
                break;
            }
            assert!(Instant::now() < deadline, "Project loader timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(!root.exists());
    }
}
