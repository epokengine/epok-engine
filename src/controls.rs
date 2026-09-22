//! Project-owned mappings from desktop devices to PlayStation virtual pads.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Button {
    Select,
    L3,
    R3,
    Start,
    Up,
    Right,
    Down,
    Left,
    L2,
    R2,
    L1,
    R1,
    Triangle,
    Circle,
    Cross,
    Square,
}
impl Button {
    pub const ALL: [Self; 16] = [
        Self::Select,
        Self::L3,
        Self::R3,
        Self::Start,
        Self::Up,
        Self::Right,
        Self::Down,
        Self::Left,
        Self::L2,
        Self::R2,
        Self::L1,
        Self::R1,
        Self::Triangle,
        Self::Circle,
        Self::Cross,
        Self::Square,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Select => "SELECT",
            Self::L3 => "L3",
            Self::R3 => "R3",
            Self::Start => "START",
            Self::Up => "D-pad Up",
            Self::Right => "D-pad Right",
            Self::Down => "D-pad Down",
            Self::Left => "D-pad Left",
            Self::L2 => "L2",
            Self::R2 => "R2",
            Self::L1 => "L1",
            Self::R1 => "R1",
            Self::Triangle => "Triangle",
            Self::Circle => "Circle",
            Self::Cross => "Cross",
            Self::Square => "Square",
        }
    }
    pub fn bit(self) -> u16 {
        1 << self as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    LeftX,
    LeftY,
    RightX,
    RightY,
}
impl Axis {
    pub const ALL: [Self; 4] = [Self::LeftX, Self::LeftY, Self::RightX, Self::RightY];
    pub fn label(self) -> &'static str {
        match self {
            Self::LeftX => "Left stick X",
            Self::LeftY => "Left stick Y",
            Self::RightX => "Right stick X",
            Self::RightY => "Right stick Y",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}
impl MouseButton {
    pub fn label(self) -> &'static str {
        match self {
            Self::Left => "Mouse Left",
            Self::Right => "Mouse Right",
            Self::Middle => "Mouse Middle",
            Self::Back => "Mouse Back",
            Self::Forward => "Mouse Forward",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseAxis {
    X,
    Y,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "device", rename_all = "snake_case")]
pub enum Binding {
    Keyboard { key: String },
    MouseButton { button: MouseButton },
    MouseAxis { axis: MouseAxis },
    GamepadButton { button: String },
    GamepadAxis { axis: GamepadAxis, positive: bool },
}
impl Binding {
    pub fn label(&self) -> String {
        match self {
            Self::Keyboard { key } => key.replace("Key", "").replace("Digit", ""),
            Self::MouseButton { button } => button.label().into(),
            Self::MouseAxis { axis } => {
                format!("Mouse {}", if *axis == MouseAxis::X { "X" } else { "Y" })
            }
            Self::GamepadButton { button } => format!("Pad {button}"),
            Self::GamepadAxis { axis, positive } => {
                format!("Pad {:?} {}", axis, if *positive { "+" } else { "-" })
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Control {
    Button(Button),
    Axis(Axis),
}
impl Control {
    pub fn label(self) -> &'static str {
        match self {
            Self::Button(b) => b.label(),
            Self::Axis(a) => a.label(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Mapping {
    pub control: Control,
    pub binding: Binding,
}
impl Default for Mapping {
    fn default() -> Self {
        Self {
            control: Control::Button(Button::Cross),
            binding: Binding::Keyboard { key: "KeyX".into() },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Profile {
    pub id: uuid::Uuid,
    pub name: String,
    pub mappings: Vec<Mapping>,
}
impl Default for Profile {
    fn default() -> Self {
        Self::keyboard_mouse()
    }
}
impl Profile {
    pub fn keyboard_mouse() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: "Keyboard & Mouse".into(),
            mappings: vec![
                map(Button::Up, "KeyW"),
                map(Button::Left, "KeyA"),
                map(Button::Down, "KeyS"),
                map(Button::Right, "KeyD"),
                map(Button::Cross, "KeyK"),
                map(Button::Circle, "KeyL"),
                map(Button::Square, "KeyJ"),
                map(Button::Triangle, "KeyI"),
                map(Button::L1, "KeyQ"),
                map(Button::R1, "KeyE"),
                map(Button::L2, "Digit1"),
                map(Button::R2, "Digit3"),
                map(Button::Start, "Enter"),
                map(Button::Select, "Backspace"),
                Mapping {
                    control: Control::Axis(Axis::RightX),
                    binding: Binding::MouseAxis { axis: MouseAxis::X },
                },
                Mapping {
                    control: Control::Axis(Axis::RightY),
                    binding: Binding::MouseAxis { axis: MouseAxis::Y },
                },
            ],
        }
    }
    pub fn binding(&self, control: Control) -> Option<&Binding> {
        self.mappings
            .iter()
            .find(|m| m.control == control)
            .map(|m| &m.binding)
    }
    pub fn set(&mut self, control: Control, binding: Binding) {
        if let Some(m) = self.mappings.iter_mut().find(|m| m.control == control) {
            m.binding = binding
        } else {
            self.mappings.push(Mapping { control, binding })
        }
    }
}
fn map(button: Button, key: &str) -> Mapping {
    Mapping {
        control: Control::Button(button),
        binding: Binding::Keyboard { key: key.into() },
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Slot {
    pub enabled: bool,
    pub profile: Option<uuid::Uuid>,
}
impl Default for Slot {
    fn default() -> Self {
        Self {
            enabled: false,
            profile: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub profiles: Vec<Profile>,
    pub pads: [Slot; 4],
    pub mouse_sensitivity: u16,
}
impl Default for Settings {
    fn default() -> Self {
        let profile = Profile::keyboard_mouse();
        Self {
            pads: [
                Slot {
                    enabled: true,
                    profile: Some(profile.id),
                },
                Slot::default(),
                Slot::default(),
                Slot::default(),
            ],
            profiles: vec![profile],
            mouse_sensitivity: 16,
        }
    }
}
impl Settings {
    pub fn profile(&self, slot: usize) -> Option<&Profile> {
        let id = self.pads.get(slot)?.profile?;
        self.profiles.iter().find(|p| p.id == id)
    }
}

/// Input state is intentionally host-only. Bindings remain in the project file;
/// device state never leaks into builds or exports.
#[derive(Default)]
pub struct HostInput {
    keys: BTreeSet<String>,
    mouse: BTreeSet<MouseButton>,
    gamepad: BTreeSet<String>,
    axes: std::collections::BTreeMap<GamepadAxis, f32>,
    mouse_delta: [f32; 2],
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PadState {
    pub buttons: u16,
    pub analog: bool,
    pub axes: [u8; 4],
}
impl HostInput {
    pub fn key(&mut self, key: String, down: bool) {
        if down {
            self.keys.insert(key);
        } else {
            self.keys.remove(&key);
        }
    }
    pub fn mouse_button(&mut self, button: MouseButton, down: bool) {
        if down {
            self.mouse.insert(button);
        } else {
            self.mouse.remove(&button);
        }
    }
    pub fn mouse_motion(&mut self, delta: [f32; 2]) {
        self.mouse_delta[0] += delta[0];
        self.mouse_delta[1] += delta[1];
    }
    pub fn gamepad_button(&mut self, button: String, down: bool) {
        if down {
            self.gamepad.insert(button);
        } else {
            self.gamepad.remove(&button);
        }
    }
    pub fn gamepad_axis(&mut self, axis: GamepadAxis, value: f32) {
        self.axes.insert(axis, value.clamp(-1., 1.));
    }
    pub fn clear_motion(&mut self) {
        self.mouse_delta = [0.; 2];
    }
    pub fn sample(&mut self, settings: &Settings) -> [PadState; 4] {
        let mouse = self.mouse_delta;
        self.mouse_delta = [0.; 2];
        std::array::from_fn(|slot| {
            let Some(profile) = settings.pads[slot]
                .enabled
                .then(|| settings.profile(slot))
                .flatten()
            else {
                return PadState::default();
            };
            let mut state = PadState {
                analog: profile
                    .mappings
                    .iter()
                    .any(|m| matches!(m.control, Control::Axis(_))),
                axes: [128; 4],
                ..Default::default()
            };
            for m in &profile.mappings {
                match m.control {
                    Control::Button(button)
                        if self.active(&m.binding, mouse, settings.mouse_sensitivity) =>
                    {
                        state.buttons |= button.bit()
                    }
                    Control::Axis(axis) => {
                        if let Some(value) =
                            self.axis(&m.binding, mouse, settings.mouse_sensitivity)
                        {
                            state.axes[axis as usize] =
                                (128. + value * 127.).round().clamp(0., 255.) as u8;
                        }
                    }
                    _ => {}
                }
            }
            state
        })
    }
    fn active(&self, binding: &Binding, _mouse: [f32; 2], _: u16) -> bool {
        match binding {
            Binding::Keyboard { key } => self.keys.contains(key),
            Binding::MouseButton { button } => self.mouse.contains(button),
            Binding::GamepadButton { button } => self.gamepad.contains(button),
            Binding::GamepadAxis { axis, positive } => {
                self.axes.get(axis).copied().unwrap_or_default()
                    * (if *positive { 1. } else { -1. })
                    > 0.5
            }
            Binding::MouseAxis { .. } => false,
        }
    }
    fn axis(&self, binding: &Binding, mouse: [f32; 2], sensitivity: u16) -> Option<f32> {
        match binding {
            Binding::MouseAxis { axis } => Some(
                (if *axis == MouseAxis::X {
                    mouse[0]
                } else {
                    -mouse[1]
                }) * f32::from(sensitivity)
                    / 100.,
            )
            .filter(|v| v.abs() > 0.001)
            .map(|v| v.clamp(-1., 1.)),
            // A stick is signed. Capturing it while pushed either way must not
            // discard the other half of its travel. `positive` still matters
            // when an axis is deliberately bound to a digital button.
            Binding::GamepadAxis { axis, .. } => {
                Some(self.axes.get(axis).copied().unwrap_or_default())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn project_controls_roundtrip_and_drive_independent_pads() {
        let mut settings = Settings::default();
        let mut second = Profile::keyboard_mouse();
        second.name = "Second player".into();
        second.set(
            Control::Button(Button::Cross),
            Binding::Keyboard { key: "KeyP".into() },
        );
        settings.pads[1] = Slot {
            enabled: true,
            profile: Some(second.id),
        };
        settings.profiles.push(second);
        let restored: Settings =
            serde_json::from_value(serde_json::to_value(&settings).unwrap()).unwrap();
        assert_eq!(restored, settings);
        let mut input = HostInput::default();
        input.key("KeyK".into(), true);
        input.key("KeyP".into(), true);
        let pads = input.sample(&settings);
        assert_ne!(pads[0].buttons & Button::Cross.bit(), 0);
        assert_ne!(pads[1].buttons & Button::Cross.bit(), 0);
        assert_eq!(pads[2], PadState::default());
        assert_eq!(pads[3], PadState::default());
    }
    #[test]
    fn mouse_axis_is_consumed_once_per_frame() {
        let mut input = HostInput::default();
        input.mouse_motion([4., 0.]);
        let first = input.sample(&Settings::default());
        let second = input.sample(&Settings::default());
        assert!(first[0].axes[Axis::RightX as usize] > 128);
        assert_eq!(second[0].axes[Axis::RightX as usize], 128);
    }
}
