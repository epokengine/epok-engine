//! Numeric defaults are edited on the canvas without changing connected inputs.
use super::*;

#[derive(Clone)]
pub(super) struct Field {
    pub socket: Socket,
    pub axis: Option<usize>,
    pub value: Value,
    pub min: [f32; 2],
    pub max: [f32; 2],
}
pub(super) struct Edit {
    field: Field,
    text: String,
    focus: bool,
}
pub(super) fn parts(node: &Node, socket: &Socket) -> Vec<(Option<usize>, Value)> {
    let SocketType::Value(ty) = &socket.ty else {
        return vec![];
    };
    let value = match node.inputs.get(&socket.pin) {
        Some(Input::Literal { value, value_type }) if value_type == ty => value.clone(),
        None => default_value(ty),
        _ => return vec![],
    };
    if !crate::script_values::valid(&value, ty) {
        return vec![];
    }
    match ty {
        schema::Type::Fixed | schema::Type::Int32 | schema::Type::UInt32 => vec![(None, value)],
        schema::Type::Vector {
            length: length @ 2..=3,
        } => (0..*length).map(|i| (Some(i), value[i].clone())).collect(),
        _ => vec![],
    }
}
pub(super) fn width(parts: &[(Option<usize>, Value)]) -> Option<f32> {
    (!parts.is_empty()).then(|| {
        parts
            .iter()
            .map(|(axis, _)| 39. + if axis.is_some() { 11. } else { 0. })
            .sum::<f32>()
            - 4.
    })
}
impl BlueprintEditor {
    pub(super) fn begin_inline_edit(&mut self, field: &Field) {
        self.select_node_graph(&field.socket.node);
        self.selected = BTreeSet::from([field.socket.node.clone()]);
        self.details = Details::Node;
        self.wire = None;
        self.drag = None;
        self.drag_before = None;
        self.inline_edit = Some(Edit {
            field: field.clone(),
            text: field.value.to_string(),
            focus: true,
        });
    }
    pub(super) fn commit_inline_value(&mut self, field: &Field, text: &str) -> Result<(), String> {
        let SocketType::Value(ty) = &field.socket.ty else {
            return Err("Not a value pin".into());
        };
        let scalar = if field.axis.is_some() {
            &schema::Type::Fixed
        } else {
            ty
        };
        let number = match scalar {
            schema::Type::Fixed => json!(
                text.trim()
                    .parse::<f64>()
                    .map_err(|_| "Enter a valid number")?
            ),
            schema::Type::Int32 => json!(
                text.trim()
                    .parse::<i32>()
                    .map_err(|_| "Enter a signed 32-bit integer")?
            ),
            schema::Type::UInt32 => json!(
                text.trim()
                    .parse::<u32>()
                    .map_err(|_| "Enter an unsigned 32-bit integer")?
            ),
            _ => return Err("Not a numeric pin".into()),
        };
        if !crate::script_values::valid(&number, scalar) {
            return Err("Number is outside this pin's range or has the wrong type".into());
        }
        let node = self
            .asset
            .as_mut()
            .ok_or("No Blueprint")?
            .functions
            .iter_mut()
            .flat_map(|g| &mut g.nodes)
            .find(|n| n.id == field.socket.node)
            .ok_or("Missing node")?;
        let mut value = match node.inputs.get(&field.socket.pin) {
            Some(Input::Literal { value, value_type }) if value_type == ty => value.clone(),
            None => default_value(ty),
            _ => return Err("Pin changed or is connected".into()),
        };
        if let Some(axis) = field.axis {
            let items = value.as_array_mut().ok_or("Not a vector")?;
            *items.get_mut(axis).ok_or("Missing axis")? = number;
        } else {
            value = number;
        }
        if !crate::script_values::valid(&value, ty) {
            return Err("Invalid pin default".into());
        }
        node.inputs.insert(
            field.socket.pin.clone(),
            Input::Literal {
                value_type: ty.clone(),
                value,
            },
        );
        Ok(())
    }
    pub(super) fn draw_inline_edit(&mut self, ui: &Ui, fields: &[Field]) {
        let Some(mut edit) = self.inline_edit.take() else {
            return;
        };
        let Some(field) = fields.iter().find(|f| {
            f.socket.node == edit.field.socket.node
                && f.socket.pin == edit.field.socket.pin
                && f.axis == edit.field.axis
        }) else {
            return;
        };
        edit.field = field.clone();
        let saved_cursor = ui.cursor_screen_pos();
        ui.set_cursor_screen_pos(field.min);
        ui.set_next_item_width(field.max[0] - field.min[0]);
        let opening = edit.focus;
        if edit.focus {
            ui.set_keyboard_focus_here();
            edit.focus = false;
        }
        let padding = ui.push_style_var(imgui::StyleVar::FramePadding([2., 0.]));
        let enter = ui
            .input_text("##bp-inline-number", &mut edit.text)
            .auto_select_all(true)
            .enter_returns_true(true)
            .build();
        record_control(ui, "bp-inline-number");
        let escape = ui.is_key_pressed(imgui::Key::Escape);
        let done = enter || (!opening && ui.is_item_deactivated());
        drop(padding);
        ui.set_cursor_screen_pos(saved_cursor);
        if escape {
            return;
        }
        if done {
            if let Err(error) = self.commit_inline_value(&edit.field, &edit.text) {
                self.error = Some(error);
                self.inline_edit = Some(edit);
            }
        } else {
            self.inline_edit = Some(edit);
        }
    }
}
